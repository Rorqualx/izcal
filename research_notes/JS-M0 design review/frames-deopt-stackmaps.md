# JS-M0 design review: frames, deoptimization and stack maps

Compiled 31 October 2026 for the JS-M0 design review (part 2), against the [JS engine spec](../../reports/Izcal%20JavaScript%20engine.md) draft 0.4.4. Index and decisions: [part-2-index.md](part-2-index.md). The IR these frames serve: [optimizer-ir.md](optimizer-ir.md). Register names per ISA: [riscv64-abi.md](riscv64-abi.md).

Confidence: **High** = primary source read and the claim checked in its text; **Medium** = recalled from a primary source or read from an abstract; **Low** = secondary only. † marks a threshold set here from first principles.

**M-1 statement.** Specifications (the ISA manuals' prose, the Linux kernel documentation, Microsoft's and Apple's ABI documentation), papers and engine teams' published prose only. No third-party source code was opened or cited.

## 1. What this note fixes

From JS-M0's exit row ([JS 9.2](../../reports/Izcal%20JavaScript%20engine.md)): **frame states, the deopt model (JS-D19), stack maps**, and from 6.9's stack-overflow row the **canonical frame budget** on **stack segments**. Concretely:

1. one frame layout per tier, the same on x86-64, AArch64 and riscv64 up to register names;
2. how a frame reports "exception pending" and "invalidated" (JS-D19's status word);
3. the deopt model: eager, lazy, with inlining, OSR entry and exit, and how execution resumes **without writing a return address**;
4. the frame-state format;
5. the stack-map format and the collector's stack walk;
6. stack segments, shadow stacks per stack, and the canonical frame budget.

## 2. History and the constraints that bind

| Source | What it established | What Izcal takes |
|---|---|---|
| [Hölzle, Chambers and Ungar, PLDI 1992](https://doi.org/10.1145/143095.143114) | Deoptimization rebuilds unoptimized frames from optimized ones | Frame states are a first-class product of compilation (Medium) |
| [Diwan, Moss and Hudson, PLDI 1992](https://doi.org/10.1145/143095.143140) | Stack maps at GC points; derived pointers need extra tables | Forbid derived pointers instead (R2-14) (Medium) |
| [Sparkplug](https://v8.dev/blog/sparkplug) | A baseline that keeps the interpreter's frame layout lets the debugger, profiler and stack walking "continue working … almost unchanged", and "any OSR logic that works for the interpreter will work for Sparkplug" | T0 and T1 share one frame (already [JS 6.2a](../../reports/Izcal%20JavaScript%20engine.md)) (High) |
| [Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/) | Deopt to the interpreter as an OSR exit; a failed speculation costs microseconds | Deopt resumes in T0; its cost does not matter on the fast path (High) |
| [D'Elia and Demetrescu, PLDI 2018](https://doi.org/10.1145/3192366.3192396) | OSR entry and exit are mappings between program points | One frame-state format serves deopt and OSR exit (Medium) |
| [Duboscq et al., PPPJ 2014](https://doi.org/10.1145/2647508.2647521) | Deopt metadata is large enough to be worth an optimization paper | Sharing and compression are designed in (Medium: title and venue only) |
| Shadow stacks: [Linux x86](https://docs.kernel.org/arch/x86/shstk.html), [Linux arm64 GCS](https://docs.kernel.org/arch/arm64/gcs.html), [Linux riscv64 Zicfiss](https://docs.kernel.org/arch/riscv/zicfiss.html) | Returns are checked against a hardware-protected stack; switching needs tokens; "the main program and its signal handlers use the same shadow stack" (x86 doc) | Every return address the engine creates is pushed by a genuine call (JS-D19) (High) |
| JS-D19 ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) | No return address written or signed outside prologues and epilogues | The binding constraint of this note |
| B1 and 6.9 ([JS 6.9](../../reports/Izcal%20JavaScript%20engine.md)) | Stack overflow is decided by a canonical frame budget, whatever the tier | Section 8.3 |

**Technology mismatch.** Published deopt designs from 1992 to the mid-2010s rebuild frames by writing them, return addresses included, then jump. Hardware return-address protection (CET from Tiger Lake, PAC-RET, Arm GCS, Zicfiss) makes that either impossible or a signing gadget ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)). The designs below are built for the hardware, not ported to it.

## 3. Frames (R2-9, R2-10)

**Decision R2-9: every JS and Wasm frame on every tier has the same prologue and epilogue shape and the same 16-byte header directly below its frame record (the saved caller frame pointer, with the return address directly above it). The frame record's address `R` follows each platform ABI: `R = fp` on x86-64 and AArch64, where the frame pointer points at the record; `R = fp − 16` on riscv64, where the psABI's frame pointer points at the canonical frame address, the `sp` on entry, with the return address at `fp − 8` and the previous `fp` at `fp − 16` ([RISC-V psABI](https://github.com/riscv-non-isa/riscv-elf-psabi-doc/releases), "Frame Pointer Convention"). Following each ABI keeps Rust frames, engine frames and frame-pointer profilers on one chain.** Below, offsets are from `R`. Consequences:
- **PAC-RET works across tier changes.** On AArch64 the prologue signs the return address with the stack pointer at entry, which is always `R + 16`. A deopt or OSR that keeps `fp` keeps the signature valid, with no re-signing.
- **The stack walker needs no per-tier unwind rules.** It follows the saved frame pointers, with one ISA constant for `R`.
- **T0, T1, T2, W0, W1 and W2 can replace each other in place**, because what changes is the body below the header, never the header or anything above it.

**Layout** (stack grows down; offsets from the frame record `R`; 4-byte slots hold 32-bit values):

| Offset | Content | All tiers? |
|---|---|---|
| `R + 16 …` | Incoming arguments, in the **caller's** outgoing area: for JS the receiver then `max(argc, param_count)` argument slots (4 bytes each, padded to 16); for Wasm the stack-passed parameters by signature | Yes |
| `R + 8` | Return address (pushed by `call` on x86-64; stored from the link register by the prologue on AArch64 and riscv64) | Yes |
| `R + 0` | Saved caller `fp` | Yes |
| `R − 4` | **Status slot** (u32 of flag bits): bit 0 exception pending, bit 1 invalidated, bit 2 tier-down requested (debugger); 0 is normal | Yes |
| `R − 8` | **Frame identity** (u32): 3-bit kind (T0/T1, T2, W0, W1, W2, entry, exit, stub) and a 29-bit ID: the trusted function ID for T0/T1, the code-object ID for compiled tiers | Yes |
| `R − 12` | JS: actual `argc`; Wasm: instance index | Yes |
| `R − 16` | JS: callee function object (Tagged32); Wasm: reserved | Yes |
| below | **T0/T1 body**: bytecode offset (u32, written at every call and safepoint), feedback reference, accumulator spill, context, then the register file `r0 … r(n−1)` with `n` from the verified bytecode in the sealed trusted region | T0, T1 |
| below | **T2/W1/W2 body**: tagged spill area, raw spill area (4-, 8- and 16-byte slots), the safepoint stubs' register-save area, outgoing argument area | Compiled tiers |
| below | **W0 body**: bytecode offset, operand-stack pointer, locals (8-byte slots, `v128` in two), operand stack | W0 |

Frames are 16-byte aligned on every ISA (the strictest of the three platform ABIs at calls). No tier relies on a red zone: the platforms disagree (System V x86-64 has one, Windows on ARM64 reserves 16 bytes below `sp`, [Microsoft ARM64 conventions](https://learn.microsoft.com/en-us/cpp/build/arm64-windows-abi-conventions)), and one lowering per ISA must not depend on the OS.

**Why arguments live in the caller's area, at `max(argc, param_count)` slots.** The parameter count comes from the callee's trusted code-table entry, never from the cage ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md) item 4). The caller reserves at least that many slots and fills missing ones with `undefined`, so the callee may address parameter `i < param_count` at a fixed offset with no check, and a corrupted function object can call the wrong function only with a correctly shaped frame. The caller cleans up, which suits variadic JS calls ([code-table-and-ledger.md](code-table-and-ledger.md)).

**Decision R2-10: the engine's internal calling conventions (JS-to-JS, Wasm-to-Wasm, JS-to-Wasm wrappers) have no callee-saved allocatable registers.** Only the pinned registers (cage base, sealed-region base, agent context, and in Wasm the instance and memory base) survive calls, and they are chosen among the platform ABI's callee-saved registers so they also survive calls into Rust ([riscv64-abi.md](riscv64-abi.md)).

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **No callee-saved allocatable registers** | A frame state never refers to a value held in a register by a callee; deopt, the GC walk and the debugger read every value from the frame that owns it; stack maps at calls name only slots | More spills around calls in T2 | **Recommended** |
| (b) A callee-saved subset, as platform ABIs have | Fewer spills in call-heavy loops | Every frame state and stack map must be able to say "in callee-saved register r, saved by some callee frame at an offset only it knows"; deopt must unwind callees to find values | Rejected |

**Cost.** Spill and reload traffic around calls in optimized code. **Reversal trigger.** If spill traffic around calls is more than 3%† of T2's executed instructions on JetStream 3 at JS-M2, a callee-saved subset is designed, with its frame-state and stack-map extension.

**Rust frames.** Runtime calls run Rust on the same segment ([JS 6.6](../../reports/Izcal%20JavaScript%20engine.md)). A runtime trampoline (an inventoried stub) builds an **exit frame** with the standard header (kind = exit), records it as the agent's top exit frame, aligns `sp` to the platform ABI and calls Rust. Rust calling into JS goes through an **entry frame** that links to the previous exit frame. The collector's walk jumps from each entry frame to the exit frame below it, so Rust frames are never scanned (their roots are root scopes, [JS 6.5](../../reports/Izcal%20JavaScript%20engine.md)).

## 4. The status slot (R2-11)

JS-D19 makes every call site in optimized code check "one return status word" after the call, for "invalidated" and "exception pending" ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)). The spec places "invalidated" in a per-code-object bit. That needs two checks (the code object's bit and the exception state) or an indirection through the code object, whose table offset does not fit an immediate on AArch64 or riscv64.

**Decision R2-11: the status word is the frame's own status slot at `R − 4`, a set of flag bits; every call site in every tier checks it with one frame-relative load and one branch; three writers set bits in it, and none clears another's.**

| Writer | When | What it writes |
|---|---|---|
| **The callee's exception exit** | A frame leaves with an exception and has no handler | Sets bit 0 in the **caller's** status slot (the caller's `R − 4`) with an OR, then returns normally |
| **Invalidation** ([code-table-and-ledger.md](code-table-and-ledger.md)) | A fact a code object depends on changes | Sets bit 1 in the status slot of every frame running that code object, found by walking every stack of the agent (the running chain and every suspended JSPI stack) |
| **The debugger** | Tier-down for breakpoints and stepping | Sets bit 2, by the same walk |

- The check is the same instruction pair on every ISA: `fp`-relative load with a small immediate, compare with zero, branch to an out-of-line dispatcher.
- The slow path tests bits 1 and 2 **first**: if either is set, the frame lazy-deopts, with the frame-state mode "after call" or, when bit 0 is also set, "after call, exception pending", so T0 rethrows at the call's offset and its own handler table decides. Only a frame whose code is still valid runs its handler or exception exit on bit 0. The slow path clears the bits it consumed. So an invalidation is never lost to a later exception (a frame F calls G, G invalidates F's code and then throws: F still deopts), and a tier-down request is never lost to either.
- T0 and T1 use the same slot: their call handlers check it after every call, so "exception pending" has one representation in every tier.
- Runtime trampolines translate Rust's `Result` into the same slot, so a call from T2 into Rust is checked like any other call.
- Writing another frame's status slot writes data in the stack, never a return address; the machine-code validator gives generated code exactly one store that may target the caller's `R − 4`: the exception exit's OR of bit 0. A planted bug that overwrites instead of ORing joins JS-10.

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Frame status slot** | One load and branch per call on every ISA; exception and invalidation share it; no code-object lookup | Invalidation walks the agent's stacks (rare; bounded by depth) | **Recommended** |
| (b) Per-code-object bit plus an exception register | No stack walk | Two checks per call, or a table load whose offset needs materializing on AArch64 and riscv64 | Rejected |
| (c) Per-agent "something happened" word | One load | Cannot be cleared while other frames on the stack still need to see it | Rejected |

The code object still carries an "invalidated" flag in the runtime-trusted area, and its code-table entries are reset, so no **new** call enters invalidated code ([code-table-and-ledger.md](code-table-and-ledger.md)). The status slot handles the frames already on the stack. This **refines** 6.4's wording; see the index's contradictions list.

**Cost.** A store of zero in every prologue (folded with the header stores); the stack walk at invalidation. Measured by Q-JS-14 with the rest of JS-15's bound.

## 5. Deoptimization (R2-12, R2-13)

**Decision R2-12: deoptimization builds a resume plan as data, then re-enters the interpreter by reusing the optimized frame for the outermost logical frame and entering each inner logical frame by a genuine call; no return address is ever written, signed or forged.**

### 5.1 Eager deopt, with or without inlining

1. **Exit.** A failing guard branches directly (no indirect branch, so no landing pad) to its out-of-line exit, which loads the deopt-point index into a scratch register and branches to the function's shared deopt tail.
2. **Snapshot.** The tail saves every allocatable register into the frame's register-save area, then makes a genuine call to the runtime's deopt entry (a runtime trampoline).
3. **Plan.** The deoptimizer (Rust, in `hg-jit-opt`'s inventoried deopt module) reads the deopt point's frame-state chain (Section 6) from the sealed region and the values from the frame and the save area. Every deopt point is also a safepoint with a stack map, so the frame's tagged values are visible to the collector throughout. It then:
   - materializes virtual objects (escape-analysed allocations) in two phases, allocate all then fill fields, so cycles work; boxes unboxed doubles;
   - writes a **resume plan** into the agent's deopt buffer outside the cage: for each logical frame, outermost first, its function, bytecode offset, resume mode, callee, `argc`, arguments (for inlined frames), context, accumulator and registers. The buffer is a root set while it is live.
   Allocation here may run the collector; nothing has been written to the stack yet, and every value is rooted in the frame or the buffer. This is the place CVE-2022-1364's class lives, so the frame-state verifier and deopt-everywhere mode test exactly this path ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)).
4. **Frame 0 in place.** The runtime returns into the deopt tail. The tail sets `sp = R − size(frame 0)` (sizes may differ; only `sp` moves, and only inside this inventoried stub) and calls a leaf, allocation-free fill routine that writes frame 0's header and body from the plan. `fp`, the saved `fp` and the return address are untouched, so frame 0 returns exactly where the optimized frame would have, with a valid shadow-stack entry and a valid PAC signature.
5. **Inner frames by genuine calls, made from T0's code.** If the plan has one frame, the tail jumps directly to T0's **resume entry** for frame 0. If it has more, frame 0 resumes at a call: the tail **branches** (no call) to T0's **resume-call sequence** for frame 0's call kind. That sequence lives in T0's code and is the `call` instruction of T0's own call handler for that kind, so the return address it pushes points into T0, not into the optimized code. The callee it calls is T0's **resume-frame entry**, which runs a standard prologue (landing pad, shadow-stack push or PAC sign, header, but **no canonical charge**, below), fills frame 1 from the plan, and either branches to the resume-call sequence for frame 1's call kind or jumps into T0 dispatch at the innermost frame's offset.
6. **Returning.** When an inner frame returns, it returns into T0's **call-return continuation** for that call kind: the generated code T0's own handler returns to. There is one continuation per call kind: plain call, construct (with its result check), property getter and setter, and the iteration protocol's steps; part 1's `.hgop` defines them. So a resumed stack is indistinguishable from one T0 built itself, for the stack walker, the debugger and the collector.
7. **Builtins.** A Rust builtin has no bytecode offset to resume at, so T2 **never inlines across a user-code call inside a Rust builtin**. Builtins written in `.hgop` (JS-D14) may be inlined: their descriptions name resume points, and their frame states use a **builtin-continuation** mode that T0 resumes inside the builtin's generated code.
8. **Space and depth.** A T2 frame is never smaller than its outermost function's T0 frame (the compiler pads it), so step 4 never grows frame 0 in place. Inner frames are new frames entered by calls, so their prologues' real-space checks and the growth stub apply as usual. Resume-frame entries skip the canonical charge; the deoptimizer sets the agent's depth counter to the value at the optimized frame's entry plus the sum of the plan's per-frame `canonical` charges (Section 6), which is what T0 would have charged.

Every return address on the stack and on the shadow stack was pushed by a call; every PAC signature was made by a prologue with its own `sp`. The cost is one call per inlined frame on a path that already costs microseconds ([Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/)); JS-D19 already accepts it.

### 5.2 Lazy deopt

A frame whose code was invalidated while it was waiting on a call sees bit 1 after the call returns, and takes the call site's lazy exit: the same tail as 5.1 with the after-call frame state, the call's result as the accumulator, mode "after call". Nothing else differs.

### 5.3 OSR entry (R2-13)

**Decision R2-13: OSR entry happens in place.** At a loop header with a pending OSR request (installed at a decision point, JS-D15), T0 or T1 makes an **indirect jump** to the OSR entry address for that loop, read from the current code object's OSR entry list (sealed metadata, reached through the code-table entry's `code_object`), to the optimized code's OSR entry, which starts with an OSR-kind landing pad ([riscv64-abi.md](riscv64-abi.md)). The OSR entry block:
1. reads every live interpreter register, the accumulator and the context from the current frame into SSA values (the register allocator may place them in registers or below the old body);
2. checks real space for the optimized frame (a frame that cannot move cannot use the growth stub); if it is short, OSR is declined and the loop stays in T0 or T1; otherwise moves `sp` to the optimized frame's size and writes the optimized body and the frame identity;
3. continues in the optimized loop.

The return address, saved `fp` and incoming arguments are untouched, so the shadow stack and PAC stay valid (R2-9). OSR exit is an eager deopt. W0 and W1 enter W2 the same way, with a Wasm frame state mapping locals and the operand stack.

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **In place, landing-pad entry** | No return address touched; one frame | The OSR entry block's parallel move must read the old body before overwriting it | **Recommended** |
| (b) Call the optimized code from the interpreter frame, leaving the interpreter frame below | Simple | The interpreter frame stays on the stack for the loop's life, costs canonical depth twice (6.9's budget would differ by tier) | Rejected |

### 5.4 Exceptions and the debugger

Exceptions unwind by normal returns with bit 0 in the caller's slot (Section 4); no unwinder skips frames, no shadow-stack pointer is adjusted. The debugger's tier-down is a lazy deopt on bit 2 (Section 4), so breakpoints use the same frame states as deopt ([JS 6.10](../../reports/Izcal%20JavaScript%20engine.md)).

## 6. Frame states (R2-15)

**Decision R2-15: a frame state is an entry in the code object's frame-state table in the sealed region; entries form parent chains for inlining; values are described by one location vocabulary shared with stack maps.**

```
FrameStateEntry {
  parent:     u32 | NONE          // the inlining caller's entry
  function:   TrustedFunctionId | WasmFunctionId
  offset:     u32                 // bytecode offset (JS) or Wasm instruction offset
  mode:       AfterCall | AfterCallException | Reexecute | AtEntry | BuiltinContinuation
  canonical:  u32                 // canonical frame charge of this logical frame (Section 8.3)
  values:     [Loc]               // callee, context, accumulator, argc and args (inlined frames only),
                                  // then registers r0..r(n-1); Wasm: locals then operand stack
  virtuals:   [VirtualObject]     // escape-analysed objects referenced by Loc::Virtual
}
Loc = Dead | Const(pool) | Slot(offset, repr) | SavedReg(index, repr) | Virtual(index)
repr = Tagged32 | I32 | F64 | I64 | F32 | V128
VirtualObject { layout: LayoutDescriptorId, fields: [Loc] }
```

- **`Dead`** is any register the bytecode verifier's liveness marks unused at `offset`; it is rebuilt as `undefined`, which T0 never reads, and the debugger shows it as "optimized out".
- **Boxing on deopt.** `I32` outside the small-integer range and `F64` become heap numbers; Wasm frames keep raw `I64` and `F64` in their 8-byte slots.
- **Encoding.** Entries are variable-length (tagged varints); consecutive deopt points of one function share their parent entries and store only the locations that differ from the previous entry of the same function.
- **The frame-state verifier** checks, per compilation: the register count equals the verified bytecode's; each location's representation matches the IR value's; each virtual object's field count matches its layout; `AfterCall` sits on a call-like operation; `Dead` agrees with liveness; the chain's canonical charges sum to the code object's recorded inlined charge (Section 8.3). A failure discards the compilation.
- **Deopt-everywhere mode** (CI and fuzzing) makes every guard fail on its first execution and every Nth† execution after, and compares with the reference interpreter (JS-4).

**Cost.** Metadata bytes. **Target R2T-3** below bounds them. **Reversal trigger.** If frame states and stack maps together exceed 40%† of T2 machine-code bytes on the corpus at JS-M2, the encoding is redesigned (for example, frame states recomputed from the IR at deopt time for cold deopt points) before any feature that adds deopt points.

## 7. Stack maps and the collector's walk (R2-14)

**Decision R2-14: no decompressed address and no derived pointer is live across a safepoint; a stack map is a bitset over the frame's tagged spill area plus, at polls and deopt points only, a bitset over the register-save area.**

**Safepoints** are calls, allocations, loop-back-edge polls and deopt points ([JS 6.5](../../reports/Izcal%20JavaScript%20engine.md) "Optimized frames"). By R2-10 no value is in a register across a call. At a poll, the fast path is a load and a branch; the slow path calls a poll stub that saves every allocatable register into the register-save area, so values may stay in registers across a poll and the map says which saved registers hold references. Raw addresses (decompressed references, `cage base + 4·v − 4`) are rematerialized after every safepoint; decompression is one or two instructions on all three ISAs ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)).

**Format** (per code object, in the sealed region):

| Part | Encoding |
|---|---|
| `pc_offsets` | Sorted u32 array of return-address offsets from the code start (binary-searched) |
| `map_ids` | u16 per safepoint, into the dictionary |
| `maps` | Deduplicated dictionary: each map is (`slots`: a bitset over the tagged spill area, one bit per 4-byte slot; `regs`: a bitset over the register-save area, empty except at polls and deopt points) |
| `frame_state` | u32 per safepoint, or NONE: the deopt point's frame state, if any |

The tagged spill area is contiguous, so maps are short; the register allocator assigns tagged spills there and raw spills elsewhere. T0 and T1 frames need no map: every register-file slot is a tagged 32-bit value ([JS 6.5](../../reports/Izcal%20JavaScript%20engine.md)), and T1 writes every live value into its interpreter frame before any safepoint. W0 frames take their maps from the Wasm function's side table: at each call and poll offset, the validator's type stack says which locals and operand-stack slots hold references ([Wasm validation algorithm](https://webassembly.github.io/spec/core/appendix/algorithm.html)); the side table stores a bitset for those offsets only, in functions that have reference-typed values.

**Outgoing arguments.** A caller's outgoing argument area is part of the caller's frame but is visited by the callee once the call is made. Between the first store into it and the call, nobody would visit it. So the scheduling rule is: **no safepoint between the first outgoing-argument store and the call** (arguments that need boxing or allocation are computed into spill slots first); the allocation checker rejects code that breaks it. Wasm frames visit their reference-typed stack parameters by signature, as JS frames visit theirs by `max(argc, param_count)`.

**The walk.** Run by the owning thread at a safepoint, for each stack of the agent (the running chain, then every suspended JSPI stack):

```
frame, pc = stack.top_exit_frame, stack.top_exit_return_pc
while frame is not the bottom:
    kind, id = frame.identity
    if kind is entry:             frame, pc = frame.previous_exit, frame.previous_exit_pc; continue
    T0/T1:  visit callee, context, feedback, accumulator, r0..r(n-1)    (n from the sealed function record)
            visit receiver and max(argc, param_count) incoming args   (each frame visits its own incoming args)
    T2/W1/W2: map = code_object(id).lookup(pc); visit map.slots, map.regs
              visit incoming args: JS by max(argc, param_count), Wasm by the signature's reference-typed stack parameters
    W0:     visit the side table's bitset at the frame's saved offset, and reference-typed stack parameters
    pc = frame.return_address; frame = frame.saved_fp
```

Every visited slot is an arbitrary 32-bit value to the collector (H-2's discipline, [JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)): tag 1 means a reference, offsets stay in the cage. The stack itself is outside the cage, written only by generated code, the interpreter and inventoried stubs, so frame identities and counts are trusted; debug builds cross-check each frame identity against a lookup of `pc` in the code map.

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Slots plus save-area bits, no derived pointers** | One bitset per safepoint; the walker is a dozen lines; the GC-stress mode that moves every object at every safepoint can check it exhaustively ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) | A rematerialized decompression after each safepoint | **Recommended** |
| (b) Full register maps and derived-pointer tables ([Diwan et al.](https://doi.org/10.1145/143095.143140); LLVM's prose on [statepoints and stack maps](https://llvm.org/docs/StackMaps.html)) | Fewer instructions after safepoints | More metadata and a more complex walker; derived pointers are where moving collectors get subtle | Rejected |

**Reversal trigger.** If rematerialization after safepoints costs more than 1%† of T2 time on JetStream 3 at JS-M2, derived pointers are allowed for loop induction addresses only, with a derived-pointer table checked by the GC-stress mode.

## 8. Stacks, segments and the canonical frame budget (R2-16, R2-17)

### 8.1 Words

The spec uses "segment" both for one guard-paged region and for a JSPI-suspendable stack ([JS 6.6](../../reports/Izcal%20JavaScript%20engine.md)). This note separates them:
- a **segment** is one guard-paged region from the substrate's `Memory` interface;
- a **stack** is what JSPI suspends and Stack Switching switches: a chain of segments plus one shadow stack.

### 8.2 Shadow stacks per stack (R2-17)

**Decision R2-17: one OS-created shadow stack per stack, shared by all its segments; segments are chained by an inventoried growth stub that switches only `sp`, never the shadow stack.**

- **Why per stack.** A return crosses a segment boundary as an ordinary return; it must be checked against the same shadow stack it was pushed on. A shadow stack per segment would need a shadow-stack switch on every segment crossing, which the architectures allow only through token protocols ([RISC-V ISA manual, CFI chapter, the stack-switching note](https://github.com/riscv/riscv-isa-manual/releases); [kernel GCS docs](https://docs.kernel.org/arch/arm64/gcs.html)) that exist for switching between stacks, not inside one.
- **Size.** A shadow stack holds one 8-byte entry per active frame, so its size follows from the canonical budget: `(B / C_min + F_rt) × 8` bytes, where `F_rt = 8,192†` entries cover the runtime frames that run on the stack: 256 KiB plus 64 KiB, so 320 KiB† with the values in Section 8.3, reserved and committed on touch.
- **Growth.** When a prologue's real-space check fails, the function calls the growth stub (a genuine call), which takes a new segment, switches `sp` to it, copies the incoming arguments, builds a link frame (kind = stub) and makes a genuine call to the function's body entry. On return it switches `sp` back, ORs the link frame's status bits into the real caller's slot, and returns. Every return address was pushed by a call, on the one shadow stack. Like the switch stub, it updates the agent's segment bounds and, on Windows, the TEB's stack bounds (Q-R2-5). Segments come from a per-agent pool refilled off-thread, so a real-time worklet thread never maps memory to grow (JS-43). A guard-page fault on a segment is handled on an alternate signal stack (`sigaltstack` on Linux and macOS), and the stubs' instruction ranges are registered so a signal that arrives mid-switch is recognized.
- **Switching stacks** (JSPI suspend and resume, Stack Switching) goes through the substrate's segment-switch stub, the only code that changes the shadow stack ([riscv64-abi.md](riscv64-abi.md)).

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Shadow stack per stack, chained segments** | Returns across segments are plain returns; JSPI stacks start small, which matters on Sv39 ([JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)) | A growth stub and argument copy on the rare growth path | **Recommended** |
| (b) Shadow stack per segment | Literal reading of 6.6 | A token switch per segment crossing in both directions | Rejected |
| (c) One large reservation per stack, no chaining | Simplest | Thousands of suspended JSPI stacks × megabytes of reservation do not fit beside a 16 GB cage and guarded Wasm memories on Sv39's 256 GiB ([RV 2.1](../Izcal%20JavaScript%20engine/riscv64.md)) | Rejected |

### 8.3 The canonical frame budget (R2-16)

**Decision R2-16: stack overflow is decided by a per-agent canonical depth counter, charged per logical frame by a cost fixed at bytecode verification or Wasm validation; optimized code checks once at entry for its whole inlining tree.**

| Element | Definition |
|---|---|
| **Charge of a JS logical frame** | `C(f) = 32 + 4 × (registers(f) + param_count(f)) + 4 × max(0, argc − param_count(f))` canonical bytes; the first two terms are computed by the bytecode verifier and stored with the trusted function record |
| **Charge of a builtin** | Every builtin that T0 calls as a frame and T2 may inline (`call`, `apply`, `Reflect.apply`, bound-function calls, accessors, `.hgop` builtins) has a fixed charge stated in its `.hgop` description, charged the same way in every tier |
| **Charge of a Wasm logical frame** | `C(f) = 32 + 8 × (params + locals + max operand-stack depth)`, from validation |
| **Budget** | `B = 1 MiB†` canonical bytes per agent, the same on every device, ISA and tier; it allows 32,768 frames of the smallest function |
| **Where charged** | T0 and T1: in the callee's prologue, `depth += C(f)`, compare with `B`, store; epilogue subtracts. Inlined frames in T2: carried as an SSA value, stored to the agent before each real call and safepoint. JSPI: suspending subtracts the suspended stack's total; resuming adds it and checks, and if the check fails the resumed stack receives a `RangeError` at its suspension point, unwinding normally |
| **T2's entry check** | `depth + C(f) + max_inlined(f) ≤ B`, where `max_inlined(f)` is the largest sum of charges along any inlined call chain in the code object. If it fails, the function takes an **entry bailout** to T0, which charges frame by frame; an entry bailout is not a deopt and never counts toward back-off, so code near the limit is not penalized. So the `RangeError` happens at exactly the same logical call in every tier, with no check at inlined calls |
| **On overflow** | `RangeError`, thrown at that call, in every tier ([JS 6.9](../../reports/Izcal%20JavaScript%20engine.md)) |
| **Real space** | Separate: each prologue compares `sp − real_frame_size` with the segment limit plus the runtime headroom and calls the growth stub. Compiled code's real frame may not exceed `ρ × C` for its inlining tree, `ρ = 4†`; the compiler declines an inlining that would. A real limit reached before the canonical one is an engine bug and crashes the renderer, never a JS-visible difference |
| **Runtime headroom** | `H = 256 KiB†` of real space reserved below every JS frame for the deepest runtime call; every recursive Rust path reachable from content has its own canonical depth limit ([JS 6.9](../../reports/Izcal%20JavaScript%20engine.md)) |

Layout's recursion uses the same segments under its own budget (LY-D15); nothing here changes it.

**Cost.** A load, add, compare, branch and store per logical call in T0 and T1; a store before real calls in T2; one entry check per T2 call. **Reversal trigger.** If the counter costs more than 1%† of T0 dispatch time on JetStream 3 at JS-M2, T0 charges by function from its call handler (fewer instructions, same values). `B` is re-set by an owner decision if JS-M1's black-box comparison finds real sites that need deeper recursion than the best shipping engine allows (Q-R2-4).

## 9. Per-ISA prologue and epilogue

The shape is R2-9's; the instructions differ. Register roles are in [riscv64-abi.md](riscv64-abi.md).

| Step | x86-64 | AArch64 | riscv64 |
|---|---|---|---|
| Landing pad | `endbr64` (only on indirectly entered code) | `bti c` (call kinds) or `bti j` (jump kinds) | `lpad <label>`, 4-byte aligned ([ISA manual, Zicfilp](https://github.com/riscv/riscv-isa-manual/releases)) |
| Protect the return address | Pushed by `call`, and on the CET shadow stack by hardware | `paciasp` where PAC is enabled; GCS push by `bl` where GCS is enabled | `sspush ra` (a no-op where Zicfiss is absent, [RV 3.2](../Izcal%20JavaScript%20engine/riscv64.md)) |
| Link the frame | `push rbp; mov rbp, rsp` | `stp x29, x30, [sp, #-16]!; mov x29, sp` | `addi sp, sp, -16; sd ra, 8(sp); sd s0, 0(sp); addi s0, sp, 16` (psABI: `fp` is the entry `sp`) |
| Body | `sub rsp, N`; header stores; status slot zero | same | same |
| Epilogue | `mov rsp, rbp; pop rbp; ret` | `mov sp, x29; ldp x29, x30, [sp], #16; autiasp; ret` | `addi sp, s0, -16; ld ra, 8(sp); ld s0, 0(sp); addi sp, sp, 16; sspopchk ra; ret` |

On every ISA, `[R]` is the saved `fp` and `[R + 8]` the return address, with `R = fp` on x86-64 and AArch64 and `R = fp − 16` on riscv64; the signing modifier on AArch64 is `R + 16`. On Windows x64, compiled code is registered for unwinding with `RtlAddGrowableFunctionTable` ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtladdgrowablefunctiontable)) so crash reports can walk it; the engine itself never uses OS unwinding.

## 10. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Self (1992) | Deoptimization made optimized code debuggable | Deopt and debugging are one mechanism | Tier-down for the debugger is lazy deopt (status 3) |
| Sparkplug | Keeping the interpreter's frame let the debugger and stack walking work "almost unchanged" and OSR logic carry over ([V8](https://v8.dev/blog/sparkplug)) | Frame-shape compatibility is worth more than a faster frame | R2-9 extends it to every tier's header and prologue |
| CVE-2022-1364 | Deopt materialization was exploited ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) | The deopt path needs its own verifier and tests | Frame-state verifier, deopt-everywhere, materialization before any stack write |
| V8's CFI plan | Legitimate return-stack changes "during optimization / deoptimization and exception handling" were left to be dealt with ([V8](https://v8.dev/blog/control-flow-integrity)) | Return addresses must never be data the JIT writes | R2-11, R2-12, R2-13, R2-17 |

## 11. Targets proposed by this note

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| R2T-3 | Deopt metadata is small | Frame-state tables plus stack maps ≤ 40%† of T2 machine-code bytes on the corpus | Code-object statistics, lab, JS-M2 | R2-15's reversal |
| R2T-4 | Stack overflow is tier-independent | For a DST set of recursive programs (direct, mutual, through inlinable callees, through JSPI), the `RangeError` occurs at the same logical call in T0, T1 and T2 on all three ISAs: 100% | DST farm, JS-50's set | Ship blocker for the tier that differs |
| R2T-5 | No return address is written | JS-15's probe additionally drives eager deopt with ≥ 3† inlined frames, lazy deopt, OSR entry and exit, segment growth and JSPI switches under CET, GCS, PAC-RET and Zicfiss: 0 faults | JS-15 probe | Ship blocker (JS-15) |
| R2T-9 | Deopt is bounded and correct | Eager deopt with 3† inlined frames completes in p99 ≤ 20 µs† on the reference devices; JS-10 gains "status bit overwritten instead of ORed", "stale refined heap fact after a call" and "outgoing argument live across a safepoint" classes, ≥ 20† variants each, 100% caught | Lab at JS-M2; JS-10 harness | R2-12's design is revisited before T2 ships; planted-bug misses are ship blockers |

## 12. Open questions and flagged claims

- **Q-R2-4.** `B = 1 MiB†` allows 32,768 frames of the smallest function. What recursion depth do the shipping browsers allow, black-box, for a few function shapes, and should Izcal match the deepest? (Measured at JS-M1; the owner decides if Izcal's depth would be lower.)
- **Q-R2-5.** Windows: segments, shadow stacks and Rust's stack probes depend on Q-JS-15's five questions. If user mode cannot create shadow stacks for engine stacks (questions 1 and 2), R2-17's growth stub and the switch stub become fiber operations, and the TEB's stack bounds must follow each switch.
- **Q-R2-6.** Whether PAC-RET is enabled for non-`arm64e` processes on macOS, and whether macOS or Windows on Arm enables BTI or GCS for user processes. Not found in Apple's or Microsoft's published ABI pages this session; a black-box check at JS-M0 on the Mac (Phase 6).

**Flagged claims.** The papers in Section 2 are cited from titles, venues and abstracts (Medium). The red-zone statement for Windows is recalled from Microsoft's x64 convention pages, not re-read this session (Medium). All thresholds marked † are first-principles values.
