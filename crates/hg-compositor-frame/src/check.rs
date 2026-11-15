//! What the wire format cannot say.
//!
//! The generated reader enforces the ten counts the IDL names --
//! 1,024 transform, clip and effect nodes, 256 scroll nodes, 256
//! layers, 4,096 tiles, 64 embeds, 16 animations, 64 keyframes, 256
//! damage rectangles -- because those are counts and the format has a
//! word for a count. Seventeen things it cannot:
//!
//! - a property-tree parent **outside its own tree**;
//! - a parent at or after its child, which would make the nodes
//!   something other than a forest in topological order;
//! - a layer, embed or animation naming a **node that does not
//!   exist**;
//! - more **compositor-mutable nodes** than the per-document cap,
//!   which is a derived count over three different vectors and so has
//!   no field to live in;
//! - a rectangle with a **negative** width or height;
//! - a tile **outside its own grid**, or **two tiles in one cell**;
//! - a grid whose tile width or height is **zero**, which makes the
//!   grid's extent meaningless;
//! - a corner **radius below zero**, which no corner has;
//! - radii **wider than the rectangle** they round;
//! - a **device scale** whose denominator is zero or above 2^16;
//! - **damage outside the viewport** it is damage in;
//! - keyframe offsets that are **not ascending** or run past the end;
//! - **fewer than two keyframes**, which is not an interpolation;
//! - a `Steps` function with **zero steps**;
//! - an animation whose **target is in the wrong tree** for its
//!   property, which the index alone cannot say because the trees are
//!   separate vectors;
//! - a keyframe whose **value is not of its animation's property**,
//!   which the union admits and the animation does not.
//!
//! Each is a value the type admits and the design does not. The
//! compositor holds every site's composited pixels and the content GPU
//! process that sends it a frame is untrusted (3.6's trust picture),
//! so each would otherwise be found by whatever reads it first -- the
//! aggregator or the sampler -- at a point where the only thing it can
//! do is stop.

use crate::compositor_frame::{
    AnimatedProperty, Animation, ClipNode, ClipShape, CompositorFrameError, Corners, EffectNode,
    Embed, Frame, KeyframeValue, Layer, Rect, ScrollNode, Tile, TimingFunction, TransformNode,
};

/// A parent index meaning "this node is a root".
pub const NO_PARENT: u32 = u32::MAX;

/// A node reference meaning "none".
pub const NO_NODE: u32 = u32::MAX;

/// How many nodes of a document the compositor may change.
///
/// §6.3: a compositor-scrolled scroll node, a compositor-animated
/// transform or opacity, and a video, canvas or iframe surface are
/// each compositor-mutable, and layerization caps them at 16† per
/// document, demoting the lowest-priority ones past the cap. The
/// content GPU process applies that policy; the compositor checks the
/// bound, because a cap enforced only by the sender is not a cap.
pub const MAX_MUTABLE_NODES: usize = 16;

/// The largest keyframe offset: the end of the animation.
pub const OFFSET_END: u32 = 65_536;

/// The largest device-scale denominator (6.5: an exact rational p/q
/// with q <= 2^16).
pub const MAX_SCALE_DEN: u32 = 65_536;

/// Corner radii are in 1/256 of a device pixel; bounds are in whole
/// device pixels.
const RADIUS_UNITS_PER_PX: i64 = 256;

/// Whether `r` has non-negative extents.
#[must_use]
pub const fn has_extent(r: &Rect) -> bool {
    r.width >= 0 && r.height >= 0
}

/// Whether `idx` names an element of a vector of `len` elements.
#[must_use]
pub fn names_element(idx: u32, len: usize) -> bool {
    match usize::try_from(idx) {
        Ok(i) => i < len,
        Err(_) => false,
    }
}

/// Whether `idx` is `NO_NODE` or names an element.
#[must_use]
pub fn optional_element(idx: u32, len: usize) -> bool {
    idx == NO_NODE || names_element(idx, len)
}

/// Whether `parent` is a root, or an **earlier** index than `child`.
///
/// Earlier rather than merely different, so that the nodes are a
/// forest in topological order. A cycle is then not expressible
/// rather than expressible and invalid, which is why there is no
/// cycle variant in [`CompositorFrameError`].
#[must_use]
pub fn parent_is_earlier(parent: u32, child: usize) -> bool {
    if parent == NO_PARENT {
        return true;
    }
    match usize::try_from(parent) {
        Ok(p) => p < child,
        Err(_) => false,
    }
}

