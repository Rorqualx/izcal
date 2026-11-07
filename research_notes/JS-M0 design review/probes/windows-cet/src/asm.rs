//! The probe's only hand-written machine code: reading SSP, the segment call
//! (switches `rsp` only, keeps the shadow stack: R2-17's growth stub), and a
//! coroutine switch that moves both `rsp` and SSP with the architectural
//! restore-token protocol (RSTORSSP + SAVEPREVSSP, Intel SDM Vol. 1 ch. 17):
//! the shape R2-26 proposes for Linux, tried here on Windows.
//!
//! Every stub is written from the SDM's instruction descriptions only.

use core::ffi::c_void;

/// A suspended stack: its saved `rsp` and the address of its restore token.
#[repr(C)]
pub struct Desc {
    pub sp: u64,
    pub tok: u64,
}

/// Arguments of `ss_start`, which enters a new stack with a genuine call.
#[repr(C)]
pub struct Start {
    /// Address of the restore token on the target shadow stack (0: no switch).
    pub tok: u64,
    /// Top of the target data stack (16-byte aligned by the stub).
    pub sp: u64,
    /// Where the caller's stack is saved; the coroutine returns to it.
    pub cur: *mut Desc,
    pub entry: unsafe extern "C" fn(*mut c_void),
    pub arg: *mut c_void,
}

unsafe extern "C" {
    /// Current SSP, or 0 when shadow stacks are off for the thread.
    pub fn cet_rdssp() -> u64;
    /// Calls `f(arg)` with `rsp = new_top` (minus home space); the shadow stack
    /// is untouched. Has unwind info (rbp frame), so unwinders can cross it.
    pub fn seg_call(f: unsafe extern "C" fn(*mut c_void), arg: *mut c_void, new_top: u64);
    /// Saves the caller's registers on its stack, switches SSP to `target.tok`
    /// (if shadow stacks are on), sets `rsp = target.sp`, restores, returns.
    pub fn ss_switch(target: *mut Desc, cur: *mut Desc);
    /// Enters a new stack: switch SSP to `s.tok`, `rsp` to `s.sp`, call
    /// `s.entry(s.arg)`; when it returns, switch back to `*s.cur` and return.
    /// On the way out the coroutine's new token address is stored in `s.tok`.
    pub fn ss_start(s: *mut Start);
    /// Runs RSTORSSP on `addr` (a forged token test).
    pub fn cet_rstorssp(addr: u64);
    /// Runs WRSSQ `val` to `addr` (needs WRSS enabled by the OS).
    pub fn cet_wrss(addr: u64, val: u64);
}

