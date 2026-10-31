# JS-M0 design review: the code table and the assumption ledger

Compiled 31 October 2026 for the JS-M0 design review (part 2), against the [JS engine spec](../../reports/Izcal%20JavaScript%20engine.md) draft 0.4.4. Index and decisions: [part-2-index.md](part-2-index.md). Related: [optimizer-ir.md](optimizer-ir.md) (guards and dependencies), [frames-deopt-stackmaps.md](frames-deopt-stackmaps.md) (status slots, deopt), [riscv64-abi.md](riscv64-abi.md) (landing pads).

Confidence: **High** = primary source read and the claim checked in its text; **Medium** = recalled from a primary source or read from an abstract; **Low** = secondary only. † marks a threshold set here from first principles.

**M-1 statement.** Published prose, papers and specifications only; no third-party source code. The design is argued from Izcal's own invariants (H-1, H-2, JS-D15 to JS-D19).

## 1. What this note fixes

JS-M0's exit row asks the design review to fix "the code-table entry format and the assumption-ledger format" ([JS 9.2](../../reports/Izcal%20JavaScript%20engine.md)). The spec already decides:
- a function object in the cage stores a **code-table index**; each entry holds the kind, the parameter count, the owner's trusted function ID, a generation and the entry address; a JS call adapts arguments to the entry's parameter count and jumps to a landing pad ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md) item 4);
- every speculation is a guard or a registration on a fact cell; fact-cell registrations live outside the cage, and invalidation is driven by the runtime's own record ([JS 6.2](../../reports/Izcal%20JavaScript%20engine.md));
- each compilation records every fact it assumed and is discarded if one is uncovered ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md));
- code installation is a recorded decision point (JS-D15).

This note fixes the formats, the call sequence, how assumptions are registered, invalidated and discarded, and one structural change to the trusted region that the formats force.

## 2. History and evidence

| Source | What it shows | What Izcal takes |
|---|---|---|
| P0's root-cause analyses: CVE-2020-16009 and CVE-2021-30632 ([P0 RCA list](https://googleprojectzero.github.io/0days-in-the-wild/rca.html)) | Missing invalidation of compiled code was exploited twice | A checker per compilation, not reviewer vigilance (High via [JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) |
| [Flückiger et al., POPL 2018](https://doi.org/10.1145/3158137) | Speculation as explicit `assume` instructions with deopt targets can be proven correct | Every assumption is an IR object with a coverage proof obligation (Medium) |
| [Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/) | Exits are counted and code recompiled with exponential back-off | Deopt history per function, back-off before recompiling (High) |
| [V8 sandbox](https://v8.dev/blog/sandbox) | An in-region attacker with arbitrary read and write is the premise | H-2: nothing outside the cage trusts what it reads from the cage ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)) (High) |
| [JS 6.7](../../reports/Izcal%20JavaScript%20engine.md), JS-43 | Per-thread write permission exists only with `MAP_JIT` or protection keys; elsewhere a writable alias exists while one compilation installs; a real-time worklet thread makes **0** permission changes | Section 3 |

## 3. Two classes of trusted data (R2-18)

The spec puts bytecode, Wasm bodies, IC programs, the IC site table, frame states, stack maps and code-table and buffer-table entries in one trusted region, "written only by the verifier, the compilers and the runtime's IC-attach path, through a write window" ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)). Three facts make that one region two:

1. **The IC-attach path runs on the script thread**, synchronously, thousands of times per page load ([JS 6.2](../../reports/Izcal%20JavaScript%20engine.md)). So do invalidation, tier installation and buffer-table updates (an `ArrayBuffer` allocation or detach).
2. **Without per-thread write permission** (riscv64 always; x86-64 without PKU) the window is a second writable mapping that exists "only while the code writer works" ([JS 6.7](../../reports/Izcal%20JavaScript%20engine.md), [JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)). Creating and removing a mapping per IC attach costs system calls and TLB shootdowns on the hottest runtime path. (First principles; not measured.)
3. **JS-43** requires 0 "code-memory writes, `mprotect` or permission changes" on a real-time worklet thread, yet that thread attaches ICs like any script thread ([JS 8.4](../../reports/Izcal%20JavaScript%20engine.md)). With `MAP_JIT` or protection keys, opening the window is itself a per-thread permission change.

