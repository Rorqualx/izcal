//! §10.3.3's constraint equation and §10.4's min and max width.

use hg_layout_reference::block::{Dir, used_width, used_width_constrained};
use hg_layout_reference::style::{BoxStyle, MaxSize, MinSize, Size};
use hg_types::geometry::LayoutUnit;

fn u(px: i32) -> LayoutUnit {
    LayoutUnit::from_px(px)
}

/// The equation must hold: margins + borders + padding + width equals
/// the containing block, in every case §10.3.3 defines.
fn equation_holds(style: &BoxStyle, containing: LayoutUnit, dir: Dir) {
    let w = used_width(style, containing, dir);
    let border = style.border();
    let padding = style.padding(Some(containing));
    let total = w
        .margin_left
        .add(border.horizontal())
        .add(padding.horizontal())
        .add(w.width)
        .add(w.margin_right);
    assert_eq!(
        total, containing,
        "the equation does not balance: {w:?} in {containing}"
    );
}

#[test]
fn w01_width_auto_fills_what_the_margins_leave() {
    let mut s = BoxStyle::block();
    s.margin_left = Size::px(10);
    s.margin_right = Size::px(20);
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.width, u(70));
    assert_eq!(w.margin_left, u(10));
    equation_holds(&s, u(100), Dir::Ltr);
}

#[test]
fn w02_width_auto_makes_every_other_auto_zero() {
    // "If 'width' is set to 'auto', any other 'auto' values become
    // '0' and 'width' follows from the resulting equality."
    let mut s = BoxStyle::block();
    s.margin_left = Size::Auto;
    s.margin_right = Size::Auto;
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.margin_left, LayoutUnit::ZERO);
    assert_eq!(w.margin_right, LayoutUnit::ZERO);
    assert_eq!(w.width, u(100));
}

#[test]
fn w03_exactly_one_auto_follows_from_the_equality() {
    // The *other* margin has to be `auto` for this rule to apply;
    // with both specified the box is over-constrained instead, which
    // is w06.
    let mut s = BoxStyle::block();
    s.width = Size::px(50);
    s.margin_left = Size::px(10);
    s.margin_right = Size::Auto;
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.margin_right, u(40));
    equation_holds(&s, u(100), Dir::Ltr);

    let mut s = BoxStyle::block();
    s.width = Size::px(50);
    s.margin_left = Size::Auto;
    s.margin_right = Size::px(10);
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.margin_left, u(40));
    equation_holds(&s, u(100), Dir::Ltr);
}

#[test]
fn w04_both_margins_auto_centres_the_box() {
    let mut s = BoxStyle::block();
    s.margin_left = Size::Auto;
    s.margin_right = Size::Auto;
    s.width = Size::px(60);
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.margin_left, u(20));
    assert_eq!(w.margin_right, u(20));
    equation_holds(&s, u(100), Dir::Ltr);
}

#[test]
fn w05_centring_an_odd_remainder_still_balances() {
    // Half of an odd number of units cannot go both ways; the
    // equation must still hold, so the remainder has a home.
    let mut s = BoxStyle::block();
    s.margin_left = Size::Auto;
    s.margin_right = Size::Auto;
    s.width = Size::Length(LayoutUnit::from_raw(101));
    let containing = LayoutUnit::from_raw(200);
    let w = used_width(&s, containing, Dir::Ltr);
    assert_eq!(w.margin_left.raw(), 49);
    assert_eq!(w.margin_right.raw(), 50);
    equation_holds(&s, containing, Dir::Ltr);
}

#[test]
fn w06_over_constrained_ignores_one_margin_and_direction_says_which() {
    // "the specified value of margin-right is ignored and the value is
    // calculated so as to make the equality true" — margin-left in rtl.
    let mut s = BoxStyle::block();
    s.width = Size::px(50);
    s.margin_left = Size::px(10);
    s.margin_right = Size::px(10);
    let ltr = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(ltr.margin_left, u(10), "the left margin is kept");
    assert_eq!(ltr.margin_right, u(40), "the right margin is recomputed");
    equation_holds(&s, u(100), Dir::Ltr);

    let rtl = used_width(&s, u(100), Dir::Rtl);
    assert_eq!(rtl.margin_right, u(10), "the right margin is kept");
    assert_eq!(rtl.margin_left, u(40));
    equation_holds(&s, u(100), Dir::Rtl);
}

#[test]
fn w07_an_over_wide_box_treats_its_auto_margins_as_zero() {
    // The rule before the others: when the non-auto parts already
    // exceed the containing block, an `auto` margin is zero rather
    // than negative.
    let mut s = BoxStyle::block();
    s.margin_left = Size::Auto;
    s.margin_right = Size::Auto;
    s.width = Size::px(200);
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.margin_left, LayoutUnit::ZERO);
    assert_eq!(w.margin_right, u(-100), "the equation still balances");
    equation_holds(&s, u(100), Dir::Ltr);
}

