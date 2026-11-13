//! §8.3.1's collapsing margins and §10.6.3's automatic height.

use hg_layout_reference::block::{Collapsed, Dir, RefBox, RefFragment, layout};
use hg_layout_reference::style::{BoxStyle, Display, MinSize, Overflow, Size};
use hg_types::geometry::LayoutUnit;

fn u(px: i32) -> LayoutUnit {
    LayoutUnit::from_px(px)
}

fn block(name: &str) -> RefBox {
    RefBox {
        name: name.to_owned(),
        style: BoxStyle::block(),
        children: Vec::new(),
        line_count: 0,
        line_height: LayoutUnit::ZERO,
    }
}

fn with_margins(name: &str, top: i32, bottom: i32) -> RefBox {
    let mut b = block(name);
    b.style.margin_top = Size::px(top);
    b.style.margin_bottom = Size::px(bottom);
    b
}

fn lines(name: &str, count: u32, height: i32) -> RefBox {
    let mut b = block(name);
    b.line_count = count;
    b.line_height = u(height);
    b
}

#[expect(clippy::panic, reason = "a missing fixture box is a test failure")]
fn child<'f>(f: &'f RefFragment, name: &str) -> &'f RefFragment {
    match f.children.iter().find(|c| c.name == name) {
        Some(c) => c,
        None => panic!("{name} is not a child of {}", f.name),
    }
}

#[test]
fn m01_the_collapsed_margin_is_the_largest_positive_less_the_largest_negative() {
    // §8.3.1: "the resulting margin width is the maximum of the
    // collapsing margins' widths"; with negatives, "the maximum of
    // the absolute values of the negative adjoining margins is
    // deducted from the maximum of the positive adjoining margins".
    assert_eq!(
        Collapsed::EMPTY.join(u(10)).join(u(20)).join(u(5)).value(),
        u(20)
    );
    assert_eq!(
        Collapsed::EMPTY.join(u(20)).join(u(-30)).value(),
        u(-10),
        "20 less 30"
    );
    // "If there are no positive margins, the maximum of the absolute
    // values of the adjoining margins is deducted from zero."
    assert_eq!(Collapsed::EMPTY.join(u(-10)).join(u(-3)).value(), u(-10));
    assert!(Collapsed::EMPTY.is_empty());
    assert_eq!(
        Collapsed::EMPTY
            .join(u(5))
            .merge(Collapsed::EMPTY.join(u(9)))
            .value(),
        u(9)
    );
}

#[test]
fn m02_adjacent_siblings_margins_collapse_to_the_larger() {
    // "bottom margin of box and top margin of its next in-flow
    // following sibling". Both boxes are given a height, so neither
    // collapses through and the only rule in play is this one.
    let mut root = block("root");
    let mut a = with_margins("a", 0, 20);
    a.style.height = Size::px(10);
    let mut b = with_margins("b", 10, 0);
    b.style.height = Size::px(10);
    root.children = vec![a, b];
    let f = layout(&root, u(100), Dir::Ltr);
    // `b` sits 20px below `a`'s bottom edge, not 30.
    assert_eq!(child(&f, "a").rect.top(), LayoutUnit::ZERO);
    assert_eq!(child(&f, "b").rect.top(), u(10).add(u(20)));
    assert_eq!(f.rect.size.height, u(40));
}

#[test]
fn m03_a_parents_top_margin_collapses_with_its_first_childs() {
    // "top margin of a box and top margin of its first in-flow child".
    let mut root = block("root");
    let mut parent = with_margins("parent", 10, 0);
    parent.children = vec![{
        let mut c = with_margins("child", 30, 0);
        c.style.height = Size::px(50);
        c
    }];
    root.children = vec![parent];
    let f = layout(&root, u(100), Dir::Ltr);
    let parent = child(&f, "parent");
    assert_eq!(parent.margin_top, u(30), "max(10, 30), not 40");
    // The child starts at the parent's top edge: the margin went
    // outside.
    assert_eq!(child(parent, "child").rect.top(), LayoutUnit::ZERO);
}

