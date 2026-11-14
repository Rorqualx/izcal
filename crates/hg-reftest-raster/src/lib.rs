//! `hg-reftest-raster`: a minimal CPU rasterizer, for reftests.
//! Tools, tier T. **It never ships.**
//!
//! A reftest asks one question — do these two renderings match? —
//! and answering it needs a raster. The graphics track's raster is
//! GR-M1's and later; this is what lets layout's reftests run before
//! then, and it is deliberately the smallest thing that can answer
//! that question honestly.
//!
//! # What it draws
//!
//! Axis-aligned rectangles into an 8-bit **coverage** buffer: one
//! byte per pixel saying how much of it the shape covers. No colour,
//! no blending, no anti-aliased edges beyond the exact area a
//! rectangle covers. That is enough for the thing layout reftests
//! actually test — *where the boxes are* — and it is not enough for
//! anything else, which is the point: a reftest that passed here and
//! failed in the real raster would be a graphics bug, and this must
//! not hide one by being approximate.
//!
//! # Why coverage and not colour
//!
//! A layout reftest's reference usually paints the same colours in
//! the same places. Comparing coverage asks only about geometry, so
//! a difference is a layout difference and never a colour one — which
//! is what makes a failure here point at this track.

use hg_types::geometry::{LayoutUnit, PhysicalRect, UNITS_PER_PX};

/// A coverage buffer: one byte per pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Surface {
    /// A surface of `width` by `height` pixels, empty.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let n = usize::try_from(width)
            .unwrap_or(0)
            .saturating_mul(usize::try_from(height).unwrap_or(0));
        Self {
            width,
            height,
            pixels: vec![0; n],
        }
    }

    /// Its width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Its height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The coverage of one pixel, or `None` outside the surface.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> Option<u8> {
        self.pixels.get(self.offset(x, y)?).copied()
    }

    fn offset(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let row = usize::try_from(y)
            .ok()?
            .checked_mul(usize::try_from(self.width).ok()?)?;
        row.checked_add(usize::try_from(x).ok()?)
    }

    /// Every pixel's coverage, row by row.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Fills `rect`, adding its coverage.
    ///
    /// A pixel the rectangle covers in part gets that part: the
    /// rectangle's edges are in 1/64 px, so a box at a fractional
    /// position covers its edge pixels partly, and rounding here
    /// would hide exactly the sub-pixel differences layout exists to
    /// get right.
    pub fn fill(&mut self, rect: PhysicalRect) {
        if rect.is_empty() {
            return;
        }
        let (x0, x1) = (rect.left(), rect.right());
        let (y0, y1) = (rect.top(), rect.bottom());
        let first_x = floor_px(x0).max(0);
        let last_x = ceil_px(x1).min(i64::from(self.width));
        let first_y = floor_px(y0).max(0);
        let last_y = ceil_px(y1).min(i64::from(self.height));
        let mut y = first_y;
        while y < last_y {
            let mut x = first_x;
            while x < last_x {
                let cover = overlap(x0, x1, x).saturating_mul(overlap(y0, y1, y));
                // `cover` is in units of (1/64)², so 4,096 is a whole
                // pixel; scale it to 0..=255.
                let value = cover.saturating_mul(255).checked_div(4_096).unwrap_or(0);
                let add = u8::try_from(value.clamp(0, 255)).unwrap_or(255);
                if let (Ok(ux), Ok(uy)) = (u32::try_from(x), u32::try_from(y))
                    && let Some(at) = self.offset(ux, uy)
                    && let Some(slot) = self.pixels.get_mut(at)
                {
                    *slot = slot.saturating_add(add);
                }
                x = x.saturating_add(1);
            }
            y = y.saturating_add(1);
        }
    }
}

/// The whole pixels at or below `v`.
fn floor_px(v: LayoutUnit) -> i64 {
    i64::from(v.raw()).div_euclid(i64::from(UNITS_PER_PX))
}

/// The whole pixels at or above `v`.
fn ceil_px(v: LayoutUnit) -> i64 {
    let units = i64::from(UNITS_PER_PX);
    i64::from(v.raw())
        .saturating_add(units.saturating_sub(1))
        .div_euclid(units)
}

/// How much of pixel column (or row) `n` lies between `a` and `b`, in
/// 1/64 px.
fn overlap(a: LayoutUnit, b: LayoutUnit, n: i64) -> i64 {
    let units = i64::from(UNITS_PER_PX);
    let low = n.saturating_mul(units);
    let high = low.saturating_add(units);
    let start = i64::from(a.raw()).max(low);
    let end = i64::from(b.raw()).min(high);
    end.saturating_sub(start).max(0)
}

/// How two surfaces differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difference {
    /// They are the same size and every pixel matches within the
    /// tolerance.
    Equal,
    /// They are not the same size, so no comparison is meaningful.
    SizeDiffers,
    /// They differ, in this many pixels, by at most this much.
    Pixels {
        /// How many pixels differ by more than the tolerance.
        count: usize,
        /// The largest difference found.
        worst: u8,
    },
}

/// Compares two surfaces, allowing a per-pixel `tolerance`.
///
/// A tolerance rather than exact equality because a rectangle's edge
/// lands on a pixel boundary differently in the two renderings when
/// the *layout* is the same and the rounding is not — and that is a
/// difference worth seeing separately from a box in the wrong place.
/// A reftest sets it to zero unless it says why.
#[must_use]
pub fn compare(a: &Surface, b: &Surface, tolerance: u8) -> Difference {
    if a.width != b.width || a.height != b.height {
        return Difference::SizeDiffers;
    }
    let mut count = 0_usize;
    let mut worst = 0_u8;
    for (x, y) in a.pixels().iter().zip(b.pixels()) {
        let d = x.abs_diff(*y);
        if d > tolerance {
            count = count.saturating_add(1);
            worst = worst.max(d);
        }
    }
    if count == 0 {
        Difference::Equal
    } else {
        Difference::Pixels { count, worst }
    }
}
