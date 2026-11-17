//! `hg-ipc-codegen`: compiles `.hgidl` into Rust for `hg-ipc-gen`
//! (Stage 1 design 4.1; security scope 6.6; coding standard CS-ST-17).
//!
//! [`generate`] parses ([`syntax`]), checks ([`check`]) and emits
//! ([`emit`]) one file, then formats it with the pinned toolchain's
//! rustfmt, so the checked-in output is the same on every OS.

pub mod check;
pub mod emit;
pub mod shape;
pub mod syntax;

use std::collections::BTreeMap;
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

/// Where an `import` finds a sibling protocol's source.
///
/// The parameter is a protocol name, never a path: an import names a
/// sibling in the same `idl/` directory and cannot leave it, so no
/// format can acquire a dependency on another package behind the back
/// of a rule stated over packages (G63).
pub trait Imports {
    /// The source of sibling protocol `protocol`, if it exists.
    fn source(&self, protocol: &str) -> Option<String>;
}

/// No imports: every `import` is refused. What [`generate`] uses.
#[derive(Debug, Clone, Copy)]
pub struct NoImports;

impl Imports for NoImports {
    fn source(&self, _protocol: &str) -> Option<String> {
        None
    }
}

/// The `.hgidl` files in one directory, by protocol name.
#[derive(Debug, Clone)]
pub struct DirImports(PathBuf);

impl DirImports {
    /// Resolves against `dir`, which is one crate's `idl/`.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self(dir.into())
    }
}

impl Imports for DirImports {
    fn source(&self, protocol: &str) -> Option<String> {
        std::fs::read_to_string(self.0.join(format!("{protocol}.hgidl"))).ok()
    }
}

/// Parses and checks `source` with no imports, and emits unformatted
/// Rust.
///
/// # Errors
/// The first syntax or rule error.
pub fn generate_unformatted(source: &str, label: &str) -> Result<String, syntax::Error> {
    generate_unformatted_with(source, label, &NoImports)
}

/// Parses and checks `source`, resolving its imports with `imports`,
/// and emits unformatted Rust.
///
/// # Errors
/// The first syntax or rule error, in this file or in one it imports.
pub fn generate_unformatted_with(
    source: &str,
    label: &str,
    imports: &dyn Imports,
) -> Result<String, syntax::Error> {
    let file = syntax::parse(source)?;
    // Only what this file asks for is read, and only one level deep:
    // `check` refuses an imported protocol that itself imports, so this
    // never recurses and no cycle can be written.
    let mut siblings = BTreeMap::new();
    for imp in &file.imports {
        if let Some(src) = imports.source(&imp.protocol) {
            // A parse or check error in the sibling carries the
            // SIBLING's line and column, and the caller prints it
            // against this file's label. Say which file it is in, or
            // the position points into the wrong one.
            let blame = |e: syntax::Error| syntax::Error {
                pos: e.pos,
                message: format!("in imported `{}.hgidl`: {}", imp.protocol, e.message),
            };
            let parsed = syntax::parse(&src).map_err(blame)?;
            // Before checking it: a sibling that imports would be
            // checked here with no siblings of its own, and would
            // report its own unresolved import rather than this file's
            // depth violation. The rule is restated in `check` for
            // callers that build `siblings` themselves.
            if !parsed.imports.is_empty() {
                return Err(syntax::Error {
                    pos: imp.pos,
                    message: format!(
                        "`{}` itself imports; an imported protocol may not import, \
                         so the graph stays one level deep",
                        imp.protocol
                    ),
                });
            }
            siblings.insert(imp.protocol.clone(), check::check(parsed).map_err(blame)?);
        }
    }
    let checked = check::check_with(file, &siblings)?;
    Ok(emit::emit(&checked, label))
}

/// The formatted Rust for `source`; `label` names the input in the
/// generated header.
///
/// # Errors
/// [`GenError`] if the IDL is invalid or rustfmt fails.
pub fn generate(source: &str, label: &str) -> Result<String, GenError> {
    generate_with(source, label, &NoImports)
}

/// The formatted Rust for `source`, resolving its imports with
/// `imports`; `label` names the input in the generated header.
///
/// # Errors
/// [`GenError`] if the IDL is invalid or rustfmt fails.
pub fn generate_with(source: &str, label: &str, imports: &dyn Imports) -> Result<String, GenError> {
    let raw = generate_unformatted_with(source, label, imports).map_err(GenError::Idl)?;
    rustfmt(&raw)
}

/// The formatted Rust for one [`Pair`], reading the `.hgidl` and
/// resolving its imports from the directory it sits in.
///
/// This is what every caller should use: it puts the resolution rule —
/// a sibling in the same `idl/` — in one place instead of in each
/// crate's freshness test.
///
/// # Errors
/// [`GenError`] if the file cannot be read, the IDL is invalid, or
/// rustfmt fails.
pub fn generate_pair(pair: &Pair) -> Result<String, GenError> {
    let src = std::fs::read_to_string(&pair.idl)
        .map_err(|e| GenError::Format(format!("{}: {e}", pair.idl.display())))?;
    let dir = pair.idl.parent().unwrap_or(Path::new("."));
    generate_with(&src, &pair.label, &DirImports::new(dir))
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
