//! The reference against answers that are arithmetic.
//!
//! An oracle nobody checks is a second implementation of the same
//! mistake. These are the shapes whose coverage can be written down
//! without rasterizing anything.

use core::cmp::Ordering;

use hg_display_list::display_list::Point;
use hg_raster::flatten::SUB;
use hg_raster_ref::{Rational, coverage};

fn p(x: i64, y: i64) -> Point {
    Point {
        x: i32::try_from(x).expect("in range"),
        y: i32::try_from(y).expect("in range"),
    }
}

/// An aligned rectangle: whole pixels, exactly.
#[test]
fn rr01_an_aligned_rectangle_is_whole_pixels() {
    let s = SUB;
    let mask =
        coverage(&[p(s, s), p(4 * s, s), p(4 * s, 3 * s), p(s, 3 * s)], 6, 5).expect("in range");
    for y in 0..5 {
        for x in 0..6 {
            let inside = (1..4).contains(&x) && (1..3).contains(&y);
            assert_eq!(mask.get(x, y), if inside { 255 } else { 0 }, "({x}, {y})");
        }
    }
}

/// A rectangle covering a third of each pixel in its column: 85,
/// because `round(255/3)` is 85 and the reference rounds once.
#[test]
fn rr02_a_third_of_a_pixel_is_eighty_five() {
    let s = SUB;
    let third = s / 3;
    let mask =
        coverage(&[p(0, 0), p(third, 0), p(third, 2 * s), p(0, 2 * s)], 2, 2).expect("in range");
    // 256/3 is not an integer, so the rectangle is 85/256 of a pixel
    // wide and the exact alpha is round(255 * 85 / 256) = 85.
    assert_eq!(mask.get(0, 0), 85);
    assert_eq!(mask.get(0, 1), 85);
    assert_eq!(mask.get(1, 0), 0);
}

/// A triangle filling half a pixel is 128 — the same answer by
/// symmetry that the production rasterizer gives, reached from the
/// other end.
#[test]
fn rr03_a_diagonal_half_is_half() {
    let mask = coverage(&[p(0, 0), p(SUB, 0), p(0, SUB)], 1, 1).expect("in range");
    assert_eq!(mask.get(0, 0), 128);
}

/// The rational type is exact where a float is not: a third plus a
/// third plus a third is one.
#[test]
fn rr04_rationals_are_exact() {
    let third = Rational::new(1, 3).expect("non-zero");
    let sum = third
        .plus(third)
        .and_then(|s| s.plus(third))
        .expect("in range");
    assert_eq!(sum, Rational::ONE);
    assert_eq!(sum.numerator(), 1);
    assert_eq!(sum.denominator(), 1);

    // And the reduction is real: 2/4 is 1/2.
    let half = Rational::new(2, 4).expect("non-zero");
    assert_eq!(half.denominator(), 2);
    assert_eq!(half.numerator(), 1);
    // A negative denominator moves to the numerator.
    let neg = Rational::new(1, -2).expect("non-zero");
    assert_eq!((neg.numerator(), neg.denominator()), (-1, 2));
    assert_eq!(Rational::new(1, 0), None, "a zero denominator is refused");
}

/// Rounding to alpha happens once, at the end, half away from zero.
#[test]
fn rr05_the_one_rounding() {
    for (n, d, want) in [
        (0_i128, 1_i128, 0_u8),
        (1, 1, 255),
        (1, 2, 128),
        (1, 3, 85),
        (2, 3, 170),
        (1, 510, 1),
        (1, 511, 0),
    ] {
        let r = Rational::new(n, d).expect("non-zero");
        assert_eq!(r.to_alpha(), Some(want), "{n}/{d}");
    }
}

/// The ordering is exact where cross-multiplication is not.
///
/// `n₁·d₂` for these operands is past `i128`, so the obvious
/// comparison saturates and returns `Equal` for two numbers that are
/// not equal. The oracle's `inside` depends on this ordering, so a
/// wrong answer here is a wrong coverage value with no sign of it.
#[test]
fn rr06_the_ordering_is_exact_at_the_extremes() {
    let big = i128::MAX / 2;
    let a = Rational::new(big, 3).expect("nonzero");
    let b = Rational::new(big, 5).expect("nonzero");
    // Both numerators saturate under cross-multiplication, which
    // would make them compare equal.
    assert_eq!(big.saturating_mul(5), big.saturating_mul(3));
    assert_eq!(a.cmp(&b), Ordering::Greater, "big/3 > big/5");
    assert_eq!(b.cmp(&a), Ordering::Less);
    assert_eq!(a.cmp(&a), Ordering::Equal);

    // Negative, where the signs of the quotients decide it.
    let na = Rational::new(-big, 3).expect("nonzero");
    let nb = Rational::new(-big, 5).expect("nonzero");
    assert_eq!(na.cmp(&nb), Ordering::Less, "-big/3 < -big/5");
    assert_eq!(na.cmp(&a), Ordering::Less);

    // Two fractions that differ by one unit in the last place of a
    // huge denominator: the integer parts and many continued-fraction
    // terms agree, so this is the case the loop exists for.
    let p = Rational::new(big, big - 1).expect("nonzero");
    let q = Rational::new(big, big - 2).expect("nonzero");
    assert_eq!(p.cmp(&q), Ordering::Less);
    assert_ne!(p, q);

    // Ordinary values still order the ordinary way, and the relation
    // is consistent with the arithmetic.
    let third = Rational::new(1, 3).expect("nonzero");
    let half = Rational::new(1, 2).expect("nonzero");
    assert_eq!(third.cmp(&half), Ordering::Less);
    assert_eq!(half.cmp(&Rational::whole(1)), Ordering::Less);
    assert_eq!(Rational::ZERO.cmp(&third), Ordering::Less);
    assert_eq!(
        third.plus(third).and_then(|x| x.plus(third)),
        Some(Rational::ONE)
    );
}
