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

/// The twin: the constant-time comparison is not distinguishable from
/// its own noise.
///
/// This is reported with its numbers when it fails, because a failure
/// here is as likely to be the machine as the code, and NW-50's real
/// gate is the per-ISA binary check rather than a timer on a build
/// agent.
#[test]
fn cc11_the_constant_time_comparison_is_not() {
    let m = timing::measure(hg_crypto::ct::eq, LEN, ROUNDS);
    assert!(
        !m.leaks(),
        "ct::eq looks data-dependent: early {} ns, late {} ns, noise {} ns, ratio {:.1}",
        m.early.min,
        m.late.min,
        m.noise,
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
