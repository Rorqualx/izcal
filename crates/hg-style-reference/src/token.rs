//! The CSS Syntax 3 tokenizer, written here and nowhere else.
//!
//! G20 is the reason this file exists rather than a call into
//! `hg-css-syntax`: a reference engine that shared the production
//! tokenizer would agree with it about a mis-tokenized stylesheet, and
//! agreement is the whole output of a differential. So the tokenizer
//! is written again, from the prose, by someone reading the prose —
//! and the two are allowed to disagree, because that disagreement is
//! the finding.
//!
//! This is Syntax 3 §4, with the preprocessing of §3.3 folded in: a
//! CR, an LF, a CR LF pair and a FF all become one LF, and a NULL or
//! an unpaired surrogate becomes U+FFFD.

/// One CSS token (Syntax 3 §4).
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// An identifier.
    Ident(String),
    /// A function name, without its `(`.
    Function(String),
    /// An at-keyword, without its `@`.
    AtKeyword(String),
    /// A hash, with whether it is an identifier (`#id`) or not
    /// (`#123`) — the type flag Syntax 3 gives it.
    Hash(String, HashKind),
    /// A string.
    Str(String),
    /// A string that ended at a newline, which is a parse error.
    BadString,
    /// A `url(...)` written without quotes.
    Url(String),
    /// A `url(` that could not be read.
    BadUrl,
    /// A delimiter: one code point that is nothing else.
    Delim(char),
    /// A number, with its text and whether it was written as an
    /// integer.
    Number(f64, bool),
    /// A percentage.
    Percentage(f64),
    /// A dimension: a number and its unit.
    Dimension(f64, bool, String),
    /// Whitespace, however much.
    Whitespace,
    /// `<!--`.
    Cdo,
    /// `-->`.
    Cdc,
    /// `:`.
    Colon,
    /// `;`.
    Semicolon,
    /// `,`.
    Comma,
    /// `[`.
    LeftSquare,
    /// `]`.
    RightSquare,
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `{`.
    LeftCurly,
    /// `}`.
    RightCurly,
}

/// Whether a hash is an identifier or just digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashKind {
    /// `#name` — usable as an id selector.
    Id,
    /// `#123` — usable only as a colour.
    Unrestricted,
}

/// The maximum code point, above which an escape becomes U+FFFD.
const MAX: u32 = 0x0010_FFFF;

/// Preprocesses the input (Syntax 3 §3.3).
#[must_use]
pub fn preprocess(input: &str) -> Vec<char> {
    let mut out = Vec::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push('\n');
            }
            '\u{C}' => out.push('\n'),
            '\0' => out.push('\u{FFFD}'),
            other => out.push(other),
        }
    }
    out
}

/// The tokenizer's cursor over preprocessed input.
pub struct Tokenizer {
    cs: Vec<char>,
    at: usize,
}

fn is_whitespace(c: char) -> bool {
    matches!(c, '\n' | '\t' | ' ')
}

/// An identifier start code point (§4.2).
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || !c.is_ascii()
}

/// An identifier code point (§4.2).
fn is_ident(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit() || c == '-'
}

/// A non-printable code point (§4.2).
fn is_non_printable(c: char) -> bool {
    matches!(c, '\0'..='\u{8}' | '\u{B}' | '\u{E}'..='\u{1F}' | '\u{7F}')
}

