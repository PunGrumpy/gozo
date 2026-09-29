use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use anyhow::Context as _;
use gozo_core::{Config, DockerTarget, KubernetesTarget, Link, Target, link};
use gozo_go::Project;
use serde::Serialize;

use crate::ctx::Ctx;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Directory to link. Defaults to the current project.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Project name to link as.
    #[arg(long, value_name = "NAME")]
    pub project: Option<String>,

    /// Deployment target: docker or kubernetes.
    #[arg(long, value_name = "TARGET")]
    pub target: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct UnlinkArgs {
    /// Directory to unlink. Defaults to the current project.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

#[derive(Serialize)]
struct LinkDoc<'a> {
    schema: &'static str,
    root: &'a Path,
    #[serde(flatten)]
    link: &'a Link,
}

#[derive(Serialize)]
struct UnlinkDoc<'a> {
    schema: &'static str,
    root: &'a Path,
    removed: bool,
}

fn resolve(ctx: &Ctx, path: &Option<PathBuf>) -> anyhow::Result<(Project, Config, Option<Link>)> {
    match path {
        Some(p) => {
            let p = if p.is_absolute() {
                p.clone()
            } else {
                ctx.cwd.join(p)
            };
            let project = Project::discover(&ctx.go, &p).with_context(|| {
                format!(
                    "no Go project found in {} or any parent directory",
                    p.display()
                )
            })?;
            let config = Config::load(&project.root)?.unwrap_or_default();
            let link = Link::load(&project.root)?;
            Ok((project, config, link))
        }
        None => Ok((ctx.project()?.clone(), ctx.config.clone(), ctx.link.clone())),
    }
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let (project, config, existing) = resolve(ctx, &args.path)?;
    let root = project.root.clone();
    ctx.ui.header();

    if !ctx
        .ui
        .confirm(&format!("Set up {:?}?", root.display().to_string()), true)?
    {
        ctx.ui.info("Canceled");
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(l) = &existing {
        ctx.ui.info(format!(
            "{} is already linked to {} ({})",
            root.display(),
            ctx.ui.bold(&l.project_name),
            l.target
        ));
        let explicit = args.project.is_some() || args.target.is_some();
        if !ctx.ui.confirm("Replace the existing link?", explicit)? {
            ctx.ui.info("Kept the existing link");
            return Ok(ExitCode::SUCCESS);
        }
    }

    let default_name = ctx
        .project_override
        .clone()
        .or_else(|| existing.as_ref().map(|l| l.project_name.clone()))
        .or_else(|| config.project.name.clone())
        .unwrap_or_else(|| project.name());
    let name = match &args.project {
        Some(n) => n.clone(),
        None => ctx
            .ui
            .input("What's your project's name?", Some(&default_name))?,
    };
    let name = name.trim().to_owned();
    if name.is_empty() {
        anyhow::bail!("project name cannot be empty");
    }

    let target = match &args.target {
        Some(t) => t.parse::<Target>()?,
        None => {
            let preset = existing
                .as_ref()
                .map(|l| l.target)
                .or_else(|| config.deploy.target.as_deref().and_then(|t| t.parse().ok()));
            let items: Vec<String> = Target::ALL
                .iter()
                .map(|t| format!("{:<11} {}", t.as_str(), ctx.ui.dim(t.description())))
                .collect();
            let default = preset
                .and_then(|p| Target::ALL.iter().position(|t| *t == p))
                .unwrap_or(0);
            Target::ALL[ctx
                .ui
                .select("Where do you want to deploy?", &items, default)?]
        }
    };

    let docker: Option<DockerTarget>;
    let mut kubernetes: Option<KubernetesTarget> = None;
    match target {
        Target::Docker => {
            let base = existing
                .as_ref()
                .and_then(|l| l.docker.clone())
                .unwrap_or_else(|| config.deploy.docker.clone());
            let image = ctx
                .ui
                .input("Image name?", Some(base.image.as_deref().unwrap_or(&name)))?;
            let ports_default = if base.ports.is_empty() {
                "8080:8080".to_owned()
            } else {
                base.ports.join(",")
            };
            let ports = ctx.ui.input(
                "Host ports (host:container, comma separated)?",
                Some(&ports_default),
            )?;
            docker = Some(DockerTarget {
                image: Some(image.trim().to_owned()),
                ports: split_list(&ports),
                container: base.container.clone().or_else(|| Some(name.clone())),
                ..base
            });
        }
        Target::Kubernetes => {
            let base = existing
                .as_ref()
                .and_then(|l| l.kubernetes.clone())
                .unwrap_or_else(|| config.deploy.kubernetes.clone());
            let context_default = base.context.clone().or_else(kubectl_current_context);
            let context = ctx
                .ui
                .input("Kubernetes context?", context_default.as_deref())?;
            let namespace = ctx.ui.input(
                "Namespace?",
                Some(base.namespace.as_deref().unwrap_or("default")),
            )?;
            let deployment = ctx.ui.input(
                "Deployment name?",
                Some(base.deployment.as_deref().unwrap_or(&name)),
            )?;
            let container = ctx.ui.input(
                "Container name?",
                Some(base.container.as_deref().unwrap_or(&deployment)),
            )?;
            let image_default = base
                .image
                .clone()
                .or_else(|| config.deploy.docker.image.clone())
                .or_else(|| {
                    existing
                        .as_ref()
                        .and_then(|l| l.docker.as_ref())
                        .and_then(|d| d.image.clone())
                });
            let image = ctx
                .ui
                .input("Image to push (e.g. ghcr.io/you/app)?", image_default.as_deref())
                .context("kubernetes needs an image repository; set deploy.kubernetes.image in gozo.toml or run interactively")?;
            if image.trim().is_empty() {
                anyhow::bail!("image cannot be empty for kubernetes deployments");
            }
            let mut d = config.deploy.docker.clone();
            if d.image.is_none() {
                d.image = Some(image.trim().to_owned());
            }
            d.push = true;
            docker = Some(d);
            kubernetes = Some(KubernetesTarget {
                context: non_empty(context),
                namespace: non_empty(namespace),
                deployment: non_empty(deployment),
                container: non_empty(container),
                image: Some(image.trim().to_owned()),
                manifests: base.manifests.clone(),
            });
        }
    }

    let had_dir = Link::dir(&root).exists();
    let is_git = root.join(".git").exists();
    let had_ignore = link::gitignore_has(&root, gozo_core::STATE_DIR);

    let new_link = Link {
        project_name: name.clone(),
        target,
        docker,
        kubernetes,
        linked_at: chrono::Utc::now(),
    };
    new_link
        .save(&root)
        .context("could not write .gozo/project.json")?;
    if ctx.project.as_ref().is_some_and(|p| p.root == root) {
        ctx.reload_link()?;
    }

    if ctx.json {
        ctx.out.json_value(&LinkDoc {
            schema: "gozo.link/v1",
            root: &root,
            link: &new_link,
        })?;
    } else {
        let mut notes = Vec::new();
        if !had_dir {
            notes.push(format!("created {}", gozo_core::STATE_DIR));
        }
        if is_git && !had_ignore {
            notes.push("added it to .gitignore".to_owned());
        }
        let suffix = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join(" and "))
        };
        ctx.ui.success(format!(
            "Linked to {} ({}){}",
            ctx.ui.bold(&name),
            target,
            suffix
        ));
    }
    Ok(ExitCode::SUCCESS)
}

pub fn unlink(ctx: &mut Ctx, args: UnlinkArgs) -> anyhow::Result<ExitCode> {
    let (project, _, _) = resolve(ctx, &args.path)?;
    let root = project.root.clone();
    let removed = Link::remove(&root)?;
    if ctx.project.as_ref().is_some_and(|p| p.root == root) {
        ctx.reload_link()?;
    }
    if ctx.json {
        ctx.out.json_value(&UnlinkDoc {
            schema: "gozo.unlink/v1",
            root: &root,
            removed,
        })?;
    } else if removed {
        ctx.ui.success("Unlinked");
    } else {
        ctx.ui.info(format!("{} is not linked", root.display()));
    }
    Ok(ExitCode::SUCCESS)
}

fn split_list(s: &str) -> Vec<String> {
    s.split([',', ' '])
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect()
}

fn non_empty(s: String) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

fn kubectl_current_context() -> Option<String> {
    let out = Command::new("kubectl")
        .args(["config", "current-context"])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    non_empty(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_port_lists() {
        assert_eq!(
            split_list("8080:8080, 9090:9090"),
            vec!["8080:8080", "9090:9090"]
        );
        assert!(split_list("  ").is_empty());
    }
}
