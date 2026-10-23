//! Randomness (Stage 1 design 3.1). Every consumer draws from a named
//! stream, so adding a consumer does not perturb the others' sequences.
//! Production seeds streams from the OS; simulation seeds them from the
//! run's seed. The cryptographic source is the OS CSPRNG in every shipping
//! binary and is seeded only under `hg-substrate-sim` (Q-CS-1).

/// What a random stream is for. Each purpose gets its own stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntropyPurpose {
    /// `Math.random()` (the engine's host hook).
    Random,
    /// The process's hash key (`hg_types::set_process_hash_key`).
    HashKey,
    /// Scheduling choices the substrate itself makes (for example the
    /// order of equally ready tasks in simulation).
    Scheduling,
}

/// A stream of random numbers for one purpose. Not cryptographic.
pub trait RandomStream: Send {
    /// The next 64 random bits.
    fn next_u64(&mut self) -> u64;

    /// Fills `dest` with random bytes.
    fn fill_bytes(&mut self, dest: &mut [u8]);
}

/// The cryptographic random source.
pub trait CryptoRandom: Send + Sync {
    /// Fills `dest` with cryptographically secure random bytes.
    ///
    /// # Errors
    /// [`EntropyError::Unavailable`] if the source cannot produce bytes;
    /// the caller must not fall back to a weaker source.
    fn fill(&self, dest: &mut [u8]) -> Result<(), EntropyError>;
}

/// The randomness interface.
pub trait Entropy: Send + Sync {
    /// A new stream for `purpose`.
    fn stream(&self, purpose: EntropyPurpose) -> Box<dyn RandomStream>;

    /// The cryptographic source.
    fn crypto(&self) -> &dyn CryptoRandom;
}

/// Why randomness could not be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropyError {
    /// The cryptographic source failed (the OS call returned an error).
    Unavailable,
}

impl core::fmt::Display for EntropyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unavailable => f.write_str("the cryptographic random source is unavailable"),
        }
    }
}

impl std::error::Error for EntropyError {}
