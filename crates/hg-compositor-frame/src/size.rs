//! GR-18's published size artifact.
//!
//! **GR-18** ("Compositor input is small") requires the
//! compositor-frame IDL's size -- types, variants, maximum nesting --
//! to be published as the IPC-R11 artifact and **not to grow without
//! a security review**, with zero page-controlled filters, paths or
//! ICC data parsed by the compositor (G63).
//!
//! The figures below are that artifact. They are constants rather
//! than a document because a number in a document is re-read only by
//! whoever already doubts it: the test beside this module recomputes
//! every one of them from `idl/compositor_frame.hgidl` and fails when
//! any disagrees. Adding a type, a variant, a field or a level of
//! nesting is therefore a **build failure** whose fix is to change
//! this module -- which is a diff a security review can be asked
//! for. The target does not depend on anyone remembering it.
//!
//! The same test asserts the G63 half: that no declared type names a
//! path, segment, fill rule, gradient, filter, mask, ICC profile or
//! glyph. That is the "**0** page-controlled filters, paths or ICC
//! data" clause, checked against the format rather than asserted
//! about it.

/// Declared types: the closed error enum, one plain enum, three
/// unions and fourteen structs.
pub const TYPES: usize = 19;

/// Variants across the error enum, the plain enum and the unions.
///
/// `CompositorFrameError` 17, `AnimatedProperty` 2, `ClipShape` 2,
/// `KeyframeValue` 2, `TimingFunction` 3.
pub const VARIANTS: usize = 17 + 2 + 2 + 2 + 3;

/// Fields across every struct and every union case.
pub const FIELDS: usize = 92;

/// The deepest nesting of one declared type inside another, counting
/// the `Frame` as level 0.
///
/// Measured, not reasoned: the deepest chain is **`Frame` ->
/// `Animation` -> `Keyframe` -> `KeyframeValue` -> `Transform`**, at
/// four. The animation record is deeper than the tile grid, which was
/// not the guess -- the tile chain (`Frame` -> `Layer` -> `Tile` ->
/// `TextureRef`) is three.
///
/// Both chains matter and for different reasons. This one is the
/// deepest, which is the figure GR-18 asks for. The tile chain is the
/// only place a **vector holds a vector**, so it is the only shape
/// where one declared count multiplies another: 256 layers x 4,096
/// tiles is the frame's worst allocation, and it is bounded because
/// both counts are.
pub const MAX_NESTING: usize = 4;

/// Vectors whose length the format bounds, each a limit rather than a
/// hint.
pub const BOUNDED_VECTORS: usize = 10;

/// The words a page-controlled compositor input must not contain
/// (G63). Checked against every declared type and field name.
pub const FORBIDDEN_VOCABULARY: [&str; 9] = [
    "path", "segment", "fillrule", "gradient", "filter", "mask", "icc", "glyph", "shader",
];
