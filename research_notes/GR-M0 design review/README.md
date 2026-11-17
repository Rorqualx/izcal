# GR-M0 design review: the agenda

Opened 13 November 2026, for the graphics track's M0 milestone
(`reports/Izcal graphics and rendering.md` §9.2). **Opening the review
with its agenda fixed is M0's *start*; completing it is M0's exit.**

GR-M0 is the one M0 whose "Needed by" is another track's milestone —
**LY-M0** — and one of its own items is co-signed at *that* review,
which needs a person who does not exist. §5 states that plainly rather
than leaving it to be discovered, and the rest of this agenda is
written so that the block is **one item and not the milestone**.

The M0 row's mandate, in its own words:

> HAL design review across three back ends (resource type states,
> render graph, allocator, fences, epochs); display-list,
> compositor-frame and canvas-command `.hgidl` formats;
> `hg-paint-order` contract co-signed at LY-M0's review; the 160 B†
> paint share co-signed; `hg-raster` CPU path for fills, AA, blending
> and images passing `hg-reftest-raster`'s cases with exact hashes on
> x86-64 and AArch64; `hg-raster-ref`; spikes GR-S1, GR-S2, GR-S6
> (first pass), GR-S9 done, GR-S3 and GR-S4 started; the
> canonical-image definition, E16 and the compositor's integer
> composition specified; the trusted-prompt layer (GR-D28) designed
> with the security owner; the process model (GR-D2) decided from
> GR-S2

---

## 1. What is already built, and therefore reviewed rather than designed

These landed before the review opened, so the review's job for each is
to find what is wrong with it, not to decide it from nothing.

| Built | Section | What the review checks |
|---|---|---|
| `hg-display-list` — the format, generated from one `.hgidl` | §6.3, GR-D6 | that every count is a limit, and §3.1's transport finding |
| `check.rs` — the six things a wire format cannot say | §6.3 | that the list is complete: premultiplication, extents, a source rectangle inside its image, arc radii, group balance, nesting depth |
| `hg-raster` — coverage, blending, images, integer only | §6.4, GR-D3, G61 | that no `f32` or `f64` entered, and §3.2's accumulator finding |
| `hg-raster-ref` — the independent reference | §6.26, GR-6 | §3.3: it is **exact**, not float64 as the crate table says |
| `hg-gpu-hal` — the contract, before any back end | §6.6 | §2 — the central item |

Each is held by tests that can fail: 19 in `hg-raster`, 9 in
`hg-raster-ref`, 17 in `hg-display-list`, 11 in `hg-gpu-hal`, and the
`raster-ref` CI job runs all of them on x86-64 and AArch64.

---

## 2. The central item: can three back ends keep this contract?

§6.6 gives the HAL five properties and three native APIs:

> devices advertise capabilities, never vendors; all work is
> asynchronous and returns fences on a per-device timeline; there is
> no blocking map; validation errors are sticky; device loss resolves
> exactly once with an epoch

Each is now a type with a test behind it. The review's job is the one
thing a test cannot do: **ask whether Metal, Direct3D 12 and Vulkan
can each keep it**, because a contract that one of them cannot keep
becomes a per-back-end exception, and the first exception is the end
of "one rendering model on every OS" (P2).

**What the review must settle, property by property.**

1. **Capabilities and never vendors.** `Capabilities` has no field
   naming a manufacturer and no constructor that could carry one
   (`gh05` checks the rendered form for seven vendor names). The
   question for the review is the *completeness* of the list: every
   driver workaround the back ends will need must be expressible as a
   capability reported absent. A workaround that is not is a vendor
   check with another name.
2. **One timeline per device.** Metal and Vulkan have timeline
   semaphores; Direct3D 12 fences are monotonic values already. The
   review checks that the three agree on what "completed" means at the
   moment of a device loss, which is where a monotonic value and a
   rebuilt device meet.
3. **No blocking map.** Every read-back returns a `Fence`. The review
   checks that no back end's native API forces a wait that the HAL
   would have to hide — and in particular what Metal's
   `MTLBuffer.contents` and D3D12's `Map` do under a device that has
   stopped answering.
4. **Sticky validation.** The first error is kept, later ones counted,
   and only a rebuild clears it. The review checks that a back end's
   own debug layer can be *fed into* this rather than reported beside
   it.
5. **Device loss resolves once per epoch.** The review checks this
   against the adapter-wide reset §6.18 describes: on most WDDM
   drivers a TDR resets the whole adapter and **every process** loses
   its device, so "resolves once" must be once per process per epoch,
   not once globally.

**And the one piece of §6.6's mandate that was not built: the
allocator** — now its interface, and only that (**§3.7**, 15 November
2026). §6.2 makes placement the thing that decides fragmentation,
residency and per-site accounting, and every allocation carries
FP 6.6's ledger tag. `crates/hg-gpu-alloc` is the ledger tag, the
budget classes and residency; **placement is still GR-M1's** and no
part of it is written, because GR-S1's measurements are what should
decide it. Writing a placeholder that allocated badly would have met
the letter of the start definition and told the review nothing, so
there is none.

---

## 3. Items the review must decide, in their own right

### 3.1 The display list does not fit in a message — **decided (a), 14 November 2026**

Measured, not estimated (`hg-display-list`'s
`a_maximal_chunk_is_far_larger_than_a_message` and
`the_measured_bounds`):

| Type | `MAX_LEN` |
|---|---|
| `Segment` | 28 B |
| `Path` (65,536 segments) | 1,835,024 B |
| `Item` | 1,835,032 B |
| `Chunk` (4,096 items) | **7,516,291,092 B** ≈ 7.0 GiB |
| Appendix C's message cap | 65,536 B |

§6.3's two bounds are deliberate and the generated reader enforces
them. But a chunk at those bounds is about **114,000×** the generated
message cap, so a chunk cannot travel as one IPC message, and GR-D6's
"the display list is a generated, typed IPC format" needs a stated
transport. FP 6.4's "display list copied then validated" points at a
shared buffer; §6.3 does not say so in the format's own terms.

**Decided: (a)** — the owner, 14 November 2026. The
copied-then-validated shared-buffer transport is written into the
format, with the chunk as the unit of validation; the message carries a
generation-tagged buffer handle and the chunk's length, never the chunk;
validation runs on the copy, not in place, which removes the
time-of-check-to-time-of-use window a shared buffer would otherwise
open; and a chunk that fails validation is dropped whole, because the
bounds are properties of the chunk. (b) was refused for moving the
complexity into path reassembly across chunks — new parsing on the
untrusted-renderer boundary, in the same class as the WebRender
"incorrect boundary conditions" escapes §6.3 cites; (c) for giving the
validator two paths to be correct on instead of one. Recorded in the
graphics spec §6.3 and in GR-D6's row.

### 3.2 The accumulator's unit — **decided, 14 November 2026: the finer unit**

