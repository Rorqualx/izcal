# ST-M0 design review: the agenda

Opened 13 November 2026, for the style track's M0 milestone
(`reports/Izcal HTML and CSS parsing and style.md` §9.2). **Opening the
review with its agenda fixed is M0's *start*; completing it is M0's
exit.** This document is the agenda and nothing else: it says what the
review must settle, quotes the clause that says so, and names the
counterpart track for every item that cannot be settled here alone.

The M0 row's own sentence is the whole of the mandate:

> **Design review** fixing, together, style groups, match keys, the
> invalidation index, the substitution interpreter, container re-entry
> with the layout spec, and the `:has()` engine.

"Together" is the operative word. Each of the six has been designed
separately in Section 6; what has not been done is checking that the
six hold at once, because they share one set of identities and one
pass structure. The review's product is a decision per item plus the
record of what each decision costs the other five.

---

## Why the six are one review and not six

They are coupled through three things, and each coupling is a place
where settling one item alone would settle it wrongly.

| Coupling | Items it binds | Why it cannot be split |
|---|---|---|
| **The identity a style result is named by** | style groups, match keys, container re-entry | A match key names a cascade result; a style group is what that result is stored in; container re-entry creates a *second* result for the same element under a different size. If the key does not carry the size, re-entry collides with the normal pass; if it does, the key is wider and the group's sharing rate falls. |
| **The pass's phase structure** | the invalidation index, container re-entry, the `:has()` engine | All three decide *what to visit*. Invalidation computes a restyle set before the pass; `:has()` inverts direction during it; re-entry runs after layout at a scheduler barrier. Each one's answer bounds the others' work, and the `:has()` memo's lifetime is "one pass" — which re-entry makes ambiguous. |
| **What a declaration's value depends on** | the substitution interpreter, style groups, the invalidation index | A `var()` reference makes a computed value depend on an ancestor's custom property, so it crosses the element boundary that both the group and the index assume. |

---

## Item 1: style groups

**What §6.9 fixes today.** Properties are partitioned into groups; a
computed style is a set of interned group pointers, shared between
elements whose cascade produced the same group.

**What the review must settle.**

1. **The group boundaries.** The spike `Q-ST-5` ("bytes per element and
   group boundaries") measures this on corpus snapshots; the review
   takes its number. Boundaries drawn too fine cost a pointer per
   group; drawn too coarse, one differing property copies a whole
   group.
2. **Where the intern table lives.** The SVG spec's R5 moved it from
   per agent to **per style scope** — "one agent or one static
   document" — so a static document is its own style scope. The review
   checks that nothing in the pass assumes one table per agent.
3. **What is in a group and what is beside it.** The computed-length
   type is **binary64** (§6.9, fixed by SV-D2's geometry needing the
   same type). Colours stay f64 through `hg-math` (MD-D28). Neither
   fits an interned small value, so the review decides whether a group
   holds them inline or by reference.

**Counterpart:** none directly, but item 1's answer is read by item 2
and item 5.

**Open before the review:** `Q-ST-5`.

---

## Item 2: match keys

**What §6.9 fixes today.** A match key summarizes everything a
cascade's result depends on, so two elements with the same key share a
style.

**What the review must settle.**

1. **What the key must carry.** Candidate inputs: the matched rule
   set, the inline-style block, the parent's computed style (for
   inherited values), the element-state bits that matched, the
   container size when inside a size container, the custom-property
   environment, and the `:visited` partition. Each one widens the key.
2. **How a key is computed without reading another element's
   provisional result.** The parallel phase uses "provisional,
   content-based identities and task-local output buffers" (§6.9,
   §6.12), so a key that named a parent's *published* identity could
   not be computed until the merge.
3. **Where a key is allowed to be wrong.** A key collision that shares
   a style between two elements that should differ is a correctness
   bug, not a performance one. The review fixes what makes a collision
   impossible rather than unlikely, and ST-4 is the test.

**Counterpart:** none directly; constrained by item 1 and item 5.

**Open before the review:** rule-index key statistics (a named ST-M0
spike).

---

## Item 3: the invalidation index

**What §6.8 fixes today.** An index from features — "id, class,
attribute name, state bit, `:has` argument feature, custom-element
state, `:dir`/`:lang` inputs" — to where that feature occurs in a
selector: the subject compound, an ancestor compound with its
combinator path, a previous-sibling compound, or inside a `:has()`
argument.

