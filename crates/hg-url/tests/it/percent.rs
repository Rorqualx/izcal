//! Percent-encoding, Section 1.3.

use hg_encoding::Encoding;
use hg_url::percent::{Set, decode, encode_after_encoding, encode_str, has_percent_encoded_byte};

#[test]
fn p01_the_six_sets_nest_except_where_the_standard_says_they_do_not() {
    // "The query percent-encode set cannot be defined in terms of the
    // fragment percent-encode set due to the omission of U+0060 (`)."
    assert!(Set::Fragment.contains('`'));
    assert!(!Set::Query.contains('`'));
    // Everything else does nest.
    for c in ('\u{0}'..='\u{7F}').chain(['\u{80}', '\u{3042}']) {
        if Set::Query.contains(c) {
            assert!(Set::SpecialQuery.contains(c), "special query misses {c:?}");
            assert!(Set::Path.contains(c), "path misses {c:?}");
        }
        if Set::Path.contains(c) {
            assert!(Set::Userinfo.contains(c), "userinfo misses {c:?}");
        }
        if Set::Userinfo.contains(c) {
            assert!(Set::Component.contains(c), "component misses {c:?}");
        }
        if Set::C0Control.contains(c) {
            assert!(Set::Fragment.contains(c), "fragment misses {c:?}");
            assert!(Set::Query.contains(c), "query misses {c:?}");
        }
    }
}

#[test]
fn p02_only_two_sets_encode_percent_and_so_round_trip() {
    // The standard: "Of the possible values for the percentEncodeSet
    // argument only two end up encoding U+0025 (%) and thus give
    // 'roundtripable data'".
    assert!(Set::Component.contains('%'));
    for set in [
        Set::C0Control,
        Set::Fragment,
        Set::Query,
        Set::SpecialQuery,
        Set::Path,
        Set::Userinfo,
    ] {
        assert!(!set.contains('%'), "{set:?} encodes %");
    }
}

#[test]
fn p03_encoding_is_upper_case_and_utf8() {
    assert_eq!(encode_str("\u{7F}", Set::C0Control), "%7F");
    assert_eq!(encode_str("\u{3042}", Set::C0Control), "%E3%81%82");
    assert_eq!(encode_str("a b", Set::Fragment), "a%20b");
}

#[test]
fn p04_a_percent_without_two_hex_digits_is_not_an_escape() {
    assert_eq!(decode(b"%41"), b"A");
    assert_eq!(decode(b"%4"), b"%4");
    assert_eq!(decode(b"%zz"), b"%zz");
    assert_eq!(decode(b"%%41"), b"%A");
    assert!(has_percent_encoded_byte("a%41"));
    assert!(!has_percent_encoded_byte("a%4"));
    assert!(!has_percent_encoded_byte("100%"));
}

#[test]
fn p05_decoding_keeps_bytes_that_are_not_utf8() {
    // Percent-decoding gives bytes, not a string: the host parser is
    // what decides whether they are text.
    assert_eq!(decode(b"%FF"), vec![0xFF]);
}

#[test]
fn p06_an_unmappable_code_point_becomes_the_escaped_numeric_reference() {
    // The standard: append "%26%23", the decimal scalar value, "%3B".
    let Some(big5) = Encoding::for_label("big5") else {
        panic!("big5 is a label");
    };
    assert_eq!(
        encode_after_encoding(big5, "\u{2020}", Set::SpecialQuery),
        "%26%238224%3B"
    );
}

#[test]
fn p07_the_escape_lands_where_the_code_point_was() {
    let Some(big5) = Encoding::for_label("big5") else {
        panic!("big5 is a label");
    };
    // `a`, then an unmappable code point, then `b`: the escape sits
    // between them, which a per-code-point loop could not promise once
    // an encoder holds state.
    assert_eq!(
        encode_after_encoding(big5, "a\u{2020}b", Set::SpecialQuery),
        "a%26%238224%3Bb"
    );
}

#[test]
fn p08_utf8_takes_the_direct_path_and_agrees_with_the_encoder_path() {
    for s in ["", "abc", "a b", "\u{3042}", "%", "a%41"] {
        assert_eq!(
            encode_after_encoding(Encoding::Utf8, s, Set::SpecialQuery),
            encode_str(s, Set::SpecialQuery),
            "{s:?}"
        );
    }
}
