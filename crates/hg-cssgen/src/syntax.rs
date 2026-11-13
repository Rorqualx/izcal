//! The `.hgprop` language: lexer, syntax tree and parser.
//!
//! `.hgprop` is the **Izcal overlay** of the style spec's ST-D6: what
//! the specifications leave as prose, plus a restatement of the parts of
//! `@webref/css` the engine was written against. It is Izcal's own
//! format, and like `.hgtok` there is no second source language.
//!
//! # The grammar
//!
//! ```text
//! file     := item*
//! item     := kind | computed | property | alias
//! kind     := "kind" Name String
//! computed := "computed" String "=>" Name
//! property := "property" Name "@" String "{" field* "}"
//! field    := "initial"   String
//!           | "inherited" ("yes" | "no")
//!           | "animation" String
//!           | "syntax"    String
//!           | "computed"  String
//!           | "longhands" Name ("," Name)*
//!           | "affects"   Affect+
//!           | "hook"      Name
//! alias    := "alias" Name "=>" Name
//! ```
//!
//! # Why the overlay repeats what webref already says
//!
//! Five of a property's fields — `initial`, `inherited`, `animation`,
//! `syntax` and `computed` — are written here *and* extracted by webref
//! from the specification. Repeating them is the point. The engine is
//! written against a particular initial value and a particular grammar;
//! when the specification changes one, webref's next release carries the
//! change and this file does not, and the build stops with the
//! difference named ([SP 6.6][SP], ST-7). Without the restatement a
//! specification change is silent, and silent is how a property ends up
//! doing one thing while its code assumes another.
//!
//! So the overlay is never regenerated from webref to make a
//! disagreement go away: that would turn the check off. A re-pin is
//! triaged difference by difference, and ST-7 blocks the re-pin until
//! the overlay is fixed.
//!
//! # What the overlay adds
//!
//! - **`computed`'s classification.** Each distinct computed-value text
//!   webref carries is classified once, into a closed set of kinds that
//!   names *which conversion* the property needs — not its type, which
//!   is per-property. The classification is exhaustive in both
//!   directions: a text webref carries and nothing classifies is an
//!   error, and so is a classification of a text webref no longer has.
//! - **`affects`**, what a change to the property dirties, which no
//!   specification states and layout needs.
//! - **`hook`**, for the handful whose computed value is defined by
//!   prose a table cannot carry.
//!
//! [SP]: ../../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md

use std::fmt;

/// A problem in a description, with where it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The file it was found in.
    pub file: String,
    /// Its line, counting from one.
    pub line: usize,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.file.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}:{}: {}", self.file, self.line, self.message)
        }
    }
}

impl std::error::Error for Error {}

/// What a change to a property dirties.
///
/// `Nothing` and `Unknown` stand alone: the first says a change is
/// invisible to layout, paint and the compositor, and the second says
/// the decision has not been made. `Unknown` is legal so that the open
/// decision is written down and counted rather than guessed; the
/// generator reports how many properties still carry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Affect {
    /// Layout must run again.
    Layout,
    /// Paint must run again; layout need not.
    Paint,
    /// Only the compositor's inputs change.
    Composite,
    /// The SVG layout pass must run again (the SVG spec's R5).
    SvgGeometry,
    /// The SVG reference graph must be rebuilt (the SVG spec's R5).
    SvgResource,
    /// Nothing downstream changes.
    Nothing,
    /// Not yet decided.
    Unknown,
}

impl Affect {
    /// Its word in a description.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::Paint => "paint",
            Self::Composite => "composite",
            Self::SvgGeometry => "svg-geometry",
            Self::SvgResource => "svg-resource",
            Self::Nothing => "nothing",
            Self::Unknown => "unknown",
        }
    }

    /// The word, read back.
    #[must_use]
    pub fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "layout" => Self::Layout,
            "paint" => Self::Paint,
            "composite" => Self::Composite,
            "svg-geometry" => Self::SvgGeometry,
            "svg-resource" => Self::SvgResource,
            "nothing" => Self::Nothing,
            "unknown" => Self::Unknown,
            _ => return None,
        })
    }

    /// Whether this one may not be written beside another.
    #[must_use]
    pub const fn stands_alone(self) -> bool {
        matches!(self, Self::Nothing | Self::Unknown)
    }
}

/// A computed-value kind: one conversion family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindDecl {
    /// Its name in a description, kebab-case.
    pub name: String,
    /// What it means, one sentence, emitted as the variant's doc.
    pub doc: String,
    /// The line it was written on.
    pub line: usize,
}

/// One computed-value text, classified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedDecl {
    /// The text, exactly as webref carries it.
    pub text: String,
    /// The kind it belongs to.
    pub kind: String,
    /// The line it was written on.
    pub line: usize,
}

