//! The boundary conversions and the rounding function.

use hg_layout_num::{
    NUMBER_FRACTION_BITS, NUMBER_ONE, Number, div_round_half_even, length_from_px, length_to_px,
    number_from_f64, number_to_f64, ratio, scale, scale_ratio, to_layout_unit,
};
use hg_types::geometry::LayoutUnit;

fn u(px: i32) -> LayoutUnit {
    LayoutUnit::from_px(px)
}

#[test]
fn n01_a_length_on_the_grid_converts_exactly_both_ways() {
    // 1/64 is a power of two, so every multiple of it is exact in
    // binary64 — the reason LY-D3 chose 1/64 over 1/60.
    for px in [0.0, 1.0, 0.5, 0.25, 0.015625, -3.75, 100.5] {
        let unit = length_from_px(px);
        assert_eq!(length_to_px(unit), px, "{px}");
    }
    assert_eq!(length_from_px(1.0), LayoutUnit::from_raw(64));
    assert_eq!(length_from_px(0.015625), LayoutUnit::from_raw(1));
}

#[test]
fn n02_a_length_off_the_grid_rounds_half_to_even() {
    // 1/128 px is exactly half a unit.
    let half = 1.0 / 128.0;
    assert_eq!(
        length_from_px(half).raw(),
        0,
        "0.5 rounds to 0, which is even"
    );
    assert_eq!(length_from_px(3.0 * half).raw(), 2, "1.5 rounds to 2");
    assert_eq!(length_from_px(5.0 * half).raw(), 2, "2.5 rounds to 2");
    assert_eq!(length_from_px(7.0 * half).raw(), 4, "3.5 rounds to 4");
    assert_eq!(length_from_px(-3.0 * half).raw(), -2);
    assert_eq!(length_from_px(-5.0 * half).raw(), -2);
}

#[test]
fn n03_nan_is_zero_and_the_infinities_are_the_range_limits() {
    // CSS Values 4 10.9 allows this for out-of-range values.
    assert_eq!(length_from_px(f64::NAN), LayoutUnit::ZERO);
    assert_eq!(length_from_px(f64::INFINITY), LayoutUnit::MAX);
    assert_eq!(length_from_px(f64::NEG_INFINITY), LayoutUnit::MIN);
    assert_eq!(number_from_f64(f64::NAN), Number::ZERO);
    assert_eq!(number_from_f64(f64::INFINITY), Number::MAX);
    assert_eq!(number_from_f64(f64::NEG_INFINITY), Number::MIN);
    // A finite value past the range saturates rather than wrapping.
    assert_eq!(length_from_px(1e30), LayoutUnit::MAX);
    assert_eq!(length_from_px(-1e30), LayoutUnit::MIN);
}

#[test]
fn n04_a_number_carries_twenty_four_fractional_bits() {
    assert_eq!(NUMBER_FRACTION_BITS, 24);
    assert_eq!(NUMBER_ONE, 1 << 24);
    // The float constant the two boundary functions use must be the
    // same number; 2^24 is exact in binary64.
    assert_eq!(number_to_f64(Number::from_raw(NUMBER_ONE)), 1.0);
    assert_eq!(number_from_f64(16_777_216.0).raw() >> 24, 16_777_216);
    assert_eq!(number_from_f64(1.0), Number::ONE);
    assert_eq!(number_from_f64(0.5).raw(), NUMBER_ONE / 2);
    assert_eq!(number_to_f64(Number::ONE), 1.0);
    assert_eq!(number_from_f64(-2.0), Number::from_int(-2));
}

#[test]
fn n05_a_flex_factor_below_two_to_the_minus_twenty_four_is_zero() {
    // The recorded precision limit (Q-LY-22). CSS Values 4: "UAs
    // should support reasonably useful ranges and precisions."
    let smallest = 1.0 / 16_777_216.0_f64;
    assert_eq!(number_from_f64(smallest).raw(), 1);
    assert!(number_from_f64(smallest / 4.0).is_zero());
    assert!(number_from_f64(1e-12).is_zero());
    // And it is a *rounding*, not a truncation: just over half a unit
    // rounds up to one.
    assert_eq!(number_from_f64(smallest * 0.75).raw(), 1);
}

