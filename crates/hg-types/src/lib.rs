//! Izcal's shared base types (Stage 1 design Section 2.1, layer L2).
//!
//! - [`Instant`] and [`SystemTime`]: Izcal's own time values. Only the
//!   substrate's `Clock` produces them, so engine code never reads an OS
//!   clock (G3, CS-AS-1).
//! - [`invariant!`]: the one sanctioned path for a broken internal
//!   invariant in tier P code (CS-ER-1, CS-ER-3; coding standard A.2).
//! - [`BUILD_ID`]: this build's identity, compiled into every binary
//!   (distribution 13 item 5).
//! - [`RecordingHeader`]: the shared recording and crash-report header
//!   (FI-11), with [`PolicyDigest`] and [`NO_POLICY`].
//! - [`HashMap`] and [`HashSet`]: std maps whose hasher reads one key per
//!   OS process, set once at startup (CS-AS-2).
//!
//! The crate has no OS effects and no `unsafe` (G5).

mod build_id;
mod hash;
mod recording;
mod time;

pub use build_id::{BUILD_ID, BUILD_IS_PINNED, UNPINNED_BUILD_ID};
pub use hash::{HashKeyAlreadySet, HashMap, HashSet, ProcessHasher, ProcessHasherBuilder};
pub use hash::{process_hash_key_is_set, set_process_hash_key};
pub use recording::{
    Common, DataVersion, DecisionLogRef, HEADER_SCHEMA_VERSION, HeaderError, HeaderKind,
    HeaderPolicy, MAX_DATA_VERSIONS, MAX_NAME, NO_POLICY, PolicyDigest, PolicyFlags, ProcessKind,
    ProfileIdentity, RecordingHeader, SandboxTier, Text,
};
pub use time::{Duration, Instant, SystemTime};

/// Aborts with a literal message. The only panic site tier P code reaches.
///
/// # Panics
/// Always; the process aborts (CS-ER-2).
#[cold]
#[inline(never)]
#[track_caller]
#[expect(
    clippy::panic,
    reason = "CS-ER-1: the one sanctioned invariant-failure path"
)]
pub fn invariant_failed(msg: &'static str) -> ! {
    panic!("invariant: {msg}")
}

/// `invariant!(cond, "literal")`: a broken internal invariant, never an
/// input error (CS-ER-1). Input from outside the code is a `Result`.
#[macro_export]
macro_rules! invariant {
    ($cond:expr, $msg:literal) => {
        if !$cond {
            $crate::invariant_failed($msg)
        }
    };
}
