//! UTS #46, on the cases the standard and the URL Standard name.
//!
//! The conformance run is `hg-unicode-build`'s, over the pinned
//! `IdnaTestV2.txt`; reading a data file is an ambient effect this
//! crate's tier does not allow. What is here is the behaviour the URL
//! Standard depends on, and the examples UTS #46 writes out itself.

use hg_unicode::idna::{Flags, process, to_ascii, to_unicode};

fn ascii(domain: &str) -> Option<String> {
    to_ascii(domain, Flags::url(false), false)
}

#[test]
fn i01_the_examples_uts46_writes_out() {
    // Table 2's own rows, Nontransitional: ß is kept, not mapped to ss.
    assert_eq!(ascii("fass.de").as_deref(), Some("fass.de"));
    assert_eq!(ascii("fa\u{00DF}.de").as_deref(), Some("xn--fa-hia.de"));
    assert_eq!(ascii("Fa\u{00DF}.de").as_deref(), Some("xn--fa-hia.de"));
    // Already Punycode: validated, not re-mapped.
    assert_eq!(ascii("xn--fa-hia.de").as_deref(), Some("xn--fa-hia.de"));
    // And back.
    assert_eq!(
        to_unicode("xn--fa-hia.de", Flags::url(false)).text,
        "fa\u{00DF}.de"
    );
}

#[test]
fn i02_transitional_processing_maps_the_deviations() {
    // Deprecated, and nothing in Izcal turns it on; it exists because
    // IdnaTestV2 measures it.
    let t = Flags::conformance(true);
    assert_eq!(
        to_ascii("fa\u{00DF}.de", t, false).as_deref(),
        Some("fass.de")
    );
    let n = Flags::conformance(false);
    assert_eq!(
        to_ascii("fa\u{00DF}.de", n, false).as_deref(),
        Some("xn--fa-hia.de")
    );
}

#[test]
fn i03_case_folding_and_full_width_mapping() {
    assert_eq!(ascii("EXAMPLE.COM").as_deref(), Some("example.com"));
    // Fullwidth characters map to their ASCII forms, including the
    // ideographic full stop as a label separator.
    assert_eq!(
        ascii("\u{FF45}\u{FF58}\u{FF41}\u{FF4D}\u{FF50}\u{FF4C}\u{FF45}\u{3002}\u{FF43}\u{FF4F}\u{FF4D}")
            .as_deref(),
        Some("example.com")
    );
}

#[test]
fn i04_an_ignored_code_point_disappears() {
    // U+00AD SOFT HYPHEN is `ignored`, so it is removed rather than
    // encoded. This is why a domain cannot smuggle one past a check.
    assert_eq!(ascii("ex\u{00AD}ample.com").as_deref(), Some("example.com"));
}

#[test]
fn i05_a_disallowed_code_point_fails() {
    // U+FDD0 is a noncharacter and `disallowed` in the mapping table.
    assert_eq!(ascii("\u{FDD0}.com"), None);
    // The conversion still happens, which is what `process` promises
    // even on an error: "Even if an error occurs, the conversion of
    // the string is performed as much as is possible."
    let p = process("\u{FDD0}.com", Flags::url(false));
    assert!(p.error);
    assert_eq!(p.text, "\u{FDD0}.com");
}

#[test]
fn i05a_idna2008_disallowed_is_not_uts46_disallowed() {
    // UTS #46's own example, Section 4.4: "a Unicode string containing
    // a character listed as DISALLOWED in IDNA2008, such as U+2665 (♥)
    // BLACK HEART SUIT, will pass the preprocessing step without an
    // error, but subsequent application of the IDNA2008 processing
    // will fail". The mapping table marks it `valid` with an IDNA2008
    // status of NV8, and UTS #46 goes by the first.
    //
    // This is the sort of thing worth a test rather than a comment: a
    // reader who assumes the two standards agree would write the
    // opposite assertion, which is what happened here first.
    assert_eq!(ascii("\u{2665}.com").as_deref(), Some("xn--g6h.com"));
}

#[test]
fn i06_a_label_may_not_begin_with_a_combining_mark() {
    // UTS #46 4.1 criterion 6.
    assert_eq!(ascii("\u{0300}a.com"), None);
    assert_eq!(ascii("a\u{0300}.com").as_deref(), Some("xn--0ca.com"));
}

