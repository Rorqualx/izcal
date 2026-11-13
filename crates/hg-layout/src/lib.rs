//! `hg-layout`: the layout contract. L4, tier R.
//!
//! **LY-D1.** Every formatting context implements one signature:
//!
//! ```text
//! layout(box, &ConstraintSpace, Option<&BreakToken>) -> LayoutResult
//! intrinsic(box, &IntrinsicInput) -> MinMaxSizes
//! ```
//!
//! and the constraint space is the **complete** input besides the
//! box's own subtree, its computed styles and the frame's recorded
//! state. Nothing else is read: no previous fragment, no sibling's
//! state, no global.
//!
//! That one rule is the design. It is what ended hysteresis in
//! LayoutNG — the "infinitely growing" box of Chrome 92 and earlier —
//! and it is what makes the cache sound, because a key that is the
//! whole input cannot miss a dependency.
//!
//! # What this crate holds at LY-M0
//!
//! The contract's **types** and the one function that compares two
//! spaces:
//!
//! - [`ConstraintSpace`] — the complete input, with [`AvailableSize`]
//!   carrying CSS Sizing's four cases.
//! - [`BreakToken`] — hashable data, never a pointer into mutable
//!   state, so it is part of fragment-tree hashes (DET-1).
//! - [`LayoutResult`] — fragment, break token, status, baselines,
//!   end margins, out-of-flow list, best break.
//! - [`AdjoiningMargins`] — the compact margin-collapsing summary.
//! - [`FloatArea`] — the per-BFC exclusions a space carries.
//! - [`relayout_needed`] — **the one central diff**.
//!
//! The formatting contexts themselves are LY-M1 and later. Writing
//! their inputs down first is deliberate: the contract is what the
//! reference engine and the production engine must share, and it is
//! the thing LY-M0's design review has to fix.
//!
//! # G40
//!
//! Every crate of the layout spec is `forbid(unsafe_code)` with no
//! inventory entry, and a panic reachable from content is a
//! denial-of-service bug. So nothing here panics: indices are typed
//! and checked, and arithmetic saturates.

use hg_fragment::{BoxId, FragmentId};
use hg_types::geometry::{Direction, Flow, LayoutUnit, LogicalSides, LogicalSize, WritingMode};

/// How much room there is along one axis (CSS Sizing 3).
///
/// Four cases, not a number and a flag: an indefinite size and a
/// zero-length definite size are different inputs, and a min-content
/// constraint is not a length at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvailableSize {
    /// A definite length.
    Definite(LayoutUnit),
    /// No definite size — the containing block does not say.
    #[default]
    Indefinite,
    /// Lay out as if under a min-content constraint.
    MinContent,
    /// Lay out as if under a max-content constraint.
    MaxContent,
}

impl AvailableSize {
    /// The length, for the one case that has one.
    #[must_use]
    pub const fn definite(self) -> Option<LayoutUnit> {
        match self {
            Self::Definite(v) => Some(v),
            Self::Indefinite | Self::MinContent | Self::MaxContent => None,
        }
    }

    /// Whether this is an intrinsic-sizing constraint, in which case a
    /// percentage resolves against nothing.
    #[must_use]
    pub const fn is_intrinsic(self) -> bool {
        matches!(self, Self::MinContent | Self::MaxContent)
    }
}

/// The margin-collapsing summary carried along the block axis
/// (CSS 2.1 §8.3.1).
///
/// The largest positive and most negative adjoining margins plus
/// flags — the compact summary that suffices, rather than a list of
/// every margin in the run. Collapsing is `max(positive) +
/// min(negative)`, so two numbers are the whole state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdjoiningMargins {
    /// The largest positive adjoining margin.
    pub positive: LayoutUnit,
    /// The most negative adjoining margin.
    pub negative: LayoutUnit,
    /// Whether the run is still open — nothing non-collapsible has
    /// been seen, so a following margin joins it.
    pub open: bool,
    /// Whether clearance applies, which stops collapsing through.
    pub clearance: bool,
}

impl AdjoiningMargins {
    /// Nothing adjoining yet.
    pub const EMPTY: Self = Self {
        positive: LayoutUnit::ZERO,
        negative: LayoutUnit::ZERO,
        open: true,
        clearance: false,
    };

    /// The collapsed margin this summary stands for.
    #[must_use]
    pub const fn collapsed(self) -> LayoutUnit {
        self.positive.add(self.negative)
    }

    /// The summary with one more margin joined to it.
    #[must_use]
    pub const fn join(self, margin: LayoutUnit) -> Self {
        if margin.is_negative() {
            Self {
                negative: self.negative.min(margin),
                ..self
            }
        } else {
            Self {
                positive: self.positive.max(margin),
                ..self
            }
        }
    }

