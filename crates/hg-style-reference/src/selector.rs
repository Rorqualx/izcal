//! Selectors 3: the grammar, specificity and matching.
//!
//! Written from the specification's own prose, independently of
//! `hg-selectors` (G20). Level 3 is the whole of ST-M0's claim —
//! "`hg-style-reference` matches Selectors 3" — so what is here is
//! Level 3 and nothing beyond it. `:has()`, `:is()`, `:where()` and
//! the level 4 attribute modifiers are named in the ST-M0 review's
//! agenda and are **not** half-implemented here: an oracle that
//! answered a selector it did not understand would be worse than one
//! that refused it.
//!
//! The matcher walks right to left, which is what every engine does
//! and what makes the combinators cheap: the rightmost compound is
//! tested against the subject, and each combinator to its left moves
//! to a set of candidate ancestors or siblings.

use hg_tree_traits::{
    AttrNs, ElementState, Tree, ancestors, element_children, is_element, is_empty_element,
    is_html_element, is_root, ns, preceding_siblings,
};

use crate::token::{HashKind, Token, Tokenizer};

/// A selector's specificity: (a, b, c).
///
/// Selectors 3 §9 counts ids in `a`, classes, attributes and
/// pseudo-classes in `b`, and element names and pseudo-elements in
/// `c`. The universal selector counts nowhere, and the three numbers
/// are compared as a tuple, "with a as the most significant" — never
/// added or carried, which is the classic bug this type exists to
/// prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Specificity {
    /// Ids.
    pub a: u32,
    /// Classes, attributes and pseudo-classes.
    pub b: u32,
    /// Element names and pseudo-elements.
    pub c: u32,
}

impl Specificity {
    /// Adds another selector's counts, used for a compound.
    #[must_use]
    pub const fn plus(self, other: Self) -> Self {
        Self {
            a: self.a.saturating_add(other.a),
            b: self.b.saturating_add(other.b),
            c: self.c.saturating_add(other.c),
        }
    }

    /// One id.
    pub const ID: Self = Self { a: 1, b: 0, c: 0 };
    /// One class, attribute or pseudo-class.
    pub const CLASS: Self = Self { a: 0, b: 1, c: 0 };
    /// One element name or pseudo-element.
    pub const TYPE: Self = Self { a: 0, b: 0, c: 1 };
}

/// How an attribute selector compares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrMatch {
    /// `[att]` — present.
    Present,
    /// `[att=val]`.
    Exact(String),
    /// `[att~=val]` — one of a whitespace-separated list.
    Includes(String),
    /// `[att|=val]` — the value, or the value then a hyphen.
    DashMatch(String),
    /// `[att^=val]`.
    Prefix(String),
    /// `[att$=val]`.
    Suffix(String),
    /// `[att*=val]`.
    Substring(String),
}

/// An `an+b` argument, as `:nth-child()` and its three siblings take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnPlusB {
    /// The step.
    pub a: i64,
    /// The offset.
    pub b: i64,
}

impl AnPlusB {
    /// Whether a 1-based index matches.
    ///
    /// Selectors 3: "the index must be a non-negative integer n", so
    /// `-n+3` matches 1, 2 and 3 and `2n+1` matches the odd indices.
    #[must_use]
    pub fn matches(self, index: i64) -> bool {
        let Some(offset) = index.checked_sub(self.b) else {
            return false;
        };
        if self.a == 0 {
            return offset == 0;
        }
        let Some(q) = offset.checked_div(self.a) else {
            return false;
        };
        offset.checked_rem(self.a) == Some(0) && q >= 0
    }
}

/// One simple selector.
#[derive(Debug, Clone, PartialEq)]
pub enum Simple {
    /// `*` or `ns|*`.
    Universal(NameMatch),
    /// An element name, with its namespace rule.
    Type(NameMatch, String),
    /// `#id`.
    Id(String),
    /// `.class`.
    Class(String),
    /// `[att]` and its six comparisons.
    Attribute(NameMatch, String, AttrMatch),
    /// A pseudo-class that is one state bit.
    State(ElementState),
    /// A structural or logical pseudo-class.
    Structural(Structural),
    /// `:not(simple)` — Level 3 takes exactly one simple selector.
    Not(Box<Simple>),
    /// A pseudo-element, which is matched by the cascade and not by
    /// the matcher.
    PseudoElement(String),
}

