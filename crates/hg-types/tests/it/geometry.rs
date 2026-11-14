//! Geometry: the layout unit and the abstract-to-physical mappings.

use hg_types::geometry::{
    Direction, Flow, LayoutUnit, LogicalPoint, LogicalRect, LogicalSide, LogicalSides, LogicalSize,
    PhysicalPoint, PhysicalRect,
    PhysicalSide::{self, Bottom, Left, Right, Top},
    PhysicalSides, PhysicalSize, UNITS_PER_PX, WritingMode,
};

use WritingMode::{HorizontalTb, SidewaysLr, SidewaysRl, VerticalLr, VerticalRl};

fn u(px: i32) -> LayoutUnit {
    LayoutUnit::from_px(px)
}

#[test]
fn g01_the_unit_is_one_sixty_fourth_of_a_css_pixel() {
    // LY-D3: a signed 32-bit integer counting 1/64 CSS px, so the
    // range is ±33,554,431.98 px.
    assert_eq!(UNITS_PER_PX, 64);
    assert_eq!(LayoutUnit::PX.raw(), 64);
    assert_eq!(LayoutUnit::from_px(1), LayoutUnit::from_raw(64));
    assert_eq!(LayoutUnit::MAX.to_px_trunc(), 33_554_431);
    assert_eq!(LayoutUnit::MIN.to_px_trunc(), -33_554_432);
}

#[test]
fn g02_every_operation_saturates_and_none_panics() {
    // LY-13: an overflowing page lays out identically on every
    // platform, which needs overflow to be a specified behaviour.
    assert_eq!(LayoutUnit::MAX.add(LayoutUnit::PX), LayoutUnit::MAX);
    assert_eq!(LayoutUnit::MIN.sub(LayoutUnit::PX), LayoutUnit::MIN);
    assert_eq!(LayoutUnit::MAX.scale(2), LayoutUnit::MAX);
    assert_eq!(LayoutUnit::MIN.scale(2), LayoutUnit::MIN);
    // `-MIN` has no two's-complement answer, so it saturates to MAX
    // rather than wrapping back to itself.
    assert_eq!(LayoutUnit::MIN.neg(), LayoutUnit::MAX);
    assert_eq!(LayoutUnit::MIN.abs(), LayoutUnit::MAX);
    assert_eq!(u(-3).abs(), u(3));
    assert_eq!(LayoutUnit::from_px(i32::MAX), LayoutUnit::MAX);
}

#[test]
fn g03_clamp_lets_a_minimum_beat_a_maximum() {
    // CSS: a `min-width` above a `max-width` wins.
    assert_eq!(u(5).clamp(u(1), u(10)), u(5));
    assert_eq!(u(0).clamp(u(1), u(10)), u(1));
    assert_eq!(u(50).clamp(u(1), u(10)), u(10));
    assert_eq!(u(50).clamp(u(10), u(1)), u(10));
}

#[test]
fn g04_a_length_prints_its_exact_value_because_one_sixty_fourth_is_a_power_of_two() {
    assert_eq!(u(3).to_string(), "3px");
    assert_eq!(LayoutUnit::from_raw(32).to_string(), "0.5px");
    assert_eq!(LayoutUnit::from_raw(1).to_string(), "0.015625px");
    assert_eq!(LayoutUnit::from_raw(-1).to_string(), "-0.015625px");
    assert_eq!(LayoutUnit::from_raw(-96).to_string(), "-1.5px");
    assert_eq!(LayoutUnit::ZERO.to_string(), "0px");
}

/// Every column of CSS Writing Modes 4 §6.4's Abstract-Physical
/// Mapping table, transcribed from the specification.
const MAPPINGS: [(WritingMode, Direction, PhysicalSide, PhysicalSide); 10] = [
    // (mode, direction, block-start, inline-start)
    (HorizontalTb, Direction::Ltr, Top, Left),
    (HorizontalTb, Direction::Rtl, Top, Right),
    (VerticalRl, Direction::Ltr, Right, Top),
    (VerticalRl, Direction::Rtl, Right, Bottom),
    (SidewaysRl, Direction::Ltr, Right, Top),
    (SidewaysRl, Direction::Rtl, Right, Bottom),
    (VerticalLr, Direction::Ltr, Left, Top),
    (VerticalLr, Direction::Rtl, Left, Bottom),
    (SidewaysLr, Direction::Ltr, Left, Bottom),
    (SidewaysLr, Direction::Rtl, Left, Top),
];

