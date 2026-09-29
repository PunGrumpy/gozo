use std::io::{BufRead, BufReader};
use std::process::{Command, ExitCode, ExitStatus, Stdio};

use anyhow::Context as _;
use chrono::{DateTime, SecondsFormat, Utc};
use gozo_deploy::LogOptions;
use serde::Serialize;

use super::util::exit_code;
use crate::ctx::Ctx;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Keep streaming new lines until interrupted.
    #[arg(short = 'f', long)]
    pub follow: bool,

    /// Only lines newer than this: `1h`, `30m`, or an RFC3339 timestamp.
    #[arg(long, value_name = "DURATION")]
    pub since: Option<String>,

    /// Maximum number of lines from the end of the log.
    #[arg(short = 'n', long, value_name = "N")]
    pub limit: Option<usize>,

    /// Container to read from (pods with several containers).
    #[arg(short = 'c', long, value_name = "NAME")]
    pub container: Option<String>,

    /// Prefix every line with its timestamp.
    #[arg(short = 't', long)]
    pub timestamps: bool,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let root = ctx.root()?.to_path_buf();
    let link = ctx.link_required()?;
    let adapter = gozo_deploy::adapter_for(link, &ctx.config, &root)?;
    let opts = LogOptions {
        follow: args.follow,
        since: args.since.clone(),
        limit: args.limit,
        container: args.container.clone(),
        timestamps: args.timestamps,
    };
    let mut cmd = adapter.logs_command(&opts)?;
    let display = display_command(&cmd);
    let tool = cmd.get_program().to_string_lossy().into_owned();

    ctx.ui.step(format!(
        "{} logs from {}",
        if args.follow { "Streaming" } else { "Fetching" },
        adapter.describe()
    ));
    ctx.ui.debug(&display);

    let status = if ctx.json {
        stream_json(ctx, cmd, args.timestamps, &tool)?
    } else {
        cmd.stdin(Stdio::null());
        let status = cmd
            .status()
            .with_context(|| format!("could not run `{display}` (is {tool} installed?)"))?;
        Some(status)
    };

    Ok(match status {
        // stdout closed early (`| head`): the reader got what it wanted.
        None => ExitCode::SUCCESS,
        Some(s) => exit_code(s),
    })
}

/// Re-emit each line as `gozo.log/v1` JSON Lines; `None` when stdout closed
/// before the tool finished.
fn stream_json(
    ctx: &Ctx,
    mut cmd: Command,
    timestamps: bool,
    tool: &str,
) -> anyhow::Result<Option<ExitStatus>> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("could not run {tool} (is it installed?)"))?;
    let Some(stdout) = child.stdout.take() else {
        anyhow::bail!("could not capture {tool} output");
    };
    for line in BufReader::new(stdout).lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let (ts, text) = split_timestamp(&line, timestamps, Utc::now());
        let doc = LogLine {
            schema: "gozo.log/v1",
            ts,
            line: text,
        };
        if ctx.out.json_line(&doc).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
    }
    Ok(Some(child.wait()?))
}

/// With `--timestamps` the tools prefix `RFC3339 <space> line`; otherwise
/// (or when it does not parse) the line is stamped with `now`.
fn split_timestamp(line: &str, timestamps: bool, now: DateTime<Utc>) -> (String, String) {
    if timestamps {
        if let Some((ts, rest)) = line.split_once(' ') {
            if let Ok(t) = DateTime::parse_from_rfc3339(ts) {
                return (rfc3339(t.with_timezone(&Utc)), rest.to_owned());
            }
        }
    }
    (rfc3339(now), line.to_owned())
}

fn rfc3339(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn display_command(cmd: &Command) -> String {
    let mut parts = vec![cmd.get_program().to_string_lossy().into_owned()];
    parts.extend(cmd.get_args().map(|a| a.to_string_lossy().into_owned()));
    parts.join(" ")
}

#[derive(Serialize)]
struct LogLine {
    schema: &'static str,
    ts: String,
    line: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn splits_tool_timestamps() {
        let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
        let (ts, line) = split_timestamp("2026-09-29T10:00:00.123456789Z hello world", true, now);
        assert_eq!(ts, "2026-09-29T10:00:00.123Z");
        assert_eq!(line, "hello world");

        let (ts, _) = split_timestamp("2026-09-29T12:00:00+02:00 x", true, now);
        assert_eq!(ts, "2026-09-29T10:00:00.000Z");

        let (ts, line) = split_timestamp("plain line", true, now);
        assert_eq!(ts, "2026-09-29T12:00:00.000Z");
        assert_eq!(line, "plain line");

        let (ts, line) = split_timestamp("2026-09-29T10:00:00Z x", false, now);
        assert_eq!(ts, "2026-09-29T12:00:00.000Z");
        assert_eq!(line, "2026-09-29T10:00:00Z x");
    }

    #[test]
    fn command_display() {
        let mut c = Command::new("docker");
        c.args(["logs", "--tail", "10", "api"]);
        assert_eq!(display_command(&c), "docker logs --tail 10 api");
    }
}
