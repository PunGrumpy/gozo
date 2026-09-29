mod commands;
mod ctx;
mod output;
mod style;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// The missing developer experience layer for Go.
#[derive(Debug, Parser)]
#[command(
    name = "gozo",
    version,
    about,
    long_about = None,
    disable_help_subcommand = true,
    after_help = "Run `gozo` with no command for an interactive menu.\nEvery command accepts --json for machine-readable output."
)]
pub struct Cli {
    /// Emit machine-readable JSON on stdout instead of human output.
    #[arg(long, global = true)]
    pub json: bool,

    /// Run as if started in this directory.
    #[arg(short = 'C', long = "cwd", global = true, value_name = "DIR")]
    pub cwd: Option<PathBuf>,

    /// Answer every prompt with its default.
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

    /// Never prompt; fail instead. Implied when stdin is not a terminal or an agent is detected.
    #[arg(long, global = true)]
    pub non_interactive: bool,

    /// Disable colors and symbols. NO_COLOR=1 does the same.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Show the underlying go/docker/kubectl commands as they run.
    #[arg(short = 'd', long, global = true)]
    pub debug: bool,

    /// Project name to use instead of the linked or configured one.
    #[arg(long, global = true, env = "GOZO_PROJECT", value_name = "NAME")]
    pub project: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create gozo.toml (and a Go module if needed) in a directory.
    Init(commands::init::Args),
    /// Link this directory to a deployment target.
    Link(commands::link::Args),
    /// Remove the link created by `gozo link`.
    Unlink(commands::link::UnlinkArgs),
    /// Run the app locally with env files loaded and restart on change.
    Dev(commands::dev::Args),
    /// Manage environment variables (ls, add, rm, update, pull, run).
    Env(commands::env::Args),
    /// Check the health of the current Go project.
    Doctor(commands::doctor::Args),
    /// Run vet, format and lint checks across every module.
    Check(commands::check::Args),
    /// Run tests across every module with a unified summary.
    Test(commands::test::Args),
    /// Build every main package with version info injected.
    Build(commands::build::Args),
    /// Run `go generate` across every module, optionally verifying nothing changed.
    Generate(commands::generate::Args),
    /// Run a task defined in gozo.toml.
    Run(commands::run::Args),
    /// Build and deploy to the linked target.
    Deploy(commands::deploy::Args),
    /// List recent deployments.
    #[command(alias = "list")]
    Ls(commands::ls::Args),
    /// Show the state of the linked target.
    Status(commands::status::Args),
    /// Show or stream logs from the linked target.
    Logs(commands::logs::Args),
    /// Roll the linked target back to a previous deployment.
    Rollback(commands::rollback::Args),
    /// Manage tools declared with `tool` directives in go.mod.
    Tool(commands::tool::Args),
    /// Update gozo itself.
    Update(commands::update::Args),
    /// Query project state as JSON, for scripts and agents.
    Api(commands::api::Args),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut ctx = match ctx::Ctx::load(&cli) {
        Ok(c) => c,
        Err(err) => {
            output::Output::new(cli.json).error(&err);
            return ExitCode::from(2);
        }
    };

    let result = match cli.command {
        None => commands::home::run(&mut ctx),
        Some(Command::Init(a)) => commands::init::run(&mut ctx, a),
        Some(Command::Link(a)) => commands::link::run(&mut ctx, a),
        Some(Command::Unlink(a)) => commands::link::unlink(&mut ctx, a),
        Some(Command::Dev(a)) => commands::dev::run(&mut ctx, a),
        Some(Command::Env(a)) => commands::env::run(&mut ctx, a),
        Some(Command::Doctor(a)) => commands::doctor::run(&mut ctx, a),
        Some(Command::Check(a)) => commands::check::run(&mut ctx, a),
        Some(Command::Test(a)) => commands::test::run(&mut ctx, a),
        Some(Command::Build(a)) => commands::build::run(&mut ctx, a),
        Some(Command::Generate(a)) => commands::generate::run(&mut ctx, a),
        Some(Command::Run(a)) => commands::run::run(&mut ctx, a),
        Some(Command::Deploy(a)) => commands::deploy::run(&mut ctx, a),
        Some(Command::Ls(a)) => commands::ls::run(&mut ctx, a),
        Some(Command::Status(a)) => commands::status::run(&mut ctx, a),
        Some(Command::Logs(a)) => commands::logs::run(&mut ctx, a),
        Some(Command::Rollback(a)) => commands::rollback::run(&mut ctx, a),
        Some(Command::Tool(a)) => commands::tool::run(&mut ctx, a),
        Some(Command::Update(a)) => commands::update::run(&mut ctx, a),
        Some(Command::Api(a)) => commands::api::run(&mut ctx, a),
    };

    match result {
        Ok(code) => code,
        Err(err) => {
            ctx.out.error(&err);
            ExitCode::from(2)
        }
    }
}
