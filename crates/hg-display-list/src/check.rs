//! What the wire format cannot say.
//!
//! The generated reader enforces every bound the IDL names — 65,536
//! segments, 4,096 items — because those are counts and the format
//! has a word for a count. Four things it cannot:
//!
//! - a **premultiplied** colour whose component is above its alpha,
//!   which is not a colour at all;
//! - a rectangle with a **negative** width or height;
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

use crate::display_list::{Chunk, Colour, DisplayListError, Item, Rect};

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
            Item::FillPath { colour, .. } => {
                if !is_premultiplied(colour) {
                    return Err(DisplayListError::NotPremultiplied);
                }
            }
            Item::DrawImage { source, dest, .. } => {
                if !has_extent(source) || !has_extent(dest) {
                    return Err(DisplayListError::NegativeExtent);
                }
            }
            Item::PushGroup { .. } => {
                depth = depth.saturating_add(1);
                if depth > MAX_GROUP_DEPTH {
                    return Err(DisplayListError::TooManyItems);
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
