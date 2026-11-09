# JS-D16 at the JS-M0 spike: what the trusted region costs per call and per dispatch

JS-D16 chose **(a)** a trusted region outside the cage for verified
bytecode, Wasm bodies, IC programs, frame states, deopt metadata, stack
maps and typed code-table entries, written only through a write window,
reached by index, with the argument count taken from the entry and never
from the cage. The alternative **(b)** keeps them in the cage and makes
the interpreter and the IC interpreter re-check every operand that
addresses anything outside the cage. The row prices (a) at "one table
indirection on calls and IC access; a write window; lifetime by reference
counts".

**The reversal trigger, verbatim:** "(b) measured cheaper by >2%† on the
corpus at the JS-M0 spike with equal JS-10 results (both built)".

## What this spike can and cannot decide

**It cannot settle the trigger, and does not claim to.** The trigger has
three parts and this spike can reach none of them whole:

- *"on the corpus"*. There is no corpus at JS-M0: no parser and no host
  hooks until JS-M1, so the only programs that exist are what the
  bytecode note's text assembler can write by hand. A share of a corpus
  cannot be computed from five hand-written programs.
- *"equal JS-10 results"*. JS-10 is the planted-bug suite (spec Section
  9.2): at least one case per bug class, 20† mutation variants each, the
  in-cage corruption harness nightly, and an external review. It gates at
  T0 and T1 and does not exist yet.
- *"both built"*. (b) is built here only as far as the reference
  interpreter's dispatch and entry paths: the IC interpreter, the
  compilers, deopt metadata and stack maps are JS-M1 work, and (b)'s cost
  for them is not measured.

**What it does measure**, on real cores with an exclusive booking: the
per-instruction cost of (b)'s re-checks against (a)'s trusted operands,
the per-entry cost of the trusted table indirection against (b)'s
re-reads from the cage, the write window's cost per unit sealed, and the
reference interpreter's own per-instruction time, so the first two can be
read as a share of something real rather than as a share of the harness.

## What an honest (b) has to do

(b) is only as expensive as H-2 forces it to be, so the first job was to
work out the minimum. H-2 (spec 6.1) makes everything read from the cage
untrusted, and Section 6.1 states the threat model it comes from: a
test-only in-cage corruption API gives a fuzzer "arbitrary read and write
inside the cage **while script runs**". From that:

1. **No parse of an in-cage image may be cached.** `UnitView::parse`
   checks the section bounds, the counts, the limits and every constant
   payload once. In the trusted region that result stays valid, which is
   the whole point of (a). In the cage it is a time-of-check result about
   bytes that may change, so (b) must re-read the header word it needs
   and re-check the section bound before every use.
2. **Every index taken from the cage must be re-checked against a bound
   re-read from the cage**: the code section's offset and length for an
   instruction fetch, the parameter and register counts a register
   operand resolves against (the frame state is in the cage under (b)
   too), the constant and slot counts, and the kind bytes.
3. **Jump targets need a bounds check only, not re-verification.** The
   decoder is total — `hg_js_bytecode::decode` turns any bytes into an
   instruction or an error, never a panic or an out-of-bounds read — so a
   jump that lands mid-instruction is a wrong program, not a memory
   error. (b) does not have to re-establish that targets are instruction
   boundaries, which would mean re-decoding from the start. This is the
   single biggest saving in the honest (b) and the harness takes it.
