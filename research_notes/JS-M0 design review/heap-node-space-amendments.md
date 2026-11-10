# Proposed amendments: the node space's two settled decisions

*Status: **proposed, not accepted**. Written 9 November 2026 for JS-M0
Phase 2 item 7, revised the same day when the remaining two decisions
were taken. All four node-space decisions are now settled; this file
writes down the amendments they imply and the derivations behind them,
for owner review. It changes no spec file — the edits below are proposed
text, not applied text.*

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
> body: traced slots, then weak slots, then **node slots**, then raw
> words. A **node slot** is 8 bytes, 8-aligned, holding a node space's
> 32-bit slot index and its 32-bit epoch; it is traced strongly, and it
> is the only way a cage object names a node (JS-D4 (a)). Every object
> starts on an 8-byte boundary, so its offset is a slot offset. | One
> layout for fixed and variable objects. The type table is outside the
> cage, so the tracer's layout comes from a bounds-checked index (H-2),
> and a corrupted type index selects another valid layout, never an
> out-of-table read. A node slot is a field kind rather than a third slot
> tag so that JS-D1 (a)'s two-tag encoding, and the Kani proof that every
> bit pattern decodes inside the cage, are unchanged. Node slots come
> after the 32-bit slots, not between the traced and weak ones, so that
> the offset of a traced or weak slot is unchanged by a type gaining a
> node slot | The object model's shapes (JS-M1) may move the header to a
> shape reference; the type table then becomes the shape's layout
> descriptor |

An earlier draft of this row put node slots between the traced and the
weak ones. Putting them after both is the same size — the 8-byte
alignment padding falls in the same place — and it leaves every existing
slot offset alone, so it is what the implementation does.

### Why the epoch is in the field, on soundness grounds

The owner asked for this to be settled on soundness alone, since the
JS-D4 spike showed a 4-byte and an 8-byte reference give **byte-identical
totals per node** (the wrapper's body pads to 8 either way, so both make
a 16-byte wrapper; see `measurements/milestones/js-m0/d4-nodes`). The
reason to carry the epoch is **not** that the traced edge could go stale:
it cannot. Decision 2 makes the edge strong, so the node a live wrapper
names is live by construction, and a strong traced edge is never
dangling.

The reason is H-2, and it is **already accepted text, not a new
choice**. The wrapper is a cage object, so its contents are untrusted,
and an attacker who can write the cage can forge a node reference in it.
The JS engine spec's Section 6.2 already requires the check that catches
that, in the "Why outside the cage (H-2)" bullet:

> The wrapper's node ID, which is cage data, is checked against the
> slot's generation on every binding call.

A check of the node ID against the slot's generation needs the
generation to be *in* the wrapper. So the field must carry it: the 8
bytes follow from text the owner has already accepted, and are not a
recommendation this file is making. Without the epoch a forged slot index
would be bounds-checked and then believed, which H-2 tolerates as a wrong
answer inside the cage but which 6.2 has already decided not to tolerate
on the binding path.

The collector uses the same field for the same purpose: `live_node`
bounds-checks the index against the space's slot count and compares the
epoch with the space's own before following the edge, so a stale or
forged reference is not followed. In a well-formed heap a strong node
reference can never be stale — the edge keeps its target alive — so a
mismatch means corruption, and the collector's response is to ignore it.

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

## Amendment 3 (decision 3): an external node-root list

**Taken: a node-root list in `hg-heap`, registered and released by the
embedder, with the generative scope available for short-lived sets.**

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

