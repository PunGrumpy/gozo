//! `gozo check` — `go vet`, gofmt and golangci-lint across every module.
//!
//! Each check produces one line in the style of `gozo doctor`, followed by
//! `file:line:col: message` details. Exit 1 when any check fails.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use crate::style::{Mark, dim, green, mark, red};
use serde::Serialize;

use crate::ctx::Ctx;

const MAX_ISSUE_LINES: usize = 20;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Skip `go vet`.
    #[arg(long)]
    pub no_vet: bool,

    /// Skip the gofmt check.
    #[arg(long)]
    pub no_fmt: bool,

    /// Skip golangci-lint even when it is installed.
    #[arg(long)]
    pub no_lint: bool,

    /// Rewrite unformatted files (`gofmt -w`) and let golangci-lint apply fixes.
    #[arg(long)]
    pub fix: bool,
}

#[derive(Debug, Serialize)]
struct Doc {
    schema: &'static str,
    project: String,
    checks: Vec<Check>,
    summary: Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Ok,
    Fail,
    Skip,
}

#[derive(Debug, Serialize)]
struct Check {
    id: &'static str,
    status: Status,
    /// Number of issues.
    count: usize,
    #[serde(skip)]
    title: String,
    #[serde(skip)]
    hint: Option<String>,
    issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize)]
struct Issue {
    /// Relative to the project root.
    file: String,
    line: u32,
    col: u32,
    message: String,
    tool: &'static str,
}

#[derive(Debug, Serialize)]
struct Summary {
    ok: usize,
    fail: usize,
    skip: usize,
    issues: usize,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let project = ctx.project()?;
    let root = project.root.clone();
    let modules: Vec<PathBuf> = project
        .modules
        .iter()
        .filter(|m| m.gomod.is_ok())
        .map(|m| m.dir.clone())
        .collect();
    if modules.is_empty() {
        anyhow::bail!(
            "no usable modules in {} (every go.mod failed to parse)",
            root.display()
        );
    }
    ctx.ui.header();

    let mut checks = Vec::new();
    if !args.no_vet {
        checks.push(run_vet(ctx, &root, &modules));
    }
    if !args.no_fmt {
        checks.push(run_fmt(ctx, &root, &modules, args.fix));
    }
    if !args.no_lint {
        checks.push(run_lint(ctx, &root, &modules, args.fix));
    }
    if checks.is_empty() {
        anyhow::bail!("nothing to check: --no-vet, --no-fmt and --no-lint were all given");
    }

