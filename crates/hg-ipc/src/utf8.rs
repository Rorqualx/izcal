//! UTF-8 validation for [`BoundedString`](crate::BoundedString), written
//! from the Unicode Standard's definition of well-formed UTF-8 (The
//! Unicode Standard, Chapter 3, D92 and Table 3-7; RFC 3629 Section 4).
//! Inventoried unsafe (G5, CS-TY-13): `unsafe-inventory.toml`, entry
//! `utf8`.
//!
//! Why not `core::str::from_utf8`: it is correct, but Kani's cost for it
//! grows much faster than the input. Measured on Kani 0.68 / CBMC 6.11,
//! for decoding a string of at most 12 bytes and checking the round
//! trip: 202 s and 3.1 GB with `from_utf8`, 58 s and 2.1 GB with the
//! first version of this validator, and building the `String` one `char` at a time (the safe
//! way to avoid `from_utf8`) did not finish in 10 minutes even at 4
//! bytes. So this module validates, then copies the bytes once with
//! `from_utf8_unchecked`. That call's precondition is exactly what
//! [`validate`] checks, and these tie [`validate`] to Rust's own
//! definition: the `validate_agrees_with_core` Kani harnesses (every
//! input of at most 4 bytes, so every single scalar value and every
//! ill-formed prefix, and every input of at most 6) and the
//! `agrees_with_core_on_short_inputs` test (every input of at most 3).
//! The validator itself is safe code.
//! Longer inputs follow because well-formedness is defined one sequence
//! at a time (D92), and [`validate`]'s state between bytes is only "a
//! sequence may start here" or "this many continuation bytes are owed,
//! the next in this range": after a sequence it is back to the first,
//! as at the start. The `matches_the_standard_on_random_inputs` test
//! checks 10 million longer inputs against `from_utf8`.
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

/// What a lead byte starts, per Table 3-7's rows: how many continuation
/// bytes follow, and the first one's range (the rest are 80..BF).
/// `None` for 80..C1 and F5..FF, which never start a well-formed sequence.
const fn lead(b: u8) -> Option<(u8, u8, u8)> {
    Some(match b {
        0x00..=0x7F => (0, 0x80, 0xBF),
        0xC2..=0xDF => (1, 0x80, 0xBF),
        0xE0 => (2, 0xA0, 0xBF),
        0xE1..=0xEC | 0xEE..=0xEF => (2, 0x80, 0xBF),
        0xED => (2, 0x80, 0x9F),
        0xF0 => (3, 0x90, 0xBF),
        0xF1..=0xF3 => (3, 0x80, 0xBF),
        0xF4 => (3, 0x80, 0x8F),
        _ => return None,
    })
}

