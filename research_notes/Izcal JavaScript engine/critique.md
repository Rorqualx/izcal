# Izcal JavaScript engine: independent design critique (phase 8)

Written 19 October 2026 by a reviewer who did not write the spec. Subject: `reports/Izcal JavaScript engine.md`, draft 0.1 (line numbers below refer to that draft). Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8 (design critique against the existing specs, looking for contradictions, missed constraints and M-1 violations).

**Read against:** HGBA-2 2.3.0 (P1–P6, M-1, I-1 to I-6, Section 4.2, S-4a/b, S-7, X-1, D5); HGBA-3 (B1, B3, B4, B9, B10, DET-1, DET-2, D10); Stage 1 design (Section 1 process map, Section 2 G1–G9, Section 3.1–3.4, Section 5.1–5.4, Section 7, Section 8); DevTools spec 0.3 (DT-D1, DT-D2, DT-1, DT-15, G10–G12, milestones M2/M7/M8); security scope 0.3 (ATK-1/2, TB-4, 6.7, 6.8, 6.13, REC-R6, UPD-R5); coding standard 0.3.1 (tiers, CS-D2, CS-D3, CS-TY-10, CS-TY-12, CS-V8-9, CS-AS-2, Q-CS-7); `m0/M0_SUMMARY.md`; `m0/determinism_crossplatform.md`; the resource-footprint notes (`research_notes/Izcal resource footprint/`); and this spec's own research notes.

**Owner direction applied as binding:** no engine libraries (M-1 part 1, 2.3.0); no shortcuts for time or complexity; the best engine from first principles, "like nothing before"; staging is acceptable only where a first-principles argument (correctness, security, measurement) favours it, never schedule alone.

**Overall.** The spec is strong where it matters most: it names exploited bug classes and a mechanism per class, it puts verification into the design of the rule languages rather than after them, and it treats determinism as a property of the engine rather than a mode. The findings below are mostly places where the spec's own premises (an attacker with arbitrary read/write inside the cage; return addresses never rewritten; GC-observable behaviour a function of inputs; a soundness argument "enforced by the Rust type checker") are stated more strongly than the design as written supports. Six findings are High. One of them is the JS-D8 staging question the owner asked about; the verdict is in finding 6 and repeated at the end.

---

## High

### 1. High — H-1 is necessary but not sufficient: four cage-escape routes the design leaves open

**Section:** 6.1 (lines 247–252), 6.3 bytecode verifier (line 277), 6.7 layer table and machine-code validator (line 463).

**Problem.** The cage's premise is V8's: an attacker with arbitrary read and write inside the cage (line 247). H-1 ("no raw pointer in the cage") is a type rule about what Rust can *store*. The security property the premise needs is different: **nothing outside the cage may trust a value it reads from the cage**. The design breaks that in four places.

