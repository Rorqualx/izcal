//! `hg-json`: Izcal's own JSON reader. Tier T, layer `tools`.
//!
//! Izcal's generators and oracle runners have no dependencies, and
//! several of their pinned inputs are JSON: WHATWG's `entities.json`,
//! which `hg-htmlgen` compiles into the tokenizer; html5lib-tests'
//! `.test` files, which `hg-html5lib` reads; and `@webref/css`'s
//! `css.json`, which `hg-cssgen` imports. One reader serves them all,
//! rather than one per generator, because three copies of a parser
//! disagree in three ways.
//!
//! It is deliberately small: it reads a whole document into a tree and
//! keeps numbers as their text, so an integer stays exact. Nothing here
//! ships — the tables the generators emit from these inputs do.
//!
//! One decision worth stating: an **unpaired surrogate** escape decodes
//! to U+FFFD. Rust's `String` cannot hold a lone surrogate, and the HTML
//! input stream is where the standard raises
//! `surrogate-in-input-stream` for one anyway — `hg-html-driver`'s, not
//! the tokenizer's. The caller says so where it matters.

use std::fmt;

/// A JSON value. Numbers keep their text, so an integer stays exact.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number, as written.
    Num(String),
    /// A string, decoded.
    Str(String),
    /// An array.
    Arr(Vec<Json>),
    /// An object, in the order written.
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Its text, when it is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Its elements, when it is an array.
    #[must_use]
    pub fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Self::Arr(v) => Some(v),
            _ => None,
        }
    }

    /// The value of `key`, when it is an object that has one.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Its pairs, when it is an object.
    #[must_use]
    pub fn as_obj(&self) -> Option<&[(String, Json)]> {
        match self {
            Self::Obj(kv) => Some(kv),
            _ => None,
        }
    }

    /// Its value as an integer, when it is a number.
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Num(n) => n.parse().ok(),
            _ => None,
        }
    }

    /// Its value, when it is a boolean.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Where a JSON document would not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The byte offset.
    pub at: usize,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "byte {}: {}", self.at, self.message)
    }
}

impl std::error::Error for Error {}

/// Parses one JSON document.
///
/// # Errors
/// [`Error`] with the byte offset of the first problem.
pub fn parse(text: &str) -> Result<Json, Error> {
    let mut s = Scan {
        s: text.as_bytes(),
        at: 0,
    };
    s.ws();
    let v = s.value()?;
    s.ws();
    if s.peek().is_some() {
        return Err(s.err("trailing text"));
    }
    Ok(v)
}

struct Scan<'a> {
    s: &'a [u8],
    at: usize,
}

