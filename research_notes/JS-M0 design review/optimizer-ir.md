# JS-M0 design review: the optimizer IR (T2 and W2)

Compiled 31 October 2026 for the JS-M0 design review (part 2), against the [JS engine spec](../../reports/Izcal%20JavaScript%20engine.md) draft 0.4.4 on `main` (0.4.5's JS-M0 row is pending on `docs/js-m0-amendments`). Method: the first-principles-spec skill. Index and decisions: [part-2-index.md](part-2-index.md).

Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a claim recalled from a primary source and not re-read this session; **Low** = secondary source only. A † marks a threshold set here from first principles; none is measured.

**M-1 statement.** Evidence is papers, published prose (engine teams' blog posts, documentation pages) and specifications. No third-party source code, header, commit, diff or code review was opened or cited. Other engines appear only as published evidence. Every name below is Izcal's own.

## 1. What this note fixes

JS-D6 decided **one SSA CFG optimizer with levels O1 and O2** for T2 and W2 ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)). JS-M0 has to fix the IR itself, because `.hgop`'s operation facts, the frame states, the stack maps, the code table and the assumption ledger all depend on it ([JS 9.1](../../reports/Izcal%20JavaScript%20engine.md), "T2 and W2 are designed at JS-M0"). This note fixes:

1. the control representation;
2. the instruction kinds;
3. effects;
4. the type lattice and representations;
5. how `.hgop` operation facts enter the IR;
6. how speculation, guards and dependencies are written;
7. how verified rules from `hg-rulecheck` attach;
8. how W2 shares the IR;
9. the IR verifier.

Frame states and stack maps are in [frames-deopt-stackmaps.md](frames-deopt-stackmaps.md); the ledger is in [code-table-and-ledger.md](code-table-and-ledger.md).

## 2. History: what came before, and why

