# M1 exit: the written Q-CS-1 answer, the JS-M0 table, and what is still open

12 November 2026. Measured against `main` at `d11deee` ("tools: hg-wast
checks its reading against the interpreter (#88)"). This is the JS-M0
plan's Phase 8. Nothing here cites or derives from any third-party
source code (M-1 part 3).

Stage 1's M1 row ([§8.1][S1]) names seven things. One of them — **the
written Q-CS-1 answer** — is this document's own deliverable; the other
six are claims about the tree, and each is checked here against
`d11deee` rather than against the note that first recorded it. The
JS-M0 exit table follows, from the JavaScript engine spec's
[§9.2][JS] row.

## The verdict, up front

**M1 does not exit yet.** Five of its seven clauses hold, one is this
document, and one does not hold:

- **Open: "ST-M0, TX-M0, LY-M0, GR-M0 and NW-M0 start."** None of the
  twenty crates those five milestones name exists in the tree. The
  workspace has 30 crates; every one belongs to the substrate, IPC,
  launch, process-skeleton or engine tracks. This is a staffing and
  sequencing fact, not a defect, but it is a clause of the exit
  criterion and it is not met.

**JS-M0 does not exit yet either**, on five items: four riscv64 ones
that need a K3 board which has not arrived, and the **ESMeta and
SpecTec harnesses, which do not exist** — found while producing this
document, because the exit row was checked against the tree rather than
against the crate's name. Whether JS-M0's state blocks M1 depends on a
reading the two specs do not settle, which §4 puts to the owner.

Everything else measured here passes, including the decision-point
probe, re-run on all three OSes at `d11deee` and bit-identical.

Two things were found by producing the evidence rather than by reading
the notes:

- **Q-CS-1's crypto rules could be escaped by two spellings**, one of
  them the spelling that beat H-3 on 10 November. Fixed in the same
  change; §1.3 is the measurement.
- **JS-M0 exit item 5's ESMeta and SpecTec harnesses do not exist.**
  The crate the spec names for them, `hg-js-oracle`, was built for the
  operation-level oracle instead, which is item 7 and passes. §3 item 5.

---

## 1. Q-CS-1, answered

**The question** ([coding standard][CS] Q-CS-1, decided at revision
0.3): *which binaries link `hg-substrate-sim`?* The answer the standard
recommended and Stage 1 recorded: process logic lives in libraries whose
entry function takes a `Substrate`; the shipping `hg-*-main` binaries
link only `hg-substrate-os`; one simulation binary (run by `hg-dst`) and
`hg-replay-main` link `hg-substrate-sim`; the graph lint enforces that
list. Stage 1's M1 row states the property to be shown at exit:
**production `Entropy::crypto()` is the OS CSPRNG only, no shipping
binary can seed it, and the seeded implementation is a
`hg-substrate-sim`-only type.**

Each of the three is a different kind of claim, and they are not equally
strong. Taken in order of how much weight they can carry.

### 1.1 The link list holds, and this is the part that is enforceable

`cargo tree -e normal --target all`, over the resolved graph rather than
over names, at `d11deee`:

| Package | links `hg-substrate-os` | links `hg-substrate-sim` |
|---|---|---|
| `hg-browser-main` | yes | **no** |
| `hg-renderer-main` | yes | **no** |
| `hg-net-main` | yes | **no** |
| `hg-substrate-os` (its probe binaries) | yes | **no** |
| `hg-dst` (the runner) | no | **no** |
| `hg-dst-sim` (`dst-sim`, `dst-smoke`, `js-probe`) | yes | **yes** |

`graph-rules.toml`'s `[q-cs-1]` list is `hg-replay-main` and
`hg-dst-sim`. `hg-replay-main` does not exist yet — it arrives with the
DevTools replay host (G12) — so the list is satisfied with one of its
two entries present and nothing else linking the simulator. The lint
checks this on three target triples in every feature combination, and
it is a **graph** property: no spelling of anything in source can
change which packages cargo resolves into a binary. This is the claim
that actually carries Q-CS-1.

### 1.2 The production source is the OS CSPRNG, and fails closed

`crates/hg-substrate-os/src/entropy.rs`: `OsCrypto::fill` is
`os::random::fill` and nothing else — `getentropy` on macOS, the
`getrandom` system call on Linux, `BCryptGenRandom` with the system
preferred generator on Windows, in the inventoried `os::random` module.
Named streams are xoshiro256\*\* **seeded from it**, and when that
seeding fails the crate calls `hg_types::invariant_failed` rather than
return predictable numbers (`unseedable()`), so there is no path from an
OS failure to a weak stream.

The seeded source is `SimCrypto` in
`crates/hg-substrate-sim/src/entropy.rs`. It and `SimEntropy` are
`pub(crate)`: they are absent from that crate's `pub use` list, so a
crate that links the simulator still cannot name either of them. That
is a compiler-enforced containment, not a lint, and it is the second
claim that carries weight.

Four implementations of the trait exist in the tree, and the other two
are test-only: `NoCrypto` inside `hg-substrate`'s own `#[cfg(test)]
mod tests` (it returns `Err(EntropyError::Unavailable)`), and one in
`hg-js-runtime`'s integration-test harness.

### 1.3 The crypto-name rules did not hold, and two spellings are recorded

`hg-graph-lint` also carried a code-level check: outside test code, only
`hg-substrate-os` and `hg-substrate-sim` may implement the
cryptographic-randomness trait. It had three rules — `crypto-impl` on
the impl, `crypto-alias` on a `use … as` rename, `crypto-macro` on a
macro definition naming the trait — and its own documentation claimed
"a rename or a generated impl cannot hide one".

That claim was false. All three rules read the trait's **name**, which
is the weakness H-3 was given a triple for and that a macro taking the
trait as an argument got past anyway on 10 November 2026
([js-10](../js-m0/js-10/README.md)). The same two questions were put to
Q-CS-1's rules here, as fixture workspaces linted with the real
`tools/graph-rules.toml`, and then on the real tree:

| | Spelling | Q-CS-1 findings, before | After |
|---|---|---|---|
| **E1** | A macro takes the trait as an argument: `macro_rules! wire { ($tr:path, $t:ty) => { impl $tr for $t { … } } }` then `wire!(hg_substrate::CryptoRandom, Seeded);` | **0** | 1 |
| **E2** | An allowed crate re-exports the trait under another name (`pub use hg_substrate::CryptoRandom as Rng;`), and the implementing crate writes only `impl hg_substrate_os::Rng for Seeded` | **0** | 1 |

E1 was then planted in the real tree, in `hg-net-service` — a crate
`hg-net-main` links — as a working seeded source whose `fill` runs a
linear congruential generator over the caller's buffer. It **compiled
with no error and no warning**, and `hg-graph-lint` reported
`clean (3 triple(s))`. The twin, the same tree with the plant removed,
was also clean, which attributes the silence to the planted code rather
than to the lint not running.

**What is fixed here.** A fourth rule, `crypto-macro-call`, owns the
macro **invocation**, so a trait name passed to a macro is seen where it
is written. And `crypto-alias` now binds **every** crate, the allowed
implementers included: an allowed crate may implement the trait, but no
crate may re-export it under another name, because that re-export is the
step that lets a later impl avoid writing the name. No legitimate
aliased re-export of the trait exists in the tree, so nothing was
grandfathered. With the fix, the real-tree plant is reported:

```
error[Q-CS-1] hg-net-service: crates/hg-net-service/src/lib.rs:257: names in a
macro call the cryptographic-randomness trait (wire!(hg_substrate::CryptoRandom,
PlantedSeeded)); only hg-substrate-os and hg-substrate-sim may, so a seeded
crypto source exists only in the simulator
```

and the tree is clean again once it is removed. Both spellings are now
regression tests in `crates/hg-graph-lint/tests/it/rules.rs`
(`a_macro_argument_cannot_hide_a_crypto_source`,
`an_aliased_re_export_cannot_hide_a_crypto_source`). `graph-rules.toml`
is unchanged, so this is not a gate-file change.

**What is still open, and why it cannot be closed by a lint.** A macro
**defined in another crate** and invoked with neither the trait's name
nor an alias in sight writes the implementation into the calling crate
while leaving no name in that crate's source. No rule on a name can see
it. Sealing the trait would, but Rust cannot express "only these two
crates": a private supertrait would lock out `hg-substrate-os` and
`hg-substrate-sim` as well, and moving the implementations into
`hg-substrate` so they could be sealed would put the seeded source in a
crate every shipping binary links — exactly what Q-CS-1 forbids.

### 1.4 The answer

**Q-CS-1 is answered as recommended, and the answer holds at `d11deee`.**
Production `Entropy::crypto()` is the OS CSPRNG only; the three shipping
`hg-*-main` binaries do not link the simulator on any of three targets;
and the seeded implementation is a `pub(crate)` type of
`hg-substrate-sim` that nothing outside that crate can name.

Those three facts rest on the resolved cargo graph and on Rust
visibility. They do **not** rest on the crypto-name rules, which are
defence in depth, which were measurably escapable in two ways before
today, and one of whose escape routes remains open by construction. The
lint's own documentation has been corrected to say so, because a
mechanism that is believed to be a guarantee and is not is worse than
one known to be a backstop. **For the owner:** whether the remaining
cross-crate-macro route is worth a stronger mechanism is a decision, not
a defect; §4 carries it.

---

## 2. Stage 1 §8.1's M1 row, clause by clause

| # | Clause | Verdict | Evidence at `d11deee` |
|---|---|---|---|
| 1 | Process logic starts under both substrates: libraries whose entry function takes a `Substrate`; shipping `hg-*-main` link only `hg-substrate-os`; only the simulation binary and `hg-replay-main` link `hg-substrate-sim`; the graph lint enforces it | **PASS** | §1.1's table; `hg-graph-lint` clean on three triples; `hg-replay-main` not yet built (DevTools G12), which the list tolerates |
| 2 | An IPC round trip runs in simulation with fault injection | **PASS** | `hg-dst run dst-smoke -- --ipc`, 39 seeds, 39 passed: a renderer launched with channels announces `ready`, then 200 `ping`/`pong` rounds at seeded delays through typed endpoints, under the fault plan's delay, reorder, drop, crash and commit-failure streams. The twin: `--bug` plants a refused-post assumption and DST finds it on **9 of 39 seeds**, groups them under one signature and prints a replay command |
| 3 | JS-M0's decision-point probe passes on the reference interpreter and `hg-heap` | **PASS** | §3, item 10: 1,000 runs of a seed give one hash on macOS, Linux and Windows, re-measured at this commit |
| 4 | ST-M0, TX-M0, LY-M0, GR-M0 and NW-M0 start | **OPEN** | None of `hg-htmlgen`, `hg-cssgen`, `hg-style`, `hg-style-reference`, `hg-encoding`, `hg-url`, `hg-fontgen`, `hg-ucd`, `hg-unicode`, `hg-layout`, `hg-layout-num`, `hg-layout-reference`, `hg-box-tree`, `hg-raster`, `hg-gpu-hal`, `hg-display-list`, `hg-net-wire`, `hg-netsim`, `hg-crypto` or `hg-tls` exists. `tools/tiers.toml` has 30 rows, none from those tracks |
| 5 | Recordings carry the engine build ID, the data versions and the decision-point log | **PASS** | `hg_types::recording`: `build_id: u64`, `data_versions: Vec<DataVersion>`, `decision_log: Option<DecisionLogRef>`, the last holding the trace hash and entry count of the one ordered log that carries the scheduler's decisions **and** every decision point a process records (JS-D15). `World::recording_header` fills it from `st.trace.hash()` and `st.trace.entries()`, so it carries the real log rather than a placeholder. The probe's **`trace=`** hash covers that log, because each GC completion is a trace line, and it is identical on all three OSes in §3's table — so the field is populated and agreeing, not merely present. (The probe's `state=` hash is the separate one: the heap's address-independent digest, every deref result, every cleanup call and the cycle count.) |
| 6 | Q-CS-1 is answered at M1's exit | **PASS (this document)** | §1 |
| 7 | SH-M0 is a gate **before M2** | n/a to M1 | The clause names M2, not M1's exit |

Clause 4 is the one that does not hold. Nothing in this session's work
could have met it: those five milestones are months of work each, in
their own specs, and the plan this session executed covers the engine
track only.

---

## 3. JS-M0's exit table ([JS spec §9.2][JS])

| # | Exit item | Verdict | Evidence |
|---|---|---|---|
| 1 | `.hgop` and `hg-opgen` generate the reference interpreter for the core operation set | **PASS** | 49 operations in `crates/hg-js-ops/ops/*.hgop` across load, convert, arithmetic, compare, control, property and intrinsic groups; `hg-opgen` emits the interpreter, the operation facts and the anchor index into `hg-js-ops/src/generated`. G16 holds: `hg-opgen check` passes at this commit and `oc16_the_checked_in_output_is_fresh_and_a_hand_edit_is_caught` runs under `cargo test --workspace` on all three OSes |
| 2 | The bytecode format, fixed by the design review, with a test assembler | **PASS** | [`bytecode-and-core-set.md`](../../../research_notes/JS-M0%20design%20review/bytecode-and-core-set.md); `crates/hg-js-bytecode` with `asm.rs` (the test assembler), `decode.rs`, `disasm.rs` and `unit.rs`. The verifier and the emitter are JS-M1, as the row says |
| 3 | `hg-heap` with the per-process cage, trusted region, branded `Mutator` API, precise roots, node space and a non-generational mark-region collector | **PASS** | `crates/hg-heap/src`: `cage.rs`, `trusted.rs`, `heap.rs`, `node.rs`, `space.rs`, with `verify.rs` (the post-GC verifier) and `corrupt.rs` (the in-cage corruption API) |
| 4 | CS-D3's five soundness criteria re-run against `hg-heap`, with the cross-heap compile-fail tests, Miri and the moving test collector | **PASS, with one half deferred and reasoned** | [cs-d3](../js-m0/cs-d3/README.md): criteria 1–4 PASS (two after fixes found by trying to produce the evidence); criterion 5 PASS for Miri, **loom deferred to JS-M1** because the cell API it would exercise has no concurrency until concurrent marking exists. The compile-fail fixtures at `d11deee` are 29 files in `crates/hg-heap/tests/ui/` and 4 in `crates/hg-js-runtime/tests/ui/` — 27 refusals and 6 controls, as `tools/checks.sh`'s `refusals` line reports, with 0 behaving other than the rule requires. (The cs-d3 note records 21 + 5 in the heap set, its state on 9 November; #86 added three more cross-heap spellings) |
| 5 | `hg-test262`, `hg-wast` and (if JS-D9) the ESMeta and SpecTec harnesses running | **PARTIAL** | **`hg-test262`: PASS.** [test262-engine262.md](../js-m0/test262-engine262.md): all 53,751 files, 103,264 runs, no exclusion list. **`hg-wast`: PASS.** [wast](../js-m0/wast/README.md): 259 scripts, 65,353 commands, 7,200 module spans re-parsed; the differential against the reference interpreter agrees on 259/259 and refuses 211/211 mutations. CI jobs `wast` and `wast-differential`. **The ESMeta and SpecTec harnesses: OPEN — neither exists.** JS-D9 was accepted on 19 October 2026, so the "if" is settled and they are required. `hg-js-oracle` has one mode, `ops` (`usage: hg-js-oracle ops --engine262 DIR …`), and the string `esmeta` or `spectec` appears nowhere under `crates/`, `tools/` or `.github/`. The `js-oracle` CI job runs the operation-level oracle alone. §6.15 assigns `hg-js-oracle` the "harness for ESMeta, SpecTec and the Wasm reference interpreter"; the third of those was built in `hg-wast` instead, and the first two were not built |
| 6 | The design review fixing the T2/W2 IR, frame states, the deopt model, stack maps, the code-table entry and the assumption ledger, together | **PASS** | [`research_notes/JS-M0 design review/`](../../../research_notes/JS-M0%20design%20review/): `optimizer-ir.md`, `frames-deopt-stackmaps.md`, `code-table-and-ledger.md`, `hgop-language.md`, `heap.md`, `js-host-hooks.md`, `riscv64-abi.md`, `windows-cet-probes.md` |
| 7 | The operation-level oracle for the core set | **PASS** | [op-oracle](../js-m0/op-oracle/README.md): 28 value-computing operations × 74 edge values = 99,308 cases against engine262, **0 mismatched**, 3 within 1 ulp (`Exp`, which ECMA-262 leaves approximated), 4 decided by Node where engine262 crashed. Each case runs twice, once under the moving test collector, and the two answers must agree. Sensitivity: one planted error (`Sar` as `>>>`) gives 657 mismatches and exit 1 |
| 8 | JS-10's planted bugs and the in-cage corruption fuzzer | **PASS, with two classes held open and reasoned** | [js-10](../js-m0/js-10/README.md): 202 variants, 199 twins, 401 compilations, 100% caught by the named mechanism at the named stage; two escapes found and the mechanisms they beat replaced. Two classes are **not** padded to twenty because they come due with concurrent marking at JS-M1 — the same reason CS-D3's loom half is deferred |
| 9 | Spikes measured: JS-D1, JS-D4, JS-D16, JS-D17 | **MEASURED; two leave live owner decisions** | [d1-values](../js-m0/d1-values/README.md), [d4-nodes](../js-m0/d4-nodes/README.md), [d16-trusted](../js-m0/d16-trusted/README.md), [d17-shared](../js-m0/d17-shared/README.md). JS-D4 and JS-D16 do not fire. **JS-D1 does on AArch64**: (c)'s 4 GB byte-offset encoding is 3.5–40% faster per dependent load than (a)'s ×8-scaled 16 GB one, because it decompresses in one instruction. **JS-D17's allocation-contention clause is live**: 23% at 8 agents, 10% at 4, 15% at 2, and the contention is in `Cage::acquire`'s chunk-pool mutex rather than in the decision |
| 10 | The decision-point probe: 1,000 runs of a seed give one heap-state, weak-state and decision-log hash, identical on macOS, Linux and Windows | **PASS, re-measured at this commit** | table below |
| 11 | The Windows shadow-stack questions of §6.6 answered on the reference device | **PASS, all five** | [`windows-cet-probes.md`](../../../research_notes/JS-M0%20design%20review/windows-cet-probes.md), on rig253 (Ryzen AI Max+ 395, Windows 11 build 26300). Switching works in user mode and `CreateFiberEx` serves as a shadow-stack allocator (creation outside the thread and fiber APIs does not); each fiber gets its own shadow stack and a switch round trip costs 60.8–71.9 ns against 50.6–55.7 ns without; JIT ranges are registered by the broker with one call per reservation, 3.4–4.0 µs plus ~9.3 µs IPC; Rust runs on a segment **once NT_TIB bounds are updated**, and without that a panic kills the process; strict mode can stay on with the JIT |
| 12 | riscv64 (JS-D22): the design review fixes register assignment, landing-pad labels and the segment-switch stub | **PASS** | [`riscv64-abi.md`](../../../research_notes/JS-M0%20design%20review/riscv64-abi.md), from the ISA manual's CFI chapter and the psABI chapter read as prose |
| 13 | riscv64: the instruction-cache protocol verified on a K3 board (JS-28 (c): 0 stale executions in 10⁶ installs, 8 threads, with migration) | **OPEN** | The board has not arrived |
| 14 | riscv64: Rust on stack segments verified on a K3 board | **OPEN** | The board has not arrived |
| 15 | riscv64: segment shadow-stack switching verified under emulation with Zicfilp and Zicfiss | **OPEN** | Not started. No shipping silicon with either extension was found, so this is the only way the JS-D19 discipline can be exercised on riscv64 |
| 16 | riscv64: the Sail model's coverage of V, Zicfilp and Zicfiss checked | **OPEN** | The riscv64 research note records this as unverified and assigned to JS-M0 |
| 17 | riscv64: the K3 pool running T1 | **OPEN** | JS-D22's CI pool is 16–24 boards; one is on its way |

**Item 10, re-measured.** Built from `git archive d11deee` on each host,
so all three trees held the same 1,441 files. The probe is not a timing
run, so the bookings were `light`.

| Host | OS, ISA | 1,000 runs of seed 7 | `trace=` / `state=` |
|---|---|---|---|
| Huey the Sixth (local) | macOS 27.0.1, AArch64 | `distinct=1` | `bb861c016d86bc8b` / `cf92a9d9bcf80240` |
| Huey the Destroyer (hueyj) | Linux 6.8.0-147-generic, x86-64 | `distinct=1` | `bb861c016d86bc8b` / `cf92a9d9bcf80240` |
| Huey the Sojourn (rig253) | Windows 10.0.26300.9974, x86-64 | `distinct=1` | `bb861c016d86bc8b` / `cf92a9d9bcf80240` |

Seed sensitivity, identical on all three hosts, so the hash discriminates
rather than being constant:

| Seed | `trace=` | `state=` |
|---|---|---|
| 1 | `447e3dfe16c4a941` | `63d8ca16c293f0f2` |
| 2 | `228ff05f37485aaf` | `8c3ec24730e6e53a` |
| 3 | `be0941ed9baa4f95` | `c1031b8ba4d8852d` |
| 7 | `bb861c016d86bc8b` | `cf92a9d9bcf80240` |
| 11 | `991264a5efbf9bed` | `0ae78e0438eff962` |

**Under the moving collector and GC stress**, which is also exit item 3's
collector and CS-D3 criterion 5's Miri configuration. `--stress move`
moves every object on every allocation; `--stress safepoint` moves at
every safepoint. Determinism must survive both, or the probe would only
be testing a collector that never moves anything:

| Host | `--stress none` | `--stress move` | `--stress safepoint` |
|---|---|---|---|
| local (macOS) | 1,000 runs, `distinct=1` | 1,000 runs, `distinct=1` | 1,000 runs, `distinct=1` |
| hueyj (Linux) | 1,000 runs, `distinct=1` | 1,000 runs, `distinct=1` | 100 runs, `distinct=1` |
| rig253 (Windows) | 1,000 runs, `distinct=1` | 1,000 runs, `distinct=1` | 100 runs, `distinct=1` |

The safepoint column is 100 runs on the two rigs rather than 1,000: that
mode is the slowest by a wide margin and 1,000 did not fit the booking.
The exit criterion's 1,000 is met in the mode it names, on all three
hosts, and the stress modes are additional evidence rather than part of
it.

Seeds 1, 2 and 3 are **bit-identical to the 31 October record**
([probe](../js-m0/probe/README.md)), which were taken on the same three
hosts before #85 and #86 changed the heap. So the probe's state hash has
not moved across the heap work, on any OS.

---

## 4. What is open, and who owns it

Each of these is a decision or a dependency, not a defect, and none can
be settled by this session.

1. **Does JS-M0's incompleteness block M1?** The two specs do not agree
   on the question. Stage 1's M1 row names only "**JS-M0**'s
   decision-point probe passes", which holds. JS spec §9.2's "Needed
   by" column says **JS-M0** — the whole milestone — is needed by
   Stage 1 M1, and items 13–17 are open on hardware. Under the first
   reading JS-M0's riscv64 items do not gate M1; under the second they
   do, and M1 waits for a board. **Owner.**
2. **Clause 4 of the M1 row: the five other tracks' M0s.** Starting
   them is staffing (D-S8 is a precondition the spec names and this
   session cannot meet). The exit criterion says "start", so the owner
   should either record what "start" means as a measurable thing, or
   amend the row. **Owner.**
3. **JS-D1 on AArch64.** (c) is 3.5–40% faster per dependent load than
   the chosen (a). The two encodings hold identical bytes, so this is
   the measured price of the 16 GB cage on that ISA. JS-D1's stated
   reversal trigger needs JetStream 3, which needs a JIT (JS-M1), so
   the trigger cannot fire yet — but the number is in hand now.
   **Owner.**
4. **JS-D17's allocation-contention clause**, 23% at 8 agents. The
   spike argues the contention is in `Cage::acquire`'s chunk-pool mutex,
   not in the one-cage-per-process decision. **Owner.**
5. **Q-CS-1's residual escape route** (§1.3): a macro defined in
   another crate. A lint cannot see it and Rust cannot seal the trait to
   two crates. Whether a stronger mechanism is wanted, or the graph
   check plus `pub(crate)` is accepted as the guarantee with the
   name rules documented as a backstop, is a decision. **Owner.**
6. **The CET Linux host** for `map_shadow_stack`'s restore token.
   Neither Linux rig has CET (hueyj is an i7-7700K; revire is a Xeon
   E5-2695 v2). **Owner names the host.**
7. **The K3 pool size.** One board covers items 13 and 14; JS-D22's CI
   pool is 16–24, so item 17 stays open with one board. **Owner.**
8. **The ESMeta and SpecTec harnesses** (§3 item 5). These are work, not
   a decision, but the owner should know the exit row is not met and
   decide the order. A note on sequencing: ESMeta is an ECMA-262 oracle
   and §8.6 names it against the parser, bytecode semantics and
   builtins, so nothing blocks it now. SpecTec's meta-interpreter is a
   differential oracle **for a Wasm validator** (§6.6), and Izcal has
   no Wasm engine until JS-M2, so there is a real sequencing argument
   for SpecTec that there is none for ESMeta. **Owner.**
9. **`hg_js_bytecode::open` re-parses a sealed unit on every call**
   (JS-D16 spike, finding 2). This needs code, not a decision, and is
   not an exit item for either milestone.
10. **JS-10 (d)**, the external review per stage, is a **JS-M3** item
   ("JS-10 green including variants and the first external review"), not
   a JS-M0 one. Recorded here so it is not lost.
11. **The design review's eight open questions** are still awaiting the
    owner, and five of them ask to accept amendments to Stage 1 3.3 and
    the JS spec that the review's own findings produced. The review
    itself is done and independently checked (§3 item 6), so this is an
    acceptance step, not missing work — but item 6's decisions are not
    yet reflected in the specs they amend. The list, with the review's
    recommendation on each, is under "Open questions for the owner" in
    [the review's README](../../../research_notes/JS-M0%20design%20review/README.md).
    **Owner.**

## 5. Method, and how to reproduce

Every number above was taken at `d11deee`. The commands:

```sh
# §1.1 the link list, over the resolved graph on every target
for p in hg-browser-main hg-renderer-main hg-net-main hg-dst hg-dst-sim; do
  cargo tree -p $p -e normal --target all --prefix none --locked
done

# §1.3 the escape probes, as fixtures against the real rule file
cargo test -p hg-graph-lint --test it -- --test-threads=1

# §1.3 on the real tree, and §2 clause 1
cargo run --locked -p hg-graph-lint -- --root . --locked

# §2 clause 2: the IPC round trip under faults, then its twin
cargo build --locked -p hg-dst -p hg-dst-sim
./target/debug/hg-dst run ./target/debug/dst-smoke --seeds 1..40 --jobs 4 -- --ipc
./target/debug/hg-dst run ./target/debug/dst-smoke --seeds 1..40 --jobs 4 -- --ipc --bug

# §3 item 1: the generated output is fresh
cargo run --locked -p hg-opgen -- check

# §3 item 10: the probe, on each host, built from the same archive
git archive d11deee | ssh <host> 'tar -x -C ~/izcal-exit'
./target/debug/hg-dst repeat ./target/debug/js-probe --seed 7 --times 1000 --jobs 4
for m in none move safepoint; do
  ./target/debug/hg-dst repeat ./target/debug/js-probe --seed 7 --times 1000 --jobs 4 -- --stress "$m"
done
for s in 1 2 3 7 11; do ./target/debug/js-probe --seed $s; done
```

On hueyj every command ran inside
`systemd-run --user --scope -p MemoryMax=9G -p MemorySwapMax=0`, as the
shared-host rule requires. On rig253 everything ran as `izcal-run`, the
standard account. Bookings: `b-133f1a` (hueyj, light) and `b-0a3d9c`
(rig253, light).

### What this document does not claim

- That any JS-M0 or M1 item not listed above is done. The two tables are
  the exit criteria as the specs write them; anything absent from them
  is outside this document.
- That the crypto-name rules are a guarantee. §1.3 says exactly what
  they are.
- That the five other tracks' M0s are close to starting. No work on
  them exists to assess.

[S1]: ../../../reports/HGBA%20Stage%201%20architecture%20design.md
[JS]: ../../../reports/Izcal%20JavaScript%20engine.md
[CS]: ../../../reports/Izcal%20coding%20standard.md
