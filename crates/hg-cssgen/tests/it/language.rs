//! The `.hgprop` language: what it accepts, and what it refuses.
//!
//! The overlay is 7,700 lines of data, so the thing that keeps it
//! trustworthy is that a malformed or self-contradictory line is an
//! error rather than something the generator quietly works around.

use hg_cssgen::check::check;
use hg_cssgen::syntax::{Affect, parse};

/// A minimal overlay that passes the first pass, for a test to spoil.
const GOOD: &str = r#"
kind prose    "Defined by prose; a hook computes it."
kind identity "The specified value, unchanged."
kind length   "Lengths made absolute."

computed "as specified" => identity
computed "see below"    => prose

property color @ "https://example.invalid/#propdef-color" {
  initial   "black"
  inherited "yes"
  inherits  yes
  computed  "as specified"
  syntax    "<color>"
  affects   paint
}

property margin-top @ "https://example.invalid/#propdef-margin-top" {
  initial   "0"
  inherited "no"
  inherits  no
  computed  "as specified"
  affects   layout
}

property margin @ "https://example.invalid/#propdef-margin" {
  initial   "0"
  inherited "no"
  inherits  no
  computed  "as specified"
  longhands margin-top
  affects   layout
}

alias -webkit-margin => margin
"#;

fn model_of(text: &str) -> Result<hg_cssgen::Model, String> {
    let f = parse(text, "test.hgprop").map_err(|e| e.to_string())?;
    check(&[("test.hgprop".to_owned(), f)]).map_err(|e| e.to_string())
}

fn refused(text: &str) -> String {
    match model_of(text) {
        Ok(_) => panic!("this should have been refused"),
        Err(e) => e,
    }
}

#[test]
fn c01_the_good_overlay_is_accepted() {
    let m = model_of(GOOD).unwrap();
    assert_eq!(m.properties.len(), 3);
    assert_eq!(m.kinds.len(), 3);
    assert_eq!(m.aliases.len(), 1);
    assert_eq!(m.undecided(), 0);
}

#[test]
fn c02_comments_and_strings_with_punctuation() {
    let text = GOOD.replace(
        "  affects   paint\n}",
        "  # a comment with { } @ => , in it\n  affects   paint\n}",
    );
    assert_eq!(model_of(&text).unwrap().properties.len(), 3);
}

#[test]
fn c03_a_property_must_say_what_a_change_dirties() {
    let text = GOOD.replace(
        "  affects   layout\n}\n\nproperty margin ",
        "}\n\nproperty margin ",
    );
    assert!(
        refused(&text).contains("has no `affects`"),
        "an omitted affects should be refused"
    );
}

#[test]
fn c04_unknown_is_how_an_open_decision_is_written() {
    let text = GOOD.replace("affects   paint", "affects   unknown");
    let m = model_of(&text).unwrap();
    assert_eq!(m.undecided(), 1);
    assert!(
        m.properties
            .iter()
            .any(|p| p.affects.contains(&Affect::Unknown))
    );
}

#[test]
fn c05_nothing_and_unknown_stand_alone() {
    let text = GOOD.replace("affects   paint", "affects   paint nothing");
    assert!(refused(&text).contains("stands alone"));
    let text = GOOD.replace("affects   paint", "affects   unknown layout");
    assert!(refused(&text).contains("stands alone"));
}

#[test]
fn c06_the_cascade_needs_a_yes_or_no() {
    // webref writes `yes?`, `?` and "see individual properties"; the
    // overlay restates those as the text they are and answers
    // separately, so the answer may not be left out.
    let text = GOOD.replace("  inherits  yes\n", "");
    assert!(refused(&text).contains("has no `inherits`"));
    let text = GOOD.replace("inherits  yes", "inherits  yes?");
    assert!(
        refused(&text).contains('?'),
        "a non-answer should be refused"
    );
}

#[test]
fn c07_an_unclassified_computed_value_is_refused() {
    let text = GOOD.replace(
        r#"computed  "as specified""#,
        r#"computed  "something new""#,
    );
    assert!(refused(&text).contains("not classified"));
}

#[test]
fn c08_only_the_prose_kind_takes_a_hook() {
    let text = GOOD.replace(
        "  affects   paint\n}",
        "  hook      colour\n  affects   paint\n}",
    );
    assert!(refused(&text).contains("only `prose` takes one"));
}

#[test]
fn c09_the_prose_kind_must_take_one() {
    let text = GOOD.replace(
        r#"computed  "as specified"
  syntax"#,
        r#"computed  "see below"
  syntax"#,
    );
    assert!(refused(&text).contains("names no `hook`"));
}

#[test]
fn c10_a_text_may_be_classified_only_once() {
    let text = format!("{GOOD}\ncomputed \"as specified\" => length\n");
    assert!(refused(&text).contains("already classified"));
}

#[test]
fn c11_a_kind_must_be_declared() {
    let text = GOOD.replace("=> identity", "=> inherit-ish");
    assert!(refused(&text).contains("not a declared kind"));
}

#[test]
fn c12_a_property_may_not_have_two_blocks() {
    let text = format!(
        "{GOOD}\nproperty color @ \"x\" {{ inherits no affects layout computed \"as specified\" }}\n"
    );
    assert!(refused(&text).contains("already has a block"));
}

#[test]
fn c13_a_field_may_not_be_given_twice() {
    let text = GOOD.replace("  inherits  yes\n", "  inherits  yes\n  inherits  no\n");
    assert!(refused(&text).contains("twice"));
}

#[test]
fn c14_a_longhand_must_exist() {
    let text = GOOD.replace("longhands margin-top", "longhands margin-nowhere");
    assert!(refused(&text).contains("neither a property nor an alias"));
}

#[test]
fn c15_an_alias_must_stand_for_something() {
    let text = GOOD.replace("=> margin\n", "=> margin-nowhere\n");
    assert!(refused(&text).contains("has no block"));
}

#[test]
fn c16_a_name_cannot_be_both_a_property_and_an_alias() {
    let text = GOOD.replace("alias -webkit-margin => margin", "alias color => margin");
    assert!(refused(&text).contains("both an alias and a property"));
}

#[test]
fn c17_an_unclosed_string_is_an_error_not_a_swallowed_block() {
    let text = GOOD.replace(r#"initial   "black""#, r#"initial   "black"#);
    assert!(refused(&text).contains("closing quote"));
}

#[test]
fn c18_an_error_names_the_file_and_line() {
    let text = GOOD.replace("affects   paint", "affects   nonesuch-effect");
    let e = refused(&text);
    assert!(e.starts_with("test.hgprop:"), "{e}");
}

#[test]
fn c19_a_kind_override_still_restates_the_specifications_text() {
    // The override says what Izcal computes; it is not a way to stop
    // restating what the specification says, which is what the
    // cross-check compares.
    let text = GOOD.replace(
        "  affects   paint\n}",
        "  kind      length\n  affects   paint\n}",
    );
    assert_eq!(
        model_of(&text)
            .unwrap()
            .properties
            .iter()
            .find(|p| p.name == "color")
            .unwrap()
            .kind,
        Some("length".to_owned())
    );
    let text = text.replace("  computed  \"as specified\"\n  syntax", "  syntax");
    assert!(refused(&text).contains("restates no `computed` text"));
}

#[test]
fn c20_a_kind_override_must_name_a_declared_kind() {
    let text = GOOD.replace(
        "  affects   paint\n}",
        "  kind      nonesuch\n  affects   paint\n}",
    );
    assert!(refused(&text).contains("not a declared kind"));
}
