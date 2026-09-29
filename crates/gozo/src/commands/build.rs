//! `gozo build` — build every main package (or a selection) for one or more
//! GOOS/GOARCH targets, with `main.version`, `main.commit` and `main.date`
//! injected via `-ldflags -X`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::style::{Mark, dim, mark};
use anyhow::Context as _;
use gozo_core::MainPackage;
use serde::Serialize;

use crate::ctx::Ctx;
use crate::ui::Timer;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Main packages to build: a name (`api`), a path (`./cmd/api`) or an
    /// import path. Default: gozo.toml `build.cmds`, else every main package.
    #[arg(value_name = "CMDS")]
    pub cmds: Vec<String>,

    /// Output directory (default: gozo.toml `build.output`, else `dist`).
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Target operating system (GOOS).
    #[arg(long = "os", value_name = "GOOS")]
    pub goos: Option<String>,

    /// Target architecture (GOARCH).
    #[arg(long = "arch", value_name = "GOARCH")]
    pub goarch: Option<String>,

    /// Cross-compile target as GOOS/GOARCH; repeat for several.
    #[arg(long, value_name = "GOOS/GOARCH")]
    pub target: Vec<String>,

    /// Version string for `main.version` (default: `git describe`, else `dev`).
    #[arg(long, value_name = "V")]
    pub version: Option<String>,

    /// Extra `-ldflags`, appended after the ones gozo injects.
    #[arg(long, value_name = "FLAGS")]
    pub ldflags: Option<String>,

    /// Do not pass `-trimpath`.
    #[arg(long)]
    pub no_trimpath: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub goos: String,
    pub goarch: String,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.goos, self.goarch)
    }
}

#[derive(Debug, Serialize)]
struct Doc {
    schema: &'static str,
    project: String,
    version: String,
    commit: String,
    artifacts: Vec<Artifact>,
    failures: Vec<Failure>,
}

#[derive(Debug, Serialize)]
struct Artifact {
    name: String,
    package: String,
    goos: String,
    goarch: String,
    path: String,
    size_bytes: u64,
    elapsed_ms: u128,
}

#[derive(Debug, Serialize)]
struct Failure {
    package: String,
    goos: String,
    goarch: String,
    error: String,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let project = ctx.project()?;
    let root = project.root.clone();
    let build_cfg = ctx.config.build.clone();

    let all = gozo_core::packages::main_packages(&ctx.go, project)?;
    if all.is_empty() {
        anyhow::bail!(
            "no main packages found in {}\n  create one under cmd/<name>/main.go, or set build.cmds in gozo.toml",
            root.display()
        );
    }
    let selectors: &[String] = if args.cmds.is_empty() {
        &build_cfg.cmds
    } else {
        &args.cmds
    };
    let selected = select_packages(&all, selectors)?;

    let host = host_target(ctx, &root)?;
    let targets = resolve_targets(
        &args.target,
        args.goos.as_deref(),
        args.goarch.as_deref(),
        &build_cfg.targets,
        &host,
    )?;

    let out_dir = match &args.output {
        Some(p) if p.is_absolute() => p.clone(),
        Some(p) => ctx.cwd.join(p),
        None => root.join(&build_cfg.output),
    };
    std::fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;

    let version = args
        .version
        .clone()
        .or_else(|| gozo_core::git::describe(&root))
        .unwrap_or_else(|| "dev".to_owned());
    let commit = gozo_core::git::short_sha(&root).unwrap_or_else(|| "none".to_owned());
    let date = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let user_ldflags = args.ldflags.clone().or_else(|| build_cfg.ldflags.clone());
    let ldflags = ldflags(&version, &commit, &date, user_ldflags.as_deref());
    let trimpath = build_cfg.trimpath && !args.no_trimpath;
    let cgo = build_cfg.cgo.map(|on| if on { "1" } else { "0" });
    let multi = targets.len() > 1;

    ctx.ui.header();
    ctx.ui.kv("version", &version);
    ctx.ui.kv("commit", &commit);
    let total = Timer::start();
    let mut artifacts = Vec::new();
    let mut failures = Vec::new();

