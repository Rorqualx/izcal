//! Turning an [`Outcome`] into a verdict (INTERPRETING.md, "Test
//! Results", `negative`, `async`).
//!
//! - No `negative`, not `async`: pass iff the run completed normally.
//! - `negative`: pass iff an exception escaped whose constructor's name is
//!   the expected `type`, in the expected `phase`. When the adapter cannot
//!   tell the phase, the name decides and the verdict says the phase was
//!   not verified, so a summary can count those passes apart.
//! - `async`: pass iff the synchronous part completed normally and `print`
//!   received `Test262:AsyncTestComplete`; a `Test262:AsyncTestFailure:`
//!   message fails it, and so does neither arriving.

use std::fmt;

use crate::engine::{Completion, Outcome};
use crate::frontmatter::Negative;
use crate::ledger::PHASE_UNVERIFIED;

/// The string an async test prints when it passes.
pub const ASYNC_COMPLETE: &str = "Test262:AsyncTestComplete";
/// The prefix of the string an async test prints when it fails.
pub const ASYNC_FAILURE: &str = "Test262:AsyncTestFailure:";

/// A run's result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    /// The test passed.
    Pass,
    /// The test failed.
    Fail,
    /// The run was stopped at its time limit.
    Timeout,
    /// The engine failed without reporting a completion.
    Crash,
    /// The test file could not be prepared (front matter refused, a
    /// harness file missing).
    Error,
    /// Not run: an excluded feature or path, or a host capability the
    /// engine lacks.
    Excluded,
}

impl Status {
    /// The status's name in ledgers.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Timeout => "timeout",
            Status::Crash => "crash",
            Status::Error => "error",
            Status::Excluded => "excluded",
        }
    }

    /// Parses a ledger's status name.
    pub fn parse(s: &str) -> Option<Status> {
        Some(match s {
            "pass" => Status::Pass,
            "fail" => Status::Fail,
            "timeout" => Status::Timeout,
            "crash" => Status::Crash,
            "error" => Status::Error,
            "excluded" => Status::Excluded,
            _ => return None,
        })
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verdict with its explanation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// Pass, fail, timeout or crash.
    pub status: Status,
    /// Why, for anything but a plain pass.
    pub detail: String,
    /// A negative test passed on the error's name alone, because the
    /// adapter could not report the phase.
    pub phase_unverified: bool,
}

impl Verdict {
    fn new(status: Status, detail: impl Into<String>) -> Verdict {
        Verdict {
            status,
            detail: detail.into(),
            phase_unverified: false,
        }
    }
}

/// Classifies one run of a test with the given expectations.
pub fn classify(negative: Option<&Negative>, is_async: bool, outcome: &Outcome) -> Verdict {
    let completion = match &outcome.completion {
        Completion::Timeout => {
            return Verdict::new(Status::Timeout, "no completion before the time limit");
        }
        Completion::Crashed(why) => return Verdict::new(Status::Crash, why.clone()),
        c => c,
    };
    if let Some(neg) = negative {
        return match completion {
            Completion::Normal => Verdict::new(
                Status::Fail,
                format!(
                    "expected {} {}, but it completed normally",
                    neg.phase, neg.error_type
                ),
            ),
            Completion::Threw {
                phase,
                name,
                message,
            } => {
                if *name != neg.error_type {
                    Verdict::new(
                        Status::Fail,
                        format!(
                            "expected {} {}, got {}: {}",
                            neg.phase, neg.error_type, name, message
                        ),
                    )
                } else {
                    match phase {
                        Some(p) if *p != neg.phase => Verdict::new(
                            Status::Fail,
                            format!(
                                "expected {} in phase {}, thrown in phase {}: {}",
                                name, neg.phase, p, message
                            ),
                        ),
                        Some(_) => Verdict::new(Status::Pass, ""),
                        None => Verdict {
                            status: Status::Pass,
                            detail: format!("{PHASE_UNVERIFIED}; expected {}", neg.phase),
                            phase_unverified: true,
                        },
                    }
                }
            }
            Completion::Timeout | Completion::Crashed(_) => unreachable!("handled above"),
        };
    }
    match completion {
        Completion::Threw {
            phase,
            name,
            message,
        } => {
            let at = phase.map(|p| format!(" ({p})")).unwrap_or_default();
            Verdict::new(Status::Fail, format!("uncaught{at} {name}: {message}"))
        }
        Completion::Normal if is_async => {
            if let Some(f) = outcome.prints.iter().find(|p| p.starts_with(ASYNC_FAILURE)) {
                return Verdict::new(Status::Fail, f.clone());
            }
            if outcome.prints.iter().any(|p| p == ASYNC_COMPLETE) {
                Verdict::new(Status::Pass, "")
            } else {
                Verdict::new(
                    Status::Fail,
                    format!("async test ended without printing {ASYNC_COMPLETE}"),
                )
            }
        }
        Completion::Normal => Verdict::new(Status::Pass, ""),
        Completion::Timeout | Completion::Crashed(_) => unreachable!("handled above"),
    }
}

#[cfg(test)]
mod tests;
