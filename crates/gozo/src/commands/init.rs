use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context as _;
use gozo_core::{Config, git, link};
use serde::Serialize;

use crate::ctx::Ctx;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Directory to initialise. Defaults to the current directory.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Project name written to gozo.toml. Defaults to the module's last element.
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,

    /// Module path for `go mod init` when there is no go.mod yet.
    #[arg(long, value_name = "MODULE")]
    pub module: Option<String>,

    /// Name of the main package to create under cmd/ when there is no go.mod yet.
    #[arg(long, value_name = "NAME", default_value = "api")]
    pub cmd: String,

    /// Overwrite an existing gozo.toml.
    #[arg(long)]
    pub force: bool,
}

#[derive(Serialize)]
struct InitDoc<'a> {
    schema: &'static str,
    root: &'a Path,
    name: &'a str,
    module: &'a str,
    created: &'a [String],
    already_initialized: bool,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let target = match &args.path {
        Some(p) => {
            let p = if p.is_absolute() {
                p.clone()
            } else {
                ctx.cwd.join(p)
            };
            std::fs::create_dir_all(&p)
                .with_context(|| format!("could not create {}", p.display()))?;
            p.canonicalize()?
        }
        None => ctx.cwd.clone(),
    };
    let dir_name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "app".to_owned());

    ctx.ui.header();

    let has_gomod = target.join("go.mod").is_file();
    let existing_module = if has_gomod {
        ctx.go
            .mod_edit(&target)
            .ok()
            .and_then(|m| m.module)
            .map(|m| m.path)
    } else {
        None
    };

    if Config::exists(&target) && !args.force {
        let module = existing_module.clone().unwrap_or_default();
        let name = ctx
            .config
            .project
            .name
            .clone()
            .unwrap_or_else(|| dir_name.clone());
        if ctx.json {
            ctx.out.json_value(&InitDoc {
                schema: "gozo.init/v1",
                root: &target,
                name: &name,
                module: &module,
                created: &[],
                already_initialized: true,
            })?;
        } else {
            ctx.ui.info(format!(
                "{} already exists in {}",
                gozo_core::config::FILE,
                target.display()
            ));
            ctx.ui.hint("pass --force to overwrite it");
        }
        return Ok(ExitCode::SUCCESS);
    }

    let mut created: Vec<String> = Vec::new();
    let mut dev_cmd: Option<String> = None;

    let module = match existing_module {
        Some(m) => m,
        None => {
            let default_module = git::remote_url(&target)
                .and_then(|u| module_path_from_remote(&u))
                .unwrap_or_else(|| format!("example.com/{}", sanitize_segment(&dir_name)));
            let module = match &args.module {
                Some(m) => m.clone(),
                None => ctx
                    .ui
                    .input("What's your module path?", Some(&default_module))?,
            };
            if module.trim().is_empty() {
                anyhow::bail!("module path cannot be empty");
            }
            ctx.ui.debug(format!("go mod init {module}"));
            ctx.go
                .run_ok(&target, ["mod", "init", module.as_str()])
                .context("`go mod init` failed")?;
            created.push("go.mod".to_owned());
            ctx.ui.success("Created go.mod");

            let cmd = sanitize_segment(&args.cmd);
            let main_rel = format!("cmd/{cmd}/main.go");
            let main_path = target.join(&main_rel);
            if !main_path.exists() {
                std::fs::create_dir_all(main_path.parent().unwrap_or(&target))?;
                std::fs::write(&main_path, main_go(&cmd))?;
                created.push(main_rel.clone());
                ctx.ui.success(format!("Created {main_rel}"));
            }
            dev_cmd = Some(format!("./cmd/{cmd}"));

            let had_gitignore = target.join(".gitignore").is_file();
            let mut changed = false;
            for entry in ["dist/", ".gozo/", ".env.local"] {
                changed |= link::ensure_gitignore_entry(&target, entry)?;
            }
            if changed {
                created.push(".gitignore".to_owned());
                ctx.ui.success(if had_gitignore {
                    "Updated .gitignore"
                } else {
                    "Created .gitignore"
                });
            }
            module
        }
    };

    let default_name = module.rsplit('/').next().unwrap_or(&dir_name).to_owned();
    let name = match &args.name {
        Some(n) => n.clone(),
        None => ctx
            .ui
            .input("What's your project's name?", Some(&default_name))?,
    };

    let config_path = Config::path(&target);
    std::fs::write(&config_path, Config::template(&name, dev_cmd.as_deref()))
        .with_context(|| format!("could not write {}", config_path.display()))?;
    created.push(gozo_core::config::FILE.to_owned());
    ctx.ui
        .success(format!("Created {}", gozo_core::config::FILE));

    if target == ctx.cwd || ctx.cwd.starts_with(&target) {
        ctx.reload_project()?;
    }

    if ctx.json {
        ctx.out.json_value(&InitDoc {
            schema: "gozo.init/v1",
            root: &target,
            name: &name,
            module: &module,
            created: &created,
            already_initialized: false,
        })?;
    } else {
        ctx.ui.blank();
        ctx.ui.info("Next steps:");
        if target != ctx.cwd {
            ctx.ui
                .detail(format!("cd {}", relative_display(&ctx.cwd, &target)));
        }
        ctx.ui
            .detail("gozo dev       start the app with live reload");
        ctx.ui
            .detail("gozo doctor    check the toolchain and modules");
        ctx.ui.detail("gozo link      choose where to deploy");
    }
    Ok(ExitCode::SUCCESS)
}

