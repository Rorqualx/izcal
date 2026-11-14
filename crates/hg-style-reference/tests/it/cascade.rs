//! The cascade: CSS Cascade 5 §6.1's six criteria.

use hg_style_reference::cascade::{Cascade, Origin, layer_rank, origin_rank};
use hg_tree_traits::testing::TestTree;

#[expect(
    clippy::panic,
    reason = "a fixture that will not parse is a test failure"
)]
fn tree(source: &str) -> TestTree {
    match TestTree::parse(source) {
        Ok(t) => t,
        Err(e) => panic!("line {}: {}", e.line, e.message),
    }
}

#[expect(clippy::panic, reason = "a missing fixture element is a test failure")]
fn value(t: &TestTree, id: &str, c: &Cascade, property: &str) -> Option<String> {
    let Some(node) = t.find(id) else {
        panic!("there is no #{id}");
    };
    c.value(t, node, property)
}

#[test]
fn c01_origin_and_importance_is_the_specs_eight_bucket_list() {
    // CSS Cascade 5 §6.1: "Declarations from origins earlier in this
    // list win over declarations from later origins." The list runs
    // transitions, important UA, important user, important author,
    // animations, normal author, normal user, normal UA.
    assert!(origin_rank(Origin::UserAgent, true) > origin_rank(Origin::User, true));
    assert!(origin_rank(Origin::User, true) > origin_rank(Origin::Author, true));
    assert!(origin_rank(Origin::Author, true) > origin_rank(Origin::Author, false));
    assert!(origin_rank(Origin::Author, false) > origin_rank(Origin::User, false));
    assert!(origin_rank(Origin::User, false) > origin_rank(Origin::UserAgent, false));
    // The two gaps are left where the spec puts them.
    assert!(hg_style_reference::cascade::transition_rank() > origin_rank(Origin::UserAgent, true));
    assert!(hg_style_reference::cascade::animation_rank() > origin_rank(Origin::Author, false));
    assert!(hg_style_reference::cascade::animation_rank() < origin_rank(Origin::Author, true));
}

#[test]
fn c02_important_reverses_the_origin_order() {
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add("p { color: ua }", Origin::UserAgent);
    c.add("p { color: author }", Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("author".to_owned()));

    let mut c = Cascade::new();
    c.add("p { color: ua !important }", Origin::UserAgent);
    c.add("p { color: author !important }", Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("ua".to_owned()));
}

#[test]
fn c03_specificity_then_order_decide_within_a_bucket() {
    let t = tree("html\n  p#x.c");
    let mut c = Cascade::new();
    c.add(
        "p { color: type } .c { color: class } #x { color: id }",
        Origin::Author,
    );
    assert_eq!(value(&t, "x", &c, "color"), Some("id".to_owned()));

    // Equal specificity: the later rule wins.
    let mut c = Cascade::new();
    c.add(".c { color: first } .c { color: second }", Origin::Author);
    assert_eq!(value(&t, "x", &c, "color"), Some("second".to_owned()));
}

#[test]
fn c04_normal_layers_are_later_wins_and_unlayered_beats_them_all() {
    // §6.1: "for normal rules the declaration whose cascade layer is
    // last wins", and unlayered styles are "an implicit final layer".
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(
        "@layer a, b; @layer a { p { color: a } } @layer b { p { color: b } }",
        Origin::Author,
    );
    assert_eq!(value(&t, "p", &c, "color"), Some("b".to_owned()));

    let mut c = Cascade::new();
    c.add(
        "@layer a, b; p { color: unlayered } @layer b { p { color: b } }",
        Origin::Author,
    );
    assert_eq!(value(&t, "p", &c, "color"), Some("unlayered".to_owned()));
}

#[test]
fn c05_important_layers_are_first_wins_and_unlayered_loses_to_them_all() {
    // §6.1: "for important rules the declaration whose cascade layer
    // is first wins". The implicit final layer is last, so it loses.
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(
        "@layer a, b; @layer a { p { color: a !important } } \
         @layer b { p { color: b !important } }",
        Origin::Author,
    );
    assert_eq!(value(&t, "p", &c, "color"), Some("a".to_owned()));

    let mut c = Cascade::new();
    c.add(
        "@layer a; p { color: unlayered !important } @layer a { p { color: a !important } }",
        Origin::Author,
    );
    assert_eq!(value(&t, "p", &c, "color"), Some("a".to_owned()));
}

