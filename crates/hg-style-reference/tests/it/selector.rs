//! Selectors 3: parsing, specificity and matching.

use hg_style_reference::selector::{
    AnPlusB, Simple, Specificity, matches, matches_list, parse, parse_in,
};
use hg_tree_traits::testing::TestTree;
use hg_tree_traits::{ElementState, Tree, ns};

const DOC: &str = r#"
html lang=en
  body
    div#main.wide
      p.a id=p1
        "text"
      p.b id=p2
      span id=s1 title=hello data-x="one two"
      span id=s2
      a id=link href=/x
      input id=box type=checkbox
      ul id=list
        li id=li1
        li id=li2
        li id=li3
        li id=li4
    div#other lang=fr-CA
      p id=p3
RSEOF_MARKER
"#;

#[expect(
    clippy::panic,
    reason = "a fixture that will not parse is a test failure"
)]
fn doc() -> TestTree {
    let source = DOC.replace("RSEOF_MARKER\n", "");
    match TestTree::parse(&source) {
        Ok(t) => t,
        Err(e) => panic!("line {}: {}", e.line, e.message),
    }
}

/// The ids of every element the selector matches, in tree order.
#[expect(
    clippy::panic,
    reason = "a selector that will not parse is a test failure"
)]
fn ids(tree: &TestTree, selector: &str) -> Vec<String> {
    let list = match parse(selector) {
        Ok(l) => l,
        Err(e) => panic!("{selector}: {}", e.message),
    };
    tree.elements()
        .into_iter()
        .filter(|n| matches_list(tree, *n, &list).is_some())
        .map(|n| tree.element_id(n).unwrap_or("").to_owned())
        .collect()
}

#[test]
fn s01_the_five_simple_selectors() {
    let t = doc();
    assert_eq!(ids(&t, "#main"), ["main"]);
    assert_eq!(ids(&t, ".wide"), ["main"]);
    assert_eq!(ids(&t, "li"), ["li1", "li2", "li3", "li4"]);
    assert_eq!(ids(&t, "[title]"), ["s1"]);
    assert_eq!(ids(&t, "p.a"), ["p1"]);
    // `*` matches every element, including the ones with no id.
    assert_eq!(ids(&t, "*").len(), t.elements().len());
}

#[test]
fn s02_the_six_attribute_comparisons() {
    let t = doc();
    assert_eq!(ids(&t, "[title=hello]"), ["s1"]);
    assert_eq!(ids(&t, "[title=\"hello\"]"), ["s1"]);
    assert_eq!(ids(&t, "[data-x~=two]"), ["s1"]);
    assert_eq!(ids(&t, "[data-x~=\"one two\"]"), Vec::<String>::new());
    assert_eq!(ids(&t, "[lang|=fr]"), ["other"]);
    assert_eq!(ids(&t, "[lang|=en]"), [""]);
    assert_eq!(ids(&t, "[title^=hel]"), ["s1"]);
    assert_eq!(ids(&t, "[title$=llo]"), ["s1"]);
    assert_eq!(ids(&t, "[title*=ell]"), ["s1"]);
}

#[test]
fn s03_an_empty_value_matches_nothing_for_the_three_substring_forms() {
    // Selectors 3 §6.3.2: "If the value is the empty string then the
    // selector does not represent anything."
    let t = doc();
    assert_eq!(ids(&t, "[title^=\"\"]"), Vec::<String>::new());
    assert_eq!(ids(&t, "[title$=\"\"]"), Vec::<String>::new());
    assert_eq!(ids(&t, "[title*=\"\"]"), Vec::<String>::new());
    assert_eq!(ids(&t, "[data-x~=\"\"]"), Vec::<String>::new());
    // `=` is not one of them: an empty exact value is a real test.
    assert_eq!(ids(&t, "[title=\"\"]"), Vec::<String>::new());
}