**Decision R2-18: the trusted region has two classes.**

| Class | Holds | Written by | Protection |
|---|---|---|---|
| **Sealed** | Verified bytecode and constant pools; validated Wasm bodies and side tables (with W0's reference maps); trusted function records (register count, parameter count, canonical frame charge, script ID); code-object metadata (frame-state tables, stack maps, pc maps, OSR entry lists, the validator certificate's digest, `max_inlined`); stencil patch data | The code-writer thread only, through `CodeMemory`'s window, once per compilation or verification | Read-only pages except during an install; never addressed from a cage value without a bound held outside the cage (G17) |
| **Runtime-trusted** | The JS and Wasm code tables (Section 4); the IC site tables and the IC program arena; fact records and their registrations (Section 6); code-object status (invalidated flag, deopt counts, back-off); the buffer table; the external table | The owning agent's script thread, in Rust runtime code only; the code writer never | Ordinary read-write pages **outside the cage**: unreachable by any cage offset (H-1), and indexed from cage values only after a mask or bounds check (H-2, G17) |

**Why this keeps the security argument.** The window protects against a write primitive **outside** the cage. The threat model's in-cage attacker (ATK-2, as [JS 6.1](../../reports/Izcal%20JavaScript%20engine.md) restates it) does not have one; the cage, H-1 and H-2 are what stop it, and they apply to both classes. What the sealed class adds, for data written once per compilation, is defence in depth against a bug in Rust code that writes outside the cage. That is cheap where writes are rare and expensive where they are hot, so the class follows the write rate.

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Sealed plus runtime-trusted** | No window on the script thread or the worklet thread; IC attach costs a store; JS-43 holds | IC programs and code-table entries lose page protection against out-of-cage write primitives | **Recommended** |
| (b) One region, window on every write (the spec's text) | Uniform protection | A mapping change per IC attach without PKU; violates JS-43 on worklet threads | Rejected |
| (c) One region, window only where per-thread permission exists, RW elsewhere | Protection where cheap | Two security models per ISA, against JS-D22's "same design on every ISA" | Rejected |

**Cost.** Out-of-cage write primitives (outside ATK-2) could redirect a code-table entry to another landing pad. Landing pads (and on riscv64 their kind labels) still bound the targets. A second cost is closer to home: a missed H-2 clamp in Rust runtime code, which ATK-2 can reach, now writes memory that holds code pointers, and IC attach, the busiest runtime code that reads the cage, works beside them. Three rules contain it:
- the code tables get their own mapping, with guard gaps, apart from the IC arena and the other runtime-trusted data;
- they are written by one inventoried module (`hg-heap`'s code-table writer), whose every write takes its index from a trusted record, never from the cage, which G17 checks;
- JS-10 gains planted bugs that try to write a code-table entry through IC attach and through an unclamped cage index, and the in-cage corruption fuzzer watches the tables' pages.

**Reversal trigger.** Any of: a published or found exploit that used an out-of-cage write to a code pointer table in a JIT with W^X and landing pads; a JS-10 planted bug, fuzzer finding or red-team result that reaches a code-table entry from the cage. Then the code tables (only) move to the sealed class, written by the code writer at decision points, and invalidation switches to a prologue check of the code-object status word (one load and branch per optimized entry). That reversal has its own cost: a real-time worklet agent's installs would again need a window change, so worklet agents would take code-table updates only from the code writer's thread, at quantum boundaries. R2-18 changes JS-D16's text; it is owner question OQ-1 in the index.

## 4. The code tables (R2-19, R2-20)

**Decision R2-19: two code tables per agent heap, one per calling convention: the JS code table and the Wasm code table. Each is a power-of-two reservation; a cage index is masked into it, never compared; every unused entry is the table's trap entry; the caller passes the entry's address to the callee in a fixed register, so nothing after the call re-reads the index from the cage.**

- **Why per convention.** On x86-64 (`endbr64`) and AArch64 (`bti c`), landing pads carry no label, so a corrupted index that reaches a valid pad of another convention would call it with the wrong argument layout. With one table per convention, a JS function object can only ever reach JS-convention entries (JS functions, builtins, JS-to-Wasm wrappers), and a Wasm `funcref` only Wasm-convention entries. On riscv64 the per-kind labels add a hardware check on top ([riscv64-abi.md](riscv64-abi.md)).
- **Why masked.** `index & (2^k − 1)` is one instruction with no branch, and every value lands in the table. Unused entries point at a trap pad, so a forged index faults or calls an existing function with a correctly shaped frame: confusion inside the cage only ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md) item 4).
- **Size.** `k = 20†`: 1,048,576 entries × 32 bytes = 32 MiB of reservation per table, 64 MiB per agent, committed on use. A process with 16 agents reserves 1 GiB, which fits Sv39 beside the 16 GB cage ([RV 2.3](../Izcal%20JavaScript%20engine/riscv64.md)); Q-R2-8 checks the entry count.
- **Per agent, not per process.** Workers share the cage (JS-D17) but not code tables. With one table per process, a forged index could run agent B's compiled code on agent A's thread with A's context, and A's runtime would then write B's runtime-trusted IC tables while B's thread uses them: a data race in Rust outside the cage, not confusion inside it. JS-D17's same-site argument covers confidentiality, not that. The table base comes from the agent context, so per-agent tables cost no instruction. In JS-D20's reduced-isolation mode each site's agents have their own cage and trusted region anyway.
- **The entry register.** The trampoline, the lazy-compile stub and every callee prologue that needs its entry (to read `param_count` or `code_object`) use the entry address the caller passed, never the cage. A callee object read from the cage twice could give two different indices under corruption, one for the caller's adaptation and one for the callee's frame; passing `e` makes both use the same trusted entry.
- **Not in any table:** runtime trampolines and stubs (called directly by known address) and the segment-switch stub. No cage value can name them.

