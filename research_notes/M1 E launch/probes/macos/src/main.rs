//! M1 phase E, SEC-8's macOS item: a recorded SDK 27 result for custom
//! sandbox profiles.
//!
//! Written from Apple's manual pages only (`sandbox_init(3)`,
//! `sandbox_check(3)`, `sandbox-exec(1)`); black-box.
//!
//! Each mode runs in its own process (a sandbox cannot be removed), and
//! the driver (`all`) runs every mode as a child of itself and records
//! its exit status or signal:
//! - `baseline`: no sandbox; the same operations, for contrast.
//! - `izcal`: the code's own path, `hg_substrate_os::sandbox::apply()`
//!   (the empty-process profile through `sandbox_init_with_parameters`),
//!   then `sandbox::probe_line()` and real operations.
//! - `custom-allow`: a deny-default custom profile with one allow rule
//!   (read of one literal file) through the same API: the allowed read
//!   works and its neighbour is denied, so rules are enforced, not only
//!   the default.
//! - `named`: the built-in named profile `kSBXProfilePureComputation`
//!   through `sandbox_init(…, SANDBOX_NAMED, …)`, which the SDK 27
//!   manual page says kills the process.
//!
//! Usage: `e-macos-sbpl-probe all <allowed-file> <denied-file>`.
#![allow(unsafe_code, clippy::all)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::process::Command;

unsafe extern "C" {
    fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
    fn sandbox_init_with_parameters(
        profile: *const c_char,
        flags: u64,
        parameters: *const *const c_char,
        errorbuf: *mut *mut c_char,
    ) -> c_int;
    /// The exported profile name: the symbol's address is the string
    /// (read below only after checking it is printable).
    static kSBXProfilePureComputation: c_char;
}

/// `SANDBOX_NAMED` (sandbox_init(3)).
const SANDBOX_NAMED: u64 = 1;

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
}

fn open_read(path: &str) -> String {
    let c = CString::new(path).unwrap();
    let fd = unsafe { libc::open(c.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        format!("errno={}", errno())
    } else {
        unsafe { libc::close(fd) };
        "OK".into()
    }
}

/// A TCP connect to 127.0.0.1:9 (discard; nothing need listen): EPERM
/// means the sandbox refused it, ECONNREFUSED that it went out.
fn connect_out() -> String {
    let s = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    if s < 0 {
        return format!("socket-errno={}", errno());
    }
    let mut a: libc::sockaddr_in = unsafe { std::mem::zeroed() };
    a.sin_len = std::mem::size_of::<libc::sockaddr_in>() as u8;
    a.sin_family = libc::AF_INET as u8;
    a.sin_port = 9u16.to_be();
    a.sin_addr.s_addr = u32::from_be_bytes([127, 0, 0, 1]).to_be();
    let rc = unsafe {
        libc::connect(
            s,
            (&raw const a).cast(),
            std::mem::size_of::<libc::sockaddr_in>() as u32,
        )
    };
    let r = if rc < 0 {
        format!("errno={}", errno())
    } else {
        "OK".into()
    };
    unsafe { libc::close(s) };
    r
}

fn ops(allowed: &str, denied: &str) -> String {
    format!(
        "open(allowed)={} open(denied)={} open(/etc/hosts)={} connect(127.0.0.1:9)={}",
        open_read(allowed),
        open_read(denied),
        open_read("/etc/hosts"),
        connect_out()
    )
}

fn err_text(rc: c_int, buf: *mut c_char) -> String {
    if buf.is_null() {
        format!("rc={rc}")
    } else {
        format!("rc={rc} err={:?}", unsafe { CStr::from_ptr(buf) })
    }
}

fn mode(m: &str, allowed: &str, denied: &str) {
    match m {
        "baseline" => println!("baseline: {}", ops(allowed, denied)),
        "izcal" => {
            let r = hg_substrate_os::sandbox::apply();
            println!("izcal: apply={r:?}");
            println!(
                "izcal: probe_line {}",
                hg_substrate_os::sandbox::probe_line()
            );
            println!("izcal: {}", ops(allowed, denied));
        }
        "custom-allow" => {
            let profile = CString::new(format!(
                "(version 1)\n(deny default)\n(allow sysctl-read)\n(allow file-read* (literal \"{allowed}\"))\n"
            ))
            .unwrap();
            let mut buf: *mut c_char = std::ptr::null_mut();
            let rc = unsafe {
                sandbox_init_with_parameters(profile.as_ptr(), 0, std::ptr::null(), &mut buf)
            };
            println!("custom-allow: init {}", err_text(rc, buf));
            println!("custom-allow: {}", ops(allowed, denied));
        }
        "named" => {
            let mut buf: *mut c_char = std::ptr::null_mut();
            println!("named: calling sandbox_init(kSBXProfilePureComputation, SANDBOX_NAMED)");
            use std::io::Write;
            let _ = std::io::stdout().flush();
            let name = &raw const kSBXProfilePureComputation;
            let bytes = unsafe { std::slice::from_raw_parts(name.cast::<u8>(), 32) };
            let end = bytes.iter().position(|&b| b == 0);
            let printable =
                end.is_some_and(|e| e > 0 && bytes[..e].iter().all(|b| b.is_ascii_graphic()));
            if !printable {
                println!("named: the symbol is not a C string here; not called");
                return;
            }
            println!("named: profile name {:?}", unsafe { CStr::from_ptr(name) });
            let _ = std::io::stdout().flush();
            let rc = unsafe { sandbox_init(name, SANDBOX_NAMED, &mut buf) };
            println!("named: init {}", err_text(rc, buf));
            println!("named: {}", ops(allowed, denied));
        }
        other => {
            eprintln!("unknown mode {other}");
            std::process::exit(2);
        }
    }
}

fn main() {
    let mut a: Vec<String> = std::env::args().collect();
    // SBPL literals match the resolved path (/var is /private/var).
    for i in [2, 3] {
        if let Ok(p) = std::fs::canonicalize(&a[i]) {
            a[i] = p.display().to_string();
        }
    }
    let (m, allowed, denied) = (a[1].as_str(), a[2].as_str(), a[3].as_str());
    if m != "all" {
        mode(m, allowed, denied);
        return;
    }
    for m in ["baseline", "izcal", "custom-allow", "named"] {
        let out = Command::new(&a[0])
            .args([m, allowed, denied])
            .output()
            .unwrap();
        print!("{}", String::from_utf8_lossy(&out.stdout));
        let st = out.status;
        use std::os::unix::process::ExitStatusExt;
        println!("STATUS {m} code={:?} signal={:?}", st.code(), st.signal());
    }
}
