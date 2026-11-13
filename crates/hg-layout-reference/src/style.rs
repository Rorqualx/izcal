//! The computed values block layout reads, and how a percentage
//! resolves.
//!
//! Written here rather than read from `hg-css-props` because G41 says
//! so: a reference engine that shared the property system would agree
//! with it about a property system bug.
//!
//! **No floating point**, although G39 binds the layout crates and not
//! this one. A percentage is a count of millionths, resolved in `i128`
//! with one rounding — the same arithmetic the production engine does,
//! reached independently, so a difference between the two is a
//! difference in the *algorithm* and never in the rounding.

use hg_types::geometry::LayoutUnit;

/// 100%, as a count of millionths.
pub const FULL: i64 = 1_000_000;

/// A `<length-percentage>` or `auto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    /// `auto`.
    #[default]
    Auto,
    /// A length.
    Length(LayoutUnit),
    /// A percentage, in millionths: `FULL` is 100%.
    Percent(i64),
}

impl Size {
    /// A whole number of CSS pixels.
    #[must_use]
    pub const fn px(px: i32) -> Self {
        Self::Length(LayoutUnit::from_px(px))
    }

    /// A percentage, in whole percent.
    #[must_use]
    pub const fn percent(whole: i64) -> Self {
        Self::Percent(whole.saturating_mul(10_000))
    }

    /// The used value against `basis`, or `None` for `auto`.
    ///
    /// A percentage against an **indefinite** basis is `auto`, which is
    /// what makes `width: 50%` of an auto-width parent behave as
    /// `auto` — a rule a naive resolution gets wrong by treating the
    /// basis as zero.
    #[must_use]
    pub fn resolve(self, basis: Option<LayoutUnit>) -> Option<LayoutUnit> {
        match self {
            Self::Auto => None,
            Self::Length(v) => Some(v),
            Self::Percent(p) => basis.map(|b| percent_of(b, p)),
        }
    }

    /// Whether this is `auto`.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

/// `p` millionths of `basis`, rounded half to even in `i128`.
#[must_use]
pub fn percent_of(basis: LayoutUnit, p: i64) -> LayoutUnit {
    let n = i128::from(basis.raw()).saturating_mul(i128::from(p));
    let d = i128::from(FULL);
    let negative = (n < 0) != (d < 0);
    let (an, ad) = (n.unsigned_abs(), d.unsigned_abs());
    let Some(q) = an.checked_div(ad) else {
        return LayoutUnit::ZERO;
    };
    let Some(r) = an.checked_rem(ad) else {
        return LayoutUnit::ZERO;
    };
    let twice = r.saturating_mul(2);
    let up = twice > ad || (twice == ad && q % 2 == 1);
    let magnitude = if up { q.saturating_add(1) } else { q };
    let bounded = i128::try_from(magnitude).unwrap_or(i128::MAX);
    let signed = if negative {
        bounded.saturating_neg()
    } else {
        bounded
    };
    LayoutUnit::from_raw(i32::try_from(signed).unwrap_or(if signed > 0 {
        i32::MAX
    } else {
        i32::MIN
    }))
}

/// A `min-width` or `min-height`, which has no `auto` in CSS 2.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MinSize {
    /// Zero, the initial value.
    #[default]
    Zero,
    /// A length.
    Length(LayoutUnit),
    /// A percentage, in millionths.
    Percent(i64),
}

/// A `max-width` or `max-height`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaxSize {
    /// `none`, the initial value.
    #[default]
    None,
    /// A length.
    Length(LayoutUnit),
    /// A percentage, in millionths.
    Percent(i64),
}

impl MinSize {
    /// The used value against `basis`.
    ///
    /// CSS 2.2: a percentage against an indefinite basis is zero for
    /// `min-*`, which is the opposite of `max-*`'s rule.
    #[must_use]
    pub fn resolve(self, basis: Option<LayoutUnit>) -> LayoutUnit {
        match self {
            Self::Zero => LayoutUnit::ZERO,
            Self::Length(v) => v,
            Self::Percent(p) => basis.map_or(LayoutUnit::ZERO, |b| percent_of(b, p)),
        }
    }
}

impl MaxSize {
    /// The used value against `basis`, or `None` for no limit.
    #[must_use]
    pub fn resolve(self, basis: Option<LayoutUnit>) -> Option<LayoutUnit> {
        match self {
            Self::None => None,
            Self::Length(v) => Some(v),
            // A percentage against an indefinite basis is `none`.
            Self::Percent(p) => basis.map(|b| percent_of(b, p)),
        }
    }
}

