//! Geometry: the layout unit, physical and flow-relative boxes, and
//! the mapping between them.
//!
//! Owned by the layout spec (`reports/Izcal layout.md` §9.5) and living
//! here because paint, the GPU service's display-list validation and
//! DevTools all use these types — so they are L2 contracts, tier P.
//!
//! # The layout unit
//!
//! [`LayoutUnit`] is **a signed 32-bit integer counting 1/64 CSS px**,
//! so the range is ±33,554,431.98 px (LY-D3). Two reasons, both from
//! the layout spec §6.2:
//!
//! - **1/64 converts to and from binary floating point exactly**,
//!   which matters because style's computed lengths are IEEE binary
//!   (ST §6.9). WebKit moved from 1/60 to 1/64 in 2012 to stop losing
//!   precision in conversions and to avoid integer division. Thirds
//!   are inexact either way; 1/60 would make thirds exact and every
//!   float conversion lossy.
//! - **It is finer than the text contract's quantum** for font sizes
//!   under 1,024 px, so cumulative text positions convert into it
//!   without drift (TX-D4).
//!
//! **Every operation saturates.** It never wraps and never panics, so
//! an overflowing page lays out identically on every platform, which is
//! a testable behaviour rather than undefined (LY-13).
//!
//! # What is not here
//!
//! **No floating point.** G39 confines `f32` and `f64` in the layout
//! crates to `hg-layout-num`'s boundary conversion functions, and the
//! conversion to [`LayoutUnit`] goes through *one* rounding function
//! there. Putting a `from_f64` here would be a second one.
//!
//! **No matrix or transform.** Those belong to the graphics track with
//! the paint pipeline; layout's geometry is axis-aligned, and a
//! transform never changes a layout box.

/// How many [`LayoutUnit`]s make one CSS pixel.
pub const UNITS_PER_PX: i32 = 64;

/// A length in 1/64 CSS px (LY-D3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct LayoutUnit(i32);

impl LayoutUnit {
    /// Zero.
    pub const ZERO: Self = Self(0);
    /// The most negative length, −33,554,432 px.
    pub const MIN: Self = Self(i32::MIN);
    /// The greatest length, 33,554,431.98 px.
    pub const MAX: Self = Self(i32::MAX);
    /// One CSS pixel.
    pub const PX: Self = Self(UNITS_PER_PX);

    /// A length from its raw 1/64 px count.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// The raw 1/64 px count.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// A whole number of CSS pixels, saturating.
    #[must_use]
    pub const fn from_px(px: i32) -> Self {
        Self(px.saturating_mul(UNITS_PER_PX))
    }

    /// The length in whole CSS pixels, truncated toward zero.
    ///
    /// For a rounded answer, the caller says which rounding it wants:
    /// layout's own conversions go through `hg-layout-num`, which
    /// rounds half to even.
    #[must_use]
    pub const fn to_px_trunc(self) -> i32 {
        match self.0.checked_div(UNITS_PER_PX) {
            Some(v) => v,
            // `UNITS_PER_PX` is 64, so division cannot fail; answering
            // keeps the type free of a panicking path.
            None => 0,
        }
    }

    /// Addition, saturating.
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Subtraction, saturating.
    #[must_use]
    pub const fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Negation, saturating — so `-MIN` is `MAX` and not an overflow.
    #[must_use]
    pub const fn neg(self) -> Self {
        Self(self.0.saturating_neg())
    }

    /// The absolute value, saturating.
    #[must_use]
    pub const fn abs(self) -> Self {
        if self.0 < 0 { self.neg() } else { self }
    }

    /// Multiplication by a whole number, saturating.
    #[must_use]
    pub const fn scale(self, by: i32) -> Self {
        Self(self.0.saturating_mul(by))
    }

    /// The smaller of two lengths.
    #[must_use]
    pub const fn min(self, other: Self) -> Self {
        if self.0 < other.0 { self } else { other }
    }

