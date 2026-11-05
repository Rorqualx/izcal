//! A fake engine: it reads `// FAKE: <directive>` lines from the test
//! source and reports what they say, recording each job it was given.
//!
//! - `normal`: complete normally.
//! - `throw <phase> <Name>`: an uncaught `Name` in `phase`.
//! - `print <text>`: `print(text)` (the async protocol), then complete.
//! - `fail-if-strict` / `fail-if-sloppy`: a runtime `Test262Error` in that
//!   mode (checks the `"use strict";` prefix).
//! - `require-harness <marker>`: a runtime `Test262Error` unless the
//!   harness holds `marker`; `require-no-harness`: unless it is empty.
//! - `require-harness-order <m>...`: unless the markers appear in order.
//! - `hang`: time out.

use std::sync::Mutex;

use hg_test262::engine::{Completion, Engine, HostCapabilities, Job, Outcome, SourceKind};
use hg_test262::expand::USE_STRICT;
use hg_test262::frontmatter::Phase;

#[derive(Default)]
pub struct Fake {
    pub jobs: Mutex<Vec<Job>>,
}

fn throw(phase: Phase, name: &str, message: &str) -> Completion {
    Completion::Threw {
        phase: Some(phase),
        name: name.into(),
        message: message.into(),
    }
}

impl Engine for Fake {
    fn name(&self) -> &str {
        "fake"
    }

    fn version(&self) -> String {
        "1".into()
    }

    fn host(&self) -> HostCapabilities {
        HostCapabilities {
            can_block: false,
            agent: false,
            is_html_dda: false,
        }
    }

    fn run(&self, job: &Job, _worker: usize) -> Outcome {
        self.jobs.lock().unwrap().push(job.clone());
        let strict = job.prefix == USE_STRICT;
        let mut prints = Vec::new();
        let mut completion = Completion::Normal;
        for line in job.test.lines() {
            let Some(d) = line.trim().strip_prefix("// FAKE: ") else {
                continue;
            };
            let words: Vec<&str> = d.split_whitespace().collect();
            let c = match words[..] {
                ["normal"] => None,
                ["throw", phase, name] => Some(throw(Phase::parse(phase).unwrap(), name, "fake")),
                ["print", ..] => {
                    prints.push(d["print ".len()..].to_string());
                    None
                }
                ["fail-if-strict"] if strict => {
                    Some(throw(Phase::Runtime, "Test262Error", "ran strict"))
                }
                ["fail-if-sloppy"] if !strict => {
                    Some(throw(Phase::Runtime, "Test262Error", "ran sloppy"))
                }
                ["fail-if-strict" | "fail-if-sloppy"] => None,
                ["require-harness", m] if !job.harness.contains(m) => {
                    Some(throw(Phase::Runtime, "ReferenceError", "harness missing"))
                }
                ["require-harness", _] => None,
                ["require-no-harness"] if !job.harness.is_empty() || !job.prefix.is_empty() => {
                    Some(throw(Phase::Runtime, "Test262Error", "source was modified"))
                }
                ["require-no-harness"] => None,
                ["require-harness-order", ..] => {
                    let at: Vec<Option<usize>> =
                        words[1..].iter().map(|m| job.harness.find(m)).collect();
                    let ordered =
                        at.iter().all(Option::is_some) && at.windows(2).all(|w| w[0] < w[1]);
                    (!ordered).then(|| throw(Phase::Runtime, "Test262Error", "harness order"))
                }
                ["hang"] => Some(Completion::Timeout),
                _ => panic!("unknown FAKE directive `{d}` in {}", job.path),
            };
            if let Some(c) = c {
                completion = c;
                break;
            }
        }
        // A module job's kind must say so; a script never is a module.
        assert_eq!(
            job.kind == SourceKind::Module,
            job.path.starts_with("module/"),
            "{}",
            job.path
        );
        Outcome {
            completion,
            prints,
            diagnostics: String::new(),
        }
    }
}
