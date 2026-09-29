//! Per-environment variables stored under `.gozo/env/<environment>.env`.
//!
//! Environments follow Vercel's naming: `development`, `preview`, `production`.

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{CoreError, Result, STATE_DIR, dotenv};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Preview,
    Production,
}

impl Environment {
    pub const ALL: [Environment; 3] = [
        Environment::Development,
        Environment::Preview,
        Environment::Production,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Preview => "preview",
            Environment::Production => "production",
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Environment {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "preview" | "staging" => Ok(Environment::Preview),
            "production" | "prod" => Ok(Environment::Production),
            other => Err(CoreError::Other(format!(
                "unknown environment {other:?} (expected development, preview or production)"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvStore {
    root: PathBuf,
}

impl EnvStore {
    pub fn new(root: &Path) -> Self {
        EnvStore {
            root: root.to_path_buf(),
        }
    }

    pub fn dir(&self) -> PathBuf {
        self.root.join(STATE_DIR).join("env")
    }

    pub fn path(&self, env: Environment) -> PathBuf {
        self.dir().join(format!("{env}.env"))
    }

    pub fn list(&self, env: Environment) -> Result<Vec<(String, String)>> {
        Ok(dotenv::load(&self.path(env))?)
    }

    pub fn get(&self, env: Environment, key: &str) -> Result<Option<String>> {
        Ok(self
            .list(env)?
            .into_iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v))
    }

    /// Set a key, returning whether it already existed.
    pub fn set(&self, env: Environment, key: &str, value: &str) -> Result<bool> {
        let mut vars = self.list(env)?;
        let existed = if let Some(slot) = vars.iter_mut().find(|(k, _)| k == key) {
            slot.1 = value.to_owned();
            true
        } else {
            vars.push((key.to_owned(), value.to_owned()));
            false
        };
        self.write(env, &vars)?;
        Ok(existed)
    }

    /// Remove a key, returning whether it existed.
    pub fn remove(&self, env: Environment, key: &str) -> Result<bool> {
        let mut vars = self.list(env)?;
        let before = vars.len();
        vars.retain(|(k, _)| k != key);
        if vars.len() == before {
            return Ok(false);
        }
        self.write(env, &vars)?;
        Ok(true)
    }

    fn write(&self, env: Environment, vars: &[(String, String)]) -> Result<()> {
        std::fs::create_dir_all(self.dir())?;
        let header = format!("{env} environment variables managed by gozo env. Do not commit.");
        std::fs::write(self.path(env), dotenv::render(vars, Some(&header)))?;
        Ok(())
    }
}

/// Validate an environment variable name.
pub fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && !key.starts_with(|c: char| c.is_ascii_digit())
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