**Decision R2-20: the entry is 32 bytes.**

| Offset | Field | Type | Meaning |
|---|---|---|---|
| 0 | `entry` | u64 | Address of the current entry point, which starts with the landing pad of this table's call kind: T2, T1, the T0 trampoline, a lazy-compile stub, or the trap pad |
| 8 | `owner` | u32 | The trusted function ID (JS) or process-wide Wasm function ID: indexes the sealed function record |
| 12 | `generation` | u32 | Incremented each time the index is freed and reused; trusted-side references (IC programs, registrations, code objects) store `(index, generation)` |
| 16 | `code_object` | u32 | The current code object, or 0 for the interpreter; used by the stack walker, invalidation and the debugger |
| 20 | `sig` | u32 | Wasm: canonical type ID, checked by `call_indirect`; JS: 0 |
| 24 | `param_count` | u16 | JS: the count the caller adapts to; Wasm: parameter count (the signature gives types) |
| 26 | `kind` | u8 | JS table: `JsFunction`, `JsBuiltin` (bound functions, Proxy `[[Call]]` and `[[Construct]]`, and host callables are builtins whose code reads their target from the object and re-enters through the table), `JsToWasm`, `Trap`. Wasm table: `WasmFunction`, `WasmImport` (a Wasm-to-JS or host wrapper), `Trap` |
| 27 | `tier` | u8 | T0, T1, T2, W0, W1, W2, builtin |
| 28 | `flags` | u16 | Class constructor (plain `[[Call]]` throws), constructible (has `[[Construct]]`; `new.target` is passed in its register), sloppy receiver conversion, has OSR entries (listed in the code object), debugger tier-down pinned |
| 30 | `agent` | u16 | Owning agent heap, for invalidation and debug cross-checks |

