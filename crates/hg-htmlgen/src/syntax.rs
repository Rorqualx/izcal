//! The `.hgtok` language: lexer, syntax tree and parser.
//!
//! `.hgtok` describes the HTML tokenizer the way the HTML Standard
//! writes it (§13.2.5): one block per state, and inside it one rule per
//! code-point class, each naming the actions the prose names and the
//! clause it came from. It is Izcal's own format (style spec 6, ST-D3),
//! and there is no second source language — the lesson of Gecko's
//! translator.
//!
//! # The grammar
//!
//! ```text
//! file   := item*
//! item   := "state" Name anchor "{" rule* "}"
//!         | "extern" "state" Name anchor
//! rule   := match "=>" actions anchor?
//! match  := Char | Class | "_"
//! actions:= action ("," action)*
//! action := verb arg*
//!         | "if" cond "{" actions "}" "else" "{" actions "}"
//! anchor := "@" String
//! ```
//!
//! An `extern state` is one the prose does not write as a per-character
//! table — "consume the maximum number of characters possible" in the
//! named character reference state is the example — so it is implemented
//! by an anchored function in the runtime and the generator only routes
//! to it. Marking those explicitly is what keeps the rest honest: a
//! state that *could* be described is never quietly hand-written.

use std::fmt;

/// What a rule matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Match {
    /// One code point.
    Char(char),
    /// A named class of code points.
    Class(Class),
    /// End of file.
    Eof,
    /// "Anything else", the prose's own last row.
    Anything,
}

/// A named class of code points, as the standard's prose names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Tab, line feed, form feed or space.
    Whitespace,
    /// U+0000.
    Null,
    /// `A`–`Z` or `a`–`z`.
    AsciiAlpha,
    /// `A`–`Z`.
    AsciiUpper,
    /// `a`–`z`.
    AsciiLower,
    /// `0`–`9`.
    AsciiDigit,
    /// `0`–`9`, `A`–`F` or `a`–`f`.
    AsciiHexDigit,
}

impl Class {
    fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "whitespace" => Self::Whitespace,
            "null" => Self::Null,
            "ascii_alpha" => Self::AsciiAlpha,
            "ascii_upper" => Self::AsciiUpper,
            "ascii_lower" => Self::AsciiLower,
            "ascii_digit" => Self::AsciiDigit,
            "ascii_hex" => Self::AsciiHexDigit,
            _ => return None,
        })
    }

    /// The Rust test for membership, given a `char` called `c`.
    #[must_use]
    pub const fn test(self) -> &'static str {
        match self {
            Self::Whitespace => "matches!(c, '\\t' | '\\n' | '\\u{000C}' | ' ')",
            Self::Null => "c == '\\0'",
            Self::AsciiAlpha => "c.is_ascii_alphabetic()",
            Self::AsciiUpper => "c.is_ascii_uppercase()",
            Self::AsciiLower => "c.is_ascii_lowercase()",
            Self::AsciiDigit => "c.is_ascii_digit()",
            Self::AsciiHexDigit => "c.is_ascii_hexdigit()",
        }
    }
}

/// A buffer the tokenizer appends to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Buffer {
    /// The current tag token's name.
    TagName,
    /// The current attribute's name.
    AttrName,
    /// The current attribute's value.
    AttrValue,
    /// The current comment token's data.
    Comment,
    /// The current DOCTYPE token's name.
    DoctypeName,
    /// Its public identifier.
    PublicId,
    /// Its system identifier.
    SystemId,
    /// The temporary buffer.
    Temp,
}

impl Buffer {
    fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "tag_name" => Self::TagName,
            "attr_name" => Self::AttrName,
            "attr_value" => Self::AttrValue,
            "comment" => Self::Comment,
            "doctype_name" => Self::DoctypeName,
            "public_id" => Self::PublicId,
            "system_id" => Self::SystemId,
            "temp" => Self::Temp,
            _ => return None,
        })
    }

    /// The method on the tokenizer that appends to it.
    #[must_use]
    pub const fn method(self) -> &'static str {
        match self {
            Self::TagName => "push_tag_name",
            Self::AttrName => "push_attr_name",
            Self::AttrValue => "push_attr_value",
            Self::Comment => "push_comment",
            Self::DoctypeName => "push_doctype_name",
            Self::PublicId => "push_public_id",
            Self::SystemId => "push_system_id",
            Self::Temp => "push_temp",
        }
    }
}

