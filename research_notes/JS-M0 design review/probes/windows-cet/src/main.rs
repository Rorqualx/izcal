//! JS-M0 Q-JS-15: black-box probes of Windows user-mode shadow stacks.
//!
//! `cetprobe launch <policy> <cmd...>` runs `cetprobe <cmd...>` in a child
//! process created with PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY (policy2
//! word: CET user shadow stacks on/off/strict, optionally
//! CetDynamicApisOutOfProcOnly) and prints the child's exit code. The other
//! subcommands are the probes; see ../../windows-cet-probes.md.

mod asm;
mod sys;

use asm::*;
use core::ffi::c_void;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::SeqCst};
use sys::*;
use sys::{ConvertThreadToFiber, CreateFiberEx, DeleteFiber, SwitchToFiber};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, EXCEPTION_POINTERS, FlushInstructionCache,
};
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::Threading::*;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn WriteFile(h: HANDLE, buf: *const u8, n: u32, written: *mut u32, ov: *mut c_void) -> i32;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let a: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();
    match a.as_slice() {
        ["launch", policy, rest @ ..] => {
            let code = launch(policy, rest);
            println!("CHILD {policy} {:?} exit=0x{code:08x}", rest);
        }
        ["policy"] => cmd_policy(),
        ["upgrade"] => cmd_upgrade(),
        ["fiber"] => cmd_fiber(),
        ["fiberbench"] => cmd_fiberbench(),
        ["fibersize"] => cmd_fibersize(),
        ["ssprotect"] => cmd_ssprotect(),
        ["own"] => cmd_own(),
        ["forge"] => cmd_forge(),
        ["wrss"] => cmd_wrss(),
        ["seg", mode, test] => cmd_seg(mode, test),
        ["jit", kind, reg] => cmd_jit(kind, reg),
        ["jitchild", kind, file, ev] => cmd_jitchild(kind, file, ev),
        ["broker", policy, what] => cmd_broker(policy, what),
        ["regbench"] => cmd_regbench(),
        _ => {
            eprintln!("usage: see windows-cet-probes.md");
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------- launching

fn policy2(policy: &str) -> Option<u64> {
    const ON: u64 = 1 << 28;
    const OFF: u64 = 2 << 28;
    const STRICT: u64 = 3 << 28;
    const OOP: u64 = 1 << 48;
    const NOOOP: u64 = 2 << 48;
    Some(match policy {
        "none" => return None,
        "on" => ON,
        "off" => OFF,
        "strict" => STRICT,
        "on-oop" => ON | OOP,
        "strict-oop" => STRICT | OOP,
        "on-nooop" => ON | NOOOP,
        "strict-nooop" => STRICT | NOOOP,
        "off-nooop" => OFF | NOOOP,
        _ => panic!("unknown policy {policy}"),
    })
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Starts this executable with `args` under `policy`; returns the process
/// handle and pid. Standard handles and inheritable handles are inherited.
fn spawn(policy: &str, args: &[&str]) -> (HANDLE, u32) {
    unsafe {
        for h in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            let h = GetStdHandle(h);
            SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
        }
        let exe = std::env::current_exe().unwrap();
        let mut cmd = format!("\"{}\"", exe.display());
        for a in args {
            cmd.push_str(&format!(" \"{a}\""));
        }
        let mut cmdw = wide(&cmd);
        let mut si: STARTUPINFOEXW = std::mem::zeroed();
        si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        si.StartupInfo.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
        si.StartupInfo.hStdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
        si.StartupInfo.hStdError = GetStdHandle(STD_ERROR_HANDLE);
        let pol = policy2(policy);
        let value: Box<[u64; 2]> = Box::new([0, pol.unwrap_or(0)]);
        let mut list_buf: Vec<u8> = Vec::new();
        let mut flags = 0u32;
        if pol.is_some() {
            let mut size = 0usize;
            InitializeProcThreadAttributeList(core::ptr::null_mut(), 1, 0, &mut size);
            list_buf.resize(size, 0);
            let list = list_buf.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
            assert!(InitializeProcThreadAttributeList(list, 1, 0, &mut size) != 0);
            let ok = UpdateProcThreadAttribute(
                list,
                0,
                PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY as usize,
                value.as_ptr() as *const c_void,
                16,
                core::ptr::null_mut(),
                core::ptr::null(),
            );
            assert!(ok != 0, "UpdateProcThreadAttribute {}", last_error());
            si.lpAttributeList = list;
            flags |= EXTENDED_STARTUPINFO_PRESENT;
        }
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        let ok = CreateProcessW(
            core::ptr::null(),
            cmdw.as_mut_ptr(),
            core::ptr::null(),
            core::ptr::null(),
            1,
            flags,
            core::ptr::null(),
            core::ptr::null(),
            &si.StartupInfo,
            &mut pi,
        );
        if ok == 0 {
            println!("CreateProcessW({policy}) failed: {}", last_error());
            std::process::exit(3);
        }
        CloseHandle(pi.hThread);
        (pi.hProcess, pi.dwProcessId)
    }
}

fn wait_exit(h: HANDLE) -> u32 {
    unsafe {
        WaitForSingleObject(h, INFINITE);
        let mut code = 0u32;
        GetExitCodeProcess(h, &mut code);
        CloseHandle(h);
        code
    }
}

fn launch(policy: &str, args: &[&str]) -> u32 {
    let (h, _) = spawn(policy, args);
    wait_exit(h)
}

// ---------------------------------------------------------------- baseline

fn report_ssp(label: &str, ssp: u64) {
    if ssp == 0 {
        println!("{label}: SSP=0 (shadow stack inactive)");
        return;
    }
    let r = query(ssp as usize).unwrap();
    println!("{label}: SSP=0x{ssp:x} region {}", fmt_region(&r));
}

fn cmd_policy() {
    let p = shadow_policy(unsafe { GetCurrentProcess() });
    match p {
        Ok(f) => println!("policy {}", decode_policy(f)),
        Err(e) => println!("policy query failed {e}"),
    }
    report_ssp("main thread", unsafe { cet_rdssp() });
    let (base, limit) = tib_bounds();
    let (mut lo, mut hi) = (0usize, 0usize);
    unsafe { GetCurrentThreadStackLimits(&mut lo, &mut hi) };
    println!(
        "tib StackBase=0x{base:x} StackLimit=0x{limit:x}; GetCurrentThreadStackLimits=[0x{lo:x},0x{hi:x})"
    );
    std::thread::spawn(|| report_ssp("second thread", unsafe { cet_rdssp() }))
        .join()
        .unwrap();
}

fn cmd_upgrade() {
    let me = unsafe { GetCurrentProcess() };
    let f = shadow_policy(me).unwrap();
    println!("before {}", decode_policy(f));
    let strict = f | 1 | (1 << 4);
    println!("set strict: {:?}", set_shadow_policy(strict));
    println!("after {}", decode_policy(shadow_policy(me).unwrap()));
    let f = shadow_policy(me).unwrap();
    println!(
        "set CetDynamicApisOutOfProcOnly: {:?}",
        set_shadow_policy(f | (1 << 8))
    );
    println!("after {}", decode_policy(shadow_policy(me).unwrap()));
    let f = shadow_policy(me).unwrap();
    println!(
        "try clear strict (downgrade): {:?}",
        set_shadow_policy(f & !(1 << 4))
    );
    println!(
        "try clear CetDynamicApisOutOfProcOnly: {:?}",
        set_shadow_policy(f & !(1 << 8))
    );
    println!("final {}", decode_policy(shadow_policy(me).unwrap()));
}

// ---------------------------------------------------------------- fibers (Q2)

static MAIN_FIBER: AtomicUsize = AtomicUsize::new(0);
static FIB_SSP: AtomicU64 = AtomicU64::new(0);
static FIB_RUNS: AtomicU64 = AtomicU64::new(0);

unsafe extern "system" fn fib_start(_: *mut c_void) {
    loop {
        FIB_SSP.store(unsafe { cet_rdssp() }, SeqCst);
        FIB_RUNS.fetch_add(1, SeqCst);
        unsafe { SwitchToFiber(MAIN_FIBER.load(SeqCst) as *mut c_void) };
    }
}

unsafe extern "system" fn fib_pingpong(_: *mut c_void) {
    loop {
        unsafe { SwitchToFiber(MAIN_FIBER.load(SeqCst) as *mut c_void) };
    }
}

fn new_regions(before: &[Region], after: &[Region]) -> Vec<Region> {
    after
        .iter()
        .filter(|r| !before.iter().any(|b| b.base == r.base && b.size == r.size))
        .copied()
        .collect()
}

/// Creates a fiber and reports the regions it added and any restore tokens in
/// them. Returns (fiber, tokens).
fn create_fiber_scan(label: &str) -> (*mut c_void, Vec<(usize, u64)>) {
    let before = all_regions();
    let f = unsafe { CreateFiberEx(0x10000, 0x100000, 0, fib_start, core::ptr::null_mut()) };
    assert!(!f.is_null());
    let after = all_regions();
    let mut toks = Vec::new();
    for r in new_regions(&before, &after) {
        let t = scan_tokens(&r);
        println!("{label}: new region {} tokens={:x?}", fmt_region(&r), t);
        toks.extend(t);
    }
    (f, toks)
}

fn cmd_fiber() {
    let main = unsafe { ConvertThreadToFiber(core::ptr::null_mut()) };
    MAIN_FIBER.store(main as usize, SeqCst);
    let mssp = unsafe { cet_rdssp() };
    report_ssp("main fiber", mssp);
    let (f1, t1) = create_fiber_scan("CreateFiberEx#1 (not yet run)");
    unsafe { SwitchToFiber(f1) };
    let s1 = FIB_SSP.load(SeqCst);
    report_ssp("fiber#1 running", s1);
    if s1 != 0 {
        let r = query(s1 as usize).unwrap();
        println!(
            "fiber#1 suspended: tokens in its region {:x?} (fresh-fiber tokens were {:x?})",
            scan_tokens(&r),
            t1
        );
        println!(
            "main SSP in fiber#1 region? {}",
            mssp as usize >= r.alloc_base && (mssp as usize) < r.base + r.size
        );
    }
    let (f2, _) = create_fiber_scan("CreateFiberEx#2 (not yet run)");
    unsafe { SwitchToFiber(f2) };
    let s2 = FIB_SSP.load(SeqCst);
    report_ssp("fiber#2 running", s2);
    if s1 != 0 && s2 != 0 {
        let a = query(s1 as usize).unwrap().alloc_base;
        let b = query(s2 as usize).unwrap().alloc_base;
        let m = query(mssp as usize).unwrap().alloc_base;
        println!(
            "distinct shadow stacks: main/f1 {} f1/f2 {} main/f2 {}",
            m != a,
            a != b,
            m != b
        );
    }
    // Resume fiber#1 again: its shadow stack and token survived fiber#2.
    unsafe { SwitchToFiber(f1) };
    println!(
        "fiber#1 resumed again: SSP=0x{:x} runs={}",
        FIB_SSP.load(SeqCst),
        FIB_RUNS.load(SeqCst)
    );
    let before = all_regions();
    unsafe { DeleteFiber(f2) };
    let after = all_regions();
    for r in new_regions(&after, &before) {
        println!("DeleteFiber#2 freed/changed {}", fmt_region(&r));
    }
}

fn cmd_fibersize() {
    for (commit, reserve) in [
        (0x1000usize, 0x10000usize),
        (0x10000, 0x100000),
        (0x10000, 0x800000),
    ] {
        let before = all_regions();
        let f = unsafe { CreateFiberEx(commit, reserve, 0, fib_start, core::ptr::null_mut()) };
        assert!(!f.is_null());
        let after = all_regions();
        let mut by_alloc: Vec<(usize, u32, usize, usize)> = Vec::new();
        for r in new_regions(&before, &after) {
            let c = if r.state == MEM_COMMIT { r.size } else { 0 };
            match by_alloc.iter_mut().find(|x| x.0 == r.alloc_base) {
                Some(x) => {
                    x.2 += r.size;
                    x.3 += c;
                }
                None => by_alloc.push((r.alloc_base, r.alloc_protect, r.size, c)),
            }
        }
        for (b, ap, total, com) in by_alloc {
            let what = if ap == PAGE_READONLY {
                "shadow stack (read-only allocation)"
            } else {
                "other"
            };
            println!(
                "CreateFiberEx(commit=0x{commit:x}, reserve=0x{reserve:x}): allocation 0x{b:x} reserved=0x{total:x} committed=0x{com:x} alloc_protect=0x{ap:x} {what}"
            );
        }
        unsafe { DeleteFiber(f) };
    }
}

fn cmd_ssprotect() {
    let ssp = unsafe { cet_rdssp() } as usize;
    let page = ssp & !0xfff;
    let mut old = 0u32;
    let r = unsafe { VirtualProtect(page as *const c_void, 0x1000, PAGE_READWRITE, &mut old) };
    println!(
        "VirtualProtect(own shadow-stack page, PAGE_READWRITE): ok={} err={} old=0x{old:x}",
        r != 0,
        last_error()
    );
    let r = unsafe { VirtualProtect(page as *const c_void, 0x1000, PAGE_READONLY, &mut old) };
    println!(
        "VirtualProtect(own shadow-stack page, PAGE_READONLY): ok={} err={} old=0x{old:x}",
        r != 0,
        last_error()
    );
    let m = alloc(0x1000, PAGE_READONLY);
    let mut z = 0u32;
    let r = unsafe { VirtualProtect(m as *const c_void, 0x1000, PAGE_READWRITE, &mut z) };
    println!(
        "control: VirtualProtect(ordinary RO page, RW): ok={}",
        r != 0
    );
}

fn cmd_fiberbench() {
    let main = unsafe { ConvertThreadToFiber(core::ptr::null_mut()) };
    MAIN_FIBER.store(main as usize, SeqCst);
    println!(
        "SSP=0x{:x} policy {}",
        unsafe { cet_rdssp() },
        decode_policy(shadow_policy(unsafe { GetCurrentProcess() }).unwrap())
    );
    let f = unsafe { CreateFiberEx(0x10000, 0x100000, 0, fib_pingpong, core::ptr::null_mut()) };
    bench(
        "SwitchToFiber round trip (2 switches)",
        301,
        1000,
        || unsafe { SwitchToFiber(f) },
    );
    bench(
        "CreateFiberEx(64K commit,1M reserve)+DeleteFiber",
        201,
        20,
        || unsafe {
            let g = CreateFiberEx(0x10000, 0x100000, 0, fib_pingpong, core::ptr::null_mut());
            assert!(!g.is_null());
            DeleteFiber(g);
        },
    );
    bench(
        "CreateFiberEx+first SwitchToFiber round trip+DeleteFiber",
        201,
        20,
        || unsafe {
            let g = CreateFiberEx(0x10000, 0x100000, 0, fib_pingpong, core::ptr::null_mut());
            SwitchToFiber(g);
            DeleteFiber(g);
        },
    );
}

// ---------------------------------------------------------------- own switching (Q1)

struct Coro {
    main: *mut Desc,
    me: *mut Desc,
    seg_lo: u64,
    seg_hi: u64,
    main_bounds: (u64, u64),
}

fn quiet_panics() {
    std::panic::set_hook(Box::new(|_| {}));
}

unsafe extern "C" fn coro_entry(arg: *mut c_void) {
    let c = unsafe { &mut *(arg as *mut Coro) };
    set_tib_bounds(c.seg_hi, c.seg_lo);
    report_ssp("coroutine (own switch) running", unsafe { cet_rdssp() });
    let mut rsp: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp) };
    println!(
        "coroutine rsp=0x{rsp:x} (own segment [0x{:x},0x{:x}))",
        c.seg_lo, c.seg_hi
    );
    let v: Vec<u64> = (0..100_000).collect();
    println!("coroutine: Rust work ok, sum={}", v.iter().sum::<u64>());
    let r = std::panic::catch_unwind(|| panic!("probe panic"));
    println!(
        "coroutine: catch_unwind of a panic -> caught={}",
        r.is_err()
    );
    for _ in 0..20 {
        unsafe { Sleep(1) };
    }
    println!("coroutine: 20 x Sleep(1) (context switches) ok");
    let t = std::thread::spawn(|| 7).join().unwrap();
    println!("coroutine: thread spawn/join ok ({t})");
    // Suspend mid-stack (JSPI suspend): back to main.
    set_tib_bounds(c.main_bounds.0, c.main_bounds.1);
    unsafe { ss_switch(c.main, c.me) };
    set_tib_bounds(c.seg_hi, c.seg_lo);
    println!("coroutine: resumed, SSP=0x{:x}", unsafe { cet_rdssp() });
    set_tib_bounds(c.main_bounds.0, c.main_bounds.1);
}

