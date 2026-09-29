//! Bare `gozo`: show what was detected and offer the common actions, like
//! running `vercel` with no arguments.

use std::path::Path;
use std::process::ExitCode;

use gozo_go::ProjectKind;
use serde::Serialize;

use crate::commands;
use crate::ctx::Ctx;

const SCHEMA: &str = "gozo.project/v1";

#[derive(Serialize)]
struct ProjectDoc<'a> {
    schema: &'static str,
    detected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    root: Option<&'a Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<ProjectKind>,
    modules: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    go: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    linked: Option<LinkedDoc>,
    cwd: &'a Path,
}

#[derive(Serialize)]
struct LinkedDoc {
    target: gozo_core::Target,
    project: String,
}

/// Menu entries, in display order.
const ACTIONS: [(&str, &str); 7] = [
    ("Start development", "gozo dev"),
    ("Run tests", "gozo test"),
    ("Check project", "gozo check"),
    ("Doctor", "gozo doctor"),
    ("Deploy", "gozo deploy"),
    ("Show logs", "gozo logs"),
    ("Link project", "gozo link"),
];

pub fn run(ctx: &mut Ctx) -> anyhow::Result<ExitCode> {
    let go_version = ctx.go.version().ok().map(|v| v.to_string());

    let Some(project) = ctx.project.clone() else {
        if ctx.json {
            ctx.out.json_value(&ProjectDoc {
                schema: SCHEMA,
                detected: false,
                name: None,
                root: None,
                module: None,
                kind: None,
                modules: 0,
                go: go_version,
                linked: None,
                cwd: &ctx.cwd,
            })?;
            return Ok(ExitCode::from(2));
        }
        ctx.ui.header();
        ctx.ui
            .info(format!("No Go project found in {}", ctx.cwd.display()));
        ctx.ui
            .detail("gozo wraps the go command with dev, env, link and deploy workflows.");
        if ctx.ui.interactive && !ctx.ui.yes {
            if ctx.ui.confirm("Create a project here?", true)? {
                return commands::init::run(ctx, default_args::<commands::init::Args>()?);
            }
            return Ok(ExitCode::SUCCESS);
        }
        ctx.ui
            .hint("run `gozo init` to create one, or cd into a directory with go.mod");
        return Ok(ExitCode::from(2));
    };

    let name = ctx.project_name();
    let module = project
        .modules
        .iter()
        .find(|m| m.rel == ".")
        .or_else(|| project.modules.first())
        .and_then(|m| m.path())
        .map(str::to_owned);

    if ctx.json {
        ctx.out.json_value(&ProjectDoc {
            schema: SCHEMA,
            detected: true,
            name: Some(name),
            root: Some(&project.root),
            module,
            kind: Some(project.kind),
            modules: project.modules.len(),
            go: go_version,
            linked: ctx.link.as_ref().map(|l| LinkedDoc {
                target: l.target,
                project: l.project_name.clone(),
            }),
            cwd: &ctx.cwd,
        })?;
        return Ok(ExitCode::SUCCESS);
    }

    ctx.ui.header();
    ctx.ui.info(ctx.ui.bold("Detected Go project"));
    ctx.ui.kv("name", &name);
    ctx.ui.kv("module", module.as_deref().unwrap_or("unknown"));
    ctx.ui.kv("go", go_version.as_deref().unwrap_or("unknown"));
    match project.kind {
        ProjectKind::Workspace => ctx
            .ui
            .kv("workspace", format!("{} modules", project.modules.len())),
        ProjectKind::Module => ctx.ui.kv("layout", "module"),
    }
    ctx.ui.kv(
        "linked",
        match &ctx.link {
            Some(l) => format!("{} ({})", l.target, l.project_name),
            None => ctx.ui.dim("not linked"),
        },
    );
    ctx.ui.blank();

    if !ctx.ui.interactive || ctx.ui.yes {
        for (label, cmd) in ACTIONS {
            ctx.ui.info(format!(
                "  {:<20} {}",
                ctx.ui.dim(&format!("{cmd:<12}")),
                label
            ));
        }
        ctx.ui.blank();
        ctx.ui.hint("run `gozo --help` for every command");
        return Ok(ExitCode::SUCCESS);
    }

    let items: Vec<String> = ACTIONS
        .iter()
        .map(|(label, cmd)| format!("{label:<20} {}", ctx.ui.dim(cmd)))
        .collect();
    let choice = ctx.ui.select("What do you want to do?", &items, 0)?;
    ctx.ui.blank();
    match ACTIONS[choice].1 {
        "gozo dev" => commands::dev::run(ctx, default_args()?),
        "gozo test" => commands::test::run(ctx, default_args()?),
        "gozo check" => commands::check::run(ctx, default_args()?),
        "gozo doctor" => commands::doctor::run(ctx, default_args()?),
        "gozo deploy" => commands::deploy::run(ctx, default_args()?),
        "gozo logs" => commands::logs::run(ctx, default_args()?),
        _ => commands::link::run(ctx, default_args()?),
    }
}

/// A command's `Args` as if invoked with no flags at all.
fn default_args<A: clap::Args>() -> anyhow::Result<A> {
    #[derive(clap::Parser)]
    struct Wrap<A: clap::Args> {
        #[command(flatten)]
        inner: A,
    }
    let w = <Wrap<A> as clap::Parser>::try_parse_from(["gozo"]).map_err(|e| {
        anyhow::anyhow!(
            "this command needs arguments; run it directly: {}",
            e.to_string().trim()
        )
    })?;
    Ok(w.inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args_build_for_menu_commands() {
        assert!(default_args::<commands::dev::Args>().is_ok());
        assert!(default_args::<commands::doctor::Args>().is_ok());
        assert!(default_args::<commands::link::Args>().is_ok());
        assert!(default_args::<commands::init::Args>().is_ok());
    }
}
