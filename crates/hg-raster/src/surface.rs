//! A premultiplied RGBA8 surface, and an 8-bit coverage mask.

use hg_display_list::display_list::Colour;

/// A premultiplied RGBA8 image, row-major, no padding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    width: usize,
    height: usize,
    /// Four bytes a pixel: red, green, blue, alpha, premultiplied.
    pixels: Vec<u8>,
}

impl Surface {
    /// A transparent surface.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let n = width.saturating_mul(height).saturating_mul(4);
        Self {
            width,
            height,
            pixels: vec![0; n],
        }
    }

    /// Its width in pixels.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Its height in pixels.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Its bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.pixels
    }

    /// The pixel at `(x, y)`, or `None` when it is outside.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<Colour> {
        let at = self.offset(x, y)?;
        let [red, green, blue, alpha] = *self.pixels.get(at..)?.first_chunk::<4>()?;
        Some(Colour {
            red,
            green,
            blue,
            alpha,
        })
    }

    /// Writes the pixel at `(x, y)`. Outside the surface it does
    /// nothing, which is what a clipped draw wants.
    pub fn set(&mut self, x: usize, y: usize, c: Colour) {
        let Some(at) = self.offset(x, y) else { return };
        let Some(slot) = self
            .pixels
            .get_mut(at..)
            .and_then(<[u8]>::first_chunk_mut::<4>)
        else {
            return;
        };
        *slot = [c.red, c.green, c.blue, c.alpha];
    }

    fn offset(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        y.checked_mul(self.width)?.checked_add(x)?.checked_mul(4)
    }
}
