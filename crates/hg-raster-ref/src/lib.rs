//! `hg-raster-ref`: the same coverage, computed a different way.
//!
//! §6.21 gives this crate one job: be the thing `hg-raster` can be
//! wrong against. That is worth something only if it is wrong in
//! different ways, so nothing here shares an algorithm with the
//! production path:
//!
//! | | `hg-raster` | `hg-raster-ref` |
//! |---|---|---|
//! | Method | signed-area accumulation, one edge at a time, prefix-summed along each row | the polygon clipped to each pixel, and the clipped polygon's area by the shoelace formula |
//! | Arithmetic | `i64`, with a specified rounding where an edge crosses a pixel boundary | **exact rationals** in `i128`, no rounding until the final 8-bit value |
//! | Speed | linear in the edges a shape touches | a clip per pixel per edge, which is why this never ships |
//!
//! # What "exact" buys
//!
//! The production rasterizer rounds the x of an edge where it crosses
//! a pixel boundary. That rounding is specified and identical on
//! every machine, so GR-D3's bit-identical claim holds — but it is
//! not the number an exact computation gives, and the difference is
//! the thing nobody measures until an image looks wrong. Here it is
//! measured: `check` reports how many pixels differ and by how much,
//! and the oracle holds both numbers.
//!
//! # What is here, and what is not
//!
//! Exact coverage for the **non-zero** fill rule. The even-odd rule
//! is not here, and the reason is worth stating rather than hiding: a
//! clipped polygon's signed area is the integral of the *winding
//! number* over the pixel, which is exactly what non-zero needs and
//! is not what even-odd needs — even-odd is the integral of the
//! winding's parity, which is not an integral of anything the
//! shoelace formula computes. Doing it exactly needs an arrangement
//! of the edges inside each pixel, which is GR-M1's. Until then
//! `hg-raster`'s even-odd path is checked against shapes whose answer
//! is known by construction.

pub mod exact;
pub mod rational;

pub use exact::{Difference, check, coverage};
pub use rational::Rational;
