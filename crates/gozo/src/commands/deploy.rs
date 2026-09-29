//! stdout carries only the deployment URL (or the image when there is none).

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use anyhow::{Context as _, anyhow, bail};
use chrono::{DateTime, Utc};
use gozo_core::{Deployment, DeploymentHistory, EnvStore, Environment, Link, Target, git};
use gozo_deploy::DeployRequest;
use owo_colors::{OwoColorize, Stream::Stderr};
use serde::Serialize;

use super::util::{exit_for, plural};
use crate::ctx::Ctx;
use crate::ui::{Timer, UiReporter};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Deploy to production instead of preview.
    #[arg(long)]
    pub prod: bool,

    /// Deploy to this target instead of the linked one (docker or kubernetes).
    #[arg(long, value_name = "TARGET", value_parser = parse_target)]
    pub target: Option<Target>,

    /// Runtime variable for the deployed process, on top of `gozo env`. Repeatable.
    #[arg(short = 'e', long = "env", value_name = "KEY=VALUE")]
    pub env: Vec<String>,

    /// Build argument passed to `docker build --build-arg`. Repeatable.
    #[arg(short = 'b', long = "build-env", value_name = "KEY=VALUE")]
    pub build_env: Vec<String>,

    /// Image tag. Defaults to the short git SHA (plus `-dirty`) or a timestamp.
    #[arg(long, value_name = "TAG")]
    pub tag: Option<String>,

    /// Rebuild the image without the build cache.
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Return as soon as the rollout is requested instead of waiting for it.
    #[arg(long)]
    pub no_wait: bool,

    /// Stream the underlying build and rollout output.
    #[arg(long)]
    pub logs: bool,

    /// Build the image but do not start or roll out anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Free-form metadata recorded with the deployment. Repeatable.
    #[arg(short = 'm', long = "meta", value_name = "KEY=VALUE")]
    pub meta: Vec<String>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let root = ctx.root()?.to_path_buf();
    ctx.ui.header();
    let link = resolve_link(ctx, args.target)?;
    let adapter = gozo_deploy::adapter_for(&link, &ctx.config, &root)?;

    let environment = if args.prod {
        Environment::Production
    } else {
        Environment::Preview
    };
    let mut runtime_env: BTreeMap<String, String> = EnvStore::new(&root)
        .list(environment)
        .with_context(|| format!("reading `gozo env` variables for {environment}"))?
        .into_iter()
        .collect();
    let stored = runtime_env.len();
    runtime_env.extend(parse_list(&args.env, "--env", parse_env_kv)?);
    let build_env: BTreeMap<String, String> =
        parse_list(&args.build_env, "--build-env", parse_env_kv)?
            .into_iter()
            .collect();
    let meta: BTreeMap<String, String> = parse_list(&args.meta, "--meta", parse_kv)?
        .into_iter()
        .collect();

    let git_sha = git::sha(&root);
    let git_branch = git::branch(&root);
    let tag = match args.tag {
        Some(t) if !t.trim().is_empty() => t,
        _ => default_tag(&root, Utc::now()),
    };

    ctx.ui.step(format!(
        "Deploying {} to {} ({environment})",
        ctx.ui.bold(&link.project_name),
        link.target
    ));
    if stored > 0 {
        ctx.ui.detail(format!(
            "{stored} variable{} from `gozo env` ({environment})",
            plural(stored)
        ));
    }

    let mut reporter = UiReporter(&ctx.ui);
    let checks = adapter
        .preflight(&mut reporter)
        .context("preflight checks could not run")?;
    let mut failed = Vec::new();
    for (what, ok, detail) in &checks {
        if *ok {
            ctx.ui.success(format!("{what} {}", ctx.ui.dim(detail)));
        } else {
            ctx.ui
                .info(format!("{} {what} {detail}", fail_mark(ctx.ui.color)));
            failed.push(what.as_str());
        }
    }
    if !failed.is_empty() {
        bail!(
            "preflight failed ({})\n  fix the checks marked above and run `gozo deploy` again",
            failed.join(", ")
        );
    }

    let req = DeployRequest {
        root: root.clone(),
        project_name: link.project_name.clone(),
        environment,
        tag: tag.clone(),
        git_sha: git_sha.clone(),
        git_branch: git_branch.clone(),
        build_env,
        runtime_env,
        verbose: args.logs || ctx.debug,
        dry_run: args.dry_run,
        force: args.force,
        no_wait: args.no_wait,
    };

    let timer = Timer::start();
    let started = Instant::now();
    let result = adapter.deploy(&req, &mut reporter);
    let elapsed_ms = started.elapsed().as_millis();

    let mut deployment = Deployment {
        id: Deployment::new_id(),
        target: link.target,
        environment,
        created_at: Utc::now(),
        image: None,
        git_sha,
        git_branch,
        status: "ready".to_owned(),
        url: None,
        meta,
    };

    let (inspect, error) = match result {
        Ok(outcome) => {
            deployment.image = Some(outcome.image);
            deployment.url = outcome.url;
            (outcome.inspect, None)
        }
        Err(e) => {
            deployment.status = "error".to_owned();
            (None, Some(format!("{e:#}")))
        }
    };

    if args.dry_run {
        ctx.ui.detail("dry run: not recorded in deployment history");
    } else {
        let mut history = DeploymentHistory::load(&root)?;
        history.push(deployment.clone());
        history.save(&root)?;
    }

    if ctx.json {
        ctx.out.json_value(&DeployDoc {
            schema: "gozo.deploy/v1",
            deployment: DeploymentDoc {
                id: &deployment.id,
                target: deployment.target,
                environment: deployment.environment,
                image: deployment.image.as_deref(),
                url: deployment.url.as_deref(),
                inspect: inspect.as_deref(),
                git_sha: deployment.git_sha.as_deref(),
                created_at: deployment.created_at,
                status: &deployment.status,
            },
            elapsed_ms,
            dry_run: args.dry_run,
            error: error.as_deref(),
        })?;
        return Ok(exit_for(error.is_none()));
    }

    if let Some(err) = error {
        ctx.ui.error(format!("deploy failed {}", timer.elapsed()));
        ctx.ui.error(err);
        ctx.ui
            .hint("re-run with --logs to see the full tool output");
        return Ok(ExitCode::from(1));
    }

    let image = deployment.image.clone().unwrap_or_default();
    if args.dry_run {
        ctx.ui
            .success(format!("Built {image} {} (dry run)", timer.elapsed()));
    } else {
        ctx.ui
            .success(format!("Deployed {image} {}", timer.elapsed()));
    }
    if let Some(cmd) = &inspect {
        ctx.ui.detail(format!("Inspect: {cmd}"));
    }
    ctx.out.line(deployment.url.as_deref().unwrap_or(&image));
    Ok(ExitCode::SUCCESS)
}

