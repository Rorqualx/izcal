//! A TOML reader (TOML 1.0) for Cargo manifests and the gate files.
//!
//! It reads everything those files use: tables, arrays of tables, dotted
//! and quoted keys, the four string forms, integers, booleans, arrays and
//! inline tables. Dates and times, which no manifest or gate file uses,
//! are rejected rather than misread, and so is any redefinition, so the
//! lint never acts on a value it parsed differently from Cargo.

use std::collections::{BTreeMap, BTreeSet};

use crate::value::{Cursor, ParseError, Value};

type Table = BTreeMap<String, Value>;

/// Parses one TOML document into a [`Value::Table`].
///
/// # Errors
/// A [`ParseError`] naming the line of the first defect.
pub fn parse(text: &str) -> Result<Value, ParseError> {
    let mut c = Cursor::new(text);
    let mut root = Table::new();
    // The table that key/value lines go into, as a path from the root.
    let mut current: Vec<String> = Vec::new();
    // Tables defined by a header or by dotted keys, which may not be
    // defined again (TOML 1.0, "Table").
    let mut defined: BTreeSet<Vec<String>> = BTreeSet::new();
    loop {
        skip_blank(&mut c);
        match c.peek() {
            None => break,
            Some('\n') => {
                c.bump();
            }
            Some('[') => {
                c.bump();
                let array = c.eat('[');
                let path = key_path(&mut c)?;
                inline_ws(&mut c);
                if !c.eat(']') || (array && !c.eat(']')) {
                    return Err(c.error("unclosed table header"));
                }
                end_of_line(&mut c)?;
                if array {
                    let parent = navigate(&mut root, &path[..path.len() - 1], &c)?;
                    let last = &path[path.len() - 1];
                    let entry = parent
                        .entry(last.clone())
                        .or_insert_with(|| Value::Array(Vec::new()));
                    match entry {
                        Value::Array(a) if a.iter().all(|v| matches!(v, Value::Table(_))) => {
                            a.push(Value::Table(Table::new()));
                        }
                        _ => return Err(c.error(format!("{last:?} is not an array of tables"))),
                    }
                    // A fresh element: its sub-tables may be defined again.
                    defined.retain(|d| !d.starts_with(&path));
                } else {
                    if !defined.insert(path.clone()) {
                        return Err(c.error(format!("table [{}] defined twice", path.join("."))));
                    }
                    navigate(&mut root, &path, &c)?;
                }
                current = path;
            }
            Some(_) => {
                let path = key_path(&mut c)?;
                inline_ws(&mut c);
                if !c.eat('=') {
                    return Err(c.error("expected '=' after a key"));
                }
                let v = value(&mut c, 0)?;
                end_of_line(&mut c)?;
                let mut full = current.clone();
                for k in &path[..path.len() - 1] {
                    full.push(k.clone());
                    defined.insert(full.clone());
                }
                let table = navigate(&mut root, &full, &c)?;
                let last = path[path.len() - 1].clone();
                if table.insert(last.clone(), v).is_some() {
                    return Err(c.error(format!("key {last:?} defined twice")));
                }
            }
        }
    }
    Ok(Value::Table(root))
}

/// The table at `path`, creating missing tables; through an array of
/// tables it goes to the last element.
fn navigate<'t>(
    root: &'t mut Table,
    path: &[String],
    c: &Cursor<'_>,
) -> Result<&'t mut Table, ParseError> {
    let mut t = root;
    for k in path {
        let entry = t
            .entry(k.clone())
            .or_insert_with(|| Value::Table(Table::new()));
        t = match entry {
            Value::Table(next) => next,
            Value::Array(a) => match a.last_mut() {
                Some(Value::Table(next)) => next,
                _ => return Err(c.error(format!("{k:?} is not a table"))),
            },
            _ => return Err(c.error(format!("{k:?} is not a table"))),
        };
    }
    Ok(t)
}

