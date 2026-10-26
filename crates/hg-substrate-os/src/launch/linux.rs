//! Linux launch (launch design E-R2 to E-R6): the broker's side of the
//! spawner link ([`SpawnerLink`]) and the spawner itself ([`spawner`]).
//!
//! The broker starts the spawner once, from a handle it opened at its
//! own start, and asks it for each child over one `SOCK_SEQPACKET` pair
//! with `hg_launch_plan`'s fixed records. The broker never forks a child
//! itself. Records on the link are refused whole when truncated, when
//! their control data is cut (`MSG_CTRUNC`), or when they carry another
//! number of descriptors than the record declares (review minor);
//! received descriptors are close-on-exec (`MSG_CMSG_CLOEXEC`).

pub(crate) mod child;
pub mod spawner;
mod sys;
#[cfg(test)]
mod tests;

use std::ffi::CString;
use std::io;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::path::Path;

use hg_launch_plan::{Exe, Kind, REPLY_LEN, Reply, Request, Stage, Tier};

use crate::ipc::linux::sys::{self as ipc_sys, RecvOutcome, SendOutcome};

/// The spawner's file name in the build.
pub const SPAWNER_NAME: &str = "hg-spawner-main";

/// How long the broker waits for any spawner reply: `Ready` at start, or
/// a spawn's answer (D5's ready deadline†).
const REPLY_DEADLINE_MS: i32 = 5_000;

/// One record received on the link.
pub(crate) enum LinkRecv {
    /// A whole record of `len` bytes and its descriptors, in order.
    Record {
        /// Bytes in the buffer.
        len: usize,
        /// The descriptors it carried, owned.
        fds: Vec<OwnedFd>,
    },
    /// A record refused whole (truncated, cut control data, control the
    /// parser refused); its descriptors, already owned, close on drop.
    Refused {
        /// Every descriptor the kernel installed for it.
        fds: Vec<OwnedFd>,
    },
    /// Nothing queued.
    WouldBlock,
    /// The other side closed.
    Eof,
    /// Any other error.
    Failed,
}

/// Receives one link record into `buf`, never blocking.
pub(crate) fn recv_link(fd: BorrowedFd<'_>, buf: &mut [u8]) -> LinkRecv {
    let w = match ipc_sys::recv_with_rights(fd, buf) {
        RecvOutcome::Record(w) => w,
        RecvOutcome::WouldBlock => return LinkRecv::WouldBlock,
        RecvOutcome::Reset => return LinkRecv::Eof,
        RecvOutcome::Failed(_) => return LinkRecv::Failed,
    };
    let mut owned = w.owned;
    let in_order: Vec<OwnedFd> = w
        .control
        .fds()
        .iter()
        .filter_map(|n| owned.remove(n))
        .collect();
    // Anything else the kernel installed closes here.
    drop(owned);
    if w.rec.len == 0 && in_order.is_empty() && w.rec.control_len == 0 {
        return LinkRecv::Eof;
    }
    if w.rec.truncated || w.rec.control_truncated || w.control.outcome.is_err() {
        return LinkRecv::Refused { fds: in_order };
    }
    LinkRecv::Record {
        len: w.rec.len,
        fds: in_order,
    }
}

/// Sends one link record with `fds`; `false` if it was not sent whole.
pub(crate) fn send_link(fd: BorrowedFd<'_>, bytes: &[u8], fds: &[BorrowedFd<'_>]) -> bool {
    loop {
        match ipc_sys::send_record(fd, bytes, fds) {
            SendOutcome::Sent => return true,
            // The link holds one record each way at a time (one request in
            // flight); a full buffer means the peer is not reading.
            SendOutcome::WouldBlock => {
                if !sys::poll_out(fd, REPLY_DEADLINE_MS) {
                    return false;
                }
            }
            SendOutcome::PeerGone | SendOutcome::Failed(_) => return false,
        }
    }
}

/// The executable handles a broker opens at its start (G251, DU 13.1):
/// the spawner and each child executable, from one build directory.
#[derive(Debug)]
pub struct BuildFiles {
    spawner: OwnedFd,
    exes: [OwnedFd; 2],
}

