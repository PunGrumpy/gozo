//! Deployment target adapters.
//!
//! An adapter turns the abstract verbs `deploy`, `status`, `logs`, `rollback`
//! and `env` into concrete `docker` / `kubectl` invocations. The CLI never
//! talks to those tools directly.

pub mod docker;
mod image;
pub mod kubernetes;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Utc};
use serde::Serialize;

use gozo_core::{Config, Deployment, Environment, Link, Target};

#[derive(Debug, thiserror::Error)]
pub enum DeployError {
    #[error("{tool} is not installed or not on PATH")]
    ToolMissing { tool: &'static str },
    #[error("`{cmd}` failed with exit code {status}\n{stderr}")]
    CommandFailed {
        cmd: String,
        status: i32,
        stderr: String,
    },
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Core(#[from] gozo_core::CoreError),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DeployError>;

/// Progress sink so adapters can narrate without knowing about colors or TTYs.
pub trait Reporter {
    /// A high-level step, rendered as `> message`.
    fn step(&mut self, msg: &str);
    /// A command about to run, rendered dimmed. Only shown in debug/verbose.
    fn command(&mut self, cmd: &str);
    /// Secondary information under a step.
    fn detail(&mut self, msg: &str);
    /// A non-fatal problem.
    fn warn(&mut self, msg: &str);
}

/// A no-op reporter for JSON mode and tests.
pub struct SilentReporter;
impl Reporter for SilentReporter {
    fn step(&mut self, _: &str) {}
    fn command(&mut self, _: &str) {}
    fn detail(&mut self, _: &str) {}
    fn warn(&mut self, _: &str) {}
}

#[derive(Debug, Clone)]
pub struct DeployRequest {
    pub root: PathBuf,
    pub project_name: String,
    pub environment: Environment,
    /// Image tag to build (usually the short git SHA or a timestamp).
    pub tag: String,
    pub git_sha: Option<String>,
    pub git_branch: Option<String>,
    /// Variables for the build step (`--build-env`).
    pub build_env: BTreeMap<String, String>,
    /// Runtime variables for the deployed process (`--env` plus the env store).
    pub runtime_env: BTreeMap<String, String>,
    /// Print the underlying tool output (`--logs`).
    pub verbose: bool,
    /// Do everything except the final apply/run (`--dry-run`).
    pub dry_run: bool,
    /// Skip the build cache (`--force`).
    pub force: bool,
    /// Return as soon as the rollout is requested (`--no-wait`).
    pub no_wait: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeployOutcome {
    /// Full image reference that was deployed.
    pub image: String,
    /// Where the service is reachable, when the adapter can tell.
    pub url: Option<String>,
    /// A command the user can run to inspect the rollout.
    pub inspect: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetStatus {
    pub target: Target,
    pub healthy: bool,
    /// Short human summary, e.g. `3/3 replicas ready`.
    pub summary: String,
    pub image: Option<String>,
    pub ready: Option<u32>,
    pub desired: Option<u32>,
    pub since: Option<String>,
    /// Raw JSON from the underlying tool, for `--json`.
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct LogOptions {
    pub follow: bool,
    /// e.g. `1h`, `30m`, or an RFC3339 timestamp.
    pub since: Option<String>,
    /// Maximum lines from the tail.
    pub limit: Option<usize>,
    pub container: Option<String>,
    /// Ask the tool for timestamps.
    pub timestamps: bool,
}

#[derive(Debug, Clone)]
pub enum RollbackTo<'a> {
    /// Whatever the target considers the previous revision.
    Previous,
    /// A specific recorded deployment.
    Deployment(&'a Deployment),
}

pub trait Adapter {
    fn target(&self) -> Target;

    /// Verify required tools exist and the target is reachable. Each entry is
    /// `(what, ok, detail)`.
    fn preflight(&self, r: &mut dyn Reporter) -> Result<Vec<(String, bool, String)>>;

    fn deploy(&self, req: &DeployRequest, r: &mut dyn Reporter) -> Result<DeployOutcome>;

    fn status(&self) -> Result<TargetStatus>;

    /// A ready-to-spawn command that streams logs to the inherited stdout.
    fn logs_command(&self, opts: &LogOptions) -> Result<Command>;

    fn rollback(&self, to: RollbackTo<'_>, r: &mut dyn Reporter) -> Result<()>;

    /// Runtime environment currently configured on the target.
    fn env(&self) -> Result<Vec<(String, String)>>;

    /// A short description of where this adapter points, for `gozo status`.
    fn describe(&self) -> String;

    /// How long to wait for a rollout (deploy or rollback) before giving up.
    /// Adapters that cannot wait ignore it.
    fn set_timeout(&mut self, _secs: u64) {}
}

/// Vercel-style age: `45s`, `3m`, `2h`, `4d`.
pub fn age(from: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = (now - from).num_seconds().max(0);
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86_400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86_400)
    }
}

/// `age` relative to now.
pub fn age_now(from: DateTime<Utc>) -> String {
    age(from, Utc::now())
}

/// Build the adapter for a linked directory. `config` supplies defaults when
/// the link leaves fields empty.
pub fn adapter_for(link: &Link, config: &Config, root: &Path) -> Result<Box<dyn Adapter>> {
    match link.target {
        Target::Docker => {
            let cfg = link
                .docker
                .clone()
                .unwrap_or_else(|| config.deploy.docker.clone());
            Ok(Box::new(docker::DockerAdapter::new(
                root,
                &link.project_name,
                cfg,
            )))
        }
        Target::Kubernetes => {
            let k = link
                .kubernetes
                .clone()
                .unwrap_or_else(|| config.deploy.kubernetes.clone());
            let d = link
                .docker
                .clone()
                .unwrap_or_else(|| config.deploy.docker.clone());
            Ok(Box::new(kubernetes::KubernetesAdapter::new(
                root,
                &link.project_name,
                k,
                d,
            )))
        }
    }
}

/// Run a command, capturing output. Shared by adapters.
pub(crate) fn run_capture(mut cmd: Command, r: &mut dyn Reporter) -> Result<String> {
    let display = format_command(&cmd);
    r.command(&display);
    let out = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DeployError::Other(format!("could not run `{display}`: {e}"))
        } else {
            DeployError::Io(e)
        }
    })?;
    if !out.status.success() {
        return Err(DeployError::CommandFailed {
            cmd: display,
            status: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run a command with inherited stdio (for long builds when `--logs` is set).
pub(crate) fn run_inherit(mut cmd: Command, r: &mut dyn Reporter) -> Result<()> {
    let display = format_command(&cmd);
    r.command(&display);
    let status = cmd.status()?;
    if !status.success() {
        return Err(DeployError::CommandFailed {
            cmd: display,
            status: status.code().unwrap_or(-1),
            stderr: String::new(),
        });
    }
    Ok(())
}

pub(crate) fn format_command(cmd: &Command) -> String {
    let mut parts = vec![cmd.get_program().to_string_lossy().into_owned()];
    parts.extend(cmd.get_args().map(|a| {
        let s = a.to_string_lossy();
        if s.contains(' ') {
            format!("{s:?}")
        } else {
            s.into_owned()
        }
    }));
    parts.join(" ")
}

pub(crate) fn which(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(name).is_file()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn age_buckets() {
        let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
        let at = |secs: i64| now - chrono::Duration::seconds(secs);
        assert_eq!(age(at(0), now), "0s");
        assert_eq!(age(at(59), now), "59s");
        assert_eq!(age(at(60), now), "1m");
        assert_eq!(age(at(3599), now), "59m");
        assert_eq!(age(at(3600), now), "1h");
        assert_eq!(age(at(7200 + 30), now), "2h");
        assert_eq!(age(at(86_400 * 4 + 5), now), "4d");
        // clock skew never yields a negative age
        assert_eq!(age(now + chrono::Duration::seconds(10), now), "0s");
    }

    #[test]
    fn format_command_quotes_spaces() {
        let mut c = Command::new("kubectl");
        c.args([
            "annotate",
            "deployment/x",
            "kubernetes.io/change-cause=gozo deploy abc",
        ]);
        assert_eq!(
            format_command(&c),
            "kubectl annotate deployment/x \"kubernetes.io/change-cause=gozo deploy abc\""
        );
    }
}
