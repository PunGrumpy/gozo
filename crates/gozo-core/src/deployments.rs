//! Local deployment history in `.gozo/deployments.json`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::envstore::Environment;
use crate::link::Target;
use crate::{CoreError, Result, STATE_DIR};

pub const FILE: &str = "deployments.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deployment {
    /// `dpl_` followed by a short unique suffix.
    pub id: String,
    pub target: Target,
    pub environment: Environment,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    /// `ready`, `error`, `rolled-back`, `canceled`.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meta: BTreeMap<String, String>,
}

impl Deployment {
    pub fn new_id() -> String {
        let nanos = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
        let pid = std::process::id() as u64;
        let mut x = nanos ^ (pid << 32) ^ 0x9e37_79b9_7f4a_7c15;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        format!("dpl_{}", base36(x))
    }
}

fn base36(mut n: u64) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while n > 0 {
        out.push(CHARS[(n % 36) as usize]);
        n /= 36;
    }
    out.reverse();
    let s = String::from_utf8(out).unwrap_or_default();
    s.chars().take(12).collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentHistory {
    pub deployments: Vec<Deployment>,
}

impl DeploymentHistory {
    pub fn path(root: &Path) -> PathBuf {
        root.join(STATE_DIR).join(FILE)
    }

    pub fn load(root: &Path) -> Result<DeploymentHistory> {
        let path = Self::path(root);
        if !path.is_file() {
            return Ok(DeploymentHistory::default());
        }
        let text = std::fs::read_to_string(&path)?;
        serde_json::from_str(&text).map_err(|source| CoreError::Json {
            path: path.display().to_string(),
            source,
        })
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let path = Self::path(root);
        std::fs::create_dir_all(path.parent().unwrap_or(root))?;
        let json = serde_json::to_string_pretty(self).map_err(|source| CoreError::Json {
            path: path.display().to_string(),
            source,
        })?;
        std::fs::write(path, json + "\n")?;
        Ok(())
    }

    /// Newest first.
    pub fn list(&self) -> Vec<&Deployment> {
        let mut v: Vec<&Deployment> = self.deployments.iter().collect();
        v.sort_by_key(|d| std::cmp::Reverse(d.created_at));
        v
    }

    pub fn push(&mut self, d: Deployment) {
        self.deployments.push(d);
    }

    pub fn find(&self, id_or_prefix: &str) -> Option<&Deployment> {
        self.deployments
            .iter()
            .find(|d| d.id == id_or_prefix)
            .or_else(|| {
                self.deployments
                    .iter()
                    .find(|d| d.id.starts_with(id_or_prefix))
            })
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Deployment> {
        self.deployments.iter_mut().find(|d| d.id == id)
    }

    pub fn previous(&self, env: Environment) -> Option<&Deployment> {
        self.list()
            .into_iter()
            .filter(|d| d.environment == env && d.status == "ready")
            .nth(1)
    }
}
