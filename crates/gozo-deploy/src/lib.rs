//! Deployment target adapters: the deploy verbs as concrete `docker` / `kubectl` invocations.

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

pub trait Reporter {
    fn step(&mut self, msg: &str);
    /// A command about to run; only shown in verbose mode.
    fn command(&mut self, cmd: &str);
    fn detail(&mut self, msg: &str);
    fn warn(&mut self, msg: &str);
}

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
    pub tag: String,
    pub git_sha: Option<String>,
    pub git_branch: Option<String>,
    pub build_env: BTreeMap<String, String>,
    /// `--env` merged over the env store.
    pub runtime_env: BTreeMap<String, String>,
    /// Stream the underlying tool output (`--logs`).
    pub verbose: bool,
    pub dry_run: bool,
    /// Skip the build cache.
    pub force: bool,
    pub no_wait: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeployOutcome {
    pub image: String,
    pub url: Option<String>,
    pub inspect: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetStatus {
    pub target: Target,
    pub healthy: bool,
    /// e.g. `3/3 replicas ready`.
    pub summary: String,
    pub image: Option<String>,
    pub ready: Option<u32>,
    pub desired: Option<u32>,
    pub since: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct LogOptions {
    pub follow: bool,
    /// e.g. `1h`, `30m`, or an RFC3339 timestamp.
    pub since: Option<String>,
    pub limit: Option<usize>,
    pub container: Option<String>,
    pub timestamps: bool,
}

#[derive(Debug, Clone)]
pub enum RollbackTo<'a> {
    Previous,
    Deployment(&'a Deployment),
}

pub trait Adapter {
    fn target(&self) -> Target;

    /// Each entry is `(what, ok, detail)`.
    fn preflight(&self, r: &mut dyn Reporter) -> Result<Vec<(String, bool, String)>>;

    fn deploy(&self, req: &DeployRequest, r: &mut dyn Reporter) -> Result<DeployOutcome>;

    fn status(&self) -> Result<TargetStatus>;

    fn logs_command(&self, opts: &LogOptions) -> Result<Command>;

    fn rollback(&self, to: RollbackTo<'_>, r: &mut dyn Reporter) -> Result<()>;

    fn env(&self) -> Result<Vec<(String, String)>>;

    fn describe(&self) -> String;

    /// Rollout wait limit; adapters that cannot wait ignore it.
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

pub fn adapter_for(link: &Link, config: &Config, root: &Path) -> Result<Box<dyn Adapter>> {
    let docker = link
        .docker
        .clone()
        .unwrap_or_else(|| config.deploy.docker.clone());
    match link.target {
        Target::Docker => Ok(Box::new(docker::DockerAdapter::new(
            root,
            &link.project_name,
            docker,
        ))),
        Target::Kubernetes => {
            let k = link
                .kubernetes
                .clone()
                .unwrap_or_else(|| config.deploy.kubernetes.clone());
            Ok(Box::new(kubernetes::KubernetesAdapter::new(
                root,
                &link.project_name,
                k,
                docker,
            )))
        }
    }
}

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
