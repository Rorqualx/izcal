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
use crate::launch::{ChildError, MainThread};
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

/// How many fresh AppContainer profiles the launcher keeps registered
/// ahead of their launches (†, set from first principles: the browser
/// starts two kinds at once).
const SPARE_PROFILES: usize = 2;

/// One fresh profile, registered and never used.
struct Spare {
    name: String,
    sid: sb::AppContainerSid,
}

/// Fresh AppContainer profiles made ahead of the launches that use them
/// (M1 phase G). Registering a profile (`CreateAppContainerProfile`)
/// took about 5 ms of FP-27's 11 ms on rig253, inside the launch; it
/// does not depend on the launch, so it is done before: at broker start,
/// and on a background thread after each launch takes one. Each child
/// still gets its own profile and SID, never used by any process before
/// it (D5: one SID per live child; nothing is reused). A launch that
/// finds none registers one itself, as before.
struct Spares {
    ready: Mutex<Vec<Spare>>,
    /// Set when the launcher ends: profiles made after it are removed.
    closed: AtomicBool,
    /// A refill thread is running.
    refilling: AtomicBool,
    /// The latest refill thread, joined by [`Spares::close`] so that no
    /// registration outlives the launcher.
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}

/// The number in the next profile's name: one counter for the process,
/// so two launchers in one process never name the same profile.
static NEXT_PROFILE: AtomicU64 = AtomicU64::new(0);

/// The prefix of every profile a launcher registers. A name is
/// `izcal.l.<pid>.<start>.<n>`: the broker process's id and creation time
/// (`GetProcessTimes`) name the broker instance that owns it, and `n`
/// counts its profiles. The pair tells a crashed broker's profiles apart
/// from a live one's, even after the pid is reused.
const PROFILE_PREFIX: &str = "izcal.l.";

/// This process's creation time, as profile names carry it.
fn own_start() -> Option<u64> {
    static START: std::sync::OnceLock<Option<u64>> = std::sync::OnceLock::new();
    *START.get_or_init(|| sys::running_since(std::process::id()))
}

/// The owner (pid, creation time) a profile name of this scheme names.
fn owner_of(name: &str) -> Option<(u32, u64)> {
    let mut parts = name.strip_prefix(PROFILE_PREFIX)?.split('.');
    let pid = parts.next()?.parse().ok()?;
    let start = parts.next()?.parse().ok()?;
    parts.next()?.parse::<u64>().ok()?;
    parts.next().is_none().then_some((pid, start))
}

