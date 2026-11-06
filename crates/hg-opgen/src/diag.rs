//! Diagnostics: every rejection names the check of the language note's
//! Section 6 (OC-1 to OC-17) that made it, and where.

use std::fmt;

/// A position in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    /// The file's index in the input list.
    pub file: u16,
    /// Line, from 1.
    pub line: u32,
    /// Column, from 1, in characters.
    pub col: u32,
}

/// One rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    /// Where.
    pub span: Span,
    /// The check (`OC-1` to `OC-17`).
    pub check: &'static str,
    /// What is wrong.
    pub msg: String,
}

impl Diag {
    /// A rejection by `check` at `span`.
    pub fn new(span: Span, check: &'static str, msg: impl Into<String>) -> Self {
        Self {
            span,
            check,
            msg: msg.into(),
        }
    }

    /// The diagnostic with file names resolved from `files`.
    pub fn render(&self, files: &[String]) -> String {
        let name = files
            .get(usize::from(self.span.file))
            .map_or("<input>", String::as_str);
        format!(
            "{name}:{}:{}: {}: {}",
            self.span.line, self.span.col, self.check, self.msg
        )
    }
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}: {}",
            self.span.line, self.span.col, self.check, self.msg
        )
    }
}

/// A list of rejections; generation stops if it is not empty.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Diags(pub Vec<Diag>);

impl Diags {
    /// Adds a rejection.
    pub fn push(&mut self, d: Diag) {
        self.0.push(d);
    }

    /// Adds a rejection by `check` at `span`.
    pub fn err(&mut self, span: Span, check: &'static str, msg: impl Into<String>) {
        self.0.push(Diag::new(span, check, msg));
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether any rejection is by `check`.
    pub fn has(&self, check: &str) -> bool {
        self.0.iter().any(|d| d.check == check)
    }

    /// Sorted by position, deduplicated: deterministic output (C15).
    pub fn sorted(mut self) -> Self {
        self.0
            .sort_by(|a, b| (a.span, a.check, &a.msg).cmp(&(b.span, b.check, &b.msg)));
        self.0.dedup();
        self
    }
}