/// `--target` overrides the linked target; unlinked directories can still
/// deploy with `--target` or `deploy.target` in gozo.toml.
fn resolve_link(ctx: &Ctx, target: Option<Target>) -> anyhow::Result<Link> {
    if let Some(linked) = &ctx.link {
        let mut link = linked.clone();
        link.project_name = ctx.project_name();
        if let Some(t) = target {
            if t != link.target {
                ctx.ui.detail(format!(
                    "using --target {t} instead of the linked {}",
                    link.target
                ));
                link.target = t;
            }
        }
        return Ok(link);
    }
    let configured = ctx
        .config
        .deploy
        .target
        .as_deref()
        .map(|t| t.parse::<Target>())
        .transpose()?;
    match target.or(configured) {
        Some(t) => {
            ctx.ui
                .detail(format!("not linked; deploying to {t} without a link"));
            Ok(Link {
                project_name: ctx.project_name(),
                target: t,
                docker: None,
                kubernetes: None,
                linked_at: Utc::now(),
            })
        }
        None => Err(anyhow!(
            "this directory is not linked to a deployment target\n  run `gozo link` first, or pass --target docker|kubernetes"
        )),
    }
}

fn default_tag(root: &Path, now: DateTime<Utc>) -> String {
    match git::short_sha(root) {
        Some(sha) if git::is_dirty(root) => format!("{sha}-dirty"),
        Some(sha) => sha,
        None => timestamp_tag(now),
    }
}

