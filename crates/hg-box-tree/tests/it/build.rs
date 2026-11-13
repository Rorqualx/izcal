//! Box generation from a DOM tree and a style source.

use std::collections::BTreeMap;

use hg_box_tree::{BoxKind, BoxTree, Display, Inner, Source, StyleSource, build};
use hg_tree_traits::testing::TestTree;
use hg_tree_traits::{Tree, is_element};

/// A style source a test writes down: `display` per element id, with
/// everything else taking Display 3's default for its element name.
struct Styles {
    by_slot: BTreeMap<u32, Display>,
}

impl Styles {
    fn new() -> Self {
        Self {
            by_slot: BTreeMap::new(),
        }
    }

    fn set(&mut self, slot: u32, display: Display) {
        self.by_slot.insert(slot, display);
    }
}

impl StyleSource for Styles {
    fn display(&self, node: u32) -> Display {
        self.by_slot.get(&node).copied().unwrap_or(Display::BLOCK)
    }

    fn style_id(&self, node: u32) -> u32 {
        node
    }

    fn anonymous_style_id(&self, parent_style: u32) -> u32 {
        // Distinguishable from a real style id, so a test can see that
        // the anonymous box took one.
        parent_style.saturating_add(1_000)
    }
}

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

/// The styles for a tree, with every element named in `inline` given
/// `display: inline`.
fn styles(t: &TestTree, inline: &[&str]) -> Styles {
    let mut s = Styles::new();
    for node in t.elements() {
        let slot = t.id(node).slot;
        let name = t.local_name(node).unwrap_or("");
        let is_inline =
            inline.contains(&name) || t.element_id(node).is_some_and(|id| inline.contains(&id));
        s.set(
            slot,
            if is_inline {
                Display::INLINE
            } else {
                Display::BLOCK
            },
        );
    }
    s
}

fn kinds(bt: &BoxTree, id: hg_box_tree::BoxId) -> Vec<BoxKind> {
    bt.children(id)
        .into_iter()
        .filter_map(|c| bt.get(c).map(|b| b.kind))
        .collect()
}

#[test]
fn b01_one_box_per_element_and_the_root_is_the_document_element() {
    let t = tree("html\n  body\n    div#a\n    div#b");
    let bt = build(&t, &styles(&t, &[]));
    let Some(root) = bt.root() else {
        panic!("there is a root box");
    };
    let Some(record) = bt.get(root) else {
        panic!("the root is there");
    };
    assert_eq!(record.kind, BoxKind::Block);
    assert!(record.is_bfc_root, "the root box always establishes a BFC");
    assert!(record.relayout_boundary);
    assert_eq!(bt.len(), 4, "html, body and two divs");
    assert_eq!(bt.descendants(root).len(), 4);
}

#[test]
fn b02_the_node_to_box_map_goes_both_ways() {
    let t = tree("html\n  body\n    div#a");
    let bt = build(&t, &styles(&t, &[]));
    let Some(node) = t.find("a") else {
        panic!("there is an #a");
    };
    let slot = t.id(node).slot;
    let Some(id) = bt.box_of(slot) else {
        panic!("#a generates a box");
    };
    let Some(record) = bt.get(id) else {
        panic!("the box is there");
    };
    assert_eq!(record.source, Source::Node(slot));
}

#[test]
fn b03_display_none_generates_no_box_and_no_box_for_the_subtree() {
    let t = tree("html\n  body\n    div#gone\n      p#inside\n    div#kept");
    let mut s = styles(&t, &[]);
    let Some(gone) = t.find("gone") else {
        panic!("there is a #gone");
    };
    s.set(t.id(gone).slot, Display::None);
    let bt = build(&t, &s);
    assert_eq!(bt.box_of(t.id(gone).slot), None);
    let Some(inside) = t.find("inside") else {
        panic!("there is an #inside");
    };
    assert_eq!(bt.box_of(t.id(inside).slot), None, "the subtree goes too");
    let Some(kept) = t.find("kept") else {
        panic!("there is a #kept");
    };
    assert!(bt.box_of(t.id(kept).slot).is_some());
    assert_eq!(bt.len(), 3, "html, body, #kept");
}

