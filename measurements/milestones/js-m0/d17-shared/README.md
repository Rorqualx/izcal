# JS-D17 at the JS-M0 spike: several agent heaps in one cage

JS-D17 chose **(a) one cage per process**, each agent heap in its own
blocks, with a later shared heap as further blocks in the same cage,
against (b) one cage per agent with overlapping reservations for a
shared heap and (c) one cage per agent cluster. Its stated reason is
"one base for every reference, so shared heaps and same-site bytecode
sharing need no second encoding; one reservation per process, as the
footprint budget assumes", and its accepted cost is "Agents of a process
share 16 GB†".

Its reversal trigger, verbatim: *"Address-space or allocation contention
between agents measured on a reference device; or a design that needs
isolation between same-site agents"*. This is that measurement, on the
real `ProcessCage`, `Heap::in_cage` and OS memory substrate that landed
in #60, #64 and #71.

**The trigger's allocation-contention clause is live.** In the one shape
that moves chunks — agents created and destroyed — per-agent throughput
in one shared cage is **23% below** the same agents in a cage each at 8
agents, and 10% below at 4. In the two shapes that do not move chunks —
a heap growing, and steady allocation with the collector reusing blocks
— the three arms are the same within run-to-run spread. The
address-space clause is **not** live on Linux and is **a finding on
macOS**, where one reservation becomes one kernel region per committed
megabyte.

## What this spike can and cannot decide

The trigger names two things and this harness measures both, but neither
is a whole-engine figure: JS-M0 has no JIT, no shared heap and no real
agent lifecycle, so "contention between agents" is measured as the three
allocation shapes an agent heap can have, driven as hard as the API
allows. The rates below (about 2,300 agent-heap creations per second per
thread in the churn shape, each taking and returning two 1 MiB chunks)
are far above anything a browser does, so they are an upper bound on the
contention, not a prediction of it.

The shared heap itself **does not exist yet**. `hg-heap` has
`ProcessCage` and `Heap::in_cage`, so the "several agent heaps in one
cage" half of JS-D17 is real code and is measured as such. The
shared-heap brand of the shared-structs paragraph is not, so leg 3
measures the two things (a) claims to avoid — a second base and a
run-time brand check — and tests the one-way rule as a prototype in the
same brand discipline `hg-heap` already uses. That prototype is not a
test of shipped code, and the README says so again where it appears.

## The validity gate

`d17-heaps gate` runs before any timing is kept and prints PASS or FAIL
per check; `d17-run.sh` stops the whole run if any line fails. A fast
wrong number is this spike's main failure mode, so each claim the legs
rest on is checked first. From `results/hueyj-x86-64/gate.txt`:

| Check | What it proves | Result on hueyj |
|---|---|---|
| `one-reservation` | four agent heaps made through `Heap::in_cage` cause exactly **one** `Memory::reserve`, of 16 GiB | 1 reservation, 17179869184 bytes |
| `offsets-in-one-cage` | every object of every heap decodes, by JS spec 6.1's public formula `4·v − 4`, to an offset inside that one cage | 24 chunks, all in [1, 16384) |
| `chunks-disjoint` | no 1 MiB chunk is used by two agents | 20 chunks, each one agent's |
| `pool-interleaved` | the agents really draw from **one** pool: ownership changes hands repeatedly across the cage's chunks, which a cage per agent could not produce | 8 runs of ownership |
| `chunk-size` | the 1 MiB chunk this harness assumes is the cage's own, and chunks come out contiguous | 3 contiguous chunks of 1048576 B |
| `work-checked` | the allocations are not dead code: the checksum read back out of the heap is the one the arithmetic must give | 499500 = 499500, 1001 allocs, 0 refused |
| `clock` | the per-cell clock reading is sound | OS max 4449 MHz, reference 1.218 ns/add |
| `os-view` | the kernel's own view of the address space is readable, so the substrate's ledger can be checked against it | 30 mappings, VmSize 15228928 B |
| `concurrency` | four barrier-synchronised threads really overlap | overlap 1.000 |