/// Which namespace a name selector asks about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameMatch {
    /// `ns|name` — one namespace. Resolved from the sheet's `@namespace`
    /// rules at parse time.
    One(String),
    /// `|name` — no namespace.
    None,
    /// `*|name`, and also a bare `name` when no default namespace is
    /// declared.
    Any,
}

/// The structural pseudo-classes of Selectors 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Structural {
    /// `:root`.
    Root,
    /// `:empty`.
    Empty,
    /// `:nth-child()`.
    NthChild(AnPlusB),
    /// `:nth-last-child()`.
    NthLastChild(AnPlusB),
    /// `:nth-of-type()`.
    NthOfType(AnPlusB),
    /// `:nth-last-of-type()`.
    NthLastOfType(AnPlusB),
    /// `:first-child`.
    FirstChild,
    /// `:last-child`.
    LastChild,
    /// `:only-child`.
    OnlyChild,
    /// `:first-of-type`.
    FirstOfType,
    /// `:last-of-type`.
    LastOfType,
    /// `:only-of-type`.
    OnlyOfType,
    /// `:lang()`.
    Lang(String),
}

/// A sequence of simple selectors with no combinator between them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Compound {
    /// The simple selectors, in the order written.
    pub simples: Vec<Simple>,
}

/// How one compound reaches the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// A space: an ancestor.
    Descendant,
    /// `>`: the parent.
    Child,
    /// `+`: the immediately preceding sibling.
    NextSibling,
    /// `~`: any preceding sibling.
    LaterSibling,
}

/// A complex selector: compounds joined by combinators, written left
/// to right, with the subject last.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selector {
    /// The compounds, subject last.
    pub compounds: Vec<Compound>,
    /// One combinator fewer than there are compounds; `combinators[i]`
    /// joins `compounds[i]` to `compounds[i + 1]`.
    pub combinators: Vec<Combinator>,
}

/// A comma-separated list of selectors.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectorList {
    /// The selectors, in the order written.
    pub selectors: Vec<Selector>,
}

/// What went wrong parsing a selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// What was wrong.
    pub message: String,
}

fn err<T>(message: impl Into<String>) -> Result<T, Error> {
    Err(Error {
        message: message.into(),
    })
}

// ---------------------------------------------------------------- parse

