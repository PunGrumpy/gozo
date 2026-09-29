use std::path::PathBuf;

use serde::Serialize;

/// Schema identifier written into JSON output so agents can detect changes.
pub const SCHEMA: &str = "gozo.doctor/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Stable machine identifier, e.g. `go.toolchain`.
    pub id: &'static str,
    pub status: Status,
    /// One line, human readable.
    pub title: String,
    /// Extra lines shown under the title.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
    /// A suggested command or action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl Finding {
    pub fn new(id: &'static str, status: Status, title: impl Into<String>) -> Self {
        Finding {
            id,
            status,
            title: title.into(),
            details: Vec::new(),
            hint: None,
        }
    }

    /// Add detail text. Multi-line input becomes one entry per line so
    /// renderers can indent consistently.
    pub fn detail(mut self, line: impl Into<String>) -> Self {
        let text: String = line.into();
        self.details.extend(
            text.lines()
                .map(str::trim_end)
                .filter(|l| !l.is_empty())
                .map(str::to_owned),
        );
        self
    }

    pub fn details<I, S>(mut self, lines: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for l in lines {
            self = self.detail(l);
        }
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub ok: usize,
    pub warn: usize,
    pub fail: usize,
    pub skip: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSummary {
    pub name: String,
    pub kind: gozo_go::ProjectKind,
    pub root: PathBuf,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolchainSummary {
    pub bin: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub project: ProjectSummary,
    pub go: ToolchainSummary,
    pub findings: Vec<Finding>,
    pub summary: Summary,
}

impl Report {
    pub fn healthy(&self) -> bool {
        self.summary.fail == 0
    }
}
