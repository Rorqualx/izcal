//! Reading the overlay, comparing it with webref, and writing the
//! generated files.
//!
//! The same shape `hg-opgen`, `hg-fontgen` and `hg-htmlgen` have: the
//! generator owns the list of outputs, reads its sources sorted by name
//! so the order does not depend on the file system, and pipes every
//! Rust file through `rustfmt` so what is checked in is byte-identical
//! on every OS.
//!
//! One difference. The other generators can run from the repository
//! alone; this one cannot. Every output carries webref's data, so
//! without the pinned `css.json` there is nothing to generate and
//! nothing to check — and quietly doing nothing would let a stale table
//! pass. So a run without the data **fails**, with the command that
//! fetches it, unless `--allow-missing-data` is given.

use std::path::{Path, PathBuf};

use crate::check::Disagreement;
use crate::{Error, Model, check, emit, model, webref};

/// Where the overlay lives, relative to the workspace root.
pub const SOURCES: &str = "crates/hg-css-props/props";

/// Where the generated files go, relative to the workspace root.
pub const OUTPUT_DIR: &str = "crates/hg-css-props/src/generated";

/// What a run writes.
#[must_use]
pub fn outputs(m: &Model) -> Vec<(PathBuf, String)> {
    let dir = PathBuf::from(OUTPUT_DIR);
    vec![
        (dir.join("kinds.rs"), emit::kinds(m)),
        (dir.join("properties.rs"), emit::properties(m)),
        (dir.join("grammars.rs"), emit::grammars(m)),
        (dir.join("properties.tsv"), emit::table(m)),
    ]
}

/// Reads every `.hgprop` under `root/SOURCES`, sorted by name.
///
/// # Errors
/// A message when the directory or a file cannot be read.
pub fn sources(root: &Path) -> Result<Vec<(String, String)>, String> {
    let dir = root.join(SOURCES);
    // An entry that cannot be read is an error, never a skip: dropping
    // one would leave the overlay short of a property and let `check`
    // call a stale table fresh.
    let mut names: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.extension().is_some_and(|x| x == "hgprop") {
            names.push(path);
        }
    }
    names.sort();
    if names.is_empty() {
        return Err(format!("{}: no .hgprop files", dir.display()));
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

/// Builds the model from the overlay under `root`, checked against
/// itself.
///
/// # Errors
/// A message when a file cannot be read, parsed or checked.
pub fn build(root: &Path) -> Result<Model, String> {
    let m = model(&sources(root)?).map_err(|e: Error| e.to_string())?;
    // Two properties whose names differ only where the constant rule
    // does not look would compile to one constant, silently.
    let mut seen: std::collections::BTreeMap<String, &str> = std::collections::BTreeMap::new();
    for (c, p) in emit::constants(&m) {
        if let Some(other) = seen.insert(c.clone(), &p.name) {
            return Err(format!(
                "{} and {other} would both be the constant `{c}`",
                p.name
            ));
        }
    }
    Ok(m)
}

/// The pinned catalogue, when it is in the cache.
///
/// # Errors
/// A message when the file is there but cannot be read or parsed.
pub fn catalogue() -> Result<Option<webref::Catalogue>, String> {
    let path = webref::path_from_env()?;
    if !path.is_file() {
        return Ok(None);
    }
    webref::read(&path).map(Some)
}

/// The sentence a run prints when the data is not there.
#[must_use]
pub fn missing_data_message() -> String {
    format!(
        "the pinned {} {} is not in the cache; run `tools/fetch-test-data.sh {}`",
        webref::DATA_SET,
        webref::PIN,
        webref::DATA_SET
    )
}

/// Builds the model and compares it with the pinned catalogue.
///
/// # Errors
/// A message when the overlay cannot be read or checked, when the data
/// is missing and `allow_missing` is false, or when the two disagree.
pub fn verified(root: &Path, allow_missing: bool) -> Result<(Model, Vec<Disagreement>), String> {
    let m = build(root)?;
    let Some(cat) = catalogue()? else {
        if allow_missing {
            return Ok((m, Vec::new()));
        }
        return Err(missing_data_message());
    };
    let found = check::against_webref(&m, &cat);
    Ok((m, found))
}

/// Writes every output.
///
/// # Errors
/// A message when the overlay and webref disagree, or a file cannot be
/// written.
pub fn generate(root: &Path) -> Result<Vec<PathBuf>, String> {
    let (m, found) = verified(root, false)?;
    if !found.is_empty() {
        return Err(report(&found));
    }
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
/// A message when the overlay and webref disagree, or a file cannot be
/// read.
pub fn stale(root: &Path) -> Result<Vec<PathBuf>, String> {
    let (m, found) = verified(root, false)?;
    if !found.is_empty() {
        return Err(report(&found));
    }
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

/// How many disagreements a message lists before it stops counting.
const SHOWN: usize = 40;

/// The message a disagreeing run fails with.
#[must_use]
pub fn report(found: &[Disagreement]) -> String {
    let mut out = format!(
        "the overlay and {} {} disagree in {} place(s):\n",
        webref::DATA_SET,
        webref::PIN,
        found.len()
    );
    for d in found.iter().take(SHOWN) {
        out.push_str(&format!("  {d}\n"));
    }
    if let Some(rest) = found.len().checked_sub(SHOWN)
        && rest > 0
    {
        out.push_str(&format!("  ... and {rest} more\n"));
    }
    out.push_str(
        "Triage each one: either the specification changed and the overlay follows it, or the \
         overlay is right and the change is a regression to report. ST-7 blocks the re-pin until \
         the overlay is fixed. Never regenerate the overlay from webref to make this go away — \
         that turns the check off.",
    );
    out
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
