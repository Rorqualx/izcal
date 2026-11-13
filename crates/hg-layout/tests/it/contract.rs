//! The constraint space, the diff, margins and the float area.

use hg_layout::{
    AdjoiningMargins, AvailableSize, Axis, BoxFlags, ConstraintSpace, Exclusion, FloatArea,
    FloatSide, Fragmentainer, FragmentainerKind, IntrinsicInput, ModeState, relayout_needed,
};
use hg_types::geometry::{Direction, Flow, LayoutUnit, LogicalSize, WritingMode};

fn u(px: i32) -> LayoutUnit {
    LayoutUnit::from_px(px)
}

fn space() -> ConstraintSpace {
    ConstraintSpace::default()
}

#[test]
fn l01_available_size_has_four_cases_and_zero_is_not_indefinite() {
    // A definite zero and an indefinite size are different inputs; a
    // min-content constraint is not a length at all.
    assert_eq!(
        AvailableSize::Definite(LayoutUnit::ZERO).definite(),
        Some(LayoutUnit::ZERO)
    );
    assert_eq!(AvailableSize::Indefinite.definite(), None);
    assert_ne!(
        AvailableSize::Definite(LayoutUnit::ZERO),
        AvailableSize::Indefinite
    );
    assert!(AvailableSize::MinContent.is_intrinsic());
    assert!(AvailableSize::MaxContent.is_intrinsic());
    assert!(!AvailableSize::Definite(u(10)).is_intrinsic());
    assert!(!AvailableSize::Indefinite.is_intrinsic());
}

#[test]
fn l02_the_diff_ignores_a_change_the_box_does_not_read() {
    // The whole point of the flags: a box whose inline size does not
    // depend on the available inline size reuses its result across a
    // change to it.
    let mut old = space();
    old.available_inline = AvailableSize::Definite(u(100));
    let mut new = old.clone();
    new.available_inline = AvailableSize::Definite(u(200));

    let independent = BoxFlags::default();
    assert!(!relayout_needed(&old, &new, independent));

    let dependent = BoxFlags {
        inline_size_depends_on_available: true,
        ..BoxFlags::default()
    };
    assert!(relayout_needed(&old, &new, dependent));
}

#[test]
fn l03_an_intrinsic_constraint_always_relays_out() {
    // Laying out under a min-content constraint is a different
    // question, not a different width, so no flag excuses it.
    let mut old = space();
    old.available_inline = AvailableSize::Definite(u(100));
    let mut new = old.clone();
    new.available_inline = AvailableSize::MinContent;
    assert!(relayout_needed(&old, &new, BoxFlags::default()));
    assert!(relayout_needed(&new, &old, BoxFlags::default()));
}

#[test]
fn l03a_an_intrinsic_constraint_relays_out_on_the_block_axis_too() {
    // The inline axis had this rule and the block axis did not, so a
    // switch between Definite and MaxContent on the block axis served
    // a stale result — exactly what LY-3 forbids.
    let mut old = space();
    old.available_block = AvailableSize::Definite(u(100));
    let mut new = old.clone();
    new.available_block = AvailableSize::MaxContent;
    assert!(relayout_needed(&old, &new, BoxFlags::default()));
    assert!(relayout_needed(&new, &old, BoxFlags::default()));

    new.available_block = AvailableSize::MinContent;
    assert!(relayout_needed(&old, &new, BoxFlags::default()));
    // A definite change on an axis the box does not read is still
    // reused: the exception is about *intrinsic*, not about the axis.
    let mut taller = old.clone();
    taller.available_block = AvailableSize::Definite(u(200));
    assert!(!relayout_needed(&old, &taller, BoxFlags::default()));
}

#[test]
fn l04_a_percentage_box_relays_out_only_when_its_basis_changed() {
    let mut old = space();
    old.percentage_basis = LogicalSize::new(u(100), u(50));
    let mut new = old.clone();
    new.percentage_basis = LogicalSize::new(u(200), u(50));

    let uses_inline = BoxFlags {
        uses_inline_percentage: true,
        ..BoxFlags::default()
    };
    let uses_block = BoxFlags {
        uses_block_percentage: true,
        ..BoxFlags::default()
    };
    assert!(relayout_needed(&old, &new, uses_inline));
    assert!(
        !relayout_needed(&old, &new, uses_block),
        "only the inline basis moved"
    );
    assert!(!relayout_needed(&old, &new, BoxFlags::default()));
}

