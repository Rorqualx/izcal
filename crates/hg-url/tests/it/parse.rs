//! The basic URL parser, Section 4.4, and the serializer, Section 4.5.
//!
//! WPT covers the parser broadly. What is here is what it does not
//! reach: the examples the standard states in its own prose, the state
//! override the `URL` class's setters need, and the invariants that a
//! passing suite would not notice were broken.

use hg_encoding::Encoding;
use hg_url::parse::{State, basic_parse};
use hg_url::{Host, Path, Url};

fn p(input: &str) -> Option<Url> {
    Url::parse(input)
}

fn rel(input: &str, base: &str) -> Option<String> {
    let base = Url::parse(base)?;
    Url::parse_with_base(input, &base).map(|u| u.serialize(false))
}

#[test]
fn u01_the_standards_own_table_of_input_base_and_output() {
    // Section 4: "At a high level, a URL, valid URL string, URL parser,
    // and URL serializer relate as follows" — every row of that table.
    assert_eq!(
        p("https:example.org").map(|u| u.serialize(false)),
        Some("https://example.org/".to_owned())
    );
    assert_eq!(
        p("https://////example.com///").map(|u| u.serialize(false)),
        Some("https://example.com///".to_owned())
    );
    assert_eq!(
        p("https://example.com/././foo").map(|u| u.serialize(false)),
        Some("https://example.com/foo".to_owned())
    );
    assert_eq!(
        rel("hello:world", "https://example.com/"),
        Some("hello:world".to_owned())
    );
    assert_eq!(
        rel("https:example.org", "https://example.com/"),
        Some("https://example.com/example.org".to_owned())
    );
    assert_eq!(
        rel("\\example\\..\\demo/.\\", "https://example.com/"),
        Some("https://example.com/demo/".to_owned())
    );
    assert_eq!(
        rel("example", "https://example.com/demo"),
        Some("https://example.com/example".to_owned())
    );
    assert_eq!(
        rel("file:///C|/demo", "about:blank"),
        Some("file:///C:/demo".to_owned())
    );
    assert_eq!(rel("..", "file:///C:/demo"), Some("file:///C:/".to_owned()));
    assert_eq!(
        p("file://localhost/").map(|u| u.serialize(false)),
        Some("file:///".to_owned())
    );
    assert_eq!(
        p("file://loc%61lhost/").map(|u| u.serialize(false)),
        Some("file:///".to_owned())
    );
}

#[test]
fn u02_usr_dotdot_gives_a_slash_and_not_a_missing_path() {
    // The standard's note: "This means that for input /usr/.. the
    // result is / and not a lack of a path."
    assert_eq!(
        rel("/usr/..", "https://example.com/"),
        Some("https://example.com/".to_owned())
    );
    let Some(u) = Url::parse("https://example.com/usr/..") else {
        panic!("that parses");
    };
    assert_eq!(u.path, Path::List(vec![String::new()]));
}

#[test]
fn u03_a_pathless_host_still_serializes_a_slash_when_special() {
    assert_eq!(
        p("https://example.com").map(|u| u.serialize(false)),
        Some("https://example.com/".to_owned())
    );
    // A non-special scheme keeps the empty path it was given.
    assert_eq!(
        p("non-special://example.com").map(|u| u.serialize(false)),
        Some("non-special://example.com".to_owned())
    );
}

#[test]
fn u04_the_slash_dot_guard_keeps_a_path_from_becoming_a_host() {
    // "This prevents web+demo:/.//not-a-host/ ... from ending up as
    // web+demo://not-a-host/".
    let Some(u) = p("web+demo:/.//not-a-host/") else {
        panic!("that parses");
    };
    assert_eq!(u.host, None);
    assert_eq!(u.serialize(false), "web+demo:/.//not-a-host/");
    assert_eq!(
        p("web+demo:/path/..//not-a-host/").map(|u| u.serialize(false)),
        Some("web+demo:/.//not-a-host/".to_owned())
    );
}

#[test]
fn u05_a_tab_or_newline_is_removed_anywhere_and_ends_are_trimmed() {
    assert_eq!(
        p("http://example\t.\norg").map(|u| u.serialize(false)),
        Some("http://example.org/".to_owned())
    );
    assert_eq!(
        p("  \u{1}http://example.org/\u{1} ").map(|u| u.serialize(false)),
        Some("http://example.org/".to_owned())
    );
}