**What the review must settle.**

1. **The precision/cost line per occurrence kind.** §6.8's table gives
   a precision step per kind; the review fixes which are worth their
   bookkeeping, measured, not assumed.
2. **The state-source table.** The M0 row names it as a deliverable of
   its own, and §6.7.1 says "the table of sources, with each writer and
   B1 input, is an ST-M0 deliverable". A draft is in
   [state-sources.md](state-sources.md); the review accepts or amends
   it. Until it is accepted, the index's state-bit features have no
   agreed set of writers, so every one of them is a possible
   determinism hole.
3. **The index's own identity.** It is "per sheet, memoized, and merged
   per scope", so it is derived data under FP-D16 and must be
   rebuildable. The review checks that the rebuild is a pure function
   of the sheets.

**Counterpart:** none directly.

**Open before the review:** `Q-ST-15` (element-state usage).

---

## Item 4: the substitution interpreter

**What §6.9 fixes today.** `var()` and `env()` are substituted by an
interpreter, after the cascade and before the value parse.

**What the review must settle.**

1. **Where a custom property's value is stored and what the chain
   costs.** A custom property is inherited, so a `var()` reference
   reaches an arbitrary ancestor. That is the one place a computed
   value depends on something outside the element, the parent's
   inherited set and the environment.
2. **Cycle detection, and what a cycle computes to.** CSS says the
   declaration becomes invalid at computed-value time; the review fixes
   where that is detected, and whether detection is per element or per
   chain.
3. **How substitution interacts with lazy declaration parsing
   (ST-D19).** A substituted value is a token sequence, so the
   substituted result must be parsed; a declaration whose parse was
   deferred and which turns out to need substitution is parsed twice
   unless the review says otherwise.
4. **The interpreter's place in the parallel pass.** Substitution reads
   the parent's result, so it has the same ordering constraint as item
   2's key.

**Counterpart:** none directly; measured by `Q-ST-7` (the share of
declarations never used), because a deferred declaration that is
substituted is the case where laziness can lose.

---

## Item 5: container re-entry — **with the layout spec**

**This is a cross-track item.** The M0 row says "container re-entry
**with the layout spec**", and §6.10 says plainly:

> The API belongs to both specs and the layout spec owns the
> scheduling; ST-M2 tests re-entry against **`hg-layout-reference`**,
> the independent reference block layout with size containment.

**Counterpart track: LY (`reports/Izcal layout.md`), LY-M0's design
review of §6 as a whole.** LY-M0's own row names re-entry as agreed at
*this* review; the two reviews must not settle it differently, which is
the reason it is on both agendas. The LY side's clause is quoted in
that review's agenda.

**What is already agreed, from the layout spec's §9.5 row (applied in
revision 0.3.7):**

> container re-entry runs at a scheduler barrier with no layout worker,
> once per (container, distinct queried size) per pass with a
> pass-local overlay

**What the two reviews must settle together.**

1. **Who calls whom, and with what.** §6.10's entry point is
   `style_container_subtree(container, size)`, returning what changed.
   The review fixes the type of "what changed" — the "affects" mask's
   five categories plus the scroll-anchoring flag, per the layout
   spec's row.
2. **The bound on the loop.** "Each container is re-entered once per
   (container, distinct queried size) per pass, each size producing a
   style version in a pass-local overlay of which only the final size's
   version is published, so the loop is bounded structurally." The
   review checks that the bound holds when containers nest.
3. **What the `:has()` memo does across a re-entry.** §6.7.2 says the
   memo "lives for one pass because the tree is fixed only during a
   pass". Re-entry is within a pass but after layout has run. The two
   reviews decide whether the memo survives it; the tree is unchanged,
   so the answer is probably yes, and "probably" is not a decision.
4. **Scroll-state queries' input.** They "read the snapshot taken right
   after the ResizeObserver loop, which is a B1-recorded input". The
   layout spec owns that step.

**Open before the review:** nothing of ST's; LY-M0's §6 review must be
open, which is Phase 3's work.

---

## Item 6: the `:has()` engine

**What §6.7.2 fixes today.** Four mechanisms: a per-pass memo of
(anchor, argument); a per-argument choice of direction, made from
counts the rule index already has; bottom-up subtree summaries using
the ancestor filter's hashes; and a work budget per pass with a
recorded fallback.

**What the review must settle.**

