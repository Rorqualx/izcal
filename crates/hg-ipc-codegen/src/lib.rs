//! `hg-ipc-codegen`: compiles `.hgidl` into Rust for `hg-ipc-gen`
//! (Stage 1 design 4.1; security scope 6.6; coding standard CS-ST-17).
//!
//! [`generate`] parses ([`syntax`]), checks ([`check`]) and emits
//! ([`emit`]) one file, then formats it with the pinned toolchain's
//! rustfmt, so the checked-in output is the same on every OS.

pub mod check;
pub mod emit;
pub mod syntax;

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Why generation failed.
#[derive(Debug)]
pub enum GenError {
    /// The `.hgidl` is invalid.
    Idl(syntax::Error),
    /// rustfmt could not run or rejected the output.
    Format(String),
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idl(e) => write!(f, "{e}"),
            Self::Format(e) => write!(f, "rustfmt: {e}"),
        }
    }
}

impl std::error::Error for GenError {}

/// Parses and checks `source`, and emits unformatted Rust.
///
/// # Errors
/// The first syntax or rule error.
pub fn generate_unformatted(source: &str, label: &str) -> Result<String, syntax::Error> {
    let file = syntax::parse(source)?;
    let checked = check::check(file)?;
    Ok(emit::emit(&checked, label))
}

/// The formatted Rust for `source`; `label` names the input in the
/// generated header.
///
/// # Errors
/// [`GenError`] if the IDL is invalid or rustfmt fails.
pub fn generate(source: &str, label: &str) -> Result<String, GenError> {
    let raw = generate_unformatted(source, label).map_err(GenError::Idl)?;
    rustfmt(&raw)
}

fn rustfmt(src: &str) -> Result<String, GenError> {
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
        .map_err(|e| GenError::Format(e.to_string()))?;
    child
        .stdin
        .take()
        .ok_or_else(|| GenError::Format("no stdin".into()))?
        .write_all(src.as_bytes())
        .map_err(|e| GenError::Format(e.to_string()))?;
    let out = child
        .wait_with_output()
        .map_err(|e| GenError::Format(e.to_string()))?;
    if !out.status.success() {
        return Err(GenError::Format(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| GenError::Format(e.to_string()))
}

/// One `.hgidl` file and the Rust file it generates.
#[derive(Debug, Clone)]
pub struct Pair {
    /// The input.
    pub idl: PathBuf,
    /// The output.
    pub rust: PathBuf,
    /// The label in the generated header (a repository-relative path).
    pub label: String,
}

/// The pairs for every `.hgidl` in `crate_dir/idl`, generating into
/// `crate_dir/src/generated`, sorted by name.
///
/// # Errors
/// The directory could not be read.
pub fn pairs(crate_dir: &Path, crate_name: &str) -> std::io::Result<Vec<Pair>> {
    let mut v = Vec::new();
    for entry in std::fs::read_dir(crate_dir.join("idl"))? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("hgidl") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        v.push(Pair {
            rust: crate_dir.join("src/generated").join(format!("{stem}.rs")),
            label: format!("crates/{crate_name}/idl/{stem}.hgidl"),
            idl: path,
        });
    }
    v.sort_by(|a, b| a.idl.cmp(&b.idl));
    Ok(v)
}
