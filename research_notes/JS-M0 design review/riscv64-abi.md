# JS-M0 design review: register assignment, landing pads and the segment-switch stub (riscv64, with x86-64 and AArch64)

Compiled 31 October 2026 for the JS-M0 design review (part 2), against the [JS engine spec](../../reports/Izcal%20JavaScript%20engine.md) draft 0.4.4, Section 6.18 (JS-D22). Index and decisions: [part-2-index.md](part-2-index.md). Frames: [frames-deopt-stackmaps.md](frames-deopt-stackmaps.md). Code tables: [code-table-and-ledger.md](code-table-and-ledger.md).

Confidence: **High** = primary source read and the claim checked in its text this session; **Medium** = recalled from a primary source, or from a mirror of it, not re-read this session; **Low** = secondary only. † marks a value set here from first principles.

**M-1 statement.** Sources are the RISC-V ISA manual's CFI chapter and the RISC-V psABI calling-convention chapter (specification documents, read as prose), the Linux kernel's documentation, Microsoft's and Apple's ABI pages, and the Intel and Arm architecture manuals. No compiler, kernel, library or engine source was opened. The riscv64 research note's evidence rules apply ([RV](../Izcal%20JavaScript%20engine/riscv64.md)).

## 1. What this note fixes

JS-M0's riscv64 item: "the design review fixes riscv64's register assignment, landing-pad labels and segment-switch stub with the other ISAs" ([JS 9.2](../../reports/Izcal%20JavaScript%20engine.md)). Section 6.18 already fixes: cage base, trusted-region base and the agent's context in callee-saved registers; x7 reserved for landing-pad labels; one 20-bit label per entry kind (JS call, Wasm call, OSR entry, deopt-resume stub, IC stub, runtime trampoline); `sspush` in every prologue and `sspopchk` in every epilogue; `ssp` writes and `ssamoswap` only in the substrate's segment-switch stub ([JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)). This note fixes the registers, the labels and the stub, and gives the same tables for x86-64 and AArch64 so the three are decided together.

## 2. The ISA facts this rests on

