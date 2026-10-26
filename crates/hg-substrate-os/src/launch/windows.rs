//! Windows launch (E-R3, review M9): D5's `launch` with the FP-M0
//! sandbox merged in. Each child gets its own AppContainer (one SID per
//! live child), the lockdown token and its own job (kill on close, at
//! most one active process), assigned while it is still suspended; one
//! inherited handle, its link to the broker; a minimal environment; and
//! the kind's token as its only argument. The build directory and the
//! executables are held open from broker start, shared for reading only.
//! The child lowers itself to Untrusted first thing and probes.

mod sys;

/// FP-27's phase timestamps (measurement builds only).
#[cfg(feature = "measure-launch")]
pub use sys::timing;

/// Stamps a launch phase in a measurement build; nothing otherwise.
macro_rules! stamp {
    ($phase:literal) => {
        #[cfg(feature = "measure-launch")]
        sys::timing::mark($phase);
    };
}

use std::collections::BTreeMap;
use std::io;
use std::os::windows::io::OwnedHandle;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use hg_launch_plan::{Exe, Kind};
use hg_substrate::Substrate;
use hg_substrate::ipc::OsPeer;
use hg_substrate::launch::{
    ChildKind, LaunchError, Launched, Launcher, SandboxFeature, SandboxReached, SandboxTier,
};

use crate::ipc::windows::{LaunchSpec, Sandbox, WindowsIpc, sys as win};
use crate::launch::ChildError;
use crate::sandbox::windows as sb;
use crate::{Threads, sandbox};

/// The executables a Windows broker opens at its start.
#[derive(Debug)]
pub struct BuildFiles {
    /// Held: the directory cannot be renamed or deleted.
    _dir: OwnedHandle,
    /// Held: the executables cannot be written, renamed or deleted.
    _held: [OwnedHandle; 2],
    paths: [PathBuf; 2],
}

impl BuildFiles {
    /// Opens the build directory `dir` and each [`Exe`] in it.
    ///
    /// # Errors
    /// The failing open.
    pub fn open(dir: &Path) -> io::Result<Self> {
        let [r, n] = Exe::ALL.map(|e| dir.join(format!("{}.exe", e.file_name())));
        Self::from_paths(dir, [r, n])
    }

    fn from_paths(dir: &Path, paths: [PathBuf; 2]) -> io::Result<Self> {
        let [a, b] = &paths;
        Ok(Self {
            _dir: sys::hold(dir, true)?,
            _held: [sys::hold(a, false)?, sys::hold(b, false)?],
            paths,
        })
    }

    /// The executable `child` standing in for every kind (tests).
    #[cfg(test)]
    pub(crate) fn for_test(child: &Path) -> io::Result<Self> {
        let dir = child.parent().unwrap_or(child);
        Self::from_paths(dir, [child.to_path_buf(), child.to_path_buf()])
    }

    fn path(&self, e: Exe) -> Option<&Path> {
        Exe::ALL
            .iter()
            .position(|&x| x == e)
            .and_then(|i| self.paths.get(i))
            .map(PathBuf::as_path)
    }
}

/// One live child.
struct Entry {
    child: win::spawn::Child,
    job: sb::OwnedHandle,
    container: String,
}

impl Entry {
    /// Ends it: its job first (every process in it), then waits and
    /// unregisters its container.
    fn end(self) {
        sys::terminate_job(&self.job);
        let _ = win::spawn::wait(&self.child, 5_000);
        win::spawn::delete_container(&self.container);
    }
}

/// The broker's launcher on Windows.
pub struct WinLauncher {
    ipc: Arc<WindowsIpc>,
    files: BuildFiles,
    children: Mutex<BTreeMap<OsPeer, Entry>>,
    launches: AtomicU64,
    halted: AtomicBool,
}

impl core::fmt::Debug for WinLauncher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WinLauncher").finish_non_exhaustive()
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

impl WinLauncher {
    pub(crate) fn new(ipc: Arc<WindowsIpc>, files: BuildFiles) -> Self {
        Self {
            ipc,
            files,
            children: Mutex::new(BTreeMap::new()),
            launches: AtomicU64::new(0),
            halted: AtomicBool::new(false),
        }
    }

    fn lock(&self) -> MutexGuard<'_, BTreeMap<OsPeer, Entry>> {
        self.children.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Unregisters the containers of children that have ended.
    fn sweep(&self) {
        let mut map = self.lock();
        let ended: Vec<OsPeer> = map
            .iter()
            .filter(|(_p, e)| win::spawn::wait(&e.child, 0).is_some())
            .map(|(p, _e)| *p)
            .collect();
        let gone: Vec<Entry> = ended.iter().filter_map(|p| map.remove(p)).collect();
        drop(map);
        for e in gone {
            e.end();
        }
    }

    /// Whether `peer` is a live child of this launcher (tests).
    #[cfg(test)]
    pub(crate) fn exit_code(&self, peer: OsPeer, wait_ms: u32) -> Option<u32> {
        self.lock()
            .get(&peer)
            .and_then(|e| win::spawn::wait(&e.child, wait_ms))
    }
}