    let summary = Summary {
        ok: checks.iter().filter(|c| c.status == Status::Ok).count(),
        fail: checks.iter().filter(|c| c.status == Status::Fail).count(),
        skip: checks.iter().filter(|c| c.status == Status::Skip).count(),
        issues: checks.iter().map(|c| c.issues.len()).sum(),
    };
    let failed = summary.fail > 0;

    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.check/v1",
            project: ctx.project_name(),
            checks,
            summary,
        })?;
    } else {
        print_human(&checks, &summary, ctx.ui.color);
    }
    Ok(if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

// ---------------------------------------------------------------------------
// vet

fn run_vet(ctx: &Ctx, root: &Path, modules: &[PathBuf]) -> Check {
    let mut issues = Vec::new();
    let mut packages = 0;
    let mut errors = Vec::new();
    for dir in modules {
        ctx.ui.debug(format!(
            "go vet -json ./...  (in {})",
            display_rel(root, dir)
        ));
        let (out, report) = match ctx.go.vet(dir, ["./..."]) {
            Ok(x) => x,
            Err(e) => {
                errors.push(e.to_string());
                continue;
            }
        };
        packages += report.packages;
        for i in &report.issues {
            let (file, line, col) = i.location();
            issues.push(Issue {
                file: display_rel(root, Path::new(file)),
                line,
                col,
                message: i.message.clone(),
                tool: "vet",
            });
        }
        if !out.success() && report.issues.is_empty() {
            // Build errors: vet prints them as plain text on stderr.
            for line in out.stderr.lines() {
                if let Some(issue) = parse_diagnostic(line, dir, root, "vet") {
                    issues.push(issue);
                }
            }
            if issues.is_empty() {
                errors.push(first_lines(&out.stderr, 3));
            }
        }
    }
    let n = issues.len();
    if !errors.is_empty() {
        return Check {
            id: "vet",
            status: Status::Fail,
            count: n,
            title: "go vet could not run".to_owned(),
            hint: Some(errors.join("\n")),
            issues,
        };
    }
    if n == 0 {
        Check {
            id: "vet",
            status: Status::Ok,
            count: 0,
            title: format!("vet clean ({packages} package{})", plural(packages)),
            hint: None,
            issues,
        }
    } else {
        Check {
            id: "vet",
            status: Status::Fail,
            count: n,
            title: format!("vet found {n} issue{}", plural(n)),
            hint: None,
            issues,
        }
    }
}

// ---------------------------------------------------------------------------
// gofmt

fn run_fmt(ctx: &Ctx, root: &Path, modules: &[PathBuf], fix: bool) -> Check {
    let gofmt = gofmt_bin(&ctx.go.bin);
    let mut files = BTreeSet::new();
    let mut issues = Vec::new();
    for dir in modules {
        let mut cmd = Command::new(&gofmt);
        cmd.arg("-l");
        if fix {
            cmd.arg("-w");
        }
        cmd.arg(".").current_dir(dir);
        ctx.ui.debug(format!(
            "{} -l{} .  (in {})",
            gofmt.display(),
            if fix { " -w" } else { "" },
            display_rel(root, dir)
        ));
        let out = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                return Check {
                    id: "fmt",
                    status: Status::Skip,
                    count: 0,
                    title: format!("gofmt skipped: {e}"),
                    hint: Some("gofmt ships with Go; check GOROOT/bin is intact".to_owned()),
                    issues,
                };
            }
        };
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let line = line.trim();
            if !line.is_empty() {
                files.insert(display_rel(root, &dir.join(line)));
            }
        }
        // Syntax errors make gofmt exit 2 and print `file:line:col: msg` on stderr.
        for line in String::from_utf8_lossy(&out.stderr).lines() {
            if let Some(issue) = parse_diagnostic(line, dir, root, "fmt") {
                issues.push(issue);
            }
        }
    }
    let n_files = files.len();
    let message = if fix { "reformatted" } else { "not gofmt-ed" };
    issues.extend(files.into_iter().map(|file| Issue {
        file,
        line: 0,
        col: 0,
        message: message.to_owned(),
        tool: "fmt",
    }));
    let syntax_errors = issues.iter().any(|i| i.line > 0);
    let n = issues.len();
    if syntax_errors {
        Check {
            id: "fmt",
            status: Status::Fail,
            count: n,
            title: format!("gofmt found {n} problem{}", plural(n)),
            hint: None,
            issues,
        }
    } else if n_files == 0 {
        Check {
            id: "fmt",
            status: Status::Ok,
            count: 0,
            title: "gofmt clean".to_owned(),
            hint: None,
            issues,
        }
    } else if fix {
        Check {
            id: "fmt",
            status: Status::Ok,
            count: n,
            title: format!("gofmt reformatted {n_files} file{}", plural(n_files)),
            hint: None,
            issues,
        }
    } else {
        Check {
            id: "fmt",
            status: Status::Fail,
            count: n,
            title: format!(
                "{n_files} file{} need{} gofmt",
                plural(n_files),
                if n_files == 1 { "s" } else { "" }
            ),
            hint: Some("run `gozo check --fix` to reformat".to_owned()),
            issues,
        }
    }
}