/// A cursor over the significant tokens of a selector.
struct Parser {
    tokens: Vec<Token>,
    at: usize,
    /// The sheet's default namespace, if `@namespace` declared one.
    default_namespace: Option<String>,
    /// The sheet's prefixes.
    prefixes: Vec<(String, String)>,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn peek_at(&self, ahead: usize) -> Option<&Token> {
        self.at.checked_add(ahead).and_then(|i| self.tokens.get(i))
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.at).cloned();
        if t.is_some() {
            self.at = self.at.saturating_add(1);
        }
        t
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(Token::is_whitespace) {
            self.at = self.at.saturating_add(1);
        }
    }

    fn resolve(&self, prefix: &str) -> Result<NameMatch, Error> {
        if prefix == "*" {
            return Ok(NameMatch::Any);
        }
        if prefix.is_empty() {
            return Ok(NameMatch::None);
        }
        match self.prefixes.iter().find(|(p, _)| p == prefix) {
            Some((_, url)) => Ok(NameMatch::One(url.clone())),
            None => err(format!("namespace prefix `{prefix}` was never declared")),
        }
    }

    /// A name, which may carry a namespace prefix: `ns|name`, `|name`,
    /// `*|name`, `name`.
    ///
    /// The bare form takes the default namespace for an element name,
    /// and *no* namespace for an attribute — Selectors 3 §6.3.2 says
    /// so plainly, and it is the rule people get wrong.
    fn qualified_name(
        &mut self,
        for_attribute: bool,
    ) -> Result<(NameMatch, Option<String>), Error> {
        let mut prefix: Option<String> = None;
        // `ns|`, `|` or `*|`.
        let head = match self.peek() {
            Some(Token::Ident(s)) => Some(s.clone()),
            Some(Token::Delim('*')) => Some("*".to_owned()),
            Some(Token::Delim('|')) => Some(String::new()),
            _ => None,
        };
        if let Some(h) = head {
            let bar_ahead = if h.is_empty() { 0 } else { 1 };
            if matches!(self.peek_at(bar_ahead), Some(Token::Delim('|')))
                // `a|=` is an attribute comparison, not a prefix.
                && !matches!(self.peek_at(bar_ahead.saturating_add(1)), Some(Token::Delim('=')))
            {
                prefix = Some(h);
                self.at = self.at.saturating_add(bar_ahead.saturating_add(1));
            }
        }
        let name_match = match &prefix {
            Some(p) => self.resolve(p)?,
            None if for_attribute => NameMatch::None,
            None => match &self.default_namespace {
                Some(url) => NameMatch::One(url.clone()),
                None => NameMatch::Any,
            },
        };
        match self.peek().cloned() {
            Some(Token::Ident(name)) => {
                self.at = self.at.saturating_add(1);
                Ok((name_match, Some(name)))
            }
            Some(Token::Delim('*')) if !for_attribute => {
                self.at = self.at.saturating_add(1);
                Ok((name_match, None))
            }
            _ if prefix.is_some() => err("a namespace prefix needs a name after it"),
            _ => err("a name was expected"),
        }
    }

    fn attribute(&mut self) -> Result<Simple, Error> {
        self.skip_whitespace();
        let (name_match, Some(local)) = self.qualified_name(true)? else {
            return err("an attribute selector needs a name");
        };
        self.skip_whitespace();
        let compare = match self.peek().cloned() {
            Some(Token::RightSquare) => {
                self.at = self.at.saturating_add(1);
                return Ok(Simple::Attribute(name_match, local, AttrMatch::Present));
            }
            Some(Token::Delim(d @ ('~' | '|' | '^' | '$' | '*'))) => {
                self.at = self.at.saturating_add(1);
                if self.peek() != Some(&Token::Delim('=')) {
                    return err(format!("`{d}` must be followed by `=`"));
                }
                self.at = self.at.saturating_add(1);
                d
            }
            Some(Token::Delim('=')) => {
                self.at = self.at.saturating_add(1);
                '='
            }
            other => {
                return err(format!(
                    "an attribute comparison was expected, got {other:?}"
                ));
            }
        };
        self.skip_whitespace();
        let value = match self.bump() {
            Some(Token::Ident(s) | Token::Str(s)) => s,
            other => return err(format!("an attribute value was expected, got {other:?}")),
        };
        self.skip_whitespace();
        if self.bump() != Some(Token::RightSquare) {
            return err("an attribute selector needs a `]`");
        }
        let m = match compare {
            '=' => AttrMatch::Exact(value),
            '~' => AttrMatch::Includes(value),
            '|' => AttrMatch::DashMatch(value),
            '^' => AttrMatch::Prefix(value),
            '$' => AttrMatch::Suffix(value),
            _ => AttrMatch::Substring(value),
        };
        Ok(Simple::Attribute(name_match, local, m))
    }

    /// `an+b`, as the four `:nth-` pseudo-classes take it.
    fn an_plus_b(&mut self) -> Result<AnPlusB, Error> {
        self.skip_whitespace();
        // `odd` and `even` are the two keywords.
        if let Some(Token::Ident(word)) = self.peek().cloned() {
            if word.eq_ignore_ascii_case("odd") {
                self.at = self.at.saturating_add(1);
                self.close_function()?;
                return Ok(AnPlusB { a: 2, b: 1 });
            }
            if word.eq_ignore_ascii_case("even") {
                self.at = self.at.saturating_add(1);
                self.close_function()?;
                return Ok(AnPlusB { a: 2, b: 0 });
            }
        }
        // Everything else is read from the raw text between here and
        // the `)`, because `2n+1`, `2n + 1`, `+2n-1` and `-n+6` all
        // tokenize differently and the grammar is about the text.
        let mut text = String::new();
        let mut depth = 0_usize;
        while let Some(t) = self.peek().cloned() {
            if t == Token::RightParen && depth == 0 {
                break;
            }
            if t == Token::LeftParen {
                depth = depth.saturating_add(1);
            }
            if t == Token::RightParen {
                depth = depth.saturating_sub(1);
            }
            self.at = self.at.saturating_add(1);
            match t {
                Token::Ident(s) => text.push_str(&s),
                Token::Number(v, _) => {
                    push_sign(&mut text, v);
                    text.push_str(&format!("{v}"));
                }
                Token::Dimension(v, _, unit) => {
                    push_sign(&mut text, v);
                    text.push_str(&format!("{v}{unit}"));
                }
                Token::Delim(c) => text.push(c),
                Token::Whitespace => {}
                other => return err(format!("`an+b` cannot hold {other:?}")),
            }
        }
        self.close_function()?;
        parse_an_plus_b(&text)
    }

    fn close_function(&mut self) -> Result<(), Error> {
        self.skip_whitespace();
        if self.bump() == Some(Token::RightParen) {
            Ok(())
        } else {
            err("a `)` was expected")
        }
    }

    fn function_argument(&mut self) -> Result<String, Error> {
        self.skip_whitespace();
        let arg = match self.bump() {
            Some(Token::Ident(s) | Token::Str(s)) => s,
            other => return err(format!("an argument was expected, got {other:?}")),
        };
        self.close_function()?;
        Ok(arg)
    }

    fn pseudo(&mut self) -> Result<Simple, Error> {
        // One `:` was consumed; a second means a pseudo-element.
        let double = self.peek() == Some(&Token::Colon);
        if double {
            self.at = self.at.saturating_add(1);
        }
        match self.bump() {
            Some(Token::Ident(name)) => {
                let lower = name.to_ascii_lowercase();
                if double || is_legacy_pseudo_element(&lower) {
                    return Ok(Simple::PseudoElement(lower));
                }
                simple_pseudo_class(&lower)
            }
            Some(Token::Function(name)) => {
                let lower = name.to_ascii_lowercase();
                if double {
                    return err(format!("`::{lower}()` is not a Selectors 3 pseudo-element"));
                }
                match lower.as_str() {
                    "nth-child" => Ok(Simple::Structural(Structural::NthChild(self.an_plus_b()?))),
                    "nth-last-child" => Ok(Simple::Structural(Structural::NthLastChild(
                        self.an_plus_b()?,
                    ))),
                    "nth-of-type" => {
                        Ok(Simple::Structural(Structural::NthOfType(self.an_plus_b()?)))
                    }
                    "nth-last-of-type" => Ok(Simple::Structural(Structural::NthLastOfType(
                        self.an_plus_b()?,
                    ))),
                    "lang" => Ok(Simple::Structural(Structural::Lang(
                        self.function_argument()?.to_ascii_lowercase(),
                    ))),
                    "not" => {
                        self.skip_whitespace();
                        let inner = self.simple()?;
                        if matches!(inner, Simple::Not(_) | Simple::PseudoElement(_)) {
                            return err("Selectors 3 forbids `:not()` holding `:not()` or a \
                                 pseudo-element");
                        }
                        self.close_function()?;
                        Ok(Simple::Not(Box::new(inner)))
                    }
                    other => err(format!(
                        "`:{other}()` is not in Selectors 3; this reference engine refuses a \
                         selector it does not understand rather than guessing"
                    )),
                }
            }
            other => err(format!("a pseudo-class name was expected, got {other:?}")),
        }
    }

    /// One simple selector.
    fn simple(&mut self) -> Result<Simple, Error> {
        match self.peek().cloned() {
            Some(Token::Hash(name, HashKind::Id)) => {
                self.at = self.at.saturating_add(1);
                Ok(Simple::Id(name))
            }
            Some(Token::Delim('.')) => {
                self.at = self.at.saturating_add(1);
                match self.bump() {
                    Some(Token::Ident(name)) => Ok(Simple::Class(name)),
                    other => err(format!("a class name was expected, got {other:?}")),
                }
            }
            Some(Token::LeftSquare) => {
                self.at = self.at.saturating_add(1);
                self.attribute()
            }
            Some(Token::Colon) => {
                self.at = self.at.saturating_add(1);
                self.pseudo()
            }
            Some(Token::Ident(_) | Token::Delim('*' | '|')) => {
                let (name_match, name) = self.qualified_name(false)?;
                Ok(match name {
                    Some(n) => Simple::Type(name_match, n),
                    None => Simple::Universal(name_match),
                })
            }
            other => err(format!("a simple selector was expected, got {other:?}")),
        }
    }

    /// One compound: a sequence of simple selectors.
    fn compound(&mut self) -> Result<Compound, Error> {
        let mut simples = Vec::new();
        // A type or universal selector, if there is one, comes first.
        loop {
            match self.peek() {
                None
                | Some(
                    Token::Comma
                    | Token::Whitespace
                    | Token::RightParen
                    | Token::Delim('>' | '+' | '~'),
                ) => break,
                _ => {}
            }
            let simple = self.simple()?;
            if !simples.is_empty() && matches!(simple, Simple::Type(_, _) | Simple::Universal(_)) {
                return err("a type selector must be first in its compound");
            }
            if simples
                .iter()
                .any(|s| matches!(s, Simple::PseudoElement(_)))
            {
                return err("a pseudo-element must be last in its compound");
            }
            simples.push(simple);
        }
        if simples.is_empty() {
            return err("an empty compound selector");
        }
        Ok(Compound { simples })
    }

    /// One complex selector.
    fn selector(&mut self) -> Result<Selector, Error> {
        let mut out = Selector::default();
        self.skip_whitespace();
        out.compounds.push(self.compound()?);
        loop {
            let had_space = self.peek().is_some_and(Token::is_whitespace);
            self.skip_whitespace();
            let combinator = match self.peek() {
                Some(Token::Delim('>')) => Combinator::Child,
                Some(Token::Delim('+')) => Combinator::NextSibling,
                Some(Token::Delim('~')) => Combinator::LaterSibling,
                None | Some(Token::Comma | Token::RightParen) => break,
                Some(_) if had_space => Combinator::Descendant,
                Some(t) => return err(format!("a combinator was expected, got {t:?}")),
            };
            if combinator != Combinator::Descendant {
                self.at = self.at.saturating_add(1);
                self.skip_whitespace();
            }
            out.combinators.push(combinator);
            out.compounds.push(self.compound()?);
        }
        Ok(out)
    }
}

