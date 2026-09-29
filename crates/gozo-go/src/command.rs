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

    fn checked(self, args: &str) -> Result<GoOutput, GoError> {
        if self.success() {
            Ok(self)
        } else {
            Err(GoError::Failed {
                args: args.to_owned(),
                status: self.status,
                stderr: self.stderr.trim().to_owned(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Go {
    pub bin: PathBuf,
}

impl Go {
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
        let args = strings(args);
        let mut cmd = Command::new(&self.bin);
        cmd.args(&args).current_dir(dir);
        for (k, v) in env {
            cmd.env(k, v);
        }
        let out = cmd.output().map_err(|source| GoError::Spawn {
            args: args.join(" "),
            source,
        })?;
        Ok(GoOutput {
            status: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }

    /// Run with `GOTOOLCHAIN=local`, for read-only commands (`env`, `mod edit`,
    /// `version`) that must never start a toolchain download.
    pub fn run_local<I, S>(&self, dir: &Path, args: I) -> Result<GoOutput, GoError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_env(dir, args, &[("GOTOOLCHAIN", "local")])
    }

    pub fn run_ok<I, S>(&self, dir: &Path, args: I) -> Result<GoOutput, GoError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = strings(args);
        self.run(dir, &args)?.checked(&args.join(" "))
    }

    pub fn run_json_local<T, I, S>(&self, dir: &Path, args: I) -> Result<T, GoError>
    where
        T: serde::de::DeserializeOwned,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = strings(args);
        let joined = args.join(" ");
        let out = self.run_local(dir, &args)?.checked(&joined)?;
        serde_json::from_str(&out.stdout).map_err(|source| GoError::Parse {
            args: joined,
            source,
        })
    }

    /// Stdout as concatenated JSON documents, the way `go list -json` prints them.
    pub fn run_json_stream<T, I, S>(&self, dir: &Path, args: I) -> Result<Vec<T>, GoError>
    where
        T: serde::de::DeserializeOwned,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = strings(args);
        let out = self.run_ok(dir, &args)?;
        serde_json::Deserializer::from_str(&out.stdout)
            .into_iter::<T>()
            .collect::<std::result::Result<Vec<T>, _>>()
            .map_err(|source| GoError::Parse {
                args: args.join(" "),
                source,
            })
    }

    pub fn version(&self) -> Result<GoVersion, GoError> {
        let out = self
            .run_local(Path::new("."), ["version"])?
            .checked("version")?;
        GoVersion::parse_go_version_output(&out.stdout)
    }

    /// `go env -json` in `dir` under `GOTOOLCHAIN=local` (no download), with the
    /// real `GOTOOLCHAIN` restored from [`Self::toolchain_mode`].
    pub fn env(&self, dir: &Path) -> Result<BTreeMap<String, String>, GoError> {
        let mut env: BTreeMap<String, String> = self.run_json_local(dir, ["env", "-json"])?;
        if let Ok(mode) = self.toolchain_mode() {
            env.insert("GOTOOLCHAIN".to_owned(), mode);
        }
        Ok(env)
    }

    /// Read from a directory without go.mod so it can never start a download.
    pub fn toolchain_mode(&self) -> Result<String, GoError> {
        let out = self.run_ok(&std::env::temp_dir(), ["env", "GOTOOLCHAIN"])?;
        Ok(out.stdout.trim().to_owned())
    }
}

fn strings<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter()
        .map(|a| a.as_ref().to_string_lossy().into_owned())
        .collect()
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}
