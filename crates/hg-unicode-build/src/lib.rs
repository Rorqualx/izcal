//! `hg-unicode-build`: builds `hg-ucd`'s tables from the pinned Unicode
//! Character Database. Tier T, layer `tools` (TX-M0).
//!
//! The database is test-only *input*, fetched by
//! `tools/fetch-test-data.sh` and never committed. What is committed is
//! the Rust this crate emits, so `hg-ucd` and every crate above it build
//! with no network, no build script and no Unicode files on disk. That is
//! the same arrangement `hg-opgen` has with `.hgop`, and it carries the
//! same obligation: the checked-in output must be fresh, which
//! [`check`] decides and a test enforces (G16).
//!
//! ```text
//! hg-unicode-build generate      write the tables
//! hg-unicode-build check         fail if what is checked in is stale
//! ```
//!
//! The derived tables are standards data and ship; the database they are
//! derived from does not.

pub mod emit;
pub mod ucd;

use std::path::{Path, PathBuf};

use ucd::Ucd;

/// Where the generated tables go, relative to the workspace root.
pub const OUTPUT: &str = "crates/hg-ucd/src/generated/tables.rs";

/// The data sets of `tools/test-data.toml` this generator reads.
pub const DATA_SETS: [&str; 2] = ["unicode-ucd", "unicode-idna-mapping"];

/// The pinned Unicode version, which the data sets must agree with.
pub const VERSION: &str = "18.0.0";

/// Where a run reads from and writes to.
#[derive(Debug, Clone)]
pub struct Paths {
    /// The workspace root.
    pub root: PathBuf,
    /// The unpacked `UCD.zip` tree.
    pub ucd: PathBuf,
    /// The directory holding `IdnaMappingTable.txt`.
    pub idna: PathBuf,
}

impl Paths {
    /// The default locations: the cache `tools/fetch-test-data.sh` writes
    /// to, which is `$IZCAL_TEST_DATA` or `~/.cache/izcal/test-data`.
    ///
    /// # Errors
    /// A message when neither `$IZCAL_TEST_DATA` nor `$HOME` is set.
    pub fn from_env(root: impl Into<PathBuf>) -> Result<Self, String> {
        let cache = match std::env::var_os("IZCAL_TEST_DATA") {
            Some(v) => PathBuf::from(v),
            None => {
                let home = std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .ok_or("neither IZCAL_TEST_DATA nor HOME is set")?;
                PathBuf::from(home).join(".cache/izcal/test-data")
            }
        };
        Ok(Self {
            root: root.into(),
            ucd: cache.join(format!("unicode-ucd-{VERSION}")),
            idna: cache.join(format!("unicode-idna-mapping-{VERSION}")),
        })
    }
}

/// Reads the database and renders the tables, without writing anything.
///
/// # Errors
/// A message naming the file when the database cannot be read.
pub fn render(paths: &Paths) -> Result<String, String> {
    let u = Ucd::read(&paths.ucd, &paths.idna)?;
    if u.version != VERSION {
        return Err(format!(
            "the UCD at {} is version {}, and this generator is pinned to {VERSION}",
            paths.ucd.display(),
            u.version
        ));
    }
    Ok(emit::tables(&u))
}

/// Writes the tables, returning the path written.
///
/// # Errors
/// A message when the database cannot be read or the output written.
pub fn generate(paths: &Paths) -> Result<PathBuf, String> {
    let text = render(paths)?;
    let out = paths.root.join(OUTPUT);
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    write_formatted(&out, &text)?;
    Ok(out)
}

/// Whether what is checked in matches what the database says.
///
/// # Errors
/// A message when the database cannot be read, or when the output is
/// stale — the message says how to refresh it (G16).
pub fn check(paths: &Paths) -> Result<(), String> {
    let want = formatted(&render(paths)?)?;
    let out = paths.root.join(OUTPUT);
    let have = std::fs::read_to_string(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    if have == want {
        return Ok(());
    }
    Err(format!(
        "{} is stale (run `cargo run -p hg-unicode-build -- generate`)",
        OUTPUT
    ))
}

/// Runs the text through `rustfmt`, as `hg-opgen` does, so the checked-in
/// file is byte-identical on every OS.
fn formatted(text: &str) -> Result<String, String> {
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
    // Written from a thread: a large table fills the pipe's buffer, and
    // writing it inline would deadlock against rustfmt's own output.
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

fn write_formatted(path: &Path, text: &str) -> Result<(), String> {
    let formatted = formatted(text)?;
    std::fs::write(path, formatted).map_err(|e| format!("{}: {e}", path.display()))
}
