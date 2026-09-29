//! `.gozo/project.json`: machine-local, uncommitted binding of a directory to a deployment target.

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::{DockerTarget, KubernetesTarget};
use crate::{CoreError, Result, STATE_DIR};

pub const FILE: &str = "project.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Docker,
    Kubernetes,
}

impl Target {
    pub const ALL: [Target; 2] = [Target::Docker, Target::Kubernetes];

    pub fn as_str(&self) -> &'static str {
        match self {
            Target::Docker => "docker",
            Target::Kubernetes => "kubernetes",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Target::Docker => "build an image and run it with Docker on this machine",
            Target::Kubernetes => "build, push and roll out to a Kubernetes deployment",
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Target {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "docker" => Ok(Target::Docker),
            "kubernetes" | "k8s" | "kube" => Ok(Target::Kubernetes),
            other => Err(CoreError::Other(format!(
                "unknown deployment target {other:?} (expected docker or kubernetes)"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub project_name: String,
    pub target: Target,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kubernetes: Option<KubernetesTarget>,
    pub linked_at: DateTime<Utc>,
}

impl Link {
    pub fn dir(root: &Path) -> PathBuf {
        root.join(STATE_DIR)
    }

    pub fn path(root: &Path) -> PathBuf {
        Self::dir(root).join(FILE)
    }

    pub fn load(root: &Path) -> Result<Option<Link>> {
        let path = Self::path(root);
        if !path.is_file() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path)?;
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|source| CoreError::Json {
                path: path.display().to_string(),
                source,
            })
    }

    /// Also writes `.gozo/README.txt` and gitignores `.gozo` in git repositories.
    pub fn save(&self, root: &Path) -> Result<()> {
        let dir = Self::dir(root);
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self).map_err(|source| CoreError::Json {
            path: Self::path(root).display().to_string(),
            source,
        })?;
        std::fs::write(Self::path(root), json + "\n")?;
        std::fs::write(dir.join("README.txt"), README)?;
        ensure_gitignore(root)?;
        Ok(())
    }

    pub fn remove(root: &Path) -> Result<bool> {
        let path = Self::path(root);
        if path.is_file() {
            std::fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

const README: &str = "> Why do I have a folder named \".gozo\" in my project?
The \".gozo\" folder is created when you link a directory to a deployment
target with `gozo link`.

> What does the \"project.json\" file contain?
The \"project.json\" file records the project name and the deployment target
(docker or kubernetes) this directory is linked to, so commands like
`gozo deploy`, `gozo logs` and `gozo status` know where to look.

> What about \"env/\" and \"deployments.json\"?
\"env/\" holds per-environment variables managed by `gozo env`.
\"deployments.json\" is the local deployment history used by `gozo rollback`.

> Should I commit the \".gozo\" folder?
No, you should not commit the \".gozo\" folder. It may contain secrets and
machine-specific settings. It is added to \".gitignore\" automatically.
";

pub fn ensure_gitignore(root: &Path) -> Result<bool> {
    if !root.join(".git").exists() {
        return Ok(false);
    }
    ensure_gitignore_entry(root, STATE_DIR)
}

/// Matches `entry` with or without a leading or trailing `/`.
pub fn gitignore_has(root: &Path, entry: &str) -> bool {
    let existing = std::fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    let entry = entry.trim_matches('/');
    existing
        .lines()
        .map(str::trim)
        .any(|l| l.trim_start_matches('/').trim_end_matches('/') == entry)
}

/// Returns whether `.gitignore` was changed.
pub fn ensure_gitignore_entry(root: &Path, entry: &str) -> Result<bool> {
    if gitignore_has(root, entry) {
        return Ok(false);
    }
    let path = root.join(".gitignore");
    let mut out = std::fs::read_to_string(&path).unwrap_or_default();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(entry);
    out.push('\n');
    std::fs::write(path, out)?;
    Ok(true)
}
