use std::path::{Path, PathBuf};
use std::process::{ExitCode, ExitStatus};

pub fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// `path` relative to `root` with `/` separators; `.` for the root itself.
pub fn display_rel(root: &Path, path: &Path) -> String {
    let p = path.strip_prefix(root).unwrap_or(path);
    let s = p.to_string_lossy().replace('\\', "/");
    if s.is_empty() { ".".to_owned() } else { s }
}

pub fn exit_for(ok: bool) -> ExitCode {
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Propagate a child's exit status; a signal death becomes 1.
pub fn exit_code(status: ExitStatus) -> ExitCode {
    match status.code() {
        Some(c) => ExitCode::from(c.clamp(0, 255) as u8),
        None => ExitCode::from(1),
    }
}

pub fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}

pub fn which(name: &str) -> Option<PathBuf> {
    let exe = exe_name(name);
    std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p)
            .map(|d| d.join(&exe))
            .find(|c| c.is_file())
    })
}