unsafe extern "C" fn coro_pingpong(arg: *mut c_void) {
    let c = unsafe { &mut *(arg as *mut Coro) };
    loop {
        unsafe { ss_switch(c.main, c.me) };
    }
}

fn find_token(label: &str) -> (*mut c_void, u64) {
    let (f, toks) = create_fiber_scan(label);
    let run_once = std::env::var_os("CETPROBE_RUNONCE").is_some();
    if toks.len() == 1 && !run_once {
        return (f, toks[0].0 as u64);
    }
    println!(
        "{label}: {} tokens in fresh fiber; running it once",
        toks.len()
    );
    unsafe { SwitchToFiber(f) };
    let s = FIB_SSP.load(SeqCst);
    if s == 0 {
        return (f, 0);
    }
    let r = query(s as usize).unwrap();
    let t = scan_tokens(&r);
    println!("{label}: after one run tokens {:x?}", t);
    (f, t.first().map(|x| x.0 as u64).unwrap_or(0))
}

fn cmd_own() {
    quiet_panics();
    let main = unsafe { ConvertThreadToFiber(core::ptr::null_mut()) };
    MAIN_FIBER.store(main as usize, SeqCst);
    let mssp = unsafe { cet_rdssp() };
    report_ssp("main", mssp);
    let (f, tok) = find_token("fiber A");
    if mssp != 0 && tok == 0 {
        println!("RESULT own-switch: no restore token found; cannot test");
        return;
    }
    if tok != 0 {
        println!(
            "fiber A token at 0x{tok:x} value 0x{:x}",
            peek(tok as usize).unwrap()
        );
    }
    const SEG: usize = 1 << 20;
    let seg = alloc(SEG, PAGE_READWRITE) as u64;
    let main_desc: *mut Desc = Box::into_raw(Box::new(Desc { sp: 0, tok: 0 }));
    let coro_desc: *mut Desc = Box::into_raw(Box::new(Desc { sp: 0, tok: 0 }));
    let main_bounds = tib_bounds();
    let mut c = Coro {
        main: main_desc,
        me: coro_desc,
        seg_lo: seg,
        seg_hi: seg + SEG as u64,
        main_bounds,
    };
    let mut s = Start {
        tok,
        sp: seg + SEG as u64,
        cur: main_desc,
        entry: coro_entry,
        arg: &mut c as *mut Coro as *mut c_void,
    };
    unsafe { ss_start(&mut s) };
    let (main_desc, coro_desc) = unsafe { (&mut *main_desc, &mut *coro_desc) };
    println!(
        "main: back after coroutine suspended; SSP=0x{:x} (expect 0x{mssp:x})",
        unsafe { cet_rdssp() }
    );
    if coro_desc.tok != 0 {
        println!(
            "main: coroutine's restore token at 0x{:x} value 0x{:x}",
            coro_desc.tok,
            peek(coro_desc.tok as usize).unwrap()
        );
        println!(
            "main: main's token slot after return 0x{:x} = 0x{:x} (expect a consumed previous-ssp token)",
            main_desc.tok,
            peek(main_desc.tok as usize).unwrap_or(0)
        );
    }
    unsafe { ss_switch(coro_desc, main_desc) };
    println!(
        "main: coroutine finished; SSP=0x{:x}; fiber A token now at 0x{:x} value 0x{:x} (same address: {})",
        unsafe { cet_rdssp() },
        s.tok,
        peek(s.tok as usize).unwrap_or(0),
        s.tok == tok
    );
    let runs = FIB_RUNS.load(SeqCst);
    unsafe { SwitchToFiber(f) };
    println!(
        "main: SwitchToFiber(A) after foreign use: fiber ran={} its SSP=0x{:x}",
        FIB_RUNS.load(SeqCst) > runs,
        FIB_SSP.load(SeqCst)
    );
    println!("RESULT own-switch: ok");

    // Ping-pong cost of the own switch against a second fiber's shadow stack.
    let (_f2, tok2) = find_token("fiber B");
    let seg2 = alloc(SEG, PAGE_READWRITE) as u64;
    let m2p: *mut Desc = Box::into_raw(Box::new(Desc { sp: 0, tok: 0 }));
    let c2p: *mut Desc = Box::into_raw(Box::new(Desc { sp: 0, tok: 0 }));
    let mut c2 = Coro {
        main: m2p,
        me: c2p,
        seg_lo: seg2,
        seg_hi: seg2 + SEG as u64,
        main_bounds,
    };
    let mut s2 = Start {
        tok: tok2,
        sp: seg2 + SEG as u64,
        cur: m2p,
        entry: coro_pingpong,
        arg: &mut c2 as *mut Coro as *mut c_void,
    };
    unsafe { ss_start(&mut s2) };
    bench(
        "own ss_switch round trip (2 switches)",
        301,
        1000,
        || unsafe { ss_switch(c2p, m2p) },
    );
    bench(
        "own ss_switch round trip + TIB bounds x2",
        301,
        1000,
        || unsafe {
            set_tib_bounds(seg2 + SEG as u64, seg2);
            ss_switch(c2p, m2p);
            set_tib_bounds(main_bounds.0, main_bounds.1);
        },
    );
}

