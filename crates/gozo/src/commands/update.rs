//! `gozo update`: replace this binary with the latest GitHub release.

use std::cmp::Ordering;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, anyhow};
use serde::Serialize;

use crate::ctx::Ctx;

const REPO: &str = "PunGrumpy/gozo";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Only report whether a newer version exists.
    #[arg(long)]
    pub check: bool,

    /// Install this exact version instead of the latest.
    #[arg(long, value_name = "VERSION")]
    pub version: Option<String>,
}

#[derive(Serialize)]
struct UpdateDoc<'a> {
    schema: &'static str,
    current: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest: Option<&'a str>,
    update_available: bool,
    checked_only: bool,
    updated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    binary: Option<&'a Path>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<&'a str>,
}

struct Release {
    version: String,
    assets: Vec<(String, String)>,
}

pub fn run(ctx: &mut Ctx, args: Args) -> anyhow::Result<ExitCode> {
    let want = args.version.as_deref().map(strip_v);
    let url = match want {
        // Releases are tagged `gozo@<version>` by `changeset publish`.
        Some(v) => format!("https://api.github.com/repos/{REPO}/releases/tags/gozo@{v}"),
        None => format!("https://api.github.com/repos/{REPO}/releases/latest"),
    };
    ctx.ui.debug(format!("GET {url}"));

    let release = match fetch_release(&url)? {
        Some(r) => r,
        None => {
            let message = match want {
                Some(v) => format!("release v{v} not found"),
                None => "No releases published yet".to_owned(),
            };
            if ctx.json {
                ctx.out.json_value(&UpdateDoc {
                    schema: "gozo.update/v1",
                    current: CURRENT,
                    latest: None,
                    update_available: false,
                    checked_only: args.check,
                    updated: false,
                    binary: None,
                    message: Some(&message),
                })?;
            } else {
                ctx.ui.info(&message);
            }
            return Ok(if args.check {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            });
        }
    };

    let newer = compare_versions(&release.version, CURRENT) == Ordering::Greater;
    let differs = release.version != CURRENT;
    if args.check {
        if ctx.json {
            ctx.out.json_value(&UpdateDoc {
                schema: "gozo.update/v1",
                current: CURRENT,
                latest: Some(&release.version),
                update_available: newer,
                checked_only: true,
                updated: false,
                binary: None,
                message: None,
            })?;
        } else if newer {
            ctx.ui.info(format!(
                "Update available: {} (run gozo update)",
                ctx.ui.bold(&release.version)
            ));
        } else {
            ctx.ui.info(format!("gozo {CURRENT} is up to date"));
        }
        return Ok(ExitCode::SUCCESS);
    }

    // Install: only when newer, or when an explicit version was requested.
    if !(newer || (want.is_some() && differs)) {
        if ctx.json {
            ctx.out.json_value(&UpdateDoc {
                schema: "gozo.update/v1",
                current: CURRENT,
                latest: Some(&release.version),
                update_available: false,
                checked_only: false,
                updated: false,
                binary: None,
                message: None,
            })?;
        } else {
            ctx.ui.info(format!("gozo {CURRENT} is up to date"));
        }
        return Ok(ExitCode::SUCCESS);
    }

    let asset = asset_name().ok_or_else(|| {
        anyhow!(
            "no prebuilt binary for {}/{}\n  build from source: cargo install --git https://github.com/{REPO}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let (_, download_url) = release
        .assets
        .iter()
        .find(|(n, _)| *n == asset)
        .ok_or_else(|| anyhow!("release v{} has no asset named {asset}", release.version))?;

    let exe = std::env::current_exe().context("could not locate the running gozo binary")?;
    let exe = exe.canonicalize().unwrap_or(exe);
    ctx.ui.header();
    ctx.ui
        .step(format!("Downloading gozo {} ({asset})", release.version));
    ctx.ui.debug(format!("GET {download_url}"));

    let mut archive =
        tempfile::NamedTempFile::new().context("could not create a temporary file")?;
    let mut resp = ureq::get(download_url)
        .header("User-Agent", format!("gozo/{CURRENT}"))
        .call()
        .with_context(|| format!("download of {download_url} failed"))?;
    std::io::copy(&mut resp.body_mut().as_reader(), &mut archive)
        .context("download interrupted")?;
    archive.flush()?;

    ctx.ui.step("Installing");
    let new_path = PathBuf::from(format!("{}.new", exe.display()));
    extract_binary(archive.path(), &new_path)
        .with_context(|| format!("could not extract gozo from {asset}"))?;
    if let Err(e) = std::fs::rename(&new_path, &exe) {
        let _ = std::fs::remove_file(&new_path);
        return Err(anyhow!(
            "could not replace {}: {e}\n  the directory may not be writable; re-run with elevated permissions or reinstall gozo somewhere on your PATH that you own",
            exe.display()
        ));
    }

    if ctx.json {
        ctx.out.json_value(&UpdateDoc {
            schema: "gozo.update/v1",
            current: CURRENT,
            latest: Some(&release.version),
            update_available: false,
            checked_only: false,
            updated: true,
            binary: Some(&exe),
            message: None,
        })?;
    } else {
        ctx.ui.success(format!(
            "Updated gozo {CURRENT} -> {} ({})",
            release.version,
            exe.display()
        ));
    }
    Ok(ExitCode::SUCCESS)
}

/// GET a GitHub release document. `None` on 404 (no releases, or unknown tag).
fn fetch_release(url: &str) -> anyhow::Result<Option<Release>> {
    let mut resp = ureq::get(url)
        .header("User-Agent", format!("gozo/{CURRENT}"))
        .header("Accept", "application/vnd.github+json")
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .context("could not reach api.github.com; check your network connection")?;
    let status = resp.status().as_u16();
    if status == 404 {
        return Ok(None);
    }
    if !(200..300).contains(&status) {
        anyhow::bail!("GitHub API returned HTTP {status} for {url}");
    }
    let doc: serde_json::Value = resp
        .body_mut()
        .read_json()
        .context("unexpected response from GitHub")?;
    let tag = doc
        .get("tag_name")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow!("release document has no tag_name"))?;
    let assets = doc
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| {
                    Some((
                        x.get("name")?.as_str()?.to_owned(),
                        x.get("browser_download_url")?.as_str()?.to_owned(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Some(Release {
        version: strip_v(tag).to_owned(),
        assets,
    }))
}

/// Pull the `gozo` binary out of a `.tar.gz` and write it to `dest` (mode 755).
fn extract_binary(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let wanted = if cfg!(windows) { "gozo.exe" } else { "gozo" };
    for entry in tar.entries()? {
        let mut entry = entry?;
        let is_bin = entry
            .path()?
            .file_name()
            .is_some_and(|n| n.to_string_lossy() == wanted);
        if !is_bin || !entry.header().entry_type().is_file() {
            continue;
        }
        let mut out = std::fs::File::create(dest)?;
        std::io::copy(&mut entry, &mut out)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;
        }
        return Ok(());
    }
    anyhow::bail!("archive does not contain a {wanted} binary")
}

/// `gozo-<os>-<arch>.tar.gz` for this machine.
pub fn asset_name() -> Option<String> {
    asset_name_for(std::env::consts::OS, std::env::consts::ARCH)
}

pub fn asset_name_for(os: &str, arch: &str) -> Option<String> {
    let os = match os {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        _ => return None,
    };
    let arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return None,
    };
    Some(format!("gozo-{os}-{arch}.tar.gz"))
}

/// Normalize a tag or version string: `gozo@1.2.3`, `v1.2.3` and `1.2.3` all
/// become `1.2.3`.
pub fn strip_v(v: &str) -> &str {
    let v = v.trim();
    let v = v.strip_prefix("gozo@").unwrap_or(v);
    v.trim_start_matches(['v', 'V'])
}

/// Compare `x.y.z` strings numerically; a pre-release suffix sorts below the
/// plain release (`1.2.0-rc1 < 1.2.0`). Unparseable parts count as 0.
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    fn parts(v: &str) -> (Vec<u64>, Option<String>) {
        let v = strip_v(v);
        let (nums, pre) = match v.split_once('-') {
            Some((n, p)) => (n, Some(p.to_owned())),
            None => (v, None),
        };
        let mut n: Vec<u64> = nums.split('.').map(|p| p.parse().unwrap_or(0)).collect();
        while n.len() < 3 {
            n.push(0);
        }
        (n, pre)
    }
    let (an, ap) = parts(a);
    let (bn, bp) = parts(b);
    an.cmp(&bn).then_with(|| match (ap, bp) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => x.cmp(&y),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tag_prefixes() {
        assert_eq!(strip_v("gozo@1.2.3"), "1.2.3");
        assert_eq!(strip_v("v1.2.3"), "1.2.3");
        assert_eq!(strip_v(" 1.2.3 "), "1.2.3");
    }

    #[test]
    fn compares_versions() {
        assert_eq!(compare_versions("0.2.0", "0.1.0"), Ordering::Greater);
        assert_eq!(compare_versions("v0.1.0", "0.1.0"), Ordering::Equal);
        assert_eq!(compare_versions("0.1.10", "0.1.9"), Ordering::Greater);
        assert_eq!(compare_versions("1.0", "1.0.0"), Ordering::Equal);
        assert_eq!(compare_versions("1.0.0-rc1", "1.0.0"), Ordering::Less);
        assert_eq!(compare_versions("0.9.9", "1.0.0"), Ordering::Less);
    }

    #[test]
    fn asset_names() {
        assert_eq!(
            asset_name_for("linux", "x86_64").as_deref(),
            Some("gozo-linux-x86_64.tar.gz")
        );
        assert_eq!(
            asset_name_for("macos", "aarch64").as_deref(),
            Some("gozo-darwin-aarch64.tar.gz")
        );
        assert_eq!(asset_name_for("freebsd", "x86_64"), None);
        assert_eq!(asset_name_for("linux", "riscv64"), None);
    }
}
