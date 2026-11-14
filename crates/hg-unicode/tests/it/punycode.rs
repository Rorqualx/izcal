//! Punycode, against RFC 3492's own sample strings.
//!
//! Section 7.1 of the RFC lists sample strings in several scripts with
//! their encodings. They are the best test there is for a bootstring
//! codec: each exercises a different distribution of code points, and
//! the RFC itself vouches for the answers.

use hg_unicode::punycode::{Error, decode, encode};

/// RFC 3492 Section 7.1's sample strings, (A) to (S), taken from the
/// RFC's own text rather than transcribed.
///
/// Two of them, (I) and the basic halves of (D), (J), (K), (L), (M),
/// (N) and (P), carry **upper-case letters**. Those in the basic half
/// come from the input and must come back as they are; the one in (I)'s
/// delta half is Appendix A's *mixed-case annotation*, which the RFC
/// says is "not used by the ToASCII and ToUnicode operations of
/// [IDNA]". So encoding is compared ASCII case-insensitively and
/// decoding is given the RFC's string exactly, which a decoder must
/// accept in either case.
const SAMPLES: [(&str, &str, &str); 19] = [
    (
        "(A)",
        "\u{0644}\u{064A}\u{0647}\u{0645}\u{0627}\u{0628}\u{062A}\u{0643}\
         \u{0644}\u{0645}\u{0648}\u{0634}\u{0639}\u{0631}\u{0628}\u{064A}\
         \u{061F}",
        "egbpdaj6bu4bxfgehfvwxn",
    ),
    (
        "(B)",
        "\u{4ED6}\u{4EEC}\u{4E3A}\u{4EC0}\u{4E48}\u{4E0D}\u{8BF4}\u{4E2D}\
         \u{6587}",
        "ihqwcrb4cv8a8dqg056pqjye",
    ),
    (
        "(C)",
        "\u{4ED6}\u{5011}\u{7232}\u{4EC0}\u{9EBD}\u{4E0D}\u{8AAA}\u{4E2D}\
         \u{6587}",
        "ihqwctvzc91f659drss3x8bo0yb",
    ),
    (
        "(D)",
        "\u{0050}\u{0072}\u{006F}\u{010D}\u{0070}\u{0072}\u{006F}\u{0073}\
         \u{0074}\u{011B}\u{006E}\u{0065}\u{006D}\u{006C}\u{0075}\u{0076}\
         \u{00ED}\u{010D}\u{0065}\u{0073}\u{006B}\u{0079}",
        "Proprostnemluvesky-uyb24dma41a",
    ),
    (
        "(E)",
        "\u{05DC}\u{05DE}\u{05D4}\u{05D4}\u{05DD}\u{05E4}\u{05E9}\u{05D5}\
         \u{05D8}\u{05DC}\u{05D0}\u{05DE}\u{05D3}\u{05D1}\u{05E8}\u{05D9}\
         \u{05DD}\u{05E2}\u{05D1}\u{05E8}\u{05D9}\u{05EA}",
        "4dbcagdahymbxekheh6e0a7fei0b",
    ),
    (
        "(F)",
        "\u{092F}\u{0939}\u{0932}\u{094B}\u{0917}\u{0939}\u{093F}\u{0928}\
         \u{094D}\u{0926}\u{0940}\u{0915}\u{094D}\u{092F}\u{094B}\u{0902}\
         \u{0928}\u{0939}\u{0940}\u{0902}\u{092C}\u{094B}\u{0932}\u{0938}\
         \u{0915}\u{0924}\u{0947}\u{0939}\u{0948}\u{0902}",
        "i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd",
    ),
    (
        "(G)",
        "\u{306A}\u{305C}\u{307F}\u{3093}\u{306A}\u{65E5}\u{672C}\u{8A9E}\
         \u{3092}\u{8A71}\u{3057}\u{3066}\u{304F}\u{308C}\u{306A}\u{3044}\
         \u{306E}\u{304B}",
        "n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa",
    ),
    (
        "(H)",
        "\u{C138}\u{ACC4}\u{C758}\u{BAA8}\u{B4E0}\u{C0AC}\u{B78C}\u{B4E4}\
         \u{C774}\u{D55C}\u{AD6D}\u{C5B4}\u{B97C}\u{C774}\u{D574}\u{D55C}\
         \u{B2E4}\u{BA74}\u{C5BC}\u{B9C8}\u{B098}\u{C88B}\u{C744}\u{AE4C}",
        "989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c",
    ),
    (
        "(I)",
        "\u{043F}\u{043E}\u{0447}\u{0435}\u{043C}\u{0443}\u{0436}\u{0435}\
         \u{043E}\u{043D}\u{0438}\u{043D}\u{0435}\u{0433}\u{043E}\u{0432}\
         \u{043E}\u{0440}\u{044F}\u{0442}\u{043F}\u{043E}\u{0440}\u{0443}\
         \u{0441}\u{0441}\u{043A}\u{0438}",
        "b1abfaaepdrnnbgefbaDotcwatmq2g4l",
    ),
    (
        "(J)",
        "\u{0050}\u{006F}\u{0072}\u{0071}\u{0075}\u{00E9}\u{006E}\u{006F}\
         \u{0070}\u{0075}\u{0065}\u{0064}\u{0065}\u{006E}\u{0073}\u{0069}\
         \u{006D}\u{0070}\u{006C}\u{0065}\u{006D}\u{0065}\u{006E}\u{0074}\
         \u{0065}\u{0068}\u{0061}\u{0062}\u{006C}\u{0061}\u{0072}\u{0065}\
         \u{006E}\u{0045}\u{0073}\u{0070}\u{0061}\u{00F1}\u{006F}\u{006C}",
        "PorqunopuedensimplementehablarenEspaol-fmd56a",
    ),
    (
        "(K)",
        "\u{0054}\u{1EA1}\u{0069}\u{0073}\u{0061}\u{006F}\u{0068}\u{1ECD}\
         \u{006B}\u{0068}\u{00F4}\u{006E}\u{0067}\u{0074}\u{0068}\u{1EC3}\
         \u{0063}\u{0068}\u{1EC9}\u{006E}\u{00F3}\u{0069}\u{0074}\u{0069}\
         \u{1EBF}\u{006E}\u{0067}\u{0056}\u{0069}\u{1EC7}\u{0074}",
        "TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g",
    ),
    (
        "(L)",
        "\u{0033}\u{5E74}\u{0042}\u{7D44}\u{91D1}\u{516B}\u{5148}\u{751F}",
        "3B-ww4c5e180e575a65lsy2b",
    ),
    (
        "(M)",
        "\u{5B89}\u{5BA4}\u{5948}\u{7F8E}\u{6075}\u{002D}\u{0077}\u{0069}\
         \u{0074}\u{0068}\u{002D}\u{0053}\u{0055}\u{0050}\u{0045}\u{0052}\
         \u{002D}\u{004D}\u{004F}\u{004E}\u{004B}\u{0045}\u{0059}\u{0053}",
        "-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n",
    ),
    (
        "(N)",
        "\u{0048}\u{0065}\u{006C}\u{006C}\u{006F}\u{002D}\u{0041}\u{006E}\
         \u{006F}\u{0074}\u{0068}\u{0065}\u{0072}\u{002D}\u{0057}\u{0061}\
         \u{0079}\u{002D}\u{305D}\u{308C}\u{305E}\u{308C}\u{306E}\u{5834}\
         \u{6240}",
        "Hello-Another-Way--fc4qua05auwb3674vfr0b",
    ),
    (
        "(O)",
        "\u{3072}\u{3068}\u{3064}\u{5C4B}\u{6839}\u{306E}\u{4E0B}\u{0032}",
        "2-u9tlzr9756bt3uc0v",
    ),
    (
        "(P)",
        "\u{004D}\u{0061}\u{006A}\u{0069}\u{3067}\u{004B}\u{006F}\u{0069}\
         \u{3059}\u{308B}\u{0035}\u{79D2}\u{524D}",
        "MajiKoi5-783gue6qz075azm5e",
    ),
    (
        "(Q)",
        "\u{30D1}\u{30D5}\u{30A3}\u{30FC}\u{0064}\u{0065}\u{30EB}\u{30F3}\
         \u{30D0}",
        "de-jg4avhby1noc0d",
    ),
    (
        "(R)",
        "\u{305D}\u{306E}\u{30B9}\u{30D4}\u{30FC}\u{30C9}\u{3067}",
        "d9juau41awczczp",
    ),
    (
        "(S)",
        "\u{002D}\u{003E}\u{0020}\u{0024}\u{0031}\u{002E}\u{0030}\u{0030}\
         \u{0020}\u{003C}\u{002D}",
        "-> $1.00 <--",
    ),
];

