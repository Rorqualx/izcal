//! Izcal's OS substrate (tier O, G3 — coding standard 0.3.7 tier table).
//!
//! The one crate allowed ambient OS effects. Holds the real-OS
//! implementation of the substrate traits (Stage 1 design 3.1), the
//! sandbox backends (security scope 6.7: `hg-sandbox` lives inside
//! `hg-substrate-os`), the process-kind machinery and the sanctioned
//! thread interface.
//!
//! - FP-M0 half B (23 October 2026): the sandbox backends, the tier
//!   probe, and the two FP-M0 binaries (`fp-m0-probe`, the supervisor,
//!   and `fp-m0-empty`, the sandboxed empty process).
//! - M1 phase B: [`os_substrate`] — the OS clock, entropy, executor and
//!   memory.

pub mod clock;
pub mod entropy;
pub mod executor;
pub mod install;
pub mod ipc;
pub mod launch;
pub mod memory;
pub mod os;
pub mod sandbox;
pub mod thread;

use std::sync::Arc;

use hg_substrate::{Entropy, Substrate};

/// Worker counts for [`os_substrate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Threads {
    /// Workers for the three ordinary priorities.
    pub general: usize,
    /// Workers for `MayBlock` sequences.
    pub may_block: usize,
}

impl Threads {
    /// One general worker per available core, at least 2 and at most 16†,
    /// and 2† `MayBlock` workers.
    pub fn for_this_machine() -> Self {
        let cores = std::thread::available_parallelism().map_or(2, std::num::NonZero::get);
        Self {
            general: cores.clamp(2, 16),
            may_block: 2,
        }
    }
}

/// Why the substrate could not start.
#[derive(Debug)]
pub enum StartError {
    /// The OS random source failed, so no hash key could be drawn.
    Entropy,
    /// A worker thread could not be started.
    Thread(std::io::Error),
    /// The IPC transport could not start (its epoll instance, eventfd or
    /// IO thread).
    Ipc(std::io::Error),
    /// The broker's launcher could not start: the build's executables, or
    /// the Linux spawner.
    Launch(std::io::Error),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entropy => f.write_str("the OS random source failed"),
            Self::Thread(e) => write!(f, "a substrate thread could not start: {e}"),
            Self::Ipc(e) => write!(f, "the IPC transport could not start: {e}"),
            Self::Launch(e) => write!(f, "the launcher could not start: {e}"),
        }
    }
}

impl std::error::Error for StartError {}

/// The process's OS substrate. Call it once, first thing in `main`: it
/// draws the process hash key from the OS (CS-AS-2), which must happen
/// before the process creates any hash map, then starts the executor's
/// threads.
///
/// # Errors
/// [`StartError`] if the OS random source or thread creation fails.
pub fn os_substrate(threads: Threads) -> Result<Substrate, StartError> {
    start(threads, Launch::default())
}

/// The launch channels a process was started with, already checked.
#[cfg(target_os = "linux")]
type Launch = Option<(std::os::fd::OwnedFd, std::os::fd::OwnedFd)>;
/// On macOS, whether the process was launched with them: they are taken
/// from the launcher over the bootstrap port when the transport starts.
#[cfg(target_os = "macos")]
type Launch = bool;
/// The control link to the broker a process was started with.
#[cfg(windows)]
type Launch = Option<std::os::windows::io::OwnedHandle>;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
type Launch = Option<core::convert::Infallible>;

/// [`os_substrate`] for a process its launcher started with launch
/// channels. On Linux these are the descriptors at
/// [`ipc::linux::BOOTSTRAP_FDS`], which the process's entry point takes
/// ownership of (it alone can know nothing else owns them) and passes
/// here; `Ipc::bootstrap` then returns them as channels. Descriptors that
/// are not channel ends are closed, and `bootstrap` returns `None`.
///
/// # Errors
/// As [`os_substrate`].
#[cfg(target_os = "linux")]
pub fn os_substrate_launched(
    threads: Threads,
    up: std::os::fd::OwnedFd,
    down: std::os::fd::OwnedFd,
) -> Result<Substrate, StartError> {
    start(threads, ipc::linux::launch_ends(up, down).ok())
}

