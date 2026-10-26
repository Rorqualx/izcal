//! M1 D6 probe child: lowers itself with the repo's sandbox profile, then
//! runs commands read from its inherited stdin pipe and answers on its
//! stdout pipe, each answer ending with "END". Handle values arrive as
//! decimal integers. Sources: Microsoft Learn reference pages only.
#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use std::ffi::c_void;
use std::io::{BufRead, Write};
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};

use d6probe::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::*;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::Threading::*;

struct St {
    view: *mut u8,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // `--medium`: keep the integrity the broker gave (finding 5 probes).
    let lowered = if args.iter().any(|a| a == "--medium") {
        "kept".to_string()
    } else {
        match d6probe::sandbox_windows::apply_empty_process_profile() {
            Ok(()) => "ok".to_string(),
            Err(e) => e,
        }
    };
    let mut out = std::io::stdout();
    let _ = writeln!(
        out,
        "hello pid={} lowered={} args={:?} {}",
        std::process::id(),
        lowered,
        &args[1..],
        d6probe::sandbox_windows::probe_line(None)
    );
    let _ = writeln!(out, "END");
    let _ = out.flush();
    let mut st = St { view: null_mut() };
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.is_empty() {
            continue;
        }
        if p[0] == "exit" {
            break;
        }
        for r in run(&p, &mut st) {
            let _ = writeln!(out, "{r}");
        }
        let _ = writeln!(out, "END");
        let _ = out.flush();
    }
}

fn n(s: &str) -> isize {
    s.strip_prefix("0x").map_or_else(
        || s.parse().unwrap(),
        |x| isize::from_str_radix(x, 16).unwrap(),
    )
}

fn token_ac_sid() -> String {
    let mut tok: H = null_mut();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut tok) };
    let mut buf = [0u64; 64];
    let mut len = 0;
    let ok = unsafe {
        GetTokenInformation(
            tok,
            TokenAppContainerSid,
            buf.as_mut_ptr() as *mut c_void,
            512,
            &mut len,
        )
    };
    unsafe { CloseHandle(tok) };
    if ok == 0 {
        return format!("err={}", le());
    }
    let sid =
        unsafe { (*(buf.as_ptr() as *const TOKEN_APPCONTAINER_INFORMATION)).TokenAppContainer };
    if sid.is_null() {
        "none".into()
    } else {
        sid_string(sid)
    }
}

type NtQs = unsafe extern "system" fn(H, i32, *mut c_void, usize, *mut usize) -> i32;

fn nt_query_section(h: H) -> String {
    let m = unsafe { GetModuleHandleW(w("ntdll.dll").as_ptr()) };
    let Some(f) = (unsafe { GetProcAddress(m, c"NtQuerySection".as_ptr().cast()) }) else {
        return "NtQuerySection not exported".into();
    };
    let f: NtQs = unsafe { std::mem::transmute(f) };
    let mut b = [0u64; 4];
    let mut got = 0usize;
    let st = unsafe { f(h, 0, b.as_mut_ptr().cast(), 24, &mut got) };
    format!(
        "NtQuerySection(basic) status={:#x} len={got} attrs={:#x} max_size={}",
        st as u32, b[1] as u32, b[2]
    )
}

fn vq(p: *const c_void) -> String {
    let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };
    let r = unsafe { VirtualQuery(p, &mut mbi, size_of::<MEMORY_BASIC_INFORMATION>()) };
    format!(
        "VirtualQuery r={r} RegionSize={} Protect={:#x} AllocationProtect={:#x} Type={:#x} State={:#x}",
        mbi.RegionSize, mbi.Protect, mbi.AllocationProtect, mbi.Type, mbi.State
    )
}