Two further gates are **per cell**, and a cell that fails one prints
`FAIL` in its verdict column instead of being quietly kept:

- **overlap** — each contention cell records every worker's own window
  and keeps the cell only if `(min end − max start) / mean duration ≥
  0.9`. This gate earned its place: the first sweep used cells of
  24–60 ms, where thread start-up was a large part of the window, and
  **nine** cells across the three shapes failed it. The sweep was re-run
  with cells of 130 ms to 1.3 s, and **two** cells still fail — both of
  them the third rep of the 4-agent grow shape, at overlap 0.87.
  `contend-grow.tsv` keeps them, marked `FAIL`, and the 4-agent grow
  medians below are taken from the two reps that passed.
- **cycle and sum** — the leg 3 micro-benchmark walks a random cycle and
  requires that each lap returns to its starting slot and that the sum
  of the payloads is the value the walk must have produced. A walk that
  fell out of its cycle would be fast and wrong.

**The clock is measured per cell**, not taken from a nominal figure. Two
readings are kept: the per-core MHz the kernel reports, sampled before
and after each cell, and a dependent add chain. The chain is **not**
assumed to be one add per cycle — it goes through a `black_box` barrier
each iteration, which on these cores costs about 5.4 cycles per add, as
the data shows — so it serves as a relative speed reference whose
`ref_cyc_per_add` column must hold steady across a sweep. It does:
5.21–5.54 across every cell of every leg. hueyj's cores sat between
4400 and 4508 MHz throughout, so unlike the JS-D1 spike (which saw
1874–2215 MHz) there is no frequency drift to correct for here. **The
`powersave` governor was left alone**; changing it is a system setting.

## Leg 1: address space

`results/hueyj-x86-64/space.tsv`, `space-max.tsv`, `reservations.tsv`,
`churn-maps.tsv`, `maps.txt`, and the same files under
`results/local-arm64/`.

### "One reservation per process" holds — on Linux, all the way down

The substrate's ledger says one reservation at every agent count from 1
to 256, as the footprint budget assumes. The question the budget does
not ask is what that one reservation becomes in the kernel, and the two
OSes answer differently:

| Agents | Chunks committed | Committed | Linux: mappings in the cage | macOS: mappings in the cage |
|---|---|---|---|---|
| 1 | 2 | 2 MiB | 3 | 4 |
| 8 | 16 | 16 MiB | 3 | 18 |
| 32 | 64 | 64 MiB | 3 | 66 |
| 128 | 256 | 256 MiB | 3 | 258 |
| 256 | 512 | 512 MiB | **3** | **514** |

On Linux the cage stays three mappings whatever happens to it — with
256 agent heaps in it, and through 2048 rounds of agents being created
and destroyed (`churn-maps.tsv`, 4608 commits and 4096 decommits, and
still three). `maps.txt` is the raw evidence:

```
72a063200000-72a063300000 ---p    # chunk 0, never handed out
72a063300000-72a066b00000 rw-p    # every agent's committed chunks, one mapping
72a066b00000-72a463200000 ---p    # the rest of the 16 GiB, still reserved
```

Two properties of the cage's chunk pool produce this: chunks are handed
out from a rising watermark, so agents' chunks are adjacent and their
`mprotect` ranges merge; and a returned chunk goes back to a free list
from which the lowest adjacent run is reused first, so a hole is
refilled rather than left. `vm.max_map_count` on hueyj is 1048576 and
the whole process used 33 mappings.