| # | Fact | Source | Conf. |
|---|---|---|---|
| 2.1 | An indirect `jalr`/`c.jr`/`c.jalr` sets the expected-landing-pad state unless `rs1` is `x1`, `x5` or `x7`: `is_lp_expected = (JALR ∨ C.JR ∨ C.JALR) ∧ rs1 ∉ {x1, x5, x7}` | [RISC-V ISA manual, unprivileged, CFI chapter](https://github.com/riscv/riscv-isa-manual/releases) | High |
| 2.2 | A `jalr` through `x7` is a "software-guarded branch" that needs no landing pad; the compiler "is required to ensure it has full control on the possible jump targets" | same | High |
| 2.3 | `x7` is the label register: the caller puts the expected label in bits 31:12 (for example with `lui`); `lpad`'s 20-bit label 0 disables the check; a mismatch or a target that is not a 4-byte-aligned `lpad` raises a software-check exception (cause 18) | same | High |
| 2.4 | `sspush` and `sspopchk` accept only `x1` or `x5`; they are encoded as may-be-operations and behave as such where Zicfiss is not active; `ssamoswap` is not | same | High |
| 2.5 | `ssp` is an unprivileged read-write CSR (0x011) | same | High |
| 2.6 | Switching shadow stacks: store the outgoing stack's top on the outgoing shadow stack as a **checkpoint** (a word holding its own address), and check the incoming checkpoint with `ssamoswap` before writing `ssp`; the manual gives the sequence; a new shadow stack needs a checkpoint at its highest address | same, stack-switching notes | High |
| 2.7 | psABI: `x3`/`gp` and `x4`/`tp` are unallocatable and "procedures should not modify" them; `x5`–`x7` are temporaries; `s0`–`s11` (`x8`, `x9`, `x18`–`x27`) are callee-saved; a frame pointer lives in `s0` | [RISC-V psABI](https://github.com/riscv-non-isa/riscv-elf-psabi-doc/releases), calling convention chapter | High |
| 2.8 | Linux riscv64: shadow stacks are enabled with `PR_SET_SHADOW_STACK_STATUS`; writing `ssp` "will change the active shadow stack"; at signal delivery the token is saved on the current shadow stack | [kernel Zicfiss docs](https://docs.kernel.org/arch/riscv/zicfiss.html) | High |
| 2.9 | Linux arm64 GCS: `map_shadow_stack` with `SHADOW_STACK_SET_TOKEN` places a cap token; `GCSPR_EL0` is "readable by userspace but can only be updated via specific GCS instructions"; the architecture provides switching instructions with validity checks | [kernel GCS docs](https://docs.kernel.org/arch/arm64/gcs.html) | High |
| 2.10 | Arm's switch instructions are `GCSSS1` (check the incoming cap, mark it in progress, switch) and `GCSSS2` (write a valid cap on the outgoing stack, return its address) | [Arm Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/latest), read through a mirror of Arm's register pages | Medium |
| 2.11 | x86-64 CET: `RSTORSSP` switches to a shadow stack after checking its restore token; `SAVEPREVSSP` leaves a restore token for the previous one; `ENDBR64` marks indirect-branch targets; the `NOTRACK` prefix exempts an indirect branch | [Intel SDM](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html), Vol. 1 ch. 17 and Vol. 2 | Medium |
| 2.12 | Linux x86: "the main program and its signal handlers use the same shadow stack"; WRSS is a separately enabled feature | [kernel x86 shadow-stack docs](https://docs.kernel.org/arch/x86/shstk.html) | High |
| 2.13 | AArch64 BTI: `BLR` targets need `BTI c`; `BR` through `x16`/`x17` may land on `BTI c` or `BTI j`; `BR` through any other register needs `BTI j` | [Arm Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/latest) | Medium |
| 2.14 | AAPCS64: `x19`–`x28` callee-saved, `x29` frame pointer, `x30` link register, `x16`/`x17` intra-procedure-call scratch; `x18` is the platform register: reserved on Apple platforms and Windows | [Apple](https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms), [Microsoft ARM64](https://learn.microsoft.com/en-us/cpp/build/arm64-windows-abi-conventions) | Medium (pages fetched, bodies not quoted this session) |
| 2.15 | x86-64: `rbx`, `rbp`, `r12`–`r15` are callee-saved in both the System V ABI and Microsoft's x64 convention, which also saves `rdi`, `rsi` and `xmm6`–`xmm15` | [System V psABI](https://gitlab.com/x86-psABIs/x86-64-ABI), [Microsoft x64](https://learn.microsoft.com/en-us/cpp/build/x64-software-conventions) | Medium |

Two consequences of 2.1 and 2.2 matter more than any register choice:
- **On riscv64, an indirect branch through `x1`, `x5` or `x7` bypasses landing pads.** Generated code must never branch through them except `ret` (`jalr x0, 0(x1)`) in an epilogue right after `sspopchk x1`. The validator enforces it (R2-27).
- **`x7` must hold only labels.** If a value loaded from memory reached `x7` and was used as a branch source, it would be a software-guarded branch with no pad. So `x7` is written only by `lui x7, label` immediately before an indirect call or jump, never allocated.

## 3. Register assignment (R2-24)

**Decision R2-24: the same roles on every ISA, each pinned role in a register that the platform ABI preserves across calls (so it survives calls into Rust), and no other callee-saved register in the engine's internal conventions ([frames-deopt-stackmaps.md](frames-deopt-stackmaps.md), R2-10).**

| Role | riscv64 | AArch64 | x86-64 | Notes |
|---|---|---|---|---|
| Stack pointer | `sp` (`x2`) | `sp` | `rsp` | |
| Frame pointer | `s0` (`x8`) | `x29` | `rbp` | Frame record `R` = `fp` on x86-64 and AArch64, `fp − 16` on riscv64 (psABI); `[R]` saved fp, `[R + 8]` return address ([frames-deopt-stackmaps.md](frames-deopt-stackmaps.md), R2-9) |
| Link register | `ra` (`x1`) | `x30` | (stack) | |
| **Cage base** | `s11` (`x27`) | `x28` | `r15` | Pinned in all generated code; decompression `sh2add.uw` / `add …, uxtw #2` / `[r15 + r*4 − 4]` ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)) |
| **Sealed-region base** | `s10` (`x26`) | `x27` | `r14` | The trusted region's sealed class ([code-table-and-ledger.md](code-table-and-ledger.md), R2-18) |
| **Agent context** | `s9` (`x25`) | `x26` | `r13` | Pointer to the agent's runtime-trusted block: code-table bases, canonical depth counter, interrupt flag, top exit frame, deopt buffer |
| Wasm instance | `s8` (`x24`) | `x25` | `r12` | Pinned in Wasm code; allocatable in JS code |
| Wasm memory 0 base | `s7` (`x23`) | `x24` | `rbx` | Pinned in Wasm code when memory 0's reservation covers its maximum size, so the base never moves; otherwise (memory64, or memories beyond the Sv39 guarded count that reserve less than their maximum) the base is reloaded from the instance after every call and `memory.grow` |
| T0 bytecode pointer | `s8` (`x24`) | `x25` | `r12` | T0 and the IC interpreter only (they run no Wasm) |
| T0 dispatch-table base | `s7` (`x23`) | `x24` | `rbx` | Same |
| T0 accumulator | `a0` (`x10`) | `x0` | `rax` | The result register, so a call's result lands in it; spilled to its frame slot across calls |
| **Landing-pad label** | `x7` (`t2`) | — | — | Never allocated (2.3) |
| Indirect-branch target | `t1` (`x6`) | `x16` (calls, tail calls), `x9` (jumps) | `r11` | Never `x1`, `x5`, `x7` on riscv64 |
| Assembler scratch | `t0` (`x5`), `t3`–`t6` | `x17` | `r10` | `t0` is never a branch source or a link |
| JS call: callee, `argc`, `new.target`, result | `a0`, `a1`, `a2`; `a0` | `x0`, `x1`, `x2`; `x0` | `rdi`, `rsi`, `rdx`; `rax` | Arguments on the stack in the caller's area (frames note) |
| Wasm parameters | `a0`–`a7`, `fa0`–`fa7`, then stack | `x0`–`x7`, `v0`–`v7`, then stack | `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`, `xmm0`–`xmm7`, then stack | One internal convention on each ISA, whatever the OS |
| Never touched | `gp` (`x3`), `tp` (`x4`) | `x18` | — | psABI and platform reservations (2.7, 2.14) |

- **Why the intersection on x86-64.** Microsoft's convention also preserves `rdi` and `rsi`, System V does not (2.15). Pinning only `rbx`, `rbp`, `r12`–`r15` gives one lowering for Linux, macOS and Windows, which JS-D12 and JS-D22's "one lowering per ISA" ask for.
- **Why `x18` is never used on AArch64.** Apple and Windows reserve it (2.14); Linux does not, but one lowering per ISA forbids an OS-dependent register set.
- **Allocatable registers left for T2.** Counted the same way on every ISA (scratch and branch-target registers excluded; they are usable between their fixed uses): riscv64 16 general registers (`a0`–`a7`, `s1`–`s6`, and `s7`/`s8` in JS code) plus 5 scratch or target (`t0`, `t1`, `t3`–`t6` less `t0`'s branch ban) and all 32 FP registers; AArch64 22 (`x0`–`x8`, `x10`–`x15`, `x19`–`x23`, and `x24`/`x25` in JS code) plus `x9`, `x16`, `x17`; x86-64 9 (`rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`, `r9`, and `rbx`/`r12` in JS code) plus `r10`, `r11`. x86-64 is the tightest, as it is for every engine on that ISA; the pinned set is the minimum the cage design needs.
- **Floating-point control state** (`fcsr`/`frm`, FPCR, MXCSR) is never written by generated code; rounding is static per instruction on riscv64 and the default mode elsewhere ([JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)); the renderer asserts the control register at each frame start ([JS 6.9](../../reports/Izcal%20JavaScript%20engine.md)).

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Three pinned registers (cage, sealed, agent), plus two in Wasm and T0** | One instruction to reach the cage, the sealed region and the agent block; survives Rust calls | Three fewer allocatable registers everywhere | **Recommended** |
| (b) Only the cage base pinned; the rest loaded from a thread-local | More registers on x86-64 | Every table access, poll and depth check pays a thread-local load; on riscv64 `tp` is the psABI's thread pointer and Izcal may not repurpose it | Rejected |

**Reversal trigger.** Build x86-64 T2 with and without the sealed-region base pinned (in T2 and W2 the sealed base is needed only by deopt tails and constant-table addressing, which can load it from the agent block). If the unpinned build is faster by more than 1%† on JetStream 3 at JS-M2, T2 and W2 on x86-64 stop pinning it and `r14` becomes allocatable there; T0 keeps it.

## 4. Landing pads and their labels (R2-25)

**Decision R2-25: eight indirectly entered kinds, each with a fixed label on riscv64, a fixed BTI kind on AArch64 and `endbr64` on x86-64; per-convention code tables give x86-64 and AArch64 what labels give riscv64 for calls.** The spec's six kinds ([JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)) miss two indirect jumps that the generated code itself makes: interpreter dispatch and jump tables.

| Entry kind | Reached by | riscv64 label | AArch64 | x86-64 |
|---|---|---|---|---|
| JS-call entry | Indirect call through the JS code table | `0x1A501` | `bti c` | `endbr64` |
| Wasm-call entry | `call_indirect`, `call_ref`, `return_call_indirect` through the Wasm code table | `0x1A502` | `bti c` | `endbr64` |
| OSR entry | Indirect jump from a T0/T1/W0/W1 loop header | `0x1A503` | `bti j` | `endbr64` |
| Deopt-resume entry | Genuine call from a deopt tail into T0's resume entries (address from the agent block) | `0x1A504` | `bti c` | `endbr64` |
| IC stub | Indirect call from a T0/T1 IC site to a compiled IC stub | `0x1A505` | `bti c` | `endbr64` |
| Runtime trampoline | Indirect call from generated code to a runtime entry (address from the agent block) | `0x1A506` | `bti c` | `endbr64` |
| Dispatch target | Indirect jump from T0's, W0's or the IC interpreter's dispatch to an operation handler | `0x1A507` | `bti j` | `endbr64` |
| Jump-table target | Indirect jump from a bounds-checked `Switch` jump table (T1, T2, W1, W2) to a case block | `0x1A508` | `bti j` | `endbr64` |
| Trap pad | Unused entries of a code table | The table's call label, so a forged index reaches it and traps | `bti c` | `endbr64` |

- **Label values.** Non-zero (0 disables the check, 2.3); the high 12 bits are a fixed Izcal prefix (`0x1A5`) and the low 8 bits the kind. A collision with labels that the platform toolchain uses for Rust's own indirect calls would let such a call reach an engine pad; whether and how the Rust toolchain labels riscv64 landing pads is unknown (Q-R2-9).
- **Direct calls and branches** (deopt exits, out-of-line paths, the status slow path) need no pad. Code reached only directly carries none, so it is not a valid indirect target at all.
- **The entry trampoline** (Rust calling into the engine) saves every register the platform ABI makes callee-saved, because the engine's internal conventions save none (R2-10): `rbx`, `rbp`, `r12`–`r15`, and on Windows also `rdi`, `rsi` and `xmm6`–`xmm15`, with Windows' 32-byte home space for the calls it makes back into Rust; `x19`–`x28` and `d8`–`d15` on AArch64; `s0`–`s11` and `fs0`–`fs11` on riscv64. Its landing pad must accept Rust's indirect calls: if the Rust toolchain emits unlabeled calls (`x7` arbitrary), the trampoline's `lpad` uses label 0, which is the only label-0 pad in the engine (Q-R2-9).
- **The whole renderer is built with landing pads.** BTI, IBT and Zicfilp protect only if every executable page of the process is built with them; Rust code, the C libraries a renderer links and the engine's generated code all have to be. Whether each platform toolchain does so is checked black-box at JS-M0 (Q-R2-9 and [RV 3.8](../Izcal%20JavaScript%20engine/riscv64.md)).
- **AArch64 jump discipline.** Indirect calls are `blr x16`; OSR jumps are `br x9`, which cannot land on a `bti c` pad; tail calls are `br x16`, which must reach call-kind pads and therefore can also reach `bti j` pads (2.13). That one overlap is accepted: OSR pads check the frame's identity against the code table before running.
- **x86-64.** `notrack` is never emitted, so every indirect branch is tracked (2.11).
- **riscv64.** Landing pads are 4-byte aligned (the assembler pads with `c.nop`); the caller's sequence is `lui x7, LABEL … ; jalr ra, 0(t1)` (`lui` places its 20-bit immediate in bits 31:12) with nothing between the label write and the branch that writes `x7`.

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **One label per kind** (the spec's kinds plus dispatch and jump tables) | Simple; a wrong-kind target faults | Within a kind, any pad of that kind is reachable | **Recommended** |
| (b) Wasm labels from a hash of the canonical signature | Hardware check of `call_indirect` types | `call_indirect` already checks the exact canonical type ID in the trusted entry ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)); 17-bit hashes collide; a function reachable through several types would need several pads | Rejected; revisit if a type-confusion bug is found that the trusted check missed |

## 5. The segment-switch stub (R2-26)

**Decision R2-26: one stub per ISA in `hg-substrate-os`, inventoried with the segments ([JS 6.6](../../reports/Izcal%20JavaScript%20engine.md)), is the only code that moves `sp` to another stack and the only code that switches the shadow stack; it uses each architecture's token protocol and never writes a return address.**

**Interface.** `switch_stack(target: &StackDesc, current: &mut StackDesc)`, called from Rust (JSPI suspend and resume, Stack Switching) with the platform ABI. A `StackDesc` lives in the runtime-trusted class and holds the saved `sp`, the shadow-stack checkpoint or token address, the segment chain, the stack's canonical depth total and, on Windows, the TEB stack bounds.

**Steps, the same on every ISA:**
1. Standard prologue on the current stack (pad, protect the return address, link the frame); save the platform ABI's callee-saved registers.
2. `current.sp = sp`.
3. Switch the shadow stack with the architecture's checked protocol (below), recording the outgoing checkpoint or token in `current`.
4. `sp = target.sp`; update the agent's segment bounds (used by prologue checks) and, on Windows, the TEB's stack base and limit.
5. Restore callee-saved registers from the target stack and run the standard epilogue: the return goes to the instruction after the call that suspended the target stack, and the shadow-stack check passes because that call pushed it.
6. **A fresh stack** has no suspended frame: `target.fresh` makes step 5 a direct jump to the substrate's stack-start routine, which never returns and ends by switching back.

**Step 3 per ISA:**

| ISA | Sequence | Source |
|---|---|---|
| riscv64 | `ssrdp t1; beqz t1, done` (Zicfiss inactive); `ld t3, CKPT(target)`; `ssamoswap.d t4, x0, (t3)` (take the checkpoint, zero it); `bne t4, t3, fault` (a legal checkpoint holds its own address); `addi t4, t4, 8`; `csrrw t4, ssp, t4` (switch; `t4` = old `ssp`); `addi t4, t4, -8`; `ssamoswap.d x0, t4, (t4)` (checkpoint on the outgoing stack); `sd t4, CKPT(current)` | The ISA manual's stack-switching sequence (2.6), with `t1`/`t3`/`t4` instead of `ra`, which step 1 already saved |
| AArch64 with GCS | `gcsss1` with the target's cap address; `gcsss2` to obtain the outgoing stack's cap address; store it in `current` | 2.9, 2.10 (Medium) |
| AArch64 without GCS (macOS today, Q-R2-6) | No shadow-stack step. PAC-RET signatures stay valid because every frame was signed with its own `sp` and step 5 restores the exact `sp` | [JS 6.6](../../reports/Izcal%20JavaScript%20engine.md); verified at JS-M0 on the Mac |
| x86-64 (Linux) | `rdsspq` (zeroed first; stays zero if shadow stacks are off) to skip; `rstorssp` on the target's restore token; `saveprevssp` to leave a restore token on the outgoing stack; record its address in `current` | 2.11, 2.12 (Medium); the token address arithmetic is checked black-box at JS-M0 on the CET Linux host the owner names |
| x86-64 (Windows) | Open: if user mode may switch shadow stacks for engine stacks (Q-JS-15 (1)), as Linux; otherwise every stack is a fiber and steps 2–5 are `SwitchToFiber` | [JS 6.6](../../reports/Izcal%20JavaScript%20engine.md), Q-JS-15 |

**Creating a stack's shadow stack.** On Linux, `map_shadow_stack` with `SHADOW_STACK_SET_TOKEN` (x86 restore token; arm64 cap; riscv64 to be checked at JS-M0, [JS 6.6](../../reports/Izcal%20JavaScript%20engine.md)). If the riscv64 kernel places no checkpoint, the creation routine writes one with `ssamoswap` at the highest address (2.6). The creation routine is inventoried with the stub and validated with it.

**Segment growth** inside one stack uses a separate growth stub that changes only `sp`, never the shadow stack ([frames-deopt-stackmaps.md](frames-deopt-stackmaps.md), R2-17).

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **One substrate stub per ISA, token-checked** | One audited place; the hardware checks every switch target | Windows depends on Q-JS-15 | **Recommended** (the spec's design, made concrete) |
| (b) Shadow-stack writes enabled (`WRSS`, GCS store) to build stacks by hand | Flexible | A write primitive to the shadow stack is exactly what CFI is meant to deny; the spec forbids it ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) | Rejected |

## 6. Validator rules for control-flow-sensitive instructions (R2-27)

**Decision R2-27: the machine-code validator ([JS 6.7](../../reports/Izcal%20JavaScript%20engine.md)) runs three profiles: generated code (T1, T2, W1, W2, IC stubs, run time), generated interpreters and stencils (T0, W0, IC interpreter, build time), and inventoried stubs (the switch, growth, deopt-tail, poll and trampoline stubs, build time), with these rules.**

| Rule | x86-64 | AArch64 | riscv64 | Allowed in |
|---|---|---|---|---|
| Shadow-stack writes and pointer changes | `WRSS`, `WRUSS`, `INCSSP`, `RSTORSSP`, `SAVEPREVSSP`, `SETSSBSY`, `CLRSSBSY` | `GCSSS1`, `GCSSS2`, `GCSPUSHM`, `GCSPOPM`, GCS stores | any CSR access to `ssp`, `ssamoswap.w/.d` | The switch stub and the creation routine only |
| Shadow-stack push and check | (by `call` and `ret`) | (by `bl`/`blr` and `ret`) | `sspush`/`sspopchk` with `x1` only in the standard prologue and epilogue | Prologues and epilogues |
| Pointer signing | — | `pac*`, `aut*`, `pacga`, `xpac*` except `paciasp` first in a prologue and `autiasp` last before `ret` | — | Prologues and epilogues |
| Indirect-branch sources (dispatch and jump-table jumps included, which land only on their own kinds' pads; `x7` software-guarded branches are never used, even for read-only tables) | no `notrack` | `blr x16`, `br x9`, `br x16` only | `jalr` with `rs1` ∉ {`x1`, `x5`, `x7`} except `ret` after `sspopchk x1` | All profiles |
| Label register | — | — | `x7` written only by `lui x7, label` immediately before the indirect branch | All profiles |
| Reserved registers | `r13`–`r15` written only in stubs | `x18` never; `x26`–`x28` only in stubs | `gp`, `tp` never; `s9`–`s11` only in stubs | Stubs |
| Stack-pointer writes | `rsp` only in prologue, epilogue, outgoing-argument reservation | same | same | Plus: deopt tail and OSR entry may set `sp = R − size` with a size from the sealed metadata, 16-aligned and below the header; switch and growth stubs may load `sp` from a `StackDesc` |
| CSRs and system state | no `wrpkru`, no `xrstor` of CET state | no FPCR or system-register writes | no CSR access except `ssrdp` (and the stub's `ssp`); no `fence.i` | `fence.i` only in the install-safepoint routine ([JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)) |
| Buffer accesses | A checked buffer access loads its base and length from the buffer table after the last call or safepoint on every path ([optimizer-ir.md](optimizer-ir.md) 6.1) | same | same | All profiles |
| Constant tables | Loads from a code object's constant table use its embedded address and an in-range slot ([code-table-and-ledger.md](code-table-and-ledger.md)) | same | same | Generated code |

The stubs are a few dozen instructions per ISA; their build-time profile is an allow-list of exact sequences, so a change to a stub is a change to the validator's profile and is reviewed as both.

## 7. Lessons learned

| Source | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| RISC-V CFI design | Returns and "software-guarded" branches are exempt from landing pads by register (2.1, 2.2) | On riscv64 the register a branch uses is a security property | R2-24 keeps `x1`, `x5`, `x7` out of indirect branches; R2-27 checks it |
| The CFI chapter's stack-switch note | A checkpoint that holds its own address, taken with `ssamoswap`, lets user code switch stacks without trusting a pointer in memory (2.6) | Store switch targets on the protected stack, not in a context structure | R2-26's riscv64 sequence; `StackDesc` holds only the checkpoint's address, which the hardware re-checks |
| V8's CFI plan | Legitimate return-stack changes "during optimization / deoptimization and exception handling" were left to be dealt with ([V8](https://v8.dev/blog/control-flow-integrity)) | Shadow-stack changes must be confined to one place | One stub per ISA |

## 8. Open questions and flagged claims

- **Q-R2-9.** Does the Rust toolchain for riscv64 emit Zicfilp landing pads, labeled or not, and do the Rust and C toolchains of each platform build every executable page with BTI, IBT or Zicfilp? It decides the entry trampoline's label (Rust calls into the engine through a function pointer), whether `0x1A5xx` can collide with toolchain labels, and whether forward-edge CFI is whole-process. Checked black-box at JS-M0 (build CFI-marked binaries, read their ELF properties, inspect the emitted pads), like the C toolchain item in [RV 3.8](../Izcal%20JavaScript%20engine/riscv64.md).
- **Q-R2-10.** The exact `RSTORSSP`/`SAVEPREVSSP` token addresses and the GCS cap behaviour of `GCSSS1`/`GCSSS2` are Medium here. Both are verified black-box at JS-M0 (Phase 6: the CET Linux host, an arm64 GCS host when one exists).
- **Q-R2-6** (frames note) applies: PAC, BTI and GCS availability on macOS and Windows on Arm.

**Flagged claims.** Rows 2.10, 2.11, 2.13, 2.14 and 2.15 are from the architecture manuals and ABI pages as recalled or seen through mirrors, not quoted from the primary text this session (Medium). Label values and register choices are Izcal's own choices, not thresholds, so they carry no †.