#[test]
fn m04_a_border_or_padding_separates_the_margins() {
    // "no line boxes, no clearance, no padding and no border separate
    // them".
    for (what, apply) in [("border", 0_u8), ("padding", 1)] {
        let mut root = block("root");
        let mut parent = with_margins("parent", 10, 0);
        if apply == 0 {
            parent.style.border_top = u(1);
        } else {
            parent.style.padding_top = Size::px(1);
        }
        parent.children = vec![with_margins("child", 30, 0)];
        root.children = vec![parent];
        let f = layout(&root, u(100), Dir::Ltr);
        let p = child(&f, "parent");
        assert_eq!(p.margin_top, u(10), "{what} keeps the parent's own margin");
        assert_eq!(
            child(p, "child").rect.top(),
            u(30).add(u(1)),
            "{what}: the child's margin is inside"
        );
    }
}

#[test]
fn m05_a_new_block_formatting_context_does_not_collapse_with_its_children() {
    // "Margins of elements that establish new block formatting
    // contexts do not collapse with their in-flow children."
    let mut root = block("root");
    let mut parent = with_margins("parent", 10, 0);
    parent.style.overflow = Overflow::Clipped;
    parent.children = vec![{
        let mut c = with_margins("child", 30, 0);
        c.style.height = Size::px(50);
        c
    }];
    root.children = vec![parent];
    let f = layout(&root, u(100), Dir::Ltr);
    let p = child(&f, "parent");
    assert_eq!(p.margin_top, u(10));
    assert_eq!(child(p, "child").rect.top(), u(30), "inside, not outside");
    assert_eq!(p.rect.size.height, u(80), "30 of margin plus 50 of child");
}

#[test]
fn m06_the_root_elements_margins_do_not_collapse() {
    // "Margins of the root element's box do not collapse."
    let mut root = with_margins("root", 10, 10);
    root.children = vec![with_margins("child", 30, 30)];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(f.margin_top, u(10), "the root keeps its own");
}

#[test]
fn m07_a_last_childs_bottom_margin_collapses_out_of_an_auto_height_parent() {
    // "bottom margin of a last in-flow child and bottom margin of its
    // parent if the parent has 'auto' computed height".
    let mut root = block("root");
    let mut parent = block("parent");
    parent.children = vec![{
        let mut c = block("child");
        c.style.height = Size::px(50);
        c.style.margin_bottom = Size::px(20);
        c
    }];
    root.children = vec![parent];
    let f = layout(&root, u(100), Dir::Ltr);
    let p = child(&f, "parent");
    assert_eq!(
        p.rect.size.height,
        u(50),
        "the margin is outside the parent"
    );
    assert_eq!(p.margin_bottom, u(20));

    // With a given height it does not collapse out: §10.6.3's rule
    // only applies to an automatic height.
    let mut root = block("root");
    let mut parent = block("parent");
    parent.style.height = Size::px(80);
    parent.children = vec![{
        let mut c = block("child");
        c.style.height = Size::px(50);
        c.style.margin_bottom = Size::px(20);
        c
    }];
    root.children = vec![parent];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(child(&f, "parent").rect.size.height, u(80));
    assert_eq!(child(&f, "parent").margin_bottom, LayoutUnit::ZERO);
}

#[test]
fn m08_a_box_with_nothing_in_it_collapses_through() {
    // §8.3.1's fourth pair: "top and bottom margins of a box that does
    // not establish a new block formatting context, and that has zero
    // computed 'min-height', zero or 'auto' computed 'height', and no
    // in-flow children".
    let mut root = block("root");
    root.children = vec![
        {
            let mut a = block("a");
            a.style.height = Size::px(10);
            a.style.margin_bottom = Size::px(20);
            a
        },
        with_margins("empty", 30, 5),
        {
            let mut b = block("b");
            b.style.height = Size::px(10);
            b.style.margin_top = Size::px(10);
            b
        },
    ];
    let f = layout(&root, u(100), Dir::Ltr);
    // Two things the prose settles: the empty box contributes no
    // height, and all four margins are one adjoining run, so the gap
    // between `a` and `b` is max(20, 30, 5, 10) = 30 and not their sum.
    assert_eq!(child(&f, "a").rect.top(), LayoutUnit::ZERO);
    assert_eq!(child(&f, "empty").rect.size.height, LayoutUnit::ZERO);
    assert_eq!(child(&f, "b").rect.top(), u(10).add(u(30)));
    assert_eq!(f.rect.size.height, u(10).add(u(30)).add(u(10)));
    // Where the *collapsed-through box itself* sits inside that run is
    // not settled by the prose quoted above, and browsers differ on
    // the hypothetical position. It is on the ST-M0/LY-M0 agenda; what
    // is asserted here is only that it adds nothing.
}

