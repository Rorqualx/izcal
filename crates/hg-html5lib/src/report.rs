//! The ledger: what passed, what did not, and by how much.

use std::collections::BTreeMap;

/// How one run of one case came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Tokens and parse errors both match, codes and positions.
    Pass,
    /// Tokens match; a parse error's code or position does not.
    ErrorsDiffer,
    /// The tokens do not match.
    TokensDiffer,
}

/// Counts, by file and in total.
#[derive(Debug, Default, Clone)]
pub struct Ledger {
    /// Cases run, counting one per (case, initial state, back end).
    pub runs: usize,
    /// Runs where everything matched.
    pub pass: usize,
    /// Runs where only the parse errors differed.
    pub errors_differ: usize,
    /// Runs where the tokens differed.
    pub tokens_differ: usize,
    /// Cases whose input holds a lone surrogate (see the crate docs).
    pub lone_surrogate: usize,
    /// Per file: runs, then passes.
    pub by_file: BTreeMap<String, (usize, usize)>,
    /// The first few failures, for the report to name.
    pub examples: Vec<String>,
}

/// How many failures the report prints.
pub const EXAMPLES: usize = 20;

impl Ledger {
    /// Records one run.
    pub fn record(&mut self, file: &str, outcome: Outcome, describe: impl FnOnce() -> String) {
        self.runs = self.runs.saturating_add(1);
        let e = self.by_file.entry(file.to_owned()).or_insert((0, 0));
        e.0 = e.0.saturating_add(1);
        match outcome {
            Outcome::Pass => {
                self.pass = self.pass.saturating_add(1);
                e.1 = e.1.saturating_add(1);
            }
            Outcome::ErrorsDiffer => {
                self.errors_differ = self.errors_differ.saturating_add(1);
            }
            Outcome::TokensDiffer => {
                self.tokens_differ = self.tokens_differ.saturating_add(1);
            }
        }
        if outcome != Outcome::Pass && self.examples.len() < EXAMPLES {
            self.examples.push(describe());
        }
    }

    /// The share that passed, as a percentage to one decimal place.
    #[must_use]
    pub fn percent(&self) -> f64 {
        if self.runs == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "a count of test runs, far below 2^53"
        )]
        {
            self.pass as f64 * 100.0 / self.runs as f64
        }
    }

    /// The report, as the CI job prints it.
    #[must_use]
    pub fn render(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "hg-html5lib: {}/{} runs pass ({:.1}%)",
            self.pass,
            self.runs,
            self.percent()
        );
        let _ = writeln!(
            out,
            "  tokens differ: {}; only parse errors differ: {}; inputs with a lone surrogate: {}",
            self.tokens_differ, self.errors_differ, self.lone_surrogate
        );
        for (file, (runs, pass)) in &self.by_file {
            let _ = writeln!(out, "  {file}: {pass}/{runs}");
        }
        for e in &self.examples {
            let _ = writeln!(out, "  ! {e}");
        }
        out
    }
}
