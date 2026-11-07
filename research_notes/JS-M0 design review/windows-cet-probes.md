# JS-M0: Windows shadow stacks, the five Q-JS-15 questions, probed

*31 October 2026. Host: rig253 (Windows 11 build 26300.9711, AMD Ryzen AI Max+ 395, Zen 5: CET shadow stacks, no IBT). Probes: [`probes/windows-cet/`](probes/windows-cet/) (a standalone cargo workspace, rustc 1.99.1, windows-sys 0.61.2). Raw output: [`probes/windows-cet/logs/`](probes/windows-cet/logs/).*

This note answers the five Windows questions that the JS spec leaves open for JS-M0 ([JS 6.6](../../reports/Izcal%20JavaScript%20engine.md), Q-JS-15), checks part 2's decisions against the answers (R2-17 in `frames-deopt-stackmaps.md`, R2-26 in `riscv64-abi.md`, Q-R2-5; both notes are part 2 of the design review, on branch `docs/js-m0-design-2` and not yet on main), and proposes the changes they lead to.

**Method (M-1 part 3).** No third-party source code was read. The probes were written from Microsoft Learn's prose and struct definitions ([PROCESS_MITIGATION_USER_SHADOW_STACK_POLICY](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_user_shadow_stack_policy), [SetProcessDynamicEnforcedCetCompatibleRanges](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessdynamicenforcedcetcompatibleranges), [PROCESS_DYNAMIC_ENFORCED_ADDRESS_RANGE](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_dynamic_enforced_address_range), [SetProcessDynamicEHContinuationTargets](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessdynamicehcontinuationtargets), [PROCESS_DYNAMIC_EH_CONTINUATION_TARGET](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_dynamic_eh_continuation_target), [UpdateProcThreadAttribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)'s policy2 bits) and from the Intel SDM's descriptions of `RDSSP`, `RSTORSSP`, `SAVEPREVSSP` and `WRSS`. The CET APIs are resolved with `GetProcAddress`; the fiber functions are declared from their documented signatures; everything else comes from windows-sys. All behaviour below was observed, not inferred from code.

**Rig rules.** The rig was booked (`b-ea3ca1` heavy for building, then `b-6df736` exclusive for timing; `rig status` showed rig253 "unreachable" while ssh worked). Each timing run is preceded by a `rig check rig253 --kind exclusive --session js-m0` line in the timing logs. No system setting, registry value or system-wide mitigation was read for writing or changed. Every policy was set per process, on a child the probe started with `PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY`, or by the probe on itself with `SetProcessMitigationPolicy`. The system policy was only read (`Get-ProcessMitigation -System`: every UserShadowStack field `NOTSET`).

## 1. Answers