/// `git@github.com:acme/api.git` -> `github.com/acme/api`; `None` for
/// local paths or anything that is not `host/owner/repo`.
fn module_path_from_remote(url: &str) -> Option<String> {
    let url = url.trim();
    let rest = if let Some(r) = url.strip_prefix("git@") {
        r.replacen(':', "/", 1)
    } else {
        let r = url
            .strip_prefix("ssh://")
            .or_else(|| url.strip_prefix("https://"))
            .or_else(|| url.strip_prefix("http://"))?;
        // Drop user[:token]@ credentials.
        r.split_once('@').map(|(_, h)| h).unwrap_or(r).to_owned()
    };
    let rest = rest.trim_end_matches('/');
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let mut parts = rest.split('/').filter(|p| !p.is_empty());
    let host = parts.next()?;
    let host = host.split_once(':').map(|(h, _)| h).unwrap_or(host);
    let owner = parts.next()?;
    let repo = parts.next()?;
    if !host.contains('.') || parts.next().is_some() {
        return None;
    }
    Some(format!("{host}/{owner}/{repo}"))
}

/// Lowercase `[a-z0-9_-]` only; `app` when nothing is left.
fn sanitize_segment(s: &str) -> String {
    let out: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let out = out.trim_matches('-').to_owned();
    if out.is_empty() {
        "app".to_owned()
    } else {
        out
    }
}

fn relative_display(from: &Path, to: &Path) -> String {
    to.strip_prefix(from)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| to.display().to_string())
}

fn main_go(name: &str) -> String {
    format!(
        r#"package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
)

// Set at build time by `gozo build` (-ldflags -X main.version=...).
var (
	version = "dev"
	commit  = "none"
	date    = "unknown"
)

func main() {{
	port := os.Getenv("PORT")
	if port == "" {{
		port = "8080"
	}}
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {{
		fmt.Fprintln(w, "ok")
	}})
	log.Printf("{name} %s (%s, %s) listening on :%s", version, commit, date, port)
	log.Fatal(http.ListenAndServe(":"+port, nil))
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_path_from_git_remotes() {
        assert_eq!(
            module_path_from_remote("git@github.com:acme/api.git").as_deref(),
            Some("github.com/acme/api")
        );
        assert_eq!(
            module_path_from_remote("https://github.com/acme/api").as_deref(),
            Some("github.com/acme/api")
        );
        assert_eq!(
            module_path_from_remote("https://user:tok@gitlab.com/acme/api.git/").as_deref(),
            Some("gitlab.com/acme/api")
        );
        assert_eq!(
            module_path_from_remote("ssh://git@github.com:22/acme/api.git").as_deref(),
            Some("github.com/acme/api")
        );
        assert_eq!(module_path_from_remote("/srv/git/api.git"), None);
        assert_eq!(module_path_from_remote("https://github.com/acme"), None);
        assert_eq!(module_path_from_remote("https://github.com/a/b/c"), None);
    }

    #[test]
    fn sanitizes_segments() {
        assert_eq!(sanitize_segment("My App!"), "my-app");
        assert_eq!(sanitize_segment("---"), "app");
        assert_eq!(sanitize_segment("api"), "api");
    }
}