#[test]
fn g05_every_row_of_the_abstract_physical_mapping_table() {
    for (mode, direction, block_start, inline_start) in MAPPINGS {
        let f = Flow::new(mode, direction);
        assert_eq!(f.block_start(), block_start, "{mode:?} {direction:?}");
        assert_eq!(f.inline_start(), inline_start, "{mode:?} {direction:?}");
        // The table's block-end and inline-end rows are the opposites
        // of the two above, in every column.
        assert_eq!(f.block_end(), block_start.opposite());
        assert_eq!(f.inline_end(), inline_start.opposite());
        // The two axes are never the same one.
        assert_ne!(
            f.block_start().is_horizontal(),
            f.inline_start().is_horizontal(),
            "{mode:?} {direction:?}"
        );
    }
}

#[test]
fn g06_sideways_lr_is_the_row_that_catches_a_guess() {
    // Every other vertical mode puts inline-start at the top for ltr;
    // `sideways-lr` puts it at the bottom. A mapping written from
    // memory gets this one wrong.
    assert_eq!(Flow::new(SidewaysLr, Direction::Ltr).inline_start(), Bottom);
    assert_eq!(Flow::new(SidewaysLr, Direction::Rtl).inline_start(), Top);
    assert_eq!(Flow::new(VerticalLr, Direction::Ltr).inline_start(), Top);
    // And `sideways-lr` shares block-start with `vertical-lr`, not
    // with `vertical-rl`.
    assert_eq!(Flow::new(SidewaysLr, Direction::Ltr).block_start(), Left);
}

#[test]
fn g07_line_relative_over_is_not_block_start() {
    // §6.4's `over` row: top for horizontal-tb, **right** for
    // vertical-rl, sideways-rl AND vertical-lr, left for sideways-lr.
    // So in vertical-lr, block-start is the left and over is the
    // right — the one place the two differ, and the reason they are
    // separate functions.
    let f = Flow::new(VerticalLr, Direction::Ltr);
    assert_eq!(f.block_start(), Left);
    assert_eq!(f.over(), Right);
    assert_ne!(f.over(), f.block_start());

    for (mode, over, line_left) in [
        (HorizontalTb, Top, Left),
        (VerticalRl, Right, Top),
        (SidewaysRl, Right, Top),
        (VerticalLr, Right, Top),
        (SidewaysLr, Left, Bottom),
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let f = Flow::new(mode, direction);
            assert_eq!(f.over(), over, "{mode:?} over");
            assert_eq!(f.under(), over.opposite(), "{mode:?} under");
            // Line-relative sides do not depend on `direction`.
            assert_eq!(f.line_left(), line_left, "{mode:?} line-left");
            assert_eq!(f.line_right(), line_left.opposite(), "{mode:?}");
        }
    }
}

#[test]
fn g08_upright_text_orientation_forces_the_used_direction_to_ltr() {
    // §6.4's note: "in vertical writing modes, a `text-orientation`
    // value of `upright` forces the used `direction` to `ltr`".
    let f = Flow::used(VerticalRl, Direction::Rtl, true);
    assert_eq!(f.direction, Direction::Ltr);
    assert_eq!(f.inline_start(), Top);
    // In a horizontal writing mode it changes nothing.
    let f = Flow::used(HorizontalTb, Direction::Rtl, true);
    assert_eq!(f.direction, Direction::Rtl);
    assert_eq!(f.inline_start(), Right);
    // And without `upright` the direction is the computed one.
    let f = Flow::used(VerticalRl, Direction::Rtl, false);
    assert_eq!(f.inline_start(), Bottom);
}