| # | Question | Answer | Confidence |
|---|---|---|---|
| 1 | Can user mode create and switch shadow stacks for engine segments outside the thread and fiber APIs? | **Switch: yes. Create: no.** `RSTORSSP` and `SAVEPREVSSP` work in user mode on Windows in compatibility and strict mode, and the probe's own stub switches between the thread's shadow stack and a fiber's shadow stack, with Rust, SEH unwinding, `Sleep` and thread creation running on the switched stack. Windows offers no documented way to make shadow-stack memory outside thread and fiber creation, and the probes found none: a forged token in ordinary memory faults (`0xC0000005`), `WRSS` is not enabled (`0xC000001D`), and `VirtualProtect` refuses to change a shadow-stack page (error 87). **`CreateFiberEx` works as a shadow-stack allocator**: a new fiber that has never run already has a shadow stack with a restore token at its top, and the engine's stub can switch to it without ever calling `SwitchToFiber`. | Switching: **High** (observed directly, 0 faults in both modes). That Windows will keep tolerating it: **Medium**, because the behaviour is architectural but not documented by Microsoft |
| 2 | Does each fiber get its own shadow stack, and what does `SwitchToFiber` cost? | **Yes.** Each fiber gets its own read-only shadow-stack allocation, reserved at the same size as the fiber's stack reservation, committed on demand behind a guard page, and freed by `DeleteFiber`. `SwitchToFiber` uses the architectural restore-token protocol. A round trip (two switches) costs **60.8–71.9 ns** with shadow stacks on, against **50.6–55.7 ns** with them off. Creating and deleting a fiber costs **11.1–13.4 µs** with shadow stacks on, against **4.4–5.0 µs** off | **High** |
| 3 | How are JIT ranges registered under `CetDynamicApisOutOfProcOnly`? | **By the broker, with a `PROCESS_SET_INFORMATION` handle.** In-process calls fail with error 5. The broker's call succeeds and takes effect: bad JIT code becomes fatal. **One call per code reservation is enough**, because a range that is only reserved, and even an unallocated one, is accepted, and code committed into it later is enforced. A call costs **3.4–4.0 µs** for one range and **8.0–9.4 µs** for 64, plus about **9.3 µs** for a broker IPC round trip (D5). On this build, `CetDynamicApisOutOfProcOnly` is **on by default** in every process | **High** |
| 4 | Does Rust run correctly on a segment once the thread's stack bounds are updated? | **Yes, and the bounds are required.** With NT_TIB `StackBase` and `StackLimit` set to the segment, formatting, allocation, a 256 KiB frame (stack probes), `catch_unwind` of a panic, backtraces through the switch stub, and thread spawn and join all work. **Without the update**, ordinary code still runs, but a panic kills the process (`0xE06D7363`, the exception goes unhandled). Stack overflow on a segment is reported (`STATUS_STACK_OVERFLOW` reaches a vectored handler, and Rust's own overflow message prints) only if the segment ends in `PAGE_GUARD` pages and the bounds are updated. With a no-access guard alone, the process dies silently with `0xC0000005` and no handler runs. `GetCurrentThreadStackLimits` keeps reporting the thread's original low bound | **High** |
| 5 | Can strict mode stay on with the JIT? | **Yes.** Under strict mode, generated code that keeps call and return paired runs. A return-address overwrite is fatal (`0xC0000409`) whether or not the code is registered. Registration therefore adds nothing under strict mode. In compatibility mode, the same violation in **unregistered** JIT code is **silently tolerated** (the code returns 7), and becomes fatal only once the range is registered. A process can be upgraded from compatibility to strict mode at run time but never downgraded | **High** |

## 2. Baseline

`cetprobe policy` (CETCOMPAT image, started from the ssh shell without a policy attribute), and the same for an image linked without `/CETCOMPAT`:

```
policy 0x105 [EnableUserShadowStack SetContextIpValidation CetDynamicApisOutOfProcOnly]
main thread: SSP=0xa0a05fefc0 region base=0xa0a05fe000 alloc=0xa0a0500000 size=0x1000 COMMIT PRIVATE protect=0x2 alloc_protect=0x2
```

- User-mode Hardware-enforced Stack Protection is **on by default** in compatibility mode, with set-context IP validation and `CetDynamicApisOutOfProcOnly`, **for both images**: the non-CETCOMPAT image gets the same `0x105`. A child started with policy2 `CET_USER_SHADOW_STACKS_ALWAYS_OFF` has SSP 0 and still carries `CetDynamicApisOutOfProcOnly`; only `..._OUT_OF_PROC_ONLY_ALWAYS_OFF` (2 << 48) clears that bit. `STRICT_MODE` (3 << 28) gives `0x115`.
- A thread's shadow stack is a private, read-only (`PAGE_READONLY`, 0x2) allocation of 1 MiB here, committed near the top.
- At run time, `SetProcessMitigationPolicy` upgraded compatibility mode to strict and set `CetDynamicApisOutOfProcOnly`. Clearing either of them failed with error 5. Turning on strict mode in a process whose shadow stacks are off failed (error 5). This matches Microsoft's field descriptions.

