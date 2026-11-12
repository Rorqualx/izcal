//! The `.wast` token grammar.
//!
//! The text format's "Lexical Format" gives five tokens that matter to a
//! script reader — `(`, `)`, an identifier, a string and a word — with
//! whitespace and comments between them. Everything else the format
//! distinguishes (keywords, the number forms, reserved tokens) is a word
//! here, told apart later by what reads it: [`crate::script`] matches a
//! command's name, [`crate::value`] decodes a number.
//!
//! # Why the full grammar, for a reader that skips modules
//!
//! [`crate::script`] keeps module bodies as byte ranges, so it might
//! look as though finding them were a matter of counting parentheses.
//! It is not. The suite's `comments.wast` holds, deliberately,
//! `(;comment";)` — a quotation mark inside a comment — `(;)))))))))));)`
//! and `(;(((((((((( ;)` — unbalanced parentheses inside comments —
//! `;;)` and `;; (;` — comment openers and closers inside line comments
//! — `(;comment(;nested(;further;)nested;)comment;)` — three levels of
//! nesting — and a NUL byte with every other control character inside a
//! block comment. A counter fed raw bytes mis-reads every one of them.
//! A span is only as trustworthy as the lexer that ends it, so the
//! lexer is written to the grammar and tested against that file.

use std::fmt;

/// A byte range of the source text.
///
/// Both ends fall on token boundaries, which are character boundaries:
/// a word ends at the first byte that cannot continue it, and a
/// multi-byte character's first byte is one of those.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// The offset of the first byte.
    pub start: usize,
    /// The offset one past the last byte.
    pub end: usize,
}

impl Span {
    /// The source text the span covers.
    #[must_use]
    pub fn text<'a>(&self, src: &'a str) -> &'a str {
        &src[self.start..self.end]
    }

    /// The span from the start of `self` to the end of `other`.
    #[must_use]
    pub fn to(self, other: Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }
}

/// What a token is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// An identifier: `$` and a name, as `$f` or `$"a b"`.
    Id,
    /// A quoted string, quotation marks included in the span.
    Str,
    /// A single run of identifier characters: a keyword such as
    /// `module`, a number such as `-0x1p-3`, or a NaN pattern such as
    /// `nan:canonical`.
    ///
    /// The grammar also reserves a run that is none of those, `0$x`
    /// among them, and this kind does not separate that case out:
    /// telling a keyword from a number from a reserved run means
    /// applying the number grammar, which is [`crate::value`]'s work
    /// and is done where a number is actually wanted. A run that is no
    /// keyword and no number is refused there, or by the command parser
    /// as a word it does not know, so nothing reaches an engine that
    /// the format would reject — except inside a module body, which
    /// this crate does not read at all and the engine judges.
    Word,
    /// A reserved token that could not be anything else: a run that
    /// mixes identifier characters, strings or the punctuation
    /// `, ; [ ] { }`. `"a""b"` is one and so is `x")"y`. They occur
    /// inside annotations, which a reader skips; anywhere else the
    /// parser refuses them, which is what the grammar means by
    /// reserving them.
    Reserved,
}

/// One token and where it came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    /// What it is.
    pub kind: Kind,
    /// Where it is.
    pub span: Span,
}

/// A source position, counted from one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    /// The line, counting `\n`.
    pub line: usize,
    /// The column, in characters.
    pub column: usize,
}

/// Where in the source an offset falls.
///
/// Offsets past the end report the position of the end, so an error at
/// end of file still names a place.
#[must_use]
pub fn position(src: &str, offset: usize) -> Position {
    let offset = offset.min(src.len());
    let mut line = 1;
    let mut column = 1;
    for (i, c) in src.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    Position { line, column }
}

/// A place where the source does not match the grammar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// The byte offset the error is reported at.
    pub offset: usize,
    /// What was wrong, as a sentence without the position.
    pub message: String,
}

impl Error {
    /// The error with its position in `src`, for a person to read.
    #[must_use]
    pub fn display(&self, src: &str) -> String {
        let p = position(src, self.offset);
        format!("{}:{}: {}", p.line, p.column, self.message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "byte {}: {}", self.offset, self.message)
    }
}

/// True for the text format's identifier characters: the digits, the
/// letters and the punctuation the format lists. Everything in the set
/// is ASCII, so a byte test is the whole test.
#[must_use]
pub const fn is_idchar(b: u8) -> bool {
    matches!(b,
        b'0'..=b'9'
        | b'A'..=b'Z'
        | b'a'..=b'z'
        | b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-'
        | b'.' | b'/' | b':' | b'<' | b'=' | b'>' | b'?' | b'@' | b'\\'
        | b'^' | b'_' | b'`' | b'|' | b'~')
}

