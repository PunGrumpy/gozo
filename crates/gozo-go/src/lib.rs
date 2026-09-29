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
