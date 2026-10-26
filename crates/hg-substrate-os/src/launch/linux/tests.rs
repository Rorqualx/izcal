//! Tests of the spawner, run against the real `hg-spawner-main` and the
//! `hg-launch-probe` test child, which `cargo test` builds next to this
//! test binary (the crate's integration test makes Cargo build its
//! binaries).

use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use hg_launch_plan::{Kind, REQUEST_LEN, Reply, Request, Stage, Tier};

use super::{BuildFiles, SpawnError, SpawnerLink, sys};
use crate::ipc::linux::launch_pair;
use crate::ipc::linux::sys as ipc_sys;
use crate::ipc::unix::cmsg;

/// `target/<profile>`: the binaries' directory.
fn bin_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap().parent().unwrap().to_path_buf();
    assert!(
        dir.join(super::SPAWNER_NAME).exists() && dir.join("hg-launch-probe").exists(),
        "build the crate's binaries first (cargo test builds them; cargo test --lib does not)"
    );
    dir
}

fn files_with(child: &Path) -> BuildFiles {
    BuildFiles::for_test(&bin_dir(), child).unwrap()
}

fn probe_files() -> BuildFiles {
    files_with(&bin_dir().join("hg-launch-probe"))
}

/// Reads one record from `fd` within 10 s.
fn read_record(fd: BorrowedFd<'_>) -> Option<Vec<u8>> {
    let ev = sys::poll_in(&[fd], 10_000).unwrap();
    if ev[0] == 0 {
        return None;
    }
    let mut buf = vec![0u8; 4096];
    match ipc_sys::recv_record(fd, &mut buf) {
        ipc_sys::RecvOutcome::Record(r) => {
            buf.truncate(r.len);
            Some(buf)
        }
        _ => None,
    }
}

/// Whether `fd` reaches end of file within `within`.
fn reaches_eof(fd: BorrowedFd<'_>, within: Duration) -> bool {
    let end = Instant::now() + within;
    while Instant::now() < end {
        let ev = sys::poll_in(&[fd], 100).unwrap();
        if ev[0] == 0 {
            continue;
        }
        let mut buf = [0u8; 4096];
        match ipc_sys::recv_record(fd, &mut buf) {
            ipc_sys::RecvOutcome::Record(r) if r.len == 0 => return true,
            ipc_sys::RecvOutcome::Reset => return true,
            _ => {}
        }
    }
    false
}

fn report_field<'a>(report: &'a str, key: &str) -> &'a str {
    report
        .split(' ')
        .find_map(|kv| kv.strip_prefix(key).and_then(|v| v.strip_prefix('=')))
        .unwrap_or_else(|| panic!("{key} missing in {report}"))
}

#[test]
fn the_spawner_says_ready_with_the_session_tier() {
    let link = SpawnerLink::start(&probe_files()).unwrap();
    eprintln!("session tier: {}", link.tier().name());
    // Full is never claimed until it is built and measured (E-Q1).
    assert_ne!(link.tier(), Tier::Full);
}

#[test]
fn a_child_starts_sandboxed_with_exactly_its_two_ends() {
    let mut link = SpawnerLink::start(&probe_files()).unwrap();
    let ((from_child, to_child), (up, down)) = launch_pair().unwrap();
    let c = link.spawn(Kind::Renderer, up, down).unwrap();
    let report = String::from_utf8(read_record(from_child.as_fd()).unwrap()).unwrap();
    eprintln!(
        "tier={} outer={} ns={} {report}",
        link.tier().name(),
        c.outer_pid,
        c.ns_pid
    );
    // Stdio on /dev/null plus the two ends, none close-on-exec, nothing
    // else (E9).
    assert_eq!(report_field(&report, "fds"), "0,1,2,3,4");
    assert_eq!(report_field(&report, "nnp"), "1");
    assert_eq!(report_field(&report, "seccomp"), "2");
    // Seen from outside (the filter refuses these queries inside): its
    // own session with no terminal (review M1), and its credentials.
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", c.outer_pid)).unwrap();
    let after = stat.rsplit_once(") ").unwrap().1;
    let fields: Vec<&str> = after.split(' ').collect();
    assert_eq!(
        fields[3],
        c.outer_pid.to_string(),
        "session id is its own pid"
    );
    assert_eq!(fields[4], "0", "no controlling terminal");
    // No signal blocked: the spawner's SIGCHLD mask is not inherited.
    let status = std::fs::read_to_string(format!("/proc/{}/status", c.outer_pid)).unwrap();
    let blocked = status
        .lines()
        .find_map(|l| l.strip_prefix("SigBlk:"))
        .unwrap();
    assert_eq!(blocked.trim(), "0000000000000000");
    // No file, no socket outside IPC.
    assert_eq!(report_field(&report, "open"), libc::EPERM.to_string());
    assert_eq!(report_field(&report, "inet"), libc::EPERM.to_string());
    if link.tier() == Tier::RestrictedUserns {
        assert_eq!(c.ns_pid, 1);
        assert_eq!(report_field(&report, "pid"), "1");
    }
    // It runs until end of file on its receiver ends it.
    assert_eq!(sys::poll_in(&[c.pidfd.as_fd()], 300).unwrap()[0], 0);
    drop(to_child);
    assert_eq!(
        String::from_utf8(read_record(from_child.as_fd()).unwrap()).unwrap(),
        "end rc=0 errno=0"
    );
    let ev = sys::poll_in(&[c.pidfd.as_fd()], 10_000).unwrap();
    assert_ne!(ev[0], 0, "the child ends when its bootstrap closes");
}

