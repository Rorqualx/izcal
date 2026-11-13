//! The cases the standard writes out in its own prose and notes.
//!
//! These are the ones a table-driven test would miss: the security
//! rule of Section 2, the four Big5 pointers that are two code points,
//! the EUDC range, the ISO-2022-JP note about concatenation, and the
//! handful of asymmetric mappings each encoder names.

use hg_encoding::{DecodeError, Decoder, Encoder, Encoding, ErrorMode, decode};

fn dec(e: Encoding, bytes: &[u8]) -> String {
    Decoder::decode_all(e, bytes)
}

fn enc(e: Encoding, s: &str) -> Vec<u8> {
    Encoder::encode_all(e, s).unwrap_or_default()
}

#[test]
fn c01_an_illegal_pair_does_not_mask_an_ascii_byte() {
    // Section 2, the 2011 attack: a Shift_JIS lead byte 0x82 used to
    // mask a 0x22 trailing byte in JSON. "The consumer decoded it as a
    // single U+FFFD and therefore changed the overall interpretation
    // as U+0022 is an important delimiter. ... For the aforementioned
    // sequence the output would be U+FFFD U+0022."
    assert_eq!(dec(Encoding::ShiftJis, &[0x82, 0x22]), "\u{FFFD}\"");
    // The same rule in every multi-byte decoder that has a lead byte.
    for e in [
        Encoding::Big5,
        Encoding::EucKr,
        Encoding::Gb18030,
        Encoding::Gbk,
        Encoding::EucJp,
    ] {
        assert_eq!(
            dec(e, &[0x81, 0x22]),
            "\u{FFFD}\"",
            "{} masked a quotation mark",
            e.name()
        );
    }
}

#[test]
fn c02_gb18030_is_the_one_that_may_mask_a_byte_at_the_end() {
    // Section 2: "As an unfortunate exception to this, the gb18030
    // decoder will mask up to one such byte at end-of-queue."
    // 0x81 0x30 0x81 is an incomplete four-byte sequence; the 0x30 is
    // consumed with it.
    assert_eq!(dec(Encoding::Gb18030, &[0x81, 0x30, 0x81]), "\u{FFFD}");
}

#[test]
fn c03_the_four_big5_pointers_that_are_two_code_points() {
    // Section 11.1.1's table. An index holds one code point per
    // pointer, so the standard writes these four out.
    assert_eq!(dec(Encoding::Big5, &[0x88, 0x62]), "\u{00CA}\u{0304}");
    assert_eq!(dec(Encoding::Big5, &[0x88, 0x64]), "\u{00CA}\u{030C}");
    assert_eq!(dec(Encoding::Big5, &[0x88, 0xA3]), "\u{00EA}\u{0304}");
    assert_eq!(dec(Encoding::Big5, &[0x88, 0xA5]), "\u{00EA}\u{030C}");
}

#[test]
fn c04_the_six_big5_code_points_that_use_the_last_pointer() {
    // Section 5's "index Big5 pointer": for these six the *last*
    // pointer is used, for every other duplicate the first.
    for (c, bytes) in [
        ('\u{2550}', [0xF9, 0xF9]),
        ('\u{255E}', [0xF9, 0xE9]),
        ('\u{2561}', [0xF9, 0xEB]),
        ('\u{256A}', [0xF9, 0xEA]),
        ('\u{5341}', [0xA4, 0x51]),
        ('\u{5345}', [0xA4, 0xCA]),
    ] {
        assert_eq!(enc(Encoding::Big5, &c.to_string()), bytes, "{c:?}");
    }
}

#[test]
fn c05_the_shift_jis_eudc_range_decodes_but_does_not_encode() {
    // Section 12.3.1: pointers 8836 to 10715 are "interoperable legacy
    // from Windows known as EUDC".
    assert_eq!(dec(Encoding::ShiftJis, &[0xF0, 0x40]), "\u{E000}");
    assert_eq!(dec(Encoding::ShiftJis, &[0xF9, 0xFC]), "\u{E757}");
    // Encoding one back is not possible: index jis0208 has no entry.
    assert_eq!(enc(Encoding::ShiftJis, "\u{E000}"), b"&#57344;");
}