    /// The greater of two lengths.
    #[must_use]
    pub const fn max(self, other: Self) -> Self {
        if self.0 > other.0 { self } else { other }
    }

    /// Clamped to `[low, high]`.
    ///
    /// When `low` is greater than `high`, `low` wins — which is what
    /// CSS says for a `min-width` above a `max-width`.
    #[must_use]
    pub const fn clamp(self, low: Self, high: Self) -> Self {
        self.min(high).max(low)
    }

    /// Whether the length is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Whether the length is below zero.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }
}

impl core::ops::Add for LayoutUnit {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::add(self, other)
    }
}

impl core::ops::Sub for LayoutUnit {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::sub(self, other)
    }
}

impl core::ops::Neg for LayoutUnit {
    type Output = Self;
    fn neg(self) -> Self {
        Self::neg(self)
    }
}

impl core::fmt::Display for LayoutUnit {
    /// The length in CSS pixels, exactly — 1/64 is a power of two, so
    /// every value has a terminating decimal of at most 6 places.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let negative = self.0 < 0;
        // `i64` so that `i32::MIN` has an absolute value.
        let raw = i64::from(self.0).saturating_abs();
        let whole = raw.checked_div(64).unwrap_or(0);
        let frac = raw.checked_rem(64).unwrap_or(0);
        if negative {
            f.write_str("-")?;
        }
        if frac == 0 {
            return write!(f, "{whole}px");
        }
        // frac/64 = frac * 15625 / 1_000_000.
        let millionths = frac.saturating_mul(15_625);
        let text = format!("{millionths:06}");
        write!(f, "{whole}.{}px", text.trim_end_matches('0'))
    }
}

/// One of the four physical sides of a box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalSide {
    /// The top.
    Top,
    /// The right.
    Right,
    /// The bottom.
    Bottom,
    /// The left.
    Left,
}

/// A `writing-mode` value (CSS Writing Modes 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum WritingMode {
    /// `horizontal-tb`.
    #[default]
    HorizontalTb,
    /// `vertical-rl`.
    VerticalRl,
    /// `vertical-lr`.
    VerticalLr,
    /// `sideways-rl`.
    SidewaysRl,
    /// `sideways-lr`.
    SidewaysLr,
}

/// A `direction` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Direction {
    /// `ltr`.
    #[default]
    Ltr,
    /// `rtl`.
    Rtl,
}

/// The pair every abstract-to-physical mapping is a function of.
///
/// Writing Modes 4 §6.4 is explicit that the table is "based on the
/// **used** `direction` and `writing-mode`", and its note says what
/// makes the used value differ: "in vertical writing modes, a
/// `text-orientation` value of `upright` forces the used `direction`
/// to `ltr`". So `text-orientation` never appears here — style
/// resolves it into the used direction before layout sees it, and
/// [`Flow::used`] is where that is written down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Flow {
    /// The writing mode.
    pub mode: WritingMode,
    /// The **used** direction.
    pub direction: Direction,
}

impl Flow {
    /// A flow from a writing mode and direction.
    #[must_use]
    pub const fn new(mode: WritingMode, direction: Direction) -> Self {
        Self { mode, direction }
    }

    /// The used flow, given the computed `direction` and whether
    /// `text-orientation` is `upright`.
    ///
    /// Writing Modes 4 §6.4's note: in a vertical writing mode,
    /// `upright` forces the used direction to `ltr`.
    #[must_use]
    pub const fn used(mode: WritingMode, direction: Direction, upright: bool) -> Self {
        let direction = if upright && mode.is_vertical() {
            Direction::Ltr
        } else {
            direction
        };
        Self { mode, direction }
    }

    /// The physical side `block-start` maps to.
    #[must_use]
    pub const fn block_start(self) -> PhysicalSide {
        match self.mode {
            WritingMode::HorizontalTb => PhysicalSide::Top,
            WritingMode::VerticalRl | WritingMode::SidewaysRl => PhysicalSide::Right,
            WritingMode::VerticalLr | WritingMode::SidewaysLr => PhysicalSide::Left,
        }
    }