/// Review M6: the spawner closes its copies of the child's ends, so the
/// broker sees end of file once the child is gone.
#[test]
fn the_spawner_keeps_no_copy_of_a_spawned_child_s_ends() {
    let mut link = SpawnerLink::start(&probe_files()).unwrap();
    let ((from_child, _to_child), (up, down)) = launch_pair().unwrap();
    let c = link.spawn(Kind::NetworkPartition, up, down).unwrap();
    assert!(read_record(from_child.as_fd()).is_some());
    sys::pidfd_signal(c.pidfd.as_fd(), libc::SIGKILL).unwrap();
    assert!(
        reaches_eof(from_child.as_fd(), Duration::from_secs(5)),
        "another process still holds the child's sender"
    );
}

/// Review M6, failure path: a child that cannot exec leaves no copy.
#[test]
fn a_child_that_fails_to_exec_leaves_no_copy_and_reports_its_stage() {
    // A regular file without exec permission stands in for the child.
    let dir = std::env::temp_dir().join(format!("izcal-spawner-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let not_exec = dir.join("not-exec");
    std::fs::write(&not_exec, b"not a program").unwrap();
    let mut link = SpawnerLink::start(&files_with(&not_exec)).unwrap();
    let ((from_child, _to_child), (up, down)) = launch_pair().unwrap();
    let got = link.spawn(Kind::Renderer, up, down);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        got.unwrap_err(),
        SpawnError::Failed {
            stage: Stage::Exec,
            errno: u32::try_from(libc::EACCES).unwrap()
        }
    );
    assert!(reaches_eof(from_child.as_fd(), Duration::from_secs(5)));
}

/// Review minor: anything but an exact request with exactly two
/// descriptors is refused, its descriptors closed, and the spawner goes
/// on serving.
#[test]
fn wrong_requests_are_refused_and_their_descriptors_closed() {
    let mut link = SpawnerLink::start(&probe_files()).unwrap();
    let ((from_child, _to_child), (up, down)) = launch_pair().unwrap();
    let req = Request::Spawn {
        id: 7,
        kind: Kind::Renderer,
    }
    .encode();
    // One descriptor instead of two.
    assert!(super::send_link(link.link(), &req, &[up.as_fd()]));
    drop((up, down));
    assert_eq!(
        next_reply(&link),
        Reply::Failed {
            id: 7,
            stage: Stage::Request,
            errno: 22
        }
    );
    assert!(reaches_eof(from_child.as_fd(), Duration::from_secs(5)));
    // A record of the wrong length.
    assert!(super::send_link(link.link(), &req[..REQUEST_LEN - 1], &[]));
    assert!(matches!(
        next_reply(&link),
        Reply::Failed {
            stage: Stage::Request,
            ..
        }
    ));
    // Control data cut by the receiver (MSG_CTRUNC): more descriptors than
    // its buffer holds. Refused whole, every installed one closed.
    let ((from_child, _to_child), (up, down)) = launch_pair().unwrap();
    let many: Vec<i32> = std::iter::repeat_n(up.as_raw_fd(), 40).collect();
    let control = cmsg::encode(&many, None);
    assert!(matches!(
        ipc_sys::send_with_control(link.link(), &req, &control),
        ipc_sys::SendOutcome::Sent
    ));
    drop((up, down));
    assert!(matches!(
        next_reply(&link),
        Reply::Failed {
            stage: Stage::Request,
            ..
        }
    ));
    assert!(reaches_eof(from_child.as_fd(), Duration::from_secs(5)));
    // Still serving.
    let ((from_child, _to_child), (up, down)) = launch_pair().unwrap();
    link.spawn(Kind::Renderer, up, down).unwrap();
    assert!(read_record(from_child.as_fd()).is_some());
}

fn next_reply(link: &SpawnerLink) -> Reply {
    let (r, fds) = link.next_reply().unwrap();
    assert!(fds.is_empty());
    r
}

/// E-R2: the spawner reaps its children (no zombie stays), and exits when
/// the broker's end closes, leaving its children running.
#[test]
fn the_spawner_reaps_and_exits_on_end_of_file_leaving_children() {
    let mut link = SpawnerLink::start(&probe_files()).unwrap();
    let ((from_child, to_child), (up, down)) = launch_pair().unwrap();
    let a = link.spawn(Kind::Renderer, up, down).unwrap();
    assert!(read_record(from_child.as_fd()).is_some());
    drop(to_child);
    assert_ne!(sys::poll_in(&[a.pidfd.as_fd()], 10_000).unwrap()[0], 0);
    let gone = |pid: u32| {
        Path::new(&format!("/proc/{pid}"))
            .exists()
            .then_some(())
            .is_none()
    };
    let end = Instant::now() + Duration::from_secs(5);
    while !gone(a.outer_pid) && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(gone(a.outer_pid), "the spawner reaped its child");

    let ((from_child, to_child), (up, down)) = launch_pair().unwrap();
    let b = link.spawn(Kind::Renderer, up, down).unwrap();
    assert!(read_record(from_child.as_fd()).is_some());
    let spawner = link.pid();
    assert_eq!(
        sys::poll_in(&[b.pidfd.as_fd()], 300).unwrap()[0],
        0,
        "b runs"
    );
    drop(link);
    assert!(gone(spawner), "the spawner exited and was reaped");
    // Its child runs on until its own end.
    assert_eq!(sys::poll_in(&[b.pidfd.as_fd()], 200).unwrap()[0], 0);
    drop(to_child);
    assert_ne!(sys::poll_in(&[b.pidfd.as_fd()], 10_000).unwrap()[0], 0);
}
