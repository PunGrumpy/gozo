use std::process::ExitCode;

use anyhow::Context as _;
use chrono::Utc;
use gozo_core::{Deployment, DeploymentHistory, Target};
use serde::Serialize;

use super::util::exit_for;
use crate::ctx::Ctx;
use crate::style::{Mark, bold, dim, mark};

#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn run(ctx: &mut Ctx, _args: Args) -> anyhow::Result<ExitCode> {
    let root = ctx.root()?.to_path_buf();
    let link = ctx.link_required()?;
    let adapter = gozo_deploy::adapter_for(link, &ctx.config, &root)?;
    let history = DeploymentHistory::load(&root)?;
    let latest: Option<Deployment> = history.list().first().map(|d| (*d).clone());

    let status = adapter
        .status()
        .with_context(|| format!("could not read the status of {}", adapter.describe()))?;

    if ctx.json {
        ctx.out.json_value(&StatusDoc {
            schema: "gozo.status/v1",
            project: ctx.project_name(),
            target: status.target,
            description: adapter.describe(),
            healthy: status.healthy,
            summary: &status.summary,
            image: status.image.as_deref(),
            ready: status.ready,
            desired: status.desired,
            since: status.since.as_deref(),
            deployment: latest.as_ref(),
            raw: &status.raw,
        })?;
        return Ok(exit_for(status.healthy));
    }

    let health = format!(
        "{} {}",
        mark(
            ctx.ui.color,
            if status.healthy { Mark::Ok } else { Mark::Fail }
        ),
        status.summary
    );
    println!();
    kv("project", &bold(&ctx.project_name()));
    kv(
        "target",
        &format!("{} ({})", status.target, adapter.describe()),
    );
    kv("health", &health);
    kv("image", status.image.as_deref().unwrap_or("-"));
    match &latest {
        Some(d) => kv(
            "deployment",
            &format!(
                "{} ({}, {}, {} ago)",
                d.id,
                d.environment,
                d.status,
                gozo_deploy::age(d.created_at, Utc::now())
            ),
        ),
        None => kv("deployment", "none recorded (run `gozo deploy`)"),
    }
    if let Some(s) = &status.since {
        kv("since", s);
    }
    println!();
    Ok(exit_for(status.healthy))
}

fn kv(label: &str, value: &str) {
    println!("  {} {}", dim(&format!("{label:<11}")), value);
}

#[derive(Serialize)]
struct StatusDoc<'a> {
    schema: &'static str,
    project: String,
    target: Target,
    description: String,
    healthy: bool,
    summary: &'a str,
    image: Option<&'a str>,
    ready: Option<u32>,
    desired: Option<u32>,
    since: Option<&'a str>,
    deployment: Option<&'a Deployment>,
    raw: &'a serde_json::Value,
}
