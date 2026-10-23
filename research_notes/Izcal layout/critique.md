# Izcal layout: independent design critique (phase 8)

Written 19 October 2026 by a reviewer who did not write the spec. Subject: `reports/Izcal layout.md`, draft 0.1 (line numbers below refer to that draft). Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8, with `references/spec-template.md` and `references/izcal-capabilities.md`.

**Read against:** HGBA-2 2.3.2 (M-1 parts 1–4, P1–P6, I-1, Section 4.3, Section 4.8, targets P-1 and ANC-1); HGBA-3 (B1, B3, B4, B5, B6, B7, B8, B9, B10, DET-1, DET-2, INC-1 to INC-4, FLUSH-1, PRIV-2, A11Y-1); Stage 1 design (2.1, 2.2 G1–G9, 3.3, 3.4, 5.6–5.8, 8.1–8.3, 13.2); JavaScript engine spec 0.3 (JS-D4, JS-D15, JS-D19, G13–G18, 6.5, 6.9); resource-footprint spec 0.3 (FP-2, FP-11, FP-12, FP-24, FP-34, FP-D16, Sections 6.4 and 8.2); HTML and CSS parsing and style spec 0.3.3 (6.10, 6.11, 6.12, 6.14, 6.15 with L2s/L2p and G19–G24, 9.5, ST-D7, ST-D13, ST-D18, ST-34); fonts and text spec 0.3.3 (6.8, TX-D4, G25–G29, 9.5 R22); networking spec 0.3.3 (6.3 priorities, 6.8 NW-D7, 6.21 G30–G36); DevTools spec 0.3 (DT-D3, DT-D4); Phase 2 reconciliation (P7, P9, P17–P20, P23) and licence audit (2.7, OD-L6); the seven research notes and `frame.md`. Quality bar: the JS and footprint specs and the style and text critiques.

**Owner direction applied as binding:** engine work is all Izcal's (no Taffy, Servo layout, Blitz, Yoga, Morphorm or any layout library; generic infrastructure only); MIT only for shipped code and data, with standards data, test-only inputs and OS interfaces and the toolchain as the exceptions; another engine's source is never evidence; no shortcuts for time or complexity, staging only for correctness, security or measurement; accepted decisions hold (JS-D4, JS-D15, FP §8.2 and FP-D16, ST-D7, ST-D13, TX-D4); this spec's G-rules start at G37.

**Phase artifacts.** Phase 1 (`frame.md`) exists and is good. Phases 2–3 are in seven notes plus Sections 2–3; phase 4 (Section 4, including the keep/break table), phase 5 (Section 5) and phases 6–7 (Sections 6–11) are present. Phase 8 had not run (header says draft 0.1); this critique is half of it.

**Overall.** The core contract is right and well argued: a pure `layout(box, space, token)` over a separate box tree, fragmentation in the contract from LY-M0, integer geometry, an independent reference engine, primary layout state split from class (a) data, no script during layout (G37). The weak points are where layout meets something it does not own or that changes during the pass: the style re-entry under parallelism and at several sizes, the text stack's numeric contract, the identity of fragments across incremental, parallel and discarded runs, and the graphics spec that hit testing, IntersectionObserver v2 and the 160 B paint budget depend on. Five findings are High.

**Counts:** 30 findings: **5 High, 15 Medium, 10 Low**, plus 4 engine-source and licence items (E1–E4) and 14 reconciliation items.

---

## High

### C1. High — Container re-entry from a parallel layout worker writes shared style and box-tree state in the middle of the phase

**Location:** 6.9 rule 1 and rule 3 (lines 318, 320); 6.12 (lines 365–372); G42, G43 (6.18); LY-4.

**Problem.** Rule 1 has layout call `style_container_subtree(container, size)` and "apply the returned box-tree changes to the container's subtree (a query can change `display`)". Rule 3 runs each re-entry "on the worker laying out the container, with style's own merge". Style's merge is the **only writer of the style columns and the per-agent intern tables** ([ST 6.11, 6.12], ST-D7), and it publishes to state every other layout worker is reading. The box-tree change writes `hg-box-tree`'s columns, which 6.12 lists as read-only to workers. So one worker's re-entry mutates the inputs of the others in the middle of a fork/join phase. That is exactly what phase purity forbids (Stage 1 3.3: "workers read immutable inputs and write disjoint outputs"; HGBA-3 B3). G43 governs only `hg-layout-state` and the fragment store, so the spec's own rules do not catch it. LY-4 (byte identity for 1, 2, 4, 8 workers, "including pages with re-entry") cannot be argued from the design as written: style ids assigned by a merge that runs while sibling workers intern their own re-entries depend on which merge ran first.

**Evidence.** Stage 1 3.3 (phase purity; a shared mutable cache inside a phase "breaks it"); HGBA-3 B3 (determinism "must come from phase purity"); ST 6.10 ("its own deterministic merge before layout reads the subtree's styles"); ST-D7 (a) (final ids assigned by a merge that only publishes); JS-D4 (workers never write shared columns).

**Fix.** Make re-entry a **phase boundary**, not a call inside a worker. Replace rule 3 with: "A layout task that reaches a size container whose subtree is pending returns `NeedsStyle(container, size)` to the scheduler and stops that subtree. When no runnable layout task remains, the scheduler runs every pending re-entry of the frontier, in box-tree order, as one style phase (style's own parallel traversal and merge, with no layout worker running), then applies the box-tree changes sequentially, then resumes layout. Re-entry output is published only at that barrier." Add G44: "No layout task can call `hg-style`'s write API or `hg-box-tree`'s rebuild API; only the sequential scheduler step can." State the cost (one barrier per re-entry wave; measured at LY-M2 as a counter) and add re-entry pages with 1, 2, 4, 8 workers to LY-4's corpus by name. Amend ST 6.10 to match (reconciliation item R5).

### C2. High — Re-entry at several sizes in one pass has no sound semantics: single-valued style and box state, and a history-dependent exhaustion rule

**Location:** 6.9 rule 2 (line 319), rule 5 (line 322); 6.4 caching (line 229); LY-D12; LY-3; LY-6.