**Taken: (a), with (b) available for short-lived sets.** (a) is the only
one that can root a detached subtree held by ID, which the DOM needs
(ranges, the parser's insertion point, `MutationObserver` records).
(c) was ruled out because it makes the document's liveness depend on a
wrapper existing, which inverts the lazy-wrapper rule for the one node
that least deserves a wrapper.

Two things carried through, as the decision required:

* **The leak risk is handled the way the kept-alive list's is.** The list
  is the embedder's to manage: `add_node_root` returns a handle,
  `remove_node_root` releases that entry, and `clear_node_roots` empties
  the list at a defined point, as `clear_kept_objects` does. A forgotten
  root keeps its node alive, which is exactly the exposure `keep_alive`
  already has, and no more.

  The handle carries a **generation** as well as an index, for the same
  reason a `NodeRef` does. Entries are reused, and the first draft of
  this change used a bare index: a stale or twice-removed handle would
  then have unrooted whichever node had taken that entry since — and
  unrooting a live node releases it. The entry's generation is raised on
  every reuse and a handle that does not match is ignored, so the
  property the list is supposed to have ("a stale handle can never name
  another node") is now actually true rather than merely claimed. The
  review in `tools/checks.sh` caught that claim before it shipped.
* **The verifier checks every node root names a live slot.** That is the
  `DeadNodeRoot` violation, with a test that releases a rooted node
  without dropping its root and asserts the verifier names the root's
  index and the slot. It lands with the roots, in this change.

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

## An open question: external traced slots and the brands

One part of the node-space interface is **not** enforced by the type
system, and it should be on the record rather than discovered later.

A cage-side traced field is traced because its object's layout declares
it. An external traced slot has no layout: it is traced because the
space's `trace` visits it. So the interface carries an obligation —
*every external slot the space owns must be visited by `trace` for the
node that owns it* — and nothing checks it. A slot `trace` never visits is
never updated, so after a cycle that moved its target, reading it gives a
reference to where the object used to be. H-2 keeps that inside the cage,
so it is a wrong answer and not an invalid access, but the rooting rules
(JS spec 6.5, soundness step 2) are supposed to make a stale reference
impossible in safe code, and here they do not.

Worse, the **brands cannot cover the cross-heap case** the way they cover
every cage-side store. `Gc` and `Root` carry the invariant brand `'h`
from a generative `Heap::enter`, which is what makes a cross-heap store a
type error (soundness step 6). An external slot cannot carry that brand,
because it lives in a rare-data table that outlives any one `enter`,
while a fresh brand is minted per `enter`. So heap A's mutator writing
into a slot heap B traces is a type error nowhere.

What this change does about it: `ExternalSlot::set` **asserts in debug
builds** that the reference belongs to the writing mutator's heap, which
is the same device soundness step 6 already uses ("every GC point asserts
in debug builds that the mutator's heap is the thread's current heap"),
and the obligation is documented on both `ExternalSlot` and
`NodeSpace::trace`.

What it does not do: make either property a type error. Two directions
exist if the owner wants one, and both are larger than this change:

* **Brand the table, not the slot.** `hg-dom`'s rare-data tables become
  per-heap and hand out `ExternalSlot` references only through a token
  that carries the heap's identity — closer to how `TokenCell` and
  `ScriptToken` guard DOM side tables already (CS-D3), and probably where
  this wants to end up.
* **Make the space declare its slots once**, so `hg-heap` holds the list
  and `trace` cannot omit one. This is the push registry that constraint
  1 rules out for the *columns*, but the rare-data tables are `hg-dom`'s,
  not `hg-node-store`'s, so it is not ruled out for them.

**This is the CS-D3 re-run's business** (JS-M0 plan Phase 2 item 2, which
is also where Q-JS-10 is settled), since it is exactly the question of
what a token has to guard. Recorded here so that re-run has it.

**Answered by the re-run (9 November 2026), with a third direction.**
Neither of the two above: the obligation is not made a type error, it is
made a *checked* obligation. Each slot now remembers the cycle it was
last written or traced in — `set` stamps it with the cycles completed so
far, the collector stamps it again as it traces it, and `get` asserts in
debug builds that the stamp is not older than the last completed cycle —
so a space that forgets a slot panics on the first read after a cycle,
naming the fault, instead of answering with a stale offset. It catches
the omission per slot, where a declared-slot registry would catch it per
space; it costs nothing in a release build, the stamp being a second
`AtomicU32` present only under `debug_assertions`; and it runs under
Miri with the moving test collector. The two structural directions above
are decisions about `hg-dom`'s tables and stay open, unforeclosed, for
M2. The cross-heap half keeps the debug assertion in `ExternalSlot::set`
and stays open in the same terms. See
`measurements/milestones/js-m0/cs-d3/README.md`.

## A choice for the owner: H-4's forwarding bit

**The spec and the implementation disagree, and the gap costs the
verifier its strongest invariant.** H-4 says the object header is "a type
index into the heap's type table ... **plus a forwarding bit**". The
implementation has no forwarding bit: `State::visit` keeps an external
`BTreeMap<u64, u64>` for the duration of a cycle and marks both the old
and the new copy of every object it moves.

That is sound — nothing reads the bit, so nothing is misled — but it
means the heap state after a cycle cannot distinguish a **stale old
copy** from a **lost object**. The stale copies are real and documented:
when `alloc_copy` runs out of room mid-cycle its block leaves the
evacuation set, the sweep keeps the block, and the objects already copied
out of it stay behind, marked and unreachable (the case
`an_evacuation_that_runs_out_of_room_keeps_its_live_objects` covers). So
the verifier can check that *every reachable object is marked and live*,
but not the converse, *every marked object in a live block is reachable
from the roots* — which is the check that would catch a collector that
marked something it should not have.

Two ways to close it, and the choice is the owner's. **Not proposed
here, and deliberately not fixed inside this change.**

| Option | What it costs | What it buys |
|---|---|---|
| **Implement the bit H-4 already specifies.** The old copy's header says "forwarded" and its length word holds the new offset | One store per moved object, and the `BTreeMap` goes away, so a cycle gets cheaper as well | Two-directional reachability in the verifier, and the JS-D4 spike found the same technique made its own evacuation fix-up O(1) instead of a hash lookup |
| **Amend H-4 to match the implementation**, dropping the forwarding bit from the header's description | Nothing | The spec stops describing a field that does not exist; the verifier's one-directional check becomes the documented limit |

The first is the better engineering answer and the cheaper one at
runtime. The second is honest if the bit is not wanted.

## Open JS-M0 item: the key-custody assertion pass (WebCrypto WC-9)

JS spec 6.5's soundness step 8 says the debug-build heap verifier
"includes the **key-custody assertion pass** (the WebCrypto spec's WC-9,
owned jointly with the WebCrypto track): at GC-safe points it asserts
that engine-held key material stays in the per-process secret-type tables
outside the cage and is never reachable from an agent heap."

The verifier built at Phase 2 item 4 does **not** include it, because the
per-process secret-type tables it would assert over do not exist yet:
nothing in the tree allocates key material, so there is nothing to
assert. This is recorded as an open JS-M0 item rather than left to be
noticed later. It needs the WebCrypto track's tables first, and then one
pass added to `verify.rs`.
