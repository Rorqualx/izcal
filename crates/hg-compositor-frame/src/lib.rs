//! `hg-compositor-frame`: what a content GPU process sends the shared
//! compositor GPU service.
//!
//! # Why this is its own package
//!
//! The graphics spec's crate table put all three `.hgidl` formats --
//! display list, compositor frame, canvas command -- in
//! `hg-display-list`. That cannot hold for the compositor frame, and
//! the reason is mechanical rather than aesthetic.
//!
//! **G63** says the compositor GPU service's binary closure contains
//! no display-list reader. **`hg-graph-lint` reads `cargo
//! metadata`**, so the finest granularity at which it can state or
//! check a rule is the *package*. A compositor that reads compositor
//! frames out of `hg-display-list` necessarily puts that package --
//! and so the display-list reader -- in its closure. The rule would
//! then be unenforceable by the one tool that enforces every other
//! rule like it, and "the linker probably dropped it" is not a
//! security argument.
//!
//! So the format lives here, and `tools/graph-rules.toml` can carry
//! G63 as an ordinary `forbid` row. The crate-table amendment this
//! implies is **proposed** in the spec's §6.24 and in GR-M0's review,
//! and is the owner's to accept or amend: it changes a gate file and a
//! spec table, which this project does not do on its own authority.
//!
//! # Why the vocabulary is small, and stays small
//!
//! The compositor holds **every site's** composited pixels, which is
//! why GR-18 makes the size of this format a published artifact that
//! does not grow without a security review. [`size`] computes that
//! artifact from the IDL itself, and the test beside it fails when a
//! type, a variant or a level of nesting is added -- so the target is
//! a build failure rather than a line in a report nobody re-reads.
//!
//! The format therefore has no paths, segments, fill rules,
//! gradients, filters, masks, ICC data or glyphs. Its whole
//! page-controlled vocabulary is tile grids, a property-tree subset
//! of transforms, rectangular and rounded-rectangle clips in fixed
//! point, opacity, scroll nodes, surface embeds, transform and
//! opacity animations, damage and one CPU-observed fence.
//!
//! # Why the property trees are flat
//!
//! The IDL forbids a recursive type, because a recursive type has no
//! size bound. The trees are therefore vectors with parent indices,
//! which is also what HGBA-3's B6 asks for ("property trees as
//! data"). A parent must be an **earlier** index than its child, so a
//! cycle is not expressible rather than merely invalid -- the same
//! reasoning `hg-gpu-graph`'s pass sequence uses, and the reason a
//! cycle error variant would be dead code here.
//!
//! The types are not hand-written: `hg-ipc-codegen` emits them from
//! `idl/compositor_frame.hgidl`, and a hand edit fails CI (G29).
//! [`check`] and [`size`] are the hand-written modules.

pub mod check;
pub mod generated;
pub mod size;

#[cfg(test)]
pub mod sample;
#[cfg(test)]
mod tests;

pub use check::validate;
pub use generated::compositor_frame;
