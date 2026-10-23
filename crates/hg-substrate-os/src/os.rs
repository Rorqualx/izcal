//! The OS calls behind the substrate's randomness and memory. Each
//! submodule is an inventoried `unsafe` module (CS-TY-13,
//! `unsafe-inventory.toml`).

pub mod random;
pub mod vm;

/// An OS call failed with this error code (`errno` on unix,
/// `GetLastError` or an NTSTATUS on Windows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsError(pub i64);
