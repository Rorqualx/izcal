//! Hosts, Sections 3.2 to 3.7.

use hg_url::host::{
    Host, compressed_piece_index, ends_in_a_number, forbidden_domain, forbidden_host, parse,
    parse_ipv4, parse_ipv4_number, parse_ipv6, serialize_ipv4, serialize_ipv6,
};

#[test]
fn h01_the_forbidden_sets_are_the_standards_two_lists() {
    for c in [
        '\0', '\t', '\n', '\r', ' ', '#', '/', ':', '<', '>', '?', '@', '[', '\\', ']', '^', '|',
    ] {
        assert!(forbidden_host(c), "{c:?} is a forbidden host code point");
        assert!(
            forbidden_domain(c),
            "{c:?} is a forbidden domain code point"
        );
    }
    // A forbidden domain code point is a superset: C0 controls, % and
    // DELETE are forbidden in a domain but not in an opaque host.
    for c in ['\u{1}', '%', '\u{7F}'] {
        assert!(!forbidden_host(c), "{c:?} is allowed in an opaque host");
        assert!(forbidden_domain(c), "{c:?} is forbidden in a domain");
    }
}

#[test]
fn h02_ends_in_a_number_is_about_the_last_label_not_the_whole_host() {
    assert!(ends_in_a_number("1.2.3.4"));
    assert!(ends_in_a_number("a.1"));
    assert!(ends_in_a_number("1.2.3.4."));
    // "0x" and "0x" + hex digits count, which is the note's own test.
    assert!(ends_in_a_number("a.0x"));
    assert!(ends_in_a_number("a.0xff"));
    assert!(!ends_in_a_number("example.com"));
    assert!(!ends_in_a_number("1.2.3.4a"));
    assert!(!ends_in_a_number("a.0xzz"));
}

#[test]
fn h03_the_erroneous_09_is_caught_later_not_here() {
    // The standard's own note: "The erroneous input '09' will be
    // caught by the IPv4 parser at a later stage."
    assert!(ends_in_a_number("a.09"));
    assert_eq!(parse_ipv4("a.09"), None);
}

#[test]
fn h04_the_ipv4_number_parser_reads_three_radixes() {
    assert_eq!(parse_ipv4_number("10"), Some((10, false)));
    assert_eq!(parse_ipv4_number("0x10"), Some((16, true)));
    assert_eq!(parse_ipv4_number("0X10"), Some((16, true)));
    assert_eq!(parse_ipv4_number("010"), Some((8, true)));
    assert_eq!(parse_ipv4_number("0"), Some((0, false)));
    assert_eq!(parse_ipv4_number("0x"), Some((0, true)));
    assert_eq!(parse_ipv4_number("09"), None);
    assert_eq!(parse_ipv4_number(""), None);
}

#[test]
fn h05_a_number_too_large_for_a_machine_word_still_fails() {
    // The standard speaks of a mathematical integer. Saturating gives
    // the same answer because every use is a comparison with a bound.
    let huge = "9".repeat(40);
    assert_eq!(parse_ipv4(&huge), None);
    assert_eq!(parse_ipv4(&format!("0x{}", "f".repeat(40))), None);
}

#[test]
fn h06_fewer_than_four_parts_fill_from_the_last() {
    assert_eq!(parse_ipv4("1.2"), Some(0x0100_0002));
    assert_eq!(serialize_ipv4(0x0100_0002), "1.0.0.2");
    assert_eq!(parse_ipv4("1.2.3.4"), Some(0x0102_0304));
    assert_eq!(parse_ipv4("0x7f000001"), Some(0x7F00_0001));
    // More than four is a failure; the last part's bound is 256^(5−n),
    // so with three parts the last may be up to 65,535 — `1.2.256` is
    // `1.2.1.0` and not an error, which is the rule that makes
    // `0x7f000001` work at all.
    assert_eq!(parse_ipv4("1.2.3.4.5"), None);
    assert_eq!(parse_ipv4("1.2.256"), Some(0x0102_0100));
    assert_eq!(serialize_ipv4(0x0102_0100), "1.2.1.0");
    assert_eq!(parse_ipv4("1.2.65536"), None);
    // Only the *last* part may exceed 255.
    assert_eq!(parse_ipv4("256.1.1.1"), None);
}

#[test]
fn h07_ipv6_compression_takes_the_longest_run_not_the_first() {
    let Some(a) = parse_ipv6("1:2:0:0:5:0:0:0") else {
        panic!("that is an IPv6 address");
    };
    assert_eq!(compressed_piece_index(&a), Some(5));
    assert_eq!(serialize_ipv6(&a), "1:2:0:0:5::");
    // The standard's own example: in 0:f:0:0:f:f:0:0 it points to the
    // second 0 — index 2, because the trailing run is no longer.
    let Some(b) = parse_ipv6("0:f:0:0:f:f:0:0") else {
        panic!("that is an IPv6 address");
    };
    assert_eq!(compressed_piece_index(&b), Some(2));
    assert_eq!(serialize_ipv6(&b), "0:f::f:f:0:0");
}

