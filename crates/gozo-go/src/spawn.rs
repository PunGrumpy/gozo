//! Building a `std::process::Command` for `go`, for callers that need to
//! stream output or inherit stdio instead of collecting it.

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use crate::command::Go;

impl Go {
    /// A `go <args>` command rooted in `dir`, ready for the caller to
    /// configure stdio and spawn.
    pub fn command<I, S>(&self, dir: &Path, args: I) -> Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new(&self.bin);
        cmd.args(args).current_dir(dir);
        cmd
    }
}
