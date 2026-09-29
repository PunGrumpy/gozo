//! Docker adapter: build an image, run it locally as a named container.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Utc};
use gozo_core::{DockerTarget, Target};

use crate::image;
use crate::{
    Adapter, DeployError, DeployOutcome, DeployRequest, LogOptions, Reporter, Result, RollbackTo,
    SilentReporter, TargetStatus, run_capture,
};

const LABEL_PROJECT: &str = "gozo.project";
const LABEL_TAG: &str = "gozo.tag";
const LABEL_ENV: &str = "gozo.environment";
/// Comma-separated runtime env keys gozo passed with `-e`, so a rollback can
/// carry them over without re-passing variables baked into the image.
const LABEL_ENV_KEYS: &str = "gozo.env";

pub struct DockerAdapter {
    pub root: PathBuf,
    pub project: String,
    pub cfg: DockerTarget,
}

impl DockerAdapter {
    pub fn new(root: &Path, project: &str, cfg: DockerTarget) -> Self {
        DockerAdapter {
            root: root.to_path_buf(),
            project: project.to_owned(),
            cfg,
        }
    }

    pub fn container_name(&self) -> String {
        self.cfg
            .container
            .clone()
            .filter(|c| !c.trim().is_empty())
            .unwrap_or_else(|| image::sanitize_repo(&self.project).replace('/', "-"))
    }

    pub fn repository(&self) -> String {
        image::repository(self.cfg.image.as_deref(), &self.project)
    }