/// A property's block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropertyDecl {
    /// The property name, as CSS spells it.
    pub name: String,
    /// Its defining URL, restated from webref.
    pub href: String,
    /// Its initial value, restated. `None` when webref carries none.
    pub initial: Option<String>,
    /// What webref says about inheritance, restated word for word.
    /// `None` when webref carries nothing.
    ///
    /// It is a **text**, not a flag: at the 8.7.9 pin 76 shorthands say
    /// "see individual properties", nine properties say nothing, and
    /// five say `yes?`, `?`, `n/a` or "no (but see prose)". The
    /// specifications do not always answer the question the cascade has
    /// to answer, so the answer is [`PropertyDecl::inherits`], written
    /// here and owned by Izcal.
    pub inherited: Option<String>,
    /// Whether the cascade inherits it. Izcal's own decision, with no
    /// counterpart in webref, and required of every property.
    pub inherits: Option<bool>,
    /// Its animation type, restated. `None` when webref carries none.
    pub animation: Option<String>,
    /// Its value grammar, restated. `None` when webref carries none.
    pub syntax: Option<String>,
    /// Its computed-value text, restated. `None` when webref carries
    /// none — eight properties at the 8.7.9 pin, all scraped from specs
    /// that state it in a form webref did not extract.
    pub computed: Option<String>,
    /// Its longhands, restated, in webref's order. Empty for a property
    /// that is not a shorthand.
    pub longhands: Vec<String>,
    /// What a change dirties. Never empty after parsing.
    pub affects: Vec<Affect>,
    /// The hand-written function that computes the value, for a
    /// property whose kind is the prose one.
    pub hook: Option<String>,
    /// The initial value, where `@webref/css` extracted none and so
    /// the restatement above is absent. Izcal's own, with the
    /// specification that states it named in a comment beside it.
    ///
    /// Three at the 8.7.9 pin. A property with no initial value cannot
    /// be cascaded, so this is required wherever webref has none and
    /// the property is not a mapping.
    pub initial_is: Option<String>,
    /// The property this one maps to, for a name that is not a property
    /// of its own.
    ///
    /// The Compatibility Standard's §3.4.4 "property mappings" are
    /// six: `-webkit-box-align`, `-webkit-box-flex`,
    /// `-webkit-box-ordinal-group`, `-webkit-box-orient` and
    /// `-webkit-box-pack` map to `align-items`, `flex-grow`, `order`,
    /// `flex-direction` and `justify-content`, and CSS UI 4 requires
    /// `-webkit-user-select` to be supported as an alias of
    /// `user-select`. They are not legacy *name* aliases — the value is
    /// mapped too — so webref keeps them as properties, with no initial
    /// value and no computed value, which is exactly right: nothing is
    /// stored under these names.
    pub maps_to: Option<String>,
    /// The computed-value kind, where the specification's own text is
    /// wrong **for this property** and classifying the text would
    /// classify it wrongly.
    ///
    /// One at the 8.7.9 pin: css-borders-4 gives `border-shape` the
    /// computed value of `box-shadow-color`, which sits beside it in
    /// the same draft, while its grammar has no color in it. The
    /// restated `computed` text still agrees with webref word for word,
    /// so the cross-check is untouched and the day the draft is fixed
    /// arrives as a difference; this says only what Izcal computes
    /// meanwhile.
    pub kind: Option<String>,
    /// The line the block starts on.
    pub line: usize,
}

/// A legacy alias: one name that is another property under a different
/// spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDecl {
    /// The alias.
    pub name: String,
    /// The property it stands for.
    pub target: String,
    /// The line it was written on.
    pub line: usize,
}

/// One parsed file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct File {
    /// Its kind declarations, in the order written.
    pub kinds: Vec<KindDecl>,
    /// Its classifications, in the order written.
    pub computed: Vec<ComputedDecl>,
    /// Its property blocks, in the order written.
    pub properties: Vec<PropertyDecl>,
    /// Its aliases, in the order written.
    pub aliases: Vec<AliasDecl>,
}

/// A token of the language.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    /// A bare word: a keyword, a property name or a kind name.
    Word(String),
    /// A double-quoted string, unescaped.
    Str(String),
    /// One of `{ } @ => ,`.
    Punct(&'static str),
}