#[test]
fn i07_check_hyphens_is_off_for_urls() {
    // The URL Standard turns CheckHyphens off, so a label with hyphens
    // in the third and fourth positions is allowed — unless it is
    // `xn--`, which criterion 4 refuses for exactly that reason.
    assert_eq!(ascii("ab--cd.com").as_deref(), Some("ab--cd.com"));
    assert_eq!(ascii("-a-.com").as_deref(), Some("-a-.com"));
    // A `xn--` label whose Punycode does not decode is an error.
    assert_eq!(ascii("xn--a.com"), None);
}

#[test]
fn i08_verify_dns_length_is_only_applied_when_asked() {
    let long = "a".repeat(64);
    let domain = format!("{long}.com");
    assert_eq!(
        to_ascii(&domain, Flags::url(false), false).as_deref(),
        Some(domain.as_str()),
        "without VerifyDnsLength a 64-character label passes"
    );
    assert_eq!(
        to_ascii(&domain, Flags::url(true), true),
        None,
        "with it, the label is too long"
    );
    // An empty domain fails only with the check on: the URL Standard
    // relies on that to reject `http://`.
    assert_eq!(to_ascii("", Flags::url(false), false).as_deref(), Some(""));
    assert_eq!(to_ascii("", Flags::url(true), true), None);
}

#[test]
fn i08a_the_empty_root_label_is_disallowed_by_verify_dns_length() {
    // A trailing dot leaves an empty root label, and UTS #46 4.2 step
    // 4 is explicit about it: "When VerifyDnsLength is false, the
    // empty root label is passed through. When VerifyDnsLength is
    // true, the empty root label is disallowed."
    //
    // It reads like an oversight — `example.com.` is a perfectly
    // ordinary fully qualified name — so it is written down here. The
    // conformance file agrees: its lines for `a.b.c.d.` expect the
    // `A4_2` status, which is this very check.
    assert_eq!(
        to_ascii("example.com.", Flags::url(false), false).as_deref(),
        Some("example.com.")
    );
    assert_eq!(to_ascii("example.com.", Flags::url(true), true), None);
    assert_eq!(
        to_ascii("example.com", Flags::url(true), true).as_deref(),
        Some("example.com")
    );
}

#[test]
fn i09_the_bidi_conditions_apply_to_the_whole_name() {
    // RFC 5893: a Bidi domain name is one where *some* label holds an
    // R, AL or AN character, and then *every* label must satisfy the
    // conditions. So a perfectly good ASCII label can fail because of
    // its neighbour.
    assert_eq!(ascii("\u{05D0}.com").as_deref(), Some("xn--4db.com"));
    // A right-to-left label that ends in a left-to-right character
    // fails condition 3.
    assert_eq!(ascii("\u{05D0}a"), None);
}

#[test]
fn i10_the_context_j_rules() {
    // RFC 5892 Appendix A. A joiner is allowed after a Virama, and a
    // zero width joiner *only* there.
    assert_eq!(ascii("a\u{200D}b"), None);
    // U+094D DEVANAGARI SIGN VIRAMA has canonical combining class 9,
    // which is what both rules turn on. The expected ASCII is not
    // asserted as a literal — what matters is that it is accepted and
    // comes back unchanged.
    for joiner in ['\u{200C}', '\u{200D}'] {
        let domain = format!("\u{0915}\u{094D}{joiner}\u{0937}");
        let encoded = ascii(&domain).unwrap_or_else(|| panic!("{joiner:?} after a Virama"));
        assert!(encoded.starts_with("xn--"), "{encoded}");
        assert_eq!(to_unicode(&encoded, Flags::url(false)).text, domain);
    }
    // A zero width non-joiner also passes the `(L|D) T* ZWNJ T* (R|D)`
    // clause, which has no Virama in it: Arabic lam-alef.
    let ligature = "\u{0644}\u{200C}\u{0627}";
    assert!(ascii(ligature).is_some(), "the A.1 joining-type clause");
}

#[test]
fn i11_use_std3_ascii_rules_is_what_be_strict_turns_on() {
    // An underscore is `disallowed_STD3_valid`: fine by default, not
    // under STD3 rules.
    assert_eq!(ascii("a_b.com").as_deref(), Some("a_b.com"));
    assert_eq!(to_ascii("a_b.com", Flags::url(true), false), None);
}
