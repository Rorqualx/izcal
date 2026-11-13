//! The CSS Syntax 3 tokenizer, §4.

use hg_style_reference::token::{HashKind, Token, preprocess, tokenize, tokenize_significant};

fn t(input: &str) -> Vec<Token> {
    tokenize_significant(input)
}

#[test]
fn k01_preprocessing_folds_four_line_endings_into_one() {
    // §3.3: CR, FF and CR LF each become LF; NULL becomes U+FFFD.
    assert_eq!(
        preprocess("a\r\nb\rc\u{C}d\0e"),
        "a\nb\nc\nd\u{FFFD}e".chars().collect::<Vec<_>>()
    );
}

#[test]
fn k02_identifiers_functions_and_at_keywords() {
    assert_eq!(t("foo"), vec![Token::Ident("foo".to_owned())]);
    assert_eq!(t("foo("), vec![Token::Function("foo".to_owned())]);
    assert_eq!(t("@media"), vec![Token::AtKeyword("media".to_owned())]);
    assert_eq!(t("--custom"), vec![Token::Ident("--custom".to_owned())]);
    assert_eq!(t("-x"), vec![Token::Ident("-x".to_owned())]);
    // `@` not followed by an identifier is a delimiter.
    assert_eq!(t("@1"), vec![Token::Delim('@'), Token::Number(1.0, true)]);
}

#[test]
fn k03_a_hash_carries_the_type_flag_that_decides_if_it_is_an_id() {
    assert_eq!(
        t("#name"),
        vec![Token::Hash("name".to_owned(), HashKind::Id)]
    );
    // `#123` is a colour, never an id selector — that is what the
    // unrestricted flag is for.
    assert_eq!(
        t("#123"),
        vec![Token::Hash("123".to_owned(), HashKind::Unrestricted)]
    );
    assert_eq!(t("#-a"), vec![Token::Hash("-a".to_owned(), HashKind::Id)]);
    assert_eq!(t("# "), vec![Token::Delim('#')]);
}

#[test]
fn k04_numbers_percentages_and_dimensions() {
    assert_eq!(t("1"), vec![Token::Number(1.0, true)]);
    assert_eq!(t("1.5"), vec![Token::Number(1.5, false)]);
    assert_eq!(t("-1"), vec![Token::Number(-1.0, true)]);
    assert_eq!(t("+1"), vec![Token::Number(1.0, true)]);
    assert_eq!(t(".5"), vec![Token::Number(0.5, false)]);
    assert_eq!(t("1e3"), vec![Token::Number(1000.0, false)]);
    assert_eq!(t("1e-3"), vec![Token::Number(0.001, false)]);
    assert_eq!(t("50%"), vec![Token::Percentage(50.0)]);
    assert_eq!(
        t("10px"),
        vec![Token::Dimension(10.0, true, "px".to_owned())]
    );
    assert_eq!(t("2n"), vec![Token::Dimension(2.0, true, "n".to_owned())]);
    // A `.` with no digit after it is a delimiter, which is what makes
    // `.class` work.
    assert_eq!(
        t(".a"),
        vec![Token::Delim('.'), Token::Ident("a".to_owned())]
    );
}

#[test]
fn k05_strings_end_at_a_quote_and_badly_at_a_newline() {
    assert_eq!(t("\"a\""), vec![Token::Str("a".to_owned())]);
    assert_eq!(t("'a'"), vec![Token::Str("a".to_owned())]);
    // A newline ends a string badly, and is *not* consumed. The
    // trailing quote then opens a string that runs to EOF, which
    // §4.3.5 says to return as a <string-token>, not a bad one: "EOF:
    // This is a parse error. Return the <string-token>."
    assert_eq!(
        tokenize("\"a\nb\""),
        vec![
            Token::BadString,
            Token::Whitespace,
            Token::Ident("b".to_owned()),
            Token::Str(String::new()),
        ]
    );
    // A backslash before a newline continues the string.
    assert_eq!(t("\"a\\\nb\""), vec![Token::Str("ab".to_owned())]);
}

#[test]
fn k06_escapes_are_hex_with_an_optional_trailing_space() {
    assert_eq!(t("\\41"), vec![Token::Ident("A".to_owned())]);
    assert_eq!(t("\\41 b"), vec![Token::Ident("Ab".to_owned())]);
    assert_eq!(
        t("\\41  b"),
        vec![Token::Ident("A".to_owned()), Token::Ident("b".to_owned())]
    );
    assert_eq!(t("\\-"), vec![Token::Ident("-".to_owned())]);
    // Zero, a surrogate and anything above the maximum become U+FFFD.
    assert_eq!(t("\\0"), vec![Token::Ident("\u{FFFD}".to_owned())]);
    assert_eq!(t("\\D800"), vec![Token::Ident("\u{FFFD}".to_owned())]);
    assert_eq!(t("\\110000"), vec![Token::Ident("\u{FFFD}".to_owned())]);
}

#[test]
fn k07_url_is_a_token_unquoted_and_a_function_quoted() {
    assert_eq!(t("url(a.png)"), vec![Token::Url("a.png".to_owned())]);
    assert_eq!(t("url( a.png )"), vec![Token::Url("a.png".to_owned())]);
    // Quoted, it is an ordinary function, so the string is a token.
    assert_eq!(
        t("url(\"a.png\")"),
        vec![
            Token::Function("url".to_owned()),
            Token::Str("a.png".to_owned()),
            Token::RightParen,
        ]
    );
    assert_eq!(t("url(a'b)"), vec![Token::BadUrl]);
    // A bad url swallows to the closing paren, so the next token is
    // what follows it.
    assert_eq!(
        t("url(a'b) x"),
        vec![Token::BadUrl, Token::Ident("x".to_owned())]
    );
}

#[test]
fn k08_comments_are_removed_and_an_unclosed_one_eats_the_rest() {
    assert_eq!(
        t("a/* c */b"),
        vec![Token::Ident("a".to_owned()), Token::Ident("b".to_owned())]
    );
    assert_eq!(t("a/* c"), vec![Token::Ident("a".to_owned())]);
}

#[test]
fn k09_cdo_and_cdc() {
    assert_eq!(t("<!--"), vec![Token::Cdo]);
    assert_eq!(t("-->"), vec![Token::Cdc]);
    assert_eq!(t("<"), vec![Token::Delim('<')]);
}

#[test]
fn k10_a_long_run_of_comments_is_consumed_without_recursing() {
    // A reference engine that overflowed its stack would abort the run
    // rather than give a verdict, and a run of comments is ordinary
    // input for a fuzzer.
    let input = format!("{}x", "/**/".repeat(100_000));
    assert_eq!(t(&input), vec![Token::Ident("x".to_owned())]);
}