    for pkg in &selected {
        for target in &targets {
            let name = binary_name(&pkg.name, target, &host, multi);
            let out_path = out_dir.join(&name);
            ctx.ui.step(format!("Building {} ({target})", pkg.name));

            let mut go_args: Vec<String> = vec![
                "build".to_owned(),
                "-o".to_owned(),
                out_path.display().to_string(),
            ];
            if trimpath {
                go_args.push("-trimpath".to_owned());
            }
            go_args.push("-ldflags".to_owned());
            go_args.push(ldflags.clone());
            go_args.push(pkg.rel.clone());

            let mut env: Vec<(&str, &str)> =
                vec![("GOOS", &target.goos), ("GOARCH", &target.goarch)];
            if let Some(c) = cgo {
                env.push(("CGO_ENABLED", c));
            }
            ctx.ui.debug(format!(
                "GOOS={} GOARCH={} go {}  (in {})",
                target.goos,
                target.goarch,
                shell_words::join(&go_args),
                display_rel(&root, &pkg.module_dir)
            ));

            let timer = Timer::start();
            let started = std::time::Instant::now();
            let out = ctx.go.run_env(&pkg.module_dir, &go_args, &env)?;
            if out.success() {
                let size = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
                let shown = display_path(&ctx.cwd, &root, &out_path);
                if !ctx.json {
                    println!(
                        "  {} {}  {}  {}",
                        mark(ctx.ui.color, Mark::Ok),
                        shown,
                        dim(&human_size(size)),
                        dim(&timer.elapsed())
                    );
                }
                artifacts.push(Artifact {
                    name,
                    package: pkg.import_path.clone(),
                    goos: target.goos.clone(),
                    goarch: target.goarch.clone(),
                    path: shown,
                    size_bytes: size,
                    elapsed_ms: started.elapsed().as_millis(),
                });
            } else {
                let error = out.stderr.trim().to_owned();
                if !ctx.json {
                    println!(
                        "  {} {} ({target})  {}",
                        mark(ctx.ui.color, Mark::Fail),
                        pkg.name,
                        dim(&timer.elapsed())
                    );
                    for line in error.lines() {
                        println!("      {line}");
                    }
                }
                failures.push(Failure {
                    package: pkg.import_path.clone(),
                    goos: target.goos.clone(),
                    goarch: target.goarch.clone(),
                    error,
                });
            }
        }
    }

    let n = artifacts.len();
    if failures.is_empty() {
        ctx.ui.success(format!(
            "Built {n} binar{} in {}/ {}",
            if n == 1 { "y" } else { "ies" },
            display_path(&ctx.cwd, &root, &out_dir),
            ctx.ui.dim(&total.elapsed())
        ));
    } else {
        let f = failures.len();
        ctx.ui.error(format!(
            "{f} build{} failed, {n} succeeded {}",
            if f == 1 { "" } else { "s" },
            ctx.ui.dim(&total.elapsed())
        ));
    }

    let failed = !failures.is_empty();
    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.build/v1",
            project: ctx.project_name(),
            version,
            commit,
            artifacts,
            failures,
        })?;
    }
    Ok(if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Pick the main packages matching `selectors` (name, `./cmd/x` path or import
/// path). No selectors means every main package.
fn select_packages<'a>(
    all: &'a [MainPackage],
    selectors: &[String],
) -> anyhow::Result<Vec<&'a MainPackage>> {
    if selectors.is_empty() {
        return Ok(all.iter().collect());
    }
    let mut out: Vec<&MainPackage> = Vec::new();
    for sel in selectors {
        let want = sel.trim_start_matches("./").trim_end_matches('/');
        let found: Vec<&MainPackage> = all
            .iter()
            .filter(|p| {
                p.name == want || p.rel.trim_start_matches("./") == want || p.import_path == *sel
            })
            .collect();
        if found.is_empty() {
            let available: Vec<String> = all
                .iter()
                .map(|p| format!("{} ({})", p.name, p.rel))
                .collect();
            anyhow::bail!(
                "no main package matches `{sel}`\n  available: {}",
                available.join(", ")
            );
        }
        for p in found {
            if !out.iter().any(|q| q.import_path == p.import_path) {
                out.push(p);
            }
        }
    }
    Ok(out)
}

