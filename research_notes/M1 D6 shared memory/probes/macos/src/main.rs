//! M1 D6 black-box probes: Mach memory entries as Izcal's shared regions.
//! Bindings are the D4 probe crate's (signatures from developer.apple.com
//! kernel reference pages; `mach_vm_map` from the `libc` crate). Entries
//! are made from a mapped range with a plain protection (1 = READ,
//! 3 = READ|WRITE, as D4's P33), so no MAP_MEM_* flag is used.
#![allow(deprecated, clippy::all)]

mod cross;
#[path = "../../../../M1 D4 macOS IPC/probes/src/ffi.rs"]
mod ffi;
#[path = "../../../../M1 D4 macOS IPC/probes/src/mach.rs"]
#[allow(dead_code)]
mod mach;

use libc::{c_char, mach_port_t};
use std::time::Instant;

extern "C" {
    fn mach_make_memory_entry_64(
        t: mach_port_t,
        size: *mut u64,
        offset: u64,
        perm: i32,
        h: *mut mach_port_t,
        parent: mach_port_t,
    ) -> i32;
    fn mach_vm_allocate(t: mach_port_t, addr: *mut u64, size: u64, flags: i32) -> i32;
    fn mach_vm_deallocate(t: mach_port_t, addr: u64, size: u64) -> i32;
    fn mach_vm_protect(t: mach_port_t, addr: u64, size: u64, set_max: i32, prot: i32) -> i32;
    fn mach_vm_region(
        t: mach_port_t,
        addr: *mut u64,
        size: *mut u64,
        flavor: i32,
        info: *mut i32,
        cnt: *mut u32,
        obj: *mut mach_port_t,
    ) -> i32;
    fn mach_port_kobject_description(
        t: mach_port_t,
        n: u32,
        ty: *mut u32,
        addr: *mut u64,
        d: *mut c_char,
    ) -> i32;
    fn mach_port_deallocate(t: mach_port_t, n: u32) -> i32;
}

fn me() -> mach_port_t {
    unsafe { libc::mach_task_self() }
}
const SZ: u64 = 16384 * 4; // 64 KiB

fn desc(n: u32) -> String {
    let (mut ty, mut a) = (0u32, 0u64);
    let mut d = [0 as c_char; 512];
    let kr = unsafe { mach_port_kobject_description(me(), n, &mut ty, &mut a, d.as_mut_ptr()) };
    format!(
        "kobject kr={kr:#x} type={ty} desc={:?}",
        unsafe { std::ffi::CStr::from_ptr(d.as_ptr()) }.to_string_lossy()
    )
}

fn region(addr: u64) -> String {
    let (mut a, mut s, mut info, mut cnt, mut obj) = (addr, 0u64, [0i32; 16], 16u32, 0u32);
    let kr = unsafe {
        mach_vm_region(
            me(),
            &mut a,
            &mut s,
            9,
            info.as_mut_ptr(),
            &mut cnt,
            &mut obj,
        )
    };
    format!(
        "region kr={kr:#x} size={s} prot={} max={}",
        info[0], info[1]
    )
}

fn map(entry: u32, size: u64, cur: i32, max: i32, copy: i32) -> Result<u64, i32> {
    let mut addr = 0u64;
    let kr = unsafe { libc::mach_vm_map(me(), &mut addr, size, 0, 1, entry, 0, copy, cur, max, 1) };
    if kr == 0 {
        Ok(addr)
    } else {
        Err(kr)
    }
}

fn sub_entry(parent: u32, size: u64, perm: i32) -> (i32, u64, u32) {
    let mut sz = size;
    let mut h = 0u32;
    let kr = unsafe { mach_make_memory_entry_64(me(), &mut sz, 0, perm, &mut h, parent) };
    (kr, sz, h)
}

