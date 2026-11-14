//! Exact-area coverage by signed-area accumulation (§6.4).
//!
//! # The method
//!
//! Every edge of the polygon is walked once. For each pixel it passes
//! through, two numbers are accumulated:
//!
//! - **area**, the signed area of the trapezoid between the edge and
//!   the pixel's right edge, over the part of the edge inside that
//!   pixel;
//! - **cover**, the signed height of that part, which is what every
//!   pixel to its *right* gains.
//!
//! A left-to-right prefix sum along each row then turns the pair into
//! the winding number at every pixel, as an exact area. Nothing is
//! sampled, so a thin shape never disappears between samples and an
//! edge at any angle gets the same answer.
//!
//! # Units, and the one place this is not exact
//!
//! Coordinates are 24.8, so a pixel is 256 subunits across and an
//! exact area is in 1/65,536 px². §6.4 says "1/256 px² units", which
//! would mean a rounding per edge per pixel; accumulating in
//! 1/65,536 is both exact and no more work, and **the difference is
//! an item for GR-M0's review** rather than a silent choice.
//!
//! What is *not* exact is the x of an edge where it crosses a
//! horizontal pixel boundary: it is a rational, and it is rounded to
//! the nearest subunit. That is one specified rounding per crossing,
//! identical on every machine because it is integer arithmetic — so
//! GR-D3's bit-identical claim holds — but it is not the same number
//! an exact computation gives, and `hg-raster-ref` computes the exact
//! one. How far apart they are is measured rather than assumed.

use hg_display_list::display_list::{FillRule, Point};

use crate::flatten::SUB;

/// Winding of exactly one, in the accumulator's units.
///
/// `2 · SUB²`: the factor of two is there so the trapezoid's
/// half never needs a division, which is one rounding removed.
pub const FULL: i64 = 2 * SUB * SUB;

/// An 8-bit coverage mask.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    width: usize,
    height: usize,
    alpha: Vec<u8>,
}

impl Mask {
    /// An empty mask.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            alpha: vec![0; width.saturating_mul(height)],
        }
    }

    /// Its width.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Its height.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// The coverage at `(x, y)`, or 0 outside.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x >= self.width {
            return 0;
        }
        y.checked_mul(self.width)
            .and_then(|r| r.checked_add(x))
            .and_then(|i| self.alpha.get(i))
            .copied()
            .unwrap_or(0)
    }

    /// Its bytes, row-major.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.alpha
    }

    /// A mask from bytes already computed, for a reference
    /// implementation that computes them another way.
    ///
    /// Short input is padded with zeros and long input is truncated,
    /// so the mask's size is always the one asked for: a reference
    /// that miscounted would otherwise compare against a mask of a
    /// different shape and find no differences at all.
    #[must_use]
    pub fn from_alpha(width: usize, height: usize, mut alpha: Vec<u8>) -> Self {
        alpha.resize(width.saturating_mul(height), 0);
        Self {
            width,
            height,
            alpha,
        }
    }
}

/// The accumulator a polygon is rasterized into.
#[derive(Debug, Clone)]
pub struct Coverage {
    width: usize,
    height: usize,
    area: Vec<i64>,
    cover: Vec<i64>,
}