4. **A cheaper sound (b) exists under one assumption, and is measured
   separately.** If the only writer of the cage is the agent's own
   script, corruption can only happen where script could have run, so
   (b) may validate the whole image at a unit entry and after every
   operation whose facts allow user code (`Op::info().may_call_user`),
   and trust the operands in between. That is `b-cage-revalidate` below.
   It is **not** sound under JS-D17, which puts every agent of a process
   in one cage with per-agent blocks: a sibling agent on another thread
   writes concurrently, and the spec admits that case ("a corrupted agent
   heap reaching a sibling's gains nothing Spectre does not already
   give"). Both are reported, with which assumption each needs.

What (b) is **not** made to do here: it does not re-verify instruction
boundaries, it does not re-validate constants it does not read, and it
does not re-check anything twice per use. Both sides run the same check
code (`source::admits`, generic over a `Source` trait), so the only
difference between them is how the bytes and bounds are derived.

## Method

`harness/` is a standalone Rust crate (its own `[workspace]`, never part
of Izcal's workspace, no `unsafe`) that depends on the engine crates by
path and drives them through their public APIs only.

**The variants.** All five run the same program and must agree on a
digest of every instruction they decoded:

| Variant | What it is |
|---|---|
| `a-trusted` | (a) as a production tier would run it: the image in the trusted region, parsed once when it was sealed, operands trusted, no per-instruction check |
| `a-trusted-vec` | the same, with the image in an ordinary buffer — the placement control, which shows whether the address range itself matters |
| `a-ref-admits` | (a) as the reference interpreter runs it **today**: trusted bytes, but `Cx::admits` on every instruction anyway |
| `b-cage-use` | (b), sound with concurrent cage writers: no cached parse, every bound re-read, every operand re-checked |
| `b-cage-revalidate` | (b), sound only single-threaded: the whole image re-parsed at entry and after every `may_call_user` operation, trusted in between |
| `x-pad2`, `x-pad8` | not variants of JS-D16: (a)'s loop plus 2 or 8 dependent ALU operations that read nothing. The resolution diagnostic (below) |

**The loop** is the reference interpreter's own
(`hg_js_ops::generated::interp::run`): decode at `pc`, admit the
operands, take the next `pc` from the flow. The operation's semantics are
left out, because they are identical under (a) and (b) and would bury the
difference; a cheap digest of the decoded instruction stands in, and
branch directions come from a per-stream state that does not depend on
which variant is running, so every variant walks the same path.

**Both shapes.** One stream is the critical path: each `pc` depends on
the instruction before it. Four interleaved streams are the pipelined
shape, where independent dispatches can overlap. Both are reported,
because they answer differently — and on one of the two machines they
answer very differently.

**The clock is measured per cell.** hueyj runs the `powersave` governor
and moves between 800 MHz and 4.50 GHz, so every cell group reads
`/proc/cpuinfo` again, and the busiest core's figure is the one used. The
governor was read, never written. A dependent add chain runs beside each
calibration and is checked against the value the arithmetic must have
produced. On macOS there is no `/proc/cpuinfo`, so on mini the add chain
*is* the clock, which makes a cycle figure there circular: the AArch64
tables below are in nanoseconds only.

**Programs** (`harness/src/progs.rs`): `arith` (a ten-instruction
arithmetic loop, every operand kind but constants), `consts8` and
`consts64` (a constant index on every instruction, 8 and 64 constants),
`props` (`GetNamed`/`SetNamed` — a key constant and an IC slot on every
instruction, the IC-access shape the JS-D16 row names), `branchy`
(forward and backward targets). JS-M0 has **no JS-to-JS call opcode** —
`CallIntrinsic` is the only call — so the call path is measured on the
unit-entry path (`harness/src/entry.rs`), which is the mechanism a JS-M1
call will use: reach the unit, then take its parameter count, register
count and frame charge.

## The validity gate

`harness/src/main.rs` runs it before every measurement mode and sends it
to stderr so no results file can hold a measurement without it; a failed
gate prints FAIL and exits, and `d16-run.sh` stops the run. Gate output
is kept in `results/*/gate.txt`.

| Gate | What it proves |
|---|---|
| **G1 clock** | the calibration chain computed the value it must have, and the frequency is plausible. **This gate already earned its place**: the first hueyj run failed it, because the clock was taken as the mean of all cores and the idle ones sit at the governor's 800 MHz floor — 2.65 GHz instead of 4.50, which would have inflated every cycle figure by 1.7× |
| **G2 agreement** | all five variants decode the same instructions and agree on the digest and the instruction count, on every program. A variant that read the wrong bytes or stopped early cannot agree by accident |
| **G3 checks live** | with the counter on, (b) executes 5.1 to 8.6 checks per instruction, (a)+admits 1.0 to 2.8, and (a) exactly 0. The timed runs are built with the counter off, so this proves the work being timed is there |
| **G4 poison** | one operand byte is changed to a register outside the frame: (b) refuses the program, the harness's check refuses it, and (a)'s untouched trusted copy still runs — so the re-checks, not the storage, are what catches it |
| **G4b real admits** | the **reference interpreter itself** returns 500500 for the intact program and `BadBytecode(2)` for the corrupted one. Without this the harness could be timing a check that is not the engine's |
| **G5 interpreter** | the real interpreter returns the same value under (a) and (b), and it is the arithmetic the program must have done (sum 1..=1000 = 500500) |
| **G6 read-only** | on Linux, `/proc/self/maps` shows the trusted region's committed page as `r--p` with the rest of the reservation `---p`: `73ed68400000-73ed68401000 r--p` / `73ed68401000-73ed68800000 ---p`. macOS has no such file and the gate says so rather than passing quietly |

## The sequences, as emitted

From the binary that produced the numbers (`results/*/sequences.txt`),
through four `#[inline(never)]` functions. They are *representative* of
the inlined sequence, not identical to it, and each carries a `black_box`
stack round-trip of 3–5 instructions that the inlined version does not.

| Operation | AArch64 | x86-64 |
|---|---|---|
| (a) code access from the sealed view | **6** | **9** (5 of them the `black_box` round-trip; the access itself is two field loads) |
| (b) code access re-derived from the cage | **23** | **31** (two header loads, the `MAX_CODE` limit, the section bound, a `csel`) |
| (a) `TrustedRegion::get` — "one table indirection" | **34** | **42** (index bound, ×24 entry stride, live byte, kind byte, generation compare, offset+len overflow, committed bound) |
| (b) frame charge re-derived and re-checked | **26** | **36** |

## Dispatch: hueyj (Intel i7-7700K, 4.40–4.50 GHz measured per cell, `powersave`)

Minimum of 7 repeats, nanoseconds per dispatched instruction. Full data
in `results/hueyj-x86-64/dispatch.tsv`. Evidence:
`rig check OK: hueyj kind=exclusive session=js-m0-d16 | load 0.07/cpu (0.53 / 8 cpus) | bookings: b-209313 exclusive cpu by js-m0-d16 until 1:17am | game lock: free q0 | 2026-11-09T00:52:49-07:00`,
booking b-209313, released after the run (b-c6b508 held an earlier full
leg, run with the same harness before it was formatted; the line kept in
`results/hueyj-x86-64/rig-check.txt` is the one for the data here).

| Program | (a) trusted | (a)+admits (today's reference tier) | (b) per-use re-check | (b) revalidated |
|---|---|---|---|---|
| `arith` | 10.45 | 22.53 | **24.23** | 16.44 |
| `consts8` | 10.32 | 22.92 | **23.58** | 22.51 |
| `consts64` | 10.48 | 23.21 | **23.86** | 77.46 |
| `props` | 15.51 | 31.82 | **36.38** | 34.53 |
| `branchy` | 10.12 | 21.25 | **21.81** | 12.37 |

One stream above; with four interleaved streams the same ordering holds
and no figure moves by more than about a fifth (`arith` 11.60 / 22.60 / 23.84 / 17.72; `props` (a) 13.09 against 15.51),
because this core is throughput-bound in both shapes.

The placement control `a-trusted-vec` is within 1.1% of `a-trusted` on
every program, so none of this is about which address range the bytes
sit in.

## Dispatch: mini (Apple M4, macOS; nanoseconds only — see the clock note)

Minimum of 7 repeats. Full data in `results/mini-aarch64/dispatch.tsv`.
Evidence:
`rig check OK: mini kind=exclusive session=js-m0-d16 | load 0.13/cpu (1.31 / 10 cpus) | bookings: b-da3fe9 exclusive cpu by js-m0-d16 until 1:17am | game lock: free q0 | 2026-11-09T00:52:49-07:00`,
booking b-da3fe9, released after the run (b-78cc7c held an earlier full
leg).

| Program | shape | (a) trusted | (a)+admits | (b) per-use | (b) revalidated |
|---|---|---|---|---|---|
| `arith` | 1 stream | 9.08 | 7.94 | 8.21 | 9.54 |
| `arith` | 4 streams | **3.93** | 7.94 | **9.29** | 7.09 |
| `consts64` | 1 stream | 9.18 | 7.97 | 8.02 | 39.01 |
| `consts64` | 4 streams | **3.78** | 7.48 | **8.72** | 39.60 |
| `props` | 1 stream | 9.13 | 10.80 | 10.87 | 11.10 |
| `props` | 4 streams | **5.02** | 10.59 | 11.99 | 11.52 |

**At one stream on this core the numbers invert**: (b) measures *faster*
than (a) by 10–13%, which cannot be true of strictly more work. The
diagnostic says why, and it is the most useful single row in this note.

## The resolution diagnostic (`x-pad`)

`x-pad8` is (a)'s loop plus eight dependent multiply-rotate-xor
operations per instruction — at least 24 cycles of ALU work that reads
nothing and cannot be cheaper than not doing it.

| | (a) trusted | +2 ops | +8 ops |
|---|---|---|---|
| mini, `arith`, 1 stream | 9.08 | 9.00 (−0.9%) | 9.22 (**+1.5%**) |
| mini, `arith`, 4 streams | 3.93 | 4.24 (+7.9%) | 9.04 (**+130%**) |
| hueyj, `arith`, 1 stream | 10.45 | 11.04 (+5.6%) | 12.48 (**+19%**) |
| hueyj, `arith`, 4 streams | 11.60 | 11.79 (+1.6%) | 13.30 (+15%) |

So on the M4 at one stream, 24 cycles of extra work cost 1.5%: the loop
is latency-bound on the `pc` → decode → `pc` chain and has that much
slack per instruction, which is why the (b) rows land inside ±13% of (a)
in either direction. **The one-stream AArch64 inversions are the
measurement's resolution, not a result**, and nothing below ±13% should
be read from that column. The four-stream column on the M4 and both
columns on hueyj are throughput-bound, the work is exposed, and the
ordering is strictly monotone in the work each variant does.

The engine-level reading of the same fact: when dispatch is
latency-bound, (b)'s re-checks are nearly free because they fit in the
shadow of the fetch dependency; when it is throughput-bound — which is
what a production tier's dispatch looks like, and what this older x86-64
core looks like even in the reference interpreter — they are paid in
full.

## What it is a share of: the real reference interpreter

`results/*/e2e.tsv` runs the real `hg_js_runtime::Agent` on the same
program, taking the slope between a 1000-iteration and a 2000-iteration
loop so per-job work cancels:

| Rig | reference interpreter, (a) | (b) image re-parsed at entry |
|---|---|---|
| hueyj | **44.68 ns/instruction** | 44.92 ns/instruction |
| mini | **17.41 ns/instruction** | 17.36 ns/instruction |

Those two columns are the same within noise, and that is itself a result:
at JS-M0 the real interpreter's steady state cannot tell (a) from (b),
because the public API gives no way to make it re-derive per operand —
only the harness loop can express that. With the hueyj denominator, the
per-instruction deltas become:

| Comparison | hueyj delta | share of the reference interpreter |
|---|---|---|
| (b) per-use **against today's reference tier** (which already admits) | +0.6 to +4.6 ns | **+1.3% to +10.2%** (`arith` +3.8%) |
| (b) per-use **against (a) on a tier that trusts its operands** | +11.7 to +20.9 ns | **+26% to +47%** |
| (b) revalidated against today's reference tier, single-threaded assumption only | −8.9 (`branchy`) to +54.3 ns (`consts64`) | **−20% to +121%** |

The second row is the one that matters for JS-M1, and the share there is
an **underestimate**, not an overestimate: it is a share of the *slowest*
tier. The same absolute 12–21 ns of re-checking lands on every tier, so
on a production interpreter whose dispatch costs single-digit
nanoseconds it is a multiple, not a third.

## The call path

Nanoseconds per entry, minimum of 7 repeats
(`results/*/entry.tsv`). "Entry" is: reach the unit, then read its
parameter count, register count and frame charge — what a caller needs
before it can size a frame.

| Path | hueyj | mini | scales with |
|---|---|---|---|
| (a) `TrustedRegion::get` + three trusted fields | **2.91–2.93** | 1.39–1.44 | nothing |
| (b) re-read and re-check the same three from the cage | **2.92–2.93** | 1.08–1.09 | nothing |
| (a) `hg_js_bytecode::open` (the API as it stands: re-parses) | 18.9 / 28.9 / 140.0 | 6.6 / 14.6 / 70.0 | constants (0 / 8 / 64) |
| (b) revalidate at entry (`UnitView::parse`) | 13.8 / 24.6 / 131.2 | 3.6 / 13.4 / 68.5 | constants (0 / 8 / 64) |

End to end, at job granularity, the difference does not survive: 1190.3
ns per job under (a) against 1178.4 under (b) for `tiny8` on hueyj, and
5176.0 against 5157.6 for `tiny64` — the realm, constant materialization
and frame work dwarf a 10–30 ns entry. A JS-M1 call, at roughly a
hundred nanoseconds, is where an entry cost of this size would show.

The constant-read rows in `entry.tsv` (`a-const-read`, `b-const-read`)
are **not comparable** and are kept for completeness only: the (a) side
goes through `UnitView::constant`, which materializes a `ConstantRef`,
while the (b) side reads the payload bounds and one byte. The (b) figure
bounds (b)'s constant re-check from below, nothing more.

## The write window

(a)'s own cost, paid once per unit sealed, not per call
(`results/*/window.tsv`): **5321–7416 ns** per unit on hueyj and
**941–1112 ns** on mini, for a 128–1448-byte image, with exactly one
window (two protection flips) per insert as `TrustedRegion::windows()`
reports. The hueyj figure moved between this spike's two legs (3425–5392
ns in the first), which is itself the point: the cost is in the two
`mprotect` calls and it moves with the kernel's state, not with the
image. The 1448-byte unit costs 1.4× the 128-byte one on hueyj and 1.2×
on mini, while the image is 11×
bigger. Sealing a page's units one at a time therefore costs milliseconds
per thousand units; one window around many inserts would amortise it,
and nothing in the current API offers that.

## Where this contradicts the spec

1. **"One table indirection on calls and IC access" understates (a) and
   mis-attributes (b)'s cost.** The indirection is 34 machine
   instructions (index bound, entry stride, live byte, kind byte,
   generation compare, offset+length overflow, committed bound), 42 on
   x86-64, and 2.9 ns on hueyj, and it is **not cheaper than (b) doing
   the same job**: re-reading and re-checking the three frame numbers
   from the cage measured 2.92–2.93 ns against (a)'s 2.91–2.93 on hueyj,
   and *cheaper* than (a) on the M4 (1.08–1.09 against 1.39–1.44). (b)'s real cost is
   in the operand stream, not on the call. The cost column should say so;
   as written it suggests calls are where (a) pays and (b) saves, and the
   measurement says the opposite.
2. **The only API for reading a sealed unit re-parses it.**
   `hg_js_bytecode::open` runs `UnitView::parse` on every call, which
   re-validates every section bound and every constant payload: 18.9 ns
   for a unit with no constants and 140.0 ns for one with 64, against 2.9 ns
   for the table lookup alone. A JS-M1 call path built on `open` would
   pay O(constants) per call and (a) would not have the cost the row
   claims — and, worse, re-validating in (a) is pure waste, since the
   bytes are read-only. The trusted entry needs to carry the parsed
   header, or the view must be cached per unit. **This is the one finding
   that asks for code to change.**
3. **The reference interpreter already pays (b)'s per-operand check.**
   `Cx::admits` re-checks every operand of every instruction against the
   unit and the frame, which is exactly what H-2 would force on (b). So
   at JS-M0 (a) buys the *reference* tier almost nothing: the measured
   difference between today's interpreter and (b) is only the
   uncacheable view, +0.6 to +4.6 ns per instruction. What (a) buys is
   the right of JS-M1's *production* tiers to skip the check, worth
   11.7–20.9 ns per instruction at this core's speeds. The JS-D16 row and
   Section 6.1 read as though the interpreter were already exempt; it is
   not, and the saving is in a tier that does not exist yet.
4. **(b)'s cheapest sound form depends on JS-D17, not on JS-D16.**
   Revalidating only where script could have run is sound in a
   single-threaded cage and beats per-use re-checking on units with few
   constants (hueyj `branchy` 12.37 against 21.81 ns, `arith` 16.44
   against 24.23) while losing badly on constant-heavy ones (`consts64`
   77.46 against 23.86, because every revalidation re-walks the constant
   table). Under JS-D17 (a) — one cage per process, agents in their own
   blocks, possibly on other threads — it is not sound at all. If (b) is
   ever reconsidered, its cost is a function of the cage's thread model,
   and the two decisions have to be weighed together.

## What would settle the trigger

JS-M1, with: a parser and host hooks, so the corpus exists; T0 built
both ways, since the saving is in the tier that trusts its operands; the
IC interpreter, deopt metadata and stack maps built both ways, because
the JS-D16 row covers them and this spike covers none of them; and the
JS-10 planted-bug suite, for the "equal JS-10 results" half, which is a
security result and not a timing one.

**What these numbers bound.** They bound the per-instruction and
per-entry cost of the two storage disciplines on the reference
interpreter's dispatch, on two cores, with the checks proven to execute.
They do **not** bound a share of any benchmark, the cost for IC programs
or compiled code, or anything about JS-10. Nothing here fires the trigger
in either direction: (b) was nowhere cheaper than (a) by any margin on a
throughput-bound core, so the >2% condition was never approached from the
side it would need.

## What this leaves for the owner

**JS-D16 (a) stands, and nothing in this spike argues against it.** On
the path that matters most for JS-M1 — a production tier that trusts its
operands — (b) costs 11.7–20.9 ns per instruction on this x86-64 core, 26% to
47% of the reference interpreter's per-instruction time and a larger
share of any faster tier. The trigger needs (b) *cheaper* by 2%; the
measurement puts it 26–47% dearer in the shape that counts.

**One item needs code, not a decision:** `hg_js_bytecode::open`
re-parses a sealed unit on every call (finding 2). It should be fixed
before the JS-M1 call path is written on top of it, or (a) will quietly
pay 19–140 ns per call for re-validating read-only bytes.

**Two items are for the owner's judgement.** The cost column of JS-D16
says calls are where (a) pays; the measurement says calls are a wash and
the operand stream is where (b) pays (finding 1). And the reference
interpreter's `Cx::admits` means (a)'s benefit arrives only with JS-M1's
tiers (finding 3) — worth knowing when the JS-M1 plan decides what to
build first, because until a tier drops `admits`, (a) is paying for a
write window and a table indirection it is not yet collecting on.

**The write window wants batching.** 5.3–7.4 µs per unit sealed, almost
all of it in two `mprotect` calls, with no API to seal several units in
one window.

**riscv64 is untouched**, as in the JS-D1 spike: there is no riscv64
machine yet (the K3 board is on its way, JS-D22, plan Phase 6). Nothing
here is ISA-specific beyond the two cores' scheduling, but the
latency-versus-throughput finding says that a narrower core pays (b) in
full where a wide one hides it, so the riscv64 figure should be measured
rather than interpolated.

## Reproducing

```
cd harness
cargo build --release
./target/release/d16-trusted gate                 # must print only PASS
./d16-run.sh <rig-label> <out-dir> "<rig check line>" 4000000 2000000 7
python3 analyse.py <out-dir>
```

`d16-run.sh` refuses to run without a passing `rig check` line, writes it
beside the data, and stops if the gate fails. `rig` runs only on the Mac
that books the rigs, so for a remote leg the line is captured there and
passed in. No system setting is changed by anything here; the governor and
the clock are read for the record.
