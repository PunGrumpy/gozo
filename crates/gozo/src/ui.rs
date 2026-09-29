//! Human-facing output in the style of Vercel CLI.
//!
//! Rules:
//! - Everything here goes to **stderr**. stdout is reserved for the primary
//!   result of a command (a URL, a file path, JSON) so it can be piped.
//! - `> ` prefixes progress steps, `✓` success, `!` warnings, `Error:` failures.
//! - Prompts honor `--yes` (take the default) and `--non-interactive` (fail).

use std::io::{IsTerminal, Write};
use std::time::Instant;

use anyhow::{Result, anyhow};
use owo_colors::{OwoColorize, Stream::Stderr};

#[derive(Debug, Clone)]
pub struct Ui {
    pub color: bool,
    pub interactive: bool,
    pub yes: bool,
    pub debug: bool,
    /// JSON mode: suppress all chatter so stdout stays clean and stderr quiet.
    pub quiet: bool,
}

impl Ui {
    pub fn new(color: bool, interactive: bool, yes: bool, debug: bool, quiet: bool) -> Self {
        let tty = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
        let agent = ["CLAUDECODE", "CURSOR_AGENT", "CODEX_SANDBOX", "AGENT", "CI"]
            .iter()
            .any(|k| {
                std::env::var_os(k).is_some_and(|v| !v.is_empty() && v != "0" && v != "false")
            });
        if !color {
            owo_colors::set_override(false);
        }
        Ui {
            color,
            interactive: interactive && tty && !agent,
            yes,
            debug,
            quiet,
        }
    }

    fn line(&self, s: String) {
        if self.quiet {
            return;
        }
        let _ = writeln!(std::io::stderr(), "{s}");
    }

    /// `gozo CLI 0.1.0`, printed once at the top like Vercel does.
    pub fn header(&self) {
        self.line(format!(
            "{}",
            format!("gozo CLI {}", env!("CARGO_PKG_VERSION"))
                .if_supports_color(Stderr, |t| t.dimmed())
        ));
    }

    pub fn step(&self, msg: impl AsRef<str>) {
        self.line(format!(
            "{} {}",
            ">".if_supports_color(Stderr, |t| t.dimmed()),
            msg.as_ref()
        ));
    }

    pub fn success(&self, msg: impl AsRef<str>) {
        let mark = if self.color { "✓" } else { "OK" };
        self.line(format!(
            "{} {}",
            mark.if_supports_color(Stderr, |t| t.green()),
            msg.as_ref()
        ));
    }

    pub fn warn(&self, msg: impl AsRef<str>) {
        self.line(format!(
            "{} {}",
            "WARN!".if_supports_color(Stderr, |t| t.yellow()),
            msg.as_ref()
        ));
    }

    pub fn error(&self, msg: impl AsRef<str>) {
        let _ = writeln!(
            std::io::stderr(),
            "{} {}",
            "Error:".if_supports_color(Stderr, |t| t.red()),
            msg.as_ref()
        );
    }

    pub fn info(&self, msg: impl AsRef<str>) {
        self.line(msg.as_ref().to_owned());
    }

    /// Secondary line under a step.
    pub fn detail(&self, msg: impl AsRef<str>) {
        self.line(format!(
            "  {}",
            msg.as_ref().if_supports_color(Stderr, |t| t.dimmed())
        ));
    }

    /// Only shown with --debug.
    pub fn debug(&self, msg: impl AsRef<str>) {
        if self.debug {
            self.line(format!(
                "  {}",
                format!("$ {}", msg.as_ref()).if_supports_color(Stderr, |t| t.dimmed())
            ));
        }
    }

    /// Aligned `label   value` pair.
    pub fn kv(&self, label: &str, value: impl AsRef<str>) {
        self.line(format!(
            "  {} {}",
            format!("{label:<11}").if_supports_color(Stderr, |t| t.dimmed()),
            value.as_ref()
        ));
    }

    /// A blank line.
    pub fn blank(&self) {
        self.line(String::new());
    }

    pub fn hint(&self, msg: impl AsRef<str>) {
        self.line(format!(
            "  {} {}",
            "hint:".if_supports_color(Stderr, |t| t.dimmed()),
            msg.as_ref()
        ));
    }

    pub fn bold(&self, s: &str) -> String {
        s.if_supports_color(Stderr, |t| t.bold()).to_string()
    }

    pub fn dim(&self, s: &str) -> String {
        s.if_supports_color(Stderr, |t| t.dimmed()).to_string()
    }

    pub fn cyan(&self, s: &str) -> String {
        s.if_supports_color(Stderr, |t| t.cyan()).to_string()
    }

    fn non_interactive_err(&self, what: &str) -> anyhow::Error {
        anyhow!(
            "{what} requires a terminal\n  pass --yes to accept defaults, or supply the value with a flag"
        )
    }

    /// Yes/no question. `--yes` returns `default`; non-interactive errors.
    pub fn confirm(&self, question: &str, default: bool) -> Result<bool> {
        if self.yes {
            return Ok(default);
        }
        if !self.interactive {
            return Err(self.non_interactive_err(&format!("confirming {question:?}")));
        }
        Ok(dialoguer::Confirm::new()
            .with_prompt(format!("? {question}"))
            .default(default)
            .interact()?)
    }

    /// Pick one item. `--yes` picks `default`; non-interactive errors.
    pub fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize> {
        if self.yes {
            return Ok(default.min(items.len().saturating_sub(1)));
        }
        if !self.interactive {
            return Err(self.non_interactive_err(&format!("choosing {prompt:?}")));
        }
        Ok(dialoguer::Select::new()
            .with_prompt(format!("? {prompt}"))
            .items(items)
            .default(default)
            .interact()?)
    }

    /// Free text. `--yes` returns `default` when there is one.
    pub fn input(&self, prompt: &str, default: Option<&str>) -> Result<String> {
        if self.yes {
            if let Some(d) = default {
                return Ok(d.to_owned());
            }
        }
        if !self.interactive {
            return match default {
                Some(d) => Ok(d.to_owned()),
                None => Err(self.non_interactive_err(&format!("entering {prompt:?}"))),
            };
        }
        let mut q = dialoguer::Input::<String>::new().with_prompt(format!("? {prompt}"));
        if let Some(d) = default {
            q = q.default(d.to_owned());
        }
        Ok(q.interact_text()?)
    }

    /// Hidden text (secrets).
    pub fn password(&self, prompt: &str) -> Result<String> {
        if !self.interactive {
            return Err(self.non_interactive_err(&format!("entering {prompt:?}")));
        }
        Ok(dialoguer::Password::new()
            .with_prompt(format!("? {prompt}"))
            .interact()?)
    }
}

/// Elapsed-time suffix like Vercel's `[2s]`.
pub struct Timer(Instant);

impl Timer {
    pub fn start() -> Self {
        Timer(Instant::now())
    }

    pub fn elapsed(&self) -> String {
        let ms = self.0.elapsed().as_millis();
        if ms < 1000 {
            format!("[{ms}ms]")
        } else {
            format!("[{:.1}s]", ms as f64 / 1000.0)
        }
    }
}

/// Bridge so `gozo-deploy` adapters can narrate through the UI.
pub struct UiReporter<'a>(pub &'a Ui);

impl gozo_deploy::Reporter for UiReporter<'_> {
    fn step(&mut self, msg: &str) {
        self.0.step(msg);
    }
    fn command(&mut self, cmd: &str) {
        self.0.debug(cmd);
    }
    fn detail(&mut self, msg: &str) {
        self.0.detail(msg);
    }
    fn warn(&mut self, msg: &str) {
        self.0.warn(msg);
    }
}
