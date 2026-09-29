//! Shapes of `go mod edit -json` and `go work edit -json`.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::command::{Go, GoError};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GoMod {
    #[serde(default)]
    pub module: Option<ModuleRef>,
    /// The `go` directive, e.g. `1.27.1`. Absent in very old go.mod files.
    #[serde(default)]
    pub go: Option<String>,
    /// The `toolchain` directive, e.g. `go1.27.1`.
    #[serde(default)]
    pub toolchain: Option<String>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub require: Vec<Require>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub exclude: Vec<ModuleRef>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub replace: Vec<Replace>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub retract: Vec<serde_json::Value>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub tool: Vec<Tool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModuleRef {
    pub path: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Require {
    pub path: String,
    pub version: String,
    #[serde(default)]
    pub indirect: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Replace {
    pub old: ModuleRef,
    pub new: ModuleRef,
}

impl Replace {
    /// A replace whose target is a filesystem path rather than a module version.
    pub fn is_local(&self) -> bool {
        self.new.version.is_none()
            && (self.new.path.starts_with("./")
                || self.new.path.starts_with("../")
                || self.new.path.starts_with('/')
                || self.new.path == "."
                || self.new.path == "..")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tool {
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GoWork {
    #[serde(default)]
    pub go: Option<String>,
    #[serde(default)]
    pub toolchain: Option<String>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub r#use: Vec<Use>,
    #[serde(default, deserialize_with = "crate::null_default")]
    pub replace: Vec<Replace>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Use {
    pub disk_path: String,
    #[serde(default)]
    pub module_path: Option<String>,
}

impl Go {
    /// `go mod edit -json` for the module rooted at `dir`.
    pub fn mod_edit(&self, dir: &Path) -> Result<GoMod, GoError> {
        self.run_json_local(dir, ["mod", "edit", "-json"])
    }

    /// `go work edit -json` for the workspace whose go.work lives in `dir`.
    pub fn work_edit(&self, dir: &Path) -> Result<GoWork, GoError> {
        self.run_json_local(dir, ["work", "edit", "-json"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_null_lists_from_older_go() {
        // Go 1.25 prints explicit nulls; Go 1.27 omits the keys entirely.
        let text = r#"{"Module":{"Path":"example.com/demo"},"Go":"1.25.0","Require":null,"Exclude":null,"Replace":null,"Retract":null,"Tool":null}"#;
        let m: GoMod = serde_json::from_str(text).unwrap();
        assert_eq!(m.module.unwrap().path, "example.com/demo");
        assert!(m.require.is_empty() && m.tool.is_empty());

        let w: GoWork =
            serde_json::from_str(r#"{"Go":"1.25.0","Use":null,"Replace":null}"#).unwrap();
        assert!(w.r#use.is_empty());
    }
}
