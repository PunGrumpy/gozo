//! `go vet -json`: one JSON object per package, keyed by package path, then
//! analyzer name, then a list of `{posn, message}` findings. Older Go versions
//! print a `# pkg` comment line before each object.

use std::ffi::OsStr;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::command::{Go, GoError, GoOutput};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VetIssue {
    pub package: String,
    pub analyzer: String,
    /// `file:line:col` exactly as vet printed it (file is usually absolute).
    pub posn: String,
    pub message: String,
}

impl VetIssue {
    /// Split `posn` into `(file, line, col)`. Missing parts are `0`.
    pub fn location(&self) -> (&str, u32, u32) {
        split_posn(&self.posn)
    }
}

/// Split `file:line:col` from the right so Windows drive letters survive.
pub fn split_posn(posn: &str) -> (&str, u32, u32) {
    let mut parts = posn.rsplitn(3, ':');
    let last = parts.next().unwrap_or("");
    let mid = parts.next();
    let first = parts.next();
    match (first, mid) {
        (Some(file), Some(line)) => match (line.parse::<u32>(), last.parse::<u32>()) {
            (Ok(l), Ok(c)) => (file, l, c),
            // `file:line` where `line` failed to parse as col means two parts only.
            _ => (posn, 0, 0),
        },
        (None, Some(file)) => match last.parse::<u32>() {
            Ok(l) => (file, l, 0),
            Err(_) => (posn, 0, 0),
        },
        _ => (posn, 0, 0),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VetReport {
    /// Number of package objects seen (one per vetted package).
    pub packages: usize,
    pub issues: Vec<VetIssue>,
}

/// Parse the stdout of `go vet -json`. Tolerates `# pkg` header lines, empty
/// `{}` objects and trailing garbage.
pub fn parse_vet_json(stdout: &str) -> VetReport {
    let body: String = stdout
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let mut report = VetReport::default();
    for doc in serde_json::Deserializer::from_str(&body).into_iter::<serde_json::Value>() {
        let Ok(doc) = doc else { break };
        let Some(pkgs) = doc.as_object() else {
            continue;
        };
        report.packages += 1;
        for (pkg, analyzers) in pkgs {
            let Some(analyzers) = analyzers.as_object() else {
                continue;
            };
            for (analyzer, findings) in analyzers {
                let Some(list) = findings.as_array() else {
                    continue;
                };
                for f in list {
                    let get = |k: &str| f.get(k).and_then(|v| v.as_str()).unwrap_or("").to_owned();
                    report.issues.push(VetIssue {
                        package: pkg.clone(),
                        analyzer: analyzer.clone(),
                        posn: get("posn"),
                        message: get("message"),
                    });
                }
            }
        }
    }
    report
}

impl Go {
    /// `go vet -json <patterns>` in `dir`. Never fails on a non-zero exit (vet
    /// exits 1 when it finds something); inspect `GoOutput` for build errors.
    pub fn vet<I, S>(&self, dir: &Path, patterns: I) -> Result<(GoOutput, VetReport), GoError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut args: Vec<String> = vec!["vet".to_owned(), "-json".to_owned()];
        args.extend(
            patterns
                .into_iter()
                .map(|p| p.as_ref().to_string_lossy().into_owned()),
        );
        let out = self.run(dir, &args)?;
        let report = parse_vet_json(&out.stdout);
        Ok((out, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"# example.com/m/a
{}
# example.com/m/bad
{
	"example.com/m/bad": {
		"printf": [
			{
				"posn": "/tmp/p/bad.go:6:22",
				"end": "/tmp/p/bad.go:6:24",
				"message": "fmt.Sprintf format %d has arg \"x\" of wrong type string"
			}
		],
		"unusedresult": [
			{ "posn": "/tmp/p/bad.go:9:2", "message": "result of fmt.Sprintf call not used" }
		]
	}
}
{}
"#;

    #[test]
    fn parses_packages_and_issues() {
        let r = parse_vet_json(SAMPLE);
        assert_eq!(r.packages, 3);
        assert_eq!(r.issues.len(), 2);
        let first = &r.issues[0];
        assert_eq!(first.package, "example.com/m/bad");
        assert_eq!(first.analyzer, "printf");
        assert_eq!(first.location(), ("/tmp/p/bad.go", 6, 22));
        assert!(first.message.starts_with("fmt.Sprintf format"));
        assert_eq!(r.issues[1].analyzer, "unusedresult");
    }

    #[test]
    fn empty_and_garbage_are_tolerated() {
        assert_eq!(parse_vet_json(""), VetReport::default());
        let r = parse_vet_json("{}\n{}\nnot json at all");
        assert_eq!(r.packages, 2);
        assert!(r.issues.is_empty());
    }

    #[test]
    fn splits_positions_from_the_right() {
        assert_eq!(split_posn("C:\\src\\a.go:3:4"), ("C:\\src\\a.go", 3, 4));
        assert_eq!(split_posn("a.go:12"), ("a.go", 12, 0));
        assert_eq!(split_posn("a.go"), ("a.go", 0, 0));
        assert_eq!(split_posn(""), ("", 0, 0));
    }
}
