//! `hg-layout-num`: the boundary between style's IEEE values and
//! layout's integers. L4, tier R.
//!
//! **G39.** `f32` and `f64` appear in the layout crates *only* in this
//! crate's boundary conversion functions, and every conversion to
//! [`LayoutUnit`] goes through one rounding function here. A lint
//! rejects floating point anywhere else in layout. The reason is
//! DET-1: the same page must lay out identically on every ISA, and
//! floating point that is re-associated or contracted by a compiler
//! does not promise that. Integers do.
//!
//! So this file is where the `as` casts live, each one named and each
//! one saturating, and nowhere else.
//!
//! # The two types that cross the boundary
//!
//! - **Lengths** arrive as computed lengths in CSS px, binary64
//!   (ST §6.9), and become [`LayoutUnit`] — 1/64 px.
//! - **Unitless numbers** — flex factors, `aspect-ratio` terms,
//!   `<number>` line heights, zoom — arrive as binary64 and become
//!   [`Number`]: a signed 64-bit value with 24 fractional bits,
//!   clamped to ±2³⁹.
//!
//! Both round **half to even**, map NaN to zero and map the infinities
//! to the range limits, which CSS Values 4 §10.9 permits for
//! out-of-range values.
//!
//! One consequence is recorded rather than hidden: **a flex factor
//! below 2⁻²⁴ counts as zero**. That is a deterministic precision
//! limit of the kind CSS Values 4 permits — "UAs should support
//! reasonably useful ranges and precisions" — and it is checked
//! black-box (Q-LY-22).
//!
//! # After the boundary
//!
//! Every ratio — free space divided among flex factors, a size
//! transferred through an aspect ratio, a percentage of a basis — is
//! computed in **`i128` with one rounding**, by [`ratio`]. One
//! rounding, not one per term, so a sum of many parts does not drift.

use hg_types::geometry::{LayoutUnit, UNITS_PER_PX};

/// How many fractional bits a [`Number`] carries.
pub const NUMBER_FRACTION_BITS: u32 = 24;

/// One, as a [`Number`]'s raw value: 2²⁴.
pub const NUMBER_ONE: i64 = 1 << NUMBER_FRACTION_BITS;

/// The same constant as a float, for the two boundary functions.
/// 2²⁴ is exact in binary64; `n04` checks the two agree.
const NUMBER_ONE_F64: f64 = 16_777_216.0;

/// A CSS `<number>` after the boundary: fixed point with
/// [`NUMBER_FRACTION_BITS`] fractional bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Number(i64);

impl Number {
    /// Zero.
    pub const ZERO: Self = Self(0);
    /// One.
    pub const ONE: Self = Self(NUMBER_ONE);
    /// The greatest value, just under 2³⁹.
    pub const MAX: Self = Self(i64::MAX);
    /// The least value, −2³⁹.
    pub const MIN: Self = Self(i64::MIN);

    /// A number from its raw fixed-point value.
    #[must_use]
    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    /// The raw fixed-point value.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// A whole number, saturating.
    #[must_use]
    pub fn from_int(value: i32) -> Self {
        Self(i64::from(value).saturating_mul(NUMBER_ONE))
    }

    /// Whether the number is zero.
    ///
    /// A flex factor below 2⁻²⁴ is zero here, which is the recorded
    /// precision limit (Q-LY-22).
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Whether the number is below zero.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
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

    /// The smaller of two numbers.
    #[must_use]
    pub const fn min(self, other: Self) -> Self {
        if self.0 < other.0 { self } else { other }
    }

    /// The greater of two numbers.
    #[must_use]
    pub const fn max(self, other: Self) -> Self {
        if self.0 > other.0 { self } else { other }
    }

    /// The product, in `i128` with one rounding back to fixed point.
    #[must_use]
    pub fn times(self, other: Self) -> Self {
        let product = i128::from(self.0).saturating_mul(i128::from(other.0));
        match div_round_half_even(product, i128::from(NUMBER_ONE)) {
            Some(v) => Self(saturate_i64(v)),
            // The divisor is a non-zero constant, so this cannot
            // happen; answering keeps the crate free of a panic.
            None => Self::ZERO,
        }
    }
}

/// Saturates an `i128` into an `i64`.
///
/// `try_from` rather than `as`: the conversion that saturates by
/// design is the one at the float boundary, and this crate keeps the
/// two kinds apart so a reader never has to work out which a given
/// `as` was.
fn saturate_i64(value: i128) -> i64 {
    i64::try_from(value).unwrap_or(if value > 0 { i64::MAX } else { i64::MIN })
}

/// Saturates an `i128` into an `i32`.
fn saturate_i32(value: i128) -> i32 {
    i32::try_from(value).unwrap_or(if value > 0 { i32::MAX } else { i32::MIN })
}

