//! Izcal's JS engine heap (JS spec 6.1 and 6.5; JS-M0). One agent heap
//! lives in the process's cage; a branded [`Mutator`] is the only way to
//! allocate or reach a GC point; [`Root`]s survive GC points and
//! [`Gc`]s cannot; the collector is precise, non-generational
//! mark-region (Immix) with opportunistic evacuation. The design and its
//! decisions are in `research_notes/JS-M0 design review/heap.md`.

mod cage;
mod heap;
mod object;
mod slot;
mod space;
#[cfg(test)]
mod tests;
mod trusted;

pub use cage::CageError;
pub use heap::{Gc, Heap, HeapConfig, HeapError, HeapStats, Mutator, Root, Scope, Stress, Value};
pub use object::{Any, Layout, Object, Raw, RawValue, Referent, Tail, Traced, Weak};
pub use slot::{MAX_CAGE_BYTES, Slot};
pub use space::AllocError;
pub use trusted::{TrustedError, TrustedId, TrustedKind, TrustedRegion};