impl Tokenizer {
    /// A tokenizer over `input`, preprocessed.
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            cs: preprocess(input),
            at: 0,
        }
    }

    fn peek(&self, ahead: usize) -> Option<char> {
        self.at
            .checked_add(ahead)
            .and_then(|i| self.cs.get(i))
            .copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek(0);
        if c.is_some() {
            self.at = self.at.saturating_add(1);
        }
        c
    }

    /// Whether the next two code points are a valid escape (§4.3.8).
    fn valid_escape_at(&self, ahead: usize) -> bool {
        self.peek(ahead) == Some('\\') && self.peek(ahead.saturating_add(1)) != Some('\n')
    }

    /// Whether the next three would start an identifier (§4.3.9).
    fn would_start_ident(&self, ahead: usize) -> bool {
        match self.peek(ahead) {
            Some('-') => {
                let second = self.peek(ahead.saturating_add(1));
                second.is_some_and(is_ident_start)
                    || second == Some('-')
                    || self.valid_escape_at(ahead.saturating_add(1))
            }
            Some(c) if is_ident_start(c) => true,
            Some('\\') => self.valid_escape_at(ahead),
            _ => false,
        }
    }

    /// Whether the next three would start a number (§4.3.10).
    fn would_start_number(&self, ahead: usize) -> bool {
        match self.peek(ahead) {
            Some('+' | '-') => {
                let second = self.peek(ahead.saturating_add(1));
                second.is_some_and(|c| c.is_ascii_digit())
                    || (second == Some('.')
                        && self
                            .peek(ahead.saturating_add(2))
                            .is_some_and(|c| c.is_ascii_digit()))
            }
            Some('.') => self
                .peek(ahead.saturating_add(1))
                .is_some_and(|c| c.is_ascii_digit()),
            Some(c) => c.is_ascii_digit(),
            None => false,
        }
    }

    /// Consumes an escaped code point (§4.3.7).
    fn escape(&mut self) -> char {
        let Some(c) = self.bump() else {
            // "This is a parse error" — the token stream gets U+FFFD.
            return '\u{FFFD}';
        };
        if !c.is_ascii_hexdigit() {
            return c;
        }
        let mut value = c.to_digit(16).unwrap_or(0);
        let mut digits = 1_usize;
        while digits < 6
            && let Some(d) = self.peek(0).and_then(|c| c.to_digit(16))
        {
            value = value.saturating_mul(16).saturating_add(d);
            self.at = self.at.saturating_add(1);
            digits = digits.saturating_add(1);
        }
        if self.peek(0).is_some_and(is_whitespace) {
            self.at = self.at.saturating_add(1);
        }
        if value == 0 || value > MAX || (0xD800..=0xDFFF).contains(&value) {
            return '\u{FFFD}';
        }
        char::from_u32(value).unwrap_or('\u{FFFD}')
    }

    /// Consumes an identifier sequence (§4.3.11).
    fn ident_sequence(&mut self) -> String {
        let mut out = String::new();
        loop {
            match self.peek(0) {
                Some(c) if is_ident(c) => {
                    self.at = self.at.saturating_add(1);
                    out.push(c);
                }
                Some('\\') if self.valid_escape_at(0) => {
                    self.at = self.at.saturating_add(1);
                    out.push(self.escape());
                }
                _ => return out,
            }
        }
    }

    /// Consumes a number (§4.3.12): its value and whether it is an
    /// integer.
    fn number(&mut self) -> (f64, bool) {
        let mut text = String::new();
        let mut integer = true;
        if let Some(c @ ('+' | '-')) = self.peek(0) {
            text.push(c);
            self.at = self.at.saturating_add(1);
        }
        while let Some(c) = self.peek(0).filter(char::is_ascii_digit) {
            text.push(c);
            self.at = self.at.saturating_add(1);
        }
        if self.peek(0) == Some('.') && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
            integer = false;
            text.push('.');
            self.at = self.at.saturating_add(1);
            while let Some(c) = self.peek(0).filter(char::is_ascii_digit) {
                text.push(c);
                self.at = self.at.saturating_add(1);
            }
        }
        let exp_digits = if matches!(self.peek(1), Some('+' | '-')) {
            2
        } else {
            1
        };
        if matches!(self.peek(0), Some('e' | 'E'))
            && self.peek(exp_digits).is_some_and(|c| c.is_ascii_digit())
        {
            integer = false;
            for _ in 0..exp_digits {
                if let Some(c) = self.bump() {
                    text.push(c);
                }
            }
            while let Some(c) = self.peek(0).filter(char::is_ascii_digit) {
                text.push(c);
                self.at = self.at.saturating_add(1);
            }
        }
        (text.parse::<f64>().unwrap_or(0.0), integer)
    }

    /// Consumes a numeric token (§4.3.3).
    fn numeric(&mut self) -> Token {
        let (value, integer) = self.number();
        if self.would_start_ident(0) {
            let unit = self.ident_sequence();
            return Token::Dimension(value, integer, unit);
        }
        if self.peek(0) == Some('%') {
            self.at = self.at.saturating_add(1);
            return Token::Percentage(value);
        }
        Token::Number(value, integer)
    }

    /// Consumes a string token (§4.3.5).
    fn string(&mut self, quote: char) -> Token {
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Token::Str(out),
                Some(c) if c == quote => return Token::Str(out),
                Some('\n') => {
                    // A newline ends a string badly: the newline is
                    // *not* consumed, so the next token sees it.
                    self.at = self.at.saturating_sub(1);
                    return Token::BadString;
                }
                Some('\\') => match self.peek(0) {
                    None => {}
                    Some('\n') => self.at = self.at.saturating_add(1),
                    Some(_) => out.push(self.escape()),
                },
                Some(c) => out.push(c),
            }
        }
    }

    /// Consumes an unquoted url token (§4.3.6).
    fn url(&mut self) -> Token {
        let mut out = String::new();
        while self.peek(0).is_some_and(is_whitespace) {
            self.at = self.at.saturating_add(1);
        }
        loop {
            match self.bump() {
                None | Some(')') => return Token::Url(out),
                Some(c) if is_whitespace(c) => {
                    while self.peek(0).is_some_and(is_whitespace) {
                        self.at = self.at.saturating_add(1);
                    }
                    return match self.bump() {
                        None | Some(')') => Token::Url(out),
                        Some(_) => {
                            self.bad_url_remnants();
                            Token::BadUrl
                        }
                    };
                }
                Some('"' | '\'' | '(') => {
                    self.bad_url_remnants();
                    return Token::BadUrl;
                }
                Some(c) if is_non_printable(c) => {
                    self.bad_url_remnants();
                    return Token::BadUrl;
                }
                Some('\\') => {
                    if self.valid_escape_at(0) {
                        out.push(self.escape());
                    } else {
                        self.bad_url_remnants();
                        return Token::BadUrl;
                    }
                }
                Some(c) => out.push(c),
            }
        }
    }

    /// Consumes the remnants of a bad url (§4.3.14).
    fn bad_url_remnants(&mut self) {
        loop {
            match self.bump() {
                None | Some(')') => return,
                Some('\\') if self.valid_escape_at(0) => {
                    self.escape();
                }
                Some(_) => {}
            }
        }
    }

    /// Consumes an ident-like token (§4.3.4).
    fn ident_like(&mut self) -> Token {
        let name = self.ident_sequence();
        if self.peek(0) == Some('(') {
            self.at = self.at.saturating_add(1);
            if name.eq_ignore_ascii_case("url") {
                // `url(` followed by whitespace then a quote is a
                // function, not a url token.
                let mut ahead = 0_usize;
                while self.peek(ahead).is_some_and(is_whitespace) {
                    ahead = ahead.saturating_add(1);
                }
                if matches!(self.peek(ahead), Some('"' | '\'')) {
                    return Token::Function(name);
                }
                return self.url();
            }
            return Token::Function(name);
        }
        Token::Ident(name)
    }

    /// The next token, or `None` at the end of the input.
    ///
    /// Comments are consumed in a loop rather than by recursing: a
    /// long run of them is ordinary input for a fuzzer, and a
    /// reference engine that overflowed its stack would abort the run
    /// instead of giving a verdict.
    pub fn next_token(&mut self) -> Option<Token> {
        while self.peek(0) == Some('/') && self.peek(1) == Some('*') {
            self.at = self.at.saturating_add(2);
            while self.peek(0).is_some() {
                if self.peek(0) == Some('*') && self.peek(1) == Some('/') {
                    self.at = self.at.saturating_add(2);
                    break;
                }
                self.at = self.at.saturating_add(1);
            }
        }
        let c = self.peek(0)?;
        match c {
            c if is_whitespace(c) => {
                while self.peek(0).is_some_and(is_whitespace) {
                    self.at = self.at.saturating_add(1);
                }
                Some(Token::Whitespace)
            }
            '"' | '\'' => {
                self.at = self.at.saturating_add(1);
                Some(self.string(c))
            }
            '#' => {
                self.at = self.at.saturating_add(1);
                if self.peek(0).is_some_and(is_ident) || self.valid_escape_at(0) {
                    let kind = if self.would_start_ident(0) {
                        HashKind::Id
                    } else {
                        HashKind::Unrestricted
                    };
                    let name = self.ident_sequence();
                    return Some(Token::Hash(name, kind));
                }
                Some(Token::Delim('#'))
            }
            '(' => self.one(Token::LeftParen),
            ')' => self.one(Token::RightParen),
            '[' => self.one(Token::LeftSquare),
            ']' => self.one(Token::RightSquare),
            '{' => self.one(Token::LeftCurly),
            '}' => self.one(Token::RightCurly),
            ',' => self.one(Token::Comma),
            ':' => self.one(Token::Colon),
            ';' => self.one(Token::Semicolon),
            '+' | '.' => {
                if self.would_start_number(0) {
                    Some(self.numeric())
                } else {
                    self.one(Token::Delim(c))
                }
            }
            '-' => {
                if self.would_start_number(0) {
                    Some(self.numeric())
                } else if self.peek(1) == Some('-') && self.peek(2) == Some('>') {
                    self.at = self.at.saturating_add(3);
                    Some(Token::Cdc)
                } else if self.would_start_ident(0) {
                    Some(self.ident_like())
                } else {
                    self.one(Token::Delim('-'))
                }
            }
            '<' => {
                if self.peek(1) == Some('!')
                    && self.peek(2) == Some('-')
                    && self.peek(3) == Some('-')
                {
                    self.at = self.at.saturating_add(4);
                    Some(Token::Cdo)
                } else {
                    self.one(Token::Delim('<'))
                }
            }
            '@' => {
                self.at = self.at.saturating_add(1);
                if self.would_start_ident(0) {
                    let name = self.ident_sequence();
                    return Some(Token::AtKeyword(name));
                }
                Some(Token::Delim('@'))
            }
            '\\' => {
                if self.valid_escape_at(0) {
                    Some(self.ident_like())
                } else {
                    self.one(Token::Delim('\\'))
                }
            }
            c if c.is_ascii_digit() => Some(self.numeric()),
            c if is_ident_start(c) => Some(self.ident_like()),
            other => self.one(Token::Delim(other)),
        }
    }

    fn one(&mut self, token: Token) -> Option<Token> {
        self.at = self.at.saturating_add(1);
        Some(token)
    }

    /// Every token, to the end of the input.
    #[must_use]
    pub fn all(mut self) -> Vec<Token> {
        let mut out = Vec::new();
        while let Some(t) = self.next_token() {
            out.push(t);
        }
        out
    }
}

/// Every token of `input`.
#[must_use]
pub fn tokenize(input: &str) -> Vec<Token> {
    Tokenizer::new(input).all()
}

impl Token {
    /// The identifier's text, for a token that carries one.
    #[must_use]
    pub fn ident(&self) -> Option<&str> {
        match self {
            Self::Ident(s) | Self::Function(s) | Self::AtKeyword(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Whether this is whitespace.
    #[must_use]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace)
    }
}

impl core::fmt::Display for HashKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Id => "id",
            Self::Unrestricted => "unrestricted",
        })
    }
}

/// Every token of `input`, with whitespace removed.
#[must_use]
pub fn tokenize_significant(input: &str) -> Vec<Token> {
    tokenize(input)
        .into_iter()
        .filter(|t| !t.is_whitespace())
        .collect()
}