§6.4 specifies "exact area coverage by signed-area accumulation in
**1/256 px² units**". `hg-raster` accumulates in `SUB`-squared units
(`FULL = 2 · SUB · SUB`) because an exactly-representable trapezoid
area needs the finer denominator; at 1/256 px² the accumulation itself
rounds, and "exact area coverage" and "1/256 px² units" cannot both
hold. The code took the exact route and the difference is recorded
here rather than resolved silently.

**Decided: correct §6.4's unit to the finer one, keeping "exact"** — the
owner, 14 November 2026. §6.4 now reads "exact area coverage by
signed-area accumulation in **1/131,072 px² units**", which is
`2 · SUB²` at `SUB = 256`: the denominator that makes a trapezoid's area
exactly representable, with the factor of two so that its half needs no
division. The code is unchanged; the specification moved to the code,
which is the right direction here because the code's unit is the one
that keeps GR-D3's bit-identical claim free of a rounding inside the
accumulator.

**The provenance of this answer, recorded because the id was polluted.**
This decision was put to the owner with §3.1, §3.3 and four others. A
first row against it read "defer this one", which the coordinator
identified as its own browser-test string re-sent by a stale tab and
declined to relay as an answer — a test fixture is not a decision, and
refusing to pass it on was correct. That row was purged, and the answer
above arrived afterwards in a batch with unrelated sessions' answers.
The distinction is worth keeping in the record: for a few hours this
item was *unanswered*, which is not the same as *deferred*, and
treating the fixture as a deferral would have left the spec and the code
disagreeing with a note saying the owner had chosen to let them.

### 3.3 `hg-raster-ref` is exact, not float64 — **confirmed, 14 November 2026**

§6.26's crate table says "an independent **float64** analytic
reference rasterizer". It is built instead on exact `i128` rational
arithmetic (Sutherland–Hodgman clipping and a doubled shoelace area),
which is strictly stronger: a float64 reference has rounding of its
own, so it cannot say whether a one-level difference is the
rasterizer's or its own.

**The measurement GR-6 is phrased against**, on the first accuracy
cases: **18 of 720 pixels differ, worst difference 1 level**, and
axis-aligned shapes agree exactly. GR-6 asks for ≤ 1/255 per channel
on 99.9% of edge pixels and ≤ 2/255 on all; every differing pixel here
is at 1/255, so both halves hold — and they hold against an oracle
with no error of its own, which is the point of the deviation.

**Decided: confirm the deviation** — the owner, 14 November 2026.
§6.26's crate table is amended to read "an independent **exact**
analytic reference rasterizer ... on `i128` rational arithmetic", with
the reason recorded in the row: a float64 reference rounds too, so it
cannot say whether a one-level difference is the rasterizer's or its
own. The 18-of-720 measurement above stands as GR-6's evidence, and it
now stands against an oracle with no error of its own.

### 3.4 The two formats not yet written

The M0 row names three `.hgidl` formats. One exists
(`display_list.hgidl`). **`compositor_frame.hgidl`** and
**`canvas_command.hgidl`** do not, and both have counterparts: the
compositor frame is what the shared compositor validates (G63 forbids
it a display-list reader at all), and the canvas command log is read
by the in-renderer CPU replay GR-D12 requires. The review fixes both
before GR-M1.

**The compositor frame is written (15 November 2026), and writing it
settled a question the crate table had answered wrongly.**

§6.24's L2 row put all three formats in `hg-display-list`. That cannot
hold for the compositor frame, for a mechanical reason rather than an
aesthetic one: **`hg-graph-lint` reads `cargo metadata`**, so the
finest thing it can name in a rule is a *package*. A compositor that
reads compositor frames out of `hg-display-list` necessarily carries
that package — and so the display-list reader — in its own binary
closure, and G63 becomes unenforceable by the tool that enforces every
other rule of its kind. "The linker probably dropped it" is not a
security argument.

So `crates/hg-compositor-frame` is its own package, and **G63 is now an
ordinary `forbid` row** in `tools/graph-rules.toml`. The crate-table
amendment this implies is **proposed, not decided**: it changes a gate
file and a spec table, so it is in §9 for the owner.

Three things about it are worth recording, because each is a mechanism
rather than an intention:

- **G63's rule is tested before the crate it governs exists.** Every
  crate the rule names but `hg-display-list` and `hg-raster` is
  unbuilt, so the row would otherwise be one nothing has ever seen
  fire — the same dead-code problem §3.5 raises about the cycle
  variant, and the reason that problem is worth taking seriously. Two
  fixture tests fire it against the real `graph-rules.toml`: a
  synthetic `hg-compositor` that links `hg-display-list` is refused,
  and the twin that links `hg-compositor-frame` instead is clean. The
  second case is what stops the rule being satisfied by a compositor
  that cannot read its own input.
- **GR-18's size artifact is a constant with a test, not a figure in a
  report.** GR-18 asks for the IDL's size — types, variants, maximum
  nesting — to be published and not to grow without a security review.
  `src/size.rs` holds it (**19** types, **26** variants, **92** fields,
  **10** bounded vectors, maximum nesting **4**) and the test beside it
  recomputes every figure from the IDL, so adding a type or a variant
  is a **build failure** whose fix is a diff a review can be asked
  for. Two of those five numbers were wrong when written by hand and
  the test corrected them, which is the argument for the mechanism in
  one line. The deepest chain is `Frame` → `Animation` → `Keyframe` →
  `KeyframeValue` → `Transform`, not the tile grid as guessed; the tile
  grid is the only place a vector holds a vector, and so the only place
  one declared count multiplies another (256 layers × 4,096 tiles).
- **G63's "0 page-controlled filters, paths or ICC data" is checked
  against the format.** A second test asserts no declared type or
  field name contains `path`, `segment`, `fillrule`, `gradient`,
  `filter`, `mask`, `icc`, `glyph` or `shader`, with a planted name to
  prove the check can fail. The clause stops being a sentence about the
  format and becomes a property of it.
- **The four generated crates' `src/sample.rs` are now checked to be
  one file.** The generator emits `impl crate::sample::Sample` under
  `cfg(test)`, so every crate in `GENERATED_CRATES` must supply that
  generic module; the three existing copies were byte-identical and
  nothing checked it, so adding a fourth would have made an unguarded
  duplication wider. `oc20` in `hg-ipc-codegen` now compares all four
  and fails on a planted edit. It is the cheap half of §9.2's
  question, settled in the direction that is obviously right for a
  generic support module whatever is decided for the IDL.

A fourth mechanism was not planned and is worth recording, because it
came from a lint rather than from the design. `StepPosition`'s four
variants — `jump-start`, `jump-end`, `jump-none`, `jump-both` — share a
prefix, which clippy's `enum_variant_names` refuses, and the generator
emits no lint escapes, so generated output has to be clean under the
crate's own block. Renaming the variants would have broken their
correspondence with Easing Functions 1. The four values **are two
booleans** (`jump-none` is neither, `jump-both` is both), so the enum
became `jump_at_start` and `jump_at_end`, which is the exact semantics
and removes a type and four variants from the published size. A lint
about naming found a redundant encoding in a format whose stated goal
is to be small.