/// gofmt lives next to `go` in GOROOT/bin; fall back to PATH.
fn gofmt_bin(go_bin: &Path) -> PathBuf {
    let name = if cfg!(windows) { "gofmt.exe" } else { "gofmt" };
    go_bin
        .parent()
        .map(|d| d.join(name))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

// ---------------------------------------------------------------------------
// golangci-lint

fn run_lint(ctx: &Ctx, root: &Path, modules: &[PathBuf], fix: bool) -> Check {
    let Some(bin) = which("golangci-lint") else {
        return Check {
            id: "lint",
            status: Status::Skip,
            count: 0,
            title: "lint skipped: golangci-lint not on PATH".to_owned(),
            hint: Some(
                "install it: go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@latest".to_owned(),
            ),
            issues: Vec::new(),
        };
    };
    let v2 = golangci_is_v2(&bin);
    let mut issues = Vec::new();
    let mut errors = Vec::new();
    for dir in modules {
        let mut cmd = Command::new(&bin);
        cmd.arg("run");
        if v2 {
            cmd.args(["--output.json.path", "stdout", "--show-stats=false"]);
        } else {
            cmd.args(["--out-format", "json"]);
        }
        if fix {
            cmd.arg("--fix");
        }
        cmd.arg("./...").current_dir(dir);
        ctx.ui.debug(format!(
            "golangci-lint run {} ./...  (in {})",
            if v2 {
                "--output.json.path stdout"
            } else {
                "--out-format json"
            },
            display_rel(root, dir)
        ));
        let out = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                errors.push(format!("failed to run golangci-lint: {e}"));
                continue;
            }
        };
        let stdout = String::from_utf8_lossy(&out.stdout);
        match parse_golangci(&stdout) {
            Some(found) => {
                for (file, line, col, text, linter) in found {
                    issues.push(Issue {
                        file: display_rel(root, &dir.join(file)),
                        line,
                        col,
                        message: if linter.is_empty() {
                            text
                        } else {
                            format!("{text} ({linter})")
                        },
                        tool: "lint",
                    });
                }
            }
            None => {
                let code = out.status.code().unwrap_or(-1);
                if code != 0 {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    errors.push(format!(
                        "golangci-lint exited with {code}: {}",
                        first_lines(&stderr, 3)
                    ));
                }
            }
        }
    }
    let n = issues.len();
    if !errors.is_empty() {
        return Check {
            id: "lint",
            status: Status::Fail,
            count: n,
            title: "golangci-lint could not run".to_owned(),
            hint: Some(errors.join("\n")),
            issues,
        };
    }
    if n == 0 {
        Check {
            id: "lint",
            status: Status::Ok,
            count: 0,
            title: "lint clean".to_owned(),
            hint: None,
            issues,
        }
    } else {
        Check {
            id: "lint",
            status: Status::Fail,
            count: n,
            title: format!("lint found {n} issue{}", plural(n)),
            hint: if fix {
                None
            } else {
                Some("run `gozo check --fix` to apply auto-fixes".to_owned())
            },
            issues,
        }
    }
}

/// golangci-lint v2 renamed `--out-format` to `--output.<fmt>.path`.
fn golangci_is_v2(bin: &Path) -> bool {
    let Ok(out) = Command::new(bin).arg("version").output() else {
        return true;
    };
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // "golangci-lint has version 1.64.8 built ..." / "... version v2.1.6 ..."
    match text.split("version ").nth(1) {
        Some(rest) => !rest.trim_start_matches('v').starts_with("1."),
        None => true,
    }
}

type LintIssue = (String, u32, u32, String, String);

/// Pull `(file, line, col, text, linter)` tuples from golangci-lint's JSON report.
fn parse_golangci(stdout: &str) -> Option<Vec<LintIssue>> {
    let start = stdout.find('{')?;
    let doc: serde_json::Value = serde_json::from_str(stdout[start..].trim()).ok()?;
    let mut out = Vec::new();
    if let Some(list) = doc.get("Issues").and_then(|v| v.as_array()) {
        for i in list {
            let pos = i.get("Pos");
            let s = |v: Option<&serde_json::Value>, k: &str| {
                v.and_then(|p| p.get(k))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_owned()
            };
            let n = |k: &str| {
                pos.and_then(|p| p.get(k))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as u32
            };
            out.push((
                s(pos, "Filename"),
                n("Line"),
                n("Column"),
                s(Some(i), "Text"),
                s(Some(i), "FromLinter"),
            ));
        }
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// shared helpers

/// Parse a Go-style `file:line:col: message` (or `file:line: message`) line.
fn parse_diagnostic(
    line: &str,
    module_dir: &Path,
    root: &Path,
    tool: &'static str,
) -> Option<Issue> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with("vet:") {
        return None;
    }
    // Locate ": " after the position; the position itself may contain ':' (drive letters).
    let (posn, message) = line.split_once(": ")?;
    let (file, l, c) = gozo_go::split_posn(posn);
    if l == 0 || file.is_empty() {
        return None;
    }
    Some(Issue {
        file: display_rel(root, &module_dir.join(file)),
        line: l,
        col: c,
        message: message.trim().to_owned(),
        tool,
    })
}

fn display_rel(root: &Path, path: &Path) -> String {
    let p = path.strip_prefix(root).unwrap_or(path);
    let s = p.to_string_lossy().replace('\\', "/");
    if s.is_empty() { ".".to_owned() } else { s }
}

fn which(name: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p)
            .map(|d| d.join(&exe))
            .find(|c| c.is_file())
    })
}

