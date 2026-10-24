//! UTF-8 validation for [`BoundedString`](crate::BoundedString), written
//! from the Unicode Standard's definition of well-formed UTF-8 (The
//! Unicode Standard, Chapter 3, D92 and Table 3-7; RFC 3629 Section 4).
//! Inventoried unsafe (G5, CS-TY-13): `unsafe-inventory.toml`, entry
//! `utf8`.
//!
//! Why not `core::str::from_utf8`: it is correct, but Kani's cost for it
//! grows much faster than the input. Measured on Kani 0.68 / CBMC 6.11,
//! for decoding a string of at most 12 bytes and checking the round
//! trip: 202 s and 3.1 GB with `from_utf8`, 58 s and 2.1 GB with this
//! validator, and building the `String` one `char` at a time (the safe
//! way to avoid `from_utf8`) did not finish in 10 minutes even at 4
//! bytes. So this module validates, then copies the bytes once with
//! `from_utf8_unchecked`. That call's precondition is exactly what
//! [`validate`] checks, and two things tie [`validate`] to Rust's own
//! definition: the `validate_agrees_with_core` Kani harness (every input
//! of at most 4 bytes, so every single scalar value and every
//! ill-formed prefix) and the `agrees_with_core_on_short_inputs` test.
//! Longer inputs follow because well-formedness is defined one sequence
//! at a time (D92), and [`validate`] reads one sequence per step.
//!
//! Table 3-7, the only well-formed byte sequences:
//!
//! | Scalar values      | Byte 1 | Byte 2 | Byte 3 | Byte 4 |
//! |--------------------|--------|--------|--------|--------|
//! | U+0000..U+007F     | 00..7F |        |        |        |
//! | U+0080..U+07FF     | C2..DF | 80..BF |        |        |
//! | U+0800..U+0FFF     | E0     | A0..BF | 80..BF |        |
//! | U+1000..U+CFFF     | E1..EC | 80..BF | 80..BF |        |
//! | U+D000..U+D7FF     | ED     | 80..9F | 80..BF |        |
//! | U+E000..U+FFFF     | EE..EF | 80..BF | 80..BF |        |
//! | U+10000..U+3FFFF   | F0     | 90..BF | 80..BF | 80..BF |
//! | U+40000..U+FFFFF   | F1..F3 | 80..BF | 80..BF | 80..BF |
//! | U+100000..U+10FFFF | F4     | 80..8F | 80..BF | 80..BF |

use crate::wire::BadMessage;

/// Whether `src[at]` is a continuation byte in `lo..=hi`.
fn cont(src: &[u8], at: Option<usize>, lo: u8, hi: u8) -> Result<(), BadMessage> {
    match at.and_then(|i| src.get(i)) {
        Some(&b) if (lo..=hi).contains(&b) => Ok(()),
        _ => Err(BadMessage::BadUtf8),
    }
}

/// The length of the well-formed sequence starting at `src[at]`.
fn sequence(src: &[u8], at: usize) -> Result<usize, BadMessage> {
    let Some(&b0) = src.get(at) else {
        return Err(BadMessage::BadUtf8);
    };
    let next = |k: usize| at.checked_add(k);
    // (length, the second byte's range) per Table 3-7's rows.
    let (len, lo, hi) = match b0 {
        0x00..=0x7F => return Ok(1),
        0xC2..=0xDF => (2, 0x80, 0xBF),
        0xE0 => (3, 0xA0, 0xBF),
        0xE1..=0xEC | 0xEE..=0xEF => (3, 0x80, 0xBF),
        0xED => (3, 0x80, 0x9F),
        0xF0 => (4, 0x90, 0xBF),
        0xF1..=0xF3 => (4, 0x80, 0xBF),
        0xF4 => (4, 0x80, 0x8F),
        // 80..C1 and F5..FF never start a well-formed sequence.
        _ => return Err(BadMessage::BadUtf8),
    };
    cont(src, next(1), lo, hi)?;
    if len >= 3 {
        cont(src, next(2), 0x80, 0xBF)?;
    }
    if len == 4 {
        cont(src, next(3), 0x80, 0xBF)?;
    }
    Ok(len)
}