The format carries what §3.1's diagram says it carries and nothing
else: tile grids of textures granted once (G71), one CPU-observed
fence per (content process, compositor) pair (G72), a property-tree
subset of transforms, rectangular and rounded-rectangle clips in fixed
point, opacity, scroll nodes, surface embeds, the animation spec's
bounded record for transform and opacity, and damage. The property
trees are **flat vectors with parent indices**, because the IDL forbids
a recursive type and because B6 asks for property trees as data — and
a parent must be an **earlier** index than its child, which makes a
cycle unexpressible rather than merely invalid. That is why there is no
cycle variant in its error enum, and it is the same move §3.5 has to
decide for the render graph.

**One measurement the format forced, and it bears on GR-18 rather than
only on transport.** A maximal frame encodes to **29,591,432 B**
(28.2 MiB) against the 64 KiB `MAX_MESSAGE_BYTES` cap — 451 times over.
`hg-display-list` has the same problem and records it the same way
(its `a_maximal_chunk_is_far_larger_than_a_message`), but the numbers
are worth putting side by side, because GR-18 asks *this* format to be
the small one:

| | Worst case | Over the 64 KiB message cap |
|---|---|---|
| `Frame` (compositor) | 29,591,432 B | 451× |
| `Chunk` (display list) | 7,516,291,092 B | 114,688× |

So GR-18 holds **directionally** — the compositor's input is 254 times
smaller in the worst case — and does not hold **absolutely**: 28 MiB is
not small, and it is bytes an untrusted content GPU process can make
the compositor allocate.

*Corrected, same day:* this passage first said the review "owes a third
option alongside the display list's two", naming shrink-to-a-message
and the shared-buffer transport as if both were still open. **They are
not: §3.1 records the owner deciding (a) on 14 November 2026** — the
copied-then-validated shared-buffer transport, written into the format,
with the chunk as the unit of validation and the message carrying a
generation-tagged buffer handle and length. Citing a decided item as
open is the mirror image of §3.6's mistake below, where a *met* item
was carried as open; both come from reading one's own notes for what
they say rather than for what the record says, and both are an argument
for the habit §3.4 settles on — put the check where the build runs.

So the transport question is **answered** for the compositor frame
too, by extension rather than by a new decision: the frame has the same
shape of problem at 451× rather than 114,688×, and §3.1's reasoning for
refusing (b) and (c) does not change with the multiple. What this
review adds is only that the extension should be **stated** in §6.3's
terms rather than assumed, since §3.1's record names the display list.

What *is* genuinely new is smaller and is about GR-18 rather than about
transport: **GR-18's published size should record worst-case bytes next
to the type and variant counts**, because a format can be small in
types and enormous in bytes, and it is the bytes an untrusted peer
makes the receiver allocate. The counts are what GR-18 names today, and
the counts are the thing this format is already good at.

**And one gap that was already there, which writing the second format
made visible.** §3.6's threat table promises "generated readers with
**Kani harnesses** (VER-1)" against a hostile display list, command
stream or compositor frame. Neither format has one. Measured:

| IDL | `interface` declarations | Kani attributes in the generated reader |
|---|---|---|
| `os.hgidl` | 2 | 463 |
| `display_list.hgidl` | 0 | 0 |
| `compositor_frame.hgidl` | 0 | 0 |

The cause is structural rather than an omission: `hg-ipc-codegen`
emits validator harnesses **per interface message**, and a format
written as a type library declares no interface. Neither
`hg-display-list` nor `hg-compositor-frame` is in `KANI_PACKAGES`,
which is consistent — there is nothing for Kani to prove — but it
means the row in §3.6 describes something that does not exist for
either format, and has not since the display list landed.

This is the **same** finding as the transport measurement above, which
is why it is worth stating once: both follow from the formats
declaring no interface. The message cap that makes a 28 MiB frame a
problem is the *interface's* cap, and the harnesses that are missing
are the *interface's* harnesses. So the options converge: if the two
formats gain the interfaces that carry them, the transport decision is
forced into the open and the harnesses arrive with it; if they stay
type libraries, VER-1's row needs amending to say where the proof
actually lives (today: the property and mutation tests in
`src/tests.rs`, 4,000 seeded cases per leaf type with every mutation
required to be rejected or to re-encode exactly). Either way §3.6's
table should stop claiming the harnesses. That is a review decision,
not a code change, and it is added to §8.

The validator holds **seventeen** things the wire format cannot say
against the **ten** counts it can. Two are worth naming because they
are decisions rather than bounds: radii wider than the rectangle they
round are **refused rather than scaled**, because CSS scales them and
scaling is a decision, and every decision about page pixels belongs to
the content GPU process if the composed screen is to equal the
canonical image (6.4); and an embed's clip is **never optional**,
because LY-15 holds against a compromised child only through the clip
the parent drew.

**Two of the seventeen were found by review rather than by design, and
both were the same mistake in different clothes: treating an index as
if it identified a node.**

- `mutable_node_count` keyed each compositor-mutable node by its
  index, and the three trees are separate vectors — so transform 0,
  effect 0 and scroll 0 counted as **one** node. A content GPU process
  could have held far more than the 16† the cap allows while appearing
  not to. Keyed by **(tree, index)** the same frame counts 3 where it
  counted 2.
- `check_animation` accepted a target that named an element of
  *either* the transform or the effect vector, whatever `property`
  said, and never checked that a keyframe's value was of the
  animation's property. So an `Opacity` animation could drive a
  transform node, or carry `Transform` keyframes, and the sampler
  would have received the mismatch from an untrusted sender. Both are
  now refused (`TargetWrongKind`, `KeyframeWrongProperty`).

The lesson generalises past this format: **a flat property tree
replaces pointers with indices, and an index is only a node relative to
the vector it indexes.** Every check that compares or deduplicates
indices across trees has to carry the tree with it. The parent-earlier
rule got this right by construction, because a parent is always in its
own tree; the two bugs were in exactly the places where a value from
one tree met a value from another. That is the shape to look for when
§3.5 decides the render graph's edges, which will have the same
representation and so the same trap.

Worth recording about the mechanism too: adding the two error variants
made GR-18's size test fail, which is what it is for — the format grew,
and growing it required editing the published artifact rather than
merely passing CI.

