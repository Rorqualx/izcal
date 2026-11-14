//! Segments to a polyline, in fixed point.
//!
//! §6.4's flattening row: "Analytic subdivision counts known before
//! emitting points, parallel per segment, error spread evenly,
//! evaluated in fixed point to a tolerance of 1/16† device px."
//!
//! **Counts known before emitting points** is the part that matters
//! for a rasterizer that must not allocate on a peer's say-so: the
//! number of line segments a curve becomes is computed from the
//! curve's own control points before any point is produced, so the
//! output buffer is sized once and the work is bounded. A subdivision
//! that recursed until flat would be a renderer choosing how much
//! memory the GPU process spends.

use hg_display_list::display_list::{Path, Point, Segment};

/// Why a path could not be flattened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FlattenError {
    /// An elliptical arc.
    ///
    /// Flattening one in integer arithmetic needs a fixed-point sine
    /// table, and its size decides the error; that is a decision
    /// GR-M0's design review should take rather than this crate
    /// should assume. Refused rather than approximated, because an
    /// approximation nobody chose is the kind that ships.
    ArcNotYetSupported,
    /// A coordinate outside the 24.8 range, so the flattened point
    /// would not be representable.
    OutOfRange,
}

/// The tolerance, in subunits: 1/16 of a device pixel is 16 subunits.
pub const TOLERANCE: i64 = 16;

/// Subunits per pixel: the `.8` of 24.8.
pub const SUB: i64 = 256;

/// Flattens `path` into `out` as a closed polygon.
///
/// The first point is the path's start and the contour is closed by
/// the caller reading it as a ring; no duplicate closing point is
/// emitted, because a duplicate would give an edge of zero length
/// that every later stage has to skip.
///
/// # Errors
/// [`FlattenError`] for an arc or an out-of-range coordinate.
pub fn flatten(path: &Path, out: &mut Vec<Point>) -> Result<(), FlattenError> {
    out.clear();
    out.push(path.start.clone());
    let mut from = path.start.clone();
    for seg in path.segments.as_slice() {
        match seg {
            Segment::Line { to } => {
                out.push(to.clone());
                from = to.clone();
            }
            Segment::Quad { control, to } => {
                let n = quad_steps(&from, control, to);
                emit_quad(&from, control, to, n, out)?;
                from = to.clone();
            }
            Segment::Cubic {
                control0,
                control1,
                to,
            } => {
                let n = cubic_steps(&from, control0, control1, to);
                emit_cubic(&from, control0, control1, to, n, out)?;
                from = to.clone();
            }
            Segment::Arc { .. } => return Err(FlattenError::ArcNotYetSupported),
        }
    }
    Ok(())
}

/// How many line segments a quadratic needs.
///
/// A quadratic's deviation from the chord is bounded by a quarter of
/// the distance from the control point to the chord's midpoint, and
/// subdividing into `n` pieces divides that by `n²`. So
/// `n = ceil(sqrt(d / (4 * tolerance)))`, computed with an integer
/// square root and no floating point.
#[must_use]
pub fn quad_steps(from: &Point, control: &Point, to: &Point) -> usize {
    let mid_x = i64::from(from.x).saturating_add(i64::from(to.x)) / 2;
    let mid_y = i64::from(from.y).saturating_add(i64::from(to.y)) / 2;
    let dx = i64::from(control.x).saturating_sub(mid_x);
    let dy = i64::from(control.y).saturating_sub(mid_y);
    steps_for(dx.abs().saturating_add(dy.abs()))
}

/// How many line segments a cubic needs: the larger of the two
/// control points' deviations, by the same bound.
#[must_use]
pub fn cubic_steps(from: &Point, c0: &Point, c1: &Point, to: &Point) -> usize {
    let a = quad_steps(from, c0, to);
    let b = quad_steps(from, c1, to);
    a.max(b)
}

fn steps_for(deviation: i64) -> usize {
    // `deviation` is a Manhattan distance in subunits; the tolerance
    // is in subunits too, so the ratio is dimensionless.
    let scaled = deviation
        .saturating_mul(4)
        .checked_div(TOLERANCE)
        .unwrap_or(0);
    let n = isqrt(scaled).saturating_add(1);
    // A curve never becomes more pieces than its own tolerance can
    // justify; the cap keeps a pathological control point from
    // turning one segment into a million.
    let capped = n.clamp(1, 4_096);
    usize::try_from(capped).unwrap_or(1)
}

