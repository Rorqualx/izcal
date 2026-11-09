// izq: the child of the cross-container leakage probe (no dependencies;
// `rustc -O --edition 2024 izq.rs -o izq.exe`). GUI subsystem, so it
// (console subsystem: the GUI one died with STATUS_DLL_INIT_FAILED in the
// container). Results go to a key=value file
// (the caller passes the path; the container's own folder is writable
// by the child and readable by the driver).
//   izq.exe plant <res> <guid> <helper> <stay_secs> [sharedlist]
//   izq.exe check <res> <guid> <helper> [sharedlist]
// Channels: cred (Credential Manager CRED_PERSIST_LOCAL_MACHINE),
// cookie (WinINet), atom (global atom), gev/gmap/gpipe (Global\
// event/mapping/pipe at plant time; open at check time), wo<N>
// (shared-writable location opens, nothing written).
// NOT probed from inside the container: the clipboard and WinRT's
// PasswordVault. user32.dll fails DllMain inside an AppContainer in
// this ssh session (any child importing it dies with 0xC0000142 at
// load; verified by calling IsClipboardFormatAvailable in a minimal
// binary), and powershell.exe dies the same way, so neither the
// clipboard APIs nor the WinRT helper can run here. advapi32
// (CredWriteW) and wininet load and run fine.
#![allow(non_snake_case)]
use std::ffi::c_void;
use std::ptr::null_mut;

type H = *mut c_void;
const BAD: H = -1isize as H;
const NO_WINDOW: u32 = 0x0800_0000;
const PS: &str = r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe";

unsafe extern "system" {
    fn GlobalAddAtomW(n: *const u16) -> u16;
    fn GlobalFindAtomW(n: *const u16) -> u16;
    fn GlobalAlloc(f: u32, b: usize) -> H;
    fn GlobalLock(h: H) -> *mut u8;
    fn GlobalUnlock(h: H) -> i32;
    fn OpenEventW(a: u32, i: i32, n: *const u16) -> H;
    fn OpenFileMappingW(a: u32, i: i32, n: *const u16) -> H;
    fn CreateEventW(sa: H, m: i32, i: i32, n: *const u16) -> H;
    fn CreateFileMappingW(f: H, sa: H, p: u32, hi: u32, lo: u32, n: *const u16) -> H;
    fn CreateNamedPipeW(
        n: *const u16,
        om: u32,
        pm: u32,
        mx: u32,
        ob: u32,
        ib: u32,
        t: u32,
        sa: H,
    ) -> H;
    fn CreateFileW(n: *const u16, a: u32, s: u32, sa: H, c: u32, f: u32, t: H) -> H;
    fn CloseHandle(h: H) -> i32;
    fn GetLastError() -> u32;
    fn Sleep(ms: u32);
}
#[link(name = "advapi32")]
unsafe extern "system" {
    fn CredWriteW(c: *const CRED, f: u32) -> i32;
    fn CredReadW(t: *const u16, k: u32, f: u32, out: *mut *mut CRED) -> i32;
    fn CredEnumerateW(f: *const u16, fl: u32, n: *mut u32, out: *mut *mut *mut CRED) -> i32;
    fn CredFree(p: H);
    fn RegOpenKeyExW(k: H, s: *const u16, o: u32, a: u32, out: *mut H) -> i32;
    fn RegCloseKey(k: H) -> i32;
}
#[link(name = "wininet")]
unsafe extern "system" {
    fn InternetSetCookieW(u: *const u16, n: *const u16, d: *const u16) -> i32;
    fn InternetGetCookieW(u: *const u16, n: *const u16, b: *mut u16, l: *mut u32) -> i32;
}

#[repr(C)]
struct CRED {
    flags: u32,
    kind: u32,
    target: *const u16,
    comment: *const u16,
    last: u64,
    blob_size: u32,
    blob: *const u8,
    persist: u32,
    attr_count: u32,
    attrs: *const c_void,
    target_alias: *const u16,
    user: *const u16,
}