## 3. Evidence per question

Commands are run from `probes/windows-cet` on rig253. `launch <policy>` starts a child with that policy2 word (`on`, `strict`, `off`, `*-oop` adds out-of-proc-only, `*-nooop` sets it always off) and prints `CHILD … exit=…`. All lines below are copied from [`logs/functional-2026-10-31.log`](probes/windows-cet/logs/functional-2026-10-31.log) and the two timing logs.

### Q2 first: fibers (`launch on fiber`, `launch strict fibersize`)

```
CreateFiberEx#1 (not yet run): new region base=0xa1a27fb000 alloc=0xa1a2700000 size=0x4000 COMMIT PRIVATE protect=0x2 alloc_protect=0x2 tokens=[(a1a27feff0, a1a27feff9)]
CreateFiberEx#1 (not yet run): new region base=0xa1a27fa000 alloc=0xa1a2700000 size=0x1000 COMMIT PRIVATE protect=0x102 ...
fiber#1 running: SSP=0xa1a27fefe8 region base=0xa1a27fe000 alloc=0xa1a2700000 ...
fiber#1 suspended: tokens in its region [(a1a27fefd8, a1a27fefe1)] (fresh-fiber tokens were [(a1a27feff0, a1a27feff9)])
distinct shadow stacks: main/f1 true f1/f2 true main/f2 true
CreateFiberEx(commit=0x1000, reserve=0x10000): allocation 0xfd3930000 reserved=0x10000 committed=0x2000 alloc_protect=0x2 shadow stack (read-only allocation)
CreateFiberEx(commit=0x10000, reserve=0x800000): allocation 0xfd4c00000 reserved=0x800000 committed=0x5000 alloc_protect=0x2 shadow stack (read-only allocation)
```

- `CreateFiberEx` makes two allocations: the data stack and a read-only shadow stack **reserved at the same size as the data stack** (64 KiB, 1 MiB or 8 MiB here). The shadow stack has a `PAGE_GUARD` read-only page (0x102) below its committed top, so it grows on demand like a thread stack. `DeleteFiber` frees both.
- A fiber that has never run already carries a 64-bit **restore token** (a word whose value is its own address + 8, with bit 0 set) 16 bytes below its committed top. Once the fiber has suspended through `SwitchToFiber`, a token sits just below its live SSP. So `SwitchToFiber` uses the SDM's token protocol.
- Costs are in Section 4.

### Q1: switching shadow stacks without the fiber API (`launch strict own`, `forge`, `wrss`, `ssprotect`)

`ss_start` and `ss_switch` ([`src/asm.rs`](probes/windows-cet/src/asm.rs)) are the probe's own versions of R2-26's stub. Each saves the callee-saved registers (including `xmm6`–`xmm15`) and records `rsp`. If shadow stacks are on, it runs `RSTORSSP` on the target's token and `SAVEPREVSSP`, recording the outgoing token's address (old SSP − 8). It then loads the target's `rsp` and returns. A new stack is entered by a genuine `call`, so no return address is ever written. The target shadow stack is the token of a **fiber that was created and never run**:

```
fiber A token at 0xf8024feff0 value 0xf8024feff9
coroutine (own switch) running: SSP=0xf8024fefe8 region base=0xf8024fe000 alloc=0xf802400000 ...
coroutine rsp=0x27c8c40feb0 (own segment [0x27c8c310000,0x27c8c410000))
coroutine: Rust work ok, sum=4999950000
coroutine: catch_unwind of a panic -> caught=true
coroutine: 20 x Sleep(1) (context switches) ok
coroutine: thread spawn/join ok (7)
main: back after coroutine suspended; SSP=0xf801cfefc0 (expect 0xf801cfefc0)
main: coroutine's restore token at 0xf8024fefe0 value 0xf8024fefe9
coroutine: resumed, SSP=0xf8024fefe8
main: coroutine finished; SSP=0xf801cfefc0; fiber A token now at 0xf8024feff0 value 0xf8024feff9 (same address: true)
main: SwitchToFiber(A) after foreign use: fiber ran=true its SSP=0xf8024fefe8
RESULT own-switch: ok
CHILD strict ["own"] exit=0x00000000
```