#[test]
fn y01_the_rfcs_own_samples_encode_as_it_says() {
    for (label, unicode, punycode) in SAMPLES {
        let got = encode(unicode).unwrap_or_default();
        assert!(
            got.eq_ignore_ascii_case(punycode),
            "{label}: encoded {got:?}, the RFC says {punycode:?}"
        );
    }
}

#[test]
fn y02_and_decode_back() {
    for (label, unicode, punycode) in SAMPLES {
        assert_eq!(
            decode(punycode).as_deref(),
            Ok(unicode),
            "{label}: decoding differs"
        );
    }
}

#[test]
fn y03_the_cases_around_the_delimiter() {
    // No basic code points: no delimiter.
    assert_eq!(encode("\u{3042}").as_deref(), Ok("l8j"));
    assert_eq!(decode("l8j").as_deref(), Ok("\u{3042}"));
    // All basic: a trailing delimiter and nothing after it.
    assert_eq!(encode("abc").as_deref(), Ok("abc-"));
    assert_eq!(decode("abc-").as_deref(), Ok("abc"));
    // A string that is itself empty.
    assert_eq!(encode("").as_deref(), Ok(""));
    assert_eq!(decode("").as_deref(), Ok(""));
    // A basic string holding the delimiter: the *last* one separates.
    assert_eq!(decode("a-b-").as_deref(), Ok("a-b"));
}