fn regions_count() -> usize {
    let mut n = 0;
    let mut a = 0u64;
    loop {
        let (mut s, mut info, mut cnt, mut obj) = (0u64, [0i32; 16], 16u32, 0u32);
        let kr = unsafe {
            mach_vm_region(
                me(),
                &mut a,
                &mut s,
                9,
                info.as_mut_ptr(),
                &mut cnt,
                &mut obj,
            )
        };
        if kr != 0 {
            break;
        }
        n += 1;
        a += s;
    }
    n
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("child") => return cross::child_main(&a[2]),
        Some("cross") => return cross::parent_main(),
        _ => {}
    }
    // The broker's side: memory, a RW entry and a RO entry from it.
    let mut src = 0u64;
    assert_eq!(unsafe { mach_vm_allocate(me(), &mut src, SZ, 1) }, 0);
    unsafe { *(src as *mut u8) = 0x11 };
    let mut sz = SZ;
    let mut rw = 0u32;
    let kr = unsafe { mach_make_memory_entry_64(me(), &mut sz, src, 3, &mut rw, 0) };
    println!(
        "M1 RW entry from a mapping: kr={kr:#x} size={sz} {}",
        desc(rw)
    );
    let (kr, s, ro_from_rw) = sub_entry(rw, SZ, 1);
    println!(
        "M1 RO entry with parent = RW entry: kr={kr:#x} size={s} {}",
        desc(ro_from_rw)
    );
    let mut sz2 = SZ;
    let mut ro_from_addr = 0u32;
    let kr = unsafe { mach_make_memory_entry_64(me(), &mut sz2, src, 1, &mut ro_from_addr, 0) };
    println!(
        "M1 RO entry from the mapping: kr={kr:#x} {}",
        desc(ro_from_addr)
    );

    // M2: what a holder of the RO entry (derived from RW) can do.
    for (label, cur, max) in [("RW/RW", 3, 3), ("R/RW", 1, 3), ("R/R", 1, 1)] {
        match map(ro_from_rw, SZ, cur, max, 0) {
            Ok(a) => {
                let up = unsafe { mach_vm_protect(me(), a, SZ, 0, 3) };
                println!(
                    "M2 RO(from RW) map {label} copy=0: ok {} reads {:#x}; protect->RW {up:#x}",
                    region(a),
                    unsafe { *(a as *const u8) }
                );
                unsafe { mach_vm_deallocate(me(), a, SZ) };
            }
            Err(e) => println!("M2 RO(from RW) map {label} copy=0: {e:#x}"),
        }
    }
    let (kr, _, h) = sub_entry(ro_from_rw, SZ, 3);
    println!("M2 RW sub-entry from RO(from RW): kr={kr:#x}");
    if kr == 0 {
        unsafe { mach_port_deallocate(me(), h) };
    }
    let (kr, _, h) = sub_entry(ro_from_rw, SZ, 1);
    println!(
        "M2 RO sub-entry from RO(from RW): kr={kr:#x} {}",
        if kr == 0 { desc(h) } else { String::new() }
    );
    if kr == 0 {
        unsafe { mach_port_deallocate(me(), h) };
    }

    // M3: size discovery by the holder.
    for (label, ask) in [
        ("0", 0u64),
        ("SZ/2", SZ / 2),
        ("SZ", SZ),
        ("SZ+16K", SZ + 16384),
        ("1 TiB", 1u64 << 40),
    ] {
        let (kr, got, h) = sub_entry(ro_from_rw, ask, 1);
        println!("M3 RO sub-entry asking size {label}: kr={kr:#x} returned size={got}");
        if kr == 0 {
            unsafe { mach_port_deallocate(me(), h) };
        }
    }
    for (label, size) in [("SZ+16K", SZ + 16384), ("2*SZ", 2 * SZ)] {
        println!(
            "M3 map R/R of {label} from a {SZ}-byte entry: {:?}",
            map(ro_from_rw, size, 1, 1, 0)
                .map(|a| {
                    let r = region(a);
                    unsafe { mach_vm_deallocate(me(), a, size) };
                    r
                })
                .map_err(|e| format!("{e:#x}"))
        );
    }

    // M4: shared versus snapshot (copy=1) mappings of the RO entry.
    let shared = map(ro_from_rw, SZ, 1, 1, 0).unwrap();
    let snap = map(ro_from_rw, SZ, 1, 1, 1);
    unsafe {
        *(src as *mut u8) = 0x22;
        *((src + 20000) as *mut u8) = 0x33
    };
    match snap {
        Ok(a) => println!(
            "M4 after the creator writes 0x22 at 0 and 0x33 at 20000: shared map reads {:#x}/{:#x}; copy=1 map reads {:#x}/{:#x} ({})",
            unsafe { *(shared as *const u8) }, unsafe { *((shared + 20000) as *const u8) },
            unsafe { *(a as *const u8) }, unsafe { *((a + 20000) as *const u8) }, region(a)
        ),
        Err(e) => println!("M4 copy=1 R/R map: {e:#x}"),
    }
    // A copy=1 map taken after the first read of a page.
    let snap2 = map(ro_from_rw, SZ, 1, 1, 1).unwrap();
    let before = unsafe { std::ptr::read_volatile((snap2 + 40000) as *const u8) };
    unsafe { *((src + 40000) as *mut u8) = 0x44 };
    println!(
        "M4 copy=1 map read 40000 before ({before:#x}) and after the creator writes 0x44: {:#x}",
        unsafe { std::ptr::read_volatile((snap2 + 40000) as *const u8) }
    );

    // M5: the creator lets go of everything (as at its death): its mapping and its RW entry.
    unsafe {
        mach_vm_deallocate(me(), src, SZ);
        mach_port_deallocate(me(), rw);
    }
    println!("M5 after the creator deallocates its mapping and RW entry: existing shared map reads {:#x}", unsafe { *(shared as *const u8) });
    match map(ro_from_rw, SZ, 1, 1, 0) {
        Ok(a) => println!(
            "M5 a new map of the RO entry: ok, reads {:#x}/{:#x}",
            unsafe { *(a as *const u8) },
            unsafe { *((a + 20000) as *const u8) }
        ),
        Err(e) => println!("M5 a new map of the RO entry: {e:#x}"),
    }

    // M6: costs and region counts.
    for size in [64u64 << 10, 4u64 << 20] {
        let n = 2000;
        let mut src = 0u64;
        unsafe { mach_vm_allocate(me(), &mut src, size, 1) };
        let t = Instant::now();
        for _ in 0..n {
            let mut s = size;
            let mut h = 0u32;
            unsafe {
                mach_make_memory_entry_64(me(), &mut s, src, 1, &mut h, 0);
                mach_port_deallocate(me(), h)
            };
        }
        let mk = t.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
        let mut s = size;
        let mut e = 0u32;
        unsafe { mach_make_memory_entry_64(me(), &mut s, src, 1, &mut e, 0) };
        let t = Instant::now();
        for _ in 0..n {
            let a = map(e, size, 1, 1, 0).unwrap();
            unsafe { mach_vm_deallocate(me(), a, size) };
        }
        let mp = t.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
        let m = n / 10;
        let t = Instant::now();
        for _ in 0..m {
            let a = map(e, size, 1, 1, 0).unwrap();
            let mut x = 0u64;
            for i in 0..(size / 16384) {
                x = x.wrapping_add(
                    unsafe { std::ptr::read_volatile((a + i * 16384) as *const u8) } as u64,
                );
            }
            std::hint::black_box(x);
            unsafe { mach_vm_deallocate(me(), a, size) };
        }
        let touched = t.elapsed().as_nanos() as f64 / m as f64 / 1000.0;
        let b = regions_count();
        let a = map(e, size, 1, 1, 0).unwrap();
        let after = regions_count();
        unsafe { mach_vm_deallocate(me(), a, size) };
        println!("M6 {} KiB: make RO entry+release {mk:.2} µs; map+unmap {mp:.2} µs; map+read every 16 KiB page+unmap {touched:.2} µs; regions +{}", size >> 10, after as i64 - b as i64);
    }
}
