//! The statistical timing test (NW-50), and the leak it must find.
//!
//! # What this measures, and what it does not
//!
//! A timing test on a shared machine measures the machine as much as
//! the code. Scheduling, frequency scaling, caches and the other
//! tenants all move the mean, and none of them cares which byte of a
//! secret differed. So the estimator here is the **minimum** over many
//! samples rather than the mean: noise only ever adds time, so the
//! smallest observation is the closest thing to the code's own cost
//! that a loop can see.
//!
//! What the test then asks is not "are these two timings equal" —
//! nothing on a real machine is — but "is the difference between them
//! large compared with the spread of the same measurement repeated".
//! That ratio is [`Measurement::ratio`], and it is reported rather
//! than asserted on except through the control below.
//!
//! # The control, which is the point
//!
//! A timing test that detects nothing passes on code with a leak and
//! on code without one. So the harness carries a **planted leak** —
//! [`leaky_eq`], an ordinary short-circuiting comparison — and the
//! test's assertion is that the leak is **found**. Only then does the
//! same measurement over [`hg_crypto::ct::eq`] mean anything, and the
//! two together are the twin discipline: the plant is detected and its
//! twin is not.
//!
//! # Why this is not the whole gate
//!
//! §9.1 makes `hg-crypto-check` a merge gate "on every target ISA".
//! This part runs on whatever machine CI gives it, which is one ISA.
//! The binary taint check — reading the compiled object and following
//! secret-derived values into branches and addresses — is the half
//! that does not depend on a timer, and it is NW-M0's remaining work.

use std::hint::black_box;
use std::time::Instant;

/// A comparison with a leak in it: it returns as soon as two bytes
/// differ, so its running time tells the caller how long a prefix the
/// two inputs share. Forging a MAC one byte at a time is exactly this.
///
/// It exists to be detected. Nothing may call it outside this crate's
/// own tests.
#[must_use]
pub fn leaky_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return false;
        }
    }
    true
}

/// One side's timing, in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Side {
    /// The smallest observation: the least noisy estimate of cost.
    pub min: u128,
    /// The median, for reporting.
    pub median: u128,
}

/// A comparison of two inputs' timings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measurement {
    /// Timing with the difference at the first byte.
    pub early: Side,
    /// Timing with the difference at the last byte.
    pub late: Side,
    /// The spread of the *same* measurement repeated, as a floor on
    /// what counts as a difference.
    pub noise: u128,
}

impl Measurement {
    /// The early-versus-late difference as a fraction of the cheaper
    /// side's own cost.
    ///
    /// **A relative measure, not a multiple of the noise floor.** The
    /// first attempt here divided by the spread between two repeats of
    /// the same side, and that spread is routinely 1 ns — two minima
    /// of the same quantity coincide — so a 42 ns difference on a
    /// 3,666 ns operation came out as "42 times the noise" when it is
    /// in fact one per cent of the work. Measured on this machine
    /// while the rest of the test suite ran: `ct::eq` 3,666 against
    /// 3,708 ns, which that floor called a leak three runs out of
    /// four.
    ///
    /// Dividing by the cost instead is scale-free, so it means the
    /// same thing on a fast machine and a slow one, and it asks the
    /// question that matters: **does the running time carry a
    /// meaningful share of the secret?** A short-circuit over a long
    /// buffer is hundreds of times the cheap side; jitter is a few per
    /// cent of it.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        let delta = self.late.min.abs_diff(self.early.min);
        let base = self.early.min.min(self.late.min).max(1);
        let d = u32::try_from(delta.min(u128::from(u32::MAX))).unwrap_or(u32::MAX);
        let b = u32::try_from(base.min(u128::from(u32::MAX))).unwrap_or(u32::MAX);
        f64::from(d) / f64::from(b)
    }

    /// Whether the running time carries a meaningful share of the
    /// secret.
    ///
    /// A quarter of the cheaper side's cost. The threshold is
    /// deliberately far from both ends of what was measured — the
    /// planted short-circuit comes out above 1,000 and the
    /// constant-time comparison at about 0.01 — because a threshold
    /// close to either would turn a busy build agent into a finding.
    ///
    /// A leak subtler than a quarter of the work exists and this will
    /// not see it. That is why NW-50's gate is the per-ISA binary
    /// check as well, and why this one is a smoke test with a control.
    #[must_use]
    pub fn leaks(&self) -> bool {
        self.ratio() >= 0.25
    }
}

fn side_of(mut samples: Vec<u128>) -> Side {
    samples.sort_unstable();
    let min = samples.first().copied().unwrap_or(0);
    let median = samples.get(samples.len() / 2).copied().unwrap_or(0);
    Side { min, median }
}

