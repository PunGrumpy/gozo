//! `gozo env`: per-environment variables stored under `.gozo/env/`, with the
//! verbs of `vercel env` (ls, add, rm, update, pull, run).

use std::io::{IsTerminal, Read as _};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::Context as _;
use gozo_core::{EnvStore, Environment, dotenv, envstore, link};
use serde::Serialize;

use crate::ctx::Ctx;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    /// List variables, values masked unless --show-values.
    #[command(alias = "list")]
    Ls {
        /// development, preview or production. All three when omitted.
        environment: Option<String>,
        /// Print the values instead of masking them.
        #[arg(long)]
        show_values: bool,
    },
    /// Add a variable. The value comes from --value, stdin, or a hidden prompt.
    Add(SetArgs),
    /// Change an existing variable (same as `add --force`).
    Update(SetArgs),
    /// Remove a variable.
    #[command(alias = "remove")]
    Rm {
        name: String,
        /// development, preview or production. All three when omitted.
        environment: Option<String>,
    },
    /// Write an environment's variables to a dotenv file (default .env.local).
    Pull {
        /// File to write. Defaults to .env.local in the current directory.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        /// Environment to pull.
        #[arg(short, long, default_value = "development")]
        environment: String,
    },
    /// Run a command with an environment's variables injected (no file is written).
    Run {
        /// Environment whose variables to inject.
        #[arg(short, long, default_value = "development")]
        environment: String,
        /// The command to run, after `--`.
        #[arg(value_name = "CMD", trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct SetArgs {
    /// Variable name, e.g. DATABASE_URL.
    pub name: String,
    /// development, preview or production. All three when omitted (asked interactively).
    pub environment: Option<String>,
    /// The value. Otherwise read from stdin when piped, or asked for hidden.
    #[arg(long, value_name = "VALUE")]
    pub value: Option<String>,
    /// Overwrite an existing variable without asking.
    #[arg(long)]
    pub force: bool,
    /// Accepted for parity with other CLIs: gozo never prints values unless --show-values.
    #[arg(long)]
    pub sensitive: bool,
}

const SCHEMA: &str = "gozo.env/v1";
const MASK: &str = "••••";

#[derive(Serialize)]
struct Var {
    key: String,
    /// Present only with --show-values.
    value: Option<String>,
    environment: Environment,
}

#[derive(Serialize)]
struct LsDoc {
    schema: &'static str,
    variables: Vec<Var>,
}

#[derive(Serialize)]
struct MutationDoc<'a> {
    schema: &'static str,
    ok: bool,
    action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<&'a str>,
    environments: Vec<Environment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<&'a Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<usize>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let root = ctx.root()?.to_path_buf();
    let store = EnvStore::new(&root);
    match args.cmd {
        Cmd::Ls {
            environment,
            show_values,
        } => ls(ctx, &store, environment.as_deref(), show_values),
        Cmd::Add(a) => set(ctx, &store, a, false),
        Cmd::Update(a) => set(ctx, &store, a, true),
        Cmd::Rm { name, environment } => rm(ctx, &store, &name, environment.as_deref()),
        Cmd::Pull { file, environment } => pull(ctx, &root, &store, file, &environment),
        Cmd::Run {
            environment,
            command,
        } => run_cmd(ctx, &store, &environment, &command),
    }
}

fn parse_env(s: &str) -> anyhow::Result<Environment> {
    Ok(s.parse::<Environment>()?)
}

fn envs_from(arg: Option<&str>) -> anyhow::Result<Vec<Environment>> {
    Ok(match arg {
        Some(e) => vec![parse_env(e)?],
        None => Environment::ALL.to_vec(),
    })
}

fn ls(
    ctx: &mut Ctx,
    store: &EnvStore,
    environment: Option<&str>,
    show_values: bool,
) -> anyhow::Result<ExitCode> {
    let mut vars = Vec::new();
    for env in envs_from(environment)? {
        for (key, value) in store.list(env)? {
            vars.push(Var {
                key,
                value: show_values.then_some(value),
                environment: env,
            });
        }
    }

    if ctx.json {
        ctx.out.json_value(&LsDoc {
            schema: SCHEMA,
            variables: vars,
        })?;
        return Ok(ExitCode::SUCCESS);
    }
    if vars.is_empty() {
        ctx.ui.info(match environment {
            Some(e) => format!("No environment variables found for {e}"),
            None => "No environment variables found".to_owned(),
        });
        ctx.ui.hint("add one with `gozo env add NAME`");
        return Ok(ExitCode::SUCCESS);
    }
    let kw = vars.iter().map(|v| v.key.len()).max().unwrap_or(4).max(4);
    let vw = vars
        .iter()
        .map(|v| v.value.as_deref().unwrap_or(MASK).chars().count())
        .max()
        .unwrap_or(5)
        .max(5);
    ctx.out
        .line(format!("{:<kw$}  {:<vw$}  environment", "name", "value"));
    for v in &vars {
        let value = v.value.as_deref().unwrap_or(MASK);
        let pad = vw.saturating_sub(value.chars().count());
        ctx.out.line(format!(
            "{:<kw$}  {}{}  {}",
            v.key,
            value,
            " ".repeat(pad),
            v.environment
        ));
    }
    Ok(ExitCode::SUCCESS)
}

