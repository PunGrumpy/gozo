use std::process::ExitCode;

use gozo_doctor::{Options, Report, Status};

use super::util::{exit_for, plural};
use crate::ctx::Ctx;
use crate::style::{bold, dim, green, red, yellow};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Skip checks that need the network.
    #[arg(long)]
    pub offline: bool,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let project = ctx.project()?;
    let report = gozo_doctor::run(
        &ctx.go,
        project,
        &Options {
            offline: args.offline,
        },
    );

    if ctx.json {
        ctx.out.json_value(&report)?;
    } else {
        print_human(&report);
    }
    Ok(exit_for(report.healthy()))
}

fn print_human(r: &Report) {
    println!();
    println!("  {}", bold("gozo doctor"));
    println!();
    let kind = match r.project.kind {
        gozo_go::ProjectKind::Workspace => {
            format!("workspace, {} modules", r.project.modules.len())
        }
        gozo_go::ProjectKind::Module => "module".to_owned(),
    };
    println!(
        "  {} {}  {}",
        dim(&format!("{:<10}", "project")),
        bold(&r.project.name),
        dim(&format!("({kind})"))
    );
    println!(
        "  {} {}",
        dim(&format!("{:<10}", "root")),
        r.project.root.display()
    );
    println!(
        "  {} {}  {}",
        dim(&format!("{:<10}", "go")),
        r.go.version.as_deref().unwrap_or("unknown"),
        dim(&r.go.bin.display().to_string())
    );
    println!();

    for f in &r.findings {
        let mark = match f.status {
            Status::Ok => green("✓"),
            Status::Warn => yellow("⚠"),
            Status::Fail => red("✗"),
            Status::Skip => dim("-"),
        };
        let title = match f.status {
            Status::Skip => dim(&f.title),
            _ => f.title.clone(),
        };
        println!("  {mark} {title}");
        for d in &f.details {
            println!("      {}", dim(d));
        }
        if let Some(h) = &f.hint {
            println!("      {} {}", dim("hint:"), h);
        }
    }

    println!();
    let s = &r.summary;
    let mut parts = vec![green(&format!("{} ok", s.ok))];
    if s.warn > 0 {
        parts.push(yellow(&format!("{} warning{}", s.warn, plural(s.warn))));
    }
    if s.fail > 0 {
        parts.push(red(&format!("{} error{}", s.fail, plural(s.fail))));
    }
    if s.skip > 0 {
        parts.push(dim(&format!("{} skipped", s.skip)));
    }
    println!("  {}", parts.join(", "));
    println!();
}
