//! Locating the Go project that contains a directory.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::command::{Go, GoError};
use crate::gomod::{GoMod, GoWork};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    /// A single module rooted at go.mod.
    Module,
    /// A multi-module workspace rooted at go.work.
    Workspace,
}

/// A Go project: either one module or a go.work workspace.
#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub kind: ProjectKind,
    /// Directory containing go.work (workspace) or go.mod (module).
    pub root: PathBuf,
    /// Parsed go.work, when `kind` is `Workspace`.
    pub work: Option<GoWork>,
    /// Every module in the project, in go.work `use` order (or just the one).
    pub modules: Vec<ProjectModule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectModule {
    /// Absolute directory of the module.
    pub dir: PathBuf,
    /// Directory relative to the project root, `.` for the root module.
    pub rel: String,
    /// Parsed go.mod, or the error string if `go mod edit -json` failed.
    pub gomod: Result<GoMod, String>,
}

impl ProjectModule {
    pub fn path(&self) -> Option<&str> {
        self.gomod
            .as_ref()
            .ok()
            .and_then(|m| m.module.as_ref())
            .map(|m| m.path.as_str())
    }
}

impl Project {
    /// Walk up from `start` looking for go.work first, then go.mod.
    ///
    /// A go.work anywhere above `start` wins over a nearer go.mod, matching
    /// how the `go` command itself resolves GOWORK.
    pub fn discover(go: &Go, start: &Path) -> Result<Project, GoError> {
        let start = start.canonicalize()?;
        let mut nearest_mod: Option<PathBuf> = None;
        for dir in start.ancestors() {
            if dir.join("go.work").is_file() {
                return Self::from_workspace(go, dir);
            }
            if nearest_mod.is_none() && dir.join("go.mod").is_file() {
                nearest_mod = Some(dir.to_path_buf());
            }
        }
        match nearest_mod {
            Some(dir) => Self::from_module(go, &dir),
            None => Err(GoError::NotAProject(start)),
        }
    }

    fn from_workspace(go: &Go, root: &Path) -> Result<Project, GoError> {
        let work = go.work_edit(root)?;
        let modules = work
            .r#use
            .iter()
            .map(|u| {
                let dir = root.join(&u.disk_path);
                let dir = dir.canonicalize().unwrap_or(dir);
                load_module(go, root, &dir)
            })
            .collect();
        Ok(Project {
            kind: ProjectKind::Workspace,
            root: root.to_path_buf(),
            work: Some(work),
            modules,
        })
    }

    fn from_module(go: &Go, root: &Path) -> Result<Project, GoError> {
        Ok(Project {
            kind: ProjectKind::Module,
            root: root.to_path_buf(),
            work: None,
            modules: vec![load_module(go, root, root)],
        })
    }

    /// A short human name: the last path element of the root module, or the
    /// root directory name for workspaces without a root module.
    pub fn name(&self) -> String {
        self.modules
            .iter()
            .find(|m| m.rel == ".")
            .and_then(|m| m.path())
            .or_else(|| self.modules.first().and_then(|m| m.path()))
            .and_then(|p| p.rsplit('/').next())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                self.root
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "project".to_owned())
            })
    }
}

fn load_module(go: &Go, root: &Path, dir: &Path) -> ProjectModule {
    let rel = dir
        .strip_prefix(root)
        .map(|p| {
            let s = p.to_string_lossy();
            if s.is_empty() {
                ".".to_owned()
            } else {
                s.into_owned()
            }
        })
        .unwrap_or_else(|_| dir.to_string_lossy().into_owned());
    let gomod = if dir.join("go.mod").is_file() {
        go.mod_edit(dir).map_err(|e| e.to_string())
    } else {
        Err(format!("{} has no go.mod", dir.display()))
    };
    ProjectModule {
        dir: dir.to_path_buf(),
        rel,
        gomod,
    }
}