/// A `+` before a non-negative number that follows something, so
/// `2n 1` cannot read as `2n1` — but not after a sign the author
/// already wrote, which is how `2n + 1` tokenizes.
fn push_sign(text: &mut String, value: f64) {
    if value < 0.0 || text.is_empty() {
        return;
    }
    if text.ends_with('+') || text.ends_with('-') {
        return;
    }
    text.push('+');
}

/// `odd`, `even` and the `an+b` forms, read from the text between the
/// parentheses.
fn parse_an_plus_b(text: &str) -> Result<AnPlusB, Error> {
    let t = text.replace(' ', "").to_ascii_lowercase();
    if t.is_empty() {
        return err("`an+b` cannot be empty");
    }
    let Some((before, after)) = t.split_once('n') else {
        // Just `b`.
        return match t.parse::<i64>() {
            Ok(b) => Ok(AnPlusB { a: 0, b }),
            Err(_) => err(format!("`{text}` is not an `an+b`")),
        };
    };
    let a = match before {
        "" | "+" => 1,
        "-" => -1,
        other => match other.parse::<i64>() {
            Ok(v) => v,
            Err(_) => return err(format!("`{text}` is not an `an+b`")),
        },
    };
    let b = if after.is_empty() {
        0
    } else {
        match after.parse::<i64>() {
            Ok(v) => v,
            Err(_) => return err(format!("`{text}` is not an `an+b`")),
        }
    };
    Ok(AnPlusB { a, b })
}