fn inline_ws(c: &mut Cursor<'_>) {
    while matches!(c.peek(), Some(' ' | '\t')) {
        c.bump();
    }
}

/// Whitespace and a comment, but not the newline.
fn skip_blank(c: &mut Cursor<'_>) {
    inline_ws(c);
    if c.peek() == Some('#') {
        while !matches!(c.peek(), None | Some('\n')) {
            c.bump();
        }
    }
    if c.peek() == Some('\r') && c.peek2() == Some('\n') {
        c.bump();
    }
}

/// Whitespace, newlines and comments (inside arrays and inline tables).
fn skip_all(c: &mut Cursor<'_>) {
    loop {
        skip_blank(c);
        if !c.eat('\n') {
            break;
        }
    }
}

fn end_of_line(c: &mut Cursor<'_>) -> Result<(), ParseError> {
    skip_blank(c);
    match c.peek() {
        None => Ok(()),
        Some('\n') => {
            c.bump();
            Ok(())
        }
        Some(other) => Err(c.error(format!("unexpected {other:?} after a value"))),
    }
}

fn key_path(c: &mut Cursor<'_>) -> Result<Vec<String>, ParseError> {
    let mut path = Vec::new();
    loop {
        inline_ws(c);
        let k = match c.peek() {
            Some('"') => basic_string(c)?,
            Some('\'') => literal_string(c)?,
            _ => {
                let mut k = String::new();
                while let Some(ch) = c.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                        k.push(ch);
                        c.bump();
                    } else {
                        break;
                    }
                }
                if k.is_empty() {
                    return Err(c.error("expected a key"));
                }
                k
            }
        };
        path.push(k);
        inline_ws(c);
        if !c.eat('.') {
            return Ok(path);
        }
    }
}

/// Deep enough for any manifest, shallow enough to bound the stack.
const MAX_DEPTH: usize = 64;

fn value(c: &mut Cursor<'_>, depth: usize) -> Result<Value, ParseError> {
    if depth > MAX_DEPTH {
        return Err(c.error("value nested too deeply"));
    }
    inline_ws(c);
    match c.peek() {
        Some('"') => {
            if c.eat_str("\"\"\"") {
                multiline_basic(c).map(Value::Str)
            } else {
                basic_string(c).map(Value::Str)
            }
        }
        Some('\'') => {
            if c.eat_str("'''") {
                multiline_literal(c).map(Value::Str)
            } else {
                literal_string(c).map(Value::Str)
            }
        }
        Some('[') => {
            c.bump();
            let mut a = Vec::new();
            loop {
                skip_all(c);
                if c.eat(']') {
                    return Ok(Value::Array(a));
                }
                a.push(value(c, depth + 1)?);
                skip_all(c);
                if c.eat(',') {
                    continue;
                }
                skip_all(c);
                if c.eat(']') {
                    return Ok(Value::Array(a));
                }
                return Err(c.error("expected ',' or ']' in an array"));
            }
        }
        Some('{') => {
            c.bump();
            let mut t = Table::new();
            skip_all(c);
            if c.eat('}') {
                return Ok(Value::Table(t));
            }
            loop {
                skip_all(c);
                let path = key_path(c)?;
                inline_ws(c);
                if !c.eat('=') {
                    return Err(c.error("expected '=' in an inline table"));
                }
                let v = value(c, depth + 1)?;
                let mut inner = &mut t;
                for k in &path[..path.len() - 1] {
                    let e = inner
                        .entry(k.clone())
                        .or_insert_with(|| Value::Table(Table::new()));
                    inner = match e {
                        Value::Table(next) => next,
                        _ => return Err(c.error(format!("{k:?} is not a table"))),
                    };
                }
                let last = path[path.len() - 1].clone();
                if inner.insert(last.clone(), v).is_some() {
                    return Err(c.error(format!("key {last:?} defined twice")));
                }
                skip_all(c);
                if c.eat(',') {
                    continue;
                }
                if c.eat('}') {
                    return Ok(Value::Table(t));
                }
                return Err(c.error("expected ',' or '}' in an inline table"));
            }
        }
        Some('t') if c.eat_str("true") => Ok(Value::Bool(true)),
        Some('f') if c.eat_str("false") => Ok(Value::Bool(false)),
        Some(_) => scalar(c),
        None => Err(c.error("expected a value")),
    }
}