/// [`os_substrate`] for a process its launcher started with launch
/// channels. On macOS the launcher made this process's bootstrap port a
/// port of its own; the transport sends a hello there and takes the
/// launch channels from the answer, which `Ipc::bootstrap` then returns.
///
/// # Errors
/// As [`os_substrate`], and [`StartError::Ipc`] if no answer comes.
#[cfg(target_os = "macos")]
pub fn os_substrate_launched(threads: Threads) -> Result<Substrate, StartError> {
    start(threads, true)
}

/// [`os_substrate`] for a process its broker started. On Windows the
/// launcher hands the child one inherited handle, its control link to
/// the broker, whose value it finds in the `IZCAL_IPC_LINK` environment
/// variable; the process's entry point takes ownership of it (it alone
/// can know nothing else owns it) and passes it here. The transport
/// reads the broker's `Hello` (the launch channels) before it returns.
///
/// # Errors
/// As [`os_substrate`]; a link that carries no valid `Hello` is an
/// [`StartError::Ipc`].
#[cfg(windows)]
pub fn os_substrate_launched(
    threads: Threads,
    link: std::os::windows::io::OwnedHandle,
) -> Result<Substrate, StartError> {
    start(threads, Some(link))
}

/// The broker's substrate: [`os_substrate`] plus its launcher (phase E),
/// which starts children from the executables next to this one (the
/// build directory, G251). On Linux the launcher starts the spawner now.
///
/// # Errors
/// As [`os_substrate`], and [`StartError::Launch`] when the build's
/// executables cannot be opened or the spawner does not start.
pub fn os_broker_substrate(threads: Threads) -> Result<Substrate, StartError> {
    let exe = std::env::current_exe().map_err(StartError::Launch)?;
    let dir = exe
        .parent()
        .ok_or_else(|| StartError::Launch(std::io::Error::other("no build directory")))?;
    os_broker_substrate_in(threads, dir)
}

/// [`os_broker_substrate`] with the children from `dir`.
///
/// # Errors
/// As [`os_broker_substrate`].
pub fn os_broker_substrate_in(
    threads: Threads,
    dir: &std::path::Path,
) -> Result<Substrate, StartError> {
    #[cfg(target_os = "linux")]
    {
        let files = launch::linux::BuildFiles::open(dir).map_err(StartError::Launch)?;
        broker_with(threads, files)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = dir;
        os_substrate(threads)
    }
}

/// The Linux broker substrate with the launcher over `files`.
#[cfg(target_os = "linux")]
pub(crate) fn broker_with(
    threads: Threads,
    files: launch::linux::BuildFiles,
) -> Result<Substrate, StartError> {
    let (entropy, exec) = base(threads)?;
    let ipc = Arc::new(match ipc::linux::LinuxIpc::start(Arc::clone(&exec), None) {
        Ok(t) => t,
        Err(ipc::linux::StartError::Os(e)) => return Err(StartError::Ipc(e)),
        Err(ipc::linux::StartError::Bootstrap) => {
            return Err(StartError::Ipc(std::io::Error::other(
                "no launch descriptors expected",
            )));
        }
    });
    let launcher =
        launch::linux::LinuxLauncher::start(Arc::clone(&ipc), files).map_err(StartError::Launch)?;
    Ok(Substrate {
        clock: Arc::new(clock::OsClock::new()),
        entropy: Arc::new(entropy),
        exec,
        mem: Arc::new(memory::OsMemory::new()),
        ipc,
        os: None,
        launcher: Some(Arc::new(launcher)),
    })
}

