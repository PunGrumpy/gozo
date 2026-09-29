//! `gozo dev`: build the main package, run it with env files loaded, and
//! rebuild + restart when Go sources change.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use anyhow::{Context as _, anyhow};
use globset::{Glob, GlobSet, GlobSetBuilder};
use gozo_core::{EnvStore, Environment, MainPackage, dotenv, link, packages};
use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};

use crate::ctx::Ctx;
use crate::ui::Timer;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Main package to run, e.g. ./cmd/api. Overrides dev.cmd in gozo.toml.
    #[arg(long, value_name = "PKG")]
    pub cmd: Option<String>,

    /// Port to expose as PORT. Overrides dev.port in gozo.toml.
    #[arg(short = 'l', long = "listen", value_name = "PORT")]
    pub listen: Option<u16>,

    /// Run once and exit when the program exits; do not watch for changes.
    #[arg(long)]
    pub no_watch: bool,

    /// Env file(s) to load instead of the defaults. Repeatable; later files win.
    #[arg(long = "env-file", value_name = "FILE")]
    pub env_file: Vec<PathBuf>,

    /// Arguments passed to the program, after `--`.
    #[arg(value_name = "ARGS", last = true)]
    pub args: Vec<String>,
}

/// SIGINT/SIGTERM set a flag that the run loops poll, so the child is always
/// stopped through [`ChildGuard`] instead of being orphaned.
#[cfg(unix)]
mod signals {
    use std::sync::atomic::{AtomicI32, Ordering};

    static RECEIVED: AtomicI32 = AtomicI32::new(0);

    unsafe extern "C" {
        fn signal(signum: i32, handler: extern "C" fn(i32)) -> usize;
    }

    extern "C" fn on_signal(sig: i32) {
        RECEIVED.store(sig, Ordering::SeqCst);
    }

    pub fn install() {
        // SAFETY: installing an async-signal-safe handler that only stores an int.
        unsafe {
            signal(2, on_signal); // SIGINT
            signal(15, on_signal); // SIGTERM
        }
    }

    /// The signal received so far, if any.
    pub fn received() -> Option<i32> {
        match RECEIVED.load(Ordering::SeqCst) {
            0 => None,
            s => Some(s),
        }
    }
}

#[cfg(not(unix))]
mod signals {
    pub fn install() {}
    pub fn received() -> Option<i32> {
        None
    }
}