fn cmd_forge() {
    let m = alloc(0x1000, PAGE_READWRITE);
    let tok = m + 0x800;
    unsafe { *(tok as *mut u64) = (tok as u64 + 8) | 1 };
    println!("forge: RSTORSSP on a forged token in ordinary RW memory at 0x{tok:x}");
    unsafe { cet_rstorssp(tok as u64) };
    println!(
        "RESULT forge: RSTORSSP accepted a forged token; SSP=0x{:x}",
        unsafe { cet_rdssp() }
    );
}

fn cmd_wrss() {
    let ssp = unsafe { cet_rdssp() };
    println!(
        "wrss: WRSSQ to the current shadow stack at 0x{:x}",
        ssp - 64
    );
    unsafe { cet_wrss(ssp - 64, 0x4141) };
    println!("RESULT wrss: WRSS executed (WRSS enabled for user mode)");
}

// ---------------------------------------------------------------- segments (Q4)

struct SegCtx {
    update: bool,
    test: String,
    lo: u64,
    hi: u64,
}

#[inline(never)]
fn bigframe() -> u64 {
    let mut a = [0u8; 256 * 1024];
    black_box(&mut a);
    a.iter().map(|x| *x as u64).sum::<u64>() + a.len() as u64
}

#[inline(never)]
fn recurse(n: u64) -> u64 {
    let mut a = [0u8; 4096];
    black_box(&mut a);
    a[0] = n as u8;
    if n == u64::MAX {
        0
    } else {
        recurse(n + 1) + a[0] as u64
    }
}