struct Lexer<'a> {
    s: Vec<char>,
    at: usize,
    line: usize,
    file: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(text: &str, file: &'a str) -> Self {
        Self {
            s: text.chars().collect(),
            at: 0,
            line: 1,
            file,
        }
    }

    fn err(&self, message: impl Into<String>) -> Error {
        Error {
            file: self.file.to_owned(),
            line: self.line,
            message: message.into(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.s.get(self.at).copied()
    }

    fn bump(&mut self) {
        if self.peek() == Some('\n') {
            self.line = self.line.saturating_add(1);
        }
        self.at = self.at.saturating_add(1);
    }

    /// Whitespace and `#` comments.
    fn trivia(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r' | '\n') => self.bump(),
                Some('#') => {
                    while !matches!(self.peek(), None | Some('\n')) {
                        self.bump();
                    }
                }
                _ => return,
            }
        }
    }

    /// The next token and the line it started on.
    fn next(&mut self) -> Result<Option<(Tok, usize)>, Error> {
        self.trivia();
        let line = self.line;
        let Some(c) = self.peek() else {
            return Ok(None);
        };
        let tok = match c {
            '{' | '}' | '@' | ',' => {
                self.bump();
                Tok::Punct(match c {
                    '{' => "{",
                    '}' => "}",
                    '@' => "@",
                    _ => ",",
                })
            }
            '=' => {
                self.bump();
                if self.peek() != Some('>') {
                    return Err(self.err("`=` is only ever part of `=>`"));
                }
                self.bump();
                Tok::Punct("=>")
            }
            '"' => Tok::Str(self.string()?),
            _ => Tok::Word(self.word()?),
        };
        Ok(Some((tok, line)))
    }

    fn string(&mut self) -> Result<String, Error> {
        self.bump(); // the opening quote
        let mut out = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => return Err(self.err("a string with no closing quote")),
                Some('"') => {
                    self.bump();
                    return Ok(out);
                }
                Some('\\') => {
                    self.bump();
                    match self.peek() {
                        Some(c @ ('"' | '\\')) => out.push(c),
                        _ => return Err(self.err("only \\\" and \\\\ are escapes")),
                    }
                    self.bump();
                }
                Some(c) => {
                    out.push(c);
                    self.bump();
                }
            }
        }
    }

    /// A bare word: letters, digits, `-`, `_`, `.` and `*`, which is
    /// enough for every CSS property name, kind name and keyword here.
    fn word(&mut self) -> Result<String, Error> {
        let mut out = String::new();
        while matches!(self.peek(),
            Some(c) if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '*'))
        {
            if let Some(c) = self.peek() {
                out.push(c);
            }
            self.bump();
        }
        if out.is_empty() {
            return Err(self.err(format!(
                "{:?} does not start a word, a string or punctuation",
                self.peek().unwrap_or(' ')
            )));
        }
        Ok(out)
    }
}

/// Parses one file.
///
/// # Errors
/// [`Error`] for the first problem, naming the file and line.
pub fn parse(text: &str, file: &str) -> Result<File, Error> {
    // Every token first, and a lexer error stops the run. Collecting
    // what lexed and parsing that would turn an unterminated string
    // into a short file, which parses as something else entirely.
    let mut lex = Lexer::new(text, file);
    let mut toks = Vec::new();
    while let Some(t) = lex.next()? {
        toks.push(t);
    }
    Parser { toks, at: 0, file }.file()
}

struct Parser<'a> {
    toks: Vec<(Tok, usize)>,
    at: usize,
    file: &'a str,
}