#[test]
fn l05_the_flow_changes_what_every_other_field_means() {
    let old = space();
    let mut new = old.clone();
    new.flow = Flow::new(WritingMode::VerticalRl, Direction::Ltr);
    assert!(relayout_needed(&old, &new, BoxFlags::default()));

    // A direction change is a flow change too, even in the same mode.
    let mut rtl = old.clone();
    rtl.flow = Flow::new(WritingMode::HorizontalTb, Direction::Rtl);
    assert!(relayout_needed(&old, &rtl, BoxFlags::default()));
}

#[test]
fn l06_a_parent_that_fixes_a_size_overrides_the_boxs_own_sizing() {
    let old = space();
    let mut new = old.clone();
    new.fixed.inline = true;
    // No flag excuses it: the box's own dependence does not apply when
    // the parent has taken the decision.
    assert!(relayout_needed(&old, &new, BoxFlags::default()));
}

#[test]
fn l07_a_style_version_is_a_different_style_and_so_a_different_layout() {
    // Container re-entry produces size-specific style versions
    // (ST 6.9, LY 6.9); a cached result from another version is for
    // another page.
    let old = space();
    let mut new = old.clone();
    new.style_version = Some(1);
    assert!(relayout_needed(&old, &new, BoxFlags::default()));
}

#[test]
fn l08_floats_and_the_bfc_offset_matter_only_where_they_are_read() {
    let mut old = space();
    old.floats = FloatArea::new();
    let mut new = old.clone();
    new.floats = old.floats.with(Exclusion {
        side: FloatSide::InlineStart,
        block_start: LayoutUnit::ZERO,
        block_end: u(20),
        inline_extent: u(30),
    });
    assert!(!relayout_needed(&old, &new, BoxFlags::default()));
    assert!(relayout_needed(
        &old,
        &new,
        BoxFlags {
            depends_on_floats: true,
            ..BoxFlags::default()
        }
    ));

    let mut moved = old.clone();
    moved.bfc_offset = Some(u(5));
    assert!(!relayout_needed(&old, &moved, BoxFlags::default()));
    assert!(relayout_needed(
        &old,
        &moved,
        BoxFlags {
            is_bfc_root: true,
            ..BoxFlags::default()
        }
    ));
}

#[test]
fn l09_the_fragmentainer_matters_only_to_a_box_that_can_fragment() {
    let old = space();
    let mut new = old.clone();
    new.fragmentainer = Some(Fragmentainer {
        kind: FragmentainerKind::Page,
        block_size: u(800),
        block_offset: LayoutUnit::ZERO,
        is_first: true,
        break_appeal: 0,
    });
    assert!(!relayout_needed(&old, &new, BoxFlags::default()));
    assert!(relayout_needed(
        &old,
        &new,
        BoxFlags {
            can_fragment: true,
            ..BoxFlags::default()
        }
    ));
    // And a fragmentable box reads the available block size whether or
    // not its own block size depends on it.
    let mut taller = old.clone();
    taller.available_block = AvailableSize::Definite(u(400));
    assert!(relayout_needed(
        &old,
        &taller,
        BoxFlags {
            can_fragment: true,
            ..BoxFlags::default()
        }
    ));
}

#[test]
fn l10_an_identical_space_never_relays_out_whatever_the_flags() {
    let s = space();
    let all = BoxFlags {
        inline_size_depends_on_available: true,
        block_size_depends_on_available: true,
        uses_inline_percentage: true,
        uses_block_percentage: true,
        is_bfc_root: true,
        depends_on_floats: true,
        can_fragment: true,
        has_baseline: true,
    };
    assert!(!relayout_needed(&s, &s, all));
    assert!(!relayout_needed(&s, &s, BoxFlags::default()));
}

#[test]
fn l11_margin_collapsing_is_max_positive_plus_min_negative() {
    // CSS 2.1 8.3.1. Two numbers are the whole state, which is why the
    // summary is compact enough to be part of the constraint space.
    let m = AdjoiningMargins::EMPTY.join(u(10)).join(u(20)).join(u(5));
    assert_eq!(m.collapsed(), u(20));

    let mixed = AdjoiningMargins::EMPTY.join(u(20)).join(u(-30)).join(u(-5));
    assert_eq!(mixed.positive, u(20));
    assert_eq!(mixed.negative, u(-30));
    assert_eq!(mixed.collapsed(), u(-10));

    // Only negatives: the result is the most negative, not zero.
    let negative = AdjoiningMargins::EMPTY.join(u(-10)).join(u(-3));
    assert_eq!(negative.collapsed(), u(-10));
    assert!(AdjoiningMargins::EMPTY.collapsed().is_zero());
}

