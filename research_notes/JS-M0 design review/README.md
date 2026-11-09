# JS-M0 design review

*Design review for the first milestone of Izcal's JavaScript engine (JS-M0), written with the first-principles-spec method. Date: 31 October 2026. Status: parts 1 and 2, draft 0.2, verified, for owner review; no decision is accepted until the owner says so. Relates to: the [JS spec](../../reports/Izcal%20JavaScript%20engine.md), draft 0.4.4 on `main` (9.2's JS-M0 row as amended by 0.4.5, which is on the unmerged `docs/js-m0-amendments` branch; 6.1 to 6.18); [Stage 1 design](../../reports/HGBA%20Stage%201%20architecture%20design.md) 3.3; the [coding standard](../../reports/Izcal%20coding%20standard.md); the JS-M0 plan's Phases 3 and 4.*

*Revision 0.2 (31 October 2026): phase 8 of part 1, and parts 1 and 2 checked against each other. **Fact-check** (an agent that did not write the notes): 40 load-bearing claims of part 1 checked, 30 correct, 9 partly correct, 0 wrong, 1 unverifiable. 136 links checked (128 external, 8 relative), with no failures; all 114 `#fragment`s into ECMA-262, Temporal, Dynamic Code Brand Checks and HTML exist. The hook count of 18, the 16 host-defined clauses, Annex D.1's omission, the proposals' stages and HTML 8.1.6's seven parameters were all confirmed. Three must-fix items, all applied: Node's string limit is 536,870,888 code units (2^29 − 24), not the 1,342,177,280 that draft 0.1 quoted; the "16 host hooks" text is in JS spec 6.0 and 6.14, not 4.2; and draft 0.1's mnemonics matched another engine's bytecode names with no permitted source cited, so every one is renamed to Izcal's own (the unverifiable item). All seven recommended fixes are applied too (row 18's global object, HR-Time's coarsening, the evaluation-order clause, Icarus versus Arrival, the RVA22 quotation, a ≤ bound, internal allocation on comparisons). **Design critique** (a second agent), against the JS spec, Stage 1 3.3, the coding standard, `heap.md` and part 2: 7 High, 16 Medium, 12 Low. Every High and Medium is applied: the frame is based on part 2's frame record `R` with part 2's header (H1, H2); a user call ends its path, operations declare their after-call accumulator, and five tail user-call kinds are T0's call kinds and continuations (H3, H4, R0-O11); a separate liveness pass (H5); the progress counter and decision-applicable points (H6, R0-B11); `ROPE_MIN_LENGTH` derived from Izcal's own layout instead of an unexplained value that matched another engine's constant (H7); the frame charge's formula (M1); the facts clauses and the declared category tree (M2, M3, R0-O12); hook contexts as effects (M4); an `Input` effect (M5); weak state at GC points (M6); engine-thrown errors charged (M7); helper rooting (M8); forward handlers (M9); a probe that keeps its registry (M10); the spec version stated everywhere (M11); all six intrinsics in the amendment (M12); IC heap constants placed (M13); the mnemonics (M14); ID collisions (M15, with heap.md's part left to the heap change); targets R0T-1 to R0T-6 (M16). Ten Low items are applied and two are recorded below with their reasons. **Cross-part contradictions** found: six, all resolved, five of them with a minimal change on part 2's side (listed under "Contradictions between the parts"); no part 2 decision, target or threshold changed. **M-1 part 3:** no link points to third-party source code; the two values that matched another engine's (the mnemonics, the rope length) are replaced. Part 2's own verification is in [part-2-index.md](part-2-index.md).*

*Revision 0.1 (31 October 2026): part 1 (bytecode, `.hgop`, host hooks); part 2 (IR, frames, code table, ABIs) on its own index.*

The JS spec's JS-M0 row asks for a design review that fixes, together, the parts of the engine every later tier depends on. **Part 1** fixes the bytecode format and the core operation set, the `.hgop` operation description language with what `hg-opgen` emits at JS-M0, and the `hg-js-host` hook table. **Part 2** fixes the T2 and W2 IR, frame states, the deopt model, stack maps, the code-table entry, the assumption ledger and the riscv64 items. **`heap.md`** fixes the first `hg-heap`; it lands with the heap change (branch `js/hg-heap`), and its decisions are listed here as a pointer.

## Files

