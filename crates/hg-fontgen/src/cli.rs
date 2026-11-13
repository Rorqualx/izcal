//! Reading the descriptions and writing the generated files.
//!
//! The same shape `hg-opgen` has: the generator owns the list of outputs,
//! reads its sources sorted by name so the order does not depend on the
//! file system, and pipes every Rust file through `rustfmt` so what is
//! checked in is byte-identical on every OS.

use std::path::{Path, PathBuf};

use crate::{Error, Model, emit, model};

/// Where the descriptions live, relative to the workspace root.
pub const SOURCES: &str = "crates/hg-fontfmt/tables";

/// What a run writes, relative to the workspace root.
#[must_use]
pub fn outputs(m: &Model) -> Vec<(PathBuf, String)> {
    vec![
        (
            PathBuf::from("crates/hg-fontfmt/src/generated/tables.rs"),
            emit::tables(m),
        ),
        (
            PathBuf::from("crates/hg-fontfmt/src/generated/proofs.rs"),
            emit::proofs(m),
        ),
        (
            PathBuf::from("crates/hg-fontfmt/src/generated/grammar.tsv"),
            emit::grammar(m),
        ),
        (
            PathBuf::from("crates/hg-fontfmt/src/generated/anchors.tsv"),
            emit::anchors(m),
        ),
    ]
}

/// Reads every `.hgfont` under `root/SOURCES`, sorted by name.
///
/// # Errors
/// A message when the directory or a file cannot be read.
pub fn sources(root: &Path) -> Result<Vec<(String, String)>, String> {
    let dir = root.join(SOURCES);
    let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "hgfont"))
        .collect();
    names.sort();
    if names.is_empty() {
        return Err(format!("{}: no .hgfont files", dir.display()));
    }
    let mut out = Vec::new();
    for p in names {
        let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let label = p
            .file_name()
            .map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into());
        // Normalized so a checkout with CRLF endings generates the same
        // bytes as one without.
        out.push((label, text.replace("\r\n", "\n")));
    }
    Ok(out)
}

/// Builds the model from the descriptions under `root`.
///
/// # Errors
/// A message when a file cannot be read, parsed or checked.
pub fn build(root: &Path) -> Result<Model, String> {
    let src = sources(root)?;
    model(&src).map_err(|e: Error| e.to_string())
}

/// Writes every output.
///
/// # Errors
/// A message when a file cannot be read or written.
pub fn generate(root: &Path) -> Result<Vec<PathBuf>, String> {
    let m = build(root)?;
    let mut written = Vec::new();
    for (rel, text) in outputs(&m) {
        let path = root.join(&rel);
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
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
/// A message when a file cannot be read.
pub fn stale(root: &Path) -> Result<Vec<PathBuf>, String> {
    let m = build(root)?;
    let mut out = Vec::new();
    for (rel, text) in outputs(&m) {
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