    /// `docker inspect NAME`, or `None` when there is no such container.
    fn inspect(&self, r: &mut dyn Reporter) -> Result<Option<ContainerInfo>> {
        let mut cmd = Command::new("docker");
        cmd.args(["inspect", "--type", "container", &self.container_name()]);
        match run_capture(cmd, r) {
            Ok(json) => parse_inspect(&json),
            Err(DeployError::CommandFailed { stderr, .. }) if is_not_found(&stderr) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Stop and remove the container if it exists.
    fn remove_container(&self, r: &mut dyn Reporter) -> Result<bool> {
        let name = self.container_name();
        let mut stop = Command::new("docker");
        stop.args(["stop", &name]);
        match run_capture(stop, r) {
            Ok(_) => {}
            Err(DeployError::CommandFailed { stderr, .. }) if is_not_found(&stderr) => {
                return Ok(false);
            }
            Err(e) => return Err(e),
        }
        let mut rm = Command::new("docker");
        rm.args(["rm", &name]);
        match run_capture(rm, r) {
            Ok(_) => Ok(true),
            Err(DeployError::CommandFailed { stderr, .. }) if is_not_found(&stderr) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn run_container(
        &self,
        image: &str,
        env: &[(String, String)],
        labels: &BTreeMap<String, String>,
        r: &mut dyn Reporter,
    ) -> Result<()> {
        let name = self.container_name();
        r.step(&format!("Starting container {name}"));
        let mut cmd = Command::new("docker");
        cmd.args(["run", "-d", "--name", &name, "--restart", "unless-stopped"]);
        for p in &self.cfg.ports {
            cmd.args(["-p", p]);
        }
        for (k, v) in env {
            cmd.arg("-e").arg(format!("{k}={v}"));
        }
        for (k, v) in labels {
            cmd.arg("--label").arg(format!("{k}={v}"));
        }
        cmd.arg(image);
        run_capture(cmd, r).map(|_| ())
    }

    fn url(&self) -> Option<String> {
        self.cfg
            .ports
            .iter()
            .find_map(|p| host_port(p))
            .map(|port| format!("http://localhost:{port}"))
    }

    fn inspect_hint(&self) -> String {
        format!("docker logs -f {}", self.container_name())
    }

    fn labels(
        &self,
        tag: &str,
        environment: &str,
        env_keys: &[String],
    ) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert(LABEL_PROJECT.to_owned(), self.project.clone());
        labels.insert(LABEL_TAG.to_owned(), tag.to_owned());
        labels.insert(LABEL_ENV.to_owned(), environment.to_owned());
        labels.insert(LABEL_ENV_KEYS.to_owned(), env_keys.join(","));
        labels
    }
}

impl Adapter for DockerAdapter {
    fn target(&self) -> Target {
        Target::Docker
    }

    fn preflight(&self, r: &mut dyn Reporter) -> Result<Vec<(String, bool, String)>> {
        let mut checks = image::docker_checks(r);
        let (ok, detail) = image::dockerfile_summary(&self.root, &self.cfg);
        checks.push(("Dockerfile".to_owned(), ok, detail));
        Ok(checks)
    }

    fn deploy(&self, req: &DeployRequest, r: &mut dyn Reporter) -> Result<DeployOutcome> {
        let image_ref = image::image_ref(&self.repository(), &req.tag);
        image::build(
            &self.root,
            &self.project,
            &self.cfg,
            &image_ref,
            req,
            self.cfg.push,
            r,
        )?;

        let inspect = Some(self.inspect_hint());
        if req.dry_run {
            r.detail(&format!(
                "dry run: not starting container {}",
                self.container_name()
            ));
            return Ok(DeployOutcome {
                image: image_ref,
                url: None,
                inspect,
            });
        }

        if self.remove_container(r)? {
            r.detail(&format!("replaced container {}", self.container_name()));
        }
        let env: Vec<(String, String)> = req
            .runtime_env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let keys: Vec<String> = env.iter().map(|(k, _)| k.clone()).collect();
        let labels = self.labels(&req.tag, req.environment.as_str(), &keys);
        self.run_container(&image_ref, &env, &labels, r)?;

        Ok(DeployOutcome {
            image: image_ref,
            url: self.url(),
            inspect,
        })
    }

    fn status(&self) -> Result<TargetStatus> {
        let mut silent = SilentReporter;
        let info = self.inspect(&mut silent)?;
        Ok(match info {
            None => TargetStatus {
                target: Target::Docker,
                healthy: false,
                summary: format!("no container named {}", self.container_name()),
                image: None,
                ready: Some(0),
                desired: Some(1),
                since: None,
                raw: serde_json::Value::Null,
            },
            Some(info) => {
                let image_note = info
                    .image
                    .as_deref()
                    .map(|i| format!(" (image {i})"))
                    .unwrap_or_default();
                let summary = if info.running {
                    let since = info
                        .started_at
                        .as_deref()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|t| crate::age_now(t.with_timezone(&Utc)))
                        .unwrap_or_else(|| "?".to_owned());
                    let restarts = if info.restart_count > 0 {
                        format!(", {} restarts", info.restart_count)
                    } else {
                        String::new()
                    };
                    format!("running since {since}{restarts}{image_note}")
                } else {
                    format!("not running ({}){image_note}", info.status)
                };
                TargetStatus {
                    target: Target::Docker,
                    healthy: info.running,
                    summary,
                    image: info.image.clone(),
                    ready: Some(u32::from(info.running)),
                    desired: Some(1),
                    since: info.started_at.clone(),
                    raw: info.raw,
                }
            }
        })
    }

    fn logs_command(&self, opts: &LogOptions) -> Result<Command> {
        let mut cmd = Command::new("docker");
        cmd.arg("logs");
        if opts.follow {
            cmd.arg("--follow");
        }
        if let Some(s) = &opts.since {
            cmd.args(["--since", s]);
        }
        if let Some(n) = opts.limit {
            cmd.arg("--tail").arg(n.to_string());
        }
        if opts.timestamps {
            cmd.arg("--timestamps");
        }
        cmd.arg(self.container_name());
        Ok(cmd)
    }

    fn rollback(&self, to: RollbackTo<'_>, r: &mut dyn Reporter) -> Result<()> {
        let d = match to {
            RollbackTo::Deployment(d) => d,
            RollbackTo::Previous => {
                return Err(DeployError::Other(
                    "docker keeps no revision history; pass a deployment id from `gozo ls`".into(),
                ));
            }
        };
        let image_ref = d.image.clone().ok_or_else(|| {
            DeployError::Config(format!("deployment {} has no image recorded", d.id))
        })?;

        // Carry over the runtime variables gozo passed to the current container.
        let current = self.inspect(r)?;
        let (env, environment) = match &current {
            Some(c) => {
                let keys: Vec<&str> = c
                    .labels
                    .get(LABEL_ENV_KEYS)
                    .map(|s| s.split(',').filter(|k| !k.is_empty()).collect())
                    .unwrap_or_default();
                let env = c
                    .env
                    .iter()
                    .filter(|(k, _)| keys.contains(&k.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                let environment = c
                    .labels
                    .get(LABEL_ENV)
                    .cloned()
                    .unwrap_or_else(|| d.environment.as_str().to_owned());
                (env, environment)
            }
            None => (Vec::new(), d.environment.as_str().to_owned()),
        };
        let keys: Vec<String> = env.iter().map(|(k, _)| k.clone()).collect();
        let tag = image_ref.rsplit(':').next().unwrap_or("").to_owned();
        let labels = self.labels(&tag, &environment, &keys);

        r.step(&format!("Rolling back to {image_ref}"));
        self.remove_container(r)?;
        self.run_container(&image_ref, &env, &labels, r)
    }

    fn env(&self) -> Result<Vec<(String, String)>> {
        let mut silent = SilentReporter;
        Ok(self
            .inspect(&mut silent)?
            .map(|c| c.env)
            .unwrap_or_default())
    }

    fn describe(&self) -> String {
        format!("docker container {}", self.container_name())
    }
}

fn is_not_found(stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    s.contains("no such container") || s.contains("no such object")
}

/// Host port of a `docker run -p` mapping: `8080:80` -> 8080,
/// `127.0.0.1:8080:80` -> 8080, `80` (random host port) -> None.
pub(crate) fn host_port(spec: &str) -> Option<u16> {
    let spec = spec.split('/').next().unwrap_or(spec);
    let parts: Vec<&str> = spec.split(':').collect();
    let host = match parts.as_slice() {
        [_, host, _] => host,
        [host, _] => host,
        _ => return None,
    };
    host.parse().ok()
}

/// What `gozo status` needs from `docker inspect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContainerInfo {
    pub status: String,
    pub running: bool,
    pub started_at: Option<String>,
    pub image: Option<String>,
    pub restart_count: u64,
    pub env: Vec<(String, String)>,
    pub labels: BTreeMap<String, String>,
    pub raw: serde_json::Value,
}

/// Parse `docker inspect` output (a JSON array). `None` when it is empty.
pub(crate) fn parse_inspect(json: &str) -> Result<Option<ContainerInfo>> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| DeployError::Other(format!("could not parse docker inspect output: {e}")))?;
    let Some(c) = value.as_array().and_then(|a| a.first()) else {
        return Ok(None);
    };
    let state = &c["State"];
    let env = c["Config"]["Env"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|kv| match kv.split_once('=') {
                    Some((k, v)) => (k.to_owned(), v.to_owned()),
                    None => (kv.to_owned(), String::new()),
                })
                .collect()
        })
        .unwrap_or_default();
    let labels = c["Config"]["Labels"]
        .as_object()
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    Ok(Some(ContainerInfo {
        status: state["Status"].as_str().unwrap_or("unknown").to_owned(),
        running: state["Running"].as_bool().unwrap_or(false),
        started_at: state["StartedAt"]
            .as_str()
            .filter(|s| !s.starts_with("0001-"))
            .map(str::to_owned),
        image: c["Config"]["Image"].as_str().map(str::to_owned),
        restart_count: c["RestartCount"].as_u64().unwrap_or(0),
        env,
        labels,
        raw: c.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSPECT: &str = r#"[{
      "Id": "abc",
      "RestartCount": 2,
      "State": {"Status": "running", "Running": true, "StartedAt": "2026-09-29T10:00:00.123456789Z"},
      "Config": {
        "Image": "hello:abc123",
        "Env": ["PATH=/usr/bin", "PORT=8080", "EMPTY", "URL=http://x?a=b=c"],
        "Labels": {"gozo.project": "hello", "gozo.env": "PORT,URL", "gozo.environment": "preview"}
      }
    }]"#;

    #[test]
    fn parses_inspect_output() {
        let info = parse_inspect(INSPECT).unwrap().unwrap();
        assert_eq!(info.status, "running");
        assert!(info.running);
        assert_eq!(
            info.started_at.as_deref(),
            Some("2026-09-29T10:00:00.123456789Z")
        );
        assert_eq!(info.image.as_deref(), Some("hello:abc123"));
        assert_eq!(info.restart_count, 2);
        assert_eq!(
            info.env,
            vec![
                ("PATH".to_owned(), "/usr/bin".to_owned()),
                ("PORT".to_owned(), "8080".to_owned()),
                ("EMPTY".to_owned(), String::new()),
                ("URL".to_owned(), "http://x?a=b=c".to_owned()),
            ]
        );
        assert_eq!(
            info.labels.get("gozo.env").map(String::as_str),
            Some("PORT,URL")
        );
    }

    #[test]
    fn inspect_edge_cases() {
        assert!(parse_inspect("[]").unwrap().is_none());
        assert!(parse_inspect("not json").is_err());
        let stopped = r#"[{"State":{"Status":"exited","Running":false,"StartedAt":"0001-01-01T00:00:00Z"},"Config":{}}]"#;
        let info = parse_inspect(stopped).unwrap().unwrap();
        assert!(!info.running);
        assert_eq!(info.status, "exited");
        assert!(info.started_at.is_none());
        assert!(info.image.is_none());
        assert!(info.env.is_empty());
    }

    #[test]
    fn host_port_forms() {
        assert_eq!(host_port("18080:8080"), Some(18080));
        assert_eq!(host_port("127.0.0.1:9000:8080"), Some(9000));
        assert_eq!(host_port("8080:8080/tcp"), Some(8080));
        assert_eq!(host_port("8080"), None);
        assert_eq!(host_port("abc:80"), None);
    }

    #[test]
    fn container_name_and_logs_command() {
        let mut cfg = DockerTarget {
            ports: vec!["18080:8080".into()],
            ..Default::default()
        };
        let a = DockerAdapter::new(Path::new("/tmp/x"), "My Svc", cfg.clone());
        assert_eq!(a.container_name(), "my-svc");
        assert_eq!(a.repository(), "my-svc");
        assert_eq!(a.url().as_deref(), Some("http://localhost:18080"));
        assert_eq!(a.describe(), "docker container my-svc");

        cfg.container = Some("api".into());
        cfg.image = Some("ghcr.io/acme/api".into());
        let a = DockerAdapter::new(Path::new("/tmp/x"), "svc", cfg);
        let cmd = a
            .logs_command(&LogOptions {
                follow: true,
                since: Some("1h".into()),
                limit: Some(50),
                container: None,
                timestamps: true,
            })
            .unwrap();
        assert_eq!(
            crate::format_command(&cmd),
            "docker logs --follow --since 1h --tail 50 --timestamps api"
        );
        assert_eq!(a.repository(), "ghcr.io/acme/api");
    }

    #[test]
    fn not_found_detection() {
        assert!(is_not_found(
            "Error response from daemon: No such container: api"
        ));
        assert!(is_not_found("Error: No such object: api"));
        assert!(!is_not_found("Cannot connect to the Docker daemon"));
    }
}