| File | Part | What it fixes |
|---|---|---|
| [bytecode-and-core-set.md](bytecode-and-core-set.md) | 1 | Instruction encoding, operand kinds, registers and the frame as the interpreter sees it, the accumulator, constants, IC slots, jumps, the bytecode unit and its image form, the progress counter, the 49 opcodes and 9 intrinsics of the core set with anchors, canonical units, the text assembler, and what the JS-M1 verifier checks |
| [hgop-language.md](hgop-language.md) | 1 | The grammar, types, effects, GC and deopt points, tail user calls, anchors, the totality and rooting rules, the checks OC-1 to OC-17, `hg-opgen`'s JS-M0 outputs, the SMT argument, and six operations written in full |
| [js-host-hooks.md](js-host-hooks.md) | 1 | The audited hook list (18 methods), each hook's Rust signature, context, substrate trait and determinism class; the layering points that are not hooks; the environment reads; the JS-M0 test host |
| [part-2-index.md](part-2-index.md) | 2 | Part 2's decisions R2-1 to R2-27, its requirements on part 1, its contradictions C1 to C16, its owner questions and its verification |
| [optimizer-ir.md](optimizer-ir.md) | 2 | The one SSA IR for T2 and W2: control, instruction kinds, effects, types, `.hgop` facts, guards and dependencies, verified rules, W2, the IR verifier |
| [frames-deopt-stackmaps.md](frames-deopt-stackmaps.md) | 2 | Frame layouts for every tier on three ISAs; the status slot; eager and lazy deopt, OSR, exceptions; frame states; stack maps and the GC walk; stacks, segments and the canonical frame budget |
| [code-table-and-ledger.md](code-table-and-ledger.md) | 2 | Sealed and runtime-trusted data; the code tables and their 32-byte entry; calls through them; heap constants; facts, the ledger, installation, invalidation and discard |
| [riscv64-abi.md](riscv64-abi.md) | 2 | Register roles on riscv64, AArch64 and x86-64; landing-pad kinds and labels; the segment-switch stub per ISA; validator rules |
| `heap.md` (lands with the heap change) | heap | `hg-heap` at JS-M0: the cage module, layout, Immix geometry, tracing, debt, weak cells, brands |
| [heap-node-space-amendments.md](heap-node-space-amendments.md) | heap | **Proposed, not accepted.** The two node-space decisions taken after the review (a `node` field kind amending H-4; node→wrapper strong, amending Stage 1 5.2), the two still open with their options, and what the heap verifier found about H-4's missing forwarding bit |

**Identifiers.** Part 1's decisions are **R0-B** (bytecode), **R0-O** (`.hgop`) and **R0-H** (hooks), its targets **R0T**, its questions **Q-R0-1 to Q-R0-9** and its `.hgop` checks **OC-1 to OC-17**. Part 2's are **R2-**, **R2T-**, **Q-R2-**, **OQ-** and its contradictions **C1 to C16**. In part 1, **H-1** and **H-2** are the JS spec's invariants (6.1); heap.md's decisions are cited as "heap.md H-n". The critique's finding labels (H1, M1, L1) appear only in this header and in "Low findings recorded".

## Decisions fixed in part 1

### Bytecode and the core set

