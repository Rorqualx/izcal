//! `hg-fontc`: Izcal's font compiler, at the table layer.
//!
//! Fonts and text 6.15 gives `hg-fontc` the step from design sources to
//! OpenType binaries, "through writers generated from the same
//! `.hgfont` descriptions", and TX-D24 adds the rule that makes that
//! worth doing: **every output is re-read by `hg-fontfmt` before it
//! enters the font package.** TX-M0 asks for the writers themselves —
//! "`hg-fontc`'s writers generated for the first wave's tables" — and
//! that is what this crate holds today. The front end that reads a
//! designer's sources is the type-design program's, and the program is
//! not staffed (D-S8).
//!
//! # What a writer promises
//!
//! The reader's rule is that **memory safety never depends on
//! validation**: every accessor checks its bounds and returns `None`
//! rather than panic, for any bytes at all. The writer's rule is the
//! mirror of it, and it is about *agreement* rather than about safety:
//!
//! > A writer never emits a table whose declared counts disagree with
//! > what it was given, and never emits a table with a field it was
//! > never told.
//!
//! So [`Error::Count`] is the writer catching the compiler's own
//! arithmetic — a `numberOfHMetrics` of 3 with four metrics pushed is
//! refused, not padded and not truncated — and [`Error::Unset`] catches
//! the field a new table's author forgot. Both are mistakes a font
//! compiler makes, and both produce a font that a conforming reader
//! declines, which is the expensive way to find out.
//!
//! # Why the writers are generated
//!
//! Because the readers are. A hand-written writer beside a generated
//! reader is two descriptions of one table, and the pair drifts in the
//! direction nobody tests. Generating both from one `.hgfont` leaves
//! one place to be wrong — and the round-trip tests in `tests/it` then
//! check that place against bytes transcribed from the OpenType prose,
//! because a round-trip alone would agree with itself however wrong the
//! shared description was (the common-mode risk the JavaScript spec's
//! 6.3 names for differential testing).
//!
//! Nothing here is derived from any third-party source code (M-1 part
//! 3): the descriptions are Izcal's own, written from the published
//! specification.

pub mod generated;
pub mod write;

pub use generated::writers;
pub use write::Error;