impl BuildFiles {
    /// Opens the spawner and each [`Exe`] in `dir` (`O_PATH`, no symlink
    /// followed). Each is exec'd from this handle only, never by path.
    ///
    /// # Errors
    /// The failing open's error.
    pub fn open(dir: &Path) -> io::Result<Self> {
        let d = CString::new(dir.as_os_str().as_encoded_bytes())
            .map_err(|_e| io::Error::from_raw_os_error(libc::EINVAL))?;
        let dfd = sys::open_dir(&d)?;
        let open = |name: &str| {
            let n = CString::new(name).map_err(|_e| io::Error::from_raw_os_error(libc::EINVAL))?;
            sys::open_exe(dfd.as_fd(), &n)
        };
        let [r, n] = Exe::ALL;
        Ok(Self {
            spawner: open(SPAWNER_NAME)?,
            exes: [open(r.file_name())?, open(n.file_name())?],
        })
    }

    /// The spawner from `dir`, with the file `child` standing in for
    /// every child executable (tests: a probe binary, or a file that
    /// cannot exec).
    #[cfg(test)]
    pub(crate) fn for_test(dir: &Path, child: &Path) -> io::Result<Self> {
        let open = |d: &Path, name: &std::ffi::OsStr| {
            let d = CString::new(d.as_os_str().as_encoded_bytes())
                .map_err(|_e| io::Error::from_raw_os_error(libc::EINVAL))?;
            let n = CString::new(name.as_encoded_bytes())
                .map_err(|_e| io::Error::from_raw_os_error(libc::EINVAL))?;
            sys::open_exe(sys::open_dir(&d)?.as_fd(), &n)
        };
        let cdir = child.parent().unwrap_or(dir);
        let cname = child.file_name().unwrap_or_default();
        Ok(Self {
            spawner: open(dir, std::ffi::OsStr::new(SPAWNER_NAME))?,
            exes: [open(cdir, cname)?, open(cdir, cname)?],
        })
    }
}

/// A child the spawner started.
#[derive(Debug)]
pub struct Spawned {
    /// Its pid in the broker's namespace.
    pub outer_pid: u32,
    /// Its pid in its own namespace.
    pub ns_pid: u32,
    /// Its pidfd: exit notice and kill, never reused (launch facts E16,
    /// E17). Only the spawner reaps it.
    pub pidfd: OwnedFd,
}

/// Why a spawn failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnError {
    /// The spawner refused or the child failed at `stage` with `errno`.
    Failed {
        /// Where.
        stage: Stage,
        /// The OS error.
        errno: u32,
    },
    /// The kind's floor is above the session tier (review M2, M3).
    BelowFloor,
    /// The spawner is gone, broke the protocol, or did not answer in time.
    Spawner,
    /// The reply did not check out: another tier than the session's, or
    /// a pidfd whose pids differ from the reply's (the child is killed).
    Mismatch,
}

/// The broker's link to a running spawner.
#[derive(Debug)]
pub struct SpawnerLink {
    link: OwnedFd,
    pid: u32,
    pidfd: OwnedFd,
    tier: Tier,
    next_id: u64,
}

impl SpawnerLink {
    /// Starts the spawner from `files` and waits for its `Ready`.
    ///
    /// # Errors
    /// The spawner could not start, or sent no valid `Ready` in time (it
    /// is killed and reaped).
    pub fn start(files: &BuildFiles) -> io::Result<Self> {
        // A child whose spawner died is re-parented to the broker, never
        // to init (review minor), so it stays in the broker's tree.
        sys::set_child_subreaper()?;
        let (ours, theirs) = ipc_sys::socketpair()?;
        let devnull = sys::open_devnull()?;
        let name = c_name(SPAWNER_NAME)?;
        let [r, n] = &files.exes;
        let (pid, pidfd) = sys::spawn_spawner(
            files.spawner.as_fd(),
            &name,
            theirs.as_fd(),
            &[r.as_fd(), n.as_fd()],
            devnull.as_fd(),
        )?;
        drop(theirs);
        let mut me = Self {
            link: ours,
            pid,
            pidfd,
            tier: Tier::NoUserns,
            next_id: 1,
        };
        match me.next_reply() {
            Ok((Reply::Ready { tier }, fds)) if fds.is_empty() => {
                me.tier = tier;
                Ok(me)
            }
            _ => {
                me.stop();
                Err(io::Error::other("the spawner sent no Ready"))
            }
        }
    }

