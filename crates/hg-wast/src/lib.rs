//! `hg-wast`: Izcal's runner for the WebAssembly specification's `.wast`
//! test scripts. Tier T, layer `tools` (coding standard 6.2; JS engine
//! spec, crate table).
//!
//! ORC-1 asks for an oracle to run in CI before the code it judges
//! exists. Izcal has no WebAssembly engine yet, so at JS-M0 this crate
//! is the half that can be built and checked on its own: the script
//! grammar. It reads a `.wast` file into [`script::Script`], a list of
//! [`script::Cmd`]s, and nothing else — it never decodes, validates or
//! runs a module. An engine does that, behind the adapter the runner
//! drives; the adapter and the ledger follow.
//!
//! The grammar comes from the specification's prose (the text format's
//! "Lexical Format", and the reference interpreter's documented script
//! commands), never from another parser's source (HGBA-2 M-1 part 3).
//!
//! # Why a module is a span, not a tree
//!
//! A script command names modules, functions and expected results; the
//! module bodies between those commands are the engine's business, not
//! the runner's. So [`script`] parses the command layer exactly and
//! keeps each module as a byte range of the source ([`lex::Span`]),
//! re-emitted verbatim when a module is handed to an engine. That is
//! what lets a script runner exist before an engine does, and it keeps
//! the runner out of the business of being a second WebAssembly
//! implementation, which it must never become: it is the thing that
//! judges one.
//!
//! Finding those ranges is still a lexical problem and not a matter of
//! counting parentheses. `comments.wast` in the suite holds a NUL byte,
//! nested block comments and parentheses inside both strings and
//! comments, so [`lex`] implements the token grammar in full.
//!
//! # Pipeline
//!
//! 1. [`lex::Lexer`] turns the source into tokens with their spans.
//! 2. [`script::parse`] reads the commands: modules (text, `binary`,
//!    `quote`, `definition` and `instance` forms), `register`, the
//!    actions `invoke` and `get`, the eight assertions, and the meta
//!    commands.
//! 3. [`value`] decodes the constants an action passes and the patterns
//!    an assertion expects, including the NaN patterns, the `v128` lane
//!    forms and `either`.

pub mod lex;
pub mod script;
pub mod value;