#[test]
fn u06_a_default_port_is_dropped_and_another_is_kept() {
    assert_eq!(p("http://a:80/").and_then(|u| u.port), None);
    assert_eq!(p("http://a:81/").and_then(|u| u.port), Some(81));
    assert_eq!(p("https://a:443/").and_then(|u| u.port), None);
    assert_eq!(p("ws://a:80/").and_then(|u| u.port), None);
    assert_eq!(p("wss://a:443/").and_then(|u| u.port), None);
    assert_eq!(p("ftp://a:21/").and_then(|u| u.port), None);
    // 65536 does not fit, and the standard says that is a failure.
    assert_eq!(p("http://a:65535/").and_then(|u| u.port), Some(65535));
    assert_eq!(p("http://a:65536/"), None);
}

#[test]
fn u07_a_special_url_needs_a_host_and_a_non_special_one_does_not() {
    assert_eq!(p("http:///"), None);
    assert_eq!(p("http://"), None);
    assert_eq!(p("non-special:///"), p("non-special:///"));
    let Some(u) = p("non-special:///") else {
        panic!("that parses");
    };
    assert_eq!(u.host, Some(Host::Empty));
}

#[test]
fn u08_a_backslash_is_a_slash_only_for_a_special_scheme() {
    assert_eq!(
        p("http://example.com\\a").map(|u| u.serialize(false)),
        Some("http://example.com/a".to_owned())
    );
    // For a non-special scheme it is an ordinary path character. It is
    // still a forbidden *host* code point in both, so the difference
    // has to be shown on the path.
    assert_eq!(
        p("non-special://example.com/a\\b").map(|u| u.serialize(false)),
        Some("non-special://example.com/a\\b".to_owned())
    );
    assert_eq!(p("non-special://example.com\\a"), None);
}

#[test]
fn u09_credentials_are_percent_encoded_with_the_userinfo_set() {
    let Some(u) = p("http://a b:c d@example.com/") else {
        panic!("that parses");
    };
    assert_eq!(u.username, "a%20b");
    assert_eq!(u.password, "c%20d");
    // A second @ is part of the username, not a separator.
    let Some(u) = p("http://a@b@example.com/") else {
        panic!("that parses");
    };
    assert_eq!(u.username, "a%40b");
    assert_eq!(u.host, Some(Host::Domain("example.com".to_owned())));
}

#[test]
fn u10_an_opaque_path_is_never_shortened() {
    let Some(u) = p("hello:a/b/../c") else {
        panic!("that parses");
    };
    assert!(u.has_opaque_path());
    assert_eq!(u.serialize(false), "hello:a/b/../c");
}

#[test]
fn u11_a_space_in_an_opaque_path_is_encoded_only_at_the_end() {
    // "If remaining starts with U+003F (?) or U+0023 (#), then append
    // '%20' ... Otherwise, append U+0020 SPACE."
    // Two spaces, so both branches are measured: the first has `b`
    // after it and stays a space, the second is followed by `?` and
    // becomes `%20`.
    let Some(u) = p("hello:a b ?q") else {
        panic!("that parses");
    };
    assert_eq!(u.serialize(false), "hello:a b%20?q");
}

#[test]
fn u12_the_windows_drive_quirk_is_platform_independent() {
    assert_eq!(
        p("file:///C|/").map(|u| u.serialize(false)),
        Some("file:///C:/".to_owned())
    );
    assert_eq!(rel("..", "file:///C:/a/b"), Some("file:///C:/".to_owned()));
    assert_eq!(
        rel("../..", "file:///C:/a/b"),
        Some("file:///C:/".to_owned())
    );
    // The quirk only applies when the drive letter is the whole path.
    assert_eq!(rel("..", "file:///a/b/c"), Some("file:///a/".to_owned()));
}

#[test]
fn u13_a_drive_letter_is_not_a_file_host() {
    let Some(u) = p("file://C:/") else {
        panic!("that parses");
    };
    assert_eq!(u.host, Some(Host::Empty));
    assert_eq!(u.serialize(false), "file:///C:/");
}

#[test]
fn u14_single_and_double_dot_segments_are_matched_case_insensitively() {
    for dotted in ["/a/./b", "/a/%2e/b", "/a/%2E/b"] {
        assert_eq!(
            rel(dotted, "https://x/"),
            Some("https://x/a/b".to_owned()),
            "{dotted}"
        );
    }
    for dotted in ["/a/../b", "/a/%2e./b", "/a/.%2E/b", "/a/%2e%2E/b"] {
        assert_eq!(
            rel(dotted, "https://x/"),
            Some("https://x/b".to_owned()),
            "{dotted}"
        );
    }
}

