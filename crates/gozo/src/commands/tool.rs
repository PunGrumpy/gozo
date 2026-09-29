use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};

use anyhow::Context as _;
use gozo_go::{GoMod, ProjectModule};
use serde::Serialize;

use super::util::{display_rel, exit_for, plural};
use crate::ctx::Ctx;
use crate::style::{Mark, bold, dim, green, mark};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Option<ToolCmd>,

    /// Module directory to operate on (default: the module containing the
    /// current directory, else the root module).
    #[arg(long, global = true, value_name = "DIR")]
    pub module: Option<PathBuf>,
}

#[derive(Debug, clap::Subcommand)]
pub enum ToolCmd {
    /// List tools declared in go.mod (default).
    #[command(alias = "list")]
    Ls,
    /// Add a tool: `go get -tool <pkg[@version]>`.
    Add {
        /// Package path, optionally with `@version`.
        #[arg(value_name = "PKG[@VERSION]")]
        pkg: String,
    },
    /// Remove a tool: `go get -tool <pkg>@none`.
    #[command(alias = "remove")]
    Rm {
        /// Tool name (last path element) or full package path.
        name: String,
    },
    /// Run a tool: `go tool <path> [ARGS...]`.
    Run {
        /// Tool name (last path element) or full package path.
        name: String,
        /// Arguments passed to the tool.
        #[arg(
            value_name = "ARGS",
            trailing_var_arg = true,
            allow_hyphen_values = true
        )]
        args: Vec<String>,
    },
    /// Show tool modules that have a newer version (needs the network).
    Outdated,
    /// Update one tool, or all, to `@latest`.
    Update {
        /// Tool name or full package path. Omit to update every tool.
        name: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
struct ToolEntry {
    /// Last path element; what `gozo tool run <name>` accepts.
    name: String,
    path: String,
    /// The `require` providing the tool (longest path prefix), if any.
    module: Option<String>,
    version: Option<String>,
    /// Module directory relative to the project root.
    dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    update: Option<String>,
}

#[derive(Debug, Serialize)]
struct Doc {
    schema: &'static str,
    project: String,
    tools: Vec<ToolEntry>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let project = ctx.project()?;
    let root = project.root.clone();
    let module = select_module(ctx, args.module.as_deref())?;

    match args.cmd.unwrap_or(ToolCmd::Ls) {
        ToolCmd::Ls => ls(ctx, args.module.as_ref().map(|_| &module)),
        ToolCmd::Add { pkg } => add(ctx, &root, &module, &pkg),
        ToolCmd::Rm { name } => rm(ctx, &root, &module, &name),
        ToolCmd::Run { name, args } => run_tool(ctx, &root, &module, &name, &args),
        ToolCmd::Outdated => outdated(ctx, &root, &module),
        ToolCmd::Update { name } => update(ctx, &root, &module, name.as_deref()),
    }
}

fn tools_of(gomod: &GoMod, dir: &str) -> Vec<ToolEntry> {
    gomod
        .tool
        .iter()
        .map(|t| {
            let provider = gomod
                .require
                .iter()
                .filter(|r| t.path == r.path || t.path.starts_with(&format!("{}/", r.path)))
                .max_by_key(|r| r.path.len());
            ToolEntry {
                name: t.path.rsplit('/').next().unwrap_or(&t.path).to_owned(),
                path: t.path.clone(),
                module: provider.map(|r| r.path.clone()),
                version: provider.map(|r| r.version.clone()),
                dir: dir.to_owned(),
                update: None,
            }
        })
        .collect()
}

/// Re-reads go.mod so add/rm/update see their own changes.
fn module_tools(ctx: &Ctx, module: &ProjectModule) -> anyhow::Result<Vec<ToolEntry>> {
    let gomod = ctx
        .go
        .mod_edit(&module.dir)
        .with_context(|| format!("reading {}", module.dir.join("go.mod").display()))?;
    Ok(tools_of(&gomod, &module.rel))
}

fn find_tool<'a>(tools: &'a [ToolEntry], name: &str) -> anyhow::Result<&'a ToolEntry> {
    if let Some(t) = tools.iter().find(|t| t.path == name) {
        return Ok(t);
    }
    let by_name: Vec<&ToolEntry> = tools.iter().filter(|t| t.name == name).collect();
    match by_name.as_slice() {
        [one] => Ok(one),
        [] => {
            if tools.is_empty() {
                anyhow::bail!(
                    "unknown tool `{name}`: go.mod declares no tools\n  {}",
                    ADD_HINT
                );
            }
            let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
            anyhow::bail!("unknown tool `{name}`\n  available: {}", names.join(", "));
        }
        many => {
            let paths: Vec<&str> = many.iter().map(|t| t.path.as_str()).collect();
            anyhow::bail!(
                "`{name}` is ambiguous; use the full path: {}",
                paths.join(", ")
            );
        }
    }
}

