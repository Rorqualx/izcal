//! Running the vectors `hg-crypto` has a primitive for, and
//! classifying the rest.
//!
//! The report has two halves and the second is the one that matters at
//! NW-M0. Izcal implements three primitives today, so a pass rate over
//! the whole set would be a number near zero that says nothing. What
//! the runner reports instead is, per algorithm: how many cases exist,
//! whether Izcal has the primitive, and — for the ones it has — how
//! many pass. A primitive that lands later turns a row from
//! "unimplemented" into a count, and the rows that are already counts
//! can only go up.
//!
//! An `acceptable` vector is counted apart from a pass and a failure.
//! Wycheproof uses it for legacy behaviour where either answer
//! conforms, so counting it either way would make the number a
//! statement about Izcal's policy rather than its correctness.

use std::collections::BTreeMap;

use hg_crypto::hash::Sha256;
use hg_crypto::hkdf::{HkdfSha256, Kdf};
use hg_crypto::hmac::{HmacSha256, Mac};

use crate::vectors::{Suite, Verdict};

/// How one algorithm's vectors came out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Cases that behaved as the vector says.
    pub pass: usize,
    /// Cases that did not.
    pub fail: usize,
    /// Cases marked `acceptable`, counted apart.
    pub acceptable: usize,
    /// Cases the runner has no primitive for.
    pub unimplemented: usize,
    /// The first few failures, as `tcId: comment`.
    pub failures: Vec<String>,
}

impl Outcome {
    /// How many cases it covers.
    #[must_use]
    pub fn total(&self) -> usize {
        self.pass
            .saturating_add(self.fail)
            .saturating_add(self.acceptable)
            .saturating_add(self.unimplemented)
    }

    fn note(&mut self, ok: bool, tc_id: i64, comment: &str) {
        if ok {
            self.pass = self.pass.saturating_add(1);
        } else {
            self.fail = self.fail.saturating_add(1);
            if self.failures.len() < 8 {
                self.failures.push(format!("{tc_id}: {comment}"));
            }
        }
    }
}

/// The whole run.
#[derive(Debug, Clone, Default)]
pub struct Summary {
    /// Per algorithm, as Wycheproof names it.
    pub by_algorithm: BTreeMap<String, Outcome>,
    /// Files whose declared count disagreed with the cases read, which
    /// would mean the reader is dropping cases silently.
    pub miscounted: Vec<String>,
}

impl Summary {
    /// Every case, over every algorithm.
    #[must_use]
    pub fn total(&self) -> usize {
        self.by_algorithm.values().map(Outcome::total).sum()
    }

    /// Cases that ran and passed.
    #[must_use]
    pub fn passed(&self) -> usize {
        self.by_algorithm.values().map(|o| o.pass).sum()
    }

    /// Cases that ran and failed.
    #[must_use]
    pub fn failed(&self) -> usize {
        self.by_algorithm.values().map(|o| o.fail).sum()
    }

    /// Cases with no primitive behind them yet.
    #[must_use]
    pub fn unimplemented(&self) -> usize {
        self.by_algorithm.values().map(|o| o.unimplemented).sum()
    }

    /// Algorithms with at least one case that ran.
    #[must_use]
    pub fn implemented_algorithms(&self) -> Vec<&str> {
        self.by_algorithm
            .iter()
            .filter(|(_, o)| o.pass.saturating_add(o.fail) > 0)
            .map(|(a, _)| a.as_str())
            .collect()
    }
}

/// Runs one suite, adding to `summary`.
pub fn run_suite(suite: &Suite, summary: &mut Summary) {
    if usize::try_from(suite.declared).unwrap_or(usize::MAX) != suite.case_count() {
        summary.miscounted.push(format!(
            "{}: declares {} and holds {}",
            suite.name,
            suite.declared,
            suite.case_count()
        ));
    }
    let out = summary
        .by_algorithm
        .entry(suite.algorithm.clone())
        .or_default();
    for group in &suite.groups {
        for case in &group.cases {
            if case.result == Verdict::Acceptable {
                out.acceptable = out.acceptable.saturating_add(1);
                continue;
            }
            let ok = match suite.algorithm.as_str() {
                "HMACSHA256" => mac_case(case, group),
                "HKDF-SHA-256" => hkdf_case(case),
                _ => {
                    out.unimplemented = out.unimplemented.saturating_add(1);
                    continue;
                }
            };
            match ok {
                Some(agreed) => out.note(agreed, case.tc_id, &case.comment),
                None => {
                    // A vector whose own fields the runner cannot read
                    // is a failure of the runner, not of the primitive,
                    // and is reported as one.
                    out.note(false, case.tc_id, "the runner could not read this vector");
                }
            }
        }
    }
}

/// A `MacTest` case: the tag is checked, not recomputed, because that
/// is the operation an attacker reaches.
fn mac_case(case: &crate::vectors::Case, group: &crate::vectors::Group) -> Option<bool> {
    let key = case.hex("key")?;
    let msg = case.hex("msg")?;
    let tag = case.hex("tag")?;
    // The group's `tagSize` is in bits, and Wycheproof truncates the
    // tag to it. A verifier that compared only the prefix would pass
    // every truncated vector while accepting a forged full-length tag,
    // so the tag's own length is what is compared.
    let bits = group.fields.get("tagSize")?.as_i64()?;
    let want_len = usize::try_from(bits).ok()?.checked_div(8)?;
    if tag.len() != want_len {
        return Some(case.result == Verdict::Invalid);
    }
    let full = HmacSha256::tag(&key, &msg);
    let truncated = full.as_ref().get(..tag.len())?;
    let agrees = hg_crypto::ct::eq(truncated, &tag);
    Some(agrees == (case.result == Verdict::Valid))
}

/// An `HkdfTest` case: extract then expand, against the published
/// output keying material.
fn hkdf_case(case: &crate::vectors::Case) -> Option<bool> {
    let ikm = case.hex("ikm")?;
    let salt = case.hex("salt")?;
    let info = case.hex("info")?;
    let size = usize::try_from(case.int("size")?).ok()?;
    let okm = case.hex("okm")?;

    let mut prk = [0_u8; 32];
    if !HkdfSha256::extract(&salt, &ikm, &mut prk) {
        return Some(case.result != Verdict::Valid);
    }
    let mut got = vec![0_u8; size];
    if !HkdfSha256::expand(&prk, &info, &mut got) {
        // Refused: correct exactly when the vector says invalid.
        return Some(case.result == Verdict::Invalid);
    }
    let agrees = got == okm;
    Some(agrees == (case.result == Verdict::Valid))
}

/// The hash the runner's own primitives rest on, named so the report
/// can say which it used.
#[must_use]
pub fn hash_name() -> &'static str {
    let _ = core::marker::PhantomData::<Sha256>;
    "SHA-256"
}