/// True for the punctuation the grammar admits only inside a reserved
/// token: `,`, `;`, `[`, `]`, `{` and `}`.
#[must_use]
pub const fn is_reserved_punct(b: u8) -> bool {
    matches!(b, b',' | b';' | b'[' | b']' | b'{' | b'}')
}

/// Reads a `.wast` source into tokens.
#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// A lexer over `src`, positioned at its start.
    #[must_use]
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    /// The source being read.
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.src
    }

    /// The next token, or `None` at the end of the source.
    pub fn next_token(&mut self) -> Result<Option<Token>, Error> {
        self.skip_trivia()?;
        let start = self.pos;
        let Some(&b) = self.bytes.get(self.pos) else {
            return Ok(None);
        };
        let kind = match b {
            b'(' => {
                self.pos += 1;
                Kind::LParen
            }
            b')' => {
                self.pos += 1;
                Kind::RParen
            }
            _ => self.scan_run(start, b)?,
        };
        Ok(Some(Token {
            kind,
            span: Span {
                start,
                end: self.pos,
            },
        }))
    }

    /// All the tokens, or the first error.
    pub fn tokens(mut self) -> Result<Vec<Token>, Error> {
        let mut out = Vec::new();
        while let Some(t) = self.next_token()? {
            out.push(t);
        }
        Ok(out)
    }

    /// Skips whitespace, line comments and block comments, which may
    /// nest. Called before every token, so comments never reach the
    /// parser and never affect a span's ends.
    fn skip_trivia(&mut self) -> Result<(), Error> {
        loop {
            match self.bytes.get(self.pos) {
                Some(b' ' | b'\t' | b'\n' | b'\r') => self.pos += 1,
                // `;;` runs to the end of the line. Its contents are
                // raw: `;; (;` opens nothing.
                Some(b';') if self.bytes.get(self.pos + 1) == Some(&b';') => {
                    self.pos += 2;
                    while let Some(&b) = self.bytes.get(self.pos) {
                        self.pos += 1;
                        if b == b'\n' {
                            break;
                        }
                    }
                }
                // `(;` ... `;)`, nesting. Inside one, only those two
                // pairs mean anything: not quotation marks, not lone
                // parentheses, not `;;`, not control characters.
                Some(b'(') if self.bytes.get(self.pos + 1) == Some(&b';') => {
                    let open = self.pos;
                    self.pos += 2;
                    let mut depth = 1usize;
                    while depth > 0 {
                        match (self.bytes.get(self.pos), self.bytes.get(self.pos + 1)) {
                            (Some(b'('), Some(b';')) => {
                                depth += 1;
                                self.pos += 2;
                            }
                            (Some(b';'), Some(b')')) => {
                                depth -= 1;
                                self.pos += 2;
                            }
                            (Some(_), _) => self.pos += 1,
                            (None, _) => {
                                return Err(
                                    self.err(open, "a block comment is never closed".to_string())
                                );
                            }
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    /// Scans a string from its opening quotation mark to its closing
    /// one, checking the escapes but decoding nothing:
    /// [`crate::value::string_bytes`] decodes.
    fn scan_string(&mut self) -> Result<(), Error> {
        let open = self.pos;
        self.pos += 1;
        loop {
            let Some(&b) = self.bytes.get(self.pos) else {
                return Err(self.err(open, "a string is never closed".to_string()));
            };
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(());
                }
                b'\\' => self.scan_escape()?,
                // The format admits any character from U+0020 up, bar
                // the quotation mark and the backslash handled above,
                // and bar DEL. A control character must be escaped, so
                // a raw one is an error and not a character to keep.
                0x00..=0x1f | 0x7f => {
                    return Err(self.err(
                        self.pos,
                        format!(
                            "a string holds no raw control character; write \\{b:02x} for this one"
                        ),
                    ));
                }
                _ => self.pos += 1,
            }
        }
    }

    /// Scans one escape, the backslash included.
    fn scan_escape(&mut self) -> Result<(), Error> {
        let at = self.pos;
        self.pos += 1;
        match self.bytes.get(self.pos) {
            Some(b't' | b'n' | b'r' | b'"' | b'\'' | b'\\') => {
                self.pos += 1;
                Ok(())
            }
            // `\u{...}`: at least one hexadecimal digit, then `}`.
            Some(b'u') => {
                self.pos += 1;
                if self.bytes.get(self.pos) != Some(&b'{') {
                    return Err(self.err(at, "\\u must be followed by {".to_string()));
                }
                self.pos += 1;
                let digits = self.pos;
                while self.bytes.get(self.pos).is_some_and(u8::is_ascii_hexdigit) {
                    self.pos += 1;
                }
                if self.pos == digits {
                    return Err(self.err(at, "\\u{} needs a hexadecimal digit".to_string()));
                }
                if self.bytes.get(self.pos) != Some(&b'}') {
                    return Err(self.err(at, "\\u{ is never closed".to_string()));
                }
                self.pos += 1;
                Ok(())
            }
            // Two hexadecimal digits are one byte.
            Some(b) if b.is_ascii_hexdigit() => {
                let hi = self.pos;
                self.pos += 1;
                if !self.bytes.get(self.pos).is_some_and(u8::is_ascii_hexdigit) {
                    return Err(self.err(
                        hi,
                        "an escaped byte needs two hexadecimal digits".to_string(),
                    ));
                }
                self.pos += 1;
                Ok(())
            }
            Some(&b) => {
                let c = self.src[self.pos..].chars().next().unwrap_or('\u{fffd}');
                Err(self.err(at, format!("\\{} is not an escape", describe(c, b))))
            }
            None => Err(self.err(at, "a string ends in a backslash".to_string())),
        }
    }

    /// Scans one token that is not a parenthesis.
    ///
    /// The grammar's longest-match rule decides this, not the first
    /// character: a token is the longest run of identifier characters,
    /// strings and the punctuation `, ; [ ] { }` that follows, and only
    /// a run that is one thing is a word, an identifier or a string.
    /// `0$x` and `"a""b"` are each a single reserved token, which is how
    /// the format forces tokens apart with space or parentheses.
    fn scan_run(&mut self, start: usize, first: u8) -> Result<Kind, Error> {
        if !is_idchar(first) && first != b'"' && !is_reserved_punct(first) {
            // A byte that can begin no token. Name the character, not
            // the byte, so a stray non-ASCII character reads as what it
            // looks like in an editor.
            let c = self.src[self.pos..].chars().next().unwrap_or('\u{fffd}');
            return Err(self.err(
                start,
                format!("{} cannot start a token", describe(c, first)),
            ));
        }

        // `$` and a string is the quoted identifier form, which the
        // suite's `id.wast` uses: `(func $" random \t \n stuff ")`.
        if first == b'$' && self.bytes.get(self.pos + 1) == Some(&b'"') {
            self.pos += 1;
            self.scan_string()?;
            return Ok(if self.run_continues() {
                self.finish_run()?;
                Kind::Reserved
            } else {
                Kind::Id
            });
        }

        let mut parts = 0usize;
        let mut idchars = false;
        let mut string = false;
        let mut punct = false;
        loop {
            match self.bytes.get(self.pos) {
                Some(&b) if is_idchar(b) => {
                    while self.bytes.get(self.pos).is_some_and(|&b| is_idchar(b)) {
                        self.pos += 1;
                    }
                    idchars = true;
                }
                Some(b'"') => {
                    self.scan_string()?;
                    string = true;
                }
                // A run of the reserved punctuation, stopping before
                // `;;`, which opens a comment and so ends any token.
                Some(&b) if is_reserved_punct(b) => {
                    if b == b';' && self.bytes.get(self.pos + 1) == Some(&b';') {
                        break;
                    }
                    self.pos += 1;
                    punct = true;
                }
                _ => break,
            }
            parts += 1;
        }
        Ok(match (parts, idchars, string, punct) {
            (1, true, false, false) => {
                if self.src.as_bytes()[start] == b'$' && self.pos - start > 1 {
                    Kind::Id
                } else {
                    Kind::Word
                }
            }
            (1, false, true, false) => Kind::Str,
            _ => Kind::Reserved,
        })
    }

    /// True when another part of a token run follows immediately.
    fn run_continues(&self) -> bool {
        self.bytes
            .get(self.pos)
            .is_some_and(|&b| is_idchar(b) || b == b'"' || is_reserved_punct(b))
    }

    /// Consumes the rest of a run whose classification is already
    /// settled as reserved.
    fn finish_run(&mut self) -> Result<(), Error> {
        while let Some(&b) = self.bytes.get(self.pos) {
            if is_idchar(b) {
                self.pos += 1;
            } else if b == b'"' {
                self.scan_string()?;
            } else if is_reserved_punct(b)
                && !(b == b';' && self.bytes.get(self.pos + 1) == Some(&b';'))
            {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn err(&self, offset: usize, message: String) -> Error {
        Error { offset, message }
    }
}

/// Names a character for an error: printable ASCII as itself, anything
/// else by its code point, so a NUL or a non-breaking space is legible.
fn describe(c: char, b: u8) -> String {
    if b.is_ascii_graphic() {
        format!("`{c}`")
    } else if c == ' ' {
        "a space".to_string()
    } else {
        format!("U+{:04X}", c as u32)
    }
}

#[cfg(test)]
mod tests;
