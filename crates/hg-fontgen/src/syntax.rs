//! The `.hgfont` language: lexer, syntax tree and parser.
//!
//! `.hgfont` describes a binary table as the OpenType prose does — a
//! sequence of named fields with types, each carrying the anchor of the
//! specification clause that defines it. It is Izcal's own language
//! (fonts and text 6.2): no official machine-readable OpenType
//! description exists, and the community ones are partial or another
//! project's artifact, so nothing here is derived from one (M-1 part 3).
//!
//! # The grammar
//!
//! ```text
//! file    := item*
//! item    := record | table
//! record  := "record" Name anchor "{" field* "}"
//! table   := "table" Name params? anchor "{" field* "}"
//! params  := "(" param ("," param)* ")"
//! param   := Name ":" scalar
//! field   := Name type anchor
//! type    := scalar | Name | "[" expr "]" (scalar | Name)
//! expr    := term (("+" | "-") term)* ("/" Integer)?
//! term    := Name | Integer
//! anchor  := "@" String
//! ```
//!
//! A `record` holds only fixed-size fields, so its size is a constant
//! and an array of records can be indexed by multiplication. A `table`
//! may hold arrays, and the offset of a field after one is computed with
//! checked arithmetic at read time.

use std::fmt;

/// A scalar type, with the width the OpenType data-types table gives it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scalar {
    /// `uint8`.
    U8,
    /// `uint16`.
    U16,
    /// `uint24`.
    U24,
    /// `uint32`.
    U32,
    /// `int8`.
    I8,
    /// `int16` (also `FWORD`).
    I16,
    /// `int32`.
    I32,
    /// `F2DOT14`, read as its raw `int16`.
    F2Dot14,
    /// `Fixed` (16.16), read as its raw `int32`.
    Fixed,
    /// `LONGDATETIME`, read as `int64`.
    LongDateTime,
    /// `Tag`: four bytes, read as `[u8; 4]`.
    Tag,
    /// `Offset16`, read as `u16`.
    Offset16,
    /// `Offset24`, read as `u32`.
    Offset24,
    /// `Offset32`, read as `u32`.
    Offset32,
}

impl Scalar {
    /// Its width in bytes.
    #[must_use]
    pub const fn width(self) -> usize {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 | Self::F2Dot14 | Self::Offset16 => 2,
            Self::U24 | Self::Offset24 => 3,
            Self::U32 | Self::I32 | Self::Fixed | Self::Offset32 | Self::Tag => 4,
            Self::LongDateTime => 8,
        }
    }

    /// The Rust type an accessor returns.
    #[must_use]
    pub const fn rust(self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::U16 | Self::Offset16 => "u16",
            Self::U24 | Self::U32 | Self::Offset24 | Self::Offset32 => "u32",
            Self::I8 => "i8",
            Self::I16 | Self::F2Dot14 => "i16",
            Self::I32 | Self::Fixed => "i32",
            Self::LongDateTime => "i64",
            Self::Tag => "[u8; 4]",
        }
    }

    /// The reader in `hg_fontfmt::read` that gets it.
    #[must_use]
    pub const fn reader(self) -> &'static str {
        match self {
            Self::U8 => "u8_at",
            Self::U16 | Self::Offset16 => "u16_at",
            Self::U24 | Self::Offset24 => "u24_at",
            Self::U32 | Self::Offset32 => "u32_at",
            Self::I8 => "i8_at",
            Self::I16 | Self::F2Dot14 => "i16_at",
            Self::I32 | Self::Fixed => "i32_at",
            Self::LongDateTime => "i64_at",
            Self::Tag => "tag_at",
        }
    }

    fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u24" => Self::U24,
            "u32" => Self::U32,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "f2dot14" => Self::F2Dot14,
            "fixed" => Self::Fixed,
            "longdatetime" => Self::LongDateTime,
            "tag" => Self::Tag,
            "offset16" => Self::Offset16,
            "offset24" => Self::Offset24,
            "offset32" => Self::Offset32,
            _ => return None,
        })
    }
}

/// A count expression: a sum and difference of names and integers,
/// optionally divided by a literal.
///
/// The divisor is written last and applies to the whole sum, which is
/// what the tables need — `cmap` format 4 counts its segments as
/// `segCountX2 / 2`, and its glyph array as a byte length divided by
/// two. Anything richer would need precedence rules that no described
/// table has yet asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    /// The terms, each with its sign.
    pub terms: Vec<(bool, Term)>,
    /// The literal the sum is divided by, when one is written.
    pub divisor: Option<u32>,
}

