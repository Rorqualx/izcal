//! Exact coverage: the polygon clipped to each pixel, by the shoelace
//! formula over rationals.
//!
//! Sutherland and Hodgman's clip against four half-planes, then
//! Green's theorem. The clipped polygon's signed area is the integral
//! of the winding number over that pixel, which is what the non-zero
//! rule needs before its clamp — so the comparison with `hg-raster`
//! is of the same quantity, computed two ways, and not of two
//! definitions that happen to look alike.

use hg_display_list::display_list::Point;
use hg_raster::cover::Mask;
use hg_raster::flatten::SUB;

use crate::rational::Rational;

/// A point with exact coordinates, in subunits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Exact {
    x: Rational,
    y: Rational,
}

/// Which side of an axis-aligned line a point is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

fn inside(p: Exact, side: Side, at: Rational) -> bool {
    match side {
        Side::Left => p.x >= at,
        Side::Right => p.x <= at,
        Side::Top => p.y >= at,
        Side::Bottom => p.y <= at,
    }
}

/// Where the segment `a`→`b` crosses the axis-aligned line.
fn crossing(a: Exact, b: Exact, side: Side, at: Rational) -> Option<Exact> {
    match side {
        Side::Left | Side::Right => {
            let dx = b.x.minus(a.x)?;
            if dx == Rational::ZERO {
                return Some(Exact { x: at, y: a.y });
            }
            let t = at.minus(a.x)?.over(dx)?;
            Some(Exact {
                x: at,
                y: a.y.plus(b.y.minus(a.y)?.times(t)?)?,
            })
        }
        Side::Top | Side::Bottom => {
            let dy = b.y.minus(a.y)?;
            if dy == Rational::ZERO {
                return Some(Exact { x: a.x, y: at });
            }
            let t = at.minus(a.y)?.over(dy)?;
            Some(Exact {
                x: a.x.plus(b.x.minus(a.x)?.times(t)?)?,
                y: at,
            })
        }
    }
}

fn clip(poly: &[Exact], side: Side, at: Rational) -> Option<Vec<Exact>> {
    let mut out: Vec<Exact> = Vec::with_capacity(poly.len().saturating_add(4));
    let n = poly.len();
    for i in 0..n {
        let a = *poly.get(i)?;
        let j = if i.saturating_add(1) < n {
            i.saturating_add(1)
        } else {
            0
        };
        let b = *poly.get(j)?;
        let a_in = inside(a, side, at);
        let b_in = inside(b, side, at);
        if a_in {
            out.push(a);
        }
        if a_in != b_in {
            out.push(crossing(a, b, side, at)?);
        }
    }
    Some(out)
}

/// Twice the signed area of a closed polygon, by the shoelace
/// formula.
fn shoelace2(poly: &[Exact]) -> Option<Rational> {
    let n = poly.len();
    if n < 3 {
        return Some(Rational::ZERO);
    }
    let mut acc = Rational::ZERO;
    for i in 0..n {
        let a = *poly.get(i)?;
        let j = if i.saturating_add(1) < n {
            i.saturating_add(1)
        } else {
            0
        };
        let b = *poly.get(j)?;
        acc = acc.plus(a.x.times(b.y)?.minus(a.y.times(b.x)?)?)?;
    }
    Some(acc)
}

/// The exact non-zero coverage of `points` over a `width` × `height`
/// mask, as 8-bit alpha.
///
/// # Errors
/// `None` when the exact arithmetic would leave `i128`, which no
/// shape inside a reasonable mask reaches and which is reported
/// rather than wrapped.
#[must_use]
pub fn coverage(points: &[Point], width: usize, height: usize) -> Option<Mask> {
    let poly: Vec<Exact> = points
        .iter()
        .map(|p| Exact {
            x: Rational::whole(i128::from(p.x)),
            y: Rational::whole(i128::from(p.y)),
        })
        .collect();
    if poly.len() < 3 {
        return Some(Mask::new(width, height));
    }
    let mut alpha: Vec<u8> = Vec::with_capacity(width.saturating_mul(height));
    let pixel_area2 = Rational::whole(
        i128::from(SUB)
            .checked_mul(i128::from(SUB))?
            .checked_mul(2)?,
    );
    for y in 0..height {
        for x in 0..width {
            let left = Rational::whole(i128::try_from(x).ok()?.checked_mul(i128::from(SUB))?);
            let right = Rational::whole(
                i128::try_from(x.checked_add(1)?)
                    .ok()?
                    .checked_mul(i128::from(SUB))?,
            );
            let top = Rational::whole(i128::try_from(y).ok()?.checked_mul(i128::from(SUB))?);
            let bottom = Rational::whole(
                i128::try_from(y.checked_add(1)?)
                    .ok()?
                    .checked_mul(i128::from(SUB))?,
            );

            let mut clipped = clip(&poly, Side::Left, left)?;
            if !clipped.is_empty() {
                clipped = clip(&clipped, Side::Right, right)?;
            }
            if !clipped.is_empty() {
                clipped = clip(&clipped, Side::Top, top)?;
            }
            if !clipped.is_empty() {
                clipped = clip(&clipped, Side::Bottom, bottom)?;
            }
            let area2 = shoelace2(&clipped)?.abs()?;
            let fraction = area2.over(pixel_area2)?;
            alpha.push(fraction.to_alpha()?);
        }
    }
    Some(Mask::from_alpha(width, height, alpha))
}

/// How two masks differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Difference {
    /// How many pixels differ at all.
    pub pixels: usize,
    /// The largest difference, in 8-bit levels.
    pub worst: u8,
    /// Where the worst one is.
    pub at: (usize, usize),
}

/// Compares a mask with the exact one.
#[must_use]
pub fn check(got: &Mask, want: &Mask) -> Difference {
    let mut d = Difference::default();
    for y in 0..got.height().min(want.height()) {
        for x in 0..got.width().min(want.width()) {
            let a = got.get(x, y);
            let b = want.get(x, y);
            let delta = a.abs_diff(b);
            if delta != 0 {
                d.pixels = d.pixels.saturating_add(1);
                if delta > d.worst {
                    d.worst = delta;
                    d.at = (x, y);
                }
            }
        }
    }
    d
}
