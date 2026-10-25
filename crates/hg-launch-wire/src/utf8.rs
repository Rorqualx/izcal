//! Well-formed UTF-8, from the Unicode Standard's table of well-formed
//! byte sequences (Unicode 16.0, Section 3.9, Table 3-7), with one rule
//! of IZLH's own on top: no NUL (OS integration 6.2, "validated, no
//! NUL").
//!
//! Written here rather than taken from `core::str::from_utf8` so the
//! decoder's Kani harnesses see a small loop with a fixed shape; a test
//! checks it agrees with `core` on every one- and two-byte input and on
//! the table's boundary sequences.

/// Whether `bytes` is well-formed UTF-8 with no NUL.
pub(crate) fn is_text(bytes: &[u8]) -> bool {
    let mut i = 0usize;
    // Each pass consumes at least one byte, so `bytes.len()` passes bound
    // the loop.
    while let Some(&lead) = bytes.get(i) {
        let (len, lo, hi) = match lead {
            // NUL is refused; the rest of U+0001..U+007F is one byte.
            0x00 => return false,
            0x01..=0x7F => {
                i = i.saturating_add(1);
                continue;
            }
            0xC2..=0xDF => (2, 0x80, 0xBF),
            0xE0 => (3, 0xA0, 0xBF),
            0xE1..=0xEC | 0xEE..=0xEF => (3, 0x80, 0xBF),
            0xED => (3, 0x80, 0x9F),
            0xF0 => (4, 0x90, 0xBF),
            0xF1..=0xF3 => (4, 0x80, 0xBF),
            0xF4 => (4, 0x80, 0x8F),
            // 0x80..=0xC1 and 0xF5..=0xFF never start a sequence.
            _ => return false,
        };
        // The second byte has the lead's own range; later ones 80..BF.
        let Some(&second) = bytes.get(i.saturating_add(1)) else {
            return false;
        };
        if !(lo..=hi).contains(&second) {
            return false;
        }
        let mut k = 2usize;
        while k < len {
            match bytes.get(i.saturating_add(k)) {
                Some(&b) if (0x80..=0xBF).contains(&b) => {}
                _ => return false,
            }
            k = k.saturating_add(1);
        }
        i = i.saturating_add(len);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::is_text;

    fn core_says(b: &[u8]) -> bool {
        core::str::from_utf8(b).is_ok() && !b.contains(&0)
    }

    #[test]
    fn agrees_with_core_on_every_short_input() {
        for a in 0..=255u8 {
            assert_eq!(is_text(&[a]), core_says(&[a]), "{a:#x}");
            for b in 0..=255u8 {
                assert_eq!(is_text(&[a, b]), core_says(&[a, b]), "{a:#x} {b:#x}");
            }
        }
    }

    #[test]
    fn agrees_with_core_on_the_table_boundaries() {
        let leads = [
            0xE0u8, 0xE1, 0xEC, 0xED, 0xEE, 0xEF, 0xF0, 0xF1, 0xF3, 0xF4, 0xF5,
        ];
        let edges = [0x7Fu8, 0x80, 0x8F, 0x90, 0x9F, 0xA0, 0xBF, 0xC0];
        for &l in &leads {
            for &a in &edges {
                for &b in &edges {
                    let three = [l, a, b];
                    assert_eq!(is_text(&three), core_says(&three), "{three:x?}");
                    for &c in &edges {
                        let four = [l, a, b, c];
                        assert_eq!(is_text(&four), core_says(&four), "{four:x?}");
                    }
                }
            }
        }
    }
}

/// VER-1 for the text check.
#[cfg(kani)]
mod proofs {
    use super::is_text;

    /// Never panics, and agrees with `core` on every input of up to five
    /// bytes.
    #[kani::proof]
    #[kani::unwind(7)]
    fn text_check_agrees_with_core() {
        let bytes: [u8; 5] = kani::any();
        let len: usize = kani::any_where(|&n: &usize| n <= 5);
        let b = &bytes[..len];
        let ours = is_text(b);
        let mut nul = false;
        for &x in b {
            nul |= x == 0;
        }
        assert!(ours == (core::str::from_utf8(b).is_ok() && !nul));
        kani::cover!(ours && len == 5, "five bytes of text");
    }
}
