//! `hg-html-syntax`'s tests: the tokenizer runs, the two generated back
//! ends agree, and the parts the description declares `extern` do what
//! the prose says.

mod agree;
mod tokenize;

use hg_html_syntax::{Back, EntryState, Sink, Token, Tokenizer};

/// A sink that keeps everything, for tests to look at.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Collect {
    /// Every token, in order.
    pub tokens: Vec<Token>,
    /// Every parse error's name, in order.
    pub errors: Vec<&'static str>,
    /// What [`Sink::adjusted_node_not_html`] answers — the one question
    /// the tokenizer asks a tree builder.
    pub foreign: bool,
}

impl Sink for Collect {
    fn token(&mut self, token: Token) {
        self.tokens.push(token);
    }
    fn error(&mut self, code: &'static str, _line: u32, _column: u32) {
        self.errors.push(code);
    }
    fn adjusted_node_not_html(&mut self) -> bool {
        self.foreign
    }
}

/// Tokenizes `input` with one back end, from the data state throughout.
#[must_use]
pub fn run(input: &str, back: Back) -> Collect {
    let mut t = Tokenizer::new(Collect::default(), input, back);
    t.run();
    t.into_sink()
}

/// The same, with the adjusted current node outside the HTML namespace.
#[must_use]
pub fn run_foreign(input: &str, back: Back) -> Collect {
    let sink = Collect {
        foreign: true,
        ..Collect::default()
    };
    let mut t = Tokenizer::new(sink, input, back);
    t.run();
    t.into_sink()
}

/// The tree builder's own switch table (§13.2.6: the generic raw text
/// and RCDATA element parsing algorithms, and the `plaintext` rule),
/// and nothing else of a tree builder.
///
/// Four of the tokenizer's states are reached only this way, so without
/// it a third of the description is never exercised.
fn switch_for(name: &str) -> Option<EntryState> {
    Some(match name {
        "title" | "textarea" => EntryState::Rcdata,
        "style" | "xmp" | "iframe" | "noembed" | "noframes" | "noscript" => EntryState::Rawtext,
        "script" => EntryState::ScriptData,
        "plaintext" => EntryState::Plaintext,
        _ => return None,
    })
}

/// Tokenizes `input`, switching states on the start tags the tree
/// builder switches on.
#[must_use]
pub fn run_driven(input: &str, back: Back) -> Collect {
    let mut t = Tokenizer::new(Collect::default(), input, back);
    let mut seen = 0usize;
    while !t.is_done() {
        t.step();
        let tokens = &t.sink().tokens;
        if tokens.len() > seen {
            let next = tokens.get(seen..).unwrap_or_default().to_vec();
            seen = tokens.len();
            for token in next {
                if let Token::StartTag { name, .. } = token
                    && let Some(e) = switch_for(&name)
                {
                    t.switch_to(e);
                }
            }
        }
    }
    t.into_sink()
}

/// The characters of the run, joined — what most of these tests check.
#[must_use]
pub fn text(c: &Collect) -> String {
    c.tokens
        .iter()
        .filter_map(|t| match *t {
            Token::Character(ch) => Some(ch),
            Token::Doctype { .. }
            | Token::StartTag { .. }
            | Token::EndTag { .. }
            | Token::Comment(_)
            | Token::ProcessingInstruction { .. }
            | Token::Eof => None,
        })
        .collect()
}