/// Checks that `src`, at most `N` bytes, is well-formed UTF-8.
///
/// The loop runs at most `N` times (each sequence is at least one
/// byte), so Kani unrolls it to the type's maximum, not the harness's.
///
/// # Errors
/// [`BadMessage::BadUtf8`] unless `src` is well-formed;
/// [`BadMessage::OverBound`] if it is longer than `N`.
pub(crate) fn validate<const N: usize>(src: &[u8]) -> Result<(), BadMessage> {
    let mut at = 0usize;
    for _ in 0..N {
        if at >= src.len() {
            return Ok(());
        }
        at = at
            .checked_add(sequence(src, at)?)
            .ok_or(BadMessage::BadUtf8)?;
    }
    if at >= src.len() {
        Ok(())
    } else {
        // More than N bytes: the caller's count check already refused it.
        Err(BadMessage::OverBound)
    }
}

/// `src` as a `String`, if it is well-formed UTF-8 of at most `N` bytes.
///
/// # Errors
/// As [`validate`], or [`BadMessage::OutOfMemory`].
pub(crate) fn to_string<const N: usize>(src: &[u8]) -> Result<String, BadMessage> {
    validate::<N>(src)?;
    let mut s = String::new();
    s.try_reserve_exact(src.len())
        .map_err(|_alloc| BadMessage::OutOfMemory)?;
    s.push_str(as_str_validated(src));
    Ok(s)
}

/// `src` as a `&str`. Call only on bytes [`validate`] accepted.
///
/// What would break it: a change to [`validate`] that accepted any
/// sequence outside Table 3-7, or a caller that skipped [`validate`].
#[expect(unsafe_code, reason = "G5: utf8 (from_utf8_unchecked after validate)")]
fn as_str_validated(src: &[u8]) -> &str {
    // SAFETY: needs well-formed UTF-8; the only caller, `to_string`, passes only bytes
    // `validate` accepted, and `validate` accepts exactly Table 3-7's sequences (Kani
    // `validate_agrees_with_core`; test `agrees_with_core_on_short_inputs`).
    unsafe { core::str::from_utf8_unchecked(src) }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    /// The unsafe block's precondition (CS-TY-13): on every input of at
    /// most 4 bytes, `validate` accepts exactly what `core::str::from_utf8`
    /// accepts. Four bytes covers every scalar value's encoding and every
    /// ill-formed prefix of one; see the module docs for longer inputs.
    #[kani::proof]
    #[kani::unwind(6)]
    fn validate_agrees_with_core() {
        let bytes: [u8; 4] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 4);
        let Some(src) = bytes.get(..len) else { return };
        assert!(validate::<4>(src).is_ok() == core::str::from_utf8(src).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ours(bytes: &[u8]) -> Option<String> {
        to_string::<4>(bytes).ok()
    }

    /// Agrees with `core::str::from_utf8` on every 1-, 2- and 3-byte
    /// input, and on every 4-byte input whose last two bytes are taken
    /// from each range boundary in Table 3-7.
    #[test]
    fn agrees_with_core_on_short_inputs() {
        let check = |b: &[u8]| {
            let theirs = core::str::from_utf8(b).ok().map(str::to_owned);
            assert_eq!(ours(b), theirs, "input {b:02x?}");
        };
        for a in 0..=255u8 {
            check(&[a]);
            for b in 0..=255u8 {
                check(&[a, b]);
                for c in 0..=255u8 {
                    check(&[a, b, c]);
                }
            }
        }
        let edges = [
            0x00, 0x7F, 0x80, 0x8F, 0x90, 0x9F, 0xA0, 0xBF, 0xC0, 0xC1, 0xC2, 0xF4, 0xF5, 0xFF,
        ];
        for a in 0..=255u8 {
            for b in 0..=255u8 {
                for &c in &edges {
                    for &d in &edges {
                        check(&[a, b, c, d]);
                    }
                }
            }
        }
    }

    #[test]
    fn accepts_each_length_and_refuses_over_n() {
        assert_eq!(ours("a\u{e9}".as_bytes()).as_deref(), Some("a\u{e9}"));
        assert_eq!(ours("\u{20ac}".as_bytes()).as_deref(), Some("\u{20ac}"));
        assert_eq!(ours("\u{1f600}".as_bytes()).as_deref(), Some("\u{1f600}"));
        assert_eq!(validate::<2>(b"abc"), Err(BadMessage::OverBound));
    }
}
