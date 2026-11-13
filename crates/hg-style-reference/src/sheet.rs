//! Reading a stylesheet: declaration blocks, style rules, `@layer`
//! and `@namespace`.
//!
//! This is as much of CSS Syntax 3 §5 as the cascade needs, written
//! here rather than called into `hg-css-syntax` (G20). What it does
//! **not** do is as important as what it does: there is no nesting, no
//! `@media`, no `@supports` and no `@import`. A conditional group rule
//! is skipped whole, by balancing braces, and recorded as skipped —
//! an oracle that guessed at a rule it could not read would be worse
//! than one that said so.
//!
//! Values are kept as the text between the colon and the semicolon.
//! The cascade's job is to pick the winning declaration, not to
//! understand it; the spec-literal value interpreter over webref's
//! grammars is a separate piece, named in the ST-M0 review's agenda.

use crate::cascade::{Declaration, Origin, Rule};
use crate::selector;
use crate::token::{Token, Tokenizer};

/// A stylesheet, read.
#[derive(Debug, Clone, Default)]
pub struct Sheet {
    /// The style rules, in the order written.
    pub rules: Vec<Rule>,
    /// The cascade layers, in the order they were first declared.
    pub layers: Vec<String>,
    /// The default namespace, if `@namespace` declared one.
    pub default_namespace: Option<String>,
    /// The namespace prefixes.
    pub prefixes: Vec<(String, String)>,
    /// At-rules that were skipped whole, with the reason. A reader of
    /// a differential needs to know what the oracle did not read.
    pub skipped: Vec<String>,
    /// Rules whose selector would not parse, with the message.
    pub refused: Vec<String>,
}

/// Renders a token back to text, for a declaration's value.
fn render(token: &Token) -> String {
    match token {
        Token::Ident(s) => s.clone(),
        Token::Function(s) => format!("{s}("),
        Token::AtKeyword(s) => format!("@{s}"),
        Token::Hash(s, _) => format!("#{s}"),
        Token::Str(s) => format!("\"{s}\""),
        Token::BadString => "\"".to_owned(),
        Token::Url(s) => format!("url({s})"),
        Token::BadUrl => "url(".to_owned(),
        Token::Delim(c) => c.to_string(),
        Token::Number(v, _) => format!("{v}"),
        Token::Percentage(v) => format!("{v}%"),
        Token::Dimension(v, _, u) => format!("{v}{u}"),
        Token::Whitespace => " ".to_owned(),
        Token::Cdo => "<!--".to_owned(),
        Token::Cdc => "-->".to_owned(),
        Token::Colon => ":".to_owned(),
        Token::Semicolon => ";".to_owned(),
        Token::Comma => ",".to_owned(),
        Token::LeftSquare => "[".to_owned(),
        Token::RightSquare => "]".to_owned(),
        Token::LeftParen => "(".to_owned(),
        Token::RightParen => ")".to_owned(),
        Token::LeftCurly => "{".to_owned(),
        Token::RightCurly => "}".to_owned(),
    }
}

fn render_all(tokens: &[Token]) -> String {
    tokens.iter().map(render).collect::<String>()
}

/// Reads a declaration block: the text between `{` and `}`, or a
/// `style` attribute.
///
/// A declaration with no colon, or no property name, is dropped —
/// which is what Syntax 3 says to do, and what makes forward
/// compatibility work.
#[must_use]
pub fn declarations(input: &str) -> Vec<Declaration> {
    let tokens = Tokenizer::new(input).all();
    let mut out = Vec::new();
    for chunk in split_top_level(&tokens, &Token::Semicolon) {
        let mut it = chunk.iter().skip_while(|t| t.is_whitespace());
        let Some(Token::Ident(property)) = it.next() else {
            continue;
        };
        let rest: Vec<Token> = it.cloned().collect();
        let mut rest = rest.into_iter().skip_while(Token::is_whitespace);
        if rest.next() != Some(Token::Colon) {
            continue;
        }
        let value_tokens: Vec<Token> = rest.collect();
        let (value_tokens, important) = strip_important(&value_tokens);
        let value = render_all(&value_tokens).trim().to_owned();
        if value.is_empty() {
            continue;
        }
        out.push(Declaration {
            property: property.to_ascii_lowercase(),
            value,
            important,
        });
    }
    out
}