const ADD_HINT: &str = "add one with `gozo tool add golang.org/x/tools/cmd/stringer`";

/// `--module DIR`, else the module containing cwd, else the root module.
fn select_module(ctx: &Ctx, wanted: Option<&Path>) -> anyhow::Result<ProjectModule> {
    let project = ctx.project()?;
    let usable: Vec<&ProjectModule> = project.modules.iter().filter(|m| m.gomod.is_ok()).collect();
    if let Some(w) = wanted {
        let abs = if w.is_absolute() {
            w.to_path_buf()
        } else {
            ctx.cwd.join(w)
        };
        let abs = abs
            .canonicalize()
            .with_context(|| format!("--module {}", w.display()))?;
        return usable
            .iter()
            .find(|m| m.dir == abs)
            .map(|m| (*m).clone())
            .with_context(|| {
                let rels: Vec<&str> = usable.iter().map(|m| m.rel.as_str()).collect();
                format!(
                    "{} is not a module of this project\n  modules: {}",
                    abs.display(),
                    rels.join(", ")
                )
            });
    }
    let containing = usable
        .iter()
        .filter(|m| ctx.cwd.starts_with(&m.dir))
        .max_by_key(|m| m.dir.as_os_str().len());
    if let Some(m) = containing {
        return Ok((*m).clone());
    }
    usable
        .iter()
        .find(|m| m.rel == ".")
        .or_else(|| usable.first())
        .map(|m| (*m).clone())
        .context("no usable module in this project (every go.mod failed to parse)")
}

fn ls(ctx: &Ctx, only: Option<&ProjectModule>) -> anyhow::Result<ExitCode> {
    let project = ctx.project()?;
    let multi = project.modules.len() > 1 && only.is_none();
    let tools: Vec<ToolEntry> = project
        .modules
        .iter()
        .filter(|m| only.is_none_or(|o| o.dir == m.dir))
        .filter_map(|m| m.gomod.as_ref().ok().map(|g| tools_of(g, &m.rel)))
        .flatten()
        .collect();
    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.tools/v1",
            project: ctx.project_name(),
            tools,
        })?;
        return Ok(ExitCode::SUCCESS);
    }
    if tools.is_empty() {
        ctx.ui.info("no tool directives in go.mod");
        ctx.ui.hint(ADD_HINT);
        return Ok(ExitCode::SUCCESS);
    }
    print_table(&tools, multi, false);
    Ok(ExitCode::SUCCESS)
}

fn add(ctx: &Ctx, root: &Path, module: &ProjectModule, pkg: &str) -> anyhow::Result<ExitCode> {
    let path = pkg.split('@').next().unwrap_or(pkg).to_owned();
    ctx.ui.step(format!("Adding {pkg}"));
    ctx.ui.debug(format!(
        "go get -tool {pkg}  (in {})",
        display_rel(root, &module.dir)
    ));
    ctx.go
        .run_ok(&module.dir, ["get", "-tool", pkg])
        .with_context(|| format!("`go get -tool {pkg}` failed"))?;
    let tools = module_tools(ctx, module)?;
    let entry = tools.iter().find(|t| t.path == path);
    let name = entry.map(|t| t.name.as_str()).unwrap_or(&path);
    let version = entry.and_then(|t| t.version.as_deref()).unwrap_or("?");
    ctx.ui.success(format!("added {name} ({path} {version})"));
    ctx.ui.hint(format!("run it with `gozo tool run {name}`"));
    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.tools/v1",
            project: ctx.project_name(),
            tools,
        })?;
    }
    Ok(ExitCode::SUCCESS)
}