static VEH_HIT: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn veh(ep: *mut EXCEPTION_POINTERS) -> i32 {
    if VEH_HIT.swap(true, SeqCst) {
        return 0;
    }
    let rec = unsafe { &*(*ep).ExceptionRecord };
    let mut buf = *b"VEH: exception 0x00000000 caught on the segment\r\n";
    let code = rec.ExceptionCode as u32;
    for i in 0..8 {
        let d = ((code >> (28 - 4 * i)) & 0xf) as u8;
        buf[17 + i] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
    }
    let mut w = 0u32;
    unsafe {
        WriteFile(
            GetStdHandle(STD_OUTPUT_HANDLE),
            buf.as_ptr(),
            buf.len() as u32,
            &mut w,
            core::ptr::null_mut(),
        );
        ExitProcess(0x77);
    }
}

fn seg_tests(c: &SegCtx) {
    let mut rsp: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp) };
    let on = rsp >= c.lo && rsp < c.hi;
    let tests: Vec<&str> = if c.test == "all" {
        vec!["basic", "bigframe", "panic", "backtrace", "thread"]
    } else {
        vec![c.test.as_str()]
    };
    for t in tests {
        match t {
            "basic" => {
                let (mut lo, mut hi) = (0usize, 0usize);
                unsafe { GetCurrentThreadStackLimits(&mut lo, &mut hi) };
                let v: Vec<String> = (0..1000).map(|i| format!("{i}")).collect();
                println!(
                    "seg basic: rsp=0x{rsp:x} on segment={on} SSP=0x{:x} GetCurrentThreadStackLimits=[0x{lo:x},0x{hi:x}) strings={}",
                    unsafe { cet_rdssp() },
                    v.len()
                );
            }
            "bigframe" => println!("seg bigframe (256 KiB frame, stack probes): {}", bigframe()),
            "panic" => {
                let r = std::panic::catch_unwind(|| panic!("probe"));
                println!("seg panic: caught={}", r.is_err());
            }
            "backtrace" => {
                let b = std::backtrace::Backtrace::force_capture().to_string();
                let frames = b
                    .lines()
                    .filter(|l| {
                        l.trim_start()
                            .chars()
                            .next()
                            .is_some_and(|ch| ch.is_ascii_digit())
                    })
                    .count();
                println!(
                    "seg backtrace: {frames} frames; reaches main: {}",
                    b.contains("cetprobe::main")
                );
            }
            "thread" => println!("seg thread: {}", std::thread::spawn(|| 5).join().unwrap()),
            "overflow" => {
                unsafe { AddVectoredExceptionHandler(1, Some(veh)) };
                println!("seg overflow (VEH installed): {}", recurse(0));
            }
            "overflow-noveh" => println!("seg overflow (no VEH): {}", recurse(0)),
            _ => panic!("unknown test"),
        }
    }
}

