//! `gozo api`: read-only JSON views of project state for scripts and agents,
//! in the spirit of `vercel api`.

use std::path::Path;
use std::process::ExitCode;

use gozo_core::{DeploymentHistory, EnvStore, Environment, packages};
use serde::Serialize;

use crate::ctx::Ctx;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Resource to read, e.g. project, env/production. `ls` lists them.
    #[arg(value_name = "RESOURCE")]
    pub resource: Option<String>,

    /// Compact single-line JSON instead of pretty-printed.
    #[arg(long)]
    pub raw: bool,

    /// Include env values (masked by default).
    #[arg(long)]
    pub show_values: bool,
}

const SCHEMA: &str = "gozo.api/v1";

/// Every resource with a one-line description.
pub const RESOURCES: [(&str, &str); 11] = [
    (
        "project",
        "name, root, kind, modules, Go version and link target",
    ),
    ("config", "gozo.toml as parsed (defaults filled in)"),
    ("link", ".gozo/project.json, or null when not linked"),
    (
        "env",
        "variables of every environment; env/<environment> for one",
    ),
    ("deployments", "local deployment history, newest first"),
    ("packages", "main packages that gozo dev/build can run"),
    ("modules", "go list -m all: path, version, main, indirect"),
    ("go-env", "go env -json for the project root"),
    ("tools", "tool directives declared in go.mod files"),
    ("doctor", "the full gozo doctor report"),
    ("ls", "this list"),
];

#[derive(Serialize)]
struct Doc<'a> {
    schema: &'static str,
    resource: &'a str,
    data: serde_json::Value,
}

#[derive(Serialize)]
struct Resource {
    name: &'static str,
    description: &'static str,
}

#[derive(Serialize)]
struct LsDoc {
    schema: &'static str,
    resources: Vec<Resource>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let resource = match args.resource {
        Some(r) => r,
        None if ctx.ui.interactive && !ctx.ui.yes && !ctx.json => {
            let items: Vec<String> = RESOURCES
                .iter()
                .filter(|(n, _)| *n != "ls")
                .map(|(n, d)| format!("{n:<12} {}", ctx.ui.dim(d)))
                .collect();
            let i = ctx.ui.select("Which resource?", &items, 0)?;
            RESOURCES[i].0.to_owned()
        }
        None => "ls".to_owned(),
    };

    if resource == "ls" {
        return ls(ctx, args.raw);
    }

    let (name, sub) = match resource.split_once('/') {
        Some((n, s)) => (n, Some(s)),
        None => (resource.as_str(), None),
    };
    if !RESOURCES.iter().any(|(n, _)| *n == name) {
        anyhow::bail!(
            "unknown resource {resource:?}\n  available: {}",
            resource_names().join(", ")
        );
    }
    let data = read(ctx, name, sub, args.show_values)?;
    emit(
        ctx,
        &Doc {
            schema: SCHEMA,
            resource: &resource,
            data,
        },
        args.raw,
    )?;
    Ok(ExitCode::SUCCESS)
}

pub fn resource_names() -> Vec<&'static str> {
    RESOURCES.iter().map(|(n, _)| *n).collect()
}

fn ls(ctx: &mut Ctx, raw: bool) -> anyhow::Result<ExitCode> {
    if ctx.json || raw {
        let doc = LsDoc {
            schema: SCHEMA,
            resources: RESOURCES
                .iter()
                .map(|(n, d)| Resource {
                    name: n,
                    description: d,
                })
                .collect(),
        };
        emit(ctx, &doc, raw)?;
        return Ok(ExitCode::SUCCESS);
    }
    for (n, d) in RESOURCES {
        ctx.out.line(format!("{n:<13} {d}"));
    }
    Ok(ExitCode::SUCCESS)
}

fn emit<T: Serialize>(ctx: &Ctx, doc: &T, raw: bool) -> anyhow::Result<()> {
    if raw {
        ctx.out.json_line(doc)
    } else {
        ctx.out.json_value(doc)
    }
}

fn read(
    ctx: &Ctx,
    name: &str,
    sub: Option<&str>,
    show_values: bool,
) -> anyhow::Result<serde_json::Value> {
    let project = ctx.project()?;
    let root: &Path = &project.root;
    Ok(match name {
        "project" => serde_json::json!({
            "name": ctx.project_name(),
            "root": root,
            "kind": project.kind,
            "modules": project.modules.iter().map(|m| serde_json::json!({
                "rel": m.rel,
                "dir": m.dir,
                "path": m.path(),
                "go": m.gomod.as_ref().ok().and_then(|g| g.go.clone()),
            })).collect::<Vec<_>>(),
            "go": ctx.go.version().ok().map(|v| v.to_string()),
            "goBin": ctx.go.bin,
            "config": gozo_core::Config::exists(root),
            "linked": ctx.link.as_ref().map(|l| serde_json::json!({
                "target": l.target,
                "project": l.project_name,
            })),
        }),
        "config" => serde_json::to_value(&ctx.config)?,
        "link" => serde_json::to_value(&ctx.link)?,
        "env" => {
            let store = EnvStore::new(root);
            let envs: Vec<Environment> = match sub {
                Some(e) => vec![e.parse::<Environment>()?],
                None => Environment::ALL.to_vec(),
            };
            let mut vars = Vec::new();
            for env in envs {
                for (key, value) in store.list(env)? {
                    vars.push(serde_json::json!({
                        "key": key,
                        "value": if show_values { Some(value) } else { None },
                        "environment": env,
                    }));
                }
            }
            serde_json::Value::Array(vars)
        }
        "deployments" => serde_json::to_value(DeploymentHistory::load(root)?.list())?,
        "packages" => serde_json::to_value(packages::main_packages(&ctx.go, project)?)?,
        "modules" => {
            let mods = ctx.go.list_modules(root, false)?;
            serde_json::Value::Array(
                mods.iter()
                    .map(|m| {
                        serde_json::json!({
                            "path": m.path,
                            "version": m.version,
                            "main": m.main,
                            "indirect": m.indirect,
                        })
                    })
                    .collect(),
            )
        }
        "go-env" => serde_json::to_value(ctx.go.env(root)?)?,
        "tools" => serde_json::Value::Array(
            project
                .modules
                .iter()
                .flat_map(|m| {
                    let module = m.path().map(str::to_owned);
                    m.gomod
                        .as_ref()
                        .map(|g| g.tool.clone())
                        .unwrap_or_default()
                        .into_iter()
                        .map(move |t| serde_json::json!({ "path": t.path, "module": module }))
                })
                .collect(),
        ),
        "doctor" => serde_json::to_value(gozo_doctor::run(
            &ctx.go,
            project,
            &gozo_doctor::Options { offline: false },
        ))?,
        other => anyhow::bail!("unknown resource {other:?}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources_are_unique_and_described() {
        let names = resource_names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
        assert!(RESOURCES.iter().all(|(_, d)| !d.is_empty()));
        assert!(names.contains(&"project") && names.contains(&"ls") && names.contains(&"doctor"));
    }
}
