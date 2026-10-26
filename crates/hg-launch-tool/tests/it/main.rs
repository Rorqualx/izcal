//! The M1 exit test on this OS (launch design E-R3, phase E "done
//! when"): the browser, renderer and network service, each its own
//! process in its sandbox, make the round trip. Runs `hg-launch-tool
//! round-trip` from the build directory, where `cargo test` also builds
//! the mains (each main crate's own test makes Cargo build it).

#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::path::Path;
use std::process::Command;

// Windows joins when its launcher lands (phase E item 4).
#[test]
fn the_round_trip_runs_in_the_os_sandboxes() {
    let tool = Path::new(env!("CARGO_BIN_EXE_hg-launch-tool"));
    let dir = tool.parent().unwrap();
    for exe in ["hg-renderer-main", "hg-net-main", "hg-browser-main"] {
        let p = dir.join(format!("{exe}{}", std::env::consts::EXE_SUFFIX));
        assert!(
            p.exists(),
            "{} is not built: run the workspace's tests (cargo test --workspace) or build its binaries",
            p.display()
        );
    }
    let out = Command::new(tool).arg("round-trip").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    eprintln!("{stdout}{}", String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "{stdout}");
}