unsafe extern "C" fn seg_entry(arg: *mut c_void) {
    let c = unsafe { &*(arg as *const SegCtx) };
    let saved = tib_bounds();
    if c.update {
        set_tib_bounds(c.hi, c.lo);
    }
    seg_tests(c);
    set_tib_bounds(saved.0, saved.1);
}

fn cmd_seg(mode: &str, test: &str) {
    quiet_panics();
    // 2 MiB reservation: the top 1 MiB committed, the bottom 1 MiB reserved
    // (no access) as the guard.
    const RES: usize = 2 << 20;
    const COM: usize = 1 << 20;
    let base = unsafe { VirtualAlloc(core::ptr::null(), RES, MEM_RESERVE, PAGE_READWRITE) } as u64;
    assert!(base != 0);
    let lo = base + (RES - COM) as u64;
    let p = unsafe { VirtualAlloc(lo as *const c_void, COM, MEM_COMMIT, PAGE_READWRITE) };
    assert!(!p.is_null());
    let test = if let Some(t) = test.strip_suffix("-pg") {
        // 64 KiB of PAGE_GUARD pages below the segment, as Windows thread
        // stacks have: the first touch raises STATUS_GUARD_PAGE_VIOLATION
        // with the page already accessible, so the exception frame fits.
        let g = unsafe {
            VirtualAlloc(
                (lo - 0x10000) as *const c_void,
                0x10000,
                MEM_COMMIT,
                PAGE_READWRITE | PAGE_GUARD,
            )
        };
        assert!(!g.is_null());
        println!("guard pages [0x{:x},0x{lo:x}) PAGE_GUARD", lo - 0x10000);
        t
    } else {
        test
    };
    let hi = base + RES as u64;
    let c = SegCtx {
        update: mode == "update",
        test: test.to_string(),
        lo,
        hi,
    };
    println!(
        "segment [0x{lo:x},0x{hi:x}) guard [0x{base:x},0x{lo:x}) mode={mode} main SSP=0x{:x}",
        unsafe { cet_rdssp() }
    );
    unsafe { seg_call(seg_entry, &c as *const SegCtx as *mut c_void, hi) };
    println!("RESULT seg {mode} {test}: returned normally");
}

