//! The sanctioned thread interface (coding standard tier O row: threads
//! only through the substrate; the media spec's real-time interface
//! MD-D10/MD-D29 is the same entry point).
//!
//! First drop: the spawn seed the FP-M0 probe needs (two threads).
//! The full interface — named threads, affinity and the real-time
//! class — grows here when its first consumer names it.

use std::io;
use std::thread::{self, JoinHandle};

/// Spawns a named OS thread through the substrate.
///
/// This is the only sanctioned way to create threads in Izcal: tier O's
/// generated lint table keeps `std::thread` spawning banned everywhere
/// else (coding standard 0.3.7, A.7 note).
pub fn spawn<F>(name: &'static str, f: F) -> io::Result<JoinHandle<()>>
where
    F: FnOnce() + Send + 'static,
{
    thread::Builder::new().name(name.into()).spawn(f)
}
