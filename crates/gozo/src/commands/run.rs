//! `gozo run [TASK] [ARGS...]` — run a task from gozo.toml `[tasks]`, after
//! its `deps`, through the shell. With no task, list the tasks.

use std::collections::BTreeMap;
use std::process::{Command, ExitCode};

use anyhow::Context as _;
use gozo_core::config::Task;
use serde::Serialize;

use crate::ctx::Ctx;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Task name from gozo.toml `[tasks]`. Omit to list tasks.
    #[arg(value_name = "TASK")]
    pub task: Option<String>,

    /// Arguments appended to the task command.
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub args: Vec<String>,

    /// List tasks and exit.
    #[arg(short, long)]
    pub list: bool,
}

#[derive(Debug, Serialize)]
struct Doc {
    schema: &'static str,
    project: String,
    tasks: Vec<TaskEntry>,
}

#[derive(Debug, Serialize)]
struct TaskEntry {
    name: String,
    cmd: String,
    description: Option<String>,
    cwd: Option<String>,
    env: BTreeMap<String, String>,
    deps: Vec<String>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let root = ctx.root()?.to_path_buf();
    let tasks = ctx.config.tasks.clone();

    let Some(name) = args.task.filter(|_| !args.list) else {
        return list(ctx, &tasks);
    };

    if !tasks.contains_key(&name) {
        anyhow::bail!("unknown task `{name}`\n  {}", available(&tasks));
    }
    let order = plan(&tasks, &name)?;
    let project_name = ctx.project_name();

