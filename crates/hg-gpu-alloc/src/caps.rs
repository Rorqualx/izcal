//! The caps, which are not a function of the device's memory.
//!
//! FP-D19 decided (a): "Canonical limits identical on every device
//! ... device memory protected by the ladder", because "a
//! device-dependent JS-visible limit is a fingerprint of the device's
//! memory". FP 6.6 applies it to the glyph pools by name: both sizes
//! are "fixed per build and identical on every device, never a
//! function of RAM, GPU or any other web-observable device property
//! (FP-D19's rule)".
//!
//! # What that forbids here, exactly
//!
//! The obvious allocator sizes its pools from the budget it just read
//! from the OS. That is the fingerprint: how many glyphs a page can
//! use before re-raster starts, how large a canvas it can get, when
//! an atlas stops growing — all of them page-observable through
//! timing, all of them then a reading of the machine's VRAM. The
//! graphics spec already treats far less than this as a fingerprint:
//! "3.05 bits from one canvas text test, 5.73 bits from six canvas
//! and WebGL tests".
//!
//! So **no function in this module takes a
//! [`Budget`](crate::budget::Budget)**, and the budget's only job is
//! to decide *when reclamation runs*, never *how large anything is*.
//! That rule is not written in prose alone. A test in `tests/it`
//! reads this file, takes the parameter list of every `pub const fn`
//! in it, and fails if any of them mentions a budget — the same trick
//! GR-18's published size uses to stop a number going stale, applied
//! to a signature instead of a count. It reads parameter lists and
//! not the whole file because this very comment says "budget" nine
//! times, and a test that the documentation defeats is a test that
//! gets deleted.
//!
//! # What it permits
//!
//! A cap may depend on the surface geometry, because the page can
//! already read its own viewport. FP 6.6's tile budget is "≤ 2.5 ×
//! viewport pixels × 4 B", which is geometry and not memory. The
//! distinction the rule actually draws is between what the page
//! already knows and what it would learn.

/// FP 6.6's tile budget numerator: 2.5 viewports, as 5/2, because
/// 2.5 is not an integer and this crate does no floating point
/// (GR-D3's rule for pixels, kept here for the arithmetic too).
pub const TILE_VIEWPORTS_NUM: u64 = 5;

/// FP 6.6's tile budget denominator.
pub const TILE_VIEWPORTS_DEN: u64 = 2;

/// Bytes per tile pixel: FP 6.6 counts tiles at 4 B.
pub const TILE_BYTES_PER_PX: u64 = 4;

/// FP 6.6's swapchain rule: "Two buffers by default, three only while
/// an animation misses frames".
///
/// A count, held as `u64` because it is only ever multiplied by
/// bytes: `u64::from` is not a const function yet, and a widening
/// `as` is what tier P's `as_conversions` denies, so the type that
/// needs no conversion is the right one.
pub const SWAPCHAIN_IMAGES_DEFAULT: u64 = 2;

/// The most a swapchain ever holds.
pub const SWAPCHAIN_IMAGES_MAX: u64 = 3;

/// FP 6.6: "Each atlas starts at 256 KiB†".
pub const ATLAS_START_BYTES: u64 = 256 * 1024;

/// FP 6.6: until TX-52 measures them, the A8 and RGBA8 pools together
/// "stay within 0.3's ≤ 2 MiB† per partition".
pub const ATLAS_POOLS_BYTES_PER_PARTITION: u64 = 2 * 1024 * 1024;

/// GR 6.3 and FP 6.6: "At most **16†** per document".
pub const COMPOSITOR_MUTABLE_NODES_MAX: usize = 16;

/// The tile budget for a viewport of `viewport_px` pixels.
///
/// Saturating, not wrapping: a viewport large enough to overflow this
/// is a viewport that gets the largest budget expressible, and never
/// a small one.
#[must_use]
pub const fn tile_bytes(viewport_px: u64) -> u64 {
    viewport_px
        .saturating_mul(TILE_BYTES_PER_PX)
        .saturating_mul(TILE_VIEWPORTS_NUM)
        .saturating_div(TILE_VIEWPORTS_DEN)
}

/// The room [`Reserved`](crate::budget::Class::Reserved) is
/// guaranteed, for a surface of `surface_px` pixels and `images`
/// swapchain images.
///
/// This is the number [`admit`](crate::budget::admit) holds
/// discretionary work off, and it depends on the surface and on
/// FP 6.6's swapchain rule — on geometry and a build constant, and on
/// nothing the OS reported.
#[must_use]
pub const fn reserved_cap_bytes(surface_px: u64, images: u64) -> u64 {
    let n = if images > SWAPCHAIN_IMAGES_MAX {
        SWAPCHAIN_IMAGES_MAX
    } else {
        images
    };
    surface_px
        .saturating_mul(TILE_BYTES_PER_PX)
        .saturating_mul(n)
}