The fields the spec names (kind, parameter count, owner, generation, entry) are all here; `sig`, `code_object`, `tier`, `flags` and `agent` are added. **The cage stores only the index.** A generation stored beside it in the cage would add nothing: the threat model's attacker reads process memory ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md), ATK-2) and could forge both.

**Lifetime.** An entry is allocated when a function's trusted record is created (closures of one function share it) and freed through the release list when the sweeper finds the record dead ([JS 6.5](../../reports/Izcal%20JavaScript%20engine.md)): `generation` increments and `entry` is reset to the trap pad, as integer work at an event-loop point.

**Updates.** `entry`, `code_object` and `tier` change only on the owning agent's script thread, at the safepoint that applies an install or a flush (JS-D15), or synchronously during invalidation (Section 6). `entry` is written with one aligned 64-bit store, after the code object's metadata is published. On riscv64 the applying thread executes `fence.i` at that safepoint before the store ([JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)).

## 5. Calls through the table (R2-21)

**Decision R2-21: the caller adapts arguments to `param_count` and cleans up; the callee never reads a count from the cage.**

A generic JS call (callee unknown at compile time), in pseudo-assembly; [riscv64-abi.md](riscv64-abi.md) gives the registers:

```
idx    = load32 [cage + 4·callee − 4 + FUNCTION_CODE_INDEX]   ; cage data: untrusted
idx    = idx & (2^20 − 1)                                     ; R2-19
e      = js_code_table + (idx << 5)                           ; table base from the agent context
pc     = load16 [e + 24]                                      ; param_count, trusted
reserve max(argc, pc) argument slots; write receiver and args; fill [argc, pc) with undefined
argc_reg = argc; callee_reg = callee; entry_reg = e                ; R2-19: the callee uses e, never the cage
(riscv64 only: lui x7, JS_CALL_LABEL)
call  [e + 0]                                                 ; lands on the JS-call landing pad
check [R − 4] == 0, else status slow path                     ; frames note, R2-9 and R2-11
release the argument area
```

- **Semantic checks stay semantic.** Whether the callee is callable at all, or a class constructor called without `new`, is decided from the shape (cage data) and from `flags` (trusted). A corrupted shape gives a wrong answer, never an out-of-cage access.
- **Known callees.** When T2 has guarded the callee's identity (`CheckValue`) or holds a dependency on it, the entry index and `param_count` are compile-time constants, but the call still goes **through the entry** (load `entry` at the constant index, indirect call, landing pad): a direct call to a code address would keep entering the callee's code after it is invalidated, and after a discard would jump into freed memory. A code object keeps alive every code-table entry, trusted function record and heap constant it embeds (below), so a constant index can never be freed and reused under it with a different `param_count`.
- **Wasm `call_indirect`**: load the table element, mask into the Wasm code table, compare `sig` with the expected canonical type ID (an immediate) and trap on mismatch ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md) item 4), then call with the Wasm-call landing pad.
- **Wasm `return_call_indirect`**: the same checks, then the epilogue, then an indirect jump that lands on a call-kind pad ([riscv64-abi.md](riscv64-abi.md) gives the per-ISA rule).

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Caller adapts** | The callee's parameter addressing is a fixed offset; the caller already knows `argc` | The caller reads `param_count` (one load) for unknown callees | **Recommended** |
| (b) Callee adapts (an adaptor frame) | Caller simpler | The arguments live in the caller's area, so the callee cannot grow it; needs an extra frame and moves | Rejected |

