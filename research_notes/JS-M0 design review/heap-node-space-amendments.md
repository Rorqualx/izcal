# Proposed amendments: the node space's two settled decisions

*Status: **proposed, not accepted**. Written 9 November 2026 for JS-M0
Phase 2 item 7. Two of the four decisions the JS-M0 design review left
open for the node space have been taken; this file writes down the
amendments they imply, for owner review. It changes no spec file — the
edits below are proposed text, not applied text. The other two decisions
are still open and are recorded at the end with the options, so the owner
has them in one place.*

## Why this file exists

[`heap.md`](heap.md) fixes `hg-heap` at JS-M0 in decisions H-1 to H-12.
It contains **no node-space decision**: not the side mark bitmap, not
external traced slots, not the wrapper's edge, not evacuation's
interaction with any of them. The JS engine spec's Section 6.2 fixes the
*shape* of JS-D4 (a) — columns outside the cage, mark bits in a per-slot
side bitmap, edges that are the links plus external traced slots, a lazy
wrapper held strongly once created — but not the interface, and
`hg-heap`'s tier row (Section 6.15) assigns `hg-heap` the "node-space
tracing interface" without saying what it is.

Four decisions were needed before the interface could be built. Two are
now taken.

## Amendment 1 (decision 1): a `node` field kind, amending H-4

**Taken: the node reference is a new field kind in the object layout,
holding the 32-bit node slot and its 32-bit epoch, traced strongly.**

JS-D4 (a) requires the wrapper to hold "the node's generational ID" and
requires the collector to follow that edge: without it, `var x = node;
node.remove()` would free a node that script still holds. But H-4's
object layout has only traced slots, weak slots, raw 64-bit words and a
tail, none of which can carry a traced node reference, and JS-D1 (a)'s
slot encoding has exactly two tags — a small integer and a cage offset —
with a Kani harness (`any_slot_decodes_inside_the_cage`) proving every
bit pattern decodes to an 8-aligned offset inside the cage.

The rejected alternative was a **third slot tag** for node references.
It would have changed the accepted JS-D1 (a) encoding and invalidated
that proof. A new field kind changes neither: the encoding is untouched
and the proof stands as written. That is why it wins.

### Proposed replacement for H-4

> | H-4 | **Object layout:** a 4-byte header (a type index into the heap's
> type table, which lives outside the cage, plus a forwarding bit), a
> 4-byte length word (for variable-size objects; 0 otherwise), then the
> body: traced slots first, then **node slots**, then weak slots, then
> raw words. A **node slot** is 8 bytes holding a node space's 32-bit
> slot index and its 32-bit epoch; it is traced strongly, and it is the
> only way a cage object names a node (JS-D4 (a)). Every object starts on
> an 8-byte boundary, so its offset is a slot offset. | One layout for
> fixed and variable objects. The type table is outside the cage, so the
> tracer's layout comes from a bounds-checked index (H-2), and a
> corrupted type index selects another valid layout, never an
> out-of-table read. A node slot is a field kind rather than a third slot
> tag so that JS-D1 (a)'s two-tag encoding, and the Kani proof that every
> bit pattern decodes inside the cage, are unchanged | The object model's
> shapes (JS-M1) may move the header to a shape reference; the type table
> then becomes the shape's layout descriptor |

### Why the epoch is in the field, on soundness grounds

The owner asked for this to be settled on soundness alone, since the
JS-D4 spike showed a 4-byte and an 8-byte reference give **byte-identical
totals per node** (the wrapper's body pads to 8 either way, so both make
a 16-byte wrapper; see `measurements/milestones/js-m0/d4-nodes`). The
reason to carry the epoch is **not** that the traced edge could go stale:
it cannot. Decision 2 makes the edge strong, so the node a live wrapper
names is live by construction, and a strong traced edge is never
dangling.

The reason is H-2. The wrapper is a cage object, so its contents are
untrusted, and an attacker who can write the cage can forge a node
reference in it. JS spec 6.2 already requires the check that catches
that: *"The wrapper's node ID, which is cage data, is checked against the
slot's generation on every binding call."* That check needs the
generation to be in the wrapper, so the field must carry it. Without the
epoch a forged slot index would be bounds-checked and then believed,
which H-2 tolerates as a wrong answer but which the spec has already
decided not to tolerate on the binding path.

So: 8 bytes, and the cost is nil.

## Amendment 2 (decision 2): node→wrapper is strong, amending Stage 1 5.2

**Taken: strong, confirmed.**

The JS engine spec is already explicit — "Once created, the node holds
its wrapper strongly and the wrapper holds the node ... and the pair is
collected together", with the DOM spec adding that JS-D4 "holds the
wrapper strongly instead" of Chrome's drop-and-recreate behaviour. But
[Stage 1 design](../../reports/HGBA%20Stage%201%20architecture%20design.md)
Section 5.2 still says the opposite, in its node-storage table:

