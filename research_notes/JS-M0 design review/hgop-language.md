# JS-M0 design review: the `.hgop` operation description language

*Part 1 of the JS-M0 design review for Izcal's JavaScript engine. Date: 31 October 2026. Status: draft 0.2, for owner review; verified (fact-check and design critique of 31 October 2026, recorded in the [README](README.md)'s header). Relates to: the [JS spec](../../reports/Izcal%20JavaScript%20engine.md) (draft 0.4.4 on `main`; 0.4.5's JS-M0 row on the unmerged `docs/js-m0-amendments` branch) 6.1 (H-2), 6.3 (the one semantic description), 6.4 (operation facts, speculation, deopt), 6.5 (GC points, `Init`, the soundness argument), 6.9 (canonical units, decision points), 6.15 (G16, G17), 9.2; JS-D5, JS-D7, JS-D14, JS-D15; [coding standard](../../reports/Izcal%20coding%20standard.md) CS-ST-17, CS-D4, CS-TY-10; [the bytecode note](bytecode-and-core-set.md). Index and decision list: [README](README.md).*

*Revision 0.2 (31 October 2026): phase-8 findings applied. A user call ends its path, each operation declares its after-call accumulator, and tail user calls of five kinds give T0 its call kinds (R0-O11, OC-17); heap categories and fact kinds are a declared tree shared with T2's IR (R0-O12); the grammar gains `invalidates`, `results`, `transfer`, `smt` and `frame_charge`; engine-thrown errors and hooks with a heap context are GC points; weak state ends OC-9's epoch at every GC point; helper parameters say how they are rooted; `ROPE_MIN_LENGTH` is derived from Izcal's own layout; the checks are renamed OC-1 to OC-17 so they no longer collide with part 2's contradictions C1 to C16. Mnemonics follow the bytecode note's renaming.*

*Revision 0.1 (31 October 2026): first draft.*

`.hgop` is the language in which every bytecode operation, intrinsic and hot built-in is written once (JS-D5, JS-D14). This note fixes its grammar, its types, its effect system and the rules that keep it total, and it gives six operations in full. The central choices: the language has **no loop construct**, so every description is a finite decision tree whose unbounded work happens only in **declared runtime calls**; each path carries an **ECMA-262 anchor**; **effects are declared and also inferred**, and the inferred set must fit the declaration; the effects of a runtime call become the **Rust signature** that `hg-opgen` generates for it, so the Rust compiler rejects a runtime function that does more than its description says; a heap value crosses a GC point **only in the frame or as an argument of that GC point**; a description that may run user code must **re-read** any mutable heap fact it uses afterwards (JS-D14's rule); **deopt happens only at instruction boundaries**; and **a user call ends its path**, so a lazy deopt after it resumes at the next instruction. At JS-M0, `hg-opgen` emits four things: the reference interpreter in plain Rust, the opcode table, the operation-facts tables and the anchor index.

## 1. What the language must do, and what it must not

| Requirement | Source | Consequence for the language |
|---|---|---|
| Generate the reference interpreter, T0 (assembly per ISA), T1 stencils, the IC language's semantics, optimizer facts, debugger facts, SMT encodings, the fuzzer grammar and anchors from one description | JS spec 6.3 | The description is data that every generator reads, not code that one compiler compiles |
| Small and total; no unbounded loops except through runtime calls; no raw memory access except typed heap operations; each description has an SMT encoding | JS spec 6.3 | No loops, no recursion, no pointers; a fixed set of primitive operations with SMT-LIB semantics |
| Effects derived, never hand-written per compiler | JS spec 6.4 (CVE-2020-6418's class) | Effects are part of the language and checked |
| A description that calls user code must re-validate what it read before the call | JS-D14 | A checked typestate rule (Section 6, OC-9) |
| Every invariant the heap's soundness argument rests on holds in generated code | JS spec 6.5, steps 1–8 | The rooting rule (R0-O6) and the GC-point annotations |
| Everything read from the cage is untrusted outside it | H-2, G17 | No language construct turns a heap value into an address, an index or a size |
| Canonical allocation units fixed by each description | JS spec 6.9 | Two allocation effects; a units formula over semantic quantities only |
| The generator's output is checked in and fresh | G16, CS-D4, CS-ST-17 | Output layout and header fixed (Section 7) |

## 2. Why a language of Izcal's own

| Option | What it is | For | Against | Verdict |
|---|---|---|---|---|
| **(a) A small, total, effect-typed DSL** | This note | Every requirement above holds by construction; SMT encoding is mechanical; the language is designed for exactly Izcal's representation | A parser, a checker and generators to build before the first tier (JS-D5's stated cost) | **Recommended** (JS-D5 restated) |
| (b) Rust functions with attributes and a procedural macro | Write semantics in Rust, annotate effects | Familiar; no new parser | Rust is not total, so termination and effects cannot be inferred; an SMT encoding of arbitrary Rust is research; procedural macros in L1–L3 are against the crate rules except `hg-heap-derive` (plan Phase 0) | Rejected |
| (c) Generate from ECMA-262 directly | Extract semantics from `spec.html` | One source with the spec | No published work derives shapes, ICs, compressed values or bytecode from the spec (JS spec 6.3, [SP 10.4](../Izcal%20JavaScript%20engine/specs_semantics.md)) | Rejected; ESMeta and engine262 check `.hgop` from outside instead (plan Phase 1) |
| (d) Semantics in a general language compiled through a compiler toolkit at build time | The generated-interpreter approach Deegen published ([Xu, 2022](https://sillycross.github.io/2022/11/22/2022-11-22/)) | Proven to beat a hand-written assembly interpreter | Needs a large compiler toolkit at build time; its intermediate form is not a totality- or effect-checked description, and SMT encoding would need a second model | Rejected; its lesson (generation beats hand-writing for lower tiers) is kept |
| (e) An existing specification DSL (an ISA-description or rewrite-rule language) | Reuse another project's language | Mature | Izcal builds its own (M-1); none of them models JS values, the cage or GC points | Rejected; ideas only |

## 3. Grammar (R0-O1)

```ebnf
(* Items *)
file           = { item } ;
item           = const_decl | layout_decl | category_decl | fact_kind_decl
               | runtime_decl | helper_decl | intrinsic_decl | op_decl ;
const_decl     = "const" CONST_NAME ":" scalar_type "=" literal ";" ;
layout_decl    = "layout" kind "{" { field_decl } "}" ;
field_decl     = ident ":" type [ "immutable" ] ";" ;
category_decl  = "category" CAT_NAME [ "(" ident ":" scalar_type ")" ]   (* a parameterised heap category *)
                 [ "under" CAT_NAME ] [ "gc_mutable" ] ";" ;           (* tree parent; changed at GC points *)
fact_kind_decl = "fact" FACT_NAME "on" category { "," category } ";" ;  (* a fact kind and the categories that invalidate it *)
runtime_decl   = "runtime" ident "(" [ params ] ")" "->" type effects facts anchor
                 [ contract ] ";" ;
contract       = "ensures" cond ;
helper_decl    = "helper" IDENT "(" [ params ] ")" "->" type effects facts anchor block ;
intrinsic_decl = "intrinsic" "%" IDENT "(" [ params ] ")" "->" type [ after_call ]
                 effects facts anchor block ;
op_decl        = "op" IDENT "(" [ operand { "," operand } ] ")" acc_mode [ after_call ]
                 effects facts anchor block ;
after_call     = "after_call" ( "result" | "preserve" ) ;  (* the accumulator a lazy deopt after this item's user call restores *)
params         = param { "," param } ;
param          = ident ":" [ "frame" ] type ;    (* "frame": every caller passes a frame-backed value (OC-6) *)

(* Facts the optimizer reads (part 2, optimizer-ir.md Section 9) *)
facts          = { fact_clause } ;
fact_clause    = "invalidates" "(" FACT_NAME { "," FACT_NAME } ")"
               | "results" "(" repr { "," repr } ")"   (* result representations: smi, f64, tagged, bool *)
               | "transfer" IDENT                     (* a type-transfer helper over the abstract-type lattice *)
               | "smt" IDENT                          (* the SMT encoding's ID, emitted from JS-M1 *)
               | "frame_charge" uint ;                (* canonical depth bytes when called as a frame (R2-16) *)

(* Encoding: one source for decoder, assembler, disassembler and verifier tables *)
operand        = ident ":" operand_kind ;
operand_kind   = "reg" | "cnt" | "imm" | "fwd" | "back" | "intr"
               | "const" "<" const_kind { "|" const_kind } ">"
               | "slot" "<" slot_kind ">" ;
acc_mode       = "acc" ( "none" | "in" | "out" | "inout" ) ;

(* Effects and anchors *)
effects        = "effects" "{" [ effect { "," effect } ] "}" ;
effect         = "alloc" "(" units_name ")"          (* semantic: charged in canonical units *)
               | "alloc_internal"                    (* boxes, ropes, flattening: no charge *)
               | "reads" "(" category { "," category } ")"
               | "writes" "(" category { "," category } ")"
               | "throw" "(" error_kind { "," error_kind } ")"
               | "call_user"                         (* may run script *)
               | "host" "(" HOOK_NAME ")"            (* reaches an hg-js-host hook *)
               | "input" "(" ( "entropy" | "clock" ) ")"   (* reads a B1 input *)
               | "interrupt" ;                       (* polls the interrupt and safepoint request *)
category       = CAT_NAME [ "(" expr ")" ] ;          (* a declared category, with its parameter *)
anchor         = "anchor" spec_ref ;
spec_ref       = spec STRING [ "step" STRING ]       (* clause id, optional step *)
               | "izcal" STRING ;                    (* an engine-defined path, with its reason *)
spec           = "ecma262" | "ecma402" | "temporal" | "dcbc" | "html" | "webidl" | "wasm" ;

(* Bodies: decision trees, no loops *)
block          = "{" { stmt } [ expr ] "}" ;               (* a trailing expression is the block's value *)
stmt           = "let" binder [ ":" type ] "=" expr ";"
               | place "=" expr ";"
               | "if" cond block [ "else" ( block | if_stmt ) ]
               | "match" expr "{" arm { arm } "}"
               | path
               | "charge" "(" units_name "(" expr ")" ")" ";"   (* GC point *)
               | effect_stmt ";"                     (* a typed heap operation with no result *)
               | terminal ;
if_stmt        = "if" cond block [ "else" ( block | if_stmt ) ] ;
arm            = pattern [ "if" cond ] "=>" ( path | block | terminal | expr "," ) ;
path           = "path" ident { anchor | speculate } block ;
speculate      = "speculate" ident "." ident ;       (* slot operand . feedback predicate *)
terminal       = "next" ";" | "jump" ident ";" | "return" [ expr ] ";"
               | "throw" error_kind ";"
               | "call_user" call_kind "(" expr { "," expr } ")" ";" ;  (* a tail user call, then next *)
call_kind      = "call" | "construct" | "getter" | "setter" | "iter_step" ;
place          = "acc" | "reg" "[" ident "]" ;
binder         = ident | "(" ident { "," ident } ")" ;

(* Expressions *)
expr           = literal | ident | "acc" | "reg" "[" ident "]" | "const" "[" ident "]"
               | call | heap_read | constructor | alloc_expr
               | "match" expr "{" arm { arm } "}"    (* as an expression: every arm yields a value *)
               | conversion "(" expr ")"            (* i32, u32, i64, f64: exact or range-checked *)
               | expr binop expr | unop expr | "(" expr ")" | "(" expr "," expr ")" ;
call           = ident "(" [ expr { "," expr } ] ")" ;         (* runtime, helper or primitive *)
heap_read      = kind "." ident "(" expr ")" ;                 (* a typed heap operation *)
constructor    = "smi" "(" expr ")" | "number" "(" expr ")" | "bool" "(" expr ")" ;
alloc_expr     = "alloc" kind "{" field_init { "," field_init } "}" ;
field_init     = ident ":" expr ;
pattern        = "Undefined" | "Null" | "True" | "False" | "Empty"
               | "Smi" "(" ident ")" | "Num" "(" ident ")"
               | "Ref" "<" kind ">" "(" ident ")" | "Value" "(" ident ")"
               | "(" pattern "," pattern ")" | pattern "|" pattern | "_" ;
cond           = expr ;                              (* of type bool *)
```

Comments are `//` to end of line. `Num(x)` matches a small integer or a heap number and binds its value as `f64`. There is no `while`, `for`, `loop`, recursion, indexing of memory, or pointer type.

**Heap categories are declared, as a tree.** The categories of part 2's IR (R2-4: `Heap`, `Field(key class)`, `Elements(layout)`, `FactHolder`, `Context`, `Buffer(n)`, the Wasm categories, `External(kind)`) are `category` items in one `.hgop` file, with their parents, and so are the categories JS-M0 needs that the IR list does not name: `weak` (weak slots; `gc_mutable`, because a cycle clears them), `kept` (the `[[KeptAlive]]` list) and `realm` (the realm's random stream), each under `Heap`. Fact kinds name the categories whose writes invalidate them. `hg-opgen` emits the tree and the fact kinds to the operation facts and to T2's effect bitset, so the two parts read one list (R0-O12); part 2's table lists the three ([optimizer-ir.md](optimizer-ir.md) Section 7). The examples below write `props` for `Field(any)`, the JS-M0 dictionary's named slots.

## 4. Types and representations

| Type | Values | Representation | Where values come from |
|---|---|---|---|
| `Value` | Any JS value at JS-M0's domain and beyond | A tagged 32-bit slot (JS-D1) | Registers, the accumulator, constants, heap reads, runtime results |
| `Smi` | Integers in [−2^30, 2^30 − 1] | Tag 0, 31 bits | `Smi(x)` patterns; `smi(e)` needs a range proof |
| `Ref<K>` | A reference to a heap object of kind `K`: `HeapNumber`, `String`, `BigInt`, `Symbol`, `Object`, and object kinds `Dict`, `WeakRef`, `FinReg`, `HostFn` | Tag 1, compressed | `Ref<K>(x)` patterns (a kind test) |
| `i32`, `u32`, `i64` | Machine integers | Untagged | Primitive operations; untagging a `Smi` |
| `f64` | IEEE 754 binary64 | Untagged | `Num(x)`; primitive float operations |
| `bool` | Machine Boolean | Untagged | Tests and comparisons; `bool(b)` makes the JS value |
| `Key` | An interned property key | `Ref<String>` atom or `Ref<Symbol>` | `const[k]` of kind `key`; `rt_to_property_key` |
| `Empty \| Value` | A weak slot that may be cleared | Tagged slot with the static `empty` root | Weak heap reads |

**Kind tests refine types.** In `match v { Ref<String>(s) => … }`, `s` is a `Ref<String>` in that arm. A kind test reads the object's type index (heap.md H-4; from JS-M1, its shape), which is cage data: under corruption the test may give a wrong answer, never an out-of-cage access (H-2), because typed heap operations clamp every access inside the cage or a checked table (JS spec 6.1).

**Representation is canonical.** `number(f)` makes a Number from an `f64`: a small integer when the value is an integer in range and not −0, otherwise a new heap number (an `alloc_internal` GC point). `smi(i)` makes a small integer and requires that `i` is provably in range. Every tier produces the same representation ([bytecode note](bytecode-and-core-set.md), R0-B3).

**Primitive operations** are total functions with SMT-LIB semantics: `i32` and `i64` add, subtract and multiply with an overflow result or a range proof; bitwise and shift operations with ECMA-262's count masking; conversions between integer widths and to `f64` (exact); `f64` add, subtract, multiply and divide in round-to-nearest-even; ordered comparisons; `f64.is_nan`; `f64.to_int32` and `f64.to_uint32` with [ToInt32](https://tc39.es/ecma262/#sec-toint32)'s modular semantics; and `f64.fmod` with [Number::remainder](https://tc39.es/ecma262/#sec-numeric-types-number-remainder)'s truncating semantics. No primitive is implementation-approximated: `Math` functions and [Number::exponentiate](https://tc39.es/ecma262/#sec-numeric-types-number-exponentiate) are runtime calls to `hg-math` (JS spec 6.9). No primitive contracts a multiply and an add into a fused operation.

**Typed heap operations** are `hg-heap`'s, each O(1), each with its H-2 verification condition stated in `hg-heap`'s inventory. At JS-M0: `String.length` (immutable), `HeapNumber.value` (immutable), `BigInt.is_zero` (immutable), `Object.kind` (immutable), `Object.is_htmldda` (a type-table flag at JS-M0, a shape bit from JS-M1), `Object.is_callable`, `WeakRef.target` (reads `weak`), `kept.push` (writes `kept`), `realm.random_next` (writes `realm`, input `entropy`), and allocation of the fixed-size kinds (`HeapNumber`, `Rope`, `WeakRef`). A field marked `immutable` in its `layout` never changes after the object is published; that matters for rule OC-9.

## 5. Effects, frame effects, GC points and deopt points

### 5.1 Effects (R0-O3)

| Effect | Meaning | Implies |
|---|---|---|
| `alloc(u)` | Semantic allocation, charged in canonical units by the formula `u` ([bytecode note](bytecode-and-core-set.md) Section 6) | GC point |
| `alloc_internal` | Allocation that charges nothing: heap numbers, rope nodes, flattening | GC point |
| `reads(c)`, `writes(c)` | Reads or writes heap category `c` | — |
| `throw(k)` | May complete abruptly with an error of kind `k` (`any` for a user value) | For an error the engine creates (`TypeError`, `RangeError`, …): `alloc(error_units)` and so a GC point ([bytecode note](bytecode-and-core-set.md) Section 6); `throw(any)` rethrows a value that already exists |
| `call_user` | May run script | GC point; `reads` and `writes` of every category; `throw(any)`; an interrupt check |
| `host(H)` | Reaches hook `H` of `hg-js-host` | The hook's context ([hook table](js-host-hooks.md)): none gives no heap effect; `&HostCx` gives reads of `Heap`; `&mut HostCx` gives a GC point and writes of `Heap`; `&mut ScriptCx` gives `call_user` |
| `input(entropy)`, `input(clock)` | Reads a B1 input | Not foldable, not reorderable across another `input`; T2's `Input` effect ([optimizer-ir.md](optimizer-ir.md) Section 7) |
| `interrupt` | Polls the interrupt and safepoint request | GC point |

**Declared and inferred (R0-O3).** Each runtime function, helper, intrinsic and operation declares its effects. `hg-opgen` infers each path's effects from what the path calls and reads, and rejects a description whose inferred effects exceed its declaration. A declaration wider than the inference is allowed but reported, because optimizer facts would then be needlessly pessimistic. Facts are kept per path, so T2, which speculates one path, sees that path's effects and not the union.

**Runtime effects become Rust signatures.** `hg-opgen` emits a `Runtime` trait with one method per declared runtime function, and the method's parameters follow from the declared effects:

| Declared effect | What the generated Rust signature requires |
|---|---|
| None, or only `reads` and `throw(any)` | `&'m Mutator<'h>` and unrooted `Gc<'h, 'm, T>` arguments: no GC point is reachable |
| Any GC point (`alloc`, `alloc_internal`, `interrupt`) | `&mut Mutator<'h>`, a `Scope`, and `Root` arguments (JS spec 6.5, steps 1–3) |
| `call_user` | `&mut Mutator<'h>` and `&mut ScriptToken` (CS-TY-10) |
| `throw` | A `Result<_, Throw<'h, 's>>` return |
| `host(H)` | The hook context the [hook table](js-host-hooks.md) gives `H` |
| `input(entropy)` or `input(clock)` | The realm's stream or the host's time hook, passed in |

A runtime function declared without `call_user` has no `ScriptToken` and cannot run script; one declared without a GC point holds only a shared borrow of the `Mutator` and cannot allocate. The declaration is therefore checked twice: by inference on the description side and by `rustc` on the Rust side. This is how the optimizer's side-effect model cannot disagree with the runtime (JS spec 6.4).

### 5.2 Frame effects

Each operation's frame effect is inferred from its body: which registers it reads and writes, whether it reads or writes the accumulator, and its control (`next`, `jump`, a backward jump, `return`, `throw`). It must agree with the declared `acc` mode and the operand kinds: a `fwd` operand may only be the target of `jump`, a `back` operand only of a backward jump, and only an operation with a `back` operand may jump backward. The verifier's tables, T2's SSA renaming and the debugger read the same facts.

### 5.3 GC points and the rooting rule (R0-O6)

A **GC point** is any `alloc`, `alloc_internal`, `charge`, `interrupt`, `throw` of an engine-created error, `host(H)` whose hook context is `&mut HostCx` or `&mut ScriptCx`, or call of a runtime function, helper or intrinsic whose effects include one. **Rule: a heap value may be live across a GC point only if it is held in the frame (a register or the accumulator, unwritten since it was read) or is an argument of that GC point.** `hg-opgen` checks this by liveness over the decision tree, which has no loops and so needs no fixpoint.

Why this rule and not automatic rooting of temporaries: T0 keeps temporaries in machine registers, which the collector cannot scan. With the rule, the only value T0 must spill at a GC point is the accumulator, and the stack map of an interpreter frame is fixed ([bytecode note](bytecode-and-core-set.md) Section 4.3). Arguments of a GC point are rooted by its callee, which is exactly JS spec 6.5's `Init` ("roots and scalars only"). In the generated Rust, the rule surfaces as `Gc` values that `rustc` will not let live across `&mut Mutator`, so a generator bug is caught a second time.

**Helpers declare how their parameters are rooted.** A helper is checked once, not per caller, so OC-6 cannot see what a caller passed. A helper parameter is therefore either a `Root` (rooted by the helper's own scope, at the cost of a root slot) or declared `frame`: every call site must pass a value that is frame-backed at the call, and OC-6 checks that at each call site; inside the helper a `frame` parameter counts as frame-backed across its GC points.

### 5.4 Deopt points and speculation (R0-O7)

- **Deopt happens only at instruction boundaries.** An eager deopt resumes the interpreter at the start of the instruction whose guard failed; a lazy deopt (the callee returned with the invalidated status, JS-D19) resumes it at the start of the next instruction, with the accumulator the instruction's `after_call` mode names. There is no resume point inside a description.
- **A user call ends its path (R0-O11, OC-17).** On every path, the step that may run user code is the last effect before `next`. It is one of three forms: `acc = rt_f(…)` (mode `result`: the call's value is the accumulator); a runtime call whose value is discarded (mode `preserve`: the accumulator is unchanged, as in `SetNamed`); or the tail construct `call_user kind(…)`. So "resume after the call" is always "resume at the next instruction with a known accumulator", which is what part 2's after-call frame state restores ([frames note](frames-deopt-stackmaps.md) Section 5.2). Work after a call, such as `Lt` mapping IsLessThan's undefined to false, moves into the runtime function (Section 9.2).
- **Tail user calls give T0 its call kinds.** `call_user call|construct|getter|setter|iter_step (callee, this, args…)` is a genuine call made from T0's own code, through T0's call sequence for that kind, which returns to that kind's fixed call-return continuation. These are the per-kind continuations part 2's deopt resumes inner frames into ([frames note](frames-deopt-stackmaps.md) Section 5.1, steps 5 and 6). A Rust runtime function that calls user code internally stays opaque, and T2 never inlines across it (step 7). At JS-M0 no user function exists; the reference interpreter implements the construct as a call of a host function.
- **A speculable guard comes before every effect on its path.** `speculate fb.p` marks the test that selects a path as one T2 may turn into a deopt guard when the slot's feedback shows only predicate `p`. Re-executing the instruction after an eager deopt is then correct, because nothing on the path happened yet. `hg-opgen` rejects a `speculate` placed after a GC point, a write, a `call_user`, a `host` or an `input`.
- **User code runs only inside runtime calls or tail user calls**, never between two halves of a description, so a description never needs a frame state in its middle. A built-in that must do work after a user call (JS-D14's inlinable built-ins, for example an array method's callback loop) is written as **pieces split at tail user calls**: each piece is a description, and the continuation of a call is the next piece. Part 2's builtin-continuation frame state resumes at the start of a piece, never inside one ([frames note](frames-deopt-stackmaps.md) Section 5.1, step 7).

### 5.5 Anchors (R0-O5)

Every path has exactly one anchor, its own or the nearest enclosing one. An anchor names a specification, a clause id at the pinned revision (ANC-1) and optionally a step; `izcal "…"` marks an engine-defined path such as boxing or the canonical string-length limit, with its reason. `hg-opgen` checks every clause id against the pinned `spec.html` in the `test-data` cache (plan Phase 0, item 5); nothing from the specification is copied into the output except the ids Izcal wrote. Spec-coverage measurement counts `izcal` anchors separately, so a path with no spec step is visible as such.

## 6. Checks `hg-opgen` performs

| # | Check | Rejects |
|---|---|---|
| OC-1 | Parse and resolve names; every kind, field, runtime function and hook is declared | Unknown names |
| OC-2 | Types: every expression is well typed; every `match` is exhaustive over its scrutinee's type | A path that does not handle, say, `Ref<BigInt>` |
| OC-3 | Totality: every path ends in exactly one terminal; the helper call graph is acyclic; integer arithmetic either yields an overflow result or is proved in range by interval analysis | Fall-through, recursion, unchecked overflow |
| OC-4 | Frame effects agree with the operand kinds and the `acc` mode; jumps go in the operand's direction | A forward jump through a `back` operand |
| OC-5 | Effects: each path's inferred effects fit the declaration; the union is recorded | An undeclared allocation or user call |
| OC-6 | Rooting (R0-O6): no heap value outside the frame lives across a GC point except as its argument; at each helper call, every `frame` parameter receives a frame-backed value | An unrooted temporary across `alloc` |
| OC-7 | Speculation (R0-O7): a `speculate` guard precedes every effect on its path | A guard after a write |
| OC-8 | Canonical units: `alloc(u)` formulas use only semantic quantities (lengths, counts, constants), never a representation test such as "is a rope" | A charge that differs by tier |
| OC-9 | **Re-validation (JS-D14):** a value read from a mutable heap field is stamped with an epoch; `call_user` or `host` ends the epoch, and every GC point ends it for `gc_mutable` categories (`weak`: a cycle clears weak slots); using a stale value afterwards is an error unless it is read again | A length read before a user callback and used after it; a weak target read before an allocation and used after it |
| OC-10 | H-2 at the description level: no value read from the heap reaches an operand of a typed heap operation that takes a bound, except through that operation's own clamp | A length from the cage used as a loop or copy bound outside `hg-heap` |
| OC-11 | G13 at the description level: clocks and entropy only through `input(…)` effects, which name a hook or the realm stream | A description reading time any other way |
| OC-12 | Anchors: every path has one, and each clause id exists in the pinned `spec.html` | A missing or stale anchor |
| OC-13 | Encoding: opcodes fit the opcode space; each operand kind is valid for its use; intrinsic arities match | An encoding that the verifier could not check |
| OC-14 | Determinism: no primitive is implementation-approximated; no fused multiply-add; NaN-observable stores canonicalize (JS-D12 (a)) | A description whose result could differ by ISA |
| OC-15 | Output is byte-identical across OSes and runs (CS-ST-17 (e)) | Non-deterministic generation |
| OC-16 | Freshness: regenerating the checked-in output changes nothing (G16, `codegen-fresh`) | A hand edit |
| OC-17 | User calls end paths (R0-O11): a step that may run user code is the last effect before `next`, in one of the three forms of Section 5.4, and agrees with the item's `after_call` mode | `let r = rt_f(x); acc = bool(r == True);` |

## 7. What `hg-opgen` emits at JS-M0 (R0-O9)

| Output | Where | Content |
|---|---|---|
| **Opcode table** | `crates/hg-js-bytecode/src/generated/` | Opcode numbers, mnemonics, operand kinds and the format identifier; the decoder, the assembler's and disassembler's tables, and the JS-M1 verifier's kind tables, all from one list |
| **Reference interpreter** | `crates/hg-js-ops/src/generated/` | The dispatch loop, one function per operation and intrinsic, and the `Runtime` trait (Section 5.1) that `hg-js-runtime` implements by hand |
| **Operation facts** | `crates/hg-js-ops/src/generated/facts.rs`, plus `facts.tsv` for review diffs | One record per operation and per path (below) |
| **Anchor index** | `crates/hg-js-ops/src/generated/anchors.tsv` | One entry per path (below) |

Each file starts with the header CS-ST-17 (a) requires, naming `hg-opgen` and its input files. The T0 assembly, T1 stencils, IC semantics, SMT encodings and the fuzzer grammar are later outputs of the same descriptions (JS spec 6.3); the language already carries everything they need.

**The reference interpreter's shape.** A plain `match` loop: correctness and Miri runs matter, speed does not. It is generic over the runtime, so there is no `dyn` dispatch and no dependency cycle: `hg-js-ops` defines the trait, `hg-js-runtime` implements it and depends on `hg-js-ops`.

```rust
// crates/hg-js-ops/src/generated/interp.rs: generated by hg-opgen from ops/*.hgop. Do not edit.
pub fn run<'h, R: Runtime<'h>>(
    m: &mut Mutator<'h>,
    t: &mut ScriptToken,
    rt: &mut R,
    unit: TrustedRef<'_, Unit>,   // a verified unit in the trusted region
    frame: &mut Frame<'h, '_>,    // a root scope: receiver, parameters, registers, accumulator
) -> Completion<'h> {
    let code = unit.code();
    let mut pc = 0u32;
    loop {
        let i = decode(code, pc); // generated from the opcode table
        let flow = match i.op {
            Op::Add => ops::add(m, t, rt, frame, i.reg(0), i.slot(1)),
            Op::Lt => ops::lt(m, t, rt, frame, i.reg(0), i.slot(1)),
            Op::BranchFalsy => ops::branch_falsy(m, frame, i.fwd(0)),
            Op::CallIntrinsic => intrinsics::call(m, t, rt, frame, i.intr(0), i.reg(1), i.cnt(2)),
            // ... one arm per opcode
        };
        match flow {
            Flow::Next => pc = i.end,
            Flow::Jump(target) => pc = target,
            Flow::Return => return Completion::Normal(frame.accumulator_root()),
            Flow::Throw(e) => return Completion::Throw(e), // JS-M1: the handler table first
        }
    }
}
```

Frame access goes through `hg-heap`'s API only: `frame.reg(m, r)` returns a `Gc<'h, 'm, Value>` borrowed from the mutator, `frame.set_reg(m, r, v)` stores through the frame's root slots, and nothing in generated code dereferences the cage (G15). The runtime methods the operations call look like this (two of them, from the examples below):

```rust
pub trait Runtime<'h> {
    /// sec-applystringornumericbinaryoperator, generic path: may run user code.
    fn rt_add<'s>(&mut self, m: &mut Mutator<'h>, t: &mut ScriptToken, s: &Scope<'h, 's>,
                  x: Root<'h, 's, Value>, y: Root<'h, 's, Value>)
                  -> Result<Root<'h, 's, Value>, Throw<'h, 's>>;
    /// sec-islessthan step 3: loops over code units; reads only, no GC point.
    fn rt_string_less_than<'m>(&self, m: &'m Mutator<'h>,
                               a: Gc<'h, 'm, JsString>, b: Gc<'h, 'm, JsString>) -> bool;
}
```

**Operation-facts fields.** Per operation: opcode, mnemonic, operand kinds, accumulator mode, control kind, the union of effects, whether it is a terminator, and the debugger's class (no effect; writes the heap; may run user code; reaches a host hook; reads an input). Per path: its name, its guard and the feedback predicate it may be speculated on, its effects, its GC points with the frame values live across each (by R0-O6, only the accumulator and registers), its canonical-units formula, whether it is pure and constant-foldable (no effect, no allocation, no input), its runtime calls, and its anchor. T2's effect lattice, the debugger's side-effect-free evaluation and the T0 generator read these, so none of them restates an effect by hand.

**Anchor-index entries.** One per path: the item (operation, intrinsic or helper), the path name, the specification, the pinned revision, the clause id, the step if given, and whether it is a spec anchor or an `izcal` one. ANC-1's coverage tooling joins this with test262 and ESMeta coverage.

## 8. Why each description has an SMT encoding

A description denotes a function from (frame, heap, inputs) to (frame, heap, outcome). With no loops and an acyclic helper graph it unfolds into a finite decision tree, and each part maps onto a standard SMT-LIB theory:

| Construct | Theory |
|---|---|
| Tags, small integers, `i32`, `u32`, `i64`, shifts and bitwise operations | Fixed-size bit-vectors ([SMT-LIB](https://smt-lib.org/theories-FixedSizeBitVectors.shtml)) |
| `f64` arithmetic and comparisons in round-to-nearest-even | Floating point ([SMT-LIB](https://smt-lib.org/theories-FloatingPoint.shtml)); `to_int32` as conversion to a bit-vector wide enough for every finite binary64 value, then extraction of the low 32 bits; `fmod` by its defining equation with an integral quotient |
| The heap | Arrays from (object, field) to slot ([SMT-LIB](https://smt-lib.org/theories-ArraysEx.shtml)), read and written only by typed operations |
| Runtime calls | Uninterpreted functions, constrained by their declared contract (`ensures`) and effects: a call without `writes(c)` leaves category `c` unchanged |
| Paths and guards | Boolean structure ([SMT-LIB core](https://smt-lib.org/theories-Core.shtml)) |

A T2 rewrite rule ("`Add` on two small integers whose sum is in range is a 32-bit add") is then checked by asking a solver whether the rule's result differs from the description's on any input that satisfies its premise, the shape of check Arrival runs on instruction-selection rules against ISA semantics ([OOPSLA 2025](https://www.contrib.andrew.cmu.edu/~bparno/papers/arrival.pdf)); Icarus checks a different property, the safety of IC stubs that generators emit, by symbolic execution of the generators ([SOSP 2024](https://www.cs.utexas.edu/~hovav/dist/icarus.pdf)), which is the check `hg-rulecheck` needs for the IC language. The encodings are emitted from JS-M1, when `hg-rulecheck` verifies its first rules (JS spec 9.2); the language guarantees they exist from JS-M0.

## 9. Six operations in full

The declarations these examples use:

```hgop
const MAX_STRING_LENGTH: i64 = 1073741823;   // 2^30 − 1 code units† (bytecode note, Section 6)
const ROPE_MIN_LENGTH: i64 = 9;              // shorter results are copied flat† (below)

layout String   { length: u32 immutable; }
layout Rope     { left: Ref<String> immutable; right: Ref<String> immutable; length: u32 immutable; }
layout WeakRef  { target: Empty | Value; }

runtime rt_add(x: Value, y: Value) -> Value
  effects { alloc(string_units), alloc_internal, throw(any), call_user }
  anchor ecma262 "sec-applystringornumericbinaryoperator";
runtime rt_less_than(x: Value, y: Value, left_first: bool) -> Value
  effects { alloc_internal, throw(any), call_user }
  anchor ecma262 "sec-islessthan"
  ensures result is True | False;     // IsLessThan, with undefined mapped to false (the operator's step)
runtime rt_string_less_than(a: Ref<String>, b: Ref<String>) -> bool
  effects { }
  anchor ecma262 "sec-islessthan" step "3";
runtime rt_string_flat_concat(a: Ref<String>, b: Ref<String>, length: i32) -> Ref<String>
  effects { alloc_internal }
  anchor izcal "flat copy of both operands; a loop over code units";
runtime rt_ordinary_get(o: Ref<Object>, k: Key, receiver: Value) -> Value
  effects { reads(props), alloc_internal, throw(any), call_user }
  anchor ecma262 "sec-ordinaryget";
runtime rt_get(base: Value, k: Key) -> Value
  effects { reads(props), alloc_internal, throw(any), call_user }
  anchor ecma262 "sec-getv";
```

### 9.1 `Add`: fast paths, a helper, and a generic runtime path

```hgop
op Add(lhs: reg, fb: slot<binary>) acc inout after_call result
  effects { alloc(string_units), alloc_internal, throw(any), call_user }
  anchor ecma262 "sec-applystringornumericbinaryoperator"
{
  let x = reg[lhs];                      // frame-backed until reg[lhs] is written
  let y = acc;                           // frame-backed until acc is written
  match (x, y) {
    (Smi(a), Smi(b)) => path smi speculate fb.smi
        anchor ecma262 "sec-numeric-types-number-add" {
      acc = number(f64(i64(a) + i64(b)));   // |a|, |b| ≤ 2^30: the i64 sum cannot overflow
      next;
    }
    (Num(a), Num(b)) => path number speculate fb.number
        anchor ecma262 "sec-numeric-types-number-add" {
      acc = number(f64.add(a, b));       // a heap number if the sum is not a small integer
      next;
    }
    (Ref<String>(a), Ref<String>(b)) => path strings speculate fb.string
        anchor ecma262 "sec-applystringornumericbinaryoperator" step "1.c" {
      acc = StringConcat(a, b);
      next;
    }
    _ => path generic {                  // ToPrimitive, ToString, ToNumeric, BigInt, mixed types
      acc = rt_add(x, y);
      next;
    }
  }
}
```

What `hg-opgen` derives: three speculable paths whose guards precede all effects (OC-7); a GC point on every path (`number` boxing, the helper's `charge` and allocation, the runtime call); across each, `x` and `y` are still in `reg[lhs]` and the accumulator, so the rooting rule holds (OC-6). The `generic` path is the only one with `call_user`, so a T2 that speculated `smi` sees an operation with no user call, no throw and at most an internal allocation.

### 9.2 `Lt`: a loop that must be a runtime call

```hgop
op Lt(lhs: reg, fb: slot<compare>) acc inout after_call result
  effects { alloc_internal, throw(any), call_user }
  anchor ecma262 "sec-relational-operators-runtime-semantics-evaluation"
{
  let x = reg[lhs];
  let y = acc;
  match (x, y) {
    (Smi(a), Smi(b)) => path smi speculate fb.smi
        anchor ecma262 "sec-numeric-types-number-lessThan" {
      acc = bool(a < b);
      next;
    }
    (Num(a), Num(b)) => path number speculate fb.number
        anchor ecma262 "sec-numeric-types-number-lessThan" {
      acc = bool(f64.lt(a, b));          // ordered: false when either is NaN, which is the
      next;                              // spec's undefined mapped to false by the operator
    }
    (Ref<String>(a), Ref<String>(b)) => path strings speculate fb.string
        anchor ecma262 "sec-islessthan" step "3" {
      acc = bool(rt_string_less_than(a, b));   // the code-unit loop; reads only, no GC point
      next;
    }
    _ => path generic {
      acc = rt_less_than(x, y, true);          // the call is the path's last effect (OC-17); "if r
                                               // is undefined, return false" happens inside it
      next;
    }
  }
}
```

[IsLessThan](https://tc39.es/ecma262/#sec-islessthan) compares strings code unit by code unit (step 3), which is a loop, so the language forces it into `rt_string_less_than`. Its declared effects are empty, so its generated signature takes `&Mutator` and unrooted `Gc` arguments: it cannot allocate or run script, and T2 may treat that path as pure.

### 9.3 `BranchFalsy`: total and effect-free

```hgop
op BranchFalsy(target: fwd) acc in
  effects { }
  anchor ecma262 "sec-toboolean"
{
  let truthy: bool = match acc {
    True => true,
    False | Undefined | Null => false,
    Smi(i) => i != 0,
    Ref<HeapNumber>(h) => {               // canonical form: never +0, but may be −0 or NaN
      let d = HeapNumber.value(h);
      !(d == 0.0 || f64.is_nan(d))
    }
    Ref<String>(s) => String.length(s) != 0,
    Ref<BigInt>(n) => !BigInt.is_zero(n),
    Ref<Symbol>(_) => true,
    Ref<Object>(o) => path htmldda_check
        anchor ecma262 "sec-IsHTMLDDA-internal-slot-to-boolean" {
      !Object.is_htmldda(o)               // Annex B: document.all-like objects are falsy
    }
  };
  if truthy { next; } else { jump target; }
}
```

Every arm is an O(1) read of an immutable field or a type flag, so the whole operation has no effect and every tier inlines it. With JS-M0's canonical representation (R0-B3) a heap number is −0 or non-integral or out of range, so the `d == 0.0` test catches exactly −0.

### 9.4 `StringConcat`: canonical units, a limit, and two representations

```hgop
helper StringConcat(a: frame Ref<String>, b: frame Ref<String>) -> Ref<String>
  effects { alloc(string_units), alloc_internal, throw(RangeError) }
  anchor ecma262 "string-concatenation"
{
  let n: i64 = i64(String.length(a)) + i64(String.length(b));
  if n > MAX_STRING_LENGTH {
    path too_long anchor izcal "canonical string-length limit (R0-B10)" { throw RangeError; }
  }
  charge(string_units(n));               // semantic: the same in every tier and representation
  if String.length(a) == 0 { return b; }
  if String.length(b) == 0 { return a; }
  if n < ROPE_MIN_LENGTH {
    path flat { return rt_string_flat_concat(a, b, i32(n)); }
  }
  path rope { return alloc Rope { left: a, right: b, length: u32(n) }; }
}
```

The charge depends on `n` only, so a tier that folds a concatenation or builds a flat string where another builds a rope charges the same (OC-8). `charge` is a GC point: `a` and `b` are `frame` parameters, so OC-6 checks at each call site that the caller passes frame-backed values (`Add` passes `reg[lhs]` and the accumulator), and in the allocation they are the arguments, so OC-6 holds.

**Why `ROPE_MIN_LENGTH` is 9†.** A rope node is 16 bytes: heap.md H-4's 4-byte header, the length word and two 4-byte slots. A flat one-byte string of *n* code units is the 8-byte header plus *n* bytes. For *n* ≤ 8 the flat result is no larger than a rope node, the copy is at most one 8-byte store, and nothing is left to flatten later, so a rope never pays below 9 code units. Above it, a rope saves a copy that grows with *n*. This is a layout bound only; JS-M1 re-sets it from measurement (Q-R0-5). The length is immutable, so reading it again after `charge` is not a re-validation error (OC-9).

### 9.5 `GetNamed` on a dictionary object

```hgop
op GetNamed(obj: reg, key: const<key>, fb: slot<load>) acc out after_call result
  effects { reads(props), alloc_internal, throw(any), call_user }
  anchor ecma262 "sec-evaluate-property-access-with-identifier-key"
{
  let o = reg[obj];
  let k = const[key];                    // a Key: the trusted constant kind guarantees the kind
  match o {                              // checked when the array was materialized
    Undefined | Null => path nullish anchor ecma262 "sec-getvalue" step "3.a" {
      throw TypeError;                   // ToObject throws on undefined and null
    }
    Ref<Object>(d) if Object.kind(d) == Dict => path dictionary
        anchor ecma262 "sec-ordinaryget" {
      acc = rt_ordinary_get(d, k, o);    // hash lookup, then the prototype walk: unbounded
      next;
    }
    Ref<Object>(x) => path other_object anchor ecma262 "sec-getv" {
      acc = rt_get(x, k);                // exotic and host objects ([[Get]] dispatch)
      next;
    }
    _ => path primitive anchor ecma262 "sec-getv" {
      acc = rt_get(o, k);                // wrapper prototypes: JS-M1's realm
      next;
    }
  }
}
```

The prototype walk is a loop, so it is a runtime call. Getters may run script, so the dictionary path declares `call_user` even though JS-M0's dictionary objects hold only data properties: the description is written for the language, not for JS-M0's subset, and does not change at JS-M1. At JS-M1 an IC step is inserted before the `dictionary` path, written in the IC language, and the slot operand `fb` stops being ignored.

### 9.6 `%WeakRefDeref`: weak state and kept objects

```hgop
intrinsic %WeakRefDeref(this: Value) -> Value
  effects { reads(weak), writes(kept), throw(TypeError) }
  anchor ecma262 "sec-weak-ref.prototype.deref"
{
  match this {
    Ref<WeakRef>(w) => path deref anchor ecma262 "sec-weakrefderef" {
      match WeakRef.target(w) {
        Empty => return Undefined;
        Value(v) => {
          kept.push(v);                  // AddToKeptObjects: not a GC point (below)
          return v;
        }
      }
    }
    _ => path not_a_weakref anchor ecma262 "sec-requireinternalslot" {
      throw TypeError;
    }
  }
}
```

**The kept-objects list lives outside the cage (R0-O10).** [AddToKeptObjects](https://tc39.es/ecma262/#sec-addtokeptobjects) appends to the agent's `[[KeptAlive]]` list. Izcal keeps it as a per-agent root list of compressed references outside the cage, so `kept.push` is a typed heap operation that never allocates in the cage and is not a GC point; the list's own growth is Rust memory, whose exhaustion crashes the renderer (allowed for tier R, JS spec 6.5) and is never JS-visible. `kept.push` also **shades the target when concurrent marking is active**: with a snapshot-at-the-beginning marker and no read barrier, a target reachable only through a weak reference at the snapshot could otherwise be stored into the heap by the mutator and never marked. That is the one place a weak read needs marker cooperation, and the typed operation carries it, so no description can forget it.

### 9.7 What OC-9 rejects: a read kept across user code

`SetKeyed` must convert its key with [ToPropertyKey](https://tc39.es/ecma262/#sec-topropertykey), which may run the key's `toString`. A description that reads the object's mutable state first and uses it after the conversion is rejected:

```hgop
let ext = Object.is_extensible(d);       // a mutable field: epoch 0
let k = rt_to_property_key(key);         // call_user: epoch 0 ends
if ext { rt_dict_add(d, k, acc); }       // OC-9: `ext` read at epoch 0, used at epoch 1
```

`hg-opgen` reports that `ext` was read before a call that may run user code and is used after it, and asks for the read to move after the call. This is JS-D14's structural TOCTOU rule: the class of a length read before a user callback becomes a generator error instead of a bug found by fuzzing.

## 10. Decisions in this note

| ID | Decision | Alternative | Cost | Reversal trigger |
|---|---|---|---|---|
| R0-O1 | `.hgop` is Izcal's own small, total, effect-typed language, with the grammar of Section 3 | (b)–(e) of Section 2 | A parser, checker and four generators before the reference interpreter | JS-D5's triggers |
| R0-O2 | The operand and encoding declarations in `.hgop` are the one source of the opcode table | A hand-written opcode table in `hg-js-bytecode` | None beyond generation | — |
| R0-O3 | Effects are declared and inferred; runtime effects become Rust signatures | Inferred only; declared only | Declarations to write | — |
| R0-O4 | Two allocation effects: semantic (charged) and internal (free) | One allocation effect | A units formula per semantic allocation | — |
| R0-O5 | One anchor per path, checked against the pinned `spec.html`; nothing copied | Anchors per operation only | An anchor per path | — |
| R0-O6 | Heap values cross GC points only in the frame or as GC-point arguments | Automatic rooting of temporaries | Some descriptions restructured | A hot built-in that cannot be written under the rule without a measured cost above 2%† of its run time at JS-M2 |
| R0-O7 | Deopt only at instruction boundaries; speculable guards before effects; user code only inside runtime calls or tail user calls | Resume points inside descriptions | Inlinable built-ins that call script are written as pieces split at tail user calls | A built-in whose split costs more than 2%† of its run time at JS-M2 |
| R0-O8 | OC-9's epoch rule enforces JS-D14's re-validation | Review and fuzzing only | Descriptions ordered to read after calls | — |
| R0-O9 | At JS-M0, `hg-opgen` emits the opcode table, the reference interpreter, the facts and the anchor index, checked in with a freshness check | Generation at build time (`build.rs`) | Checked-in generated code (CS-T14 bounds it) | CS-D4's trigger |
| R0-O10 | `[[KeptAlive]]` is a per-agent root list outside the cage; `kept.push` is not a GC point and shades under concurrent marking | A list object in the cage | A root list per agent | — |
| R0-O11 | A user call is its path's last effect; each item declares its after-call accumulator (`result` or `preserve`); tail user calls of five kinds (`call`, `construct`, `getter`, `setter`, `iter_step`) are T0's call kinds and their continuations | Resume points after arbitrary post-call work | Post-call work moves into runtime functions or the next piece | — (a soundness rule for lazy deopt) |
| R0-O12 | Heap categories and fact kinds are declared in `.hgop` as one parameterised tree, which T2's effect sets (R2-4) are generated from | A fixed keyword list | The tree's declarations | — |

## 11. Open questions

- **Q-R0-4.** Which feedback predicates does each slot kind offer to `speculate` (`smi`, `number`, `string`, and later shape-based ones)? Fixed with the IC language at JS-M1; the grammar already names them as `slot.predicate`.
- **Q-R0-5.** `ROPE_MIN_LENGTH` (9†) is derived from layout alone (Section 9.4), and whether ropes exist at all at JS-M0 is the runtime's choice; the charge does not depend on it. Re-set at JS-M1 from JS-31's string footprint and flattening cost.
- **Q-R0-6.** Can the operation-level oracle (plan Phase 1) be driven from the anchor index, generating operands per path? That would turn anchor coverage into oracle coverage directly.