/// Whether every radius of `c` is non-negative.
#[must_use]
pub const fn radii_are_positive(c: &Corners) -> bool {
    c.top_left_x >= 0
        && c.top_left_y >= 0
        && c.top_right_x >= 0
        && c.top_right_y >= 0
        && c.bottom_right_x >= 0
        && c.bottom_right_y >= 0
        && c.bottom_left_x >= 0
        && c.bottom_left_y >= 0
}

/// Whether `a + b` fits inside `extent` whole pixels.
///
/// In `i64`, where two `i32` radii and an `i32` extent scaled by 256
/// cannot overflow -- and still checked, because a bound that holds
/// only by the author's reasoning is the bound that stops holding
/// when a field type changes.
fn pair_fits(a: i32, b: i32, extent: i32) -> bool {
    let (Some(sum), Some(room)) = (
        i64::from(a).checked_add(i64::from(b)),
        i64::from(extent).checked_mul(RADIUS_UNITS_PER_PX),
    ) else {
        return false;
    };
    sum <= room
}

/// Whether `c`'s radii fit inside `bounds`.
///
/// CSS scales radii that do not fit. The compositor refuses them,
/// because scaling is a decision and the compositor makes none: every
/// decision about page pixels belongs to the content GPU process, so
/// that the composed screen equals the canonical image (6.4).
#[must_use]
pub fn radii_fit(c: &Corners, bounds: &Rect) -> bool {
    pair_fits(c.top_left_x, c.top_right_x, bounds.width)
        && pair_fits(c.bottom_left_x, c.bottom_right_x, bounds.width)
        && pair_fits(c.top_left_y, c.bottom_left_y, bounds.height)
        && pair_fits(c.top_right_y, c.bottom_right_y, bounds.height)
}

/// Whether `inner` lies inside `outer`. Assumes both have extent.
#[must_use]
pub fn is_inside(inner: &Rect, outer: &Rect) -> bool {
    let (Some(ir), Some(ib), Some(or), Some(ob)) = (
        i64::from(inner.x).checked_add(i64::from(inner.width)),
        i64::from(inner.y).checked_add(i64::from(inner.height)),
        i64::from(outer.x).checked_add(i64::from(outer.width)),
        i64::from(outer.y).checked_add(i64::from(outer.height)),
    ) else {
        return false;
    };
    i64::from(inner.x) >= i64::from(outer.x)
        && i64::from(inner.y) >= i64::from(outer.y)
        && ir <= or
        && ib <= ob
}

/// The cell a tile claims, as one key, or `None` if it is outside the
/// grid.
fn cell_key(t: &Tile, columns: u32, rows: u32) -> Option<u64> {
    if t.column >= columns || t.row >= rows {
        return None;
    }
    u64::from(t.row)
        .checked_mul(u64::from(columns))?
        .checked_add(u64::from(t.column))
}

fn check_transform_nodes(nodes: &[TransformNode]) -> Result<(), CompositorFrameError> {
    for (i, n) in nodes.iter().enumerate() {
        if !names_element(n.parent, nodes.len()) && n.parent != NO_PARENT {
            return Err(CompositorFrameError::ParentOutOfRange);
        }
        if !parent_is_earlier(n.parent, i) {
            return Err(CompositorFrameError::ParentNotEarlier);
        }
    }
    Ok(())
}

fn check_clip_nodes(nodes: &[ClipNode], transforms: usize) -> Result<(), CompositorFrameError> {
    for (i, n) in nodes.iter().enumerate() {
        if !names_element(n.parent, nodes.len()) && n.parent != NO_PARENT {
            return Err(CompositorFrameError::ParentOutOfRange);
        }
        if !parent_is_earlier(n.parent, i) {
            return Err(CompositorFrameError::ParentNotEarlier);
        }
        if !names_element(n.transform, transforms) {
            return Err(CompositorFrameError::NodeOutOfRange);
        }
        match &n.shape {
            ClipShape::Rectangular { bounds } => {
                if !has_extent(bounds) {
                    return Err(CompositorFrameError::NegativeExtent);
                }
            }
            ClipShape::Rounded { bounds, radii } => {
                if !has_extent(bounds) {
                    return Err(CompositorFrameError::NegativeExtent);
                }
                if !radii_are_positive(radii) {
                    return Err(CompositorFrameError::NegativeRadius);
                }
                if !radii_fit(radii, bounds) {
                    return Err(CompositorFrameError::RadiiExceedBounds);
                }
            }
        }
    }
    Ok(())
}

