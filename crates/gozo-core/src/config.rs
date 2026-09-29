//! `gozo.toml`, the shared, committed project configuration.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{CoreError, Result};

pub const FILE: &str = "gozo.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub project: ProjectSection,
    pub dev: DevSection,
    pub build: BuildSection,
    pub tasks: BTreeMap<String, Task>,
    pub deploy: DeploySection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectSection {
    /// Human name. Defaults to the last element of the root module path.
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DevSection {
    /// Main package to run, e.g. `./cmd/api`. Auto-detected when absent.
    pub cmd: Option<String>,
    /// Extra arguments passed to the program.
    pub args: Vec<String>,
    /// Port the app listens on, used for the "ready" message. `PORT` is set.
    pub port: Option<u16>,
    /// Env files to load, in order; later files win. Defaults to
    /// `.env`, `.env.local`, `.env.development`, `.env.development.local`.
    pub env: Vec<String>,
    /// Glob patterns that trigger a restart. Defaults to Go sources and go.mod.
    pub watch: Vec<String>,
    /// Glob patterns that never trigger a restart.
    pub ignore: Vec<String>,
    /// Docker Compose file to bring up before starting (`docker compose up -d`).
    pub services: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BuildSection {
    /// Main packages to build. Defaults to every `main` package under `cmd/`.
    pub cmds: Vec<String>,
    /// Output directory.
    pub output: String,
    /// Extra `-ldflags`. gozo always injects `main.version`, `main.commit`, `main.date`.
    pub ldflags: Option<String>,
    pub trimpath: bool,
    /// Force CGO on or off. Unset leaves the environment alone.
    pub cgo: Option<bool>,
    /// Cross-compile targets as `GOOS/GOARCH`. Empty means the host.
    pub targets: Vec<String>,
}

impl Default for BuildSection {
    fn default() -> Self {
        BuildSection {
            cmds: Vec::new(),
            output: "dist".to_owned(),
            ldflags: None,
            trimpath: true,
            cgo: None,
            targets: Vec::new(),
        }
    }
}

/// A task is either a bare command string or a table with options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Task {
    Command(String),
    Full(TaskDef),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TaskDef {
    pub cmd: String,
    pub description: Option<String>,
    /// Working directory relative to the project root.
    pub cwd: Option<String>,
    pub env: BTreeMap<String, String>,
    /// Tasks to run first, in order.
    pub deps: Vec<String>,
}

impl Task {
    pub fn def(&self) -> TaskDef {
        match self {
            Task::Command(cmd) => TaskDef {
                cmd: cmd.clone(),
                ..Default::default()
            },
            Task::Full(d) => d.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DeploySection {
    /// Default target when the directory is not linked: `docker` or `kubernetes`.
    pub target: Option<String>,
    pub docker: DockerTarget,
    pub kubernetes: KubernetesTarget,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DockerTarget {
    /// Image repository, e.g. `ghcr.io/acme/api`. Defaults to the project name.
    pub image: Option<String>,
    /// Dockerfile path. When absent and no Dockerfile exists, gozo generates one.
    pub dockerfile: Option<String>,
    /// Build context, defaults to the project root.
    pub context: Option<String>,
    /// e.g. `linux/amd64`.
    pub platform: Option<String>,
    /// Push after build.
    pub push: bool,
    /// For local `docker run` deployments: container name. Defaults to project name.
    pub container: Option<String>,
    /// Port mappings for local runs, e.g. `8080:8080`.
    pub ports: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct KubernetesTarget {
    pub context: Option<String>,
    pub namespace: Option<String>,
    /// Deployment name. Defaults to the project name.
    pub deployment: Option<String>,
    /// Container name inside the pod. Defaults to the deployment name.
    pub container: Option<String>,
    /// Directory of manifests to `kubectl apply -f` before setting the image.
    pub manifests: Option<String>,
    /// Image repository to push to. Falls back to `deploy.docker.image`.
    pub image: Option<String>,
}

impl Config {
    pub fn path(root: &Path) -> PathBuf {
        root.join(FILE)
    }

    /// Load `gozo.toml` if present.
    pub fn load(root: &Path) -> Result<Option<Config>> {
        let path = Self::path(root);
        if !path.is_file() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text)
            .map(Some)
            .map_err(|source| CoreError::Toml {
                path: path.display().to_string(),
                source,
            })
    }

    /// Whether `gozo.toml` exists.
    pub fn exists(root: &Path) -> bool {
        Self::path(root).is_file()
    }

    /// A commented starter file for `gozo init`.
    pub fn template(name: &str, cmd: Option<&str>) -> String {
        let cmd_line = match cmd {
            Some(c) => format!("cmd = \"{c}\""),
            None => "# cmd = \"./cmd/api\"".to_owned(),
        };
        format!(
            r#"# gozo project configuration. Commit this file.
# Machine-specific link data lives in .gozo/ and is ignored by git.

[project]
name = "{name}"

[dev]
# Main package to run. Auto-detected from ./cmd when omitted.
{cmd_line}
# port = 8080
# env = [".env", ".env.local"]
# services = "docker-compose.yml"

[build]
output = "dist"
trimpath = true
# targets = ["linux/amd64", "darwin/arm64"]

[tasks]
lint = "golangci-lint run ./..."
fmt = "gofmt -l -w ."

[deploy]
# target = "docker"   # or "kubernetes"

[deploy.docker]
# image = "ghcr.io/you/{name}"
# platform = "linux/amd64"
# push = true

[deploy.kubernetes]
# context = "prod"
# namespace = "default"
# deployment = "{name}"
"#
        )
    }
}