#[test]
fn c06_gb18030_cannot_round_trip_u_e5e5() {
    // Section 10.2.2: "Index gb18030 maps 0xA3 0xA0 to U+3000
    // IDEOGRAPHIC SPACE rather than U+E5E5 for compatibility with
    // deployed content. Therefore it cannot roundtrip."
    assert_eq!(dec(Encoding::Gb18030, &[0xA3, 0xA0]), "\u{3000}");
    assert_eq!(enc(Encoding::Gb18030, "\u{E5E5}"), b"&#58853;");
}

#[test]
fn c07_gbk_differs_from_gb18030_only_in_its_encoder() {
    // Section 10.1: GBK's decoder *is* gb18030's; its encoder is
    // gb18030's with `is GBK` true, which is "a conservative move to
    // decrease the chances of breaking legacy servers".
    for bytes in [vec![0x81, 0x40], vec![0x80], vec![0xA3, 0xA0]] {
        assert_eq!(
            dec(Encoding::Gbk, &bytes),
            dec(Encoding::Gb18030, &bytes),
            "{bytes:02X?}"
        );
    }
    // The euro is one byte in GBK and the four-byte form is refused.
    assert_eq!(enc(Encoding::Gbk, "\u{20AC}"), [0x80]);
    assert_eq!(enc(Encoding::Gb18030, "\u{20AC}"), [0xA2, 0xE3]);
    // A code point only the four-byte form reaches: GBK refuses it.
    assert_eq!(
        enc(Encoding::Gb18030, "\u{1D11E}"),
        [0x94, 0x32, 0xBE, 0x34]
    );
    assert_eq!(enc(Encoding::Gbk, "\u{1D11E}"), b"&#119070;");
}

#[test]
fn c08_the_gb18030_asymmetric_encoder_table() {
    // Section 10.2.2's eighteen rows, which preserve GB18030-2005's
    // byte sequences for private-use code points.
    assert_eq!(enc(Encoding::Gb18030, "\u{E78D}"), [0xA6, 0xD9]);
    assert_eq!(enc(Encoding::Gb18030, "\u{E864}"), [0xFE, 0xA0]);
    // They are asymmetric on purpose: decoding those bytes gives
    // something else.
    assert_ne!(dec(Encoding::Gb18030, &[0xA6, 0xD9]), "\u{E78D}");
}

#[test]
fn c09_the_gb18030_ranges_special_pointer() {
    // Section 5: "If pointer is 7457, then return code point U+E7C7",
    // and the reverse.
    assert_eq!(enc(Encoding::Gb18030, "\u{E7C7}"), [0x81, 0x35, 0xF4, 0x37]);
    assert_eq!(
        dec(Encoding::Gb18030, &[0x81, 0x35, 0xF4, 0x37]),
        "\u{E7C7}"
    );
}

#[test]
fn c10_iso_2022_jp_escapes_and_the_concatenation_note() {
    // Section 12.2.2's own worked example: "Encoding U+00A5 gives
    // 0x1B 0x28 0x4A 0x5C 0x1B 0x28 0x42. Doing that twice,
    // concatenating the results, and then decoding yields U+00A5
    // U+FFFD U+00A5."
    let once = enc(Encoding::Iso2022Jp, "\u{00A5}");
    assert_eq!(once, [0x1B, 0x28, 0x4A, 0x5C, 0x1B, 0x28, 0x42]);
    let twice: Vec<u8> = once.iter().chain(once.iter()).copied().collect();
    assert_eq!(dec(Encoding::Iso2022Jp, &twice), "\u{00A5}\u{FFFD}\u{00A5}");
}

