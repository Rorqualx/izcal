# TX-M0 design review: the agenda

Opened 13 November 2026, for the fonts and text track's M0 milestone
(`reports/Izcal fonts and text.md` §9.2). **Opening the review with its
agenda fixed is M0's *start*; completing it is M0's exit** — and
completing this one has a precondition the project cannot meet today,
which Section 5 states plainly rather than leaving to be discovered.

TX-M0's row is unlike the other four M0 rows in two ways, and both
shape this agenda:

- **It names no "design review of Section 6".** What it names is a
  **measured-velocity review at month 6** and two **freezes** — the
  layout contract (§6.8) "frozen with the layout spec", and the style
  metrics query (§6.8a) frozen with the style spec. The freezes are
  this review's substance; the velocity review is a separate
  instrument with its own trigger, and Section 6 keeps it separate.
- **It leads the five tracks.** Its own row puts "normalization and
  the IDNA data **first, by month 6**, for `hg-url` at the style
  spec's ST-M0", and TX-1 — a ship blocker — demands 100% of the nine
  Unicode conformance files from TX-M0. So ST-M0 waits on part of
  TX-M0, and nothing in this agenda may be reordered behind it.

---

## 1. What is already built, and therefore reviewed rather than designed

These landed before the review opened, so the review's job for each is
to find what is wrong with it, not to decide it from nothing.

| Built | Section | What the review checks |
|---|---|---|
| `.hgfont` and `hg-fontgen` | §6.2, G29 | that the description language can express every table TX-M0 names, and that the generator is still the only writer of its outputs |
| The readers (`hg-fontfmt`) | §6.2, TX-11 | that a Kani harness exists per accessor, and that `PROOF_BOUND` does not hide a case a larger slice would reach |
| The writers (`hg-fontc`) | §6.15, TX-D24 | that every output round-trips through the proved reader, and that the byte expectations are transcribed from the prose rather than from the description |
| `hg-ucd` and `hg-unicode` | §6.6, TX-D11 | the pinned Unicode version, and that the generated tables are built rather than committed |
| Normalization and the UTS 46 data | §6.6 | **the item ST-M0 is waiting on** — see §4.1 |

Two of the five are new since the milestone was written: `hg-fontc`'s
writers, and the `hg-url` host parsing that consumes the IDNA data.
Both are evidence for clauses of "start", and neither is evidence that
TX-M0 can *exit*.

---

## 2. The central items: the two freezes

A freeze is not a design decision this track may take alone. Each is a
contract with a named counterpart, and the review's job is to put it in
a form the counterpart can sign.

### 2.1 The layout contract (§6.8) — **with the layout spec**

> the layout contract (6.8) frozen with the layout spec

What has to be fixed, item by item:

1. **The numeric contract (TX-D4).** Font size quantized "to the finer
   of 1/64 of a unit and 2⁻¹⁶ em, with round-half-to-even"; advances
   and metrics scaled as `round_half_even(value_fu × size_q /
   unitsPerEm)` in 64-bit integers, once per value. The layout spec's
   LY-D3 fixes `LayoutUnit` at 1/64 CSS px, and LY-9 converts
   cumulative positions **once**. The review has to state, in one
   place, where that single conversion happens and which side owns the
   rounding — because two roundings on the same path is the drift
   LY-D3 exists to prevent.
2. **The shaped-segment type.** What crosses the boundary: cumulative
   positions in TX-D4's quantum, the cluster map, and the three flags
   `break_reshapes`, `join_reshapes` and `kashida_point`. These are
   Izcal's own names (§7, M-1 part 3) and freezing them freezes the
   vocabulary the layout spec's §6.5.2 is written against.
3. **Who decides a break.** §6.8: the helper "provides break
   opportunities with cumulative positions"; "the **layout spec
   decides** which opportunity fits". The review records that
   division, so neither side later reshapes on the other's behalf.
4. **Reshaping on demand.** Only an edge marked `break_reshapes` is
   reshaped at a break. The CI check that reshapes every line from
   scratch and compares (B5) is the freeze's evidence, and belongs in
   the contract rather than beside it.