/// The value for `add`/`update`: flag, piped stdin, or a hidden prompt.
fn read_value(ctx: &Ctx, explicit: Option<String>, key: &str) -> anyhow::Result<String> {
    if let Some(v) = explicit {
        return Ok(v);
    }
    if !std::io::stdin().is_terminal() {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .context("could not read the value from stdin")?;
        return Ok(strip_one_newline(&s).to_owned());
    }
    ctx.ui.password(&format!("What's the value of {key}?"))
}

fn strip_one_newline(s: &str) -> &str {
    s.strip_suffix("\r\n")
        .or_else(|| s.strip_suffix('\n'))
        .unwrap_or(s)
}

/// Pick target environments: the argument, or all three (asked when interactive).
fn choose_envs(ctx: &Ctx, arg: Option<&str>, key: &str) -> anyhow::Result<Vec<Environment>> {
    if let Some(e) = arg {
        return Ok(vec![parse_env(e)?]);
    }
    if ctx.ui.yes || !ctx.ui.interactive {
        return Ok(Environment::ALL.to_vec());
    }
    let items: Vec<String> = Environment::ALL.iter().map(|e| e.to_string()).collect();
    let picked = dialoguer::MultiSelect::new()
        .with_prompt(format!(
            "? Add {key} to which environments? (space to toggle)"
        ))
        .items(&items)
        .defaults(&[true, true, true])
        .interact()?;
    if picked.is_empty() {
        anyhow::bail!("no environment selected");
    }
    Ok(picked.into_iter().map(|i| Environment::ALL[i]).collect())
}