**`canvas_command.hgidl` is not written, and the reason is a real
dependency rather than a shortage of time.** The IDL language has **no
import**: each `.hgidl` is a standalone protocol, and `pairs()` globs
`idl/*.hgidl` per crate. The canvas log needs the bounded path
vocabulary — `Point`, `Segment`, `Path`, `FillRule`, `Colour`,
`BlendMode`, `Rect`, `ImageRef`, `Sampling` — which is about **140 of
`display_list.hgidl`'s 240 lines**. Writing it standalone would copy
60% of another format into the same crate, with nothing making the two
copies agree. That is a drift source of exactly the kind this review
is meant to remove, so the choice was the owner's and it went to §9.
**They took (a) on 16 November 2026: add an `import` to the IDL.** The
generator has it; the format itself is the next step. The compositor
frame did not raise the question because its vocabulary is **disjoint
from the display list's by design** — the duplication there is the
smallness G63 and GR-18 are asking for.

### 3.5 The render graph's remaining half

`RenderGraph::compile` derived barriers from a *sequence* of passes,
and `ValidationError::Cycle` was a variant nothing could produce
because a cycle is not expressible in a sequence (`gh11` said so rather
than leaving a dead variant). The review had to decide whether the
graph gains explicit edges — and if it does, whether the cycle check
arrives with them.

**Decided, 15 November 2026: the graph gains explicit edges, the edges
point backwards, and `Cycle` was deleted rather than kept.**

A pass now declares the passes it runs `after`, by index, and **every
such index must be smaller than the pass's own**. The reasoning is the
same one §3.4's property trees arrived at from the other direction, and
it is worth stating as one principle because this review now relies on
it twice: *where an ordering can be encoded so that the bad case cannot
be written down, encode it that way, and then do not carry a check for
the case you made impossible.*

Three things follow, and the second is why edges were worth adding at
all:

- **A cycle is still unexpressible**, because any cycle needs an edge
  pointing forwards or at itself, and those are refused one at a time
  without looking at the graph. So gaining edges did not bring the
  cycle back, and `ValidationError::Cycle` went instead of being kept
  reserved. The answer to "when does the dead variant become live" was
  *never*, and keeping it would have left a refusal nobody has seen
  work — which is exactly the objection §3.4 raised about G63's rule
  before it was given fixture tests.
- **Independence became expressible, and with it a mistake the
  sequence could not have.** A sequence orders every pass against
  every other, so it can never tell you that you ordered two passes
  you did not mean to order. With edges, two passes can be genuinely
  unordered — and two unordered passes touching one resource, at least
  one of them writing, is a result that depends on how a driver
  schedules: a different image on a different machine, which is the
  thing GR-D3 exists to prevent. That is the new
  `ValidationError::UnorderedConflict`, and it is the check that
  justifies the change. Two passes that only *read* are no conflict,
  and two passes touching different resources need no edge at all.
- **The barrier walk is unchanged, and the edges are what make it
  correct rather than plausible.** Because edges point backwards,
  submission order is always a topological order; because every
  conflicting pair must be ordered by an edge *path*, submission order
  agrees with dependency order wherever the agreement matters. Before,
  walking the sequence was right by assumption. `gh13` tests the
  transitive case and its twin with the middle edge removed.

The reachability computation is one forward pass rather than a
traversal with a visited set and a cycle guard, for the same reason:
when pass *i* is reached, every pass it depends on already has its own
row complete, so the transitive closure is a union of finished rows.
The representation pays for itself twice — once in what cannot be
written down, once in what does not have to be computed.

**The existing contract tests had to gain edges, which is itself the
finding.** `gh07` and `gh08` shared resources between passes and relied
on the sequence to order them; under the new check they report
`UnorderedConflict` until the edges they always implied are written
down. Nothing about those passes changed except that their ordering is
now stated rather than inherited from the order they happened to be
added in.

### 3.6 The canonical image, E16 and integer composition

Named by the M0 row and not started. These are specification items
rather than code, and they are the precondition for GR-50 and GR-51's
hashes at GR-M1.

**That paragraph was wrong, and checking it was the work (15 November
2026).** The M0 row asks for these three "specified", and all three
already are:

| Item | Where | State |
|---|---|---|
| The canonical-image definition | §6.4, GR-D3's rule | Specified: one integer pass over the whole display list at the frame's recorded scroll offsets and animation samples, offsets snapped to whole device pixels |
| The compositor's integer composition | §6.4, same paragraph | Specified: tiles composited by `hg-raster`'s integer blending at integer layer origins, compositor opacity by the same integer multiply, so for translation and opacity the composed screen equals the canonical image |
| E16 | §6.5 | Specified bit for bit: scale 4,080, range, clamping, rounding — and C19 in §9.6 already records it as "Applied" |

So this item was being carried as open when it was met. A review that
tracks its own items by what someone remembers writing will do that,
which is an argument for §3.4's habit of putting the check where the
build runs.

**One real gap, and it was in the one place the specification sounded
most finished.** §6.5 said every E16 conversion "rounds half to even"
and §6.4 gives S8 an exact add-shift identity that rounds **half away
from zero**. Those are different rules, and nothing reconciled them.

Measured, exhaustively, rather than argued:

- Over all **65,026** products of two bytes, ÷255 has **zero** ties —
  `x/255 = k + 1/2` requires `x = 255k + 127.5`, which is not an
  integer. So the two rules never disagree for S8, and `div255` gets
  §6.5's rounding **for free**. The specification was not
  contradictory; the reason was simply unstated.
- Over the range E16 compositing uses, ÷4,080 has **32,767** ties,
  each at `4080k + 2040`. So the licence does not transfer, and E16
  needs the tie rule written down. GR-D3 asks for "one specified
  rounding at each step", and naming a rounding *mode* is not
  specifying a *step* when the hot per-pixel operation is where the
  mode gets decided — two implementors reading "half to even" and
  copying `div255`'s shape would have produced results one level apart
  on 32,767 inputs, which is exactly the class of difference GR-D3
  exists to forbid and GR-51 would have caught only after the fact.

§6.5 now carries the operation: `floor(x/4080) = (x · 134,744,073) >>
39` — exact over the whole range, no division — then one multiply for
the remainder and a three-way compare against 2,040 resolving a tie to
the **even** quotient. It is implemented as `hg-raster`'s
`blend::div4080` and verified **exhaustively over every product of
both signs**, in the spirit of `ra01`'s ÷255 test rather than at spot
values. The half-up variant was planted to confirm the test fails on
it, and it does — at a tie, which is the only place it could.

**The multiplier then had to be widened, and the review finding that
prompted it was half right — which is worth recording in both
directions.** The first `div4080` used `(m · 134,744,073) >> 39`,
whose *floor* is exact only to 139,111,678, while the exhaustive test
stops at E16's own 133,689,360 and so could not have seen it. The
review predicted that the overshoot would make answers wrong. Measured
over the whole `i32` range, it does not: the floor overshoots at
**3,803,748** values and at **every one** the true remainder is at
least **4,065** — far above half — so round-half-even rounded up to
exactly the overshot value. Every rounded answer was right; what was
false was the comment calling the floor exact.