/// `numerator / denominator`, rounded half to even.
///
/// Returns `None` for a zero denominator, because the layout spec is
/// explicit that "division by zero is defined **per call site** from
/// the spec (for example a zero flex-grow sum)". A function that chose
/// an answer here would make every call site take that choice without
/// stating it.
#[must_use]
pub fn div_round_half_even(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator == 0 {
        return None;
    }
    // Work with magnitudes so the rounding rule does not depend on the
    // language's truncation direction.
    let negative = (numerator < 0) != (denominator < 0);
    let n = numerator.unsigned_abs();
    let d = denominator.unsigned_abs();
    let quotient = n.checked_div(d)?;
    let remainder = n.checked_rem(d)?;
    let twice = remainder.saturating_mul(2);
    let round_up = if twice > d {
        true
    } else if twice < d {
        false
    } else {
        // Exactly half: to even.
        quotient % 2 == 1
    };
    let magnitude = if round_up {
        quotient.saturating_add(1)
    } else {
        quotient
    };
    // The positive side of `i128` reaches one less than the negative,
    // so a magnitude of exactly `2^127` saturates; every other
    // operation here makes the same choice.
    let bounded = i128::try_from(magnitude).unwrap_or(i128::MAX);
    Some(if negative {
        bounded.saturating_neg()
    } else {
        bounded
    })
}

/// **The one rounding function** (G39): an amount in 1/64 px units,
/// saturated into a [`LayoutUnit`].
///
/// Sums and products of many terms are accumulated in `i128` and come
/// back through here once.
#[must_use]
pub fn to_layout_unit(units: i128) -> LayoutUnit {
    LayoutUnit::from_raw(saturate_i32(units))
}

/// `numerator / denominator` in 1/64 px units, rounded half to even
/// and saturated — one rounding for the whole expression.
///
/// Returns `None` for a zero denominator; see [`div_round_half_even`].
#[must_use]
pub fn ratio(numerator: i128, denominator: i128) -> Option<LayoutUnit> {
    div_round_half_even(numerator, denominator).map(to_layout_unit)
}

/// `value × factor`, with one rounding.
#[must_use]
pub fn scale(value: LayoutUnit, factor: Number) -> LayoutUnit {
    let product = i128::from(value.raw()).saturating_mul(i128::from(factor.raw()));
    match ratio(product, i128::from(NUMBER_ONE)) {
        Some(v) => v,
        None => LayoutUnit::ZERO,
    }
}

/// `value × numerator / denominator`, with one rounding — a
/// percentage of a basis, or a size through an aspect ratio.
///
/// Returns `None` for a zero denominator.
#[must_use]
pub fn scale_ratio(value: LayoutUnit, numerator: i128, denominator: i128) -> Option<LayoutUnit> {
    ratio(
        i128::from(value.raw()).saturating_mul(numerator),
        denominator,
    )
}

/// What a floating-point value is before the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Nan,
    PositiveInfinity,
    NegativeInfinity,
    Finite,
}

fn kind(value: f64) -> Kind {
    if value.is_nan() {
        Kind::Nan
    } else if value == f64::INFINITY {
        Kind::PositiveInfinity
    } else if value == f64::NEG_INFINITY {
        Kind::NegativeInfinity
    } else {
        Kind::Finite
    }
}

/// A computed length in CSS px, as style produces it, converted into a
/// [`LayoutUnit`] — **the boundary** (G39).
///
/// NaN becomes zero and the infinities become the range limits, which
/// CSS Values 4 §10.9 allows for out-of-range values. Everything else
/// is multiplied by 64 and rounded half to even. 1/64 is a power of
/// two, so a value already on the grid converts exactly.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "G39: this is the one place a float becomes an integer in layout, and \
              `as` from a float saturates rather than wrapping"
)]
pub fn length_from_px(px: f64) -> LayoutUnit {
    match kind(px) {
        Kind::Nan => LayoutUnit::ZERO,
        Kind::PositiveInfinity => LayoutUnit::MAX,
        Kind::NegativeInfinity => LayoutUnit::MIN,
        Kind::Finite => {
            let units = (px * f64::from(UNITS_PER_PX)).round_ties_even();
            LayoutUnit::from_raw(units as i32)
        }
    }
}

/// A CSS `<number>` as style produces it, converted into a
/// [`Number`] — **the boundary** (G39).
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "G39: as `length_from_px`, and `as` from a float saturates"
)]
pub fn number_from_f64(value: f64) -> Number {
    match kind(value) {
        Kind::Nan => Number::ZERO,
        Kind::PositiveInfinity => Number::MAX,
        Kind::NegativeInfinity => Number::MIN,
        Kind::Finite => {
            let raw = (value * NUMBER_ONE_F64).round_ties_even();
            Number::from_raw(raw as i64)
        }
    }
}

/// A length back to CSS px, for a used-value read or a report.
///
/// Exact: 1/64 is a power of two and an `i32` has fewer significant
/// bits than a binary64 mantissa, so nothing is lost.
#[must_use]
pub fn length_to_px(value: LayoutUnit) -> f64 {
    f64::from(value.raw()) / f64::from(UNITS_PER_PX)
}

/// A number back to binary64.
///
/// **Not exact** in general: a [`Number`] carries up to 63 significant
/// bits and binary64 holds 53, so a value with more than 53 is
/// rounded. The conversion exists for reporting; nothing in layout
/// goes back across the boundary.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "stated above: 63 bits into a 53-bit mantissa, for reporting only"
)]
pub fn number_to_f64(value: Number) -> f64 {
    value.raw() as f64 / NUMBER_ONE_F64
}
