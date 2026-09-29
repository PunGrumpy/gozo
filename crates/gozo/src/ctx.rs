use std::path::{Path, PathBuf};

use anyhow::{Context as _, anyhow};
use gozo_core::{Config, Link};
use gozo_go::{Go, Project};

use crate::Cli;
use crate::output::Output;
use crate::ui::Ui;

pub struct Ctx {
    pub cwd: PathBuf,
    pub go: Go,
    /// `None` outside a Go project; `config` is then the defaults.
    pub project: Option<Project>,
    pub config: Config,
    pub link: Option<Link>,
    pub ui: Ui,
    pub out: Output,
    pub json: bool,
    pub yes: bool,
    pub debug: bool,
    pub project_override: Option<String>,
}

impl Ctx {
    pub fn load(cli: &Cli) -> anyhow::Result<Ctx> {
        let cwd = match &cli.cwd {
            Some(p) => p
                .canonicalize()
                .with_context(|| format!("--cwd {}", p.display()))?,
            None => std::env::current_dir()?,
        };
        let no_color = cli.no_color || std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        let ui = Ui::new(
            !no_color,
            !cli.non_interactive,
            cli.yes,
            cli.debug,
            cli.json,
        );
        let out = Output::new(cli.json);

        let go = Go::find().context("gozo needs the `go` command on PATH")?;
        let project = match Project::discover(&go, &cwd) {
            Ok(p) => Some(p),
            Err(gozo_go::GoError::NotAProject(_)) => None,
            Err(e) => return Err(e.into()),
        };
        let root = project.as_ref().map(|p| p.root.clone());
        let config = match &root {
            Some(r) => Config::load(r)?.unwrap_or_default(),
            None => Config::default(),
        };
        let link = match &root {
            Some(r) => Link::load(r)?,
            None => None,
        };

        Ok(Ctx {
            cwd,
            go,
            project,
            config,
            link,
            ui,
            out,
            json: cli.json,
            yes: cli.yes,
            debug: cli.debug,
            project_override: cli.project.clone(),
        })
    }

    pub fn project(&self) -> anyhow::Result<&Project> {
        self.project.as_ref().ok_or_else(|| {
            anyhow!(
                "no Go project found in {} or any parent directory\n  run `gozo init` to create one, or cd into a directory with go.mod",
                self.cwd.display()
            )
        })
    }

    pub fn root(&self) -> anyhow::Result<&Path> {
        Ok(&self.project()?.root)
    }

    /// Precedence: `--project`, link, gozo.toml, module name, directory name.
    pub fn project_name(&self) -> String {
        if let Some(p) = &self.project_override {
            return p.clone();
        }
        if let Some(l) = &self.link {
            return l.project_name.clone();
        }
        if let Some(n) = &self.config.project.name {
            return n.clone();
        }
        self.project.as_ref().map(|p| p.name()).unwrap_or_else(|| {
            self.cwd
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "project".to_owned())
        })
    }

    pub fn link_required(&self) -> anyhow::Result<&Link> {
        self.link.as_ref().ok_or_else(|| {
            anyhow!("this directory is not linked to a deployment target\n  run `gozo link` first")
        })
    }

    pub fn reload_link(&mut self) -> anyhow::Result<()> {
        if let Some(root) = self.project.as_ref().map(|p| p.root.clone()) {
            self.link = Link::load(&root)?;
        }
        Ok(())
    }

    pub fn reload_config(&mut self) -> anyhow::Result<()> {
        if let Some(root) = self.project.as_ref().map(|p| p.root.clone()) {
            self.config = Config::load(&root)?.unwrap_or_default();
        }
        Ok(())
    }

    pub fn reload_project(&mut self) -> anyhow::Result<()> {
        self.project = match Project::discover(&self.go, &self.cwd) {
            Ok(p) => Some(p),
            Err(gozo_go::GoError::NotAProject(_)) => None,
            Err(e) => return Err(e.into()),
        };
        self.reload_config()?;
        self.reload_link()
    }
}
