//! The tree literal and the walks over it.

use hg_tree_traits::testing::TestTree;
use hg_tree_traits::{
    AttrNs, ElementState, NodeKind, Tree, ancestors, children, descendants, element_children,
    is_empty_element, is_html_element, is_root, ns, preceding_siblings,
};

const DOC: &str = r#"
html
  body
    div#main.wide.tall lang=en
      p.first
        "hello"
      p
        <!-- a comment -->
      span#empty
      svg|svg
        svg|circle cx=1
"#;

// Clippy counts a `#[test]` body as test code, but not a helper
// beside it; the panic here is a test failure like any other.
#[expect(
    clippy::panic,
    reason = "a fixture that will not parse is a test failure"
)]
fn doc() -> TestTree {
    match TestTree::parse(DOC) {
        Ok(t) => t,
        Err(e) => panic!("line {}: {}", e.line, e.message),
    }
}

#[test]
fn t01_the_literal_reads_elements_text_and_comments() {
    let t = doc();
    let Some(main) = t.find("main") else {
        panic!("there is a #main");
    };
    assert_eq!(t.local_name(main), Some("div"));
    assert_eq!(t.namespace(main), Some(ns::HTML));
    assert_eq!(t.attribute(main, AttrNs::None, "lang"), Some("en"));
    assert_eq!(t.attribute(main, AttrNs::None, "id"), Some("main"));
    assert_eq!(t.attribute(main, AttrNs::None, "class"), Some("wide tall"));
    assert_eq!(t.attribute(main, AttrNs::None, "absent"), None);
}

#[test]
fn t02_shorthands_keep_the_order_they_were_written_in() {
    let Ok(t) = TestTree::parse("div.a.b.c#x") else {
        panic!("that parses");
    };
    let Some(d) = t.find("x") else {
        panic!("there is an #x");
    };
    assert_eq!(t.attribute(d, AttrNs::None, "class"), Some("a b c"));
}

#[test]
fn t03_an_explicit_attribute_replaces_a_shorthand() {
    let Ok(t) = TestTree::parse(r#"div#x.a class="p q""#) else {
        panic!("that parses");
    };
    let Some(d) = t.find("x") else {
        panic!("there is an #x");
    };
    // One class attribute, not two: the tree cannot hold a duplicate.
    assert_eq!(t.attribute(d, AttrNs::None, "class"), Some("p q"));
}

#[test]
fn t04_a_prefix_names_a_namespace() {
    let t = doc();
    let circles: Vec<usize> = t
        .elements()
        .into_iter()
        .filter(|e| t.local_name(*e) == Some("circle"))
        .collect();
    let Some(circle) = circles.first().copied() else {
        panic!("there is a circle");
    };
    assert_eq!(t.namespace(circle), Some(ns::SVG));
    assert!(!is_html_element(&t, circle));
    assert_eq!(t.attribute(circle, AttrNs::None, "cx"), Some("1"));
}

#[test]
fn t05_an_attribute_namespace_is_asked_for_three_ways() {
    let Ok(t) = TestTree::parse("a#x xlink|href=/p href=/q") else {
        panic!("that parses");
    };
    let Some(a) = t.find("x") else {
        panic!("there is an #x");
    };
    assert_eq!(t.attribute(a, AttrNs::None, "href"), Some("/q"));
    assert_eq!(t.attribute(a, AttrNs::One(ns::XLINK), "href"), Some("/p"));
    // `[*|att]` takes the first in document order, which is how the
    // attribute list is written.
    assert_eq!(t.attribute(a, AttrNs::Any, "href"), Some("/p"));
    assert_eq!(t.attribute(a, AttrNs::One(ns::SVG), "href"), None);
}

#[test]
fn t06_the_walks_agree_with_the_shape() {
    let t = doc();
    let Some(main) = t.find("main") else {
        panic!("there is a #main");
    };
    // Two `p` and a `span` and an `svg` are element children; nothing
    // else is.
    let kids = element_children(&t, main);
    assert_eq!(kids.len(), 4);
    assert_eq!(t.local_name(kids.first().copied().unwrap_or(0)), Some("p"));
    assert_eq!(children(&t, main).len(), 4);

    let last = kids.last().copied().unwrap_or(0);
    assert_eq!(t.local_name(last), Some("svg"));
    assert_eq!(preceding_siblings(&t, last).len(), 3);

    // body, html, document.
    assert_eq!(ancestors(&t, main).len(), 3);
    assert_eq!(
        t.kind(ancestors(&t, main).last().copied().unwrap_or(0)),
        NodeKind::Document
    );
}