#[test]
fn m09_a_min_height_stops_a_box_collapsing_through() {
    let mut root = block("root");
    let mut empty = with_margins("empty", 30, 5);
    empty.style.min_height = MinSize::Length(u(1));
    root.children = vec![empty, {
        let mut b = block("b");
        b.style.height = Size::px(10);
        b.style.margin_top = Size::px(10);
        b
    }];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(child(&f, "empty").rect.size.height, u(1));
    // The root seals, so `empty`'s 30px top margin sits inside it.
    assert_eq!(child(&f, "empty").rect.top(), u(30));
    // Its bottom margin no longer joins its top one, so the gap below
    // it is max(5, 10) = 10 and not max(30, 5, 10) = 30.
    assert_eq!(child(&f, "b").rect.top(), u(30).add(u(1)).add(u(10)));
}

#[test]
fn m10_an_automatic_height_takes_the_last_line_box_when_there_is_one() {
    // §10.6.3 clause 1: "the bottom edge of the last line box, if the
    // box establishes an inline formatting context with one or more
    // lines".
    let mut root = block("root");
    root.children = vec![lines("text", 3, 60)];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(child(&f, "text").rect.size.height, u(60));
    // And a line box separates the margins, so the box keeps its own.
    let mut with_line = lines("text", 1, 20);
    with_line.style.margin_top = Size::px(5);
    with_line.children = vec![with_margins("inner", 50, 0)];
    let mut root = block("root");
    root.children = vec![with_line];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(child(&f, "text").margin_top, u(5));
}

#[test]
fn m11_an_automatic_height_is_zero_when_there_is_nothing_in_it() {
    // §10.6.3 clause 4: "zero, otherwise".
    let root = block("root");
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(f.rect.size.height, LayoutUnit::ZERO);
}

#[test]
fn m12_display_none_takes_no_part() {
    let mut root = block("root");
    let mut gone = block("gone");
    gone.style.display = Display::None;
    gone.style.height = Size::px(100);
    root.children = vec![gone, {
        let mut b = block("b");
        b.style.height = Size::px(10);
        b
    }];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(f.children.len(), 1);
    assert_eq!(f.rect.size.height, u(10));
}

#[test]
fn m13_children_are_placed_inside_their_parents_border_and_padding() {
    let mut root = block("root");
    let mut parent = block("parent");
    parent.style.border_left = u(5);
    parent.style.padding_left = Size::px(10);
    parent.style.border_top = u(2);
    parent.children = vec![{
        let mut c = block("child");
        c.style.height = Size::px(10);
        c
    }];
    root.children = vec![parent];
    let f = layout(&root, u(100), Dir::Ltr);
    let p = child(&f, "parent");
    let c = child(p, "child");
    assert_eq!(c.rect.left(), u(15), "inside the border and padding");
    assert_eq!(c.rect.top(), u(2));
    // The child's width is the parent's content width.
    assert_eq!(c.rect.size.width, u(85));
}

#[test]
fn m14_a_height_is_clamped_by_min_and_max() {
    let mut root = block("root");
    let mut b = block("b");
    b.style.height = Size::px(100);
    b.style.max_height = hg_layout_reference::style::MaxSize::Length(u(40));
    root.children = vec![b];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(child(&f, "b").rect.size.height, u(40));

    let mut root = block("root");
    let mut b = block("b");
    b.style.height = Size::px(10);
    b.style.min_height = MinSize::Length(u(50));
    root.children = vec![b];
    let f = layout(&root, u(100), Dir::Ltr);
    assert_eq!(child(&f, "b").rect.size.height, u(50));
}