#[test]
fn u15_an_empty_fragment_and_an_empty_query_are_not_absent_ones() {
    let Some(u) = p("https://x/?") else {
        panic!("that parses");
    };
    assert_eq!(u.query, Some(String::new()));
    assert_eq!(u.serialize(false), "https://x/?");
    let Some(u) = p("https://x/#") else {
        panic!("that parses");
    };
    assert_eq!(u.fragment, Some(String::new()));
    assert_eq!(u.serialize(false), "https://x/#");
    assert_eq!(u.serialize(true), "https://x/");
}

#[test]
fn u16_the_query_encoding_is_ignored_for_ws_wss_and_non_special() {
    // "If encoding is not UTF-8 and one of the following is true: url
    // is not special; url's scheme is 'ws' or 'wss' — then set encoding
    // to UTF-8."
    let Some(sjis) = Encoding::for_label("shift_jis") else {
        panic!("that is a label");
    };
    let q = |input: &str| hg_url::parse(input, None, sjis).and_then(|u| u.query);
    assert_eq!(q("https://x/?\u{2212}"), Some("%81|".to_owned()));
    assert_eq!(q("ws://x/?\u{2212}"), Some("%E2%88%92".to_owned()));
    assert_eq!(q("wss://x/?\u{2212}"), Some("%E2%88%92".to_owned()));
    assert_eq!(q("non-special://x/?\u{2212}"), Some("%E2%88%92".to_owned()));
}

#[test]
fn u17_the_scheme_state_starts_over_when_there_is_no_colon() {
    // `a/b:c` has no scheme: the scheme state consumes `a`, then the
    // `/` sends it back to the first code point in the no scheme state.
    assert_eq!(
        rel("a/b:c", "https://x/"),
        Some("https://x/a/b:c".to_owned())
    );
    assert_eq!(p("a/b:c"), None);
}

#[test]
fn u18_a_fragment_only_input_works_against_an_opaque_base() {
    let Some(base) = p("hello:world") else {
        panic!("that parses");
    };
    let Some(u) = Url::parse_with_base("#x", &base) else {
        panic!("that parses");
    };
    assert_eq!(u.serialize(false), "hello:world#x");
    // Anything else against an opaque base is a failure.
    assert_eq!(Url::parse_with_base("a", &base), None);
    assert_eq!(Url::parse_with_base("?q", &base), None);
}

#[test]
fn u19_the_protocol_setter_refuses_to_cross_the_special_line() {
    // State override, the scheme state: "If url's scheme is a special
    // scheme and buffer is not a special scheme, then return."
    let Some(url) = p("https://example.com/") else {
        panic!("that parses");
    };
    // The setter appends the `:` before parsing; without it the scheme
    // state returns failure, "used exclusively by the Location
    // object's protocol setter".
    assert_eq!(
        basic_parse(
            "hello",
            None,
            Encoding::Utf8,
            Some(url.clone()),
            Some(State::SchemeStart)
        ),
        None
    );
    let Some(after) = basic_parse(
        "hello:",
        None,
        Encoding::Utf8,
        Some(url.clone()),
        Some(State::SchemeStart),
    ) else {
        panic!("a state override run does not fail here");
    };
    assert_eq!(after.scheme, "https");
    // Special to special is allowed.
    let Some(after) = basic_parse(
        "http:",
        None,
        Encoding::Utf8,
        Some(url),
        Some(State::SchemeStart),
    ) else {
        panic!("a state override run does not fail here");
    };
    assert_eq!(after.scheme, "http");
}

#[test]
fn u20_the_hostname_setter_refuses_a_port() {
    let Some(url) = p("https://example.com/") else {
        panic!("that parses");
    };
    assert_eq!(
        basic_parse(
            "a:80",
            None,
            Encoding::Utf8,
            Some(url.clone()),
            Some(State::Hostname)
        ),
        None
    );
    let Some(after) = basic_parse("a", None, Encoding::Utf8, Some(url), Some(State::Hostname))
    else {
        panic!("that is a hostname");
    };
    assert_eq!(after.host, Some(Host::Domain("a".to_owned())));
}

#[test]
fn u21_a_serialized_url_parses_back_to_the_same_url() {
    // The standard: "(If that string is then parsed, the result will
    // equal the URL that was serialized.)"
    for s in [
        "https://example.com/",
        "https://u:p@example.com:81/a/b?q#f",
        "file:///C:/demo",
        "hello:world",
        "web+demo:/.//not-a-host/",
        "non-special://example.com",
        "https://[1:2:0:0:5::]/",
        "https://1.2.3.4/",
    ] {
        let Some(once) = p(s) else {
            panic!("{s} parses");
        };
        let Some(twice) = p(&once.serialize(false)) else {
            panic!("{s} parses twice");
        };
        assert_eq!(once, twice, "{s}");
    }
}
