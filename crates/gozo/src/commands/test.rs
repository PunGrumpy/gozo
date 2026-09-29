//! `gozo test` — `go test -json` across every module with a gotestsum-like
//! summary: one line per package as it finishes, failures replayed at the
//! end, and a single `N passed, N failed, N skipped in Ns` line.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};

use crate::style::{Mark, bold, dim, green, mark, red, yellow};
use anyhow::Context as _;
use gozo_go::{TestAction, TestEvent};
use serde::Serialize;

use crate::ctx::Ctx;
use crate::ui::Timer;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Packages to test, e.g. `./internal/...`. Default: `./...` in every module.
    #[arg(value_name = "PACKAGES")]
    pub packages: Vec<String>,

    /// Only run tests matching this regexp (`go test -run`).
    #[arg(long, value_name = "PATTERN")]
    pub run: Option<String>,

    /// Enable the race detector.
    #[arg(long)]
    pub race: bool,

    /// Enable coverage analysis.
    #[arg(long)]
    pub cover: bool,

    /// Tell long-running tests to shorten their run.
    #[arg(long)]
    pub short: bool,

    /// Print every test name, not just packages.
    #[arg(short, long)]
    pub verbose: bool,

    /// Stop after the first test failure.
    #[arg(long)]
    pub failfast: bool,

    /// Extra arguments passed to `go test` verbatim (after `--`).
    #[arg(last = true, value_name = "GO TEST ARGS")]
    pub extra: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Doc {
    schema: &'static str,
    project: String,
    packages: Vec<PkgResult>,
    failures: Vec<Failure>,
    summary: Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum PkgStatus {
    Running,
    Pass,
    Fail,
    Skip,
    /// `[no test files]`
    NoTests,
}

#[derive(Debug, Serialize)]
struct PkgResult {
    package: String,
    status: PkgStatus,
    /// Seconds.
    elapsed: f64,
    tests: Vec<TestResult>,
    #[serde(skip)]
    test_index: HashMap<String, usize>,
    /// Output not attributed to a test (build errors, panics, TestMain).
    #[serde(skip)]
    output: Vec<String>,
    #[serde(skip)]
    test_output: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
struct TestResult {
    name: String,
    status: &'static str,
    elapsed: f64,
}

#[derive(Debug, Serialize)]
struct Failure {
    package: String,
    test: Option<String>,
    output: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
struct Summary {
    passed: usize,
    failed: usize,
    skipped: usize,
    /// Wall-clock seconds.
    elapsed: f64,
}

/// Folds test2json events into per-package results.
#[derive(Default)]
struct Collector {
    packages: Vec<PkgResult>,
    index: HashMap<String, usize>,
    failures: Vec<Failure>,
    build_output: HashMap<String, Vec<String>>,
    passed: usize,
    failed: usize,
    skipped: usize,
}

impl Collector {
    fn pkg(&mut self, name: &str) -> &mut PkgResult {
        let idx = match self.index.get(name) {
            Some(&i) => i,
            None => {
                self.packages.push(PkgResult {
                    package: name.to_owned(),
                    status: PkgStatus::Running,
                    elapsed: 0.0,
                    tests: Vec::new(),
                    test_index: HashMap::new(),
                    output: Vec::new(),
                    test_output: HashMap::new(),
                });
                let i = self.packages.len() - 1;
                self.index.insert(name.to_owned(), i);
                i
            }
        };
        &mut self.packages[idx]
    }

    /// Apply one event. Returns the index of a package that just finished.
    fn handle(&mut self, ev: &TestEvent) -> Option<usize> {
        let action = ev.action();
        let pkg_name = ev.package_path().map(str::to_owned)?;
        match action {
            TestAction::BuildOutput => {
                if let Some(line) = ev.output_line() {
                    self.build_output
                        .entry(pkg_name)
                        .or_default()
                        .push(line.to_owned());
                }
                None
            }
            TestAction::BuildFail => None,
            TestAction::Start => {
                self.pkg(&pkg_name);
                None
            }
            TestAction::Output => {
                let line = ev.output_line().unwrap_or("").to_owned();
                let pkg = self.pkg(&pkg_name);
                match &ev.test {
                    Some(t) => pkg.test_output.entry(t.clone()).or_default().push(line),
                    None => pkg.output.push(line),
                }
                None
            }
            TestAction::Run => {
                if let Some(t) = &ev.test {
                    let pkg = self.pkg(&pkg_name);
                    if !pkg.test_index.contains_key(t) {
                        pkg.tests.push(TestResult {
                            name: t.clone(),
                            status: "run",
                            elapsed: 0.0,
                        });
                        pkg.test_index.insert(t.clone(), pkg.tests.len() - 1);
                    }
                }
                None
            }
            TestAction::Pass | TestAction::Fail | TestAction::Skip => {
                let elapsed = ev.elapsed.unwrap_or(0.0);
                match &ev.test {
                    Some(t) => {
                        let status = match action {
                            TestAction::Pass => "pass",
                            TestAction::Fail => "fail",
                            _ => "skip",
                        };
                        match action {
                            TestAction::Pass => self.passed += 1,
                            TestAction::Fail => self.failed += 1,
                            _ => self.skipped += 1,
                        }
                        let pkg = self.pkg(&pkg_name);
                        let idx = match pkg.test_index.get(t) {
                            Some(&i) => i,
                            None => {
                                pkg.tests.push(TestResult {
                                    name: t.clone(),
                                    status,
                                    elapsed,
                                });
                                pkg.tests.len() - 1
                            }
                        };
                        pkg.test_index.insert(t.clone(), idx);
                        pkg.tests[idx].status = status;
                        pkg.tests[idx].elapsed = elapsed;
                        if action == TestAction::Fail {
                            let output = pkg.test_output.remove(t).unwrap_or_default();
                            self.failures.push(Failure {
                                package: pkg_name.clone(),
                                test: Some(t.clone()),
                                output: tidy_output(output),
                            });
                        }
                        None
                    }
                    None => {
                        let build = self.build_output.remove(&pkg_name).unwrap_or_default();
                        let pkg = self.pkg(&pkg_name);
                        pkg.elapsed = elapsed;
                        let no_tests = pkg.output.iter().any(|l| l.contains("[no test files]"));
                        pkg.status = match action {
                            TestAction::Pass => PkgStatus::Pass,
                            TestAction::Fail => PkgStatus::Fail,
                            _ if no_tests => PkgStatus::NoTests,
                            _ => PkgStatus::Skip,
                        };
                        if pkg.status == PkgStatus::Fail {
                            let any_test_failed = pkg.tests.iter().any(|t| t.status == "fail");
                            if !any_test_failed || ev.failed_build.is_some() {
                                let mut output = build;
                                output.extend(pkg.output.iter().cloned());
                                let output = tidy_output(output);
                                if !output.is_empty() {
                                    self.failures.push(Failure {
                                        package: pkg_name.clone(),
                                        test: None,
                                        output,
                                    });
                                }
                            }
                        }
                        self.index.get(&pkg_name).copied()
                    }
                }
            }
            TestAction::Pause | TestAction::Cont | TestAction::Bench | TestAction::Other => None,
        }
    }
}

/// Drop the `=== RUN` style frames and trailing blank lines from captured output.
fn tidy_output(lines: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = lines
        .into_iter()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("=== RUN") || t.starts_with("=== PAUSE") || t.starts_with("=== CONT"))
        })
        .collect();
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    out
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let project = ctx.project()?;
    let root = project.root.clone();

    // Explicit patterns run once, relative to the module containing cwd;
    // the default `./...` runs in every module.
    let runs: Vec<(PathBuf, Vec<String>)> = if args.packages.is_empty() {
        project
            .modules
            .iter()
            .filter(|m| m.gomod.is_ok())
            .map(|m| (m.dir.clone(), vec!["./...".to_owned()]))
            .collect()
    } else {
        vec![(ctx.cwd.clone(), args.packages.clone())]
    };
    if runs.is_empty() {
        anyhow::bail!("no usable modules in {}", root.display());
    }

    let mut go_args: Vec<String> = vec!["test".to_owned(), "-json".to_owned()];
    if args.race {
        go_args.push("-race".to_owned());
    }
    if args.cover {
        go_args.push("-cover".to_owned());
    }
    if args.short {
        go_args.push("-short".to_owned());
    }
    if args.failfast {
        go_args.push("-failfast".to_owned());
    }
    if let Some(p) = &args.run {
        go_args.push("-run".to_owned());
        go_args.push(p.clone());
    }
    go_args.extend(args.extra.iter().cloned());

    ctx.ui.header();
    if runs.len() > 1 {
        ctx.ui.step(format!("Testing {} modules", runs.len()));
    }
    let timer = Timer::start();
    let started = std::time::Instant::now();
    let mut col = Collector::default();
    let mut go_failed = false;
    let mut stray: Vec<String> = Vec::new();
    let mut reported: Vec<usize> = Vec::new();

    if !ctx.json {
        println!();
    }
    for (dir, patterns) in &runs {
        let mut full = go_args.clone();
        full.extend(patterns.iter().cloned());
        ctx.ui.debug(format!(
            "go {}  (in {})",
            full.join(" "),
            display_rel(&root, dir)
        ));
        let mut child = ctx
            .go
            .command(dir, &full)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to run `go test` in {}", dir.display()))?;
        let stdout = child.stdout.take().context("no stdout from go test")?;
        let stderr = child.stderr.take().context("no stderr from go test")?;
        let stderr_thread = std::thread::spawn(move || {
            let mut buf = String::new();
            let _ = std::io::Read::read_to_string(&mut BufReader::new(stderr), &mut buf);
            buf
        });

        for line in BufReader::new(stdout).lines() {
            let line = line?;
            match TestEvent::parse(&line) {
                Some(ev) => {
                    if let Some(idx) = col.handle(&ev) {
                        if !reported.contains(&idx) {
                            reported.push(idx);
                            if !ctx.json {
                                print_package(&col.packages[idx], args.verbose, ctx.ui.color);
                            }
                        }
                    }
                }
                None => {
                    if !line.trim().is_empty() {
                        stray.push(line);
                    }
                }
            }
        }
        let status = child.wait()?;
        let stderr_text = stderr_thread.join().unwrap_or_default();
        if !status.success() {
            go_failed = true;
            let mut lines: Vec<String> = std::mem::take(&mut stray);
            lines.extend(stderr_text.lines().map(str::to_owned));
            let lines = tidy_output(lines);
            if !lines.is_empty() && (col.failures.is_empty() || !stderr_text.trim().is_empty()) {
                col.failures.push(Failure {
                    package: format!("go test (in {})", display_rel(&root, dir)),
                    test: None,
                    output: lines,
                });
            }
        }
    }

    // Packages that never reached a terminal event (e.g. -failfast, crashes).
    for (idx, p) in col.packages.iter().enumerate() {
        if p.status != PkgStatus::Running && !reported.contains(&idx) && !ctx.json {
            print_package(p, args.verbose, ctx.ui.color);
        }
    }

    let summary = Summary {
        passed: col.passed,
        failed: col.failed,
        skipped: col.skipped,
        elapsed: (started.elapsed().as_secs_f64() * 1000.0).round() / 1000.0,
    };
    let ok = !go_failed && col.failed == 0 && col.failures.is_empty();

    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.test/v1",
            project: ctx.project_name(),
            packages: col.packages,
            failures: col.failures,
            summary,
        })?;
    } else {
        print_failures(&col.failures, ctx.ui.color);
        print_summary(&summary, ok, &timer.elapsed(), ctx.ui.color);
    }
    Ok(if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn display_rel(root: &Path, path: &Path) -> String {
    let p = path.strip_prefix(root).unwrap_or(path);
    let s = p.to_string_lossy().replace('\\', "/");
    if s.is_empty() { ".".to_owned() } else { s }
}

// ---------------------------------------------------------------------------
// human output

fn secs(s: f64) -> String {
    format!("{s:.2}s")
}

fn print_package(p: &PkgResult, verbose: bool, color: bool) {
    match p.status {
        PkgStatus::Pass => println!(
            "  {} {}  {}",
            mark(color, Mark::Ok),
            p.package,
            dim(&secs(p.elapsed))
        ),
        PkgStatus::Fail => println!(
            "  {} {}  {}",
            mark(color, Mark::Fail),
            p.package,
            dim(&secs(p.elapsed))
        ),
        PkgStatus::Skip => println!(
            "  {} {}  {}",
            mark(color, Mark::Skip),
            p.package,
            dim("[skipped]")
        ),
        PkgStatus::NoTests => println!("  {}", dim(&format!("- {}  [no test files]", p.package))),
        PkgStatus::Running => println!(
            "  {} {}  {}",
            mark(color, Mark::Warn),
            p.package,
            dim("[incomplete]")
        ),
    }
    if verbose {
        for t in &p.tests {
            let m = match t.status {
                "pass" => mark(color, Mark::Ok),
                "fail" => mark(color, Mark::Fail),
                "skip" => mark(color, Mark::Skip),
                _ => mark(color, Mark::Warn),
            };
            println!("      {m} {}  {}", t.name, dim(&secs(t.elapsed)));
        }
    }
}

fn print_failures(failures: &[Failure], color: bool) {
    if failures.is_empty() {
        return;
    }
    for f in failures {
        println!();
        let what = match &f.test {
            Some(t) => format!("{}  {}", f.package, bold(t)),
            None => f.package.clone(),
        };
        let label = if color {
            red(&bold("FAIL"))
        } else {
            "FAIL".to_owned()
        };
        println!("  {label} {what}");
        for line in &f.output {
            println!("      {line}");
        }
    }
}

fn print_summary(s: &Summary, ok: bool, wall: &str, color: bool) {
    if s.passed + s.failed + s.skipped == 0 && ok {
        println!();
        println!("  {}", dim(&format!("no tests ran {wall}")));
        println!();
        return;
    }
    let mut parts = vec![green(&format!("{} passed", s.passed))];
    if s.failed > 0 {
        parts.push(red(&format!("{} failed", s.failed)));
    }
    if s.skipped > 0 {
        parts.push(yellow(&format!("{} skipped", s.skipped)));
    }
    let m = mark(color, if ok { Mark::Ok } else { Mark::Fail });
    println!();
    println!(
        "  {m} {} {}",
        parts.join(", "),
        dim(&format!("in {:.1}s", s.elapsed))
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(col: &mut Collector, lines: &[&str]) -> Vec<usize> {
        lines
            .iter()
            .filter_map(|l| TestEvent::parse(l))
            .filter_map(|ev| col.handle(&ev))
            .collect()
    }

    #[test]
    fn collects_package_and_test_results() {
        let mut col = Collector::default();
        let done = feed(
            &mut col,
            &[
                r#"{"Action":"start","Package":"m/calc"}"#,
                r#"{"Action":"run","Package":"m/calc","Test":"TestAdd"}"#,
                r#"{"Action":"output","Package":"m/calc","Test":"TestAdd","Output":"=== RUN   TestAdd\n"}"#,
                r#"{"Action":"pass","Package":"m/calc","Test":"TestAdd","Elapsed":0.01}"#,
                r#"{"Action":"run","Package":"m/calc","Test":"TestSub"}"#,
                r#"{"Action":"output","Package":"m/calc","Test":"TestSub","Output":"    calc_test.go:14: expected 1, got 2\n"}"#,
                r#"{"Action":"output","Package":"m/calc","Test":"TestSub","Output":"--- FAIL: TestSub (0.00s)\n"}"#,
                r#"{"Action":"fail","Package":"m/calc","Test":"TestSub","Elapsed":0}"#,
                r#"{"Action":"run","Package":"m/calc","Test":"TestSkip"}"#,
                r#"{"Action":"skip","Package":"m/calc","Test":"TestSkip","Elapsed":0}"#,
                r#"{"Action":"output","Package":"m/calc","Output":"FAIL\tm/calc\t0.003s\n"}"#,
                r#"{"Action":"fail","Package":"m/calc","Elapsed":0.003}"#,
            ],
        );
        assert_eq!(done, vec![0]);
        let p = &col.packages[0];
        assert_eq!(p.status, PkgStatus::Fail);
        assert_eq!(p.elapsed, 0.003);
        assert_eq!(p.tests.len(), 3);
        assert_eq!((col.passed, col.failed, col.skipped), (1, 1, 1));
        assert_eq!(col.failures.len(), 1);
        assert_eq!(col.failures[0].test.as_deref(), Some("TestSub"));
        assert_eq!(
            col.failures[0].output,
            vec![
                "    calc_test.go:14: expected 1, got 2",
                "--- FAIL: TestSub (0.00s)"
            ]
        );
    }

    #[test]
    fn no_test_files_is_its_own_status() {
        let mut col = Collector::default();
        feed(
            &mut col,
            &[
                r#"{"Action":"start","Package":"m/cmd"}"#,
                r#"{"Action":"output","Package":"m/cmd","Output":"?   \tm/cmd\t[no test files]\n"}"#,
                r#"{"Action":"skip","Package":"m/cmd","Elapsed":0}"#,
            ],
        );
        assert_eq!(col.packages[0].status, PkgStatus::NoTests);
        assert_eq!((col.passed, col.failed, col.skipped), (0, 0, 0));
    }

    #[test]
    fn build_failure_becomes_a_package_failure() {
        let mut col = Collector::default();
        feed(
            &mut col,
            &[
                r##"{"ImportPath":"m/bad","Action":"build-output","Output":"# m/bad\n"}"##,
                r#"{"ImportPath":"m/bad","Action":"build-output","Output":"bad.go:6:22: wrong type\n"}"#,
                r#"{"ImportPath":"m/bad","Action":"build-fail"}"#,
                r#"{"Action":"start","Package":"m/bad"}"#,
                r#"{"Action":"output","Package":"m/bad","Output":"FAIL\tm/bad [build failed]\n"}"#,
                r#"{"Action":"fail","Package":"m/bad","Elapsed":0,"FailedBuild":"m/bad"}"#,
            ],
        );
        assert_eq!(col.packages[0].status, PkgStatus::Fail);
        assert_eq!(col.failures.len(), 1);
        assert_eq!(col.failures[0].test, None);
        assert_eq!(
            col.failures[0].output,
            vec![
                "# m/bad",
                "bad.go:6:22: wrong type",
                "FAIL\tm/bad [build failed]"
            ]
        );
    }

    #[test]
    fn tidy_strips_frames_and_trailing_blanks() {
        let out = tidy_output(vec![
            "=== RUN   TestX".to_owned(),
            "=== PAUSE TestX".to_owned(),
            "=== CONT  TestX".to_owned(),
            "    x_test.go:3: boom".to_owned(),
            "".to_owned(),
        ]);
        assert_eq!(out, vec!["    x_test.go:3: boom"]);
    }
}
