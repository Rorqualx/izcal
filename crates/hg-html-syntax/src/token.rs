//! What the tokenizer produces, and who receives it.

/// One attribute of a tag token.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attribute {
    /// Its name, lowercased as the prose lowercases it.
    pub name: String,
    /// Its value.
    pub value: String,
}

/// A token, as §13.2.5 names them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A DOCTYPE token. A missing name, public or system identifier is
    /// `None`, which is not the same as an empty one.
    Doctype {
        /// Its name.
        name: Option<String>,
        /// Its public identifier.
        public_id: Option<String>,
        /// Its system identifier.
        system_id: Option<String>,
        /// Its force-quirks flag.
        force_quirks: bool,
    },
    /// A start tag.
    StartTag {
        /// Its tag name.
        name: String,
        /// Its attributes, in the order they survive the
        /// duplicate-attribute rule.
        attrs: Vec<Attribute>,
        /// Its self-closing flag.
        self_closing: bool,
    },
    /// An end tag. The standard gives it attributes and a self-closing
    /// flag too, and the tree builder is what reports them as errors.
    EndTag {
        /// Its tag name.
        name: String,
        /// Its attributes.
        attrs: Vec<Attribute>,
        /// Its self-closing flag.
        self_closing: bool,
    },
    /// A comment, with its data.
    Comment(String),
    /// One character token.
    Character(char),
    /// The end-of-file token.
    Eof,
}

/// Where tokens and parse errors go.
///
/// The tokenizer holds no tree and asks the tree builder exactly one
/// question, which is [`Sink::adjusted_node_not_html`]. "Appropriate end
/// tag token" is not one of them: it is decided from the last start tag
/// this tokenizer emitted, which the tokenizer itself remembers.
pub trait Sink {
    /// Receives a token.
    fn token(&mut self, token: Token);

    /// Receives a parse error: its name, and the position of the
    /// character that caused it, counting lines and columns from one.
    fn error(&mut self, code: &'static str, line: u32, column: u32);

    /// "If there is an adjusted current node and it is not an element in
    /// the HTML namespace" — asked only by the markup declaration open
    /// state, and the reason a CDATA section is not simply a comment.
    /// With no tree builder there is no adjusted current node, so the
    /// default is `false`.
    fn adjusted_node_not_html(&mut self) -> bool {
        false
    }
}
