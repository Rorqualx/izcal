//! Izcal's runtime substrate (Stage 1 design Section 3, HGBA-3 B1).
//!
//! Every effect an engine crate can have on the world goes through these
//! traits: reading a clock, drawing randomness, running work on a
//! thread, reserving memory. Two implementations exist: `hg-substrate-os`
//! (the real OS, in shipping binaries) and `hg-substrate-sim` (a seeded,
//! single-process simulator, for deterministic tests and replay). Engine
//! code takes a [`Substrate`] and cannot tell which one it got.
//!
//! The traits are synchronous (CS-D8, CS-AS-4): work is a [`Task`], a
//! closure that runs to completion on a named sequence. There is no
//! `async` here and no executor of anyone else's.
//!
//! Phase A of M1 defines the traits M1 needs first. The IPC trait joins in
//! phase D, and the code-memory, network and storage traits join with the
//! milestones that first use them (Stage 1 design 3.1 lists all of them).

mod clock;
mod entropy;
mod executor;
mod memory;
pub mod prng;

use std::sync::Arc;

pub use clock::Clock;
pub use entropy::{CryptoRandom, Entropy, EntropyError, EntropyPurpose, RandomStream};
pub use executor::{Executor, PhasePool, PostError, Priority, SequenceHandle, Task};
pub use memory::{Memory, MemoryError, MemoryKind, MemoryStats, Region};

/// The substrate a process runs on: one instance per (real or simulated)
/// process, handed to the process's entry function.
#[derive(Clone)]
pub struct Substrate {
    /// Monotonic and wall time.
    pub clock: Arc<dyn Clock>,
    /// Named, seeded randomness and the cryptographic source.
    pub entropy: Arc<dyn Entropy>,
    /// Sequences, posted tasks and the fork/join phase pool.
    pub exec: Arc<dyn Executor>,
    /// Address-space reservation and commit, charged to ledger lines.
    pub mem: Arc<dyn Memory>,
}

#[cfg(test)]
mod tests {
    //! The traits are object-safe and a [`Substrate`] can be assembled
    //! from them. The fakes here are the minimum to show that; the real
    //! implementations are `hg-substrate-os` and `hg-substrate-sim`.

    use super::*;
    use hg_types::{Duration, Instant, SystemTime};
    use std::sync::Mutex;

    struct FixedClock;
    impl Clock for FixedClock {
        fn monotonic(&self) -> Instant {
            Instant::from_nanos(42)
        }
        fn wall(&self) -> SystemTime {
            SystemTime::UNIX_EPOCH
        }
    }

    struct Counter(u64);
    impl RandomStream for Counter {
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.saturating_add(1);
            self.0
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            dest.fill(7);
        }
    }
    struct NoCrypto;
    impl CryptoRandom for NoCrypto {
        fn fill(&self, _dest: &mut [u8]) -> Result<(), EntropyError> {
            Err(EntropyError::Unavailable)
        }
    }
    struct FakeEntropy(NoCrypto);
    impl Entropy for FakeEntropy {
        fn stream(&self, _purpose: EntropyPurpose) -> Box<dyn RandomStream> {
            Box::new(Counter(0))
        }
        fn crypto(&self) -> &dyn CryptoRandom {
            &self.0
        }
    }

    /// Runs each task inline on post; enough to show the shape.
    struct InlineExecutor(InlinePool);
    struct InlinePool;
    impl PhasePool for InlinePool {
        fn for_each_index(&self, n: usize, f: &(dyn Fn(usize) + Sync)) {
            (0..n).for_each(f);
        }
    }
    impl Executor for InlineExecutor {
        fn sequence(&self, _name: &'static str, _priority: Priority) -> SequenceHandle {
            SequenceHandle::from_raw(0)
        }
        fn post(
            &self,
            seq: SequenceHandle,
            task: Task,
            _delay: Option<Duration>,
            _tolerance: Duration,
        ) -> Result<(), PostError> {
            if seq.raw() != 0 {
                return Err(PostError::UnknownSequence);
            }
            task.run();
            Ok(())
        }
        fn fork_join(&self) -> &dyn PhasePool {
            &self.0
        }
    }

    struct NoMemory;
    impl Memory for NoMemory {
        fn reserve(&self, _bytes: usize, _kind: MemoryKind) -> Result<Region, MemoryError> {
            Err(MemoryError::OutOfAddressSpace)
        }
        fn commit(&self, _region: &Region, _bytes: usize) -> Result<(), MemoryError> {
            Err(MemoryError::CommitFailed)
        }
        fn decommit(&self, _region: &Region, _bytes: usize) {}
        fn purge(&self, _region: &Region, _bytes: usize) {}
        fn release(&self, _region: Region) {}
        fn stats(&self) -> MemoryStats {
            MemoryStats::default()
        }
    }

    fn substrate() -> Substrate {
        Substrate {
            clock: Arc::new(FixedClock),
            entropy: Arc::new(FakeEntropy(NoCrypto)),
            exec: Arc::new(InlineExecutor(InlinePool)),
            mem: Arc::new(NoMemory),
        }
    }

    #[test]
    fn a_substrate_assembles_and_each_trait_is_callable() {
        let s = substrate();
        assert_eq!(s.clock.monotonic(), Instant::from_nanos(42));

        let mut r = s.entropy.stream(EntropyPurpose::Random);
        assert_eq!(r.next_u64(), 1);
        assert_eq!(
            s.entropy.crypto().fill(&mut [0u8; 4]),
            Err(EntropyError::Unavailable)
        );

        let ran = Arc::new(Mutex::new(false));
        let seq = s.exec.sequence("test", Priority::UserVisible);
        let flag = Arc::clone(&ran);
        s.exec
            .post(
                seq,
                Task::new(move || *flag.lock().unwrap() = true),
                None,
                Duration::ZERO,
            )
            .unwrap();
        assert!(*ran.lock().unwrap());
        assert_eq!(
            s.exec.post(
                SequenceHandle::from_raw(9),
                Task::new(|| {}),
                None,
                Duration::ZERO
            ),
            Err(PostError::UnknownSequence)
        );

        let out: Vec<Mutex<usize>> = (0..4).map(|_| Mutex::new(0)).collect();
        s.exec.fork_join().for_each_index(4, &|i| {
            *out[i].lock().unwrap() = i.saturating_mul(10);
        });
        let got: Vec<usize> = out.iter().map(|m| *m.lock().unwrap()).collect();
        assert_eq!(got, [0, 10, 20, 30]);

        assert_eq!(
            s.mem.reserve(4096, MemoryKind::ProcessPart),
            Err(MemoryError::OutOfAddressSpace)
        );
    }
}
