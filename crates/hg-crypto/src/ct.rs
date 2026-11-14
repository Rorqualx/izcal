//! Constant-time helpers.
//!
//! Every function here must take the same time whatever its secret
//! inputs are. That is a property of the compiled code, not of the
//! source, so what this module owes is a *small, marked* set of
//! functions that `hg-crypto-check` can measure and a binary taint
//! check can follow (G36, NW-50) — and nothing else in the crate may
//! branch or index on a secret.
//!
//! The technique is the usual one and it is not subtle: fold the
//! difference of every byte into one accumulator with `|`, and turn
//! the accumulator into a `bool` only at the end. There is no early
//! return, so there is no length of common prefix to measure.
//!
//! **A caveat that belongs in the source rather than in a report.**
//! Rust does not promise that these compile to branch-free code, and
//! LLVM is permitted to introduce a branch. That is precisely why
//! NW-50 is a separate gate on the *binary*, per ISA, and why §9.1
//! makes it a merge condition rather than a review item.

/// Whether two byte strings are equal, in time that depends on their
/// length and not on their contents.
///
/// Different lengths return `false` immediately — a length is public
/// in every use this crate has, and pretending otherwise would mean
/// reading past one of the two.
#[must_use]
pub fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    is_zero_u8(diff)
}

/// Whether `v` is zero, without a branch on `v`.
///
/// `(v | -v) >> 7` is 0 exactly when `v` is 0 and 1 otherwise, over
/// the wrapping negation; the result is inverted to give "is zero".
#[must_use]
pub fn is_zero_u8(v: u8) -> bool {
    let folded = v | v.wrapping_neg();
    // The top bit is 1 for every non-zero `v`.
    let top = folded >> 7;
    top == 0
}

/// `0xFF` when `choose_b` is true and `0x00` when it is false, with no
/// branch on the flag.
#[must_use]
pub fn mask(choose_b: bool) -> u8 {
    // `u8::from(bool)` is 0 or 1; the wrapping negation turns 1 into
    // 0xFF and leaves 0 alone. The conversion is what the compiler
    // lowers to a `setcc`, and it is the reason this is branch-free
    // rather than an `if`.
    u8::from(choose_b).wrapping_neg()
}

/// `b` where `choose_b` is true and `a` where it is false, byte by
/// byte, with no branch on the flag.
///
/// Returns `false` and changes nothing when the three slices are not
/// the same length, which is a caller's bug rather than a secret.
pub fn select(out: &mut [u8], a: &[u8], b: &[u8], choose_b: bool) -> bool {
    if out.len() != a.len() || a.len() != b.len() {
        return false;
    }
    let m = mask(choose_b);
    for ((o, x), y) in out.iter_mut().zip(a.iter()).zip(b.iter()) {
        *o = (x & !m) | (y & m);
    }
    true
}