// ---------------------------------------------------------------- JIT (Q3, Q5)

/// call L; ret; L: mov eax, 42; ret
const GOOD: [u8; 12] = [0xe8, 1, 0, 0, 0, 0xc3, 0xb8, 42, 0, 0, 0, 0xc3];
/// call inner; mov eax,1; ret; inner: lea rax,[after]; mov [rsp],rax; ret; after: mov eax,7; ret
const BAD: [u8; 29] = [
    0xe8, 6, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0xc3, 0x48, 0x8d, 0x05, 5, 0, 0, 0, 0x48, 0x89, 0x04, 0x24,
    0xc3, 0xb8, 7, 0, 0, 0, 0xc3,
];

fn make_code(kind: &str, size: usize) -> usize {
    let m = alloc(size, PAGE_READWRITE);
    let code: &[u8] = if kind == "good" { &GOOD } else { &BAD };
    unsafe { core::ptr::copy_nonoverlapping(code.as_ptr(), m as *mut u8, code.len()) };
    let mut old = 0u32;
    unsafe {
        assert!(VirtualProtect(m as *const c_void, size, PAGE_EXECUTE_READ, &mut old) != 0);
        FlushInstructionCache(GetCurrentProcess(), m as *const c_void, size);
    }
    m
}

fn run_code(m: usize) -> u32 {
    let f: extern "C" fn() -> u32 = unsafe { core::mem::transmute(m) };
    f()
}

