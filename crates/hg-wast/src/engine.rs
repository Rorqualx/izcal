//! The engine a script is run against.
//!
//! At JS-M0 there is one implementation, [`Reference`]: the
//! WebAssembly specification's own interpreter, built from the pinned
//! tree and driven black-box through its documented command line. It is
//! never linked, never vendored and its source is never read (M-1 part
//! 3); the only things used are its exit status and what it prints.
//!
//! When Izcal has an engine, it implements this same trait and is
//! judged by the same runs.
//!
//! # Why a whole script at a time
//!
//! A `.wast` script is stateful: a module is defined, registered,
//! imported by the next one. Handing an engine one command at a time
//! would mean rebuilding that state outside the engine, which is a
//! second implementation of the thing being tested. Running the script
//! whole keeps the runner out of that, and the verdict it reads back
//! is the engine's own.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// What an engine made of a script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// Whether every assertion in the script held.
    pub passed: bool,
    /// What the engine printed, kept for diagnosing a disagreement.
    pub output: String,
}

/// An engine that can run a whole `.wast` script.
pub trait ScriptEngine {
    /// A short name, used in ledgers.
    fn name(&self) -> &str;
    /// The engine's version, recorded in ledgers.
    fn version(&self) -> String;
    /// Runs the script at `path` and reports whether it held.
    fn run_script(&self, path: &Path) -> Verdict;
}

/// The WebAssembly reference interpreter, run as a child process.
#[derive(Clone, Debug)]
pub struct Reference {
    exe: PathBuf,
}

impl Reference {
    /// An interpreter at `exe`.
    #[must_use]
    pub fn new(exe: impl Into<PathBuf>) -> Self {
        Self { exe: exe.into() }
    }
}

impl ScriptEngine for Reference {
    fn name(&self) -> &str {
        "wasm"
    }

    /// The interpreter's banner.
    ///
    /// `-v` prints it and then falls into the interactive loop, because
    /// the documented rule is that a run with no file left to do lands
    /// in the REPL. With a terminal on standard input that waits for
    /// ever, so standard input is closed here and only the first line
    /// is kept: the banner is printed again on the way into the loop.
    /// A tool that can hang is a tool that will, on some machine, at
    /// some hour.
    fn version(&self) -> String {
        Command::new(&self.exe)
            .arg("-v")
            .stdin(Stdio::null())
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// The interpreter exits 0 when every assertion in the script held
    /// and non-zero otherwise, naming the command that failed. Both are
    /// read; neither is interpreted further.
    fn run_script(&self, path: &Path) -> Verdict {
        // Run in the script's own directory, so a path the script names
        // resolves the way it would for the suite.
        let dir = path.parent().unwrap_or(Path::new("."));
        let file = path.file_name().unwrap_or(path.as_os_str());
        // Standard input is closed for the same reason as in
        // `version`: an argument the interpreter cannot use leaves it
        // in the interactive loop, and a run that waits for a person is
        // a run that never ends in CI.
        match Command::new(&self.exe)
            .current_dir(dir)
            .arg(file)
            .stdin(Stdio::null())
            .output()
        {
            Ok(out) => {
                let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&out.stderr));
                Verdict {
                    passed: out.status.success(),
                    output: text,
                }
            }
            Err(e) => Verdict {
                passed: false,
                output: format!("could not run {}: {e}", self.exe.display()),
            },
        }
    }
}