**Heap constants.** Compiled code embeds heap objects: `CheckValue` constants, known function objects, `PropertyConstant` values, frame-state `Const` entries. The copying nursery and evacuation move objects ([JS 6.5](../../reports/Izcal%20JavaScript%20engine.md)), and code and sealed metadata are written once. So every heap constant lives in the code object's **constant table** in the runtime-trusted class: one compressed reference per slot, traced as a strong root while the code object lives and updated when its object moves. Generated code loads a constant from the table (its address is fixed for the code object's life and embedded in the code; the validator gives it a "constant table" role); frame states name table slots. Strong references keep their objects alive as long as the code lives; a weak variant that invalidates the code when its object dies is a later optimization, measured first. The GC-stress mode, which moves every object at every safepoint, covers it ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)). Image objects (static roots) need no slot.

**Cost.** One load, one mask, a shift and an add before each generic call; one load before each known call; one load per heap constant. Measured with JS-D16's spike (Q-JS-16) and published in JS-27. **Reversal trigger** for R2-19 to R2-21: if generic and known calls through the table cost more than R2T-8's 1%† of JetStream 3, known callees may call directly under a `CalleeCodeCurrent(index)` dependency whose invalidation fires on every install and discard of that index; the tables themselves stay.

## 6. The assumption ledger (R2-22, R2-23)

### 6.1 What an assumption is

An **assumption** is a predicate over heap state that compiled code relies on at some instruction without checking it there. There are two kinds:
- **guarded**: checked by an earlier guard whose refined value the instruction uses ([optimizer-ir.md](optimizer-ir.md), R2-2);
- **dependent**: established by a `Depend(fact)` and kept true by invalidation.

**Facts** are the assumptions that may be dependent. Each fact has a **fact cell** in the cage (the spec's object, which code and ICs name) and a **fact record** in the runtime-trusted class.

| Fact kind | Predicate | Invalidated by |
|---|---|---|
| `PropertyAbsent(chain, key)` | No object on a prototype chain has `key` | Adding `key` to any object of the chain; changing a prototype on the chain |
| `PropertyAt(holder, key, slot, representation)` | `key` is a data property at a slot with a representation | Deleting or redefining it; a representation change |
| `PropertyConstant(holder, key, value)` | The property's value has never changed since the fact was created | Any write to it |
| `ShapeStable(shape)` | No transition will leave `shape` | Creating a transition from it |
| `GlobalCell(realm, name, state)` | A global binding is a constant, or keeps a representation | A write that breaks the state |
| `IntrinsicUnmodified(realm, which)` | A realm's intrinsic (for example the array iterator's `next`) is the image's | The copy-on-write of that intrinsic ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)) |
| `NoElementsOnPrototypes(realm)` | `Array.prototype` and `Object.prototype` have no indexed elements | Adding one |
| `NotDetached(buffer)` | A buffer-table entry is not detached | Detach, transfer, resize below an access |

Names are Izcal's. New kinds are added by a `.hgop` change, because the operation facts list which kinds each operation may invalidate ([optimizer-ir.md](optimizer-ir.md), Section 9).

### 6.2 Formats (R2-22)

**Decision R2-22: the ledger is a per-compilation table checked before code emission and kept, in compact form, with the code object.**

```
FactRecord {                       // runtime-trusted class, indexed by fact ID
  kind:          FactKind
  generation:    u32               // incremented on every invalidation
  state:         Valid | Invalid
  subject:       cage offset or image offset of the fact cell's subject
  registrations: [(code_object_id, code_generation)]
}

LedgerEntry {                      // per compilation
  id:          u32
  assumption:  Dependent { fact_id, generation_seen } | Guarded { guard_inst, predicate }
  coverage:    Depend(inst) | Guard(inst)
  users:       [inst]              // instructions that rely on it
  reason:      DeoptReasonId       // anchor and plain-language text for JS-60
}

CodeObjectLedger {                 // sealed class, with the code object
  dependencies: [(fact_id, generation_seen)]
  deopt_reasons: [DeoptReasonId]   // indexed by deopt point
  rules_fired:   [RuleId]          // optimizer-ir.md R2-7
}
```

