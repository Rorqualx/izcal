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
//!
//! # The third state: a clause attempted and refused
//!
//! `claimed` is two-state, so a clause that was *tried* and found not to
//! hold leaves no trace, and the next session re-attempts it and
//! re-derives the same refusal. A `[[refuted]]` row records that attempt:
//! the track, the clause, the date, the evidence and why.
//!
//! A refusal is a **record, never an exemption**, and two findings keep it
//! that way:
//!
//! - **It cannot excuse a claim.** A track that is `claimed = true` with a
//!   refuted clause is a contradiction — it says both that the track has
//!   started and that one of the five clauses of starting was refused —
//!   and that is a finding. Without this rule, `refuted` would be a way to
//!   weaken clause 4 of M1's exit one clause at a time.
//! - **It cannot rot into a false claim.** If a refuted clause now holds,
//!   the record is stale, and that is a finding too. A refusal must stay
//!   checkable against the tree, exactly as a claim must; the only way to
//!   remove it is deliberately, in the gate file.
//!
//! So the three states are: claimed and checked, refused and checked, or
//! silent. What the third state buys is that the second is no longer
//! written as the third.

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
    /// Clause numbers recorded as attempted and refused, with the date.
    pub refuted: Vec<(u8, String)>,
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

            let refuted: Vec<(u8, String)> = inputs
                .rules
                .refuted
                .iter()
                .filter(|r| r.track == t.track)
                .map(|r| (r.clause, r.date.clone()))
                .collect();

            TrackStatus {
                track: t.track.clone(),
                claimed: t.claimed,
                refuted,
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

/// Adds a finding for every claimed track whose clauses do not hold, and
/// for every `[[refuted]]` record that has stopped being true.
pub fn check(inputs: &Inputs, report: &mut Report) {
    for t in status(inputs) {
        for &(clause, ref date) in &t.refuted {
            // A refusal cannot excuse a claim: claiming to have started
            // while recording that a clause of starting was refused says
            // both things at once.
            if t.claimed {
                report.add(
                    "started",
                    &t.track,
                    format!(
                        "claims to have started and records clause {clause} as refused on \
                         {date}; a refusal is a record, not an exemption, so one of the two is \
                         wrong (Stage 1 design 8.1a)"
                    ),
                    None,
                );
            }
            // A refusal cannot rot into a false claim: if the clause holds
            // now, the record is stale and must be removed deliberately.
            if t.clauses.iter().any(|c| c.number == clause && c.ok) {
                report.add(
                    "started",
                    &t.track,
                    format!(
                        "records clause {clause} as refused on {date}, but it holds against the \
                         tree now; a stale refusal is removed in the gate file, not left to \
                         contradict the artefacts"
                    ),
                    None,
                );
            }
        }
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
        for &(clause, ref date) in &t.refuted {
            let _ = writeln!(out, "         {clause}. refused {date}");
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