**On macOS the same reservation fragments into one region per committed
megabyte.** `results/local-arm64/maps.txt` shows each 1 MiB chunk as its
own `Untagged ... rw-/rwx SM=COW` region; the count is `chunks + 2` and
rises linearly, reaching 514 regions at 256 agents and 846 for the whole
process. A 16 GiB cage committed in full would be 16,384 regions.
macOS has no `vm.max_map_count`, and nothing failed, but anything whose
cost is per region — `vmmap`, `task_info`, the kernel's VM map — scales
with committed cage memory on macOS and not on Linux. **This is not what
the plain reading of "one reservation per process" suggests, and it is
reported here rather than smoothed over.** It is a property of the OS,
not of JS-D17: (b) would pay it too, per cage.

### The per-agent cost is the chunk, and the cage holds 16,383 agents

An agent heap that has allocated nothing costs nothing: `Heap::in_cage`
takes no chunk until the first allocation. An agent heap that has
allocated **one object** costs **one 1 MiB chunk**, and that is the
floor. `space-max.tsv` walks it: 3,073 agent heaps, each with one
object, committed 3,222,274,048 bytes — 1 MiB each, exactly — and
stopped at the harness's own 3 GiB commit cap, not at anything the cage
did. The cage's arithmetic ceiling is **16,383 agents** at that floor
(16,384 chunks of 1 MiB, less chunk 0, which is never handed out so that
no object lives at cage offset 0).

So "agents of a process share 16 GB†" resolves to: 16,383 agents that
allocate almost nothing, or 1,023 agents with 16 MiB of heap each, or 63
with 256 MiB. Resident memory tracked the commit closely throughout
(419 MB RSS at 537 MB committed, 256 agents). The same floor holds on
macOS, whose page size is 16 KiB rather than 4 KiB: 1,537 agent heaps
with one object each committed 1,611,661,312 bytes, again 1 MiB apiece
(`results/local-arm64/space-max.tsv`, stopped at that machine's 1.5 GiB
cap). Note for anyone reading a footprint budget off `VmSize` or
`VmPeak`: the reservation is in both, so a process with one empty cage
reports **17.19 GB** of virtual size.

### What alternative (b) would have cost in address space: not much

(b) needs a 16 GiB reservation per agent, and the obvious question is
whether a process can hold them. It can hold far more than any agent
count needs:

| OS | 16 GiB reservations one process made | Total |
|---|---|---|
| Linux x86-64 (hueyj) | **8,190**, the 8,191st refused | 128.0 TiB |
| macOS arm64 (local) | **8,161**, the next refused | 127.5 TiB |

Both land on the 47-bit user address space. At 4,096 reservations the
process had 64 TiB reserved and 3.7 MB resident, and on Linux the
reservations themselves still merged to 30 mappings. **So JS-D17's "one
reservation per process" is a footprint-accounting argument, not an
address-space-exhaustion argument**: (b) would not have run out of
address space at any plausible agent count. The reason to prefer (a)
stands on the encoding (leg 3), not on this.

## Leg 2: allocation contention — the trigger's own measurement

`results/hueyj-x86-64/contend-grow.tsv`, `contend-steady.tsv`,
`contend-churn.tsv`. Three arms, each with one agent heap per thread,
barrier-synchronised, three reps, medians of the reps that passed the
overlap gate:

- **shared** — JS-D17 (a): one `ProcessCage` and one `Memory`, every
  agent heap inside it.
- **percage** — JS-D17 (b): a cage per agent, one `Memory`. This is the
  honest comparison for JS-D17, because a process has one memory
  substrate whichever cage granularity it picks.
- **private** — a cage *and* a memory per agent: nothing shared in the
  substrate at all. Not reachable in a real process; it is here to
  separate the cage's own lock from the substrate's.

Three shapes, because they stress the cage differently. **grow**: one
live chain of 2,000,000 cells per agent, so the heap grows and takes
chunks (46 chunks per agent). **steady**: 8,000,000 cells allocated into
scopes that exit, so the collector reuses blocks and the heap stops
growing (1 chunk per agent). **churn**: 800 agent heaps per thread, each
taking 40 blocks and then dropped, so chunks are acquired and returned
continuously (1,600 commits and 1,600 decommits per agent).

