//! `hg-gpu-alloc`: the allocator's interface, before any placement.
//!
//! GR-M0's row asks for the "HAL design review across three back ends
//! (resource type states, render graph, **allocator**, fences,
//! epochs)" and not for the allocator. GR 6.2 says why there is an
//! Izcal allocator at all: memory "is Izcal's own sub-allocator
//! (`hg-gpu-alloc`), because placement decides fragmentation,
//! residency and per-site accounting (FP-14, FP 6.6), so allocator
//! crates such as `gpu-allocator` are engine work, not bindings".
//!
//! So what is here is the **interface**: the ledger tag every
//! allocation carries, who may name a site, what happens to each
//! class under pressure, and what a volatile allocation is worth when
//! it comes back. **Placement is not here** — no free lists, no
//! heaps, no suballocation strategy, no fragmentation policy. That is
//! GR-M1's, and it is the part the measurements (GR-S1) exist to
//! inform. A placeholder that allocated badly would have met the
//! start definition's letter and told the review nothing.
//!
//! # The six things this interface decides
//!
//! Each is a decision the review owes, and each is a type here rather
//! than a convention, because the failure mode of every one of them
//! is silence.
//!
//! 1. **The ledger tag is not five fields the caller fills in.** A
//!    content GPU process is one per renderer key, so its site,
//!    container and partition are properties of the *process*; only
//!    the compositor GPU service holds every site's pixels and
//!    genuinely varies them per allocation. So the scope is a type
//!    ([`ledger::KeyScope`], [`ledger::CompositorScope`]), a
//!    key-scoped process has no function that accepts a
//!    [`ledger::SiteId`], and a [`ledger::Ledger`] refuses a tag its
//!    scope could not have made.
//!
//! 2. **Over budget is a state, not an error.** The OS budget arrives
//!    asynchronously and can shrink below what is already allocated.
//!    Refusing everything there deadlocks presentation, so
//!    [`budget::Class`] splits what may be refused from what may not,
//!    and gives reclamation one order for all three back ends.
//!
//! 3. **The class is not a function of the kind.** A viewport tile
//!    and a prepaint tile are one [`ledger::Kind`] and two classes,
//!    because FP 6.6 shrinks the prepaint margin first. The kind
//!    answers which ledger line, the class answers what happens under
//!    pressure, and one field cannot answer both.
//!
//! 4. **A volatile allocation's contents may be gone, and two of the
//!    three back ends cannot promise otherwise.** Metal's volatile
//!    storage is reclaimed "without informing the app" and Vulkan has
//!    no purgeable state at all, so the contract takes the weakest
//!    guarantee and [`residency::Allocation::can_sample`] stays false
//!    until something writes the allocation again.
//!
//! 5. **A grant is a move.** G71 makes a shared texture "granted once
//!    and never re-granted", so [`ledger::Grant`] is neither `Clone`
//!    nor `Copy`, and a released texture a holder still pins stays on
//!    the owner's ledger — which is why [`ledger::Ledger::charged`]
//!    is live **plus** pinned bytes and not a sum of live rows.
//!
//! 6. **Caps come from geometry and build constants, never from the
//!    budget.** FP-D19: "a device-dependent JS-visible limit is a
//!    fingerprint of the device's memory". So [`caps`] takes no
//!    budget, and a test reads its signatures to keep it that way.
//!
//! # What this interface still owes
//!
//! Two things, both named here so they are not mistaken for settled.
//!
//! - **Whether a back end can answer `survived`.**
//!   [`residency::Allocation::make_resident`] takes the back end's
//!   report of whether the contents came back, and a back end that
//!   cannot tell must say `false`. Whether Metal can tell — whether
//!   the purgeable-state transition reports the prior state usefully
//!   — is a question for GR-S1, and until it is measured the sound
//!   answer is the conservative one.
//! - **The OS's usage and the ledger's total are two numbers.**
//!   [`budget::Budget::usage`] is the OS's view and
//!   [`ledger::Ledger::charged`] is Izcal's; the driver's own
//!   allocations are in one and not the other. Reconciling them is a
//!   measurement, so this interface keeps them apart rather than
//!   assuming a relation between them.

pub mod budget;
pub mod caps;
pub mod ledger;
pub mod residency;

pub use budget::{Budget, Class, Decision, Request, admit, default_class};
pub use ledger::{
    AllocationId, CompositorScope, ContainerId, Grant, KeyScope, Kind, Ledger, LedgerError,
    PartitionId, Release, Scope, SiteId, TabId, Tag,
};
pub use residency::{Allocation, Contents, Residency, ResidencyError};