    for task_name in &order {
        let def = tasks
            .get(task_name)
            .map(Task::def)
            .context("task vanished")?;
        let extra = if task_name == &name {
            args.args.as_slice()
        } else {
            &[]
        };
        let cmdline = command_line(&def.cmd, extra);
        ctx.ui
            .step(ctx.ui.dim(&format!("Running {task_name}: {cmdline}")));

        let cwd = match &def.cwd {
            Some(c) => root.join(c),
            None => root.clone(),
        };
        let mut cmd = shell_command(&cmdline);
        cmd.current_dir(&cwd)
            .env("GOZO_PROJECT", &project_name)
            .envs(&def.env);
        ctx.ui.debug(format!("{cmdline}  (in {})", cwd.display()));
        let status = cmd
            .status()
            .with_context(|| format!("failed to start task `{task_name}` in {}", cwd.display()))?;
        if !status.success() {
            let code = status.code().unwrap_or(1);
            ctx.ui
                .error(format!("task `{task_name}` exited with {code}"));
            return Ok(ExitCode::from(code.clamp(1, 255) as u8));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn list(ctx: &Ctx, tasks: &BTreeMap<String, Task>) -> anyhow::Result<ExitCode> {
    if ctx.json {
        let entries = tasks
            .iter()
            .map(|(name, t)| {
                let d = t.def();
                TaskEntry {
                    name: name.clone(),
                    cmd: d.cmd,
                    description: d.description,
                    cwd: d.cwd,
                    env: d.env,
                    deps: d.deps,
                }
            })
            .collect();
        ctx.out.json_value(&Doc {
            schema: "gozo.tasks/v1",
            project: ctx.project_name(),
            tasks: entries,
        })?;
        return Ok(ExitCode::SUCCESS);
    }
    if tasks.is_empty() {
        ctx.ui.warn("no tasks defined in gozo.toml");
        ctx.ui
            .hint("add one:  [tasks]\n        lint = \"golangci-lint run ./...\"");
        return Ok(ExitCode::SUCCESS);
    }
    let width = tasks.keys().map(String::len).max().unwrap_or(0);
    for (name, t) in tasks {
        let d = t.def();
        let what = d.description.clone().unwrap_or_else(|| d.cmd.clone());
        let deps = if d.deps.is_empty() {
            String::new()
        } else {
            format!(
                "  {}",
                ctx.ui.dim(&format!("(after {})", d.deps.join(", ")))
            )
        };
        ctx.out.line(format!("{name:<width$}  {what}{deps}"));
    }
    Ok(ExitCode::SUCCESS)
}

fn available(tasks: &BTreeMap<String, Task>) -> String {
    if tasks.is_empty() {
        "no tasks are defined in gozo.toml".to_owned()
    } else {
        format!(
            "available: {}",
            tasks.keys().cloned().collect::<Vec<_>>().join(", ")
        )
    }
}

/// Dependency-first execution order ending with `name`. Errors on unknown
/// tasks and cycles.
pub fn plan(tasks: &BTreeMap<String, Task>, name: &str) -> anyhow::Result<Vec<String>> {
    fn visit(
        tasks: &BTreeMap<String, Task>,
        name: &str,
        stack: &mut Vec<String>,
        done: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        if done.iter().any(|d| d == name) {
            return Ok(());
        }
        if let Some(pos) = stack.iter().position(|s| s == name) {
            let mut cycle: Vec<&str> = stack[pos..].iter().map(String::as_str).collect();
            cycle.push(name);
            anyhow::bail!("task dependency cycle: {}", cycle.join(" → "));
        }
        let Some(task) = tasks.get(name) else {
            let via = stack
                .last()
                .map(|s| format!(" (required by `{s}`)"))
                .unwrap_or_default();
            anyhow::bail!("unknown task `{name}`{via}\n  {}", available(tasks));
        };
        stack.push(name.to_owned());
        for dep in task.def().deps {
            visit(tasks, &dep, stack, done)?;
        }
        stack.pop();
        done.push(name.to_owned());
        Ok(())
    }
    let mut done = Vec::new();
    visit(tasks, name, &mut Vec::new(), &mut done)?;
    Ok(done)
}

/// The task command with shell-escaped extra arguments appended.
pub fn command_line(cmd: &str, extra: &[String]) -> String {
    if extra.is_empty() {
        cmd.to_owned()
    } else {
        format!("{cmd} {}", shell_words::join(extra))
    }
}

fn shell_command(cmdline: &str) -> Command {
    if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", cmdline]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmdline]);
        c
    }
}

#[cfg(test)]
mod tests {
    use gozo_core::config::TaskDef;

    use super::*;

    fn tasks(spec: &[(&str, &[&str])]) -> BTreeMap<String, Task> {
        spec.iter()
            .map(|(name, deps)| {
                let t = if deps.is_empty() {
                    Task::Command(format!("echo {name}"))
                } else {
                    Task::Full(TaskDef {
                        cmd: format!("echo {name}"),
                        deps: deps.iter().map(|d| (*d).to_owned()).collect(),
                        ..Default::default()
                    })
                };
                ((*name).to_owned(), t)
            })
            .collect()
    }

    #[test]
    fn plans_deps_first_without_repeats() {
        let t = tasks(&[
            ("fmt", &[]),
            ("lint", &["fmt"]),
            ("test", &["fmt"]),
            ("ci", &["lint", "test"]),
        ]);
        assert_eq!(plan(&t, "ci").unwrap(), vec!["fmt", "lint", "test", "ci"]);
        assert_eq!(plan(&t, "fmt").unwrap(), vec!["fmt"]);
    }

    #[test]
    fn detects_cycles_and_unknown_deps() {
        let t = tasks(&[("a", &["b"]), ("b", &["a"]), ("c", &["missing"])]);
        let err = plan(&t, "a").unwrap_err().to_string();
        assert!(err.contains("a → b → a"), "{err}");
        let err = plan(&t, "c").unwrap_err().to_string();
        assert!(err.contains("unknown task `missing`"), "{err}");
        assert!(err.contains("required by `c`"), "{err}");
    }

    #[test]
    fn escapes_extra_args() {
        assert_eq!(command_line("go test", &[]), "go test");
        assert_eq!(
            command_line("go test", &["-run".to_owned(), "Test A".to_owned()]),
            "go test -run 'Test A'"
        );
    }
}
