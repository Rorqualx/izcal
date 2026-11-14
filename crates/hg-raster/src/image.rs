//! Integer image sampling (§6.4's images row).
//!
//! > Texels fetched by integer loads, **never hardware filtering**:
//! > bilinear for magnification with 8-bit weights from fixed-point
//! > coordinates; exact area averaging for minification; `pixelated`
//! > and `crisp-edges` as specified.
//!
//! "Never hardware filtering" is the whole point. A GPU's bilinear
//! unit has vendor-defined precision — the weights are not specified
//! — so two devices filtering the same texture give different bytes,
//! and a page that reads those bytes back has a fingerprint. Doing it
//! with 8-bit weights in integers costs a few instructions and
//! removes the fingerprint.

use hg_display_list::display_list::{Colour, Sampling};

use crate::blend::{div255, mul255};

/// A premultiplied RGBA8 source image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image<'a> {
    /// Its width in texels.
    pub width: usize,
    /// Its height in texels.
    pub height: usize,
    /// Four bytes a texel, premultiplied.
    pub texels: &'a [u8],
}

impl Image<'_> {
    /// The texel at `(x, y)`, clamped to the edge, or transparent
    /// when the image has no texels at all.
    #[must_use]
    pub fn texel(&self, x: i64, y: i64) -> Colour {
        if self.width == 0 || self.height == 0 {
            return Colour {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            };
        }
        let w = i64::try_from(self.width).unwrap_or(1);
        let h = i64::try_from(self.height).unwrap_or(1);
        let cx = x.clamp(0, w.saturating_sub(1));
        let cy = y.clamp(0, h.saturating_sub(1));
        let at = usize::try_from(cy.saturating_mul(w).saturating_add(cx))
            .unwrap_or(0)
            .saturating_mul(4);
        let Some([red, green, blue, alpha]) =
            self.texels.get(at..).and_then(<[u8]>::first_chunk::<4>)
        else {
            return Colour {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            };
        };
        Colour {
            red: *red,
            green: *green,
            blue: *blue,
            alpha: *alpha,
        }
    }

    /// Samples at a point given in 1/256 texel units.
    ///
    /// `scale_num` over `scale_den` is how many source texels one
    /// destination pixel covers; at more than one it is a
    /// minification and the exact area average is used.
    #[must_use]
    pub fn sample(&self, x256: i64, y256: i64, sampling: Sampling, minifying: bool) -> Colour {
        match sampling {
            Sampling::Pixelated | Sampling::CrispEdges => {
                self.texel(x256.div_euclid(256), y256.div_euclid(256))
            }
            Sampling::Auto => {
                if minifying {
                    // The caller gives the box in `area` instead; a
                    // point sample under minification is the nearest
                    // texel, which `area` refines.
                    self.texel(x256.div_euclid(256), y256.div_euclid(256))
                } else {
                    self.bilinear(x256, y256)
                }
            }
        }
    }

    /// Bilinear with 8-bit weights.
    ///
    /// The sample point is at the texel's centre, so the weights come
    /// from the fractional part of `point - 1/2` in texels, which is
    /// the convention every specification that defines one uses and
    /// the one that keeps an image drawn at its own size identical to
    /// its texels.
    #[must_use]
    pub fn bilinear(&self, x256: i64, y256: i64) -> Colour {
        let sx = x256.saturating_sub(128);
        let sy = y256.saturating_sub(128);
        let x0 = sx.div_euclid(256);
        let y0 = sy.div_euclid(256);
        let fx = sx.rem_euclid(256);
        let fy = sy.rem_euclid(256);
        // 8-bit weights: 256 is one, so the pair sums to 256 exactly.
        let wx1 = u32::try_from(fx).unwrap_or(0);
        let wx0 = 256_u32.saturating_sub(wx1);
        let wy1 = u32::try_from(fy).unwrap_or(0);
        let wy0 = 256_u32.saturating_sub(wy1);

        let c00 = self.texel(x0, y0);
        let c10 = self.texel(x0.saturating_add(1), y0);
        let c01 = self.texel(x0, y0.saturating_add(1));
        let c11 = self.texel(x0.saturating_add(1), y0.saturating_add(1));

        let mix = |a: u8, b: u8, c: u8, d: u8| -> u8 {
            let top = u32::from(a)
                .wrapping_mul(wx0)
                .wrapping_add(u32::from(b).wrapping_mul(wx1));
            let bottom = u32::from(c)
                .wrapping_mul(wx0)
                .wrapping_add(u32::from(d).wrapping_mul(wx1));
            // One rounding at the end: the two axes' weights multiply
            // to 65,536, so the sum is divided once.
            let total = top.wrapping_mul(wy0).wrapping_add(bottom.wrapping_mul(wy1));
            let v = total.wrapping_add(32_768) >> 16;
            u8::try_from(v.min(255)).unwrap_or(255)
        };
        Colour {
            red: mix(c00.red, c10.red, c01.red, c11.red),
            green: mix(c00.green, c10.green, c01.green, c11.green),
            blue: mix(c00.blue, c10.blue, c01.blue, c11.blue),
            alpha: mix(c00.alpha, c10.alpha, c01.alpha, c11.alpha),
        }
    }

    /// The exact area average of the texels under a box, for
    /// minification.
    ///
    /// Each texel is weighted by how much of it the box covers, in
    /// 1/256 units per axis, and the whole sum is divided once. An
    /// implementation that point-sampled instead would alias, and one
    /// that used a mip chain would give a different answer per device
    /// because the chain's construction is not specified.
    #[must_use]
    pub fn area(&self, x0_256: i64, y0_256: i64, x1_256: i64, y1_256: i64) -> Colour {
        let (left, right) = (x0_256.min(x1_256), x0_256.max(x1_256));
        let (top, bottom) = (y0_256.min(y1_256), y0_256.max(y1_256));
        if left == right || top == bottom {
            return self.texel(left.div_euclid(256), top.div_euclid(256));
        }
        let mut acc = [0_i64; 4];
        let mut weight_total = 0_i64;
        let first_x = left.div_euclid(256);
        let last_x = right.saturating_sub(1).div_euclid(256);
        let first_y = top.div_euclid(256);
        let last_y = bottom.saturating_sub(1).div_euclid(256);
        let mut ty = first_y;
        while ty <= last_y {
            let row_top = ty.saturating_mul(256).max(top);
            let row_bottom = ty.saturating_add(1).saturating_mul(256).min(bottom);
            let wy = row_bottom.saturating_sub(row_top);
            if wy > 0 {
                let mut tx = first_x;
                while tx <= last_x {
                    let col_left = tx.saturating_mul(256).max(left);
                    let col_right = tx.saturating_add(1).saturating_mul(256).min(right);
                    let wx = col_right.saturating_sub(col_left);
                    if wx > 0 {
                        let w = wx.saturating_mul(wy);
                        let c = self.texel(tx, ty);
                        acc = [
                            acc[0].saturating_add(i64::from(c.red).saturating_mul(w)),
                            acc[1].saturating_add(i64::from(c.green).saturating_mul(w)),
                            acc[2].saturating_add(i64::from(c.blue).saturating_mul(w)),
                            acc[3].saturating_add(i64::from(c.alpha).saturating_mul(w)),
                        ];
                        weight_total = weight_total.saturating_add(w);
                    }
                    tx = tx.saturating_add(1);
                }
            }
            ty = ty.saturating_add(1);
        }
        if weight_total == 0 {
            return self.texel(first_x, first_y);
        }
        let half = weight_total / 2;
        let one = |v: i64| -> u8 {
            let q = v
                .saturating_add(half)
                .checked_div(weight_total)
                .unwrap_or(0);
            u8::try_from(q.clamp(0, 255)).unwrap_or(255)
        };
        Colour {
            red: one(acc[0]),
            green: one(acc[1]),
            blue: one(acc[2]),
            alpha: one(acc[3]),
        }
    }
}

/// A convenience: `src` scaled by `coverage` and composited onto
/// `dst`, which is what filling a mask with a colour does.
#[must_use]
pub fn fill_pixel(src: Colour, coverage: u8, dst: Colour) -> Colour {
    let _ = div255(0);
    let _ = mul255(0, 0);
    crate::blend::over(crate::blend::scale(src, coverage), dst)
}