/// An integer or float, written as TOML allows.
fn scalar(c: &mut Cursor<'_>) -> Result<Value, ParseError> {
    let mut raw = String::new();
    while let Some(ch) = c.peek() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '+' | '-' | '.' | ':') {
            raw.push(ch);
            c.bump();
        } else {
            break;
        }
    }
    let clean: String = raw.chars().filter(|&ch| ch != '_').collect();
    let (neg, body) = match clean.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, clean.strip_prefix('+').unwrap_or(&clean)),
    };
    let radix = [("0x", 16), ("0o", 8), ("0b", 2)]
        .into_iter()
        .find_map(|(p, r)| body.strip_prefix(p).map(|digits| (digits, r)));
    let int = match radix {
        Some((digits, r)) if !neg && !digits.is_empty() => i64::from_str_radix(digits, r).ok(),
        Some(_) => None,
        None if !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit()) => {
            if body.len() > 1 && body.starts_with('0') {
                None
            } else {
                clean.parse::<i64>().ok()
            }
        }
        None => None,
    };
    if let Some(i) = int {
        return Ok(Value::Int(i));
    }
    // A float has a fraction or an exponent, and its integer part has no
    // leading zero (TOML 1.0, "Float").
    let int_part: String = body.chars().take_while(char::is_ascii_digit).collect();
    let is_float = matches!(body, "inf" | "nan")
        || (!int_part.is_empty()
            && !(int_part.len() > 1 && int_part.starts_with('0'))
            && body.contains(['.', 'e', 'E'])
            && body
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-'))
            && clean.parse::<f64>().is_ok());
    if is_float {
        Ok(Value::Number(raw))
    } else if raw.contains(':') || raw.matches('-').count() >= 2 {
        Err(c.error(format!("dates and times are not supported: {raw:?}")))
    } else {
        Err(c.error(format!("unrecognised value {raw:?}")))
    }
}

fn escape(c: &mut Cursor<'_>) -> Result<char, ParseError> {
    Ok(match c.bump() {
        Some('b') => '\u{8}',
        Some('t') => '\t',
        Some('n') => '\n',
        Some('f') => '\u{c}',
        Some('r') => '\r',
        Some('e') => '\u{1b}',
        Some('"') => '"',
        Some('\\') => '\\',
        Some('u') => {
            let v = c.hex_scalar(4)?;
            char::from_u32(v).ok_or_else(|| c.error("invalid \\u escape"))?
        }
        Some('U') => {
            let v = c.hex_scalar(8)?;
            char::from_u32(v).ok_or_else(|| c.error("invalid \\U escape"))?
        }
        _ => return Err(c.error("bad escape in a string")),
    })
}

fn basic_string(c: &mut Cursor<'_>) -> Result<String, ParseError> {
    c.bump();
    let mut s = String::new();
    loop {
        match c.bump() {
            None | Some('\n') => return Err(c.error("unterminated string")),
            Some('"') => return Ok(s),
            Some('\\') => s.push(escape(c)?),
            Some(ch) => s.push(ch),
        }
    }
}

fn literal_string(c: &mut Cursor<'_>) -> Result<String, ParseError> {
    c.bump();
    let mut s = String::new();
    loop {
        match c.bump() {
            None | Some('\n') => return Err(c.error("unterminated string")),
            Some('\'') => return Ok(s),
            Some(ch) => s.push(ch),
        }
    }
}

