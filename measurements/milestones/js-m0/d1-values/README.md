# JS-D1 at the JS-M0 spike: what compression costs per load

JS-D1 chose (a) 32-bit compressed tagged slots, references scaled by 8
in a 16 GB cage, against (b) 64-bit NaN-boxed values with full pointers
and (c) the same as (a) with 4 GB byte-offset references. Section 6.1
claims an instruction count per ISA and calls the cost "a
first-principles claim measured at the JS-M0 spike". This is that
measurement.

**What this spike can and cannot decide.** JS-D1's reversal trigger is
*decompression measured at >1% on JetStream 3*, and its heap-number
trigger is *heap-number allocation >5% of T0/T1 time*. Both need a
running engine with a JIT, which JS-M0 does not have (JS-M1 does). So
this spike measures the thing underneath the trigger — the cost of one
decompressed load against one full-pointer load, on real cores, with
each encoding's instruction sequence pinned in assembly — and leaves the
trigger itself open with its evidence recorded. The same applies to
"share of integers outside the 31-bit range", which needs the corpus.

## Method

`harness/d1-chase.c` walks a random cycle of nodes, one dependent load
per step, with the decompression written in inline assembly so the
compiler cannot turn one encoding into another. A node is two slots, the
smallest object that holds a reference: 8 bytes under (a) and (c), 16
under (b). `harness/d1-run.sh` builds it, records the emitted sequences
and sweeps the working sets.

Four things make the numbers mean what they say:

- **The cycle is checked.** One lap must return to its starting slot
  before any timing is kept. A walk that fell out of its cycle would be
  fast and wrong.
- **Two footprints.** `equal-nodes` holds the graph the same, so (b)
  walks twice the bytes and its extra cache misses count against it —
  the question a heap designer asks. `equal-bytes` holds the footprint
  the same, which isolates the decompression instructions from the
  footprint — the question an ISA designer asks. Both are reported,
  because they answer differently.
- **Two shapes of load.** A dependent chain (`chains=1`) puts the
  decompression on the critical path, the worst case. Four independent
  chains (`chains=4`) let one load's decompression overlap another's
  latency, which is closer to compiled code. Both are reported.
- **The clock is measured per cell**, from a dependent add chain, not
  taken from the nominal figure. hueyj runs the `powersave` governor and
  its cores idle at 800 MHz; during these runs the clock sat between
  1874 and 2215 MHz, so a single calibration would have turned a
  frequency difference into an apparent encoding difference. The
  governor was left alone: changing it is a system setting.

**What the microbenchmark overstates.** Every load here pays a fresh
decompression. Real generated code decompresses a reference once and
then reads several fields of that object at immediate offsets, so the
per-field cost is lower than the per-load cost measured here. These
figures are an upper bound on the per-load part, which is why the >1%
JetStream trigger cannot be settled from them.

## The instruction sequences, as emitted

Taken from the binaries that produced the numbers
(`results/*/sequences.txt`), which is the check that each encoding is
the one Section 6.1 names:

| Encoding | x86-64 | AArch64 |
|---|---|---|
| (a) `scaled32` | `mov -0x4(%rdi,%rax,4),%eax` — **1** | `add x9, x0, w8, uxtw #2` then `ldur w8, [x9, #-0x4]` — **2** |
| (b) `raw64` | `mov (%rax),%rax` — **1** | `ldr x8, [x8]` — **1** |
| (c) `byte32` | `mov (%rdi,%rax,1),%eax` — **1** | `ldr w8, [x0, w8, uxtw]` — **1** |

Section 6.1 is confirmed on x86-64: the scale and the −4 are the
addressing mode, so (a) decompresses in no instructions of its own.

On AArch64 it is confirmed for (a) — the extended-register add and a
load at −4, two instructions — but the spec adds that this is "the same
instruction count as V8's unscaled scheme". The byte-offset variant (c)
assembles to **one** instruction here, a register-offset load, because
it needs neither a scale nor a displacement. So on AArch64 (a) costs one
dependent ALU operation per decompression that (c) does not. That
asymmetry is what the numbers below price, and it is the reason JS-D1's
trigger names (c) as the fallback.

## x86-64 (hueyj, i7-7700K, L1d 32 KiB/core, L2 256 KiB/core, L3 8 MiB)

Full data in `results/hueyj-x86-64/`. Evidence:
`rig check OK: hueyj kind=exclusive session=js-m0 | load 0.17/cpu | bookings: b-24f95d exclusive cpu by js-m0 | 2026-11-08T23:51:07-07:00`,
with a closing line at 23:49 on the run before it. Booking b-24f95d,
released after the run.

**At equal footprint** (`equal-bytes`, dependent chain) the three
encodings are the same within run-to-run spread, which is the direct
statement that decompression is free on this ISA:

