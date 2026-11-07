//! Win32 helpers. Bindings come from windows-sys; the fiber functions and the
//! CET dynamic-code APIs are declared here from their Microsoft Learn
//! signatures (the CET ones resolved with GetProcAddress).

use core::ffi::c_void;
use std::mem::{size_of, zeroed};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::Threading::*;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn ConvertThreadToFiber(param: *mut c_void) -> *mut c_void;
    pub fn CreateFiberEx(
        commit: usize,
        reserve: usize,
        flags: u32,
        start: unsafe extern "system" fn(*mut c_void),
        param: *mut c_void,
    ) -> *mut c_void;
    pub fn SwitchToFiber(fiber: *mut c_void);
    pub fn DeleteFiber(fiber: *mut c_void);
}

/// PROCESS_DYNAMIC_ENFORCED_ADDRESS_RANGE (Microsoft Learn).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EnforcedRange {
    pub base: usize,
    pub size: usize,
    pub flags: u32,
}
pub const DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD: u32 = 1;
pub const DYNAMIC_ENFORCED_ADDRESS_RANGE_PROCESSED: u32 = 2;

/// PROCESS_DYNAMIC_EH_CONTINUATION_TARGET (Microsoft Learn).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EhTarget {
    pub addr: usize,
    pub flags: usize,
}
pub const DYNAMIC_EH_CONTINUATION_TARGET_ADD: usize = 1;

type SetRangesFn = unsafe extern "system" fn(HANDLE, u16, *mut EnforcedRange) -> i32;
type SetEhFn = unsafe extern "system" fn(HANDLE, u16, *mut EhTarget) -> i32;

fn kernel_proc(name: &[u8]) -> Option<unsafe extern "system" fn() -> isize> {
    for dll in [&b"kernelbase.dll\0"[..], &b"kernel32.dll\0"[..]] {
        let m = unsafe { GetModuleHandleA(dll.as_ptr()) };
        if m.is_null() {
            continue;
        }
        let p = unsafe { GetProcAddress(m, name.as_ptr()) };
        if p.is_some() {
            return p;
        }
    }
    None
}

pub fn set_cet_ranges(proc_: HANDLE, ranges: &mut [EnforcedRange]) -> Result<(), u32> {
    let f = kernel_proc(b"SetProcessDynamicEnforcedCetCompatibleRanges\0").ok_or(0xFFFF_FFFFu32)?;
    let f: SetRangesFn = unsafe { core::mem::transmute(f) };
    let ok = unsafe { f(proc_, ranges.len() as u16, ranges.as_mut_ptr()) };
    if ok != 0 {
        Ok(())
    } else {
        Err(unsafe { GetLastError() })
    }
}

pub fn set_eh_targets(proc_: HANDLE, t: &mut [EhTarget]) -> Result<(), u32> {
    let f = kernel_proc(b"SetProcessDynamicEHContinuationTargets\0").ok_or(0xFFFF_FFFFu32)?;
    let f: SetEhFn = unsafe { core::mem::transmute(f) };
    let ok = unsafe { f(proc_, t.len() as u16, t.as_mut_ptr()) };
    if ok != 0 {
        Ok(())
    } else {
        Err(unsafe { GetLastError() })
    }
}

/// PROCESS_MITIGATION_USER_SHADOW_STACK_POLICY flags of a process.
pub fn shadow_policy(p: HANDLE) -> Result<u32, u32> {
    let mut flags: u32 = 0;
    let ok = unsafe {
        GetProcessMitigationPolicy(
            p,
            ProcessUserShadowStackPolicy,
            &mut flags as *mut u32 as *mut c_void,
            4,
        )
    };
    if ok != 0 {
        Ok(flags)
    } else {
        Err(unsafe { GetLastError() })
    }
}

pub fn set_shadow_policy(flags: u32) -> Result<(), u32> {
    let ok = unsafe {
        SetProcessMitigationPolicy(
            ProcessUserShadowStackPolicy,
            &flags as *const u32 as *const c_void,
            4,
        )
    };
    if ok != 0 {
        Ok(())
    } else {
        Err(unsafe { GetLastError() })
    }
}

pub fn decode_policy(f: u32) -> String {
    const NAMES: [&str; 10] = [
        "EnableUserShadowStack",
        "AuditUserShadowStack",
        "SetContextIpValidation",
        "AuditSetContextIpValidation",
        "EnableUserShadowStackStrictMode",
        "BlockNonCetBinaries",
        "BlockNonCetBinariesNonEhcont",
        "AuditBlockNonCetBinaries",
        "CetDynamicApisOutOfProcOnly",
        "SetContextIpValidationRelaxedMode",
    ];
    let v: Vec<&str> = (0..10)
        .filter(|i| f & (1 << i) != 0)
        .map(|i| NAMES[i])
        .collect();
    format!("0x{f:03x} [{}]", v.join(" "))
}