/// The process hash key and the executor.
fn base(threads: Threads) -> Result<(entropy::OsEntropy, Arc<executor::OsExecutor>), StartError> {
    let entropy = entropy::OsEntropy::new();
    if !hg_types::process_hash_key_is_set() {
        let mut key = [0u8; 16];
        entropy
            .crypto()
            .fill(&mut key)
            .map_err(|_e| StartError::Entropy)?;
        let (k0, k1) = key.split_at(8);
        let word = |b: &[u8]| u64::from_le_bytes(b.try_into().unwrap_or([0; 8]));
        // A second caller lost a race to the same end state: a key is set.
        let _ = hg_types::set_process_hash_key(word(k0), word(k1));
    }
    let exec = Arc::new(
        executor::OsExecutor::new(threads.general, threads.may_block)
            .map_err(StartError::Thread)?,
    );
    Ok((entropy, exec))
}

fn start(threads: Threads, launch: Launch) -> Result<Substrate, StartError> {
    let (entropy, exec) = base(threads)?;
    let ipc = os_ipc(&exec, launch)?;
    Ok(Substrate {
        clock: Arc::new(clock::OsClock::new()),
        entropy: Arc::new(entropy),
        exec,
        mem: Arc::new(memory::OsMemory::new()),
        ipc,
        // The production OS port is OI-M1's (M5): the broker's per-OS
        // modules. Until then no process has one.
        os: None,
        // The broker's launcher is set by the broker's start-up (phase E);
        // a plain substrate has none.
        launcher: None,
    })
}

/// This OS's IPC transport.
#[cfg(target_os = "linux")]
fn os_ipc(
    exec: &Arc<executor::OsExecutor>,
    boot: Launch,
) -> Result<Arc<dyn hg_substrate::Ipc>, StartError> {
    match ipc::linux::LinuxIpc::start(Arc::clone(exec), boot) {
        Ok(t) => Ok(Arc::new(t)),
        Err(ipc::linux::StartError::Os(e)) => Err(StartError::Ipc(e)),
        Err(ipc::linux::StartError::Bootstrap) => Err(StartError::Ipc(std::io::Error::other(
            "launch descriptors are not channel ends",
        ))),
    }
}

/// This OS's IPC transport.
#[cfg(target_os = "macos")]
fn os_ipc(
    exec: &Arc<executor::OsExecutor>,
    launched: Launch,
) -> Result<Arc<dyn hg_substrate::Ipc>, StartError> {
    match ipc::macos::MacIpc::start(Arc::clone(exec), launched) {
        Ok(t) => Ok(Arc::new(t)),
        Err(ipc::macos::StartError::Os(e)) => Err(StartError::Ipc(e)),
        Err(ipc::macos::StartError::Bootstrap) => Err(StartError::Ipc(std::io::Error::other(
            "no launch channels from the launcher",
        ))),
    }
}

/// This OS's IPC transport.
#[cfg(windows)]
fn os_ipc(
    exec: &Arc<executor::OsExecutor>,
    link: Launch,
) -> Result<Arc<dyn hg_substrate::Ipc>, StartError> {
    match ipc::windows::WindowsIpc::start(Arc::clone(exec), link) {
        Ok(t) => Ok(Arc::new(t)),
        Err(ipc::windows::StartError::Os(e)) => Err(StartError::Ipc(e)),
        Err(ipc::windows::StartError::Bootstrap) => Err(StartError::Ipc(std::io::Error::other(
            "the launch link carried no valid Hello",
        ))),
    }
}

/// This OS's IPC transport (none on this OS).
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn os_ipc(
    _exec: &Arc<executor::OsExecutor>,
    _boot: Launch,
) -> Result<Arc<dyn hg_substrate::Ipc>, StartError> {
    Ok(Arc::new(ipc::OsIpc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_substrate_starts_and_keys_hashing() {
        let s = os_substrate(Threads {
            general: 2,
            may_block: 1,
        })
        .unwrap();
        assert!(hg_types::process_hash_key_is_set());
        let _ = s.clock.monotonic();
    }
}
