//! Events emitted by `go test -json` (the `test2json` format).
//!
//! One JSON object per line. Go 1.24+ additionally emits `build-output` and
//! `build-fail` events keyed by `ImportPath` rather than `Package`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TestEvent {
    #[serde(default)]
    pub time: Option<String>,
    pub action: String,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub test: Option<String>,
    /// Seconds, present on `pass`/`fail`/`skip`.
    #[serde(default)]
    pub elapsed: Option<f64>,
    #[serde(default)]
    pub output: Option<String>,
    /// Set on `build-output`/`build-fail` events (Go 1.24+).
    #[serde(default)]
    pub import_path: Option<String>,
    /// Set on a package `fail` event when the failure was a build error.
    #[serde(default)]
    pub failed_build: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestAction {
    Start,
    Run,
    Pause,
    Cont,
    Pass,
    Bench,
    Fail,
    Skip,
    Output,
    BuildOutput,
    BuildFail,
    Other,
}

impl TestEvent {
    /// Parse one line of `go test -json` output. Returns `None` for lines that
    /// are not events (older Go versions print build errors as plain text).
    pub fn parse(line: &str) -> Option<TestEvent> {
        let line = line.trim();
        if !line.starts_with('{') {
            return None;
        }
        serde_json::from_str(line).ok()
    }

    pub fn action(&self) -> TestAction {
        match self.action.as_str() {
            "start" => TestAction::Start,
            "run" => TestAction::Run,
            "pause" => TestAction::Pause,
            "cont" => TestAction::Cont,
            "pass" => TestAction::Pass,
            "bench" => TestAction::Bench,
            "fail" => TestAction::Fail,
            "skip" => TestAction::Skip,
            "output" => TestAction::Output,
            "build-output" => TestAction::BuildOutput,
            "build-fail" => TestAction::BuildFail,
            _ => TestAction::Other,
        }
    }

    /// The package this event is about. Build events carry `ImportPath`, which
    /// may be suffixed like `pkg [pkg.test]` for test binaries.
    pub fn package_path(&self) -> Option<&str> {
        self.package.as_deref().or_else(|| {
            self.import_path
                .as_deref()
                .map(|p| p.split(" [").next().unwrap_or(p))
        })
    }

    /// `Output` with the trailing newline removed.
    pub fn output_line(&self) -> Option<&str> {
        self.output
            .as_deref()
            .map(|o| o.trim_end_matches(['\n', '\r']))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_test_events() {
        let ev = TestEvent::parse(
            r#"{"Time":"2026-09-29T15:29:55.83Z","Action":"fail","Package":"example.com/m/calc","Test":"TestSub","Elapsed":0.01}"#,
        )
        .expect("event");
        assert_eq!(ev.action(), TestAction::Fail);
        assert_eq!(ev.package_path(), Some("example.com/m/calc"));
        assert_eq!(ev.test.as_deref(), Some("TestSub"));
        assert_eq!(ev.elapsed, Some(0.01));
    }

    #[test]
    fn parses_output_and_strips_newline() {
        let ev = TestEvent::parse(
            r#"{"Action":"output","Package":"p","Test":"T","Output":"    x_test.go:3: boom\n","OutputType":"error"}"#,
        )
        .expect("event");
        assert_eq!(ev.action(), TestAction::Output);
        assert_eq!(ev.output_line(), Some("    x_test.go:3: boom"));
    }

    #[test]
    fn parses_build_events_with_import_path() {
        let ev = TestEvent::parse(
            r#"{"ImportPath":"example.com/m/bad [example.com/m/bad.test]","Action":"build-fail"}"#,
        )
        .expect("event");
        assert_eq!(ev.action(), TestAction::BuildFail);
        assert_eq!(ev.package_path(), Some("example.com/m/bad"));
    }

    #[test]
    fn rejects_non_json_lines() {
        assert!(TestEvent::parse("# example.com/m/bad").is_none());
        assert!(TestEvent::parse("").is_none());
        assert!(TestEvent::parse("{not json").is_none());
    }

    #[test]
    fn unknown_actions_are_other() {
        let ev = TestEvent::parse(r#"{"Action":"something-new","Package":"p"}"#).expect("event");
        assert_eq!(ev.action(), TestAction::Other);
    }
}