> | Wrapper | A `hg-bindings`-generated wrapper is created lazily on first
> script access and holds a checked `NodeHandle` (slot plus epoch).
> **Node→wrapper is a weak slot; the collector traces handles, not tree
> pointers** |

JS spec line 1129 lists Stage 1 5.2 among the sections JS-D4 supersedes,
so the engine spec governs. The amendment is proposed anyway, rather than
left to that supersession, because **Stage 1 5.2 is the only place
`NodeHandle` and release-by-ID are written down**, so it is a section
people will keep reading for those, and a reader who takes its wrapper
row at face value gets the lifetime rule backwards.

### Proposed replacement for that row

> | Wrapper | A `hg-bindings`-generated wrapper is created lazily on first
> script access and holds a checked `NodeHandle` (slot plus epoch) in a
> **node slot** (heap.md H-4). **The node holds its wrapper strongly once
> created, and the wrapper holds the node; the pair is collected
> together** (JS-D4 (a)). There is no wrapper-lifetime heuristic. The
> collector traces handles, not tree pointers |

Nothing else in 5.2 changes: `NodeId` stays a 32-bit slot plus a 32-bit
epoch, release-by-ID stays as written, and `StoreWriter<TracedBarrier>`
stays the only write path.

## Still open: decisions 3 and 4

Neither is needed for the heap verifier, which is why the verifier went
first. Both are needed before the node-space interface.

### Decision 3: how node-space roots reach the collector

Nothing in the spec, the design review or Stage 1 says this. A node is
live if it is reachable from another live node's links, or from a live
wrapper's node slot — but something has to be the root of the tree
itself, and `Root<'h, 's, T>` holds a `Slot`, which is cage-only. The
document node is not a cage object, so it cannot be rooted today.

| Option | How | For | Against |
|---|---|---|---|
| **(a) An external root list in `hg-heap`**, of node slot indices, registered and released by `hg-dom` | `m.add_node_root(id) -> NodeRootHandle`, released on drop | Mirrors the kept-alive list, which R0-O10 already puts outside the cage; one mechanism for "a root the embedder holds"; cheap to enumerate | A second root kind for the collector to walk; the embedder can leak by forgetting to release, with no type-level scope |
| **(b) A generative node-root scope**, like `Mutator::scope` | `m.node_scope(\|m, s\| …)`, roots tied to `'s` | Same discipline as cage roots, so one rule to learn; cannot outlive its scope | A document outlives any scope a turn can open, so the document itself still needs (a) or (c) |
| **(c) The document is reached only through its wrapper** | `hg-dom` keeps the document's wrapper in an ordinary `Root`; the node slot is traced through the wrapper's node slot | No new root kind at all; falls straight out of amendment 1 | Forces a wrapper to exist for every document from the start, which contradicts "most nodes are never touched by script" for the one node that matters least; and a detached subtree `hg-dom` holds by ID alone would have no root |

**Recommendation: (a), with (b) available for short-lived sets.** (a) is
the only one that can root a detached subtree held by ID, which the DOM
needs (ranges, the parser's insertion point, `MutationObserver` records).
(c) is tempting because it needs nothing new, but it makes the document's
liveness depend on a wrapper existing, which inverts the lazy-wrapper
rule. The leak risk in (a) is the same one the kept-alive list already
carries and is handled the same way: the handle releases on drop, and the
verifier can check that every node root names a live slot.

### Decision 4: the direction of the tracing interface

Recorded in my earlier report and unchanged: a **pull trait** that
`hg-dom` implements over `hg-node-store`'s dependency-free columns, with
external traced slots as `hg-heap` types living in `hg-dom`'s rare-data
tables. It is the only shape that satisfies all three written
constraints at once — `hg-node-store` has no `hg-heap` dependency (JS
6.2), external traced slots *are* `hg-heap` field types (JS 6.2), and the
wrapper's slot is a field of the node's rare data (DOM 6.4) — because a
push registry of slot addresses would force `hg-node-store` to hold
`hg-heap` types.

## One thing the verifier found, for the record

H-4 specifies "a type index **plus a forwarding bit**" in the object
header. The implementation has no forwarding bit: evacuation keeps an
external `BTreeMap` for the cycle and marks both copies. That is sound,
but it costs the verifier its strongest invariant — *every marked object
in a live block is reachable from the roots* — because a stale old copy
left behind when `alloc_copy` runs out of room (the documented case in
`an_evacuation_that_runs_out_of_room_keeps_its_live_objects`) cannot be
told from a lost object without it. The verifier therefore checks
reachability in one direction only. Adding the forwarding bit H-4 already
specifies would let it check both; that is a separate change and is not
proposed here.