- **Registration during compilation.** The graph builder records an entry each time it relies on a fact: transpiling an IC program that depended on a fact cell, inlining a known callee, using a constant global, eliding a hole check. Every instruction created by speculation carries a `relies_on` list of ledger IDs.
- **The ledger checker**, run after the last IR pass and before emission:
  1. every `relies_on` entry exists and is covered;
  2. a **guarded** entry's guard dominates each user, the user consumes the guard's refined value, and, for a heap fact, no instruction that may invalidate the fact lies on any path from the guard to the user (the IR verifier's rules V1 and V10, [optimizer-ir.md](optimizer-ir.md) 6.1);
  3. a **dependent** entry's `Depend` dominates each user, and every instruction on any path from the `Depend` to a user whose effects intersect the fact kind's invalidating effects is a status-checked lazy-deopt point ([optimizer-ir.md](optimizer-ir.md), R2-3);
  4. no user lies in a region (an inlined frame, a loop) whose frame states cannot resume after invalidation.
  Any failure discards the compilation and records an event with the entry's reason; it is never "fixed up".
- **The background compiler** reads cage data only through `hg-heap`'s concurrent read API and treats everything it reads as a hint; every fact it relied on is re-validated at installation (6.3), which is what makes the hints safe. It never reads the script thread's mutable runtime-trusted data: when a compilation is requested, the script thread snapshots the fact generations it may need and the IC programs of the function's sites into the job. IC programs are immutable once published, and the IC arena frees them by epoch, after every compile job that may hold them has finished.
- **No facts over shared objects.** A `SharedArrayBuffer`, a future shared struct or a Wasm shared type can change from another thread at any time, so no fact kind may have one as its subject; speculation on them is guards only. For the same reason the buffer table has two kinds of entry: an unshared buffer's entry has one writer (its agent's script thread); a growable shared buffer's length is written with atomics by any agent of its cluster and only ever grows, so a stale read is always in bounds.

### 6.3 Installation, invalidation and discard (R2-23)

**Decision R2-23: installation re-validates every dependency on the script thread at the decision point; invalidation is synchronous and semantic; discarding code is deferred to a decision point at which no stack holds it.**