1. **The direction rule's inputs.** "The choice is made per argument
   per pass from counts the rule index already has, so it is
   deterministic." The review fixes exactly which counts, because a
   count that changes with thread count would make the choice
   non-deterministic and ST-4 would catch it as a difference with no
   obvious cause.
2. **The budget, and what exceeding it costs.** The fallback is "the
   plain downward search for that argument, recorded as a cause". The
   review fixes the budget's unit and whether it is per pass, per
   anchor or per argument.
3. **The summary's upkeep.** It is "computed only for subtrees under
   elements that the last match flagged as possible anchors" — a
   dependency on the *previous* pass, which is state carried between
   passes and therefore derived data that must be rebuildable.
4. **The interaction with item 3.** The invalidation index has a
   `:has()`-argument occurrence kind whose walk "visits only elements
   whose last match flagged them as possible anchors" — the same flag.
   One mechanism, two readers, and they must agree on when it is
   written.

**Open before the review:** the `:has()`-heavy corpus measurement
ST-D11's trigger names.

---

## Also on the agenda: named by the M0 row, not by the "together" sentence

These are not part of the six, and they are listed because the M0 row
requires them and the review is where they are accepted.

| Item | Clause | Counterpart |
|---|---|---|
| **The state-source table** | "The state-source table (Section 6.7.1)" | — (draft: [state-sources.md](state-sources.md)) |
| **The computed-length numeric type** | "the computed-length numeric type fixed (Section 6.9)" — already fixed as binary64 by the SVG spec's R5; the review records it as fixed and checks the conversions | **LY**: `hg-layout-num` is the same type, "so it is designed once" (M1-X1 Phase 3); **TX**: every conversion into the text stack's integer contract uses TX-D4's round-half-to-even rule |
| **ST-15** | "`hg-encoding` and `hg-url` meeting ST-15, which the networking spec's NW-M1 (9–12 months) needs" | **NW** (`reports/Izcal networking.md`), NW-M1. **Met as of 13 November 2026**: WPT `url/` is ST-15's oracle (§6.5) and `hg-wpt-url` reports 904/904 on `urltestdata.json`, 87/87 on `toascii.json` and 16/16 on `percent-encoding.json`, in CI on pinned data. |
| **The provenance review** | "A **provenance review** of Section 6 against 6.18 by a reviewer who has not read the engine source files the notes once cited" | — ; the independence requirement is a precondition on *who*, not on *what*, and is recorded here so it cannot be met by the author |
| **The spikes** | `Q-ST-5`, `Q-ST-6`, `Q-ST-7`, rule-index key statistics, `Q-ST-15`, tokenizer throughput, ST-34, `Q-ST-3`, `Q-ST-4` | — |

---

## The interlock, in one place

Every clause of this track's M0 that names another track, with the
direction stated.

| This track owes | To | Clause |
|---|---|---|
| The style-metrics query's shape | **TX** | TX-M0 owes the *freeze* of the metrics query with ST; §6.9 fixes that the query returns **exact ratios** so quantization never enters computed values ([P17]) |
| `hg-url` and `hg-encoding` meeting ST-15 | **NW** | "which the networking spec's NW-M1 (9–12 months) needs" |
| The "affects" mask's categories | **LY**, **GR** | LY reads it to know what a change dirties; GR's §9.5 row adds the paint-only category feeding chunk invalidation |
| The computed-value colour contract | **GR/MD** | MD-D28: computed colours stay f64 through `hg-math`; `hg-colour` is the one pixel path with one documented hand-off |

| This track is owed | By | Clause |
|---|---|---|
| Container re-entry's scheduling | **LY** | "the layout spec owns the scheduling" (§6.10) |
| `hg-layout-reference` | **LY** | ST-M2 tests re-entry against it; LY-M1, by month 21† |
| The font-metrics query, live | **TX** | TX-M0; [P17] |
| The registrable domain (`hg-psl`) | **NW** | §6.5: `hg-url`'s registrable domain comes from the networking spec's `hg-psl` |
| The reference block layout's size containment | **LY** | LY-M1 |

---

## What is *not* on this agenda

The M0 row does not name them, and putting them here would make the
review's completion depend on work it does not own:

- the CSSOM's object model (`hg-cssom`, L3);
- the speculative parser's heuristics beyond its private atom scope;
- nesting's back-track, which §6.6 fixes and no clause reopens;
- the UA sheet's contents.