fn rm(ctx: &Ctx, root: &Path, module: &ProjectModule, name: &str) -> anyhow::Result<ExitCode> {
    let tools = module_tools(ctx, module)?;
    let path = find_tool(&tools, name)?.path.clone();
    ctx.ui.step(format!("Removing {path}"));
    let spec = format!("{path}@none");
    ctx.ui.debug(format!(
        "go get -tool {spec}  (in {})",
        display_rel(root, &module.dir)
    ));
    ctx.go
        .run_ok(&module.dir, ["get", "-tool", &spec])
        .with_context(|| format!("`go get -tool {spec}` failed"))?;
    ctx.ui.success(format!("removed {path}"));
    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.tools/v1",
            project: ctx.project_name(),
            tools: module_tools(ctx, module)?,
        })?;
    }
    Ok(ExitCode::SUCCESS)
}

fn run_tool(
    ctx: &Ctx,
    root: &Path,
    module: &ProjectModule,
    name: &str,
    args: &[String],
) -> anyhow::Result<ExitCode> {
    let tools = module_tools(ctx, module)?;
    let path = find_tool(&tools, name)?.path.clone();
    let mut go_args: Vec<&str> = vec!["tool", &path];
    go_args.extend(args.iter().map(String::as_str));
    ctx.ui.debug(format!(
        "go {}  (in {})",
        shell_words::join(&go_args),
        display_rel(root, &module.dir)
    ));
    let status = ctx
        .go
        .command(&module.dir, &go_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to run `go tool {path}`"))?;
    Ok(match status.code() {
        Some(0) => ExitCode::SUCCESS,
        Some(c) => ExitCode::from(c.clamp(1, 255) as u8),
        None => ExitCode::from(1),
    })
}

fn outdated(ctx: &Ctx, root: &Path, module: &ProjectModule) -> anyhow::Result<ExitCode> {
    let mut tools = module_tools(ctx, module)?;
    if tools.is_empty() {
        return no_tools(ctx);
    }
    ctx.ui.step("Checking for updates");
    ctx.ui.debug(format!(
        "go list -m -u -json all  (in {})",
        display_rel(root, &module.dir)
    ));
    let mods = ctx
        .go
        .list_modules(&module.dir, true)
        .context("`go list -m -u all` failed (is the network available?)")?;
    for t in &mut tools {
        let Some(provider) = &t.module else { continue };
        if let Some(m) = mods.iter().find(|m| &m.path == provider) {
            t.update = m.update.as_ref().map(|u| u.version.clone());
            if t.version.is_none() {
                t.version = m.version.clone();
            }
        }
    }
    let stale: Vec<&ToolEntry> = tools.iter().filter(|t| t.update.is_some()).collect();
    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.tools/v1",
            project: ctx.project_name(),
            tools: tools.clone(),
        })?;
    } else if stale.is_empty() {
        let n = tools.len();
        println!(
            "  {} {n} tool{} up to date",
            mark(ctx.ui.color, Mark::Ok),
            plural(n)
        );
    } else {
        let owned: Vec<ToolEntry> = stale.iter().map(|t| (*t).clone()).collect();
        print_table(&owned, false, true);
        println!();
        println!("  {} run `gozo tool update` to update all", dim("hint:"));
    }
    Ok(exit_for(stale.is_empty()))
}

