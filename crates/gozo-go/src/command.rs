use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::version::GoVersion;

#[derive(Debug, thiserror::Error)]
pub enum GoError {
    #[error("`go` was not found on PATH")]
    NotFound,
    #[error("failed to run `go {args}`: {source}")]
    Spawn {
        args: String,
        #[source]
        source: std::io::Error,
    },
    #[error("`go {args}` exited with {status}:\n{stderr}")]
    Failed {
        args: String,
        status: i32,
        stderr: String,
    },
    #[error("could not parse output of `go {args}`: {source}")]
    Parse {
        args: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("could not parse Go version from {0:?}")]
    Version(String),
    #[error("no go.mod or go.work found in {0} or any parent directory")]
    NotAProject(PathBuf),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Raw result of a `go` invocation. Callers that need to tolerate failure
/// (for example an offline `go mod tidy -diff`) inspect this directly.
#[derive(Debug, Clone)]
pub struct GoOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl GoOutput {
    pub fn success(&self) -> bool {
        self.status == 0
    }
}

/// A located `go` binary.
#[derive(Debug, Clone)]
pub struct Go {
    pub bin: PathBuf,
}

impl Go {
    /// Locate `go` on PATH, or under `$GOROOT/bin` as a fallback.
    pub fn find() -> Result<Go, GoError> {
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path) {
                let candidate = dir.join(exe("go"));
                if candidate.is_file() {
                    return Ok(Go { bin: candidate });
                }
            }
        }
        if let Some(root) = std::env::var_os("GOROOT") {
            let candidate = Path::new(&root).join("bin").join(exe("go"));
            if candidate.is_file() {
                return Ok(Go { bin: candidate });
            }
        }
        Err(GoError::NotFound)
    }

    /// Run `go <args>` in `dir`. Never fails on a non-zero exit; see [`Self::run_ok`].
    pub fn run<I, S>(&self, dir: &Path, args: I) -> Result<GoOutput, GoError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_env(dir, args, &[])
    }

    /// Like [`Self::run`] but with extra environment variables.
    pub fn run_env<I, S>(
        &self,
        dir: &Path,
        args: I,
        env: &[(&str, &str)],
    ) -> Result<GoOutput, GoError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<String> = args
            .into_iter()
            .map(|a| a.as_ref().to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        let mut cmd = Command::new(&self.bin);
        cmd.args(&args).current_dir(dir);
        for (k, v) in env {
            cmd.env(k, v);
        }
        let out = cmd.output().map_err(|source| GoError::Spawn {
            args: joined,
            source,
        })?;
        Ok(GoOutput {
            status: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }

    /// Run `go <args>` with `GOTOOLCHAIN=local`, for commands that only read
    /// files (`env`, `mod edit`, `work edit`, `version`) and must never start
    /// a toolchain download just because go.mod asks for a newer Go.
    pub fn run_local<I, S>(&self, dir: &Path, args: I) -> Result<GoOutput, GoError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_env(dir, args, &[("GOTOOLCHAIN", "local")])
    }

    /// Run `go <args>` and turn a non-zero exit into an error.
    pub fn run_ok<I, S>(&self, dir: &Path, args: I) -> Result<GoOutput, GoError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<String> = args
            .into_iter()
            .map(|a| a.as_ref().to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        let out = self.run(dir, &args)?;
        if out.success() {
            Ok(out)
        } else {
            Err(GoError::Failed {
                args: joined,
                status: out.status,
                stderr: out.stderr.trim().to_owned(),
            })
        }
    }

    /// Run `go <args>` and deserialize stdout as a single JSON document.
    pub fn run_json<T, I, S>(&self, dir: &Path, args: I) -> Result<T, GoError>
    where
        T: serde::de::DeserializeOwned,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<String> = args
            .into_iter()
            .map(|a| a.as_ref().to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        let out = self.run_ok(dir, &args)?;
        serde_json::from_str(&out.stdout).map_err(|source| GoError::Parse {
            args: joined,
            source,
        })
    }

    /// Like [`Self::run_json`] but via [`Self::run_local`].
    pub fn run_json_local<T, I, S>(&self, dir: &Path, args: I) -> Result<T, GoError>
    where
        T: serde::de::DeserializeOwned,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<String> = args
            .into_iter()
            .map(|a| a.as_ref().to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        let out = self.run_local(dir, &args)?;
        if !out.success() {
            return Err(GoError::Failed {
                args: joined,
                status: out.status,
                stderr: out.stderr.trim().to_owned(),
            });
        }
        serde_json::from_str(&out.stdout).map_err(|source| GoError::Parse {
            args: joined,
            source,
        })
    }

    /// Run `go <args>` and deserialize stdout as a stream of concatenated JSON
    /// documents, which is how `go list -json` reports multiple items.
    pub fn run_json_stream<T, I, S>(&self, dir: &Path, args: I) -> Result<Vec<T>, GoError>
    where
        T: serde::de::DeserializeOwned,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<String> = args
            .into_iter()
            .map(|a| a.as_ref().to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        let out = self.run_ok(dir, &args)?;
        serde_json::Deserializer::from_str(&out.stdout)
            .into_iter::<T>()
            .collect::<std::result::Result<Vec<T>, _>>()
            .map_err(|source| GoError::Parse {
                args: joined,
                source,
            })
    }

    /// `go version`, parsed.
    pub fn version(&self) -> Result<GoVersion, GoError> {
        let out = self.run_local(Path::new("."), ["version"])?;
        if !out.success() {
            return Err(GoError::Failed {
                args: "version".to_owned(),
                status: out.status,
                stderr: out.stderr.trim().to_owned(),
            });
        }
        GoVersion::parse_go_version_output(&out.stdout)
    }

    /// `go env -json` evaluated in `dir`, so GOMOD and GOWORK reflect the project.
    ///
    /// Runs with `GOTOOLCHAIN=local` so a go.mod that asks for a newer Go does
    /// not trigger a download; the reported `GOTOOLCHAIN` value is then
    /// restored from [`Self::toolchain_mode`] so callers see the real setting.
    pub fn env(&self, dir: &Path) -> Result<BTreeMap<String, String>, GoError> {
        let mut env: BTreeMap<String, String> = self.run_json_local(dir, ["env", "-json"])?;
        if let Ok(mode) = self.toolchain_mode() {
            env.insert("GOTOOLCHAIN".to_owned(), mode);
        }
        Ok(env)
    }

    /// The effective `GOTOOLCHAIN` setting (`auto`, `local`, `go1.x`, ...),
    /// read from a directory with no go.mod so it can never start a download.
    pub fn toolchain_mode(&self) -> Result<String, GoError> {
        let out = self.run_ok(&std::env::temp_dir(), ["env", "GOTOOLCHAIN"])?;
        Ok(out.stdout.trim().to_owned())
    }
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}
