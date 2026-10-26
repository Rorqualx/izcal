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

/// The profile folder Windows keeps for a registered AppContainer
/// profile (`%LOCALAPPDATA%\Packages\<name>`), present while registered.
fn profile_folder(name: &str) -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
        .join("Packages")
        .join(name)
}

/// M1 phase G (FP-27): profiles are registered ahead of their launches.
/// The launcher starts with [`super::SPARE_PROFILES`] fresh profiles; a
/// launch takes one, and a refill replaces it off the launch path. Every
/// child still gets a profile no process used before (D5: one SID per
/// live child; nothing is reused), and the spares are unregistered when
/// the launcher ends.
#[test]
fn each_launch_takes_a_fresh_profile_made_ahead() {
    let l = launcher();
    let spare_names = |l: &WinLauncher| -> Vec<String> {
        l.spares.lock().iter().map(|s| s.name.clone()).collect()
    };
    let first = spare_names(&l);
    assert_eq!(first.len(), super::SPARE_PROFILES, "made at broker start");
    for n in &first {
        assert!(profile_folder(n).exists(), "{n} registered");
    }
    let mut used = Vec::new();
    for _ in 0..3 {
        let c = l.launch(ChildKind::Renderer, 64).unwrap();
        let name = l.lock().get(&c.peer).unwrap().container.clone();
        assert!(!used.contains(&name), "{name} used twice");
        used.push(name);
        let t = Instant::now();
        while spare_names(&l).len() < super::SPARE_PROFILES {
            assert!(t.elapsed() < Duration::from_secs(5), "refilled");
            std::thread::sleep(Duration::from_millis(5));
        }
        for n in spare_names(&l) {
            assert!(!used.contains(&n), "a spare {n} was already used");
        }
        l.kill(c.peer);
    }
    assert!(first.contains(&used[0]), "the first launch took a spare");
    let left = spare_names(&l);
    drop(l);
    for n in left.iter().chain(&used) {
        assert!(!profile_folder(n).exists(), "{n} unregistered");
    }
}

/// A profile left registered under a spare's name (an earlier broker with
/// this pid that ended without removing it) is never handed out as it
/// was: it is removed and registered again, empty.
#[test]
fn a_stale_profile_is_registered_again_empty() {
    let name = format!("izcal.{}.stale-test", std::process::id());
    drop(crate::sandbox::windows::app_container(&name, &name).unwrap());
    let marker = profile_folder(&name).join("left-by-an-earlier-child");
    std::fs::write(&marker, b"x").unwrap();
    let s = super::Spares::register_fresh(name.clone()).unwrap();
    assert_eq!(s.name, name);
    assert!(profile_folder(&name).exists(), "registered again");
    assert!(!marker.exists(), "its old contents went with it");
    crate::ipc::windows::sys::spawn::delete_container(&s.name);
}

/// The start-up sweep removes only profiles of this launcher's naming
/// scheme whose owning broker instance is gone, by pid and creation time:
/// a pid now held by another process (reused) and a pid no process holds.
/// A live instance's profiles (this process's), and every name outside
/// the scheme, are left alone.
#[test]
fn the_sweep_removes_only_profiles_of_brokers_that_are_gone() {
    let me = std::process::id();
    let start = super::own_start().unwrap();
    let reused = format!("izcal.l.{me}.{}.1", start + 1);
    let no_process = format!("izcal.l.{}.1.1", u32::MAX - 1);
    let live = format!("izcal.l.{me}.{start}.900001");
    let other = format!("izcal.other-scheme.{me}");
    let malformed = format!("izcal.l.{me}.x");
    // Another test's launcher may sweep the first two before this one
    // does (every launcher sweeps at its start), so only the outcome is
    // checked for them.
    for n in [&reused, &no_process, &live, &other, &malformed] {
        drop(crate::sandbox::windows::app_container(n, n).unwrap());
    }
    for n in [&live, &other, &malformed] {
        assert!(profile_folder(n).exists(), "{n} registered");
    }
    let _ = super::sweep_stale_profiles();
    assert!(!profile_folder(&reused).exists(), "a reused pid's profile");
    assert!(
        !profile_folder(&no_process).exists(),
        "a gone pid's profile"
    );
    for n in [&live, &other, &malformed] {
        assert!(profile_folder(n).exists(), "{n} was left alone");
        crate::ipc::windows::sys::spawn::delete_container(n);
    }
}