fn update(
    ctx: &Ctx,
    root: &Path,
    module: &ProjectModule,
    name: Option<&str>,
) -> anyhow::Result<ExitCode> {
    let tools = module_tools(ctx, module)?;
    if tools.is_empty() {
        return no_tools(ctx);
    }
    let targets: Vec<ToolEntry> = match name {
        Some(n) => vec![find_tool(&tools, n)?.clone()],
        None => tools.clone(),
    };
    for t in &targets {
        let spec = format!("{}@latest", t.path);
        ctx.ui.step(format!("Updating {}", t.name));
        ctx.ui.debug(format!(
            "go get -tool {spec}  (in {})",
            display_rel(root, &module.dir)
        ));
        ctx.go
            .run_ok(&module.dir, ["get", "-tool", &spec])
            .with_context(|| format!("`go get -tool {spec}` failed"))?;
    }
    let after = module_tools(ctx, module)?;
    for t in &targets {
        let now = after
            .iter()
            .find(|a| a.path == t.path)
            .and_then(|a| a.version.clone());
        match (&t.version, &now) {
            (Some(before), Some(now)) if before != now => {
                ctx.ui.success(format!("{}  {before} → {now}", t.name));
            }
            (_, Some(now)) => ctx
                .ui
                .success(format!("{}  {now} (already latest)", t.name)),
            _ => ctx.ui.success(format!("{} updated", t.name)),
        }
    }
    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.tools/v1",
            project: ctx.project_name(),
            tools: after,
        })?;
    }
    Ok(ExitCode::SUCCESS)
}

fn no_tools(ctx: &Ctx) -> anyhow::Result<ExitCode> {
    if ctx.json {
        ctx.out.json_value(&Doc {
            schema: "gozo.tools/v1",
            project: ctx.project_name(),
            tools: Vec::new(),
        })?;
    } else {
        ctx.ui.info("no tool directives in go.mod");
        ctx.ui.hint(ADD_HINT);
    }
    Ok(ExitCode::SUCCESS)
}

fn print_table(tools: &[ToolEntry], show_dir: bool, show_update: bool) {
    let w_name = tools.iter().map(|t| t.name.len()).max().unwrap_or(0);
    let w_path = tools.iter().map(|t| t.path.len()).max().unwrap_or(0);
    let w_ver = tools
        .iter()
        .map(|t| t.version.as_deref().unwrap_or("?").len())
        .max()
        .unwrap_or(0);
    for t in tools {
        let version = t.version.as_deref().unwrap_or("?");
        let mut line = format!(
            "  {:<w_name$}  {:<w_path$}  {version:<w_ver$}",
            bold(&t.name),
            t.path
        );
        if show_update {
            if let Some(u) = &t.update {
                line.push_str(&format!("  → {}", green(u)));
            }
        }
        if show_dir {
            line.push_str(&format!("  {}", dim(&t.dir)));
        }
        println!("{}", line.trim_end());
    }
}

#[cfg(test)]
mod tests {
    use gozo_go::{Require, Tool};

    use super::*;

    fn gomod() -> GoMod {
        GoMod {
            tool: vec![
                Tool {
                    path: "golang.org/x/tools/cmd/stringer".to_owned(),
                },
                Tool {
                    path: "github.com/acme/lint".to_owned(),
                },
                Tool {
                    path: "example.com/orphan/cmd/x".to_owned(),
                },
            ],
            require: vec![
                Require {
                    path: "golang.org/x".to_owned(),
                    version: "v0.0.1".to_owned(),
                    indirect: true,
                },
                Require {
                    path: "golang.org/x/tools".to_owned(),
                    version: "v0.50.0".to_owned(),
                    indirect: true,
                },
                Require {
                    path: "github.com/acme/lint".to_owned(),
                    version: "v1.2.3".to_owned(),
                    indirect: false,
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn resolves_versions_by_longest_prefix() {
        let tools = tools_of(&gomod(), ".");
        assert_eq!(tools[0].name, "stringer");
        assert_eq!(tools[0].module.as_deref(), Some("golang.org/x/tools"));
        assert_eq!(tools[0].version.as_deref(), Some("v0.50.0"));
        assert_eq!(tools[1].version.as_deref(), Some("v1.2.3"));
        assert_eq!(tools[2].version, None);
    }

    #[test]
    fn finds_tools_by_name_or_path() {
        let tools = tools_of(&gomod(), ".");
        assert_eq!(
            find_tool(&tools, "stringer").unwrap().path,
            "golang.org/x/tools/cmd/stringer"
        );
        assert_eq!(
            find_tool(&tools, "github.com/acme/lint").unwrap().name,
            "lint"
        );
        let err = find_tool(&tools, "nope").unwrap_err().to_string();
        assert!(err.contains("available: stringer, lint, x"), "{err}");
    }
}