fn null_const<T>() -> *const T {
    std::ptr::null()
}
fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn err() -> u32 {
    unsafe { GetLastError() }
}

struct Out {
    lines: Vec<String>,
}
impl Out {
    fn add(&mut self, k: &str, v: String) {
        self.lines.push(format!("{k}={v}"));
    }
    fn save(&self, p: &str) -> bool {
        std::fs::write(p, self.lines.join("\n") + "\n").is_ok()
    }
}

fn atom(plant: bool, guid: &str, o: &mut Out) {
    let name = w(&format!("izq-atom-{guid}"));
    unsafe {
        if plant {
            let a = GlobalAddAtomW(name.as_ptr());
            if a == 0 {
                o.add("atom_new", format!("ERROR({})", err()));
            } else {
                o.add("atom_new", format!("{a}"));
            }
        } else {
            let a = GlobalFindAtomW(name.as_ptr());
            o.add(
                "atom",
                if a != 0 {
                    "found".into()
                } else {
                    "gone".into()
                },
            );
        }
    }
}

fn cred(plant: bool, guid: &str, o: &mut Out) {
    let target = w(&format!("izq-cred-{guid}"));
    unsafe {
        if plant {
            let pw = format!("izq-pass-{guid}");
            let c = CRED {
                flags: 0,
                kind: 1, // CRED_TYPE_GENERIC
                target: target.as_ptr(),
                comment: null_mut(),
                last: 0,
                blob_size: pw.len() as u32,
                blob: pw.as_ptr(),
                persist: 2, // CRED_PERSIST_LOCAL_MACHINE
                attr_count: 0,
                attrs: null_mut(),
                target_alias: null_mut(),
                user: null_mut(),
            };
            if CredWriteW(&c, 0) != 0 {
                o.add("cred_new", "ok".into());
            } else {
                o.add("cred_new", format!("ERROR({})", err()));
            }
        } else {
            let mut p: *mut CRED = null_mut();
            if CredReadW(target.as_ptr(), 1, 0, &mut p) != 0 {
                o.add("cred", "found".into());
                if !p.is_null() {
                    CredFree(p.cast());
                }
            } else {
                let e = err();
                o.add(
                    "cred",
                    if e == 1168 {
                        "gone".into()
                    } else {
                        format!("ERROR({e})")
                    },
                );
            }
            let f = w("izq-*");
            let mut n: u32 = 0;
            let mut arr: *mut *mut CRED = null_mut();
            if CredEnumerateW(f.as_ptr(), 0, &mut n, &mut arr) != 0 {
                o.add("credenum", format!("{n}"));
                CredFree(arr.cast());
            } else {
                let e = err();
                o.add(
                    "credenum",
                    if e == 1168 {
                        "0".into()
                    } else {
                        format!("ERROR({e})")
                    },
                );
            }
        }
    }
}

fn cookie(plant: bool, guid: &str, o: &mut Out) {
    let url = w(&format!("https://izq-{guid}.probe/"));
    unsafe {
        if plant {
            let name = w("izq");
            let data = w(&format!("izqval={guid}"));
            if InternetSetCookieW(url.as_ptr(), name.as_ptr(), data.as_ptr()) != 0 {
                o.add("cookie_new", "ok".into());
                // read back in the setter itself: is the jar even per-process?
                let mut buf = [0u16; 4096];
                let mut len = buf.len() as u32;
                if InternetGetCookieW(url.as_ptr(), null_mut(), buf.as_mut_ptr(), &mut len) != 0 {
                    let v = String::from_utf16_lossy(&buf[..(len as usize).min(buf.len())]);
                    o.add(
                        "cookie_self",
                        if v.contains(guid) {
                            "found".into()
                        } else {
                            "other".into()
                        },
                    );
                } else {
                    let e = err();
                    o.add(
                        "cookie_self",
                        if e == 259 {
                            "gone".into()
                        } else {
                            format!("DENIED({e})")
                        },
                    );
                }
            } else {
                o.add("cookie_new", format!("ERROR({})", err()));
            }
        } else {
            let mut buf = [0u16; 4096];
            let mut len = buf.len() as u32;
            if InternetGetCookieW(url.as_ptr(), null_mut(), buf.as_mut_ptr(), &mut len) != 0 {
                let s = String::from_utf16_lossy(&buf[..(len as usize).min(buf.len())]);
                o.add(
                    "cookie",
                    if s.contains(guid) {
                        "found".into()
                    } else {
                        "other".into()
                    },
                );
            } else {
                let e = err();
                o.add(
                    "cookie",
                    if e == 259 {
                        "gone".into()
                    } else {
                        format!("DENIED({e})")
                    },
                );
            }
        }
    }
}