    /// Closed, because something non-collapsible came between.
    #[must_use]
    pub const fn close(self) -> Self {
        Self {
            open: false,
            ..self
        }
    }
}

/// One float or shape intruding into a BFC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exclusion {
    /// Which inline edge it sits against.
    pub side: FloatSide,
    /// Where it starts along the block axis, in the BFC's coordinates.
    pub block_start: LayoutUnit,
    /// Where it ends.
    pub block_end: LayoutUnit,
    /// How far it reaches in from its edge.
    pub inline_extent: LayoutUnit,
}

/// Which side a float sits against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatSide {
    /// The inline-start edge.
    InlineStart,
    /// The inline-end edge.
    InlineEnd,
}

/// The floats and shapes intruding from the containing BFC
/// (CSS 2.1 §9.5.1, CSS Shapes 1).
///
/// **Persistent**: an unchanged prefix is shared, so a later layout
/// with the same incoming exclusions hits the cache rather than
/// re-deriving float positions by scanning preceding siblings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FloatArea {
    exclusions: Vec<Exclusion>,
}

impl FloatArea {
    /// No floats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            exclusions: Vec::new(),
        }
    }

    /// The exclusions, in the order they were added.
    #[must_use]
    pub fn exclusions(&self) -> &[Exclusion] {
        &self.exclusions
    }

    /// The area with one more exclusion, leaving this one unchanged.
    #[must_use]
    pub fn with(&self, exclusion: Exclusion) -> Self {
        let mut next = self.clone();
        next.exclusions.push(exclusion);
        next
    }

    /// How much room a line at `block_start` of height `block_size`
    /// has, given an inline size of `available`.
    ///
    /// Returns the offset from the inline-start edge and the width.
    #[must_use]
    pub fn line_box(
        &self,
        block_start: LayoutUnit,
        block_size: LayoutUnit,
        available: LayoutUnit,
    ) -> (LayoutUnit, LayoutUnit) {
        let block_end = block_start.add(block_size);
        let mut start = LayoutUnit::ZERO;
        let mut end = available;
        for e in &self.exclusions {
            // Half-open along the block axis, as a zero-height line at
            // a float's end edge is clear of it.
            if e.block_end.raw() <= block_start.raw() || e.block_start.raw() >= block_end.raw() {
                continue;
            }
            match e.side {
                FloatSide::InlineStart => start = start.max(e.inline_extent),
                FloatSide::InlineEnd => end = end.min(available.sub(e.inline_extent)),
            }
        }
        (start, end.sub(start).max(LayoutUnit::ZERO))
    }
}

/// Which kind of fragmentainer a box is being laid out into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FragmentainerKind {
    /// A page.
    Page,
    /// A multicol column.
    Column,
    /// A `line-clamp` or `max-lines` clamp.
    Clamp,
}

/// The fragmentainer a box is being laid out into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fragmentainer {
    /// Its kind.
    pub kind: FragmentainerKind,
    /// Its block size.
    pub block_size: LayoutUnit,
    /// How far into it this box starts.
    pub block_offset: LayoutUnit,
    /// Whether it is the first fragmentainer of its chain.
    pub is_first: bool,
    /// The break-appeal threshold below which a break is not taken.
    pub break_appeal: i8,
}

/// Which sizes the parent has fixed (flex and grid stretch, a table
/// cell's second pass).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FixedSizes {
    /// The inline size is fixed by the parent.
    pub inline: bool,
    /// The block size is fixed by the parent.
    pub block: bool,
}

/// What baselines the parent is asking for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BaselineRequests {
    /// The first baseline is wanted.
    pub first: bool,
    /// The last baseline is wanted.
    pub last: bool,
}

/// **The complete input to a layout** besides the box's own subtree,
/// its computed styles and the frame's recorded state (LY-D1).
///
/// Every field is here because something reads it; nothing here is
/// optional for the cache, because the space *is* the cache key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintSpace {
    /// Available inline size.
    pub available_inline: AvailableSize,
    /// Available block size.
    pub available_block: AvailableSize,
    /// What a percentage resolves against, per axis.
    pub percentage_basis: LogicalSize,
    /// What a replaced element's percentage resolves against, which
    /// Sizing 3 makes a separate basis.
    pub replaced_percentage_basis: LogicalSize,
    /// Which sizes the parent fixed.
    pub fixed: FixedSizes,
    /// The containing block's writing mode and used direction.
    pub flow: Flow,
    /// The BFC offset, when it is known. Collapsing through empty
    /// boxes and the clearance rules can leave it unknown until a
    /// later sibling resolves it.
    pub bfc_offset: Option<LayoutUnit>,
    /// The floats and shapes intruding from the containing BFC.
    pub floats: FloatArea,
    /// The incoming adjoining-margin summary.
    pub margins: AdjoiningMargins,
    /// The fragmentainer, when fragmenting.
    pub fragmentainer: Option<Fragmentainer>,
    /// Writing Modes 4 §7.3.1's fallback size for an orthogonal flow.
    pub orthogonal_fallback: Option<LayoutUnit>,
    /// Which baselines the parent wants.
    pub baselines: BaselineRequests,
    /// The style version of the box's subtree, when a container
    /// re-entry produced a size-specific one (ST §6.9, LY §6.9).
    pub style_version: Option<u32>,
}