const DEFAULT_ENV_FILES: [&str; 4] = [
    ".env",
    ".env.local",
    ".env.development",
    ".env.development.local",
];
const DEFAULT_WATCH: [&str; 3] = ["**/*.go", "go.mod", "go.sum"];
const DEFAULT_IGNORE: [&str; 4] = [".git/**", ".gozo/**", "dist/**", "**/*_test.go"];
const COMPOSE_FILES: [&str; 4] = [
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yaml",
    "compose.yml",
];

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    if ctx.json {
        ctx.out.error(&anyhow!(
            "`gozo dev` is interactive and has no JSON output; run it without --json"
        ));
        return Ok(ExitCode::from(2));
    }
    let project = ctx.project()?.clone();
    let root = project.root.clone();
    ctx.ui.header();

    // Which main package.
    let pkgs = packages::main_packages(&ctx.go, &project)?;
    if pkgs.is_empty() {
        anyhow::bail!(
            "no main packages found\n  create one under cmd/, e.g. cmd/api/main.go, or run `gozo init`"
        );
    }
    let configured = args.cmd.clone().or_else(|| ctx.config.dev.cmd.clone());
    let pkg = match packages::pick_dev_package(&pkgs, configured.as_deref()) {
        Some(p) => p.clone(),
        None if configured.is_some() => anyhow::bail!(
            "main package {:?} not found; available:\n{}",
            configured.unwrap_or_default(),
            list_pkgs(&pkgs, &root)
        ),
        None if ctx.ui.interactive && !ctx.ui.yes => {
            let items: Vec<String> = pkgs.iter().map(|p| display_rel(p, &root)).collect();
            let i = ctx
                .ui
                .select("Which main package do you want to run?", &items, 0)?;
            pkgs[i].clone()
        }
        None => anyhow::bail!(
            "several main packages found; pick one with --cmd or dev.cmd in gozo.toml:\n{}",
            list_pkgs(&pkgs, &root)
        ),
    };
    ctx.ui
        .step(format!("detected {}", display_rel(&pkg, &root)));

    // Environment: store (lowest) < env files in order < PORT.
    let store_vars = EnvStore::new(&root).list(Environment::Development)?;
    let file_names: Vec<String> = if !args.env_file.is_empty() {
        args.env_file
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    } else if !ctx.config.dev.env.is_empty() {
        ctx.config.dev.env.clone()
    } else {
        DEFAULT_ENV_FILES.iter().map(|s| (*s).to_owned()).collect()
    };
    let mut layers: Vec<Vec<(String, String)>> = vec![store_vars];
    let mut loaded_names = Vec::new();
    for name in &file_names {
        let path = if Path::new(name).is_absolute() {
            PathBuf::from(name)
        } else {
            root.join(name)
        };
        if path.is_file() {
            layers.push(
                dotenv::load(&path)
                    .with_context(|| format!("could not read {}", path.display()))?,
            );
            loaded_names.push(name.clone());
        } else if !args.env_file.is_empty() {
            ctx.ui.warn(format!("{name} not found"));
        }
    }
    let mut env = layer_env(layers);
    let port = resolve_port(args.listen, ctx.config.dev.port, &env);
    env.insert("PORT".to_owned(), port.to_string());
    if loaded_names.is_empty() {
        ctx.ui
            .step(format!("no env files found ({} vars)", env.len()));
    } else {
        ctx.ui.step(format!(
            "loaded {} ({} vars)",
            loaded_names.join(", "),
            env.len()
        ));
    }

    // Services via docker compose.
    if let Some(compose) = compose_file(&root, ctx.config.dev.services.as_deref()) {
        start_services(ctx, &root, &compose);
    }

    // Build output lives under .gozo/dev; keep it out of git.
    let bin_dir = root.join(gozo_core::STATE_DIR).join("dev");
    std::fs::create_dir_all(&bin_dir)?;
    let _ = link::ensure_gitignore(&root);
    let bin = bin_dir.join(exe_name(&pkg.name));

    let mut program_args = ctx.config.dev.args.clone();
    program_args.extend(args.args.iter().cloned());

    let mut runner = Runner {
        ctx,
        pkg: &pkg,
        root: &root,
        bin: &bin,
        env: &env,
        program_args: &program_args,
        port,
        child: None,
        last_build: SystemTime::now(),
    };

    signals::install();
    let started = runner.build_and_start()?;
    if args.no_watch {
        if !started {
            return Ok(ExitCode::from(1));
        }
        return Ok(runner.wait());
    }
    if !started {
        runner
            .ctx
            .ui
            .warn("waiting for changes before trying again");
    }
    runner.watch(&project)
}

struct Runner<'a> {
    ctx: &'a mut Ctx,
    pkg: &'a MainPackage,
    root: &'a Path,
    bin: &'a Path,
    env: &'a BTreeMap<String, String>,
    program_args: &'a [String],
    port: u16,
    child: Option<ChildGuard>,
    /// When the last `go build` started; older mtimes are not real changes.
    last_build: SystemTime,
}