/// `GOOS/GOARCH` from flags, else gozo.toml `build.targets`, else the host.
fn resolve_targets(
    flags: &[String],
    goos: Option<&str>,
    goarch: Option<&str>,
    configured: &[String],
    host: &Target,
) -> anyhow::Result<Vec<Target>> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<Target>, t: Target| {
        if !out.contains(&t) {
            out.push(t);
        }
    };
    if !flags.is_empty() {
        for f in flags {
            push(&mut out, parse_target(f)?);
        }
    } else if goos.is_some() || goarch.is_some() {
        push(
            &mut out,
            Target {
                goos: goos.unwrap_or(&host.goos).to_owned(),
                goarch: goarch.unwrap_or(&host.goarch).to_owned(),
            },
        );
    } else if !configured.is_empty() {
        for c in configured {
            push(
                &mut out,
                parse_target(c).context("in gozo.toml build.targets")?,
            );
        }
    } else {
        push(&mut out, host.clone());
    }
    Ok(out)
}

pub fn parse_target(s: &str) -> anyhow::Result<Target> {
    let s = s.trim();
    let Some((goos, goarch)) = s.split_once('/') else {
        anyhow::bail!("invalid target `{s}`: expected GOOS/GOARCH, e.g. linux/amd64");
    };
    let (goos, goarch) = (goos.trim(), goarch.trim());
    if goos.is_empty() || goarch.is_empty() || goarch.contains('/') {
        anyhow::bail!("invalid target `{s}`: expected GOOS/GOARCH, e.g. linux/amd64");
    }
    Ok(Target {
        goos: goos.to_owned(),
        goarch: goarch.to_owned(),
    })
}

fn host_target(ctx: &Ctx, root: &Path) -> anyhow::Result<Target> {
    let out = ctx.go.run_local(root, ["env", "GOOS", "GOARCH"])?;
    let mut lines = out.stdout.lines().map(str::trim);
    match (lines.next(), lines.next()) {
        (Some(goos), Some(goarch)) if !goos.is_empty() && !goarch.is_empty() => Ok(Target {
            goos: goos.to_owned(),
            goarch: goarch.to_owned(),
        }),
        _ => anyhow::bail!(
            "could not read GOOS/GOARCH from `go env`: {}",
            out.stderr.trim()
        ),
    }
}

/// `api`, `api_linux_arm64`, `api_windows_amd64.exe`. The suffix appears when
/// building for several targets or for something other than the host.
pub fn binary_name(name: &str, target: &Target, host: &Target, multi: bool) -> String {
    let mut out = name.to_owned();
    if multi || target != host {
        out.push_str(&format!("_{}_{}", target.goos, target.goarch));
    }
    if target.goos == "windows" {
        out.push_str(".exe");
    }
    out
}

