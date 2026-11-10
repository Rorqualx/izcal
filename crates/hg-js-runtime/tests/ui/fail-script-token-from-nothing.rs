//! CS-D3 criterion 1 for the script token: it must be derivable only
//! from `&mut Mutator`. Before the JS-M0 re-run `ScriptToken::for_job`
//! took nothing at all, so any crate could make one out of thin air and
//! the rule was a comment on the function. Now the mutator is the
//! argument.
//!
//! `clippy.toml` also disallows the call outside the agent's two job
//! loops, each of which carries an `#[expect]` naming this rule; that is
//! the half a type cannot express, and clippy refuses a third site.
//!
//! EXPECT: does not compile — the token needs the mutator.

// EXPECT-ERROR: E0061
use hg_js_ops::support::ScriptToken;

pub fn case() -> ScriptToken {
    ScriptToken::for_job()
}