#[test]
fn l12_a_closed_margin_run_is_a_different_input_from_an_open_one() {
    // Something non-collapsible came between, so a following margin
    // does not join — and a cached result from the open case is wrong.
    let mut old = space();
    old.margins = AdjoiningMargins::EMPTY.join(u(10));
    let mut new = old.clone();
    new.margins = old.margins.close();
    assert_ne!(old.margins, new.margins);
    assert!(relayout_needed(&old, &new, BoxFlags::default()));
    assert!(old.margins.open);
    assert!(!new.margins.open);
}

#[test]
fn l13_a_line_is_narrowed_by_the_floats_it_overlaps_and_no_others() {
    let area = FloatArea::new()
        .with(Exclusion {
            side: FloatSide::InlineStart,
            block_start: LayoutUnit::ZERO,
            block_end: u(20),
            inline_extent: u(30),
        })
        .with(Exclusion {
            side: FloatSide::InlineEnd,
            block_start: u(10),
            block_end: u(40),
            inline_extent: u(25),
        });
    // A line at 0..10 meets only the start float.
    assert_eq!(
        area.line_box(LayoutUnit::ZERO, u(10), u(100)),
        (u(30), u(70))
    );
    // A line at 10..20 meets both.
    assert_eq!(area.line_box(u(10), u(10), u(100)), (u(30), u(45)));
    // A line at 20..30 meets only the end float: the first float's
    // block-end edge is exclusive, so a line starting there is clear.
    assert_eq!(
        area.line_box(u(20), u(10), u(100)),
        (LayoutUnit::ZERO, u(75))
    );
    // A line below both is full width.
    assert_eq!(
        area.line_box(u(50), u(10), u(100)),
        (LayoutUnit::ZERO, u(100))
    );
}

#[test]
fn l14_a_line_narrowed_past_nothing_is_zero_wide_and_never_negative() {
    let area = FloatArea::new()
        .with(Exclusion {
            side: FloatSide::InlineStart,
            block_start: LayoutUnit::ZERO,
            block_end: u(100),
            inline_extent: u(80),
        })
        .with(Exclusion {
            side: FloatSide::InlineEnd,
            block_start: LayoutUnit::ZERO,
            block_end: u(100),
            inline_extent: u(80),
        });
    let (start, width) = area.line_box(LayoutUnit::ZERO, u(10), u(100));
    assert_eq!(start, u(80));
    assert!(width.is_zero(), "a negative width would reach paint");
}

#[test]
fn l15_the_float_area_is_persistent_so_a_prefix_is_shared() {
    // An unchanged prefix is shared, so a later layout with the same
    // incoming exclusions hits the cache.
    let base = FloatArea::new().with(Exclusion {
        side: FloatSide::InlineStart,
        block_start: LayoutUnit::ZERO,
        block_end: u(10),
        inline_extent: u(10),
    });
    let extended = base.with(Exclusion {
        side: FloatSide::InlineEnd,
        block_start: LayoutUnit::ZERO,
        block_end: u(10),
        inline_extent: u(10),
    });
    assert_eq!(base.exclusions().len(), 1, "the base is unchanged");
    assert_eq!(extended.exclusions().len(), 2);
    assert_eq!(extended.exclusions().first(), base.exclusions().first());
}

#[test]
fn l16_a_break_token_is_data_and_so_can_be_compared_and_nested() {
    // "A token is data, never a pointer into mutable state, so it is
    // part of fragment-tree hashes (DET-1)."
    let inner = hg_layout::BreakToken {
        box_id: hg_fragment::BoxId(2),
        consumed_block_size: u(50),
        children: Vec::new(),
        mode: ModeState::Flow { item: 3, lines: 2 },
    };
    let outer = hg_layout::BreakToken {
        box_id: hg_fragment::BoxId(1),
        consumed_block_size: u(100),
        children: vec![inner.clone()],
        mode: ModeState::Multicol {
            column_row: 1,
            spanner: None,
        },
    };
    assert_eq!(outer.clone(), outer);
    assert_ne!(outer.children.first(), None);
    let mut different = outer.clone();
    different.consumed_block_size = u(101);
    assert_ne!(different, outer);
}

#[test]
fn l17_an_intrinsic_query_names_its_axis() {
    let inline = IntrinsicInput::default();
    assert_eq!(inline.axis, Axis::Inline);
    let block = IntrinsicInput {
        axis: Axis::Block,
        ..IntrinsicInput::default()
    };
    assert_ne!(inline, block);
}