The same run passes in compatibility mode. It also passes with `CETPROBE_RUNONCE=1`, which enters the fiber once through `SwitchToFiber` and then reuses the token `SwitchToFiber` left. Exceptions are dispatched by the kernel and SEH unwinds while SSP points into a shadow stack that is not the current fiber's, and nothing objects. Because the coroutine leaves at the depth it entered, its token is restored at the original address, and the fiber still works through `SwitchToFiber` afterwards.

Making a shadow stack by any other route failed:

```
forge: RSTORSSP on a forged token in ordinary RW memory at 0x1492ef50800
CHILD strict ["forge"] exit=0xc0000005          (on: 0xc0000005; off: 0xc000001d, RSTORSSP is #UD without CET)
wrss: WRSSQ to the current shadow stack at 0xbff17fef80
CHILD strict ["wrss"] exit=0xc000001d            (WRSS not enabled for user mode)
VirtualProtect(own shadow-stack page, PAGE_READWRITE): ok=false err=87 old=0x1
```

**Conclusion.** Create: no (High). The documented interfaces create shadow stacks only with threads and fibers, and nothing else tried could make one. Switch: yes (High as observed). Microsoft does not document whether user code may switch shadow stacks itself, so whether later builds keep allowing it is Medium.

### Q3: registering JIT ranges out of process (`launch … jit`, `broker …`)

```
child policy 0x105 [EnableUserShadowStack SetContextIpValidation CetDynamicApisOutOfProcOnly]
in-proc SetProcessDynamicEnforcedCetCompatibleRanges: Err(5) flags-out=0x1
in-proc SetProcessDynamicEHContinuationTargets: Err(5) flags-out=0x1
broker: with PROCESS_QUERY_LIMITED_INFORMATION only: Err(5)
broker: register whole region with PROCESS_SET_INFORMATION: Ok(()) flags-out=0x3
broker: EH continuation target: Ok(()) flags-out=0x3
CHILD broker on-oop bad exit=0xc0000409          (the same code, unregistered, returns 7)
CHILD broker on-oop late exit=0xc0000409         (range registered while only MEM_RESERVE; code committed after)
broker: register unallocated range: Ok(())
broker: register unaligned 16-byte range: Err(487)
```

- Under `CetDynamicApisOutOfProcOnly`, both APIs fail in process with `ERROR_ACCESS_DENIED`. A broker holding a `PROCESS_SET_INFORMATION` handle succeeds, and the registration takes effect.
- Registration is by address range, not by allocation. A reserved range can be registered before any page in it is committed, and the probe also registered an unallocated range. Ranges must be page-aligned (error 487 otherwise). So a renderer needs **one broker call per JIT code reservation, at startup**, not one per code region.
- Without `CetDynamicApisOutOfProcOnly` (`on-nooop`), in-process registration succeeds and has the same effect.
- The broker in this probe ran at High integrity from the elevated ssh session, and the child was not in an AppContainer. The access check observed is on the handle's rights (`PROCESS_SET_INFORMATION`), which Izcal's broker holds on the renderer it starts.

**Conclusion (High).** It costs one IPC round trip plus one API call, once per code reservation.

### Q4: Rust on a segment (`launch strict seg <keep|update> <test>`)

`seg_call` switches only `rsp` to a 1 MiB committed segment and keeps the thread's shadow stack, which is R2-17's growth stub. It has SEH unwind info (an `rbp` frame). In `update` mode, NT_TIB's `StackBase` and `StackLimit` (the documented first fields of `NT_TIB`, reached at `gs:[0x30]`) are set to the segment and restored afterwards.

