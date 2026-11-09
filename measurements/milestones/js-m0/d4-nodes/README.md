# JS-D4 at the JS-M0 spike: bytes per node, and the node space's GC share

JS-D4 chose **(a)**: node columns outside the cage, traced as a node
space by the one collector; a lazily created wrapper held strongly once
created; DOM text in an arena shared with JS as external strings; the
data-plane mirror retired. Its reversal trigger, verbatim from the
[JS spec](../../../../reports/Izcal%20JavaScript%20engine.md)'s Section
9.4: *"At the JS-M0 spike or on the corpus, total bytes per node across
every copy, or the node space's share of GC time, worse than (c) by
>10%† (the spike builds both)"*. This is the spike.

The comparison is against **(c)** — the node as a cage heap object that
workers read during a flush — because that is the option the trigger
names. Option (b), 0.1's design with the data-plane mirror, stores links
twice and is not the comparand.

## What this spike can and cannot decide

**The corpus half belongs to JS-M1.** The trigger has two limbs, "at the
JS-M0 spike **or** on the corpus". There is no corpus at JS-M0 — no
parser, no host hooks and no DOM exist yet — so this spike settles only
the first limb, on synthetic trees sized from published statistics. The
corpus limb is JS-M1's, and nothing here should be read as having closed
it.

**One term has no published value, and it decides the answer.** Both
limbs of the byte comparison turn on the fraction of a page's nodes that
script ever touches, because that is what creates wrappers. The DOM
spec's **Q-DM-6** records plainly that no published figure exists and
sets it as a corpus measurement for DM-M1. So every total below is
reported as a function of that fraction, with the crossover computed,
rather than as a single number. Where this note states a verdict it
states the condition on it.