#[test]
fn s04_the_four_combinators() {
    let t = doc();
    assert_eq!(ids(&t, "div p"), ["p1", "p2", "p3"]);
    assert_eq!(ids(&t, "#main > p"), ["p1", "p2"]);
    assert_eq!(ids(&t, "#p1 + p"), ["p2"]);
    assert_eq!(ids(&t, "#p1 ~ span"), ["s1", "s2"]);
    // `+` skips a text node between the two elements.
    assert_eq!(ids(&t, "p.a + p.b"), ["p2"]);
    assert_eq!(ids(&t, "html body div p"), ["p1", "p2", "p3"]);
}

#[test]
fn s05_the_structural_pseudo_classes_count_elements_only() {
    let t = doc();
    assert_eq!(ids(&t, "li:first-child"), ["li1"]);
    assert_eq!(ids(&t, "li:last-child"), ["li4"]);
    assert_eq!(ids(&t, "li:nth-child(2)"), ["li2"]);
    assert_eq!(ids(&t, "li:nth-child(odd)"), ["li1", "li3"]);
    assert_eq!(ids(&t, "li:nth-child(even)"), ["li2", "li4"]);
    assert_eq!(ids(&t, "li:nth-child(2n+1)"), ["li1", "li3"]);
    assert_eq!(ids(&t, "li:nth-last-child(1)"), ["li4"]);
    assert_eq!(ids(&t, "li:nth-last-child(2)"), ["li3"]);
    assert_eq!(ids(&t, "#list li:only-child"), Vec::<String>::new());
    assert_eq!(ids(&t, "#list:only-child"), Vec::<String>::new());
}

#[test]
fn s06_of_type_counts_only_siblings_of_the_same_name() {
    let t = doc();
    // `#main` holds p, p, span, span, a, input, ul.
    assert_eq!(
        ids(&t, "#main > :first-of-type"),
        ["p1", "s1", "link", "box", "list"]
    );
    assert_eq!(ids(&t, "#main > span:last-of-type"), ["s2"]);
    assert_eq!(ids(&t, "#main > a:only-of-type"), ["link"]);
    assert_eq!(ids(&t, "#main > p:nth-of-type(2)"), ["p2"]);
    assert_eq!(ids(&t, "#main > span:nth-last-of-type(2)"), ["s1"]);
}

#[test]
fn s07_an_plus_b_takes_non_negative_n_only() {
    // Selectors 3 §6.6.5: "the index ... n can be zero or any positive
    // integer". So `-n+3` is the first three, and `2n+1` never reaches
    // index 0.
    assert!(AnPlusB { a: -1, b: 3 }.matches(1));
    assert!(AnPlusB { a: -1, b: 3 }.matches(3));
    assert!(!AnPlusB { a: -1, b: 3 }.matches(4));
    assert!(AnPlusB { a: 2, b: 1 }.matches(1));
    assert!(!AnPlusB { a: 2, b: 1 }.matches(2));
    assert!(AnPlusB { a: 0, b: 3 }.matches(3));
    assert!(!AnPlusB { a: 0, b: 3 }.matches(2));
    let t = doc();
    assert_eq!(ids(&t, "li:nth-child(-n+2)"), ["li1", "li2"]);
    assert_eq!(ids(&t, "li:nth-child(n)"), ["li1", "li2", "li3", "li4"]);
    assert_eq!(ids(&t, "li:nth-child( 2n + 1 )"), ["li1", "li3"]);
}

#[test]
fn s08_empty_root_and_lang() {
    let t = doc();
    assert_eq!(ids(&t, ":root"), [""]);
    // Both spans are childless, so both are `:empty`; the attributes
    // on #s1 are not children.
    assert_eq!(ids(&t, "span:empty"), ["s1", "s2"]);
    // `:lang()` reads the nearest ancestor's lang, and matches on
    // hyphen boundaries — `fr` matches `fr-CA`.
    assert_eq!(ids(&t, "p:lang(fr)"), ["p3"]);
    assert_eq!(ids(&t, "p:lang(fr-CA)"), ["p3"]);
    assert_eq!(ids(&t, "p:lang(f)"), Vec::<String>::new());
    assert_eq!(ids(&t, "p:lang(en)"), ["p1", "p2"]);
}