/// How a box takes part in layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    /// Block-level, normal flow.
    #[default]
    Block,
    /// Inline-level.
    Inline,
    /// Block-level, establishing a block formatting context.
    FlowRoot,
    /// No box.
    None,
    /// No box of its own.
    Contents,
}

/// The `overflow` values that matter to layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    /// `visible`, the initial value, which does not establish a BFC.
    #[default]
    Visible,
    /// Anything else, which does.
    Clipped,
}

/// The computed values block layout reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoxStyle {
    /// `display`.
    pub display: Display,
    /// `overflow`.
    pub overflow: Overflow,
    /// `width`.
    pub width: Size,
    /// `min-width`.
    pub min_width: MinSize,
    /// `max-width`.
    pub max_width: MaxSize,
    /// `height`.
    pub height: Size,
    /// `min-height`.
    pub min_height: MinSize,
    /// `max-height`.
    pub max_height: MaxSize,
    /// `margin-top`.
    pub margin_top: Size,
    /// `margin-right`.
    pub margin_right: Size,
    /// `margin-bottom`.
    pub margin_bottom: Size,
    /// `margin-left`.
    pub margin_left: Size,
    /// `border-top-width`.
    pub border_top: LayoutUnit,
    /// `border-right-width`.
    pub border_right: LayoutUnit,
    /// `border-bottom-width`.
    pub border_bottom: LayoutUnit,
    /// `border-left-width`.
    pub border_left: LayoutUnit,
    /// `padding-top`.
    pub padding_top: Size,
    /// `padding-right`.
    pub padding_right: Size,
    /// `padding-bottom`.
    pub padding_bottom: Size,
    /// `padding-left`.
    pub padding_left: Size,
}

impl BoxStyle {
    /// A block box with every property at its **initial** value.
    ///
    /// `width` and `height` are `auto`; `margin` and `padding` are
    /// **zero**, which `Size::default()` is not — getting that wrong
    /// makes every box centred.
    #[must_use]
    pub const fn block() -> Self {
        let zero = Size::Length(LayoutUnit::ZERO);
        Self {
            display: Display::Block,
            overflow: Overflow::Visible,
            width: Size::Auto,
            min_width: MinSize::Zero,
            max_width: MaxSize::None,
            height: Size::Auto,
            min_height: MinSize::Zero,
            max_height: MaxSize::None,
            margin_top: zero,
            margin_right: zero,
            margin_bottom: zero,
            margin_left: zero,
            border_top: LayoutUnit::ZERO,
            border_right: LayoutUnit::ZERO,
            border_bottom: LayoutUnit::ZERO,
            border_left: LayoutUnit::ZERO,
            padding_top: zero,
            padding_right: zero,
            padding_bottom: zero,
            padding_left: zero,
        }
    }

    /// Whether the box establishes a block formatting context.
    ///
    /// CSS 2.2 §9.4.1: a box with `overflow` other than `visible`
    /// does, and so does `display: flow-root`.
    #[must_use]
    pub const fn establishes_bfc(&self) -> bool {
        matches!(self.display, Display::FlowRoot) || matches!(self.overflow, Overflow::Clipped)
    }

    /// The used padding, which is never negative and whose
    /// percentages resolve against the **inline** size on every side.
    #[must_use]
    pub fn padding(&self, basis: Option<LayoutUnit>) -> Sides {
        Sides {
            top: self.padding_top.resolve(basis).unwrap_or(LayoutUnit::ZERO),
            right: self
                .padding_right
                .resolve(basis)
                .unwrap_or(LayoutUnit::ZERO),
            bottom: self
                .padding_bottom
                .resolve(basis)
                .unwrap_or(LayoutUnit::ZERO),
            left: self.padding_left.resolve(basis).unwrap_or(LayoutUnit::ZERO),
        }
    }

    /// The used border widths.
    #[must_use]
    pub const fn border(&self) -> Sides {
        Sides {
            top: self.border_top,
            right: self.border_right,
            bottom: self.border_bottom,
            left: self.border_left,
        }
    }
}

/// Four used lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sides {
    /// The top.
    pub top: LayoutUnit,
    /// The right.
    pub right: LayoutUnit,
    /// The bottom.
    pub bottom: LayoutUnit,
    /// The left.
    pub left: LayoutUnit,
}

impl Sides {
    /// The sum of the left and right lengths.
    #[must_use]
    pub const fn horizontal(self) -> LayoutUnit {
        self.left.add(self.right)
    }

    /// The sum of the top and bottom lengths.
    #[must_use]
    pub const fn vertical(self) -> LayoutUnit {
        self.top.add(self.bottom)
    }
}