#[test]
fn c06_the_layer_rank_is_a_mirror_image_either_side_of_importance() {
    // Three layers plus the implicit one. Normal: 0 < 1 < 2 < none.
    assert!(layer_rank(Some(0), 3, false) < layer_rank(Some(1), 3, false));
    assert!(layer_rank(Some(2), 3, false) < layer_rank(None, 3, false));
    // Important: none < 2 < 1 < 0.
    assert!(layer_rank(None, 3, true) < layer_rank(Some(2), 3, true));
    assert!(layer_rank(Some(2), 3, true) < layer_rank(Some(1), 3, true));
    assert!(layer_rank(Some(1), 3, true) < layer_rank(Some(0), 3, true));
}

#[test]
fn c07_a_layer_is_ordered_by_where_it_was_first_declared() {
    // §6.4.3: "Cascade layers are sorted by the order in which they
    // first are declared." Re-opening a layer does not move it.
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(
        "@layer a { p { color: a1 } } @layer b { p { color: b } } \
         @layer a { p { color: a2 } }",
        Origin::Author,
    );
    // `b` is still after `a`, although `a` was written last.
    assert_eq!(value(&t, "p", &c, "color"), Some("b".to_owned()));
}

#[test]
fn c08_the_style_attribute_beats_a_style_rule_of_the_same_importance() {
    // §6.1: "Separately for normal and important declarations,
    // declarations that are attached directly to an element ... take
    // precedence over declarations [of] the same importance that are
    // mapped via style rule."
    let t = tree("html\n  p id=p style=\"color: attached\"");
    let mut c = Cascade::new();
    c.add("#p { color: rule }", Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("attached".to_owned()));

    // Important the same way round.
    let t = tree("html\n  p id=p style=\"color: attached !important\"");
    let mut c = Cascade::new();
    c.add("#p { color: rule !important }", Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("attached".to_owned()));

    // But origin is compared first, so an important user-agent
    // declaration still beats it.
    let mut c = Cascade::new();
    c.add("p { color: ua !important }", Origin::UserAgent);
    assert_eq!(value(&t, "p", &c, "color"), Some("ua".to_owned()));
}

#[test]
fn c09_important_beats_normal_inside_one_origin() {
    let t = tree("html\n  p#x.c");
    let mut c = Cascade::new();
    c.add(
        "#x { color: specific } .c { color: important !important }",
        Origin::Author,
    );
    // A class with `!important` beats an id without it: importance is
    // compared before specificity.
    assert_eq!(value(&t, "x", &c, "color"), Some("important".to_owned()));
}

#[test]
fn c10_a_declaration_block_drops_what_it_cannot_read_and_keeps_the_rest() {
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(
        "p { color: red; ; not-a-declaration; width: 1px }",
        Origin::Author,
    );
    assert_eq!(value(&t, "p", &c, "color"), Some("red".to_owned()));
    assert_eq!(value(&t, "p", &c, "width"), Some("1px".to_owned()));
}

