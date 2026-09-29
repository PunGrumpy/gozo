//! Typed wrapper around the `go` command; every project fact comes from `go`'s own JSON output.

mod command;
mod gomod;
mod modules;
mod packages;
mod project;
mod spawn;
mod testjson;
mod version;
mod vet;

use serde::Deserialize as _;

pub use command::{Go, GoError, GoOutput};
pub use gomod::{GoMod, GoWork, ModuleRef, Replace, Require, Tool, Use};
pub use modules::{ModuleInfo, ModuleUpdate, TidyStatus};
pub use packages::{PackageInfo, PackageModule};
pub use project::{Project, ProjectKind, ProjectModule};
pub use testjson::{TestAction, TestEvent};
pub use version::GoVersion;
pub use vet::{VetIssue, VetReport, parse_vet_json, split_posn};

pub type Result<T> = std::result::Result<T, GoError>;

/// Go 1.25 emits list fields as explicit `null` (`"Require": null`); treat that as the default.
pub(crate) fn null_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
