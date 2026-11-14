//! What the wire format cannot say.
//!
//! The generated reader enforces every bound the IDL names — 65,536
//! segments, 4,096 items — because those are counts and the format
//! has a word for a count. Six things it cannot:
//!
//! - a **premultiplied** colour whose component is above its alpha,
//!   which is not a colour at all;
//! - a rectangle with a **negative** width or height;
//! - a source rectangle **outside the image** it names, which the
//!   format states the extent of but cannot relate to the rectangle;
//! - an arc **radius below zero**, which no ellipse has;
//! - `PopGroup` with no `PushGroup`, or a chunk that ends inside a
//!   group;
//! - a group nested deeper than the compositor will hold.
//!
//! Each is a value the type admits and the design does not, and each
//! would otherwise be found by whatever reads it first — which in a
//! content GPU process means found by the rasterizer, at a point where
//! the only thing it can do is stop.
//!
//! This is the one hand-written module in the crate.

use crate::display_list::{Chunk, Colour, DisplayListError, ImageRef, Item, Rect, Segment};

/// How deep groups may nest. Each open group is a buffer the
/// compositor holds, so the depth is memory a renderer chooses.
pub const MAX_GROUP_DEPTH: usize = 64;

/// Whether `c` is a colour premultiplied alpha can represent.
#[must_use]
pub const fn is_premultiplied(c: &Colour) -> bool {
    c.red <= c.alpha && c.green <= c.alpha && c.blue <= c.alpha
}

/// Whether `r` has non-negative extents.
#[must_use]
pub const fn has_extent(r: &Rect) -> bool {
    r.width >= 0 && r.height >= 0
}

/// Whether `r` lies inside `image`.
///
/// The format states the image's extent and the rectangle's, and
/// cannot say that one contains the other. Unchecked, a renderer that
/// the content GPU process does not trust names a rectangle past the
/// edge of a texture, and whatever the sampler then reads is memory
/// the renderer did not put there. Assumes `has_extent(r)`.
#[must_use]
pub fn is_inside(r: &Rect, image: &ImageRef) -> bool {
    // In `i64`, where an `i32` edge plus an `i32` extent cannot
    // overflow -- the arithmetic is still checked, because a bound
    // that holds only by the author's reasoning is the bound that
    // stops holding when the field type changes.
    let (w, h) = (i64::from(image.width), i64::from(image.height));
    let (x, y) = (i64::from(r.x), i64::from(r.y));
    let (Some(right), Some(bottom)) = (
        x.checked_add(i64::from(r.width)),
        y.checked_add(i64::from(r.height)),
    ) else {
        return false;
    };
    x >= 0 && y >= 0 && right <= w && bottom <= h
}

/// Whether every arc in `segments` has non-negative radii.
#[must_use]
pub fn radii_are_positive(segments: &[Segment]) -> bool {
    segments.iter().all(|s| match s {
        Segment::Arc { radii, .. } => radii.x >= 0 && radii.y >= 0,
        Segment::Line { .. } | Segment::Quad { .. } | Segment::Cubic { .. } => true,
    })
}

/// Checks a chunk.
///
/// # Errors
/// The first problem found, as a [`DisplayListError`]. The variants
/// name limits the format itself states, so a refusal tells a renderer
/// nothing it did not know when it built the list.
pub fn validate(chunk: &Chunk) -> Result<(), DisplayListError> {
    let mut depth = 0_usize;
    for item in chunk.items.as_slice() {
        match item {
            Item::FillPath { path, colour } => {
                if !is_premultiplied(colour) {
                    return Err(DisplayListError::NotPremultiplied);
                }
                if !radii_are_positive(path.segments.as_slice()) {
                    return Err(DisplayListError::NegativeRadius);
                }
            }
            Item::DrawImage {
                image,
                source,
                dest,
                ..
            } => {
                if !has_extent(source) || !has_extent(dest) {
                    return Err(DisplayListError::NegativeExtent);
                }
                if !is_inside(source, image) {
                    return Err(DisplayListError::SourceOutsideImage);
                }
            }
            Item::PushGroup { .. } => {
                depth = depth.saturating_add(1);
                if depth > MAX_GROUP_DEPTH {
                    return Err(DisplayListError::GroupTooDeep);
                }
            }
            Item::PopGroup => {
                depth = match depth.checked_sub(1) {
                    Some(d) => d,
                    None => return Err(DisplayListError::UnbalancedGroup),
                };
            }
        }
    }
    if depth != 0 {
        return Err(DisplayListError::UnbalancedGroup);
    }
    Ok(())
}