#[test]
fn t07_root_is_the_document_element_and_nothing_else() {
    let t = doc();
    let elements = t.elements();
    let roots: Vec<usize> = elements
        .iter()
        .copied()
        .filter(|e| is_root(&t, *e))
        .collect();
    assert_eq!(roots.len(), 1);
    assert_eq!(
        t.local_name(roots.first().copied().unwrap_or(0)),
        Some("html")
    );
}

#[test]
fn t08_empty_counts_elements_and_non_empty_text_only() {
    let t = doc();
    let Some(empty) = t.find("empty") else {
        panic!("there is a #empty");
    };
    assert!(is_empty_element(&t, empty));
    // A `p` holding only a comment is still empty, which is what
    // Selectors 3 says.
    let with_comment: Vec<usize> = t
        .elements()
        .into_iter()
        .filter(|e| {
            t.local_name(*e) == Some("p")
                && children(&t, *e)
                    .into_iter()
                    .all(|c| t.kind(c) == NodeKind::Comment)
        })
        .collect();
    let Some(p) = with_comment.first().copied() else {
        panic!("there is a p holding a comment");
    };
    assert!(is_empty_element(&t, p));
    // A `p` holding text is not.
    let Some(first) = t
        .elements()
        .into_iter()
        .find(|e| t.has_class(*e, "first", true))
    else {
        panic!("there is a .first");
    };
    assert!(!is_empty_element(&t, first));
}

#[test]
fn t09_descendants_are_in_tree_order_and_include_the_root() {
    let t = doc();
    let all = descendants(&t, t.document());
    assert_eq!(all.first().copied(), Some(t.document()));
    assert_eq!(all.len(), t.all().len());
    // Tree order: every node comes after its parent.
    for (i, n) in all.iter().enumerate() {
        if let Some(p) = t.parent(*n) {
            let Some(pi) = all.iter().position(|x| *x == p) else {
                panic!("the parent is in the list");
            };
            assert!(pi < i, "node {n} comes before its parent");
        }
    }
}

#[test]
fn t10_classes_match_case_sensitively_or_not_as_asked() {
    let t = doc();
    let Some(main) = t.find("main") else {
        panic!("there is a #main");
    };
    assert!(t.has_class(main, "wide", true));
    assert!(t.has_class(main, "tall", true));
    assert!(!t.has_class(main, "WIDE", true));
    assert!(t.has_class(main, "WIDE", false));
    assert!(!t.has_class(main, "wid", true));
}

#[test]
fn t11_state_bits_are_set_read_and_cleared() {
    let mut t = doc();
    let hover = ElementState::default().with_hover(true).with_focus(true);
    assert!(t.set_state("main", hover));
    assert!(!t.set_state("no-such-id", hover));
    let Some(main) = t.find("main") else {
        panic!("there is a #main");
    };
    let s = t.element_state(main);
    assert!(s.contains(ElementState::HOVER));
    assert!(s.contains(ElementState::FOCUS));
    assert!(!s.contains(ElementState::ACTIVE));
    assert!(
        s.contains(ElementState::HOVER.union(ElementState::FOCUS)),
        "contains asks for every bit, not any"
    );
    assert!(s.without(ElementState::FOCUS).contains(ElementState::HOVER));
    assert!(ElementState::default().is_empty());
    assert_eq!(ElementState::from_bits(s.bits()), s);
}

#[test]
fn t12_bad_indentation_is_refused_with_its_line() {
    let Err(e) = TestTree::parse("html\n   body") else {
        panic!("three spaces is not a level");
    };
    assert_eq!(e.line, 2);
    let Err(e) = TestTree::parse("html\n    body") else {
        panic!("two levels at once is not allowed");
    };
    assert_eq!(e.line, 2);
    let Err(e) = TestTree::parse("html\n  #id-with-no-name") else {
        panic!("an element needs a name");
    };
    assert_eq!(e.line, 2);
}
