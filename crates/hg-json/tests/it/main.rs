//! `hg-json`'s tests: the shapes its inputs actually take, and the two
//! decisions a reader of JSON has to make for itself.
//!
//! The inputs are WHATWG's `entities.json`, html5lib-tests' `.test`
//! files and `@webref/css`'s `css.json`, so the cases here are drawn
//! from what those contain: deep nesting, long arrays of objects,
//! numbers that must stay exact, and escapes.

#![expect(clippy::unwrap_used, reason = "tests: a failed unwrap is the failure")]

use hg_json::{Json, parse};

fn obj(text: &str) -> Vec<(String, Json)> {
    match parse(text).unwrap() {
        Json::Obj(pairs) => pairs,
        other => panic!("not an object: {other:?}"),
    }
}

#[test]
fn j01_the_scalars() {
    assert_eq!(parse("null").unwrap(), Json::Null);
    assert_eq!(parse("true").unwrap(), Json::Bool(true));
    assert_eq!(parse("false").unwrap(), Json::Bool(false));
    assert_eq!(parse(r#""x""#).unwrap(), Json::Str("x".to_owned()));
}

#[test]
fn j02_a_number_keeps_its_text() {
    // The UCD and webref both carry integers that a f64 would round:
    // the reader never converts, so nothing is lost before the caller
    // decides what the number means.
    for text in ["0", "-1", "1e400", "9007199254740993", "1.5e-7"] {
        assert_eq!(parse(text).unwrap(), Json::Num(text.to_owned()));
    }
}

#[test]
fn j03_an_object_keeps_the_order_written() {
    let pairs = obj(r#"{"b": 1, "a": 2, "c": 3}"#);
    let names: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(names, ["b", "a", "c"], "order is the file's, not sorted");
}

#[test]
fn j04_get_finds_a_member_and_as_str_reads_it() {
    let v = parse(r#"{"name": "color", "syntax": "<color>"}"#).unwrap();
    assert_eq!(v.get("syntax").and_then(Json::as_str), Some("<color>"));
    assert_eq!(v.get("absent"), None);
}

#[test]
fn j05_nesting_and_whitespace() {
    let v = parse("  {\n\t\"a\" : [ 1 , { \"b\" : [] } ]\r\n}  ").unwrap();
    let a = v.get("a").unwrap();
    let Json::Arr(items) = a else {
        panic!("not an array: {a:?}")
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items[1].get("b"), Some(&Json::Arr(Vec::new())));
}

#[test]
fn j06_the_simple_escapes() {
    let v = parse(r#""\"\\\/\b\f\n\r\t""#).unwrap();
    assert_eq!(v.as_str(), Some("\"\\/\u{8}\u{C}\n\r\t"));
}

#[test]
fn j07_a_surrogate_pair_is_one_character() {
    // html5lib's doubleEscaped cases carry astral characters as pairs.
    let v = parse(r#""a😀b""#).unwrap();
    assert_eq!(v.as_str(), Some("a\u{1F600}b"));
    assert_eq!(v.as_str().unwrap().chars().count(), 3);
}

#[test]
fn j08_a_lone_surrogate_becomes_the_replacement_character() {
    // The module's stated decision. Rust's String cannot hold a lone
    // surrogate; the HTML input stream is where the standard raises
    // surrogate-in-input-stream for one.
    for text in [r#""\uD800""#, r#""\uDC00""#, r#""\uD800x""#] {
        let v = parse(text).unwrap();
        assert!(
            v.as_str().unwrap().contains('\u{FFFD}'),
            "{text} should decode to U+FFFD, got {v:?}"
        );
    }
    // A high surrogate followed by a *non*-surrogate escape keeps both:
    // the second escape is not swallowed by the failed pairing.
    let v = parse(r#""\uD800A""#).unwrap();
    assert_eq!(v.as_str(), Some("\u{FFFD}A"));
}

#[test]
fn j09_what_is_rejected() {
    for text in [
        "",                 // nothing at all
        "{",                // unterminated object
        r#"{"a": }"#,       // no value
        r#"{"a" 1}"#,       // no colon
        r#"{"a": 1,}"#,     // trailing comma
        "[1 2]",            // no comma
        r#""\q""#,          // not an escape
        r#""\u00""#,        // a short escape
        "nul",              // not a literal
        r#"{"a": 1} tail"#, // trailing content
        "01",               // a leading zero
    ] {
        assert!(parse(text).is_err(), "{text:?} should not parse");
    }
}

#[test]
fn j10_an_error_names_the_offset() {
    let e = parse(r#"{"a": tru}"#).unwrap_err().to_string();
    assert!(e.contains("6"), "the error should name the offset: {e}");
}