    /// The physical side `block-end` maps to.
    #[must_use]
    pub const fn block_end(self) -> PhysicalSide {
        self.block_start().opposite()
    }

    /// The physical side `inline-start` maps to.
    #[must_use]
    pub const fn inline_start(self) -> PhysicalSide {
        match (self.mode, self.direction) {
            (WritingMode::HorizontalTb, Direction::Ltr) => PhysicalSide::Left,
            (WritingMode::HorizontalTb, Direction::Rtl) => PhysicalSide::Right,
            (
                WritingMode::VerticalRl | WritingMode::SidewaysRl | WritingMode::VerticalLr,
                Direction::Ltr,
            )
            | (WritingMode::SidewaysLr, Direction::Rtl) => PhysicalSide::Top,
            (
                WritingMode::VerticalRl | WritingMode::SidewaysRl | WritingMode::VerticalLr,
                Direction::Rtl,
            )
            | (WritingMode::SidewaysLr, Direction::Ltr) => PhysicalSide::Bottom,
        }
    }

    /// The physical side `inline-end` maps to.
    #[must_use]
    pub const fn inline_end(self) -> PhysicalSide {
        self.inline_start().opposite()
    }

    /// The physical side `over` maps to — the **line-relative** top.
    ///
    /// Not the same as [`Flow::block_start`]: in `vertical-lr`,
    /// block-start is the left and over is the right. That one row is
    /// the reason the two are separate functions.
    #[must_use]
    pub const fn over(self) -> PhysicalSide {
        match self.mode {
            WritingMode::HorizontalTb => PhysicalSide::Top,
            WritingMode::VerticalRl | WritingMode::SidewaysRl | WritingMode::VerticalLr => {
                PhysicalSide::Right
            }
            WritingMode::SidewaysLr => PhysicalSide::Left,
        }
    }

    /// The physical side `under` maps to.
    #[must_use]
    pub const fn under(self) -> PhysicalSide {
        self.over().opposite()
    }

    /// The physical side `line-left` maps to.
    ///
    /// Line-relative, so it does not depend on `direction`: it is the
    /// side a line starts from before bidi reordering.
    #[must_use]
    pub const fn line_left(self) -> PhysicalSide {
        match self.mode {
            WritingMode::HorizontalTb => PhysicalSide::Left,
            WritingMode::VerticalRl | WritingMode::SidewaysRl | WritingMode::VerticalLr => {
                PhysicalSide::Top
            }
            WritingMode::SidewaysLr => PhysicalSide::Bottom,
        }
    }

    /// The physical side `line-right` maps to.
    #[must_use]
    pub const fn line_right(self) -> PhysicalSide {
        self.line_left().opposite()
    }

    /// Whether the block axis is horizontal, which is the same as
    /// asking whether the writing mode is vertical.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        self.mode.is_vertical()
    }
}

impl WritingMode {
    /// Whether this is a vertical writing mode: the inline axis runs
    /// up or down and `block-size` is a width.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        !matches!(self, Self::HorizontalTb)
    }
}

impl PhysicalSide {
    /// The side across the box.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Whether the side is on the horizontal axis.
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// A point in physical coordinates, x rightward and y downward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalPoint {
    /// The distance from the left.
    pub x: LayoutUnit,
    /// The distance from the top.
    pub y: LayoutUnit,
}

/// A size in physical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalSize {
    /// The width.
    pub width: LayoutUnit,
    /// The height.
    pub height: LayoutUnit,
}

/// An axis-aligned rectangle in physical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalRect {
    /// The top-left corner.
    pub origin: PhysicalPoint,
    /// The size.
    pub size: PhysicalSize,
}

