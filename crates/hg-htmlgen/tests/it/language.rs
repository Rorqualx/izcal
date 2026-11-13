//! Each rule of the language refuses one thing, and the same description
//! without that thing is accepted — so every refusal is attributable to
//! the rule and not to the fixture.

use hg_htmlgen::model;
use hg_htmlgen::syntax::{Action, Buffer, Cond, Lookahead, Match, parse};

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

/// The commit hook's review found this: `append <buffer>` used to take
/// an optional literal, and the lexer has no line structure, so a rule
/// ending in `append comment` would swallow the next rule's `'/'` as the
/// thing to append. The literal form is now a verb of its own.
#[test]
fn an_append_does_not_swallow_the_next_rules_character() {
    let src =
        "state data @ \"a\" {\n  eof => append comment\n  '/' => emit_eof\n  _ => emit_current\n}";
    let f = parse(src, "t").expect("parses");
    let rules = &f.states[0].rules;
    assert_eq!(rules.len(), 3, "the three rules stay three rules");
    assert_eq!(rules[0].actions, vec![Action::Append(Buffer::Comment)]);
    assert_eq!(rules[1].matches, Match::Char('/'));
    assert_eq!(rules[1].actions, vec![Action::EmitEof]);
}

#[test]
fn appending_a_literal_takes_its_own_verb() {
    let src = "state data @ \"a\" {\n  eof => append_char comment '/'\n  _ => emit_current\n}";
    let f = parse(src, "t").expect("parses");
    assert_eq!(
        f.states[0].rules[0].actions,
        vec![Action::AppendChar(Buffer::Comment, '/')]
    );
}

// ---------------------------------------------------------------------
// The four things the real 80 states need that the first language could
// not say. Each is tested by what it accepts and by what it refuses.
// ---------------------------------------------------------------------

#[test]
fn alternatives_are_one_rule_because_the_prose_groups_them() {
    // "U+002F SOLIDUS (/), U+003E GREATER-THAN SIGN (>), or EOF —
    // Reconsume in the after attribute name state."
    let src = "state data @ \"a\" {\n  '/' | '>' | eof => reconsume data\n  _ => emit_current\n}";
    let f = parse(src, "t").expect("parses");
    assert_eq!(
        f.states[0].rules[0].matches,
        Match::OneOf(vec![Match::Char('/'), Match::Char('>'), Match::Eof])
    );
}

#[test]
fn one_dead_alternative_condemns_the_group() {
    // `whitespace` already covers `' '`. The group could still fire on
    // `'/'`, but the `' '` alternative never will, and that is the slip
    // grouping invites — so it is refused rather than silently kept.
    let src = GOOD.replace(
        "  whitespace  => switch before_attribute_name",
        "  whitespace  => switch before_attribute_name\n  '/' | ' ' => emit_eof",
    );
    let e = refuse(&src);
    assert!(e.contains("one of its alternatives"), "{e}");
}

#[test]
fn a_group_with_no_covered_alternative_is_accepted() {
    let src = GOOD.replace(
        "  '>'         => switch data, emit_token",
        "  '>' | '/'   => switch data, emit_token",
    );
    assert!(
        model(&[("t.hgtok".into(), src)]).is_ok(),
        "nothing above covers `>` or `/`"
    );
}

#[test]
fn anything_else_is_a_row_of_its_own_not_an_alternative() {
    let src = "state data @ \"a\" {\n  '/' | _ => emit_eof\n  _ => emit_current\n}";
    let e = refuse(src);
    assert!(e.contains("row of its own"), "{e}");
}

#[test]
fn the_two_lookahead_questions_are_two_words() {
    // The markup declaration open state asks about characters it has
    // not consumed; the after DOCTYPE name state asks about ones
    // starting at the character it has.
    let src = "state data @ \"a\" {\n  next \"--\" => switch data\n  next ci \"DOCTYPE\" => \
               switch data\n  from ci \"PUBLIC\" => switch data\n  _ => emit_current\n}";
    let f = parse(src, "t").expect("parses");
    let got: Vec<Match> = f.states[0]
        .rules
        .iter()
        .map(|r| r.matches.clone())
        .collect();
    assert_eq!(
        got[0],
        Match::Lookahead(Lookahead {
            text: "--".into(),
            from_current: false,
            ci: false
        })
    );
    assert_eq!(
        got[1],
        Match::Lookahead(Lookahead {
            text: "DOCTYPE".into(),
            from_current: false,
            ci: true
        })
    );
    assert_eq!(
        got[2],
        Match::Lookahead(Lookahead {
            text: "PUBLIC".into(),
            from_current: true,
            ci: true
        })
    );
}

#[test]
fn a_case_insensitive_lookahead_covers_its_own_spellings() {
    let src = "state data @ \"a\" {\n  next ci \"DOCTYPE\" => switch data\n  next \"doctype\" => \
               switch data\n  _ => emit_current\n}";
    let e = refuse(src);
    assert!(e.contains("can never fire"), "{e}");
}

#[test]
fn a_next_lookahead_may_not_share_a_state_with_a_rule_that_consumes() {
    // The markup declaration open state's own "anything else" ends
    // "(do not consume anything)", so a state that asks about the next
    // few characters consumes nothing and may hold only those
    // lookaheads and `_`. Both orders are refused.
    let after = "state data @ \"a\" {\n  '<' => switch data\n  next \"--\" => switch data\n  _ \
                 => emit_current\n}";
    assert!(refuse(after).contains("comes after a rule that consumes"));
    let before = "state data @ \"a\" {\n  next \"--\" => switch data\n  '<' => switch data\n  _ \
                  => emit_current\n}";
    assert!(refuse(before).contains("follows a `next` lookahead"));
}

