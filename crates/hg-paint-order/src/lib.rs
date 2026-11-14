//! `hg-paint-order`: the paint-order contract. L4, tier R.
//!
//! **Ownership is split, deliberately.** The graphics spec owns the
//! production implementation (GR §6.24); this spec owns the
//! *requirements* (LY §6.14) and the Appendix-E reference builder
//! below, which is a **test oracle**. `hg-hit` and `hg-layout-query`
//! read paint's output only through this contract (G38), so neither
//! depends on how paint happens to be organized.
//!
//! # Why there is a reference builder at all
//!
//! CSS 2.1 Appendix E gives painting order as a numbered procedure.
//! A production painter does not follow it literally — it builds
//! display lists, groups chunks, and reorders what it can prove is
//! equivalent. Each of those is a claim that the result is the order
//! Appendix E describes, and this is what that claim is checked
//! against.
//!
//! # What Appendix E says
//!
//! Within a stacking context, in this order:
//!
//! 1. the background and borders of the element forming the context;
//! 2. the child stacking contexts with **negative** stack levels, most
//!    negative first;
//! 3. the in-flow, non-inline-level, non-positioned descendants;
//! 4. the non-positioned floats;
//! 5. the in-flow, inline-level, non-positioned descendants;
//! 6. the child stacking contexts with stack level **0** and the
//!    positioned descendants with stack level 0;
//! 7. the child stacking contexts with **positive** stack levels,
//!    least positive first.
//!
//! Within each of those, **tree order**. That tie-break is what makes
//! the result a total order rather than a partial one, and it is the
//! part a reordering optimization is most likely to lose.

/// A box's identity, as the fragment store names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaintId(pub u32);

/// Which of Appendix E's seven steps a box is painted in.
///
/// Ordered so that the lesser paints first, which is what lets the
/// whole order be one sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    /// 1. The stacking context's own background and borders.
    RootBackground,
    /// 2. Negative-stack-level child stacking contexts.
    NegativeContexts,
    /// 3. In-flow, non-inline-level, non-positioned descendants.
    BlockLevel,
    /// 4. Non-positioned floats.
    Floats,
    /// 5. In-flow, inline-level, non-positioned descendants.
    InlineLevel,
    /// 6. Stack level 0: child stacking contexts and positioned
    ///    descendants.
    Level0,
    /// 7. Positive-stack-level child stacking contexts.
    PositiveContexts,
}

/// One box, as the paint order sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintBox {
    /// Its identity.
    pub id: PaintId,
    /// Which step it paints in.
    pub step: Step,
    /// Its `z-index`, for the two steps that are ordered by it.
    pub stack_level: i32,
    /// Its position in tree order, which is every step's tie-break.
    pub tree_order: u32,
}

/// Appendix E's order for one stacking context's contents.
///
/// The sort is **stable** by construction: boxes are ordered by step,
/// then by stack level where the step says so, then by tree order. No
/// two boxes compare equal unless their tree order does, so the result
/// is a total order and a painter that produced a different one
/// differs for a reason this names.
#[must_use]
pub fn paint_order(boxes: &[PaintBox]) -> Vec<PaintId> {
    let mut sorted: Vec<PaintBox> = boxes.to_vec();
    sorted.sort_by(|a, b| {
        a.step
            .cmp(&b.step)
            .then_with(|| {
                // Only the two stacking-context steps are ordered by
                // stack level; within the others it says nothing, and
                // consulting it would invent an order the
                // specification does not give.
                if matches!(a.step, Step::NegativeContexts | Step::PositiveContexts) {
                    a.stack_level.cmp(&b.stack_level)
                } else {
                    core::cmp::Ordering::Equal
                }
            })
            .then_with(|| a.tree_order.cmp(&b.tree_order))
    });
    sorted.into_iter().map(|b| b.id).collect()
}

/// The step a box paints in, from the facts the box tree has.
///
/// The rules are Appendix E's, read as a decision in this order: a
/// stacking context of its own first, because that is what moves a box
/// out of its parent's flow steps entirely.
#[must_use]
pub const fn step_of(
    establishes_stacking_context: bool,
    stack_level: i32,
    positioned: bool,
    floated: bool,
    inline_level: bool,
) -> Step {
    if establishes_stacking_context {
        if stack_level < 0 {
            return Step::NegativeContexts;
        }
        if stack_level > 0 {
            return Step::PositiveContexts;
        }
        return Step::Level0;
    }
    if positioned {
        // A positioned descendant with `z-index: auto` paints in step
        // 6 beside the stack-level-0 contexts.
        return Step::Level0;
    }
    if floated {
        return Step::Floats;
    }
    if inline_level {
        return Step::InlineLevel;
    }
    Step::BlockLevel
}
