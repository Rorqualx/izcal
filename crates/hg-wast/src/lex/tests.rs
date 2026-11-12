//! Token grammar tests.
//!
//! The cases here are written for these tests. Where one is modelled on
//! something the suite exercises — nested comments, a quotation mark
//! inside a comment, the longest-match rule — the shape is the point
//! and the text is ours: the suite is test-only data, fetched at a pin
//! and never committed (CONTRIBUTING, Licence).

use super::*;

/// The kinds of a source's tokens, for a compact expectation.
fn kinds(src: &str) -> Vec<Kind> {
    Lexer::new(src)
        .tokens()
        .expect("should lex")
        .iter()
        .map(|t| t.kind)
        .collect()
}

/// The texts of a source's tokens.
fn texts(src: &str) -> Vec<&str> {
    Lexer::new(src)
        .tokens()
        .expect("should lex")
        .iter()
        .map(|t| t.span.text(src))
        .collect()
}

fn error(src: &str) -> Error {
    Lexer::new(src).tokens().expect_err("should not lex")
}

#[test]
fn the_four_shapes_of_token() {
    assert_eq!(
        kinds("(module $m \"name\")"),
        [Kind::LParen, Kind::Word, Kind::Id, Kind::Str, Kind::RParen]
    );
    assert_eq!(
        texts("(module $m \"name\")"),
        ["(", "module", "$m", "\"name\"", ")"]
    );
}

#[test]
fn a_line_comment_runs_to_the_newline_and_no_further() {
    assert_eq!(texts("a ;; b (; c\nd"), ["a", "d"]);
}

/// A line comment's contents are raw: an opener inside one opens
/// nothing, and a closer inside one closes nothing.
#[test]
fn a_comment_opener_inside_a_line_comment_is_text() {
    assert_eq!(texts(";; (;\na"), ["a"]);
    assert_eq!(texts(";; ;)\na"), ["a"]);
}

#[test]
fn block_comments_nest() {
    assert_eq!(texts("a (; one (; two ;) one ;) b"), ["a", "b"]);
    assert_eq!(texts("a (;(;(;(;;);););) b"), ["a", "b"]);
}

/// Inside a block comment only `(;` and `;)` mean anything. A parser
/// that counted parentheses over raw bytes would leave the group open
/// on the first of these and never recover.
#[test]
fn a_block_comment_holds_anything_at_all() {
    assert_eq!(texts("a (; \" ;) b"), ["a", "b"]);
    assert_eq!(texts("a (; ((((( ;) b"), ["a", "b"]);
    assert_eq!(texts("a (; ))))) ;) b"), ["a", "b"]);
    assert_eq!(texts("a (; ;; not a line comment ;) b"), ["a", "b"]);
    assert_eq!(texts("a (;\u{0}\u{1}\u{7f};) b"), ["a", "b"]);
    assert_eq!(
        texts("a (;\u{0};) b"),
        ["a", "b"],
        "a NUL right before the closer"
    );
    assert_eq!(texts("a (;Heiße Würstchen;) b"), ["a", "b"]);
}

#[test]
fn an_unclosed_block_comment_is_reported_where_it_opened() {
    let e = error("a (; b");
    assert_eq!(e.offset, 2);
    assert!(e.message.contains("never closed"), "{}", e.message);
}