#[test]
fn s09_state_pseudo_classes_read_the_bits_and_never_compute_them() {
    let mut t = doc();
    assert_eq!(ids(&t, ":hover"), Vec::<String>::new());
    assert!(t.set_state("link", ElementState::default().with_hover(true)));
    assert!(t.set_state("box", ElementState::default().with_checked(true)));
    assert_eq!(ids(&t, "a:hover"), ["link"]);
    assert_eq!(ids(&t, "input:checked"), ["box"]);
    assert_eq!(ids(&t, "input:disabled"), Vec::<String>::new());
}

#[test]
fn s10_not_takes_exactly_one_simple_selector() {
    let t = doc();
    assert_eq!(ids(&t, "#main > p:not(.a)"), ["p2"]);
    assert_eq!(
        ids(&t, "#list > li:not(:first-child)"),
        ["li2", "li3", "li4"]
    );
    // Level 3 forbids a nested negation and a pseudo-element inside.
    assert!(parse(":not(:not(a))").is_err());
    assert!(parse(":not(::before)").is_err());
    // It also forbids a compound: one simple selector only.
    assert!(parse(":not(a.b)").is_err());
}

#[test]
fn s11_specificity_is_a_tuple_and_never_carries() {
    // Selectors 3 §9's own examples.
    let cases: [(&str, Specificity); 7] = [
        ("*", Specificity { a: 0, b: 0, c: 0 }),
        ("li", Specificity { a: 0, b: 0, c: 1 }),
        ("ul li", Specificity { a: 0, b: 0, c: 2 }),
        ("ul ol+li", Specificity { a: 0, b: 0, c: 3 }),
        ("h1 + *[rel=up]", Specificity { a: 0, b: 1, c: 1 }),
        ("ul ol li.red", Specificity { a: 0, b: 1, c: 3 }),
        ("#x34y", Specificity { a: 1, b: 0, c: 0 }),
    ];
    for (text, want) in cases {
        let Ok(list) = parse(text) else {
            panic!("{text} parses");
        };
        let Some(first) = list.selectors.first() else {
            panic!("{text} has a selector");
        };
        assert_eq!(first.specificity(), want, "{text}");
    }
    // Eleven classes do not become one id: (0,11,0) loses to (1,0,0).
    let eleven = ".a".repeat(11);
    let Ok(list) = parse(&eleven) else {
        panic!("that parses");
    };
    let Some(first) = list.selectors.first() else {
        panic!("there is a selector");
    };
    assert_eq!(first.specificity(), Specificity { a: 0, b: 11, c: 0 });
    assert!(first.specificity() < Specificity { a: 1, b: 0, c: 0 });
}

#[test]
fn s12_a_negation_counts_its_argument_and_not_itself() {
    // §9: "the negation pseudo-class itself does not count".
    let Ok(list) = parse(":not(.c)") else {
        panic!("that parses");
    };
    let Some(first) = list.selectors.first() else {
        panic!("there is a selector");
    };
    assert_eq!(first.specificity(), Specificity { a: 0, b: 1, c: 0 });
    let Ok(list) = parse(":not(#i)") else {
        panic!("that parses");
    };
    let Some(first) = list.selectors.first() else {
        panic!("there is a selector");
    };
    assert_eq!(first.specificity(), Specificity { a: 1, b: 0, c: 0 });
}

#[test]
fn s13_a_pseudo_element_counts_as_an_element_name_and_matches_no_element() {
    let t = doc();
    let Ok(list) = parse("p::before") else {
        panic!("that parses");
    };
    let Some(first) = list.selectors.first() else {
        panic!("there is a selector");
    };
    assert_eq!(first.specificity(), Specificity { a: 0, b: 0, c: 2 });
    assert_eq!(first.pseudo_element(), Some("before"));
    // The element still matches; the cascade decides which box the
    // declarations reach.
    assert_eq!(ids(&t, "p::before"), ["p1", "p2", "p3"]);
    // One colon is accepted for the four legacy pseudo-elements.
    let Ok(list) = parse("p:before") else {
        panic!("that parses");
    };
    assert_eq!(
        list.selectors.first().and_then(|s| s.pseudo_element()),
        Some("before")
    );
    assert!(parse("p:nth-of-nothing").is_err());
}

