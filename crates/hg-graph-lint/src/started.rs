//! "Started", the M1 row's five M0 clauses (Stage 1 design 8.1a).
//!
//! M1's exit requires that ST-M0, TX-M0, LY-M0, GR-M0 and NW-M0 *start*.
//! Section 8.1a defines starting as five clauses, each an artefact in the
//! tree rather than a claim about one. This module checks the four it can
//! see from the workspace and the repository:
//!
//! 1. **Crates.** Every crate the track's M0 row names is a workspace
//!    member with a row in `tools/tiers.toml`. (The tier and layer those
//!    rows give are checked by the `tiers` and `layers` rules already;
//!    this clause is only that they exist.)
//! 2. **Generator.** The crate the rest of the track generates from or
//!    builds on is among them and is a member.
//! 3. **Oracle.** The named job exists in `.github/workflows/checks.yml`.
//!    The job's *result* is CI's to report; this checks only that the
//!    track cannot claim an oracle it never wired up.
//! 4. **Review** and **5. interlock**: the two documents exist.
//!
//! # Why a claim, and not a gate
//!
//! A track is checked only when its row says `claimed = true`. An
//! unclaimed track is reported and never failed, because the rule would
//! otherwise fail every pull request from the day it lands until the last
//! track is finished — it would block the work that satisfies it. What
//! the rule does buy is that a track cannot be *said* to have started
//! until the artefacts are there: flipping `claimed` is a gate-file
//! change (CS-GA-1), and it fails unless every clause holds.
//!
//! This is the same shape as the rest of `graph-rules.toml`, whose header
//! notes that "a rule whose crates do not exist yet costs nothing and
//! binds the moment they do".

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::{Inputs, Report};

/// How one clause stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    /// Which of 8.1a's five.
    pub number: u8,
    /// Its one-word name.
    pub name: &'static str,
    /// Whether it holds.
    pub ok: bool,
    /// What is missing, when it does not.
    pub missing: Vec<String>,
}

/// One track's five clauses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackStatus {
    /// The milestone.
    pub track: String,
    /// Whether the gate file claims it has started.
    pub claimed: bool,
    /// The five clauses, in order.
    pub clauses: Vec<Clause>,
}

impl TrackStatus {
    /// Whether every clause holds.
    #[must_use]
    pub fn started(&self) -> bool {
        self.clauses.iter().all(|c| c.ok)
    }

    /// How many hold.
    #[must_use]
    pub fn met(&self) -> usize {
        self.clauses.iter().filter(|c| c.ok).count()
    }
}

/// Reads one repository file, returning its text when it is there.
fn read(inputs: &Inputs, rel: &str) -> Option<String> {
    std::fs::read_to_string(inputs.root.join(rel)).ok()
}

/// Whether `.github/workflows/checks.yml` defines a job called `job`.
///
/// A text scan, not a YAML parse: jobs sit at one indent level under
/// `jobs:`, which is how `tools/checks.sh`'s `kanipkgs` check reads the
/// same file. The scan starts at `jobs:` so that a key of `on:` — which
/// sits at the same indent — cannot be mistaken for a job.
fn has_job(workflow: &str, job: &str) -> bool {
    let want = format!("  {job}:");
    workflow
        .lines()
        .skip_while(|l| l.trim_end() != "jobs:")
        .skip(1)
        .any(|l| l.trim_end() == want)
}

/// Works out where every track stands.
#[must_use]
pub fn status(inputs: &Inputs) -> Vec<TrackStatus> {
    let members: BTreeSet<&str> = inputs.graphs.first().map_or_else(BTreeSet::new, |g| {
        g.members()
            .into_iter()
            .map(|i| g.packages[i].name.as_str())
            .collect()
    });
    let workflow = read(inputs, ".github/workflows/checks.yml").unwrap_or_default();

    inputs
        .rules
        .started
        .iter()
        .map(|t| {
            let missing_crates: Vec<String> = t
                .crates
                .iter()
                .filter(|c| !members.contains(c.as_str()))
                .cloned()
                .collect();
            let no_row: Vec<String> = t
                .crates
                .iter()
                .filter(|c| members.contains(c.as_str()) && !inputs.tiers.crates.contains_key(*c))
                .map(|c| format!("{c} (no tools/tiers.toml row)"))
                .collect();
            let mut clause1 = missing_crates.clone();
            clause1.extend(no_row);

            let generator_ok = members.contains(t.generator.as_str());
            let oracle_ok = has_job(&workflow, &t.oracle_job);
            let review_ok = read(inputs, &t.review).is_some();
            let interlock_ok = read(inputs, &t.interlock).is_some();

            TrackStatus {
                track: t.track.clone(),
                claimed: t.claimed,
                clauses: vec![
                    Clause {
                        number: 1,
                        name: "crates",
                        ok: clause1.is_empty(),
                        missing: clause1,
                    },
                    Clause {
                        number: 2,
                        name: "generator",
                        ok: generator_ok,
                        missing: if generator_ok {
                            Vec::new()
                        } else {
                            vec![t.generator.clone()]
                        },
                    },
                    Clause {
                        number: 3,
                        name: "oracle",
                        ok: oracle_ok,
                        missing: if oracle_ok {
                            Vec::new()
                        } else {
                            vec![format!(
                                "no job `{}` in .github/workflows/checks.yml",
                                t.oracle_job
                            )]
                        },
                    },
                    Clause {
                        number: 4,
                        name: "review",
                        ok: review_ok,
                        missing: if review_ok {
                            Vec::new()
                        } else {
                            vec![t.review.clone()]
                        },
                    },
                    Clause {
                        number: 5,
                        name: "interlock",
                        ok: interlock_ok,
                        missing: if interlock_ok {
                            Vec::new()
                        } else {
                            vec![t.interlock.clone()]
                        },
                    },
                ],
            }
        })
        .collect()
}

/// Adds a finding for every claimed track whose clauses do not hold.
pub fn check(inputs: &Inputs, report: &mut Report) {
    for t in status(inputs) {
        if !t.claimed {
            continue;
        }
        for c in t.clauses.iter().filter(|c| !c.ok) {
            report.add(
                "started",
                &t.track,
                format!(
                    "claims to have started, but clause {} ({}) of Stage 1 design 8.1a does not \
                     hold: {}",
                    c.number,
                    c.name,
                    c.missing.join(", ")
                ),
                None,
            );
        }
    }
}

/// The five-by-five matrix, for the milestone evidence to quote.
#[must_use]
pub fn render(inputs: &Inputs) -> String {
    let mut out = String::new();
    let all = status(inputs);
    for t in &all {
        let marks: String = t
            .clauses
            .iter()
            .map(|c| if c.ok { '+' } else { '-' })
            .collect();
        let _ = writeln!(
            out,
            "{:<6} {marks}  {}/5  {}",
            t.track,
            t.met(),
            if t.claimed { "claimed" } else { "not claimed" }
        );
        for c in t.clauses.iter().filter(|c| !c.ok) {
            let _ = writeln!(
                out,
                "         {}. {}: {}",
                c.number,
                c.name,
                c.missing.join(", ")
            );
        }
    }
    let started = all.iter().filter(|t| t.started()).count();
    let _ = writeln!(
        out,
        "started: {started}/{} (clauses: 1 crates, 2 generator, 3 oracle, 4 review, 5 interlock)",
        all.len()
    );
    out
}
