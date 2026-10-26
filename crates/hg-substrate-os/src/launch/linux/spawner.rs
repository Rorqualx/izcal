//! The Linux spawner (launch design E-R2, review B2 and M1–M6): a
//! single-threaded process the broker starts once, at its own start,
//! that makes every sandboxed child, so the broker itself never forks.
//!
//! It starts with exactly: stdin and stdout on `/dev/null`, the broker's
//! stderr at 2, its link to the broker at 3 (a `SOCK_SEQPACKET` end) and
//! one `O_PATH` handle per Izcal executable at 4 onwards, in
//! [`Exe::ALL`] order. It parses no arguments and reads no environment.
//!
//! At start it decides the session's [`Tier`] once (review M3) and tells
//! the broker with `Ready`. Then, per `Spawn` request: refuse anything
//! but an exact record with exactly the two bootstrap ends (truncated
//! control data included), clone the child with the tier's namespaces,
//! **close its own copies of the child's ends at once** (review M6),
//! read the child's failure pipe, and reply `Spawned` with the child's
//! pidfd, or `Failed`. It reaps every child (`SIGCHLD` through a
//! signalfd), and exits when the broker's end closes; its children are
//! not killed by that (the broker kills through their pidfds).

use std::os::fd::{AsFd, BorrowedFd, OwnedFd, RawFd};

use hg_launch_plan::{Exe, REQUEST_LEN, Reply, Request, Stage, Tier, WireError};

use super::sys::{self, ChildPlan};
use super::{LinkRecv, recv_link, send_link};
use crate::sandbox::seccomp::{
    ARCH_AARCH64, ARCH_X86_64, Insn, launch_rules, program, steady_rules,
};

/// The broker link's descriptor.
pub const LINK_FD: RawFd = 3;
/// The first executable handle's descriptor.
pub const FIRST_EXE_FD: RawFd = 4;

/// Exit codes: the start-up failed (no `Ready` was sent).
const EXIT_START: i32 = 2;
/// Landlock is missing: below every kind's floor (review M2).
const EXIT_NO_LANDLOCK: i32 = 3;

fn arch() -> u32 {
    if cfg!(target_arch = "aarch64") {
        ARCH_AARCH64
    } else {
        ARCH_X86_64
    }
}

/// The launch filter for a child whose own pid is `own` (review B1: the
/// steady list plus `execveat`).
fn launch_filter(own: u32) -> Option<Vec<Insn>> {
    let a = arch();
    program(a, &launch_rules(a, &steady_rules(a, own))).ok()
}

/// The clone flags a tier applies.
const fn flags(tier: Tier) -> u64 {
    match tier {
        Tier::NoUserns => 0,
        // No mount namespace: under the restricted profile no exec by
        // handle works in one (launch facts E4, E22), and nothing could be
        // mounted there anyway (E2).
        Tier::RestrictedUserns | Tier::Full => {
            sys::CLONE_NEWUSER | sys::CLONE_NEWPID | sys::CLONE_NEWNET
        }
    }
}

/// Decides the session tier with test clones (review M3: once, at start).
/// The full tier (an empty root) is never claimed until it is built and
/// measured (E-Q1); a host that would allow it is logged.
fn decide_tier() -> Tier {
    let ns = sys::CLONE_NEWUSER | sys::CLONE_NEWPID | sys::CLONE_NEWNET;
    match sys::test_clone(ns | sys::CLONE_NEWNS) {
        Some(true) => {
            eprintln!(
                "hg-spawner: this host allows mounts in a user namespace; the full tier is \
                 not built yet, so the session runs at {}",
                Tier::RestrictedUserns.name()
            );
            Tier::RestrictedUserns
        }
        Some(false) => Tier::RestrictedUserns,
        None => match sys::test_clone(ns) {
            Some(_) => Tier::RestrictedUserns,
            None => Tier::NoUserns,
        },
    }
}

/// What the spawner holds.
struct Spawner {
    link: OwnedFd,
    exes: Vec<OwnedFd>,
    devnull: OwnedFd,
    tier: Tier,
    abi: i32,
}

/// The spawner's `main`. Returns the process's exit code.
#[must_use]
pub fn run() -> i32 {
    // Claim the start-up descriptors, then close anything else: a restart
    // keeps the same discipline as the first start (review minor).
    let Some(link) = sys::take_inherited(LINK_FD) else {
        return EXIT_START;
    };
    let mut exes = Vec::new();
    for (i, _e) in Exe::ALL.iter().enumerate() {
        let n = FIRST_EXE_FD.saturating_add(RawFd::try_from(i).unwrap_or(RawFd::MAX));
        match sys::take_inherited(n) {
            Some(fd) if sys::is_regular(fd.as_fd()) => exes.push(fd),
            _ => return EXIT_START,
        }
    }
    let after = FIRST_EXE_FD.saturating_add(RawFd::try_from(Exe::ALL.len()).unwrap_or(0));
    sys::close_from(after);
    for fd in exes.iter().chain([&link]) {
        if sys::set_cloexec(fd.as_fd()).is_err() {
            return EXIT_START;
        }
    }
    let Ok(devnull) = sys::open_devnull() else {
        return EXIT_START;
    };
    let Ok(sigchld) = sys::sigchld_fd() else {
        return EXIT_START;
    };
    let abi = sys::landlock_abi();
    if abi < 1 {
        eprintln!("hg-spawner: no Landlock on this host; every kind needs it (review M2)");
        return EXIT_NO_LANDLOCK;
    }
    let s = Spawner {
        link,
        exes,
        devnull,
        tier: decide_tier(),
        abi,
    };
    if !send_link(s.link.as_fd(), &Reply::Ready { tier: s.tier }.encode(), &[]) {
        return EXIT_START;
    }
    s.serve(sigchld.as_fd())
}

