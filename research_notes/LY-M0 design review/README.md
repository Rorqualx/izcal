# LY-M0 design review: the agenda

Opened 13 November 2026, for the layout track's M0 milestone
(`reports/Izcal layout.md` §9). **Opening the review with its agenda
fixed is M0's *start*; completing it is M0's exit** — and completing
this one has a precondition the project cannot meet today, which
Section 5 states plainly rather than leaving to be discovered.

The M0 row's mandate is unusually broad:

> **design review** of Section 6 as a whole, with a provenance review
> by a reviewer who has not read the engine source the notes avoided

"Section 6 as a whole" is 28 sub-sections. What this document does is
say which of them the review must *decide* and which it only has to
*check*, because a review with no such distinction decides nothing.

---

## 1. What is already built, and therefore reviewed rather than designed

These landed before the review opened, so the review's job for each is
to find what is wrong with it, not to decide it from nothing.

| Built | Section | What the review checks |
|---|---|---|
| `LayoutUnit` and the geometry | §6.2, LY-D3 | that 1/64 px meets TX-D4's cumulative positions without drift, and that the range limit is a recorded deviation (Q-LY-2) |
| The numeric boundary (`hg-layout-num`) | §6.2, G39 | that the lint's crate list is the whole of layout, and that the rounding function is the only one |
| The fragment store (`hg-fragment`) | §6.13, LY-D18, G45 | that the canonical tree holds nothing history-dependent, and that reclamation cannot leave a dangling reference |
| The layout contract (`hg-layout`) | §6.4, LY-D1 | **the central item** — see §2 |
| The box tree (`hg-box-tree`) | §6.3, LY-D2 | the anonymous-box rules, and the incremental rebuild's boundary |
| The reference engine (`hg-layout-reference`) | §6.1, LY-D5, G41 | that G41's list is complete and that nothing else crept in |
| `hg-paint-order`'s reference builder | §6.14 | Appendix E's order — but the *contract* is blocked, see §5 |

---

## 2. The central item: is the constraint space the complete input?

LY-D1 says the constraint space is "the **complete** input besides the
box's own subtree, its computed styles and the frame's recorded state.
Nothing else is read: no previous fragment, no sibling's state, no
global."

That sentence is the design. Everything else in §6.4 — the cache, the
diff, the 2ⁿ bound — follows from it, and it is the claim the review
must attack hardest, because the failure it prevents is the one
LayoutNG took about six years to fix:

> This is the rule that ended hysteresis in LayoutNG.

**What the review must settle.**

1. **Is every input in the space?** The list in §6.4 has fourteen
   entries. The review goes through the formatting contexts LY-M1 will
   write and asks, for each, what it reads. A context that needs
   something not in the list is a finding; adding it later is a cache
   invalidation bug that will not show until a page hits it.
2. **Is anything in the space that is not an input?** A field nothing
   reads widens the cache key and lowers the hit rate for nothing.
3. **Does `relayout_needed` match the list?** It is the one central
   diff, and a field it compares that nothing reads is the same waste;
   a field it *fails* to compare is a wrong answer served from cache.
   It is property-tested against a fresh layout (LY-3, G42), and the
   review fixes what that property test must cover.
4. **The interning.** Spaces are "interned per pass (siblings usually
   share one)". The review fixes what equality means for an interned
   space, because that equality *is* the cache key.

---

## 3. Items the review must decide, in their own right

| Item | Section | Why it needs deciding |
|---|---|---|
| **The canonical stack budget** | §6.17, LY-D15 | The M0 row names it: "the canonical stack budget set". It must give a box-depth limit of at least 10,000† levels, be identical on every platform, and be set from *measured* frame sizes — so the measurement comes first and the number second. |
| **Fragmentation's break appeal** | §6.7 | Fragmentation "does not define whether content breaks at a particular allowed break", so Izcal writes the rule. The review fixes the appeal scale and the threshold's units. |
| **The float area's representation** | §6.5 | It is in the constraint space and it is persistent, so its equality is part of the cache key. The review fixes what "the same float area" means. |
| **The incremental rebuild's boundary** | §6.3 | "the box subtree of the nearest ancestor whose box structure can be affected" — the review fixes that ancestor's definition per change kind, because getting it too narrow is an under-rebuild and too wide is a performance cliff. |
| **The early cutoff's comparison** | §6.12 | "if a box's new fragment equals its old one in size, baselines, adjoining-margin summary, overflow and out-of-flow list". The review fixes that list; anything left out is an under-propagation. |
| **Eviction** | §6.4 | 64† passes, "driven only by canonical pass counters, so eviction is deterministic". The review checks that nothing else can evict. |

---

## 4. Cross-track items, each with its counterpart named

### 4.1 Container re-entry — **with the style spec**

The M0 row: "the re-entry protocol agreed with the style spec at
**ST-M0's design review**". So this item is settled *there*, not here,
and the two reviews must not settle it differently.

**Counterpart: ST** (`research_notes/ST-M0 design review/`), item 5 of
its agenda, which quotes both sides. The layout spec's §9.5 row is
already applied in the style spec (revision 0.3.7):

> container re-entry runs at a scheduler barrier with no layout
> worker, once per (container, distinct queried size) per pass with a
> pass-local overlay