#[test]
fn n06_division_rounds_half_to_even_in_both_signs() {
    assert_eq!(div_round_half_even(5, 2), Some(2), "2.5 to even is 2");
    assert_eq!(div_round_half_even(7, 2), Some(4), "3.5 to even is 4");
    assert_eq!(div_round_half_even(3, 2), Some(2), "1.5 to even is 2");
    assert_eq!(div_round_half_even(1, 2), Some(0), "0.5 to even is 0");
    assert_eq!(div_round_half_even(-5, 2), Some(-2));
    assert_eq!(div_round_half_even(-7, 2), Some(-4));
    assert_eq!(div_round_half_even(5, -2), Some(-2));
    assert_eq!(div_round_half_even(-5, -2), Some(2));
    // Not half: ordinary rounding.
    assert_eq!(div_round_half_even(7, 3), Some(2));
    assert_eq!(div_round_half_even(8, 3), Some(3));
    assert_eq!(div_round_half_even(-8, 3), Some(-3));
}

#[test]
fn n07_division_by_zero_is_the_call_sites_decision_and_not_this_crates() {
    // The layout spec: "Division by zero is defined per call site from
    // the spec (for example a zero flex-grow sum)."
    assert_eq!(div_round_half_even(1, 0), None);
    assert_eq!(ratio(1, 0), None);
    assert_eq!(scale_ratio(u(10), 1, 0), None);
}

#[test]
fn n08_one_rounding_for_the_whole_expression_and_not_one_per_term() {
    // Three thirds of a pixel: rounding each would give 21+21+21 = 63
    // units, a unit short of the 64 the whole makes.
    let third = ratio(i128::from(u(1).raw()), 3);
    assert_eq!(third, Some(LayoutUnit::from_raw(21)));
    let three_thirds_separately = third.map(|t| t.add(t).add(t)).unwrap_or(LayoutUnit::ZERO);
    assert_eq!(three_thirds_separately.raw(), 63);
    // Done once, in i128, it is exact.
    let together = ratio(i128::from(u(1).raw()) * 3, 3);
    assert_eq!(together, Some(u(1)));
    assert_ne!(three_thirds_separately, u(1));
}

#[test]
fn n09_a_percentage_of_a_basis_is_one_multiply_and_one_divide() {
    // 33% of 100px.
    assert_eq!(
        scale_ratio(u(100), 33, 100),
        Some(LayoutUnit::from_raw(2112))
    );
    assert_eq!(length_to_px(LayoutUnit::from_raw(2112)), 33.0);
    // A ratio big enough to overflow i64 is fine in i128.
    let big = scale_ratio(LayoutUnit::MAX, 1_000_000, 1_000_000);
    assert_eq!(big, Some(LayoutUnit::MAX));
}

#[test]
fn n10_scaling_by_a_number_rounds_once() {
    assert_eq!(scale(u(10), Number::from_int(3)), u(30));
    assert_eq!(scale(u(10), number_from_f64(0.5)), u(5));
    assert_eq!(scale(u(10), Number::ZERO), LayoutUnit::ZERO);
    assert_eq!(
        scale(u(10), number_from_f64(0.9)),
        LayoutUnit::from_raw(576)
    );
    // A factor one part in 2^24 under one rounds *to* the whole at
    // 1000px, because 63999.996 units is nearer 64000 than 63999 —
    // the rounding is to the nearest unit, not a truncation.
    let almost = Number::from_raw(NUMBER_ONE - 1);
    assert_eq!(scale(u(1000), almost), u(1000));
    // At a length where the difference exceeds half a unit, it shows.
    let long = LayoutUnit::from_raw(i32::MAX);
    assert!(scale(long, almost) < long);
}

#[test]
fn n11_the_conversion_to_a_layout_unit_saturates_and_never_wraps() {
    assert_eq!(to_layout_unit(i128::from(i32::MAX) + 1), LayoutUnit::MAX);
    assert_eq!(to_layout_unit(i128::from(i32::MIN) - 1), LayoutUnit::MIN);
    assert_eq!(to_layout_unit(0), LayoutUnit::ZERO);
    assert_eq!(to_layout_unit(64), u(1));
}

#[test]
fn n12_number_arithmetic_saturates_and_multiplies_through_i128() {
    assert_eq!(Number::MAX.add(Number::ONE), Number::MAX);
    assert_eq!(Number::MIN.sub(Number::ONE), Number::MIN);
    assert_eq!(
        Number::from_int(3).times(Number::from_int(4)),
        Number::from_int(12)
    );
    assert_eq!(
        number_from_f64(0.5).times(number_from_f64(0.5)),
        number_from_f64(0.25)
    );
    // A product past the range saturates rather than wrapping.
    assert_eq!(Number::MAX.times(Number::from_int(2)), Number::MAX);
    assert_eq!(
        Number::from_int(2).min(Number::from_int(3)),
        Number::from_int(2)
    );
    assert_eq!(
        Number::from_int(2).max(Number::from_int(3)),
        Number::from_int(3)
    );
    assert!(Number::from_int(-1).is_negative());
}
