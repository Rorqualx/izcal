//! Streaming gives the same answer as one call.
//!
//! Every decoder keeps state between bytes, and `TextDecoderStream`
//! hands it chunks of whatever size the network produced. A decoder
//! that answered differently for `[0x82], [0x22]` than for
//! `[0x82, 0x22]` would make the Section 2 masking defence depend on
//! packet boundaries, which is no defence at all.

use hg_encoding::{Decoder, Encoding};

fn whole(e: Encoding, bytes: &[u8]) -> String {
    Decoder::decode_all(e, bytes)
}

fn chunked(e: Encoding, bytes: &[u8], size: usize) -> String {
    let mut d = e.new_decoder();
    let mut out = String::new();
    let mut at = 0_usize;
    while at < bytes.len() {
        let end = at.saturating_add(size).min(bytes.len());
        let chunk = bytes.get(at..end).unwrap_or(&[]);
        let _ = d.decode(chunk, false, &mut out);
        at = end;
    }
    let _ = d.decode(&[], true, &mut out);
    out
}

/// Bytes that exercise each encoding's lead-byte paths, including
/// truncated and illegal sequences.
fn sample(e: Encoding) -> Vec<u8> {
    use hg_encoding::Family;
    let mut v: Vec<u8> = b"abc".to_vec();
    match e.family() {
        Family::Utf8 => v.extend([0xC3, 0xA9, 0xE2, 0x82, 0xAC, 0xF0, 0x9F, 0x98, 0x80, 0xC3]),
        Family::Utf16le | Family::Utf16be => {
            v.extend([0x41, 0x00, 0x3D, 0xD8, 0x00, 0xDE, 0x3D, 0xD8, 0x41]);
        }
        Family::Iso2022Jp => v.extend([
            0x1B, 0x24, 0x42, 0x24, 0x22, 0x1B, 0x28, 0x4A, 0x5C, 0x1B, 0x28, 0x42, 0x1B, 0x24,
        ]),
        Family::Gb18030 | Family::Gbk => {
            v.extend([
                0x81, 0x40, 0x81, 0x30, 0x81, 0x30, 0xA3, 0xA0, 0x82, 0x22, 0x81,
            ]);
        }
        Family::Big5
        | Family::EucJp
        | Family::EucKr
        | Family::Replacement
        | Family::ShiftJis
        | Family::SingleByte
        | Family::XUserDefined => {
            v.extend([0x81, 0x40, 0x82, 0x22, 0xA1, 0xA1, 0x8E, 0xB1, 0xFF, 0x81]);
        }
    }
    v
}

#[test]
fn s01_every_encoding_decodes_the_same_whatever_the_chunk_size() {
    for e in Encoding::ALL {
        let bytes = sample(e);
        let want = whole(e, &bytes);
        for size in 1..=5 {
            assert_eq!(
                chunked(e, &bytes, size),
                want,
                "{} differs at chunk size {size} on {bytes:02X?}",
                e.name()
            );
        }
    }
}

#[test]
fn s02_the_masking_defence_does_not_depend_on_where_a_chunk_ends() {
    // The pair that Section 2's attack used, split between chunks.
    let mut d = Encoding::ShiftJis.new_decoder();
    let mut out = String::new();
    let _ = d.decode(&[0x82], false, &mut out);
    let _ = d.decode(&[0x22], false, &mut out);
    let _ = d.decode(&[], true, &mut out);
    assert_eq!(out, "\u{FFFD}\"");
}

#[test]
fn s03_an_incomplete_sequence_at_the_end_is_one_error() {
    // Not an error per missing byte, and not silence.
    for (e, bytes) in [
        (Encoding::Utf8, &[0xE2, 0x82][..]),
        (Encoding::Big5, &[0xA1][..]),
        (Encoding::EucKr, &[0x81][..]),
        (Encoding::ShiftJis, &[0x81][..]),
        (Encoding::EucJp, &[0x8F, 0xA1][..]),
        (Encoding::Utf16le, &[0x41][..]),
    ] {
        assert_eq!(whole(e, bytes), "\u{FFFD}", "{}", e.name());
    }
}

#[test]
fn s04_a_decoder_that_is_not_told_it_is_last_reports_nothing_yet() {
    let mut d = Encoding::Utf8.new_decoder();
    let mut out = String::new();
    let _ = d.decode(&[0xE2, 0x82], false, &mut out);
    assert_eq!(out, "", "an incomplete sequence waits for more bytes");
    let _ = d.decode(&[0xAC], false, &mut out);
    assert_eq!(out, "\u{20AC}");
}

#[test]
fn s05_an_empty_last_chunk_finishes_a_clean_stream_without_an_error() {
    for e in Encoding::ALL {
        let mut d = e.new_decoder();
        let mut out = String::new();
        let _ = d.decode(b"ab", false, &mut out);
        let _ = d.decode(&[], true, &mut out);
        if e == Encoding::Replacement {
            // It is the one decoder that answers an error to *any*
            // input, which is the whole of what it is for.
            assert_eq!(out, "\u{FFFD}");
            continue;
        }
        assert!(
            !out.contains('\u{FFFD}'),
            "{} invented an error at the end: {out:?}",
            e.name()
        );
    }
}
