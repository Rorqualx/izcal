//! Premultiplied integer compositing, and the separable blend modes.
//!
//! # The exact ÷255
//!
//! Compositing is written with products of 8-bit values divided by
//! 255, and a division per channel per pixel is the hottest
//! instruction in a rasterizer. The identity used here is
//!
//! ```text
//! div255(x) = (x + 128 + ((x + 128) >> 8)) >> 8
//! ```
//!
//! which §6.4 calls "exact ÷255 by the add-shift identity". *Exact*
//! means it equals `round(x / 255)` for every `x` a product of two
//! 8-bit values can be, and the crate's test checks that over the
//! whole range rather than at a few points — because "approximately
//! 255" is how a rasterizer ends up one off on one device and not
//! another, which is the whole of what GR-D3 forbids.
//!
//! # Premultiplied, and why
//!
//! A premultiplied colour is one whose components are already
//! multiplied by its alpha. Source-over is then
//! `dst = src + dst * (255 - src_a) / 255`, with one multiply and one
//! ÷255 per channel; straight alpha needs a divide to recover the
//! colour and a multiply to put it back, which is two roundings where
//! this has one. §6.5 makes premultiplied the page working encoding
//! for that reason, and [`hg_display_list::check::is_premultiplied`]
//! refuses a colour that is not one.

use hg_display_list::display_list::{BlendMode, Colour};

/// `round(x / 255)`, exactly, for every `x` up to 65,535.
///
/// # Panics
/// Never: every operation is on `u32` and the largest input a product
/// of two bytes can give is 65,025.
#[must_use]
pub const fn div255(x: u32) -> u32 {
    let t = x.wrapping_add(128);
    t.wrapping_add(t >> 8) >> 8
}

/// `round(a * b / 255)` for two 8-bit values.
#[must_use]
pub fn mul255(a: u8, b: u8) -> u8 {
    let product = u32::from(a).wrapping_mul(u32::from(b));
    // `div255` of a product of two bytes is at most 255; the
    // conversion cannot fail, and is written as a conversion rather
    // than a cast so that nothing here is a silent truncation.
    u8::try_from(div255(product)).unwrap_or(255)
}

/// Source-over: `src` composited onto `dst`, both premultiplied.
///
/// The one compositing operation every other mode is expressed
/// through (Compositing 1 §9: a blend mode changes the colour that is
/// then composited, not the composition).
#[must_use]
pub fn over(src: Colour, dst: Colour) -> Colour {
    let inv = 255_u8.saturating_sub(src.alpha);
    Colour {
        red: src.red.saturating_add(mul255(dst.red, inv)),
        green: src.green.saturating_add(mul255(dst.green, inv)),
        blue: src.blue.saturating_add(mul255(dst.blue, inv)),
        alpha: src.alpha.saturating_add(mul255(dst.alpha, inv)),
    }
}

/// Scales a premultiplied colour by an 8-bit coverage or opacity.
///
/// Every component including alpha, which is what keeps the result
/// premultiplied: scaling the colour and not the alpha would make a
/// colour above its own alpha, which §6.5 does not admit.
#[must_use]
pub fn scale(c: Colour, by: u8) -> Colour {
    Colour {
        red: mul255(c.red, by),
        green: mul255(c.green, by),
        blue: mul255(c.blue, by),
        alpha: mul255(c.alpha, by),
    }
}

/// Un-premultiplies one channel for a blend-mode formula, which
/// Compositing 1 writes on unpremultiplied values.
///
/// Returns 0 when alpha is 0, where the colour is not defined and
/// every mode's result is the backdrop.
#[must_use]
pub fn unpremultiply(channel: u8, alpha: u8) -> u8 {
    if alpha == 0 {
        return 0;
    }
    let scaled = u32::from(channel).wrapping_mul(255);
    // `channel <= alpha` (the colour is premultiplied), so the
    // quotient is at most 255; a colour that is not premultiplied is
    // clamped rather than wrapped, and `hg-display-list`'s validator
    // is where it is refused.
    let q = scaled.checked_div(u32::from(alpha)).unwrap_or(255);
    u8::try_from(q.min(255)).unwrap_or(255)
}