#[test]
fn c11_iso_2022_jp_refuses_the_three_bytes_that_would_let_content_switch_mode() {
    // Section 12.2.2: U+000E, U+000F and U+001B "return U+FFFD rather
    // than codePoint to prevent attacks".
    for c in ['\u{E}', '\u{F}', '\u{1B}'] {
        assert_eq!(
            enc(Encoding::Iso2022Jp, &c.to_string()),
            b"&#65533;",
            "{c:?} should be refused as U+FFFD"
        );
    }
}

#[test]
fn c12_iso_2022_jp_round_trips_a_mixed_string() {
    // No halfwidth katakana: the encoder maps those to their fullwidth
    // forms through its own index, so they do not come back as
    // themselves. c13 is about that.
    let text = "a\u{3042}b\u{00A5}c\u{30A2}d";
    let bytes = enc(Encoding::Iso2022Jp, text);
    assert_eq!(dec(Encoding::Iso2022Jp, &bytes), text);
    // A string that ends in jis0208 gets the escape back to ASCII at
    // the very end, which is the one thing an encoder writes on its
    // own at end-of-queue (Section 12.2.2).
    let ends_in_jis = enc(Encoding::Iso2022Jp, "a\u{3042}");
    assert_eq!(
        ends_in_jis,
        [0x61, 0x1B, 0x24, 0x42, 0x24, 0x22, 0x1B, 0x28, 0x42]
    );
    assert_eq!(dec(Encoding::Iso2022Jp, &ends_in_jis), "a\u{3042}");
}

#[test]
fn c13_the_halfwidth_katakana_each_encoding_maps_its_own_way() {
    // EUC-JP: two bytes, 0x8E and the offset (Section 12.1.2).
    assert_eq!(enc(Encoding::EucJp, "\u{FF71}"), [0x8E, 0xB1]);
    // Shift_JIS: one byte (Section 12.3.2).
    assert_eq!(enc(Encoding::ShiftJis, "\u{FF71}"), [0xB1]);
    // ISO-2022-JP: through the katakana index to the fullwidth form.
    assert_eq!(
        dec(Encoding::Iso2022Jp, &enc(Encoding::Iso2022Jp, "\u{FF71}")),
        "\u{30A2}"
    );
}

#[test]
fn c14_the_yen_and_overline_shortcuts() {
    // Three encoders map U+00A5 to 0x5C and U+203E to 0x7E.
    for e in [Encoding::EucJp, Encoding::ShiftJis] {
        assert_eq!(enc(e, "\u{00A5}"), [0x5C], "{}", e.name());
        assert_eq!(enc(e, "\u{203E}"), [0x7E], "{}", e.name());
    }
    // And U+2212 MINUS SIGN becomes U+FF0D FULLWIDTH HYPHEN-MINUS
    // before the index is consulted.
    assert_eq!(
        enc(Encoding::ShiftJis, "\u{2212}"),
        enc(Encoding::ShiftJis, "\u{FF0D}")
    );
    assert_eq!(
        enc(Encoding::EucJp, "\u{2212}"),
        enc(Encoding::EucJp, "\u{FF0D}")
    );
}

#[test]
fn c15_the_replacement_encoding_decodes_to_exactly_one_error() {
    // Section 14.1: it "exists to prevent certain attacks that abuse a
    // mismatch between encodings supported on the server and the
    // client". Whatever the input, one U+FFFD and nothing else.
    for bytes in [&b""[..], b"hello", b"\xFF\xFE\x00abc"] {
        let text = dec(Encoding::Replacement, bytes);
        assert_eq!(
            text,
            if bytes.is_empty() { "" } else { "\u{FFFD}" },
            "{bytes:02X?}"
        );
    }
}

#[test]
fn c16_utf16_surrogates() {
    assert_eq!(
        dec(Encoding::Utf16le, &[0x3D, 0xD8, 0x00, 0xDE]),
        "\u{1F600}"
    );
    assert_eq!(
        dec(Encoding::Utf16be, &[0xD8, 0x3D, 0xDE, 0x00]),
        "\u{1F600}"
    );
    // A lone leading surrogate, then a normal unit: the standard
    // restores the second unit's bytes, so it decodes on its own.
    assert_eq!(
        dec(Encoding::Utf16le, &[0x3D, 0xD8, 0x41, 0x00]),
        "\u{FFFD}A"
    );
    // A lone trailing surrogate.
    assert_eq!(dec(Encoding::Utf16le, &[0x00, 0xDC]), "\u{FFFD}");
    // An odd number of bytes.
    assert_eq!(dec(Encoding::Utf16le, &[0x41, 0x00, 0x42]), "A\u{FFFD}");
}