#[test]
fn w08_borders_and_padding_come_out_of_the_width() {
    let mut s = BoxStyle::block();
    s.border_left = u(5);
    s.border_right = u(5);
    s.padding_left = Size::px(10);
    s.padding_right = Size::px(10);
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.width, u(70));
    equation_holds(&s, u(100), Dir::Ltr);
}

#[test]
fn w09_a_percentage_resolves_against_the_containing_block() {
    let mut s = BoxStyle::block();
    s.width = Size::percent(50);
    let w = used_width(&s, u(100), Dir::Ltr);
    assert_eq!(w.width, u(50));
    // Padding percentages resolve against the inline size too, on
    // every side.
    let mut s = BoxStyle::block();
    s.padding_left = Size::percent(10);
    s.padding_top = Size::percent(10);
    let padding = s.padding(Some(u(200)));
    assert_eq!(padding.left, u(20));
    assert_eq!(padding.top, u(20), "a top padding percentage is horizontal");
}

#[test]
fn w10_an_awkward_percentage_rounds_once_and_exactly() {
    let mut s = BoxStyle::block();
    s.width = Size::Percent(333_333); // 33.3333%
    let w = used_width(&s, u(100), Dir::Ltr);
    // 100px is 6400 units; 33.3333% of that is 2133.33, to even 2133.
    assert_eq!(w.width.raw(), 2133);
}

#[test]
fn w11_max_width_reruns_the_rules_and_so_keeps_a_box_centred() {
    // §10.4's three steps, not a clamp. A clamp would leave the
    // margins where the tentative width put them and the box would
    // sit off-centre.
    let mut s = BoxStyle::block();
    s.max_width = MaxSize::Length(u(40));
    let w = used_width_constrained(&s, u(100), Dir::Ltr);
    assert_eq!(w.width, u(40));
    // The initial margins are zero, so the box stays at the left and
    // the equation balances through the right margin.
    assert_eq!(w.margin_left, LayoutUnit::ZERO);
    assert_eq!(w.margin_right, u(60));

    let mut s = BoxStyle::block();
    s.margin_left = Size::Auto;
    s.margin_right = Size::Auto;
    s.width = Size::px(80);
    s.max_width = MaxSize::Length(u(40));
    let w = used_width_constrained(&s, u(100), Dir::Ltr);
    assert_eq!(w.width, u(40));
    assert_eq!(w.margin_left, u(30), "still centred, at the new width");
    assert_eq!(w.margin_right, u(30));
}

#[test]
fn w12_min_width_is_applied_after_max_width() {
    // The order matters: max first, then min, so a min above a max
    // wins — which is what the specification's step order says.
    let mut s = BoxStyle::block();
    s.width = Size::px(10);
    s.min_width = MinSize::Length(u(60));
    s.max_width = MaxSize::Length(u(40));
    let w = used_width_constrained(&s, u(100), Dir::Ltr);
    assert_eq!(w.width, u(60), "min beats max");

    let mut s = BoxStyle::block();
    s.width = Size::px(10);
    s.min_width = MinSize::Length(u(30));
    let w = used_width_constrained(&s, u(100), Dir::Ltr);
    assert_eq!(w.width, u(30));
}

#[test]
fn w13_a_percentage_against_an_indefinite_basis_is_auto_not_zero() {
    // The rule a naive resolution gets wrong.
    assert_eq!(Size::percent(50).resolve(None), None);
    assert_eq!(Size::percent(50).resolve(Some(u(100))), Some(u(50)));
    // `min-*` resolves to zero instead, and `max-*` to none.
    assert_eq!(MinSize::Percent(500_000).resolve(None), LayoutUnit::ZERO);
    assert_eq!(MaxSize::Percent(500_000).resolve(None), None);
}

#[test]
fn w14_the_equation_balances_for_every_combination() {
    // The invariant §10.3.3 exists to keep, over the cross product of
    // the cases.
    let widths = [Size::Auto, Size::px(50), Size::percent(30)];
    let margins = [Size::Auto, Size::px(10), Size::px(-5)];
    for w in widths {
        for ml in margins {
            for mr in margins {
                for dir in [Dir::Ltr, Dir::Rtl] {
                    let mut s = BoxStyle::block();
                    s.width = w;
                    s.margin_left = ml;
                    s.margin_right = mr;
                    s.border_left = u(2);
                    s.padding_right = Size::px(3);
                    equation_holds(&s, u(100), dir);
                }
            }
        }
    }
}