fn timestamp_tag(now: DateTime<Utc>) -> String {
    now.format("%Y%m%d%H%M%S").to_string()
}

fn fail_mark(color: bool) -> String {
    if color {
        "✗".if_supports_color(Stderr, |t| t.red()).to_string()
    } else {
        "FAIL".to_owned()
    }
}

fn parse_target(s: &str) -> Result<Target, String> {
    s.parse::<Target>().map_err(|e| e.to_string())
}

/// `KEY=VALUE`; the value may contain `=`.
fn parse_kv(s: &str) -> Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("expected KEY=VALUE, got {s:?}"))?;
    let k = k.trim();
    if k.is_empty() || k.chars().any(char::is_whitespace) {
        return Err(format!("invalid key in {s:?}"));
    }
    Ok((k.to_owned(), v.to_owned()))
}

fn parse_env_kv(s: &str) -> Result<(String, String), String> {
    let (k, v) = parse_kv(s)?;
    if !gozo_core::envstore::valid_key(&k) {
        return Err(format!(
            "invalid variable name {k:?} (use letters, digits and underscores)"
        ));
    }
    Ok((k, v))
}

fn parse_list(
    items: &[String],
    flag: &str,
    parse: fn(&str) -> Result<(String, String), String>,
) -> anyhow::Result<Vec<(String, String)>> {
    items
        .iter()
        .map(|s| parse(s).map_err(|e| anyhow!("{flag}: {e}")))
        .collect()
}

#[derive(Serialize)]
struct DeployDoc<'a> {
    schema: &'static str,
    deployment: DeploymentDoc<'a>,
    elapsed_ms: u128,
    dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeploymentDoc<'a> {
    id: &'a str,
    target: Target,
    environment: Environment,
    image: Option<&'a str>,
    url: Option<&'a str>,
    inspect: Option<&'a str>,
    git_sha: Option<&'a str>,
    created_at: DateTime<Utc>,
    status: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn kv_parsing() {
        assert_eq!(parse_kv("A=1").unwrap(), ("A".into(), "1".into()));
        assert_eq!(
            parse_kv("URL=http://x?a=b").unwrap(),
            ("URL".into(), "http://x?a=b".into())
        );
        assert_eq!(parse_kv("EMPTY=").unwrap(), ("EMPTY".into(), String::new()));
        assert_eq!(
            parse_kv("build-id=7").unwrap(),
            ("build-id".into(), "7".into())
        );
        assert!(parse_kv("NOEQ").is_err());
        assert!(parse_kv("=v").is_err());
        assert!(parse_kv("a b=v").is_err());
    }

    #[test]
    fn env_kv_requires_valid_name() {
        assert_eq!(
            parse_env_kv("PORT=8080").unwrap(),
            ("PORT".into(), "8080".into())
        );
        assert!(parse_env_kv("build-id=7").is_err());
        assert!(parse_env_kv("1ABC=x").is_err());
        assert!(parse_list(&["A=1".into(), "B".into()], "--env", parse_env_kv).is_err());
        let ok = parse_list(&["A=1".into(), "B=2".into()], "--env", parse_env_kv).unwrap();
        assert_eq!(ok.len(), 2);
    }

    #[test]
    fn target_parsing() {
        assert_eq!(parse_target("docker").unwrap(), Target::Docker);
        assert_eq!(parse_target("K8s").unwrap(), Target::Kubernetes);
        assert!(parse_target("heroku").is_err());
    }

    #[test]
    fn timestamp_tag_format() {
        let t = Utc.with_ymd_and_hms(2026, 9, 29, 13, 5, 9).unwrap();
        assert_eq!(timestamp_tag(t), "20260929130509");
    }

    #[test]
    fn default_tag_outside_git_is_timestamp() {
        let t = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let dir = std::env::temp_dir();
        let tag = default_tag(&dir, t);
        assert!(tag == "20260102030405" || tag.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    }
}