/// `floor(sqrt(n))` for a non-negative `n`.
fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut r = n;
    let mut last = -1_i64;
    for _ in 0..40 {
        if r == last || r == 0 {
            break;
        }
        last = r;
        r = r
            .saturating_add(n.checked_div(r).unwrap_or(0))
            .checked_div(2)
            .unwrap_or(0);
    }
    while r.saturating_mul(r) > n {
        r = r.saturating_sub(1);
    }
    r
}

fn point_at(x: i64, y: i64) -> Result<Point, FlattenError> {
    let x = i32::try_from(x).map_err(|_e| FlattenError::OutOfRange)?;
    let y = i32::try_from(y).map_err(|_e| FlattenError::OutOfRange)?;
    Ok(Point { x, y })
}

fn emit_quad(
    from: &Point,
    c: &Point,
    to: &Point,
    n: usize,
    out: &mut Vec<Point>,
) -> Result<(), FlattenError> {
    let nn = i64::try_from(n).unwrap_or(1).max(1);
    for i in 1..=n {
        let t = i64::try_from(i).unwrap_or(1);
        let u = nn.saturating_sub(t);
        // B(t) = (1-t)^2 P0 + 2(1-t)t C + t^2 P1, with t = i/n, all
        // multiplied through by n^2 and divided once at the end: one
        // rounding per point.
        let denominator = nn.saturating_mul(nn);
        let wx = u
            .saturating_mul(u)
            .saturating_mul(i64::from(from.x))
            .saturating_add(
                2_i64
                    .saturating_mul(u)
                    .saturating_mul(t)
                    .saturating_mul(i64::from(c.x)),
            )
            .saturating_add(t.saturating_mul(t).saturating_mul(i64::from(to.x)));
        let wy = u
            .saturating_mul(u)
            .saturating_mul(i64::from(from.y))
            .saturating_add(
                2_i64
                    .saturating_mul(u)
                    .saturating_mul(t)
                    .saturating_mul(i64::from(c.y)),
            )
            .saturating_add(t.saturating_mul(t).saturating_mul(i64::from(to.y)));
        out.push(point_at(
            round_div(wx, denominator),
            round_div(wy, denominator),
        )?);
    }
    Ok(())
}

fn emit_cubic(
    from: &Point,
    c0: &Point,
    c1: &Point,
    to: &Point,
    n: usize,
    out: &mut Vec<Point>,
) -> Result<(), FlattenError> {
    let nn = i64::try_from(n).unwrap_or(1).max(1);
    for i in 1..=n {
        let t = i64::try_from(i).unwrap_or(1);
        let u = nn.saturating_sub(t);
        let denominator = nn.saturating_mul(nn).saturating_mul(nn);
        let term = |p0: i32, p1: i32, p2: i32, p3: i32| -> i64 {
            u.saturating_mul(u)
                .saturating_mul(u)
                .saturating_mul(i64::from(p0))
                .saturating_add(
                    3_i64
                        .saturating_mul(u)
                        .saturating_mul(u)
                        .saturating_mul(t)
                        .saturating_mul(i64::from(p1)),
                )
                .saturating_add(
                    3_i64
                        .saturating_mul(u)
                        .saturating_mul(t)
                        .saturating_mul(t)
                        .saturating_mul(i64::from(p2)),
                )
                .saturating_add(
                    t.saturating_mul(t)
                        .saturating_mul(t)
                        .saturating_mul(i64::from(p3)),
                )
        };
        let wx = term(from.x, c0.x, c1.x, to.x);
        let wy = term(from.y, c0.y, c1.y, to.y);
        out.push(point_at(
            round_div(wx, denominator),
            round_div(wy, denominator),
        )?);
    }
    Ok(())
}

/// `round(a / b)` for a positive `b`, half away from zero: the one
/// rounding a flattened point carries.
fn round_div(a: i64, b: i64) -> i64 {
    if b == 0 {
        return 0;
    }
    let half = b / 2;
    if a >= 0 {
        a.saturating_add(half).checked_div(b).unwrap_or(0)
    } else {
        a.saturating_sub(half).checked_div(b).unwrap_or(0)
    }
}