fn try_register_inproc(m: usize, size: usize) {
    let mut r = [EnforcedRange {
        base: m,
        size,
        flags: DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD,
    }];
    let res = set_cet_ranges(unsafe { GetCurrentProcess() }, &mut r);
    println!(
        "in-proc SetProcessDynamicEnforcedCetCompatibleRanges: {:?} flags-out=0x{:x}",
        res, r[0].flags
    );
    let mut t = [EhTarget {
        addr: m,
        flags: DYNAMIC_EH_CONTINUATION_TARGET_ADD,
    }];
    let res = set_eh_targets(unsafe { GetCurrentProcess() }, &mut t);
    println!(
        "in-proc SetProcessDynamicEHContinuationTargets: {:?} flags-out=0x{:x}",
        res, t[0].flags
    );
}

fn cmd_jit(kind: &str, reg: &str) {
    println!(
        "policy {}",
        decode_policy(shadow_policy(unsafe { GetCurrentProcess() }).unwrap())
    );
    let m = make_code(kind, 0x10000);
    if reg == "inproc" {
        try_register_inproc(m, 0x10000);
    }
    println!("calling {kind} JIT code at 0x{m:x}");
    let r = run_code(m);
    println!("RESULT jit {kind} {reg}: returned {r}");
}

fn cmd_jitchild(kind: &str, file: &str, ev: &str) {
    println!(
        "child policy {}",
        decode_policy(shadow_policy(unsafe { GetCurrentProcess() }).unwrap())
    );
    const SIZE: usize = 1 << 20;
    if kind == "late" {
        // Reserve only; the broker registers the reservation before any
        // page is committed or any code exists.
        let m =
            unsafe { VirtualAlloc(core::ptr::null(), SIZE, MEM_RESERVE, PAGE_NOACCESS) } as usize;
        std::fs::write(file, format!("{m} {SIZE}")).unwrap();
        let ev: usize = ev.parse().unwrap();
        unsafe { WaitForSingleObject(ev as HANDLE, INFINITE) };
        let p = unsafe { VirtualAlloc(m as *const c_void, 0x1000, MEM_COMMIT, PAGE_READWRITE) }
            as usize;
        assert_eq!(p, m);
        unsafe { core::ptr::copy_nonoverlapping(BAD.as_ptr(), m as *mut u8, BAD.len()) };
        let mut old = 0u32;
        unsafe { VirtualProtect(m as *const c_void, 0x1000, PAGE_EXECUTE_READ, &mut old) };
        println!("child: calling bad JIT code committed after registration at 0x{m:x}");
        let r = run_code(m);
        println!("RESULT jitchild late: returned {r}");
        return;
    }
    let m = make_code(kind, SIZE);
    try_register_inproc(m, SIZE);
    std::fs::write(file, format!("{m} {SIZE}")).unwrap();
    let ev: usize = ev.parse().unwrap();
    unsafe { WaitForSingleObject(ev as HANDLE, INFINITE) };
    println!("child: calling {kind} JIT code at 0x{m:x}");
    let r = run_code(m);
    println!("RESULT jitchild {kind}: returned {r}");
}