fn first_lines(s: &str, n: usize) -> String {
    s.lines()
        .filter(|l| !l.trim().is_empty())
        .take(n)
        .collect::<Vec<_>>()
        .join("\n")
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

// ---------------------------------------------------------------------------
// human output

fn print_human(checks: &[Check], s: &Summary, color: bool) {
    println!();
    for c in checks {
        let (mark, title) = match c.status {
            Status::Ok => (mark(color, Mark::Ok), c.title.clone()),
            Status::Fail => (mark(color, Mark::Fail), c.title.clone()),
            Status::Skip => (mark(color, Mark::Skip), dim(&c.title)),
        };
        println!("  {mark} {title}");
        for i in c.issues.iter().take(MAX_ISSUE_LINES) {
            println!("      {}", format_issue(i));
        }
        if c.issues.len() > MAX_ISSUE_LINES {
            println!(
                "      {}",
                dim(&format!("… and {} more", c.issues.len() - MAX_ISSUE_LINES))
            );
        }
        if let Some(h) = &c.hint {
            for (n, line) in h.lines().enumerate() {
                if n == 0 {
                    println!("      {} {line}", dim("hint:"));
                } else {
                    println!("            {line}");
                }
            }
        }
    }
    println!();
    let mut parts = vec![green(&format!("{} ok", s.ok))];
    if s.fail > 0 {
        parts.push(red(&format!("{} failed", s.fail)));
    }
    if s.skip > 0 {
        parts.push(dim(&format!("{} skipped", s.skip)));
    }
    if s.issues > 0 {
        parts.push(format!("{} issue{}", s.issues, plural(s.issues)));
    }
    println!("  {}", parts.join(", "));
    println!();
}

fn format_issue(i: &Issue) -> String {
    let mut loc = i.file.clone();
    if i.line > 0 {
        loc.push_str(&format!(":{}", i.line));
        if i.col > 0 {
            loc.push_str(&format!(":{}", i.col));
        }
    }
    if i.message.is_empty() {
        loc
    } else {
        format!("{loc}: {}", dim(&i.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_go_diagnostic_lines() {
        let root = Path::new("/p");
        let dir = Path::new("/p/sub");
        let i =
            parse_diagnostic("internal/x/x.go:6:22: bad thing", dir, root, "vet").expect("issue");
        assert_eq!(i.file, "sub/internal/x/x.go");
        assert_eq!((i.line, i.col), (6, 22));
        assert_eq!(i.message, "bad thing");
        assert!(parse_diagnostic("# example.com/x", dir, root, "vet").is_none());
        assert!(parse_diagnostic("vet: something", dir, root, "vet").is_none());
        assert!(parse_diagnostic("no position here", dir, root, "vet").is_none());
    }

    #[test]
    fn parses_golangci_json() {
        let doc = r#"{"Issues":[{"FromLinter":"errcheck","Text":"unchecked","Pos":{"Filename":"a/b.go","Line":3,"Column":9}}],"Report":{}}"#;
        let got = parse_golangci(doc).expect("parsed");
        assert_eq!(
            got,
            vec![(
                "a/b.go".to_owned(),
                3,
                9,
                "unchecked".to_owned(),
                "errcheck".to_owned()
            )]
        );
        assert_eq!(parse_golangci(r#"{"Issues":null}"#), Some(Vec::new()));
        assert!(parse_golangci("level=error msg=boom").is_none());
    }
}
