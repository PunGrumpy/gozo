use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gozo_go::{Go, GoVersion, ModuleInfo, Project, ProjectKind, TidyStatus};

use crate::Options;
use crate::report::{Finding, ProjectSummary, Report, Status, Summary, ToolchainSummary};

pub struct Context<'a> {
    pub go: &'a Go,
    pub project: &'a Project,
    pub opts: &'a Options,
    pub version: Option<GoVersion>,
    pub env: Option<BTreeMap<String, String>>,
    modules: Option<Result<Vec<ModuleInfo>, String>>,
    findings: Vec<Finding>,
}

impl<'a> Context<'a> {
    pub fn new(go: &'a Go, project: &'a Project, opts: &'a Options) -> Self {
        Context {
            go,
            project,
            opts,
            version: go.version().ok(),
            env: go.env(&project.root).ok(),
            modules: None,
            findings: Vec::new(),
        }
    }

    fn env_var(&self, key: &str) -> Option<&str> {
        self.env
            .as_ref()
            .and_then(|e| e.get(key))
            .map(String::as_str)
    }

    /// `None` when offline.
    fn modules(&mut self) -> Option<&Result<Vec<ModuleInfo>, String>> {
        if self.opts.offline {
            return None;
        }
        if self.modules.is_none() {
            let dir = self.list_dir();
            let r = self.go.list_modules(&dir, true).map_err(|e| e.to_string());
            self.modules = Some(r);
        }
        self.modules.as_ref()
    }

    /// In a workspace, `go list -m all` from any module dir reports every module.
    fn list_dir(&self) -> PathBuf {
        match self.project.kind {
            ProjectKind::Workspace => self
                .project
                .modules
                .first()
                .map(|m| m.dir.clone())
                .unwrap_or_else(|| self.project.root.clone()),
            ProjectKind::Module => self.project.root.clone(),
        }
    }

    pub fn finish(self) -> Report {
        let mut summary = Summary::default();
        for f in &self.findings {
            match f.status {
                Status::Ok => summary.ok += 1,
                Status::Warn => summary.warn += 1,
                Status::Fail => summary.fail += 1,
                Status::Skip => summary.skip += 1,
            }
        }
        Report {
            schema: crate::report::SCHEMA,
            project: ProjectSummary {
                name: self.project.name(),
                kind: self.project.kind,
                root: self.project.root.clone(),
                modules: self
                    .project
                    .modules
                    .iter()
                    .map(|m| m.path().unwrap_or(&m.rel).to_owned())
                    .collect(),
            },
            go: ToolchainSummary {
                bin: self.go.bin.clone(),
                version: self.version.as_ref().map(|v| v.raw.clone()),
            },
            findings: self.findings,
            summary,
        }
    }
}

pub fn toolchain(cx: &mut Context) {
    let f = match &cx.version {
        Some(v) => Finding::new(
            "go.toolchain",
            Status::Ok,
            format!("Go toolchain {}", v.raw),
        )
        .detail(cx.go.bin.display().to_string()),
        None => Finding::new(
            "go.toolchain",
            Status::Fail,
            "Go toolchain version could not be read",
        )
        .detail(cx.go.bin.display().to_string())
        .hint("run `go version` and check the installation"),
    };
    cx.findings.push(f);
}

pub fn gomod_valid(cx: &mut Context) {
    let bad: Vec<String> = cx
        .project
        .modules
        .iter()
        .filter_map(|m| m.gomod.as_ref().err().map(|e| format!("{}: {}", m.rel, e)))
        .collect();
    let n = cx.project.modules.len();
    let f = if bad.is_empty() {
        Finding::new(
            "go.mod.valid",
            Status::Ok,
            format!("go.mod valid ({} module{})", n, plural(n)),
        )
    } else {
        Finding::new(
            "go.mod.valid",
            Status::Fail,
            format!(
                "{} go.mod file{} could not be parsed",
                bad.len(),
                plural(bad.len())
            ),
        )
        .details(bad)
    };
    cx.findings.push(f);
}