**Counterpart:** the layout spec's LY-M0 review, §4.2 of its agenda
("The text interface — with the fonts and text spec"). Neither review
can complete this item alone.

### 2.2 The style metrics query (§6.8a) — **with the style spec**

> the style metrics query and the layout contract frozen, and the
> metrics query implemented over the faces that exist (6.8a), which the
> style spec's ST-M1 needs

What has to be fixed:

1. **The query's shape.** For (font list, size, style, language), for
   the first available font: the metrics `ex`, `ch`, `ic`, `cap`, `lh`
   and `font-size-adjust` need, "each returned as an **exact ratio**"
   — the value in font units over `unitsPerEm`, with `size-adjust` and
   the metric overrides applied as exact factors.
2. **Why a ratio and not a length.** Style multiplies the ratio by the
   computed font size, "so the quantization of 6.8 stays a used-value
   rule and never enters computed values". The review has to check
   that nothing in the query's signature makes it possible to return a
   quantized length by accident, because a single such return turns a
   used-value rule into a computed-value one across the whole cascade.
3. **What it answers over today.** "implemented at TX-M0 over the
   faces that exist then (the OS tier, the interim tier and the
   last-resort face, with Izcal faces joining as their waves ship)".
   The last-resort face is TX-W1's and is not drawn; the OS tier needs
   the manifests of §3.3. **So this clause is downstream of §5's
   block, and the review states which half can be signed without it:
   the query's shape and its ratio rule can; the implementation over
   Izcal faces cannot.**
4. **Mutation logging.** "A web-font installation that changes a
   result is a mutation-log entry for the elements that used it (B4)."
   That is an obligation on the style spec's invalidation, not on this
   one, and the freeze names it.