/// Four physical lengths, one per side — a margin, border, padding or
/// inset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalSides {
    /// The top.
    pub top: LayoutUnit,
    /// The right.
    pub right: LayoutUnit,
    /// The bottom.
    pub bottom: LayoutUnit,
    /// The left.
    pub left: LayoutUnit,
}

/// A point in flow-relative coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LogicalPoint {
    /// The distance from the inline-start edge.
    pub inline: LayoutUnit,
    /// The distance from the block-start edge.
    pub block: LayoutUnit,
}

/// A size in flow-relative coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LogicalSize {
    /// The inline size.
    pub inline: LayoutUnit,
    /// The block size.
    pub block: LayoutUnit,
}

/// Four flow-relative lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LogicalSides {
    /// The block-start side.
    pub block_start: LayoutUnit,
    /// The block-end side.
    pub block_end: LayoutUnit,
    /// The inline-start side.
    pub inline_start: LayoutUnit,
    /// The inline-end side.
    pub inline_end: LayoutUnit,
}

/// A rectangle in flow-relative coordinates, placed from the
/// block-start inline-start corner of its containing block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LogicalRect {
    /// The block-start inline-start corner.
    pub start: LogicalPoint,
    /// The size.
    pub size: LogicalSize,
}

impl PhysicalPoint {
    /// A point.
    #[must_use]
    pub const fn new(x: LayoutUnit, y: LayoutUnit) -> Self {
        Self { x, y }
    }
}

impl PhysicalSize {
    /// A size.
    #[must_use]
    pub const fn new(width: LayoutUnit, height: LayoutUnit) -> Self {
        Self { width, height }
    }

    /// The size in flow-relative terms.
    ///
    /// Writing Modes 4 §6.4: `block-size` is the height in a
    /// horizontal writing mode and the width in every vertical one.
    #[must_use]
    pub const fn to_logical(self, flow: Flow) -> LogicalSize {
        if flow.is_vertical() {
            LogicalSize {
                inline: self.height,
                block: self.width,
            }
        } else {
            LogicalSize {
                inline: self.width,
                block: self.height,
            }
        }
    }
}

impl LogicalSize {
    /// A size.
    #[must_use]
    pub const fn new(inline: LayoutUnit, block: LayoutUnit) -> Self {
        Self { inline, block }
    }

    /// The size in physical terms.
    #[must_use]
    pub const fn to_physical(self, flow: Flow) -> PhysicalSize {
        if flow.is_vertical() {
            PhysicalSize {
                width: self.block,
                height: self.inline,
            }
        } else {
            PhysicalSize {
                width: self.inline,
                height: self.block,
            }
        }
    }
}

impl LogicalSides {
    /// Four lengths.
    #[must_use]
    pub const fn new(
        block_start: LayoutUnit,
        block_end: LayoutUnit,
        inline_start: LayoutUnit,
        inline_end: LayoutUnit,
    ) -> Self {
        Self {
            block_start,
            block_end,
            inline_start,
            inline_end,
        }
    }

    /// The length on one flow-relative side.
    #[must_use]
    pub const fn get(self, side: LogicalSide) -> LayoutUnit {
        match side {
            LogicalSide::BlockStart => self.block_start,
            LogicalSide::BlockEnd => self.block_end,
            LogicalSide::InlineStart => self.inline_start,
            LogicalSide::InlineEnd => self.inline_end,
        }
    }

    /// The same four lengths, named physically.
    #[must_use]
    pub const fn to_physical(self, flow: Flow) -> PhysicalSides {
        let mut out = PhysicalSides {
            top: LayoutUnit::ZERO,
            right: LayoutUnit::ZERO,
            bottom: LayoutUnit::ZERO,
            left: LayoutUnit::ZERO,
        };
        out = out.with(flow.block_start(), self.block_start);
        out = out.with(flow.block_end(), self.block_end);
        out = out.with(flow.inline_start(), self.inline_start);
        out.with(flow.inline_end(), self.inline_end)
    }