fn run(p: &[&str], st: &mut St) -> Vec<String> {
    let mut o = vec![];
    match p[0] {
        "info" => o.push(format!(
            "pid={} ac-sid={}",
            std::process::id(),
            token_ac_sid()
        )),
        // query <h>: what the receiver can learn about a handle it got.
        "query" => {
            let h = hp(n(p[1]));
            o.push(format!("describe {}", describe(h)));
            o.push(nt_query_section(h));
            // SAFETY: a valid section handle; a whole-section view.
            let v = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
            if v.Value.is_null() {
                o.push(format!("map READ size 0 FAILED err={}", le()));
            } else {
                o.push(format!("map READ size 0: {}", vq(v.Value)));
                unsafe { UnmapViewOfFile(v) };
            }
        }
        // map <h> <r|rw>: keep one view.
        "map" => {
            let h = hp(n(p[1]));
            let acc = if p[2] == "rw" {
                FILE_MAP_WRITE
            } else {
                FILE_MAP_READ
            };
            let v = unsafe { MapViewOfFile(h, acc, 0, 0, 0) };
            if v.Value.is_null() {
                o.push(format!("map {} FAILED err={}", p[2], le()));
            } else {
                st.view = v.Value as *mut u8;
                o.push(format!("map {} OK {}", p[2], vq(v.Value)));
            }
        }
        // offer/discard on the current view (S11).
        "offer" => {
            let len: usize = p[1].parse().unwrap();
            let r = unsafe { OfferVirtualMemory(st.view.cast(), len, VmOfferPriorityNormal) };
            o.push(format!("OfferVirtualMemory rc={r}"));
            let r = unsafe { ReclaimVirtualMemory(st.view.cast(), len) };
            o.push(format!("ReclaimVirtualMemory rc={r}"));
            let r = unsafe { DiscardVirtualMemory(st.view.cast(), len) };
            o.push(format!("DiscardVirtualMemory rc={r}"));
            let mut old = 0u32;
            let ok = unsafe { VirtualProtect(st.view.cast(), len, PAGE_NOACCESS, &mut old) };
            o.push(format!(
                "VirtualProtect(NOACCESS) ok={ok} err={}",
                if ok == 0 { le() } else { 0 }
            ));
            if ok != 0 {
                unsafe { VirtualProtect(st.view.cast(), len, old, &mut old) };
            }
        }
        "touch" => {
            let len: usize = p[1].parse().unwrap();
            let mut x = 0u64;
            for i in (0..len).step_by(4096) {
                x = x.wrapping_add(unsafe { std::ptr::read_volatile(st.view.add(i)) } as u64);
            }
            o.push(format!("touched {} pages sum {x}", len / 4096));
        }
        "fill" => {
            let len: usize = p[1].parse().unwrap();
            for i in (0..len).step_by(4096) {
                unsafe { std::ptr::write_volatile(st.view.add(i), 1) };
            }
            o.push(format!("wrote {} pages", len / 4096));
        }
        "unmap" => {
            let ok = unsafe {
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: st.view.cast(),
                })
            };
            st.view = std::ptr::null_mut();
            o.push(format!("unmap ok={ok}"));
        }
        "close" => {
            let ok = unsafe { CloseHandle(hp(n(p[1]))) };
            o.push(format!("close ok={ok}"));
        }
        "write" => {
            let off: usize = p[1].parse().unwrap();
            let s = p[2].as_bytes();
            unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), st.view.add(off), s.len()) };
            o.push(format!("wrote {:?} at {off}", p[2]));
        }
        "read" => {
            let off: usize = p[1].parse().unwrap();
            let len: usize = p[2].parse().unwrap();
            let b = unsafe { std::slice::from_raw_parts(st.view.add(off), len) };
            o.push(format!("read {:?} at {off}", String::from_utf8_lossy(b)));
        }
        // widen <h>: every way the consumer might get write access.
        "widen" => {
            let h = hp(n(p[1]));
            // SAFETY: a valid section handle; a whole-section view.
            let v = unsafe { MapViewOfFile(h, FILE_MAP_WRITE, 0, 0, 0) };
            o.push(format!(
                "MapViewOfFile(WRITE) {}",
                if v.Value.is_null() {
                    format!("FAILED err={}", le())
                } else {
                    "MAPPED".into()
                }
            ));
            // SAFETY: a valid section handle; a whole-section view.
            let v = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, 0) };
            if !v.Value.is_null() {
                let mut old = 0u32;
                let ok = unsafe { VirtualProtect(v.Value, 4096, PAGE_READWRITE, &mut old) };
                o.push(format!(
                    "VirtualProtect(view, RW) ok={ok} err={}",
                    if ok == 0 { le() } else { 0 }
                ));
                let ok = unsafe { VirtualProtect(v.Value, 4096, PAGE_WRITECOPY, &mut old) };
                o.push(format!(
                    "VirtualProtect(view, WRITECOPY) ok={ok} err={}",
                    if ok == 0 { le() } else { 0 }
                ));
                unsafe { UnmapViewOfFile(v) };
            }
            for (label, acc) in [
                ("FILE_MAP_WRITE", FILE_MAP_WRITE),
                ("FILE_MAP_ALL_ACCESS", FILE_MAP_ALL_ACCESS),
                ("SECTION_QUERY", SECTION_QUERY),
                ("SECTION_EXTEND_SIZE", SECTION_EXTEND_SIZE),
                ("WRITE_DAC", 0x0004_0000), // Learn, "Standard Access Rights"
            ] {
                o.push(match dup(me(), h, me(), acc, 0) {
                    Ok(h2) => format!("DuplicateHandle(self, {label}) OK -> {}", describe(h2)),
                    Err(e) => format!("DuplicateHandle(self, {label}) FAILED err={e}"),
                });
            }
            let v = unsafe { MapViewOfFile(h, FILE_MAP_COPY, 0, 0, 0) };
            o.push(format!(
                "MapViewOfFile(COPY) {}",
                if v.Value.is_null() {
                    format!("FAILED err={}", le())
                } else {
                    "MAPPED (private copy-on-write)".into()
                }
            ));
        }
        "steal" => {
            // A hostile holder goes around its handle: it opens the
            // broker's process and duplicates the broker's own handle.
            let (pid, v) = (n(p[1]) as u32, hp(n(p[2])));
            // SAFETY: plain call; a null result is reported.
            let bp = unsafe { OpenProcess(PROCESS_DUP_HANDLE, 0, pid) };
            if bp.is_null() {
                o.push(format!("OpenProcess(DUP_HANDLE) FAILED err={}", le()));
            } else {
                o.push("OpenProcess(DUP_HANDLE) OK".into());
                match dup(bp, v, me(), FILE_MAP_WRITE | FILE_MAP_READ, 0) {
                    Ok(h) => {
                        o.push(format!(
                            "DuplicateHandle(broker's, RW) OK -> {}",
                            describe(h)
                        ));
                        // SAFETY: a valid section handle; a whole-section view.
                        let w = unsafe { MapViewOfFile(h, FILE_MAP_WRITE, 0, 0, 0) };
                        if w.Value.is_null() {
                            o.push(format!("MapViewOfFile(WRITE) FAILED err={}", le()));
                        } else {
                            // SAFETY: the writable view maps the whole section, at least 64 KiB.
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    b"stolen".as_ptr(),
                                    w.Value as *mut u8,
                                    6,
                                )
                            };
                            o.push("MapViewOfFile(WRITE) MAPPED, wrote \"stolen\"".into());
                            // SAFETY: the view mapped above.
                            unsafe { UnmapViewOfFile(w) };
                        }
                    }
                    Err(e) => o.push(format!("DuplicateHandle(broker's, RW) FAILED err={e}")),
                }
                // SAFETY: the process handle opened above.
                unsafe { CloseHandle(bp) };
            }
            // SAFETY: plain call; a null result is reported.
            let vp = unsafe { OpenProcess(PROCESS_VM_WRITE | PROCESS_VM_OPERATION, 0, pid) };
            o.push(if vp.is_null() {
                format!("OpenProcess(VM_WRITE) FAILED err={}", le())
            } else {
                // SAFETY: the process handle opened above.
                unsafe { CloseHandle(vp) };
                "OpenProcess(VM_WRITE) OK".into()
            });
        }
        "mksection" => {
            let h = unsafe {
                CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    null(),
                    PAGE_READWRITE | SEC_COMMIT,
                    0,
                    65536,
                    null(),
                )
            };
            o.push(if h.is_null() {
                format!("CreateFileMappingW FAILED err={}", le())
            } else {
                format!("made {} {}", hv(h), describe(h))
            });
        }
        _ => o.push(format!("unknown command {:?}", p)),
    }
    o
}