1. **Bytecode and IC programs live in the cage and are trusted after one verification.** The cage table lists "bytecode, IC programs" as cage contents (line 32 of 6.1's table), and 6.3 says "the interpreter and baseline assume only what the verifier proved" (line 277). Verification is a time-of-check; the attacker writes the bytecode after it. Register operands index the native interpreter frame, which is outside the cage, so a rewritten register operand is an out-of-cage stack write. The layer table's "interpreter bounds on verified operands" (6.7) contradicts 6.3's "assume only what the verifier proved"; the spec must pick one.
2. **Code-table indices can be swapped.** A function object stores a code-table index (line 249). An attacker who swaps two indices calls function A's compiled code with function B's closure and argument count. Compiled code addresses its frame and arguments at offsets fixed for A; the machine-code validator explicitly permits frame accesses (line 463), so an arity mismatch becomes an out-of-frame read or write that the validator accepts.
3. **Buffer bounds may come from the cage.** The validator accepts "a buffer through a base from the buffer table and a checked or guard-covered index" (line 463). It does not say *checked against what*. Typed-array length, byte offset and the "is detached" state are naturally cage fields, and range analysis will remove checks using them. If the final check is not against the buffer table's length (outside the cage) or a reservation that covers every possible index, a corrupted length is a write outside the buffer.
4. **Rust consumers of cage data.** The GC computes object sizes and traced regions from shapes (cage data); `Trace` code, builtins written in Rust, the heap-image loader and `hg-js-observe` read lengths and kinds from objects. Steps 1–8 of the soundness argument (6.5) assume a well-formed heap; under the cage premise the heap is attacker-controlled. Any `unsafe` slice or unchecked index derived from a cage value is an escape, and the type-level argument does not cover it.

**Evidence.** The spec's own research note records that V8 moved bytecode and other trusted objects into a **trusted space outside the sandbox** as part of its sandbox work ([GC 3.5][GC]: "trusted space outside the sandbox ... code and pointer tables"). V8's dispatch table stores the parameter count with each entry for reason 2 (first principles here; not in the notes, so the spec should cite a primary source if it relies on it). The V8 CFI plan in the notes lists "corrupted syscall arguments, `mmap` remapping, signal frames" as open problems under the same attacker model ([SE 4.9][SE]). Security scope ATK-2 assumes an attacker who "reads its own memory, including the cage" (security scope 6.1).

**Required change.**
- Add a **trusted region** outside the cage (read-only to the mutator after verification) for bytecode, IC programs, stencils' patch data, frame-state and deopt metadata and code-table entries, or state that the interpreter and IC interpreter re-check every operand they use to address anything outside the cage. Record it as a decision (with the measured cost of each option at JS-M1), because it changes the bytecode format and the code cache.
- Make every **code-table entry typed**: kind, parameter count and owner, checked at the call (or by a call trampoline the validator knows).
- State that **buffer bounds come only from the buffer table** or from a reservation sized so that every index the access can form is inside it, and have the validator certificate name which.
- Add invariant **H-2: data read from the cage is untrusted input** for all code outside it, with a lint over `hg-heap`'s inventoried modules (no slice or unchecked index derived from a cage value without a clamp to the cage or table bound), and extend the unsafe-core verification conditions (step 8) to hold under arbitrary cage contents.
- Run the machine-code validator over the **generated production interpreter** (T0, W0, the IC interpreter) at build time; today the spec applies it only to JIT output.
- Add an **in-cage corruption harness** to JS-10 and JS-13: a test-only API that gives a fuzzer arbitrary read and write inside the cage, and counts any fault outside the cage as a security bug.

### 2. High — The `Mutator` soundness argument fails when one thread holds two heaps, which the spec allows

**Section:** 6.5, Rust API sketch (lines 360–393) and soundness steps 3 and 6 (lines 399–402); 6.12 (line 562).

**Problem.** Step 6 says heaps do not mix because `Mutator<'h>` carries an invariant brand. But the brand stops at the mutator: `Gc<'m, T>`, `Root<'s, T>` and `Field<T>` carry no `'h`, and `Field::get`, `Field::set` and `Root::get` take any `&Mutator` (the brand is elided). Section 6.12 then allows "window event loops of the same site may share one" thread, while also saying "one heap and one thread per agent" (line 562; the two halves contradict each other). If two agent heaps share a thread, safe code can:
- read a field of an object in heap A using heap B's mutator, getting a `Gc` whose validity is tied to B's borrow, then trigger a GC in A (through `&mut` A's mutator, which nothing prevents) and use the now-moved or freed reference;
- store a reference into heap A's object into a `Field` of heap B's object (`Field::set` accepts any mutator and any `Gc`), creating a cross-heap edge that neither collector traces;
- read a `Root` from A's scope through B's mutator, indexing B's root table with A's slot: type confusion in safe Rust.

`Gc` is also 4 bytes with no base: dereferencing it needs a cage base from somewhere (a thread-local, presumably), which is ambiguous with two heaps on one thread. Separately, `alloc` takes an `Init<T>` (line 386); if `Init` may hold `Gc` values for initial fields, `&mut self` cannot be taken while they live, so `Init` must hold `Root`s or be applied after allocation. The sketch does not say.

**Evidence.** The claim is "no safe Rust code using this API can read a freed or moved object, store an untraced reference into the heap, or skip a barrier", "steps 1–6 ... enforced by the Rust type checker" (line 395). Nova's unsound-variance finding is the precedent for this class of hole in exactly this kind of API ([GC 6.2][GC]). CS-D3's spike criteria were run against V8's API, not this one (CS-D3, 0.3.1).

**Required change.** Either (a) carry the heap brand in every handle type (`Gc<'h, 'm, T>`, `Root<'h, 's, T>`) and expose field access only through branded projections (`obj.field(m)` on a `Gc<'h, 'm, Obj>`, never a bare `&Field`), or (b) make "one `Mutator` per OS thread" a runtime invariant enforced at heap creation and remove the shared-thread allowance from 6.12. State which, fix 6.12's contradiction, specify `Init<T>`, and add the cross-heap cases (read through the wrong mutator, store across heaps, root through the wrong mutator, `Gc` deref with two heaps on a thread) as compile-fail tests in JS-M0's re-run of the CS-D3 criteria and as JS-10 planted bugs.

### 3. High — "Return addresses are never rewritten" covers lazy deopt only; eager deopt with inlining, OSR and stack switching still need shadow-stack and PAC designs

**Section:** 6.4 speculation and deoptimization (line 312), 6.6 stacks (line 436), 6.7 control-flow integrity, 2.2 last row, 4.4 "lazy deopt" row.

**Problem.** The spec claims that because invalidated code checks a bit on return, "shadow stacks (CET, Arm GCS) and PAC on return addresses can stay on in renderers that run the JIT". That solves lazy deoptimization only. Three other mechanisms in the design write or replace return addresses:
- **Eager deopt with inlining.** One optimized frame becomes several interpreter frames (line 312: "an OSR exit to the interpreter"). If interpreter frames are linked by native `call`/`ret`, the materialized frames' return addresses are not on the shadow stack, and returning from them faults; with PAC, the deoptimizer must sign return addresses, which is a signing gadget.
- **OSR entry** (T0/T1 into T2, and W0/W1 into W2 at loop headers, 6.6) replaces a live frame.
- **Engine-managed stack segments** for JSPI and Stack Switching (line 436) need a shadow stack per segment and a switch that the OS permits. On Linux that means `map_shadow_stack` and restore tokens; whether Windows permits user-mode shadow-stack switching outside its own fiber and thread APIs needs to be checked before the design depends on it. Running Rust runtime code on these segments also interacts with the OS's stack-limit bookkeeping and the Rust runtime's guard-page handling on each OS.

**Evidence.** The V8 CFI plan in the notes lists the backward edge (CET, GCS, PAC-RET) as a design problem for a JIT, with open problems around signal frames and remapping ([SE 4.9][SE]). HGBA-2 S-7 requires "PKU/POE JIT W^X with CET/BTI" at ≤2% Speedometer 3. Security scope 6.7 makes CET part of the Windows renderer mitigation set. The spec lists the return-path check cost as Q-JS-14 but none of the three mechanisms above.

**Required change.** Add a **return-address discipline** to 6.4 and 6.6 that covers every frame transition: (a) interpreter and baseline frames are entered and left without native `ret` into materialized frames (for example, the interpreter's JS-to-JS return is a table-dispatched jump to a landing pad, so materialization writes only data), or an equivalent design that never forges shadow-stack entries or PAC signatures; (b) OSR entry and exit in the same terms; (c) stack-segment switching per OS, with the Windows mechanism verified at JS-M0 and Arm GCS on Linux included; (d) exception unwinding across segments. Make "no shadow-stack write and no PAC signing outside prologue and epilogue" a property the machine-code validator checks, add deopt-with-inlining, OSR and JSPI switching under CET/GCS/PAC to JS-15's probe, and carry S-7's ≤2% bound into JS-15.

### 4. High — Deterministic decision points as designed contradict JS-40 and U2, and the alternative (decision points as recorded inputs) is not considered

**Section:** 6.9 (lines 500–504, 512), JS-40, JS-50, JS-52.

**Problem.** Two rules make the main thread wait for background work at a point fixed in canonical units:
- GC completion: "if helpers are not done, the mutator finishes the marking itself" (line 501). The pause is then all remaining marking work at that point, which is unbounded by anything in the spec and can be tens of milliseconds on a large heap. JS-40 requires main-thread GC pause p99 ≤ 2 ms† and p99.9 ≤ 5 ms†. Nothing reconciles them.
- Tier install: "waiting if compilation is late" (line 503). In production, that is a main-thread stall on a compiler thread, again unbounded, against U2 ("keep interactions under a frame").

JS-52 bounds only the *mean* cost (≤1%† on Speedometer, JetStream Worst and p95 frame time); the risk is in the tails. The spec also does not say why tier installation must be deterministic when "all tiers implement one semantics, so the installed tier changes speed, never results" (line 503). If tiers are semantically identical, install timing is invisible to script; the real reason is to make **engine bugs** reproducible, which is a good reason but a different one, with a different cheapest design.

The missing alternative: **record each decision point as a B1 input** (the substrate already records scheduler choices, clocks and entropy). In production the engine installs code and completes cycles when ready; when recording, it logs the canonical position at which it did; replay enforces the logged positions. That keeps replay exact (DT-1, JS-51), never stalls the mutator, and costs a log entry per decision. Its cost: two unrecorded runs from the same seed may differ, which weakens seed-only DST, exact leak diffs and "exact profiles by replay" across fresh runs. That trade-off is the decision the spec should present.

**Evidence.** The spec itself says the cost "has not been measured anywhere" (line 512, Q-JS-3). Stage 1 3.3 states that under the virtual clock "JIT speed does not matter", which is the simulation-mode argument; it does not carry over to production latency. LXR's lesson, which the spec adopts, is that pauses are not latency and tails matter ([GC 1.3–1.4][GC]).

**Required change.** Present both designs as a decision (JS-D15): (a) pure function of inputs, with forced waits; (b) decision points as recorded inputs; (c) a hybrid (canonical points with a bounded wait, and a recorded override when the bound is hit). Recommend from first principles, but add a **tail metric** to JS-52 (main-thread wait at decision points: p99 and p99.9 per frame, and the share of JS-40's pause budget it uses), and make JS-40 and JS-52 jointly satisfiable or state which wins under P3 > P4. State explicitly that deterministic tier install exists to reproduce engine bugs.

### 5. High — Stage 1 DevTools loses its JavaScript debugger: no JS milestone before Stage 2 delivers what `script.v8` provided

**Section:** 6.10 "Debugger and profiler core" (line 530), 9.2 milestones (JS-M0 to JS-M4), 9.5 DevTools amendment (line 880).

**Problem.** The DevTools spec's owner-decided Stage 1 scope (DT-D10 Option B, 18 October 2026) includes the CDP profile's **JS domains** (Runtime, Debugger, Profiler, HeapProfiler, Console) at M7, `script.v8` in `hg-inspect-renderer` at M2, and recorded sessions with a scrubbable **console** view (DT-1, DT-15). All of these assumed V8's inspector. This spec removes `script.v8` (9.5) and replaces it with "native IIP script domains", but puts those in **JS-M4, Stage 2** (line 794). The debugger core (breakpoints, stepping, scopes, side-effect-free evaluation, sampling profiler) is in Section 6.10 but in no milestone at all. WebDriver at Stage 1 M2 needs script evaluation. The result is a Stage 1 browser whose owner-approved DevTools scope cannot be met.

**Evidence.** DevTools spec 0.3 Section 9.1 and milestones M2, M7 and M8 (lines 382–388 of that spec); DT-D1's `script.v8` exception; this spec's JS-M2 lists only "event stream v1".

**Required change.** Add the debugger and profiler core, console API, and the native IIP script domains (with `hg-cdp`'s translation of the CDP JS profile) to JS-M2 or JS-M3, as Stage 1 work, with a target (CDP JS profile passing the DevTools spec's client-suite harness) and staffing (the performance-and-observability row is 3–4 and already carries the lab and event stream). Or, if the spec believes a Stage 1 debugger is wrong, say why on first principles and raise it as an amendment to DT-D10, which is an owner decision this spec cannot override silently.

### 6. High — JS-D8 stages the optimizer by schedule; the first-principles part of its argument is a gate, not a stage

**Section:** 9.1 (line 778), JS-D8 (line 841), 6.17 row, JS-20, JS-M3/JS-M4, Risks row 2.

**Problem.** Section 9.1 says "none is staged to save time" and gives JS-D8 two arguments.
- *"Its speculation needs IC programs recorded on real pages by the lower tiers."* Those data exist as soon as T0 runs the corpus with the event stream, which is JS-M1 to JS-M2, more than a year before Stage 1 exit (JS-M3 at 30–42 months†). And IC data tune the optimizer's heuristics; they are not needed to build its IR, rules, deopt model or verification. This argument supports ordering *enablement* after data exist, not a stage boundary.
- *"Its rules need `hg-rulecheck`, the assumption ledger and the validator to exist before the first rule is enabled. Shipping it earlier would trade P1 for P4."* That is a genuine first-principles **gate** (P1 > P4): no unverified rule runs. But `hg-rulecheck` and the validator are already JS-M3 (Stage 1) deliverables, because T1 needs them; the ledger is part of T2 itself. The gate therefore does not place T2 in Stage 2; it places T2 *wherever its gates pass*. Option (b), "the optimizer in Stage 1", is framed as if it meant shipping unverified, which nobody proposes.
- **W2 has no IC-data dependency at all**: Wasm optimization is non-speculative except `call_ref` inlining and cast elimination (6.6). Its only gate is verified rules, which JS-M3 already delivers for instruction selection. HGBA-2 4.2 requires "a baseline tier for startup, an optimizing tier for steady state" for page Wasm; U3 (compute-heavy apps and Wasm ports) is served in Stage 1 by baseline code only.
- What actually fixes T2 in Stage 2 is that **Stage 1 exit follows JS-M3** and JS-20 sets a 0.3×† Stage 1 threshold that presumes the optimizer's absence. That is a calendar decision: Stage 1 is allowed to exit without waiting for T2.
- There is also a **design-risk** argument against JS-D8 that the spec's own logic supplies: frame states, deopt points, stack maps, the code table and the optimizer facts that `.hgop` generates all constrain T0, T1 and `hg-heap`. If T2 is designed after them, its requirements arrive after their interfaces are fixed, which is the retrofit the spec rejects for rooting and the cage ("rooting is near-impossible to change later").

Other items staged without a first-principles argument: **JSPI and Stack Switching** (JS-M4, Stage 2) although HGBA-2 4.2 names JSPI as a reason page Wasm lives in the engine and the specifications note says a 2026 engine "has to target Wasm 3.0 plus Threads, JSPI ... from the first edition" ([SP 6.3][SP]); and the native IIP script domains (finding 5). Neither appears in 9.1's table, which claims to list every staged item.

**Verdict on JS-D8: mixed, and the deciding part is schedule.** "No optimizer rule runs before it is verified, and no speculation ships before the assumption ledger, validator and deopt-everywhere differential exist" is first-principles (P1 > P4). "T2 and W2 are Stage 2" is schedule: it lets Stage 1 exit at JS-M3 regardless of whether those gates could have been met.

**Required change.** Replace JS-D8 with a gate rule:
1. T2 and W2 are **designed from JS-M0**, alongside `.hgop`, frames, stack maps and the code table, so their requirements are fixed before the lower tiers freeze.
2. They are **built in parallel** with T1 and W1 by the optimizer and backend tracks, which are staffed from JS-M0.
3. Each rule is enabled when `hg-rulecheck` verifies it (JS-11). T2 and W2 **ship when their gates pass** (ledger, validator, deopt-everywhere and GC-stress differential, translation validation, JS-4 at 0 divergences), whatever the stage.
4. Stage 1 exit requires either T2 and W2 shipped under those gates, or an owner decision recording which gate failed. The reason must be correctness or security evidence, never the date.
5. JS-20's Stage 1 threshold is set for the full engine, or stated as "the tiers whose gates have passed".
6. JSPI moves to JS-M2 or JS-M3 with W0/W1, or 9.1 gives a first-principles reason for deferring it.
7. Section 9.1 lists every staged item.

The other 9.1 rows are acceptable: oracles and the rooting API first (correctness), shared structs and decorators waiting for TC39 (correctness), and live restore, hot reload and taint after measurement. "T0 and W0 complete before T1 and W1" is acceptable if it means the jitless ceiling must be complete to ship. It should not delay T1 work.

---

## Medium

### 7. Medium — The concurrent marker's data-race protocol is not specified

**Section:** 6.5 (`Field::set(&self, …)`, line 378; collector table "incremental and concurrent ... SATB"), 6.1 raw fields, 6.13 rope flattening.

**Problem.** `Field::set` mutates through `&self` while concurrent marking threads read the same slots. In Rust that is a data race (undefined behaviour) unless slot storage is atomic. Three cases need a stated protocol:
- field slots as atomics with named orderings;
- **shape transitions that change a slot's representation**: double to tagged, an in-object field becoming a raw field, or dictionary-mode conversion. The marker reads the layout descriptor and the slot at different times, so it can trace raw bits as a reference. Corruption stays inside the cage, but it is a GC-integrity bug that gives an attacker a primitive;
- the rope cache write, which the spec calls "an idempotent write that is safe to race" (6.13). That is only true for an atomic write.

Loom and TSAN (step 8) can test a protocol, but they cannot find one that has not been written down.

**Required change.** Specify slot atomicity and orderings, a rule for representation changes (never reinterpret a slot in place while marking is active, or synchronize with the marker), and the rope-cache store. Add each as a loom model and as a JS-10 planted bug.

### 8. Medium — Canonical GC accounting does not cover real memory, engine-internal allocation or nursery scavenges

**Section:** 6.9 GC rows (lines 500–502), 6.5 weak references (line 415).

**Problem.** GC triggers count only "each operation that semantically allocates". A real heap also fills with allocations that no description charges: shapes and transition tables, IC programs and feedback, compiled-code metadata, rope flattening, interned atoms and dictionary growth. These depend on the tier and on timing. The nursery fills by real bytes, so it can overflow before a canonical trigger fires, and a heap limit stated in canonical units can be exceeded in real bytes. If any collection that real memory forces can clear a weak target or enqueue a cleanup job, determinism fails. A scavenge that collects a young `WeakRef` target is exactly that case. The spec says "cycles finish only at deterministic points" but does not say whether scavenges are cycles.

**Required change.** State that collections forced by real memory are **unobservable**: they treat weak targets strongly, keep ephemeron entries and enqueue no cleanup, so only canonical-point cycles clear. Define how engine-internal allocation is charged. Define the real-OOM behaviour (a tier R process crash is allowed; a JS-visible difference is not). Add a DST case that forces real-memory GCs at random points and checks digests.

### 9. Medium — The Spectre posture assumes a co-location rule that I-1 relaxes

**Section:** 6.0 ("it never holds two sites", line 198), 6.7 Spectre posture, 6.2 ("no cross-site cache: one process holds one site").

**Problem.** HGBA-2 I-1 allows a **documented reduced-isolation mode** below a RAM threshold (Android, Stage 2 S-8), and the Stage 1 design repeats it. The spec states one-site-per-process as an absolute and builds on it: no speculation barriers or index masking, a per-heap megamorphic stub cache, and code caches. The spec targets Android in Stage 2 (Risks, last row), but it says nothing about what the engine does when sites share a process.

**Required change.** Add the reduced-isolation mode to 6.7. The broker tells the engine at launch, as part of the tier-ceiling grant, that the process is multi-site. The engine then enables Spectre hardening for JS and Wasm (index masking, speculation barriers on type checks, at a measured cost) and keeps its caches per site. The mode is published on `izcal://sandbox`. Add a reversal trigger for "no in-process Spectre hardening": any process that co-locates sites.

### 10. Medium — The code cache is keyed per site rather than by the network partition key

**Section:** 6.11 (line 547).

**Problem.** "Per site, in that site's storage partition." The renderer for a third-party iframe belongs to the script's site, so a cache keyed only by that site is shared across top-level sites. Whether a script compiles from cache is visible to page script through timing. I-1 extension (b) partitions the HTTP cache, and all network state, by (top-level site, container) for this reason. A code cache derived from HTTP responses must not undo that partitioning.

**Required change.** Key the code cache by the full network partition key (top-level site, container), the script URL, a content hash and the engine build. Add a cross-site cache-probing case to PRIV-2's test set.

### 11. Medium — Per-agent cages conflict with the shared heap the spec promises "needs no retrofit"

**Section:** 6.1 ("cage, one per agent heap"), 6.12 shared structs (line 564), JS-D1.

**Problem.** A compressed reference is an offset from *its* cage's base. A "shared heap per agent cluster ... with one-way references from agent heaps into it" (6.12) cannot be addressed by 32-bit offsets from several different cage bases, unless the shared heap lies inside every agent's cage (overlapping reservations) or all agents of a process share one cage. Shared structs, Wasm shared-everything threads and cross-agent GC objects all depend on this. The spec's own lesson is that representation and boundary must be decided together, before code (JS-D1). "Fits the cage model" is asserted without a design.

**Required change.** Decide now between one cage per agent and one cage per process (or per agent cluster) with per-agent sub-regions, or describe the overlapping-reservation layout and show that compression and H-1 still hold. Record the shared-heap reference encoding as part of JS-D1 and measure it at the JS-M0 spike.

### 12. Medium — Coding-standard tiers: engine crates handle untrusted sizes with release overflow wrapping, and JS-D10 puts the engine into a non-per-site process

**Section:** 6.15 tier column ("R" for every L2e crate), JS-D10.

**Problem.** (a) Tier R crates follow CS-D2 (b): **no overflow checks in release builds**. The validator, the bytecode verifier, the heap-image loader, the buffer table and the Wasm type canonicalizer do arithmetic on attacker-supplied counts and sizes. CVE-2024-2887 (a type-count limit aliasing an internal type) is the class the spec itself cites, and with checks off in release, an overflow wraps silently. CS-TY-12's checked arithmetic applies only to tier P. (b) JS-D10 runs Izcal's Wasm engine in a **component host** for extensions and privileged components. That is not a per-site process, so under CS-ST-10's "declared ≥ computed tier" rule the crates it links (`hg-wasm`, `hg-heap`, `hg-codegen`) compute to tier P, which may never panic on any input.

**Required change.** Require CS-TY-12-style checked arithmetic, with an `ast-grep` or lint check, in the engine's untrusted-input parsers and limit checks regardless of tier. Resolve the component host's tier (a separate tier-P build of the Wasm crates, or a documented exception), and add it to the coding standard's amendment row in 9.5.

### 13. Medium — Footprint: nodes are stored up to three times, the JS-D4 comparison uses the wrong baseline, and per-process engine cost has no target

**Section:** 6.2 JS-D4 (side tables "next to the data plane"), JS-30, JS-D4's reversal trigger.

**Problem.**
- Under JS-D4 a node's tree links are traced heap fields. Stage 1 5.4's data plane holds the same five links again. The Rust payloads, attributes included, sit in side tables "next to the data plane", and the data plane keeps its own `AttrArena`. The footprint strand already flags "Izcal's node exists twice" ([footprint compact_engine_data, section 1](../Izcal%20resource%20footprint/compact_engine_data.md)), and this spec adds a third copy of attributes, or leaves it unclear whether the side tables *are* the data plane.
- If the side tables are the data plane, script reads it, which contradicts B4 ("script never reads the data plane") and the F1 flush contract.
- JS-D4's reversal trigger compares against "a lazy wrapper" only. The comparison that matters for the budget is total bytes per node across heap node, side tables and data plane.
- The footprint notes identify **per-process fixed cost** as "the single most leveraged number" for site isolation, and "the JS engine's per-process baseline is the largest single item" ([history_postmortems 2–3](../Izcal%20resource%20footprint/history_postmortems.md)). This spec defines no engine fixed-cost metric: the private bytes of an empty agent heap and realm, the startup image's shared versus private pages, and per-worker cost.

**Required change.** State whether the side tables and the data plane are one store or two, and how B4's rules still hold. Make JS-D4's metric total bytes per node across all three structures. Add a JS-3x target, "engine private bytes for an empty renderer and per additional realm and worker", whose thresholds come from the footprint spec, so that spec has a number to budget against.

### 14. Medium — JS-D12 gives up cross-ISA determinism on an unmeasured cost, against M0 evidence

**Section:** 6.9 last row, JS-D12.

**Problem.** The recommendation (b) rests on "a measurable cost for a narrow benefit", but the cost has not been measured. M0 found the forked V8's hashes **identical across macOS arm64 and x86-64** for every variant ([determinism_crossplatform.md](../../m0/determinism_crossplatform.md), cross-OS table), so cross-ISA identity is not hypothetical. For JS, NaN canonicalization is needed only where bits become observable (typed-array and `DataView` float stores, Wasm reinterpret instructions), not on every float operation. Wasm 3.0 defines a deterministic profile that fixes relaxed-SIMD and NaN behaviour ([SP 6–7][SP]). Replay on the recorded ISA is also a real limit: a recording made on an arm64 Mac cannot be replayed on an x86-64 CI host.

**Required change.** Measure first: canonicalize at the observable boundaries for JS, and measure the Wasm deterministic profile's cost at JS-M3. Make (a) the default if the cost is at or below the stated bound. Keep (b) as the measured fallback, not the starting decision.

### 15. Medium — Several decisions have no reversal trigger, no stated alternative, or a trigger that can never fire

**Section:** 9.4, 6.17, and design choices that have no decision ID.

**Problem.**
- JS-D10 and JS-D14 have "—" as their reversal trigger.
- Design choices that are as load-bearing as the numbered decisions have no ID, no alternative and no trigger: engine-managed stack segments (6.6), bytecode in the cage with no trusted region (finding 1), cage per agent (finding 11), no in-process Spectre hardening (finding 9), and decision points as forced waits (finding 4).
- Some triggers need an alternative that no milestone builds: JS-D5's "hand-tuned handler set", 6.7's "measured out-of-process compile path" and JS-D3's LXR (which is scheduled at JS-M3, correctly). As written, the first two can never fire.

**Required change.** Give every listed choice an ID, alternatives, a cost and a trigger. For each trigger that needs a comparison build, name the milestone that builds it, or say the trigger is a watch item rather than a test.

### 16. Medium — M-1 part 3: the representation layer follows V8's internal design closely enough to need its own derivation

**Section:** 6.1–6.3, 4.4, 7.

**Problem.** Section 7 says "no ... design template or internal API shape is taken from V8". Yet the representation layer reproduces V8's internal taxonomy almost item for item:
- a low-bit small-integer tag with 31-bit payloads;
- an elements-kind lattice ("packed or holey, small-integer, double or tagged"), cited to V8's fast-properties post;
- validity cells, dictionary mode, a megamorphic stub cache keyed by (shape, key) and static roots;
- in the tiers, a register-accumulator bytecode, a Sparkplug-style frame-sharing baseline and an optimizer that transpiles CacheIR-style programs "as Warp does".

Each is cited as evidence. But 4.4's "assumptions kept" derivation covers only shapes and ICs, register bytecode and bytecode-first. The other items are adopted without a first-principles argument or a considered alternative. The owner asked for an engine "like nothing before". The spec's novelty is in generation, verification, determinism and the unified heap, while its representation layer is a recombination of shipping engines. That may be the right answer, but the spec has to show it was derived rather than inherited.

**Required change.** Add, per representation choice (tag scheme, elements kinds, validity cells, dictionary mode, stub cache), a short first-principles derivation from the properties of JS programs and the hardware, with at least one considered alternative and the evidence that decided it. Use Izcal's own names where a V8 term would suggest a template. Record in Section 7 that these were checked against M-1 part 3.

### 17. Medium — Staffing is likely understated in the tracks that carry the spec's distinctive claims

**Section:** 9.3.

**Problem.** Three rows look under-sized for what the spec commits them to, judged against its own scope rather than any engine's team:
- **Verification (4–5).** It must deliver ISA semantics for x86-64 and RISC-V where no official machine-readable specification exists (Q-JS-4), SMT verification of every lowering and check-removing rule on two ISAs, translation validation, the validator certificate format and a mechanized validator proof.
- **Engine security (3).** It must deliver 72-hour fixes for exploited bugs (JS-16), which in practice means an on-call rotation, plus nightly fuzzing on every tier and CFI work per OS.
- **Debugger and DevTools engine work.** It has no row (finding 5).

The plan also has no **ramp**. A team of 50–58† cannot be hired at JS-M0, given the spec's own "hiring JIT and GC specialists: High" risk, and JS-M1's 12–18 months† assumes the parser, runtime and generator tracks are full from the start. D-S8 (funding) is deferred and is a prerequisite for JS-M0. None of this is a reason to shrink the design. It is a reason to state the plan honestly.

**Required change.** Re-estimate the verification, security and observability rows from their deliverables. Add a hiring ramp, and make the milestone dates follow from it. State the on-call model behind JS-16. Keep the totals labelled †.

### 18. Medium — The security targets are graded by the people who built the mechanisms

**Section:** JS-10 (line 722), JS-13.

**Problem.** JS-10 passes when Izcal's own planted bugs, one per class, are caught by the mechanism that Izcal designed to catch them. That shows the mechanism exists, not that the class is closed. Variants of a fixed bug are the documented failure mode ([P0 "déjà vu"](https://projectzero.google/2021/02/deja-vu-lnerability.html), cited in Section 3), and an incomplete check is the usual result of being designed alongside the thing it checks.

**Required change.** Add mutation-generated variants per class, the in-cage corruption harness (finding 1), and an external review or red-team exercise per stage whose findings are published against JS-10's classes. Make JS-10's pass condition "caught" across the generated variants, not only the hand-written cases.

### 19. Medium — Release-list processing and `TokenCell` borrows need an explicit rule

**Section:** 6.5 "Finalization and DOM cleanup" (line 410), "Relation to `TokenCell`".

**Problem.** Side tables live in `TokenCell` (borrowed against `&ScriptToken`), and allocation takes only `&mut Mutator` (Q-CS-7). If the release list is processed inside a GC point reachable from `alloc`, a live `TokenCell` borrow of a side-table slot can be freed under it. Rust will not catch this if the release code reaches the table through `unsafe` or through ID lookups. The spec says only "at the next deterministic point".

**Required change.** State that release-list processing takes `&mut ScriptToken`. It runs only at event-loop points, never inside `alloc` or `call`, and the GC never touches side tables. Add the M2 test CS-TY-10 already plans ("allocates under a live `TokenCell` borrow with GC stress on"), with the release list non-empty.

### 20. Medium — A read-only startup image conflicts with mutable shapes and per-realm intrinsics

**Section:** 6.1 static roots, 6.2 ("an immutable heap object holding ... a transition table", line 255), 6.11 (line 546), 6.14.

**Problem.**
- A shape cannot be both immutable and hold a transition table, because adding a property to an object of a core shape adds a transition.
- Intrinsics are mutable per realm (`Array.prototype` can be modified).
- Mark bits and remembered-set entries for objects in a read-only mapping cannot be stored in the objects.

The spec maps the image read-only at every cage start and makes WebIDL's per-realm objects come from it without saying which parts are copied.

**Required change.** Define the read-only image's contents: immutable objects only, with transitions kept in a per-heap side structure and mark state kept outside the objects. Define what is instantiated per realm from the image, with JS-22's 1 ms† realm budget covering that copy.

---

## Low

### 21. Low — The canonical frame budget does not cover recursion inside Rust

**Section:** 6.9 stack overflow row (line 504), 6.6 stacks.

**Problem.** Parser recursion on deeply nested source, `JSON.parse` and `JSON.stringify`, structured clone, regex backtracking and Proxy trap chains recurse in Rust between or without JS frames. "The real segment is larger" only works if Rust recursion is bounded. Otherwise the result is a real stack overflow, which is an abort, and it happens at a depth that varies by compiler and OS.

**Required change.** Add canonical depth limits to every recursive Rust path reachable from content, each throwing the spec-appropriate error. Add a DST case with deeply nested inputs.

### 22. Low — A replay host that is always jitless cannot reproduce JIT-tier bugs

**Section:** 6.9 "Consequences for other specs" (line 514), JS-51.

**Problem.** Reversing REC-R6 so that "the replay host always runs jitless" is a good default for untrusted bundles (REC-R1). But exploited engine bugs live mostly in compilers. An engine engineer needs to replay a recording at its recorded tier, in an isolated replay host, to reproduce a T1 or T2 bug.

**Required change.** Keep jitless as the default for imported bundles. Add an engine-triage replay mode at the recorded tier, with REC-R1's isolation.

### 23. Low — The tier-ceiling state must live outside the cage

**Section:** 6.8 Baseline row (line 480).

**Problem.** The Baseline ceiling is enforced by an engine flag only. If the flag, or any tier-policy state, can be reached from the cage, an in-cage write raises the ceiling. This matters most when the emergency channel (UPD-R5) lowers ceilings because T2 is under active exploitation.

**Required change.** Store ceiling state outside the cage and make it read-only after initialization. Add a JS-10 case that tries to raise the ceiling from inside the cage.

### 24. Low — The W^X fallback and syscall filtering are not tied to HGBA-2 I-4

**Section:** 6.7 W^X (line 457).

**Problem.** The fallback "write-only alias mapped in the compiler thread's window" relies on a page permission that x86-64 and Arm do not offer: a writable page is also readable. A dual mapping is visible to every thread while it exists. HGBA-2 I-4 requires that pkeys always be paired with filtering of `pkey_mprotect` and `/proc/self/mem`, and the spec does not restate it. S-7's ≤2% bound is also missing from JS-15.

**Required change.** Name the syscall filters W^X depends on in each mode. Describe the alias window's exposure honestly. Carry S-7's bound into JS-15.

### 25. Low — There is no overhead target for the cage and its tables

**Section:** 9.5's S-4b row ("JS-20's overhead accounting"), JS-20.

**Problem.** JS-20 contains no overhead accounting. Decompression, buffer-table and code-table indirections, landing pads and the return-path check each cost something. JS-D1 measures only decompression. P6 requires a measurement protocol, and S-4b's replacement needs one.

**Required change.** Add a target: the cage and CFI overhead against an experimental build without them, on Speedometer 3 and JetStream 3, published per release, as S-4b did. Under P1 > P4 it is not a ship blocker.

### 26. Low — JS-5's denominator can be gamed

**Section:** JS-5 (line 715).

**Problem.** "≥95%† of steps in implemented clauses" lets the coverage figure rise by declaring clauses unimplemented.

**Required change.** Use all clauses at the pinned commit as the denominator, minus a published exclusion list (Stage 2.7 features, Annex B items not shipped).

### 27. Low — B10 is cited for tiering heuristics

**Section:** 6.3 Tiering (line 304).

**Problem.** "Never by runtime machine learning (B10)". B10 governs AI features: agents, models, AI network traffic. It does not govern internal heuristics.

**Required change.** Justify offline-tuned thresholds by determinism (B1) and reproducibility, which are the real reasons, and drop the B10 citation.

### 28. Low — The opening security premise needs a fact-check before it carries the design

**Section:** opening paragraph, Section 11.2.

**Problem.** "Every in-the-wild JS-engine exploit from 2018 to September 2026 was memory corruption reached through an engine logic error" is the security note's own tally. The spec already flags it, and the security scope's fact-check corrected a similar claim. The design does not depend on the word "every". The opening should not either.

**Required change.** Soften it to the measured share, with its source, once the fact-checker has confirmed it.

---

## Verdict on JS-D8 (owner's question)

**JS-D8 is mixed, and the part that decides the staging is schedule.**

- **First-principles part (keep).** No optimizer rule may run until `hg-rulecheck` has verified it, and no speculation may ship until the assumption ledger, the machine-code validator and the deopt-everywhere and GC-stress differential exist (P1 > P4). This is a **gate**.
- **Schedule part (remove).** Assigning T2 and W2 to Stage 2, setting a 0.3×† Stage 1 JetStream threshold that assumes they are absent, and letting Stage 1 exit at JS-M3 regardless of whether the gates could have been met.
- **Weak parts of the stated argument.** The IC-data argument supports the order in which T2 is enabled, not a stage boundary, because the data exist from JS-M1 or JS-M2. The infrastructure argument is already met at JS-M3, because T1 needs the same tools. W2 has no IC-data dependency at all. Option (b) is framed as shipping unverified, which nobody proposes.

**What the spec should say instead:** "T2 and W2 are designed from JS-M0 together with `.hgop`, frames, stack maps and the code table. They are built in parallel with T1 and W1 by tracks staffed from JS-M0. Each rule is enabled when verified (JS-11). Both tiers ship when their gates pass (ledger, validator, deopt-everywhere and GC-stress differential, translation validation, JS-4), independent of stage. Stage 1 exits with them, or with an owner decision recording which correctness or security gate failed. JS-20's Stage 1 threshold applies to whichever tiers have passed their gates." JSPI should be held to the same rule (finding 6).

## Summary

| Severity | Count | Findings |
|---|---|---|
| High | 6 | 1 (cage-escape routes beyond H-1), 2 (`Mutator` brand soundness), 3 (deopt, OSR and stack switching vs CET, GCS and PAC), 4 (deterministic decision points vs JS-40; missing recorded-input alternative), 5 (no Stage 1 JS debugger for DevTools), 6 (JS-D8 and JSPI staged by schedule) |
| Medium | 14 | 7–20 |
| Low | 8 | 21–28 |
| **Total** | **28** | |

[GC]: gc_memory.md
[SE]: security.md
[SP]: specs_semantics.md
