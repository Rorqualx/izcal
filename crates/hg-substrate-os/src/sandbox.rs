//! Sandbox backends, one per OS behind a trait (security scope 6.7).
//!
//! Every process **opens what it needs before it locks down, then
//! receives everything else as pre-opened handles from the broker**; no
//! process opens a path after start-up. At start-up each process probes
//! and records the **sandbox tier it achieved**.

#[cfg(target_os = "macos")]
pub mod macos;

/// What the tier probe found, one row per probed operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The operation is denied by the applied profile.
    Denied,
    /// The operation is allowed by the applied profile.
    Allowed,
    /// The probe itself failed (the sandbox API returned an error).
    ProbeFailed,
}

impl Verdict {
    /// The one-letter wire form used in the ready line.
    pub const fn as_char(self) -> char {
        match self {
            Self::Denied => 'D',
            Self::Allowed => 'A',
            Self::ProbeFailed => '?',
        }
    }
}
