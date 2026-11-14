//! Cases from UAX #15's own prose, so the algorithms are checked against
//! the text and not only against the conformance file.

use hg_unicode::{Form, is_normalized, nfc, nfd, nfkc, nfkd, normalize};

#[test]
fn the_tables_are_the_pinned_version() {
    assert_eq!(hg_unicode::VERSION, "18.0.0");
}

#[test]
fn singletons_and_canonical_pairs() {
    // Angstrom sign decomposes to A with ring, and NFC gives the ring.
    assert_eq!(nfd("\u{212B}"), "A\u{030A}");
    assert_eq!(nfc("\u{212B}"), "\u{00C5}");
    assert_eq!(nfc("A\u{030A}"), "\u{00C5}");
    // A composition exclusion stays decomposed under NFC.
    assert_eq!(nfc("\u{0958}"), "\u{0915}\u{093C}");
}

#[test]
fn compatibility_mappings_only_move_under_the_k_forms() {
    // Vulgar fraction one half.
    assert_eq!(nfc("\u{00BD}"), "\u{00BD}");
    assert_eq!(nfkc("\u{00BD}"), "1\u{2044}2");
    // Fi ligature.
    assert_eq!(nfkd("\u{FB01}"), "fi");
    assert_eq!(nfd("\u{FB01}"), "\u{FB01}");
}

#[test]
fn canonical_ordering_is_by_combining_class_and_stable() {
    // Cedilla (202) must come before acute (230) whatever the input
    // order, and two marks of equal class keep theirs (UAX #15 D108).
    assert_eq!(nfd("q\u{0301}\u{0327}"), "q\u{0327}\u{0301}");
    assert_eq!(nfd("q\u{0327}\u{0301}"), "q\u{0327}\u{0301}");
    assert_eq!(nfc("\u{1E0B}\u{0323}"), "\u{1E0D}\u{0307}");
}

#[test]
fn a_blocked_combining_mark_does_not_compose() {
    // D115: C is blocked from the last starter L when a character
    // between them has class zero or a class not lower than C's.
    //
    // Overline and acute both have class 230, so canonical ordering
    // leaves them as written and the overline blocks the acute: `a`
    // keeps its own form. Drop the blocking rule and the acute reaches
    // the `a` over the overline's head, giving U+00E1 — which is why
    // this case is here and `A` + dot below + ring above is not: that
    // pair agrees whether or not blocking is applied.
    assert_eq!(nfc("a\u{0305}\u{0301}"), "a\u{0305}\u{0301}");
    // The same marks with nothing in the way do compose.
    assert_eq!(nfc("a\u{0301}"), "\u{00E1}");
    // A lower class does not block: the overlay (1) lets the acute
    // (230) through to the `a`.
    assert_eq!(nfc("a\u{0334}\u{0301}"), "\u{00E1}\u{0334}");
}

#[test]
fn hangul_is_arithmetic_in_both_directions() {
    assert_eq!(nfd("\u{AC00}"), "\u{1100}\u{1161}");
    assert_eq!(nfd("\u{AC01}"), "\u{1100}\u{1161}\u{11A8}");
    assert_eq!(nfc("\u{1100}\u{1161}"), "\u{AC00}");
    assert_eq!(nfc("\u{1100}\u{1161}\u{11A8}"), "\u{AC01}");
    // A trailing consonant with no syllable before it stays put.
    assert_eq!(nfc("\u{11A8}"), "\u{11A8}");
}

#[test]
fn every_form_is_idempotent_on_its_own_output() {
    for s in [
        "\u{1E0B}\u{0323}",
        "\u{FB01}",
        "\u{AC01}",
        "A\u{0323}\u{030A}",
        "\u{212B}",
    ] {
        for form in [Form::Nfc, Form::Nfd, Form::Nfkc, Form::Nfkd] {
            let once = normalize(s, form);
            assert_eq!(normalize(&once, form), once, "{s:?} under {form:?}");
        }
    }
}

#[test]
fn the_quick_check_never_says_yes_to_a_string_that_moves() {
    for s in [
        "\u{1E0B}\u{0323}",
        "\u{FB01}",
        "\u{212B}",
        "A\u{030A}",
        "q\u{0301}\u{0327}",
        "plain ascii",
        "\u{AC00}",
    ] {
        for form in [Form::Nfc, Form::Nfd, Form::Nfkc, Form::Nfkd] {
            if is_normalized(s, form) {
                assert_eq!(
                    normalize(s, form),
                    s,
                    "quick check said yes: {s:?} {form:?}"
                );
            }
        }
    }
}