/// A flag a rule may set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// The current tag's self-closing flag.
    SelfClosing,
    /// The current DOCTYPE's force-quirks flag.
    ForceQuirks,
    /// Its public identifier, set to the empty string.
    PublicIdEmpty,
    /// Its system identifier, set to the empty string.
    SystemIdEmpty,
}

impl Flag {
    fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "self_closing" => Self::SelfClosing,
            "force_quirks" => Self::ForceQuirks,
            "public_id_empty" => Self::PublicIdEmpty,
            "system_id_empty" => Self::SystemIdEmpty,
            _ => return None,
        })
    }

    /// The method that applies it.
    #[must_use]
    pub const fn method(self) -> &'static str {
        match self {
            Self::SelfClosing => "set_self_closing",
            Self::ForceQuirks => "set_force_quirks",
            Self::PublicIdEmpty => "set_public_id_empty",
            Self::SystemIdEmpty => "set_system_id_empty",
        }
    }
}

/// A condition a rule may branch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    /// "If the current end tag token is an appropriate end tag token".
    Appropriate,
    /// "If the temporary buffer is the string `script`".
    TempIsScript,
}

impl Cond {
    fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "appropriate" => Self::Appropriate,
            "temp_is_script" => Self::TempIsScript,
            _ => return None,
        })
    }

    /// The method that tests it.
    #[must_use]
    pub const fn method(self) -> &'static str {
        match self {
            Self::Appropriate => "is_appropriate_end_tag",
            Self::TempIsScript => "temp_is_script",
        }
    }
}

/// One action of a rule, in the prose's own vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// "Switch to the X state".
    Switch(String),
    /// "Reconsume in the X state".
    Reconsume(String),
    /// "Set the return state to X".
    SetReturn(String),
    /// "Switch to the return state".
    SwitchReturn,
    /// "Reconsume in the return state".
    ReconsumeReturn,
    /// "Emit the current input character as a character token".
    EmitCurrent,
    /// "Emit a U+XXXX character token".
    EmitChar(char),
    /// "Emit an end-of-file token".
    EmitEof,
    /// "Emit the current tag, comment or DOCTYPE token".
    EmitToken,
    /// "Emit the temporary buffer as character tokens".
    EmitTemp,
    /// "Create a new start tag token".
    CreateStartTag,
    /// "Create a new end tag token".
    CreateEndTag,
    /// "Create a comment token".
    CreateComment,
    /// "Create a new DOCTYPE token".
    CreateDoctype,
    /// "Start a new attribute".
    StartAttribute,
    /// "Append the current input character to X".
    Append(Buffer),
    /// "Append the lowercase version of the current input character".
    AppendLower(Buffer),
    /// "Append U+XXXX to X".
    AppendChar(Buffer, char),
    /// "Set the temporary buffer to the empty string".
    ClearTemp,
    /// "Flush code points consumed as a character reference".
    FlushTemp,
    /// Set a flag.
    Set(Flag),
    /// "This is a *named* parse error".
    Error(String),
    /// A branch the prose writes as "if … otherwise".
    If {
        /// What is tested.
        cond: Cond,
        /// Taken when it holds.
        then: Vec<Action>,
        /// Taken when it does not.
        otherwise: Vec<Action>,
    },
}

/// One rule of a state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// What it matches.
    pub matches: Match,
    /// What it does.
    pub actions: Vec<Action>,
    /// The clause it came from, when the rule carries its own.
    pub anchor: Option<String>,
    /// The line it was written on.
    pub line: usize,
}

/// One state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// Its name, in `snake_case`.
    pub name: String,
    /// The clause that defines it.
    pub anchor: String,
    /// Its rules, in the order written, which is the prose's order.
    pub rules: Vec<Rule>,
    /// Whether the runtime implements it instead (see the module docs).
    pub external: bool,
    /// The line it starts on.
    pub line: usize,
}

/// Everything one `.hgtok` file declares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct File {
    /// The states, in the order written.
    pub states: Vec<State>,
}

/// A syntax error, with the file and line that caused it.
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Word(String),
    Str(String),
    Char(char),
    Punct(char),
    Arrow,
}

struct Lexer {
    toks: Vec<(Tok, usize)>,
    at: usize,
    file: String,
}

