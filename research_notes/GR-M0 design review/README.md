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

**And the one piece of §6.6's mandate that is not built: the
allocator.** `hg-gpu-alloc` does not exist. §6.5 makes placement the
thing that decides fragmentation, residency and per-site accounting,
and every allocation carries FP 6.6's ledger tag. The review decides
its interface here; the implementation is GR-M1's. Writing a
placeholder crate would have met the letter of the start definition
and told the review nothing, so there is none.

---

## 3. Items the review must decide, in their own right

### 3.1 The display list does not fit in a message — **decide the transport**

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

**The review decides one of:** (a) write the copied-then-validated
shared-buffer transport into the format, with the chunk as the unit of
validation; (b) shrink the per-chunk bounds so a chunk *is* a message
and let a path spill across chunks; (c) a hybrid — small chunks as
messages, large ones by buffer. The bounds are §6.3's, so this is the
graphics spec's decision and not the IPC layer's.

### 3.2 The accumulator's unit — **1/256 px² or exact**

§6.4 specifies "exact area coverage by signed-area accumulation in
**1/256 px² units**". `hg-raster` accumulates in `SUB`-squared units
(`FULL = 2 · SUB · SUB`) because an exactly-representable trapezoid
area needs the finer denominator; at 1/256 px² the accumulation itself
rounds, and "exact area coverage" and "1/256 px² units" cannot both
hold. The code took the exact route and the difference is recorded
here rather than resolved silently.

**The review decides:** whether §6.4's unit is corrected to the finer
one, or the coarser unit is kept and "exact" is weakened. The
measurement that bears on it is in §3.3.

### 3.3 `hg-raster-ref` is exact, not float64 — **confirm or revert**

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

**The review decides:** confirm the deviation and amend §6.26, or
require float64 and explain what the extra rounding buys.

### 3.4 The two formats not yet written

The M0 row names three `.hgidl` formats. One exists
(`display_list.hgidl`). **`compositor_frame.hgidl`** and
**`canvas_command.hgidl`** do not, and both have counterparts: the
compositor frame is what the shared compositor validates (G63 forbids
it a display-list reader at all), and the canvas command log is read
by the in-renderer CPU replay GR-D12 requires. The review fixes both
before GR-M1.

### 3.5 The render graph's remaining half

`RenderGraph::compile` derives barriers from a *sequence* of passes,
and `ValidationError::Cycle` is a variant nothing can produce because
a cycle is not expressible in a sequence (`gh11` says so rather than
leaving a dead variant). The review decides whether the graph gains
explicit edges — and if it does, the cycle check arrives with them.

### 3.6 The canonical image, E16 and integer composition

Named by the M0 row and not started. These are specification items
rather than code, and they are the precondition for GR-50 and GR-51's
hashes at GR-M1.

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

Everything in §3 decided; §2's five properties checked against three
back ends by someone who has written one; `hg-gpu-alloc`'s interface;
the two remaining `.hgidl` formats; the canonical image, E16 and
integer composition specified; GR-S1, GR-S2, GR-S6 (first pass) and
GR-S9 done, GR-S3 and GR-S4 started; and §4.1 and §4.5 unblocked by
D-S13.