static mut KEEP_EV: H = null_mut();
static mut KEEP_MP: H = null_mut();
static mut KEEP_PIPE: H = null_mut();

fn globals(plant: bool, guid: &str, o: &mut Out) {
    let ev = w(&format!(r"Global\izq-g-{guid}"));
    let mp = w(&format!(r"Global\izq-m-{guid}"));
    let pp = w(&format!(r"\\.\pipe\izq-p-{guid}"));
    unsafe {
        if plant {
            // the handles are held until the planter ends, so the
            // objects live exactly as long as the planter does
            let h = CreateEventW(null_mut(), 1, 0, ev.as_ptr());
            o.add(
                "gev_new",
                if h.is_null() {
                    format!("DENIED({})", err())
                } else {
                    "ok".into()
                },
            );
            KEEP_EV = h;
            let h = CreateFileMappingW(BAD, null_mut(), 4, 0, 4096, mp.as_ptr());
            o.add(
                "gmap_new",
                if h.is_null() {
                    format!("DENIED({})", err())
                } else {
                    "ok".into()
                },
            );
            KEEP_MP = h;
            let h = CreateNamedPipeW(pp.as_ptr(), 3, 0, 255, 512, 512, 0, null_mut());
            o.add(
                "gpipe_new",
                if h == BAD {
                    format!("DENIED({})", err())
                } else {
                    "ok".into()
                },
            );
            KEEP_PIPE = if h == BAD { null_mut() } else { h };
        } else {
            let h = OpenEventW(1, 0, ev.as_ptr());
            if h.is_null() {
                let e = err();
                o.add(
                    "gev",
                    if e == 2 {
                        "gone".into()
                    } else {
                        format!("DENIED({e})")
                    },
                );
            } else {
                o.add("gev", "found".into());
                CloseHandle(h);
            }
            let h = OpenFileMappingW(4, 0, mp.as_ptr());
            if h.is_null() {
                let e = err();
                o.add(
                    "gmap",
                    if e == 2 {
                        "gone".into()
                    } else {
                        format!("DENIED({e})")
                    },
                );
            } else {
                o.add("gmap", "found".into());
                CloseHandle(h);
            }
            let h = CreateFileW(pp.as_ptr(), 0xC000_0000, 0, null_mut(), 3, 0, null_mut());
            if h != BAD {
                o.add("gpipe", "found".into());
                CloseHandle(h);
            } else {
                let e = err();
                o.add(
                    "gpipe",
                    if e == 2 {
                        "gone".into()
                    } else {
                        format!("DENIED({e})")
                    },
                );
            }
        }
    }
}