#[test]
fn b04_display_contents_generates_no_box_but_its_children_do() {
    // The difference from `none`, and the whole point of `contents`.
    let t = tree("html\n  body\n    div#wrapper\n      p#one\n      p#two");
    let mut s = styles(&t, &[]);
    let Some(w) = t.find("wrapper") else {
        panic!("there is a #wrapper");
    };
    s.set(t.id(w).slot, Display::Contents);
    let bt = build(&t, &s);
    assert_eq!(bt.box_of(t.id(w).slot), None, "no box of its own");
    let Some(one) = t.find("one") else {
        panic!("there is a #one");
    };
    assert!(bt.box_of(t.id(one).slot).is_some());
    // The children take its place in the parent, so body has two
    // block children and not one.
    let Some(body) = t.find("wrapper").and_then(|n| t.parent(n)) else {
        panic!("there is a body");
    };
    let Some(body_box) = bt.box_of(t.id(body).slot) else {
        panic!("body generates a box");
    };
    assert_eq!(bt.children(body_box).len(), 2);
}

#[test]
fn b05_an_inline_only_parent_is_an_inline_formatting_context() {
    // Nothing to wrap: no block-level child, so no anonymous box.
    let t = tree("html\n  body\n    p\n      span#a\n      span#b");
    let bt = build(&t, &styles(&t, &["span"]));
    let Some(p) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("p"))
    else {
        panic!("there is a p");
    };
    let Some(p_box) = bt.box_of(t.id(p).slot) else {
        panic!("p generates a box");
    };
    assert_eq!(kinds(&bt, p_box), [BoxKind::Inline, BoxKind::Inline]);
}

#[test]
fn b06_a_block_among_inlines_wraps_each_run_in_an_anonymous_block() {
    // CSS 2.1 9.2.1.1, and the part that is easy to get wrong: the
    // rule is about *runs*, so `a <p/> c` gives three children, not
    // four.
    let t = tree("html\n  body\n    div\n      span#a\n      p#block\n      span#c");
    let bt = build(&t, &styles(&t, &["span"]));
    let Some(div) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("div"))
    else {
        panic!("there is a div");
    };
    let Some(div_box) = bt.box_of(t.id(div).slot) else {
        panic!("div generates a box");
    };
    assert_eq!(
        kinds(&bt, div_box),
        [
            BoxKind::AnonymousBlock,
            BoxKind::Block,
            BoxKind::AnonymousBlock
        ]
    );
    // Each anonymous box holds its run.
    let kids = bt.children(div_box);
    let Some(first) = kids.first().copied() else {
        panic!("there is a first child");
    };
    assert_eq!(bt.children(first).len(), 1);
    let Some(anon) = bt.get(first) else {
        panic!("the box is there");
    };
    assert_eq!(anon.source, Source::Anonymous);
}

#[test]
fn b07_a_run_of_several_inlines_is_one_anonymous_box_not_several() {
    let t = tree(
        "html\n  body\n    div\n      span#a\n      span#b\n      p#block\n      span#c\n      \
         span#d",
    );
    let bt = build(&t, &styles(&t, &["span"]));
    let Some(div) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("div"))
    else {
        panic!("there is a div");
    };
    let Some(div_box) = bt.box_of(t.id(div).slot) else {
        panic!("div generates a box");
    };
    let kids = bt.children(div_box);
    assert_eq!(kids.len(), 3);
    let Some(first) = kids.first().copied() else {
        panic!("there is a first");
    };
    let Some(last) = kids.last().copied() else {
        panic!("there is a last");
    };
    assert_eq!(
        bt.children(first).len(),
        2,
        "both spans in one anonymous box"
    );
    assert_eq!(bt.children(last).len(), 2);
}

