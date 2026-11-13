//! What the tokenizer does, state by state.
//!
//! Every case here runs on both back ends, so a case that only one
//! satisfies fails rather than passing on the other's behalf.

use hg_html_syntax::{Attribute, Back, Token};

use crate::{run, text};

const BOTH: [Back; 2] = [Back::Generated, Back::Table];

fn tokens(input: &str) -> Vec<Token> {
    let mut first: Option<Vec<Token>> = None;
    for b in BOTH {
        let got = run(input, b).tokens;
        if let Some(prev) = &first {
            assert_eq!(prev, &got, "the two back ends disagree on {input:?}");
        }
        first = Some(got);
    }
    first.unwrap_or_default()
}

fn errors(input: &str) -> Vec<&'static str> {
    let mut first: Option<Vec<&'static str>> = None;
    for b in BOTH {
        let got = run(input, b).errors;
        if let Some(prev) = &first {
            assert_eq!(prev, &got, "the two back ends disagree on {input:?}");
        }
        first = Some(got);
    }
    first.unwrap_or_default()
}

fn attr(name: &str, value: &str) -> Attribute {
    Attribute {
        name: name.to_owned(),
        value: value.to_owned(),
    }
}

fn start(name: &str, attrs: Vec<Attribute>) -> Token {
    Token::StartTag {
        name: name.to_owned(),
        attrs,
        self_closing: false,
    }
}

#[test]
fn plain_text_is_character_tokens_then_the_end_of_the_file() {
    assert_eq!(
        tokens("hi"),
        vec![Token::Character('h'), Token::Character('i'), Token::Eof]
    );
}

#[test]
fn a_start_tag_with_attributes() {
    assert_eq!(
        tokens("<a href=\"x\" id=y>"),
        vec![
            start("a", vec![attr("href", "x"), attr("id", "y")]),
            Token::Eof
        ]
    );
}

#[test]
fn a_tag_name_and_an_attribute_name_are_lowercased() {
    assert_eq!(
        tokens("<DIV CLASS=A>"),
        vec![start("div", vec![attr("class", "A")]), Token::Eof]
    );
}

#[test]
fn a_duplicate_attribute_is_an_error_and_the_second_is_removed() {
    assert_eq!(
        tokens("<a b=1 b=2>"),
        vec![start("a", vec![attr("b", "1")]), Token::Eof]
    );
    assert_eq!(errors("<a b=1 b=2>"), vec!["duplicate-attribute"]);
}

#[test]
fn a_self_closing_tag_sets_its_flag() {
    assert_eq!(
        tokens("<br/>"),
        vec![
            Token::StartTag {
                name: "br".to_owned(),
                attrs: Vec::new(),
                self_closing: true
            },
            Token::Eof
        ]
    );
}

#[test]
fn a_comment_keeps_its_data() {
    assert_eq!(
        tokens("<!-- a -->"),
        vec![Token::Comment(" a ".to_owned()), Token::Eof]
    );
}

#[test]
fn a_bogus_comment_is_what_an_unopened_declaration_becomes() {
    assert_eq!(
        tokens("<!x>"),
        vec![Token::Comment("x".to_owned()), Token::Eof]
    );
    assert_eq!(errors("<!x>"), vec!["incorrectly-opened-comment"]);
}

#[test]
fn a_doctype_with_a_public_and_a_system_identifier() {
    assert_eq!(
        tokens("<!DOCTYPE html PUBLIC \"p\" \"s\">"),
        vec![
            Token::Doctype {
                name: Some("html".to_owned()),
                public_id: Some("p".to_owned()),
                system_id: Some("s".to_owned()),
                force_quirks: false,
            },
            Token::Eof
        ]
    );
}

#[test]
fn a_doctype_keyword_is_matched_without_regard_to_case() {
    // `from ci "PUBLIC"`: the six characters starting from the current
    // input character, which this state has consumed.
    assert_eq!(
        tokens("<!doctype html system \"s\">"),
        vec![
            Token::Doctype {
                name: Some("html".to_owned()),
                public_id: None,
                system_id: Some("s".to_owned()),
                force_quirks: false,
            },
            Token::Eof
        ]
    );
}

#[test]
fn a_named_character_reference_becomes_its_characters() {
    assert_eq!(text(&run("&amp;", Back::Generated)), "&");
    assert_eq!(text(&run("&amp;", Back::Table)), "&");
}