impl Spawner {
    fn serve(&self, sigchld: BorrowedFd<'_>) -> i32 {
        let mut buf = [0u8; REQUEST_LEN + 1];
        loop {
            let Ok([link_ev, sig_ev, ..]) = sys::poll_in(&[self.link.as_fd(), sigchld], -1) else {
                return EXIT_START;
            };
            if sig_ev != 0 {
                sys::drain(sigchld);
                sys::reap_all();
            }
            if link_ev == 0 {
                continue;
            }
            match recv_link(self.link.as_fd(), &mut buf) {
                LinkRecv::WouldBlock => {}
                // The broker is gone: so is this session. Children live on
                // until the broker's pidfds (or their channels) end them.
                LinkRecv::Eof | LinkRecv::Failed => return 0,
                LinkRecv::Refused { fds } => {
                    drop(fds);
                    self.reply(Reply::Failed {
                        id: 0,
                        stage: Stage::Request,
                        errno: errno_u32(libc::EINVAL),
                    });
                }
                LinkRecv::Record { len, fds } => {
                    let bytes = buf.get(..len).unwrap_or(&[]);
                    let sent = match self.spawn(Request::decode(bytes), fds) {
                        Ok((reply, pidfd)) => {
                            // The pidfd goes with the reply; ours closes
                            // after it. A child the broker never hears of
                            // is killed.
                            let ok =
                                send_link(self.link.as_fd(), &reply.encode(), &[pidfd.as_fd()]);
                            if !ok {
                                let _ = sys::pidfd_signal(pidfd.as_fd(), libc::SIGKILL);
                            }
                            ok
                        }
                        Err(reply) => self.reply(reply),
                    };
                    if !sent {
                        return 0;
                    }
                }
            }
            sys::reap_all();
        }
    }

    fn reply(&self, r: Reply) -> bool {
        send_link(self.link.as_fd(), &r.encode(), &[])
    }

    /// One request. The received descriptors are dropped (closed) on
    /// every path before this returns, and on the success path right
    /// after the clone (review M6).
    fn spawn(
        &self,
        req: Result<Request, WireError>,
        fds: Vec<OwnedFd>,
    ) -> Result<(Reply, OwnedFd), Reply> {
        let failed = |id, stage, errno| {
            Err(Reply::Failed {
                id,
                stage,
                errno: errno_u32(errno),
            })
        };
        let Ok(Request::Spawn { id, kind }) = req else {
            return failed(0, Stage::Request, libc::EINVAL);
        };
        let Ok([up, down]) = <[OwnedFd; 2]>::try_from(fds) else {
            return failed(id, Stage::Request, libc::EINVAL);
        };
        if kind.floor() > self.tier {
            return failed(id, Stage::Request, libc::EPERM);
        }
        let exe = kind.exe();
        let Some(exe_fd) = Exe::ALL
            .iter()
            .position(|&e| e == exe)
            .and_then(|i| self.exes.get(i))
        else {
            return failed(id, Stage::Request, libc::ENOENT);
        };
        let Ok(name) = std::ffi::CString::new(exe.file_name()) else {
            return failed(id, Stage::Request, libc::EINVAL);
        };
        let Ok((err_r, err_w)) = sys::pipe() else {
            return failed(id, Stage::Clone, libc::EMFILE);
        };
        let plan = ChildPlan {
            flags: flags(self.tier),
            up: up.as_fd(),
            down: down.as_fd(),
            devnull: self.devnull.as_fd(),
            exe: exe_fd.as_fd(),
            name: &name,
            err: err_w.as_fd(),
            abi: self.abi,
            filter: launch_filter,
        };
        let spawned = sys::spawn_child(&plan);
        // Review M6: the child has its own copies now (or none exists);
        // ours close before anything else happens.
        drop((up, down, err_w));
        let (outer, pidfd) = match spawned {
            Ok(c) => c,
            Err(e) => return failed(id, Stage::Clone, e.raw_os_error().unwrap_or(0)),
        };
        if let Some((stage, errno)) = sys::read_failure(err_r.as_fd()) {
            let _ = sys::wait_pidfd(pidfd.as_fd());
            let stage = match stage {
                3 => Stage::Descriptors,
                4 => Stage::Landlock,
                5 => Stage::Seccomp,
                6 => Stage::Exec,
                _ => Stage::Unknown,
            };
            return failed(id, stage, errno);
        }
        let ns_pid = if flags(self.tier) & sys::CLONE_NEWPID != 0 {
            1
        } else {
            outer
        };
        Ok((
            Reply::Spawned {
                id,
                tier: self.tier,
                outer_pid: outer,
                ns_pid,
            },
            pidfd,
        ))
    }
}

/// An errno as the reply carries it.
fn errno_u32(e: i32) -> u32 {
    u32::try_from(e).unwrap_or(0)
}