    /// The session tier the spawner decided at its start (review M3).
    #[must_use]
    pub const fn tier(&self) -> Tier {
        self.tier
    }

    /// The spawner's pid (the broker's own child, no namespace).
    #[must_use]
    pub const fn pid(&self) -> u32 {
        self.pid
    }

    /// The spawner's pidfd.
    #[must_use]
    pub fn pidfd(&self) -> BorrowedFd<'_> {
        self.pidfd.as_fd()
    }

    /// Asks for one child of `kind` with bootstrap ends `up` (its sender
    /// to the broker) and `down` (its receiver). The broker's copies are
    /// dropped once sent. Checks the reply's tier against the session's
    /// and the pidfd's pids against the reply (launch facts E16; E-R6).
    ///
    /// # Errors
    /// [`SpawnError`]. After [`SpawnError::Spawner`] the link is dead.
    pub fn spawn(&mut self, kind: Kind, up: OwnedFd, down: OwnedFd) -> Result<Spawned, SpawnError> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let req = Request::Spawn { id, kind }.encode();
        let sent = send_link(self.link.as_fd(), &req, &[up.as_fd(), down.as_fd()]);
        drop((up, down));
        if !sent {
            return Err(SpawnError::Spawner);
        }
        loop {
            let (reply, fds) = self.next_reply().map_err(|_e| SpawnError::Spawner)?;
            if reply.fds() != fds.len() {
                return Err(SpawnError::Spawner);
            }
            match reply {
                Reply::Spawned {
                    id: rid,
                    tier,
                    outer_pid,
                    ns_pid,
                } => {
                    let Some(pidfd) = fds.into_iter().next() else {
                        return Err(SpawnError::Spawner);
                    };
                    if rid != id {
                        // An answer to a request this side gave up on: that
                        // child is not wanted.
                        let _ = sys::pidfd_signal(pidfd.as_fd(), libc::SIGKILL);
                        continue;
                    }
                    let pids_match = sys::pidfd_pids(pidfd.as_fd()) == Some((outer_pid, ns_pid));
                    if tier != self.tier || !pids_match {
                        let _ = sys::pidfd_signal(pidfd.as_fd(), libc::SIGKILL);
                        return Err(SpawnError::Mismatch);
                    }
                    return Ok(Spawned {
                        outer_pid,
                        ns_pid,
                        pidfd,
                    });
                }
                Reply::Failed {
                    id: rid,
                    stage,
                    errno,
                } => {
                    if rid != id && rid != 0 {
                        continue;
                    }
                    if stage == Stage::Request && errno == errno_u32(libc::EPERM) {
                        return Err(SpawnError::BelowFloor);
                    }
                    return Err(SpawnError::Failed { stage, errno });
                }
                Reply::Ready { .. } => return Err(SpawnError::Spawner),
            }
        }
    }

    /// Waits up to the deadline for one whole reply.
    fn next_reply(&self) -> io::Result<(Reply, Vec<OwnedFd>)> {
        let mut buf = [0u8; REPLY_LEN + 1];
        loop {
            let ev = sys::poll_in(&[self.link.as_fd()], REPLY_DEADLINE_MS)?;
            if ev.first().copied().unwrap_or(0) == 0 {
                return Err(io::Error::from_raw_os_error(libc::ETIMEDOUT));
            }
            match recv_link(self.link.as_fd(), &mut buf) {
                LinkRecv::WouldBlock => {}
                LinkRecv::Record { len, fds } => {
                    let r = Reply::decode(buf.get(..len).unwrap_or(&[]))
                        .map_err(|_e| io::Error::from_raw_os_error(libc::EPROTO))?;
                    return Ok((r, fds));
                }
                LinkRecv::Refused { .. } => {
                    return Err(io::Error::from_raw_os_error(libc::EPROTO));
                }
                LinkRecv::Eof | LinkRecv::Failed => {
                    return Err(io::Error::from_raw_os_error(libc::EPIPE));
                }
            }
        }
    }

    /// Ends the spawner: kills and reaps it (it is the broker's own
    /// child). Its children live on until their own ends.
    pub fn stop(&mut self) {
        let _ = sys::pidfd_signal(self.pidfd.as_fd(), libc::SIGKILL);
        let _ = sys::wait_pidfd(self.pidfd.as_fd());
    }

    /// The raw link, for tests that write records the broker never would.
    #[cfg(test)]
    pub(crate) fn link(&self) -> BorrowedFd<'_> {
        self.link.as_fd()
    }
}

