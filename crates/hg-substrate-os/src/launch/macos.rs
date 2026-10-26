//! macOS launch (E-R3; D4's `sys::spawn`): the broker spawns each child
//! itself, suspended, with its own bootstrap port and no other inherited
//! right, stdio on `/dev/null`, an empty environment and the kind's
//! token as its only argument; while it is suspended the launcher reads
//! its audit token and checks its code-directory hash against the one
//! recorded at broker start (review minor), then resumes it and answers
//! its hello with the launch channels. The child applies the SBPL
//! profile itself, first thing, and probes it.

mod sys;

use std::collections::BTreeMap;
use std::ffi::CString;
use std::io;
use std::os::fd::{AsFd, OwnedFd};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use hg_launch_plan::{Exe, Kind};
use hg_substrate::Substrate;
use hg_substrate::ipc::OsPeer;
use hg_substrate::launch::{
    ChildKind, LaunchError, Launched, Launcher, SandboxReached, SandboxTier,
};

use crate::ipc::macos::{MacIpc, sys as mach};
use crate::launch::ChildError;
use crate::{Threads, sandbox};

/// One executable the broker launches: held open from its start (DU
/// 13.1), and the code-directory hash it had then.
#[derive(Debug)]
struct Pinned {
    file: OwnedFd,
    cdhash: Vec<u8>,
}

/// Opens `path` and records its code-directory hash. The hash is read by
/// path, so the path must name the opened file before and after the
/// read (same device and inode): a file swapped in between is refused.
fn pin(path: &Path) -> io::Result<Pinned> {
    use std::os::unix::fs::MetadataExt;
    let file = std::fs::File::open(path)?;
    let held = file.metadata()?;
    let same =
        || std::fs::metadata(path).is_ok_and(|m| m.dev() == held.dev() && m.ino() == held.ino());
    if !same() {
        return Err(io::Error::other(
            "the executable changed while it was opened",
        ));
    }
    let cdhash = sys::file_cdhash(path.as_os_str().as_encoded_bytes())
        .ok_or_else(|| io::Error::other("no code signature"))?;
    if !same() {
        return Err(io::Error::other(
            "the executable changed while it was hashed",
        ));
    }
    Ok(Pinned {
        file: file.into(),
        cdhash,
    })
}

/// The executables a macOS broker opens at its start.
#[derive(Debug)]
pub struct BuildFiles {
    exes: [Pinned; 2],
}

impl BuildFiles {
    /// Opens each [`Exe`] in `dir` and records its code-directory hash.
    ///
    /// # Errors
    /// A file cannot be opened, or has no code signature.
    pub fn open(dir: &Path) -> io::Result<Self> {
        let [r, n] = Exe::ALL;
        Ok(Self {
            exes: [
                pin(&dir.join(r.file_name()))?,
                pin(&dir.join(n.file_name()))?,
            ],
        })
    }

    /// The file `child` standing in for every child executable (tests).
    #[cfg(test)]
    pub(crate) fn for_test(child: &Path) -> io::Result<Self> {
        Ok(Self {
            exes: [pin(child)?, pin(child)?],
        })
    }

    fn pinned(&self, e: Exe) -> Option<&Pinned> {
        Exe::ALL
            .iter()
            .position(|&x| x == e)
            .and_then(|i| self.exes.get(i))
    }
}

/// The broker's launcher on macOS.
pub struct MacLauncher {
    ipc: Arc<MacIpc>,
    files: BuildFiles,
    devnull: OwnedFd,
    st: Mutex<State>,
    halted: AtomicBool,
}

impl core::fmt::Debug for MacLauncher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MacLauncher").finish_non_exhaustive()
    }
}

#[derive(Default)]
struct State {
    /// Each child not yet reaped, by its identity: its pid. A pid is ours
    /// until we reap it, so signalling one in this map never reaches
    /// another process.
    children: BTreeMap<OsPeer, libc::pid_t>,
}

impl State {
    /// Reaps children that have ended.
    fn sweep(&mut self) {
        self.children
            .retain(|_p, &mut pid| matches!(mach::reap_if_ended(pid), Ok(false)));
    }
}

const fn plan_kind(k: ChildKind) -> Kind {
    match k {
        ChildKind::Renderer => Kind::Renderer,
        ChildKind::NetworkSystem => Kind::NetworkSystem,
        ChildKind::NetworkContainer => Kind::NetworkContainer,
        ChildKind::NetworkPartition => Kind::NetworkPartition,
    }
}

