//! Discovery of `main` packages, the things `gozo dev` runs and `gozo build` builds.

use std::path::{Path, PathBuf};

use serde::Serialize;

use gozo_go::{Go, Project};

use crate::Result;

#[derive(Debug, Clone, Serialize)]
pub struct MainPackage {
    /// Binary name: last element of the import path.
    pub name: String,
    pub import_path: String,
    /// Absolute directory.
    pub dir: PathBuf,
    /// Directory relative to its module, as a `./...` style path (`./cmd/api`).
    pub rel: String,
    /// Directory of the module that owns the package.
    pub module_dir: PathBuf,
}

/// Every `package main` in the project, cmd/ packages first.
pub fn main_packages(go: &Go, project: &Project) -> Result<Vec<MainPackage>> {
    let mut out = Vec::new();
    for m in &project.modules {
        if m.gomod.is_err() {
            continue;
        }
        for p in go.list_packages(&m.dir, "./...")? {
            if p.name != "main" {
                continue;
            }
            let dir = PathBuf::from(&p.dir);
            let rel = rel_path(&m.dir, &dir);
            out.push(MainPackage {
                name: p
                    .import_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&p.import_path)
                    .to_owned(),
                import_path: p.import_path,
                dir,
                rel,
                module_dir: m.dir.clone(),
            });
        }
    }
    out.sort_by_key(|p| (!p.rel.starts_with("./cmd/"), p.rel.clone()));
    Ok(out)
}

fn rel_path(base: &Path, dir: &Path) -> String {
    match dir.strip_prefix(base) {
        Ok(r) if r.as_os_str().is_empty() => ".".to_owned(),
        Ok(r) => format!("./{}", r.to_string_lossy().replace('\\', "/")),
        Err(_) => dir.to_string_lossy().into_owned(),
    }
}

/// Pick the package `gozo dev` should run: an explicit `dev.cmd`, else the
/// only main package, else the first one under `cmd/`.
pub fn pick_dev_package<'a>(
    pkgs: &'a [MainPackage],
    configured: Option<&str>,
) -> Option<&'a MainPackage> {
    if let Some(c) = configured {
        let want = c.trim_start_matches("./").trim_end_matches('/');
        return pkgs.iter().find(|p| {
            p.rel.trim_start_matches("./") == want || p.import_path == c || p.name == want
        });
    }
    if pkgs.len() == 1 {
        return pkgs.first();
    }
    pkgs.iter().find(|p| p.rel.starts_with("./cmd/"))
}