pub fn ldflags(version: &str, commit: &str, date: &str, user: Option<&str>) -> String {
    let mut s =
        format!("-s -w -X main.version={version} -X main.commit={commit} -X main.date={date}");
    if let Some(u) = user.map(str::trim).filter(|u| !u.is_empty()) {
        s.push(' ');
        s.push_str(u);
    }
    s
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1000.0 && i < UNITS.len() - 1 {
        v /= 1000.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

fn display_rel(root: &Path, path: &Path) -> String {
    let p = path.strip_prefix(root).unwrap_or(path);
    let s = p.to_string_lossy().replace('\\', "/");
    if s.is_empty() { ".".to_owned() } else { s }
}

/// Show a path relative to cwd when possible, else to the project root.
fn display_path(cwd: &Path, root: &Path, path: &Path) -> String {
    if let Ok(p) = path.strip_prefix(cwd) {
        return p.to_string_lossy().replace('\\', "/");
    }
    if let Ok(p) = path.strip_prefix(root) {
        return p.to_string_lossy().replace('\\', "/");
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(goos: &str, goarch: &str) -> Target {
        Target {
            goos: goos.to_owned(),
            goarch: goarch.to_owned(),
        }
    }

    #[test]
    fn parses_targets() {
        assert_eq!(parse_target("linux/amd64").unwrap(), t("linux", "amd64"));
        assert_eq!(
            parse_target(" darwin/arm64 ").unwrap(),
            t("darwin", "arm64")
        );
        assert!(parse_target("linux").is_err());
        assert!(parse_target("linux/").is_err());
        assert!(parse_target("/amd64").is_err());
        assert!(parse_target("a/b/c").is_err());
    }

    #[test]
    fn resolves_targets_in_precedence_order() {
        let host = t("linux", "amd64");
        let cfg = vec!["darwin/arm64".to_owned()];
        // flags win
        let got = resolve_targets(
            &["windows/amd64".to_owned(), "windows/amd64".to_owned()],
            None,
            None,
            &cfg,
            &host,
        )
        .unwrap();
        assert_eq!(got, vec![t("windows", "amd64")]);
        // --os/--arch fill from host
        let got = resolve_targets(&[], None, Some("arm64"), &cfg, &host).unwrap();
        assert_eq!(got, vec![t("linux", "arm64")]);
        // config next
        let got = resolve_targets(&[], None, None, &cfg, &host).unwrap();
        assert_eq!(got, vec![t("darwin", "arm64")]);
        // host last
        let got = resolve_targets(&[], None, None, &[], &host).unwrap();
        assert_eq!(got, vec![host.clone()]);
        assert!(resolve_targets(&[], None, None, &["bad".to_owned()], &host).is_err());
    }

    #[test]
    fn names_binaries() {
        let host = t("linux", "amd64");
        assert_eq!(binary_name("api", &host, &host, false), "api");
        assert_eq!(binary_name("api", &host, &host, true), "api_linux_amd64");
        assert_eq!(
            binary_name("api", &t("darwin", "arm64"), &host, false),
            "api_darwin_arm64"
        );
        assert_eq!(
            binary_name("api", &t("windows", "amd64"), &host, false),
            "api_windows_amd64.exe"
        );
        assert_eq!(
            binary_name("api", &t("windows", "amd64"), &t("windows", "amd64"), false),
            "api.exe"
        );
    }

    #[test]
    fn builds_ldflags() {
        assert_eq!(
            ldflags(
                "v1.2.3",
                "abc123",
                "2026-01-01T00:00:00Z",
                Some(" -X main.extra=1 ")
            ),
            "-s -w -X main.version=v1.2.3 -X main.commit=abc123 -X main.date=2026-01-01T00:00:00Z -X main.extra=1"
        );
        assert!(ldflags("dev", "none", "d", None).ends_with("main.date=d"));
    }

    #[test]
    fn selects_packages_by_name_path_or_import() {
        let mk = |name: &str, rel: &str| MainPackage {
            name: name.to_owned(),
            import_path: format!("example.com/m/{}", rel.trim_start_matches("./")),
            dir: PathBuf::from("/m").join(rel),
            rel: rel.to_owned(),
            module_dir: PathBuf::from("/m"),
        };
        let all = vec![mk("api", "./cmd/api"), mk("worker", "./cmd/worker")];
        let names = |v: Vec<&MainPackage>| v.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
        assert_eq!(
            names(select_packages(&all, &[]).unwrap()),
            vec!["api", "worker"]
        );
        assert_eq!(
            names(select_packages(&all, &["worker".to_owned()]).unwrap()),
            vec!["worker"]
        );
        assert_eq!(
            names(select_packages(&all, &["./cmd/api/".to_owned()]).unwrap()),
            vec!["api"]
        );
        assert_eq!(
            names(select_packages(&all, &["cmd/api".to_owned()]).unwrap()),
            vec!["api"]
        );
        assert_eq!(
            names(
                select_packages(
                    &all,
                    &["example.com/m/cmd/worker".to_owned(), "worker".to_owned()]
                )
                .unwrap()
            ),
            vec!["worker"]
        );
        let err = select_packages(&all, &["nope".to_owned()])
            .unwrap_err()
            .to_string();
        assert!(err.contains("api (./cmd/api)"), "{err}");
    }

    #[test]
    fn formats_sizes() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(4_200_000), "4.2 MB");
        assert_eq!(human_size(1_500), "1.5 kB");
    }
}