impl Drop for SpawnerLink {
    fn drop(&mut self) {
        // End of file on the link ends the spawner; reap it so no zombie
        // stays behind. Its children live on until their own ends.
        let _ = ipc_sys::shutdown_write(self.link.as_fd());
        let _ = sys::wait_pidfd(self.pidfd.as_fd());
    }
}

fn c_name(s: &str) -> io::Result<CString> {
    CString::new(s).map_err(|_e| io::Error::from_raw_os_error(libc::EINVAL))
}

fn errno_u32(e: i32) -> u32 {
    u32::try_from(e).unwrap_or(0)
}

/// The broker's launcher on Linux (E-R2 to E-R6): every child through the
/// spawner, which it starts at its own start and again if it dies.
pub struct LinuxLauncher {
    ipc: std::sync::Arc<crate::ipc::linux::LinuxIpc>,
    files: BuildFiles,
    st: std::sync::Mutex<LauncherState>,
    halted: std::sync::atomic::AtomicBool,
}

struct LauncherState {
    link: Option<SpawnerLink>,
    /// Each live child's pidfd, by the identity its launch returned.
    children: std::collections::BTreeMap<hg_substrate::ipc::OsPeer, OwnedFd>,
    /// The launch sequence number: the generation of every `OsPeer` this
    /// launcher makes, never repeated (E-R6 revised after E24: a broker
    /// label the kernel never attests, review M6).
    launches: u64,
}

impl core::fmt::Debug for LinuxLauncher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LinuxLauncher").finish_non_exhaustive()
    }
}

/// The session tier as the launch contract names it.
const fn sandbox_tier(t: Tier) -> hg_substrate::launch::SandboxTier {
    match t {
        Tier::NoUserns => hg_substrate::launch::SandboxTier::LinuxNoUserns,
        Tier::RestrictedUserns => hg_substrate::launch::SandboxTier::LinuxRestrictedUserns,
        Tier::Full => hg_substrate::launch::SandboxTier::LinuxFull,
    }
}

const fn plan_kind(k: hg_substrate::launch::ChildKind) -> Kind {
    match k {
        hg_substrate::launch::ChildKind::Renderer => Kind::Renderer,
        hg_substrate::launch::ChildKind::NetworkSystem => Kind::NetworkSystem,
        hg_substrate::launch::ChildKind::NetworkContainer => Kind::NetworkContainer,
        hg_substrate::launch::ChildKind::NetworkPartition => Kind::NetworkPartition,
    }
}