core::arch::global_asm!(
    r#"
    .text
    .p2align 4
    .globl cet_rdssp
cet_rdssp:
    xor eax, eax
    rdsspq rax
    ret

    .p2align 4
    .globl cet_rstorssp
cet_rstorssp:
    rstorssp [rcx]
    ret

    .p2align 4
    .globl cet_wrss
cet_wrss:
    wrssq qword ptr [rcx], rdx
    ret

    .p2align 4
    .globl seg_call
    .seh_proc seg_call
seg_call:
    push rbp
    .seh_pushreg rbp
    mov rbp, rsp
    .seh_setframe rbp, 0
    .seh_endprologue
    mov rsp, r8
    and rsp, -16
    sub rsp, 32
    mov rax, rcx
    mov rcx, rdx
    call rax
    mov rsp, rbp
    pop rbp
    ret
    .seh_endproc

    .p2align 4
    .globl ss_switch
ss_switch:
    push rbx
    push rbp
    push rdi
    push rsi
    push r12
    push r13
    push r14
    push r15
    sub rsp, 168
    movdqu xmmword ptr [rsp + 0], xmm6
    movdqu xmmword ptr [rsp + 16], xmm7
    movdqu xmmword ptr [rsp + 32], xmm8
    movdqu xmmword ptr [rsp + 48], xmm9
    movdqu xmmword ptr [rsp + 64], xmm10
    movdqu xmmword ptr [rsp + 80], xmm11
    movdqu xmmword ptr [rsp + 96], xmm12
    movdqu xmmword ptr [rsp + 112], xmm13
    movdqu xmmword ptr [rsp + 128], xmm14
    movdqu xmmword ptr [rsp + 144], xmm15
    mov qword ptr [rdx], rsp
    xor eax, eax
    rdsspq rax
    test rax, rax
    jz 2f
    mov r8, qword ptr [rcx + 8]
    rstorssp [r8]
    saveprevssp
    sub rax, 8
    mov qword ptr [rdx + 8], rax
2:
    mov rsp, qword ptr [rcx]
    jmp ss_restore_ret

    .p2align 4
    .globl ss_start
ss_start:
    push rbx
    push rbp
    push rdi
    push rsi
    push r12
    push r13
    push r14
    push r15
    sub rsp, 168
    movdqu xmmword ptr [rsp + 0], xmm6
    movdqu xmmword ptr [rsp + 16], xmm7
    movdqu xmmword ptr [rsp + 32], xmm8
    movdqu xmmword ptr [rsp + 48], xmm9
    movdqu xmmword ptr [rsp + 64], xmm10
    movdqu xmmword ptr [rsp + 80], xmm11
    movdqu xmmword ptr [rsp + 96], xmm12
    movdqu xmmword ptr [rsp + 112], xmm13
    movdqu xmmword ptr [rsp + 128], xmm14
    movdqu xmmword ptr [rsp + 144], xmm15
    mov rbx, rcx
    mov rdx, qword ptr [rbx + 16]
    mov qword ptr [rdx], rsp
    xor eax, eax
    rdsspq rax
    test rax, rax
    jz 3f
    mov r8, qword ptr [rbx]
    rstorssp [r8]
    saveprevssp
    sub rax, 8
    mov qword ptr [rdx + 8], rax
3:
    mov rsp, qword ptr [rbx + 8]
    and rsp, -16
    sub rsp, 32
    mov rcx, qword ptr [rbx + 32]
    call qword ptr [rbx + 24]
    mov rcx, qword ptr [rbx + 16]
    xor eax, eax
    rdsspq rax
    test rax, rax
    jz 4f
    mov r8, qword ptr [rcx + 8]
    rstorssp [r8]
    saveprevssp
    sub rax, 8
    mov qword ptr [rbx], rax
4:
    mov rsp, qword ptr [rcx]
    jmp ss_restore_ret

    .p2align 4
ss_restore_ret:
    movdqu xmm6, xmmword ptr [rsp + 0]
    movdqu xmm7, xmmword ptr [rsp + 16]
    movdqu xmm8, xmmword ptr [rsp + 32]
    movdqu xmm9, xmmword ptr [rsp + 48]
    movdqu xmm10, xmmword ptr [rsp + 64]
    movdqu xmm11, xmmword ptr [rsp + 80]
    movdqu xmm12, xmmword ptr [rsp + 96]
    movdqu xmm13, xmmword ptr [rsp + 112]
    movdqu xmm14, xmmword ptr [rsp + 128]
    movdqu xmm15, xmmword ptr [rsp + 144]
    add rsp, 168
    pop r15
    pop r14
    pop r13
    pop r12
    pop rsi
    pop rdi
    pop rbp
    pop rbx
    ret
"#
);

/// The thread's NT_TIB (winnt.h's documented layout: ExceptionList, StackBase,
/// StackLimit), reached through its self pointer at gs:[0x30].
pub fn tib() -> *mut u64 {
    let p: *mut u64;
    unsafe { core::arch::asm!("mov {}, qword ptr gs:[0x30]", out(reg) p) };
    p
}

pub fn tib_bounds() -> (u64, u64) {
    let t = tib();
    unsafe { (*t.add(1), *t.add(2)) }
}

pub fn set_tib_bounds(base: u64, limit: u64) {
    let t = tib();
    unsafe {
        *t.add(1) = base;
        *t.add(2) = limit;
    }
}