#[test]
fn a_from_lookahead_is_tested_in_rule_order_like_any_other_pattern() {
    // The after DOCTYPE name state consumes, then asks about "the six
    // characters starting from the current input character" — inside
    // its "anything else", after the whitespace, `>` and EOF rows. That
    // shape is accepted.
    let src = "state data @ \"a\" {\n  whitespace => ignore\n  '>' => switch data\n  eof => \
               emit_eof\n  from ci \"PUBLIC\" => switch data\n  from ci \"SYSTEM\" => switch \
               data\n  _ => emit_current\n}";
    assert!(
        model(&[("t.hgtok".into(), src.into())]).is_ok(),
        "a `from` lookahead after the rows that precede it in the prose"
    );
}

#[test]
fn a_from_lookahead_an_earlier_class_already_covers_is_dead() {
    // `ascii_alpha` matches every character `PUBLIC` could start with.
    let src = "state data @ \"a\" {\n  ascii_alpha => switch data\n  from ci \"PUBLIC\" => \
               switch data\n  _ => emit_current\n}";
    assert!(refuse(src).contains("can never fire"));
}

#[test]
fn a_state_that_consumes_nothing_has_no_current_character_to_use() {
    let src = "state data @ \"a\" {\n  next \"--\" => switch data\n  _ => emit_current\n}";
    let e = refuse(src);
    assert!(e.contains("needs the current input character"), "{e}");
    assert!(e.contains("emit_current"), "{e}");
}

#[test]
fn a_state_can_act_before_it_consumes_anything() {
    // "Numeric character reference state: Set the character reference
    // code to zero. Consume the next input character: …"
    let src = "state data @ \"a\" {\n  enter { char_ref_reset }\n  ascii_hex => \
               char_ref_hex_digit\n  _ => emit_current\n}";
    let f = parse(src, "t").expect("parses");
    assert_eq!(f.states[0].enter, vec![Action::CharRefReset]);
    assert_eq!(f.states[0].rules.len(), 2, "enter is not a rule");
}

#[test]
fn an_enter_block_that_moves_somewhere_is_a_transition() {
    let src = GOOD.replace(
        "state tag_open @ \"html#tag-open-state\" {",
        "state tag_open @ \"html#tag-open-state\" {\n  enter { set_return bogus_comment }",
    );
    let m = model(&[("t.hgtok".into(), src)]).expect("checks");
    assert!(
        m.transitions
            .contains(&("tag_open".into(), "bogus_comment".into())),
        "the enter block's target is recorded"
    );
    // And a misspelt one is refused the same way a rule's is.
    let bad = GOOD.replace(
        "state tag_open @ \"html#tag-open-state\" {",
        "state tag_open @ \"html#tag-open-state\" {\n  enter { set_return bogus_commnet }",
    );
    assert!(refuse(&bad).contains("does not exist"));
}

#[test]
fn the_remaining_verbs_say_what_the_prose_says() {
    let src = "state data @ \"a\" {\n  '[' => create_comment, append_str comment \"[CDATA[\"\n  \
               whitespace => ignore\n  '=' => finish_attr_name, switch data\n  ascii_digit => \
               char_ref_dec_digit\n  _ => if in_attribute { append attr_value } else { \
               emit_current }\n}";
    let f = parse(src, "t").expect("parses");
    let a = &f.states[0].rules;
    assert_eq!(
        a[0].actions[1],
        Action::AppendStr(Buffer::Comment, "[CDATA[".into())
    );
    assert_eq!(a[1].actions, vec![Action::Ignore]);
    assert_eq!(a[2].actions[0], Action::FinishAttrName);
    assert_eq!(a[3].actions, vec![Action::CharRefDecDigit]);
    assert!(matches!(
        a[4].actions[0],
        Action::If {
            cond: Cond::InAttribute,
            ..
        }
    ));
}

#[test]
fn append_str_wants_a_string_and_says_so() {
    let e = refuse("state data @ \"a\" {\n  _ => create_comment, append_str comment '['\n}");
    assert!(e.contains("append_str wants a string"), "{e}");
}

#[test]
fn a_state_the_tree_builder_switches_to_is_not_an_orphan() {
    // Four states are reached only by "switch the tokenizer to the
    // RAWTEXT state", which happens in the tree construction stage, so
    // without `entry` they would look like states nothing enters.
    let src = format!("{GOOD}\nstate aside @ \"html#aside\" {{\n  _ => emit_eof\n}}\n");
    assert!(refuse(&src).contains("no path from `data` reaches"));
    let src = format!("{GOOD}\nentry state aside @ \"html#aside\" {{\n  _ => emit_eof\n}}\n");
    let m = model(&[("t.hgtok".into(), src)]).expect("an entry state is reached from outside");
    assert!(m.states.iter().any(|s| s.name == "aside" && s.entry));
}

#[test]
fn an_extern_state_declares_where_its_hand_written_part_can_go() {
    // An extern state has no rules, so nothing in the description says
    // what it reaches, and a state only it enters would look orphaned.
    let src = GOOD.replace(
        "extern state character_reference @ \"html#character-reference-state\"",
        "extern state character_reference @ \"html#character-reference-state\" {\n  reaches { \
         switch bogus_comment }\n}",
    );
    let m = model(&[("t.hgtok".into(), src)]).expect("checks");
    assert!(
        m.transitions
            .contains(&("character_reference".into(), "bogus_comment".into()))
    );

    // And a misspelt target is refused, naming the state.
    let bad = GOOD.replace(
        "extern state character_reference @ \"html#character-reference-state\"",
        "extern state character_reference @ \"html#character-reference-state\" {\n  reaches { \
         switch bogus_commnet }\n}",
    );
    let e = refuse(&bad);
    assert!(e.contains("declares it reaches"), "{e}");
}
