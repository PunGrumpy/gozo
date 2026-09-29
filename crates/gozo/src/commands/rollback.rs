//! `gozo rollback [DEPLOYMENT_ID]` and `gozo rollback status`.
//!
//! The target is resolved from the local deployment history: an explicit id,
//! else the ready deployment before the current one. Without any history the
//! adapter is asked for its own previous revision (kubernetes only).

use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Instant;

use anyhow::{anyhow, bail};
use chrono::Utc;
use gozo_core::{Deployment, DeploymentHistory};
use gozo_deploy::{Adapter, RollbackTo};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::Serialize;

use crate::ctx::Ctx;
use crate::ui::{Timer, UiReporter};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Deployment id to roll back to (see `gozo ls`), or `status`.
    /// Defaults to the deployment before the current one.
    #[arg(value_name = "DEPLOYMENT_ID")]
    pub target: Option<String>,

    /// Seconds to wait for the rollout to finish.
    #[arg(long, default_value_t = 300, value_name = "SECS")]
    pub timeout: u64,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let root = ctx.root()?.to_path_buf();
    let link = ctx.link_required()?;
    let mut adapter = gozo_deploy::adapter_for(link, &ctx.config, &root)?;
    adapter.set_timeout(args.timeout);

    if args.target.as_deref() == Some("status") {
        return status(ctx, adapter.as_ref());
    }

    let mut history = DeploymentHistory::load(&root)?;
    let current: Option<Deployment> = history
        .list()
        .into_iter()
        .find(|d| d.status == "ready")
        .cloned();
    let target: Option<Deployment> = match &args.target {
        Some(id) => Some(history.find(id).cloned().ok_or_else(|| {
            anyhow!("no deployment {id:?} in the local history\n  run `gozo ls` to see recorded deployments")
        })?),
        None => current
            .as_ref()
            .and_then(|c| history.previous(c.environment))
            .cloned(),
    };

    ctx.ui.header();
    let from_label = current
        .as_ref()
        .map(|c| c.id.clone())
        .unwrap_or_else(|| adapter.describe());

    if let Some(t) = &target {
        let Some(image) = t.image.clone() else {
            bail!(
                "deployment {} has no image recorded, so it cannot be rolled back to",
                t.id
            );
        };
        if current.as_ref().is_some_and(|c| c.id == t.id) {
            bail!(
                "{} is already the current deployment\n  pick another id from `gozo ls`",
                t.id
            );
        }
        ctx.ui.step(format!(
            "Rolling back {from_label} to {} ({image}, {} ago)",
            ctx.ui.bold(&t.id),
            gozo_deploy::age(t.created_at, Utc::now())
        ));
    } else if args.target.is_none() && current.is_some() {
        bail!(
            "no earlier ready deployment to roll back to\n  run `gozo ls`, then `gozo rollback <id>`"
        );
    } else {
        ctx.ui.step(format!(
            "Rolling back {} to its previous revision (no local history)",
            adapter.describe()
        ));
    }

    if !ctx.ui.confirm("Roll back?", true)? {
        ctx.ui.info("Canceled.");
        return Ok(ExitCode::SUCCESS);
    }

    let timer = Timer::start();
    let started = Instant::now();
    let mut reporter = UiReporter(&ctx.ui);
    let result = match &target {
        Some(t) => adapter.rollback(RollbackTo::Deployment(t), &mut reporter),
        None => adapter.rollback(RollbackTo::Previous, &mut reporter),
    };
    let elapsed_ms = started.elapsed().as_millis();
    if let Err(e) = result {
        if ctx.json {
            ctx.out.json_value(&RollbackDoc {
                schema: "gozo.rollback/v1",
                action: "rollback",
                from: current.as_ref(),
                to: target.as_ref(),
                deployment: None,
                elapsed_ms,
                error: Some(format!("{e:#}")),
            })?;
        } else {
            ctx.ui.error(format!("rollback failed {}", timer.elapsed()));
            ctx.ui.error(format!("{e:#}"));
        }
        return Ok(ExitCode::from(1));
    }

    // Record what happened: the replaced deployment is rolled back, and the
    // restored one becomes a fresh ready entry pointing at the same image.
    if let Some(c) = &current {
        if let Some(d) = history.find_mut(&c.id) {
            d.status = "rolled-back".to_owned();
        }
    }
    let record = target.as_ref().map(|t| {
        let mut meta = BTreeMap::new();
        meta.insert("rollbackOf".to_owned(), t.id.clone());
        if let Some(c) = &current {
            meta.insert("rolledBackFrom".to_owned(), c.id.clone());
        }
        Deployment {
            id: Deployment::new_id(),
            target: t.target,
            environment: t.environment,
            created_at: Utc::now(),
            image: t.image.clone(),
            git_sha: t.git_sha.clone(),
            git_branch: t.git_branch.clone(),
            status: "ready".to_owned(),
            url: t.url.clone(),
            meta,
        }
    });
    if let Some(r) = &record {
        history.push(r.clone());
    }
    history.save(&root)?;

    if ctx.json {
        ctx.out.json_value(&RollbackDoc {
            schema: "gozo.rollback/v1",
            action: "rollback",
            from: current.as_ref(),
            to: target.as_ref(),
            deployment: record.as_ref(),
            elapsed_ms,
            error: None,
        })?;
        return Ok(ExitCode::SUCCESS);
    }

    match &target {
        Some(t) => {
            ctx.ui.success(format!(
                "Rolled back to {} ({}) {}",
                t.id,
                t.image.as_deref().unwrap_or("-"),
                timer.elapsed()
            ));
            ctx.out.line(&t.id);
        }
        None => ctx.ui.success(format!(
            "Rolled back {} to its previous revision {}",
            adapter.describe(),
            timer.elapsed()
        )),
    }
    Ok(ExitCode::SUCCESS)
}

/// `gozo rollback status`: is the target healthy after the rollout?
fn status(ctx: &Ctx, adapter: &dyn Adapter) -> anyhow::Result<ExitCode> {
    let s = adapter
        .status()
        .map_err(|e| anyhow!("could not read the status of {}: {e:#}", adapter.describe()))?;
    if ctx.json {
        ctx.out.json_value(&StatusDoc {
            schema: "gozo.rollback/v1",
            action: "status",
            target: adapter.describe(),
            healthy: s.healthy,
            summary: &s.summary,
            image: s.image.as_deref(),
            ready: s.ready,
            desired: s.desired,
        })?;
    } else {
        let mark = match (ctx.ui.color, s.healthy) {
            (true, true) => "✓".if_supports_color(Stdout, |t| t.green()).to_string(),
            (true, false) => "✗".if_supports_color(Stdout, |t| t.red()).to_string(),
            (false, true) => "OK".to_owned(),
            (false, false) => "FAIL".to_owned(),
        };
        ctx.out
            .line(format!("{mark} {}: {}", adapter.describe(), s.summary));
    }
    Ok(if s.healthy {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

#[derive(Serialize)]
struct RollbackDoc<'a> {
    schema: &'static str,
    action: &'static str,
    from: Option<&'a Deployment>,
    to: Option<&'a Deployment>,
    deployment: Option<&'a Deployment>,
    elapsed_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct StatusDoc<'a> {
    schema: &'static str,
    action: &'static str,
    target: String,
    healthy: bool,
    summary: &'a str,
    image: Option<&'a str>,
    ready: Option<u32>,
    desired: Option<u32>,
}