| Step | Where and when | What happens |
|---|---|---|
| **Install** | At the first safepoint after the compilation is ready, on the owning script thread (a JS-D15 decision point, recorded when recording) | For each `(fact_id, generation_seen)`: if the record's generation differs or the state is `Invalid`, the install is rejected and the event recorded. Otherwise append `(code_object, generation)` to each record's registrations, publish the metadata, then store the code-table `entry` (Section 4). No other thread touches fact records, so nothing can invalidate between check and registration |
| **Invalidate** | Synchronously, inside the runtime call that changes the fact (a store to a prototype, a detach, an intrinsic's copy-on-write), on the script thread | `generation += 1`, `state = Invalid`; for each registration whose code generation still matches: mark the code object invalidated; reset every code-table entry that points at it to the next lower tier; walk the agent's stacks and set status 2 in every frame running it ([frames-deopt-stackmaps.md](frames-deopt-stackmaps.md), R2-11); record the JS-60 event (fact, reason, source position). Registrations are cleared |
| **Deopt** | When each marked frame's pending call returns | Lazy deopt to T0 at the call's after-call frame state |
| **Discard** | At the end of the next canonical GC cycle, a freeze trim or navigation (decision points, JS-D15 and [JS 6.5](../../reports/Izcal%20JavaScript%20engine.md)) | For each invalidated code object that no stack of the agent holds (checked by the same walk, on frame identities **and** on return addresses, so a frame whose return address points into the code also holds it) and no code-table entry names: free its code through the code writer, free its sealed metadata, bump the code-object ID's generation |
| **Recompile** | When counters next cross the tier-up threshold | The trusted function record keeps a deopt history: the failing speculation's site is marked "do not speculate" and the threshold is multiplied by 2^k after the k-th invalidation or repeated exit ([Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/)). Feedback in the cage may also be marked, as a hint only |

**Determinism.** Invalidation happens at semantic points, so it is a function of the program's inputs. Installation happens at a recorded decision point and its acceptance is a function of the heap at that point, so replay reproduces both ([JS 6.9](../../reports/Izcal%20JavaScript%20engine.md)). Discards change memory, never results.

**What corruption can do.** The decision to invalidate is taken by runtime code that reads cage data (for example a shape's "has registered facts" flag). An attacker who corrupts that flag can suppress an invalidation, so compiled code keeps a stale assumption. Every access compiled code makes is still bounded by the cage base and a 32-bit offset, a masked table index or the buffer table, and the machine-code validator checks that on every function ([JS 6.7](../../reports/Izcal%20JavaScript%20engine.md)). So a stale assumption gives wrong answers inside the cage, never an access outside it. The ledger is the correctness mechanism; the cage is the safety net. The in-cage corruption fuzzer exercises exactly this (JS-10 (c)).

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Re-validate at install on the script thread; synchronous invalidation; deferred discard** | No locks; no race between a background compile and an invalidation; deterministic | Wasted background compilations when a fact changes during compilation | **Recommended** |
| (b) Register dependencies when compilation starts | Fewer wasted compilations | Registrations by a background thread race with the script thread; code that may never install holds registrations | Rejected |
| (c) A global epoch that invalidates all code on any fact change | Trivial bookkeeping | Throws away all optimized code on common prototype edits; the spec's 6.2a lists it as the alternative to fact cells ("one global epoch"), to be decided by measurement | Rejected |

## 7. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| V8 (CVE-2020-16009, CVE-2021-30632) | Compiled code kept relying on facts that had changed | Dependency coverage must be checked per compilation | R2-22's checker; JS-10's "missing dependency" planted bug |
| V8 sandbox | The design assumes an attacker with arbitrary read and write inside the sandbox ([V8 sandbox](https://v8.dev/blog/sandbox)); the spec records that bytecode and code metadata later had to move to a trusted space ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)) | Decide what is trusted before the first line | R2-18's classes are fixed at JS-M0 |
| JSC | Recompiles after repeated exits with back-off ([Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/)) | Unstable speculations must die quickly | R2-23's deopt history |

## 8. Targets proposed by this note

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| R2T-6 | The ledger checker catches every uncovered assumption | JS-10's "missing dependency" class plus new "invalidating instruction without a status check" and "stale install" classes, ≥ 20† variants each: 100% caught | JS-10 harness | Ship blocker for T2 |
| R2T-7 | Invalidation is quick | p99 time from a fact change to the end of its invalidation (stack walks included) ≤ 50 µs† on the corpus | Lab, JS-M2 | Each stack keeps a summary of the code objects its frames run, updated on suspend, so the walk skips stacks that hold none of the invalidated code; with thousands of suspended JSPI stacks, the walk visits only those |
| R2T-8 | Calls through the table cost little | Generic-call overhead against direct calls, published with JS-27; ≤ 1%† of JetStream 3 time | Lab, JS-M2 | JS-D16's reversal trigger is evaluated |

## 9. Open questions and flagged claims

- **Q-R2-7.** The cost of an alias mapping per write on riscv64 and on x86-64 without PKU is first-principles in Section 3 (system calls plus TLB shootdowns), not measured. JS-M0's JS-D16 spike can measure it with the K3 board and hueyj; R2-18 does not depend on the number, only on JS-43 and the write rate.
- **Q-R2-8.** Whether the code tables' 2^20† entries per agent are enough for the largest pages (functions created) is unmeasured; JS-M1 counts functions per page on the corpus.

**Flagged claims.** That IC attaches happen "thousands of times per page load" is a first-principles estimate, consistent with the absence of a public IC-state distribution (Q-JS-9).