| Footprint | (a) scaled32 | (b) raw64 | (c) byte32 |
|---|---|---|---|
| 128 KiB | 2.59 ns | 2.39 ns | 2.40 ns |
| 1 MiB | 8.97 ns | 9.04 ns | 9.02 ns |

**At equal node count** (`equal-nodes`, dependent chain) (b) pays for
its 16-byte nodes:

| Nodes (a)/(c) bytes | (a) scaled32 | (b) raw64 | (c) byte32 | (a) vs (b) |
|---|---|---|---|---|
| 16,384 — 128 KiB | 2.59 ns | 4.18 ns | 2.42 ns | **−38%** |
| 131,072 — 1 MiB | 9.16 ns | 10.24 ns | 9.02 ns | −11% |
| 2,097,152 — 16 MiB | 52.4 ns | 65.8 ns | 51.3 ns | −20% |
| 8,388,608 — 64 MiB | 73.3 ns | 78.0 ns | 72.4 ns | −6% |

With four independent chains the same ordering holds and the gaps
narrow (0.67 / 1.00 / 0.67 ns at 128 KiB; 18.2 / 19.3 / 17.9 ns at
64 MiB).

**x86-64 reading: (a) costs nothing against (c) and beats (b)
everywhere**, by 6–38% depending on how much of the working set the
smaller nodes keep in cache. Nothing here argues against (a).

## AArch64 (local, Apple M6, L1d 96 KiB, L2 8 MiB, 128-byte lines)

Full data in `results/local-aarch64/`. Evidence:
`rig check OK: local kind=exclusive session=js-m0 | load 0.39/cpu | bookings: b-2fda6b exclusive cpu by js-m0 until 12:59am | 2026-11-09T00:03:11-07:00`,
with a second line at 00:08 during the run. Booking b-2fda6b, released
after the run. The clock held between 4.47 and 4.76 GHz across every
cell, so unlike hueyj there is no frequency drift to correct for.

**At equal footprint** (`equal-bytes`, dependent chain), which prices
the extra instruction with the footprint held constant:

| Footprint | (a) scaled32 | (b) raw64 | (c) byte32 | (a) over (c) |
|---|---|---|---|---|
| 8 KiB | 1.41 ns (6.4 cyc) | 0.66 ns (3.0) | 0.88 ns (4.0) | **+59%** |
| 64 KiB | 1.42 ns (6.5) | 0.66 ns (3.0) | 0.88 ns (4.0) | **+62%** |
| 1 MiB | 3.18 ns (14.6) | 2.67 ns (12.2) | 2.54 ns (11.6) | +25% |
| 8 MiB | 7.07 ns (31.8) | 6.55 ns (29.9) | 6.42 ns (29.3) | +10% |
| 64 MiB | 64.2 ns (292) | 62.7 ns (286) | 61.0 ns (283) | +5% |

The extra dependent ALU operation costs about **2.4 cycles** per
decompression, not the 1 the instruction count suggests: the add feeds
the load's address, so it lands on the critical path and the core cannot
fold it the way x86-64's addressing mode does. Its share falls as memory
latency grows, from +60% when L1-resident to +5% out in DRAM.

**At equal node count** (`equal-nodes`, dependent chain) — the question
a heap designer asks, where (a)'s smaller nodes pay for themselves:

| Nodes — (a)/(c) bytes | (a) scaled32 | (b) raw64 | (c) byte32 |
|---|---|---|---|
| 1,024 — 8 KiB | 1.40 ns | **0.64 ns** | 0.85 ns |
| 8,192 — 64 KiB | 1.42 ns | **0.78 ns** | 0.85 ns |
| 131,072 — 1 MiB | 3.26 ns | 3.87 ns | **2.67 ns** |
| 1,048,576 — 8 MiB | 7.29 ns | 8.15 ns | **6.54 ns** |
| 8,388,608 — 64 MiB | 74.5 ns | 82.2 ns | **71.9 ns** |

Two things to read here. Compression does pay for itself against full
pointers once the working set leaves the caches: (a) beats (b) from
1 MiB upward, by 16% at 1 MiB and 9% at 64 MiB, and loses badly below
that (2.2× at 8 KiB) where footprint buys nothing and the extra
operation is the whole difference.

But **(c) is faster than (a) at every working set measured**, by 39%,
40%, 18%, 10% and 3.5% as the set grows. (a) and (c) hold the same
bytes and differ only in cage size — 16 GB against 4 GB — so on this
core the 16 GB cage costs one dependent ALU operation per decompression
and buys nothing back. With four independent chains the gap narrows but
keeps its sign (+65% at 64 KiB, +2% at 64 MiB).

**AArch64 reading: (a) is the slowest of the three when the heap is
cache-resident and the middle one when it is not; (c) dominates it
throughout.**

## riscv64