impl Parser<'_> {
    fn err(&self, message: impl Into<String>) -> Error {
        Error {
            file: self.file.to_owned(),
            line: self.line(),
            message: message.into(),
        }
    }

    fn line(&self) -> usize {
        self.toks
            .get(self.at)
            .or_else(|| self.toks.last())
            .map_or(0, |(_, l)| *l)
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at).map(|(t, _)| t)
    }

    fn bump(&mut self) {
        self.at = self.at.saturating_add(1);
    }

    fn word(&mut self, what: &str) -> Result<String, Error> {
        match self.peek() {
            Some(Tok::Word(w)) => {
                let w = w.clone();
                self.bump();
                Ok(w)
            }
            other => Err(self.err(format!("expected {what}, found {}", show(other)))),
        }
    }

    fn string(&mut self, what: &str) -> Result<String, Error> {
        match self.peek() {
            Some(Tok::Str(s)) => {
                let s = s.clone();
                self.bump();
                Ok(s)
            }
            other => Err(self.err(format!("expected {what}, found {}", show(other)))),
        }
    }

    fn punct(&mut self, p: &str) -> Result<(), Error> {
        match self.peek() {
            Some(Tok::Punct(q)) if *q == p => {
                self.bump();
                Ok(())
            }
            other => Err(self.err(format!("expected `{p}`, found {}", show(other)))),
        }
    }

    fn file(mut self) -> Result<File, Error> {
        let mut out = File::default();
        while self.peek().is_some() {
            let line = self.line();
            let head = self.word("`kind`, `computed`, `property` or `alias`")?;
            match head.as_str() {
                "kind" => {
                    let name = self.word("a kind name")?;
                    let doc = self.string("what the kind means, in one sentence")?;
                    out.kinds.push(KindDecl { name, doc, line });
                }
                "computed" => {
                    let text = self.string("a computed-value text, as webref carries it")?;
                    self.punct("=>")?;
                    let kind = self.word("a kind name")?;
                    out.computed.push(ComputedDecl { text, kind, line });
                }
                "alias" => {
                    let name = self.word("the alias")?;
                    self.punct("=>")?;
                    let target = self.word("the property it stands for")?;
                    out.aliases.push(AliasDecl { name, target, line });
                }
                "property" => out.properties.push(self.property(line)?),
                other => {
                    return Err(self.err(format!(
                        "`{other}` is not an item; expected `kind`, `computed`, `property` or \
                         `alias`"
                    )));
                }
            }
        }
        Ok(out)
    }

    fn property(&mut self, line: usize) -> Result<PropertyDecl, Error> {
        let name = self.word("a property name")?;
        self.punct("@")?;
        let href = self.string("the property's defining URL")?;
        self.punct("{")?;
        let mut p = PropertyDecl {
            name,
            href,
            line,
            ..PropertyDecl::default()
        };
        let mut seen: Vec<String> = Vec::new();
        loop {
            if matches!(self.peek(), Some(Tok::Punct("}"))) {
                self.bump();
                break;
            }
            let at = self.line();
            let field = self.word("a field or `}`")?;
            if seen.contains(&field) {
                return Err(Error {
                    file: self.file.to_owned(),
                    line: at,
                    message: format!("{} gives `{field}` twice", p.name),
                });
            }
            seen.push(field.clone());
            match field.as_str() {
                "initial" => p.initial = Some(self.string("the initial value")?),
                "animation" => p.animation = Some(self.string("the animation type")?),
                "syntax" => p.syntax = Some(self.string("the value grammar")?),
                "computed" => p.computed = Some(self.string("the computed-value text")?),
                "hook" => p.hook = Some(self.word("the hook's function name")?),
                "kind" => p.kind = Some(self.word("a kind name")?),
                "initial-is" => p.initial_is = Some(self.string("the initial value")?),
                "maps-to" => p.maps_to = Some(self.word("the property it maps to")?),
                "inherited" => p.inherited = Some(self.string("what webref says")?),
                "inherits" => {
                    let w = self.word("`yes` or `no`")?;
                    p.inherits = Some(match w.as_str() {
                        "yes" => true,
                        "no" => false,
                        other => {
                            return Err(Error {
                                file: self.file.to_owned(),
                                line: at,
                                message: format!("`inherits` is `yes` or `no`, not `{other}`"),
                            });
                        }
                    });
                }
                "longhands" => loop {
                    p.longhands.push(self.word("a longhand's name")?);
                    if matches!(self.peek(), Some(Tok::Punct(","))) {
                        self.bump();
                    } else {
                        break;
                    }
                },
                "affects" => {
                    while let Some(Tok::Word(w)) = self.peek() {
                        let Some(a) = Affect::from_word(w) else { break };
                        p.affects.push(a);
                        self.bump();
                    }
                    if p.affects.is_empty() {
                        return Err(Error {
                            file: self.file.to_owned(),
                            line: at,
                            message: format!(
                                "{} gives `affects` with nothing after it; write `unknown` if the \
                                 decision is open",
                                p.name
                            ),
                        });
                    }
                }
                other => {
                    return Err(Error {
                        file: self.file.to_owned(),
                        line: at,
                        message: format!("`{other}` is not a field of a property block"),
                    });
                }
            }
        }
        if p.inherits.is_none() {
            return Err(Error {
                file: self.file.to_owned(),
                line: p.line,
                message: format!(
                    "{} has no `inherits`; the cascade needs a yes or no, and webref does not \
                     always give one",
                    p.name
                ),
            });
        }
        if p.affects.is_empty() {
            return Err(Error {
                file: self.file.to_owned(),
                line: p.line,
                message: format!(
                    "{} has no `affects`; every property says what a change dirties, and `unknown` \
                     is how an open decision is written",
                    p.name
                ),
            });
        }
        Ok(p)
    }
}

fn show(t: Option<&Tok>) -> String {
    match t {
        None => "the end of the file".to_owned(),
        Some(Tok::Word(w)) => format!("`{w}`"),
        Some(Tok::Str(s)) => format!("the string {s:?}"),
        Some(Tok::Punct(p)) => format!("`{p}`"),
    }
}
