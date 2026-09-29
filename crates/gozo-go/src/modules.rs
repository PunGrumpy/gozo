//! `go list -m` and `go mod tidy -diff`.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::command::{Go, GoError};

/// One record from `go list -m -json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModuleInfo {
    pub path: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub main: bool,
    #[serde(default)]
    pub indirect: bool,
    #[serde(default)]
    pub dir: Option<String>,
    #[serde(default)]
    pub go_mod: Option<String>,
    #[serde(default)]
    pub go_version: Option<String>,
    #[serde(default)]
    pub update: Option<ModuleUpdate>,
    #[serde(default)]
    pub replace: Option<Box<ModuleInfo>>,
    #[serde(default)]
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModuleUpdate {
    pub path: String,
    pub version: String,
}

/// Result of `go mod tidy -diff`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TidyStatus {
    /// go.mod and go.sum already match what tidy would write.
    Clean,
    /// tidy would change the files; `diff` is the unified diff.
    Dirty { diff: String },
    /// tidy could not run, typically because the module cache is incomplete
    /// and we deliberately ran offline.
    Unknown { reason: String },
}

impl Go {
    /// `go list -m -json all`, optionally with `-u` to include available updates.
    ///
    /// With `updates = true` this touches the network (via GOPROXY).
    pub fn list_modules(&self, dir: &Path, updates: bool) -> Result<Vec<ModuleInfo>, GoError> {
        let mut args = vec!["list", "-m", "-json"];
        if updates {
            args.push("-u");
        }
        args.push("all");
        self.run_json_stream(dir, args)
    }

    /// `go mod tidy -diff`, run offline so it never blocks on the network.
    pub fn tidy_diff(&self, dir: &Path) -> Result<TidyStatus, GoError> {
        let out = self.run_env(
            dir,
            ["mod", "tidy", "-diff"],
            &[("GOPROXY", "off"), ("GOFLAGS", "-mod=mod")],
        )?;
        Ok(match out.status {
            0 => TidyStatus::Clean,
            // `-diff` exits 1 when there is a diff and prints it to stdout.
            1 if !out.stdout.trim().is_empty() => TidyStatus::Dirty { diff: out.stdout },
            _ => TidyStatus::Unknown {
                reason: first_line(&out.stderr).to_owned(),
            },
        })
    }
}

fn first_line(s: &str) -> &str {
    s.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
}