#[test]
fn the_longest_named_reference_wins_and_the_rest_is_text() {
    // "&notit;" matches "&not", not "&notin;".
    assert_eq!(tokens("&notit;"), {
        let mut v: Vec<Token> = "\u{AC}it;".chars().map(Token::Character).collect();
        v.push(Token::Eof);
        v
    });
    assert_eq!(
        errors("&notit;"),
        vec!["missing-semicolon-after-character-reference"]
    );
}

#[test]
fn a_reference_with_no_semicolon_inside_an_attribute_is_left_alone() {
    // The historical rule: in an attribute, a match without a semicolon
    // followed by `=` or an alphanumeric is not a reference.
    assert_eq!(
        tokens("<a b=\"&notit\">"),
        vec![start("a", vec![attr("b", "&notit")]), Token::Eof]
    );
    assert!(errors("<a b=\"&notit\">").is_empty());
}

#[test]
fn an_unknown_reference_is_an_ambiguous_ampersand() {
    assert_eq!(text(&run("&abcd;", Back::Generated)), "&abcd;");
    assert_eq!(errors("&abcd;"), vec!["unknown-named-character-reference"]);
}

#[test]
fn a_numeric_reference_is_decimal_or_hexadecimal() {
    assert_eq!(text(&run("&#65;&#x42;&#X43;", Back::Generated)), "ABC");
    assert_eq!(text(&run("&#65;&#x42;&#X43;", Back::Table)), "ABC");
}

#[test]
fn the_numeric_reference_end_state_applies_the_standards_own_table() {
    // 0x80 is remapped to the euro sign, with a control error.
    assert_eq!(text(&run("&#x80;", Back::Generated)), "\u{20AC}");
    assert_eq!(errors("&#x80;"), vec!["control-character-reference"]);
    // A surrogate, a zero and a value past the last code point all
    // become the replacement character, each with its own error.
    assert_eq!(text(&run("&#xD800;", Back::Generated)), "\u{FFFD}");
    assert_eq!(errors("&#xD800;"), vec!["surrogate-character-reference"]);
    assert_eq!(errors("&#0;"), vec!["null-character-reference"]);
    assert_eq!(
        errors("&#x110000;"),
        vec!["character-reference-outside-unicode-range"]
    );
}

#[test]
fn an_end_tag_closes_rcdata_only_when_it_is_the_appropriate_one() {
    use crate::Collect;
    use hg_html_syntax::{EntryState, Tokenizer};

    // "An appropriate end tag token is one whose tag name matches the
    // tag name of the last start tag to have been emitted from this
    // tokenizer." `</b>` inside `<title>` is not one, so it is text.
    for back in BOTH {
        let mut t = Tokenizer::new(Collect::default(), "<title>a</b>b</title>", back);
        let mut switched = false;
        while !t.is_done() {
            t.step();
            // What the tree builder does on `<title>`: "switch the
            // tokenizer to the RCDATA state".
            if !switched
                && t.sink()
                    .tokens
                    .iter()
                    .any(|x| matches!(*x, Token::StartTag { .. }))
            {
                t.switch_to(EntryState::Rcdata);
                switched = true;
            }
        }
        let c = t.into_sink();
        assert!(switched, "the start tag was emitted");
        assert_eq!(text(&c), "a</b>b", "{back:?}");
        assert_eq!(
            c.tokens.last(),
            Some(&Token::Eof),
            "the run ends at the end of the file"
        );
        assert!(
            c.tokens.iter().any(|x| matches!(
                x,
                Token::EndTag { name, .. } if name == "title"
            )),
            "the appropriate end tag closes it"
        );
    }
}

#[test]
fn a_cdata_section_is_a_comment_in_html_content() {
    assert_eq!(
        tokens("<![CDATA[x]]>"),
        vec![Token::Comment("[CDATA[x]]".to_owned()), Token::Eof]
    );
    assert_eq!(errors("<![CDATA[x]]>"), vec!["cdata-in-html-content"]);
}

#[test]
fn a_carriage_return_is_normalised_by_the_input_stream() {
    assert_eq!(text(&run("a\r\nb\rc", Back::Generated)), "a\nb\nc");
    assert_eq!(text(&run("a\r\nb\rc", Back::Table)), "a\nb\nc");
}

#[test]
fn an_unfinished_tag_at_the_end_of_the_file_is_an_error() {
    assert_eq!(errors("<a"), vec!["eof-in-tag"]);
    assert_eq!(tokens("<a"), vec![Token::Eof]);
}
