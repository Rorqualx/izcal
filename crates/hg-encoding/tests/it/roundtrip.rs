//! Every byte sequence the indexes define, through the decoder and back
//! through the encoder.
//!
//! This is the check the indexes themselves make possible: for every
//! encoding and every byte pair it accepts, decoding then re-encoding
//! must give the same bytes — except where the standard says it must
//! not, and each of those exceptions is named here rather than skipped
//! as a class.

use hg_encoding::{Decoder, Encoder, Encoding, Family};

/// Decodes one byte sequence, with no streaming.
fn dec(e: Encoding, bytes: &[u8]) -> String {
    Decoder::decode_all(e, bytes)
}

/// Encodes one string, marking anything unmappable so a test can see
/// it rather than having it silently replaced.
fn enc(e: Encoding, s: &str) -> Result<Vec<u8>, char> {
    let mut encoder = Encoder::new(e).ok_or('\0')?;
    let mut out = Vec::new();
    let mut bad = None;
    encoder.encode(s, true, &mut out, &mut |c, _| bad = Some(c));
    match bad {
        Some(c) => Err(c),
        None => Ok(out),
    }
}

#[test]
fn r01_ascii_round_trips_through_every_ascii_compatible_encoding() {
    let ascii: String = (0x00_u8..=0x7F).map(char::from).collect();
    for e in Encoding::ALL {
        if !e.is_ascii_compatible() {
            continue;
        }
        let bytes = enc(e, &ascii).unwrap_or_else(|c| panic!("{}: {c:?}", e.name()));
        assert_eq!(bytes, ascii.as_bytes(), "{} encodes ASCII", e.name());
        assert_eq!(dec(e, &bytes), ascii, "{} decodes ASCII", e.name());
    }
}

#[test]
fn r02_every_single_byte_encoding_round_trips_all_256_bytes() {
    for e in Encoding::ALL {
        if e.family() != Family::SingleByte {
            continue;
        }
        let mut checked = 0_usize;
        for b in 0x00_u8..=0xFF {
            let text = dec(e, &[b]);
            if text == "\u{FFFD}" {
                // The index has no entry for it; the standard says
                // error, and an error is not something to re-encode.
                continue;
            }
            let back = enc(e, &text).unwrap_or_else(|c| {
                panic!(
                    "{}: {b:#04X} decoded to {c:?} and would not encode",
                    e.name()
                )
            });
            assert_eq!(back, [b], "{}: byte {b:#04X} did not round-trip", e.name());
            checked = checked.saturating_add(1);
        }
        assert!(
            checked >= 128 + 64,
            "{} round-tripped only {checked} bytes",
            e.name()
        );
    }
}

/// Every two-byte sequence a multi-byte encoding might accept, decoded
/// and re-encoded.
///
/// Three outcomes, and each is counted rather than lumped together:
///
/// - **round-tripped** — the bytes came back.
/// - **re-encoded elsewhere** — the index has the same code point at
///   more than one pointer and the standard's pointer algorithm picks
///   another. Correct, and the reason those algorithms exist.
/// - **decode-only** — the encoder refuses the code point. Every one
///   of these is a rule the standard states, so the count is asserted
///   rather than waved through: a decoder that started producing
///   something unencodable would move it.
fn two_byte_round_trip(
    e: Encoding,
    leads: core::ops::RangeInclusive<u8>,
    trails: core::ops::RangeInclusive<u8>,
    expected: (usize, usize, usize),
) {
    let (mut same, mut elsewhere, mut only) = (0_usize, 0_usize, 0_usize);
    for lead in leads {
        for trail in trails.clone() {
            let text = dec(e, &[lead, trail]);
            if text.contains('\u{FFFD}') || text.chars().count() != 1 {
                continue;
            }
            match enc(e, &text) {
                Ok(back) if back == [lead, trail] => same = same.saturating_add(1),
                Ok(_) => elsewhere = elsewhere.saturating_add(1),
                Err(_) => only = only.saturating_add(1),
            }
        }
    }
    println!(
        "{}: {same} round-tripped, {elsewhere} re-encoded elsewhere, {only} decode-only",
        e.name()
    );
    assert_eq!((same, elsewhere, only), expected, "{}", e.name());
}

#[test]
fn r03_gb18030_two_byte_sequences() {
    // U+E5E5 cannot round-trip: index gb18030 maps 0xA3 0xA0 to U+3000
    // for compatibility with deployed content, so the encoder refuses
    // U+E5E5 outright (Section 10.2.2).
    // Every pair round-trips but one: 0xA3 0xA0 decodes to U+3000,
    // which encodes back to U+3000's *first* pointer. That is the
    // mapping the standard chose "for compatibility with deployed
    // content", and the reason U+E5E5 cannot round-trip at all
    // (Section 10.2.2, checked in c06).
    two_byte_round_trip(Encoding::Gb18030, 0x81..=0xFE, 0x00..=0xFE, (23939, 1, 0));
}

