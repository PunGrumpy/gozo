//! `go list -json` for packages.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::command::{Go, GoError};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PackageInfo {
    pub name: String,
    pub import_path: String,
    pub dir: String,
    #[serde(default)]
    pub module: Option<PackageModule>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub go_files: Vec<String>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub test_go_files: Vec<String>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PackageModule {
    pub path: String,
    #[serde(default)]
    pub dir: Option<String>,
    #[serde(default)]
    pub main: bool,
}

impl Go {
    pub fn list_packages(&self, dir: &Path, pattern: &str) -> Result<Vec<PackageInfo>, GoError> {
        self.run_json_stream(
            dir,
            [
                "list",
                "-json=Name,ImportPath,Dir,Module,GoFiles,TestGoFiles,Imports",
                pattern,
            ],
        )
    }
}