/// Finds a trailing `!important`, case-insensitively, and removes it.
fn strip_important(tokens: &[Token]) -> (Vec<Token>, bool) {
    let mut significant: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.is_whitespace())
        .map(|(i, _)| i)
        .collect();
    let Some(last) = significant.pop() else {
        return (tokens.to_vec(), false);
    };
    let Some(before) = significant.pop() else {
        return (tokens.to_vec(), false);
    };
    let is_important = matches!(tokens.get(last), Some(Token::Ident(w)) if w.eq_ignore_ascii_case("important"))
        && tokens.get(before) == Some(&Token::Delim('!'));
    if !is_important {
        return (tokens.to_vec(), false);
    }
    (tokens.get(..before).unwrap_or_default().to_vec(), true)
}

/// Splits a token list on `separator`, ignoring separators inside
/// brackets of any kind.
fn split_top_level(tokens: &[Token], separator: &Token) -> Vec<Vec<Token>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    let mut depth = 0_usize;
    for t in tokens {
        match t {
            Token::LeftParen | Token::LeftSquare | Token::LeftCurly | Token::Function(_) => {
                depth = depth.saturating_add(1);
            }
            Token::RightParen | Token::RightSquare | Token::RightCurly => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        if t == separator && depth == 0 {
            out.push(core::mem::take(&mut cur));
            continue;
        }
        cur.push(t.clone());
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Reads a stylesheet.
#[must_use]
pub fn parse(source: &str, origin: Origin) -> Sheet {
    let tokens = Tokenizer::new(source).all();
    let mut sheet = Sheet::default();
    let mut at = 0_usize;
    while at < tokens.len() {
        let Some(token) = tokens.get(at) else {
            break;
        };
        match token {
            Token::Whitespace | Token::Cdo | Token::Cdc => {
                at = at.saturating_add(1);
            }
            Token::AtKeyword(name) => {
                let name = name.to_ascii_lowercase();
                let (prelude, block, next) = at_rule(&tokens, at.saturating_add(1));
                at = next;
                match name.as_str() {
                    "namespace" => read_namespace(&mut sheet, &prelude),
                    "layer" => {
                        // The index is the one this *name* already has,
                        // because "cascade layers are sorted by the
                        // order in which they first are declared"
                        // (§6.4.3) — re-opening a layer must not move
                        // it to the end.
                        let layer = read_layer(&mut sheet, &prelude);
                        if let Some(block) = block {
                            read_rules(&mut sheet, &block, origin, layer);
                        }
                    }
                    other => sheet
                        .skipped
                        .push(format!("@{other} is not read by this reference engine")),
                }
            }
            _ => {
                let (prelude, block, next) = qualified_rule(&tokens, at);
                at = next;
                let Some(block) = block else {
                    sheet
                        .refused
                        .push("a qualified rule with no block".to_owned());
                    continue;
                };
                add_rule(&mut sheet, &prelude, &block, origin, None);
            }
        }
    }
    sheet
}

fn read_rules(sheet: &mut Sheet, tokens: &[Token], origin: Origin, layer: Option<usize>) {
    let mut at = 0_usize;
    while at < tokens.len() {
        match tokens.get(at) {
            Some(Token::Whitespace | Token::Cdo | Token::Cdc) => at = at.saturating_add(1),
            Some(Token::AtKeyword(name)) => {
                let (_, _, next) = at_rule(tokens, at.saturating_add(1));
                sheet
                    .skipped
                    .push(format!("@{name} inside a layer is not read"));
                at = next;
            }
            Some(_) => {
                let (prelude, block, next) = qualified_rule(tokens, at);
                at = next;
                if let Some(block) = block {
                    add_rule(sheet, &prelude, &block, origin, layer);
                }
            }
            None => break,
        }
    }
}

fn add_rule(
    sheet: &mut Sheet,
    prelude: &[Token],
    block: &[Token],
    origin: Origin,
    layer: Option<usize>,
) {
    let text = render_all(prelude);
    let selectors =
        match selector::parse_in(&text, sheet.default_namespace.as_deref(), &sheet.prefixes) {
            Ok(s) => s,
            Err(e) => {
                sheet
                    .refused
                    .push(format!("{}: {}", text.trim(), e.message));
                return;
            }
        };
    sheet.rules.push(Rule {
        selectors,
        declarations: declarations(&render_all(block)),
        origin,
        layer,
        order: sheet.rules.len(),
    });
}

fn read_namespace(sheet: &mut Sheet, prelude: &[Token]) {
    let significant: Vec<&Token> = prelude.iter().filter(|t| !t.is_whitespace()).collect();
    let (prefix, url) = match significant.as_slice() {
        [Token::Ident(p), Token::Str(u) | Token::Url(u)] => (Some(p.clone()), u.clone()),
        [Token::Str(u) | Token::Url(u)] => (None, u.clone()),
        _ => {
            sheet
                .refused
                .push("@namespace was not understood".to_owned());
            return;
        }
    };
    match prefix {
        Some(p) => sheet.prefixes.push((p, url)),
        None => sheet.default_namespace = Some(url),
    }
}

/// Reads `@layer`, declaring any name it has not seen, and gives back
/// the index a block after it belongs to.
///
/// A nested name such as `a.b` is taken whole rather than as a layer
/// inside a layer: §6.4.3's grouping of nested layers "within their
/// parent layers" is not modelled, and a sheet that uses it is
/// recorded as read only in part.
fn read_layer(sheet: &mut Sheet, prelude: &[Token]) -> Option<usize> {
    let text = render_all(prelude);
    let names: Vec<String> = text
        .split(',')
        .map(|n| n.trim().to_owned())
        .filter(|n| !n.is_empty())
        .collect();
    if names.iter().any(|n| n.contains('.')) {
        sheet
            .skipped
            .push("a nested @layer name is not grouped by this reference engine".to_owned());
    }
    let Some(first) = names.first().cloned() else {
        // An anonymous layer: `@layer { ... }`. It takes a slot of its
        // own in the order, and nothing can add to it later.
        sheet.layers.push(String::new());
        return sheet.layers.len().checked_sub(1);
    };
    for name in names {
        if !sheet.layers.contains(&name) {
            sheet.layers.push(name);
        }
    }
    sheet.layers.iter().position(|n| *n == first)
}

/// Reads a qualified rule: its prelude, its block, and where to carry
/// on.
fn qualified_rule(tokens: &[Token], from: usize) -> (Vec<Token>, Option<Vec<Token>>, usize) {
    let mut prelude = Vec::new();
    let mut at = from;
    while let Some(t) = tokens.get(at) {
        if *t == Token::LeftCurly {
            let (block, next) = balanced(tokens, at);
            return (prelude, Some(block), next);
        }
        prelude.push(t.clone());
        at = at.saturating_add(1);
    }
    (prelude, None, at)
}

/// Reads an at-rule after its keyword.
fn at_rule(tokens: &[Token], from: usize) -> (Vec<Token>, Option<Vec<Token>>, usize) {
    let mut prelude = Vec::new();
    let mut at = from;
    while let Some(t) = tokens.get(at) {
        match t {
            Token::Semicolon => return (prelude, None, at.saturating_add(1)),
            Token::LeftCurly => {
                let (block, next) = balanced(tokens, at);
                return (prelude, Some(block), next);
            }
            other => prelude.push(other.clone()),
        }
        at = at.saturating_add(1);
    }
    (prelude, None, at)
}

/// The contents of the `{}` starting at `from`, and the index after
/// its `}`.
fn balanced(tokens: &[Token], from: usize) -> (Vec<Token>, usize) {
    let mut out = Vec::new();
    let mut depth = 0_usize;
    let mut at = from;
    while let Some(t) = tokens.get(at) {
        at = at.saturating_add(1);
        match t {
            Token::LeftCurly => {
                depth = depth.saturating_add(1);
                if depth == 1 {
                    continue;
                }
            }
            Token::RightCurly => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return (out, at);
                }
            }
            _ => {}
        }
        out.push(t.clone());
    }
    (out, at)
}