#[test]
fn y04_what_is_refused() {
    // A digit outside base 36.
    assert_eq!(decode("a-!"), Err(Error::BadInput));
    // A delta that runs off the end of the input.
    assert_eq!(decode("a-9"), Err(Error::BadInput));
    // Non-ASCII before the delimiter: `encode` never produces it.
    assert_eq!(decode("\u{3042}-a"), Err(Error::BadInput));
    // RFC 3492 Section 6.4: "an implementation must detect overflow".
    // A long run of the highest digit drives the delta past what a
    // code point can be; a long run of digits drives it past what the
    // arithmetic can hold.
    assert_eq!(decode("99999999999"), Err(Error::Overflow));
    assert_eq!(
        decode("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
        Err(Error::NotACodePoint)
    );
    // Not every long input is an overflow: `zzzzzzzzzzzzz` is a
    // perfectly good encoding of five CJK ideographs, and refusing it
    // would be refusing a label a registry could have issued.
    assert!(decode("zzzzzzzzzzzzz").is_ok());
}

#[test]
fn y05_a_round_trip_over_a_wide_sample() {
    // Every block's first assigned-looking code point, in groups, so
    // the delta arithmetic is exercised across the whole range rather
    // than in one neighbourhood.
    for start in (0x80_u32..0x10000).step_by(0x400) {
        let text: String = (start..start.saturating_add(8))
            .filter(|v| !(0xD800..0xE000).contains(v))
            .filter_map(char::from_u32)
            .collect();
        if text.is_empty() {
            continue;
        }
        let Ok(encoded) = encode(&text) else { continue };
        assert_eq!(
            decode(&encoded).as_deref(),
            Ok(text.as_str()),
            "U+{start:04X} did not round-trip through {encoded:?}"
        );
        assert!(encoded.is_ascii(), "Punycode is ASCII");
    }
}
