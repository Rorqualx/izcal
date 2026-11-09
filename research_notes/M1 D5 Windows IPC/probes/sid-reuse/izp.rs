// izp: the child of the SID-reuse probe (no dependencies; `rustc -O izp.rs`).
//   izp plant ACDIR [exit|stay|orphan]   izp hold ACDIR   izp check ACDIR
// check's exit code: bit per channel found, plus (bit << 8) per channel whose
// probe failed in another way. plant's: 0, or 256 + bits that failed.
// Channels: 1 event, 2 mutex, 4 section (content 'A'), 8 pipe,
//           16 file in the container folder, 32 container registry value.
#![allow(non_snake_case)]
use std::ffi::c_void;
use std::ptr::null_mut;
type H = *mut c_void;
const BAD: H = -1isize as H;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateEventW(sa: H, m: i32, i: i32, n: *const u16) -> H;
    fn OpenEventW(a: u32, h: i32, n: *const u16) -> H;
    fn CreateMutexW(sa: H, o: i32, n: *const u16) -> H;
    fn OpenMutexW(a: u32, h: i32, n: *const u16) -> H;
    fn CreateFileMappingW(f: H, sa: H, p: u32, hi: u32, lo: u32, n: *const u16) -> H;
    fn OpenFileMappingW(a: u32, h: i32, n: *const u16) -> H;
    fn MapViewOfFile(m: H, a: u32, hi: u32, lo: u32, n: usize) -> *mut u8;
    fn CreateNamedPipeW(
        n: *const u16,
        om: u32,
        pm: u32,
        max: u32,
        ob: u32,
        ib: u32,
        t: u32,
        sa: H,
    ) -> H;
    fn CreateFileW(n: *const u16, a: u32, s: u32, sa: H, c: u32, f: u32, t: H) -> H;
    fn CloseHandle(h: H) -> i32;
    fn GetLastError() -> u32;
}
#[link(name = "userenv")]
unsafe extern "system" {
    fn GetAppContainerRegistryLocation(access: u32, key: *mut H) -> i32;
}
#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegSetValueExW(k: H, n: *const u16, r: u32, t: u32, d: *const u8, len: u32) -> i32;
    fn RegQueryValueExW(
        k: H,
        n: *const u16,
        r: *mut u32,
        t: *mut u32,
        d: *mut u8,
        len: *mut u32,
    ) -> i32;
    fn RegCloseKey(k: H) -> i32;
}

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
const EV: &str = r"Local\izp-ev";
const MX: &str = r"Local\izp-mx";
const SEC: &str = r"Local\izp-sec";
const PIPE: &str = r"\\.\pipe\LOCAL\izp-pipe";

fn ac_key() -> Option<H> {
    let mut k = null_mut();
    (unsafe { GetAppContainerRegistryLocation(0xF003F, &mut k) } == 0).then_some(k)
}

fn make_named() -> u32 {
    let mut fail = 0;
    unsafe {
        if CreateEventW(null_mut(), 1, 0, w(EV).as_ptr()).is_null() {
            fail |= 1
        }
        if CreateMutexW(null_mut(), 0, w(MX).as_ptr()).is_null() {
            fail |= 2
        }
        let s = CreateFileMappingW(BAD, null_mut(), 4, 0, 4096, w(SEC).as_ptr());
        let v = if s.is_null() {
            null_mut()
        } else {
            MapViewOfFile(s, 2, 0, 0, 0)
        };
        if v.is_null() {
            fail |= 4
        } else {
            *v = b'A'
        }
        if CreateNamedPipeW(w(PIPE).as_ptr(), 3, 0, 255, 512, 512, 0, null_mut()) == BAD {
            fail |= 8
        }
    }
    fail
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (mode, acdir) = (a[1].as_str(), std::path::Path::new(&a[2]));
    let file = acdir.join("izp.txt");
    let val = w("izp");
    let code = match mode {
        "plant" => {
            let mut fail = make_named();
            if std::fs::write(&file, "A").is_err() {
                fail |= 16
            }
            match ac_key() {
                Some(k) => unsafe {
                    let d = w("A");
                    if RegSetValueExW(k, val.as_ptr(), 0, 1, d.as_ptr().cast(), 4) != 0 {
                        fail |= 32
                    }
                    RegCloseKey(k);
                },
                None => fail |= 32,
            }
            match a.get(3).map(String::as_str) {
                Some("stay") => std::thread::sleep(std::time::Duration::from_secs(120)),
                Some("orphan") => {
                    std::process::Command::new(std::env::current_exe().unwrap())
                        .args(["hold", &a[2]])
                        .spawn()
                        .unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
                _ => {}
            }
            if fail == 0 { 0 } else { 256 + fail }
        }
        "hold" => {
            make_named();
            std::thread::sleep(std::time::Duration::from_secs(120));
            0
        }
        _ => {
            let (mut found, mut odd) = (0u32, 0u32);
            let mut probe = |bit: u32, h: H, bad: H| {
                if h != bad {
                    found |= bit
                } else if unsafe { GetLastError() } != 2 {
                    odd |= bit
                }
            };
            unsafe {
                probe(1, OpenEventW(0x100000, 0, w(EV).as_ptr()), null_mut());
                probe(2, OpenMutexW(0x100000, 0, w(MX).as_ptr()), null_mut());
                let s = OpenFileMappingW(4, 0, w(SEC).as_ptr());
                if s.is_null() {
                    if GetLastError() != 2 {
                        odd |= 4
                    }
                } else {
                    let v = MapViewOfFile(s, 4, 0, 0, 0);
                    if !v.is_null() && *v == b'A' {
                        found |= 4
                    } else {
                        odd |= 4
                    }
                }
                let p = CreateFileW(
                    w(PIPE).as_ptr(),
                    0xC000_0000,
                    0,
                    null_mut(),
                    3,
                    0,
                    null_mut(),
                );
                if p != BAD {
                    found |= 8;
                    CloseHandle(p);
                } else {
                    match GetLastError() {
                        231 => found |= 8,
                        2 => {}
                        _ => odd |= 8,
                    }
                }
            }
            match std::fs::exists(&file) {
                Ok(true) => found |= 16,
                Ok(false) => {}
                Err(_) => odd |= 16,
            }
            match ac_key() {
                Some(k) => unsafe {
                    let mut len = 0u32;
                    match RegQueryValueExW(
                        k,
                        val.as_ptr(),
                        null_mut(),
                        null_mut(),
                        null_mut(),
                        &mut len,
                    ) {
                        0 => found |= 32,
                        2 => {}
                        _ => odd |= 32,
                    }
                    RegCloseKey(k);
                },
                None => odd |= 32,
            }
            found | (odd << 8)
        }
    };
    std::process::exit(code as i32);
}
