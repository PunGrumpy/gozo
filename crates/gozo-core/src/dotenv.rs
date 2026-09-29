//! Minimal `.env` reader and writer: `KEY=value`, `export KEY=value`, single
//! and double quotes, `\n` escapes inside double quotes, `#` comments.

use std::path::Path;

/// Ordered key/value pairs; later keys override earlier ones.
pub fn parse(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line
            .strip_prefix("export ")
            .map(str::trim_start)
            .unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let value = unquote(value.trim());
        if let Some(slot) = out.iter_mut().find(|(k, _)| k == key) {
            slot.1 = value;
        } else {
            out.push((key.to_owned(), value));
        }
    }
    out
}

fn unquote(v: &str) -> String {
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        let inner = &v[1..v.len() - 1];
        let mut s = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some(o) => {
                        s.push('\\');
                        s.push(o);
                    }
                    None => s.push('\\'),
                }
            } else {
                s.push(c);
            }
        }
        return s;
    }
    if v.len() >= 2 && v.starts_with('\'') && v.ends_with('\'') {
        return v[1..v.len() - 1].to_owned();
    }
    match v.find(" #") {
        Some(i) => v[..i].trim_end().to_owned(),
        None => v.to_owned(),
    }
}

/// A missing file yields an empty list.
pub fn load(path: &Path) -> std::io::Result<Vec<(String, String)>> {
    match std::fs::read_to_string(path) {
        Ok(t) => Ok(parse(&t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

pub fn render(vars: &[(String, String)], header: Option<&str>) -> String {
    let mut out = String::new();
    if let Some(h) = header {
        for line in h.lines() {
            out.push_str("# ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }
    for (k, v) in vars {
        out.push_str(k);
        out.push('=');
        out.push_str(&quote(v));
        out.push('\n');
    }
    out
}

pub fn quote(v: &str) -> String {
    let needs = v.is_empty()
        || v.chars()
            .any(|c| c.is_whitespace() || matches!(c, '#' | '"' | '\'' | '\\' | '$' | '`'));
    if !needs {
        return v.to_owned();
    }
    let mut s = String::from("\"");
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            o => s.push(o),
        }
    }
    s.push('"');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_forms() {
        let v = parse(
            "# c\nA=1\nexport B=\"x y\\n\"\nC='q # not comment'\nD=val # comment\nBAD LINE\n",
        );
        assert_eq!(
            v,
            vec![
                ("A".into(), "1".into()),
                ("B".into(), "x y\n".into()),
                ("C".into(), "q # not comment".into()),
                ("D".into(), "val".into()),
            ]
        );
    }

    #[test]
    fn round_trips() {
        let vars = vec![
            ("A".to_string(), "plain".to_string()),
            ("B".to_string(), "has space".to_string()),
            ("C".to_string(), "".to_string()),
        ];
        let text = render(&vars, Some("hdr"));
        assert!(text.starts_with("# hdr\n\n"));
        assert_eq!(parse(&text), vars);
    }
}