fn unescape(body: &str, file: &str, line: usize) -> Result<char, Error> {
    let bad = || Error {
        file: file.to_owned(),
        line,
        message: format!("{body:?} is not one code point"),
    };
    let mut cs = body.chars();
    let first = cs.next().ok_or_else(bad)?;
    if first != '\\' {
        return if cs.next().is_none() {
            Ok(first)
        } else {
            Err(bad())
        };
    }
    let kind = cs.next().ok_or_else(bad)?;
    let c = match kind {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '0' => '\0',
        '\\' => '\\',
        '\'' => '\'',
        'u' => {
            let rest: String = cs.by_ref().collect();
            let hex = rest
                .strip_prefix('{')
                .and_then(|r| r.strip_suffix('}'))
                .ok_or_else(bad)?;
            let n = u32::from_str_radix(hex, 16).map_err(|_| bad())?;
            return char::from_u32(n).ok_or_else(bad);
        }
        _ => return Err(bad()),
    };
    if cs.next().is_some() {
        return Err(bad());
    }
    Ok(c)
}

impl Lexer {
    fn new(src: &str, file: &str) -> Result<Self, Error> {
        let mut toks = Vec::new();
        for (n, raw) in src.lines().enumerate() {
            let line = n.saturating_add(1);
            let mut chars = raw.char_indices().peekable();
            while let Some((i, c)) = chars.next() {
                if c.is_whitespace() {
                    continue;
                }
                // A `#` starts a comment only outside a string, because
                // every anchor carries a fragment.
                if c == '#' {
                    break;
                }
                if c == '"' || c == '\'' {
                    let rest = raw.get(i.saturating_add(1)..).unwrap_or("");
                    let mut end = None;
                    let mut escaped = false;
                    for (j, d) in rest.char_indices() {
                        if escaped {
                            escaped = false;
                        } else if d == '\\' {
                            escaped = true;
                        } else if d == c {
                            end = Some(j);
                            break;
                        }
                    }
                    let end = end.ok_or_else(|| Error {
                        file: file.to_owned(),
                        line,
                        message: "unterminated literal".into(),
                    })?;
                    let body = rest.get(..end).unwrap_or("");
                    let tok = if c == '"' {
                        Tok::Str(body.to_owned())
                    } else {
                        Tok::Char(unescape(body, file, line)?)
                    };
                    for _ in 0..=body.chars().count() {
                        chars.next();
                    }
                    toks.push((tok, line));
                    continue;
                }
                if c == '=' && matches!(chars.peek(), Some((_, '>'))) {
                    chars.next();
                    toks.push((Tok::Arrow, line));
                    continue;
                }
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    let mut v = String::from(c);
                    while let Some((_, d)) = chars.peek() {
                        if d.is_ascii_alphanumeric() || *d == '_' || *d == '-' {
                            v.push(*d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    toks.push((Tok::Word(v), line));
                    continue;
                }
                if "{}@,".contains(c) {
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

    fn bump(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.at).map(|(t, _)| t.clone());
        if t.is_some() {
            self.at = self.at.saturating_add(1);
        }
        t
    }

    fn eat_punct(&mut self, c: char) -> bool {
        if matches!(self.peek(), Some(Tok::Punct(p)) if *p == c) {
            self.at = self.at.saturating_add(1);
            true
        } else {
            false
        }
    }

    fn punct(&mut self, c: char) -> Result<(), Error> {
        if self.eat_punct(c) {
            Ok(())
        } else {
            Err(self.err(format!("expected {c:?}")))
        }
    }

    fn word(&mut self) -> Result<String, Error> {
        match self.bump() {
            Some(Tok::Word(w)) => Ok(w),
            other => Err(self.err(format!("expected a name, found {other:?}"))),
        }
    }

    fn anchor(&mut self) -> Result<String, Error> {
        self.punct('@')?;
        match self.bump() {
            Some(Tok::Str(s)) if !s.is_empty() => Ok(s),
            Some(Tok::Str(_)) => Err(self.err("an anchor may not be empty")),
            other => Err(self.err(format!("expected an anchor string, found {other:?}"))),
        }
    }
}

/// Parses one `.hgtok` file.
///
/// # Errors
/// [`Error`] naming the line when the text is not the grammar above.
pub fn parse(src: &str, file: &str) -> Result<File, Error> {
    let mut lx = Lexer::new(src, file)?;
    let mut out = File::default();
    while lx.peek().is_some() {
        let line = lx.line();
        let mut external = false;
        let mut kw = lx.word()?;
        if kw == "extern" {
            external = true;
            kw = lx.word()?;
        }
        if kw != "state" {
            return Err(lx.err(format!("expected `state` or `extern state`, found `{kw}`")));
        }
        let name = lx.word()?;
        let anchor = lx.anchor()?;
        let rules = if external {
            Vec::new()
        } else {
            rules(&mut lx)?
        };
        out.states.push(State {
            name,
            anchor,
            rules,
            external,
            line,
        });
    }
    Ok(out)
}

fn rules(lx: &mut Lexer) -> Result<Vec<Rule>, Error> {
    lx.punct('{')?;
    let mut out = Vec::new();
    while !lx.eat_punct('}') {
        let line = lx.line();
        let matches = match_of(lx)?;
        match lx.bump() {
            Some(Tok::Arrow) => {}
            other => return Err(lx.err(format!("expected `=>`, found {other:?}"))),
        }
        let actions = actions(lx)?;
        let anchor = if matches!(lx.peek(), Some(Tok::Punct('@'))) {
            Some(lx.anchor()?)
        } else {
            None
        };
        out.push(Rule {
            matches,
            actions,
            anchor,
            line,
        });
    }
    Ok(out)
}

fn match_of(lx: &mut Lexer) -> Result<Match, Error> {
    match lx.bump() {
        Some(Tok::Char(c)) => Ok(Match::Char(c)),
        Some(Tok::Word(w)) => match w.as_str() {
            "_" => Ok(Match::Anything),
            "eof" => Ok(Match::Eof),
            other => Class::from_word(other)
                .map(Match::Class)
                .ok_or_else(|| lx.err(format!("`{other}` is not a code-point class"))),
        },
        other => Err(lx.err(format!("expected a match, found {other:?}"))),
    }
}

fn actions(lx: &mut Lexer) -> Result<Vec<Action>, Error> {
    let mut out = vec![action(lx)?];
    while lx.eat_punct(',') {
        out.push(action(lx)?);
    }
    Ok(out)
}

fn action(lx: &mut Lexer) -> Result<Action, Error> {
    let verb = lx.word()?;
    Ok(match verb.as_str() {
        "switch" => Action::Switch(lx.word()?),
        "reconsume" => Action::Reconsume(lx.word()?),
        "set_return" => Action::SetReturn(lx.word()?),
        "switch_return" => Action::SwitchReturn,
        "reconsume_return" => Action::ReconsumeReturn,
        "emit_current" => Action::EmitCurrent,
        "emit_eof" => Action::EmitEof,
        "emit_token" => Action::EmitToken,
        "emit_temp" => Action::EmitTemp,
        "create_start_tag" => Action::CreateStartTag,
        "create_end_tag" => Action::CreateEndTag,
        "create_comment" => Action::CreateComment,
        "create_doctype" => Action::CreateDoctype,
        "start_attribute" => Action::StartAttribute,
        "clear_temp" => Action::ClearTemp,
        "flush_temp" => Action::FlushTemp,
        "emit_char" => match lx.bump() {
            Some(Tok::Char(c)) => Action::EmitChar(c),
            other => return Err(lx.err(format!("emit_char wants a character, found {other:?}"))),
        },
        "error" => Action::Error(lx.word()?),
        "set" => {
            let w = lx.word()?;
            Action::Set(Flag::from_word(&w).ok_or_else(|| lx.err(format!("`{w}` is not a flag")))?)
        }
        "append" | "append_lower" => {
            let w = lx.word()?;
            let buf =
                Buffer::from_word(&w).ok_or_else(|| lx.err(format!("`{w}` is not a buffer")))?;
            if verb == "append_lower" {
                Action::AppendLower(buf)
            } else if let Some(Tok::Char(c)) = lx.peek().cloned() {
                lx.bump();
                Action::AppendChar(buf, c)
            } else {
                Action::Append(buf)
            }
        }
        "if" => {
            let w = lx.word()?;
            let cond =
                Cond::from_word(&w).ok_or_else(|| lx.err(format!("`{w}` is not a condition")))?;
            lx.punct('{')?;
            let then = actions(lx)?;
            lx.punct('}')?;
            let w = lx.word()?;
            if w != "else" {
                return Err(lx.err("an `if` needs an `else`: the prose always writes both"));
            }
            lx.punct('{')?;
            let otherwise = actions(lx)?;
            lx.punct('}')?;
            Action::If {
                cond,
                then,
                otherwise,
            }
        }
        other => return Err(lx.err(format!("`{other}` is not an action"))),
    })
}