The multiplier was widened anyway (`(m · 2,155,905,153) >> 43`, exact
for every `i32`), for a reason that is the general one: **a correct
answer that depends on an unstated coincidence is not a correct
design.** The agreement above holds only for this tie rule and this
domain, so it would stop holding the moment anyone reused the floor,
changed the rounding, or raised the bound — and `mul4080` can already
form products to about 1.07e9. Documenting a domain would have been
weaker than not having one.

Two things about how this was found, since both generalise:

- **An exhaustive test is exhaustive over a domain, and the domain is
  a choice.** `ra20` sweeps every value E16 can form and passes; the
  defect sat 1.4 M above its upper bound. "Exhaustive" reads like
  "complete" and is not, so the domain belongs in the test's name and
  in the claim — which is why the second test is called *for every
  `i32`, not only for E16's range*.
- **A multiply-shift's witnesses are a proof, where a sweep is not.**
  It can only fail where the true quotient changes, so `4080k - 1` and
  `4080k` for every `k` cover the whole `i32` range in about 1.05 M
  pairs. Checking the places a thing can break beats checking many
  places it cannot.

Two notes for the record, because both are the kind of thing that
would otherwise be found later by a hash mismatch:

- **E16 is specified and nowhere implemented.** `hg-raster` is S8
  only; before this change `E16`, `4080` and `4_080` appeared nowhere
  in it. `div4080` is the arithmetic the specification owed, not the
  encoding: the E16 surface, its per-group selection (§6.5) and GR-2's
  sibling-group suite are GR-M1's. What is closed here is that the
  specification no longer has a step whose rounding is undecided.
- **S8→E16 is ÷16, not ÷4,080.** Writing the round-trip test the first
  time I conflated the two and asserted `div4080(v·16·16) == v`, which
  is false for every `v` but 0. `div4080` divides out an *alpha*
  multiply; the encoding conversion is ÷16, which leaves no remainder,
  which is what makes S8 a subset of E16 rather than a conversion with
  its own error. Conflating them is how an encoding acquires a
  rounding it was specified not to have, so the corrected test asserts
  both separately (`ra21`).

### 3.7 The allocator's interface — **decided, 15 November 2026**

GR-M0's row puts the allocator inside the "HAL design review across
three back ends (resource type states, render graph, **allocator**,
fences, epochs)", and §2's note above records that this was the one
piece of §6.6's mandate with nothing built: "The review decides its
interface here; the implementation is GR-M1's."

It now is decided, as `crates/hg-gpu-alloc` — the interface and
nothing else. **There is no placement**: no free lists, no heaps, no
suballocation strategy, no fragmentation policy, because that is what
GR-S1's measurements exist to inform and guessing it now would be a
number nobody could defend. The crate is already in §6.24's table at
L5 beside `hg-gpu-hal`, so its `tools/tiers.toml` row is the table's
own placement and **not** an amendment of the kind §9.1 asks the
owner for.

Six decisions, each a type rather than a convention, because the
failure mode of every one of them is silence.

**(1) The ledger tag is not five fields the caller fills in.** FP 6.6
says every GPU allocation carries a tag of "(site, container,
partition, tab, kind)", and the obvious interface takes five
parameters. That interface is wrong, and the reason is that the two
GPU process kinds stand in a different relation to the site. A
content GPU process is one per renderer key — §6.2: keyed "exactly
like the renderer it serves" — so its site, container and partition
are properties of the **process**, fixed before it allocates
anything. Only the compositor GPU service holds every site's
composited pixels, and only for it does the site genuinely vary per
allocation.

So a five-parameter tag hands the content GPU process a parameter it
has no right to vary, and the way that goes wrong is silent: bytes
charged to a site that did not ask for them, in the one table
attribution reads (FP 6.14). The scope is therefore a type.
`KeyScope::tag` takes a kind and a tab and **has no parameter of type
`SiteId`**, so a content GPU process cannot write another site's tag;
`CompositorScope::tag` takes the site, because GR 6.22 charges its
tiles "to the site whose content they hold". A `Ledger` additionally
refuses a tag its own scope could not have made, which catches one
that arrived by some route the type did not cover — `ga01` plants
exactly that, for the site and for the container and partition too.

**(2) Over budget is a state, not an error.** FP 6.6 requires each GPU
process to react "to **budget changes**, not only to system memory
pressure", because D3D12 budgets "fluctuate dramatically when the
user switches away to another application". So the budget can shrink
**below what is already allocated** with nothing done wrong by
anyone, and the interface has to stay usable inside that.

The failure here is circular and worth writing down because it is not
obvious: refuse every allocation while over budget, and the process
cannot allocate the swapchain image it needs in order to present the
frame whose completion releases the tiles. The budget then never
recovers. `Class` is the way out — `Reserved` is admitted whatever the
budget says, `Working` while there is room, `Discretionary` refused
first and reclaimed first, which is FP 6.6's "the prepaint margin
shrinks first under pressure" stated once for every kind instead of
for tiles alone.

**The admission rule needs the reserved cap as a parameter**, which a
naive signature would omit. Without it a run of discretionary
allocations fills the budget to the last byte, and the swapchain that
must then be admitted puts the process over with no way back; with it,
discretionary work is held off the room the reserved classes are
guaranteed and the deadlock cannot start. `ga04` holds both halves,
including the degenerate case: with a cap of zero the two classes
agree, which is the interface this parameter exists to avoid.

**(3) The class is not a function of the kind.** A viewport tile and a
prepaint tile beyond it are the same `Kind` — FP 6.6 counts both
inside the same 2.5-viewport budget — and they are not the same class,
because the prepaint one is what shrinks first. One field cannot
answer both "which ledger line" and "what happens under pressure", so
a `Request` carries both and `default_class` is documented as a
default whose answer for a prepaint tile is wrong. This is the same
shape as every other finding in this review: a single number asked to
mean two things.

The reclamation order is stated once, here, rather than left to each
back end, because three back ends that drop different things under
pressure make one page behave differently on Metal and on Vulkan for
reasons no test would catch (`ga07`).

**(4) A volatile allocation's contents may be gone, and two of the
three back ends cannot promise otherwise.** This is the item §2 asks
for — a property checked against three back ends — and the three do
not mean the same thing by "volatile":

| Back end | What "volatile" is | Contents |
|---|---|---|
| **Metal** | `setPurgeableState(_:)` with `volatile`: the system "can reclaim the underlying storage at any time **without informing the app**" ([GA 4.10](../Izcal%20graphics%20and%20rendering/gpu_apis_platforms.md)) | May be gone, with no callback saying so |
| **Direct3D 12** | `Evict`/`MakeResident` with `SetResidencyPriority` ([GA 3.18](../Izcal%20graphics%20and%20rendering/gpu_apis_platforms.md)) | Paged out, not discarded |
| **Vulkan** | No purgeable state in the surveyed APIs: `VK_EXT_memory_budget` "reports per-heap budget and usage" ([GA 5.3](../Izcal%20graphics%20and%20rendering/gpu_apis_platforms.md)) and nothing more, so volatility is free-and-reallocate | Gone by construction |