/// One term of a count expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// A parameter or an earlier field of the same table.
    Name(String),
    /// A literal.
    Int(u32),
}

/// What a field holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// One scalar.
    Scalar(Scalar),
    /// One record, by name.
    Record(String),
    /// `count` elements of a scalar or record.
    Array {
        /// How many.
        count: Expr,
        /// What each one is: a scalar, or a record by name.
        elem: Box<Ty>,
    },
}

/// One field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Its name, in `snake_case`.
    pub name: String,
    /// Its type.
    pub ty: Ty,
    /// The specification clause that defines it.
    pub anchor: String,
    /// The line it was written on, for diagnostics.
    pub line: usize,
}

/// A fixed-size structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Its name, in `UpperCamelCase`.
    pub name: String,
    /// The clause that defines it.
    pub anchor: String,
    /// Its fields.
    pub fields: Vec<Field>,
    /// The line it starts on.
    pub line: usize,
}

/// A table, which may take parameters another table supplies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// Its name, in `UpperCamelCase`.
    pub name: String,
    /// The clause that defines it.
    pub anchor: String,
    /// Parameters, each a scalar.
    pub params: Vec<(String, Scalar)>,
    /// Its fields.
    pub fields: Vec<Field>,
    /// The line it starts on.
    pub line: usize,
}

/// Everything one `.hgfont` file declares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct File {
    /// The records, in the order written.
    pub records: Vec<Record>,
    /// The tables, in the order written.
    pub tables: Vec<Table>,
}

/// A syntax or check error, with the file and line that caused it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The file's label.
    pub file: String,
    /// The line, counting from one.
    pub line: usize,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

impl std::error::Error for Error {}

/// One token.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Word(String),
    Str(String),
    Int(u32),
    Punct(char),
}

struct Lexer {
    toks: Vec<(Tok, usize)>,
    at: usize,
    file: String,
}

