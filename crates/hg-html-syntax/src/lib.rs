//! `hg-html-syntax`: the HTML tokenizer.
//!
//! The standard's 80 tokenizer states are written in `.hgtok` under
//! `states/`, and `hg-htmlgen` compiles them into two independent back
//! ends (style spec 6.2, ST-D3):
//!
//! - **the tokenizer**, generated as plain Rust — one function per
//!   state, every rule an `if`, every action a call;
//! - **the reference stepper**, generated as *data* — a table of states,
//!   patterns and operations, interpreted by [`tokenizer::Tokenizer::step_table`].
//!
//! Both are compiled from the one description, so a mistake in the
//! description appears in both and is caught by html5lib, not by
//! comparing them. What comparing them catches is a mistake in the
//! *generator*, which has two unrelated back ends and is the part most
//! likely to be wrong; and when the tokenizer grows the run-consuming
//! fast paths ST-M1 plans for the data, RCDATA, RAWTEXT, script data,
//! attribute value and comment states, the stepper stays spec-literal
//! and becomes the fixed point those are measured against.
//!
//! Nothing in `generated/` is hand-written (G29): a hand edit fails CI.
//! Two states are, and they say so — see `states/06-charref.hgtok`.

pub mod generated;
pub mod table;
pub mod token;
pub mod tokenizer;

pub use generated::states::{EntryState, Extern, State};
pub use token::{Attribute, Sink, Token};
pub use tokenizer::{Back, Tokenizer};