    /// The sum along the inline axis.
    #[must_use]
    pub const fn inline_sum(self) -> LayoutUnit {
        self.inline_start.add(self.inline_end)
    }

    /// The sum along the block axis.
    #[must_use]
    pub const fn block_sum(self) -> LayoutUnit {
        self.block_start.add(self.block_end)
    }
}

/// One of the four flow-relative sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalSide {
    /// `block-start`.
    BlockStart,
    /// `block-end`.
    BlockEnd,
    /// `inline-start`.
    InlineStart,
    /// `inline-end`.
    InlineEnd,
}

impl PhysicalSides {
    /// Four lengths.
    #[must_use]
    pub const fn new(
        top: LayoutUnit,
        right: LayoutUnit,
        bottom: LayoutUnit,
        left: LayoutUnit,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// The length on one physical side.
    #[must_use]
    pub const fn get(self, side: PhysicalSide) -> LayoutUnit {
        match side {
            PhysicalSide::Top => self.top,
            PhysicalSide::Right => self.right,
            PhysicalSide::Bottom => self.bottom,
            PhysicalSide::Left => self.left,
        }
    }

    /// The same four lengths with one side replaced.
    #[must_use]
    pub const fn with(self, side: PhysicalSide, value: LayoutUnit) -> Self {
        match side {
            PhysicalSide::Top => Self { top: value, ..self },
            PhysicalSide::Right => Self {
                right: value,
                ..self
            },
            PhysicalSide::Bottom => Self {
                bottom: value,
                ..self
            },
            PhysicalSide::Left => Self {
                left: value,
                ..self
            },
        }
    }

    /// The same four lengths, named flow-relatively.
    #[must_use]
    pub const fn to_logical(self, flow: Flow) -> LogicalSides {
        LogicalSides {
            block_start: self.get(flow.block_start()),
            block_end: self.get(flow.block_end()),
            inline_start: self.get(flow.inline_start()),
            inline_end: self.get(flow.inline_end()),
        }
    }

    /// The sum of the left and right lengths.
    #[must_use]
    pub const fn horizontal_sum(self) -> LayoutUnit {
        self.left.add(self.right)
    }

    /// The sum of the top and bottom lengths.
    #[must_use]
    pub const fn vertical_sum(self) -> LayoutUnit {
        self.top.add(self.bottom)
    }
}

impl PhysicalRect {
    /// A rectangle.
    #[must_use]
    pub const fn new(origin: PhysicalPoint, size: PhysicalSize) -> Self {
        Self { origin, size }
    }

    /// The left edge.
    #[must_use]
    pub const fn left(self) -> LayoutUnit {
        self.origin.x
    }

    /// The top edge.
    #[must_use]
    pub const fn top(self) -> LayoutUnit {
        self.origin.y
    }

    /// The right edge.
    #[must_use]
    pub const fn right(self) -> LayoutUnit {
        self.origin.x.add(self.size.width)
    }

    /// The bottom edge.
    #[must_use]
    pub const fn bottom(self) -> LayoutUnit {
        self.origin.y.add(self.size.height)
    }

    /// Whether the rectangle holds no area.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.size.width.0 <= 0 || self.size.height.0 <= 0
    }

    /// Whether the point is inside, counting the top and left edges
    /// and not the bottom and right — the half-open rule hit testing
    /// needs so that touching boxes do not both claim a point.
    #[must_use]
    pub const fn contains(self, point: PhysicalPoint) -> bool {
        point.x.0 >= self.left().0
            && point.x.0 < self.right().0
            && point.y.0 >= self.top().0
            && point.y.0 < self.bottom().0
    }

