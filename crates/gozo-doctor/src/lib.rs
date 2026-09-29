//! Health checks for a Go project. Checks never panic or abort the run; one
//! that cannot decide reports [`Status::Skip`] with a reason.

mod checks;
mod report;

pub use report::{Finding, Report, Status, Summary};

use gozo_go::{Go, Project};

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Skip checks that need the network (dependency updates).
    pub offline: bool,
}

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