**Counterpart:** the style spec's ST-M0 review. Its own agenda's
numeric item (ST §6.9's computed-length type) is the other end of the
same contract.

---

## 3. Items the review must decide, in its own right

| # | Item | Section | Why it cannot wait |
|---|---|---|---|
| 3.1 | The shaping specification's **engine model**, and the default, Arabic-type and USE engines drafted | §6.7, TX-D8 | every script engine after TX-M0 is compiled against this model; changing it later recompiles all of them |
| 3.2 | The **fuel model** implemented | §6.3, TX-D10 | the placeholders (4,096† lookup applications per code point, 64† nesting) are argued from first principles and set from real fonts at TX-M1 (Q-TX-6); the *model* must exist first or there is nothing to set |
| 3.3 | The **OS-family manifests** for macOS and Windows, the interim manifests and the build-time session-image part | §6.4, TX-D22, TX-D25 | Q-TX-12 and Q-TX-17 are per-manifest and due at TX-M0; the interim tier's entropy (TX-22) cannot be predicted without them |
| 3.4 | **Both rasterizer families spiked** | §6.9, TX-D5 | Q-TX-3 chooses between dense accumulation and sparse strips; a spike that measures only one measures nothing |
| 3.5 | **TX-D9's oracle harness, if allowed** | §6.2 | the "if allowed" is a licence question under OD-L2, and the review either settles it or records why it could not |
| 3.6 | The **ISO/IEC 14496-22:2026 text**, bought at TX-M0 | §9.2 | Q-TX-2 asks what in it is absent from the published 1.9.1 pages; avar2 is built with the other variation tables and depends on the answer |

---

## 4. Cross-track items, each with its counterpart named

### 4.1 Normalization and the IDNA data — **with the style spec**

> normalization and the IDNA data first, by month 6, for `hg-url` at
> the style spec's ST-M0

This is the one cross-track item that is **already discharged in the
tree**: `hg-ucd` and `hg-unicode` carry Unicode 18.0's normalization
and the UTS 46 mapping table, and `hg-url` is built on them. What the
review still owes is the *direction* of the obligation: TX owns the
data and the generator, ST owns nothing of it, and a Unicode version
bump is TX's change with an ST consequence, never the reverse.

### 4.2 The text interface — **with the layout spec**

Section 2.1 is this item seen from TX's side. The layout spec's R22
and its LY-M0 agenda §4.2 are the same item seen from LY's. The review
records the pair, so that completing one without the other is visibly
incomplete.

### 4.3 The shell's UI face — **with the shell spec**

Q-TX-8 is answered: "the shell uses the OS tier's UI face on macOS and
Windows and the Izcal UI face on Linux; from 0.3.3 that is the sans
alias face, with the interim tier for scripts it does not cover". The
alias face is TX-W1's, so the shell's Linux appearance is downstream of
§5's block. The review records that rather than leaving the shell spec
to discover it.

### 4.4 IME placement — **with the editing and accessibility specs**

§6.8a, as amended 20 October 2026 (ED-D5 (b)): on Windows, TSF and its
text store run in the window-owning UI host of AX-D9 (b); "**No IME
component is loaded into a renderer.**" The broker-side relay and the
caret and selection movement rules are **counted in the editing spec**
(§9.3's counting rule). This track keeps composition rendering and the
segmentation and bidi data. The review's job is to check that the
split as implemented matches the split as counted, because a boundary
that moved after the counting rule was applied moves effort with it.

---

## 5. What blocks this review's completion

> the type-design program staffed with its leads, its wave plan
> (TX-W1 to TX-W4) published with a **planned bill of materials**
> (TX-51, TX-D24), the alias reference widths measured (TX-8) and the
> last-resort face generated

**The type-design program does not exist.** TX-D24 sizes it at "about
45† type staff" plus native-script reviewers, and Stage 1 §13.1 records
**D-S8** — funding — as "Deferred by the owner (14 October 2026)",
with "**Stage 1 (M1 onward) cannot be staffed until D-S8 is
decided**".

This is not a reason to leave the review unopened — the M1-X1
definition of "start" is opening it with the agenda fixed, and that is
done. It is a reason the review **cannot be completed**. Four of the
row's clauses depend on it, and they are not independent of one
another:

| Clause | Needs | Also blocks |
|---|---|---|
| The program **staffed with its leads** | D-S8 | every clause below |
| The **wave plan** TX-W1 to TX-W4 with a planned bill of materials (Q-TX-1) | leads, to set it from measured velocity | the TX-M1 and TX-M2 reviews, which re-derive the order |
| The **alias reference widths** measured (TX-8) | a decision on Q-TX-15's open first half | TX-W1's aliases, and so TX-D19's name-resolution order |
| The **last-resort face** generated | designers | §2.2's metrics query over the faces that exist; the shell's Linux UI face (§4.3) |

Three things follow, and the review records all three:

1. **What is Izcal's to write can be written now.** The wave *order*
   is this spec's own recommendation ("readers moved off the interim
   tier per designer-year"), the widths *contract* is TX-8, and the
   last-resort face's *design rule* is in TX-W1 ("synthesized hex
   digits and frame"). What needs staff is the execution, not the
   statement.
2. **Q-TX-15's first half is an owner question, not a staffing one.**
   "Is measuring advance widths and vertical metrics from vendors'
   installed reference faces … an acceptable basis for the aliases?"
   It can be answered today and should not wait behind D-S8; the
   review puts it to the owner separately.
3. **The engineering clauses do not wait.** §§1–4 and §3's six items
   can be completed without a type designer, and doing so is what
   keeps the block to four clauses rather than to the milestone.

---

## 6. The measured-velocity review at month 6

TX-M0's parenthesis — "(with a measured-velocity review at month 6)" —
is a separate instrument, and §9.3 says what it measures:

> Velocity measures: Unicode conformance and WPT subtests passing per
> engineer-month, script engines through TX-3 per quarter, fuzz-clean
> tables per month.

It has a stated trigger in the risk table, and the trigger is about
Stage 1 rather than about this track:

| Risk | Trigger |
|---|---|
| Text on Stage 1's critical path (M4 and M8 depend on TX-M1 and TX-M2) | "TX-M0's review shows TX-M1 cannot precede M4" |
| Hiring type designers | "Type-design track below 70%† of plan for two quarters, or a wave's velocity below half the plan at a milestone review" |

Two things the review has to settle about the instrument itself:

- **A velocity measure needs a denominator.** With no staff, every
  measure above is zero over zero. The review records that the month-6
  instrument cannot run before D-S8 either, and that the *duration*
  figures it would replace ("TX-M0 at 9 months … placeholders,
  replaced from measured velocity at each review") therefore stay
  placeholders.
- **What it would do to the other specs.** Its trigger fires a
  re-dating of Stage 1's M4, which is a Stage 1 amendment, not a note
  in this track. The review names that path in advance.

---

## 7. Provenance

**TX-M0's row does not name a provenance review.** The style and
layout M0 rows do; this one does not, and the agenda does not invent
one. What this track has instead is §7's standing record and the
source-evidence pass of revision 0.3.1, which removed or replaced 41
links and source-derived claims.

The review's provenance job is therefore narrow and specific, and §7
of the spec already names the places where the design resembles
existing systems and says why each is first-principles here: linear
advances, vertical-only snapping, grayscale AA, a per-word shaping
cache with break-safety flags, table-driven script engines. The review
checks that list is still complete after the work of §1 — in
particular that `.hgfont` and `hg-fontc`'s writers, which are new,
carry nothing taken from any font library's own description format,
and that TX-D10's fuel placeholders are still argued rather than
sourced.

---

## 8. The interlock, in one place

| This track owes | To | Clause |
|---|---|---|
| Normalization and the UTS 46 data | **ST** | "first, by month 6, for `hg-url` at the style spec's ST-M0" |
| The metrics query, as exact ratios | **ST** | §6.8a; "which the style spec's ST-M1 needs" |
| Shaped segments and the three flags | **LY** | §6.8; `break_reshapes`, `join_reshapes`, `kashida_point` |
| Break opportunities with cumulative positions | **LY** | §6.8; LY decides which one fits (LY §6.5.2) |
| Composition rendering, segmentation and bidi data | **ED** | §6.8a as amended; the relay and movement rules are ED's |
| Per-glyph ink bounds; per-character bounds | **LY** (MathML), the semantic plane | §6.8 |
| The UI face, per OS | **SH** | Q-TX-8, answered; the Linux face is TX-W1's |

| This track is owed | By | Clause |
|---|---|---|
| One conversion into `LayoutUnit`, and only one | **LY** | LY-9, LY-D3; §2.1 item 1 |
| The computed-length type that multiplies the ratio | **ST** | ST §6.9; §2.2 item 2 |
| The broker-side IME relay and the movement rules | **ED** | §9.3's counting rule, applied 20 October 2026 |
| A **staffed type-design program with its leads** | **D-S8** | §5 |
| An answer to Q-TX-15's first half | the owner | §5 item 2 |

---

## 9. The measurements this review takes as input

TX-M0's own questions, from §11. Each is a **black-box** observation
(M-1 part 2), and none is done:

| Question | Due | What it feeds |
|---|---|---|
| Q-TX-1 | planned at TX-M0 | the font set's bill of materials, per wave — §5 |
| Q-TX-2 | TX-M0 | what ISO/IEC 14496-22:2026 adds beyond avar2 — §3.6 |
| Q-TX-5 | TX-M0 | which Apple system fonts are AAT-only; which Windows base fonts need instructions — §3.3 |
| Q-TX-9 | TX-M0 | whether the `SVG ` table's rules match §6.3's restricted glyph mode |
| Q-TX-12 | TX-M0, per manifest | which OS faces meet the OS-tier rule — §3.3 |
| Q-TX-16 | TX-M0 | how much of the current emoji version Fluent Emoji's MIT artwork covers — §5 |
| Q-TX-17 | TX-M0, per manifest | the metric classes per script, and so TX-22's predicted count — §3.3 |
| Q-TX-15 (first half) | before TX-W1 | whether alias widths may be measured from installed reference faces — §5 item 2 |

Three of these (Q-TX-5, Q-TX-12, Q-TX-17) are local surveys of the
supported OS releases and need no staff beyond a machine of each kind.
**They are the part of §5's block that can be lifted by work rather
than by funding**, and the review should order them first.
