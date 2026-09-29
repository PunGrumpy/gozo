//! Colors and status marks for **stdout** reports (doctor, check, test, build, ...).
//!
//! `ui.rs` styles stderr chatter; this module styles the primary result that
//! goes to stdout. Both honor `--no-color` / `NO_COLOR` through owo-colors'
//! global override, and `mark` degrades to plain ASCII words without color.

use owo_colors::{OwoColorize, Stream::Stdout};

pub fn green(s: &str) -> String {
    s.if_supports_color(Stdout, |t| t.green()).to_string()
}

pub fn red(s: &str) -> String {
    s.if_supports_color(Stdout, |t| t.red()).to_string()
}

pub fn yellow(s: &str) -> String {
    s.if_supports_color(Stdout, |t| t.yellow()).to_string()
}

pub fn dim(s: &str) -> String {
    s.if_supports_color(Stdout, |t| t.dimmed()).to_string()
}

pub fn bold(s: &str) -> String {
    s.if_supports_color(Stdout, |t| t.bold()).to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Ok,
    Fail,
    Skip,
    Warn,
}

/// `✓`/`✗`/`-`/`!` with color, plain ASCII words without (matches `Ui`).
pub fn mark(color: bool, which: Mark) -> String {
    match (which, color) {
        (Mark::Ok, true) => green("✓"),
        (Mark::Ok, false) => "OK".to_owned(),
        (Mark::Fail, true) => red("✗"),
        (Mark::Fail, false) => "FAIL".to_owned(),
        (Mark::Skip, true) => dim("-"),
        (Mark::Skip, false) => "-".to_owned(),
        (Mark::Warn, true) => yellow("!"),
        (Mark::Warn, false) => "!".to_owned(),
    }
}