impl Runner<'_> {
    /// `go build` then (re)start. Returns whether a new process is running.
    /// A failed build keeps the previous process alive.
    fn build_and_start(&mut self) -> anyhow::Result<bool> {
        let timer = Timer::start();
        self.last_build = SystemTime::now();
        let rel = self.pkg.rel.clone();
        let bin = self.bin.display().to_string();
        self.ctx.ui.debug(format!(
            "go build -o {bin} {rel} (in {})",
            self.pkg.module_dir.display()
        ));
        let out = self.ctx.go.run(
            &self.pkg.module_dir,
            ["build", "-o", bin.as_str(), rel.as_str()],
        )?;
        if !out.success() {
            eprintln!("{}", out.stderr.trim_end());
            if self.child.is_some() {
                self.ctx.ui.warn(format!(
                    "build failed, keeping previous version {}",
                    timer.elapsed()
                ));
            } else {
                self.ctx
                    .ui
                    .warn(format!("build failed {}", timer.elapsed()));
            }
            return Ok(false);
        }

        if let Some(mut old) = self.child.take() {
            old.stop();
        }
        let child = Command::new(self.bin)
            .args(self.program_args)
            .envs(self.env)
            .current_dir(self.root)
            .stdin(Stdio::null())
            .spawn()
            .with_context(|| format!("could not start {}", self.bin.display()))?;
        self.child = Some(ChildGuard(Some(child)));
        self.ctx.ui.step(format!(
            "{} {}  {}",
            self.ctx.ui.bold("Ready!"),
            self.ctx.ui.cyan(&format!("http://localhost:{}", self.port)),
            self.ctx.ui.dim(&timer.elapsed())
        ));
        Ok(true)
    }

    /// `--no-watch`: block until the program exits (or we are told to stop)
    /// and propagate its status.
    fn wait(&mut self) -> ExitCode {
        loop {
            if let Some(code) = self.interrupted() {
                return code;
            }
            let status = match self.child.as_mut().and_then(|g| g.0.as_mut()) {
                Some(c) => c.try_wait(),
                None => return ExitCode::from(1),
            };
            match status {
                Ok(Some(status)) => {
                    self.child = None;
                    return super::env::exit_code(status);
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => return ExitCode::from(1),
            }
        }
    }

    /// When a SIGINT/SIGTERM arrived: stop the child and pick the exit code.
    fn interrupted(&mut self) -> Option<ExitCode> {
        let sig = signals::received()?;
        if let Some(mut c) = self.child.take() {
            c.stop();
        }
        self.ctx.ui.blank();
        Some(ExitCode::from((128 + sig).clamp(0, 255) as u8))
    }

    /// Poll the child and report if it died on its own.
    fn poll_child(&mut self) {
        let exited = match self.child.as_mut().and_then(|g| g.0.as_mut()) {
            Some(c) => match c.try_wait() {
                Ok(Some(status)) => Some(status),
                _ => None,
            },
            None => None,
        };
        if let Some(status) = exited {
            self.child = None;
            let code = status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".to_owned());
            self.ctx.ui.warn(format!(
                "{} exited with code {code}; waiting for changes",
                self.pkg.name
            ));
        }
    }

    fn watch(&mut self, project: &gozo_go::Project) -> anyhow::Result<ExitCode> {
        let watch_patterns: Vec<String> = if self.ctx.config.dev.watch.is_empty() {
            DEFAULT_WATCH.iter().map(|s| (*s).to_owned()).collect()
        } else {
            self.ctx.config.dev.watch.clone()
        };
        let watch_set = glob_set(&watch_patterns)?;
        let mut ignore_patterns: Vec<String> =
            DEFAULT_IGNORE.iter().map(|s| (*s).to_owned()).collect();
        ignore_patterns.extend(self.ctx.config.dev.ignore.iter().cloned());
        let ignore_set = glob_set(&ignore_patterns)?;

        // Watch the project root plus any module that lives outside it.
        let mut bases: Vec<PathBuf> = vec![self.root.to_path_buf()];
        for m in &project.modules {
            if !m.dir.starts_with(self.root) && !bases.contains(&m.dir) {
                bases.push(m.dir.clone());
            }
        }
        let (tx, rx) = mpsc::channel::<DebounceEventResult>();
        let mut debouncer = new_debouncer(Duration::from_millis(300), tx)
            .context("could not start file watcher")?;
        for b in &bases {
            debouncer
                .watcher()
                .watch(b, RecursiveMode::Recursive)
                .with_context(|| format!("could not watch {}", b.display()))?;
        }
        self.ctx.ui.detail("watching for changes (Ctrl-C to stop)");

        loop {
            if let Some(code) = self.interrupted() {
                return Ok(code);
            }
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(Ok(events)) => {
                    let mut changed: Vec<String> = Vec::new();
                    for ev in events {
                        self.ctx
                            .ui
                            .debug(format!("fs event {:?} {}", ev.kind, ev.path.display()));
                        let Some(rel) = relative_to(&bases, &ev.path) else {
                            continue;
                        };
                        // notify also reports files that were merely opened
                        // (go build reads go.mod, .git/HEAD, ...); only a
                        // newer mtime, or a deletion, is a real change.
                        if watch_set.is_match(&rel)
                            && !ignore_set.is_match(&rel)
                            && modified_since(&ev.path, self.last_build)
                            && !changed.contains(&rel)
                        {
                            changed.push(rel);
                        }
                    }
                    if changed.is_empty() {
                        continue;
                    }
                    // Drain anything that piled up during the debounce window.
                    while rx.try_recv().is_ok() {}
                    let more = changed.len().saturating_sub(1);
                    let first = changed.first().cloned().unwrap_or_default();
                    self.ctx.ui.step(if more == 0 {
                        format!("Change detected: {first}")
                    } else {
                        format!("Change detected: {first} (+{more} more)")
                    });
                    self.build_and_start()?;
                }
                Ok(Err(e)) => self.ctx.ui.warn(format!("watcher error: {e}")),
                Err(mpsc::RecvTimeoutError::Timeout) => self.poll_child(),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    anyhow::bail!("file watcher stopped unexpectedly")
                }
            }
        }
    }
}