#[test]
fn h08_a_single_zero_piece_is_never_compressed() {
    // `longestSize` starts at 1, so one zero is written as `0`.
    let Some(a) = parse_ipv6("1:0:2:3:4:5:6:7") else {
        panic!("that is an IPv6 address");
    };
    assert_eq!(compressed_piece_index(&a), None);
    assert_eq!(serialize_ipv6(&a), "1:0:2:3:4:5:6:7");
}

#[test]
fn h09_ipv4_inside_ipv6_fills_two_pieces() {
    let Some(a) = parse_ipv6("::1.2.3.4") else {
        panic!("that is an IPv6 address");
    };
    assert_eq!(a, [0, 0, 0, 0, 0, 0, 0x0102, 0x0304]);
    assert_eq!(serialize_ipv6(&a), "::102:304");
    assert_eq!(parse_ipv6("::1.2.3"), None);
    assert_eq!(parse_ipv6("::1.2.3.4.5"), None);
    assert_eq!(parse_ipv6("::1.2.3.256"), None);
    // A leading zero in an IPv4 part inside IPv6 is a failure, not an
    // octal number: the inner parser is not the IPv4 parser.
    assert_eq!(parse_ipv6("::1.2.3.01"), None);
}

#[test]
fn h10_only_one_compression_is_allowed() {
    assert!(parse_ipv6("1::2::3").is_none());
    assert!(parse_ipv6(":1").is_none());
    assert!(parse_ipv6("1:").is_none());
    assert!(parse_ipv6("1:2:3:4:5:6:7:8:9").is_none());
    assert!(parse_ipv6("1:2:3:4:5:6:7").is_none());
}

#[test]
fn h11_the_host_parser_chooses_between_domain_address_and_opaque() {
    assert_eq!(
        parse("example.com", false),
        Some(Host::Domain("example.com".to_owned()))
    );
    assert_eq!(
        parse("EXAMPLE.com", false),
        Some(Host::Domain("example.com".to_owned()))
    );
    assert_eq!(parse("1.2.3.4", false), Some(Host::Ipv4(0x0102_0304)));
    assert_eq!(
        parse("[::1]", false),
        Some(Host::Ipv6([0, 0, 0, 0, 0, 0, 0, 1]))
    );
    // The same string, with `isOpaque`, is left alone.
    assert_eq!(
        parse("1.2.3.4", true),
        Some(Host::Opaque("1.2.3.4".to_owned()))
    );
    assert_eq!(
        parse("EXAMPLE.com", true),
        Some(Host::Opaque("EXAMPLE.com".to_owned()))
    );
}

#[test]
fn h12_an_unclosed_bracket_is_a_failure_in_both_modes() {
    assert_eq!(parse("[::1", false), None);
    assert_eq!(parse("[::1", true), None);
}

#[test]
fn h13_idna_runs_on_the_percent_decoded_domain() {
    // `%C3%A9` is `é`, which UTS 46 maps to its Punycode.
    assert_eq!(
        parse("%C3%A9.com", false),
        Some(Host::Domain("xn--9ca.com".to_owned()))
    );
    assert_eq!(
        parse("é.com", false),
        Some(Host::Domain("xn--9ca.com".to_owned()))
    );
}

#[test]
fn h14_a_host_serializes_back_to_what_it_was_read_from() {
    for s in ["example.com", "1.2.3.4", "[::1]", "[1:2:0:0:5::]"] {
        let Some(h) = parse(s, false) else {
            panic!("{s} is a host");
        };
        assert_eq!(h.serialize(), s, "{s}");
    }
}

#[test]
fn h15_an_ascii_domain_is_lower_cased_whatever_uts46_thinks() {
    // Section 3.3's note: when `beStrict` is false "the algorithm
    // returns domain lowercased regardless of Unicode ToASCII's
    // outcome, due to web compatibility", because "Punycode can decode
    // successfully yet still fail validity criteria".
    //
    // WPT's toascii.json pins these three, the first commented
    // "Invalid Punycode" and the third "U+FFFD character encoded in
    // Punycode". All three are hosts.
    for s in ["xn--a", "xn--0.com", "xn--zn7c.com"] {
        assert_eq!(parse(s, false), Some(Host::Domain(s.to_owned())), "{s}");
    }
    // A non-ASCII domain does go through UTS 46, and `xn--a.ß` fails
    // there — which is what the suite expects.
    assert_eq!(parse("xn--a.ß", false), None);
    // The two checks that still apply to the ASCII path.
    assert_eq!(parse("", false), Some(Host::Empty));
    assert_eq!(parse("a%b.com", false), None);
}
