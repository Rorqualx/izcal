//! Tests of the macOS launcher, against the `hg-launch-probe` child,
//! which completes the hello exchange and never sends `ready`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hg_substrate::Launcher;
use hg_substrate::ipc::OsPeer;
use hg_substrate::launch::{ChildKind, LaunchError, SandboxTier};

use super::{BuildFiles, MacLauncher};
use crate::executor::OsExecutor;
use crate::ipc::macos::{MacIpc, sys as mach};

fn probe() -> PathBuf {
    crate::launch::test_bins::bin("hg-launch-probe")
}

fn launcher() -> MacLauncher {
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let ipc = Arc::new(MacIpc::start(exec, false).unwrap());
    MacLauncher::new(ipc, BuildFiles::for_test(&probe()).unwrap()).unwrap()
}

/// Whether `pid` is gone (reaped): no process answers signal 0.
fn gone(pid: libc::pid_t) -> bool {
    mach::reap_if_ended(pid).is_err()
}

/// The launcher conformance test: a child that never sends `ready` is
/// killed and reaped well within the ready deadline; a peer the
/// launcher did not start is left alone.
#[test]
fn a_child_that_never_sends_ready_is_killed_and_reaped() {
    let l = launcher();
    let c = l.launch(ChildKind::Renderer, 64).unwrap();
    assert_eq!(c.sandbox, SandboxTier::MacosSbpl);
    let pid = l.pid_of(c.peer).unwrap();
    l.kill(OsPeer::new(c.peer.pid(), c.peer.generation() + 1));
    assert!(
        matches!(mach::reap_if_ended(pid), Ok(false)),
        "still running"
    );
    let t = Instant::now();
    l.kill(c.peer);
    assert!(gone(pid), "killed and reaped");
    eprintln!("killed and reaped in {:?}", t.elapsed());
    assert!(t.elapsed() < Duration::from_secs(5));
}

/// Review minor: a child whose code-directory hash differs from the one
/// recorded at broker start never runs.
#[test]
fn a_replaced_executable_never_runs() {
    let mut files = BuildFiles::for_test(&probe()).unwrap();
    for p in &mut files.exes {
        p.cdhash = vec![0; 20];
    }
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let ipc = Arc::new(MacIpc::start(exec, false).unwrap());
    let l = MacLauncher::new(ipc, files).unwrap();
    assert_eq!(
        l.launch(ChildKind::Renderer, 64).err(),
        Some(LaunchError::Failed)
    );
    l.restart_required();
    assert_eq!(
        launcher().launch(ChildKind::Renderer, 8).err(),
        Some(LaunchError::Unsupported)
    );
    assert_eq!(
        l.launch(ChildKind::Renderer, 64).err(),
        Some(LaunchError::RestartRequired)
    );
}