fn cmd_broker(policy: &str, what: &str) {
    let sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: core::ptr::null_mut(),
        bInheritHandle: 1,
    };
    let ev = unsafe { CreateEventW(&sa, 1, 0, core::ptr::null()) };
    let file = std::env::temp_dir().join(format!("cetprobe-{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&file);
    let kind = match what {
        "bad" => "bad",
        "late" => "late",
        _ => "good",
    };
    let evs = (ev as usize).to_string();
    let fs = file.to_string_lossy().to_string();
    let (h, pid) = spawn(policy, &["jitchild", kind, &fs, &evs]);
    let (m, size) = loop {
        if let Ok(s) = std::fs::read_to_string(&file) {
            if let Some((a, b)) = s.split_once(' ') {
                break (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap());
            }
        }
        unsafe { Sleep(1) };
    };
    println!("broker: child pid {pid} code region 0x{m:x}+0x{size:x}");
    unsafe {
        let weak = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        let mut r = [EnforcedRange {
            base: m,
            size: 0x1000,
            flags: DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD,
        }];
        println!(
            "broker: with PROCESS_QUERY_LIMITED_INFORMATION only: {:?}",
            set_cet_ranges(weak, &mut r)
        );
        CloseHandle(weak);
    }
    let hp = unsafe { OpenProcess(PROCESS_SET_INFORMATION, 0, pid) };
    assert!(!hp.is_null(), "OpenProcess {}", last_error());
    let mut r = [EnforcedRange {
        base: m,
        size,
        flags: DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD,
    }];
    println!(
        "broker: register whole region with PROCESS_SET_INFORMATION: {:?} flags-out=0x{:x}",
        set_cet_ranges(hp, &mut r),
        r[0].flags
    );
    let mut t = [EhTarget {
        addr: m + 5,
        flags: DYNAMIC_EH_CONTINUATION_TARGET_ADD,
    }];
    println!(
        "broker: EH continuation target: {:?} flags-out=0x{:x}",
        set_eh_targets(hp, &mut t),
        t[0].flags
    );
    if what == "bench" {
        // Validation: a free address range and a sub-page range.
        let mut bad = [EnforcedRange {
            base: 0x10000,
            size: 0x1000,
            flags: DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD,
        }];
        println!(
            "broker: register unallocated range: {:?}",
            set_cet_ranges(hp, &mut bad)
        );
        let mut odd = [EnforcedRange {
            base: m + 0x10,
            size: 0x10,
            flags: DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD,
        }];
        println!(
            "broker: register unaligned 16-byte range: {:?}",
            set_cet_ranges(hp, &mut odd)
        );
        let mut rm = [EnforcedRange {
            base: m,
            size,
            flags: 0,
        }];
        println!(
            "broker: remove whole region: {:?}",
            set_cet_ranges(hp, &mut rm)
        );
        let mut page = 0usize;
        bench_ranges(
            "broker add+remove 1 page range (2 calls)",
            hp,
            m,
            1,
            &mut page,
        );
        bench_ranges(
            "broker add+remove 64 page ranges (2 calls)",
            hp,
            m,
            64,
            &mut page,
        );
        let mut whole = [EnforcedRange {
            base: m,
            size,
            flags: DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD,
        }];
        println!(
            "broker: re-register whole region: {:?}",
            set_cet_ranges(hp, &mut whole)
        );
    }
    unsafe { SetEvent(ev) };
    let code = wait_exit(h);
    let _ = std::fs::remove_file(&file);
    println!("CHILD broker {policy} {what} exit=0x{code:08x}");
}

fn bench_ranges(name: &str, hp: HANDLE, m: usize, n: usize, page: &mut usize) {
    let mut v: Vec<EnforcedRange> = vec![
        EnforcedRange {
            base: 0,
            size: 0x1000,
            flags: 0
        };
        n
    ];
    bench(name, 201, 10, || {
        for (i, r) in v.iter_mut().enumerate() {
            r.base = m + ((*page + i) % 256) * 0x1000;
            r.flags = DYNAMIC_ENFORCED_ADDRESS_RANGE_ADD;
        }
        set_cet_ranges(hp, &mut v).unwrap();
        for r in v.iter_mut() {
            r.flags = 0;
        }
        set_cet_ranges(hp, &mut v).unwrap();
        *page += n;
    });
}

fn cmd_regbench() {
    println!(
        "policy {}",
        decode_policy(shadow_policy(unsafe { GetCurrentProcess() }).unwrap())
    );
    let m = make_code("good", 1 << 20);
    let me = unsafe { GetCurrentProcess() };
    let mut page = 0usize;
    bench_ranges(
        "in-proc add+remove 1 page range (2 calls)",
        me,
        m,
        1,
        &mut page,
    );
    bench_ranges(
        "in-proc add+remove 64 page ranges (2 calls)",
        me,
        m,
        64,
        &mut page,
    );
}
