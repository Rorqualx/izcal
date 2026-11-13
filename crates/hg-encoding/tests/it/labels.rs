//! "Get an encoding" (Section 4.2), and the two algorithms around it.

use hg_encoding::{Encoding, Family, bom_sniff};

#[test]
fn e01_a_label_finds_its_encoding() {
    assert_eq!(Encoding::for_label("utf-8"), Some(Encoding::Utf8));
    assert_eq!(Encoding::for_label("utf8"), Some(Encoding::Utf8));
    assert_eq!(Encoding::for_label("latin1"), Some(Encoding::Windows1252));
    assert_eq!(Encoding::for_label("shift_jis"), Some(Encoding::ShiftJis));
    assert_eq!(Encoding::for_label("nonesuch"), None);
    assert_eq!(Encoding::for_label(""), None);
}

#[test]
fn e02_labels_are_ascii_case_insensitive_and_trimmed() {
    // The standard: remove leading and trailing ASCII whitespace, then
    // match ASCII case-insensitively.
    for spelling in ["UTF-8", "Utf-8", "  utf-8  ", "\t\n\u{C}\r utf-8 \r\n"] {
        assert_eq!(
            Encoding::for_label(spelling),
            Some(Encoding::Utf8),
            "{spelling:?} should find UTF-8"
        );
    }
    // Only ASCII whitespace is removed: a non-breaking space is not.
    assert_eq!(Encoding::for_label("\u{A0}utf-8"), None);
}

#[test]
fn e03_an_unknown_label_is_failure_and_not_a_guess() {
    // The attack `replacement` exists to stop is a label the server
    // understands and the client does not. Inventing an encoding here
    // is how that attack works.
    for label in ["utf-9", "iso-8859-1-x", "big-5", "hz-gb-2312-x"] {
        assert_eq!(Encoding::for_label(label), None, "{label}");
    }
}

#[test]
fn e04_every_encodings_own_name_is_one_of_its_labels() {
    // The standard: "For each encoding, ASCII-lowercasing its name
    // yields one of its labels."
    for e in Encoding::ALL {
        assert_eq!(
            Encoding::for_label(&e.name().to_ascii_lowercase()),
            Some(e),
            "{} does not find itself",
            e.name()
        );
    }
}

#[test]
fn e05_the_labels_that_select_the_replacement_encoding() {
    // Seven labels for ASCII-incompatible encodings are mapped here
    // rather than to the unknown-encoding fallback.
    for label in [
        "iso-2022-cn",
        "iso-2022-cn-ext",
        "iso-2022-kr",
        "hz-gb-2312",
        "csiso2022kr",
        "replacement",
    ] {
        assert_eq!(
            Encoding::for_label(label),
            Some(Encoding::Replacement),
            "{label}"
        );
    }
}

#[test]
fn e06_get_an_output_encoding() {
    // Section 4.3: the three that may not be encoded to become UTF-8.
    assert_eq!(Encoding::Replacement.output_encoding(), Encoding::Utf8);
    assert_eq!(Encoding::Utf16be.output_encoding(), Encoding::Utf8);
    assert_eq!(Encoding::Utf16le.output_encoding(), Encoding::Utf8);
    assert_eq!(
        Encoding::Windows1252.output_encoding(),
        Encoding::Windows1252
    );
    for e in Encoding::ALL {
        assert!(
            e.output_encoding().new_encoder().is_some(),
            "{} has no output encoding that can encode",
            e.name()
        );
    }
}

#[test]
fn e07_only_three_encodings_have_no_encoder() {
    let without: Vec<&str> = Encoding::ALL
        .iter()
        .filter(|e| e.new_encoder().is_none())
        .map(|e| e.name())
        .collect();
    assert_eq!(without, ["replacement", "UTF-16BE", "UTF-16LE"]);
}

#[test]
fn e08_the_ascii_incompatible_encodings_are_the_four_the_standard_names() {
    let incompatible: Vec<&str> = Encoding::ALL
        .iter()
        .filter(|e| !e.is_ascii_compatible())
        .map(|e| e.name())
        .collect();
    assert_eq!(
        incompatible,
        ["ISO-2022-JP", "replacement", "UTF-16BE", "UTF-16LE"]
    );
}

#[test]
fn e09_bom_sniff() {
    assert_eq!(bom_sniff(&[0xEF, 0xBB, 0xBF]), Some((Encoding::Utf8, 3)));
    assert_eq!(bom_sniff(&[0xFE, 0xFF]), Some((Encoding::Utf16be, 2)));
    assert_eq!(bom_sniff(&[0xFF, 0xFE]), Some((Encoding::Utf16le, 2)));
    assert_eq!(bom_sniff(&[0xEF, 0xBB]), None, "a partial BOM is not one");
    assert_eq!(bom_sniff(b"hello"), None);
    assert_eq!(bom_sniff(&[]), None);
    // 0xFF 0xFE 0x00 0x00 is UTF-32LE's mark; the standard does not
    // support UTF-32, and the first two bytes win.
    assert_eq!(
        bom_sniff(&[0xFF, 0xFE, 0x00, 0x00]),
        Some((Encoding::Utf16le, 2))
    );
}

#[test]
fn e10_the_table_is_the_size_the_pin_says() {
    assert_eq!(Encoding::ALL.len(), 40);
    let single = Encoding::ALL
        .iter()
        .filter(|e| e.family() == Family::SingleByte)
        .count();
    assert_eq!(single, 28, "the legacy single-byte encodings");
}
