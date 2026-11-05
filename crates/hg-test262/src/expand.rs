//! Expanding a test file into its runs (INTERPRETING.md, "Strict Mode",
//! "Test262-Defined Bindings", `flags`, `includes`).
//!
//! - A test runs twice, non-strict then strict, unless `onlyStrict`,
//!   `noStrict`, `module` or `raw` says once.
//! - Strict mode is `"use strict";` and a newline inserted as the first
//!   characters of the file, before any other modification.
//! - Unless `raw`: `assert.js` and `sta.js`, then `doneprintHandle.js` for
//!   `async`, then `includes` in order, are evaluated in the global scope
//!   before the test.
//! - `raw`: the source is not modified and no harness file is evaluated.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::engine::{HostCapabilities, Job, SourceKind};
use crate::frontmatter::FrontMatter;

/// The strictness a run uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    /// Non-strict global code.
    Sloppy,
    /// Strict global code (the `"use strict";` prefix).
    Strict,
    /// Module code (always strict; no prefix).
    Module,
}

impl Mode {
    /// The mode's name in ledgers.
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Sloppy => "sloppy",
            Mode::Strict => "strict",
            Mode::Module => "module",
        }
    }

    /// Parses a ledger's mode name.
    pub fn parse(s: &str) -> Option<Mode> {
        match s {
            "sloppy" => Some(Mode::Sloppy),
            "strict" => Some(Mode::Strict),
            "module" => Some(Mode::Module),
            _ => None,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The strict-mode prefix (INTERPRETING.md, "Strict Mode").
pub const USE_STRICT: &str = "\"use strict\";\n";

/// The runs a test file needs, in order.
pub fn modes(fm: &FrontMatter) -> Vec<Mode> {
    let f = &fm.flags;
    if f.module {
        vec![Mode::Module]
    } else if f.raw || f.no_strict {
        vec![Mode::Sloppy]
    } else if f.only_strict {
        vec![Mode::Strict]
    } else {
        vec![Mode::Sloppy, Mode::Strict]
    }
}

/// The harness files evaluated before the test, in order.
pub fn harness_files(fm: &FrontMatter) -> Vec<String> {
    if fm.flags.raw {
        return Vec::new();
    }
    let mut files = vec!["assert.js".to_string(), "sta.js".to_string()];
    if fm.flags.is_async {
        files.push("doneprintHandle.js".to_string());
    }
    files.extend(fm.includes.iter().cloned());
    files
}

/// Why a test is not run on a host, or `None` if the host can run it.
pub fn host_gap(fm: &FrontMatter, source: &str, host: &HostCapabilities) -> Option<String> {
    if fm.flags.can_block_is_true && !host.can_block {
        return Some("host: [[CanBlock]] is false (CanBlockIsTrue)".into());
    }
    if fm.flags.can_block_is_false && host.can_block {
        return Some("host: [[CanBlock]] is true (CanBlockIsFalse)".into());
    }
    if !host.is_html_dda && fm.features.iter().any(|f| f == "IsHTMLDDA") {
        return Some("host: no $262.IsHTMLDDA".into());
    }
    // Multi-agent tests carry no feature tag of their own; they are the
    // ones that call the agent API.
    if !host.agent && source.contains("$262.agent") {
        return Some("host: no $262.agent".into());
    }
    None
}

/// Reads harness files once each from test262's `harness/` directory.
#[derive(Debug)]
pub struct Harness {
    dir: PathBuf,
    cache: Mutex<BTreeMap<String, Arc<str>>>,
}

impl Harness {
    /// A reader for `dir` (test262's `harness/`).
    pub fn new(dir: impl Into<PathBuf>) -> Harness {
        Harness {
            dir: dir.into(),
            cache: Mutex::new(BTreeMap::new()),
        }
    }

    /// One harness file's text.
    pub fn file(&self, name: &str) -> Result<Arc<str>, String> {
        if name.contains("..") || name.starts_with('/') {
            return Err(format!("include `{name}` leaves harness/"));
        }
        if let Some(t) = self.cache.lock().unwrap().get(name) {
            return Ok(t.clone());
        }
        let path = self.dir.join(name);
        let text: Arc<str> = std::fs::read_to_string(&path)
            .map_err(|e| format!("include `{name}`: {}: {e}", path.display()))?
            .into();
        self.cache
            .lock()
            .unwrap()
            .insert(name.to_string(), text.clone());
        Ok(text)
    }

    /// The concatenated harness for a test, each file ending in a newline.
    pub fn text_for(&self, fm: &FrontMatter) -> Result<String, String> {
        let mut out = String::new();
        for name in harness_files(fm) {
            out.push_str(&self.file(&name)?);
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        Ok(out)
    }
}

/// Builds the job for one run of a test.
pub fn job(
    path: &str,
    file: &Path,
    source: &str,
    fm: &FrontMatter,
    harness: &str,
    mode: Mode,
    timeout: Duration,
) -> Job {
    Job {
        path: path.to_string(),
        file: file.to_path_buf(),
        kind: if mode == Mode::Module {
            SourceKind::Module
        } else {
            SourceKind::Script
        },
        prefix: if mode == Mode::Strict { USE_STRICT } else { "" },
        harness: harness.to_string(),
        test: source.to_string(),
        raw: fm.flags.raw,
        is_async: fm.flags.is_async,
        timeout,
    }
}

#[cfg(test)]
mod tests;