/// Checks that `src`, at most `N` sequences, is well-formed UTF-8.
///
/// One step per byte, each reading the next byte at a known offset: a
/// lead byte sets how many continuation bytes must follow and the first
/// one's range, and each continuation byte is checked against it. Kani's
/// cost for this grows with the input, where reading a whole sequence at
/// a symbolic offset per step did not (a validator with two strings on
/// 62 bytes: out of memory at 10 GiB before, 181 s and 1.7 GiB now).
///
/// # Errors
/// [`BadMessage::BadUtf8`] unless `src` is well-formed;
/// [`BadMessage::OverBound`] if it holds more than `N` sequences (the
/// caller's count check, at most `N` bytes, already refused that).
pub(crate) fn validate<const N: usize>(src: &[u8]) -> Result<(), BadMessage> {
    // Continuation bytes still owed, and the next one's range.
    let (mut owed, mut lo, mut hi) = (0u8, 0x80u8, 0xBFu8);
    let mut sequences = 0usize;
    for &b in src {
        if owed > 0 {
            if !(lo..=hi).contains(&b) {
                return Err(BadMessage::BadUtf8);
            }
            owed = owed.saturating_sub(1);
            (lo, hi) = (0x80, 0xBF);
        } else {
            if sequences >= N {
                return Err(BadMessage::OverBound);
            }
            sequences = sequences.saturating_add(1);
            (owed, lo, hi) = lead(b).ok_or(BadMessage::BadUtf8)?;
        }
    }
    if owed == 0 {
        Ok(())
    } else {
        // The input ended inside a sequence.
        Err(BadMessage::BadUtf8)
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
    // `validate_agrees_with_core` to 4 bytes and `validate_agrees_with_core_on_six_bytes`;
    // tests `agrees_with_core_on_short_inputs`, every input to 3 bytes, and
    // `matches_the_standard_on_random_inputs`, 10 million longer ones).
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

    /// As [`validate_agrees_with_core`], on every input of at most 6
    /// bytes: two sequences of any lengths that fit, or one sequence
    /// followed by an ill-formed or truncated one.
    #[kani::proof]
    #[kani::unwind(8)]
    fn validate_agrees_with_core_on_six_bytes() {
        let bytes: [u8; 6] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 6);
        let Some(src) = bytes.get(..len) else { return };
        assert!(validate::<6>(src).is_ok() == core::str::from_utf8(src).is_ok());
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

    /// Throughput of [`validate`] on 1 MiB of ASCII, mixed and CJK text,
    /// in a release build: `cargo test --release -p hg-ipc --lib
    /// measure_validate_throughput -- --ignored --nocapture`.
    #[test]
    #[ignore = "a measurement, run on a booked rig"]
    #[expect(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        reason = "a throughput measurement reads the wall clock"
    )]
    fn measure_validate_throughput() {
        const MIB: usize = 1 << 20;
        let fill = |unit: &str| {
            let mut s = String::new();
            while s.len() + unit.len() <= MIB {
                s.push_str(unit);
            }
            s.into_bytes()
        };
        let inputs = [
            (
                "ascii",
                fill("The quick brown fox jumps over the lazy dog. "),
            ),
            (
                "mixed",
                fill("Caf\u{e9} na\u{ef}ve \u{20ac}5 \u{1f600} r\u{e9}sum\u{e9} ok "),
            ),
            (
                "cjk",
                fill("\u{6f22}\u{5b57}\u{3068}\u{4eee}\u{540d}\u{306e}\u{6587}\u{7ae0}"),
            ),
        ];
        type Check = fn(&[u8]) -> Result<(), BadMessage>;
        let checks: [(&str, Check); 2] = [
            ("by byte (now)", validate::<{ 1 << 30 }>),
            ("by sequence (before)", by_sequence::<{ 1 << 30 }>),
        ];
        for (name, bytes) in &inputs {
            for (which, check) in checks {
                let rounds = 200u32;
                let start = std::time::Instant::now();
                for _ in 0..rounds {
                    assert!(check(std::hint::black_box(bytes)).is_ok());
                }
                let secs = start.elapsed().as_secs_f64();
                let len = u32::try_from(bytes.len()).map_or(0.0, f64::from);
                let mib = f64::from(rounds) * len / 1_048_576.0;
                println!("validate {name} {which}: {:.0} MiB/s", mib / secs);
            }
        }
    }

    /// The previous validator, one whole sequence per step (kept to
    /// compare results, error kinds included, and speed).
    fn by_sequence<const N: usize>(src: &[u8]) -> Result<(), BadMessage> {
        let at_ok = |at: Option<usize>, lo: u8, hi: u8| match at.and_then(|i| src.get(i)) {
            Some(&b) if (lo..=hi).contains(&b) => Ok(()),
            _ => Err(BadMessage::BadUtf8),
        };
        let mut at = 0usize;
        for _ in 0..N {
            if at >= src.len() {
                return Ok(());
            }
            let b0 = *src.get(at).ok_or(BadMessage::BadUtf8)?;
            let (owed, lo, hi) = lead(b0).ok_or(BadMessage::BadUtf8)?;
            let next = |k: usize| at.checked_add(k);
            if owed >= 1 {
                at_ok(next(1), lo, hi)?;
            }
            if owed >= 2 {
                at_ok(next(2), 0x80, 0xBF)?;
            }
            if owed >= 3 {
                at_ok(next(3), 0x80, 0xBF)?;
            }
            at = at
                .checked_add(usize::from(owed).saturating_add(1))
                .ok_or(BadMessage::BadUtf8)?;
        }
        if at >= src.len() {
            Ok(())
        } else {
            Err(BadMessage::OverBound)
        }
    }

    /// A small seeded generator (xorshift64*), so a failure reproduces.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next().checked_rem(n).unwrap_or(0)
        }

        fn byte_in(&mut self, lo: u8, hi: u8) -> u8 {
            let span = u64::from(hi.saturating_sub(lo)).saturating_add(1);
            lo.saturating_add(u8::try_from(self.below(span)).unwrap_or(0))
        }
    }

    /// Appends one well-formed piece: ASCII, or a scalar of any length.
    fn good_piece(g: &mut Rng, out: &mut Vec<u8>) {
        if g.below(4) == 0 {
            out.push(g.byte_in(0x00, 0x7F));
            return;
        }
        // Each encoded length about equally often.
        let top = [0x800, 0x1_0000, 0x11_0000][usize::try_from(g.below(3)).unwrap_or(0)];
        let c = loop {
            let v = u32::try_from(g.below(top)).unwrap_or(0);
            if let Some(c) = char::from_u32(v) {
                break c;
            }
        };
        let mut buf = [0u8; 4];
        out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    }

    /// Appends one piece: mostly multi-byte and ill-formed sequences.
    fn piece(g: &mut Rng, out: &mut Vec<u8>) {
        let cont = |g: &mut Rng| g.byte_in(0x80, 0xBF);
        match g.below(12) {
            // Well-formed.
            0..=3 => good_piece(g, out),
            // Overlongs: C0/C1 leads, E0 80..9F, F0 80..8F.
            4 => out.extend([g.byte_in(0xC0, 0xC1), cont(g)]),
            5 => out.extend([0xE0, g.byte_in(0x80, 0x9F), cont(g)]),
            6 => out.extend([0xF0, g.byte_in(0x80, 0x8F), cont(g), cont(g)]),
            // Surrogates: ED A0..BF.
            7 => out.extend([0xED, g.byte_in(0xA0, 0xBF), cont(g)]),
            // Over U+10FFFF: F4 90..BF, and F5..FF leads.
            8 => out.extend([0xF4, g.byte_in(0x90, 0xBF), cont(g), cont(g)]),
            9 => out.extend([g.byte_in(0xF5, 0xFF), cont(g), cont(g), cont(g)]),
            // Truncated: a lead with too few continuation bytes.
            10 => {
                let lead = [0xC2, 0xE1, 0xF1][usize::try_from(g.below(3)).unwrap_or(0)];
                out.push(lead);
                for _ in 0..g.below(u64::from(lead >> 4 & 0x3)) {
                    out.push(cont(g));
                }
            }
            // A lone continuation byte, or any byte.
            _ => out.push(if g.below(2) == 0 {
                cont(g)
            } else {
                g.byte_in(0x00, 0xFF)
            }),
        }
    }

    /// Agrees with `core::str::from_utf8` on 10 million inputs of 1 to
    /// about 24 bytes, built mostly from multi-byte and ill-formed pieces.
    /// Run in release: `cargo test --release -p hg-ipc --lib
    /// matches_the_standard -- --ignored`.
    #[test]
    #[ignore = "10 million inputs; run in release"]
    fn matches_the_standard_on_random_inputs() {
        let mut g = Rng(0x9e37_79b9_7f4a_7c15);
        let (mut good, mut bad) = (0u64, 0u64);
        let mut b = Vec::new();
        for _ in 0..10_000_000 {
            b.clear();
            // A quarter of the inputs are built from well-formed pieces
            // only, so long well-formed inputs are common too.
            let only_good = g.below(4) == 0;
            for _ in 0..=g.below(6) {
                if only_good {
                    good_piece(&mut g, &mut b);
                } else {
                    piece(&mut g, &mut b);
                }
            }
            let theirs = core::str::from_utf8(&b).is_ok();
            let ours = validate::<64>(&b);
            assert_eq!(ours.is_ok(), theirs, "input {b:02x?}");
            assert_eq!(ours, by_sequence::<64>(&b), "input {b:02x?}");
            if theirs {
                good += 1;
            } else {
                bad += 1;
            }
        }
        println!("well-formed {good}, ill-formed {bad}");
        assert!(good > 1_000_000 && bad > 1_000_000);
    }

    #[test]
    fn accepts_each_length_and_refuses_over_n() {
        assert_eq!(ours("a\u{e9}".as_bytes()).as_deref(), Some("a\u{e9}"));
        assert_eq!(ours("\u{20ac}".as_bytes()).as_deref(), Some("\u{20ac}"));
        assert_eq!(ours("\u{1f600}".as_bytes()).as_deref(), Some("\u{1f600}"));
        assert_eq!(validate::<2>(b"abc"), Err(BadMessage::OverBound));
    }
}