fn check_effect_nodes(nodes: &[EffectNode]) -> Result<(), CompositorFrameError> {
    for (i, n) in nodes.iter().enumerate() {
        if !names_element(n.parent, nodes.len()) && n.parent != NO_PARENT {
            return Err(CompositorFrameError::ParentOutOfRange);
        }
        if !parent_is_earlier(n.parent, i) {
            return Err(CompositorFrameError::ParentNotEarlier);
        }
    }
    Ok(())
}

fn check_scroll_nodes(nodes: &[ScrollNode], transforms: usize) -> Result<(), CompositorFrameError> {
    for (i, n) in nodes.iter().enumerate() {
        if !names_element(n.parent, nodes.len()) && n.parent != NO_PARENT {
            return Err(CompositorFrameError::ParentOutOfRange);
        }
        if !parent_is_earlier(n.parent, i) {
            return Err(CompositorFrameError::ParentNotEarlier);
        }
        if !names_element(n.transform, transforms) {
            return Err(CompositorFrameError::NodeOutOfRange);
        }
        if !has_extent(&n.container) || !has_extent(&n.content) {
            return Err(CompositorFrameError::NegativeExtent);
        }
    }
    Ok(())
}

fn check_layer(layer: &Layer, f: &Frame) -> Result<(), CompositorFrameError> {
    if layer.tile_width == 0 || layer.tile_height == 0 {
        return Err(CompositorFrameError::ZeroTileExtent);
    }
    if !names_element(layer.transform, f.transforms.as_slice().len()) {
        return Err(CompositorFrameError::NodeOutOfRange);
    }
    if !optional_element(layer.clip, f.clips.as_slice().len())
        || !optional_element(layer.effect, f.effects.as_slice().len())
        || !optional_element(layer.scroll, f.scrolls.as_slice().len())
    {
        return Err(CompositorFrameError::NodeOutOfRange);
    }
    let mut keys = Vec::with_capacity(layer.tiles.as_slice().len());
    for t in layer.tiles.as_slice() {
        match cell_key(t, layer.columns, layer.rows) {
            Some(k) => keys.push(k),
            None => return Err(CompositorFrameError::TileOutsideGrid),
        }
    }
    keys.sort_unstable();
    if keys.windows(2).any(|w| w.first() == w.last()) {
        return Err(CompositorFrameError::DuplicateTile);
    }
    Ok(())
}

fn check_animation(a: &Animation, f: &Frame) -> Result<(), CompositorFrameError> {
    // The target is an index into the tree its property lives in, and
    // the two trees are different vectors. Accepting an index that
    // names an element of *either* would make the index spaces
    // interchangeable to a sender, so a `Transform` animation could
    // drive an effect node of the same index and the compositor would
    // animate the wrong thing.
    let targets = match a.property {
        AnimatedProperty::Transform => f.transforms.as_slice().len(),
        AnimatedProperty::Opacity => f.effects.as_slice().len(),
    };
    if !names_element(a.target, targets) {
        return Err(CompositorFrameError::TargetWrongKind);
    }
    if !has_extent(&a.bounds) {
        return Err(CompositorFrameError::NegativeExtent);
    }
    let frames = a.keyframes.as_slice();
    if frames.len() < 2 {
        return Err(CompositorFrameError::TooFewKeyframes);
    }
    let mut previous: Option<u32> = None;
    for k in frames {
        if k.offset > OFFSET_END {
            return Err(CompositorFrameError::KeyframesUnordered);
        }
        if previous.is_some_and(|p| k.offset <= p) {
            return Err(CompositorFrameError::KeyframesUnordered);
        }
        previous = Some(k.offset);
        // The union admits both values; the animation means one.
        let value_matches = match (&a.property, &k.value) {
            (AnimatedProperty::Transform, KeyframeValue::Transform { .. })
            | (AnimatedProperty::Opacity, KeyframeValue::Opacity { .. }) => true,
            (AnimatedProperty::Transform, KeyframeValue::Opacity { .. })
            | (AnimatedProperty::Opacity, KeyframeValue::Transform { .. }) => false,
        };
        if !value_matches {
            return Err(CompositorFrameError::KeyframeWrongProperty);
        }
        match &k.easing {
            TimingFunction::Steps { count, .. } => {
                if *count == 0 {
                    return Err(CompositorFrameError::ZeroSteps);
                }
            }
            TimingFunction::Linear | TimingFunction::CubicBezier { .. } => {}
        }
    }
    Ok(())
}