| Test | bounds kept | bounds updated |
|---|---|---|
| basic (format, Vec, SSP is the thread's) | ok | ok |
| 256 KiB frame (stack probes) | ok | ok |
| `catch_unwind` of a panic | **process dies, `0xE06D7363`** | ok, caught |
| `Backtrace::force_capture` through the stub | ok (19 frames, reaches `main`) | ok |
| thread spawn and join | ok | ok |
| overflow into a no-access guard, with a VEH | dies `0xC0000005`, VEH never runs | dies `0xC0000005`, VEH never runs |
| overflow into 64 KiB of `PAGE_GUARD` pages, with a VEH | dies `0xC0000005` | **VEH runs: `0xC00000FD` caught on the segment** (exit 0x77) |
| the same without a VEH | dies `0xC0000005` | Rust prints "has overflowed its stack", `0xC00000FD` |

`GetCurrentThreadStackLimits` with updated bounds returned `[0x6f4fc30000, 0x204d7200000)`: the high bound follows `StackBase`, but the low bound stays the thread's original one, because it comes from a field that NT_TIB does not document.

**Conclusion (High).** Rust runs correctly on a segment. Exception dispatch requires the TIB bounds to match the segment, and a guard fault needs committed `PAGE_GUARD` slack below the segment, because Windows delivers the exception on the faulting stack and has no `sigaltstack`.

### Q5: strict mode with the JIT (`launch <on|strict|on-nooop|strict-nooop> jit <good|bad> <none|inproc>`)

`good` is `call L; ret; L: mov eax,42; ret`. `bad` overwrites its own return address and returns, which is the pattern JS-D19 forbids.

| Policy | good | bad, unregistered | bad, registered |
|---|---|---|---|
| compatibility (`on-nooop`) | 42 | **7: tolerated** | `0xC0000409` |
| compatibility + out-of-proc-only (default) | 42 | 7: tolerated | in-proc registration denied → 7; broker-registered → `0xC0000409` |
| strict (with or without out-of-proc-only) | 42 | `0xC0000409` | `0xC0000409` |

Everything else in this note also passed under strict mode: fibers, the own switch, the segments and the panics.

**Conclusion (High).** Strict mode can stay on with the JIT. Strict mode makes registration unnecessary, since all violations are fatal. Compatibility mode without registration leaves the JIT's returns unprotected.

## 4. Costs

Two timing runs, each under exclusive booking `b-6df736` with a `rig check` line before every command ([run 1](probes/windows-cet/logs/timing-2026-10-31-run1.log), [run 2](probes/windows-cet/logs/timing-2026-10-31-run2.log)). Each figure is a median over 301 or 201 samples; each sample times a batch of 1,000 or 10–20 operations. The spread within a run was small (p10 to p90 within about 1%; maxima are single outliers). Between runs, absolute values moved by up to 15% (probably clock state), so both are shown; the ratios held.

| Operation | shadow stacks off | compatibility | strict |
|---|---|---|---|
| `SwitchToFiber` round trip (2 switches) | 55.7 / 50.6 ns | 71.9 / 60.8 ns | 73.2 / 61.6 ns |
| Own `ss_switch` round trip (2 switches, incl. RSTORSSP+SAVEPREVSSP each way) | 21.8 / 18.4 ns | 34.8 / 29.6 ns | 34.8 / 29.6 ns |
| … plus TIB bounds written on both switches | 22.0 / 18.7 ns | 35.0 / 29.8 ns | 35.0 / 29.8 ns |
| `CreateFiberEx` (64 KiB commit, 1 MiB reserve) + `DeleteFiber` | 5.0 / 4.4 µs | 13.4 / 11.1 µs | 12.8 / 11.8 µs |

| Range registration, per call (add and remove timed as a pair, halved) | 1 range | 64 ranges |
|---|---|---|
| Broker → renderer, `strict-oop` child | 4.0 / 3.4 µs | 9.4 / 8.0 µs |
| Broker → renderer, `on-oop` child | 4.0 / 3.4 µs | 9.4 / 8.0 µs |
| In process (out-of-proc-only off) | 3.2 / 2.7 µs | 8.5 / 7.2 µs |

With shadow stacks on, one own switch costs about 15–17 ns and one `SwitchToFiber` about 30–36 ns. Shadow stacks add about 6 ns to each own switch, and the TIB update adds less than 0.2 ns per round trip. The fiber API also costs about 7–8 µs at creation for the shadow-stack allocation. A broker registration costs about 13 µs end to end with D5's 9.3 µs round trip ([windows_facts.md](../M1%20D5%20Windows%20IPC/windows_facts.md), RTT), once per code reservation.

## 5. What this means for part 2

**R2-17 holds on Windows unchanged.** It calls for one shadow stack per stack (per JSPI stack), shared by the chained segments, with a growth stub that moves only `sp`. `seg_call` is that growth stub: Rust and SEH work across it, provided the stub also writes `StackBase` and `StackLimit`, as R2-17 already says ("on Windows, the TEB's stack bounds (Q-R2-5)"). Two Windows details need adding:

1. **Each segment ends in committed `PAGE_GUARD` pages** (64 KiB probed), not a bare no-access reservation, and the growth and switch stubs keep `StackLimit` at the top of those guard pages. Without this, a guard hit is a silent `0xC0000005` with no handler, so the runtime cannot report the engine bug R2-17's "Real space" row describes. With it, the fault arrives as `STATUS_STACK_OVERFLOW` on the segment. R2-17's "alternate signal stack" sentence should say "Linux and macOS; on Windows, the guard pages' slack" (Windows has no `sigaltstack`).
2. **No engine code may read `GetCurrentThreadStackLimits` while on a segment**, because its low bound keeps the thread's value. Prologue checks already use the agent's own segment bounds.

**R2-26's open Windows row closes in favour of the Linux design.** Riscv64-abi.md's row reads: "if user mode may switch shadow stacks for engine stacks (Q-JS-15 (1)), as Linux; otherwise every stack is a fiber". User mode may switch them. The x86-64 stub is the same on Windows as on Linux: `rdsspq` to skip when off, then `rstorssp` and `saveprevssp`, with the outgoing token at old SSP − 8, which the probe confirmed (Q-R2-10's token arithmetic is now black-box checked on Windows). Two differences remain:

- **Creation.** A stack's shadow stack comes from `CreateFiberEx(commit = 4 KiB, reserve = shadow size)`, never switched to with `SwitchToFiber`. The fiber's data stack is unused, because the engine's segments come from the substrate's pool. The shadow reservation equals the fiber's stack reservation, so R2-17's 320 KiB shadow stack means a 320 KiB data-stack reservation too, which costs address space but little memory: with a 4 KiB commit and a 64 KiB reservation, 16 KiB of the data stack and 8 KiB of the shadow stack were committed. The creation routine finds the token as the probe does: it identifies the new read-only allocation with `VirtualQuery` and takes the one self-referential token in it. That token cannot be forged by data writes, because ordinary stores cannot write a shadow-stack page. The scan relies on a layout Microsoft does not document, so the routine validates it at startup with a token check and a round trip, and falls back as below if the check fails. A more robust variant, also probed, enters the fiber once through `SwitchToFiber` (about 70 ns more) and uses the token that `SwitchToFiber` leaves just below the fiber's SSP. The creation routine joins the switch stub in the inventory and the validator's exemption list.
- **Cost.** Creating a stack costs about 11–13 µs, mostly the shadow-stack allocation; the per-agent pool that R2-17 already uses for segments should cover JSPI stacks' shadow stacks too, refilled off-thread, so a suspend or resume never allocates. A switch costs about 15 ns, against 30–36 ns for `SwitchToFiber`.

**Fallback (Q-R2-5's second branch), kept but not needed now.** If a later Windows build rejects user-mode `RSTORSSP` onto a fiber's shadow stack, every JSPI stack becomes a fiber and the switch stub's steps 2–5 become `SwitchToFiber`. That is still correct under JS-D19, since `SwitchToFiber` writes no return address, and costs about 2× per switch. The startup self-test that guards creation also guards this choice. Then JS 6.6's last resort, unregistered JIT code under compatibility mode, is never needed on this hardware.

**JS-D19 is confirmed on Windows.** Code that never writes a return address runs under strict mode, and the probe's single return-address write was fatal there. JS-D19's discipline is what lets Windows renderers run strict.

**Recommended Windows renderer policy.** Start every renderer with `CET_USER_SHADOW_STACKS_STRICT_MODE | CET_DYNAMIC_APIS_OUT_OF_PROC_ONLY_ALWAYS_ON` (plus set-context IP validation, which is on by default). Under strict mode, enforced-range registration is unnecessary. If a renderer ever has to run in compatibility mode, the broker registers each JIT code **reservation** once, at about 13 µs per renderer. In that mode, unregistered JIT code is unprotected. `izcal://sandbox` should report the mode (strict, or compatibility with registered code).

**Spec text to amend** (for the spec owner):
- JS 6.4's "Stack-segment switch" row: "Each segment has its own shadow stack" should become R2-17's "each stack has one shadow stack, shared by its chained segments".
- JS 6.6: the Windows paragraph should give the answers above in place of "what must be verified". The fallback sentence ("If (1) and (2) both fail …") should name strict mode as the Windows default and drop the claim that unregistered code under compatibility mode is the remedy.
- JS 6.6: the "Rust runtime code on segments" sentence should add the guard-page and bounds requirements.

## 6. Limits

- One machine, one build (26300.9711), AMD Zen 5. Intel CET hardware and other builds are untested. IBT and forward-edge checks were not examined, since Zen 5 has no IBT.
- Renderer children ran at High integrity, outside an AppContainer. A sandboxed renderer under the repo's AppContainer launcher was not tried; the mitigation bits are per process, so the same behaviour is expected (Medium).
- Each of these relies on behaviour Microsoft does not document, and a startup self-test should guard it: the token's position in a fresh fiber, user-mode `RSTORSSP` onto a fiber's shadow stack, and writing NT_TIB `StackBase` and `StackLimit` (the documented fields; `DeallocationStack` was left alone).
- `SetProcessDynamicEHContinuationTargets` was only checked for availability and for out-of-process enforcement. Izcal unwinds by returns (JS-D19) and does not resume SEH into JIT code, so it registers no continuation targets.
- The `CetDynamicApisOutOfProcOnly` default on this build was observed, not documented. The broker should set it explicitly.

## 7. Files

- [`probes/windows-cet/src/main.rs`](probes/windows-cet/src/main.rs): launcher (`launch <policy>`) and probes (`policy`, `upgrade`, `fiber`, `fibersize`, `fiberbench`, `own`, `forge`, `wrss`, `ssprotect`, `seg`, `jit`, `broker`, `regbench`).
- [`probes/windows-cet/src/asm.rs`](probes/windows-cet/src/asm.rs): `cet_rdssp`, `seg_call`, `ss_switch`, `ss_start`, and the forged-token and WRSS probes.
- [`probes/windows-cet/src/sys.rs`](probes/windows-cet/src/sys.rs): bindings, region scanning, timing.
- [`run-functional.cmd`](probes/windows-cet/run-functional.cmd) and [`run-timing.cmd`](probes/windows-cet/run-timing.cmd): the exact commands. `.cargo/config.toml` links `/CETCOMPAT`; the baseline image is built without it (see the script's header).
- [`logs/`](probes/windows-cet/logs/): the raw output of the functional run and the two timing runs.
