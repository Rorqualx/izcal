//! Coverage, on shapes whose answer is known by construction.

use hg_display_list::display_list::{FillRule, Point};
use hg_raster::cover::Coverage;
use hg_raster::flatten::SUB;

fn p(x: i64, y: i64) -> Point {
    Point {
        x: i32::try_from(x).expect("in range"),
        y: i32::try_from(y).expect("in range"),
    }
}

/// A rectangle on pixel boundaries: every pixel inside is 255, every
/// pixel outside is 0, and there is nothing in between. An exact-area
/// rasterizer has no excuse here, and a sampling one would still get
/// it right — which is why this is the first test and not the last.
#[test]
fn ra10_an_aligned_rectangle_is_exact() {
    let mut cov = Coverage::new(8, 8);
    let s = SUB;
    cov.add_polygon(&[p(2 * s, s), p(6 * s, s), p(6 * s, 5 * s), p(2 * s, 5 * s)]);
    let mask = cov.resolve(FillRule::NonZero);
    for y in 0..8 {
        for x in 0..8 {
            let want = if (2..6).contains(&x) && (1..5).contains(&y) {
                255
            } else {
                0
            };
            assert_eq!(mask.get(x, y), want, "({x}, {y})");
        }
    }
}

/// A rectangle covering exactly half of each pixel in its column:
/// 128, because `round(255/2)` is 128 and there is one rounding.
#[test]
fn ra11_a_half_covered_column_is_one_rounding() {
    let mut cov = Coverage::new(4, 4);
    let s = SUB;
    let half = s / 2;
    cov.add_polygon(&[p(0, 0), p(half, 0), p(half, 4 * s), p(0, 4 * s)]);
    let mask = cov.resolve(FillRule::NonZero);
    for y in 0..4 {
        assert_eq!(mask.get(0, y), 128, "half of a pixel, row {y}");
        assert_eq!(mask.get(1, y), 0);
    }
}

/// Winding direction does not change the coverage: a rectangle wound
/// the other way is the same shape.
#[test]
fn ra12_winding_direction_does_not_change_coverage() {
    let s = SUB;
    let forwards = [p(s, s), p(3 * s, s), p(3 * s, 3 * s), p(s, 3 * s)];
    let backwards = [p(s, 3 * s), p(3 * s, 3 * s), p(3 * s, s), p(s, s)];
    let mut a = Coverage::new(5, 5);
    a.add_polygon(&forwards);
    let mut b = Coverage::new(5, 5);
    b.add_polygon(&backwards);
    assert_eq!(
        a.resolve(FillRule::NonZero).bytes(),
        b.resolve(FillRule::NonZero).bytes()
    );
}

/// A square with a square hole, wound the other way: non-zero leaves
/// the hole empty, and even-odd does the same here — the two rules
/// agree on a shape with one level of nesting, which is how the
/// even-odd path is checked without an exact oracle for it.
#[test]
fn ra13_a_hole_is_empty_under_both_rules() {
    let s = SUB;
    let outer = [p(0, 0), p(6 * s, 0), p(6 * s, 6 * s), p(0, 6 * s)];
    let inner = [
        p(2 * s, 2 * s),
        p(2 * s, 4 * s),
        p(4 * s, 4 * s),
        p(4 * s, 2 * s),
    ];
    for rule in [FillRule::NonZero, FillRule::EvenOdd] {
        let mut cov = Coverage::new(6, 6);
        cov.add_polygon(&outer);
        cov.add_polygon(&inner);
        let mask = cov.resolve(rule);
        for y in 0..6 {
            for x in 0..6 {
                let in_hole = (2..4).contains(&x) && (2..4).contains(&y);
                assert_eq!(
                    mask.get(x, y),
                    if in_hole { 0 } else { 255 },
                    "{rule:?} at ({x}, {y})"
                );
            }
        }
    }
}

/// The rules part company when a shape overlaps itself the same way
/// twice: non-zero fills the double-wound middle, even-odd empties
/// it. A rasterizer that implemented one and called it the other
/// would pass every test above.
#[test]
fn ra14_the_two_rules_differ_where_they_should() {
    let s = SUB;
    // Two squares, same winding, overlapping in the middle column.
    let left = [p(0, 0), p(4 * s, 0), p(4 * s, 4 * s), p(0, 4 * s)];
    let right = [p(2 * s, 0), p(6 * s, 0), p(6 * s, 4 * s), p(2 * s, 4 * s)];
    let mut cov = Coverage::new(6, 4);
    cov.add_polygon(&left);
    cov.add_polygon(&right);
    let nonzero = cov.resolve(FillRule::NonZero);
    let evenodd = cov.resolve(FillRule::EvenOdd);
    for y in 0..4 {
        for x in 0..6 {
            assert_eq!(nonzero.get(x, y), 255, "non-zero fills everything");
            let overlapping = (2..4).contains(&x);
            assert_eq!(
                evenodd.get(x, y),
                if overlapping { 0 } else { 255 },
                "even-odd at ({x}, {y})"
            );
        }
    }
}

/// A triangle covering exactly half of a pixel, by symmetry.
#[test]
fn ra15_a_diagonal_half_is_half() {
    let mut cov = Coverage::new(1, 1);
    let s = SUB;
    cov.add_polygon(&[p(0, 0), p(s, 0), p(0, s)]);
    let mask = cov.resolve(FillRule::NonZero);
    assert_eq!(mask.get(0, 0), 128, "half a pixel, rounded once");
}

/// Everything outside the mask is clipped and nothing wraps: a shape
/// far to the left still covers the pixels it should.
#[test]
fn ra16_a_shape_crossing_the_edge_is_clipped_not_wrapped() {
    let mut cov = Coverage::new(4, 4);
    let s = SUB;
    cov.add_polygon(&[
        p(-10 * s, -10 * s),
        p(2 * s, -10 * s),
        p(2 * s, 2 * s),
        p(-10 * s, 2 * s),
    ]);
    let mask = cov.resolve(FillRule::NonZero);
    for y in 0..4 {
        for x in 0..4 {
            let inside = x < 2 && y < 2;
            assert_eq!(mask.get(x, y), if inside { 255 } else { 0 }, "({x}, {y})");
        }
    }
}

/// An empty or degenerate polygon covers nothing rather than
/// panicking.
#[test]
fn ra17_degenerate_input_covers_nothing() {
    for points in [
        vec![],
        vec![p(0, 0)],
        vec![p(0, 0), p(SUB, 0)],
        vec![p(0, 0), p(SUB, 0), p(2 * SUB, 0)],
    ] {
        let mut cov = Coverage::new(3, 3);
        cov.add_polygon(&points);
        assert!(
            cov.resolve(FillRule::NonZero)
                .bytes()
                .iter()
                .all(|b| *b == 0),
            "{points:?}"
        );
    }
}
