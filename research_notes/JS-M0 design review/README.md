# JS-M0 design review

*Design review for the first milestone of Izcal's JavaScript engine (JS-M0), written with the first-principles-spec method. Date: 31 October 2026. Status: part 1 of 2, draft 0.1, for owner review. The review's independent critic and fact-checker run over parts 1 and 2 together; they have not run yet. Relates to: [JS spec 0.4.5](../../reports/Izcal%20JavaScript%20engine.md) 9.2 (the JS-M0 row, "design review"), 6.3, 6.9 and 6.14; [Stage 1 design](../../reports/HGBA%20Stage%201%20architecture%20design.md) 3.3; the JS-M0 plan's Phase 3 and Phase 4.*

*Revision 0.1 (31 October 2026): part 1 (bytecode, `.hgop`, host hooks).*

The JS spec's JS-M0 row asks for a design review that fixes, together, the parts of the engine every later tier depends on. **Part 1**, in this folder, fixes the bytecode format and the core operation set, the `.hgop` operation description language with what `hg-opgen` emits at JS-M0, and the `hg-js-host` hook table. **Part 2** (a separate change) fixes the T2 and W2 IR, frame states, the deopt model, stack maps, the code-table entry, the assumption ledger and the riscv64 items. Part 1's decisions are numbered **R0-B** (bytecode), **R0-O** (`.hgop` and `hg-opgen`) and **R0-H** (host hooks), and its open questions **Q-R0-1 to Q-R0-9**, so that part 2 can number its own without collision.

## Files

| File | What it fixes |
|---|---|
| [bytecode-and-core-set.md](bytecode-and-core-set.md) | Instruction encoding, operand kinds, registers and the frame as the interpreter sees it, the accumulator, constants, IC slots, jumps, the bytecode unit and its image form, the 49 opcodes and 9 intrinsics of the core set with anchors, canonical units, the text assembler, and what the JS-M1 verifier checks |
| [hgop-language.md](hgop-language.md) | The grammar, types, effects, GC and deopt points, anchors, the totality and rooting rules, the checks `hg-opgen` performs, its JS-M0 outputs, the SMT argument, and six operations written in full |
| [js-host-hooks.md](js-host-hooks.md) | The audited hook list (18 methods), each hook's Rust signature, context, substrate trait and determinism class; the layering points that are not hooks; the environment reads; the JS-M0 test host |

## Decisions fixed

### Bytecode and the core set