**What this review owes that one**: the scheduling, because "the
layout spec owns the scheduling" (ST §6.10). What it is owed:
the entry point's signature and the shape of "what changed".

### 4.2 The text interface — **with the fonts and text spec**

The M0 row: "the text interface agreed with TX-M0 (**R22**)".

**Counterpart: TX** (`reports/Izcal fonts and text.md`), TX-M0.
TX-D4 is **unchanged** by this track — the layout spec's revision 0.2
records that the proposed numeric amendment to the text spec was
withdrawn. What layout does instead is convert the text stack's
**cumulative** positions to `LayoutUnit` once, at item and line
boundaries, never individual advances:

> a run's width is `to_layout_unit(end) − to_layout_unit(start)`

so line widths never drift and layout widths equal what `measureText`
reports after the same rounding. **A CI check compares them (LY-9)**,
and the review fixes what that check runs on.

### 4.3 `hg-paint-order`'s contract — **with the graphics spec**

See §5: this one is blocked.

### 4.4 The budget — **co-signed with the graphics spec**

The style spec gives layout and paint 360 B† per node; layout takes
**≤ 200 B†** and paint **160 B†** (GR-D23). The split is called
"provisional" by the layout spec's own revision 0.2, and Q-LY-15
confirms the fragment-to-box ratio at LY-M0 — so the measurement is an
input to this review, not an output of it.

---

## 5. What blocks this review's completion

> `hg-paint-order`'s contract agreed with a **named graphics owner**,
> **who must exist before this review**

There is no named graphics owner. That is **D-S8**, which Stage 1
§13.1 records as "Deferred by the owner (14 October 2026)", and which
the same section says must be decided before Stage 1 can be staffed.

This is not a reason to leave the review unopened — the M1-X1
definition of "start" is opening it with the agenda fixed, and that is
done. It is a reason the review **cannot be completed**, and the
honest record of that is here rather than in a status report later.

Three things follow, and the review records all three:

1. **The requirements are this spec's and can be written now.** §6.14
   owns them, and the Appendix-E reference builder is built and
   tested. What needs a graphics owner is the *agreement*, not the
   statement.
2. **Everything downstream of that agreement waits.** GR-M0's own row
   makes `hg-paint-order` co-signed here, so GR-M0 waits on LY-M0
   waiting on D-S8. Phase 4 of the M1-X1 plan says so: "GR-M0's
   substance waits on LY-M0's review and on a named graphics owner."
3. **Nothing else in this agenda waits on it.** §§1–4 can be completed
   without a graphics owner, and doing so is what makes the block
   *one* item rather than the whole milestone.

---

## 6. The provenance review

> a provenance review by a reviewer who has not read the engine source
> the notes avoided

M-1 part 3 says another engine's design may be cited "as evidence …
from a published source, never as a template", and part 4 forbids
reading its code to settle a disagreement. The layout spec's own
fact-check "fetched every link in the spec and notes, and **found no
link to engine source**".

So the provenance review's job here is narrower than the style spec's
was, and it is a **precondition on who, not on what**: the reviewer
must not have read the engine source files. Recording that here means
it cannot be met by the author of this document, nor by anyone who
worked on §6.

---

## 7. The interlock, in one place

| This track owes | To | Clause |
|---|---|---|
| Container re-entry's **scheduling** | **ST** | "the layout spec owns the scheduling" (ST §6.10) |
| `hg-layout-reference`, with size containment | **ST** | ST-M2 tests re-entry against it (LY-M1, by month 21†) |
| The fragment store's read API | **GR**, the semantic plane, DevTools | G38: every consumer reads it and never `hg-layout`'s internals |
| `hg-paint-order`'s **requirements** | **GR** | LY §6.14; GR §6.24 owns the implementation |
| The layout unit, and one rounding into it | **TX** | TX-D4 unchanged; cumulative positions converted once (LY-9) |
| `hg-types`' geometry | **GR**, DevTools, the GPU service | the types paint and display-list validation use |

| This track is owed | By | Clause |
|---|---|---|
| The computed-length type and the "affects" mask | **ST** | ST §6.9; binary64, fixed by the SVG spec's R5 |
| The re-entry protocol's agreement | **ST** | at ST-M0's design review |
| The metrics query and shaped segments | **TX** | TX-M0; `break_reshapes`, `join_reshapes`, `kashida_point` |
| A **named graphics owner** | **D-S8** | "who must exist before this review" — §5 |
| The paint budget's other half | **GR** | GR-D23's 160 B†, provisional |

---

## 8. The measurements this review takes as input

Named by the M0 row: `Q-LY-1`, `Q-LY-2`, `Q-LY-3`, `Q-LY-4`, `Q-LY-6`,
`Q-LY-15`. Each is a **black-box** observation — M-1 part 2 — and none
of them is done. They are inputs, so the review cannot settle the
items that rest on them until they are:

| Measurement | What it feeds |
|---|---|
| Q-LY-1 | the box-depth limit, against what three engines do |
| Q-LY-2 | whether anything needs lengths beyond ±33.5M px (LY-D3's trigger) |
| Q-LY-3 | — |
| Q-LY-4 | `hg-layout-query`'s forcing table, against an exhaustive WebIDL probe |
| Q-LY-6 | where CSS Tables 3 is silent or disputed |
| Q-LY-15 | the fragment-to-box ratio, and so the 200 B† budget |
