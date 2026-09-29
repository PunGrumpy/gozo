use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use crate::command::Go;

impl Go {
    /// For callers that stream or inherit stdio instead of collecting it.
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