#[test]
fn c17_utf8_follows_the_best_practices_for_u_fffd() {
    // Section 8.1.1: "The constraints in the UTF-8 decoder above match
    // 'Best Practices for Using U+FFFD' from the Unicode standard."
    // The count of replacement characters is what that is about.
    for (bytes, want) in [
        (&[0xC0, 0x80][..], "\u{FFFD}\u{FFFD}"),
        (&[0xE0, 0x80, 0x80][..], "\u{FFFD}\u{FFFD}\u{FFFD}"),
        (
            &[0xF5, 0x80, 0x80, 0x80][..],
            "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}",
        ),
        (&[0xED, 0xA0, 0x80][..], "\u{FFFD}\u{FFFD}\u{FFFD}"),
        (&[0xE1, 0x80][..], "\u{FFFD}"),
        (&[0xF0, 0x90, 0x80, 0x80][..], "\u{10000}"),
        (
            &[0x61, 0xF1, 0x80, 0x80, 0xE1, 0x80, 0xC2, 0x62][..],
            "a\u{FFFD}\u{FFFD}\u{FFFD}b",
        ),
    ] {
        assert_eq!(dec(Encoding::Utf8, bytes), want, "{bytes:02X?}");
    }
}

#[test]
fn c18_a_bom_outranks_the_label() {
    // Section 6.1: "For compatibility with deployed content, the byte
    // order mark is more authoritative than anything else. In a
    // context where HTTP is used this is in violation of the semantics
    // of the Content-Type header."
    let (text, used) = decode(Encoding::Windows1252, &[0xEF, 0xBB, 0xBF, 0xC3, 0xA9]);
    assert_eq!(text, "\u{E9}");
    assert_eq!(used, Encoding::Utf8);
    let (text, used) = decode(Encoding::Windows1252, &[0xFF, 0xFE, 0x41, 0x00]);
    assert_eq!(text, "A");
    assert_eq!(used, Encoding::Utf16le);
    // Without one, the label stands.
    let (text, used) = decode(Encoding::Windows1252, &[0xC3, 0xA9]);
    assert_eq!(text, "\u{C3}\u{A9}");
    assert_eq!(used, Encoding::Windows1252);
}

#[test]
fn c19_fatal_mode_stops_rather_than_replacing() {
    let mut out = String::new();
    let err = Decoder::new(Encoding::Utf8, ErrorMode::Fatal).decode(&[0x61, 0xFF], true, &mut out);
    assert_eq!(err, Err(DecodeError));
    assert_eq!(out, "a", "what decoded before the error is kept");
    let mut out = String::new();
    assert!(
        Decoder::new(Encoding::Utf8, ErrorMode::Fatal)
            .decode("héllo".as_bytes(), true, &mut out)
            .is_ok()
    );
    assert_eq!(out, "héllo");
}

#[test]
fn c20_an_unmappable_code_point_is_handed_back_not_chosen_for() {
    // The standard returns "error with codePoint" and leaves the rest
    // to the caller: HTML forms write &#NNN;, URLs percent-encode.
    let Some(mut e) = Encoder::new(Encoding::Windows1252) else {
        panic!("windows-1252 has an encoder")
    };
    let mut out = Vec::new();
    let mut seen = Vec::new();
    e.encode("a\u{1F4A9}b", true, &mut out, &mut |c, out| {
        seen.push(c);
        out.extend(format!("%{:02X}", u32::from(c) & 0xFF).as_bytes());
    });
    assert_eq!(seen, ['\u{1F4A9}']);
    assert_eq!(out, b"a%A9b");
}
