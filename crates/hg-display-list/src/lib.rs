//! `hg-display-list`: what a renderer sends a content GPU process, and
//! the bounded path format.
//!
//! Graphics §6.3 puts the path types here, in **L2**, rather than in
//! `hg-path` (L2g), so that the text stack (L2t) can emit them without
//! depending on the graphics layer: large glyphs and colour-glyph
//! vectors travel in the same format as any path (TX §6.1, and TX
//! §9.5's R21 asked for it).
//!
//! # Why every count is a limit
//!
//! A display list crosses a process boundary from a renderer the
//! content GPU process does not trust. A path with no bound on its
//! segments is a renderer that can make the GPU process allocate
//! without limit, which is the same failure the networking spec's
//! §6.2 budgets exist for. §6.3 fixes the two bounds — **65,536†
//! segments per path** and **4,096† paths per chunk** — and the
//! generated reader enforces them, so there is no path by which an
//! over-long list reaches a decoded value.
//!
//! # Why 24.8 fixed point
//!
//! GR-D3: every pixel of page content is computed by integer
//! arithmetic only, "so that the result is bit-identical on the CPU
//! and on every GPU back end of every device". A float coordinate
//! would put the first rounding before the rasterizer and make the
//! claim untrue at the boundary. 24.8 gives ±8,388,607.996 local
//! units, and the layer origin they are relative to is a separate
//! `i64`, so a page beyond 8 M device pixels still renders exactly
//! (§6.4's transform row).
//!
//! The types are not hand-written: `hg-ipc-codegen` emits them from
//! `idl/display_list.hgidl`, and a hand edit fails CI (G29). [`check`]
//! is the one hand-written module, for the six things a wire format
//! cannot say.

pub mod check;
pub mod generated;

#[cfg(test)]
pub mod sample;
#[cfg(test)]
mod tests;

pub use check::validate;
pub use generated::display_list;
