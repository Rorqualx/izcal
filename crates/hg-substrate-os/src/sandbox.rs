//! Sandbox backends, one per OS behind a trait (security scope 6.7).
//!
//! Every process **opens what it needs before it locks down, then
//! receives everything else as pre-opened handles from the broker**; no
//! process opens a path after start-up. At start-up each process probes
//! and records the **sandbox tier it achieved**.

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;

/// Applies the empty-process profile on this OS (the full sandbox of
/// security scope 6.7 for a process that has two threads and one
/// pre-opened IPC endpoint and opens nothing after lockdown).
pub fn apply() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return macos::apply_empty_process_profile();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::apply_empty_process_profile();
    }
    #[cfg(windows)]
    {
        // The Windows tier restricts the process from birth: the
        // supervisor starts it under the restricted untrusted token,
        // so there is no post-start lockdown to apply here.
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("hg-sandbox: no backend for this OS".to_string())
}

/// The behavioral tier probe for this OS, as one ready-line segment.
/// The probes are black-box: they perform the operations the tier must
/// deny (or must allow) and report what actually happened, so a
/// misbehaving profile shows up as data, never silently (security 6.7).
pub fn probe_line() -> String {
    #[cfg(target_os = "macos")]
    {
        return macos::probe_line();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::probe_line();
    }
    #[cfg(windows)]
    {
        return windows::probe_line();
    }
    #[allow(unreachable_code)]
    "no-probe".to_string()
}

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
