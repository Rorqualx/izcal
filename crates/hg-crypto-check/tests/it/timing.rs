//! The timing test, and the leak it must find.
//!
//! The assertion that carries the weight is on the **plant**: a test
//! that cannot detect a short-circuiting comparison cannot detect
//! anything, and would pass over `hg-crypto` for the rest of the
//! project's life while proving nothing. So the plant must be found,
//! and only then does the constant-time side's result mean anything.

use hg_crypto_check::timing;

/// Enough bytes that a short-circuit at the first byte is thousands of
/// instructions cheaper than one at the last.
const LEN: usize = 4096;

/// Fewer rounds than the binary's own run: a test must not take a
/// minute, and the plant's margin is large enough that it does not
/// need them.
const ROUNDS: usize = 120;

/// The plant is detected. This is the test that makes the others mean
/// something.
#[test]
fn cc10_the_planted_leak_is_found() {
    let m = timing::measure(timing::leaky_eq, LEN, ROUNDS);
    assert!(
        m.leaks(),
        "the plant was not detected: early {} ns, late {} ns, noise {} ns, ratio {:.1}",
        m.early.min,
        m.late.min,
        m.noise,
        m.ratio()
    );
    // And the direction is the one a short-circuit gives: comparing
    // buffers that differ at the first byte is the cheap case.
    assert!(
        m.early.min < m.late.min,
        "early {} ns is not below late {} ns",
        m.early.min,
        m.late.min
    );
}

/// The twin: the constant-time comparison does not look
/// data-dependent.
///
/// The threshold is the same fixed one the plant is held to — making
/// it relative to the control would have been strictly weaker, and
/// with a control near 1,000 a genuine leak at ratio 5 would have
/// passed. What is removed instead is the *transient*: the verdict is
/// the majority of three runs, because a scheduling artifact on a
/// shared agent rarely survives two more measurements and a
/// data-dependent comparison leaks in every one.
#[test]
fn cc11_the_constant_time_comparison_is_not() {
    let (control, control_leaks) = timing::majority(timing::leaky_eq, LEN, ROUNDS, 3);
    assert!(
        control_leaks,
        "the control did not come out as a leak in a majority of runs, so \
         this machine discriminates nothing: ratio {:.1}",
        control.ratio()
    );
    let (m, leaks) = timing::majority(hg_crypto::ct::eq, LEN, ROUNDS, 3);
    assert!(
        !leaks,
        "ct::eq looks data-dependent: early {} ns, late {} ns, ratio {:.4}, \
         in a majority of runs",
        m.early.min,
        m.late.min,
        m.ratio()
    );
}

/// `leaky_eq` and `ct::eq` answer the same question; they differ only
/// in how long they take. A plant that was also wrong would make the
/// measurement a test of correctness instead.
#[test]
fn cc12_the_plant_is_wrong_only_about_time() {
    for (a, b) in [
        (&b""[..], &b""[..]),
        (&b"abc"[..], &b"abc"[..]),
        (&b"abc"[..], &b"abd"[..]),
        (&b"abc"[..], &b"zbc"[..]),
        (&b"abc"[..], &b"ab"[..]),
    ] {
        assert_eq!(
            timing::leaky_eq(a, b),
            hg_crypto::ct::eq(a, b),
            "{a:?} vs {b:?}"
        );
    }
}