### Steady allocation and heap growth: no contention

| agents | grow: shared | percage | private | steady: shared | percage | private |
|---|---|---|---|---|---|---|
| 1 | 15.60 M/s | 15.81 M/s | 15.04 M/s | 15.69 M/s | 15.48 M/s | 15.32 M/s |
| 2 | 15.83 | 15.25 | 15.85 | 15.27 | 15.54 | 15.27 |
| 4 | 13.77 | 15.13 | 15.28 | 15.53 | 14.63 | 15.10 |
| 8 | 8.23 | 8.41 | 8.32 | 7.58 | 7.65 | 7.60 |

(allocations per second per agent). The arms are the same within spread
in both shapes: at 8 agents they agree to 2%. The drop from 4 to 8
agents — 1.8× to 2.1× per agent — is the machine, not the cage: hueyj
is a 4-core i7-7700K with 8 threads, so eight agents share four cores,
and the drop is the same in all three arms. **Per-agent throughput is
flat from 1 to 4 agents** (within 2% in `private` and `percage`), which
is what "no interference" looks like.

The one 4-agent grow cell where `shared` is 9.9% below `percage` is
noise, not a finding: its median is over the two reps that passed the
overlap gate, and `percage` and `private` agree with each other to 1%.

### Agents coming and going: contention, and where it is

| agents | shared | percage | private | shared vs percage | shared vs private |
|---|---|---|---|---|---|
| 1 | 92,005/s | 96,313 | 96,405 | **−4.7%** | −4.8% |
| 2 | 80,687 | 92,450 | 91,376 | **−14.6%** | −13.2% |
| 4 | 54,776 | 60,432 | 80,392 | **−10.3%** | −46.8% |
| 8 | 24,927 | 30,745 | 39,927 | **−23.3%** | −60.2% |

(allocations per second per agent, the median of three reps; every cell
passed its overlap gate, 0.94–1.00.) Per-agent throughput as agents are
added, against the ideal of no interference:

| arm | 1 → 4 agents | 1 → 8 agents |
|---|---|---|
| ideal (no interference) | ×1.00 | ×1.00 |
| the machine's own floor (from grow and steady) | ×1.00–1.06 | ×1.81–2.07 |
| **shared** (JS-D17 (a)) | **×1.68** | **×3.69** |
| percage (JS-D17 (b)) | ×1.59 | ×3.13 |
| private (nothing shared) | ×1.20 | ×2.41 |

Reading the 8-agent column against the machine's ×2.0 floor, the churn
shape loses a further ×1.85 in the shared arm. That splits in three:

- **×1.18 is the cage's own chunk pool** (shared → percage). This is the
  cost JS-D17 is accountable for.
- **×1.30 is the memory substrate's ledger** (percage → private). A
  process has one of these whichever cage granularity it picks, so (b)
  would pay it too. It is an independent finding about
  `hg-substrate-os`, not about JS-D17.
- **×1.21 remains in the private arm**, where nothing in user space is
  shared. The candidate is the kernel's per-process address-space lock,
  which every `mprotect` and `mmap` takes: all the cages of a process
  are in one process under (a) and under (b) alike. This spike does not
  separate it; the ×1.21 bounds it from above.

**The cage's share has a located cause, not a guessed one.**
`Cage::acquire` in `crates/hg-heap/src/cage.rs` takes the chunk pool's
`Mutex` and then calls `commit` — an `mprotect` syscall on Linux — while
still holding it, so agents acquiring chunks serialise for the length of
a syscall. The return path does not: `Cage::release` decommits before it
takes the lock. Two independent facts agree: the arm that gives each
agent its own pool lock (`percage`) recovers ×1.18 of the ×1.85 while
changing nothing else, and the lock is held across the commit by
inspection. **This is a measurement, not a patch**: committing outside
the pool lock, or giving each agent heap a small chunk cache, are the
obvious shapes of a fix, and neither needs JS-D17 to change. Confirming
the cause by making that change and re-running is the next step and is
not done here.