fn check_embed(e: &Embed, f: &Frame) -> Result<(), CompositorFrameError> {
    if !names_element(e.transform, f.transforms.as_slice().len()) {
        return Err(CompositorFrameError::NodeOutOfRange);
    }
    // Never optional: an embed without a clip is the failure this
    // field exists to prevent (LY-15 holds even against a compromised
    // child only because the parent's clip is enforced at the embed).
    if !names_element(e.clip, f.clips.as_slice().len()) {
        return Err(CompositorFrameError::NodeOutOfRange);
    }
    if !has_extent(&e.bounds) {
        return Err(CompositorFrameError::NegativeExtent);
    }
    Ok(())
}

/// Which tree a mutable node belongs to. The index alone does not
/// identify a node: transform 0, effect 0 and scroll 0 are three
/// different nodes, and deduplicating them together would let a
/// sender exceed [`MAX_MUTABLE_NODES`] while appearing not to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tree {
    Transform,
    Effect,
    Scroll,
}

/// How many nodes of `f` the compositor may change.
///
/// The three kinds of §6.3's definition, counted as distinct nodes:
/// compositor-animated transform and opacity nodes, compositor-scrolled
/// scroll nodes, and embedded surfaces.
///
/// Each node is keyed by **(tree, index)** rather than by index. Two
/// animations of the same node are one mutable node; an animation of
/// transform 0 and a scrollable scroll node 0 are two.
#[must_use]
pub fn mutable_node_count(f: &Frame) -> usize {
    let mut nodes: Vec<(Tree, u32)> = f
        .animations
        .as_slice()
        .iter()
        .map(|a| {
            let tree = match a.property {
                AnimatedProperty::Transform => Tree::Transform,
                AnimatedProperty::Opacity => Tree::Effect,
            };
            (tree, a.target)
        })
        .chain(
            f.scrolls
                .as_slice()
                .iter()
                .enumerate()
                .filter(|(_, s)| s.user_scrollable_x || s.user_scrollable_y)
                .filter_map(|(i, _)| u32::try_from(i).ok().map(|i| (Tree::Scroll, i))),
        )
        .collect();
    nodes.sort_unstable();
    nodes.dedup();
    // Each embed is its own surface, so each is its own mutable node.
    nodes.len().saturating_add(f.embeds.as_slice().len())
}

/// Checks a compositor frame.
///
/// # Errors
/// The first problem found, as a [`CompositorFrameError`]. The
/// variants name limits the format itself states, so a refusal tells
/// a content GPU process nothing it did not know when it built the
/// frame.
pub fn validate(f: &Frame) -> Result<(), CompositorFrameError> {
    if f.device_scale_den == 0 || f.device_scale_den > MAX_SCALE_DEN {
        return Err(CompositorFrameError::BadDeviceScale);
    }
    if !has_extent(&f.viewport) {
        return Err(CompositorFrameError::NegativeExtent);
    }
    check_transform_nodes(f.transforms.as_slice())?;
    check_clip_nodes(f.clips.as_slice(), f.transforms.as_slice().len())?;
    check_effect_nodes(f.effects.as_slice())?;
    check_scroll_nodes(f.scrolls.as_slice(), f.transforms.as_slice().len())?;
    for layer in f.layers.as_slice() {
        check_layer(layer, f)?;
    }
    for e in f.embeds.as_slice() {
        check_embed(e, f)?;
    }
    for a in f.animations.as_slice() {
        check_animation(a, f)?;
    }
    for d in f.damage.as_slice() {
        if !has_extent(d) {
            return Err(CompositorFrameError::NegativeExtent);
        }
        if !is_inside(d, &f.viewport) {
            return Err(CompositorFrameError::DamageOutsideViewport);
        }
    }
    if mutable_node_count(f) > MAX_MUTABLE_NODES {
        return Err(CompositorFrameError::TooManyMutableNodes);
    }
    Ok(())
}