pub fn last_error() -> u32 {
    unsafe { GetLastError() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub base: usize,
    pub alloc_base: usize,
    pub size: usize,
    pub state: u32,
    pub protect: u32,
    pub alloc_protect: u32,
    pub kind: u32,
}

pub fn query(addr: usize) -> Option<Region> {
    let mut m: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };
    let n = unsafe {
        VirtualQuery(
            addr as *const c_void,
            &mut m,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if n == 0 {
        return None;
    }
    Some(Region {
        base: m.BaseAddress as usize,
        alloc_base: m.AllocationBase as usize,
        size: m.RegionSize,
        state: m.State,
        protect: m.Protect,
        alloc_protect: m.AllocationProtect,
        kind: m.Type,
    })
}

pub fn all_regions() -> Vec<Region> {
    let mut v = Vec::new();
    let mut a: usize = 0x10000;
    while a < 0x7FFF_FFFF_0000 {
        match query(a) {
            Some(r) => {
                if r.state != MEM_FREE {
                    v.push(r);
                }
                a = r.base + r.size.max(0x1000);
            }
            None => break,
        }
    }
    v
}

pub fn fmt_region(r: &Region) -> String {
    let state = match r.state {
        MEM_COMMIT => "COMMIT",
        MEM_RESERVE => "RESERVE",
        _ => "?",
    };
    let kind = match r.kind {
        MEM_PRIVATE => "PRIVATE",
        MEM_MAPPED => "MAPPED",
        MEM_IMAGE => "IMAGE",
        _ => "?",
    };
    format!(
        "base=0x{:x} alloc=0x{:x} size=0x{:x} {state} {kind} protect=0x{:x} alloc_protect=0x{:x}",
        r.base, r.alloc_base, r.size, r.protect, r.alloc_protect
    )
}

/// Reads one u64 of this process without faulting.
pub fn peek(addr: usize) -> Option<u64> {
    let mut v: u64 = 0;
    let mut n: usize = 0;
    let ok = unsafe {
        ReadProcessMemory(
            GetCurrentProcess(),
            addr as *const c_void,
            &mut v as *mut u64 as *mut c_void,
            8,
            &mut n,
        )
    };
    if ok != 0 && n == 8 { Some(v) } else { None }
}

/// Restore tokens (value == address + 8, bit 0 set: 64-bit mode) found in a
/// committed region.
pub fn scan_tokens(r: &Region) -> Vec<(usize, u64)> {
    let mut out = Vec::new();
    if r.state != MEM_COMMIT {
        return out;
    }
    let mut a = r.base;
    while a < r.base + r.size {
        if let Some(v) = peek(a) {
            if v & 3 == 1 && (v & !3) as usize == a + 8 {
                out.push((a, v));
            }
        } else {
            a = (a | 0xfff) + 1;
            continue;
        }
        a += 8;
    }
    out
}

pub fn qpc() -> i64 {
    let mut t = 0i64;
    unsafe { QueryPerformanceCounter(&mut t) };
    t
}

pub fn qpf() -> f64 {
    let mut f = 0i64;
    unsafe { QueryPerformanceFrequency(&mut f) };
    f as f64
}

/// Median, p10, p90, min, max of samples (ns per operation).
pub fn stats(name: &str, mut v: Vec<f64>) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
    println!(
        "TIME {name}: n={} median={:.1}ns p10={:.1} p90={:.1} min={:.1} max={:.1}",
        v.len(),
        q(0.5),
        q(0.1),
        q(0.9),
        v[0],
        v[v.len() - 1]
    );
}

/// Times `samples` batches of `batch` calls of `f`; reports ns per call.
pub fn bench(name: &str, samples: usize, batch: usize, mut f: impl FnMut()) {
    let freq = qpf();
    for _ in 0..batch {
        f();
    }
    let mut v = Vec::with_capacity(samples);
    for _ in 0..samples {
        let t0 = qpc();
        for _ in 0..batch {
            f();
        }
        let t1 = qpc();
        v.push((t1 - t0) as f64 * 1e9 / freq / batch as f64);
    }
    stats(name, v);
}

pub fn alloc(size: usize, protect: u32) -> usize {
    let p = unsafe { VirtualAlloc(core::ptr::null(), size, MEM_RESERVE | MEM_COMMIT, protect) };
    assert!(!p.is_null(), "VirtualAlloc failed {}", last_error());
    p as usize
}