impl LinuxLauncher {
    /// A launcher for `ipc`'s process, from `files`, with the spawner
    /// started now.
    ///
    /// # Errors
    /// The spawner could not start.
    pub(crate) fn start(
        ipc: std::sync::Arc<crate::ipc::linux::LinuxIpc>,
        files: BuildFiles,
    ) -> io::Result<Self> {
        let link = SpawnerLink::start(&files)?;
        Ok(Self {
            ipc,
            files,
            st: std::sync::Mutex::new(LauncherState {
                link: Some(link),
                children: std::collections::BTreeMap::new(),
                launches: 0,
            }),
            halted: std::sync::atomic::AtomicBool::new(false),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, LauncherState> {
        self.st
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The session tier, if a spawner runs.
    #[must_use]
    pub fn tier(&self) -> Option<Tier> {
        self.lock().link.as_ref().map(SpawnerLink::tier)
    }

    /// Kills the spawner (tests: its death and restart).
    #[cfg(test)]
    pub(crate) fn kill_spawner(&self) {
        if let Some(l) = self.lock().link.as_ref() {
            let _ = sys::pidfd_signal(l.pidfd(), libc::SIGKILL);
            let _ = sys::wait_pidfd(l.pidfd());
        }
    }

    /// The pidfd of a child this launcher started (tests).
    #[cfg(test)]
    pub(crate) fn pidfd_of(&self, peer: hg_substrate::ipc::OsPeer) -> Option<OwnedFd> {
        self.lock()
            .children
            .get(&peer)
            .and_then(|f| f.try_clone().ok())
    }
}

impl LauncherState {
    /// Drops the pidfds of children that have ended; reaps any that were
    /// re-parented here because their spawner died.
    fn sweep(&mut self) {
        self.children.retain(|_p, fd| {
            let ended = sys::poll_in(&[fd.as_fd()], 0)
                .is_ok_and(|ev| ev.first().copied().unwrap_or(0) != 0);
            if ended {
                let _ = sys::try_reap(fd.as_fd());
            }
            !ended
        });
    }
}

impl hg_substrate::Launcher for LinuxLauncher {
    fn launch(
        &self,
        kind: hg_substrate::launch::ChildKind,
        capacity: u32,
    ) -> Result<hg_substrate::launch::Launched, hg_substrate::launch::LaunchError> {
        use hg_substrate::launch::LaunchError;
        if self.halted.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(LaunchError::RestartRequired);
        }
        // Both ends' counters start at the bootstrap capacity.
        if capacity != crate::ipc::linux::BOOTSTRAP_CAPACITY {
            return Err(LaunchError::Unsupported);
        }
        let mut st = self.lock();
        st.sweep();
        if st.link.is_none() {
            // The spawner died: start it again (E-R2), with the same
            // descriptor discipline as the first time.
            st.link = Some(SpawnerLink::start(&self.files).map_err(|_e| LaunchError::Failed)?);
        }
        let ((from_child, to_child), (up, down)) =
            crate::ipc::linux::launch_pair().map_err(|_e| LaunchError::Failed)?;
        let Some(link) = st.link.as_mut() else {
            return Err(LaunchError::Failed);
        };
        let tier = link.tier();
        let spawned = match link.spawn(plan_kind(kind), up, down) {
            Ok(s) => s,
            Err(SpawnError::BelowFloor) => return Err(LaunchError::BelowFloor),
            Err(SpawnError::Spawner) => {
                // In-flight spawns fail with it; the next launch restarts it.
                st.link = None;
                return Err(LaunchError::Failed);
            }
            Err(SpawnError::Failed { .. } | SpawnError::Mismatch) => {
                return Err(LaunchError::Failed);
            }
        };
        st.launches = st.launches.wrapping_add(1);
        let peer = hg_substrate::ipc::OsPeer::new(spawned.outer_pid, st.launches);
        let Some((from_child, to_child)) =
            crate::ipc::linux::adopt_parent_ends(&self.ipc, from_child, to_child)
        else {
            let _ = sys::pidfd_signal(spawned.pidfd.as_fd(), libc::SIGKILL);
            return Err(LaunchError::Failed);
        };
        st.children.insert(peer, spawned.pidfd);
        Ok(hg_substrate::launch::Launched {
            from_child,
            to_child,
            peer,
            sandbox: sandbox_tier(tier),
        })
    }

    fn kill(&self, peer: hg_substrate::ipc::OsPeer) {
        // Only through the child's own pidfd: never another process that
        // reused the pid. The spawner reaps it.
        if let Some(fd) = self.lock().children.remove(&peer) {
            let _ = sys::pidfd_signal(fd.as_fd(), libc::SIGKILL);
        }
    }

    fn restart_required(&self) {
        self.halted.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