/// The pseudo-elements Selectors 3 also accepts with one colon, "for
/// compatibility with existing style sheets".
fn is_legacy_pseudo_element(name: &str) -> bool {
    matches!(name, "first-line" | "first-letter" | "before" | "after")
}

/// A pseudo-class with no argument.
fn simple_pseudo_class(name: &str) -> Result<Simple, Error> {
    let state = |s: ElementState| Ok(Simple::State(s));
    let structural = |s: Structural| Ok(Simple::Structural(s));
    match name {
        "link" => state(ElementState::LINK),
        "visited" => state(ElementState::VISITED),
        "hover" => state(ElementState::HOVER),
        "active" => state(ElementState::ACTIVE),
        "focus" => state(ElementState::FOCUS),
        "target" => state(ElementState::TARGET),
        "enabled" => state(ElementState::ENABLED),
        "disabled" => state(ElementState::DISABLED),
        "checked" => state(ElementState::CHECKED),
        "indeterminate" => state(ElementState::INDETERMINATE),
        "root" => structural(Structural::Root),
        "empty" => structural(Structural::Empty),
        "first-child" => structural(Structural::FirstChild),
        "last-child" => structural(Structural::LastChild),
        "only-child" => structural(Structural::OnlyChild),
        "first-of-type" => structural(Structural::FirstOfType),
        "last-of-type" => structural(Structural::LastOfType),
        "only-of-type" => structural(Structural::OnlyOfType),
        other => err(format!(
            "`:{other}` is not in Selectors 3; this reference engine refuses a selector it does \
             not understand rather than guessing"
        )),
    }
}

