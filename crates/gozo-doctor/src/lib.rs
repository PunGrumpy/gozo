//! Health checks for a Go project.
//!
//! Every check produces a [`Finding`]. Checks never panic and never abort the
//! run; a check that cannot decide reports [`Status::Skip`] with a reason.

mod checks;
mod report;

pub use report::{Finding, Report, Status, Summary};

use gozo_go::{Go, GoVersion, Project};

/// Options controlling which checks run.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Skip checks that need the network (dependency updates).
    pub offline: bool,
}

/// Run every check against `project` and collect a report.
pub fn run(go: &Go, project: &Project, opts: &Options) -> Report {
    let mut cx = checks::Context::new(go, project, opts);

    checks::toolchain(&mut cx);
    checks::gomod_valid(&mut cx);
    checks::go_directive(&mut cx);
    checks::workspace(&mut cx);
    checks::tidy(&mut cx);
    checks::replace_outside(&mut cx);
    checks::outdated(&mut cx);
    checks::tools(&mut cx);
    checks::cgo(&mut cx);

    cx.finish()
}

/// Version of the installed toolchain, when one was found.
pub fn go_version(go: &Go) -> Option<GoVersion> {
    go.version().ok()
}