impl Drop for WinLauncher {
    fn drop(&mut self) {
        // The jobs would end the children on close anyway; ending them
        // here also unregisters their containers, which persist per user.
        let all = std::mem::take(&mut *self.lock());
        for e in all.into_values() {
            e.end();
        }
    }
}

impl Launcher for WinLauncher {
    fn launch(&self, kind: ChildKind, capacity: u32) -> Result<Launched, LaunchError> {
        if self.halted.load(Ordering::SeqCst) {
            return Err(LaunchError::RestartRequired);
        }
        if capacity != crate::ipc::windows::BOOTSTRAP_CAPACITY {
            return Err(LaunchError::Unsupported);
        }
        #[cfg(feature = "measure-launch")]
        let _ = sys::timing::take();
        stamp!("request");
        self.sweep();
        stamp!("sweep");
        // The plan, the executable and the token from one value (E-R7).
        let spec = plan_kind(kind).spec();
        let exe = self.files.path(spec.exe()).ok_or(LaunchError::Failed)?;
        let n = self.launches.fetch_add(1, Ordering::SeqCst).wrapping_add(1);
        let container = format!("izcal.{}.{}.{n}", spec.token(), std::process::id());
        let sid = sb::app_container(&container, &container).map_err(|_e| LaunchError::Failed)?;
        stamp!("profile");
        let fail = |e: LaunchError| {
            win::spawn::delete_container(&container);
            e
        };
        let token = sb::restricted_token().map_err(|_e| fail(LaunchError::Failed))?;
        stamp!("token");
        let job = sb::job_object().map_err(|_e| fail(LaunchError::Failed))?;
        stamp!("job");
        let launched = self
            .ipc
            .launch(LaunchSpec {
                exe,
                args: spec.token(),
                env: Vec::new(),
                sandbox: Sandbox::Contained {
                    container: &sid,
                    token: &token,
                    job: &job,
                },
            })
            .map_err(|_e| fail(LaunchError::Failed))?;
        let peer = OsPeer::new(launched.peer.pid(), n);
        stamp!("launched");
        #[cfg(feature = "measure-launch")]
        if let Ok(t) = win::spawn::thread_handle(&launched.child) {
            sys::timing::record(spec.token(), launched.peer.pid(), t);
        }
        self.lock().insert(
            peer,
            Entry {
                child: launched.child,
                job,
                container,
            },
        );
        Ok(Launched {
            from_child: launched.from_child,
            to_child: launched.to_child,
            peer,
            sandbox: SandboxTier::WindowsAppContainer,
        })
    }

    fn kill(&self, peer: OsPeer) {
        // Only a child this launcher holds, through its own job.
        let entry = self.lock().remove(&peer);
        if let Some(e) = entry {
            e.end();
        }
    }

    fn restart_required(&self) {
        self.halted.store(true, Ordering::SeqCst);
    }
}

/// The broker substrate on Windows with the launcher over `files`.
pub(crate) fn broker_with(
    threads: Threads,
    files: BuildFiles,
) -> Result<Substrate, crate::StartError> {
    let (entropy, exec) = crate::base(threads)?;
    let ipc = Arc::new(match WindowsIpc::start(Arc::clone(&exec), None) {
        Ok(t) => t,
        Err(crate::ipc::windows::StartError::Os(e)) => return Err(crate::StartError::Ipc(e)),
        Err(crate::ipc::windows::StartError::Bootstrap) => {
            return Err(crate::StartError::Ipc(io::Error::other(
                "no launch expected",
            )));
        }
    });
    let launcher = WinLauncher::new(Arc::clone(&ipc), files);
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

/// The child's start on Windows: lower to Untrusted first, check it runs
/// in its own container, then take the link and start the substrate.
///
/// # Errors
/// [`ChildError`].
pub(crate) fn child_start(threads: Threads) -> Result<(Substrate, SandboxReached), ChildError> {
    stamp!("entry");
    sandbox::windows::apply_empty_process_profile().map_err(ChildError::Sandbox)?;
    stamp!("untrusted");
    if win::app_container_sid().is_none() {
        return Err(ChildError::Sandbox("not in an AppContainer".into()));
    }
    let link = std::env::var(crate::ipc::windows::LINK_VAR)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .and_then(|v| win::adopt_pipe(v, false))
        .ok_or(ChildError::Arguments)?;
    stamp!("link");
    let s = crate::os_substrate_launched(threads, link).map_err(ChildError::Start)?;
    stamp!("substrate");
    #[cfg(feature = "measure-launch")]
    sys::timing::publish();
    Ok((
        s,
        SandboxReached {
            tier: SandboxTier::WindowsAppContainer,
            // The M5 parts of the 6.7 row (E-Q3).
            missing: vec![
                SandboxFeature::AlternateDesktop,
                SandboxFeature::Win32kLockdown,
                SandboxFeature::CodeIntegrityGuard,
            ],
        },
    ))
}

#[cfg(test)]
mod tests;