fn set(ctx: &mut Ctx, store: &EnvStore, a: SetArgs, update: bool) -> anyhow::Result<ExitCode> {
    if !envstore::valid_key(&a.name) {
        anyhow::bail!(
            "invalid variable name {:?}\n  names use letters, digits and underscores and cannot start with a digit",
            a.name
        );
    }
    let envs = choose_envs(ctx, a.environment.as_deref(), &a.name)?;
    let value = read_value(ctx, a.value, &a.name)?;

    let mut written = Vec::new();
    for env in envs {
        let exists = store.get(env, &a.name)?.is_some();
        if exists && !update && !a.force {
            let overwrite = ctx
                .ui
                .confirm(&format!("Overwrite {} in {env}?", a.name), false)
                .unwrap_or(false);
            if !overwrite {
                ctx.ui.warn(format!(
                    "{} already exists in {env}, skipped (use --force to overwrite)",
                    a.name
                ));
                continue;
            }
        }
        store.set(env, &a.name, &value)?;
        written.push(env);
    }

    if ctx.json {
        ctx.out.json_value(&MutationDoc {
            schema: SCHEMA,
            ok: !written.is_empty(),
            action: if update { "update" } else { "add" },
            key: Some(&a.name),
            environments: written.clone(),
            file: None,
            count: None,
        })?;
    } else if !written.is_empty() {
        let list: Vec<String> = written.iter().map(|e| e.to_string()).collect();
        ctx.ui.success(format!(
            "{} {} in {}",
            if update { "Updated" } else { "Added" },
            ctx.ui.bold(&a.name),
            list.join(", ")
        ));
    }
    Ok(if written.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn rm(
    ctx: &mut Ctx,
    store: &EnvStore,
    name: &str,
    environment: Option<&str>,
) -> anyhow::Result<ExitCode> {
    let envs: Vec<Environment> = envs_from(environment)?
        .into_iter()
        .filter(|e| store.get(*e, name).ok().flatten().is_some())
        .collect();
    if envs.is_empty() {
        if ctx.json {
            ctx.out.json_value(&MutationDoc {
                schema: SCHEMA,
                ok: false,
                action: "rm",
                key: Some(name),
                environments: vec![],
                file: None,
                count: None,
            })?;
        } else {
            ctx.ui.info(format!("{name} is not set"));
        }
        return Ok(ExitCode::from(1));
    }
    let list: Vec<String> = envs.iter().map(|e| e.to_string()).collect();
    if !ctx
        .ui
        .confirm(&format!("Remove {name} from {}?", list.join(", ")), true)?
    {
        ctx.ui.info("Canceled");
        return Ok(ExitCode::SUCCESS);
    }
    for env in &envs {
        store.remove(*env, name)?;
    }
    if ctx.json {
        ctx.out.json_value(&MutationDoc {
            schema: SCHEMA,
            ok: true,
            action: "rm",
            key: Some(name),
            environments: envs,
            file: None,
            count: None,
        })?;
    } else {
        ctx.ui.success(format!(
            "Removed {} from {}",
            ctx.ui.bold(name),
            list.join(", ")
        ));
    }
    Ok(ExitCode::SUCCESS)
}

fn pull(
    ctx: &mut Ctx,
    root: &Path,
    store: &EnvStore,
    file: Option<PathBuf>,
    environment: &str,
) -> anyhow::Result<ExitCode> {
    let env = parse_env(environment)?;
    let file = file.unwrap_or_else(|| PathBuf::from(".env.local"));
    let path = if file.is_absolute() {
        file.clone()
    } else {
        ctx.cwd.join(&file)
    };
    let display = file.display().to_string();

    if path.exists() && !ctx.yes && !ctx.ui.confirm(&format!("Overwrite {display}?"), false)? {
        ctx.ui.info("Canceled");
        return Ok(ExitCode::SUCCESS);
    }

    let vars = store.list(env)?;
    let header = format!("Created by gozo CLI\nEnvironment: {env}");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, dotenv::render(&vars, Some(&header)))
        .with_context(|| format!("could not write {}", path.display()))?;

    // Keep secrets out of git when the file lives inside the repository.
    let mut ignored = false;
    if root.join(".git").exists() {
        if let Ok(rel) = path.strip_prefix(root) {
            let entry = rel.to_string_lossy().replace('\\', "/");
            ignored = link::ensure_gitignore_entry(root, &entry)?;
        }
    }

    if ctx.json {
        ctx.out.json_value(&MutationDoc {
            schema: SCHEMA,
            ok: true,
            action: "pull",
            key: None,
            environments: vec![env],
            file: Some(&path),
            count: Some(vars.len()),
        })?;
    } else {
        ctx.ui.success(format!(
            "Created {display} file{} ({} var{})",
            if ignored {
                " and added it to .gitignore"
            } else {
                ""
            },
            vars.len(),
            if vars.len() == 1 { "" } else { "s" }
        ));
        ctx.out.line(path.display().to_string());
    }
    Ok(ExitCode::SUCCESS)
}

fn run_cmd(
    ctx: &mut Ctx,
    store: &EnvStore,
    environment: &str,
    command: &[String],
) -> anyhow::Result<ExitCode> {
    let env = parse_env(environment)?;
    let Some((program, rest)) = command.split_first() else {
        anyhow::bail!("nothing to run\n  usage: gozo env run [-e ENV] -- CMD [ARGS...]");
    };
    let vars = store.list(env)?;
    ctx.ui.debug(format!(
        "{} ({} vars from {env})",
        command.join(" "),
        vars.len()
    ));
    let status = Command::new(program)
        .args(rest)
        .envs(vars.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .current_dir(&ctx.cwd)
        .status()
        .with_context(|| format!("could not run `{program}`"))?;
    Ok(exit_code(status))
}

pub fn exit_code(status: std::process::ExitStatus) -> ExitCode {
    match status.code() {
        Some(c) => ExitCode::from(c.clamp(0, 255) as u8),
        None => ExitCode::from(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_exactly_one_newline() {
        assert_eq!(strip_one_newline("secret\n"), "secret");
        assert_eq!(strip_one_newline("secret\r\n"), "secret");
        assert_eq!(strip_one_newline("secret\n\n"), "secret\n");
        assert_eq!(strip_one_newline("secret"), "secret");
    }

    #[test]
    fn envs_default_to_all() {
        assert_eq!(envs_from(None).unwrap().len(), 3);
        assert_eq!(
            envs_from(Some("prod")).unwrap(),
            vec![Environment::Production]
        );
        assert!(envs_from(Some("nope")).is_err());
    }
}