impl Coverage {
    /// An accumulator for a `width` × `height` mask.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let n = width.saturating_mul(height);
        Self {
            width,
            height,
            area: vec![0; n],
            cover: vec![0; n],
        }
    }

    /// Forgets everything accumulated.
    pub fn clear(&mut self) {
        self.area.fill(0);
        self.cover.fill(0);
    }

    fn at(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        y.checked_mul(self.width)?.checked_add(x)
    }

    fn add(&mut self, x: usize, y: usize, area: i64, cover: i64) {
        let Some(i) = self.at(x, y) else { return };
        if let Some(a) = self.area.get_mut(i) {
            *a = a.saturating_add(area);
        }
        if let Some(c) = self.cover.get_mut(i) {
            *c = c.saturating_add(cover);
        }
    }

    /// Accumulates every edge of the closed ring `points`.
    pub fn add_polygon(&mut self, points: &[Point]) {
        let n = points.len();
        if n < 3 {
            return;
        }
        for i in 0..n {
            let Some(a) = points.get(i) else { continue };
            let j = i.checked_add(1).unwrap_or(0);
            let b = if j < n { points.get(j) } else { points.first() };
            let Some(b) = b else { continue };
            self.add_edge(a, b);
        }
    }

    /// Accumulates one edge.
    fn add_edge(&mut self, p0: &Point, p1: &Point) {
        let (y0, y1) = (i64::from(p0.y), i64::from(p1.y));
        if y0 == y1 {
            // A horizontal edge contributes no winding anywhere.
            return;
        }
        let direction: i64 = if y1 > y0 { 1 } else { -1 };
        let (top, bottom) = if y1 > y0 { (p0, p1) } else { (p1, p0) };
        let (ty, by) = (i64::from(top.y), i64::from(bottom.y));
        let (tx, bx) = (i64::from(top.x), i64::from(bottom.x));

        // Clip to the mask's rows.
        let height_sub = i64::try_from(self.height).unwrap_or(0).saturating_mul(SUB);
        let start_y = ty.max(0);
        let end_y = by.min(height_sub);
        if start_y >= end_y {
            return;
        }

        let dy = by.saturating_sub(ty);
        let dx = bx.saturating_sub(tx);
        // x at a given y, rounded to the nearest subunit: the one
        // specified rounding this method carries.
        let x_at = |y: i64| -> i64 {
            let num = y.saturating_sub(ty).saturating_mul(dx);
            tx.saturating_add(round_div(num, dy))
        };

        let mut y = start_y;
        while y < end_y {
            let row = y.div_euclid(SUB);
            let row_end = row.saturating_add(1).saturating_mul(SUB).min(end_y);
            let seg_y0 = y;
            let seg_y1 = row_end;
            let x0 = x_at(seg_y0);
            let x1 = x_at(seg_y1);
            if let Ok(r) = usize::try_from(row) {
                self.add_row_segment(r, seg_y0, seg_y1, x0, x1, direction);
            }
            y = row_end;
        }
    }

    /// One edge's part inside one row, split across the pixels it
    /// crosses.
    fn add_row_segment(&mut self, row: usize, y0: i64, y1: i64, x0: i64, x1: i64, direction: i64) {
        let height = y1.saturating_sub(y0);
        if height == 0 {
            return;
        }
        let width_sub = i64::try_from(self.width).unwrap_or(0).saturating_mul(SUB);
        // Everything left of the mask contributes a full-height cover
        // to column 0, as if the edge were at x = 0: that is what
        // clipping a winding number means.
        let (xa, xb) = (x0.min(x1), x0.max(x1));
        if xb <= 0 {
            self.add(
                0,
                row,
                height.saturating_mul(direction).saturating_mul(2 * SUB),
                height.saturating_mul(direction).saturating_mul(2 * SUB),
            );
            return;
        }
        if xa >= width_sub {
            // Entirely to the right of the mask: nothing inside it is
            // affected.
            return;
        }

        let first = xa.max(0).div_euclid(SUB);
        let last = xb.min(width_sub.saturating_sub(1)).div_euclid(SUB);
        let dx = x1.saturating_sub(x0);

        if first == last || dx == 0 {
            // One column.
            let col = first.clamp(0, i64::try_from(self.width).unwrap_or(0).saturating_sub(1));
            let left = col.saturating_mul(SUB);
            let ca = (x0.saturating_sub(left)).clamp(0, SUB);
            let cb = (x1.saturating_sub(left)).clamp(0, SUB);
            self.add_cell(col, row, height, ca, cb, direction);
            return;
        }

        // The edge crosses several columns in this row. Split it at
        // each vertical pixel boundary, by the y at which it reaches
        // that boundary: the same rounding as the row split.
        let mut prev_y = y0;
        let mut prev_x = x0;
        let rising = dx > 0;
        let mut col = x0.clamp(0, width_sub.saturating_sub(1)).div_euclid(SUB);
        loop {
            let boundary = if rising {
                col.saturating_add(1).saturating_mul(SUB)
            } else {
                col.saturating_mul(SUB)
            };
            let crossed = if rising { x1 > boundary } else { x1 < boundary };
            if !crossed {
                break;
            }
            // y where the edge reaches `boundary`.
            let num = boundary
                .saturating_sub(x0)
                .saturating_mul(y1.saturating_sub(y0));
            let cut_y = y0.saturating_add(round_div(num, dx)).clamp(y0, y1);
            let left = col.saturating_mul(SUB);
            self.add_cell(
                col,
                row,
                cut_y.saturating_sub(prev_y),
                prev_x.saturating_sub(left).clamp(0, SUB),
                boundary.saturating_sub(left).clamp(0, SUB),
                direction,
            );
            prev_y = cut_y;
            prev_x = boundary;
            col = if rising {
                col.saturating_add(1)
            } else {
                col.saturating_sub(1)
            };
            if col < 0 || col >= i64::try_from(self.width).unwrap_or(0) {
                return;
            }
        }
        let left = col.saturating_mul(SUB);
        self.add_cell(
            col,
            row,
            y1.saturating_sub(prev_y),
            prev_x.saturating_sub(left).clamp(0, SUB),
            x1.saturating_sub(left).clamp(0, SUB),
            direction,
        );
    }

    /// One pixel's trapezoid.
    fn add_cell(&mut self, col: i64, row: usize, height: i64, xa: i64, xb: i64, direction: i64) {
        if height == 0 {
            return;
        }
        let Ok(c) = usize::try_from(col) else { return };
        let signed = height.saturating_mul(direction);
        // The area to the *right* of the edge inside this pixel,
        // doubled so the trapezoid's half is free.
        let area = signed.saturating_mul((2 * SUB).saturating_sub(xa).saturating_sub(xb));
        let cover = signed.saturating_mul(2 * SUB);
        self.add(c, row, area, cover);
    }

    /// Resolves the accumulator into an 8-bit mask under `rule`.
    #[must_use]
    pub fn resolve(&self, rule: FillRule) -> Mask {
        let mut mask = Mask::new(self.width, self.height);
        for row in 0..self.height {
            let mut running = 0_i64;
            for col in 0..self.width {
                let Some(i) = self.at(col, row) else { continue };
                let area = self.area.get(i).copied().unwrap_or(0);
                let cover = self.cover.get(i).copied().unwrap_or(0);
                let winding = running.saturating_add(area);
                running = running.saturating_add(cover);
                let covered = match rule {
                    FillRule::NonZero => winding.abs().min(FULL),
                    FillRule::EvenOdd => even_odd(winding),
                };
                if let Some(slot) = mask.alpha.get_mut(i) {
                    *slot = to_alpha(covered);
                }
            }
        }
        mask
    }
}

/// The even-odd fold: a triangular wave of period `2 · FULL`.
fn even_odd(winding: i64) -> i64 {
    let period = FULL.saturating_mul(2);
    let m = winding.rem_euclid(period);
    if m > FULL {
        period.saturating_sub(m)
    } else {
        m
    }
}

/// Coverage to 8-bit alpha, by **one** rounding.
fn to_alpha(covered: i64) -> u8 {
    let clamped = covered.clamp(0, FULL);
    let scaled = clamped.saturating_mul(255);
    let half = FULL / 2;
    let v = scaled.saturating_add(half).checked_div(FULL).unwrap_or(0);
    u8::try_from(v.clamp(0, 255)).unwrap_or(255)
}

/// `round(a / b)` half away from zero, for a non-zero `b`.
fn round_div(a: i64, b: i64) -> i64 {
    if b == 0 {
        return 0;
    }
    let (a, b) = if b < 0 {
        (a.saturating_neg(), b.saturating_neg())
    } else {
        (a, b)
    };
    let half = b / 2;
    if a >= 0 {
        a.saturating_add(half).checked_div(b).unwrap_or(0)
    } else {
        a.saturating_sub(half).checked_div(b).unwrap_or(0)
    }
}