**Problem.** Rule 2 lets one container re-enter style at up to 4† distinct sizes per pass, and amends the style spec's "at most once per layout pass". Three problems follow.
1. **One copy of the answer.** Style columns hold one computed-style id per node, and the box tree holds one structure. A measure layout at size A and the final layout at size B need two versions of the subtree's styles and, if a query flips `display`, two box structures. The spec does not say where version A lives while B is current, or what happens to cached measure results and fragments that reference box ids rebuilt away by the B re-entry.
2. **Exhaustion depends on history.** "Beyond the bound, measurement layouts reuse the styles of the most recent size." Which size is most recent depends on which layouts ran this pass, and an incremental pass with cache hits runs a different set than a from-scratch pass. Incremental and from-scratch then compute different measure results for the same inputs: the hysteresis that LY-D1 exists to remove, and an LY-3 failure by design.
3. **Cache key versus styles used.** The re-entry key in the constraint space is the queried size, but under exhaustion the layout used another size's styles. A cache entry keyed by S then holds a result computed with S′'s styles.

The amendment also removes the termination argument the style spec relies on ("each container is re-entered at most once per layout pass, so the loop is bounded"), and replaces it with a † bound whose exhaustion behaviour is unsound.

**Evidence.** ST 6.10 (once per pass; scheduling owned by layout); LY-D1's rationale ([IP 2.4, 7.9]: layout must be a function of current inputs only); LY-3 (0 byte differences incremental against from-scratch); LY-D12 (amends ST).

**Fix.** (a) Replace the 4† bound with a **structural bound derived per formatting context**: the number of distinct constraint spaces a context can give one child in a pass (block 1, flex 2, grid ≤ 2 per sizing round with the rounds bounded by Grid 1 §11, table 2, multicol balancing ≤ 3), so re-entries per (container, pass) are bounded by the product along the ancestor chain, which LY-23 already bounds. No exhaustion rule is then needed. If a numeric cap stays for DoS, its overflow result must be a function of the inputs alone, for example "sizes beyond the cap use the styles of the container's final size, computed first", and LY-3 must test it. (b) Store re-entry results as **versions** keyed by (container, size) in a pass-local style overlay; box-tree changes from a non-final size are built in a task-local box buffer and never replace the shared box tree; only the final size's version is published. (c) Put the **style-version id**, not the size, in the constraint space's re-entry key. (d) Add LY-6 cases for a `display` flip between measure and final sizes and for cap overflow under incremental against from-scratch.

### C3. High — LY-D3 rests on a misreading of TX-D4, and the proposed text-spec amendment would break an accepted decision

**Location:** 6.2 "The text contract" (line 191); LY-D3; 9.5 row "Fonts and text spec 6.8, 9.5 (R22)"; Q-LY-9.

**Problem.** Line 191 says "TX-D4 quantizes advances and metrics to 1/64 of the local unit for HTML text in CSS px, so text widths enter layout exactly". TX-D4 says something else: the **font size** is quantized "to the finer of 1/64 of a unit and 2⁻¹⁶ em", advances are `round_half_even(value_fu × size_q / unitsPerEm)`, and "positions accumulate as integers in the same quantum". For any font size under 1,024 px, 2⁻¹⁶ em is finer than 1/64 px (at 16 px it is 1/4,096 px), so advances do **not** arrive in 1/64 CSS px. The 9.5 amendment ("advances still arrive in the 1/64 quantum of the local unit, and layout states this as the contract") would change the accepted TX-D4 and its fix for scaled SVG and canvas text (TX critique C19). It would also break TX-D4's equivalence argument that canvas `measureText` and layout widths reveal the same values: per-glyph rounding to 1/64 drifts by up to about 0.5/64 px per glyph, so a 100-glyph run can differ from `measureText` by most of a CSS px. One of LY-D3's five stated reasons is therefore wrong. The other four still hold, so the decision survives, but its interface to the text stack does not.