/// Parses a selector list.
///
/// # Errors
/// [`Error`] for anything that is not a Selectors 3 selector —
/// including a selector from a later level, which is refused by name
/// rather than ignored.
pub fn parse(input: &str) -> Result<SelectorList, Error> {
    parse_in(input, None, &[])
}

/// Parses a selector list with a sheet's namespace declarations.
///
/// # Errors
/// [`Error`], as [`parse`].
pub fn parse_in(
    input: &str,
    default_namespace: Option<&str>,
    prefixes: &[(String, String)],
) -> Result<SelectorList, Error> {
    let mut p = Parser {
        tokens: Tokenizer::new(input).all(),
        at: 0,
        default_namespace: default_namespace.map(ToOwned::to_owned),
        prefixes: prefixes.to_vec(),
    };
    let mut out = SelectorList::default();
    loop {
        out.selectors.push(p.selector()?);
        p.skip_whitespace();
        match p.bump() {
            None => return Ok(out),
            Some(Token::Comma) => {}
            Some(t) => return err(format!("a comma or the end was expected, got {t:?}")),
        }
    }
}

// ----------------------------------------------------------- specificity

impl Simple {
    /// This simple selector's contribution to specificity (§9).
    #[must_use]
    pub fn specificity(&self) -> Specificity {
        match self {
            Self::Universal(_) => Specificity::default(),
            Self::Type(_, _) | Self::PseudoElement(_) => Specificity::TYPE,
            Self::Id(_) => Specificity::ID,
            Self::Class(_) | Self::Attribute(_, _, _) | Self::State(_) | Self::Structural(_) => {
                Specificity::CLASS
            }
            // "Negations do not count as pseudo-classes, but the
            // selector inside counts."
            Self::Not(inner) => inner.specificity(),
        }
    }
}

impl Selector {
    /// The selector's specificity.
    #[must_use]
    pub fn specificity(&self) -> Specificity {
        self.compounds
            .iter()
            .flat_map(|c| c.simples.iter())
            .fold(Specificity::default(), |acc, s| acc.plus(s.specificity()))
    }

    /// The pseudo-element this selector names, if any.
    #[must_use]
    pub fn pseudo_element(&self) -> Option<&str> {
        self.compounds.last()?.simples.iter().find_map(|s| match s {
            Simple::PseudoElement(name) => Some(name.as_str()),
            _ => None,
        })
    }
}

// ------------------------------------------------------------- matching

/// Whether `value` matches `compare`.
fn attribute_matches(value: &str, compare: &AttrMatch) -> bool {
    match compare {
        AttrMatch::Present => true,
        AttrMatch::Exact(v) => value == v,
        AttrMatch::Includes(v) => !v.is_empty() && value.split_ascii_whitespace().any(|p| p == v),
        AttrMatch::DashMatch(v) => value == v || value.starts_with(&format!("{v}-")),
        // Selectors 3: "If the value is the empty string then the
        // selector does not represent anything."
        AttrMatch::Prefix(v) => !v.is_empty() && value.starts_with(v.as_str()),
        AttrMatch::Suffix(v) => !v.is_empty() && value.ends_with(v.as_str()),
        AttrMatch::Substring(v) => !v.is_empty() && value.contains(v.as_str()),
    }
}

