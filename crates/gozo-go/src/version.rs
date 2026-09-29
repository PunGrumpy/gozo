use std::cmp::Ordering;
use std::fmt;

use serde::Serialize;

use crate::command::GoError;

/// A Go release version such as `1.27.1`, `1.27`, or `1.27rc1`.
///
/// Ordering follows Go's own rules loosely: the numeric components compare
/// first, and a pre-release suffix sorts before the final release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GoVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// `rc1`, `beta2`, or empty for a final release.
    pub prerelease: String,
    /// The original string, for example `go1.27.1` or `1.27`.
    pub raw: String,
}

impl GoVersion {
    /// Parse `1.27.1`, `go1.27.1`, `1.27`, `1.27rc1`.
    pub fn parse(s: &str) -> Result<GoVersion, GoError> {
        let raw = s.trim();
        let body = raw.strip_prefix("go").unwrap_or(raw);
        let (nums, prerelease) = split_prerelease(body);
        let mut parts = nums.split('.');
        let major = parse_num(parts.next(), raw)?;
        let minor = parse_num(parts.next(), raw)?;
        let patch = match parts.next() {
            Some(p) => parse_num(Some(p), raw)?,
            None => 0,
        };
        if parts.next().is_some() {
            return Err(GoError::Version(raw.to_owned()));
        }
        Ok(GoVersion {
            major,
            minor,
            patch,
            prerelease: prerelease.to_owned(),
            raw: raw.to_owned(),
        })
    }

    /// Parse the output of `go version`, e.g. `go version go1.27.1 linux/amd64`.
    pub fn parse_go_version_output(out: &str) -> Result<GoVersion, GoError> {
        out.split_whitespace()
            .find(|w| w.starts_with("go") && w[2..].starts_with(|c: char| c.is_ascii_digit()))
            .map(GoVersion::parse)
            .unwrap_or_else(|| Err(GoError::Version(out.trim().to_owned())))
    }

    /// Whether this toolchain satisfies a `go` directive requirement.
    pub fn satisfies(&self, required: &GoVersion) -> bool {
        self >= required
    }
}

impl PartialOrd for GoVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GoVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(
                || match (self.prerelease.is_empty(), other.prerelease.is_empty()) {
                    (true, true) => Ordering::Equal,
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    (false, false) => self.prerelease.cmp(&other.prerelease),
                },
            )
    }
}

impl fmt::Display for GoVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;
        if self.patch > 0 {
            write!(f, ".{}", self.patch)?;
        }
        write!(f, "{}", self.prerelease)
    }
}

fn split_prerelease(s: &str) -> (&str, &str) {
    match s.find(|c: char| c.is_ascii_alphabetic()) {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    }
}

fn parse_num(part: Option<&str>, raw: &str) -> Result<u32, GoError> {
    part.and_then(|p| p.parse().ok())
        .ok_or_else(|| GoError::Version(raw.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_forms() {
        assert_eq!(GoVersion::parse("go1.27.1").unwrap().to_string(), "1.27.1");
        assert_eq!(GoVersion::parse("1.27").unwrap().to_string(), "1.27");
        let rc = GoVersion::parse("1.28rc1").unwrap();
        assert_eq!(rc.prerelease, "rc1");
        assert!(GoVersion::parse("banana").is_err());
    }

    #[test]
    fn orders() {
        let v = |s| GoVersion::parse(s).unwrap();
        assert!(v("1.27.1") > v("1.27"));
        assert!(v("1.28") > v("1.27.9"));
        assert!(v("1.28rc1") < v("1.28"));
        assert!(v("1.27.1").satisfies(&v("1.21")));
        assert!(!v("1.21").satisfies(&v("1.27.1")));
    }

    #[test]
    fn parses_go_version_output() {
        let v = GoVersion::parse_go_version_output("go version go1.27.1 linux/amd64").unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 27, 1));
    }
}
