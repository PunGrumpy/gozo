//! Read-only facts from the `git` binary; `None`/empty when git is missing or this is not a repo.

use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if s.is_empty() { None } else { Some(s) }
}

pub fn is_repo(root: &Path) -> bool {
    git(root, &["rev-parse", "--is-inside-work-tree"]).as_deref() == Some("true")
}

pub fn sha(root: &Path) -> Option<String> {
    git(root, &["rev-parse", "HEAD"])
}

pub fn short_sha(root: &Path) -> Option<String> {
    git(root, &["rev-parse", "--short", "HEAD"])
}

pub fn branch(root: &Path) -> Option<String> {
    git(root, &["rev-parse", "--abbrev-ref", "HEAD"]).filter(|b| b != "HEAD")
}

pub fn remote_url(root: &Path) -> Option<String> {
    git(root, &["remote", "get-url", "origin"])
}

pub fn describe(root: &Path) -> Option<String> {
    git(root, &["describe", "--tags", "--always", "--dirty"])
}

pub fn is_dirty(root: &Path) -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

pub fn changed_paths(root: &Path) -> Vec<String> {
    // Not via `git()`: trimming would eat the first line's leading status column.
    let Ok(out) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.len() > 3)
        .map(|l| l[3..].trim().to_owned())
        .collect()
}