#[test]
fn b08_an_anonymous_box_takes_a_style_of_its_own() {
    // CSS 2.1 9.2.1.1: it inherits from its parent and non-inherited
    // properties take their initial values, so it is not its
    // children's style.
    let t = tree("html\n  body\n    div\n      span#a\n      p#block");
    let bt = build(&t, &styles(&t, &["span"]));
    let Some(div) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("div"))
    else {
        panic!("there is a div");
    };
    let Some(div_box) = bt.box_of(t.id(div).slot) else {
        panic!("div generates a box");
    };
    let Some(anon_id) = bt.children(div_box).first().copied() else {
        panic!("there is an anonymous box");
    };
    let Some(anon) = bt.get(anon_id) else {
        panic!("the box is there");
    };
    let Some(span) = t.find("a") else {
        panic!("there is an #a");
    };
    assert_ne!(anon.style_id, t.id(span).slot);
    assert!(anon.style_id >= 1_000, "it took an anonymous style");
}

#[test]
fn b09_whitespace_between_blocks_generates_no_box() {
    let t = tree("html\n  body\n    \" \"\n    div#a\n    \" \"\n    div#b");
    let bt = build(&t, &styles(&t, &[]));
    assert_eq!(bt.len(), 4, "html, body and two divs; no text boxes");
}

#[test]
fn b10_text_with_content_generates_a_text_box() {
    let t = tree("html\n  body\n    p\n      \"hello\"");
    let bt = build(&t, &styles(&t, &[]));
    let Some(p) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("p"))
    else {
        panic!("there is a p");
    };
    let Some(p_box) = bt.box_of(t.id(p).slot) else {
        panic!("p generates a box");
    };
    assert_eq!(kinds(&bt, p_box), [BoxKind::Text]);
}

#[test]
fn b11_text_beside_a_block_is_wrapped_like_any_inline_level_content() {
    let t = tree("html\n  body\n    div\n      \"before\"\n      p#block\n      \"after\"");
    let bt = build(&t, &styles(&t, &[]));
    let Some(div) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("div"))
    else {
        panic!("there is a div");
    };
    let Some(div_box) = bt.box_of(t.id(div).slot) else {
        panic!("div generates a box");
    };
    assert_eq!(
        kinds(&bt, div_box),
        [
            BoxKind::AnonymousBlock,
            BoxKind::Block,
            BoxKind::AnonymousBlock
        ]
    );
}

#[test]
fn b12_an_inner_display_decides_whether_a_box_establishes_a_bfc() {
    let t = tree("html\n  body\n    div#flow\n    div#root\n    div#flex");
    let mut s = styles(&t, &[]);
    for (id, display) in [
        ("flow", Display::BLOCK),
        ("root", Display::FLOW_ROOT),
        ("flex", Display::FLEX),
    ] {
        let Some(node) = t.find(id) else {
            panic!("there is a #{id}");
        };
        s.set(t.id(node).slot, display);
    }
    let bt = build(&t, &s);
    for (id, want, inner) in [
        ("flow", false, Inner::Flow),
        ("root", true, Inner::FlowRoot),
        ("flex", true, Inner::Flex),
    ] {
        let Some(node) = t.find(id) else {
            panic!("there is a #{id}");
        };
        let Some(b) = bt.box_of(t.id(node).slot).and_then(|x| bt.get(x)) else {
            panic!("#{id} generates a box");
        };
        assert_eq!(b.is_bfc_root, want, "#{id}");
        assert_eq!(b.inner, inner, "#{id}");
    }
}

#[test]
fn b13_rebuilding_gives_a_bit_identical_tree() {
    // Box columns are class (a) derived data (FP 6.4, LY-32).
    let t = tree("html\n  body\n    div\n      span#a\n      p#b\n      \"c\"");
    let s = styles(&t, &["span"]);
    let first = build(&t, &s);
    let second = build(&t, &s);
    assert_eq!(first.len(), second.len());
    assert_eq!(first.all(), second.all());
    for id in first.all() {
        assert_eq!(first.get(id), second.get(id), "box {}", id.index());
    }
}