impl Default for ConstraintSpace {
    fn default() -> Self {
        Self {
            available_inline: AvailableSize::Indefinite,
            available_block: AvailableSize::Indefinite,
            percentage_basis: LogicalSize::default(),
            replaced_percentage_basis: LogicalSize::default(),
            fixed: FixedSizes::default(),
            flow: Flow::new(WritingMode::HorizontalTb, Direction::Ltr),
            bfc_offset: None,
            floats: FloatArea::new(),
            margins: AdjoiningMargins::EMPTY,
            fragmentainer: None,
            orthogonal_fallback: None,
            baselines: BaselineRequests::default(),
            style_version: None,
        }
    }
}

/// Per-mode state a break token carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeState {
    /// Block and inline flow: where in the item list the next
    /// fragment resumes, and how many lines are already placed, for
    /// orphans and widows.
    Flow {
        /// The item to resume at.
        item: u32,
        /// Lines placed so far.
        lines: u32,
    },
    /// Flex: the line to resume at and each item's offset.
    Flex {
        /// The flex line to resume at.
        line: u32,
        /// Where each item of that line resumed.
        item_offsets: Vec<LayoutUnit>,
    },
    /// Grid: the row to resume at.
    Grid {
        /// The grid row to resume at.
        row: u32,
    },
    /// Table: the row and cell to resume at.
    Table {
        /// The row to resume at.
        row: u32,
        /// The cell within it.
        cell: u32,
    },
    /// Multicol: the column row and where a spanner sits.
    Multicol {
        /// The column row to resume at.
        column_row: u32,
        /// The spanner's index, when one interrupts.
        spanner: Option<u32>,
    },
}

/// Where a fragmented box resumes (LY-D8).
///
/// **Data, never a pointer into mutable state**, so it is part of
/// fragment-tree hashes (DET-1) and of the reference comparison. It is
/// hashable for the same reason: the layout cache is keyed by
/// (constraint-space id, incoming break-token id), and a fragment-2
/// request must never hit fragment 1's entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakToken {
    /// The box it resumes.
    pub box_id: BoxId,
    /// How much block size has already been consumed.
    pub consumed_block_size: LayoutUnit,
    /// The children's tokens, in order.
    pub children: Vec<BreakToken>,
    /// The formatting context's own state.
    pub mode: ModeState,
}

/// How a layout ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The box finished in this fragmentainer.
    Complete,
    /// It broke inside, and the token says where to resume.
    BrokenInside,
    /// It did not fit at all and must start in the next
    /// fragmentainer.
    BreakBefore,
}

/// A break the layout could take, with how good it is.
///
/// Fragmentation "does not define whether content breaks at a
/// particular allowed break", so Izcal writes the rule down: appeal is
/// an integer, higher is better, and the fragmentainer's threshold
/// says what is acceptable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BestBreak {
    /// Where, along the block axis.
    pub block_offset: LayoutUnit,
    /// How good it is.
    pub appeal: i8,
}

/// An out-of-flow box the layout found and did not place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfFlow {
    /// The box.
    pub box_id: BoxId,
    /// Its static position, in this box's coordinates.
    pub static_position: LogicalSize,
}

/// What one layout call returns (LY-D1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutResult {
    /// The fragment it produced.
    pub fragment: FragmentId,
    /// The token to resume with, when it broke.
    pub break_token: Option<BreakToken>,
    /// How it ended.
    pub status: Status,
    /// Its baselines.
    pub baselines: hg_fragment::Baselines,
    /// The adjoining margins at its block-end edge.
    pub end_margins: AdjoiningMargins,
    /// The out-of-flow boxes it found.
    pub out_of_flow: Vec<OutOfFlow>,
    /// The best break it could take, for a parent deciding where to
    /// fragment.
    pub best_break: Option<BestBreak>,
}

/// The input to an intrinsic-size query.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntrinsicInput {
    /// The writing mode and direction to measure in.
    pub flow: Flow,
    /// The percentage basis, when one is known.
    pub percentage_basis: Option<LogicalSize>,
    /// Which axis is being asked about.
    pub axis: Axis,
}