## Leg 3: brand checks and the cost of a cross-heap reference

### The brand costs nothing at run time, because it is a type

`Gc<'h, 'm, T>` is four bytes — one 32-bit compressed slot — and `'h` is
a `PhantomData` on the mutator. There is no run-time check to time. What
can be checked is that the discipline actually rejects what it claims
to, which `d17-oneway.sh` does by compiling it:
`results/hueyj-x86-64/oneway.tsv`, with each refusal's own compiler
message under `oneway-errors/`.

| Case | Expected | Outcome | Error |
|---|---|---|---|
| `control-same-heap` | compiles | compiles | — |
| `control-oneway-allowed` | compiles | compiles | — |
| `fail-cross-heap-brand` | refused | refused | `E0521` |
| `fail-gc-across-gc-point` | refused | refused | `E0502` |
| `fail-mutator-sent` | refused | refused | `E0277` |
| `fail-oneway-cross-agent` | refused | refused | `E0521` |
| `fail-oneway-shared-to-agent` | refused | refused | `E0308` |

`fail-cross-heap-brand` is the one that matters for JS-D17: two agent
heaps in the **same** `ProcessCage`, where a slot from one decodes off
the same base to a real object in the other's blocks, so nothing in the
encoding stops the write. The brand does, at compile time (`E0521`).
Without a compiler every case would "fail to compile", which would read
as the rule holding, so the script refuses to run without `rustc` — the
first run on hueyj reported two controls as FAIL for exactly that
reason, and the check was added.

The last two rows are the **one-way rule, as a prototype**
(`harness/oneway/shared-brand.rs`). `hg-heap` has no shared heap, so
this is not a test of shipped code; it is a test of whether the rule the
spec states — "a shared-heap brand that agent heaps may reference one
way" — is expressible in the discipline `hg-heap` already uses, and
whether the compiler then enforces it. It is: with an invariant `'g`
brand for the cluster's shared heap and a fresh `'h` per agent heap, an
agent heap's object may hold `SharedRef<'g>` (the direction the spec
allows), a shared-heap object has no setter that takes an `AgentRef` at
all (the forbidden direction is unrepresentable, `E0308` when attempted),
and two agent heaps' references do not mix (`E0521`). Both reference
types are four bytes.

### What (a) buys: nothing on the load, unless the slot must say which cage

`results/hueyj-x86-64/xheap.tsv`, with the emitted sequences in
`xheap-sequences.txt`. A random cycle, one dependent load per step, each
decode pinned in inline assembly, four variants:

| Variant | What it is | x86-64 sequence for the reference load |
|---|---|---|
| `one_base` | JS-D17 (a): one process-wide base | `mov -0x4(%rbx,%rax,4),%esi` — **1** |
| `brand_dynamic` | (a) plus the one-way rule checked at run time | `mov`/`sub`/`cmp`/`jae` then the same load — **1 + 4** |
| `two_base_static` | (b) where the field's type names the cage | the same load off the other base — **1** |
| `two_base_dynamic` | (b) where the slot must name the cage | `test $0x2`/`cmovne`/`mov`/`shr` then the load — **1 + 4** |

Nanoseconds per dependent step, and the change against `one_base`:

| Footprint | `one_base` | `brand_dynamic` | `two_base_static` | `two_base_dynamic` |
|---|---|---|---|---|
| 8 KiB (L1) | 1.122 (4.9 cyc) | 1.133 (+1%) | 1.144 (+2%) | **1.799 (+60%)** |
| 64 KiB | 2.125 (9.2) | 2.045 (−4%) | 2.005 (−6%) | **2.578 (+21%)** |
| 512 KiB | 6.799 (29.5) | 6.779 (−0%) | 7.034 (+3%) | 7.445 (+10%) |
| 2 MiB | 9.687 (42.0) | 9.778 (+1%) | 9.744 (+1%) | 10.268 (+6%) |
| 16 MiB | 50.76 (220) | 49.05 (−3%) | 48.95 (−4%) | 49.51 (−2%) |
| 64 MiB | 72.27 (313) | 75.39 (+4%) | 72.48 (+0%) | 73.36 (+2%) |

