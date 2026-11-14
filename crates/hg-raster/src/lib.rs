//! `hg-raster`: every pixel by integer arithmetic (GR-D3, GR-D4).
//!
//! GR-D3 is the rule the whole crate exists to keep:
//!
//! > Every pixel of page content … and every pixel a page can read …
//! > is computed by **integer arithmetic only**, with one specified
//! > rounding at each step, so the result is **bit-identical on the
//! > CPU and on every GPU back end of every device**.
//!
//! The reason is not tidiness. WGSL, Vulkan and Metal all allow float
//! results to differ — unspecified rounding, reassociation, fusion,
//! flushing, and Metal defaults to fast math — while integer
//! arithmetic is specified to wrap exactly everywhere. So a browser
//! that wants one image on every device has to choose integers, and a
//! browser that wants `toDataURL` not to be a fingerprint has to want
//! that.
//!
//! **There is no `f32` or `f64` in this crate**, and no `as`
//! conversion that could hide one.
//!
//! # What is here at GR-M0
//!
//! The milestone row asks for "`hg-raster` CPU path for fills, AA,
//! blending and images passing `hg-reftest-raster`'s cases with exact
//! hashes on x86-64 and AArch64". So:
//!
//! - [`flatten`]: segments to a polyline, by analytic subdivision
//!   counts in fixed point;
//! - [`cover`]: exact-area coverage of a polygon by signed-area
//!   accumulation, and coverage to 8-bit alpha by one rounding;
//! - [`blend`]: premultiplied compositing with an exact ÷255, and the
//!   separable blend modes of Compositing 1;
//! - [`image`]: nearest, bilinear magnification with 8-bit weights,
//!   and exact-area minification.
//!
//! What is **not** here, and is named rather than stubbed: stroking
//! and dashing (§6.4's stroke-to-fill row), the sparse-strip coarse
//! raster, gradients, filters, masks, the non-separable blend modes,
//! the GPU fine raster, and **elliptical arcs** — [`flatten`] refuses
//! them, because flattening an arc in integer arithmetic needs a
//! fixed-point sine table whose size is a decision GR-M0's review
//! should take rather than this crate should assume.

pub mod blend;
pub mod cover;
pub mod flatten;
pub mod image;
pub mod surface;

pub use cover::{Coverage, Mask};
pub use surface::Surface;