#[test]
fn b14_a_document_with_no_element_generates_no_root() {
    let Ok(empty) = TestTree::parse("") else {
        panic!("an empty literal parses");
    };
    let bt = build(&empty, &Styles::new());
    assert_eq!(bt.root(), None);
    assert!(bt.is_empty());
    assert!(!is_element(&empty, empty.document()));
}

#[test]
fn b15_an_anonymous_box_inherits_from_the_parent_and_not_from_its_run() {
    // §9.2.1.1 says the parent. Taking the first child's style would
    // give a *text* child's id, which is zero.
    let t = tree("html\n  body\n    div#holder\n      \"text\"\n      p#block");
    let bt = build(&t, &styles(&t, &[]));
    let Some(holder) = t.find("holder") else {
        panic!("there is a #holder");
    };
    let holder_slot = t.id(holder).slot;
    let Some(holder_box) = bt.box_of(holder_slot) else {
        panic!("it generates a box");
    };
    let Some(anon_id) = bt.children(holder_box).first().copied() else {
        panic!("there is an anonymous box");
    };
    let Some(anon) = bt.get(anon_id) else {
        panic!("the box is there");
    };
    // The fixture's anonymous style id is the parent's plus 1,000.
    assert_eq!(anon.style_id, holder_slot.saturating_add(1_000));
}

#[test]
fn b16_non_breaking_space_is_not_css_white_space() {
    // CSS white space is space, tab, line feed, carriage return and
    // form feed. Unicode's `White_Space` holds U+00A0 and a dozen
    // more, and trimming by it would drop a box the page needs.
    let t = tree("html\n  body\n    p\n      \"\u{A0}\"");
    let bt = build(&t, &styles(&t, &[]));
    let Some(p) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("p"))
    else {
        panic!("there is a p");
    };
    let Some(p_box) = bt.box_of(t.id(p).slot) else {
        panic!("p generates a box");
    };
    assert_eq!(kinds(&bt, p_box), [BoxKind::Text], "nbsp generates a box");

    // An ordinary space, tab or line feed still does not. The tree
    // literal has no escapes, so the tab goes in as itself.
    let src = format!("html\n  body\n    p\n      \" {} \"", '\t');
    let t = tree(&src);
    let bt = build(&t, &styles(&t, &[]));
    let Some(p) = t
        .elements()
        .into_iter()
        .find(|n| t.local_name(*n) == Some("p"))
    else {
        panic!("there is a p");
    };
    let Some(p_box) = bt.box_of(t.id(p).slot) else {
        panic!("p generates a box");
    };
    assert_eq!(kinds(&bt, p_box), []);
}

#[test]
fn b17_a_contents_elements_children_are_wrapped_once_by_the_real_parent() {
    // `display: contents` means its children are the parent's for the
    // anonymous-box rule too. Wrapping them inside it *and* again in
    // the parent would make an inline sibling see an anonymous block
    // where the specification says it sees inline-level content.
    let t = tree(
        "html\n  body\n    div#outer\n      span#a\n      span#c1\n        span#x\n        \
         span#y\n      p#block",
    );
    let mut s = styles(&t, &["span"]);
    let Some(c1) = t.find("c1") else {
        panic!("there is a #c1");
    };
    s.set(t.id(c1).slot, Display::Contents);
    let bt = build(&t, &s);
    let Some(outer) = t.find("outer") else {
        panic!("there is an #outer");
    };
    let Some(outer_box) = bt.box_of(t.id(outer).slot) else {
        panic!("it generates a box");
    };
    // One anonymous block holding #a, #x and #y, then the p.
    assert_eq!(
        kinds(&bt, outer_box),
        [BoxKind::AnonymousBlock, BoxKind::Block]
    );
    let Some(anon) = bt.children(outer_box).first().copied() else {
        panic!("there is an anonymous box");
    };
    assert_eq!(bt.children(anon).len(), 3, "#a, #x and #y in one run");
}