impl Scan<'_> {
    fn err(&self, message: impl Into<String>) -> Error {
        Error {
            at: self.at,
            message: message.into(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.at).copied()
    }

    fn bump(&mut self) {
        self.at = self.at.saturating_add(1);
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.bump();
        }
    }

    fn byte(&mut self, want: u8) -> Result<(), Error> {
        if self.peek() == Some(want) {
            self.bump();
            Ok(())
        } else {
            Err(self.err(format!("expected {:?}", char::from(want))))
        }
    }

    fn word(&mut self, want: &str) -> Result<(), Error> {
        let end = self.at.saturating_add(want.len());
        if self.s.get(self.at..end) == Some(want.as_bytes()) {
            self.at = end;
            Ok(())
        } else {
            Err(self.err(format!("expected `{want}`")))
        }
    }

    fn value(&mut self) -> Result<Json, Error> {
        match self.peek() {
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b't') => {
                self.word("true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.word("false")?;
                Ok(Json::Bool(false))
            }
            Some(b'n') => {
                self.word("null")?;
                Ok(Json::Null)
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => self.number(),
            _ => Err(self.err("expected a value")),
        }
    }

    /// A number, by JSON's own grammar rather than by scanning the
    /// characters a number may contain.
    ///
    /// `-? ( "0" | [1-9][0-9]* ) ( "." [0-9]+ )? ( [eE] [+-]? [0-9]+ )?`
    ///
    /// The loose version — take every digit, sign, dot and `e` — reads
    /// `1-2`, `..` and a bare `e` as numbers. This reader's whole job is
    /// to notice when a pinned input stops being the shape it was, so a
    /// scan that cannot say no is the wrong tool. A leading zero is
    /// rejected for the same reason: JSON forbids it, and a file that
    /// starts producing `01` has changed in a way worth a message.
    fn number(&mut self) -> Result<Json, Error> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.bump();
        }
        match self.peek() {
            Some(b'0') => self.bump(),
            Some(c) if c.is_ascii_digit() => self.digits()?,
            _ => return Err(self.err("a number with no digits")),
        }
        if self.peek() == Some(b'.') {
            self.bump();
            self.digits()?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.bump();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.bump();
            }
            self.digits()?;
        }
        // A digit here means a leading zero (`01`) or a second point.
        if matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == b'.') {
            return Err(self.err("a number that does not end where it should"));
        }
        let bytes = self
            .s
            .get(start..self.at)
            .ok_or_else(|| self.err("a short number"))?;
        let text = std::str::from_utf8(bytes).map_err(|e| self.err(e.to_string()))?;
        Ok(Json::Num(text.to_owned()))
    }

    /// One or more digits, consumed.
    fn digits(&mut self) -> Result<(), Error> {
        if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            return Err(self.err("a digit"));
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        Ok(())
    }

    fn array(&mut self) -> Result<Json, Error> {
        self.byte(b'[')?;
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.bump();
            return Ok(Json::Arr(out));
        }
        loop {
            self.ws();
            out.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.bump(),
                Some(b']') => {
                    self.bump();
                    return Ok(Json::Arr(out));
                }
                _ => return Err(self.err("expected `,` or `]`")),
            }
        }
    }

    fn object(&mut self) -> Result<Json, Error> {
        self.byte(b'{')?;
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.bump();
            return Ok(Json::Obj(out));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.ws();
            self.byte(b':')?;
            self.ws();
            out.push((k, self.value()?));
            self.ws();
            match self.peek() {
                Some(b',') => self.bump(),
                Some(b'}') => {
                    self.bump();
                    return Ok(Json::Obj(out));
                }
                _ => return Err(self.err("expected `,` or `}`")),
            }
        }
    }

    fn string(&mut self) -> Result<String, Error> {
        self.byte(b'"')?;
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(self.err("unterminated string"));
            };
            self.bump();
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let Some(k) = self.peek() else {
                        return Err(self.err("unterminated escape"));
                    };
                    self.bump();
                    match k {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(self.unicode_escape()?),
                        other => {
                            return Err(
                                self.err(format!("unknown escape `\\{}`", char::from(other)))
                            );
                        }
                    }
                }
                // The text is UTF-8, so a character above ASCII is
                // copied through by its bytes.
                _ => {
                    let start = self.at.saturating_sub(1);
                    let len = utf8_len(c).map_err(|e| self.err(e))?;
                    let end = start.saturating_add(len);
                    let bytes = self
                        .s
                        .get(start..end)
                        .ok_or_else(|| self.err("truncated UTF-8"))?;
                    let s = std::str::from_utf8(bytes).map_err(|e| self.err(e.to_string()))?;
                    out.push_str(s);
                    self.at = end;
                }
            }
        }
    }

    /// `\uXXXX`, pairing surrogates. An unpaired one becomes U+FFFD:
    /// see the module docs.
    fn unicode_escape(&mut self) -> Result<char, Error> {
        let first = self.hex4()?;
        if (0xD800..0xDC00).contains(&first) {
            let save = self.at;
            if self.byte(b'\\').is_ok() && self.byte(b'u').is_ok() {
                let second = self.hex4()?;
                if (0xDC00..0xE000).contains(&second) {
                    let hi = first.saturating_sub(0xD800).saturating_mul(0x400);
                    let lo = second.saturating_sub(0xDC00);
                    let n = hi.saturating_add(lo).saturating_add(0x1_0000);
                    return char::from_u32(n).ok_or_else(|| self.err("not a character"));
                }
            }
            self.at = save;
            return Ok('\u{FFFD}');
        }
        Ok(char::from_u32(first).unwrap_or('\u{FFFD}'))
    }

    fn hex4(&mut self) -> Result<u32, Error> {
        let end = self.at.saturating_add(4);
        let bytes = self
            .s
            .get(self.at..end)
            .ok_or_else(|| self.err("a short \\u escape"))?;
        let s = std::str::from_utf8(bytes).map_err(|e| self.err(e.to_string()))?;
        let n = u32::from_str_radix(s, 16).map_err(|e| self.err(e.to_string()))?;
        self.at = end;
        Ok(n)
    }
}

fn utf8_len(first: u8) -> Result<usize, String> {
    Ok(match first {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        other => return Err(format!("{other:#04x} does not start a UTF-8 character")),
    })
}