impl Lexer {
    fn new(src: &str, file: &str) -> Result<Self, Error> {
        let mut toks = Vec::new();
        for (n, raw) in src.lines().enumerate() {
            let line = n.saturating_add(1);
            let text = raw;
            let mut chars = text.char_indices().peekable();
            while let Some((i, c)) = chars.next() {
                if c.is_whitespace() {
                    continue;
                }
                // A `#` starts a comment only outside a string: anchors
                // carry fragments, and `otspec/head#xMin` is one token.
                if c == '#' {
                    break;
                }
                if c == '"' {
                    let rest = text.get(i.saturating_add(1)..).unwrap_or("");
                    let end = rest.find('"').ok_or_else(|| Error {
                        file: file.to_owned(),
                        line,
                        message: "unterminated string".into(),
                    })?;
                    let s = rest.get(..end).unwrap_or("").to_owned();
                    for _ in 0..=end {
                        chars.next();
                    }
                    toks.push((Tok::Str(s), line));
                    continue;
                }
                if c.is_ascii_digit() {
                    let mut v = String::from(c);
                    while let Some((_, d)) = chars.peek() {
                        if d.is_ascii_digit() {
                            v.push(*d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let n = v.parse::<u32>().map_err(|_| Error {
                        file: file.to_owned(),
                        line,
                        message: format!("{v} does not fit in a u32"),
                    })?;
                    toks.push((Tok::Int(n), line));
                    continue;
                }
                if c.is_ascii_alphabetic() || c == '_' {
                    let mut v = String::from(c);
                    while let Some((_, d)) = chars.peek() {
                        if d.is_ascii_alphanumeric() || *d == '_' {
                            v.push(*d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    toks.push((Tok::Word(v), line));
                    continue;
                }
                if "{}[]():,@+-/".contains(c) {
                    toks.push((Tok::Punct(c), line));
                    continue;
                }
                return Err(Error {
                    file: file.to_owned(),
                    line,
                    message: format!("unexpected character {c:?}"),
                });
            }
        }
        Ok(Self {
            toks,
            at: 0,
            file: file.to_owned(),
        })
    }

    fn line(&self) -> usize {
        self.toks
            .get(self.at)
            .or_else(|| self.toks.last())
            .map_or(0, |(_, l)| *l)
    }

    fn err(&self, message: impl Into<String>) -> Error {
        Error {
            file: self.file.clone(),
            line: self.line(),
            message: message.into(),
        }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at).map(|(t, _)| t)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.at).map(|(t, _)| t.clone());
        if t.is_some() {
            self.at = self.at.saturating_add(1);
        }
        t
    }

    fn punct(&mut self, c: char) -> Result<(), Error> {
        match self.next() {
            Some(Tok::Punct(p)) if p == c => Ok(()),
            other => Err(self.err(format!("expected {c:?}, found {other:?}"))),
        }
    }

    fn eat_punct(&mut self, c: char) -> bool {
        if matches!(self.peek(), Some(Tok::Punct(p)) if *p == c) {
            self.at = self.at.saturating_add(1);
            true
        } else {
            false
        }
    }

    fn word(&mut self) -> Result<String, Error> {
        match self.next() {
            Some(Tok::Word(w)) => Ok(w),
            other => Err(self.err(format!("expected a name, found {other:?}"))),
        }
    }

    fn anchor(&mut self) -> Result<String, Error> {
        self.punct('@')?;
        match self.next() {
            Some(Tok::Str(s)) if !s.is_empty() => Ok(s),
            Some(Tok::Str(_)) => Err(self.err("an anchor may not be empty")),
            other => Err(self.err(format!("expected an anchor string, found {other:?}"))),
        }
    }
}

/// Parses one `.hgfont` file.
///
/// # Errors
/// [`Error`] naming the line when the text is not the grammar above.
pub fn parse(src: &str, file: &str) -> Result<File, Error> {
    let mut lx = Lexer::new(src, file)?;
    let mut out = File::default();
    while let Some(tok) = lx.peek().cloned() {
        let Tok::Word(kw) = tok else {
            return Err(lx.err("expected `record` or `table`"));
        };
        let line = lx.line();
        match kw.as_str() {
            "record" => {
                lx.next();
                let name = lx.word()?;
                let anchor = lx.anchor()?;
                let fields = fields(&mut lx)?;
                out.records.push(Record {
                    name,
                    anchor,
                    fields,
                    line,
                });
            }
            "table" => {
                lx.next();
                let name = lx.word()?;
                let mut params = Vec::new();
                if lx.eat_punct('(') {
                    loop {
                        let p = lx.word()?;
                        lx.punct(':')?;
                        let t = lx.word()?;
                        let s = Scalar::from_word(&t)
                            .ok_or_else(|| lx.err(format!("{t} is not a scalar type")))?;
                        params.push((p, s));
                        if !lx.eat_punct(',') {
                            break;
                        }
                    }
                    lx.punct(')')?;
                }
                let anchor = lx.anchor()?;
                let fields = fields(&mut lx)?;
                out.tables.push(Table {
                    name,
                    anchor,
                    params,
                    fields,
                    line,
                });
            }
            other => return Err(lx.err(format!("expected `record` or `table`, found `{other}`"))),
        }
    }
    Ok(out)
}

fn fields(lx: &mut Lexer) -> Result<Vec<Field>, Error> {
    lx.punct('{')?;
    let mut out = Vec::new();
    while !lx.eat_punct('}') {
        let line = lx.line();
        let name = lx.word()?;
        let ty = ty(lx)?;
        let anchor = lx.anchor()?;
        out.push(Field {
            name,
            ty,
            anchor,
            line,
        });
    }
    Ok(out)
}

fn ty(lx: &mut Lexer) -> Result<Ty, Error> {
    if lx.eat_punct('[') {
        let count = expr(lx)?;
        lx.punct(']')?;
        let elem = elem_ty(lx)?;
        return Ok(Ty::Array {
            count,
            elem: Box::new(elem),
        });
    }
    elem_ty(lx)
}

fn elem_ty(lx: &mut Lexer) -> Result<Ty, Error> {
    let w = lx.word()?;
    Ok(match Scalar::from_word(&w) {
        Some(s) => Ty::Scalar(s),
        None => Ty::Record(w),
    })
}

fn expr(lx: &mut Lexer) -> Result<Expr, Error> {
    let mut terms = Vec::new();
    terms.push((true, term(lx)?));
    loop {
        if lx.eat_punct('+') {
            terms.push((true, term(lx)?));
        } else if lx.eat_punct('-') {
            terms.push((false, term(lx)?));
        } else {
            break;
        }
    }
    let mut divisor = None;
    if lx.eat_punct('/') {
        match lx.next() {
            Some(Tok::Int(0)) => return Err(lx.err("a divisor may not be zero")),
            Some(Tok::Int(n)) => divisor = Some(n),
            other => {
                return Err(lx.err(format!("expected a number after `/`, found {other:?}")));
            }
        }
    }
    Ok(Expr { terms, divisor })
}

fn term(lx: &mut Lexer) -> Result<Term, Error> {
    match lx.next() {
        Some(Tok::Word(w)) => Ok(Term::Name(w)),
        Some(Tok::Int(n)) => Ok(Term::Int(n)),
        other => Err(lx.err(format!("expected a name or a number, found {other:?}"))),
    }
}
