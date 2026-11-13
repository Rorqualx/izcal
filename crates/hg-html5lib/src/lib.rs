//! `hg-html5lib`: the html5lib-tests tokenizer runner.
//!
//! ST-1 is a ship blocker and asks for **100% of the html5lib tokenizer
//! tests, by both the production tokenizer and the reference stepper,
//! including error codes and positions**. This is the thing that
//! measures it. At ST-M0 it reports what it finds; the number is
//! recorded, and [`BASELINE`] keeps it from going down.
//!
//! What it does not judge, and why ([`cases::NOT_THE_TOKENIZERS`]):
//!
//! - **The three input-stream parse errors** (`surrogate-in-input-stream`,
//!   `noncharacter-in-input-stream`, `control-character-in-input-stream`).
//!   The standard raises these in §13.2.3.5, the input stream, which
//!   `hg-html-driver` owns and which does not exist yet.
//! - **The two tree-construction ones** (`end-tag-with-attributes`,
//!   `end-tag-with-trailing-solidus`), raised where an end tag is
//!   *processed*, which is ST-M1's.
//! - **Lone surrogates in the input.** Rust's `String` cannot hold one,
//!   so the suite's `\uD800` escapes decode to U+FFFD on both sides.
//!   Cases whose input holds one are run, and counted, so the number is
//!   not quietly inflated by them.
//!
//! # Where parse errors are reported
//!
//! The standard does not say, and the suite does. Calibrated against it:
//! a state that consumed a character reports **that character's
//! position**; a state that consumed nothing reports the **next**
//! character's; and neither the end of the file nor a pending reconsume
//! moves the position on, because in each there is no next character to
//! move to. Columns count **UTF-16 code units**, so an astral character
//! is two.
//!
//! # The one disagreement left
//!
//! 16 cases expect `eof-in-processing-instruction` at **column 4**
//! whatever follows the target: `<?a a` and `<?Z Z?` both, where the
//! end of the file is at column 6 and 7. The same file's `<?` expects
//! column 3 and `<?a` expects column 4, which are right, so the column
//! in the longer cases simply never advanced past the target. The
//! tokenizer reports the end of the file where it is, and these 16 fail
//! — 32 runs, one per back end. Reported here rather than worked
//! around.

pub mod cases;
pub mod encoding;
pub mod report;
pub mod run;

pub use cases::{Case, Expected, ExpectedError};
pub use report::{Ledger, Outcome};
pub use run::run_case;

/// Where the tokenizer suite lives, under the pinned tree.
pub const TOKENIZER_DIR: &str = "tokenizer";

/// What the data set is called in `tools/test-data.toml`.
pub const DATA_SET: &str = "html5lib";

/// The pinned commit, as `tools/test-data.toml` records it.
pub const PIN: &str = "c5495c539ca8a79c3c5107f5f14010bcff9a35d5";

/// The recorded pass count, by both back ends, at the pinned commit:
/// **14,060 of 14,092 runs, 99.8%**, with **0** token mismatches,
/// measured 13 November 2026.
///
/// A number that can only go up: the crate's own test fails if a change
/// lowers it. Raising it is a line in a commit, with the date.
pub const BASELINE: usize = 14_060;

/// How many runs there are at the pinned commit: 6,820 cases, some run
/// from more than one initial state, each on both back ends.
pub const RUNS: usize = 14_092;

/// The pinned suite's directory, in the cache
/// `tools/fetch-test-data.sh` writes to.
///
/// # Errors
/// A message when neither `$IZCAL_TEST_DATA` nor `$HOME` is set.
pub fn suite_dir() -> Result<std::path::PathBuf, String> {
    Ok(tree_dir()?.join(TOKENIZER_DIR))
}

/// The pinned tree itself, which holds `tokenizer/` and `encoding/`.
///
/// # Errors
/// A message when neither `$IZCAL_TEST_DATA` nor `$HOME` is set.
pub fn tree_dir() -> Result<std::path::PathBuf, String> {
    let cache = match std::env::var_os("IZCAL_TEST_DATA") {
        Some(v) => std::path::PathBuf::from(v),
        None => {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .ok_or("neither IZCAL_TEST_DATA nor HOME is set")?;
            std::path::PathBuf::from(home).join(".cache/izcal/test-data")
        }
    };
    let short = PIN.get(..40).unwrap_or(PIN);
    Ok(cache.join(format!("{DATA_SET}-{short}")))
}