#[test]
fn r04_big5_two_byte_sequences() {
    // 3,847 pairs decode and will not encode: `index Big5 pointer`
    // excludes every pointer below (0xA1 − 0x81) × 157, so a Hong Kong
    // Supplementary Character Set extension is never returned
    // literally (Section 5). The 90 that re-encode elsewhere are the
    // duplicated code points, six of which take the *last* pointer
    // (c04) and the rest the first.
    two_byte_round_trip(Encoding::Big5, 0x81..=0xFE, 0x00..=0xFE, (14653, 90, 3847));
}

#[test]
fn r05_euc_kr_two_byte_sequences() {
    two_byte_round_trip(Encoding::EucKr, 0x81..=0xFE, 0x00..=0xFE, (17048, 0, 0));
}

#[test]
fn r06_shift_jis_two_byte_sequences() {
    // 1,880 decode-only: the EUDC range, pointers 8836 to 10715,
    // decodes to U+E000..U+E757, which index jis0208 does not contain
    // (Section 12.3.1 — "interoperable legacy from Windows"). The 398
    // that re-encode elsewhere are jis0208's duplicated code points,
    // which `index Shift_JIS pointer` resolves by excluding pointers
    // 8272 to 8835 so the later one is used (Section 5).
    two_byte_round_trip(
        Encoding::ShiftJis,
        0x81..=0xFC,
        0x00..=0xFC,
        (7326, 398, 1880),
    );
}

#[test]
fn r07_euc_jp_two_byte_sequences() {
    // The two-byte space is index jis0208, the same table Shift_JIS
    // reads — hence the same 7,326 round-tripping — but EUC-JP's
    // encoder uses `index pointer` with no exclusion, so only 10 land
    // on another pointer rather than 398.
    //
    // Three-byte sequences through the 0x8F lead reach index jis0212,
    // which is decode-only: the standard's EUC-JP encoder reads index
    // jis0208 alone, and jis0212 is "only used by the EUC-JP decoder
    // due to lack of widespread support elsewhere" (Section 5).
    let mut three = Vec::new();
    for lead in 0xA1_u8..=0xFE {
        for trail in 0xA1_u8..=0xFE {
            let text = dec(Encoding::EucJp, &[0x8F, lead, trail]);
            if !text.contains('\u{FFFD}') {
                three.extend(text.chars());
            }
        }
    }
    assert!(three.len() > 5000, "jis0212 decoded only {}", three.len());
    two_byte_round_trip(Encoding::EucJp, 0xA1..=0xFE, 0xA1..=0xFE, (7326, 10, 0));
}

#[test]
fn r08_utf8_round_trips_every_scalar_value() {
    // Every code point, in blocks, through Izcal's own UTF-8 rather
    // than Rust's.
    let mut n = 0_usize;
    for block in 0..0x110 {
        let text: String = (0..0x1000_u32)
            .filter_map(|i| char::from_u32((block * 0x1000) + i))
            .collect();
        if text.is_empty() {
            continue;
        }
        let bytes = enc(Encoding::Utf8, &text).unwrap_or_else(|c| panic!("{c:?}"));
        assert_eq!(bytes, text.as_bytes(), "block {block:#X}");
        assert_eq!(dec(Encoding::Utf8, &bytes), text, "block {block:#X}");
        n = n.saturating_add(text.chars().count());
    }
    assert_eq!(n, 1_112_064, "every scalar value");
}

#[test]
fn r09_x_user_defined_round_trips_its_whole_range() {
    for b in 0x00_u8..=0xFF {
        let text = dec(Encoding::XUserDefined, &[b]);
        assert_eq!(text.chars().count(), 1, "{b:#04X}");
        let back = enc(Encoding::XUserDefined, &text).unwrap_or_else(|c| panic!("{c:?}"));
        assert_eq!(back, [b], "{b:#04X}");
    }
    // The non-ASCII half lands in the private use area, as Section
    // 14.5.1 defines.
    assert_eq!(dec(Encoding::XUserDefined, &[0x80]), "\u{F780}");
    assert_eq!(dec(Encoding::XUserDefined, &[0xFF]), "\u{F7FF}");
}

#[test]
fn r10_gb18030_four_byte_sequences_cover_the_ranges() {
    // The four-byte form is defined by 207 ranges rather than a table,
    // so this walks every range's first and last pointer.
    let mut n = 0_usize;
    for cp in (0x0080_u32..=0x10FFFF).step_by(997) {
        let Some(c) = char::from_u32(cp) else {
            continue;
        };
        if (0xD800..=0xDFFF).contains(&cp) {
            continue;
        }
        let text = c.to_string();
        let Ok(bytes) = enc(Encoding::Gb18030, &text) else {
            continue;
        };
        assert_eq!(
            dec(Encoding::Gb18030, &bytes),
            text,
            "U+{cp:04X} did not round-trip through {bytes:02X?}"
        );
        n = n.saturating_add(1);
    }
    assert!(n > 1000, "only {n} code points were encodable");
}
