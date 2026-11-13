//! Each rule of the language refuses one thing, and the same description
//! without that thing is accepted — so every refusal is attributable to
//! the rule and not to the fixture.

use hg_htmlgen::model;
use hg_htmlgen::syntax::{Action, Match, parse};

/// A description in the shape the prose writes: the data state, a state
/// it reaches, and an `extern` state for the part the prose does not
/// write as a table.
const GOOD: &str = r#"
state data @ "html#data-state" {
  '&'  => set_return data, switch character_reference
  '<'  => switch tag_open
  null => error unexpected-null-character, emit_current
  eof  => emit_eof
  _    => emit_current
}

state tag_open @ "html#tag-open-state" {
  '/'         => switch end_tag_open
  ascii_alpha => create_start_tag, reconsume tag_name
  eof         => error eof-before-tag-name, emit_char '<', emit_eof
  _           => error invalid-first-character-of-tag-name, emit_char '<', reconsume data
}

state end_tag_open @ "html#end-tag-open-state" {
  ascii_alpha => create_end_tag, reconsume tag_name
  _           => error invalid-first-character-of-tag-name, switch bogus_comment
}

state tag_name @ "html#tag-name-state" {
  whitespace  => switch before_attribute_name
  '>'         => switch data, emit_token
  ascii_upper => append_lower tag_name
  eof         => error eof-in-tag, emit_eof
  _           => append tag_name
}

state before_attribute_name @ "html#before-attribute-name-state" {
  whitespace => switch before_attribute_name
  _          => start_attribute, reconsume attribute_name
}

state attribute_name @ "html#attribute-name-state" {
  whitespace => switch before_attribute_name
  '>'        => switch data, emit_token
  _          => append attr_name
}

state bogus_comment @ "html#bogus-comment-state" {
  '>'  => switch data, emit_token
  eof  => emit_token, emit_eof
  _    => append comment
}

extern state character_reference @ "html#character-reference-state"
"#;

fn refuse(src: &str) -> String {
    model(&[("t.hgtok".into(), src.into())])
        .expect_err("should be refused")
        .to_string()
}

#[test]
fn the_twin_is_accepted() {
    let m = model(&[("t.hgtok".into(), GOOD.into())]).expect("the twin parses and checks");
    assert_eq!(m.states.len(), 8);
    assert!(m.anchors.contains("html#tag-name-state"));
    assert!(m.errors.contains("eof-in-tag"));
    assert!(m.transitions.contains(&("data".into(), "tag_open".into())));
}

#[test]
fn an_extern_state_is_the_only_way_to_leave_a_state_undescribed() {
    // Declared `extern`: accepted, because the hand-off is visible.
    assert!(model(&[("t.hgtok".into(), GOOD.into())]).is_ok());
    // The same state with an empty body is refused, and the message
    // says what to do about it.
    let src = GOOD.replace(
        "extern state character_reference @ \"html#character-reference-state\"",
        "state character_reference @ \"html#character-reference-state\" { }",
    );
    let e = refuse(&src);
    assert!(e.contains("has no rules"), "{e}");
    assert!(e.contains("extern state"), "{e}");
}

#[test]
fn a_state_with_no_anything_else_row_is_refused() {
    let src = GOOD.replace("  _          => append attr_name\n", "");
    let e = refuse(&src);
    assert!(e.contains("no `_` rule"), "{e}");
}

#[test]
fn a_rule_an_earlier_one_already_covers_is_refused() {
    // `ascii_upper` after `ascii_alpha` can never fire.
    let src = GOOD.replace(
        "  ascii_alpha => create_start_tag, reconsume tag_name",
        "  ascii_alpha => create_start_tag, reconsume tag_name\n  ascii_upper => emit_eof",
    );
    let e = refuse(&src);
    assert!(e.contains("can never fire"), "{e}");
}

#[test]
fn a_character_an_earlier_class_already_covers_is_refused() {
    // `' '` after `whitespace` can never fire.
    let src = GOOD.replace(
        "  whitespace  => switch before_attribute_name",
        "  whitespace  => switch before_attribute_name\n  ' '         => emit_eof",
    );
    let e = refuse(&src);
    assert!(e.contains("can never fire"), "{e}");
}

#[test]
fn a_state_that_does_not_exist_is_refused() {
    let src = GOOD.replace("switch tag_open", "switch tag_opne");
    let e = refuse(&src);
    assert!(e.contains("does not exist"), "{e}");
}

#[test]
fn a_state_nothing_reaches_is_refused() {
    let src = format!("{GOOD}\nstate orphan @ \"html#orphan\" {{\n  _ => emit_eof\n}}\n");
    let e = refuse(&src);
    assert!(e.contains("no path from `data` reaches"), "{e}");
    assert!(e.contains("orphan"), "{e}");
}

#[test]
fn a_description_with_no_data_state_is_refused() {
    let src = GOOD.replace("state data @", "state start @");
    let e = refuse(&src);
    assert!(e.contains("no `data` state"), "{e}");
}

#[test]
fn an_if_without_an_else_is_refused() {
    let src = GOOD.replace(
        "  '>'         => switch data, emit_token",
        "  '>'         => if appropriate { switch data }",
    );
    let e = refuse(&src);
    assert!(e.contains("needs an `else`"), "{e}");
}

#[test]
fn an_if_else_parses_into_both_branches() {
    let src = GOOD.replace(
        "  '>'         => switch data, emit_token",
        "  '>'         => if appropriate { switch data, emit_token } else { append tag_name }",
    );
    let f = parse(&src, "t.hgtok").expect("parses");
    let rule = f
        .states
        .iter()
        .find(|s| s.name == "tag_name")
        .and_then(|s| s.rules.iter().find(|r| r.matches == Match::Char('>')))
        .expect("the rule is there");
    match rule.actions.first() {
        Some(Action::If {
            then, otherwise, ..
        }) => {
            assert_eq!(then.len(), 2);
            assert_eq!(otherwise.len(), 1);
        }
        other => panic!("expected an if, got {other:?}"),
    }
}

#[test]
fn an_anchor_may_carry_a_fragment_and_a_rule_may_carry_its_own() {
    let f = parse(
        "state data @ \"html#data-state\" {\n  _ => emit_current @ \"html#data-state-anything\"\n}",
        "t",
    )
    .expect("parses");
    assert_eq!(f.states[0].anchor, "html#data-state");
    assert_eq!(
        f.states[0].rules[0].anchor.as_deref(),
        Some("html#data-state-anything")
    );
}

#[test]
fn escapes_in_a_character_literal_are_the_ones_the_prose_needs() {
    let f = parse(
        "state data @ \"a\" {\n  '\\0' => emit_eof\n  '\\n' => emit_eof\n  '\\u{000C}' => emit_eof\n  _ => emit_current\n}",
        "t",
    )
    .expect("parses");
    let got: Vec<Match> = f.states[0]
        .rules
        .iter()
        .map(|r| r.matches.clone())
        .collect();
    assert_eq!(got[0], Match::Char('\0'));
    assert_eq!(got[1], Match::Char('\n'));
    assert_eq!(got[2], Match::Char('\u{000C}'));
}

#[test]
fn an_unknown_verb_or_class_names_itself() {
    assert!(refuse(&GOOD.replace("emit_current", "emit_nonsense")).contains("not an action"));
    assert!(
        refuse(&GOOD.replace("ascii_alpha =>", "ascii_nonsense =>"))
            .contains("not a code-point class")
    );
}
