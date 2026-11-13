//! Appendix E's seven steps, and the tree-order tie-break.

use hg_paint_order::{PaintBox, PaintId, Step, paint_order, step_of};

fn b(id: u32, step: Step, stack_level: i32, tree_order: u32) -> PaintBox {
    PaintBox {
        id: PaintId(id),
        step,
        stack_level,
        tree_order,
    }
}

fn ids(order: &[PaintId]) -> Vec<u32> {
    order.iter().map(|i| i.0).collect()
}

#[test]
fn o01_the_seven_steps_are_in_appendix_es_order() {
    // Written out of order on purpose: the sort is the test.
    let boxes = [
        b(7, Step::PositiveContexts, 1, 0),
        b(5, Step::InlineLevel, 0, 0),
        b(1, Step::RootBackground, 0, 0),
        b(6, Step::Level0, 0, 0),
        b(3, Step::BlockLevel, 0, 0),
        b(2, Step::NegativeContexts, -1, 0),
        b(4, Step::Floats, 0, 0),
    ];
    assert_eq!(ids(&paint_order(&boxes)), [1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn o02_negative_contexts_paint_most_negative_first() {
    let boxes = [
        b(1, Step::NegativeContexts, -1, 0),
        b(2, Step::NegativeContexts, -5, 1),
        b(3, Step::NegativeContexts, -3, 2),
    ];
    assert_eq!(ids(&paint_order(&boxes)), [2, 3, 1]);
}

#[test]
fn o03_positive_contexts_paint_least_positive_first() {
    let boxes = [
        b(1, Step::PositiveContexts, 9, 0),
        b(2, Step::PositiveContexts, 1, 1),
        b(3, Step::PositiveContexts, 5, 2),
    ];
    assert_eq!(ids(&paint_order(&boxes)), [2, 3, 1]);
}

#[test]
fn o04_tree_order_is_the_tie_break_and_makes_the_order_total() {
    // The part a reordering optimization is most likely to lose.
    let boxes = [
        b(1, Step::BlockLevel, 0, 5),
        b(2, Step::BlockLevel, 0, 1),
        b(3, Step::BlockLevel, 0, 3),
    ];
    assert_eq!(ids(&paint_order(&boxes)), [2, 3, 1]);

    // And within one stack level, too.
    let boxes = [
        b(1, Step::PositiveContexts, 2, 9),
        b(2, Step::PositiveContexts, 2, 4),
    ];
    assert_eq!(ids(&paint_order(&boxes)), [2, 1]);
}

#[test]
fn o05_a_stack_level_is_consulted_only_where_appendix_e_gives_one() {
    // Within the flow steps the specification says nothing about
    // `z-index`, so consulting it would invent an order. Tree order
    // decides, whatever the level says.
    let boxes = [
        b(1, Step::BlockLevel, 100, 0),
        b(2, Step::BlockLevel, -100, 1),
    ];
    assert_eq!(ids(&paint_order(&boxes)), [1, 2]);
}

#[test]
fn o06_a_stacking_context_leaves_its_parents_flow_steps() {
    // The first question, because it is what moves a box out of the
    // flow steps entirely.
    assert_eq!(
        step_of(true, -1, false, false, false),
        Step::NegativeContexts
    );
    assert_eq!(step_of(true, 0, false, false, false), Step::Level0);
    assert_eq!(
        step_of(true, 1, false, false, false),
        Step::PositiveContexts
    );
    // Even a float or an inline, when it makes a context.
    assert_eq!(step_of(true, 0, false, true, false), Step::Level0);
    assert_eq!(step_of(true, 2, false, false, true), Step::PositiveContexts);
}

#[test]
fn o07_the_flow_steps_are_decided_in_the_order_appendix_e_gives() {
    // Positioned, then floated, then inline-level, then block-level.
    assert_eq!(step_of(false, 0, true, false, false), Step::Level0);
    assert_eq!(
        step_of(false, 0, true, true, true),
        Step::Level0,
        "positioned wins"
    );
    assert_eq!(
        step_of(false, 0, false, true, true),
        Step::Floats,
        "float beats inline"
    );
    assert_eq!(step_of(false, 0, false, false, true), Step::InlineLevel);
    assert_eq!(step_of(false, 0, false, false, false), Step::BlockLevel);
}

#[test]
fn o08_a_positioned_box_with_z_index_auto_paints_in_step_six() {
    // It does not make a stacking context, but it paints with the
    // ones that do at level 0.
    assert_eq!(step_of(false, 0, true, false, false), Step::Level0);
    let boxes = [
        b(1, Step::Level0, 0, 1),
        b(2, Step::Level0, 0, 0),
        b(3, Step::InlineLevel, 0, 99),
    ];
    assert_eq!(ids(&paint_order(&boxes)), [3, 2, 1], "inlines first");
}

#[test]
fn o09_an_empty_context_has_an_empty_order() {
    assert!(paint_order(&[]).is_empty());
}