Two of three cannot keep the pixels, so **the contract is that they do
not**, and D3D12's stronger guarantee is invisible at the interface.
That is the right way round: an interface written to D3D12's
behaviour would show stale or foreign pixels on the other two, on the
path that runs when a tab has been hidden for half a minute — the
least-tested path there is.

This is a **safety** property and not a correctness one. Sampling a
texture whose storage the system reclaimed reads whatever is there
now, which is the class `hg-display-list`'s bounds check already
exists to stop ("whatever the sampler then reads is memory the
renderer did not put there") and the second-largest Chrome graphics
CVE class (§9's uninitialized-use row). So `Allocation::can_sample`
is **residency *and* written-since**, and nothing about residency
alone makes it true. A freshly created allocation is in the same
position as a recovered one — new GPU memory is not zero on every back
end and the engine does not get to assume it is — so it too is a
destination until written (`ga08`).

**(5) A grant is a move.** G71 makes a shared texture "granted once
and never re-granted", so `Grant` is neither `Clone` nor `Copy`: a
grant that can be duplicated is a grant that can be re-granted, and
the compiler is a better place to say that than a comment. The
consequence that a naive ledger gets wrong is the total. FP 6.6: "a
holder that keeps a released texture is charged for it, so pinning
memory is charged and killed at budget" — so `Ledger::charged` is
live **plus** pinned bytes. A ledger summing live rows alone reports a
process under budget while its pinned textures keep the memory, which
is the under-report that makes pinning free (`ga12`).

Writing this turned up an ordering the spec does not mention: the
holder's give-back can arrive **before** the owner's release, and both
orders must end in the same place without the row becoming grantable
again. The first draft of `unpin` handled only the after case and
returned "no such allocation" for the before one. `ga13` is that
order.

**(6) Caps come from geometry and build constants, never from the
budget.** FP-D19 decided this already — "a device-dependent JS-visible
limit is a fingerprint of the device's memory" — and FP 6.6 applies it
to the glyph pools by name. What the allocator adds is that the
obvious implementation violates it: an allocator that sizes its pools
from the budget it just read makes how many glyphs a page can use
before re-raster starts, how large a canvas it can get and when an
atlas stops growing into a reading of the machine's VRAM, all
page-observable through timing. This spec already treats far less as a
fingerprint: "3.05 bits from one canvas text test, 5.73 bits from six
canvas and WebGL tests".

So no function in `caps` takes a `Budget`, and that is not prose
alone: `ga16` reads `caps.rs`, takes the parameter list of every `pub
const fn` in it, and fails if any mentions a budget. It reads
parameter lists rather than the whole file because the module's own
documentation says "budget" nine times, and a test its own
documentation defeats is a test that gets deleted. **Planted and
confirmed**: a cap function taking `budget_bytes` fails it with the
file, the line and the parameter named; removing the plant passes
again. The first plant was no proof at all — changing an existing
signature broke the build before the test could run — which is the
distinction between a test that catches a violation and a compiler
that catches an edit.

#### Two defects the review found in it, both the same shape

Caught by the automated review on the commit, and both are the family
this review has now hit four times — an index or an id taken for an
identity it does not carry on its own.

- **An id was charged more than once.** `Ledger::charge` did not
  reject an `AllocationId` already in the table, so a repeated id
  double-charged the site; and because `grant`, `release` and `unpin`
  all act on the first match, the duplicate row could never be
  released and stayed charged until the epoch reset. The under- and
  over-report are in the same table attribution reads.
  `LedgerError::AlreadyCharged` now refuses it (`ga20`).
- **An epoch that did not advance.** `reset_for` took any epoch,
  including the one it was already in, which defeats the whole (id,
  epoch) discipline: a `Grant` taken before such a reset would reach
  a *new* row that reused its index. It now requires the epoch to
  advance (`LedgerError::EpochNotNewer`), and writing the test for it
  exposed a third gap — `unpin` checked no epoch at all, where
  `charge`, `grant` and `release` all did (`ga21`).

The second is worth the note it gets: the epoch check was the thing
this interface cited G45 for, and it was absent from one of the four
functions that needed it. A rule applied in three places out of four
reads, in a review, exactly like a rule applied everywhere.

#### What this interface deliberately leaves open

- **Whether a back end can answer `survived`.** `make_resident` takes
  the back end's report of whether the contents came back, and a back
  end that cannot tell **must** pass `false`. Whether Metal can tell
  is a question for GR-S1; until it is measured the sound answer is
  the conservative one, and the Vulkan path's answer is `false`
  always.
- **The OS's usage and the ledger's total are two numbers.**
  `Budget::usage` is the OS's view, `Ledger::charged` is Izcal's, and
  the driver's own allocations are in one and not the other.
  Reconciling them is a measurement (GR-S1), so the interface keeps
  them apart rather than assuming a relation it has not measured.

#### A discrepancy found while doing this, not fixed here

§6.24's L5 row and §6.2 both name **`hg-gpu-graph`** for the render
graph — §6.2: "barriers computed by Izcal's render graph
(`hg-gpu-graph`)", and §6.13's WebGPU text sends barriers "through
`hg-gpu-graph`". No such package exists: §3.5's render graph was built
in `hg-gpu-hal/src/graph.rs`, and `tools/graph-rules.toml`'s G63
comment already names `hg-gpu-graph` among the crates "the compositor
links by design".

This is the same class as §9.1 — the crate table and the tree
disagreeing about a package boundary — but **not** the same stakes:
G63's enforceability turned on `hg-compositor-frame` being its own
package, and nothing known today turns on the render graph being one.
So it is recorded rather than resolved, with the note that resolving
it one way is a one-line table correction and the other way is a
package split, and that the question to ask first is whether any rule
needs to name the graph without the HAL.

### 3.8 The threat table and GR-11 claim harnesses the formats cannot have — **resolved by §3.1's decision, not by a new one; GR-11's metric amended 16 November 2026**

§3.4 recorded the gap: §6.19's threat row and GR-11 both promise
"generated readers with **Kani harnesses** (VER-1)" against a hostile
display list, command stream or compositor frame, and neither format
has one. Measured there:

| IDL | `interface` declarations | Kani attributes in the generated reader |
|---|---|---|
| `os.hgidl` | 2 | 463 |
| `display_list.hgidl` | 0 | 0 |
| `compositor_frame.hgidl` | 0 | 0 |

Surveyed across the whole tree now, which is new: of the six `.hgidl`
files in the repository, four declare interfaces (`os` 2, `process` 2,
`sample` 4, `policy` 2) and the two graphics formats declare none. And
**no message carrying a buffer handle and a length exists anywhere**,
in any IDL.

That last measurement is what resolves this, because it shows the
claim is not wrong so much as **attached to the wrong thing**. §3.1's
decision (a) — the owner's, 14 November 2026 — already drew the line:
the display list travels as a copied-then-validated shared buffer, and
"the message carries a generation-tagged buffer handle and the chunk's
length, **never the chunk**". So there are two artefacts on this
boundary, not one, and they need different proofs:

| | What it is | Where its proof lives |
|---|---|---|
| **The transport message** | A generation-tagged buffer handle and a length. Small, fixed-size, an interface message | **Kani harnesses**, which `hg-ipc-codegen` emits per interface message. VER-1 applies by its own wording — "renderer-to-broker **message types**" — and the proof is cheap precisely because the message is small |
| **The payload** | A chunk in the shared buffer, validated on a copy and dropped whole on failure | The validator, plus the property and mutation suites in `src/tests.rs` (4,000 seeded cases per leaf type, every mutation required to be rejected or to re-encode exactly), plus GR-11's own nightly fuzzers |

So the harnesses VER-1 asks for are real and belong on a message that
**has not been written yet**: §3.1(a) decided the transport, and
nothing has declared it. GR-11 is therefore not unmeetable in
principle, as §3.4 left it reading — it is **unmeetable today for a
scheduled reason**, and it becomes meetable when the transport message
lands. Conflating the two is what made this look like an omission in
the formats.

One consequence worth stating plainly, because it bites the work
immediately next to it: GR-11 is a **T2 target whose failure clause is
"New format work stops until fixed"**. Read literally, with the
harnesses absent, that clause falls on `canvas_command.hgidl` — the
one remaining format, already waiting on §9.2. A target that stops the
work needed to satisfy it is a target that needs its wording fixed
before it is enforced, not a reason to stop.

**Applied here:** §6.19's threat row, which is a description of
mitigations and so the review's to correct.

**Not applied here, and now answered:** GR-11's metric. Amending a T2
ship target's metric is not something this review does on its own
authority, even when the amendment only aligns the target with a
decision the owner has already taken. It went to §9.3 with the exact
replacement text, and **the owner took it on 16 November 2026**
(option (a), ratify). The metric in the spec's §9.1 now reads as §9.3
proposed; the failure clause that blocked `canvas_command.hgidl` is
cleared.

---

## 4. Cross-track items, each with its counterpart named

### 4.1 `hg-paint-order`'s contract — **with the layout spec**

> `hg-paint-order` contract co-signed at **LY-M0's review**

LY §6.14 owns the requirements and the Appendix-E reference builder
(both built); GR §6.24 owns the production implementation. The
co-signature happens at LY-M0's review, which §5 shows cannot
complete. **This item is blocked**, and it is the only one here that
is.

### 4.2 The 160 B† paint share — **co-signed with layout and footprint**

GR-D23 (a) co-signs 160 B† per DOM node across processes: about 72 B
in the renderer and **≤ 88 B† of display items** in the content GPU
process. The item encoding decides whether that holds, and §3.1's
transport decision changes the encoding, so the two are reviewed
together. The failure order is shared with layout (RN2 Q3): first
representation work on the side that is over, then a joint re-split of
the 360 B†, then FP's own action — never a separate ledger line.

### 4.3 The bounded path types — **with the fonts and text spec**

TX §9.5's R21 asked for the path types to live where the text stack
can emit them without depending on L2g; they are in `hg-display-list`
(L2) and `hg-path` (L2g) flattens and strokes them. The review checks
that TX's large glyphs and COLRv1 vectors fit the bounds §6.3 fixed,
because a glyph that does not is a bound chosen without its consumer.

### 4.4 `hg-simd` and `hg-colour` — **with the images and media spec**

`hg-raster` must stay free of `unsafe` (G61's crate list), so SIMD
arrives through the safe API of the one `hg-simd` crate in L2s, which
**media owns**. `hg-colour`'s one numeric contract is media's too.
Neither exists; the review fixes the interfaces it needs.

### 4.5 The trusted-prompt layer (GR-D28) — **with the security owner**

A broker-placed layer above the content layer, unoccludable by the
page. There is no named security owner (D-S8). See §5.

---

## 5. What blocks this review's completion

Two preconditions, both **D-S8**, which Stage 1 §13.1 records as
"Deferred by the owner (14 October 2026)" and §13.1b now opens as
**D-S13**, the waiver FI-D1 requires:

1. **A named graphics owner.** LY-M0's row says the paint-order
   contract is agreed with "a **named graphics owner**, who must exist
   before this review". There is none, so LY-M0's review cannot
   complete, so GR-M0's §4.1 cannot complete.
2. **A named security owner**, for GR-D28 (§4.5).

Three things follow, and the review records all three:

1. **Everything in §§1–3 can be completed without either.** They are
   this spec's own decisions about artefacts that exist, and the
   measurements in §3 are taken.
2. **§4.1 and §4.5 wait**, and nothing else does. That is what makes
   the block two items rather than the milestone.
3. **The start is met and the exit is not**, and the exit evidence
   (`measurements/milestones/m0-starts/`) says so in the same words.

---

## 6. The measurements this review takes as input

The M0 row names six spikes. **None is done**, and four need hardware
or a running engine, so they are inputs rather than outputs:

| Spike | What it decides | What it needs |
|---|---|---|
| **GR-S1** | in-box D3D12 tier, and whether self-hashed DXIL loads on four vendors | four Windows GPUs; rig253 covers one |
| **GR-S2** | per-process driver memory, context-switch and spawn cost — and so **GR-D2's process model** | every reference device |
| **GR-S3** (started) | canonical integer GPU raster against a float prototype | a GPU back end, so GR-M1 |
| **GR-S4** (started) | macOS presentation: IOSurface contents against one `CAMetalLayer` per tile | this Mac |
| **GR-S6** (first pass) | the WGSL atomics clock's resolution | a WebGPU implementation |
| **GR-S9** | the DRM ioctl set and every OS service each GPU process kind needs | a Linux rig per driver |

GR-D2's row makes the process model "decided from GR-S2", so the
review cannot settle it before that spike runs. Recording which items
rest on which spike is what keeps the review from deciding them on
guesswork.

---

## 7. The interlock, in one place

| This track owes | To | Clause |
|---|---|---|
| The bounded path **types**, within §6.3's limits | **TX** | RN2 Q18; TX §9.5 R21 |
| `hg-paint-order`'s production implementation | **LY** | GR §6.24; LY §6.14 owns the requirements |
| The 160 B† paint share, co-signed | **LY**, **FP** | GR-D23 (a); failure order shared (RN2 Q3) |
| The canonical image and its hash | **DevTools**, **DET-1** | GR-50, GR-51 at GR-M1 |
| Routing of decoded image regions and video frames | **MD** | §6.11 |
| The display-list format itself | **LY** (paint), **TX** (glyph vectors) | §6.3 |

| This track is owed | By | Clause |
|---|---|---|
| The fragment store's read API | **LY** | G38: paint reads it, never `hg-layout`'s internals |
| `hg-types`' geometry | **LY** | the types display-list validation uses |
| `hg-paint-order`'s requirements, and the co-signature | **LY** | at LY-M0's review — **blocked** (§5) |
| `hg-simd`'s safe API | **MD** | G61: `hg-raster` stays free of `unsafe` |
| `hg-colour`'s one numeric contract | **MD** | §6.5 |
| Glyph atlases in the content GPU process | **TX** | TX §6.1, §6.10 |
| A **named graphics owner** and a **named security owner** | **D-S8 / D-S13** | §5 |

---

## 8. What this review owes before GR-M0's exit

Everything in §3 decided (§3.4's compositor frame, §3.5's render
graph, §3.6, §3.7's allocator interface and §3.8's threat-table
correction are done; §3.4's canvas format remains, and §3.6's E16 and
§3.7's placement are GR-M1's rather than this review's); §2's five
properties checked against three back ends by someone who has written
one; **one** remaining `.hgidl` format (`canvas_command.hgidl`; the
compositor frame is written, §3.4); the canonical image, E16 and
integer composition specified; GR-S1, GR-S2, GR-S6 (first pass) and
GR-S9 done, GR-S3 and GR-S4 started; and §4.1 and §4.5 unblocked by
D-S13.

## 9. For the owner

Three items — two from §3.4, one from §3.8 — all of a kind this
project does not settle on its own authority: a package boundary, a
format's vocabulary, and a ship target's metric. **All three were
answered on 16 November 2026, each option (a).** Each is recorded
below against the item it answers.

**9.1 The crate-table amendment (a gate-file and spec change)** —
**decided (a), 16 November 2026: land it as described.**
`hg-compositor-frame` is its own package, so that G63 — the compositor
links no display-list reader — is checkable at the granularity
`hg-graph-lint` works at. This changes §6.24's L2 row, adds a
`tools/tiers.toml` row and adds a `G63` row to
`tools/graph-rules.toml`. The code was written and the rule tested
when this was proposed; all three carried the amendment as *proposed
and not yet accepted*, and that wording is now retired in each.

**9.2 How `canvas_command.hgidl` gets the path vocabulary it needs** —
**decided (a), 16 November 2026: add `import` to the IDL.**
The IDL has no import, and the canvas log needs about 140 of
`display_list.hgidl`'s 240 lines. Three options, with a
recommendation:

| | Option | What it costs | What it risks |
|---|---|---|---|
| **(a)** | **Add `import` to the IDL** (recommended) | A change to `hg-ipc-codegen`: `generate()` takes one source string, so it needs a resolver, and `emit.rs` must emit `use` rather than a second definition. Its golden tests churn. Three crates depend on the generator | A language feature written for one caller |
| (b) | Duplicate the vocabulary, guarded by a byte-identity test on the shared block | Nothing in the generator; ~140 duplicated lines in the repo | The copies agree only while the test is kept honest; two `Point` types in one crate |
| (c) | Put the canvas commands in `display_list.hgidl` itself | Nothing; they share one declaration | One protocol means **one** closed error enum (CS-ER-6), so display-list readers would see canvas-only refusal variants, and the two formats' limits would stop being separable |

(a) is recommended because the duplication in (b) is a drift source of
exactly the kind this review exists to remove, and because the reason
for (c)'s cost — one error enum per protocol — is a rule worth keeping
rather than working around. But (a) is the largest change and touches a
generator three crates share, which is why it is a decision rather
than a step.

**What (a) turned out to cost, measured rather than estimated.** The
grammar gains one production, `import NAME { NAME, .. };`. The name is
a bare identifier with **no path syntax**, so an import cannot leave
the crate's own `idl/` directory and no format can acquire a
dependency on another package behind the back of a rule stated over
packages — G63's whole mechanism. An imported protocol **may not
itself import**, so the graph is one level deep and a cycle cannot be
written; that is the answer to the risk this card named, a language
feature written for one caller. `check` needed no change to resolve
imported types: `Ctx::named` already consults the facts map first, so
seeding it with the sibling's facts is the whole of it. `emit` writes
`pub use super::<protocol>::{…}` — one declaration, one set of
harnesses, and the type a consumer sees is the one the sibling
encodes.

The generator's callers got **smaller**, not larger: each of the four
freshness tests read its own `.hgidl` and called `generate`, and now
calls `generate_pair`, which puts the resolution rule in one place
instead of four. `generate` keeps its signature and refuses an import,
so nothing that does not import changed — the checked-in output of all
four generated crates is byte-identical.

**9.3 GR-11's metric, aligned with §3.1's decision (a)** — **decided
(a), 16 November 2026: take the proposed first clause.** Applied to
the spec's §9.1 GR-11 row; the text below is what was proposed and
what was taken, with the cross-reference written in the spec's own
citation form (`GR-M0's review 3.1`) because a bare `3.1` inside the
spec points at the spec's own §3. GR-11 is a
T2 security target whose metric reads "generated readers with Kani
harnesses" for the display list, compositor frame and canvas stream.
Those formats declare no interface, so `hg-ipc-codegen` emits no
harness for them, and under §3.1(a) the display list is explicitly not
a message: the message carries a handle and a length. The harnesses
belong on that transport message, which is not written yet. §6.19's
threat row is corrected here because it describes mitigations; this is
a ship target's metric, so it was yours — and #107 applied the answer.

The failure clause makes it worth doing rather than noting: GR-11's is
"New format work stops until fixed", and read literally, with the
harnesses absent, that clause falls on `canvas_command.hgidl` — the
one remaining format, already waiting on 9.2 above.

Current metric:

> Display lists, compositor frames, canvas, WebGL and WebGPU streams:
> generated readers with Kani harnesses; semantic-limit and
> degradation suites pass; nightly fuzzers with **0** crash bugs older
> than 30 days†; adversarial content processes that never signal,
> signal fences to `u64::MAX` and write after release stall **0**
> compositor frames and reach **0** bytes of another key's tiles

Proposed, with only the first clause changed:

> Display lists, compositor frames, canvas, WebGL and WebGPU streams:
> generated readers, with **Kani harnesses on every transport message
> type** and **the validator plus property and mutation suites on
> every payload that is validated on a copy** (3.1's decision (a));
> semantic-limit and degradation suites pass; nightly fuzzers with
> **0** crash bugs older than 30 days†; adversarial content processes
> that never signal, signal fences to `u64::MAX` and write after
> release stall **0** compositor frames and reach **0** bytes of
> another key's tiles

**Recommended**, because it changes no threshold and adds no
exemption: it says where each proof lives for a transport the owner
has already decided, and it is the difference between a target that
can be met and one that stops the work needed to meet it. The
alternative — leaving GR-11 as written and giving the two formats
interfaces so the harnesses attach to them — was refused in §3.1 for
the display list on its own merits, so adopting it here would reopen
a decided item in order to preserve a sentence.
