//! The value tree both parsers produce: the JSON of `cargo metadata` and
//! `ast-grep`, and the TOML of manifests and the gate files.

use std::collections::BTreeMap;
use std::fmt;

/// A parsed JSON or TOML value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// JSON `null` (TOML has none).
    Null,
    /// A boolean.
    Bool(bool),
    /// An integer that fits in `i64`.
    Int(i64),
    /// Any other number. Kept as written: no rule reads one.
    Number(String),
    /// A string.
    Str(String),
    /// An array.
    Array(Vec<Value>),
    /// An object or table, with keys in sorted order.
    Table(BTreeMap<String, Value>),
}

impl Value {
    /// The entry `key` of a table, or `None` for a missing key or a
    /// non-table.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Table(t) => t.get(key),
            _ => None,
        }
    }

    /// Follows a path of table keys.
    pub fn path(&self, keys: &[&str]) -> Option<&Value> {
        keys.iter().try_fold(self, |v, k| v.get(k))
    }

    /// The string, if this is one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The boolean, if this is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// The integer, if this is one.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// The elements, if this is an array.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// The entries, if this is a table.
    pub fn as_table(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Self::Table(t) => Some(t),
            _ => None,
        }
    }

    /// A short name of the value's kind, for error messages.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "boolean",
            Self::Int(_) | Self::Number(_) => "number",
            Self::Str(_) => "string",
            Self::Array(_) => "array",
            Self::Table(_) => "table",
        }
    }
}

/// A parse error with the 1-based line it was found on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The line of the input.
    pub line: usize,
    /// What was wrong.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// A cursor over the input's characters that tracks the line.
pub(crate) struct Cursor<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    pub(crate) line: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            chars: text.chars().peekable(),
            line: 1,
        }
    }

    pub(crate) fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// The character after the next one, without consuming either.
    pub(crate) fn peek2(&self) -> Option<char> {
        let mut it = self.chars.clone();
        it.next();
        it.next()
    }

    /// Whether the rest of the current line is spaces and tabs only (a
    /// carriage return before the newline counts as blank).
    pub(crate) fn rest_of_line_is_blank(&self) -> bool {
        let mut it = self.chars.clone();
        loop {
            match it.next() {
                Some(' ' | '\t' | '\r') => {}
                Some('\n') | None => return true,
                Some(_) => return false,
            }
        }
    }

    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c == Some('\n') {
            self.line += 1;
        }
        c
    }

    /// Consumes `c` if it is next.
    pub(crate) fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Consumes `s` if the input continues with it.
    pub(crate) fn eat_str(&mut self, s: &str) -> bool {
        let mut it = self.chars.clone();
        if s.chars().all(|c| it.next() == Some(c)) {
            s.chars().for_each(|_| {
                self.bump();
            });
            true
        } else {
            false
        }
    }

    pub(crate) fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            line: self.line,
            message: message.into(),
        }
    }

    /// Reads `n` hex digits as a Unicode scalar value.
    pub(crate) fn hex_scalar(&mut self, n: usize) -> Result<u32, ParseError> {
        let mut v: u32 = 0;
        for _ in 0..n {
            let d = self
                .bump()
                .and_then(|c| c.to_digit(16))
                .ok_or_else(|| self.error("bad hex digit in a \\u escape"))?;
            v = v * 16 + d;
        }
        Ok(v)
    }
}