| ID | Decision |
|---|---|
| **R0-B1** | A byte stream: one-byte opcodes, one-byte operands, widened for one instruction by `Wide2` (2 bytes) or `Wide4` (4 bytes); `0x00` is `Invalid`; `0xFD` is reserved for a second opcode byte; little-endian, unaligned operands; the format identifier is a hash of the generated opcode table |
| **R0-B2** | A register operand is a signed slot index relative to the frame record `R` (R2-9): `address(v) = R − 36 − 4·v`; locals non-negative, receiver and parameters negative beyond part 2's eight-slot header (R2-9, R2-11), which is a hole the verifier rejects; all registers start as `undefined` |
| **R0-B3** | The left operand of a binary operation is in a register, the right in the accumulator (ECMA-262's evaluation order); the accumulator is spilled at every GC point; Numbers have one canonical representation (a small integer whenever representable and not −0) |
| **R0-B4** | Constant kinds live in the sealed unit; constant values live in a per-(unit, agent) array in the cage, rooted by the function's per-agent unit record |
| **R0-B5** | IC slot operands are in the encoding from JS-M0, with a sealed slot-kind table; the reference interpreter ignores them until JS-M1 |
| **R0-B6** | Jumps carry unsigned distances whose direction the opcode fixes; only the unconditional `LoopBack` goes backward, so every loop passes an interrupt check; every handler lies after its range; a unit ends in a terminator |
| **R0-B7** | The bytecode unit's sealed form, with its static frame charge and liveness table, and its pointer-free, little-endian image form, loaded as hostile input with checked arithmetic |
| **R0-B8** | `WeakRef`, `FinalizationRegistry`, `Math.random`, the clock and host functions are intrinsics called through one `CallIntrinsic` opcode, not opcodes of their own |
| **R0-B9** | The JS-M0 core set: 49 opcodes (`0x01`–`0x31`) and 9 intrinsics, over a value domain without BigInt and Symbol values, with one minimal realm per agent; every opcode that may call user code declares its after-call accumulator |
| **R0-B10** | Canonical units charge only semantic allocations, engine-thrown errors included, by formulas over semantic quantities†; the canonical string-length limit is 2^30 − 1 code units† |
| **R0-B11** | The progress counter counts `LoopBack`s executed and logical function entries; decisions (GC completion, weak clearing) apply only at decision-applicable points (`LoopBack`, function entry, job end); collections forced by real memory pressure clear no weak reference |

### `.hgop` and `hg-opgen`

| ID | Decision |
|---|---|
| **R0-O1** | `.hgop` is Izcal's own small, total, effect-typed language with no loop construct and no recursion |
| **R0-O2** | Its operand and encoding declarations are the one source of the opcode table (decoder, assembler, disassembler, verifier tables) |
| **R0-O3** | Effects are declared and inferred; the inferred set must fit the declaration; a runtime function's declared effects become its generated Rust signature, so `rustc` enforces them too |
| **R0-O4** | Two allocation effects: semantic (charged in canonical units) and internal (free) |
| **R0-O5** | One spec anchor per path, checked against the pinned `spec.html`, with `izcal` anchors for engine-defined paths |
| **R0-O6** | A heap value crosses a GC point only in the frame or as an argument of that GC point; helper parameters are `Root`s or declared `frame` |
| **R0-O7** | Deopt only at instruction boundaries; a speculable guard precedes every effect on its path; user code runs only inside runtime calls or tail user calls |
| **R0-O8** | JS-D14's re-validation rule is a checked epoch rule over mutable heap reads (OC-9), with GC points ending it for weak state |
| **R0-O9** | At JS-M0 `hg-opgen` emits the opcode table, the reference interpreter (generic over a generated `Runtime` trait), the operation facts and the anchor index, checked in with a freshness check |
| **R0-O10** | `[[KeptAlive]]` is a per-agent root list outside the cage; pushing to it is not a GC point and shades the target under concurrent marking |
| **R0-O11** | A user call is its path's last effect; each item declares its after-call accumulator (`result` or `preserve`); five tail user-call kinds are T0's call kinds and continuations |
| **R0-O12** | Heap categories and fact kinds are one parameterised tree declared in `.hgop`, from which T2's effect sets (R2-4) are generated |

### Host hooks

| ID | Decision |
|---|---|
| **R0-H1** | 18 hook methods: ECMA-262's 16, Temporal's HostSystemUTCEpochNanoseconds and Dynamic Code Brand Checks' HostGetCodeForEval, with the proposal's eight-parameter HostEnsureCanCompileStrings |
| **R0-H2** | The host never holds a GC reference: engine-owned IDs, `HostData` for `[[HostDefined]]`, and held handles from engine root tables |
| **R0-H3** | Spec requirements on hooks are enforced by Rust types where possible |
| **R0-H4** | Each hook's context (no heap, read-only heap, may allocate, may run script) is frozen with its signature and is its effect for the optimizer; the cleanup-enqueue hook stays at no context |
| **R0-H5** | DOM-less embedders use a `NonBrowserHost` adapter that fixes the browser-only hooks to their spec defaults |
| **R0-H6** | `Math.random` draws per realm from the substrate's entropy; `Date.now` goes through HostSystemUTCEpochNanoseconds already; the system time zone is recorded agent configuration |
| **R0-H7** | FinalizationRegistry cleanup is always enqueued, while the GC-completion decision is applied |
| **R0-H8** | Every hook has a determinism class: pure, queued, task, input, decision or user code |
| **R0-H9** | JS-M0 implements a test host for the hooks its core set reaches |
| **R0-H10** | The hook audit is a CI test against the pinned specifications, rerun at every pin |

## Decisions in part 2 and in `heap.md`

**Part 2:** R2-1 to R2-27, with rejected alternatives and reversal triggers, are in [part-2-index.md](part-2-index.md#decisions); its targets R2T-1 to R2T-9 are in its notes.

**`heap.md`** (branch `js/hg-heap`; the file lands with the heap change):

| ID | Decision (headline) |
|---|---|
| heap H-1 | Every cage access goes through one inventoried module, `cage`, with the offset masked to the cage length and aligned to its width |
| heap H-2 | Objects are never materialised as Rust references |
| heap H-3 | `Trace` is a safe trait, implemented only through a declarative macro |
| heap H-4 | Object layout: a 4-byte header (type index and forwarding bit), a 4-byte length word, then traced slots, then raw words; 8-byte aligned |
| heap H-5 | Immix geometry: 32 KiB blocks of 128-byte lines, committed in 1 MiB chunks |
| heap H-6 | Objects larger than 8 KiB go to a non-moving large-object space |
| heap H-7 | One tracing loop serves marking and evacuation; a moving test collector evacuates every block |
| heap H-8 | GC debt is counted in canonical units, charged by the operation through `Mutator::charge(units)`, never by the allocation (part 1's R0-B10) |
| heap H-9 | A cycle's completion and its weak clearing are one decision point, `GC_COMPLETE` |
| heap H-10 | Weak cells: weak slots skipped by marking and cleared or updated at completion; `deref` adds to the kept-alive list |
| heap H-11 | Brands and roots follow the JS spec's sketch (JS-D18) |

## Contradictions between the parts, and how they were resolved

| # | Contradiction | Resolution |
|---|---|---|
| X1 | Part 1's frame formula was based on `fp`, part 2's on the frame record `R`; on riscv64 `R = fp − 16`, so every slot would have been 16 bytes off. The header's contents also differed slot by slot, so part 1's scan map would have read part 2's `argc` as a reference | Part 1 adopts part 2's base and header (same size, so the constant 36 stands); part 2 states the T0 body's offsets numerically. Q-R0-3 closed |
| X2 | R0-O7 forbade resume points inside a description; part 2's deopt resumed `.hgop` builtins inside them and expected `.hgop` to define per-kind call continuations | Tail user calls of five kinds (R0-O11); `.hgop` builtins that call script are split into pieces at them; part 2's builtin continuations resume at a piece's start |
| X3 | Lazy deopt resumed "after call" with the call's result as the accumulator, which is wrong for paths that work after the call (`Lt`) or keep the accumulator (`SetNamed`) | OC-17 makes the user call the last effect; each operation declares `after_call result` or `preserve`; part 2's 5.2 restores what the mode names |
| X4 | Part 2 needed per-offset liveness from the verifier, which part 1 said needed no dataflow | A separate liveness pass after the linear safety verifier (bytecode note, Section 8) |
| X5 | Part 2's `HostHook` implied `MayCallUser` for every hook; its category tree had no weak, kept-objects or realm categories; it had no effect for entropy and clock reads; R2-3 listed `Poll` among after-call points against its own instruction table | Per-context `HostHook`, an `Input` effect, three more categories declared in `.hgop` (R0-O12), and `Poll` removed from R2-3's list, all on part 2's side |
| X6 | Part 1 left IC-program heap constants to part 2, which had not placed them | A traced per-site constant table in the runtime-trusted IC arena (code-table note) |

**Cross-branch items for the heap change.** heap.md H-8 starts a cycle when debt reaches the trigger and H-9 advances the progress counter per operation; R0-B11 runs the requested cycle at the next decision-applicable point and counts `LoopBack`s and entries, so the heap change aligns both. heap.md's `H-` prefix collides with the JS spec's invariants H-1 and H-2; renaming it `HP-` in the heap change is recommended.

## Decisions not made here, and why

- **Feedback predicates for `speculate`.** Named by the grammar, defined with the IC language at JS-M1 (Q-R0-4).
- **Part 2's open items** (the O2 register allocator, x86-64 semantics for `hg-rulecheck`, Windows stacks, the canonical budget's value, the riscv64 trampoline label, PAC/BTI/GCS availability, token arithmetic): [part-2-index.md](part-2-index.md), "Decisions this review could not make".

## Targets proposed by part 1

| ID | Metric | Threshold | When | If missed |
|---|---|---|---|---|
| **R0T-1** | Share of instructions that need `Wide2` or `Wide4`, on the corpus | ≤ 5%† | JS-M1 | Feed R0-B1's reversal trigger at JS-M2 with fixed-word sizes measured |
| **R0T-2** | Share of corpus functions that fit 128 locals and 115 parameters (Q-R0-2) | ≥ 99%† | JS-M1 | Re-open the narrow operand width |
| **R0T-3** | Verification plus liveness time, as a share of bytecode emission time on the corpus; safety verifier time linear in code length on a size sweep | ≤ 5%†; linear | JS-M1 | Re-design the liveness pass (sparse or on demand at first deopt) |
| **R0T-4** | `hg-opgen` output byte-identical on three OSes and fresh against the checked-in copy (OC-15, OC-16) | Every CI run | From JS-M0 | CI fails |
| **R0T-5** | Hook audit: every host-defined clause of the pinned specifications has a trait method (R0-H10) | Every CI run and every pin | From JS-M0 | CI fails until a method exists |
| **R0T-6** | Assembler round trip: disassembling and reassembling a valid unit gives the same bytes | 10^5† generated units per CI run, no failure | From JS-M0 | CI fails |

## Contradictions and gaps found in the specifications (part 1)

Part 2's are C1 to C16 in [part-2-index.md](part-2-index.md).

| # | Where | What | Proposed resolution |
|---|---|---|---|
| 1 | Stage 1 3.3 | "`Math.random`, `Date.now` and `performance.now`: all three are host hooks." ECMA-262 has no hook for `Math.random` (implementation-defined) or `Date.now` (none until Temporal merges), and `performance.now` belongs to HTML, not the engine | R0-H6; amend Stage 1 3.3's row to name the routes, and the substrate's `EntropyPurpose::Random` documentation with it |
| 2 | JS spec 6.0 and 6.14 | "16 host hooks" (6.0's diagram) and "(17 with Temporal)" (6.14), while 6.14 adopts Dynamic Code Brand Checks for Trusted Types, which adds HostGetCodeForEval and changes HostEnsureCanCompileStrings's signature | R0-H1: 18 methods; amend 6.0 and 6.14's count |
| 3 | JS spec 9.2, JS-M0 row (0.4.5) | The row names "a dictionary object" and WeakRef and FinalizationRegistry "creation and deref", but the FinalizationRegistry constructor throws without a callable and observing a cleanup needs `register`; the plan's probe also draws `Math.random` and reads the clock | R0-B8, R0-B9: amend the row to list `%ObjectCreate`, `%FinalizationRegistryRegister`, `%FinalizationRegistryUnregister`, `%CreateHostFunction`, `%MathRandom` and `%DateNow` |
| 4 | The plan (no host hooks until JS-M1) | The core set's FinalizationRegistry and WeakRef reach HostMakeJobCallback, HostCallJobCallback, HostEnqueueFinalizationRegistryCleanupJob and ClearKeptObjects at JS-M0 | R0-H9: a test host at JS-M0 |
| 5 | JS spec 6.3 | The verifier checks "stack-height invariants"; a register machine has no operand stack. 6.3 also calls the verifier linear, while frame states need liveness | Reword 6.3: register-list and frame-size bounds ([bytecode note](bytecode-and-core-set.md) Section 8), and a separate liveness pass |
| 6 | `hg-substrate` and `hg-substrate-sim` | `Entropy`'s documentation says adding a consumer does not perturb other sequences, but the simulator's stream counter is shared across purposes | Q-R0-8: a keyed stream |
| 7 | The `substrate/js-m0-decisions` branch | `DecisionLog` has `record` only; the plan's Phase 0 item 3 also needs `apply_at` for replay | Add it with Phase 0, before replay depends on weak clearing |
| 8 | ECMA-262 Annex D.1 (informative, external) | Lists InitializeHostDefinedRealm, which is not a host-defined operation, and omits HostEnsureCanAddPrivateElement, which is | None needed: this audit counts clause types; worth reporting upstream |
| 9 | HTML 8.1.6 against the Dynamic Code Brand Checks proposal (external) | HTML's HostEnsureCanCompileStrings has seven parameters, without `direct`; the proposal's has eight. HTML's prose calls the hooks "implementation-defined", ECMA-262 "host-defined" | The trait takes the superset (Q-R0-7) |
| 10 | JS spec 6.3 ("states its deopt and safepoint points") | A description has no deopt points inside it under R0-O7 | Reword to "speculable guards and GC points; deopt only at instruction boundaries" |
| 11 | JS-D14 and 6.3 "Builtins" | "Every builtin the optimizer may inline" is one description, but a builtin that calls user code and then works cannot be one under R0-O7 | Inlinable builtins that call user code are split at tail user calls (R0-O11) |
| 12 | JS spec 6.9 and 6.10 | Decision points are logged at a "progress counter" whose unit is not defined, and JS-51 needs it tier-independent | R0-B11: define the counter and the decision-applicable points |

## Low findings recorded, not applied in full

- **L4 (types and layout assumptions).** Applied in part: kind tests and the `document.all` test read the type index and a type flag at JS-M0 (heap.md H-4), not a shape. Not applied: the string charge keeps its nominal 16-byte header, because canonical units are nominal by design (R0-B10) and re-derived from real layouts at JS-M1; tying them to H-4 now would change them again then.
- **L11 (terminology).** Part 1's "unit" and "per-agent unit record" and part 2's "trusted function record" are kept: the sealed unit is part 2's trusted function record, and the per-agent unit record is its cage-side companion. A rename now would touch both parts for no change in meaning; JS-M1's first implementation picks one name for both.
- Part 2's stack charges are "canonical bytes"; part 1 calls them "canonical depth bytes" to keep them apart from the GC's canonical allocation units (M1). Part 2's wording is left as it is: in its own text the context makes the meaning unambiguous.

## Open questions for the owner (both parts)

1. **Amendments.** Accept part 1's contradictions 1–3, 5 and 10–12 as amendments to Stage 1 3.3 and the JS spec (6.0, 6.3, 6.9, 6.10, 6.14, JS-D14 and the JS-M0 row), and part 2's wording amendments (part 2's OQ-2: 6.2, 6.4, 6.5, 6.6, 6.7, 6.18 and H-1), applied in a docs change after this review is accepted? Recommendation: **accept**; none changes a decision, target or threshold of the spec.
2. **Spec base.** Merge `docs/js-m0-amendments` (0.4.5) before this review is accepted, so that both parts cite one version on `main`? Recommendation: **yes**; part 1 relies on 0.4.5's JS-M0 row, part 2 on 0.4.4, and nothing in part 2 depends on the difference.
3. **Decision-applicable points (R0-B11).** Accept that a cycle requested by canonical debt runs at the next `LoopBack`, function entry or job end, and that a collection forced earlier by real memory pressure clears no weak reference? It changes heap.md H-8's "a cycle starts when the debt reaches the trigger". Recommendation: **accept**; otherwise the position of `GC_COMPLETE` differs between a tier that counts operations and one that removes them.
4. **R2-18's split of the trusted region** (part 2's OQ-1): sealed versus runtime-trusted, amending JS-D16's "written only … through a write window". Recommendation: **accept** (part 2's C2).
5. **Recursion depth** (part 2's OQ-3): should the canonical budget allow at least the deepest recursion of the shipping browsers, measured black-box at JS-M1? Recommendation: **yes**.
6. **Entropy.** Add `Entropy::stream_keyed(purpose, key)` in Phase 0 (Q-R0-8), or derive realm streams inside the engine? Recommendation: **`stream_keyed`**.
7. **Two Stage 3 hooks.** R0-H1 freezes HostGetCodeForEval and the eight-parameter HostEnsureCanCompileStrings while Dynamic Code Brand Checks is at Stage 3. JS spec 6.14 already adopts the proposal; confirm, since 6.14 otherwise waits for Stage 4 on proposals that can still change.
8. **System time zone exposure** (Q-R0-9): a recorded per-agent value is supported either way; which spec decides whether pages see the device's zone?

The remaining open questions (Q-R0-1, Q-R0-2 and Q-R0-4 to Q-R0-7; Q-R2-1 to Q-R2-10) are measurements or later-milestone choices, listed in the notes and in [part-2-index.md](part-2-index.md).
