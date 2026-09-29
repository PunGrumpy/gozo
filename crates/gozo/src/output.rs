use std::io::Write;

use serde::Serialize;

/// stdout sink for a command's primary result.
pub struct Output {
    pub json: bool,
}

impl Output {
    pub fn new(json: bool) -> Self {
        Output { json }
    }

    pub fn json_value<T: Serialize>(&self, value: &T) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer_pretty(&mut stdout, value)?;
        writeln!(stdout)?;
        Ok(())
    }

    /// One compact JSON document per line (JSON Lines).
    pub fn json_line<T: Serialize>(&self, value: &T) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, value)?;
        writeln!(stdout)?;
        Ok(())
    }

    pub fn line(&self, s: impl AsRef<str>) {
        let _ = writeln!(std::io::stdout(), "{}", s.as_ref());
    }

    /// `--json` writes `gozo.error/v1` to stdout so callers always get a
    /// parseable document; otherwise `Error: ...` on stderr.
    pub fn error(&self, err: &anyhow::Error) {
        if self.json {
            let _ = self.json_value(&ErrorDoc {
                schema: "gozo.error/v1",
                error: format!("{err:#}"),
            });
        } else {
            eprintln!("Error: {err:#}");
        }
    }
}

#[derive(Serialize)]
struct ErrorDoc {
    schema: &'static str,
    error: String,
}