#[test]
fn strings_keep_their_quotation_marks_in_the_span() {
    assert_eq!(texts(r#""a\"b""#), [r#""a\"b""#]);
    assert_eq!(kinds(r#""\00asm""#), [Kind::Str]);
}

#[test]
fn a_parenthesis_inside_a_string_is_not_a_parenthesis() {
    assert_eq!(
        kinds(r#"(f ")" "(")"#),
        [Kind::LParen, Kind::Word, Kind::Str, Kind::Str, Kind::RParen]
    );
}

#[test]
fn every_escape_the_format_admits() {
    for s in [
        r#""\t""#,
        r#""\n""#,
        r#""\r""#,
        r#""\"""#,
        r#""\'""#,
        r#""\\""#,
        r#""\00""#,
        r#""\ff""#,
        r#""\u{41}""#,
        r#""\u{1F600}""#,
    ] {
        assert_eq!(kinds(s), [Kind::Str], "{s}");
    }
}

#[test]
fn a_bad_escape_is_refused() {
    for (s, want) in [
        (r#""\q""#, "not an escape"),
        (r#""\0""#, "two hexadecimal digits"),
        (r#""\u41""#, "followed by {"),
        (r#""\u{}""#, "needs a hexadecimal digit"),
        (r#""\u{41""#, "never closed"),
    ] {
        let e = error(s);
        assert!(e.message.contains(want), "{s}: {}", e.message);
    }
}

/// The format admits no raw control character in a string: one must be
/// escaped. A reader that passed them through would accept a file the
/// specification does not.
#[test]
fn a_raw_control_character_in_a_string_is_refused() {
    let e = error("\"a\u{0}b\"");
    assert!(e.message.contains("control character"), "{}", e.message);
    let e = error("\"a\u{7f}b\"");
    assert!(e.message.contains("control character"), "{}", e.message);
}

#[test]
fn an_unclosed_string_is_reported_where_it_opened() {
    let e = error("a \"bc");
    assert_eq!(e.offset, 2);
    assert!(e.message.contains("never closed"), "{}", e.message);
}

/// The longest-match rule: a run of identifier characters and strings is
/// one token even when its parts would each be valid alone. `0$x` is
/// not `0` and `$x`, and `"a""b"` is not two strings. This is how the
/// format forces tokens apart with space or parentheses, and a lexer
/// that split them would accept text the specification rejects.
#[test]
fn a_run_that_is_more_than_one_thing_is_one_reserved_token() {
    assert_eq!(kinds(r#""a""b""#), [Kind::Reserved]);
    assert_eq!(kinds(r#"x")"y"#), [Kind::Reserved]);
    // Separated, the same characters are the tokens they look like.
    assert_eq!(kinds(r#""a" "b""#), [Kind::Str, Kind::Str]);
    // A run of identifier characters is one token whether or not it is
    // a keyword or a number: `0$x` is `0$x`, never `0` and `$x`.
    assert_eq!(texts("0$x"), ["0$x"]);
    assert_eq!(kinds("0 $x"), [Kind::Word, Kind::Id]);
}

/// The punctuation the grammar admits only inside a reserved token.
#[test]
fn reserved_punctuation_is_a_token_and_not_an_error() {
    assert_eq!(kinds("{ } [ ] , ;"), [Kind::Reserved; 6]);
    assert_eq!(texts(",{{};}]"), [",{{};}]"]);
    // A parenthesis still ends the run, so groups stay balanced.
    assert_eq!(kinds("({)"), [Kind::LParen, Kind::Reserved, Kind::RParen]);
}

/// `;;` opens a comment wherever it appears, so it ends a run rather
/// than joining it.
#[test]
fn a_run_stops_at_a_line_comment() {
    assert_eq!(texts("a;;b\nc"), ["a", "c"]);
    assert_eq!(texts(",;;b\nc"), [",", "c"]);
}

#[test]
fn identifiers_come_in_both_forms() {
    assert_eq!(kinds("$f"), [Kind::Id]);
    assert_eq!(kinds(r#"$"a b""#), [Kind::Id]);
    assert_eq!(texts(r#"$"a b""#), [r#"$"a b""#]);
    assert_eq!(kinds(r#"$!?@#a$%^&*-+_.:9'`|/\<=>~"#), [Kind::Id]);
    // `$` alone is no identifier, so it is just a word.
    assert_eq!(kinds("$"), [Kind::Word]);
}

#[test]
fn a_character_that_can_begin_no_token_is_refused() {
    let e = error("a \u{a0} b");
    assert_eq!(e.offset, 2);
    assert!(e.message.contains("U+00A0"), "{}", e.message);
    // `!` is an identifier character, so it is a token, not an error.
    assert_eq!(kinds("a ! b"), [Kind::Word; 3]);
}

#[test]
fn positions_count_lines_and_characters_from_one() {
    let src = "a\nbc\nd";
    assert_eq!(position(src, 0), Position { line: 1, column: 1 });
    assert_eq!(position(src, 2), Position { line: 2, column: 1 });
    assert_eq!(position(src, 3), Position { line: 2, column: 2 });
    assert_eq!(position(src, 5), Position { line: 3, column: 1 });
    // Past the end still names a place.
    assert_eq!(position(src, 999), Position { line: 3, column: 2 });
}

#[test]
fn a_span_covers_exactly_its_token() {
    let src = "  (module)";
    let toks = Lexer::new(src).tokens().expect("should lex");
    assert_eq!(toks[1].span, Span { start: 3, end: 9 });
    assert_eq!(toks[1].span.text(src), "module");
    assert_eq!(toks[0].span.to(toks[2].span).text(src), "(module)");
}

#[test]
fn an_error_reads_with_its_position() {
    let src = "a\n(; b";
    let e = error(src);
    assert!(e.display(src).starts_with("2:1: "), "{}", e.display(src));
}