| Year | Work | What it did | Why that way | Fate, and the lesson |
|---|---|---|---|---|
| 1992 | Self's dynamic deoptimization ([Hölzle, Chambers and Ungar, PLDI 1992](https://doi.org/10.1145/143095.143114)) | Rebuilt unoptimized frames from optimized ones on demand, first for debugging | Optimized code had to stay debuggable | Survived as the basis of every speculative JIT: deopt metadata is part of the IR's contract, not an afterthought (High: title and venue checked; content Medium) |
| 1992 | Stack maps for a moving collector ([Diwan, Moss and Hudson, PLDI 1992](https://doi.org/10.1145/143095.143140)) | Compiler-emitted tables of live pointers at GC points, including derived pointers | Precise moving GC in a compiled language | The derived-pointer problem it solved is avoided below by a rule (no raw address lives across a safepoint) (Medium) |
| 1995 | Sea of nodes ([Click and Paleczny, IR '95, SIGPLAN Notices 30(3)](https://doi.org/10.1145/202530.202534)) | One graph for data, control and effects; nodes float freely | Aggressive global optimization in a static compiler | V8 left it in 2025: in JS "only pure nodes actually float", effect chains caused subtle bugs, compile time halved after the move ([V8](https://v8.dev/blog/leaving-the-sea-of-nodes), [EX 8.1](../Izcal%20JavaScript%20engine/execution_performance.md)) (High) |
| 2000 | ABCD and e-SSA ([Bodík, Gupta and Sarkar, PLDI 2000](https://doi.org/10.1145/349299.349342)) | Bounds-check elimination on a form where each check introduces a new, refined name for the checked value | The fact a check establishes must flow with the value | The refining-definition idea is the guard design below (Medium) |
| 2013 | An IR for speculation ([Duboscq et al., VMIL 2013](https://doi.org/10.1145/2542142.2542143)) | Guards as nodes anchored to control, frame states as IR values | Speculation needs a place in the graph that optimizations respect | An alternative this note weighs in 6.1 (Medium: abstract) |
| 2013 | SSA construction without dominance frontiers ([Braun et al., CC 2013](https://doi.org/10.1007/978-3-642-37051-9_6)) | Builds SSA directly while translating from a stack or AST | Single-pass front ends | Used by the W2 and T2 graph builders (Medium) |
| 2014 | Smaller deopt metadata ([Duboscq et al., PPPJ 2014](https://doi.org/10.1145/2647508.2647521), "Speculation without regret") | Reduced frame-state metadata in a speculative compiler | Deopt metadata was a large share of compiled-code memory | Frame-state sharing and compression are designed in from the start (Medium: title and venue only) |
| 2018 | Formal semantics of speculation ([Flückiger et al., POPL 2018](https://doi.org/10.1145/3158137), published December 2017) | An IR with explicit `assume` and deopt, with proofs that optimizations preserve meaning | Speculative optimizations had no correctness argument | The `Depend` and guard rules in 6.2 follow its shape: a speculation is an instruction with a frame state, and passes must respect it (Medium: abstract) |
| 2018 | OSR, distilled ([D'Elia and Demetrescu, PLDI 2018](https://doi.org/10.1145/3192366.3192396)) | A formal account of on-stack replacement as a mapping between program points | OSR was ad hoc per VM | The frame-state mapping is the same object for OSR exit and deopt (Medium: abstract) |
| 2020 | JSC's speculation model ([Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/)) | A failed speculation (exit, slowdown and recompilation together) costs about 2.5 µs (DFG) to 10 µs (FTL), so speculation pays only if it almost never fails | Speculation is a bet with a tiny gain and a large loss | Exits are counted per site and recompiled with back-off ([EX 9.1](../Izcal%20JavaScript%20engine/execution_performance.md)) (High) |
| — | Block arguments ([MLIR rationale](https://mlir.llvm.org/docs/Rationale/Rationale/#block-arguments-vs-phi-nodes)) | SSA merges as block parameters instead of phi instructions | Phis must sit at block tops, execute "atomically", and cannot name a value on one edge only | Adopted below (High) |
| 2025 | V8 leaves the sea of nodes ([V8](https://v8.dev/blog/leaving-the-sea-of-nodes)) | CFG IR for the whole JS backend and Wasm pipeline | Compile time, cache misses, effect-chain bugs | Confirms JS-D6 (High) |
| 2026 | Acyclic e-graph ([Fallin](https://cfallin.org/blog/2026/04/09/aegraph/)) | Effectful instructions in a CFG skeleton, pure ones hash-consed with eager rewrites | Equality saturation was too slow; classical passes missed rewrites | About 2% better code for 7–8% more compile time; adopted for pure instructions at O2 ([EX 8.2](../Izcal%20JavaScript%20engine/execution_performance.md)) (High) |

**Technology mismatches.** Sea of nodes assumed that most nodes are pure; in JS they are not. Phi-based SSA assumed merges are rare and small; exception and deopt edges make them common. Most published deopt designs assume the JIT may rewrite return addresses; JS-D19 forbids that, so deopt shapes the IR's call instructions (6.2).

## 3. Inputs this IR must accept

| Input | From | What the IR needs from it |
|---|---|---|
| Verified bytecode | `hg-js-bytecode` (part 1 of the review) | Operand kinds, register liveness at every offset (dead registers are not kept for deopt), the IC slot of each operation |
| Operation facts | `hg-opgen` from `.hgop` (part 1) | Per operation: effects, may-throw, may-call-user-code, allocation and its canonical size, deopt and safepoint points, result representations per fast path, type transfer, generic lowering, SMT encoding ID (Section 9) |
| Recorded IC programs | The IC site table in the trusted region | The guards and loads each site saw, transpiled into IR (JS 6.2) |
| Value profiles | Feedback in the cage | Hints only; every use is re-checked by a guard (H-2) |
| Validated Wasm bodies | `hg-wasm`'s validator | The type of every local and operand-stack slot, canonical type IDs |

## 4. First principles

- **The IR is a contract between five checkers**, not only a compiler's data structure: the IR verifier, the ledger checker, the frame-state verifier, the allocation checker and `hg-rulecheck` ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)'s correctness table). Every choice below is judged first by whether a small checker can state and check it.
- **Most JS operations have effects.** After IC transpilation, property loads, stores and calls dominate; only arithmetic, conversions and comparisons are pure. A representation whose strength is free floating spends its cost where JS gets little back ([V8](https://v8.dev/blog/leaving-the-sea-of-nodes)).
- **Wasm arrives scheduled.** Its code is already in a good order; an IR that destroys the schedule wastes the producer's work (same source).
- **A speculation is a fact plus a way back.** Every speculative use needs (a) the fact, (b) where it is established (guard or dependency) and (c) a frame state to resume from. The IR must make all three explicit, or the ledger checker cannot work ([Flückiger et al.](https://doi.org/10.1145/3158137)).
- **Facts must travel with values.** If a guard proves "v is a small integer", the proof must stop anything that uses it from moving above the guard. Control dependence does that weakly; data dependence does it by construction ([ABCD](https://doi.org/10.1145/349299.349342)).

**Assumptions kept and broken.**

| Inherited assumption | Kept or broken | Why |
|---|---|---|
| SSA | Kept | Every checker above is simpler on SSA |
| Phi instructions | **Broken**: block parameters | Edge-specific values (exception, deopt) and no "atomic phi block" (MLIR rationale) |
| Guards are control-dependent nodes | **Broken**: guards produce refined values | Data dependence stops illegal hoisting by construction |
| Deopt metadata is side data emitted late | **Broken**: frame states are IR objects, checked by a verifier | CVE-2022-1364 was a deopt materialization bug ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) |
| The optimizer knows effects from hand-written tables | **Broken**: effects come only from generated facts | CVE-2020-6418's class ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) |
| Exceptions are a separate unwinder's business | **Broken**: exceptions are status-checked returns with explicit edges | JS-D19 forbids skipping frames |
| Derived (interior) pointers across GC points | **Broken**: no raw address lives across a safepoint | Decompression is one or two instructions ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)); derived-pointer maps are a classic source of GC bugs ([Diwan et al.](https://doi.org/10.1145/143095.143140)) |

## 5. Control representation (R2-1)

**Decision R2-1: a CFG of basic blocks in SSA form, with block parameters instead of phis; effectful instructions are ordered in their block; pure instructions are hash-consed and float, and at O2 they are re-placed by an acyclic e-graph elaboration pass.**

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **CFG, block parameters, floating pure instructions** | Effect order is the block order, so effect checking is linear; Wasm keeps its schedule; edge-specific values are natural; O2 gets GVN, LICM and rematerialization from elaboration ([Fallin](https://cfallin.org/blog/2026/04/09/aegraph/)) | Elaboration costs compile time (7–8% in Cranelift's measurement) | **Recommended** |
| (b) CFG with phi instructions | Familiar | Phis at block tops, "atomic" phi blocks and no per-edge values ([MLIR](https://mlir.llvm.org/docs/Rationale/Rationale/#block-arguments-vs-phi-nodes)); deopt and exception edges make these bite | Rejected |
| (c) Sea of nodes | Maximal motion freedom | V8 measured the opposite of its promise on JS: compile time halved after leaving it, about 3× more L1d misses, effect-chain bugs ([V8](https://v8.dev/blog/leaving-the-sea-of-nodes)) | Rejected (JS-D6) |
| (d) Full equality saturation | Finds rewrite combinations | Extraction with sharing is NP-hard; a round-trip alone cost 23% compile time ([EX 8.2](../Izcal%20JavaScript%20engine/execution_performance.md)) | Rejected |

**Structure.**
- A function is a list of blocks. Block 0's parameters are the function's inputs. Each block ends in exactly one terminator.
- **Edges carry arguments.** A terminator names each successor with the values passed to its parameters, so a value that exists only on the exception edge is simply an argument on that edge.
- **The skeleton.** Effectful instructions (any non-empty effect set, Section 7) stay in program order within their block. Pure instructions are placed by the elaboration pass at O2, and stay where the graph builder put them at O1.
- **Storage.** Instructions, values, blocks and frame states live in dense arrays indexed by 32-bit IDs, with operand lists in one pooled array: the cache-friendly layout the spec asks for ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)). Effect sets are precomputed bitsets per instruction.

**Cost.** Block-parameter bookkeeping in the builder; the elaboration pass at O2.
**Reversal trigger.** If elaboration costs more than 15%† of O2 compile time on the corpus at JS-M2 and gains less than 1%† on JetStream 3, O2 drops elaboration and keeps classical GVN and LICM. JS-D6's own trigger (O1 below 10%† of T1's throughput) is unchanged.

## 6. Instruction kinds

Each instruction has an opcode, operands (value IDs), result values, an effect set (Section 7), an optional frame state (frames note) and a **provenance**: the `.hgop` operation, IC program step, Wasm instruction or rule that created it, with its spec anchor (for JS-60's explanations and translation validation).

| Family | Examples | Effects | Notes |
|---|---|---|---|
| **Constants and inputs** | `Const(kind, bits)`, `StaticRoot(offset)`, `Param(i)` | none | `StaticRoot` names image objects (`undefined`, core shapes) by their fixed cage offset ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)) |
| **Pure arithmetic** | `IAdd32`, `IAdd64`, `FAdd64`, `FMul32`, wrapping and saturating forms, comparisons, `Select` | none | Wasm's wrapping semantics and JS's are separate opcodes, never a flag |
| **Checked arithmetic** | `IAdd32Checked(a, b) → r` | `MayDeopt` or `MayTrap` | "Add with overflow" is one operation, lowered once per ISA (flags on x86-64 and AArch64, an explicit compare on riscv64) ([JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)) |
| **Representation changes** | `TagInt`, `UntagInt`, `BoxF64` (allocates), `UnboxF64`, `ToInt32` (one instruction on riscv64 via `fcvtmod.w.d`), `F64ToF32` | none, except `BoxF64` (`Alloc`) | Inserted by the representation pass (Section 8) |
| **Guards** | `CheckSmi`, `CheckShape(v, shapes)`, `CheckBounds(i, len)`, `CheckNotHole`, `CheckValue(v, const)`, `CheckCodeIndex(f, idx)`, `CheckWasmCast(r, type)` | `MayDeopt` or `MayTrap` | Each produces a **refined value** (6.1) and names a failure action |
| **Dependencies** | `Depend(fact)` | `MayDeopt` (lazy) | Emits no code; records a ledger entry at a program point (6.2) |
| **Heap memory** | `LoadField(obj, slot, cat)`, `StoreField(obj, slot, v, barrier)`, `LoadElement`, `StoreElement`, `LoadRawF64` | `Read(cat)` / `Write(cat)` | `cat` is a heap category (Section 7); stores of tagged values carry an explicit barrier kind, or a verified elision rule's ID |
| **Buffer and linear memory** | `LoadBuffer(bt_index, i)`, `WasmLoad(mem, addr, offset, width, bound_kind)` | `Read(Buffer(n))` / `Read(WasmMem(m))` | `bound_kind` is the certificate's choice: checked against the buffer table, or covered by a guarded reservation ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md) item 5) |
| **Atomics** | `AtomicRmw`, `AtomicCmpxchg`, `Fence(order)` | `Read+Write(cat)`, ordered | Lowerings checked against the JS memory model ([JS 6.12](../../reports/Izcal%20JavaScript%20engine.md)). Any access, atomic or not, to a **shared** `Buffer` or `WasmMem` is never rematerialized, duplicated or merged by load elimination, because another agent may write between two reads (the memory model allows the observed values to differ) |
| **Allocation** | `Allocate(layout, size_class)`; `Charge(units)` | `Alloc`, `Safepoint` | `Allocate` carries its canonical allocation charge from the facts. When escape analysis removes an allocation it leaves a `Charge(units)` in its place: the canonical units are still charged and `Charge` is still a canonical GC trigger point, as the spec requires ([JS 6.5](../../reports/Izcal%20JavaScript%20engine.md), [JS 6.9](../../reports/Izcal%20JavaScript%20engine.md)). Materializing that object on deopt charges 0, because its units were charged already |
| **Calls** | `CallJs(callee, argc, args)`, `CallWasm(idx, args)`, `CallIndirectWasm(table, i, sig)`, `CallRuntime(stub, args)`, `CallBuiltin(id, args)` | from the callee's facts, at least `MayCallUser`, `MayThrow`, `Safepoint` | Every call is followed by the frame's status check; the call instruction itself carries the after-call frame state (6.2) |
| **Generic operation** | `Op(hgop_id, operands)` | exactly the operation's facts | The graph builder's fallback for any bytecode or Wasm instruction it does not specialize; lowered to the operation's generic path. This keeps the builder total |
| **Interrupt poll** | `Poll` | `Safepoint` only | At every loop back-edge, and at an inlined function entry whose inlining tree contains a `charge`; carries the canonical progress counter update ([JS 6.10](../../reports/Izcal%20JavaScript%20engine.md); the counter and the decision-applicable points are part 1's R0-B11, [bytecode note](bytecode-and-core-set.md) Section 4.10). The poll's slow path never runs JS: it serves GC and safepoint requests and applies decision points, and records any JS-visible interrupt (termination, a debugger pause) for the next status-checked call or back-edge exit to act on. **The collector never changes a fact, and a decision point applied at a non-call safepoint (a code install, a flush) never invalidates one**, so a poll does not kill load elimination, loop-invariant motion or guard hoisting |
| **Terminators** | `Jump`, `Branch`, `Switch(bounded)`, `Return`, `Throw`, `Deopt(reason)`, `Trap(kind)`, `TailCallWasm`, `Unreachable` | as named | `Switch` lowers to a bounds-checked jump table in read-only code; the jump into it is indirect and each case block starts with a jump-table landing pad ([riscv64-abi.md](riscv64-abi.md), R2-25) |

**What the IR does not have.** No raw pointer type that survives a safepoint; no instruction that writes the stack pointer, a return address, the shadow stack or a base register (those exist only in the backend's prologue, epilogue and the inventoried stubs, [riscv64-abi.md](riscv64-abi.md)); no instruction that reads a clock or entropy (G13) except an `Op` whose facts carry the `Input` effect (Section 7).

### 6.1 Guards: refining definitions (R2-2)

**Decision R2-2: a guard is an instruction that takes a value and returns a refined copy of it; every use that relies on the guard's fact takes the refined copy.** A failing guard takes its **failure action**: `Deopt(frame state, reason)` for JS speculation, `Trap(kind)` for Wasm semantics (a failed `ref.cast` traps, it never deopts), `Throw(error)` for JS checks that are semantic (a `TypeError` on a non-callable).

```
v1 = LoadField(o, slot 3, Field(prop))        ; tagged
v2 = CheckSmi(v1) deopt@fs7                   ; v2 : SmallInt, same bits as v1
v3 = UntagInt(v2)                             ; may use v2 only
```

- **Why.** `UntagInt(v1)` is not well typed; only `v2` has the type `SmallInt`. Any pass that moves `UntagInt` above `CheckSmi` breaks SSA dominance, which the IR verifier rejects. No separate control-dependence edge is needed, and none can be forgotten.
- **Guards are effectful** (`MayDeopt`), so they keep their order in the skeleton. Two identical guards on the same value in one block are merged by a verified rule; hoisting a guard out of a loop is a verified rule whose premise is that the loop body cannot change the guarded fact (its effect sets are disjoint from the fact's invalidating categories, Section 7).
- **Range facts** use the same form: `CheckBounds(i, len)` returns `i'` with range `[0, len−1]`, and range analysis reads ranges from the refined names (e-SSA, [ABCD](https://doi.org/10.1145/349299.349342)).
- **Value facts and heap facts.** A guard's predicate is one of two kinds, and the IR records which:
  - a **value fact** is about the value's bits and stays true for the value's whole life: "is a small integer", "is this exact object", a Wasm reference's type;
  - a **heap fact** is about mutable heap state reached through the value: "has shape S", "has element layout L", "`i` < the array's current length", "the buffer is not detached".
  A refinement that carries a heap fact is **valid only until the next instruction whose effects intersect the fact's invalidating categories, or that has `MayCallUser`**. A use after such an instruction needs a new guard (or a dependency, 6.2). IR verifier rule V10 checks this on every path, and the ledger checker's guarded rule uses the same path condition ([code-table-and-ledger.md](code-table-and-ledger.md), 6.2). Without it, a refined "has shape S" would outlive a call that changed the shape: wrong answers inside the cage, and, for a buffer length held across a call that detaches or shrinks the buffer, an access outside it. For buffers the backstop is in the validator: a checked buffer access must load its base and length from the buffer table after the last call or safepoint ([riscv64-abi.md](riscv64-abi.md), R2-27).

| Option | For | Against | Verdict |
|---|---|---|---|
| (a) **Refining definitions** | Hoisting bugs become dominance errors that the verifier catches; ranges flow naturally | One extra value per guard; passes must look through refinements to see equal values (`Refines(v2, v1)` is recorded) | **Recommended** |
| (b) Guards anchored to control (Duboscq et al. 2013) | Fewer values | A use that ignores the anchor is a silent bug; the ledger checker must reconstruct which guard covers which use | Rejected |
| (c) Guards as effect-chain members only | Simple | The same as (b), plus over-serialization of pure code | Rejected |

**Reversal trigger.** If the refinement values cost more than 5%† of O1 compile time on the corpus at JS-M2, O1 may drop refinements whose type the representation pass already knows, keeping them for every guard whose fact a later instruction relies on.

### 6.2 Dependencies, calls and the status check (R2-3)

A **dependency** relies on a fact that no instruction checks at the use: a fact cell ([JS 6.2](../../reports/Izcal%20JavaScript%20engine.md)) such as "the prototype chain of shape S has no property `x`". `Depend(fact)` sits at the first program point where the compiled code relies on it.

**Decision R2-3: a fact can change only inside an instruction whose effect set intersects the fact's invalidating categories, and every such instruction is a lazy-deopt point: a call or runtime call followed by the frame's status check, with an after-call frame state.** The ledger checker ([code-table-and-ledger.md](code-table-and-ledger.md)) verifies this per compilation. The consequence for the IR:
- `CallJs`, `CallRuntime`, `CallBuiltin`, `Op` with `MayCallUser`, and any inline store whose category may hold a registered fact (a prototype's slots, a global cell) carry an **after-call frame state** and are followed by the status check.
- `Poll` changes no fact: its slow path runs no JS, and the decisions it applies (GC completion with weak clearing, installs) change no fact cell; the only hook it reaches, cleanup enqueueing, has no heap context (part 1's R0-H4). It carries a frame state at its `LoopBack` for the debugger's tier-down request (bit 2), not an after-call one. (Phase-8 fix of 31 October 2026: draft 0.1 listed `Poll` here, against the instruction table.)
- An inline store to a category that may hold a registered fact lowers its fact-holder test into the store's slow path, which calls the runtime; so invalidation happens inside a call, never inside straight-line code.
- Between two such points, a dependency's fact cannot change. This is what makes "depend once, use many times" sound.

Calls are not terminators. The status check after a call (one frame-relative load and a branch, [frames-deopt-stackmaps.md](frames-deopt-stackmaps.md)) is inserted by lowering, and its slow edge leads to an out-of-line block that dispatches to: unwind (exception pending), lazy deopt (this frame's code invalidated), or continue. **Exceptional control flow** is explicit only where the compilation unit has a handler (a `try` in the function or an inlined caller): there the call's exception edge is a real CFG edge to the handler block, carrying the exception value as a block argument. Elsewhere, "exception pending" leaves the function by an ordinary return, which keeps JS-D19's return-based unwinding.

## 7. Effects (R2-4)

**Decision R2-4: an effect set is a bitset over a fixed list of effect kinds and heap categories, generated from `.hgop`; no pass may give an instruction a narrower set than its facts (G16, IR verifier rule V3).**

| Effect kind | Meaning | Ordering rule |
|---|---|---|
| `Read(c)` | Reads heap category `c` | Not moved across a `Write(c')` that may alias `c` |
| `Write(c)` | Writes `c` | Ordered with every `Read` or `Write` of an aliasing category |
| `Alloc` | Allocates in the agent heap; charges canonical units | Ordered with other allocations (canonical accounting is order-sensitive) |
| `Safepoint` | May run the collector | Needs a stack map; no raw address may be live across it (frames note) |
| `MayThrow` | May leave with "exception pending" | Needs an after-call frame state |
| `MayDeopt` | Has a frame state and may exit | Ordered with every other effect |
| `MayCallUser` | May run arbitrary JS | Implies every `Read`, `Write`, `Alloc`, `Safepoint`, `MayThrow` and fact invalidation |
| `MayTrap` | Wasm trap | Ordered like `MayThrow` (a trap is a Wasm exception at the JS boundary) |
| `HostHook(cx)` | Reaches an `hg-js-host` hook with context `cx` (part 1's R0-H4) | No context: no heap effect; `&HostCx`: `Read(Heap)`; `&mut HostCx`: `Alloc`, `Safepoint`, `Write(Heap)`; `&mut ScriptCx`: `MayCallUser` |
| `Input(src)` | Reads a B1 input (`entropy`, `clock`) through the realm stream or a hook | Ordered with every other `Input`; never folded, merged or removed, so two `Math.random()` calls stay two |

**Heap categories** form a small tree, so alias checks are a subset test:

| Category | Children (examples) | Who uses it |
|---|---|---|
| `Heap` (top) | everything below | `MayCallUser` |
| `Field(key class)` | named slots by property key; `ShapeWord`; `ElementsPointer`; `ArrayLength` | Property ICs |
| `Elements(layout)` | small-integer, double, tagged elements | Element ICs |
| `FactHolder` | named slots **and indexed elements** of objects with registered facts (prototypes, globals); `NoElementsOnPrototypes` depends on element stores | Stores that may invalidate |
| `Context` | scope slots | Closures |
| `Buffer(n)` | a buffer-table entry's backing store | Typed arrays, `DataView` |
| `WasmMem(m)`, `WasmTable(t)`, `WasmGlobal(g)` | per index | W2 |
| `WasmStruct(type, field)`, `WasmArray(type)` | per canonical type | W2 and Wasm GC |
| `External(kind)` | node columns through `hg-dom`'s API, external strings | Never written inline; only through runtime calls |
| `weak` | weak slots; changed at every `Safepoint` (a cycle clears them), so `Safepoint` writes it | `WeakRef.deref`, FinalizationRegistry |
| `kept` | the agent's `[[KeptAlive]]` list (outside the cage) | `WeakRef` creation and deref |
| `realm` | the realm's random stream | `Math.random` (with `Input(entropy)`) |

The category list is part of `.hgop`: part 1 declares it as one parameterised tree with `category` and `fact` items (R0-O12, [hgop-language.md](hgop-language.md) Section 3), and this table is that tree. Adding a category is a `.hgop` change, regenerated everywhere (G16).

## 8. Types and representations (R2-5)

**Decision R2-5: two orthogonal lattices. A value's *type* is a JS-or-Wasm abstract type; its *representation* is how its bits are held. Guards refine types; the representation pass chooses representations.**

**The type lattice** (join = union):

| Component | Domain | Notes |
|---|---|---|
| **Kind set** | A bitset of about 40† kinds: `SmallInt`, `HeapNumber` (non-small doubles), `NaN`, `MinusZero`, `Undefined`, `Null`, `True`, `False`, `InternalizedString`, `OtherString`, `Symbol`, `BigInt`, object kinds (`PlainObject`, `Array`, `TypedArray(kind)`, `Function`, `BoundFunction`, `Proxy`, `HostObject`, `WasmStruct`, `WasmArray`, `WasmFunc`, …), `Hole`, `Exception` | A bitset makes join and meet one instruction and is the shape JSC's published speculation model uses ([Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/), Medium) |
| **Shape set** | ⊤ or a set of at most 4† shapes | From IC programs and `CheckShape` |
| **Range** | ⊤ or `[lo, hi]` over int32 or int64 | Only from verified range rules |
| **Constant** | ⊤ or one value | Static roots, numbers, image objects |
| **Wasm reference type** | ⊤ or a canonical type ID with nullability | Exact from validation; refined only by casts |

**Representations:** `Tagged32` (a heap slot's form), `I32`, `I64`, `F32`, `F64`, `V128`, `Bool`, and `CageRef` (a compressed reference already known to be a reference). There is no "raw pointer" representation in the IR; decompressed addresses exist only after lowering, inside one safepoint-free span.

The representation pass picks the cheapest representation that all uses accept and inserts `Tag`, `Untag`, `Box` and `Unbox`. `BoxF64` allocates, so it is an `Alloc` with a canonical charge of zero: boxing is engine-internal, never semantic allocation ([JS 6.5](../../reports/Izcal%20JavaScript%20engine.md) "Accounting"). This keeps canonical GC triggers the same across tiers.

**Cost.** Two lattices instead of one. **Reversal trigger.** None needed for correctness; the 40-kind and 4-shape sizes are tuning, re-set from the corpus at JS-M2.

## 9. How operation facts enter the IR (R2-6)

`hg-opgen` emits, for each `.hgop` operation, one **facts row** that `hg-jit-opt` includes as generated Rust (G16). The IR never contains a hand-written effect set.

| Field | Used for |
|---|---|
| `id`, `anchor` | Provenance, JS-60's reason text, ANC-1 |
| `operands`, `results` with their representations per fast path | The graph builder's specialized lowering |
| `effects` (Section 7) per fast path and for the slow path | IR verifier rule V3; alias analysis; the ledger checker |
| `invalidates`: the fact kinds the operation may invalidate | The ledger checker (R2-3) |
| `alloc`: canonical charge | `Allocate`'s charge; GC accounting |
| `deopt_points`, `safepoints` | Which IR instructions need frame states and stack maps |
| `type_transfer` | The type of each result given operand types (a small table or expression in `.hgop`) |
| `generic_lowering` | The `Op(hgop_id)` fallback: a call to the operation's generated runtime path |
| `smt` | The SMT encoding ID that `hg-rulecheck` uses for rules mentioning this operation |
| `ic_kind` | Whether the operation has an IC slot and which IC program language applies |

**Requirement on part 1 of the review.** `.hgop` must be able to state: heap categories per access (Section 7), `invalidates`, a canonical allocation charge, per-fast-path effects, and a type transfer function. These are the minimum the IR needs; part 1 owns the syntax.

**Building the graph.** For each bytecode operation in order, the builder:
1. looks up the IC site's recorded programs; if a stable program exists, it transpiles it: each IC guard becomes an IR guard with the operation's frame state, each IC load or store an IR memory instruction, each fact the program relied on a `Depend`;
2. otherwise emits `Op(hgop_id, …)` with the operation's facts;
3. maps interpreter registers to SSA values with Braun et al.'s on-the-fly construction ([CC 2013](https://doi.org/10.1007/978-3-642-37051-9_6)), so the frame state at each point is the current register-to-value map.

## 10. Rules and how verification attaches (R2-7)

**Decision R2-7: every transformation that changes semantics-bearing instructions is a named rule in Izcal's rule language, compiled by `hg-opgen`'s rule front end into matcher code; a rule runs only if the build's verified-rules table holds a passing `hg-rulecheck` record for that rule's exact text.**

| Rule class | Where it runs | Verified against |
|---|---|---|
| **Simplification** (constant folding, algebraic identities, strength reduction) | Eagerly when an instruction is created, as in the acyclic e-graph design ([Fallin](https://cfallin.org/blog/2026/04/09/aegraph/)) | IR semantics: the core opcodes' semantics plus `.hgop`'s SMT encodings |
| **Check removal** (guard merging, guard hoisting, bounds-check removal) | Range analysis and guard passes | IR semantics with ranges; **100%** required (JS-11) |
| **Speculative transformations** (inlining's guard, escape analysis's materialization rules, load elimination) | O2 passes | IR semantics; escape analysis rules also prove that a removed allocation never reaches a weak reference ([JS 6.5](../../reports/Izcal%20JavaScript%20engine.md)) |
| **Barrier elision** | Lowering | Premise "the stored value or target needs no barrier"; the IR verifier rejects an unexplained missing barrier ([JS 6.4](../../reports/Izcal%20JavaScript%20engine.md)) |
| **Instruction selection and lowering** | IR → machine IR | ISA semantics: ASL for AArch64, Sail for riscv64, x86-64 open (Q-JS-4) |

**The verified-rules table.** `hg-rulecheck` produces, per rule, a record: rule ID, SHA-256 of the rule's normalized text, the semantics versions it was checked against (`.hgop` facts digest, ISA model version), the solver and its result. The build embeds the table; the rule matcher for an unverified rule is compiled but gated off, and a build that enables a rule without a matching record fails (JS-11: "an unverified rule cannot be enabled"). Each compilation records the IDs of the rules that fired, which translation validation and the deopt explanations use.

**Pass lists.**

| Level | Passes |
|---|---|
| **O1** | Graph building with IC transpilation and rule simplification at creation; representation selection; redundant-guard elimination (verified rules); dead-code elimination; ledger check; lowering; linear-scan allocation; allocation checker; validator |
| **O2** | O1's front plus: inlining (with frame states for every inlined frame); escape analysis and scalar replacement; load elimination over heap categories; range analysis and check removal; loop-invariant motion; e-graph elaboration of pure instructions; a better allocator (algorithm open, Q-R2-1) |

## 11. W2: Wasm on the same IR (R2-8)

**Decision R2-8: W2 builds the same IR from validated Wasm in one pass over the operand stack, with Wasm-specific opcodes only where Wasm's semantics differ from JS's, and the same rules, lowering, allocator, stack maps and validator.**

| Wasm concern | IR form |
|---|---|
| Locals and the operand stack | SSA values by on-the-fly construction ([Braun et al.](https://doi.org/10.1007/978-3-642-37051-9_6)); block and loop results as block parameters; the validator's type stack gives every value its type ([Wasm validation algorithm](https://webassembly.github.io/spec/core/appendix/algorithm.html)) |
| Integer arithmetic | Wrapping opcodes; traps (`div` by zero, overflow of `trunc`) as `MayTrap` instructions with a trap kind, never deopt |
| Floats | The same `FAdd64` etc.; NaN results follow JS-D12 (a)'s deterministic profile: one lowering per ISA, constant folding that reproduces the hardware's NaN ([JS 6.9](../../reports/Izcal%20JavaScript%20engine.md)) |
| Linear memory | `WasmLoad`/`WasmStore` with `bound_kind` guarded or checked (the Sv39 count, [JS 6.18](../../reports/Izcal%20JavaScript%20engine.md)) |
| Wasm GC | `LoadField`/`StoreField` over `WasmStruct(type, field)` categories, the same layout descriptors as JS ([JS 6.1](../../reports/Izcal%20JavaScript%20engine.md)); `ref.cast` is `CheckWasmCast` with `Trap` |
| `call_indirect` | `CallIndirectWasm` checks the canonical type ID from the Wasm code table's trusted entry ([code-table-and-ledger.md](code-table-and-ledger.md)) |
| Speculative inlining of `call_ref` | `CheckCodeIndex(f, idx)` with `Deopt` to W0, the only Wasm speculation besides verified cast elimination ([JS 6.6](../../reports/Izcal%20JavaScript%20engine.md)) |
| Exceptions (`try_table`, `exnref`) | Explicit exception edges, as for JS `try` |
| `return_call` | `TailCallWasm` terminator: the frame is popped by the epilogue and the callee entered by a jump, so no return address is written ([riscv64-abi.md](riscv64-abi.md) for the landing pad it needs). A tail call cannot grow the incoming stack-argument area, which belongs to the caller's caller, so the Wasm convention gives every Wasm call a **fixed stack-argument area of 8† slots** and passes parameters beyond the registers and that area through the agent's argument buffer; a tail call therefore always fits |
| Frame states | Wasm frame states name a function, a bytecode offset, locals and operand-stack values ([frames-deopt-stackmaps.md](frames-deopt-stackmaps.md)) |

W2 needs no assumption-ledger facts except `CheckCodeIndex` guards, so its gate stays "verified rules" ([JS 6.6](../../reports/Izcal%20JavaScript%20engine.md)).

## 12. The IR verifier

Runs after every pass in debug and CI builds, and once before lowering in release builds (its cost is part of the compile budget; † 5% of compile time).

| Rule | Statement |
|---|---|
| V1 | SSA: every use is dominated by its definition; block arguments match parameter counts and types |
| V2 | Every instruction's representation inputs match its opcode |
| V3 | Every instruction's effect set ⊇ its facts' effect set |
| V4 | Every `MayDeopt` instruction has a frame state whose values dominate it (the frame-state verifier checks the rest) |
| V5 | Every heap store of a `Tagged32` or `CageRef` value has a barrier or a verified elision rule ID |
| V6 | Every `Safepoint` instruction has no live value in a representation the stack map cannot describe |
| V7 | Every `Depend` is in the ledger, and every invalidating instruction after it is a status-checked lazy-deopt point (R2-3) |
| V8 | Only `Allocate` and `Charge` charge canonical units; `Allocate`'s charge equals its facts' charge, and every removed allocation leaves a `Charge` of the same units |
| V9 | Every fired rule ID is in the verified-rules table |
| V10 | No use of a heap-fact refinement is reachable from its guard through an instruction that may invalidate the fact (6.1) |

## 13. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| V8 TurboFan → Turboshaft | Left sea of nodes after a decade; compile time halved ([V8](https://v8.dev/blog/leaving-the-sea-of-nodes)) | Free floating does not pay for effect-heavy JS | R2-1: CFG skeleton, float only the pure part |
| Cranelift aegraph | Eager rewrites at creation plus elaboration; equality saturation rejected ([Fallin](https://cfallin.org/blog/2026/04/09/aegraph/)) | Rewrite rules in a DSL are both fast and verifiable | R2-7: one rule language for simplification and lowering, verified |
| Crocus, Arrival | SMT verification of lowering rules found a CVSS 9.9 bug and new ones ([SE 6.4](../Izcal%20JavaScript%20engine/security.md)) | Verification works when rules are data | Rules are data from the first rule |
| JSC | Speculation is cheap to win, expensive to lose ([Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/)) | Count exits per site; back off | The ledger records each speculation's site; recompilation removes it |
| CVE-2020-6418 | A JIT's side-effect model disagreed with the runtime ([JS 2](../../reports/Izcal%20JavaScript%20engine.md)) | Effects must have one source | R2-4, R2-6, V3 |

## 14. Targets proposed by this note

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| R2T-1 | The IR verifier catches planted IR bugs | 100% of JS-10's "effect set narrower than the facts", "unverified check removal" and new "use above its guard" and "heap-fact refinement used after an invalidating effect" classes, each with ≥ 20† variants | JS-10 harness | Ship blocker for T2 and W2 |
| R2T-2 | O1 is cheap | O1 compile throughput ≥ 10%† of T1's on the corpus (JS-D6's trigger, restated as a target) | Lab at JS-M2 | JS-D6's reversal: a separate quick compiler |

## 15. Open questions

- **Q-R2-1.** O2's register allocator: a graph-colouring, a second-chance linear scan or another design. Not decided at JS-M0; the allocation checker makes the choice a performance question, not a correctness one. Decided at JS-M2 from compile-time and code-quality measurements on the corpus.
- **Q-R2-2.** x86-64 ISA semantics for lowering rules (Q-JS-4), unchanged.
- **Q-R2-3.** The bitset's kind list and the shape-set cap (40†, 4†) are guesses until the corpus's IC distribution is measured (Q-JS-9, JS-M1).

**Flagged claims.** The Duboscq, Flückiger, D'Elia, Click and Diwan papers are cited from their titles, venues and abstracts as remembered and checked against Crossref metadata on 31 October 2026; their bodies were not re-read this session (Medium).
