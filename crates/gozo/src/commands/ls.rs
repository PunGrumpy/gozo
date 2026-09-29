//! `gozo ls`: list recorded deployments, newest first.

use std::process::ExitCode;

use chrono::{DateTime, Utc};
use gozo_core::{Deployment, DeploymentHistory, Environment};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::Serialize;

use crate::ctx::Ctx;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Show at most this many deployments.
    #[arg(short = 'n', long, default_value_t = 20, value_name = "N")]
    pub limit: usize,

    /// Only deployments for this environment (development, preview, production).
    #[arg(long, value_name = "ENV", value_parser = parse_environment)]
    pub environment: Option<Environment>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let root = ctx.root()?;
    let history = DeploymentHistory::load(root)?;
    let now = Utc::now();
    let all = history.list();
    let rows: Vec<&Deployment> = all
        .iter()
        .copied()
        .filter(|d| args.environment.is_none_or(|e| d.environment == e))
        .take(args.limit)
        .collect();

    if ctx.json {
        let deployments: Vec<serde_json::Value> = rows
            .iter()
            .map(|d| {
                let mut v = serde_json::to_value(d).unwrap_or(serde_json::Value::Null);
                if let Some(o) = v.as_object_mut() {
                    o.insert("age".into(), gozo_deploy::age(d.created_at, now).into());
                }
                v
            })
            .collect();
        ctx.out.json_value(&ListDoc {
            schema: "gozo.deployments/v1",
            project: ctx.project_name(),
            total: all.len(),
            deployments,
        })?;
        return Ok(ExitCode::SUCCESS);
    }

    if rows.is_empty() {
        ctx.ui.info(match args.environment {
            Some(e) => format!("No {e} deployments yet. Run `gozo deploy{}`.", prod_flag(e)),
            None => "No deployments yet. Run `gozo deploy`.".to_owned(),
        });
        return Ok(ExitCode::SUCCESS);
    }

    ctx.ui.step(format!(
        "{} of {} deployment{} for {}",
        rows.len(),
        all.len(),
        if all.len() == 1 { "" } else { "s" },
        ctx.ui.bold(&ctx.project_name())
    ));
    ctx.ui.blank();
    for line in render_table(&rows, now) {
        ctx.out.line(line);
    }
    Ok(ExitCode::SUCCESS)
}

fn prod_flag(e: Environment) -> &'static str {
    match e {
        Environment::Production => " --prod",
        _ => "",
    }
}

pub(crate) fn parse_environment(s: &str) -> Result<Environment, String> {
    s.parse::<Environment>().map_err(|e| e.to_string())
}

/// `id  age  environment  status  image` with aligned columns; the first
/// line is the (dimmed) header.
fn render_table(rows: &[&Deployment], now: DateTime<Utc>) -> Vec<String> {
    let cells: Vec<[String; 5]> = rows
        .iter()
        .map(|d| {
            [
                d.id.clone(),
                gozo_deploy::age(d.created_at, now),
                d.environment.to_string(),
                d.status.clone(),
                d.image.clone().unwrap_or_else(|| "-".to_owned()),
            ]
        })
        .collect();
    let header = ["id", "age", "environment", "status", "image"];
    let mut widths = header.map(str::len);
    for row in &cells {
        for (w, cell) in widths.iter_mut().zip(row) {
            *w = (*w).max(cell.chars().count());
        }
    }
    let pad = |s: &str, w: usize| format!("{s:<w$}");
    let mut out = Vec::with_capacity(cells.len() + 1);
    out.push(
        header
            .iter()
            .zip(widths)
            .map(|(h, w)| pad(h, w))
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .if_supports_color(Stdout, |t| t.dimmed())
            .to_string(),
    );
    for row in &cells {
        let mut line = Vec::with_capacity(5);
        for (i, (cell, w)) in row.iter().zip(widths).enumerate() {
            let text = pad(cell, w);
            line.push(if i == 3 {
                color_status(cell, text)
            } else {
                text
            });
        }
        out.push(line.join("  ").trim_end().to_owned());
    }
    out
}

fn color_status(status: &str, text: String) -> String {
    match status {
        "ready" => text.if_supports_color(Stdout, |t| t.green()).to_string(),
        "error" => text.if_supports_color(Stdout, |t| t.red()).to_string(),
        "rolled-back" | "canceled" => text.if_supports_color(Stdout, |t| t.yellow()).to_string(),
        _ => text,
    }
}

#[derive(Serialize)]
struct ListDoc {
    schema: &'static str,
    project: String,
    total: usize,
    deployments: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use gozo_core::Target;

    fn dep(
        id: &str,
        secs_ago: i64,
        env: Environment,
        status: &str,
        image: Option<&str>,
    ) -> Deployment {
        let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
        Deployment {
            id: id.into(),
            target: Target::Docker,
            environment: env,
            created_at: now - chrono::Duration::seconds(secs_ago),
            image: image.map(str::to_owned),
            git_sha: None,
            git_branch: None,
            status: status.into(),
            url: None,
            meta: Default::default(),
        }
    }

    #[test]
    fn table_is_aligned() {
        owo_colors::set_override(false);
        let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
        let a = dep(
            "dpl_abc",
            7200,
            Environment::Production,
            "ready",
            Some("hello:abc123"),
        );
        let b = dep(
            "dpl_defghijk",
            90,
            Environment::Preview,
            "rolled-back",
            None,
        );
        let lines = render_table(&[&a, &b], now);
        assert_eq!(
            lines[0],
            "id            age  environment  status       image"
        );
        assert_eq!(
            lines[1],
            "dpl_abc       2h   production   ready        hello:abc123"
        );
        assert_eq!(lines[2], "dpl_defghijk  1m   preview      rolled-back  -");
    }

    #[test]
    fn environment_parsing() {
        assert_eq!(parse_environment("prod").unwrap(), Environment::Production);
        assert!(parse_environment("qa").is_err());
    }
}
