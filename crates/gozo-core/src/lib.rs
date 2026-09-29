//! Project-level state: `gozo.toml`, the `.gozo/` link directory, env files and deployment history.

pub mod config;
pub mod deployments;
pub mod dotenv;
pub mod envstore;
pub mod git;
pub mod link;
pub mod packages;

pub use config::{
    BuildSection, Config, DeploySection, DevSection, DockerTarget, KubernetesTarget, Task,
};
pub use deployments::{Deployment, DeploymentHistory};
pub use envstore::{EnvStore, Environment};
pub use link::{Link, Target};
pub use packages::MainPackage;

pub const STATE_DIR: &str = ".gozo";

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("invalid {path}: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Go(#[from] gozo_go::GoError),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
