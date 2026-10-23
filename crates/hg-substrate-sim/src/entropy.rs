//! Seeded randomness. Every stream and the cryptographic source derive
//! from the process's seed, itself derived from the world's seed, the
//! process and its incarnation. This is the one place a cryptographic
//! source is seeded (Q-CS-1), which is why only the simulation and replay
//! binaries may link this crate.

use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::atomic::{AtomicU64, Ordering};

use hg_substrate::prng::Xoshiro256StarStar;
use hg_substrate::{CryptoRandom, Entropy, EntropyError, EntropyPurpose, RandomStream};

use crate::world::derive;

const fn purpose_tag(p: EntropyPurpose) -> u64 {
    match p {
        EntropyPurpose::Random => 1,
        EntropyPurpose::HashKey => 2,
        EntropyPurpose::Scheduling => 3,
    }
}

pub(crate) struct SimCrypto {
    stream: Mutex<Xoshiro256StarStar>,
}

impl CryptoRandom for SimCrypto {
    fn fill(&self, dest: &mut [u8]) -> Result<(), EntropyError> {
        self.stream
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .fill_bytes(dest);
        Ok(())
    }
}

pub(crate) struct SimEntropy {
    seed: u64,
    /// Streams made so far, so each new stream differs.
    made: AtomicU64,
    crypto: SimCrypto,
}

impl SimEntropy {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            seed,
            made: AtomicU64::new(0),
            crypto: SimCrypto {
                stream: Mutex::new(Xoshiro256StarStar::from_seed(derive(&[seed, 0xc0]))),
            },
        }
    }
}

impl Entropy for SimEntropy {
    fn stream(&self, purpose: EntropyPurpose) -> Box<dyn RandomStream> {
        // ordering: Relaxed — the world runs on one thread; the counter
        // only has to be unique per call.
        let n = self.made.fetch_add(1, Ordering::Relaxed);
        Box::new(Xoshiro256StarStar::from_seed(derive(&[
            self.seed,
            purpose_tag(purpose),
            n,
        ])))
    }

    fn crypto(&self) -> &dyn CryptoRandom {
        &self.crypto
    }
}