/// The 1-based index of `node` among its element siblings, and how many
/// there are.
fn child_index<T: Tree + ?Sized>(tree: &T, node: T::Node) -> (i64, i64) {
    let Some(parent) = tree.parent(node) else {
        return (1, 1);
    };
    let kids = element_children(tree, parent);
    let total = i64::try_from(kids.len()).unwrap_or(i64::MAX);
    let index = kids
        .iter()
        .position(|k| *k == node)
        .and_then(|i| i64::try_from(i).ok())
        .map_or(1, |i| i.saturating_add(1));
    (index, total)
}

/// The same, counting only siblings of the same type.
fn type_index<T: Tree + ?Sized>(tree: &T, node: T::Node) -> (i64, i64) {
    let Some(parent) = tree.parent(node) else {
        return (1, 1);
    };
    let (want_ns, want_local) = (tree.namespace(node), tree.local_name(node));
    let same: Vec<T::Node> = element_children(tree, parent)
        .into_iter()
        .filter(|k| tree.namespace(*k) == want_ns && tree.local_name(*k) == want_local)
        .collect();
    let total = i64::try_from(same.len()).unwrap_or(i64::MAX);
    let index = same
        .iter()
        .position(|k| *k == node)
        .and_then(|i| i64::try_from(i).ok())
        .map_or(1, |i| i.saturating_add(1));
    (index, total)
}

/// Whether the element's language matches `want` (§6.6.3): the
/// nearest `lang` attribute, compared as a prefix on hyphen
/// boundaries.
fn lang_matches<T: Tree + ?Sized>(tree: &T, node: T::Node, want: &str) -> bool {
    let mut at = Some(node);
    while let Some(n) = at {
        let value = tree
            .attribute(n, AttrNs::None, "lang")
            .or_else(|| tree.attribute(n, AttrNs::One(ns::XML), "lang"));
        if let Some(v) = value {
            let v = v.to_ascii_lowercase();
            return v == want || v.starts_with(&format!("{want}-"));
        }
        at = tree.parent(n);
    }
    false
}

fn name_matches<T: Tree + ?Sized>(tree: &T, node: T::Node, name_match: &NameMatch) -> bool {
    match name_match {
        NameMatch::Any => true,
        NameMatch::None => tree.namespace(node).is_none_or(str::is_empty),
        NameMatch::One(url) => tree.namespace(node) == Some(url.as_str()),
    }
}

/// Whether one simple selector matches `node`.
#[must_use]
pub fn matches_simple<T: Tree + ?Sized>(tree: &T, node: T::Node, simple: &Simple) -> bool {
    if !is_element(tree, node) {
        return false;
    }
    match simple {
        Simple::Universal(nm) => name_matches(tree, node, nm),
        Simple::Type(nm, name) => {
            name_matches(tree, node, nm)
                && tree.local_name(node).is_some_and(|local| {
                    if is_html_element(tree, node) {
                        local.eq_ignore_ascii_case(name)
                    } else {
                        local == name
                    }
                })
        }
        // An id and a class are compared case-sensitively in a standards
        // document. Quirks mode is not Selectors 3's business, and the
        // tree says which it is through `has_class`.
        Simple::Id(id) => tree.element_id(node) == Some(id.as_str()),
        Simple::Class(class) => tree.has_class(node, class, true),
        Simple::Attribute(nm, local, compare) => {
            let ans = match nm {
                NameMatch::None => AttrNs::None,
                NameMatch::Any => AttrNs::Any,
                NameMatch::One(url) => AttrNs::One(url.as_str()),
            };
            // An HTML element's attribute names are matched
            // case-insensitively; its *values* are not, except where
            // HTML says so, which Selectors 3 leaves to HTML.
            tree.attribute(node, ans, local)
                .or_else(|| {
                    is_html_element(tree, node)
                        .then(|| tree.attribute(node, ans, &local.to_ascii_lowercase()))
                        .flatten()
                })
                .is_some_and(|v| attribute_matches(v, compare))
        }
        Simple::State(bits) => tree.element_state(node).contains(*bits),
        Simple::Structural(s) => matches_structural(tree, node, s),
        Simple::Not(inner) => !matches_simple(tree, node, inner),
        // A pseudo-element does not match an element; the cascade asks
        // for it by name.
        Simple::PseudoElement(_) => false,
    }
}