    /// The overlap of two rectangles, empty when they do not meet.
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Self {
            origin: PhysicalPoint { x: left, y: top },
            size: PhysicalSize {
                width: right.sub(left).max(LayoutUnit::ZERO),
                height: bottom.sub(top).max(LayoutUnit::ZERO),
            },
        }
    }

    /// The smallest rectangle holding both.
    ///
    /// An empty rectangle contributes nothing, so a union with one is
    /// the other — which is what a bounding box of a list of fragments
    /// needs, and what a naive min/max would get wrong.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self {
            origin: PhysicalPoint { x: left, y: top },
            size: PhysicalSize {
                width: right.sub(left),
                height: bottom.sub(top),
            },
        }
    }

    /// The rectangle grown by `sides` on each side.
    #[must_use]
    pub const fn outset(self, sides: PhysicalSides) -> Self {
        Self {
            origin: PhysicalPoint {
                x: self.origin.x.sub(sides.left),
                y: self.origin.y.sub(sides.top),
            },
            size: PhysicalSize {
                width: self.size.width.add(sides.horizontal_sum()),
                height: self.size.height.add(sides.vertical_sum()),
            },
        }
    }

    /// The rectangle shrunk by `sides`, never below zero.
    #[must_use]
    pub const fn inset(self, sides: PhysicalSides) -> Self {
        Self {
            origin: PhysicalPoint {
                x: self.origin.x.add(sides.left),
                y: self.origin.y.add(sides.top),
            },
            size: PhysicalSize {
                width: self
                    .size
                    .width
                    .sub(sides.horizontal_sum())
                    .max(LayoutUnit::ZERO),
                height: self
                    .size
                    .height
                    .sub(sides.vertical_sum())
                    .max(LayoutUnit::ZERO),
            },
        }
    }

    /// The rectangle moved by `(dx, dy)`.
    #[must_use]
    pub const fn translate(self, dx: LayoutUnit, dy: LayoutUnit) -> Self {
        Self {
            origin: PhysicalPoint {
                x: self.origin.x.add(dx),
                y: self.origin.y.add(dy),
            },
            size: self.size,
        }
    }
}

impl LogicalRect {
    /// A rectangle.
    #[must_use]
    pub const fn new(start: LogicalPoint, size: LogicalSize) -> Self {
        Self { start, size }
    }

    /// The rectangle in physical terms, inside a containing block of
    /// size `container`.
    ///
    /// The container is needed because a flow-relative rectangle is
    /// placed from the block-start inline-start corner, and where that
    /// corner is depends on the mode: in `vertical-rl` the block axis
    /// runs right to left, so a block offset of zero is at the
    /// container's *right* edge. Without the container, the conversion
    /// would silently assume `horizontal-tb`.
    #[must_use]
    pub const fn to_physical(self, flow: Flow, container: PhysicalSize) -> PhysicalRect {
        let size = self.size.to_physical(flow);
        let x = match (flow.block_start(), flow.inline_start()) {
            (PhysicalSide::Left, _) => self.start.block,
            (PhysicalSide::Right, _) => container.width.sub(self.start.block).sub(size.width),
            (_, PhysicalSide::Left) => self.start.inline,
            (_, PhysicalSide::Right) => container.width.sub(self.start.inline).sub(size.width),
            // Neither axis is horizontal, which cannot happen: one of
            // the two is always the horizontal one.
            _ => LayoutUnit::ZERO,
        };
        let y = match (flow.block_start(), flow.inline_start()) {
            (PhysicalSide::Top, _) => self.start.block,
            (PhysicalSide::Bottom, _) => container.height.sub(self.start.block).sub(size.height),
            (_, PhysicalSide::Top) => self.start.inline,
            (_, PhysicalSide::Bottom) => container.height.sub(self.start.inline).sub(size.height),
            _ => LayoutUnit::ZERO,
        };
        PhysicalRect {
            origin: PhysicalPoint { x, y },
            size,
        }
    }
}

impl LogicalPoint {
    /// A point.
    #[must_use]
    pub const fn new(inline: LayoutUnit, block: LayoutUnit) -> Self {
        Self { inline, block }
    }
}
