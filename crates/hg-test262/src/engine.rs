//! The engine adapter interface.
//!
//! An [`Engine`] runs one [`Job`] in a fresh realm and reports how it
//! ended. The realm must hold what INTERPRETING.md ("Host-Defined
//! Functions") lists: `print`, and `$262` with `createRealm`,
//! `detachArrayBuffer`, `evalScript`, `gc`, `global`,
//! `AbstractModuleSource`, `agent` and (where the host can) `IsHTMLDDA`.
//! An adapter states which optional parts its host lacks in
//! [`HostCapabilities`]; the runner then does not run the tests that need
//! them and reports them as excluded, with the reason.
//!
//! At JS-M0 the one adapter is engine262 run black-box under Node
//! ([`crate::engine262`]); at JS-M1 Izcal's engine implements the same
//! trait.

use std::path::PathBuf;
use std::time::Duration;

use crate::frontmatter::Phase;

/// What an engine's host provides beyond the required `$262` members.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostCapabilities {
    /// The value of [[CanBlock]] for the agent that runs the test
    /// (`CanBlockIsTrue` / `CanBlockIsFalse` tests run only on a match).
    pub can_block: bool,
    /// `$262.agent` exists (multi-agent tests).
    pub agent: bool,
    /// `$262.IsHTMLDDA` exists (tests with the `IsHTMLDDA` feature).
    pub is_html_dda: bool,
}

/// How the test's source is interpreted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    /// Global code: `prefix`, then `harness`, then the test, as one script.
    Script,
    /// Module code: `harness` is evaluated first as a script in the global
    /// scope, then the test file is loaded as a module from its own
    /// directory, so `./name` specifiers find its `_FIXTURE` files.
    Module,
}

/// One execution of one test file, fully expanded.
#[derive(Clone, Debug)]
pub struct Job {
    /// The test's path relative to test262's `test/` directory.
    pub path: String,
    /// The test file on disk (module tests resolve `./` from its directory).
    pub file: PathBuf,
    /// Script or module.
    pub kind: SourceKind,
    /// `"use strict";\n` for a strict-mode script run, else empty
    /// (INTERPRETING.md, "Strict Mode").
    pub prefix: &'static str,
    /// The harness files' text, concatenated in evaluation order; empty
    /// for `raw` tests.
    pub harness: String,
    /// The test file's text, unmodified.
    pub test: String,
    /// The test has the `raw` flag: nothing may be added to its source.
    pub raw: bool,
    /// The test has the `async` flag.
    pub is_async: bool,
    /// How long the run may take before it is stopped.
    pub timeout: Duration,
}

impl Job {
    /// The script source an engine evaluates for a [`SourceKind::Script`]
    /// job: the strict prefix, the harness, then the test.
    pub fn script_source(&self) -> String {
        let mut s = String::with_capacity(self.prefix.len() + self.harness.len() + self.test.len());
        s.push_str(self.prefix);
        s.push_str(&self.harness);
        s.push_str(&self.test);
        s
    }
}

/// How the synchronous part of a run ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Completion {
    /// Evaluation (and, for a module, linking) completed without an
    /// uncaught exception.
    Normal,
    /// An exception was not caught.
    Threw {
        /// The phase it occurred in, when the adapter can tell.
        phase: Option<Phase>,
        /// The `name` of the thrown value's constructor; for a primitive,
        /// `typeof` the value prefixed by `non-object:`.
        name: String,
        /// The error's message, or the value as a string, for the ledger.
        message: String,
    },
    /// The run did not finish in its time.
    Timeout,
    /// The engine or its host failed without reporting a completion.
    Crashed(String),
}

/// What one run produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// How the synchronous part ended.
    pub completion: Completion,
    /// The strings passed to `print`, in order, including any after the
    /// synchronous part finished (the async protocol).
    pub prints: Vec<String>,
    /// Anything else the engine wrote, kept short, for diagnosing a
    /// failure.
    pub diagnostics: String,
}

/// An engine under test.
pub trait Engine: Sync {
    /// A short name, used in ledgers (`engine262`, `izcal`).
    fn name(&self) -> &str;
    /// The engine's version or commit, recorded in ledgers.
    fn version(&self) -> String;
    /// What the host provides beyond the required members.
    fn host(&self) -> HostCapabilities;
    /// Runs one job in a fresh realm. `worker` is a small integer unique
    /// among concurrently running calls, for per-worker scratch space.
    fn run(&self, job: &Job, worker: usize) -> Outcome;
}