fn matches_structural<T: Tree + ?Sized>(tree: &T, node: T::Node, s: &Structural) -> bool {
    match s {
        Structural::Root => is_root(tree, node),
        Structural::Empty => is_empty_element(tree, node),
        Structural::Lang(want) => lang_matches(tree, node, want),
        Structural::NthChild(ab) => ab.matches(child_index(tree, node).0),
        Structural::NthLastChild(ab) => {
            let (i, total) = child_index(tree, node);
            ab.matches(total.saturating_sub(i).saturating_add(1))
        }
        Structural::NthOfType(ab) => ab.matches(type_index(tree, node).0),
        Structural::NthLastOfType(ab) => {
            let (i, total) = type_index(tree, node);
            ab.matches(total.saturating_sub(i).saturating_add(1))
        }
        Structural::FirstChild => child_index(tree, node).0 == 1,
        Structural::LastChild => {
            let (i, total) = child_index(tree, node);
            i == total
        }
        Structural::OnlyChild => child_index(tree, node).1 == 1,
        Structural::FirstOfType => type_index(tree, node).0 == 1,
        Structural::LastOfType => {
            let (i, total) = type_index(tree, node);
            i == total
        }
        Structural::OnlyOfType => type_index(tree, node).1 == 1,
    }
}

/// Whether every simple selector of a compound matches.
#[must_use]
pub fn matches_compound<T: Tree + ?Sized>(tree: &T, node: T::Node, compound: &Compound) -> bool {
    compound
        .simples
        .iter()
        // A pseudo-element in the subject compound does not stop the
        // element from matching: the cascade decides which box the
        // declarations reach.
        .filter(|s| !matches!(s, Simple::PseudoElement(_)))
        .all(|s| matches_simple(tree, node, s))
}

/// Whether a complex selector matches `node` as its subject.
///
/// Right to left: the last compound is tested against `node`, then
/// each combinator moves to the candidates it allows.
#[must_use]
pub fn matches<T: Tree + ?Sized>(tree: &T, node: T::Node, selector: &Selector) -> bool {
    let Some(last) = selector.compounds.len().checked_sub(1) else {
        return false;
    };
    let Some(subject) = selector.compounds.get(last) else {
        return false;
    };
    if !matches_compound(tree, node, subject) {
        return false;
    }
    matches_from(tree, node, selector, last)
}

/// Whether the part of `selector` left of compound `index` matches,
/// with `node` matching compound `index`.
fn matches_from<T: Tree + ?Sized>(
    tree: &T,
    node: T::Node,
    selector: &Selector,
    index: usize,
) -> bool {
    let Some(prev) = index.checked_sub(1) else {
        return true;
    };
    let Some(combinator) = selector.combinators.get(prev) else {
        return true;
    };
    let Some(compound) = selector.compounds.get(prev) else {
        return true;
    };
    let candidates: Vec<T::Node> = match combinator {
        Combinator::Child => tree.parent(node).into_iter().collect(),
        Combinator::Descendant => ancestors(tree, node),
        Combinator::NextSibling => tree
            .previous_sibling(node)
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            // `+` is the immediately preceding *element* sibling, so a
            // text node between them is skipped.
            .flat_map(|_| {
                preceding_siblings(tree, node)
                    .into_iter()
                    .find(|n| is_element(tree, *n))
            })
            .collect(),
        Combinator::LaterSibling => preceding_siblings(tree, node)
            .into_iter()
            .filter(|n| is_element(tree, *n))
            .collect(),
    };
    candidates
        .into_iter()
        .any(|c| matches_compound(tree, c, compound) && matches_from(tree, c, selector, prev))
}

/// Whether any selector of the list matches, and the specificity of
/// the most specific one that does.
#[must_use]
pub fn matches_list<T: Tree + ?Sized>(
    tree: &T,
    node: T::Node,
    list: &SelectorList,
) -> Option<Specificity> {
    list.selectors
        .iter()
        .filter(|s| matches(tree, node, s))
        .map(Selector::specificity)
        .max()
}
