//! Izcal's OS substrate (tier O, G3 — coding standard 0.3.7 tier table).
//!
//! The one crate allowed ambient OS effects. Holds the real-OS
//! implementation of the substrate traits (Stage 1 design 3.1), the
//! sandbox backends (security scope 6.7: `hg-sandbox` lives inside
//! `hg-substrate-os`), the process-kind machinery and the sanctioned
//! thread interface.
//!
//! - FP-M0 half B (23 October 2026): the sandbox backends, the tier
//!   probe, and the two FP-M0 binaries (`fp-m0-probe`, the supervisor,
//!   and `fp-m0-empty`, the sandboxed empty process).
//! - M1 phase B: [`os_substrate`] — the OS clock, entropy, executor and
//!   memory.

pub mod clock;
pub mod entropy;
pub mod executor;
pub mod memory;
pub mod os;
pub mod sandbox;
pub mod thread;

use std::sync::Arc;

use hg_substrate::{Entropy, Substrate};

/// Worker counts for [`os_substrate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Threads {
    /// Workers for the three ordinary priorities.
    pub general: usize,
    /// Workers for `MayBlock` sequences.
    pub may_block: usize,
}

impl Threads {
    /// One general worker per available core, at least 2 and at most 16†,
    /// and 2† `MayBlock` workers.
    pub fn for_this_machine() -> Self {
        let cores = std::thread::available_parallelism().map_or(2, std::num::NonZero::get);
        Self {
            general: cores.clamp(2, 16),
            may_block: 2,
        }
    }
}

/// Why the substrate could not start.
#[derive(Debug)]
pub enum StartError {
    /// The OS random source failed, so no hash key could be drawn.
    Entropy,
    /// A worker thread could not be started.
    Thread(std::io::Error),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entropy => f.write_str("the OS random source failed"),
            Self::Thread(e) => write!(f, "a substrate thread could not start: {e}"),
        }
    }
}

impl std::error::Error for StartError {}

/// The process's OS substrate. Call it once, first thing in `main`: it
/// draws the process hash key from the OS (CS-AS-2), which must happen
/// before the process creates any hash map, then starts the executor's
/// threads.
///
/// # Errors
/// [`StartError`] if the OS random source or thread creation fails.
pub fn os_substrate(threads: Threads) -> Result<Substrate, StartError> {
    let entropy = entropy::OsEntropy::new();
    if !hg_types::process_hash_key_is_set() {
        let mut key = [0u8; 16];
        entropy
            .crypto()
            .fill(&mut key)
            .map_err(|_e| StartError::Entropy)?;
        let (k0, k1) = key.split_at(8);
        let word = |b: &[u8]| u64::from_le_bytes(b.try_into().unwrap_or([0; 8]));
        // A second caller lost a race to the same end state: a key is set.
        let _ = hg_types::set_process_hash_key(word(k0), word(k1));
    }
    let exec = executor::OsExecutor::new(threads.general, threads.may_block)
        .map_err(StartError::Thread)?;
    Ok(Substrate {
        clock: Arc::new(clock::OsClock::new()),
        entropy: Arc::new(entropy),
        exec: Arc::new(exec),
        mem: Arc::new(memory::OsMemory::new()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_substrate_starts_and_keys_hashing() {
        let s = os_substrate(Threads {
            general: 2,
            may_block: 1,
        })
        .unwrap();
        assert!(hg_types::process_hash_key_is_set());
        let _ = s.clock.monotonic();
    }
}