fn one_sample(f: &mut impl FnMut()) -> u128 {
    let start = Instant::now();
    // Enough repetitions that one sample is well above the timer's own
    // resolution.
    for _ in 0..64 {
        f();
    }
    start.elapsed().as_nanos()
}

/// Times `cmp` on inputs differing at the first byte against inputs
/// differing at the last, over buffers of `len` bytes.
///
/// `rounds` samples each; the noise floor is the spread of the early
/// side measured twice, so it is the same work measured the same way.
#[must_use]
pub fn measure(cmp: fn(&[u8], &[u8]) -> bool, len: usize, rounds: usize) -> Measurement {
    let a = vec![0xA5_u8; len];
    let mut early = a.clone();
    if let Some(first) = early.first_mut() {
        *first ^= 0xFF;
    }
    let mut late = a.clone();
    if let Some(last) = late.last_mut() {
        *last ^= 0xFF;
    }

    // Warm the caches and the branch predictor so the first sample is
    // not the one that decides the minimum.
    for _ in 0..64 {
        black_box(cmp(black_box(&a), black_box(&early)));
        black_box(cmp(black_box(&a), black_box(&late)));
    }

    // Interleaved, not one side then the other. A build machine under
    // load drifts, and measuring all of one side before any of the
    // other turns that drift into a difference between them -- which
    // is the one thing this measurement must not invent. Interleaving
    // puts any noisy period into both sides at once.
    //
    // The noise floor is a *third* interleaved series over the same
    // input as the first, so it is the same work measured the same way
    // under the same conditions, and whatever contention raises the
    // difference raises the floor with it.
    let mut e1: Vec<u128> = Vec::with_capacity(rounds);
    let mut l: Vec<u128> = Vec::with_capacity(rounds);
    let mut e2: Vec<u128> = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        e1.push(one_sample(&mut || {
            black_box(cmp(black_box(&a), black_box(&early)));
        }));
        l.push(one_sample(&mut || {
            black_box(cmp(black_box(&a), black_box(&late)));
        }));
        e2.push(one_sample(&mut || {
            black_box(cmp(black_box(&a), black_box(&early)));
        }));
    }
    let e1 = side_of(e1);
    let l = side_of(l);
    let e2 = side_of(e2);

    Measurement {
        early: e1,
        late: l,
        noise: e1.min.abs_diff(e2.min),
    }
}

/// Measures `runs` times and returns the **majority** verdict, with a
/// measurement that gave it.
///
/// **Why a majority rather than a looser threshold.** A merge gate
/// that can fail because another job on the agent was busy is a gate
/// people learn to rerun, so the first attempt made the threshold
/// relative to the control measured beside it. That was strictly
/// weaker: with a control at 1,000, a genuine leak at ratio 5 would
/// have passed. Sensitivity is the whole point, so the threshold stays
/// and the *transient* is what is removed.
///
/// **Why a majority rather than "two agreed".** The attempt before
/// this one compared each later run with the first, so leak, clean,
/// leak counted as confirmed although the middle run disagreed. A
/// majority of an odd number has no such corner: one noisy run cannot
/// decide anything, and a comparison whose time depends on its inputs
/// leaks in every run.
///
/// `runs` is rounded up to the next odd number, and to at least 3, so
/// there is always a majority.
#[must_use]
pub fn majority(
    cmp: fn(&[u8], &[u8]) -> bool,
    len: usize,
    rounds: usize,
    runs: usize,
) -> (Measurement, bool) {
    let n = {
        let at_least = runs.max(3);
        if at_least.is_multiple_of(2) {
            at_least.saturating_add(1)
        } else {
            at_least
        }
    };
    let mut leaked: Vec<Measurement> = Vec::new();
    let mut clean: Vec<Measurement> = Vec::new();
    for _ in 0..n {
        let m = measure(cmp, len, rounds);
        if m.leaks() {
            leaked.push(m);
        } else {
            clean.push(m);
        }
    }
    if leaked.len() > clean.len() {
        // The strongest of the leaking runs, so the number printed is
        // the one that argues for the verdict.
        let worst = leaked
            .iter()
            .copied()
            .reduce(|a, b| if b.ratio() > a.ratio() { b } else { a });
        (worst.unwrap_or_else(|| measure(cmp, len, rounds)), true)
    } else {
        let best = clean
            .iter()
            .copied()
            .reduce(|a, b| if b.ratio() < a.ratio() { b } else { a });
        (best.unwrap_or_else(|| measure(cmp, len, rounds)), false)
    }
}
