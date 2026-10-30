//! Tests of the Windows launcher, against the `hg-launch-probe` child,
//! which starts in its sandbox and never sends `ready`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hg_substrate::Launcher;
use hg_substrate::ipc::OsPeer;
use hg_substrate::launch::{ChildKind, LaunchError, SandboxTier};

use super::{BuildFiles, WinLauncher};
use crate::executor::OsExecutor;
use crate::ipc::windows::WindowsIpc;

fn probe() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let p = exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("hg-launch-probe.exe");
    assert!(
        p.exists(),
        "build the crate's binaries first (cargo test builds them)"
    );
    p
}

fn launcher() -> WinLauncher {
    let exec = Arc::new(OsExecutor::new(2, 1).unwrap());
    let ipc = Arc::new(WindowsIpc::start(exec, None).unwrap());
    WinLauncher::new(ipc, BuildFiles::for_test(&probe()).unwrap())
}

/// The launcher conformance test: a child that never sends `ready` is
/// ended through its job well within the ready deadline; a peer the
/// launcher did not start is left alone.
#[test]
fn a_child_that_never_sends_ready_is_killed() {
    let l = launcher();
    let c = l.launch(ChildKind::Renderer, 64).unwrap();
    assert_eq!(c.sandbox, SandboxTier::WindowsAppContainer);
    l.kill(OsPeer::new(c.peer.pid(), c.peer.generation() + 1));
    assert_eq!(l.exit_code(c.peer, 300), None, "still running");
    let t = Instant::now();
    l.kill(c.peer);
    eprintln!("killed in {:?}", t.elapsed());
    assert!(t.elapsed() < Duration::from_secs(5));
    assert_eq!(l.exit_code(c.peer, 0), None, "forgotten");
    l.restart_required();
    assert_eq!(
        l.launch(ChildKind::Renderer, 64).err(),
        Some(LaunchError::RestartRequired)
    );
}

/// Review M9: the build's files are held without write or delete
/// sharing, so they cannot be replaced while the broker runs.
#[test]
fn the_build_cannot_be_replaced_while_held() {
    let dir = std::env::temp_dir().join(format!("izcal-held-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("x.exe");
    std::fs::copy(probe(), &f).unwrap();
    let held = BuildFiles::for_test(&f).unwrap();
    assert!(std::fs::remove_file(&f).is_err(), "deleted while held");
    assert!(
        std::fs::rename(&f, dir.join("y.exe")).is_err(),
        "renamed while held"
    );
    assert!(
        std::fs::OpenOptions::new().write(true).open(&f).is_err(),
        "written while held"
    );
    drop(held);
    std::fs::remove_dir_all(&dir).unwrap();
}