pub fn go_directive(cx: &mut Context) {
    let Some(installed) = cx.version.clone() else {
        cx.findings.push(Finding::new(
            "go.directive",
            Status::Skip,
            "go directive not checked: toolchain unknown",
        ));
        return;
    };
    let auto = cx.env_var("GOTOOLCHAIN") != Some("local");
    let mut unmet = Vec::new();
    let mut highest: Option<GoVersion> = None;
    for m in &cx.project.modules {
        let Ok(gomod) = &m.gomod else { continue };
        let Some(req) = gomod.go.as_deref().and_then(|g| GoVersion::parse(g).ok()) else {
            continue;
        };
        if !installed.satisfies(&req) {
            unmet.push(format!("{} requires go {}", m.rel, req));
        }
        if highest.as_ref().is_none_or(|h| req > *h) {
            highest = Some(req);
        }
    }
    let f = if unmet.is_empty() {
        let req = highest
            .map(|h| h.to_string())
            .unwrap_or_else(|| "any".to_owned());
        Finding::new(
            "go.directive",
            Status::Ok,
            format!("Go {} satisfies go directive {}", installed, req),
        )
    } else if auto {
        Finding::new(
            "go.directive",
            Status::Warn,
            format!(
                "Installed Go {} is older than the project requires",
                installed
            ),
        )
        .details(unmet)
        .detail("GOTOOLCHAIN=auto will download a newer toolchain on first build")
    } else {
        Finding::new(
            "go.directive",
            Status::Fail,
            format!(
                "Installed Go {} is older than the project requires",
                installed
            ),
        )
        .details(unmet)
        .hint("upgrade Go or set GOTOOLCHAIN=auto")
    };
    cx.findings.push(f);
}

pub fn workspace(cx: &mut Context) {
    let Some(work) = &cx.project.work else {
        return;
    };
    let mut problems = Vec::new();
    for m in &cx.project.modules {
        if !m.dir.join("go.mod").is_file() {
            problems.push(format!("use {} has no go.mod", m.rel));
        }
    }
    if let Some(work_go) = work.go.as_deref().and_then(|g| GoVersion::parse(g).ok()) {
        for m in &cx.project.modules {
            let Ok(gomod) = &m.gomod else { continue };
            if let Some(req) = gomod.go.as_deref().and_then(|g| GoVersion::parse(g).ok()) {
                if req > work_go {
                    problems.push(format!(
                        "{} requires go {} but go.work declares {}",
                        m.rel, req, work_go
                    ));
                }
            }
        }
    }
    let n = cx.project.modules.len();
    let f = if problems.is_empty() {
        Finding::new(
            "go.work",
            Status::Ok,
            format!("go.work consistent ({} module{})", n, plural(n)),
        )
    } else {
        Finding::new("go.work", Status::Fail, "go.work is inconsistent")
            .details(problems)
            .hint("run `go work sync` or fix the `use` entries")
    };
    cx.findings.push(f);
}

