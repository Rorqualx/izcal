//! The M1 exit test on this OS (launch design E-R3, phase E "done
//! when"): the browser, renderer and network service, each its own
//! process in its sandbox, make the round trip. Runs `hg-launch-tool
//! round-trip` from the build directory, where `cargo test` also builds
//! the mains (each main crate's own test makes Cargo build it).

use std::path::Path;
use std::process::Command;

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

/// The dylibs a thin 64-bit Mach-O executable names in its load
/// commands (Apple's Mach-O format: a 32-byte header with the command
/// count at offset 16, then `cmd`, `cmdsize` and, for the dylib
/// commands, the name's offset at +8).
#[cfg(target_os = "macos")]
fn linked_dylibs(path: &Path) -> Vec<String> {
    const LOAD_DYLIB: [u32; 5] = [0xc, 0x20, 0x8000_0018, 0x8000_001f, 0x8000_0023];
    let b = std::fs::read(path).unwrap();
    let word = |at: usize| u32::from_le_bytes(b[at..at + 4].try_into().unwrap());
    assert_eq!(
        word(0),
        0xfeed_facf,
        "{} is not a thin 64-bit Mach-O",
        path.display()
    );
    let mut at = 32;
    let mut out = Vec::new();
    for _ in 0..word(16) {
        let (cmd, size) = (word(at), word(at + 4) as usize);
        if LOAD_DYLIB.contains(&cmd) {
            let name = &b[at + word(at + 8) as usize..at + size];
            let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            out.push(String::from_utf8_lossy(&name[..end]).into_owned());
        }
        at += size;
    }
    out
}

/// M1 phase G (FP-2 on macOS): the code-signing frameworks the broker's
/// launcher uses are loaded at run time by the broker, never linked, so
/// no sandboxed child loads CoreFoundation or Security (and the
/// Foundation, Swift and XPC initialisers behind them): linked, they
/// cost each child about 464 KiB of `phys_footprint`. libSystem is the
/// control that the load commands were read.
#[cfg(target_os = "macos")]
#[test]
fn no_child_links_the_code_signing_frameworks() {
    let tool = Path::new(env!("CARGO_BIN_EXE_hg-launch-tool"));
    let dir = tool.parent().unwrap();
    for exe in ["hg-renderer-main", "hg-net-main"] {
        let libs = linked_dylibs(&dir.join(exe));
        assert!(
            libs.iter().any(|l| l.ends_with("/libSystem.B.dylib")),
            "{exe}: {libs:?}"
        );
        for framework in ["CoreFoundation.framework", "Security.framework"] {
            assert!(
                !libs.iter().any(|l| l.contains(framework)),
                "{exe} links {framework}: {libs:?}"
            );
        }
    }
}