#[test]
fn s14_element_names_match_case_insensitively_only_in_html() {
    let mut t = doc();
    assert_eq!(ids(&t, "LI").len(), 4);
    t.set_html_document(false);
    let Ok(list) = parse("LI") else {
        panic!("that parses");
    };
    let any = t
        .elements()
        .into_iter()
        .any(|n| matches_list(&t, n, &list).is_some());
    assert!(!any, "outside HTML, `LI` is not `li`");
}

#[test]
fn s15_a_namespace_prefix_must_be_declared() {
    let Ok(t) = TestTree::parse("html\n  svg|svg id=g\n    svg|circle id=c") else {
        panic!("that parses");
    };
    let prefixes = vec![("s".to_owned(), ns::SVG.to_owned())];
    let Ok(list) = parse_in("s|circle", None, &prefixes) else {
        panic!("that parses");
    };
    let hit: Vec<&str> = t
        .elements()
        .into_iter()
        .filter(|n| {
            matches(
                &t,
                *n,
                list.selectors.first().unwrap_or(&Default::default()),
            )
        })
        .map(|n| t.element_id(n).unwrap_or(""))
        .collect();
    assert_eq!(hit, ["c"]);
    // An undeclared prefix is an error, not a silent non-match.
    assert!(parse_in("q|circle", None, &prefixes).is_err());
    // `*|name` needs no declaration.
    assert!(parse("*|circle").is_ok());
}

#[test]
fn s16_a_bare_name_takes_the_default_namespace_but_a_bare_attribute_does_not() {
    // Selectors 3 §6.3.2: "the default namespace does not apply to
    // attribute selectors".
    let Ok(list) = parse_in("[href]", Some(ns::SVG), &[]) else {
        panic!("that parses");
    };
    let Some(first) = list.selectors.first() else {
        panic!("there is a selector");
    };
    let Some(compound) = first.compounds.first() else {
        panic!("there is a compound");
    };
    let Some(Simple::Attribute(name_match, _, _)) = compound.simples.first() else {
        panic!("it is an attribute selector");
    };
    assert_eq!(*name_match, hg_style_reference::selector::NameMatch::None);
}

#[test]
fn s17_a_selector_list_matches_if_any_member_does_and_takes_the_highest() {
    let t = doc();
    assert_eq!(ids(&t, "#p1, #p2"), ["p1", "p2"]);
    let Ok(list) = parse("p, #p1") else {
        panic!("that parses");
    };
    let Some(p1) = t.find("p1") else {
        panic!("there is a #p1");
    };
    // Two members match; the more specific one's specificity is
    // returned.
    assert_eq!(
        matches_list(&t, p1, &list),
        Some(Specificity { a: 1, b: 0, c: 0 })
    );
}

#[test]
fn s18_level_four_selectors_are_refused_by_name_not_half_matched() {
    for s in [
        ":has(p)",
        ":is(a, b)",
        ":where(a)",
        "[a=b i]",
        ":focus-within",
    ] {
        let got = parse(s);
        assert!(got.is_err(), "{s} is not Selectors 3");
    }
}

#[test]
fn s19_malformed_selectors_are_errors() {
    for s in [
        "", "div >", "> div", "div ##x", ".", "[a", "a[", "div,", ":",
    ] {
        assert!(parse(s).is_err(), "{s:?} should not parse");
    }
}

#[test]
fn s20_a_type_selector_must_lead_its_compound() {
    assert!(parse(".a div").is_ok());
    assert!(parse(".adiv").is_ok());
    assert!(parse("div.a").is_ok());
    assert!(parse(".a|div").is_err());
}