Three readings:

- **A run-time brand check is free on this core.** Four extra
  instructions, and no measurable cost at any footprint, because the
  compare and its perfectly predicted branch sit off the load's critical
  path. If the shared-heap brand ever has to be dynamic rather than a
  type, it costs nothing here.
- **A second base is free when the type names it.** `two_base_static`
  matches `one_base` everywhere. So (b) does not cost anything per load
  for a field whose type says which heap it points into.
- **A second base the slot must carry costs +2.9 cycles per dependent
  read when the heap is cache-resident** (+60% at 8 KiB, +21% at 64 KiB)
  and nothing once memory latency dominates. That is the case JS-D17
  (a)'s "no second encoding" avoids, and it is the only one of the three
  that costs anything. It is also the case that matters: a tagged field
  that may hold either an agent-heap or a shared-heap reference cannot
  have its base chosen at compile time.

The earlier version of this harness dispatched on the variant **inside**
the inner loop, which added the same few cycles to every variant and
compressed exactly the differences being measured. Each variant now has
its own loop; `xheap-sequences.txt` is taken from the binary that
produced the table above and shows the four loops as emitted.

### Interleaving agents' chunks costs an agent nothing

`results/hueyj-x86-64/interleave.tsv`. One agent heap's own dependent
pointer chase, built in 16 rounds while 0, 1, 3 or 7 other agents grew
alongside it, so its own chunks end up 1, 2, 4 or 8 chunks apart in the
shared cage:

| Other agents growing alongside | chunk stride | ns per dependent step |
|---|---|---|
| 0 | 1 | 2.998 |
| 1 | 2 | 3.351 |
| 3 | 4 | 4.177 |
| 7 | 8 | 3.110 |

No trend: the 3-others cell is the slowest and the 7-others cell the
second fastest, and the spread within a cell (2.9–4.6 ns across reps)
is as large as the spread between them. **Sharing a cage does not hurt
an agent's own locality** at this chain length (10 chunks spanned,
10 MiB touched). It cannot change the encoding — one base, by
construction — and it does not appear to change where the misses fall.

## What this leaves for the owner

**The trigger is live on its allocation-contention clause, and the
number is 23%.** At 8 agents in the agent-lifecycle shape, per-agent
throughput in one shared cage is 23% below the same agents with a cage
each; at 4 agents, 10%; at 2 agents, 15%. Zero in the two shapes that do
not move chunks. Whether that fires the reversal is the owner's call,
and three things bear on it:

1. **The contention is in an implementation detail, not in the
   decision.** `Cage::acquire` holds the chunk-pool mutex across the
   `mprotect` that commits the chunk. Commit outside the lock, or give
   each agent heap a small chunk cache, and the ×1.18 the shared arm
   loses to the pool should go, with JS-D17 (a) untouched. That is the
   change to make and re-measure before reversing anything.
2. **The rate is far above a browser's.** The shape that produces the
   contention is 2,300 agent-heap creations per second per thread, each
   taking and returning two 1 MiB chunks: about 4,600 chunk acquisitions
   and as many returns per second per thread. No page does that. The 23%
   is the contention's ceiling, not its expected value.
3. **(b) would not escape most of it.** Of the ×1.85 the shared arm
   loses beyond the machine's own floor, only ×1.18 is the cage; ×1.30
   is the memory substrate's ledger and ×1.21 is unexplained and most
   likely the kernel's per-process address-space lock. Both of those are
   paid under (b) as well, because (b)'s cages are in the same process.

