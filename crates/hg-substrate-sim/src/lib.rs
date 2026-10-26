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
//! restarts. The simulated IPC transport joined in phase D, and the
//! broker's scripted OS ([`OsScript`], OS integration 6.1) with M1's
//! OS-integration inputs; `hg-netsim` joins with the network stack.

// G5: not inventoried. The manifest inherits the workspace table, whose
// unsafe_code is deny (inventoried crates need deny), so forbid is here.
#![forbid(unsafe_code)]

mod clock;
mod entropy;
mod exec;
mod install;
mod ipc;
mod launch;
mod memory;
mod os;
mod region;
mod trace;
mod world;

use std::sync::Arc;

#[cfg(doc)]
use hg_substrate::Memory;
use hg_substrate::Substrate;

pub use launch::{ChildEntry, ChildTable};
pub use os::{OsAnswer, OsFrames, OsScript};
pub use region::SimOs;
pub use world::{
    Entry, FaultPlan, Limits, ProcessId, RunEnd, SEQUENCE_CAPACITY, SimConfig, Step, Summary, World,
};

/// The substrate for incarnation `generation` of process `pid`.
pub(crate) fn substrate_for(
    shared: &Arc<world::Shared>,
    pid: ProcessId,
    generation: u32,
    seed: u64,
    has_os: bool,
    children: Option<Arc<launch::ChildTable>>,
) -> Substrate {
    let exec: Arc<dyn hg_substrate::Executor> = Arc::new(exec::SimExecutor {
        shared: Arc::clone(shared),
        pid,
        generation,
        pool: exec::SimPhasePool {
            shared: Arc::clone(shared),
        },
    });
    let os: Option<Arc<dyn hg_substrate::OsPort>> = if has_os {
        Some(Arc::new(os::SimOsPort {
            shared: Arc::clone(shared),
            pid,
            generation,
        }))
    } else {
        None
    };
    Substrate {
        clock: Arc::new(clock::SimClock {
            shared: Arc::clone(shared),
        }),
        entropy: Arc::new(entropy::SimEntropy::new(seed)),
        exec,
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
        os,
        launcher: children.map(|table| -> Arc<dyn hg_substrate::Launcher> {
            Arc::new(launch::SimLauncher::new(Arc::clone(shared), pid, table))
        }),
    }
}