/// Which axis a query is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Axis {
    /// The inline axis.
    #[default]
    Inline,
    /// The block axis.
    Block,
}

/// A box's min-content and max-content contributions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MinMaxSizes {
    /// The min-content size.
    pub min_content: LayoutUnit,
    /// The max-content size.
    pub max_content: LayoutUnit,
}

/// What a box's own style says about how it depends on its space.
///
/// These flags are the other half of the diff: the space alone cannot
/// say whether a change matters, because whether it does depends on
/// the box. They are computed once, from the box's computed style,
/// when the box tree is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoxFlags {
    /// Its inline size depends on the available inline size (`auto`,
    /// a percentage, `stretch`, or a shrink-to-fit).
    pub inline_size_depends_on_available: bool,
    /// Its block size depends on the available block size.
    pub block_size_depends_on_available: bool,
    /// Something in its subtree resolves a percentage against the
    /// inline basis.
    pub uses_inline_percentage: bool,
    /// Something in its subtree resolves a percentage against the
    /// block basis.
    pub uses_block_percentage: bool,
    /// It establishes a BFC, so intruding floats reach it as a whole
    /// rather than per line.
    pub is_bfc_root: bool,
    /// Its subtree holds a float or a box that clears one, so the
    /// float area matters to it.
    pub depends_on_floats: bool,
    /// It can fragment, so the fragmentainer matters.
    pub can_fragment: bool,
    /// It asks for a baseline, so a baseline request matters.
    pub has_baseline: bool,
}

/// **The one central diff** (LY-D1): may a result cached for `old` be
/// reused under `new`?
///
/// Keeping this in one function is the point. LayoutNG's published
/// account describes the same diff as kept in one place and easy to
/// unit-test; here it is also property-tested, because a cache hit
/// must equal a fresh layout (LY-3, G42).
///
/// The rule it encodes: **a difference only matters when the box
/// reads it.** A box whose inline size does not depend on the
/// available inline size reuses its result across a change to it; a
/// percentage-sized box relays out only when its basis changed.
///
/// With one exception, on **both** axes: an intrinsic constraint is a
/// different question, not a different length, so moving to or from
/// `MinContent` or `MaxContent` always relays out whatever the flags
/// say. Having that on the inline axis alone let a block-axis switch
/// between `Definite` and `MaxContent` serve a stale result, which is
/// exactly what LY-3 forbids.
#[must_use]
pub fn relayout_needed(old: &ConstraintSpace, new: &ConstraintSpace, flags: BoxFlags) -> bool {
    // The flow decides what every other field *means*, so a change to
    // it invalidates everything.
    if old.flow != new.flow {
        return true;
    }
    // A style version is a different style, so a different layout.
    if old.style_version != new.style_version {
        return true;
    }
    // A parent that fixes a size overrides the box's own sizing, so
    // the box's dependence flags do not apply to it.
    if old.fixed != new.fixed {
        return true;
    }
    if old.available_inline != new.available_inline
        && (flags.inline_size_depends_on_available
            || old.available_inline.is_intrinsic()
            || new.available_inline.is_intrinsic())
    {
        return true;
    }
    if old.available_block != new.available_block
        && (flags.block_size_depends_on_available
            || flags.can_fragment
            || old.available_block.is_intrinsic()
            || new.available_block.is_intrinsic())
    {
        return true;
    }
    if old.percentage_basis.inline != new.percentage_basis.inline && flags.uses_inline_percentage {
        return true;
    }
    if old.percentage_basis.block != new.percentage_basis.block && flags.uses_block_percentage {
        return true;
    }
    if old.replaced_percentage_basis != new.replaced_percentage_basis
        && (flags.uses_inline_percentage || flags.uses_block_percentage)
    {
        return true;
    }
    if old.floats != new.floats && flags.depends_on_floats {
        return true;
    }
    // A BFC root's position relative to intruding floats is its own
    // input; a non-root takes floats per line and does not read it.
    if old.bfc_offset != new.bfc_offset && (flags.is_bfc_root || flags.depends_on_floats) {
        return true;
    }
    if old.margins != new.margins {
        return true;
    }
    if old.fragmentainer != new.fragmentainer && flags.can_fragment {
        return true;
    }
    if old.orthogonal_fallback != new.orthogonal_fallback {
        return true;
    }
    if old.baselines != new.baselines && flags.has_baseline {
        return true;
    }
    false
}

/// Which margins a box contributes, before collapsing.
#[must_use]
pub const fn own_margins(margins: LogicalSides) -> AdjoiningMargins {
    AdjoiningMargins::EMPTY
        .join(margins.block_start)
        .join(margins.block_end)
}