**This is not `hg-heap`'s collector.** `hg-heap` has no node space: the
interface is Phase 2 item 7 of the JS-M0 plan and it is blocked on four
design decisions the design review did not fix (see "What this leaves for
the owner"). So both layouts are built in this harness, under one
collector written to the same rules — H-1's masked, aligned, atomic cage
access, JS-D1 (a)'s slot encoding, H-4's object header and field order,
H-7's one tracing loop that marks and evacuates together, and mark bits
in side bitmaps outside the cage. The absolute times are this harness's,
not Izcal's. The trigger asks for a **ratio between two layouts under one
collector**, and that is what is measured.

**The open decision is a parameter, not an assumption.** JS-D4 (a) needs
the wrapper to name its node and the collector to follow that edge, but
nothing fixes how: H-4's object layout has no node-reference field kind
and JS-D1's slot encoding has only two tags. The harness takes the size
of that reference as a named constant
(`model::NODE_REF_BYTES_DEFAULT`, defaulting to 8 bytes for a slot plus
an epoch) and sweeps 4, 8 and 12 beside it, with the alternatives written
out at the definition. The result is reported against all three, so the
owner's answer substitutes without redoing the work.

## Where the trees come from

| Parameter | Value | Source |
|---|---|---|
| Elements per page (mobile): p10, p25, p50, p75, p90 | **180, 342, 594, 1 010, 1 716** | [Web Almanac **2024**, Markup, Figure 2.11](https://almanac.httparchive.org/en/2024/markup) ("the median number of elements has dropped from 653 in 2022 to 594 in 2024") |
| Larger trees, so the trace leaves the caches | 50 000 and 500 000 | Not published; chosen to cross the cache hierarchy |
| Maximum DOM depth | **18** representative, **40** tail | [State of Web Vitals, November 2026 edition](https://www.corewebvitals.io/state-of-cwv/explorer/metrics.dom.max_depth), 210 341 sites with maximum DOM depth: 28.1% under 15 levels, **36.6% at 15–20**, 30.5% at 21–31, 4.7% at 32+. The median site is in the 15–20 bucket |

Two cautions on those figures, both of which matter for reading the
numbers below.

- **The edition is load-bearing.** The Almanac's median elements per page
  is 587 (2020), 616 (2021), 653 (2022) and 594 (2024); the trees here
  are sized from the **2024** edition's mobile figures and from nothing
  else. The 2024 dataset also widened to include secondary pages, not
  only home pages, which the chapter states explicitly.
- **Elements are not nodes.** The Almanac counts *elements*; the per-node
  metric in FP-11, DM-30 and JS-D4 counts *nodes*, which include text
  nodes. No published figure for the element-to-node ratio was found, so
  the trees here have one node per published element and the per-node
  figures should be read as **per element**. This understates both
  designs' absolute totals by the same factor and so does not move the
  ratio the trigger asks about, but it does mean these numbers are not
  directly comparable with FP-11's 72 B† per node.

Depth is a secondary parameter: it changes the trace's locality, not the
bytes. The generator's achieved maximum and mean depth are recorded in
every row of `results/*/gc.tsv` rather than assumed — at a limit of 18 it
produces a maximum of 17 and a mean of about 4.2, which is the shape an
HTML parser's open-element stack gives.

## Method

`harness/` is one Rust binary with three modes: `gates`, `bytes` and
`gc`. Every mode runs the validity gates first and refuses to print a
measurement if one fails.

**The two layouts**, counted field by field from the specs rather than
estimated:

| | (a) | (c-min) | (c-id) |
|---|---|---|---|
| Node structure | 48 B of columns outside the cage: five links, tag, flags, element state, attribute-set reference, inline-style reference, rare index, generation (JS spec 6.2's own list, which sums to its stated 48 B† core) | 56 B cage object: H-4 header, five traced link slots padded to 24 B, three packed raw words | 64 B: the same plus a generation word |
| Mark bits | 1 bit per node slot, in a side bitmap | 1 bit per 8-byte granule | as c-min |
| Wrapper | lazily created, 16 B in the cage (8 B header + the node reference), for touched nodes only; its external traced slot is 4 B in an 8 B rare-data record | none — the cage object **is** the JS object | none |
| Stable node ID | the node slot never moves, so its ID is its address | none | 4 B table entry, updated on every move |
| Text | one arena, outside the cage | one copy, in the cage | one copy |

Two things are deliberately given to (c). It pays **no wrapper at all**,
because in (c) the node object is the JS object, so a touched node costs
it nothing extra — that is (c)'s real advantage and it is counted in
full. And it is reported in two variants, because the spec describes (c)
in one clause and the stable-ID indirection is an inference, not a quote:
`c-min` assumes nothing outside the cage ever needs a stable name for a
node, which is the cheapest reading; `c-id` adds the generation and the
ID table that B8's stable node IDs, the mutation log and the semantic
plane need once nodes can move. Both are reported so the comparison is
not rigged by an assumption.

**One collector, two layouts.** Both designs are traced by the same code
in the same two phases, with **one clock read per phase, never one per
object**. An earlier version timed (c) per object and (a) per phase; at
500 000 nodes that put a million `Instant::now()` calls inside (c)'s
measurement and inflated it. An earlier version also fixed up moved
references in a separate pass over every slot, which charged (c) a whole
extra slot scan that H-7's design does not pay — H-7 folds marking and
evacuation into one loop, so the fix-up costs one compare and store on
the slots that already had to be read. Both are corrected; the harness
now reads the clock twice per cycle and evacuates inside the trace,
through a forwarding header in the moved object's own type word, exactly
as H-7 and `hg-heap`'s own `visit` do.

### The validity gates

A fast wrong number is the failure this spike most has to avoid, so the
gates run before any number is kept and print `FAIL` rather than a
measurement. `results/*/gates.txt` records the line.

| Gate | What it checks | Why |
|---|---|---|
| **G1** | The column field list sums to the 48 B core the JS spec states | A mis-transcribed field list would shift every (a) total silently |
| **G2** | The arithmetic model agrees with what the harness really allocated, to within the mark bitmap's rounding to whole words | Catches a per-node figure derived from a layout the code does not build |
| **G3** | A cycle marks exactly the attached nodes and exactly the live cage objects, in all three variants, with zero stale slots | A trace that fell out of the tree would be fast and wrong |
| **G4** | An unreachable node does not survive a cycle | The node space must collect, not just mark |
| **G5** | After an evacuating cycle the next cycle marks the same graph, the evacuation moved something, and the cycle after it has nothing left to move | A collector that missed a fix-up loses a subtree and looks fast doing it |
| **G6** | A node detached from the tree whose wrapper a JS object still holds **survives** | This is JS-D4's strong-once-created rule, and the only gate that exercises the cage-to-node edge the design review left undecided |

**The gates earned their place.** On their first run they failed four
ways, each a real bug in this harness:

```
FAIL	G2 model-vs-allocation: model 108202 B, allocated 108256 B, slack 14 B
FAIL	G3 (a) live cage objects: marked 1519, expected 1755
FAIL	G5 (c-min) after evacuation: 3000 nodes, then 4500 nodes
FAIL	G5 (c-id) after evacuation: 3000 nodes, then 4500 nodes
```

G2 was an off-by-one in the slot count the model was evaluated over. G3
counted only the wrappers reached from the cage roots, not those reached
through a node's external traced slot. G5's `4500 nodes` is the real one:
evacuation moved objects without updating the document handle, the plain
JS objects' slots or the stable-ID table, so the next cycle traced both
the old and the new copies and would have reported a 50% larger live set
as if it were correct.

**The gates are not vacuous**, which was checked rather than assumed. With
the forwarding fix-up deliberately removed from `visit`, every mode
refuses to print:

```
FAIL	G5 (c-min) stale slots: 1499 then 1499
FAIL	G5 (c-id) stale slots: 1499 then 1499
d4-nodes: 2 validity gate(s) failed; no measurement printed
```

## Bytes per node, across every copy

Machine-independent arithmetic, so it is recorded once:
`results/model/bytes.tsv`. Text is excluded, as FP-11 and DM-30 both
count it — both designs store text once, so it cancels. `touched` is the
fraction of nodes script has touched.

| touched | (a), node ref 8 B | (a), node ref 12 B | (c-min) | (c-id) |
|---|---|---|---|---|
| 0.00 | **48.125** | 48.125 | 56.875 | 69.000 |
| 0.01 | 48.367 | 48.449 | 56.875 | 69.000 |
| 0.05 | 49.337 | 49.744 | 56.875 | 69.000 |
| 0.10 | 50.550 | 51.362 | 56.875 | 69.000 |
| 0.33 | 56.128 | 58.809 | 56.875 | 69.000 |
| 1.00 | 72.375 | 80.500 | 56.875 | 69.000 |

Every term is printed separately in the file, because "across every
copy" is the point of the metric: (a)'s 48.125 at `touched = 0` is 48 B
of columns and one mark bit; its 72.375 at `touched = 1` adds an 8 B
rare-data record, a 16 B wrapper and that wrapper's two granule mark
bits. (c)'s mark bits cost **seven times** (a)'s per node — 0.875 B
against 0.125 B — because a granule bitmap over a 56 B object is seven
bits where a per-slot node bitmap is one.

**(a) is smaller than (c) on any page where script touches fewer than a
third of the nodes, and larger on pages where it touches more.** The
crossovers, from the same file:

| (a) against | node ref | equal at | (a) worse by 10% at |
|---|---|---|---|
| c-min | 4 or 8 B | touched = **36.1%** | touched = **59.5%** |
| c-min | 12 B | touched = 27.0% | touched = 44.6% |
| c-id | 4 or 8 B | touched = 86.1% | **never** |
| c-id | 12 B | touched = 64.5% | touched = 85.8% |

**The trigger does not fire unless more than 60% of a page's nodes are
touched by script** (against the cheapest reading of (c); against the
reading that keeps stable node IDs, it cannot fire at all). JS-D4's own
premise is the opposite — "most nodes of most pages are never touched by
script" — and the DOM spec cites Chrome's black-box costs in support of
making untouched nodes pay nothing. But that premise is **Q-DM-6, an
open question with no published figure**, so this spike does not claim it
is settled; it reports the condition and leaves the fraction to the
corpus at DM-M1.

**The open decision costs nothing between 4 and 8 bytes.** A 4-byte node
reference and an 8-byte one give identical totals, because the wrapper's
body pads to 8 either way: the wrapper is 16 B in both cases. Only a
12-byte answer moves the result, pulling the trigger's crossover from
59.5% down to 44.6%. So decision (1) can be taken on soundness grounds —
whether the epoch is needed for stale-handle detection — without a bytes
argument, as long as the answer is not 12.

## The node space's share of GC time

Two rigs, two ISAs, 15 repeats per cell, one exclusive window each. The
tables below are the **median** of the 15, at the largest tree (500 000
nodes plus 50 000 orphans), depth limit 18. Full data in
`results/*/gc.tsv`, every cell with its achieved max and mean depth.

**AArch64 — mini (Apple M4, 10 cores, 4 performance).** Evidence:
`rig check OK: mini kind=exclusive session=js-m0 | load 0.15/cpu (1.49 / 10 cpus) | bookings: b-4bb175 exclusive cpu by js-m0 until 1:18am | game lock: free q0 | 2026-11-09T00:48:55-07:00`,
with a closing line at 00:49:04. Booking b-4bb175, released after the run.

**x86-64 — mbp (Intel Core i7-8559U, 4 cores / 8 threads).** Evidence:
`rig check OK: mbp kind=exclusive session=js-m0 | load 0.16/cpu (1.31 / 8 cpus) | bookings: b-6e6624 exclusive cpu by js-m0 until 1:05am | game lock: free q0 | 2026-11-09T00:50:49-07:00`.
Booking b-6e6624, released after the run. An earlier identical run fell
in the four-minute gap between the previous session's booking ending and
mine starting; `rig check` passed it (the rig was free and quiet) but the
line read `bookings: none`, so the run was repeated inside the booked
window and only that one is kept. The two agreed within 0.5%, which is
also the best evidence that the window was not doing the work of hiding
noise.

hueyj was booked for a third, Linux leg and then **released**: mbp gave
the x86-64 point, and hueyj was contended by this session's JS-D16 and
JS-D17 agents the same night. So there is no Linux data point, and
whether the result holds on Linux is untested rather than confirmed.

### Nanoseconds of node-tracing time per live node

| touched | evacuating | (a) | (c-min) | (c-id) | (a) against c-min |
|---|---|---|---|---|---|
| **mini, AArch64** | | | | | |
| 0.05 | no | **9.36** | 13.75 | 14.82 | −32% |
| 0.05 | 25% | **9.90** | 17.11 | 18.43 | −42% |
| 0.33 | no | **11.33** | 15.42 | 16.45 | −27% |
| 0.33 | 25% | **11.90** | 17.08 | 18.51 | −30% |
| **mbp, x86-64** | | | | | |
| 0.05 | no | **36.52** | 68.11 | 74.55 | −46% |
| 0.05 | 25% | **36.91** | 93.21 | 103.31 | −60% |
| 0.33 | no | **44.54** | 74.14 | 81.66 | −40% |
| 0.33 | 25% | **46.76** | 95.53 | 105.18 | −51% |

**(a) spends less time per node than (c) in every cell, on both ISAs, by
27% to 60%**, and the gap widens exactly where the design predicts: when
the cycle evacuates. The whole cycle, at touched 0.05 with 25%
evacuation:

| | (a) | (c-min) | (c-id) |
|---|---|---|---|
| mini: nodes / other / sweep | 4 950 / 856 / 204 µs | 8 557 / 955 / 1 327 µs | 9 214 / 958 / 1 479 µs |
| mini: whole cycle | **6 010 µs** | 10 839 µs | 11 651 µs |
| mbp: nodes / other / sweep | 18 453 / 2 793 / 758 µs | 46 606 / 4 890 / 4 152 µs | 51 657 / 4 957 / 4 653 µs |
| mbp: whole cycle | **22 005 µs** | 55 648 µs | 61 267 µs |
| objects moved | 6 220 | 137 499 | 137 499 |

The mechanism is the `moved` row and nothing subtler. (a)'s node space is
non-moving, so a cycle that evacuates a quarter of its candidates moves
6 220 wrappers; (c)'s nodes are cage objects, so the same policy moves
137 499 of them, and every link of every live node has to be read and,
where its target moved, written. (a)'s sweep is also 6.5× cheaper on mini
and 5.5× on mbp, because walking one bit per node slot is less work than
walking one bit per 8-byte granule of a much larger cage.

### The trigger's "share of GC time" is ambiguous, and the two readings disagree

This needs flagging rather than smoothing over. JS-D4's trigger names
*"the node space's share of GC time"*. That phrase has two readings, and
on these numbers they point opposite ways.

| Reading | (a) | (c-min) | (c-id) | Fires the >10% trigger? |
|---|---|---|---|---|
| **Absolute**: node-tracing time per live node | 9.36–46.76 ns | 13.75–95.53 ns | 14.82–105.18 ns | **No** — (a) is 27–60% *better* |
| **Literal share**: node time ÷ whole cycle | 0.825–0.867 | 0.766–0.838 | 0.749–0.843 | **Yes, in places** — (a)'s share is 5.5% higher at touched 0.05 and **15.8% higher** at touched 0.33 against c-id on mini |

(a)'s *share* is higher precisely because (a) is faster at everything
else: its sweep and its non-node tracing are so much cheaper that node
work is a larger slice of a much smaller cycle. Read as a share, a design
could improve its score by making the rest of the collector slower, which
is not a property anyone wants to select for. The absolute reading is
plainly the one that matches the decision JS-D4 is making, and on that
reading (a) wins everywhere.

But the trigger as written says "share", and on the literal reading it
fires at touched = 0.33 against c-id. **This is a spec wording problem,
not a result**, and it is the owner's to resolve: Section 9.4's JS-D4 row
should say which quantity it means. The recommendation is the absolute
one — node-space GC time per node, or per cycle at equal node count —
with the share reported alongside as a diagnostic.

## What this leaves for the owner

**Nothing here reverses JS-D4 (a).** On bytes, (a) is the smaller design
on any page where script touches fewer than 36% of the nodes, and the
>10% trigger needs more than 59.5% touched against the cheapest reading
of (c), or is unreachable against the reading that keeps stable node IDs.
On GC time, read as absolute node-tracing cost, (a) is 27–60% cheaper per
node than (c) on both ISAs, and the whole cycle is 1.8–2.8× cheaper,
because a non-moving node space never pays to move a node or to rewrite a
link.

**Three things are open, and two of them are the owner's.**

1. **The "share of GC time" wording** (above). The two readings of the
   trigger disagree, and the literal one fires where the absolute one
   does not. Section 9.4's JS-D4 row needs one sentence saying which.

2. **The touched fraction is unmeasured and it is what the byte
   comparison turns on.** Q-DM-6 already records that no published figure
   exists. Everything above is a function of it. JS-D4 (a)'s own premise
   is that the fraction is small, and if it is below about a third (a) is
   simply the smaller design; the figure itself belongs to the corpus at
   DM-M1. Until then the byte half of this trigger is conditional, and
   this note does not claim otherwise.

3. **The corpus limb of the trigger is JS-M1's**, not settled here. There
   is no corpus at JS-M0.

**What is not open.** The decision this spike was told to parameterise —
how many bytes a cage object spends naming a node — **does not change the
verdict** as long as the answer is 4 or 8 bytes: both give a 16-byte
wrapper and identical totals, because the wrapper's body pads to 8 either
way. Only a 12-byte answer moves anything, pulling the trigger's
crossover from 59.5% down to 44.6% touched. So that decision can be taken
on soundness grounds alone — whether the epoch is needed for
stale-handle detection — and this spike does not constrain it.

**What this spike did not measure, and why.**

- **Izcal's own collector.** `hg-heap` has no node space; building it is
  Phase 2 item 7 of the JS-M0 plan and it is blocked on four design
  decisions the JS-M0 design review did not fix (how a cage object names
  a node and whether that edge is strong, how node-space roots reach the
  collector, and the direction of the tracing interface). Both layouts
  here are modelled under one collector written to the same rules, so the
  **ratios** are the measurement and the absolute times are not Izcal's.
- **Nodes, as opposed to elements.** The Almanac counts elements; no
  published element-to-node ratio was found, so these are per-element
  figures and are not directly comparable with FP-11's 72 B† per node.
- **Text.** Both designs store it once, so it cancels; it is excluded, as
  FP-11 and DM-30 both exclude it.
- **Linux.** The third leg was released rather than run, because hueyj
  was contended.
- **Concurrent marking and the SATB barrier.** Neither exists at JS-M0;
  the collector here is stop-the-world, as `hg-heap`'s is. JS-D4's link
  writes are supposed to carry an SATB barrier, which will cost (a)
  something this spike cannot price.
- **Rare data beyond the wrapper.** Listeners and observers are charged to
  neither design, because they cost both the same.