/// After the opening `"""`. A newline right after it is dropped, and a
/// backslash at the end of a line joins it to the next non-blank text.
fn multiline_basic(c: &mut Cursor<'_>) -> Result<String, ParseError> {
    c.eat('\r');
    c.eat('\n');
    let mut s = String::new();
    loop {
        if c.eat_str("\"\"\"") {
            // Up to two quotes may sit right before the closing three.
            while c.peek() == Some('"') {
                s.push('"');
                c.bump();
            }
            return Ok(s);
        }
        match c.bump() {
            None => return Err(c.error("unterminated multi-line string")),
            Some('\\') => {
                let mut probe = c.peek();
                // A line-ending backslash is the last non-blank character
                // on its line (TOML 1.0); `\ ` elsewhere is a bad escape.
                if c.rest_of_line_is_blank() {
                    // A line-ending backslash: skip whitespace and newlines.
                    while matches!(probe, Some(' ' | '\t' | '\r' | '\n')) {
                        c.bump();
                        probe = c.peek();
                    }
                } else {
                    s.push(escape(c)?);
                }
            }
            Some(ch) => s.push(ch),
        }
    }
}

fn multiline_literal(c: &mut Cursor<'_>) -> Result<String, ParseError> {
    c.eat('\r');
    c.eat('\n');
    let mut s = String::new();
    loop {
        if c.eat_str("'''") {
            while c.peek() == Some('\'') {
                s.push('\'');
                c.bump();
            }
            return Ok(s);
        }
        match c.bump() {
            None => return Err(c.error("unterminated multi-line string")),
            Some(ch) => s.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_cargo_manifest() {
        let text = r#"
# comment
[package]
name = "hg-x"   # trailing comment
version.workspace = true

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61", features = [
  "A",  # why
  "B",
] }

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ["cfg(kani)"] }

[[bin]]
name = "a"
[[bin]]
name = "b"

[profile.release.package.hg-types]
overflow-checks = true
n = 1_000
h = 0xff
f = 1.5
s = """
line \
  joined"""
l = '''raw\n'''
"#;
        let v = parse(text).unwrap();
        assert_eq!(v.path(&["package", "name"]).unwrap().as_str(), Some("hg-x"));
        assert_eq!(
            v.path(&["package", "version", "workspace"])
                .unwrap()
                .as_bool(),
            Some(true)
        );
        let ws = v
            .path(&[
                "target",
                "cfg(windows)",
                "dependencies",
                "windows-sys",
                "features",
            ])
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(ws.len(), 2);
        let bins = v.get("bin").unwrap().as_array().unwrap();
        assert_eq!(bins[1].get("name").unwrap().as_str(), Some("b"));
        let p = v
            .path(&["profile", "release", "package", "hg-types"])
            .unwrap();
        assert_eq!(p.get("n").unwrap().as_int(), Some(1000));
        assert_eq!(p.get("h").unwrap().as_int(), Some(255));
        assert_eq!(p.get("f").unwrap(), &Value::Number("1.5".into()));
        assert_eq!(p.get("s").unwrap().as_str(), Some("line joined"));
        assert_eq!(p.get("l").unwrap().as_str(), Some("raw\\n"));
    }

    #[test]
    fn rejects_what_it_cannot_read_faithfully() {
        for bad in [
            "a = 1\na = 2",
            "[t]\n[t]",
            "a = 1979-05-27",
            "a = \"open",
            "a = [1, 2",
            "a = 1 b = 2",
            "a = \"\"\"x\\ y\"\"\"",
            "a = 012",
            "a.b = 1\n[a.b]",
            "x = nope",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn array_tables_get_fresh_subtables() {
        let v = parse("[[r]]\n[r.s]\na = 1\n[[r]]\n[r.s]\na = 2\n").unwrap();
        let r = v.get("r").unwrap().as_array().unwrap();
        assert_eq!(r[1].path(&["s", "a"]).unwrap().as_int(), Some(2));
    }
}
