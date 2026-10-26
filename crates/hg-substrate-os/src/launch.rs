//! Launching child processes (M1 phase E; launch design E-R2 to E-R7):
//! the per-OS launch paths behind `hg_substrate::Launcher`, and the
//! child's side of a launch ([`os_child_substrate`]).

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(test)]
pub(crate) mod test_bins;
#[cfg(windows)]
pub mod windows;

use std::ffi::OsString;

pub use hg_launch_plan::Exe;
use hg_launch_plan::Kind;
use hg_substrate::Substrate;
use hg_substrate::launch::{ChildKind, SandboxReached};

use crate::{StartError, Threads};

/// A launched child's start: its substrate, what it is, and the sandbox
/// its own probes found.
pub struct Child {
    /// The substrate, with the bootstrap channels.
    pub substrate: Substrate,
    /// The kind its launcher named (`argv[1]`, E-R7 disposition).
    pub kind: ChildKind,
    /// What the child's probes found after lockdown (E-R5).
    pub sandbox: SandboxReached,
}

impl core::fmt::Debug for Child {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Child")
            .field("kind", &self.kind)
            .field("sandbox", &self.sandbox)
            .finish_non_exhaustive()
    }
}

/// Why a child could not start. Each has its own exit code, and every one
/// ends the process before `ready`.
#[derive(Debug)]
pub enum ChildError {
    /// The arguments are not exactly this executable's name and one token
    /// of a kind it runs (E-R7 disposition).
    Arguments,
    /// The sandbox could not be applied, or the probes show less than the
    /// floor (review M2: refused, never downgraded).
    Sandbox(String),
    /// The launch descriptors or the substrate failed.
    Start(StartError),
    /// No launch path on this OS yet.
    Unsupported,
}

impl ChildError {
    /// The process exit code for this failure.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Arguments => 64,
            Self::Sandbox(_) => 65,
            Self::Start(_) => 66,
            Self::Unsupported => 67,
        }
    }
}

impl core::fmt::Display for ChildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Arguments => f.write_str("not launched as a kind this executable runs"),
            Self::Sandbox(e) => write!(f, "sandbox: {e}"),
            Self::Start(e) => write!(f, "start: {e}"),
            Self::Unsupported => f.write_str("no launch path on this OS"),
        }
    }
}

/// The kind named by `args` (after `argv[0]`): exactly one token of a
/// kind `exe` runs, nothing else (E-R7 disposition).
///
/// # Errors
/// [`ChildError::Arguments`] for a missing, extra or wrong argument.
pub fn kind_from_args(
    exe: Exe,
    mut args: impl Iterator<Item = OsString>,
) -> Result<ChildKind, ChildError> {
    let (Some(token), None) = (args.next(), args.next()) else {
        return Err(ChildError::Arguments);
    };
    let kind = Kind::from_token(exe, token.as_encoded_bytes()).ok_or(ChildError::Arguments)?;
    Ok(match kind {
        Kind::Renderer => ChildKind::Renderer,
        Kind::NetworkSystem => ChildKind::NetworkSystem,
        Kind::NetworkContainer => ChildKind::NetworkContainer,
        Kind::NetworkPartition => ChildKind::NetworkPartition,
    })
}

/// The start of a process its broker launched as one of `exe`'s kinds:
/// call it first thing in `main`, before anything else runs. It reads
/// the kind from the arguments (`main` never sees them), locks the
/// process down, probes the result, and starts the substrate with the
/// bootstrap channels.
///
/// # Errors
/// [`ChildError`]; the caller exits with its code.
pub fn os_child_substrate(threads: Threads, exe: Exe) -> Result<Child, ChildError> {
    let kind = kind_from_args(exe, std::env::args_os().skip(1))?;
    #[cfg(target_os = "linux")]
    {
        let (substrate, sandbox) = linux::child::start(threads)?;
        Ok(Child {
            substrate,
            kind,
            sandbox,
        })
    }
    #[cfg(target_os = "macos")]
    {
        let (substrate, sandbox) = macos::child_start(threads)?;
        Ok(Child {
            substrate,
            kind,
            sandbox,
        })
    }
    #[cfg(windows)]
    {
        let (substrate, sandbox) = windows::child_start(threads)?;
        Ok(Child {
            substrate,
            kind,
            sandbox,
        })
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        let _ = (threads, kind);
        Err(ChildError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> impl Iterator<Item = OsString> {
        a.iter().map(OsString::from).collect::<Vec<_>>().into_iter()
    }

    /// E-R7 disposition: exactly one token, of a kind this executable runs.
    #[test]
    fn the_kind_comes_from_exactly_one_token_of_this_executable() {
        for k in Kind::ALL {
            let got = kind_from_args(k.exe(), args(&[k.token()])).unwrap();
            assert_eq!(format!("{got:?}"), format!("{k:?}"));
            let other = if k.exe() == Exe::Renderer {
                Exe::Net
            } else {
                Exe::Renderer
            };
            assert!(kind_from_args(other, args(&[k.token()])).is_err());
            assert!(kind_from_args(k.exe(), args(&[k.token(), "x"])).is_err());
        }
        assert!(kind_from_args(Exe::Net, args(&[])).is_err());
        assert!(kind_from_args(Exe::Net, args(&["--kind=2"])).is_err());
        assert_eq!(ChildError::Arguments.exit_code(), 64);
    }
}