Open. There is no riscv64 machine yet; the K3 board is on its way
(JS-D22, plan Phase 6). `harness/d1-chase.c` and `harness/d1-intadd.c`
both carry the riscv64 sequences — `sh2add.uw` plus a load at −4 for
(a), and `addw`/`add`/`bne` for the small-integer overflow check.

**Those sequences are written but unverified: they have never been
assembled.** No machine here has a riscv64 target (Apple's clang is
built without one and neither Linux rig has a cross-toolchain), and
installing one is a system change not asked for. Expect to fix syntax on
the board before the first run, and read the riscv64 instruction counts
in this note as the spec's claim, not as something this spike checked.

Section 6.18 claims two instructions for (a) on riscv64, the same as
AArch64. Note that (c) needs **two** there as well, not one: riscv64 has
no register-offset load, so the byte-offset variant needs its own `add`.
If that holds on the board, the AArch64 asymmetry does not exist on
riscv64, and (a) costs (c) nothing there.

## The small-integer overflow check

`harness/d1-intadd.c` prices the other per-ISA cost in the JS-D1 row:
with tag 0 and 31 payload bits a tagged add is one machine add whose
overflow flag decides exactness, and riscv64, having no flags register,
must instead compare a 32-bit add against a 64-bit one.

The honest limit of this file: riscv64 needs **three** instructions
(`addw` sign-extends for free and `bne` compares and branches in one),
while the same check written on AArch64 needs **five** and on x86-64
**six**. So the `explicit` rows measured on the machines we have are an
upper bound on the shape of the cost, not riscv64's figure. The figure
itself waits for the K3.

Each row is checked against the value the arithmetic must have produced.
That check earned its place: the first version of this harness let the
compiler put the accumulator and the addend in one register, which
turned the chain into a doubling, overflowed 31 bits after about 31
steps and left the loop early — reported as 0.0000 ns per add, a fast
wrong answer. The addend is now read through `volatile`, and a row whose
accumulator does not match is printed as FAIL instead of as a
measurement. Every row below passed its check.

| | AArch64 (M6) | x86-64 (i7-7700K) |
|---|---|---|
| `flags`, dependent | 0.227 ns — **1.02 cyc** | 0.236 ns — **1.03 cyc** |
| `explicit`, dependent | 0.248 ns — **1.12 cyc** | 0.493 ns — **2.18 cyc** |
| cost of the check, dependent | **+0.09 cyc (+9%)** | **+1.15 cyc (+112%)** |
| `flags`, independent | 0.101 ns — 0.45 cyc | 0.287 ns — 1.25 cyc |
| `explicit`, independent | 0.158 ns — 0.70 cyc | 0.578 ns — 2.54 cyc |
| instructions | 2 against 5 | 2 against 6 |

A tagged add with the flag branch is one cycle on both cores, as the
derivation assumes. The explicit check's cost then depends entirely on
how well the core schedules it: on the M6 the two extra operations sit
off the critical path (the 64-bit add is the one that feeds forward) and
cost under a tenth of a cycle; on the older x86-64 core the same
sequence, one instruction longer, costs a full extra cycle.

**This is good news for riscv64, but it is not riscv64's number.** Its
sequence is three instructions where AArch64 needs five and x86-64 six,
so it should land at or below the +0.09 cycle figure — the cheaper of
the two upper bounds. That has to be confirmed on the board.

## What this leaves for the owner

**x86-64 raises nothing.** Decompression is free in instructions and in
time, and (a) beats full pointers by 6–38%. JS-D1 (a) stands.

**AArch64 is the finding.** (c) — the same encoding with a 4 GB
byte-offset instead of a ×8-scaled 16 GB one — is faster than (a) at
every working set measured, by 3.5% to 40% per dependent load, because
it decompresses in one instruction where (a) needs two and the extra one
lands on the load's critical path. The two encodings hold identical
bytes, so this is the price of the 16 GB cage on this ISA, and Section
6.1's parenthetical that (a) costs "the same instruction count as V8's
unscaled scheme" does not match what the assembler emits for (c) here.

**This spike cannot fire JS-D1's trigger**, which is a >1% share of
JetStream 3, and a per-load delta is not a share of a benchmark: how
much it matters depends on what fraction of engine time is dependent
loads through compressed slots, which needs the JIT that JS-M1 builds.
The per-load deltas are far above 1%, so the trigger is live rather than
settled, and the JS-M1 run should be treated as a real decision point
rather than a formality.

**What the owner may want to weigh now**, since it is a cage-size
question and not only a speed one: (a) gives a 16 GB cage, which Section
6.1 justifies by large Wasm GC and DOM heaps and which is the limit
V8's 4 GB design is cited as paying for. (c) gives 4 GB and, on
AArch64, one fewer dependent operation per decompression. Nothing needs
deciding before JS-M1, and no change is recommended here.

**riscv64 is open** for both halves, and its sequences are unassembled.

