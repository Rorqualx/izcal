//! Writing the generated files, and saying when they are stale.
//!
//! The same shape the other generators have, with `hg-cssgen`'s one
//! difference: every output carries the standard's data, so a run
//! without it has nothing to generate and nothing to check. That fails
//! rather than quietly doing nothing, which would let a stale table
//! pass.

use std::path::{Path, PathBuf};

use crate::{data, emit};

/// Where the generated files go, relative to the workspace root.
pub const OUTPUT_DIR: &str = "crates/hg-encoding/src/generated";

/// What a run writes.
///
/// # Errors
/// A message when an index the standard defines is missing.
pub fn outputs(d: &data::Data) -> Result<Vec<(PathBuf, String)>, String> {
    let dir = PathBuf::from(OUTPUT_DIR);
    Ok(vec![
        (dir.join("encodings.rs"), emit::encodings(d)),
        (dir.join("indexes.rs"), emit::indexes(d)?),
        (dir.join("encodings.tsv"), emit::table(d)),
    ])
}

/// The sentence a run prints when the data is not there.
#[must_use]
pub fn missing_data_message() -> String {
    format!(
        "the pinned Encoding Standard data is not in the cache; run \
         `tools/fetch-test-data.sh {} {}`",
        data::ENCODINGS_SET,
        data::INDEXES_SET
    )
}

/// Reads the pinned data, failing when it is not there.
///
/// # Errors
/// A message when the data is missing or cannot be read.
pub fn require() -> Result<data::Data, String> {
    data::read()?.ok_or_else(missing_data_message)
}

/// Writes every output.
///
/// # Errors
/// A message when the data is missing or a file cannot be written.
pub fn generate(root: &Path) -> Result<Vec<PathBuf>, String> {
    let d = require()?;
    let mut written = Vec::new();
    for (rel, text) in outputs(&d)? {
        let path = root.join(&rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let body = if rel.extension().is_some_and(|x| x == "rs") {
            rustfmt(&text)?
        } else {
            text
        };
        std::fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(rel);
    }
    Ok(written)
}

/// Lists the outputs that differ from what is checked in (G29).
///
/// # Errors
/// A message when the data is missing or a file cannot be read.
pub fn stale(root: &Path) -> Result<Vec<PathBuf>, String> {
    let d = require()?;
    let mut out = Vec::new();
    for (rel, text) in outputs(&d)? {
        let want = if rel.extension().is_some_and(|x| x == "rs") {
            rustfmt(&text)?
        } else {
            text
        };
        let have = std::fs::read_to_string(root.join(&rel)).unwrap_or_default();
        if have != want {
            out.push(rel);
        }
    }
    Ok(out)
}

/// Runs `rustfmt` over generated Rust, so the checked-in file is
/// byte-identical on every OS.
fn rustfmt(text: &str) -> Result<String, String> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .args([
            "--edition",
            "2024",
            "--emit",
            "stdout",
            "--config",
            "newline_style=Unix",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("rustfmt: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("rustfmt: no stdin")?;
    let owned = text.to_owned();
    // From a thread: a large table fills the pipe and writing inline
    // would deadlock against rustfmt's own output.
    let writer = std::thread::spawn(move || stdin.write_all(owned.as_bytes()));
    let out = child
        .wait_with_output()
        .map_err(|e| format!("rustfmt: {e}"))?;
    writer
        .join()
        .map_err(|_| "rustfmt: the writing thread panicked".to_owned())?
        .map_err(|e| format!("rustfmt: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("rustfmt: {e}"))
}