#[test]
fn g09_block_size_is_the_height_horizontally_and_the_width_vertically() {
    // §6.4's first two rows.
    let size = PhysicalSize::new(u(100), u(50));
    let horizontal = size.to_logical(Flow::new(HorizontalTb, Direction::Ltr));
    assert_eq!(horizontal, LogicalSize::new(u(100), u(50)));
    for mode in [VerticalRl, VerticalLr, SidewaysRl, SidewaysLr] {
        let v = size.to_logical(Flow::new(mode, Direction::Ltr));
        assert_eq!(v, LogicalSize::new(u(50), u(100)), "{mode:?}");
        assert_eq!(v.to_physical(Flow::new(mode, Direction::Ltr)), size);
    }
}

#[test]
fn g10_four_sides_round_trip_through_every_flow() {
    let logical = LogicalSides::new(u(1), u(2), u(3), u(4));
    for (mode, direction, _, _) in MAPPINGS {
        let flow = Flow::new(mode, direction);
        let physical = logical.to_physical(flow);
        assert_eq!(physical.to_logical(flow), logical, "{mode:?} {direction:?}");
        // The sums are axis sums, so they survive the mapping.
        assert_eq!(logical.block_sum(), u(3));
        assert_eq!(logical.inline_sum(), u(7));
        if flow.is_vertical() {
            assert_eq!(physical.horizontal_sum(), u(3));
            assert_eq!(physical.vertical_sum(), u(7));
        } else {
            assert_eq!(physical.horizontal_sum(), u(7));
            assert_eq!(physical.vertical_sum(), u(3));
        }
    }
}

#[test]
fn g11_a_side_is_read_by_name_in_both_vocabularies() {
    let l = LogicalSides::new(u(1), u(2), u(3), u(4));
    assert_eq!(l.get(LogicalSide::BlockStart), u(1));
    assert_eq!(l.get(LogicalSide::BlockEnd), u(2));
    assert_eq!(l.get(LogicalSide::InlineStart), u(3));
    assert_eq!(l.get(LogicalSide::InlineEnd), u(4));
    let p = PhysicalSides::new(u(1), u(2), u(3), u(4));
    assert_eq!(p.get(Top), u(1));
    assert_eq!(p.get(Right), u(2));
    assert_eq!(p.get(Bottom), u(3));
    assert_eq!(p.get(Left), u(4));
    assert_eq!(p.with(Top, u(9)).get(Top), u(9));
    assert_eq!(p.with(Top, u(9)).get(Bottom), u(3));
}

#[test]
fn g12_a_logical_rectangle_needs_its_container_to_become_physical() {
    let container = PhysicalSize::new(u(100), u(200));
    let rect = LogicalRect::new(
        LogicalPoint::new(u(10), u(20)),
        LogicalSize::new(u(30), u(40)),
    );

    // horizontal-tb ltr: inline is x from the left, block is y from
    // the top, so the container does not enter.
    let p = rect.to_physical(Flow::new(HorizontalTb, Direction::Ltr), container);
    assert_eq!(
        p,
        PhysicalRect::new(
            PhysicalPoint::new(u(10), u(20)),
            PhysicalSize::new(u(30), u(40))
        )
    );

    // horizontal-tb rtl: inline-start is the right, so x is measured
    // back from the container's width.
    let p = rect.to_physical(Flow::new(HorizontalTb, Direction::Rtl), container);
    assert_eq!(p.origin, PhysicalPoint::new(u(100 - 10 - 30), u(20)));

    // vertical-rl ltr: block-start is the right, inline-start the top.
    // The block offset is measured back from the width.
    let p = rect.to_physical(Flow::new(VerticalRl, Direction::Ltr), container);
    assert_eq!(p.size, PhysicalSize::new(u(40), u(30)));
    assert_eq!(p.origin, PhysicalPoint::new(u(100 - 20 - 40), u(10)));

    // sideways-lr ltr: block-start is the left, inline-start the
    // bottom — the one mode where the inline offset is measured back
    // from the height.
    let p = rect.to_physical(Flow::new(SidewaysLr, Direction::Ltr), container);
    assert_eq!(p.origin, PhysicalPoint::new(u(20), u(200 - 10 - 30)));
}