/// The folder Windows keeps for each registered AppContainer profile of
/// this user, named for the profile (`%LOCALAPPDATA%\Packages\<name>`).
fn profile_folders() -> Vec<String> {
    let Some(base) = std::env::var_os("LOCALAPPDATA") else {
        return Vec::new();
    };
    std::fs::read_dir(Path::new(&base).join("Packages"))
        .map(|d| {
            d.filter_map(Result::ok)
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Unregisters the profiles of broker instances that are gone: names of
/// this scheme whose owner is no running process with that id and
/// creation time (a crashed broker; its children ended with it, through
/// their kill-on-close jobs). A live broker's profiles, this one's
/// included, and every name outside the scheme are left alone. Returns
/// the names removed.
pub(crate) fn sweep_stale_profiles() -> Vec<String> {
    let mut removed = Vec::new();
    for name in profile_folders() {
        let Some((pid, start)) = owner_of(&name) else {
            continue;
        };
        if sys::running_since(pid) == Some(start) {
            continue;
        }
        win::spawn::delete_container(&name);
        removed.push(name);
    }
    removed
}

impl Spares {
    fn lock(&self) -> MutexGuard<'_, Vec<Spare>> {
        self.ready.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Registers a new profile, named for this process and a counter.
    /// The profile must be new: one left registered under the same name
    /// (an earlier broker with this pid that ended without removing its
    /// profiles) may have held a child, so it is removed and registered
    /// again, empty, never handed out as it was.
    fn make(&self) -> Option<Spare> {
        let n = NEXT_PROFILE.fetch_add(1, Ordering::SeqCst).wrapping_add(1);
        let start = own_start()?;
        Self::register_fresh(format!(
            "{PROFILE_PREFIX}{}.{start}.{n}",
            std::process::id()
        ))
    }

    /// Registers profile `name` new, removing one left under that name
    /// first (see [`Spares::make`]).
    fn register_fresh(name: String) -> Option<Spare> {
        let sid = match sb::new_app_container(&name).ok()? {
            Some(sid) => sid,
            None => {
                win::spawn::delete_container(&name);
                sb::new_app_container(&name).ok()??
            }
        };
        Some(Spare { name, sid })
    }

    fn take(&self) -> Option<Spare> {
        self.lock().pop()
    }

    /// Keeps `s`, unless the launcher has ended (then removes it).
    fn put(&self, s: Spare) {
        let mut v = self.lock();
        if self.closed.load(Ordering::SeqCst) {
            drop(v);
            win::spawn::delete_container(&s.name);
        } else {
            v.push(s);
        }
    }

    /// Registers profiles until [`SPARE_PROFILES`] are ready; `false` if
    /// a registration failed.
    fn fill(&self) -> bool {
        while !self.closed.load(Ordering::SeqCst) && self.lock().len() < SPARE_PROFILES {
            match self.make() {
                Some(s) => self.put(s),
                None => return false,
            }
        }
        true
    }

    /// Whether the pool is short and open.
    fn short(&self) -> bool {
        !self.closed.load(Ordering::SeqCst) && self.lock().len() < SPARE_PROFILES
    }

    /// Refills on a thread of its own, off the launch path.
    fn refill(self: &Arc<Self>) {
        if self.refilling.swap(true, Ordering::SeqCst) {
            return;
        }
        let me = Arc::clone(self);
        // Spawned and stored under the lock `close` takes to join, so a
        // `close` can never miss a thread that has started.
        let mut worker = self.worker.lock().unwrap_or_else(PoisonError::into_inner);
        let started = crate::thread::spawn("hg-profile-spares", move || {
            loop {
                let filled = me.fill();
                me.refilling.store(false, Ordering::SeqCst);
                // A take after fill's last check saw `refilling` set and
                // left the refill to this thread: look again. Stop if a
                // registration failed (the next launch tries again) or
                // another refill took over.
                if !filled || !me.short() || me.refilling.swap(true, Ordering::SeqCst) {
                    break;
                }
            }
        });
        match started {
            Ok(h) => *worker = Some(h),
            Err(_e) => self.refilling.store(false, Ordering::SeqCst),
        }
    }

    /// The launcher ends: no more spares, and the unused ones removed.
    fn close(&self) {
        let mut v = self.lock();
        self.closed.store(true, Ordering::SeqCst);
        let left: Vec<Spare> = v.drain(..).collect();
        drop(v);
        for s in left {
            win::spawn::delete_container(&s.name);
        }
        // A registration in flight finishes, sees `closed` and removes
        // what it made; waiting for it keeps a process that exits next
        // from leaving that profile behind (at most one registration).
        let worker = self
            .worker
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(h) = worker {
            let _ = h.join();
        }
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
    spares: Arc<Spares>,
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
        let spares = Arc::new(Spares {
            ready: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
            refilling: AtomicBool::new(false),
            worker: Mutex::new(None),
        });
        // At broker start, before any launch is requested: first remove
        // what crashed brokers left, then register this one's spares.
        let _ = sweep_stale_profiles();
        spares.fill();
        Self {
            ipc,
            files,
            children: Mutex::new(BTreeMap::new()),
            spares,
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
        // here also unregisters their containers, which persist per user,
        // and so are the spares'.
        self.spares.close();
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
        // A fresh profile made ahead, or (none ready) one made now.
        let spare = self.spares.take().or_else(|| self.spares.make());
        let Spare {
            name: container,
            sid,
        } = spare.ok_or(LaunchError::Failed)?;
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
        // The next spare, made after this child is on its way.
        self.spares.refill();
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
pub(crate) fn child_start(
    threads: Threads,
) -> Result<(Substrate, MainThread, SandboxReached), ChildError> {
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
    let (s, main) = crate::os_substrate_launched(threads, link).map_err(ChildError::Start)?;
    stamp!("substrate");
    #[cfg(feature = "measure-launch")]
    sys::timing::publish();
    Ok((
        s,
        main,
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
