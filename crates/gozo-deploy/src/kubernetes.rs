//! Kubernetes adapter: build + push an image, then update a Deployment.

use std::path::{Path, PathBuf};
use std::process::Command;

use gozo_core::{DockerTarget, KubernetesTarget, Target};

use crate::image;
use crate::{
    Adapter, DeployError, DeployOutcome, DeployRequest, LogOptions, Reporter, Result, RollbackTo,
    SilentReporter, TargetStatus, run_capture, run_inherit, which,
};

const DEFAULT_TIMEOUT_SECS: u64 = 300;

pub struct KubernetesAdapter {
    pub root: PathBuf,
    pub project: String,
    pub cfg: KubernetesTarget,
    pub docker: DockerTarget,
    timeout_secs: u64,
}

impl KubernetesAdapter {
    pub fn new(root: &Path, project: &str, cfg: KubernetesTarget, docker: DockerTarget) -> Self {
        KubernetesAdapter {
            root: root.to_path_buf(),
            project: project.to_owned(),
            cfg,
            docker,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    pub fn deployment_name(&self) -> String {
        self.cfg
            .deployment
            .clone()
            .filter(|d| !d.trim().is_empty())
            .unwrap_or_else(|| image::sanitize_repo(&self.project).replace('/', "-"))
    }

    pub fn container_name(&self) -> String {
        self.cfg
            .container
            .clone()
            .filter(|c| !c.trim().is_empty())
            .unwrap_or_else(|| self.deployment_name())
    }

    pub fn repository(&self) -> String {
        image::repository(
            self.cfg
                .image
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .or(self.docker.image.as_deref()),
            &self.project,
        )
    }

    /// `kubectl` with `--context` and `-n` applied.
    fn kubectl(&self) -> Command {
        let mut cmd = Command::new("kubectl");
        cmd.args(self.scope_args());
        cmd
    }

    fn scope_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(c) = &self.cfg.context {
            args.push("--context".to_owned());
            args.push(c.clone());
        }
        if let Some(n) = &self.cfg.namespace {
            args.push("-n".to_owned());
            args.push(n.clone());
        }
        args
    }

    fn scope_description(&self) -> String {
        let mut parts = Vec::new();
        if let Some(n) = &self.cfg.namespace {
            parts.push(format!("namespace {n}"));
        }
        if let Some(c) = &self.cfg.context {
            parts.push(format!("context {c}"));
        }
        if parts.is_empty() {
            "current context".to_owned()
        } else {
            parts.join(", ")
        }
    }

    fn deployment_ref(&self) -> String {
        format!("deployment/{}", self.deployment_name())
    }

    fn inspect_hint(&self) -> String {
        let mut parts = vec!["kubectl".to_owned()];
        parts.extend(self.scope_args());
        parts.push("rollout".to_owned());
        parts.push("status".to_owned());
        parts.push(self.deployment_ref());
        parts.join(" ")
    }

    /// `kubectl get deployment NAME -o json`, or `None` when it does not exist.
    fn get_deployment(&self, r: &mut dyn Reporter) -> Result<Option<String>> {
        let mut cmd = self.kubectl();
        cmd.args(["get", &self.deployment_ref(), "-o", "json"]);
        match run_capture(cmd, r) {
            Ok(json) => Ok(Some(json)),
            Err(DeployError::CommandFailed { stderr, .. }) if stderr.contains("NotFound") => {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    fn set_image(&self, image_ref: &str, dry_run: bool, r: &mut dyn Reporter) -> Result<()> {
        r.step(&format!("Setting image on {}", self.deployment_ref()));
        let mut cmd = self.kubectl();
        cmd.args(["set", "image", &self.deployment_ref()])
            .arg(format!("{}={image_ref}", self.container_name()));
        if dry_run {
            cmd.arg("--dry-run=client");
        }
        run_capture(cmd, r).map(|_| ())
    }

    fn annotate(&self, cause: &str, dry_run: bool, r: &mut dyn Reporter) -> Result<()> {
        let mut cmd = self.kubectl();
        cmd.args(["annotate", &self.deployment_ref()])
            .arg(format!("kubernetes.io/change-cause={cause}"))
            .arg("--overwrite");
        if dry_run {
            cmd.arg("--dry-run=client");
        }
        run_capture(cmd, r).map(|_| ())
    }

    fn rollout_status(&self, verbose: bool, r: &mut dyn Reporter) -> Result<()> {
        r.step(&format!(
            "Waiting for rollout (timeout {}s)",
            self.timeout_secs
        ));
        let mut cmd = self.kubectl();
        cmd.args(["rollout", "status", &self.deployment_ref()])
            .arg(format!("--timeout={}s", self.timeout_secs));
        if verbose {
            run_inherit(cmd, r)
        } else {
            run_capture(cmd, r).map(|_| ())
        }
    }
}

impl Adapter for KubernetesAdapter {
    fn target(&self) -> Target {
        Target::Kubernetes
    }

    fn set_timeout(&mut self, secs: u64) {
        self.timeout_secs = secs.max(1);
    }

    fn preflight(&self, r: &mut dyn Reporter) -> Result<Vec<(String, bool, String)>> {
        let mut checks = Vec::new();
        if !which("kubectl") {
            checks.push((
                "kubectl".to_owned(),
                false,
                "not found on PATH; see https://kubernetes.io/docs/tasks/tools/".to_owned(),
            ));
        } else {
            let mut cmd = Command::new("kubectl");
            cmd.args(["version", "--client", "-o", "json"]);
            match image::probe(cmd, r) {
                Ok(json) => checks.push(("kubectl".to_owned(), true, client_version(&json))),
                Err(e) => checks.push(("kubectl".to_owned(), false, e)),
            }
        }
        checks.extend(image::docker_checks(r));
        let (ok, detail) = image::dockerfile_summary(&self.root, &self.docker);
        checks.push(("Dockerfile".to_owned(), ok, detail));

        let kubectl_ok = checks.iter().any(|(what, ok, _)| what == "kubectl" && *ok);
        let what = format!("deployment {}", self.deployment_name());
        if kubectl_ok {
            let mut cmd = self.kubectl();
            cmd.args(["get", &self.deployment_ref(), "-o", "name"]);
            match image::probe(cmd, r) {
                Ok(_) => checks.push((what, true, self.scope_description())),
                Err(e) => checks.push((what, false, e)),
            }
        } else {
            checks.push((what, false, "skipped: kubectl unavailable".to_owned()));
        }
        Ok(checks)
    }

    fn deploy(&self, req: &DeployRequest, r: &mut dyn Reporter) -> Result<DeployOutcome> {
        if !which("kubectl") {
            return Err(DeployError::ToolMissing { tool: "kubectl" });
        }
        let repo = self.repository();
        if !image::has_registry(&repo) {
            r.warn(&format!(
                "image {repo} has no registry prefix; the cluster may not be able to pull it \
                 (set deploy.kubernetes.image, e.g. ghcr.io/you/{repo})"
            ));
        }
        let image_ref = image::image_ref(&repo, &req.tag);
        image::build(
            &self.root,
            &self.project,
            &self.docker,
            &image_ref,
            req,
            true,
            r,
        )?;

        if let Some(dir) = &self.cfg.manifests {
            let path = self.root.join(dir);
            if !path.exists() {
                return Err(DeployError::Config(format!(
                    "manifests path {} does not exist",
                    path.display()
                )));
            }
            r.step(&format!("Applying manifests from {dir}"));
            let mut cmd = self.kubectl();
            cmd.arg("apply").arg("-f").arg(&path);
            if req.dry_run {
                cmd.arg("--dry-run=client");
            }
            let out = run_capture(cmd, r)?;
            for line in out.lines().filter(|l| !l.trim().is_empty()) {
                r.detail(line);
            }
        }

        self.set_image(&image_ref, req.dry_run, r)?;
        let sha = req.git_sha.as_deref().unwrap_or("no git");
        self.annotate(&format!("gozo deploy {} ({sha})", req.tag), req.dry_run, r)?;

        if req.dry_run {
            r.detail("dry run: rollout not applied");
        } else if req.no_wait {
            r.detail("not waiting for rollout (--no-wait)");
        } else {
            self.rollout_status(req.verbose, r)?;
        }

        Ok(DeployOutcome {
            image: image_ref,
            url: None,
            inspect: Some(self.inspect_hint()),
        })
    }

    fn status(&self) -> Result<TargetStatus> {
        let mut silent = SilentReporter;
        let Some(json) = self.get_deployment(&mut silent)? else {
            return Ok(TargetStatus {
                target: Target::Kubernetes,
                healthy: false,
                summary: format!(
                    "no deployment named {} ({})",
                    self.deployment_name(),
                    self.scope_description()
                ),
                image: None,
                ready: None,
                desired: None,
                since: None,
                raw: serde_json::Value::Null,
            });
        };
        let info = parse_deployment(&json, &self.container_name())?;
        let healthy = info.desired > 0 && info.ready >= info.desired && info.available;
        let mut summary = format!("{}/{} replicas ready", info.ready, info.desired);
        if let Some(i) = &info.image {
            summary.push_str(&format!(" (image {i})"));
        }
        if !healthy {
            if let Some(reason) = info.problem() {
                summary.push_str(&format!(" - {reason}"));
            }
        }
        Ok(TargetStatus {
            target: Target::Kubernetes,
            healthy,
            summary,
            image: info.image.clone(),
            ready: Some(info.ready),
            desired: Some(info.desired),
            since: info.since.clone(),
            raw: info.raw,
        })
    }

    fn logs_command(&self, opts: &LogOptions) -> Result<Command> {
        let mut cmd = self.kubectl();
        cmd.args(["logs", &self.deployment_ref()]);
        let container = opts
            .container
            .clone()
            .or_else(|| self.cfg.container.clone());
        if let Some(c) = container {
            cmd.args(["-c", &c]);
        }
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
        cmd.arg("--all-containers=false");
        Ok(cmd)
    }

    fn rollback(&self, to: RollbackTo<'_>, r: &mut dyn Reporter) -> Result<()> {
        if !which("kubectl") {
            return Err(DeployError::ToolMissing { tool: "kubectl" });
        }
        match to {
            RollbackTo::Deployment(d) => {
                let image_ref = d.image.clone().ok_or_else(|| {
                    DeployError::Config(format!("deployment {} has no image recorded", d.id))
                })?;
                r.step(&format!("Rolling back to {image_ref}"));
                self.set_image(&image_ref, false, r)?;
                self.annotate(&format!("gozo rollback to {}", d.id), false, r)?;
            }
            RollbackTo::Previous => {
                r.step(&format!(
                    "Undoing last rollout of {}",
                    self.deployment_ref()
                ));
                let mut cmd = self.kubectl();
                cmd.args(["rollout", "undo", &self.deployment_ref()]);
                run_capture(cmd, r)?;
            }
        }
        self.rollout_status(false, r)
    }

    fn env(&self) -> Result<Vec<(String, String)>> {
        let mut silent = SilentReporter;
        match self.get_deployment(&mut silent)? {
            Some(json) => Ok(parse_deployment(&json, &self.container_name())?.env),
            None => Ok(Vec::new()),
        }
    }

    fn describe(&self) -> String {
        let mut s = format!("kubernetes deployment {}", self.deployment_name());
        if let Some(n) = &self.cfg.namespace {
            s.push_str(&format!(" in {n}"));
        }
        if let Some(c) = &self.cfg.context {
            s.push_str(&format!(" ({c})"));
        }
        s
    }
}

/// `v1.31.2` from `kubectl version --client -o json`.
fn client_version(json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v["clientVersion"]["gitVersion"].as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown version".to_owned())
}

/// What `gozo status` and `gozo env` need from a Deployment object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeploymentInfo {
    pub desired: u32,
    pub ready: u32,
    pub available: bool,
    pub image: Option<String>,
    pub since: Option<String>,
    /// `(type, status, message)` from `status.conditions`.
    pub conditions: Vec<(String, String, String)>,
    /// Container env; `valueFrom` entries become `<from secret NAME>` etc.
    pub env: Vec<(String, String)>,
    pub raw: serde_json::Value,
}

impl DeploymentInfo {
    /// The message of the first condition that is not `True`, if any.
    fn problem(&self) -> Option<String> {
        self.conditions
            .iter()
            .find(|(_, status, _)| status != "True")
            .map(|(ty, _, msg)| {
                if msg.is_empty() {
                    format!("{ty} is not true")
                } else {
                    msg.clone()
                }
            })
    }
}

/// Parse `kubectl get deployment -o json`, reading the image and env from
/// the container named `container` (falling back to the first container).
pub(crate) fn parse_deployment(json: &str, container: &str) -> Result<DeploymentInfo> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| DeployError::Other(format!("could not parse kubectl output: {e}")))?;
    let containers = v["spec"]["template"]["spec"]["containers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let c = containers
        .iter()
        .find(|c| c["name"].as_str() == Some(container))
        .or_else(|| containers.first());
    let image = c.and_then(|c| c["image"].as_str().map(str::to_owned));
    let env = c
        .and_then(|c| c["env"].as_array())
        .map(|a| a.iter().filter_map(env_entry).collect())
        .unwrap_or_default();
    let conditions: Vec<(String, String, String)> = v["status"]["conditions"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|c| {
                    (
                        c["type"].as_str().unwrap_or("").to_owned(),
                        c["status"].as_str().unwrap_or("").to_owned(),
                        c["message"].as_str().unwrap_or("").to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let available = conditions
        .iter()
        .find(|(ty, _, _)| ty == "Available")
        .map(|(_, status, _)| status == "True")
        // No conditions yet (brand new object): trust the replica counts.
        .unwrap_or(true);
    let since = v["status"]["conditions"]
        .as_array()
        .and_then(|a| a.iter().find(|c| c["type"].as_str() == Some("Progressing")))
        .and_then(|c| c["lastUpdateTime"].as_str())
        .or_else(|| v["metadata"]["creationTimestamp"].as_str())
        .map(str::to_owned);
    Ok(DeploymentInfo {
        desired: v["spec"]["replicas"].as_u64().unwrap_or(1) as u32,
        ready: v["status"]["readyReplicas"].as_u64().unwrap_or(0) as u32,
        available,
        image,
        since,
        conditions,
        env,
        raw: v,
    })
}

fn env_entry(e: &serde_json::Value) -> Option<(String, String)> {
    let name = e["name"].as_str()?.to_owned();
    if let Some(v) = e["value"].as_str() {
        return Some((name, v.to_owned()));
    }
    let from = &e["valueFrom"];
    let value = if let Some(s) = from["secretKeyRef"]["name"].as_str() {
        format!("<from secret {s}>")
    } else if let Some(c) = from["configMapKeyRef"]["name"].as_str() {
        format!("<from configmap {c}>")
    } else if let Some(f) = from["fieldRef"]["fieldPath"].as_str() {
        format!("<from field {f}>")
    } else if from["resourceFieldRef"].is_object() {
        "<from resource field>".to_owned()
    } else {
        "<from valueFrom>".to_owned()
    };
    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEPLOYMENT: &str = r#"{
      "metadata": {"name": "api", "creationTimestamp": "2026-09-01T00:00:00Z"},
      "spec": {
        "replicas": 3,
        "template": {"spec": {"containers": [
          {"name": "sidecar", "image": "envoy:1.30"},
          {"name": "api", "image": "ghcr.io/acme/api:abc123",
           "env": [
             {"name": "PORT", "value": "8080"},
             {"name": "DB_URL", "valueFrom": {"secretKeyRef": {"name": "db", "key": "url"}}},
             {"name": "LOG", "valueFrom": {"configMapKeyRef": {"name": "cfg", "key": "log"}}},
             {"name": "NODE", "valueFrom": {"fieldRef": {"fieldPath": "spec.nodeName"}}}
           ]}
        ]}}
      },
      "status": {
        "readyReplicas": 3,
        "conditions": [
          {"type": "Available", "status": "True", "message": "Deployment has minimum availability."},
          {"type": "Progressing", "status": "True", "lastUpdateTime": "2026-09-29T09:00:00Z"}
        ]
      }
    }"#;

    #[test]
    fn parses_deployment_json() {
        let info = parse_deployment(DEPLOYMENT, "api").unwrap();
        assert_eq!(info.desired, 3);
        assert_eq!(info.ready, 3);
        assert!(info.available);
        assert_eq!(info.image.as_deref(), Some("ghcr.io/acme/api:abc123"));
        assert_eq!(info.since.as_deref(), Some("2026-09-29T09:00:00Z"));
        assert_eq!(
            info.env,
            vec![
                ("PORT".to_owned(), "8080".to_owned()),
                ("DB_URL".to_owned(), "<from secret db>".to_owned()),
                ("LOG".to_owned(), "<from configmap cfg>".to_owned()),
                ("NODE".to_owned(), "<from field spec.nodeName>".to_owned()),
            ]
        );
        assert!(info.problem().is_none());
        // unknown container name falls back to the first container
        let first = parse_deployment(DEPLOYMENT, "nope").unwrap();
        assert_eq!(first.image.as_deref(), Some("envoy:1.30"));
    }

    #[test]
    fn unhealthy_deployment() {
        let json = r#"{"spec":{"replicas":2,"template":{"spec":{"containers":[{"name":"api","image":"api:v1"}]}}},
            "status":{"readyReplicas":1,"conditions":[{"type":"Available","status":"False","message":"Deployment does not have minimum availability."}]}}"#;
        let info = parse_deployment(json, "api").unwrap();
        assert_eq!((info.ready, info.desired), (1, 2));
        assert!(!info.available);
        assert_eq!(
            info.problem().as_deref(),
            Some("Deployment does not have minimum availability.")
        );
        assert!(info.since.is_none());
        // no status at all: fresh object
        let bare = parse_deployment(r#"{"spec":{}}"#, "api").unwrap();
        assert_eq!((bare.ready, bare.desired), (0, 1));
        assert!(bare.available);
        assert!(bare.image.is_none());
        assert!(parse_deployment("{", "api").is_err());
    }

    #[test]
    fn kubectl_scoping_and_names() {
        let cfg = KubernetesTarget {
            context: Some("prod".into()),
            namespace: Some("apps".into()),
            deployment: None,
            container: Some("web".into()),
            manifests: None,
            image: None,
        };
        let docker = DockerTarget {
            image: Some("ghcr.io/acme/api".into()),
            ..Default::default()
        };
        let a = KubernetesAdapter::new(Path::new("/tmp/x"), "My Svc", cfg, docker);
        assert_eq!(a.deployment_name(), "my-svc");
        assert_eq!(a.container_name(), "web");
        assert_eq!(a.repository(), "ghcr.io/acme/api");
        assert_eq!(
            a.inspect_hint(),
            "kubectl --context prod -n apps rollout status deployment/my-svc"
        );
        assert_eq!(a.describe(), "kubernetes deployment my-svc in apps (prod)");
        let cmd = a
            .logs_command(&LogOptions {
                follow: true,
                since: Some("30m".into()),
                limit: Some(100),
                container: None,
                timestamps: false,
            })
            .unwrap();
        assert_eq!(
            crate::format_command(&cmd),
            "kubectl --context prod -n apps logs deployment/my-svc -c web --follow --since 30m --tail 100 --all-containers=false"
        );
    }

    #[test]
    fn client_version_parsing() {
        assert_eq!(
            client_version(r#"{"clientVersion":{"gitVersion":"v1.31.2"}}"#),
            "v1.31.2"
        );
        assert_eq!(client_version("garbage"), "unknown version");
    }
}