/// Open every location in `list` FOR WRITE but write nothing.
/// Lines: `file<TAB>path` | `dir<TAB>path` | `reg<TAB>HKx\subkey`.
fn writeopen(list: &str, o: &mut Out) {
    let t = match std::fs::read_to_string(list) {
        Ok(t) => t,
        Err(e) => {
            o.add(
                "wo_read",
                format!("ERROR({})", e.raw_os_error().unwrap_or(-1)),
            );
            return;
        }
    };
    for (i, line) in t.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let Some((kind, path)) = line.split_once('\t') else {
            continue;
        };
        let val: String = unsafe {
            match kind {
                "reg" => {
                    // a bare root name ("HKLM") is the root key itself
                    let (root, sub) = match path.split_once('\\') {
                        Some(x) => x,
                        None => (path, ""),
                    };
                    let mut h: H = null_mut();
                    let base: usize = match root {
                        "HKLM" | "HKEY_LOCAL_MACHINE" => 0x8000_0002,
                        "HKCR" | "HKEY_CLASSES_ROOT" => 0x8000_0000,
                        "HKU" | "HKEY_USERS" => 0x8000_0003,
                        _ => 0x8000_0001,
                    };
                    let rkbuf = w(sub);
                    let rk: *const u16 = if sub.is_empty() {
                        null_const()
                    } else {
                        rkbuf.as_ptr()
                    };
                    let r = RegOpenKeyExW(base as H, rk, 0, 0x6, &mut h); // SET_VALUE|CREATE_SUB_KEY
                    if r == 0 {
                        RegCloseKey(h);
                        "ok".into()
                    } else if r == 2 {
                        "gone(2)".into()
                    } else if r == 5 {
                        "DENIED(5)".into()
                    } else {
                        format!("ERROR({r})")
                    }
                }
                "dir" => {
                    let p = w(path);
                    // FILE_ADD_FILE|FILE_ADD_SUBDIRECTORY, OPEN_EXISTING, BACKUP_SEMANTICS
                    let h = CreateFileW(
                        p.as_ptr(),
                        0x2 | 0x4,
                        0,
                        null_mut(),
                        3,
                        0x0200_0000,
                        null_mut(),
                    );
                    if h == BAD {
                        let e = err();
                        if e == 2 {
                            "gone(2)".into()
                        } else if e == 5 {
                            "DENIED(5)".into()
                        } else {
                            format!("ERROR({e})")
                        }
                    } else {
                        CloseHandle(h);
                        "ok".into()
                    }
                }
                _ => {
                    let p = w(path);
                    let h = CreateFileW(p.as_ptr(), 0x4000_0000, 0, null_mut(), 3, 0, null_mut()); // GENERIC_WRITE, OPEN_EXISTING
                    if h == BAD {
                        let e = err();
                        if e == 2 {
                            "gone(2)".into()
                        } else if e == 5 {
                            "DENIED(5)".into()
                        } else {
                            format!("ERROR({e})")
                        }
                    } else {
                        CloseHandle(h);
                        "ok".into()
                    }
                }
            }
        };
        o.add(&format!("wo{i}"), val);
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 5 {
        return;
    }
    let plant = a[1] == "plant";
    // a[4] (helper path) is kept for CLI compatibility; the vault
    // helper cannot run in a container here (user32 DllMain), so the
    // PasswordVault channel is driver-side documentation only.
    let (res, guid) = (a[2].clone(), a[3].clone());
    let mut o = Out { lines: Vec::new() };
    let trace = std::env::var("IZQ_TRACE").is_ok();
    atom(plant, &guid, &mut o);
    if !o.save(&res) {
        std::process::exit(90);
    }
    if trace {
        std::process::exit(61);
    }
    cred(plant, &guid, &mut o);
    o.save(&res);
    if trace {
        std::process::exit(62);
    }
    cookie(plant, &guid, &mut o);
    o.save(&res);
    if trace {
        std::process::exit(63);
    }
    globals(plant, &guid, &mut o);
    o.save(&res);
    if trace {
        std::process::exit(64);
    }
    if !o.save(&res) {
        std::process::exit(91);
    }
    let list = if plant { a.get(6) } else { a.get(5) };
    if let Some(l) = list {
        writeopen(l, &mut o);
    }
    if plant {
        o.add("done", "1".to_string());
    }
    o.save(&res);
    if plant {
        if let Some(s) = a.get(5).and_then(|x| x.parse::<u64>().ok()) {
            for _ in 0..s {
                unsafe { Sleep(1000) };
            }
        }
    }
}
