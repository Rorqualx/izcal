//! The child's side of a Linux launch (E-R3, E-R5): it runs under the
//! launch filter the spawner installed, with its bootstrap ends at 3 and
//! 4. It applies the steady filter before any thread exists, probes what
//! it reached, and starts the substrate.

use hg_substrate::Substrate;
use hg_substrate::launch::{SandboxFeature, SandboxReached, SandboxTier};

use super::sys;
use crate::launch::ChildError;
use crate::{Threads, ipc, sandbox};

/// Locks down, probes and starts. Runs first in `main`.
///
/// # Errors
/// [`ChildError`].
pub(crate) fn start(threads: Threads) -> Result<(Substrate, SandboxReached), ChildError> {
    // The ends first: nothing else in this process has seen these numbers.
    let (fd_up, fd_down) = ipc::linux::BOOTSTRAP_FDS;
    let (Some(up), Some(down)) = (sys::take_inherited(fd_up), sys::take_inherited(fd_down)) else {
        return Err(ChildError::Arguments);
    };
    sandbox::linux::apply_launched_profile().map_err(ChildError::Sandbox)?;
    let reached = probe()?;
    let s = crate::os_substrate_launched(threads, up, down).map_err(ChildError::Start)?;
    Ok((s, reached))
}

/// What this process reached, by black-box probes (E-R5). Below the
/// floor (no filter, or a file open that is not refused) is an error.
fn probe() -> Result<SandboxReached, ChildError> {
    if sandbox::linux::seccomp_mode() != 2 {
        return Err(ChildError::Sandbox("no seccomp filter".into()));
    }
    let line = sandbox::linux::probe_line();
    if line != "openat-errno=1 socket-errno=1" {
        return Err(ChildError::Sandbox(line));
    }
    // A PID namespace makes this process pid 1 (launch facts E15); an
    // empty root hides /etc/hosts.
    let namespaced = std::process::id() == 1;
    let root_visible = sandbox::linux::host_file_visible()
        .ok_or_else(|| ChildError::Sandbox("the root probe was refused".into()))?;
    let mut missing = Vec::new();
    let tier = match (namespaced, root_visible) {
        (true, false) => SandboxTier::LinuxFull,
        (true, true) => {
            missing.extend([
                SandboxFeature::EmptyRoot,
                SandboxFeature::IdMap,
                SandboxFeature::MountNamespace,
            ]);
            SandboxTier::LinuxRestrictedUserns
        }
        (false, _) => {
            missing.extend([
                SandboxFeature::EmptyRoot,
                SandboxFeature::IdMap,
                SandboxFeature::MountNamespace,
            ]);
            SandboxTier::LinuxNoUserns
        }
    };
    // No kernel this runs on has Landlock UDP rules yet.
    missing.push(SandboxFeature::LandlockUdp);
    Ok(SandboxReached { tier, missing })
}