/// A child process that is killed when dropped, so it never outlives gozo.
struct ChildGuard(Option<Child>);

impl ChildGuard {
    fn stop(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Merge layers in order; later layers override earlier keys.
pub fn layer_env(layers: Vec<Vec<(String, String)>>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for layer in layers {
        for (k, v) in layer {
            out.insert(k, v);
        }
    }
    out
}

/// `--listen` beats `dev.port` beats a `PORT` from the env files beats 8080.
pub fn resolve_port(
    listen: Option<u16>,
    configured: Option<u16>,
    env: &BTreeMap<String, String>,
) -> u16 {
    listen
        .or(configured)
        .or_else(|| env.get("PORT").and_then(|p| p.trim().parse().ok()))
        .unwrap_or(8080)
}

fn glob_set(patterns: &[String]) -> anyhow::Result<GlobSet> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        b.add(Glob::new(p).with_context(|| format!("invalid watch pattern {p:?}"))?);
    }
    Ok(b.build()?)
}

/// Whether `path` was written after `since`. Missing files count as changed
/// (a deleted source must trigger a rebuild); directories never do.
fn modified_since(path: &Path, since: SystemTime) -> bool {
    match std::fs::metadata(path) {
        Ok(m) if m.is_dir() => false,
        Ok(m) => m.modified().map(|t| t > since).unwrap_or(true),
        Err(_) => true,
    }
}

/// Path relative to the first watched base that contains it, with `/` separators.
fn relative_to(bases: &[PathBuf], path: &Path) -> Option<String> {
    bases
        .iter()
        .find_map(|b| path.strip_prefix(b).ok())
        .map(|r| r.to_string_lossy().replace('\\', "/"))
}

fn display_rel(p: &MainPackage, root: &Path) -> String {
    match p.module_dir.strip_prefix(root) {
        Ok(m) if !m.as_os_str().is_empty() => {
            format!(
                "./{}/{}",
                m.to_string_lossy().replace('\\', "/"),
                p.rel.trim_start_matches("./")
            )
        }
        _ => p.rel.clone(),
    }
}

fn list_pkgs(pkgs: &[MainPackage], root: &Path) -> String {
    pkgs.iter()
        .map(|p| format!("  {}", display_rel(p, root)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compose_file(root: &Path, configured: Option<&str>) -> Option<PathBuf> {
    if let Some(c) = configured {
        return Some(root.join(c));
    }
    COMPOSE_FILES
        .iter()
        .map(|f| root.join(f))
        .find(|p| p.is_file())
}

fn docker_on_path() -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(exe_name("docker")).is_file()))
        .unwrap_or(false)
}