**The address-space clause is not live, and (a)'s address-space
justification is weaker than the row suggests.** One reservation per
process holds exactly, and on Linux the kernel keeps it as three
mappings with 256 agents in it and 2048 agent lifetimes through it. But
a process can hold **8,190** 16 GiB reservations on Linux and 8,161 on
macOS, so (b) was never going to run out of address space. JS-D17's
"one reservation per process, as the footprint budget assumes" is a
budget-accounting argument; the argument that carries weight is the
encoding one, which leg 3 confirms.

**Two things that contradict, or complicate, what is written down.**

- Section 6.1's cage row and the footprint budget speak of one
  reservation per process. On **macOS** that one reservation becomes
  **one kernel region per committed megabyte** — 514 regions at 256
  agents, and 16,384 if a 16 GiB cage were committed in full. Nothing
  failed and macOS sets no limit, but any cost that is per region scales
  with committed cage memory there and not on Linux. Worth a line in the
  footprint spec, and worth checking on Windows before it surprises
  anyone.
- The shared-structs paragraph says a reference from an agent heap into
  the shared heap "is an ordinary 32-bit compressed reference: no second
  base". Measured, the second base costs nothing **where the field's
  type names the heap**; it costs +2.9 cycles per dependent read only
  where the slot must carry the selector. The claim is right, but its
  value is concentrated in tagged fields, not in references generally.

**What was not measured, and why.**

- **Windows.** rig253 was held under other sessions' exclusive windows
  all evening (`jfm-hitpause` until 1:12am, `staticr-61` 1:14–2:44am),
  so no leg ran there. This is the gap that matters most, because
  Windows commits sub-ranges of a `MEM_RESERVE` block with
  `VirtualAlloc MEM_COMMIT` and tracks commit per page inside one
  reservation: its region count may behave like Linux's or like macOS's,
  and the macOS result above says the answer is not obvious. The harness
  builds there; `osview` has no Windows implementation yet, so the
  mapping count would need `VirtualQuery` added.
- **AArch64 timing.** Both arm64 Macs were held by other sessions
  (`local` under a heavy booking until 1:08am and another from 1:10am;
  `mini` exclusive until 1:07am), so no exclusive window was available
  and no timing is kept from either. `local` contributed only the
  address-space legs, which are counts and not times. The AArch64
  sequences in `d17-xheap.c` compile and run, and on an idle machine
  they should be re-run: on AArch64 the one-way check and the base
  select both need real instructions where x86-64 folds them into the
  addressing mode, and the JS-D1 spike found exactly that asymmetry
  mattering there.
- **A shared heap.** There is none in `hg-heap`, so leg 3's one-way rule
  is a prototype of the spec's rule, not a test of shipped code, and the
  cross-heap read cost is priced by proxy rather than measured on a real
  shared heap.
- **The kernel's share of the churn cost.** Separating Linux's
  per-process address-space lock from the rest would need the substrate
  calls driven with no user-space lock at all, which means unsafe code
  in the harness. The `private` arm's residual ×1.21 is the upper bound
  reported instead.
- **The trigger's second clause** — "a design that needs isolation
  between same-site agents" — is a design question about what the engine
  will need, not something this harness can answer.

## Running it

```sh
# On the measuring machine. The rig-check line is captured on the Mac
# that books the rigs and passed in, because `rig` runs only there.
cd measurements/milestones/js-m0/d17-shared
(cd harness/d17-heaps && cargo build --release)
harness/d17-run.sh "<rig check line>" results/<rig>          # all legs
harness/d17-oneway.sh results/<rig>                          # the brand cases
```

`D17_COMMIT_CAP_MIB` caps what the `space-max` leg may commit and
`D17_RES_LIMIT` caps the `reservations` leg, for a machine sharing its
memory with another session. The harness changes no system setting.

`harness/d17-heaps` is a probe, never shipped: it is its own workspace,
depends on Izcal's crates by path, and forbids `unsafe_code`.