pub fn tidy(cx: &mut Context) {
    let mut dirty = Vec::new();
    let mut unknown = Vec::new();
    let mut sibling_dep = Vec::new();
    let mut checked = 0;
    for m in &cx.project.modules {
        if m.gomod.is_err() {
            continue;
        }
        checked += 1;
        match cx.go.tidy_diff(&m.dir) {
            Ok(TidyStatus::Clean) => {}
            Ok(TidyStatus::Dirty { diff }) => {
                let changes = diff
                    .lines()
                    .filter(|l| {
                        (l.starts_with('+') || l.starts_with('-'))
                            && !l.starts_with("+++")
                            && !l.starts_with("---")
                    })
                    .count();
                dirty.push(format!(
                    "{}: {} line{} would change",
                    m.rel,
                    changes,
                    plural(changes)
                ));
            }
            Ok(TidyStatus::Unknown { reason }) => {
                if let Some(sibling) = sibling_module_in(cx, &reason) {
                    sibling_dep.push(format!(
                        "{}: depends on workspace module {}",
                        m.rel, sibling
                    ));
                } else {
                    unknown.push(format!("{}: {}", m.rel, reason));
                }
            }
            Err(e) => unknown.push(format!("{}: {}", m.rel, e)),
        }
    }
    let f = if checked == 0 {
        Finding::new(
            "go.tidy",
            Status::Skip,
            "go.sum not checked: no parseable modules",
        )
    } else if !dirty.is_empty() {
        Finding::new("go.tidy", Status::Fail, "go.mod / go.sum are not tidy")
            .details(dirty)
            .hint("run `go mod tidy` in the listed modules")
    } else if !unknown.is_empty() {
        Finding::new(
            "go.tidy",
            Status::Skip,
            "go.sum could not be verified offline",
        )
        .details(unknown)
        .hint("run `go mod download` then re-run doctor")
    } else if !sibling_dep.is_empty() {
        // `go mod tidy` ignores go.work, so a module that imports a sibling
        // workspace module cannot be tidied in isolation.
        Finding::new(
            "go.tidy",
            Status::Skip,
            "go.sum not verified: `go mod tidy` does not support workspaces",
        )
        .details(sibling_dep)
        .hint("Go limitation; run `go work sync` to keep the workspace consistent")
    } else {
        Finding::new("go.tidy", Status::Ok, "go.mod and go.sum are tidy")
    };
    cx.findings.push(f);
}

pub fn replace_outside(cx: &mut Context) {
    let root = cx
        .project
        .root
        .canonicalize()
        .unwrap_or_else(|_| cx.project.root.clone());
    let mut outside = Vec::new();
    let mut local = 0;
    for m in &cx.project.modules {
        let Ok(gomod) = &m.gomod else { continue };
        for r in gomod.replace.iter().filter(|r| r.is_local()) {
            local += 1;
            let target = normalize(&m.dir.join(&r.new.path));
            if !target.starts_with(&root) {
                outside.push(format!("{}: {} => {}", m.rel, r.old.path, r.new.path));
            }
        }
    }
    if let Some(work) = &cx.project.work {
        for r in work.replace.iter().filter(|r| r.is_local()) {
            local += 1;
            let target = normalize(&cx.project.root.join(&r.new.path));
            if !target.starts_with(&root) {
                outside.push(format!("go.work: {} => {}", r.old.path, r.new.path));
            }
        }
    }
    if local == 0 {
        return;
    }
    let f = if outside.is_empty() {
        Finding::new(
            "go.replace.local",
            Status::Ok,
            format!(
                "{} local replace directive{} inside the project",
                local,
                plural(local)
            ),
        )
    } else {
        Finding::new(
            "go.replace.local",
            Status::Fail,
            format!(
                "{} replace directive{} outside the project",
                outside.len(),
                if outside.len() == 1 {
                    " points"
                } else {
                    "s point"
                }
            ),
        )
        .details(outside)
        .hint("these builds will not reproduce on another machine or in CI")
    };
    cx.findings.push(f);
}

pub fn outdated(cx: &mut Context) {
    let tools = tool_modules(cx);
    let Some(result) = cx.modules() else {
        cx.findings.push(Finding::new(
            "deps.outdated",
            Status::Skip,
            "dependency updates not checked (offline)",
        ));
        return;
    };
    let mods = match result {
        Ok(m) => m.clone(),
        Err(e) => {
            let f = Finding::new(
                "deps.outdated",
                Status::Skip,
                "dependency updates could not be listed",
            )
            .detail(e.clone());
            cx.findings.push(f);
            return;
        }
    };
    let outdated: Vec<&ModuleInfo> = mods
        .iter()
        .filter(|m| !m.main && m.update.is_some() && !tools.contains(&m.path))
        .collect();
    let direct = outdated.iter().filter(|m| !m.indirect).count();
    let f = if outdated.is_empty() {
        Finding::new("deps.outdated", Status::Ok, "dependencies are up to date")
    } else {
        let n = outdated.len();
        let mut f = Finding::new(
            "deps.outdated",
            Status::Warn,
            format!(
                "{} dependenc{} can be updated ({} direct)",
                n,
                if n == 1 { "y" } else { "ies" },
                direct
            ),
        );
        for m in outdated.iter().filter(|m| !m.indirect).take(10) {
            f = f.detail(update_line(m));
        }
        if direct > 10 {
            f = f.detail(format!("… and {} more direct", direct - 10));
        }
        if direct == 0 {
            f = f.detail("all are indirect dependencies");
        }
        f.hint("run `go get -u ./...` or update selectively")
    };
    cx.findings.push(f);
}

