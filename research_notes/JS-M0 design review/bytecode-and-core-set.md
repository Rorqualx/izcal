# JS-M0 design review: the bytecode format and the core operation set

*Part 1 of the JS-M0 design review for Izcal's JavaScript engine. Date: 31 October 2026. Status: draft 0.2, for owner review; verified (fact-check and design critique of 31 October 2026, recorded in the [README](README.md)'s header). Relates to: the [JS spec](../../reports/Izcal%20JavaScript%20engine.md) (draft 0.4.4 on `main`; the JS-M0 row cited is 0.4.5's, on the unmerged `docs/js-m0-amendments` branch) 6.1, 6.2a, 6.3, 6.5, 6.6, 6.9 and 9.2 (the JS-M0 row), JS-D1, JS-D5, JS-D14, JS-D15, JS-D16, JS-D19; Stage 1 3.3; [coding standard](../../reports/Izcal%20coding%20standard.md) CS-ST-17, CS-D4; G13, G16, G17, G18. Index and decision list: [README](README.md).*

*Revision 0.2 (31 October 2026): phase-8 findings applied. The frame is based on part 2's frame record `R` with part 2's header (R2-9, R2-11), and Q-R0-3 is closed; the mnemonics are renamed to Izcal's own; the canonical progress counter and decision-applicable points are defined (R0-B11); the verifier gains a separate liveness pass and a forward-handler rule; engine-thrown errors are charged; the probe example keeps its registry alive; Node's string limit is corrected.*

*Revision 0.1 (31 October 2026): first draft.*

This note fixes the bytecode format of `hg-js-bytecode` and the exact operation set JS-M0's reference interpreter runs. The format is a register machine with an accumulator, a byte stream of one-byte opcodes and operands of one byte by default, widened for one instruction by a `Wide2` or `Wide4` prefix. Register operands are signed slot indices relative to the frame record, so one formula addresses locals, the receiver and the parameters. Constants are typed in the trusted region and held as values in a per-(unit, agent) array in the cage. Every property and arithmetic instruction carries an IC slot index from the first day, so JS-M1 adds ICs without changing an encoding. Jumps carry unsigned distances whose direction the opcode fixes, and only `LoopBack` goes backward, so every loop passes an interrupt check. The core set has 49 opcodes and 9 intrinsics, called through one `CallIntrinsic` opcode; it adds three things the JS-M0 row of the JS spec does not list but its probe needs: a host function object, `Math.random` and the clock.

## 1. Scope

**Fixed here:** the instruction encoding; operand kinds and widths; the register file and frame layout as the interpreter sees them; the accumulator's rules; the constant pool; IC slot indices; jump encoding; the bytecode unit and its position-independent image form; the JS-M0 core operation set with each opcode's operands, accumulator effect, effects and ECMA-262 anchors; the text assembler; and what the JS-M1 verifier will check.

**Fixed in part 2 of the review, and consumed here:** the code-table entry format, frame states, the deopt model, stack maps and the return status word (JS-D19). Section 4.3 states the interface this note needs from part 2.

**Not fixed at JS-M0:** opcodes for calls, closures, scopes, globals, exceptions handlers, iteration, classes and the rest of the language. They arrive with the parser at JS-M1. The format reserves room for them (Section 4.1) and nothing in it changes when they arrive.

## 2. Constraints carried in

| Source | Constraint on the format |
|---|---|
| JS spec 6.3 | A register machine with an accumulator and variable-width operands, 8-bit by default with wide prefixes; bytecode, constant pools and IC slot indices are one compact, position-independent format that the code cache and startup image store as is |
| JS spec 6.3, JS-D16, H-2 | Verified bytecode lives in the trusted region, outside the cage; a register operand indexes the native interpreter frame, so an operand must never be read from the cage |
| JS spec 6.1, JS-D1 | One 32-bit tagged representation for heap, stack and interpreter registers: a small integer has tag 0 and 31 bits; a reference has tag 1 |
| JS spec 6.5 | Interpreter and baseline frames share one layout; every register slot is a tagged 32-bit value, so the frame is scanned exactly |
| JS spec 6.4, JS-D19 | No frame transition writes a return address; unwinding is return-based with "exception pending" in a status word |
| JS spec 6.9 | GC debt is counted in canonical allocation units fixed by each description; stack overflow is decided by a canonical frame budget charged per function |
| JS spec 6.2a | The register-and-accumulator choice is decided by bytecode size and T0 dispatch counts on the corpus at JS-M2 |
| G18 | The verifier and the image loader use checked arithmetic on every size they read |
| JS spec 9.2 | At JS-M0 the format is fixed with a test assembler; the verifier and the emitter from a parser are JS-M1 |

## 3. What came before, and why

| Design | What it did | Why that way | What it tells Izcal |
|---|---|---|---|
| Stack bytecode (the JVM; Lua before 5.0) | Operands implicit on an operand stack | Small instructions, simple compilers | A register VM executed about 47% fewer instructions for about 25% more bytecode in Shi et al.'s measurement ([VEE 2005](https://usenix.org/legacy/events/vee05/full_papers/p153-yunhe.pdf)) |
| Lua 5.0 (2003) | Register VM with fixed 32-bit instructions: a 6-bit opcode and 8- or 9-bit fields | One machine word per instruction makes operand decoding cheap logical operations, and avoids unaligned multi-byte operand fetches; "the total code size is not much larger" ([Ierusalimschy et al., JUCS 2005](https://www.lua.org/doc/jucs05.pdf)) | The strongest alternative to a byte stream: simplest decode, fixed field widths |
| SquirrelFish (2008) | Register, direct-threaded bytecode replacing a tree walker ([WebKit](https://webkit.org/blog/189/announcing-squirrelfish/)) | Tree walking costs nodes that do no work | Registers plus a compact encoding beat both trees and stacks |
| Ignition (2016) | Register machine with an implicit accumulator; bytecode 25–50% the size of the baseline machine code it replaced ([V8](https://v8.dev/blog/ignition-interpreter)) | Memory on low-end phones | The accumulator "reduces the size of bytecodes by avoiding the need to specify specific register operands" ([V8](https://v8.dev/blog/ignition-interpreter)) |

M-1 part 3 applies: these are published prose and measurements. No engine's encoding tables, opcode numbering or source were read, and none is reproduced. One prefix that widens every operand of the next instruction is the JVM's published `wide` instruction ([JVMS 6.5](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.wide)), here with two widths. **The mnemonics are Izcal's own since revision 0.2:** the phase-8 fact-check and critique found that most of draft 0.1's mnemonics matched another engine's bytecode names, which no permitted source was cited for, so every such name was replaced by one chosen for what the operation does (`LoadReg`, `StoreReg`, `LoopBack`, `BranchFalsy`, `Wide2` and the rest).

## 4. Decisions

### 4.1 Instruction encoding (R0-B1)

**A byte stream.** An instruction is one opcode byte followed by its operands. Each operand kind has a width of 1 byte by default. A **`Wide2`** prefix (`0xFE`) makes every operand of the next instruction 2 bytes, and **`Wide4`** (`0xFF`) makes them 4 bytes. One prefix scales all operands of one instruction, so decoding reads one scale and never a per-operand width. A prefix must be followed by an opcode with at least one operand, and a prefix never follows a prefix.

**Opcode space.** `0x00` is **`Invalid`**: zeroed or truncated memory never decodes as a valid instruction, and the verifier rejects it. `0x01`–`0x31` are the JS-M0 core set (Section 5). `0xFD` is reserved for an **`Extended`** prefix that introduces a second opcode byte, so the space never runs out. `hg-opgen` assigns numbers in declaration order from the `.hgop` sources (part 1's other note, [hgop-language.md](hgop-language.md)), and the **format identifier** of a bytecode unit is a hash of the generated opcode table, so a unit from another opcode table is rejected on load. Numbers need no stability across builds: images and code-cache entries are bound to the engine build and discarded on mismatch (JS spec 6.11).

**Byte order and alignment.** Multi-byte operands are little-endian, which all three ISAs are, and unaligned. On x86-64 and AArch64 an unaligned load is ordinary. On riscv64, RVA23 requires misaligned loads to work (Zicclsm) but, in the profiles' RVA22 text, they "might execute extremely slowly" ([RISC-V profiles](https://github.com/riscv/riscv-profiles/blob/main/src/profiles.adoc); RVA23 itself states only the mandate; [RV 7.3](../Izcal%20JavaScript%20engine/riscv64.md)), so the riscv64 T0 generator reads 2- and 4-byte operands with byte loads and shifts. Widened operands are expected to be rare; their share is measured at JS-M1 (open question Q-R0-1).

**Why a byte stream and not fixed 32-bit words.** Fixed words decode with cheapest operations and no alignment question, which is Lua's argument ([JUCS 2005](https://www.lua.org/doc/jucs05.pdf)). Against it: with an accumulator most JS-M0 instructions have zero or one register operand plus a one-byte IC slot, so they are 1–3 bytes, and a word per instruction would roughly double the bytecode of cold code, which dominates real pages ([JS spec 4.1 U1, U5](../../reports/Izcal%20JavaScript%20engine.md)). Fixed 8- or 9-bit fields also cap registers and constants per instruction, so long functions still need a second form. T0 is generated assembly with next-instruction pre-decode (JS spec 6.3), which hides most of the byte stream's decode cost. The reversal trigger is the JS spec's own (6.2a): bytecode size and T0 dispatch counts on the corpus at JS-M2.

### 4.2 Operand kinds

| Kind | Assembler spelling | Meaning | Encoded as | Checked by the verifier (JS-M1) |
|---|---|---|---|---|
| `reg` | `r3`, `a0`, `this` | A frame register (Section 4.3) | Signed | Inside the frame's register or parameter range, never the header |
| `cnt` | `3` (after `:` in a list) | A count of consecutive registers | Unsigned | The whole list lies inside one range |
| `imm` | `#-5` | A signed integer immediate | Signed | `LoadInt`: inside the small-integer range [−2^30, 2^30 − 1] |
| `const` | `k4` | A constant-table index | Unsigned | Below the constant count, and the constant's kind is one the opcode accepts |
| `slot` | `s1` | An IC slot index | Unsigned | Below the slot count, and the slot's kind is the opcode's |
| `fwd` | `@done` | A forward jump distance | Unsigned | Lands on an instruction start, after this instruction |
| `back` | `@loop` | A backward jump distance | Unsigned | Lands on an instruction start, at or before this instruction |
| `intr` | `%WeakRefDeref` | An intrinsic identifier | Unsigned | A known intrinsic, with the list's count equal to its arity |

Every kind scales with the prefix. A signed operand of width *w* covers [−2^(8w−1), 2^(8w−1) − 1]; an unsigned one covers [0, 2^(8w) − 1].

### 4.3 Registers and the frame as the interpreter sees it (R0-B2)

A **register operand is a signed slot index relative to the frame record `R`**, in 4-byte slots. `R` is the address of the saved caller frame pointer, fixed per ISA by part 2 (R2-9, [frames note](frames-deopt-stackmaps.md) Section 3): `R = fp` on x86-64 and AArch64, `R = fp − 16` on riscv64, whose psABI frame pointer points at the canonical frame address. Locals are non-negative indices; the receiver and parameters are negative indices beyond the frame header. One formula gives the address of every register, so T0 needs no branch to tell a parameter from a local:

```
  address(v) = R − 36 − 4·v          (R: the frame record, R2-9; slots are 32-bit)

  higher addresses
    a(p−1) ... a1  a0        R + 20 + 4·j       v = −14 − j   tagged, in the caller's outgoing area of
    receiver (this)          R + 16             v = −13       max(argc, p) slots; p = the code-table
                                                              entry's parameter count (never a cage value)
    return address           R + 8              (header)      pushed by a genuine call or stored by the
    caller's fp              R + 0              (header)      prologue (JS-D19); 8 bytes each
    status slot              R − 4              (header)      u32 flag bits (R2-11)
    frame identity           R − 8              (header)      u32: kind and trusted function ID (R2-9)
    actual argument count    R − 12             (header)      u32
    function                 R − 16             (header)      tagged
    bytecode offset          R − 20             (header)      u32, saved at calls and GC points
    feedback vector          R − 24             (header)      tagged; undefined until JS-M1
    accumulator spill        R − 28             (header)      tagged
    context                  R − 32             (header)      tagged; undefined at JS-M0
    r0                       R − 36             v = 0         tagged
    r1 ... r(n−1)            R − 36 − 4·i       v = i         tagged, n = the unit's register count
  lower addresses
```

The header is part 2's: the first four slots below `R` are the header every tier shares (R2-9), and the next four are the T0 and T1 body's fixed part, in part 2's order. Draft 0.1 of this note had the same size (eight slots below `R`, so `r0` at `R − 36`) with other contents and based on `fp`; the phase-8 critique found that a riscv64 T0 built from it would address every slot 16 bytes off, and that its scan map would read part 2's `argc` as a reference. This revision adopts part 2's layout and closes Q-R0-3.

- **The header is a hole in the index space.** Indices −12 to −1 address the header (`R − 32` to `R + 15`), and the verifier rejects them; the header is reached only by dedicated operations (JS-M1's `LoadContext` and the like), never through a register operand.
- **The scan map is fixed.** The collector scans the receiver, the incoming argument slots (`max(argc, p)` of them, from the header's `argc` and the trusted `p`), the locals and the four tagged header slots (function, feedback vector, accumulator spill, context); the four untagged slots (status, identity, `argc`, bytecode offset) are at fixed offsets. That keeps JS spec 6.5's "the frame is scanned exactly" while the header holds machine words.
- **The parameter count comes from the code-table entry**, never from the cage (JS spec 6.1 point 4). With narrow operands a function addresses 128 locals and 115 parameters; `Wide2` covers 32,768 and 32,755.
- **All registers start as `undefined`.** The prologue fills locals with `undefined` and the caller fills missing arguments with `undefined` (R2-21). So every slot always holds a valid tagged value, and the safety verifier needs no definedness analysis. JS-M1's hole value for the temporal dead zone is a static root that the emitter checks before use; it never needs the verifier.
- **How an instruction reaches its unit and constants.** The frame identity's trusted function ID locates the sealed unit. The function object at `R − 16` holds the function's per-agent unit record, which holds the constant array (Section 4.5) and, from JS-M1, the feedback vector. The frame therefore roots everything an instruction reads, and no header slot is spent on the constant array.
- **Extra arguments** beyond the parameter count (for `arguments` and rest parameters) stay in the caller's area, addressed from `argc` (R2-21).

**The reference interpreter** in Rust has no native frame. Its `Frame` is a root scope of `hg-heap` holding the same slots under the same indices, so a unit means the same thing in the reference interpreter and in T0.

### 4.4 The accumulator (R0-B3)

The accumulator is an implicit operand: the right-hand value of a binary operation, the condition of a conditional jump, the value of a store and the result of most instructions. **Why the left operand is the register.** ECMA-262 evaluates the left operand first, and the right operand's evaluation may run arbitrary code that overwrites the accumulator. So the emitter stores the left value in a register and leaves the right value in the accumulator, and a binary instruction names one register. This is the evaluation order of [EvaluateStringOrNumericBinaryExpression](https://tc39.es/ecma262/#sec-evaluatestringornumericbinaryexpression) and of the [relational operators](https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation) (left operand evaluated, then right), read as an encoding rule; [ApplyStringOrNumericBinaryOperator](https://tc39.es/ecma262/#sec-applystringornumericbinaryoperator) and [IsLessThan](https://tc39.es/ecma262/#sec-islessthan) then order the conversions.

**The accumulator is a root.** T0 keeps it in a machine register and spills it to its header slot at every GC point, call and throw, so the collector finds it there. The reference interpreter keeps it in the frame's root scope at all times. A description may not hold a heap value across a GC point except in the frame or as an argument of that GC point ([hgop-language.md](hgop-language.md), R0-O6), which is what makes this spill sufficient.

**Canonical number representation.** A Number whose value is an integer in [−2^30, 2^30 − 1] and is not −0 is **always** a small integer; every other Number is a heap number. Every tier and every description produces the same representation, so heap-state hashes agree across tiers (JS-51) and `===` on small integers is one comparison. Heap numbers are boxes in the cage, a non-semantic allocation that charges no canonical units (Section 6).

### 4.5 Constants (R0-B4)

A unit's **constant table** has two halves. The **kinds** (for example `number`, `string`, `key`) live in the trusted unit, with any scalar payload the verifier needs. The **values** live in a **constant array in the cage**, one per (unit, agent), materialized when the unit is first used in that agent: numbers become small integers or heap numbers, strings become strings, keys become interned atoms. `LoadConst k` loads element *k* of the array; the trusted kind says what the element should be.

| Option | For | Against |
|---|---|---|
| **(a) Kinds trusted, values in a cage array** (recommended) | Sealed bytecode (R2-18) stays immutable after verification, position-independent and shareable between agents of one site (JS-D17); an atom is per agent heap, so the value must be too; the array is rooted by the function's per-agent unit record (Section 4.3) | One more load per constant; a corrupted cage can swap a constant's value, which is a wrong answer inside the cage (H-2 allows it): every consumer treats the value as an untrusted JS value, and no operand or length is taken from it |
| (b) Values as compressed references in the trusted region | One load fewer | Sealed data would need collector writes on every evacuation, or pinned objects; one copy per agent anyway, since atoms are per agent. Part 2's traced per-code-object constant table (R2-19, runtime-trusted) is this option for compiled code, where the load it saves is worth a table per code object |
| (c) Literals inline in the bytecode | No table | Doubles and strings need materialization at each execution; bytecode stops being shareable between agents |

The same question arises for heap constants in IC programs (shapes). Part 2 answers it the way R2-19 answers it for compiled code: IC programs live in the runtime-trusted arena (R2-18), and their heap constants in a traced per-site constant table of the same class ([code-table note](code-table-and-ledger.md), "Heap constants").

### 4.6 IC slots, reserved now (R0-B5)

Every instruction that will have an inline cache or a value profile carries a `slot` operand from JS-M0: the property instructions, the binary, unary and comparison instructions, `ToNumber`, `ToNumeric` and `LoopBack` (its back-edge counter). The unit's trusted **slot table** gives each slot a kind (`load`, `store`, `keyed-load`, `keyed-store`, `binary`, `unary`, `compare`, `loop`), and the verifier checks that each instruction's slot has its kind. The slot index addresses both the per-site table of attached IC programs (runtime-trusted, R2-18) and the feedback vector (cage, hints only). At JS-M0 neither exists at run time and the reference interpreter ignores the operand; the assembler assigns slots and the unit records them. Carrying the operand now costs one byte per such instruction and means JS-M1 adds ICs without changing any encoding or the image format.

### 4.7 Jumps and control (R0-B6)

- **Distances, not offsets.** A jump operand is an unsigned distance from the first byte of the jump instruction, its prefix included. The opcode fixes the direction: `Jump` and the conditional jumps go forward, `LoopBack` goes backward. The encoding holds no address, so units are position-independent, and an unsigned distance doubles the reach of each width.
- **Only `LoopBack` goes backward, and it is unconditional.** Every cycle in a unit's control flow therefore passes a `LoopBack`, which performs the interrupt check and is a safepoint (JS spec 6.5, "handshake"). Loops whose condition is at the bottom compile as a forward conditional exit plus `LoopBack`. The verifier checks the direction structurally, so no loop can avoid an interrupt check.
- **Conditional jumps come in two forms.** `BranchTrue` and `BranchFalse` test for the Boolean value exactly and are emitted after a comparison, whose result is always a Boolean; `BranchTruthy` and `BranchFalsy` apply [ToBoolean](https://tc39.es/ecma262/#sec-toboolean). Both are total: on a non-Boolean value the exact forms simply do not match, which is still a defined result.
- **Termination.** The last instruction of a unit is `Return`, `Jump` or `LoopBack`, so control never runs off the end.
- **Exceptions.** A unit has a **handler table** of (start, end, handler, kind) ranges over instruction starts. It is empty at JS-M0, which has no `try`: an abrupt completion leaves the unit through the return-based unwinding of JS-D19 with "exception pending", and reaches the host as the job's throw completion. **Every handler lies after the end of its range**, so entering a handler never closes a cycle that bypasses `LoopBack`; the verifier checks it from JS-M1.

### 4.8 The bytecode unit and its image form (R0-B7)

A **unit** is one function's bytecode. Its trusted form, written into the trusted region through the write window after verification, holds:

| Field | Content |
|---|---|
| Format identifier | Hash of the generated opcode table (Section 4.1) |
| Flags | Strict mode (a property of the function's code, never an operand); kind (script, function, module at JS-M1) |
| Counts | Parameter count, register count, slot count, constant count, handler count, code length; all checked against limits with checked arithmetic (G18) |
| Code | The instruction bytes |
| Slot table | One kind byte per slot |
| Constant kinds | One kind and an optional scalar payload per constant |
| Handler table | Empty at JS-M0 |
| Source positions | Bytecode offset to source position, delta-encoded; JS-M1 |
| Canonical frame charge | The static part of the canonical depth charge (JS spec 6.9, R2-16): `32 + 4 × (registers + params)` canonical depth bytes, the same on every ISA and tier; the call adds `4 × max(0, argc − params)` for extra arguments. Depth bytes are not the GC's canonical allocation units (Section 6) |
| Liveness | For each call and safepoint offset, the set of live registers and whether the accumulator is live, from the verifier's liveness pass (Section 8); part 2's frame states mark the rest `Dead` |

The **image form** stored in the startup image and the code cache is the same content as a byte string with no addresses: a fixed little-endian header, then sections located by offsets relative to the image start (code, slot kinds, constant descriptors, a literal pool holding number bits and string code units, handlers, source positions). Loading parses it as hostile input with checked arithmetic (G18), verifies it, copies it into the trusted region and materializes the constant array in the cage. Nothing in the image is a pointer or a heap reference, which is what lets the code-cache builder (JS-D21) write it in another process.

### 4.9 Intrinsics through one opcode (R0-B8)

JS-M0 has no calls, yet the probe needs `WeakRef`, `FinalizationRegistry`, `Math.random` and the clock. Giving each its own opcode would leave dead opcodes when JS-M1 makes them ordinary built-in functions. Instead **`CallIntrinsic %id, rN:count`** runs an **intrinsic**: a built-in operation written in `.hgop` (JS-D14) and called with the arguments in a contiguous register list. At JS-M1 the same descriptions become the bodies of `WeakRef.prototype.deref` and the rest, reachable through ordinary calls, and `CallIntrinsic` stays as the emitter's direct route to engine operations. An intrinsic runs inside the calling frame; it is not a call, pushes no frame and charges no depth. When JS-M1 makes an intrinsic's description the body of a callable built-in, that built-in gets the fixed frame charge R2-16 asks for, stated in its description (`frame_charge`, [hgop-language.md](hgop-language.md) Section 3).

### 4.10 The canonical progress counter and decision-applicable points (R0-B11)

JS spec 6.9 logs each decision point at (agent, progress counter), and JS-51 requires a recording to replay at any tier, so the counter must count something every tier sees identically. **The progress counter is a per-agent u64 that counts bytecode events: +1 at each `LoopBack` executed and +1 at each logical function entry** (at JS-M0, a job's entry). T0 and T1 add in the `LoopBack` handler and the prologue. T2 carries the counter as an SSA value through its inlining tree, adds 1 at each inlined entry and each loop back-edge, and stores it to the agent before each real call and poll, as it does the depth counter (R2-16). Counting every operation was rejected: T2 removes and merges operations, so it would have to rebuild a per-operation count at every poll.

**Decisions are applied only at decision-applicable points:** a `LoopBack`'s interrupt check, a logical function entry, and the end of a job. When a `charge` takes GC debt to its trigger, it sets a request; the cycle that the decision log records as `GC_COMPLETE`, with its weak clearing and cleanup enqueueing (R0-H7), runs at the next decision-applicable point, which every tier reaches at the same counter value. T2 therefore polls at an inlined function entry whose inlining tree contains a `charge` (one load and branch). A collection forced earlier by real memory pressure, never by canonical debt, treats every weak target as strong and clears nothing, so script cannot observe it and it needs no log entry. heap.md (on `js/hg-heap`) says a cycle starts when the debt reaches the trigger (H-8) and that the interpreter advances the counter with each operation (H-9); the heap change aligns both with this section (README, cross-branch items).

## 5. The JS-M0 core operation set (R0-B9)

**The value domain at JS-M0.** Undefined, Null, Boolean, Number (small integers and heap numbers), String, and objects of four kinds: **dictionary objects** (ordinary objects in table mode, with data properties only, prototype null or another dictionary object), **WeakRef**, **FinalizationRegistry** and **host function objects**. BigInt and Symbol values cannot be created at JS-M0: no constant kind and no intrinsic produces them. The descriptions still handle them on every path, through runtime calls, so no description changes at JS-M1; the operation-level oracle (plan Phase 1) generates operands from the JS-M0 domain only.

**One minimal realm per agent.** `WeakRef` and `FinalizationRegistry` creation reads `%WeakRef.prototype%` and `%FinalizationRegistry.prototype%` ([OrdinaryCreateFromConstructor](https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor)), and `Math.random` is per realm ([Math.random](https://tc39.es/ecma262/#sec-math.random)). So JS-M0 creates one Realm Record per agent holding only those prototypes (empty dictionary objects) and the realm's random stream. Primitive receivers of property access need the realm's wrapper prototypes, which are JS-M1; at JS-M0 that path is a runtime call, and the oracle does not generate it.

**Opcodes.** Width "s" means scaled by the prefix. Every anchor is a clause of ECMA-262 at [tc39.es/ecma262](https://tc39.es/ecma262/) (the ES2027 draft on 31 October 2026); [hgop-language.md](hgop-language.md) gives the per-path anchors of six of them in full.

| Op | Mnemonic | Operands | Accumulator | Effects (union over paths) | ECMA-262 anchors |
|---|---|---|---|---|---|
| 0x00 | `Invalid` | — | — | Rejected by the verifier | — |
| 0x01 | `LoadUndefined` | — | out | — | [Undefined type](https://tc39.es/ecma262/#sec-ecmascript-language-types-undefined-type) |
| 0x02 | `LoadNull` | — | out | — | [Literals](https://tc39.es/ecma262/#sec-literals-runtime-semantics-evaluation) |
| 0x03 | `LoadTrue` | — | out | — | [Literals](https://tc39.es/ecma262/#sec-literals-runtime-semantics-evaluation) |
| 0x04 | `LoadFalse` | — | out | — | [Literals](https://tc39.es/ecma262/#sec-literals-runtime-semantics-evaluation) |
| 0x05 | `LoadInt` | `imm` s | out | — | [Literals](https://tc39.es/ecma262/#sec-literals-runtime-semantics-evaluation) |
| 0x06 | `LoadConst` | `const` s (`number`, `string`) | out | — | [Literals](https://tc39.es/ecma262/#sec-literals-runtime-semantics-evaluation) |
| 0x07 | `LoadReg` | `reg` s | out | — | none: register transfer |
| 0x08 | `StoreReg` | `reg` s | in | writes the register | none: register transfer |
| 0x09 | `Copy` | `reg` s (source), `reg` s (destination) | — | writes the register | none: register transfer |
| 0x0A | `ToBoolean` | — | in, out | — | [ToBoolean](https://tc39.es/ecma262/#sec-toboolean), [Annex B IsHTMLDDA](https://tc39.es/ecma262/#sec-IsHTMLDDA-internal-slot-to-boolean) |
| 0x0B | `ToNumber` | `slot` s (`unary`) | in, out | throw, call user code, internal allocation | [ToNumber](https://tc39.es/ecma262/#sec-tonumber), [unary +](https://tc39.es/ecma262/#sec-unary-plus-operator-runtime-semantics-evaluation) |
| 0x0C | `ToNumeric` | `slot` s (`unary`) | in, out | throw, call user code, internal allocation | [ToNumeric](https://tc39.es/ecma262/#sec-tonumeric) |
| 0x0D | `ToString` | — | in, out | allocate, throw, call user code | [ToString](https://tc39.es/ecma262/#sec-tostring), [Number::toString](https://tc39.es/ecma262/#sec-numeric-types-number-tostring) |
| 0x0E | `Not` | — | in, out | — | [Logical NOT](https://tc39.es/ecma262/#sec-logical-not-operator-runtime-semantics-evaluation) |
| 0x0F | `TypeOf` | — | in, out | — (result strings are static roots) | [typeof](https://tc39.es/ecma262/#sec-typeof-operator-runtime-semantics-evaluation), [Annex B](https://tc39.es/ecma262/#sec-IsHTMLDDA-internal-slot-typeof) |
| 0x10 | `Add` | `reg` s, `slot` s (`binary`) | in, out | allocate, throw, call user code | [ApplyStringOrNumericBinaryOperator](https://tc39.es/ecma262/#sec-applystringornumericbinaryoperator), [Number::add](https://tc39.es/ecma262/#sec-numeric-types-number-add), [string-concatenation](https://tc39.es/ecma262/#string-concatenation) |
| 0x11 | `Sub` | `reg` s, `slot` s | in, out | throw, call user code, internal allocation | [Number::subtract](https://tc39.es/ecma262/#sec-numeric-types-number-subtract) |
| 0x12 | `Mul` | `reg` s, `slot` s | in, out | as `Sub` | [Number::multiply](https://tc39.es/ecma262/#sec-numeric-types-number-multiply) |
| 0x13 | `Div` | `reg` s, `slot` s | in, out | as `Sub` | [Number::divide](https://tc39.es/ecma262/#sec-numeric-types-number-divide) |
| 0x14 | `Mod` | `reg` s, `slot` s | in, out | as `Sub` | [Number::remainder](https://tc39.es/ecma262/#sec-numeric-types-number-remainder) |
| 0x15 | `Exp` | `reg` s, `slot` s | in, out | as `Sub`; the value is implementation-approximated, so it is a runtime call to `hg-math` | [Number::exponentiate](https://tc39.es/ecma262/#sec-numeric-types-number-exponentiate) |
| 0x16 | `BitAnd` | `reg` s, `slot` s | in, out | as `Sub` | [Number::bitwiseAND](https://tc39.es/ecma262/#sec-numeric-types-number-bitwiseAND), [ToInt32](https://tc39.es/ecma262/#sec-toint32) |
| 0x17 | `BitOr` | `reg` s, `slot` s | in, out | as `Sub` | [Number::bitwiseOR](https://tc39.es/ecma262/#sec-numeric-types-number-bitwiseOR) |
| 0x18 | `BitXor` | `reg` s, `slot` s | in, out | as `Sub` | [Number::bitwiseXOR](https://tc39.es/ecma262/#sec-numeric-types-number-bitwiseXOR) |
| 0x19 | `Shl` | `reg` s, `slot` s | in, out | as `Sub` | [Number::leftShift](https://tc39.es/ecma262/#sec-numeric-types-number-leftShift) |
| 0x1A | `Sar` | `reg` s, `slot` s | in, out | as `Sub` | [Number::signedRightShift](https://tc39.es/ecma262/#sec-numeric-types-number-signedRightShift) |
| 0x1B | `Shr` | `reg` s, `slot` s | in, out | as `Sub` (results above 2^30 − 1 are heap numbers) | [Number::unsignedRightShift](https://tc39.es/ecma262/#sec-numeric-types-number-unsignedRightShift), [ToUint32](https://tc39.es/ecma262/#sec-touint32) |
| 0x1C | `Neg` | `slot` s (`unary`) | in, out | as `Sub` (−0 is a heap number) | [unary −](https://tc39.es/ecma262/#sec-unary-minus-operator-runtime-semantics-evaluation), [Number::unaryMinus](https://tc39.es/ecma262/#sec-numeric-types-number-unaryMinus) |
| 0x1D | `BitNot` | `slot` s (`unary`) | in, out | as `Sub` | [bitwise NOT](https://tc39.es/ecma262/#sec-bitwise-not-operator-runtime-semantics-evaluation), [Number::bitwiseNOT](https://tc39.es/ecma262/#sec-numeric-types-number-bitwiseNOT) |
| 0x1E | `Inc` | `slot` s (`unary`) | in, out | as `Sub` | [prefix ++](https://tc39.es/ecma262/#sec-prefix-increment-operator-runtime-semantics-evaluation), [postfix ++](https://tc39.es/ecma262/#sec-postfix-increment-operator-runtime-semantics-evaluation) |
| 0x1F | `Dec` | `slot` s (`unary`) | in, out | as `Sub` | [prefix −−](https://tc39.es/ecma262/#sec-prefix-decrement-operator-runtime-semantics-evaluation), [postfix −−](https://tc39.es/ecma262/#sec-postfix-decrement-operator-runtime-semantics-evaluation) |
| 0x20 | `LooseEq` | `reg` s, `slot` s (`compare`) | in, out (Boolean) | throw, call user code, internal allocation | [IsLooselyEqual](https://tc39.es/ecma262/#sec-islooselyequal) |
| 0x21 | `StrictEq` | `reg` s, `slot` s | in, out (Boolean) | — | [IsStrictlyEqual](https://tc39.es/ecma262/#sec-isstrictlyequal) |
| 0x22 | `Lt` | `reg` s, `slot` s | in, out (Boolean) | throw, call user code, internal allocation | [relational operators](https://tc39.es/ecma262/#sec-relational-operators-runtime-semantics-evaluation), [IsLessThan](https://tc39.es/ecma262/#sec-islessthan) |
| 0x23 | `Gt` | `reg` s, `slot` s | in, out (Boolean) | as `Lt` | as `Lt` (operands swapped, *leftFirst* false) |
| 0x24 | `Le` | `reg` s, `slot` s | in, out (Boolean) | as `Lt` | as `Lt` |
| 0x25 | `Ge` | `reg` s, `slot` s | in, out (Boolean) | as `Lt` | as `Lt` |
| 0x26 | `Jump` | `fwd` s | — | — | none: control flow ([if](https://tc39.es/ecma262/#sec-if-statement-runtime-semantics-evaluation)) |
| 0x27 | `LoopBack` | `back` s, `slot` s (`loop`) | — | interrupt check; GC point | [LoopContinues](https://tc39.es/ecma262/#sec-loopcontinues) |
| 0x28 | `BranchTrue` | `fwd` s | in | — | [if](https://tc39.es/ecma262/#sec-if-statement-runtime-semantics-evaluation) (ToBoolean of a Boolean is the identity) |
| 0x29 | `BranchFalse` | `fwd` s | in | — | as `BranchTrue` |
| 0x2A | `BranchTruthy` | `fwd` s | in | — | [ToBoolean](https://tc39.es/ecma262/#sec-toboolean) |
| 0x2B | `BranchFalsy` | `fwd` s | in | — | [ToBoolean](https://tc39.es/ecma262/#sec-toboolean) |
| 0x2C | `Return` | — | in | terminator | [return](https://tc39.es/ecma262/#sec-return-statement-runtime-semantics-evaluation) |
| 0x2D | `GetNamed` | `reg` s (object), `const` s (`key`), `slot` s (`load`) | out | reads properties, throw, call user code, internal allocation | [EvaluatePropertyAccessWithIdentifierKey](https://tc39.es/ecma262/#sec-evaluate-property-access-with-identifier-key), [GetValue](https://tc39.es/ecma262/#sec-getvalue), [OrdinaryGet](https://tc39.es/ecma262/#sec-ordinaryget) |
| 0x2E | `SetNamed` | `reg` s (object), `const` s (`key`), `slot` s (`store`) | in (the value; left unchanged) | writes properties, allocate, throw, call user code | [PutValue](https://tc39.es/ecma262/#sec-putvalue), [OrdinarySet](https://tc39.es/ecma262/#sec-ordinaryset) |
| 0x2F | `GetKeyed` | `reg` s (object), `slot` s (`keyed-load`) | in (key), out (value) | as `GetNamed`, plus [ToPropertyKey](https://tc39.es/ecma262/#sec-topropertykey) | [EvaluatePropertyAccessWithExpressionKey](https://tc39.es/ecma262/#sec-evaluate-property-access-with-expression-key), [GetValue](https://tc39.es/ecma262/#sec-getvalue) (ToObject at step 3.a before ToPropertyKey at 3.c) |
| 0x30 | `SetKeyed` | `reg` s (object), `reg` s (key), `slot` s (`keyed-store`) | in (the value; left unchanged) | as `SetNamed`, plus ToPropertyKey | [PutValue](https://tc39.es/ecma262/#sec-putvalue) |
| 0x31 | `CallIntrinsic` | `intr` s, `reg` s, `cnt` s | out | the intrinsic's | per intrinsic, below |
| 0xFD | (reserved) | — | — | `Extended` prefix, unused at JS-M0 | — |
| 0xFE | `Wide2` | — | — | prefix | — |
| 0xFF | `Wide4` | — | — | prefix | — |

Strictness, which decides whether a failed `[[Set]]` throws, comes from the unit's flags. At JS-M0 every property is a writable data property of an extensible object, so `[[Set]]` never fails; the descriptions handle the failing case anyway.

**After-call accumulator.** Every opcode that may call user code declares what the accumulator holds after the call returns, which is what a lazy deopt at that call restores (R0-O11, [frames note](frames-deopt-stackmaps.md) Section 5.2): `result` (the call's value) for every such opcode except `SetNamed` and `SetKeyed`, which declare `preserve` (the stored value, left unchanged). `CallIntrinsic` takes its intrinsic's mode.

**Intrinsics.**

| Intrinsic | Arguments | Result | Effects | Anchor |
|---|---|---|---|---|
| `%ObjectCreate` | prototype (null or a dictionary object) | a new dictionary object | allocate; throw TypeError | [Object.create](https://tc39.es/ecma262/#sec-object.create) with *Properties* undefined |
| `%WeakRefCreate` | target | a WeakRef | allocate; writes kept objects; throw TypeError | [WeakRef ( target )](https://tc39.es/ecma262/#sec-weak-ref-target), *NewTarget* = %WeakRef%; [CanBeHeldWeakly](https://tc39.es/ecma262/#sec-canbeheldweakly); [AddToKeptObjects](https://tc39.es/ecma262/#sec-addtokeptobjects) |
| `%WeakRefDeref` | the WeakRef | target or undefined | writes kept objects; throw TypeError | [WeakRef.prototype.deref](https://tc39.es/ecma262/#sec-weak-ref.prototype.deref), [WeakRefDeref](https://tc39.es/ecma262/#sec-weakrefderef) |
| `%FinalizationRegistryCreate` | cleanup callback | a FinalizationRegistry | allocate; host hook HostMakeJobCallback; throw TypeError | [FinalizationRegistry ( cleanupCallback )](https://tc39.es/ecma262/#sec-finalization-registry-cleanup-callback) |
| `%FinalizationRegistryRegister` | registry, target, held value, unregister token | undefined | allocate (one cell); writes weak state; throw TypeError | [register](https://tc39.es/ecma262/#sec-finalization-registry.prototype.register) |
| `%FinalizationRegistryUnregister` | registry, unregister token | Boolean | writes weak state; throw TypeError | [unregister](https://tc39.es/ecma262/#sec-finalization-registry.prototype.unregister) |
| `%MathRandom` | — | a Number in [0, 1) | reads a B1 input (the realm's entropy stream); internal allocation | [Math.random](https://tc39.es/ecma262/#sec-math.random): the 53 high bits of the realm stream's next 64-bit draw, times 2^−53 |
| `%DateNow` | — | a Number | reads a B1 input through the host hook HostSystemUTCEpochNanoseconds; internal allocation (a current time in milliseconds, about 1.8 × 10^12, is always a heap number) | [Date.now](https://tc39.es/ecma262/#sec-date.now), in Temporal's form [floor(ns / 10^6)](https://tc39.es/proposal-temporal/#sec-temporal-date.now) |
| `%CreateHostFunction` | a small integer naming an embedder function | a host function object | allocate | [CreateBuiltinFunction](https://tc39.es/ecma262/#sec-createbuiltinfunction) |

**Why the three additions.** The JS-M0 row of the JS spec lists `WeakRef` and `FinalizationRegistry` creation, but the FinalizationRegistry constructor throws a TypeError unless its argument is callable ([step 2](https://tc39.es/ecma262/#sec-finalization-registry-cleanup-callback)), and JS-M0 has no functions. A host function object is the smallest callable: an embedder-supplied Rust function behind a built-in function object, which the test host also uses to log what a cleanup job received. The plan's probe draws `Math.random` and reads the clock (Phase 7), which the row does not list either. The row's "a dictionary object" and "creation and deref" also do not name `%ObjectCreate`, `%FinalizationRegistryRegister` or `%FinalizationRegistryUnregister`; the first two are needed to observe a cleanup, and the third is kept so that the registry's description is complete when JS-M1 exposes it. The README records the amendment this implies, listing all six.

**Two jobs, not one.** The WeakRef constructor adds its target to the agent's kept objects, and only [ClearKeptObjects](https://tc39.es/ecma262/#sec-clear-kept-objects), which the host calls when a synchronous run of script ends, releases it. A target therefore cannot be cleared in the job that created its WeakRef. Probe programs are split into jobs, and the test host calls ClearKeptObjects between them (example in Section 7).

## 6. Canonical allocation units for the core set (R0-B10)

GC debt is charged in canonical units by each description (JS spec 6.9). This note fixes what counts. **A semantic allocation creates a value that script can hold whose size script controls**: an object, a property added to an object, a WeakRef, a registry or a registry cell, or a string produced by an operation. Charges depend only on semantic quantities, never on representation, so every tier charges the same whatever it allocates. **Heap-number boxes, ropes versus flat strings, transient wrapper objects that script never sees (for example the one [GetValue](https://tc39.es/ecma262/#sec-getvalue) step 3.a creates for a primitive base) and the growth of the kept-objects list charge nothing.**

| What | Units† (1 unit = 8 nominal bytes†) |
|---|---|
| Dictionary object at creation | 4† |
| Each own property added | 2† |
| String result of length *n* code units | 2 + ⌈*n* / 4⌉† (a nominal 16-byte header and 2 bytes per code unit, whatever the stored width; nominal, so heap.md H-4's real 8-byte header does not change it) |
| Error object the engine throws (TypeError, RangeError) | 6† (an object and its `message` property), plus the message string's units |
| WeakRef | 2† |
| FinalizationRegistry | 4† |
| Registry cell (one `register`) | 3† |
| Host function object | 4† |
| Heap number, rope node, flattening copy, transient wrapper, kept-objects entry | 0 |

The values are first-principles approximations of a plain layout; what matters for determinism is only that they are fixed per description. JS-M1 re-derives them from `hg-heap`'s real layouts, and changing a value changes the build, never a recording made with another build.

**The canonical string-length limit.** ECMA-262 allows strings of up to 2^53 − 1 code units ([String type](https://tc39.es/ecma262/#sec-ecmascript-language-types-string-type)) and has no clause for a smaller implementation limit. Every engine has one in practice: in Node 26.13.0 the longest string is 536,870,888 code units (2^29 − 24), and one more throws `RangeError: Invalid string length` (black-box, run on this Mac on 31 October 2026, and Node's documented [`buffer.constants.MAX_STRING_LENGTH`](https://nodejs.org/api/buffer.html#bufferconstantsmax_string_length)). Izcal's limit is **2^30 − 1 code units†**, about twice Node's, the same on every device: a string's length is then always a small integer, so `length` never boxes and length arithmetic in descriptions stays inside 32 bits. A concatenation past it throws RangeError on every tier.

## 7. The text assembler

One line per instruction. `hg-js-bytecode`'s assembler and disassembler are generated from the same opcode table as the interpreter's decoder, and a property test checks that disassembling and reassembling any valid unit gives the same bytes.

```ebnf
file        = { line } ;
line        = [ label ":" ] [ directive | instruction ] [ comment ] newline ;
directive   = ".function" name { attribute }
            | ".const" constref constkind literal
            | ".slot" slotref slotkind
            | ".end" ;
attribute   = "params=" uint | "registers=" uint | "strict" ;
constkind   = "number" | "string" | "key" ;
slotkind    = "load" | "store" | "keyed-load" | "keyed-store"
            | "binary" | "unary" | "compare" | "loop" ;
instruction = mnemonic [ scale ] [ operand { "," operand } ] ;
scale       = ".w" | ".x" ;                       (* force Wide2 or Wide4; default: the smallest that fits *)
operand     = register | register ":" uint         (* a register list: first register and count *)
            | "#" [ "-" ] uint                     (* immediate *)
            | constref | slotref | "@" label | "%" name ;
register    = "r" uint | "a" uint | "this" ;
constref    = "k" uint ;
slotref     = "s" uint ;
literal     = number | string ;                    (* JSON number and string syntax *)
comment     = ";" { any character but newline } ;
```

The assembler resolves labels with iterative sizing: it starts with every jump narrow and widens any whose distance does not fit, until no width changes. It rejects what the verifier will reject (Section 8), so tests cannot build an invalid unit by accident; a separate raw-bytes builder exists for the verifier's own negative tests at JS-M1.

**Example: the probe's two jobs.** The first job creates a target, watches it through a WeakRef and a registry, and returns a holder object with both, for the host to keep between jobs; holding the registry is what lets its cleanup run, since an unreachable registry dies with its cells. The second allocates until canonical GC debt requests a cycle, which runs at the next `LoopBack` (Section 4.10), then dereferences. When the second job ends, the test host runs the cleanup job the cycle enqueued, which calls embedder function 7 with held value 1.

```
.function setup params=0 registers=8
.const k0 key "weakRef"
.const k1 key "registry"
.slot s0 store
.slot s1 store
        LoadNull
        StoreReg r0                           ; r0 = null, the prototype
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r1                           ; r1 = target
        CallIntrinsic %WeakRefCreate, r1:1
        StoreReg r2                           ; r2 = WeakRef (the target is now kept until ClearKeptObjects)
        LoadInt #7
        StoreReg r3
        CallIntrinsic %CreateHostFunction, r3:1
        StoreReg r3                           ; r3 = cleanup callback: embedder function 7
        CallIntrinsic %FinalizationRegistryCreate, r3:1
        StoreReg r3                           ; r3 = registry
        Copy r1, r4                           ; r4 = target
        LoadInt #1
        StoreReg r5                           ; r5 = held value 1
        LoadUndefined
        StoreReg r6                           ; r6 = no unregister token
        CallIntrinsic %FinalizationRegistryRegister, r3:4
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r7                           ; r7 = holder
        LoadReg r2
        SetNamed r7, k0, s0                   ; holder.weakRef = the WeakRef
        LoadReg r3
        SetNamed r7, k1, s1                   ; holder.registry = the registry
        LoadReg r7
        Return                                ; the host roots the holder between jobs
.end

.function churn params=1 registers=3
.const k0 key "weakRef"
.slot s0 unary
.slot s1 compare
.slot s2 loop
.slot s3 load
        LoadNull
        StoreReg r0
        LoadInt #0
        StoreReg r1                           ; i = 0
loop:   CallIntrinsic %ObjectCreate, r0:1     ; garbage: 4 canonical units each
        LoadReg r1
        Inc s0
        StoreReg r1
        LoadInt #10000
        Lt r1, s1                             ; i < 10000
        BranchFalse @done
        LoopBack @loop, s2                    ; decision-applicable point (Section 4.10)
done:   GetNamed a0, k0, s3                   ; a0 = the holder
        StoreReg r2
        CallIntrinsic %WeakRefDeref, r2:1     ; undefined once a canonical cycle has completed
        Return
.end
```

## 8. What the JS-M1 verifier will check, and what makes each check linear

| Check | Format property that supports it |
|---|---|
| Header counts and section bounds, with checked arithmetic (G18) | Fixed header; sections located by offsets |
| Every opcode is defined; `Invalid` and the reserved `Extended` byte are rejected; a prefix precedes an opcode with operands and never another prefix | One opcode byte; prefixes at fixed values |
| Every instruction lies inside the code | Instruction length is a function of opcode and scale |
| Register operands lie in the parameter or local range and never in the header hole; register lists lie inside one range | Signed frame-relative indices (Section 4.3) |
| Constant, slot and intrinsic operands are in range, and the constant's kind, the slot's kind and the intrinsic's arity match the opcode | Kind tables in the trusted unit |
| Jump targets are instruction starts; forward jumps go forward, `LoopBack` goes backward | Unsigned distances, direction fixed by the opcode; one decode pass records instruction starts in a bitmap, a second checks targets |
| Control never runs off the end | The last instruction is a terminator |
| Handler ranges and targets are instruction starts, and each handler lies after its range's end (JS-M1 onward) | Handler table over instruction offsets |
| `LoadInt` immediates are small integers | Signed immediate operand |

The JS spec (6.3) lists "stack-height invariants" among the verifier's checks. A register machine has no operand stack; the corresponding property is that register lists and the frame size stay in range, which the table covers. Safety needs no dataflow analysis: registers and the accumulator are always valid tagged values (Section 4.3). The safety verifier is therefore two linear passes over the code.

**Liveness is a separate pass.** Part 2's frame states mark registers the bytecode no longer reads as `Dead` ([frames note](frames-deopt-stackmaps.md) Section 6), so a third pass, run after verification succeeds, computes register and accumulator liveness: a backward fixpoint over the unit's basic blocks, whose `LoopBack` edges make it iterative. Each block's live set only grows, so every pass but the last adds at least one bit and the pass terminates; a sweep in reverse block order needs two passes for code without loops and a few more per level of loop nesting. Its time is measured at JS-M1 (R0T-3, README). The result is stored in the sealed unit at each call and safepoint offset (Section 4.8). Liveness is not a safety property of the bytecode: a wrong `Dead` only rebuilds a live register as `undefined` after a deopt, which is a wrong answer, and the frame-state verifier (R2-15) and deopt-everywhere mode test it.

## 9. Alternatives

| Choice | Options | Recommendation | Cost | What would reverse it |
|---|---|---|---|---|
| Machine model | Stack; registers; registers with an accumulator | **Registers with an accumulator** (JS spec 6.2a) | One register operand per binary operation, held in a register by the emitter | 6.2a's trigger: bytecode size and T0 dispatch counts on the corpus at JS-M2 |
| Instruction width (R0-B1) | Fixed 32-bit words; a byte stream with scaling prefixes; a byte stream with per-operand variable-length integers | **Byte stream with scaling prefixes** | Unaligned operand reads (byte loads on riscv64); three handler variants per opcode in T0 | At JS-M2, T0 operand decoding measured above 10%† of dispatch time on the corpus while fixed words would grow the bytecode by less than 25%† |
| Register operands (R0-B2) | One signed frame-relative space; separate parameter and local operand kinds; parameters copied into locals | **One signed space with a header hole** | The verifier must reject the hole | A part 2 frame layout that cannot keep parameters and locals on either side of a fixed-size header |
| Constants (R0-B4) | Kinds trusted, values in a cage array; values in the trusted region; literals inline | **Kinds trusted, values in the cage** | One extra load per constant | Constant loads above 1%† of T0 time on the corpus at JS-M2, with the trusted-region alternative measured |
| IC slots (R0-B5) | Operand now; operand added at JS-M1 | **Now** | One byte per such instruction at JS-M0 | None at JS-M0; the encoding is the reason |
| Jumps (R0-B6) | Signed relative offsets; unsigned distances with the direction in the opcode; absolute offsets | **Unsigned, direction in the opcode, only `LoopBack` backward** | A conditional loop needs two jumps | — |
| WeakRef, registry, random, clock (R0-B8) | One opcode each; intrinsics through `CallIntrinsic` | **Intrinsics** | One dispatch through the intrinsic table | — |
| Number representation (R0-B3) | Canonical (a small integer whenever representable); either form allowed | **Canonical** | Every Number-producing path checks the range | — |
| Progress counter (R0-B11) | Per operation; per `LoopBack` and logical entry; per real call and back-edge | **Per `LoopBack` and logical entry, decisions applied only there** | T2 counts inlined entries and polls at those whose tree contains a `charge` | The counter or the extra polls above 1%† of T2 time on JetStream 3 at JS-M2 |

## 10. Open questions and flagged claims

- **Q-R0-1.** What share of instructions needs `Wide2` or `Wide4` on the corpus, and are misaligned 2- and 4-byte loads slow on the K3 (Zicclsm only promises correctness)? Measured at JS-M1 on the corpus and on the K3 board.
- **Q-R0-2.** Is 128 locals enough for most functions, as the narrow encoding assumes? Measured at JS-M1 from the emitter on the corpus.
- **Q-R0-3.** Closed in revision 0.2: part 2's header (R2-9, R2-11) has the same size, so the constant 36 and the parameter base −13 stand (Section 4.3).
- **Flagged.** The canonical units of Section 6 are first-principles estimates†, re-derived at JS-M1. The string-length limit is Izcal's choice†; ECMA-262 states no smaller limit, and the claim that engines throw past their own limit rests on one engine's black-box runs and documentation.