| ID | Decision |
|---|---|
| **R0-B1** | A byte stream: one-byte opcodes, one-byte operands, widened for one instruction by `Wide` (2 bytes) or `ExtraWide` (4 bytes); `0x00` is `Illegal`; `0xFD` is reserved for a second opcode byte; little-endian, unaligned operands; the format identifier is a hash of the generated opcode table |
| **R0-B2** | A register operand is a signed slot index relative to the frame: locals non-negative, receiver and parameters negative beyond a fixed header, which is a hole the verifier rejects; one address formula for every register; all registers start as `undefined` |
| **R0-B3** | The left operand of a binary operation is in a register, the right in the accumulator (ECMA-262's evaluation order); the accumulator is spilled at every GC point; Numbers have one canonical representation (a small integer whenever representable and not −0) |
| **R0-B4** | Constant kinds live in the trusted unit; constant values live in a per-(unit, agent) array in the cage |
| **R0-B5** | IC slot operands are in the encoding from JS-M0, with a trusted slot-kind table; the reference interpreter ignores them until JS-M1 |
| **R0-B6** | Jumps carry unsigned distances whose direction the opcode fixes; only the unconditional `JumpLoop` goes backward, so every loop passes an interrupt check; a unit ends in a terminator; an empty handler table is part of the format |
| **R0-B7** | The bytecode unit's trusted form and its pointer-free, little-endian image form, loaded as hostile input with checked arithmetic |
| **R0-B8** | `WeakRef`, `FinalizationRegistry`, `Math.random`, the clock and host functions are intrinsics called through one `CallIntrinsic` opcode, not opcodes of their own |
| **R0-B9** | The JS-M0 core set: 49 opcodes (`0x01`–`0x31`) and 9 intrinsics, over a value domain without BigInt and Symbol values, with one minimal realm per agent |
| **R0-B10** | Canonical units charge only semantic allocations, by formulas over semantic quantities†; the canonical string-length limit is 2^30 − 1 code units† |

### `.hgop` and `hg-opgen`

| ID | Decision |
|---|---|
| **R0-O1** | `.hgop` is Izcal's own small, total, effect-typed language with no loop construct and no recursion |
| **R0-O2** | Its operand and encoding declarations are the one source of the opcode table (decoder, assembler, disassembler, verifier tables) |
| **R0-O3** | Effects are declared and inferred; the inferred set must fit the declaration; a runtime function's declared effects become its generated Rust signature, so `rustc` enforces them too |
| **R0-O4** | Two allocation effects: semantic (charged in canonical units) and internal (free) |
| **R0-O5** | One spec anchor per path, checked against the pinned `spec.html`, with `izcal` anchors for engine-defined paths |
| **R0-O6** | A heap value crosses a GC point only in the frame or as an argument of that GC point |
| **R0-O7** | Deopt only at instruction boundaries; a speculable guard precedes every effect on its path; user code runs only inside runtime calls |
| **R0-O8** | JS-D14's re-validation rule is a checked epoch rule over mutable heap reads (C9) |
| **R0-O9** | At JS-M0 `hg-opgen` emits the opcode table, the reference interpreter (generic over a generated `Runtime` trait), the operation facts and the anchor index, checked in with a freshness check |
| **R0-O10** | `[[KeptAlive]]` is a per-agent root list outside the cage; pushing to it is not a GC point and shades the target under concurrent marking |

### Host hooks

| ID | Decision |
|---|---|
| **R0-H1** | 18 hook methods: ECMA-262's 16, Temporal's HostSystemUTCEpochNanoseconds and Dynamic Code Brand Checks' HostGetCodeForEval, with the proposal's eight-parameter HostEnsureCanCompileStrings |
| **R0-H2** | The host never holds a GC reference: engine-owned IDs, `HostData` for `[[HostDefined]]`, and held handles from engine root tables |
| **R0-H3** | Spec requirements on hooks are enforced by Rust types where possible |
| **R0-H4** | Each hook's context (no heap, read-only heap, may allocate, may run script) is frozen with its signature |
| **R0-H5** | DOM-less embedders use a `NonBrowserHost` adapter that fixes the browser-only hooks to their spec defaults |
| **R0-H6** | `Math.random` draws per realm from the substrate's entropy; `Date.now` goes through HostSystemUTCEpochNanoseconds already; the system time zone is recorded agent configuration |
| **R0-H7** | FinalizationRegistry cleanup is always enqueued, while the GC-completion decision is applied |
| **R0-H8** | Every hook has a determinism class: pure, queued, task, input, decision or user code |
| **R0-H9** | JS-M0 implements a test host for the hooks its core set reaches |
| **R0-H10** | The hook audit is a CI test against the pinned specifications, rerun at every pin |

## Decisions not made here, and why

- **The frame header's final contents.** The return status word and any frame-state index are part 2's (JS-D19, frame states). The bytecode note fixes the address formula and the scan map, and names the one constant that moves if part 2 changes the header's size (Q-R0-3).
- **IC program constants.** Whether shapes referenced by IC programs live in the trusted region or in per-site cage arrays is the same question R0-B4 answers for bytecode constants; it belongs to part 2 and JS-M1's IC language.
- **Continuations for inlined built-ins that call script.** R0-O7 forbids resume points inside a description; whether T2 needs a separate mechanism is part 2's.
- **Feedback predicates for `speculate`.** Named by the grammar, defined with the IC language at JS-M1 (Q-R0-4).

## Contradictions and gaps found in the specifications

| # | Where | What | Proposed resolution |
|---|---|---|---|
| 1 | Stage 1 3.3 | "`Math.random`, `Date.now` and `performance.now`: all three are host hooks." ECMA-262 has no hook for `Math.random` (implementation-defined) or `Date.now` (none until Temporal merges), and `performance.now` belongs to HTML, not the engine | R0-H6; amend Stage 1 3.3's row to name the routes |
| 2 | JS spec 4.2 and 6.14 | "16 host hooks (17 with Temporal)", while 6.14 adopts Dynamic Code Brand Checks for Trusted Types, which adds HostGetCodeForEval and changes HostEnsureCanCompileStrings's signature | R0-H1: 18 methods; amend 4.2 and 6.14's count |
| 3 | JS spec 9.2, JS-M0 row | The core set lists FinalizationRegistry creation but no callable, and the FinalizationRegistry constructor throws without one; the plan's probe also draws `Math.random` and reads the clock, which the row does not list | R0-B8, R0-B9: add `%CreateHostFunction`, `%MathRandom` and `%DateNow`; amend the row |
| 4 | The plan (no host hooks until JS-M1) | The core set's FinalizationRegistry and WeakRef reach HostMakeJobCallback, HostCallJobCallback, HostEnqueueFinalizationRegistryCleanupJob and ClearKeptObjects at JS-M0 | R0-H9: a test host at JS-M0 |
| 5 | JS spec 6.3 | The verifier checks "stack-height invariants"; a register machine has no operand stack | The analogue is register-list and frame-size bounds ([bytecode note](bytecode-and-core-set.md) Section 8); reword 6.3 |
| 6 | `hg-substrate` and `hg-substrate-sim` | `Entropy`'s documentation says adding a consumer does not perturb other sequences, but the simulator's stream counter is shared across purposes | Q-R0-8: a keyed stream |
| 7 | The `substrate/js-m0-decisions` branch | `DecisionLog` has `record` only; the plan's Phase 0 item 3 also needs `apply_at` for replay | Add it with Phase 0, before replay depends on weak clearing |
| 8 | ECMA-262 Annex D.1 (informative, external) | Lists InitializeHostDefinedRealm, which is not a host-defined operation, and omits HostEnsureCanAddPrivateElement, which is | None needed: this audit counts clause types; worth reporting upstream |
| 9 | HTML 8.1.6 against the Dynamic Code Brand Checks proposal (external) | HTML's HostEnsureCanCompileStrings has seven parameters, without `direct`; the proposal's has eight. HTML's prose calls the hooks "implementation-defined", ECMA-262 "host-defined" | The trait takes the superset (Q-R0-7) |

## Open questions for the owner

1. **Amendments.** Accept contradictions 1–3 and 5 as amendments to Stage 1 3.3 and the JS spec (4.2, 6.3, 6.14 and the JS-M0 row), applied in a docs change after this review is accepted?
2. **Entropy.** Add `Entropy::stream_keyed(purpose, key)` in Phase 0 (Q-R0-8; recommended), or derive realm streams inside the engine?
3. **Two Stage 3 hooks.** R0-H1 freezes HostGetCodeForEval and the eight-parameter HostEnsureCanCompileStrings while Dynamic Code Brand Checks is at Stage 3. JS spec 6.14 already adopts the proposal; confirm, since JS spec 6.14 otherwise waits for Stage 4 on proposals that can still change.
4. **System time zone exposure** (Q-R0-9): a recorded per-agent value is supported either way; which spec decides whether pages see the device's zone?

The remaining open questions (Q-R0-1 to Q-R0-7) are measurements or later-milestone choices, listed in the notes.
