//! Typed wrapper around the `go` command.
//!
//! gozo never re-implements Go tooling. Every fact about a project comes from
//! the `go` command itself, which already exposes JSON for the pieces we need:
//! `go env -json`, `go mod edit -json`, `go work edit -json`, `go list -m -json`.

mod command;
mod gomod;
mod modules;
mod packages;
mod project;
mod spawn;
mod testjson;
mod version;
mod vet;

pub use command::{Go, GoError, GoOutput};
pub use gomod::{GoMod, GoWork, ModuleRef, Replace, Require, Tool, Use};
pub use modules::{ModuleInfo, ModuleUpdate, TidyStatus};
pub use packages::{PackageInfo, PackageModule};
pub use project::{Project, ProjectKind, ProjectModule};
pub use testjson::{TestAction, TestEvent};
pub use version::GoVersion;
pub use vet::{VetIssue, VetReport, parse_vet_json, split_posn};

pub type Result<T> = std::result::Result<T, GoError>;

/// Deserialize a field that older `go` versions emit as an explicit `null`
/// (for example `"Require": null` from Go 1.25) as its default value.
pub(crate) fn null_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

use serde::Deserialize as _;
