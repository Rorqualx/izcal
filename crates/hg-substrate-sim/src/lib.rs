//! Izcal's simulation substrate (Stage 1 design 3.2; HGBA-3 DET-1, DST-1).
//!
//! `hg-substrate-sim` runs every simulated process of a browser in one OS
//! process. Each process gets its own [`hg_substrate::Substrate`]; one
//! seeded scheduler picks the next task among all of them; virtual time
//! moves only when nothing is runnable, jumping to the next timer; and
//! the scheduler injects faults. Every decision goes into a trace whose
//! hash is the same for the same seed on every run and every OS (CS-T5).
//!
//! Tier R, with G3 (coding standard 6.2, Q-CS-1): this crate has no
//! ambient OS effects. Real addresses come from a backing [`Memory`]
//! that the binary building the world passes in. Only the simulation
//! binary run by `hg-dst` and `hg-replay-main` may link it, because its
//! cryptographic source is seeded.
//!
//! M1 phase C: processes, sequences, timers, phases, memory, crashes and
//! restarts. The simulated IPC transport joins in phase D and
//! `hg-netsim` with the network stack.

// G5: not inventoried. The manifest inherits the workspace table, whose
// unsafe_code is deny (inventoried crates need deny), so forbid is here.
#![forbid(unsafe_code)]

mod clock;
mod entropy;
mod exec;
mod ipc;
mod memory;
mod trace;
mod world;

use std::sync::Arc;

#[cfg(doc)]
use hg_substrate::Memory;
use hg_substrate::Substrate;

pub use world::{
    Entry, FaultPlan, Limits, ProcessId, RunEnd, SEQUENCE_CAPACITY, SimConfig, Step, Summary, World,
};

/// The substrate for incarnation `generation` of process `pid`.
pub(crate) fn substrate_for(
    shared: &Arc<world::Shared>,
    pid: ProcessId,
    generation: u32,
    seed: u64,
) -> Substrate {
    Substrate {
        clock: Arc::new(clock::SimClock {
            shared: Arc::clone(shared),
        }),
        entropy: Arc::new(entropy::SimEntropy::new(seed)),
        exec: Arc::new(exec::SimExecutor {
            shared: Arc::clone(shared),
            pid,
            generation,
            pool: exec::SimPhasePool {
                shared: Arc::clone(shared),
            },
        }),
        mem: Arc::new(memory::SimMemory {
            shared: Arc::clone(shared),
            pid,
            generation,
        }),
        ipc: Arc::new(ipc::SimIpc {
            shared: Arc::clone(shared),
            pid,
            generation,
        }),
    }
}
