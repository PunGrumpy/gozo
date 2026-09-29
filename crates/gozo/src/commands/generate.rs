//! `gozo generate` — `go generate` in every module, optionally verifying
//! (via git) that the committed generated code is current.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::style::{Mark, dim, mark};
use serde::Serialize;

use crate::ctx::Ctx;
use crate::ui::Timer;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Fail (exit 1) if generating changed any tracked or untracked file.
    /// Needs a git repository.
    #[arg(long)]
    pub check: bool,

    /// Package patterns to generate (default `./...`).
    #[arg(value_name = "PACKAGES")]
    pub packages: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Doc {
    schema: &'static str,
    project: String,
    modules: Vec<ModuleResult>,
    stale: Vec<String>,
    check: bool,
}

#[derive(Debug, Serialize)]
struct ModuleResult {
    /// Module directory relative to the project root.
    path: String,
    ok: bool,
    error: Option<String>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let project = ctx.project()?;
    let root = project.root.clone();
    let modules: Vec<(String, PathBuf)> = project
        .modules
        .iter()
        .filter(|m| m.gomod.is_ok())
        .map(|m| (m.rel.clone(), m.dir.clone()))
        .collect();
    if modules.is_empty() {
        anyhow::bail!("no usable modules in {}", root.display());
    }
    if args.check && !gozo_core::git::is_repo(&root) {
        anyhow::bail!(
            "`gozo generate --check` needs a git repository to detect changes\n  {} is not inside one; run `git init` or drop --check",
            root.display()
        );
    }

    let patterns: Vec<String> = if args.packages.is_empty() {
        vec!["./...".to_owned()]
    } else {
        args.packages.clone()
    };

    ctx.ui.header();
    ctx.ui.step(format!(
        "Generating ({} module{})",
        modules.len(),
        if modules.len() == 1 { "" } else { "s" }
    ));
    let timer = Timer::start();
    let before: BTreeSet<String> = if args.check {
        gozo_core::git::changed_paths(&root).into_iter().collect()
    } else {
        BTreeSet::new()
    };

    let mut results = Vec::new();
    for (rel, dir) in &modules {
        let mut go_args = vec!["generate".to_owned()];
        go_args.extend(patterns.iter().cloned());
        ctx.ui
            .debug(format!("go {}  (in {rel})", go_args.join(" ")));
        let res = match ctx.go.run(dir, &go_args) {
            Ok(out) => {
                if ctx.debug {
                    for line in out.stdout.lines().chain(out.stderr.lines()) {
                        ctx.ui.detail(line);
                    }
                }
                if out.success() {
                    ModuleResult {
                        path: rel.clone(),
                        ok: true,
                        error: None,
                    }
                } else {
                    ModuleResult {
                        path: rel.clone(),
                        ok: false,
                        error: Some(out.stderr.trim().to_owned()),
                    }
                }
            }
            Err(e) => ModuleResult {
                path: rel.clone(),
                ok: false,
                error: Some(e.to_string()),
            },
        };
        if res.ok {
            ctx.ui.detail(format!("{rel}  ok"));
        }
        results.push(res);
    }

    let stale: Vec<String> = if args.check {
        gozo_core::git::changed_paths(&root)
            .into_iter()
            .filter(|p| !before.contains(p))
            .collect()
    } else {
        Vec::new()
    };
    let failed_modules = results.iter().filter(|r| !r.ok).count();
    let ok = failed_modules == 0 && stale.is_empty();

    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.generate/v1",
            project: ctx.project_name(),
            modules: results,
            stale,
            check: args.check,
        })?;
    } else {
        print_human(&results, &stale, args.check, &timer.elapsed(), ctx.ui.color);
    }
    Ok(if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn print_human(
    results: &[ModuleResult],
    stale: &[String],
    check: bool,
    elapsed: &str,
    color: bool,
) {
    for r in results.iter().filter(|r| !r.ok) {
        println!(
            "  {} go generate failed in {}",
            mark(color, Mark::Fail),
            r.path
        );
        for line in r.error.as_deref().unwrap_or("").lines() {
            println!("      {line}");
        }
    }
    let n_ok = results.iter().filter(|r| r.ok).count();
    if check {
        if stale.is_empty() {
            if n_ok == results.len() {
                println!(
                    "  {} generated code is up to date  {}",
                    mark(color, Mark::Ok),
                    dim(elapsed)
                );
            }
        } else {
            let n = stale.len();
            println!(
                "  {} {n} generated file{} {} stale  {}",
                mark(color, Mark::Fail),
                if n == 1 { "" } else { "s" },
                if n == 1 { "is" } else { "are" },
                dim(elapsed)
            );
            for p in stale {
                println!("      {p}");
            }
            println!(
                "      {} generated code is stale, commit the result of `gozo generate`",
                dim("hint:")
            );
        }
    } else if n_ok == results.len() {
        println!(
            "  {} go generate ran in {n_ok} module{}  {}",
            mark(color, Mark::Ok),
            if n_ok == 1 { "" } else { "s" },
            dim(elapsed)
        );
    }
}
