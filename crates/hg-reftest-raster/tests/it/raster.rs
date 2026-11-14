//! Filling rectangles and comparing surfaces.

use hg_reftest_raster::{Difference, Surface, compare};
use hg_types::geometry::{LayoutUnit, PhysicalPoint, PhysicalRect, PhysicalSize};

fn u(px: i32) -> LayoutUnit {
    LayoutUnit::from_px(px)
}

fn rect(x: i32, y: i32, w: i32, h: i32) -> PhysicalRect {
    PhysicalRect::new(
        PhysicalPoint::new(u(x), u(y)),
        PhysicalSize::new(u(w), u(h)),
    )
}

#[test]
fn r01_a_whole_pixel_rectangle_covers_exactly_its_pixels() {
    let mut s = Surface::new(4, 4);
    s.fill(rect(1, 1, 2, 2));
    assert_eq!(s.get(0, 0), Some(0));
    assert_eq!(s.get(1, 1), Some(255));
    assert_eq!(s.get(2, 2), Some(255));
    assert_eq!(s.get(3, 3), Some(0));
    assert_eq!(s.pixels().iter().filter(|p| **p == 255).count(), 4);
}

#[test]
fn r02_a_fractional_edge_covers_its_pixel_in_part() {
    // Rounding here would hide exactly the sub-pixel differences
    // layout exists to get right.
    let mut s = Surface::new(2, 1);
    // Half a pixel wide, starting at zero.
    let half = PhysicalRect::new(
        PhysicalPoint::new(LayoutUnit::ZERO, LayoutUnit::ZERO),
        PhysicalSize::new(LayoutUnit::from_raw(32), u(1)),
    );
    s.fill(half);
    assert_eq!(s.get(0, 0), Some(127), "half of 255, rounded down");
    assert_eq!(s.get(1, 0), Some(0));
}

#[test]
fn r03_a_rectangle_straddling_a_boundary_splits_its_coverage() {
    let mut s = Surface::new(3, 1);
    // From 0.5px to 2.5px: half of pixel 0, all of 1, half of 2.
    let r = PhysicalRect::new(
        PhysicalPoint::new(LayoutUnit::from_raw(32), LayoutUnit::ZERO),
        PhysicalSize::new(u(2), u(1)),
    );
    s.fill(r);
    assert_eq!(s.get(0, 0), Some(127));
    assert_eq!(s.get(1, 0), Some(255));
    assert_eq!(s.get(2, 0), Some(127));
}

#[test]
fn r04_coverage_in_two_axes_multiplies() {
    let mut s = Surface::new(1, 1);
    // A quarter of the pixel: half in each axis.
    let r = PhysicalRect::new(
        PhysicalPoint::new(LayoutUnit::ZERO, LayoutUnit::ZERO),
        PhysicalSize::new(LayoutUnit::from_raw(32), LayoutUnit::from_raw(32)),
    );
    s.fill(r);
    assert_eq!(s.get(0, 0), Some(63), "a quarter of 255, rounded down");
}

#[test]
fn r05_a_rectangle_outside_the_surface_draws_nothing_and_does_not_panic() {
    let mut s = Surface::new(2, 2);
    s.fill(rect(10, 10, 5, 5));
    s.fill(rect(-10, -10, 5, 5));
    assert!(s.pixels().iter().all(|p| *p == 0));
    // A rectangle crossing the edge draws its visible part only.
    s.fill(rect(1, 1, 100, 100));
    assert_eq!(s.get(1, 1), Some(255));
    assert_eq!(s.get(0, 0), Some(0));
}

#[test]
fn r06_an_empty_rectangle_draws_nothing() {
    let mut s = Surface::new(2, 2);
    s.fill(rect(0, 0, 0, 5));
    s.fill(rect(0, 0, 5, 0));
    s.fill(PhysicalRect::default());
    assert!(s.pixels().iter().all(|p| *p == 0));
}

#[test]
fn r07_overlapping_fills_saturate_rather_than_wrapping() {
    let mut s = Surface::new(1, 1);
    for _ in 0..5 {
        s.fill(rect(0, 0, 1, 1));
    }
    assert_eq!(s.get(0, 0), Some(255), "saturated, not wrapped to 251");
}

#[test]
fn r08_equal_surfaces_compare_equal_and_different_sizes_do_not_compare() {
    let mut a = Surface::new(4, 4);
    let mut b = Surface::new(4, 4);
    a.fill(rect(1, 1, 2, 2));
    b.fill(rect(1, 1, 2, 2));
    assert_eq!(compare(&a, &b, 0), Difference::Equal);
    assert_eq!(
        compare(&a, &Surface::new(5, 4), 0),
        Difference::SizeDiffers,
        "no comparison is meaningful"
    );
}

#[test]
fn r09_a_difference_reports_how_many_pixels_and_how_far() {
    let mut a = Surface::new(4, 1);
    let mut b = Surface::new(4, 1);
    a.fill(rect(0, 0, 2, 1));
    b.fill(rect(0, 0, 3, 1));
    assert_eq!(
        compare(&a, &b, 0),
        Difference::Pixels {
            count: 1,
            worst: 255
        }
    );
}

#[test]
fn r10_a_tolerance_forgives_a_rounding_difference_and_not_a_moved_box() {
    let mut a = Surface::new(2, 1);
    let mut b = Surface::new(2, 1);
    // Two edges a hair apart: a rounding difference.
    a.fill(PhysicalRect::new(
        PhysicalPoint::new(LayoutUnit::ZERO, LayoutUnit::ZERO),
        PhysicalSize::new(LayoutUnit::from_raw(64), u(1)),
    ));
    b.fill(PhysicalRect::new(
        PhysicalPoint::new(LayoutUnit::ZERO, LayoutUnit::ZERO),
        PhysicalSize::new(LayoutUnit::from_raw(63), u(1)),
    ));
    assert_ne!(compare(&a, &b, 0), Difference::Equal);
    assert_eq!(compare(&a, &b, 8), Difference::Equal);

    // A box a whole pixel away is not forgiven by any sane tolerance.
    let mut c = Surface::new(2, 1);
    c.fill(rect(1, 0, 1, 1));
    assert_ne!(compare(&a, &c, 8), Difference::Equal);
}