impl MacLauncher {
    /// A launcher for `ipc`'s process over `files`.
    ///
    /// # Errors
    /// `/dev/null` cannot be opened.
    pub(crate) fn new(ipc: Arc<MacIpc>, files: BuildFiles) -> io::Result<Self> {
        let devnull = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")?
            .into();
        Ok(Self {
            ipc,
            files,
            devnull,
            st: Mutex::new(State::default()),
            halted: AtomicBool::new(false),
        })
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.st.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether `peer` is a child this launcher holds (tests).
    #[cfg(test)]
    pub(crate) fn pid_of(&self, peer: OsPeer) -> Option<libc::pid_t> {
        self.lock().children.get(&peer).copied()
    }
}

impl Launcher for MacLauncher {
    fn launch(&self, kind: ChildKind, capacity: u32) -> Result<Launched, LaunchError> {
        if self.halted.load(Ordering::SeqCst) {
            return Err(LaunchError::RestartRequired);
        }
        if capacity != crate::ipc::macos::BOOTSTRAP_CAPACITY {
            return Err(LaunchError::Unsupported);
        }
        // The plan, the executable and the token from one value (E-R7).
        let spec = plan_kind(kind).spec();
        let pinned = self.files.pinned(spec.exe()).ok_or(LaunchError::Failed)?;
        let path = sys::path_of(pinned.file.as_fd()).ok_or(LaunchError::Failed)?;
        let token = CString::new(spec.token()).map_err(|_e| LaunchError::Failed)?;
        self.lock().sweep();
        let boot = mach::allocate_receive().map_err(|_e| LaunchError::Failed)?;
        if mach::make_send(boot).is_err() {
            mach::destroy_receive(boot);
            return Err(LaunchError::Failed);
        }
        let expected = &pinned.cdhash;
        let verify = |pid: libc::pid_t| sys::process_cdhash(pid).is_some_and(|h| &h == expected);
        let opts = mach::SpawnOpts {
            stdio: Some(self.devnull.as_fd()),
            verify: Some(&verify),
        };
        let spawned = mach::spawn(&path, &[token.as_c_str()], &[], boot, &opts);
        // The child holds its own send right now (or none exists).
        mach::deallocate(boot);
        let child = match spawned {
            Ok(c) => c,
            Err(_e) => {
                mach::destroy_receive(boot);
                return Err(LaunchError::Failed);
            }
        };
        let peer = OsPeer::new(child.token.pid, u64::from(child.token.version));
        let Some((from_child, to_child)) = self.ipc.adopt_child(boot, child.token) else {
            mach::kill_and_reap(child.pid);
            return Err(LaunchError::Failed);
        };
        self.lock().children.insert(peer, child.pid);
        Ok(Launched {
            from_child,
            to_child,
            peer,
            sandbox: SandboxTier::MacosSbpl,
        })
    }

    fn kill(&self, peer: OsPeer) {
        // Only a child not yet reaped: its pid cannot have been reused.
        if let Some(pid) = self.lock().children.remove(&peer) {
            mach::kill_and_reap(pid);
        }
    }

    fn restart_required(&self) {
        self.halted.store(true, Ordering::SeqCst);
    }
}

/// The broker substrate on macOS with the launcher over `files`.
pub(crate) fn broker_with(
    threads: Threads,
    files: BuildFiles,
) -> Result<Substrate, crate::StartError> {
    let (entropy, exec) = crate::base(threads)?;
    let ipc = Arc::new(match MacIpc::start(Arc::clone(&exec), false) {
        Ok(t) => t,
        Err(crate::ipc::macos::StartError::Os(e)) => return Err(crate::StartError::Ipc(e)),
        Err(crate::ipc::macos::StartError::Bootstrap) => {
            return Err(crate::StartError::Ipc(io::Error::other(
                "no launch expected",
            )));
        }
    });
    let launcher = MacLauncher::new(Arc::clone(&ipc), files).map_err(crate::StartError::Launch)?;
    Ok(Substrate {
        clock: Arc::new(crate::clock::OsClock::new()),
        entropy: Arc::new(entropy),
        exec,
        mem: Arc::new(crate::memory::OsMemory::new()),
        ipc,
        os: None,
        launcher: Some(Arc::new(launcher)),
    })
}

/// The child's start on macOS: the SBPL profile first (the hello and
/// everything after run inside it), its probe, then the substrate.
///
/// # Errors
/// [`ChildError`].
pub(crate) fn child_start(threads: Threads) -> Result<(Substrate, SandboxReached), ChildError> {
    sandbox::macos::apply_empty_process_profile().map_err(ChildError::Sandbox)?;
    if !sandbox::macos::floor_holds() {
        return Err(ChildError::Sandbox(sandbox::macos::probe_line()));
    }
    let s = crate::os_substrate_launched(threads).map_err(ChildError::Start)?;
    Ok((
        s,
        SandboxReached {
            tier: SandboxTier::MacosSbpl,
            missing: Vec::new(),
        },
    ))
}

#[cfg(test)]
mod tests;