/// One separable blend mode's formula, on unpremultiplied 8-bit
/// channels (Compositing 1 §9.1 and §9.2).
///
/// # Errors
/// `None` for a non-separable mode (`hue`, `saturation`, `color`,
/// `luminosity`), which is defined on all three channels at once and
/// is not this function's shape. They are GR-M1's.
#[must_use]
pub fn blend_channel(mode: BlendMode, source: u8, backdrop: u8) -> Option<u8> {
    let cs = u32::from(source);
    let cb = u32::from(backdrop);
    let v = match mode {
        BlendMode::Normal => cs,
        BlendMode::Multiply => div255(cs.wrapping_mul(cb)),
        // `cb + cs - cb*cs`, in 8-bit terms.
        BlendMode::Screen => cb
            .wrapping_add(cs)
            .saturating_sub(div255(cs.wrapping_mul(cb))),
        BlendMode::Overlay => return blend_channel(BlendMode::HardLight, backdrop, source),
        BlendMode::Darken => cs.min(cb),
        BlendMode::Lighten => cs.max(cb),
        BlendMode::ColorDodge => {
            if cb == 0 {
                0
            } else if cs == 255 {
                255
            } else {
                // min(255, backdrop / (1 - source))
                let denominator = 255_u32.saturating_sub(cs);
                let q = cb.wrapping_mul(255).checked_div(denominator).unwrap_or(255);
                q.min(255)
            }
        }
        BlendMode::ColorBurn => {
            if cb == 255 {
                255
            } else if cs == 0 {
                0
            } else {
                let numerator = 255_u32.saturating_sub(cb).wrapping_mul(255);
                let q = numerator.checked_div(cs).unwrap_or(255).min(255);
                255_u32.saturating_sub(q)
            }
        }
        BlendMode::HardLight => {
            if cs <= 127 {
                // multiply(backdrop, 2 * source)
                div255(cb.wrapping_mul(cs.wrapping_mul(2)))
            } else {
                // screen(backdrop, 2 * source - 255)
                let s = cs.wrapping_mul(2).saturating_sub(255);
                cb.wrapping_add(s)
                    .saturating_sub(div255(s.wrapping_mul(cb)))
            }
        }
        BlendMode::SoftLight => soft_light(cs, cb),
        BlendMode::Difference => cs.abs_diff(cb),
        BlendMode::Exclusion => cs
            .wrapping_add(cb)
            .saturating_sub(div255(cs.wrapping_mul(cb)).wrapping_mul(2)),
        BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => {
            return None;
        }
    };
    // Every arm is already inside 0..=255; the clamp is the one place
    // that says so rather than each arm repeating it.
    Some(u8::try_from(v.min(255)).unwrap_or(255))
}

/// Compositing 1 §9.1's `soft-light`, in integers.
///
/// The specification's formula has a square root in one branch. The
/// integer form here uses the same branch structure and a 16-bit
/// fixed-point square root, so the result is one specified rounding
/// rather than a float's.
fn soft_light(cs: u32, cb: u32) -> u32 {
    // Work in 1/255 units throughout.
    if cs <= 127 {
        // cb - (1 - 2*cs) * cb * (1 - cb)
        let one_minus_2cs = 255_u32.saturating_sub(cs.wrapping_mul(2));
        let cb_1mcb = div255(cb.wrapping_mul(255_u32.saturating_sub(cb)));
        cb.saturating_sub(div255(one_minus_2cs.wrapping_mul(cb_1mcb)))
    } else {
        let d = if cb <= 63 {
            // ((16*cb - 12) * cb + 4) * cb, all in 1/255 units
            let t = div255(cb.wrapping_mul(16).saturating_sub(12_u32.wrapping_mul(255)));
            let t = div255(t.wrapping_mul(cb)).wrapping_add(4_u32.wrapping_mul(255) / 255);
            div255(t.wrapping_mul(cb))
        } else {
            isqrt255(cb)
        };
        // cb + (2*cs - 1) * (d - cb)
        let two_cs_minus_one = cs.wrapping_mul(2).saturating_sub(255);
        let delta = div255(two_cs_minus_one.wrapping_mul(d.abs_diff(cb)));
        if d >= cb {
            cb.saturating_add(delta)
        } else {
            cb.saturating_sub(delta)
        }
    }
}

/// `round(255 * sqrt(x / 255))` for `x` in 0..=255, by integer
/// Newton iteration: `sqrt(255 * x)`.
fn isqrt255(x: u32) -> u32 {
    let n = x.wrapping_mul(255);
    if n == 0 {
        return 0;
    }
    let mut r = n;
    let mut last = 0_u32;
    // Converges in a handful of steps for `n` under 65,025; the bound
    // is a loop limit rather than a convergence test so the function
    // is total.
    for _ in 0..24_u32 {
        if r == 0 || r == last {
            break;
        }
        last = r;
        r = r
            .wrapping_add(n.checked_div(r).unwrap_or(0))
            .wrapping_div(2);
    }
    // One rounding at the end: `r` is the floor, so round up when the
    // next square is closer.
    let next = r.saturating_add(1);
    if next.wrapping_mul(next) <= n {
        next
    } else {
        r
    }
}
