//! The M1 exit test on this OS (launch design E-R3, phase E "done
//! when"): the browser, renderer and network service, each its own
//! process in its sandbox, make the round trip. Runs `hg-launch-tool
//! round-trip` from the build directory.
//!
//! The mains it launches are not dependencies of this package, so Cargo
//! builds them only when their own packages' tests are selected; `cargo
//! test -p hg-launch-tool` alone would run whatever older build sat in
//! the directory (M1 phase G found this with the thread count). So the
//! first test of a run builds them, in this tool's target directory,
//! target and profile ([`build_dir`]).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The mains the tool launches (and the browser main beside them).
const MAINS: [&str; 3] = ["hg-renderer-main", "hg-net-main", "hg-browser-main"];

/// The tool's directory, `<target>/[<triple>/]<profile>`, with [`MAINS`]
/// built there from this source, once per test process.
fn build_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let tool = Path::new(env!("CARGO_BIN_EXE_hg-launch-tool"));
        let dir = tool.parent().unwrap().to_path_buf();
        let profile = dir.file_name().unwrap().to_os_string();
        let is_root = |d: &Path| d.join("CACHEDIR.TAG").is_file();
        let above = dir.parent().unwrap();
        let (root, triple) = if is_root(above) {
            (above.to_path_buf(), None)
        } else {
            let root = above.parent().unwrap();
            assert!(
                is_root(root),
                "{} is not in a target directory",
                dir.display()
            );
            (
                root.to_path_buf(),
                Some(above.file_name().unwrap().to_os_string()),
            )
        };
        let mut cmd = Command::new(env!("CARGO"));
        cmd.arg("build").current_dir(env!("CARGO_MANIFEST_DIR"));
        for m in MAINS {
            cmd.args(["-p", m]);
        }
        // `dev` and `test` both build into `debug`.
        if profile == "debug" {
            cmd.args(["--profile", "dev"]);
        } else {
            cmd.arg("--profile").arg(&profile);
        }
        cmd.arg("--target-dir").arg(&root);
        if let Some(t) = triple {
            cmd.arg("--target").arg(t);
        }
        let out = cmd.output().unwrap();
        assert!(
            out.status.success(),
            "building the mains failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        dir
    })
}

/// The tool, with the mains it launches built beside it.
fn tool() -> PathBuf {
    build_dir().join(format!("hg-launch-tool{}", std::env::consts::EXE_SUFFIX))
}

#[test]
fn the_round_trip_runs_in_the_os_sandboxes() {
    let out = Command::new(tool()).arg("round-trip").output().unwrap();
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
    let dir = build_dir();
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

/// M1 phase G (FP-27 on Windows): the child executables are built for
/// the GUI subsystem, so a launch starts no conhost.exe for them (FP-M0
/// decision log, 23 October 2026; measured again in G: a console child
/// got its own conhost, which cost about 18 ms of spawn-to-ready and
/// 944 KiB private per child). The PE optional header's `Subsystem`
/// field: 2 is the Windows GUI, 3 the console.
#[cfg(windows)]
#[test]
fn the_children_are_built_for_the_gui_subsystem() {
    let dir = build_dir();
    for exe in ["hg-renderer-main.exe", "hg-net-main.exe"] {
        let b = std::fs::read(dir.join(exe)).unwrap();
        let pe = u32::from_le_bytes(b[0x3c..0x40].try_into().unwrap()) as usize;
        assert_eq!(&b[pe..pe + 4], b"PE\0\0", "{exe}");
        let subsystem = u16::from_le_bytes(b[pe + 24 + 68..pe + 24 + 70].try_into().unwrap());
        assert_eq!(subsystem, 2, "{exe} is not a GUI-subsystem executable");
    }
}

/// M1 phase G: a broker that ends normally unregisters every AppContainer
/// profile it registered, its spares included. The browser and its
/// children held each other (the serving handlers held the browser), so
/// before the fix no launcher was ever dropped and every profile stayed.
/// Profiles are named `izcal.l.<broker pid>.<start>.<n>`.
#[cfg(windows)]
#[test]
fn a_broker_that_ends_leaves_no_profile() {
    let child = Command::new(tool())
        .arg("round-trip")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let packages = Path::new(&std::env::var_os("LOCALAPPDATA").unwrap()).join("Packages");
    let prefix = format!("izcal.l.{pid}.");
    let left: Vec<String> = std::fs::read_dir(packages)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with(&prefix))
        .collect();
    assert!(left.is_empty(), "left registered: {left:?}");
}

/// The OS threads of process `pid` now, read from outside: Linux
/// `/proc/PID/status`, macOS `ps -M` (one line per thread), Windows
/// `Get-Process` (FP-M0's counters, by process).
fn os_threads(pid: u32) -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        let s = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
        s.lines()
            .find_map(|l| l.strip_prefix("Threads:"))
            .and_then(|n| n.trim().parse().ok())
    }
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("ps")
            .args(["-M", "-p", &pid.to_string()])
            .output()
            .ok()?;
        let lines = String::from_utf8_lossy(&out.stdout).lines().count();
        lines.checked_sub(1).filter(|n| *n > 0)
    }
    #[cfg(windows)]
    {
        let out = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-Process -Id {pid}).Threads.Count"),
            ])
            .output()
            .ok()?;
        String::from_utf8_lossy(&out.stdout).trim().parse().ok()
    }
}

/// FP 6.3 and FP-32 (M1 phase G): an idle empty site process has two
/// threads, and so does the network service beside it. Each launched
/// child runs its sequences on `main` and its IPC on `hg-ipc-io`; a
/// MayBlock worker and the timer thread start only when used (CS-AS-3:
/// the substrate's count is the process's). Counted from outside, on
/// each OS. On Windows the loader's own pool threads (FP-M0: three the
/// process does not create) end by themselves within about 45 s, so the
/// count is awaited for up to 90 s.
#[test]
fn an_idle_child_runs_two_threads() {
    use std::io::BufRead as _;
    let mut broker = Command::new(tool())
        .args(["hold", "150"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let out = std::io::BufReader::new(broker.stdout.take().unwrap());
    let mut pids = Vec::new();
    for line in out.lines() {
        let line = line.unwrap();
        eprintln!("{line}");
        if let Some(rest) = line.strip_prefix("hold: children ") {
            for kv in rest.split_whitespace() {
                let (kind, pid) = kv.split_once('=').unwrap();
                pids.push((kind.to_owned(), pid.parse::<u32>().unwrap()));
            }
            break;
        }
        assert!(!line.contains("FAIL"), "{line}");
    }
    assert_eq!(pids.len(), 2, "both children reported ready: {pids:?}");
    let started = std::time::Instant::now();
    let mut seen = Vec::new();
    for (kind, pid) in &pids {
        let mut n = os_threads(*pid);
        while n != Some(2) && started.elapsed() < std::time::Duration::from_secs(90) {
            std::thread::sleep(std::time::Duration::from_millis(500));
            n = os_threads(*pid);
        }
        seen.push((kind.clone(), n));
    }
    let _ = broker.kill();
    let _ = broker.wait();
    eprintln!("threads after {:?}: {seen:?}", started.elapsed());
    for (kind, n) in seen {
        assert_eq!(n, Some(2), "{kind}");
    }
}
