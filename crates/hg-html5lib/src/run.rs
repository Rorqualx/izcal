//! Running one case against one back end.

use std::collections::BTreeMap;

use hg_html_syntax::{Attribute, Back, Sink, State, Token, Tokenizer};

use crate::cases::{Case, Expected, ExpectedError};

/// What one run produced.
#[derive(Debug, Default)]
pub struct Got {
    /// The tokens, with runs of characters joined as the suite writes
    /// them.
    pub tokens: Vec<Expected>,
    /// The parse errors, in order.
    pub errors: Vec<ExpectedError>,
}

#[derive(Default)]
struct Keep {
    tokens: Vec<Token>,
    errors: Vec<ExpectedError>,
}

impl Sink for Keep {
    fn token(&mut self, token: Token) {
        self.tokens.push(token);
    }
    fn error(&mut self, code: &'static str, line: u32, column: u32) {
        self.errors.push(ExpectedError {
            code: code.to_owned(),
            line,
            col: column,
        });
    }
    fn adjusted_node_not_html(&mut self) -> bool {
        // The suite reaches the CDATA section state by naming it as an
        // initial state, never through the markup declaration open
        // state, so this answer is never the one that gets it there.
        false
    }
}

fn attrs_of(attrs: &[Attribute]) -> BTreeMap<String, String> {
    attrs
        .iter()
        .map(|a| (a.name.clone(), a.value.clone()))
        .collect()
}

/// Joins runs of character tokens, which is how the suite writes them,
/// and drops the end-of-file token, which it does not write at all.
fn fold(tokens: Vec<Token>) -> Vec<Expected> {
    let mut out: Vec<Expected> = Vec::new();
    for t in tokens {
        match t {
            Token::Character(c) => {
                if let Some(Expected::Character(s)) = out.last_mut() {
                    s.push(c);
                } else {
                    out.push(Expected::Character(c.to_string()));
                }
            }
            Token::Comment(c) => out.push(Expected::Comment(c)),
            Token::ProcessingInstruction { target, data } => {
                out.push(Expected::ProcessingInstruction { target, data });
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } => out.push(Expected::StartTag {
                name,
                attrs: attrs_of(&attrs),
                self_closing,
            }),
            Token::EndTag { name, .. } => out.push(Expected::EndTag(name)),
            Token::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } => out.push(Expected::Doctype {
                name,
                public_id,
                system_id,
                correct: !force_quirks,
            }),
            Token::Eof => {}
        }
    }
    out
}

/// Runs one case from one initial state, on one back end.
#[must_use]
pub fn run_case(case: &Case, state: State, back: Back) -> Got {
    let mut t = Tokenizer::for_conformance(
        Keep::default(),
        &case.input,
        back,
        state,
        &case.last_start_tag,
    );
    t.run();
    let keep = t.into_sink();
    Got {
        tokens: fold(keep.tokens),
        errors: keep.errors,
    }
}
