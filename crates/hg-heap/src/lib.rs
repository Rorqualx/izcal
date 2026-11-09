//! Izcal's JS engine heap (JS spec 6.1 and 6.5; JS-M0). One agent heap
//! lives in the process's cage; a branded [`Mutator`] is the only way to
//! allocate or reach a GC point; [`Root`]s survive GC points and
//! [`Gc`]s cannot; the collector is precise, non-generational
//! mark-region (Immix) with opportunistic evacuation. The design and its
//! decisions are in `research_notes/JS-M0 design review/heap.md`.

mod cage;
mod heap;
mod node;
mod object;
mod slot;
mod space;
#[cfg(test)]
mod tests;
mod trusted;
pub mod verify;

pub use cage::CageError;
pub use heap::{
    Gc, Heap, HeapConfig, HeapError, HeapStats, Mutator, ProcessCage, Root, Scope, Stress, Value,
};
pub use node::{ExternalCell, ExternalSlot, NodeRef, NodeRootId, NodeSpace, NodeVisitor};
pub use object::{Any, Layout, Node, Object, Raw, RawValue, Referent, Tail, Traced, Weak};
pub use slot::{MAX_CAGE_BYTES, Slot};
pub use space::AllocError;
pub use trusted::{TrustedError, TrustedId, TrustedKind, TrustedRegion};
pub use verify::{Audit, Violation};

/// Support for the tests of crates built on the heap: memory backed by an
/// ordinary allocation, as this crate's own tests use (feature `testing`,
/// enabled only by dev-dependencies).
#[cfg(feature = "testing")]
pub mod testing {
    pub use crate::cage::test_memory::TestMemory;
    pub use crate::node::TestNodeSpace;
}