#[test]
fn c11_what_the_reference_engine_cannot_read_it_records() {
    let mut c = Cascade::new();
    c.add(
        "@media print { p { color: red } } p:has(a) { color: blue } p { color: green }",
        Origin::Author,
    );
    let Some(sheet) = c.sheets().first() else {
        panic!("there is a sheet");
    };
    assert_eq!(sheet.skipped.len(), 1, "@media is skipped, not guessed at");
    assert_eq!(sheet.refused.len(), 1, ":has() is refused by name");
    // The rule it could read is still there.
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn c12_a_pseudo_element_gets_its_own_declarations() {
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(
        "p { color: element } p::before { color: before }",
        Origin::Author,
    );
    let Some(node) = t.find("p") else {
        panic!("there is a #p");
    };
    assert_eq!(
        c.winners(&t, node, None)
            .get("color")
            .map(|w| w.declaration.value.clone()),
        Some("element".to_owned())
    );
    assert_eq!(
        c.winners(&t, node, Some("before"))
            .get("color")
            .map(|w| w.declaration.value.clone()),
        Some("before".to_owned())
    );
}

#[test]
fn c13_namespaces_from_the_sheet_reach_its_selectors() {
    let t = tree("html\n  svg|svg id=g");
    let mut c = Cascade::new();
    c.add(
        "@namespace s url(http://www.w3.org/2000/svg); s|svg { color: svg }",
        Origin::Author,
    );
    assert_eq!(value(&t, "g", &c, "color"), Some("svg".to_owned()));
}

#[test]
fn c14_property_names_are_lower_cased_and_values_are_not() {
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add("p { COLOR: Red }", Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("Red".to_owned()));
    assert_eq!(value(&t, "p", &c, "COLOR"), Some("Red".to_owned()));
}

#[test]
fn c15_order_of_appearance_runs_across_sheets_not_within_one() {
    // §6.1: order of appearance is document order. A later sheet's
    // first rule beats an earlier sheet's last at equal specificity —
    // holding the order per sheet gets this backwards.
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(
        "p { color: a } p { color: b } p { color: c }",
        Origin::Author,
    );
    c.add("p { color: later }", Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("later".to_owned()));
}

#[test]
fn c16_a_layer_declared_in_two_sheets_of_one_origin_is_one_layer() {
    // §6.4.3: layers belong to an origin. Two sheets that both say
    // `@layer base` name the same layer, and its place is where the
    // origin first declared it.
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add("@layer base, theme;", Origin::Author);
    c.add("@layer theme { p { color: theme } }", Origin::Author);
    c.add("@layer base { p { color: base } }", Origin::Author);
    // `theme` is after `base` in the origin's order, although its rule
    // was written first.
    assert_eq!(value(&t, "p", &c, "color"), Some("theme".to_owned()));
}

#[test]
fn c17_two_anonymous_layers_in_two_sheets_stay_two_layers() {
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add("@layer { p { color: first } }", Origin::Author);
    c.add("@layer { p { color: second } }", Origin::Author);
    // The second is later in the origin's layer order, so it wins.
    assert_eq!(value(&t, "p", &c, "color"), Some("second".to_owned()));
}

#[test]
fn c18_layers_of_different_origins_do_not_interfere() {
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add("@layer a { p { color: ua } }", Origin::UserAgent);
    c.add("@layer a { p { color: author } }", Origin::Author);
    // Origin is compared first, whatever the layers are called.
    assert_eq!(value(&t, "p", &c, "color"), Some("author".to_owned()));
}

#[test]
fn c19_custom_property_names_are_case_sensitive() {
    // Custom Properties 1: "custom property names are case-sensitive".
    // Ordinary property names are not.
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(
        "p { --Foo: upper; --foo: lower; COLOR: red }",
        Origin::Author,
    );
    assert_eq!(value(&t, "p", &c, "--Foo"), Some("upper".to_owned()));
    assert_eq!(value(&t, "p", &c, "--foo"), Some("lower".to_owned()));
    assert_eq!(value(&t, "p", &c, "color"), Some("red".to_owned()));
    let Some(node) = t.find("p") else {
        panic!("there is a #p");
    };
    assert_eq!(c.winners(&t, node, None).len(), 3);
}

#[test]
fn c20_a_value_holding_a_quote_survives_the_sheet_reader() {
    // The block's tokens go straight to the declaration reader rather
    // than through text and back, so a quote inside a string cannot
    // re-tokenize into something else.
    let t = tree("html\n  p id=p");
    let mut c = Cascade::new();
    c.add(r#"p { content: "a\"b"; color: red }"#, Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("red".to_owned()));
    let got = value(&t, "p", &c, "content");
    assert!(
        got.as_deref()
            .is_some_and(|v| v.contains("a") && v.contains("b")),
        "the value kept both halves: {got:?}"
    );
    // A semicolon inside a string does not end the declaration.
    let mut c = Cascade::new();
    c.add(r#"p { content: "a;b"; color: blue }"#, Origin::Author);
    assert_eq!(value(&t, "p", &c, "color"), Some("blue".to_owned()));
}
