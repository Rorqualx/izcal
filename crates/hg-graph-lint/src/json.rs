//! A JSON reader (RFC 8259) for `cargo metadata` and `ast-grep --json`.
//! Strict: trailing commas, comments and unpaired surrogates are errors.

use std::collections::BTreeMap;

use crate::value::{Cursor, ParseError, Value};

/// Parses one JSON document.
///
/// # Errors
/// A [`ParseError`] naming the line of the first defect.
pub fn parse(text: &str) -> Result<Value, ParseError> {
    let mut c = Cursor::new(text);
    let v = value(&mut c, 0)?;
    ws(&mut c);
    if c.peek().is_some() {
        return Err(c.error("trailing characters after the JSON value"));
    }
    Ok(v)
}

/// Deep enough for any `cargo metadata` document, shallow enough that a
/// hostile input cannot exhaust the stack.
const MAX_DEPTH: usize = 128;

fn ws(c: &mut Cursor<'_>) {
    while matches!(c.peek(), Some(' ' | '\t' | '\n' | '\r')) {
        c.bump();
    }
}

fn value(c: &mut Cursor<'_>, depth: usize) -> Result<Value, ParseError> {
    if depth > MAX_DEPTH {
        return Err(c.error("JSON nested too deeply"));
    }
    ws(c);
    match c.peek() {
        Some('{') => {
            c.bump();
            let mut t = BTreeMap::new();
            ws(c);
            if c.eat('}') {
                return Ok(Value::Table(t));
            }
            loop {
                ws(c);
                if c.peek() != Some('"') {
                    return Err(c.error("expected a string key"));
                }
                let k = string(c)?;
                ws(c);
                if !c.eat(':') {
                    return Err(c.error("expected ':' after a key"));
                }
                let v = value(c, depth + 1)?;
                if t.insert(k.clone(), v).is_some() {
                    return Err(c.error(format!("duplicate key {k:?}")));
                }
                ws(c);
                if c.eat(',') {
                    continue;
                }
                if c.eat('}') {
                    return Ok(Value::Table(t));
                }
                return Err(c.error("expected ',' or '}'"));
            }
        }
        Some('[') => {
            c.bump();
            let mut a = Vec::new();
            ws(c);
            if c.eat(']') {
                return Ok(Value::Array(a));
            }
            loop {
                a.push(value(c, depth + 1)?);
                ws(c);
                if c.eat(',') {
                    continue;
                }
                if c.eat(']') {
                    return Ok(Value::Array(a));
                }
                return Err(c.error("expected ',' or ']'"));
            }
        }
        Some('"') => Ok(Value::Str(string(c)?)),
        Some('t') if c.eat_str("true") => Ok(Value::Bool(true)),
        Some('f') if c.eat_str("false") => Ok(Value::Bool(false)),
        Some('n') if c.eat_str("null") => Ok(Value::Null),
        Some('-' | '0'..='9') => number(c),
        Some(other) => Err(c.error(format!("unexpected character {other:?}"))),
        None => Err(c.error("unexpected end of input")),
    }
}

fn string(c: &mut Cursor<'_>) -> Result<String, ParseError> {
    c.bump(); // the opening quote
    let mut s = String::new();
    loop {
        match c.bump() {
            None => return Err(c.error("unterminated string")),
            Some('"') => return Ok(s),
            Some('\\') => {
                let ch = match c.bump() {
                    Some('"') => '"',
                    Some('\\') => '\\',
                    Some('/') => '/',
                    Some('b') => '\u{8}',
                    Some('f') => '\u{c}',
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('u') => unicode_escape(c)?,
                    _ => return Err(c.error("bad escape in a string")),
                };
                s.push(ch);
            }
            Some(ch) if u32::from(ch) < 0x20 => {
                return Err(c.error("control character in a string"));
            }
            Some(ch) => s.push(ch),
        }
    }
}

/// `\uXXXX`, joining a surrogate pair into one scalar value.
fn unicode_escape(c: &mut Cursor<'_>) -> Result<char, ParseError> {
    let hi = c.hex_scalar(4)?;
    let code = if (0xD800..0xDC00).contains(&hi) {
        if !(c.eat('\\') && c.eat('u')) {
            return Err(c.error("unpaired surrogate in a \\u escape"));
        }
        let lo = c.hex_scalar(4)?;
        if !(0xDC00..0xE000).contains(&lo) {
            return Err(c.error("unpaired surrogate in a \\u escape"));
        }
        0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
    } else {
        hi
    };
    char::from_u32(code).ok_or_else(|| c.error("invalid \\u escape"))
}

fn number(c: &mut Cursor<'_>) -> Result<Value, ParseError> {
    let mut s = String::new();
    while let Some(ch) = c.peek() {
        if ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E') {
            s.push(ch);
            c.bump();
        } else {
            break;
        }
    }
    // Validate the grammar: -?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?
    let b = s.as_bytes();
    let mut i = 0;
    if b.get(i) == Some(&b'-') {
        i += 1;
    }
    let digits = |i: &mut usize| {
        let start = *i;
        while b.get(*i).is_some_and(u8::is_ascii_digit) {
            *i += 1;
        }
        *i > start
    };
    let int_start = i;
    let ok_int = digits(&mut i) && !(b.get(int_start) == Some(&b'0') && i - int_start > 1);
    let mut ok = ok_int;
    if ok && b.get(i) == Some(&b'.') {
        i += 1;
        ok = digits(&mut i);
    }
    if ok && matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        ok = digits(&mut i);
    }
    if ok && i == b.len() {
        Ok(s.parse::<i64>().map_or(Value::Number(s), Value::Int))
    } else {
        Err(c.error(format!("bad number {s:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_shapes_cargo_metadata_uses() {
        let v = parse(
            r#"{"a": [1, -2, 3.5e2, true, false, null], "b": {"c": "d\u00e9\ud83d\ude00\n"}}"#,
        )
        .unwrap();
        assert_eq!(v.path(&["b", "c"]).unwrap().as_str(), Some("dé😀\n"));
        let a = v.get("a").unwrap().as_array().unwrap();
        assert_eq!(a[0], Value::Int(1));
        assert_eq!(a[1], Value::Int(-2));
        assert_eq!(a[2], Value::Number("3.5e2".into()));
        assert_eq!(a[5], Value::Null);
    }

    #[test]
    fn rejects_malformed_input() {
        for bad in [
            "{\"a\": 1,}",
            "[1 2]",
            "\"\\ud800\"",
            "01",
            "{\"a\": 1, \"a\": 2}",
            "[1] x",
            "\"tab\there\"",
            "1.",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        let deep = "[".repeat(200) + &"]".repeat(200);
        assert!(parse(&deep).is_err());
    }
}