**Evidence.** TX 6.8 (numeric contract, quoted above; "the layout spec maps these values to its layout unit"); TX-D4 (accepted; reversal only for web-compatibility failures, toward finer quantization); reconciliation P17 (conversions into TX's contract use round-half-to-even); owner direction (TX-D4 is an accepted decision to honour).

**Fix.** Withdraw the 9.5 text amendment's numeric clause and Q-LY-9. Write the mapping in 6.2: "Layout keeps the text stack's cumulative positions in its quantum as i64 and converts **cumulative** offsets, never individual advances, to `LayoutUnit` with `to_layout_unit` (round half to even), at item and line boundaries. A run's width is `to_layout_unit(end) − to_layout_unit(start)`, so line widths never drift and equal what `measureText` reports after the same rounding." Ask the text track for the opportunity-with-cumulative-width API in its quantum (the rest of the 9.5 row stands). Rewrite LY-D3's reason to "1/64 converts exactly to binary floating point and is fine enough for TX-D4's positions once converted cumulatively", and add a TX-6-style CI check that layout line widths equal the text stack's cumulative positions after conversion.

### C4. High — Fragment identity is undefined, so the byte-identity targets are ill-posed and primary state can point into discarded data

**Location:** 6.11 relayout (line 353), 6.12 determinism (line 372), 6.13 reclamation and classes (lines 378–380); LY-3, LY-4, LY-32, LY-40; 6.17 planted-bug suite.

**Problem.** Fragment ids are 32-bit store indices. Incremental relayout "keeps fragment ids" of unchanged subtrees and allocates new ones on the spine; reclamation runs when canonical garbage passes a threshold. A from-scratch layout allocates every id fresh. Ids and store layout therefore depend on history, yet:
- LY-3 compares incremental with from-scratch "byte for byte" in the fragment tree, and LY-4 requires identical "ids and canonical bytes". The first cannot hold if ids are in the bytes; the second holds only for equal histories. No canonical, id-free serialization is defined.
- The reclamation roots include "`hg-layout-state` references", so **primary** state holds fragment ids. FP-12 discards class (a) stores and rebuilds them with new ids, leaving primary state with dangling or wrong ids. LY-32 says it retains `hg-layout-state` across the discard, which is exactly the broken case.
- FP 6.4 requires handles that cross threads or are stored by the semantic plane to be **generational**. Fragment ids have no generation, yet `hg-semantic`, `hg-hit` caches, DevTools and the decision-record replays hold them across passes. The planted-bug case "a fragment id used after reclamation", which 6.17 says is caught by "generation checks", has no generation to check.

**Evidence.** FP 6.4 (narrow references; generational handles); FP-12 (bit-identical rebuild of class (a) at random points); FP-24 (canonical bytes deterministic); HGBA-3 DET-1 (fragment-tree hash); JS spec release-by-ID discipline.

**Fix.** (a) Define the **canonical fragment tree** in 6.13: a pre-order serialization of records with child order, geometry, side-table contents and break tokens inlined, no ids and no store offsets. DET-1, LY-3, LY-32 and LY-40 hash that. LY-4 can additionally compare raw store bytes, because equal histories are its premise. (b) Rule G45: "Primary state never stores a fragment id; it stores node slots or box-independent values (offsets, sizes)". Remove `hg-layout-state` from the reclamation roots. (c) Give the store a 32-bit epoch that reclamation increments; any fragment handle held outside one pass is (id, epoch) and is checked on use. Add the epoch to CS-PF-1's size asserts. (d) State in FP-24 terms that store canonical bytes are counted as **live** records only, so the footprint figure is history-independent.

### C5. High — Hit testing, IntersectionObserver v2 and cross-process frames rest on a graphics contract that no spec owns, under ship-blocker targets

**Location:** 6.14 (line 388); 6.15 "Out-of-process frames" (line 396); G38; LY-D14; LY-7, LY-15; Risk "Graphics spec not written"; Q-LY-17.

**Problem.** LY-D14 makes `hg-hit` walk "the graphics spec's paint-chunk order", and IntersectionObserver v2 needs proof from "the fragment and paint trees" that a target is unoccluded, unfiltered and fully opaque. Both are owned here, with ship-blocker targets (LY-7, LY-15), yet:
1. **Crate graph.** G38 says consumers depend only on `hg-fragment`; `hg-hit` and IO v2 need stacking order, property trees (transforms, clips, effects, filters) and ink overflow, which are paint's. The dependency of an L4 layout crate on paint's output is not stated, and Stage 1 2.1 has no paint read API.
2. **Cross-process hit testing.** "Cross-process frames are hit-tested by the browser process on composited surfaces as the graphics spec defines". The browser process can only do that with per-surface hit-test data that renderers send (regions, transforms, `pointer-events`, a "needs exact test" flag). The spec does not define that message, its recording (JS-D15) or its validation (I-2: the browser treats renderer data as untrusted).
3. **IO v2 across processes.** The child's visibility needs the parent's occlusion proof. "Sent as a flag with the intersection" (Q-LY-17) is a placeholder. Whether the bit leaks the parent's layout to a cross-site child is a P1 question, and no target covers it.
4. **Timing.** The risk trigger "No graphics spec by LY-M0's review: the owner names an owner" comes after the design review that freezes the contract.

**Evidence.** Stage 1 5.7 (paint owns property trees and chunks); HGBA-2 I-1 and I-2 (out-of-process frames; zero ambient authority); IntersectionObserver ED §3.2.5–3.2.6 and the "no false positives" rule ([SR 8.1–8.3]); JS-D15.

**Fix.** (a) Define in this spec, now, the **hit-test contract** that graphics must satisfy: a read-only `PaintOrder` trait (stacking-context order, chunk-to-fragment map, property-tree lookups) in an L4 contract crate owned jointly, with G38 amended to allow `hg-hit` and `hg-layout-query` to depend on it. (b) Specify the renderer-to-browser **hit-test region message** in 6.14 (fields, recording, the browser-side validation rule), and the parent-to-child **visibility proof** for IO v2 as one recorded bit per frame per frame-boundary update, with an LY-14 probe showing the child learns nothing beyond what IO v2's definition already exposes. (c) Make LY-7, LY-15 and IO v2 conditional on the graphics spec's acceptance, and move the "name an owner" trigger to **before** LY-M0's design review. (d) Add LY-D14's missing reversal trigger: "if the graphics spec's paint order cannot be walked at hit-test latency (p99 > 1 ms† on the corpus), a hit-test index derived from it, checked by LY-7".

---

## Medium

### C6. Medium — The on-demand layout stack duplicates JS-D19's engine-managed stacks, and the depth limit is not derived

**Location:** 6.17 "Depth and fuel" (line 412); G40; Section 7 dependencies; LY-D15; Q-LY-18, Q-LY-1.

**Problem.** (a) The JS spec already owns a stack mechanism: **engine-managed stack segments**, guard-paged, reserved through the substrate's `Memory` interface, with per-segment shadow stacks for CET and Arm GCS, and a **canonical frame budget** deciding overflow (JS-D19, JS 6.9). A second stack-switching mechanism in the same process (`stacker`/`psm`) has to coexist with those shadow stacks, with the crash handler's and sampling profiler's unwinding, and with sanitizers. The spec does not discuss any of these. (b) "`psm` contains assembly, which is a dependency's internal matter, not an Izcal inventory entry" decides an M-1 part 1 classification that HGBA-2 reserves for the owner ("a dependency whose class is unclear is decided by the owner ... the default is to build it"). Q-LY-18 asks the right question, but Section 7 has already answered it. (c) The style spec, accepted, takes the other route: no recursion on content depth on any native stack, with explicit work stacks (ST 6.14, G21). LY-D15 does not list that option. (d) 65,536† is justified as "a memory attack" bound, but stack memory per level (frame size × frames per box level across block → inline → atomic inline → flex) is not measured. At 1 KiB per frame and three frames per level, 65,536 levels is about 192 MiB of stack, more than half of FP-5's p90 page.

**Fix.** Add LY-D15 option (d) "explicit work stacks, as ST's G21" and option (e) "the substrate's JS-D19 segments, shared with the engine, inventoried once". Recommend (e) if layout stays recursive, because one audited mechanism serves both. Remove `stacker`/`psm` from Section 7 until the owner classifies them. Replace the box-depth limit with a **canonical stack budget** in bytes (each recursive layout entry charges a fixed size, as JS's frame budget does), set from a measurement at LY-M0. Keep the deviation record and Q-LY-1, and state what the limit does to geometry APIs (`getBoundingClientRect` of an ungenerated box, `checkVisibility`).

### C7. Medium — The layout cache cannot implement its own central diff, ignores break tokens, and conflicts with FP 6.4's cache rule

**Location:** 6.4 "Caching" (line 229); 6.7; 6.13 budget.

**Problem.** (a) The slot stores a "constraint-space **digest**", but `relayout_needed(old_space, new_space, box_flags)` compares fields ("a percentage-sized box relays out only when the percentage basis changed"). A digest supports only equality, so either the full old space is stored (at roughly 60–80 B with exclusion-space and fragmentainer references, absent from the budget) or the diff degrades to equality. (b) The incoming break token is not in the key. A box resumed in fragmentainer 2 with the same space as fragmentainer 1 would hit fragment 1's entry. (c) Two entries per box cannot hold a box fragmented across N fragmentainers. (d) FP 6.4 states "one fragment per box per constraint space" and eviction of "offscreen, unchanged subtrees first". The spec has no eviction policy, and any eviction must be deterministic (FP-24) and invisible (LY-3). No amendment is listed.

**Fix.** Key entries by (interned constraint-space id, break-token id). Intern constraint spaces per pass, since siblings usually share one. Store the interned space and let the diff read it. State the multi-fragment caching rule (for example, cache only the first fragment's result, and re-run from break tokens for the rest, checked by LY-3). Add an eviction rule driven by canonical counters only, or state that there is none, and amend FP 6.4's sentence.

### C8. Medium — The 200/160 B split is not plausible as derived, and paint's share is set without its owner

**Location:** 6.13 "Budget" (line 382); LY-30; 9.5 rows for footprint and graphics.

**Problem.** The 200 B ledger counts cache entries "only for boxes that were measured (amortized 48 B†)". The incremental design needs a cached result for **every** box laid out, since a clean box returns "its cached result". With C7's stored space, each box carries roughly 40 B of `LayoutResult` (fragment id, baselines, margin strut, out-of-flow list head, status) plus the space. Paint's 160 B† is fixed by this spec for a graphics spec that does not exist. Property trees, display items for backgrounds, borders and text, chunk metadata and the chunk hashes FP 6.4 keeps after acknowledgement can exceed 160 B on box-heavy pages. The arithmetic adds up (32 + 41 + 48 + 24 + 24 + 8 + 23 = 200), but its inputs are guesses with no sensitivity range.

**Fix.** Recompute the ledger with a cache entry per laid-out box and interned spaces (C7), and show a low, expected and high column. Keep 360 B† as the joint line and make the 200/160 split **provisional until the graphics spec co-signs it** and Q-LY-15 has measured the fragment-to-box ratio at LY-M0. State the failure order: layout representation work first, then a joint re-split, never FP-11 itself.

### C9. Medium — Scroll ownership with an asynchronous compositor lacks a reconciliation protocol

**Location:** 6.10 "Who owns scroll offsets" (line 343), scroll anchoring (line 345), snap; LY-D13.

**Problem.** Compositor offsets arrive "as recorded inputs applied at the frame's scroll step". The spec does not say whether they are absolute offsets or deltas. Main-thread changes (scroll anchoring adjustments, `scrollTo`, re-snap after layout, `scrollIntoView`, anchor-positioning shifts) race with in-flight compositor scrolling: an absolute protocol loses either the user's scroll or the anchoring adjustment. Snap ownership during a gesture (compositor) against after a layout change (main thread) is not divided, and the timing of `scrollend` relative to the scroll step is not stated. Sticky "moves on the compositor with no relayout", but `getBoundingClientRect` on a sticky box must still use the main thread's recorded offset; that is implied, not written.

**Fix.** Specify a **delta protocol with sequence numbers**: the compositor sends (scroll node, delta, gesture phase, sequence); the main thread applies deltas at the scroll step and sends back its own adjustments as deltas tagged with the last sequence it consumed; both are recorded. Snap: the compositor snaps during gestures, the main thread re-snaps after layout and sends the result as a programmatic scroll. Define `scrollend` as dispatched at the first scroll step after the gesture-end message. Give LY-D13 a reversal trigger (for example, anchoring adjustments lost on a scripted-scroll-during-fling suite).

### C10. Medium — Fixed scrollbar and form-control metrics leave zoom, WCAG, OS accessibility settings and localized controls undefined

**Location:** LY-D11 (line 655); 6.17 privacy (line 416); 6.3 form controls (line 212); 6.19.

**Problem.** (a) "15 CSS px† in every OS" does not say whether thickness scales with page zoom. In CSS px it grows to 60 screen px at 400% zoom, consuming viewport exactly where WCAG 1.4.10 Reflow is tested (320 CSS px). In device-independent px, the CSS-px value depends on zoom (deterministic, but then "identical geometry" holds per zoom level only). (b) `scrollbar-width: thin` and `none`, and `scrollbar-gutter: stable both-edges`, need Izcal values. (c) The overlay/classic choice is a shell preference "which is an environment input", so it is one bit of fingerprint (D-S1, PRIV-2) and differs from the OS default on macOS. OS accessibility settings such as Windows' "Always show scrollbars" and macOS's "Show scroll bars: Always" are not mapped, which is an accessibility regression for users who set them. (d) Form controls: the file input's label and button text are localized, so its intrinsic width depends on the UI locale (a fingerprint and a determinism input). `<select>` and date pickers open UI that the browser process renders. `appearance: none` must still devolve per CSS UI 4 and HTML's widget rules, and customizable `<select>` (`appearance: base-select`) is single-engine, so LY-D10 holds it.

**Fix.** Define thickness in device-independent px, converted to CSS px through the recorded zoom and scale (state the formula), with values for `thin` and `none`. Make the overlay/classic preference default to the OS accessibility setting, read once per session into the environment snapshot, count it in PRIV-2's entropy budget, and have the A11Y track review it. Add rows to 6.3: localized control strings come from the environment snapshot's UI locale (recorded); pickers are browser-process UI whose anchor rect comes from `hg-layout-query`; `appearance: base-select` is held by LY-D10. Add an LY-14 probe at 100% and 400% zoom.

### C11. Medium — Layout Instability, IntersectionObserver v2 and `interpolate-size` are scheduled although LY-D10 would hold them

**Location:** LY-M2 (line 603); LY-1 (`intersection-observer`, `layout-instability` at 95%†); 6.15; 6.19 (`interpolate-size`); LY-D10.

**Problem.** LY-D10 adopts ST-D18's rule (WG resolution, WPT tests, and two shipping engines or an Interop focus area). As far as this reviewer knows, the Layout Instability API (a WICG draft) and IntersectionObserver v2's `trackVisibility` ship only in Chromium, and `interpolate-size`/`calc-size()` only in Chrome ([SP 7.5], [GO 6.7]). Yet LY-M2 schedules LI and IO v2, LY-1 sets a floor on the `layout-instability` directory, and 6.19 plans `interpolate-size`. The spec contradicts its own admission rule and the accepted ST-D18 without saying why.

**Fix.** Check each against BCD on the pin date. Then either hold them under LY-D10 or record an explicit **owner exception** with its reason: LI is the basis of CLS, which users experience; IO v2 is an anti-clickjacking feature (P1 outranks P2). Move the exceptions into the LY-D10 row, and exclude held directories from LY-1 by name.

### C12. Medium — The `content-visibility` accessibility rule as chosen removes skipped content from assistive technology

**Location:** 6.9 "`content-visibility`" (line 337); 6.19 accessibility.

**Problem.** CSS Containment 2 §4.6 offers two compliant behaviours for skipped `auto` content: keep it in the accessibility tree **without** exact position, or, where a focusable element's representation needs exact position, omit it. The spec picks omission whenever "a focusable skipped element's accessible bounds would need layout". Izcal's semantic plane carries bounds for every node (B8), so that condition is always true, and every offscreen `auto` section containing a link drops out of screen-reader browse mode. That is the common long-article and feed case. The rule exists to prevent a timing signal, not to hide content.

**Fix.** Keep skipped content in the semantic tree with bounds marked **not determined** (no layout work); compute bounds only when the visual path lays the content out. Omit content only where a platform API demands exact position synchronously. Add an A11Y-1-style test: semantic-tree contents for skipped `auto` subtrees are identical with AT on and off, and layout work counters do not change when AT attaches. Have the A11Y track review it.

### C13. Medium — Fragmentation bounds do not compose under nesting, against LY-12's linear-work bound

**Location:** 6.5.6 balancing (line 273); 6.7 break scoring and nested fragmentation; LY-24; LY-12.

**Problem.** LY-24 bounds re-layout per flow (≤ 1 per fragmentainer) and balancing (≤ 3 layouts per column row). Nesting multiplies these. A balanced multicol inside a balanced multicol inside pages lays out its inner content up to 3 times for each of the outer's 3 trial heights, so 3ᵏ at depth k. Early-break reruns compound the same way. The cache does not save the work, because each trial height is a different fragmentainer size. LY-12 requires every DoS case to finish within 10× the canonical work of a linear pass, which a 10-deep nest of balanced columns (3¹⁰ ≈ 59,000×) violates.

**Fix.** State a composition rule: inner fragmentation roots inside a trial layout record break opportunities once (the 6.5.6 record) and are **not** re-balanced per outer trial; the outer re-runs only the final placement. Or cap balanced-multicol nesting at a canonical depth, beyond which `column-fill: auto` applies (recorded deviation). Add a nested-multicol case to LY-12's DoS suite and give LY-24 a per-nesting formula.

### C14. Medium — LY-26 conflicts with HGBA-2's target P-1 and its failure action

**Location:** LY-26 (line 550); LY-D6; 9.5 row "HGBA-2 Section 4.3".

**Problem.** HGBA-2's P-1 requires a median ≥ 1.5× style-plus-layout gain at 4 threads on style-heavy pages, ≤ 2% regression on the 20 simplest pages and bit-identical reftests, and its failure action is "demote parallel layout to opt-in". LY-26 sets ≥ 1.3× on a different set, and its failure action is "threshold raised; parallel code stays". The 9.5 amendment to HGBA-2 4.3 does not mention P-1.

**Fix.** Add an HGBA-2 amendment that restates P-1 as the joint ST-26 plus LY-26 target with one corpus. Pick one failure action; recommend LY-D6's, because demoting to opt-in leaves untested code in the binary. Cite P-1 in LY-26.

### C15. Medium — The networking interface is not "None required"

**Location:** 9.5 row "Networking spec" (line 688); 6.15 lazy loading.

**Problem.** The networking spec has the renderer's loader set request priority "from destination, render-blocking status, `fetchpriority` and viewport position, and update it (an image entering the viewport)", sending `PRIORITY_UPDATE` (NW 6.3, "Priorities and the request scheduler"). Viewport position is layout output, and each priority change is a timing-dependent decision that JS-D15 requires recorded. Lazy-load start points are layout decisions too. Image natural sizes can be known from the header bytes before full decode (in the image decoder process, FP-D11), which changes when layout can size an image and so when shifts occur.

**Fix.** Add a networking row: `hg-layout-query` publishes per-frame viewport-intersection classes for pending image and iframe fetches to the loader; each priority update and lazy-load start is a recorded decision point; natural sizes are taken from the first decoded header (a recorded input), and full decode never changes layout. Add the corresponding JS-D15 decision points to 6.1.

### C16. Medium — Layout records exposed to agents lack B9 origin filtering, and B10's "no agent-only computation" is not addressed

**Location:** 6.16 (line 400: "Agents receive the same records through MCP when AI is on (B9, B10)").

**Problem.** Decision records and the overflow finder can name cross-origin frames, their sizes, and frame-boundary state. B9 requires that "content from origins outside the set is never placed in the agent's observation", which needs per-record provenance. B10 requires that agent-only computation does not happen when AI is off. The accessibility layout checks and the MCP export path are not placed relative to G4.

**Fix.** Tag every decision record with the origin and frame of the box it describes; the MCP adapter filters by the task's read set (B9) and redacts cross-origin geometry beyond what the page itself could observe. State that records are computed only under replay on request (DevTools or agent), so B10 holds, and that the MCP adapter lives in an `ai`-feature crate (G4).

### C17. Medium — M-1 part 3: the representation layer follows LayoutNG's published taxonomy without per-choice alternatives

**Location:** 6.4 `LayoutResult` (line 222); 6.4 caching; 6.7 early break; 6.5.1 margin strut and exclusion space; 6.21.

**Problem.** Every term is sourced from published prose (RenderingNG and LayoutNG articles and the design document, [HP 2.3], [FR 3.4], [IP 1.5]), so no source-code rule is broken. But the combination of constraint space, `LayoutResult` with {fragment, break token, status Complete/BrokenInside/BreakBefore, baselines, end margin strut, out-of-flow list, early break}, a two-entry measure/layout cache, margin struts, exclusion space, early-break chains and spine rebuilds reproduces one engine's internal taxonomy closely. 6.21 derives five items from specifications, but gives no alternative considered for any of them and no measurement that decides between them. The JS spec answered the same concern with a per-choice derivation table (its 6.2a), and that is the quality bar here.

**Fix.** Add a table to 6.21 in the JS 6.2a form: for each structure, its derivation from spec text, one alternative actually considered (for example, out-of-flow placement by a separate post-pass instead of an upward list; margin collapsing as a resolved pass over sibling lists instead of struts; a cache keyed per space without a measure/final distinction), and the measurement that decides. Use Izcal or specification names where the term is an engine's own: "adjoining-margin summary" for margin strut, "float area" for exclusion space, "best-break record" for early break.

### C18. Medium — Staffing arithmetic and schedule placeholders contradict the spec's own evidence

**Location:** 9.3 (lines 612–637); 9.2 LY-M1, LY-M2.

**Problem.** (a) The ramp (18–22 at LY-M0, about 36–40 by month 12, full by month 18–24) averages about 29 engineer-years in year one and about 45–50 in year two, not 20 and 38. Total engineer-years to parity are therefore about 220–350†, not 200–330†. (b) The track rows sum to 48–59, not 48–60. (c) The reference-engine track (7–9) also carries `hg-reftest-raster`, WPT triage, the black-box harness, fuzzers and DST cases, yet must build a spec-literal engine for **every** context by LY-M2, while production has about 40 engineers. (d) LY-M2 at 24–33 months delivers every context, fragmentation, paged media, MathML, anchors, both traversals and parallel layout. The spec's own evidence is LayoutNG at about six years with a much larger existing team, and TablesNG alone as "a multi-year effort". (e) LY-M1 at 15–21 months exits on TX-M1 and ST-M1, which land at 15–21 months themselves. Under the owner's rule, cost is a planning fact, so the answer is to state the numbers correctly, not to cut scope.

**Fix.** Correct the engineer-year arithmetic and the total. Split the verification track into "reference engine" (8–12†) and "verification infrastructure" (3–4†). Set LY-M1 to at least 3 months after TX-M1 and ST-M1, and LY-M2 to a range consistent with the evidence (state it, for example 36–54 months†), with Stage 1 M4 and M6 moved accordingly in the Stage 1 8.1 amendment. Add a velocity checkpoint at LY-M1 that replans dates, never scope.

### C19. Medium — Several targets lack a binding milestone or a measurable owner, and decisions lack inline primary sources

**Location:** Section 8 intro, LY-20–LY-22, LY-21, LY-1; the spec's sourcing overall.

**Problem.** (a) Section 8 says relative speed thresholds are "first measured at Stage 1 exit", LY-M3 says "LY-20 to LY-26 measured", and LY-M4 says "LY-20 to LY-22 at threshold". Which milestone a miss triggers the failure action at is ambiguous. (b) LY-21's p95 interaction-to-next-paint ratio includes script, style, paint and raster, so layout cannot own it. (c) LY-1's floor max(F, B) is "B the best shipping engine's rate the same day", so it moves between runs and can fail without any Izcal change. (d) The method requires "inline links to primary sources for every claim a decision or target depends on". The spec carries one external link (LayoutNG) in 769 lines and cites note rows everywhere else. A reader cannot check, without opening a note, the ±10,000 grid clamp, HTML's span caps, IO v2's rules, `balance`'s line limits, the Fragmentation rules or the HiPC figures.

**Fix.** Add a "binding at" column to 8.3 (LY-20 and LY-22 at LY-M4, LY-26's never-hurts clause at LY-M3, and so on). Restate LY-21 as layout's share of the interaction's main-thread time, with INP reported only. Fix B at the release-branch date and record it. Add inline primary links for every row a decision or target cites (the notes already hold the URLs).

### C20. Medium — Class (a) discard and the safepoint handshake are not placed relative to a running layout pass

**Location:** 6.13 "Classes" (line 380); LY-32.

**Problem.** FP 6.4 lets the memory manager drop class (a) arenas on a running page after a safepoint handshake, which the script thread acknowledges at its next interrupt check. During a flush the script thread is running layout (or blocked on its workers). The spec does not say whether layout polls the pressure flag, whether a discard can arrive mid-pass, or what a long print or bulk layout does under Critical pressure. A discard mid-pass would free the previous store that workers hold `&` references into, which no type rule can stop across an unmap.

**Fix.** State that layout's arenas are discardable only at a safepoint **outside** a flush; that the pass checks the pressure flag at task boundaries and, at Critical, finishes the pass with caches off (no new cache entries) instead of stopping; and that LY-32's random discard points are drawn from those safepoints only. Add the rule to G43.

---

## Low

### C21. Low — Decision table gaps

LY-D8, LY-D13, LY-D14 and LY-D20 have no reversal trigger, and LY-D1's is "none foreseen"; LY-D21 has no cost. LY-D15 lacks the explicit-stack option (C6), and LY-D6 lacks "text preparation only". Missing decisions: fragment identity (C4), scrollbar unit (C10), and the cross-process hit-test contract (C5). **Fix:** fill each cell, for example LY-D8 "a fragmentation context's incremental path fails INC-4 for two releases: that context runs from scratch in fragmented mode", LY-D20 "a store's primary state exceeds 1%† of LY-30: move it to rare data".

### C22. Low — "Nothing else enters" (6.1, line 185) omits clock-based and decision inputs

IntersectionObserver v2's 100 ms delay, Layout Instability's 500 ms recent-input exclusion, LCP render times, ResizeObserver delivery, lazy-load starts and loader priority updates (C15) all read the clock or are decision points. They are B1 inputs, so determinism is not broken, but the list claims completeness. **Fix:** list them, and say which are clock reads and which are JS-D15 decision points (logged when recording).

### C23. Low — 6.12 and G43 disagree on who writes cache slots

6.12 says "each box's cache slot is written only by the task that laid the box out ... and the merge publishes it"; G43 says "cache slots are written only by the merge". **Fix:** use G43's wording in 6.12. Tasks write slot entries into task-local buffers with provisional ids, and the merge publishes them.

### C24. Low — G-rule registry statement is incomplete, and two sibling documents are stale

Line 439 lists "G13–G18 JS, G19–G24 style, G25–G29 text, G30–G36 networking" and omits G10–G12 (DevTools). ST 6.15 and 9.5 and the reconciliation's P7 table still say "G30–G35 networking, G36 onward the next spec", although networking 0.3.2 took G36. G37–G43 do not collide with anything. **Fix:** list G10–G12 in line 439, and add reconciliation items R12 and R13 below.

### C25. Low — Floating point in `hg-layout-num` is avoidable where it is observable

TX-D4 and G28 keep floating point off every text path that layout or script observes. Layout reintroduces IEEE binary64 for flex distribution and aspect-ratio transfer. Basic IEEE operations are deterministic in Rust on both ISAs, but NaN payloads are not specified (Rust's float semantics), and the reference engine must reproduce the exact operation order. Flex factors and ratios can be converted once at the boundary to exact dyadic or rational integers, and distribution done in i128 with one rounding. **Fix:** add LY-D3 option (e) "integer rational arithmetic throughout", and state NaN canonicalization at the boundary if floats stay.

### C26. Low — LY-31's placement in FP's budget, and the partition wording

ST-34 sits "inside FP §6.3's 0.25 MiB document line and FP-34"; LY-31's 64 KiB is said to be "inside the per-agent line" without naming the line. Separately, line 384 says layout data is "within one (site, container) process" and per agent, and 6.17 says caches are "per agent, inside one partition". Per-agent tables can span two partitions when both documents are in one agent (ST 6.11's argument, reconciliation P18). **Fix:** name the FP line (the 0.25 MiB document line, together with ST-34), and copy ST 6.11's same-agent argument instead of the unconditional "inside one partition".

### C27. Low — The scroll-anchoring suppression boundary is over-inclusive, and the "affects" mask lacks the trigger anchoring needs

Boundary (b) ends the suppression window at "every API in the forcing table", but Scroll Anchoring ends it before operations "whose result depends on scroll position", a subset. Adjustments then apply earlier than in other engines, observably. Suppression triggers include `transform` changes on the anchor path, but `transform` is paint-only in the five-category mask (6.3). **Fix:** use a scroll-position-dependent subset of the forcing table, and add a "scroll-anchoring trigger" flag to the mask's paint-only category.

### C28. Low — ST-M2's "reference" block layout is the production layout

LY-M1 delivers "the reference block layout with size containment that ST-M2 needs (the production block layout itself)". The style spec asked for a reference to test re-entry against. Using production makes a bug in production block layout invisible to ST-M2's re-entry tests. **Fix:** deliver `hg-layout-reference`'s block layout (due at LY-M0/LY-M1 anyway) for ST-M2, and test production against it separately (LY-5).

### C29. Low — `hashbrown` needs a hasher and an iteration rule

Section 7 allows `hashbrown` "with an MIT-dual hasher" but names none. Content-keyed tables with an unkeyed hasher (FxHash) are HashDoS targets; a keyed hasher must take its seed from the substrate (G3, B1). Iteration order must never reach output. **Fix:** name the hasher (for example a keyed hasher seeded from `Entropy` per agent), and add a lint or rule: no iteration over hash maps in layout crates except into a sorted buffer.

### C30. Low — `balance` is misdescribed

LY-D16 and 6.5.2 say Izcal balances "up to 10 lines, the spec's limit for a mandatory attempt". CSS Text 4 says balancing must not change the line count for 5 or fewer lines and **may** fall back above 10 lines ([SP 4.6]). **Fix:** "Izcal balances up to 10 lines (the most CSS Text 4 lets a UA skip) and falls back to greedy above."

---

## Engine-source evidence and licence check

**E1. No link to engine source; one source-derived prose list.** Every external URL in the spec and its seven notes was listed (about 600 links, 81 hosts). None points to engine code files, headers, commits, diffs, code review or source browsers. GitHub links are CSSWG issues, the WPT directory tree (tests are not engine source), the Servo wiki's Layout Engines Report (prose), a TAG review and Readium's documentation; `docs.google.com` links are the LayoutNG design document (a design document written as a document) and Project Zero's spreadsheet; Bugzilla and WebKit Bugzilla links are bug discussion. Each note records search results that pointed at source and were not opened. Two items need a line in the spec. (i) Paul Irish's "What forces layout" gist was built by its author from Blink source ([SM 7.12], [IP 4.6]). The spec uses it only as hypotheses, which the letter of M-1 part 3 allows. It is still the only input to the forcing table that is not a specification. **Fix:** generate the table from the specification-derived list plus an exhaustive black-box probe of every WebIDL geometry and scroll getter (Q-LY-4), and drop the gist from the method, so the table's provenance is clean. (ii) `firefox-source-docs.mozilla.org` pages are prose documentation rendered from the Firefox tree. They are allowed as documentation; record that classification in Section 7, as the text spec's 0.3.1 did for prose design docs hosted in repositories.

**E2. Values that look source-derived.** [SR 6.3] gives Gecko's ±17.9 million px range "from prior knowledge" of the app-unit width (2³⁰/60). That is a constant stated only in source. The spec does not use it, but the note row should be graded Low and moved to Q-LY-2 as a black-box measurement. [SR 6.2]'s 33,554,428 px figure is attributed to developer reports and the spec uses only "~33.5M px" under Q-LY-2; acceptable. No threshold in Section 8 or value in Section 6 was found to come from engine source. The † values (15 px, 65,536, 4 sizes, 8 lines, 3 layouts, 3 px) are marked and listed in 11.2.

**E3. Dependencies, licence and class.**

| Dependency | Licence | Generic infrastructure? | Finding |
|---|---|---|---|
| Rust std | MIT OR Apache-2.0 | Yes (named in M-1 part 1) | OK |
| `smallvec` | MIT OR Apache-2.0 | Yes (data structure; licence audit row) | OK |
| `bitflags` | MIT OR Apache-2.0 | Yes | OK |
| `hashbrown` (`default-features = false`) | MIT OR Apache-2.0; its default `foldhash` is Zlib ([LA 2.7]) | Yes | OK only with defaults off, and a named MIT-dual hasher (C29). Its other optional dependencies (`allocator-api2`, `equivalent`) are MIT-dual; confirm in `deny.toml` |
| `stacker` + `psm` | MIT OR Apache-2.0 (both); `psm` compiles assembly at build time through `cc` (toolchain exception) | **Unclear**: generic in purpose, but it switches stacks in `unsafe` assembly, overlapping JS-D19's engine-managed segments | Section 7 should not pre-decide this. Owner classification (Q-LY-18); recommendation in C6: use the substrate's JS-D19 segments or explicit work stacks instead |
| Phase pool (B3) | Izcal's | — | OK; layout takes no direct Rayon dependency |
| Test-only: proptest, cargo-fuzz, Miri, Kani, loom, WPT (BSD-3-Clause) | Various | Test-only (OD-L2) | OK; must not reach a shipped binary (a `deny.toml` check on the shipped graph) |

No layout library (Taffy, Yoga, Morphorm, Servo layout, Blitz) is a dependency, and Section 7 excludes them explicitly. The `hg-reftest-raster` tool is Izcal's and test-only.

**E4. Data sets.** The HTML Rendering UA sheet (CC BY 4.0, WHATWG) is allowed under OD-L6. The MathML Core and SVG UA sheets were confirmed by the owner as W3C standards data (ST 0.3.3, Q-ST-19). The **MathML Core operator dictionary** and the **Unicode-based glyph assemblies** (MathML Core appendix B.4) are a further W3C data set shipped in the engine image. The spec asserts they are "the same class" but the owner's confirmation named only the UA sheets. **Fix:** put both in Q-LY-19 as a classification question for the owner (not only the licence text), and check the document's licence: W3C documents published since 2023 use the W3C Software and Document License, which is permissive, so the check is likely quick. Unicode data used for assemblies falls under OD-L1. WPT is test-only.

---

## Reconciliation list (amendments to other documents that Section 9.5 misses or gets wrong)

| # | Document | Amendment needed | Finding |
|---|---|---|---|
| R1 | Fonts and text spec 6.8, 9.5 | **Withdraw** the proposed numeric clause; TX-D4 stands. Add instead: the opportunity API returns cumulative positions in TX's quantum, and layout converts cumulative offsets with round-half-to-even | C3 |
| R2 | HGBA-2 target P-1 | Restate as the joint ST-26 plus LY-26 target with one failure action | C14 |
| R3 | Networking spec 6.3 (priorities) | Layout supplies viewport-intersection classes to the loader; priority updates and lazy-load starts are recorded decision points | C15 |
| R4 | Footprint spec 6.4 "Layout and paint" | Replace "one fragment per box per constraint space ... evicts entries for offscreen, unchanged subtrees first" with this spec's cache keying and eviction rule; fragment handles outside a pass are (id, epoch); store canonical bytes count live records | C4, C7 |
| R5 | Style spec 6.10, 6.12 | Re-entry runs at a scheduler barrier with no layout worker running (not on a layout worker); multi-size re-entry produces versions in a pass-local overlay; the bound is structural, not 4† | C1, C2 |
| R6 | Future graphics spec | Co-sign the 360 B split before it is binding; provide the `PaintOrder` read contract and the renderer-to-browser hit-test region message | C5, C8 |
| R7 | Stage 1 2.2 (G38) and 2.1 | `hg-hit` and `hg-layout-query` may depend on the paint-order contract crate; add G44 (no layout task calls style or box-tree write APIs) and G45 (primary state stores no fragment ids) | C1, C4, C5 |
| R8 | JavaScript engine spec 6.9 (JS-D19) | If owner chooses option (e): the substrate's stack segments serve layout's recursion too, with one canonical stack budget | C6 |
| R9 | HGBA-3 INC-1, INC-2, INC-4, FLUSH-1 staging | These are Stage B in HGBA-3 and Stage 1 5.6; this spec measures them at Stage 1 exit (LY-22, LY-25). Restage them in HGBA-3, not only in Stage 1 5.6 | — |
| R10 | Shell UI design | The overlay/classic scrollbar preference, defaulting to the OS accessibility setting; the page-zoom levels as recorded environment inputs; form-control picker UI anchored by `hg-layout-query` rectangles | C10 |
| R11 | HGBA-3 PRIV-2 | Count the scrollbar-preference bit and the localized form-control strings in the residual-entropy budget | C10 |
| R12 | Style spec 6.15 and 9.5 (registry text) | "G30–G36 networking, G37–G43 layout" replaces "G30–G35 networking, G36 onward the next spec" | C24 |
| R13 | Phase 2 reconciliation P7 table | Same registry correction; add G10–G12 DevTools in every registry statement | C24 |
| R14 | Stage 1 8.1 and 8.3 | M4 and M6 dates follow corrected LY-M1 and LY-M2 placeholders; the layout track's engineer-years are about 220–350† | C18 |

## Summary of the High findings

1. **C1:** Container re-entry runs on a parallel layout worker and publishes style ids and box-tree changes while sibling workers read them, which breaks phase purity and LY-4. Fix: a scheduler barrier for re-entry waves.
2. **C2:** Multi-size re-entry has one copy of styles and boxes per node, and the bound-exhaustion rule depends on which layouts ran, so incremental and from-scratch results diverge (hysteresis). Fix: a structural bound, versioned re-entry results and a style-version cache key.
3. **C3:** LY-D3's text rationale misreads TX-D4 (advances are finer than 1/64 px), and the proposed amendment would change an accepted decision. Fix: convert cumulative text positions to `LayoutUnit` and keep TX-D4.
4. **C4:** Fragment ids depend on history and carry no generation, and primary state references them. Byte-identity targets are therefore ill-posed, and discard breaks primary state. Fix: an id-free canonical fragment tree, no fragment ids in primary state, and epoch-tagged handles.
5. **C5:** Hit testing, IntersectionObserver v2 and cross-process hit testing depend on an unwritten graphics contract under ship-blocker targets. Fix: define the paint-order contract, the hit-test region message and the IO v2 proof here, and name a graphics owner before LY-M0's review.