pub fn tools(cx: &mut Context) {
    let declared: Vec<String> = cx
        .project
        .modules
        .iter()
        .filter_map(|m| m.gomod.as_ref().ok())
        .flat_map(|g| g.tool.iter().map(|t| t.path.clone()))
        .collect();
    if declared.is_empty() {
        return;
    }
    let tool_mods = tool_modules(cx);
    let n = declared.len();
    let Some(Ok(mods)) = cx.modules().cloned() else {
        let f = Finding::new(
            "tools",
            Status::Ok,
            format!("{} tool{} declared in go.mod", n, plural(n)),
        )
        .details(declared);
        cx.findings.push(f);
        return;
    };
    let stale: Vec<String> = mods
        .iter()
        .filter(|m| tool_mods.contains(&m.path) && m.update.is_some())
        .map(update_line)
        .collect();
    let f = if stale.is_empty() {
        Finding::new(
            "tools",
            Status::Ok,
            format!("{} tool{} declared and up to date", n, plural(n)),
        )
        .details(declared)
    } else {
        Finding::new(
            "tools",
            Status::Warn,
            format!(
                "{} tool module{} outdated",
                stale.len(),
                plural(stale.len())
            ),
        )
        .details(stale)
        .hint("run `go get -tool <path>@latest`")
    };
    cx.findings.push(f);
}

pub fn cgo(cx: &mut Context) {
    if cx.env.is_none() {
        cx.findings.push(Finding::new(
            "cgo",
            Status::Skip,
            "CGO not checked: `go env` failed",
        ));
        return;
    }
    if cx.env_var("CGO_ENABLED") != Some("1") {
        cx.findings
            .push(Finding::new("cgo", Status::Ok, "CGO disabled"));
        return;
    }
    let cc = cx.env_var("CC").unwrap_or("gcc").to_owned();
    let f = if which(&cc) {
        Finding::new(
            "cgo",
            Status::Ok,
            format!("CGO enabled, C compiler `{}` found", cc),
        )
    } else {
        Finding::new(
            "cgo",
            Status::Warn,
            format!("CGO enabled but C compiler `{}` is not on PATH", cc),
        )
        .detail("packages that import \"C\" will fail to build")
        .hint("install a C toolchain or set CGO_ENABLED=0")
    };
    cx.findings.push(f);
}

fn sibling_module_in(cx: &Context, text: &str) -> Option<String> {
    cx.project
        .modules
        .iter()
        .filter_map(|m| m.path())
        .find(|p| text.contains(p))
        .map(str::to_owned)
}

/// Module paths that provide a declared tool, by longest-prefix match.
fn tool_modules(cx: &Context) -> Vec<String> {
    let mut out = Vec::new();
    for m in &cx.project.modules {
        let Ok(gomod) = &m.gomod else { continue };
        for t in &gomod.tool {
            let best = gomod
                .require
                .iter()
                .filter(|r| t.path == r.path || t.path.starts_with(&format!("{}/", r.path)))
                .max_by_key(|r| r.path.len());
            if let Some(r) = best {
                if !out.contains(&r.path) {
                    out.push(r.path.clone());
                }
            }
        }
    }
    out
}

fn update_line(m: &ModuleInfo) -> String {
    format!(
        "{} {} → {}",
        m.path,
        m.version.as_deref().unwrap_or("?"),
        m.update.as_ref().map(|u| u.version.as_str()).unwrap_or("?")
    )
}

fn normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out.canonicalize().unwrap_or(out)
}

fn which(name: &str) -> bool {
    if name.contains('/') {
        return Path::new(name).is_file();
    }
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(name).is_file()))
        .unwrap_or(false)
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