/// `docker compose up -d`; problems are warnings, never fatal.
fn start_services(ctx: &mut Ctx, root: &Path, compose: &Path) {
    if !compose.is_file() {
        ctx.ui
            .warn(format!("compose file {} not found", compose.display()));
        return;
    }
    if !docker_on_path() {
        ctx.ui.warn(format!(
            "docker not found on PATH; not starting {}",
            compose.display()
        ));
        return;
    }
    let file = compose.display().to_string();
    ctx.ui.debug(format!("docker compose -f {file} up -d"));
    let up = Command::new("docker")
        .args(["compose", "-f", &file, "up", "-d"])
        .current_dir(root)
        .stdin(Stdio::null())
        .output();
    match up {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            ctx.ui.warn(format!(
                "docker compose up failed: {}",
                last_line(&o.stderr)
                    .or_else(|| last_line(&o.stdout))
                    .unwrap_or_else(|| { format!("exit code {}", o.status.code().unwrap_or(-1)) })
            ));
            return;
        }
        Err(e) => {
            ctx.ui.warn(format!("could not run docker compose: {e}"));
            return;
        }
    }
    let ps = Command::new("docker")
        .args(["compose", "-f", &file, "ps", "--format", "json"])
        .current_dir(root)
        .stdin(Stdio::null())
        .output();
    let names = ps
        .ok()
        .filter(|o| o.status.success())
        .map(|o| compose_service_names(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    if names.is_empty() {
        ctx.ui
            .step(format!("started services from {}", compose.display()));
    } else {
        ctx.ui.step(format!("started {}", names.join(", ")));
    }
}

/// Last non-empty line of a command's output, for one-line error summaries.
fn last_line(bytes: &[u8]) -> Option<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_owned)
}

/// `docker compose ps --format json` prints one object per line (newer
/// versions) or a single array (older ones); accept both.
fn compose_service_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut push = |v: &serde_json::Value| {
        if let Some(n) = v
            .get("Service")
            .or_else(|| v.get("Name"))
            .and_then(|n| n.as_str())
        {
            if !names.contains(&n.to_owned()) {
                names.push(n.to_owned());
            }
        }
    };
    if let Ok(serde_json::Value::Array(items)) =
        serde_json::from_str::<serde_json::Value>(text.trim())
    {
        items.iter().for_each(&mut push);
        return names;
    }
    for line in text.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            push(&v);
        }
    }
    names
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kv(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn later_layers_win() {
        let env = layer_env(vec![
            kv(&[("A", "store"), ("B", "store"), ("PORT", "9000")]),
            kv(&[("A", "dotenv")]),
            kv(&[("A", "local"), ("C", "local")]),
        ]);
        assert_eq!(env["A"], "local");
        assert_eq!(env["B"], "store");
        assert_eq!(env["C"], "local");
        assert_eq!(env.len(), 4);
    }

    #[test]
    fn port_precedence() {
        let env = layer_env(vec![kv(&[("PORT", "9000")])]);
        assert_eq!(resolve_port(Some(3000), Some(4000), &env), 3000);
        assert_eq!(resolve_port(None, Some(4000), &env), 4000);
        assert_eq!(resolve_port(None, None, &env), 9000);
        assert_eq!(resolve_port(None, None, &BTreeMap::new()), 8080);
    }

    #[test]
    fn watch_globs_match_relative_paths() {
        let watch = glob_set(&DEFAULT_WATCH.map(str::to_owned)).unwrap();
        let ignore = glob_set(&DEFAULT_IGNORE.map(str::to_owned)).unwrap();
        assert!(watch.is_match("main.go"));
        assert!(watch.is_match("internal/x/y.go"));
        assert!(watch.is_match("go.mod"));
        assert!(!watch.is_match("README.md"));
        assert!(ignore.is_match("internal/x_test.go"));
        assert!(ignore.is_match(".gozo/dev/api"));
        assert!(ignore.is_match(".git/index"));
        assert!(!ignore.is_match("cmd/api/main.go"));
    }

    #[test]
    fn parses_compose_ps_json() {
        assert_eq!(
            compose_service_names(
                "{\"Name\":\"app-db-1\",\"Service\":\"db\"}\n{\"Name\":\"app-redis-1\",\"Service\":\"redis\"}\n"
            ),
            vec!["db", "redis"]
        );
        assert_eq!(compose_service_names("[{\"Service\":\"pg\"}]"), vec!["pg"]);
        assert!(compose_service_names("").is_empty());
    }
}