#[test]
fn g13_a_rectangle_claims_its_top_left_edges_and_not_its_bottom_right() {
    // The half-open rule, so two touching boxes never both claim a
    // point in a hit test.
    let r = PhysicalRect::new(
        PhysicalPoint::new(u(10), u(10)),
        PhysicalSize::new(u(10), u(10)),
    );
    assert!(r.contains(PhysicalPoint::new(u(10), u(10))));
    assert!(r.contains(PhysicalPoint::new(u(19), u(19))));
    assert!(!r.contains(PhysicalPoint::new(u(20), u(10))));
    assert!(!r.contains(PhysicalPoint::new(u(10), u(20))));
    assert!(!r.contains(PhysicalPoint::new(u(9), u(10))));
    let next = PhysicalRect::new(
        PhysicalPoint::new(u(20), u(10)),
        PhysicalSize::new(u(10), u(10)),
    );
    assert!(next.contains(PhysicalPoint::new(u(20), u(10))));
}

#[test]
fn g14_an_empty_rectangle_contributes_nothing_to_a_union() {
    let empty = PhysicalRect::default();
    let r = PhysicalRect::new(
        PhysicalPoint::new(u(10), u(10)),
        PhysicalSize::new(u(5), u(5)),
    );
    assert!(empty.is_empty());
    assert!(!r.is_empty());
    // A naive min/max would drag the union back to the origin.
    assert_eq!(empty.union(r), r);
    assert_eq!(r.union(empty), r);
    assert_eq!(empty.union(empty), empty);
    let other = PhysicalRect::new(
        PhysicalPoint::new(u(20), u(20)),
        PhysicalSize::new(u(5), u(5)),
    );
    assert_eq!(
        r.union(other),
        PhysicalRect::new(
            PhysicalPoint::new(u(10), u(10)),
            PhysicalSize::new(u(15), u(15))
        )
    );
}

#[test]
fn g15_an_intersection_that_does_not_meet_is_empty_and_never_negative() {
    let a = PhysicalRect::new(
        PhysicalPoint::new(u(0), u(0)),
        PhysicalSize::new(u(10), u(10)),
    );
    let b = PhysicalRect::new(
        PhysicalPoint::new(u(5), u(5)),
        PhysicalSize::new(u(10), u(10)),
    );
    assert_eq!(
        a.intersection(b),
        PhysicalRect::new(
            PhysicalPoint::new(u(5), u(5)),
            PhysicalSize::new(u(5), u(5))
        )
    );
    let far = PhysicalRect::new(
        PhysicalPoint::new(u(100), u(100)),
        PhysicalSize::new(u(1), u(1)),
    );
    let none = a.intersection(far);
    assert!(none.is_empty());
    assert!(!none.size.width.is_negative());
    assert!(!none.size.height.is_negative());
}

#[test]
fn g16_inset_never_gives_a_negative_size_and_outset_is_its_inverse() {
    let r = PhysicalRect::new(
        PhysicalPoint::new(u(10), u(10)),
        PhysicalSize::new(u(20), u(20)),
    );
    let sides = PhysicalSides::new(u(2), u(3), u(4), u(5));
    assert_eq!(r.inset(sides).outset(sides), r);
    // A border wider than the box leaves a zero size, not a negative
    // one — a negative width would reach paint and the GPU service.
    let thick = PhysicalSides::new(u(100), u(100), u(100), u(100));
    let crushed = r.inset(thick);
    assert_eq!(
        crushed.size,
        PhysicalSize::new(LayoutUnit::ZERO, LayoutUnit::ZERO)
    );
    assert!(crushed.is_empty());
}

#[test]
fn g17_translate_moves_the_origin_and_keeps_the_size() {
    let r = PhysicalRect::new(
        PhysicalPoint::new(u(1), u(2)),
        PhysicalSize::new(u(3), u(4)),
    );
    let moved = r.translate(u(10), u(-20));
    assert_eq!(moved.origin, PhysicalPoint::new(u(11), u(-18)));
    assert_eq!(moved.size, r.size);
    assert_eq!(moved.right(), u(14));
    assert_eq!(moved.bottom(), u(-14));
}
