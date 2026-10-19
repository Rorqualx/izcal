# Izcal HTML and CSS parsing and style: independent design critique (phase 8)

Written 19 October 2026 by a reviewer who did not write the spec. Subject: `reports/Izcal HTML and CSS parsing and style.md`, draft 0.1 (line numbers below refer to that draft). Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8 (design critique against the existing specs, looking for contradictions, missed constraints and M-1 violations).

**Read against:** HGBA-2 2.3.0 (P1–P6, M-1 parts 1–4, I-1, Section 4.3, C-1, ANC-1); HGBA-3 (B1, B3, B4, B5, B6, B8, B9, B10, DET-1, INC-1 to INC-4, FLUSH-1, D8, D9); Stage 1 design (Section 2.1 layers, 2.2 G1–G9, 3.3, 3.4, 5.4, 5.5, D-S1, D-S2, 8.3); JavaScript engine spec draft 0.3 (JS-D4, JS-D15, JS-D17, H-2, G13–G18, Sections 6.0, 6.2, 6.5, 6.9, 6.12, 6.15, JS-17, JS-M0/M1 durations); resource-footprint spec draft 0.3 (FP-2, FP-3, FP-5, FP-11, FP-12, FP-23, FP-24, FP-36, FP-D16, Sections 6.3, 6.4 including the derived-store table and class (a) rules, 8.2); DevTools spec 0.3 (G10–G12, DT-D3, DT-2, M3, Q-DT-4, staffing); networking spec draft 0.1 (Section 6 renderer/network-service split of Fetch and CSP, speculative requests); fonts and text spec draft 0.1 (6.5 web-font installation, interface table, TX-D1, TX-D17); security scope 0.3 on its worktree (ATK-1, SEC-24, IPC-R15); coding standard 0.3.1 on its worktree (tiers P/R/T, CS-ER-1, CS-ER-3, CS-TY-10, CS-AS-2, CS-PF-1); this spec's research notes (`frame.md`, `history_postmortems.md`, `specs_semantics.md`, `performance_invalidation.md`, `security_robustness.md`, `gaps_opportunities.md`).

**Owner direction applied as binding:** engine work is all ours (no html5ever, Stylo, `selectors`, `cssparser`, lightningcss or any browser-project or engine library; generic infrastructure only); no shortcuts for time or complexity; the best component from first principles; cost is a planning fact; staging only for correctness, security or measurement, never schedule; the accepted JS-spec decisions (one-copy DOM outside the cage, JS-D4; determinism B1 and recorded decision points, JS-D15; precise GC) and the footprint budgets (§8.2), regenerable caches (FP-D16) and per-process fixed cost.

**Overall.** This is a serious, well-evidenced draft. Its central idea (one implementation per grammar, every derived structure a deterministic function of its inputs, every fast path checked against a reference) is the right one, and it makes good use of JS-D4, JS-D15 and the footprint spec. Most findings below are places where the draft states that idea more strongly than its mechanisms deliver. Four are about determinism and data ownership in the style store: IDs that do not exist when the parallel phase needs them, partition-wide tables shared by agents on different threads, lazy parsing that writes to "immutable" sheets during a parallel flush, and a class (a) arena that holds state that cannot be rebuilt. One is a determinism hole in the speculative parser. Two are privacy and security keying gaps: `:visited` keyed without the container, and parsed-sheet sharing keyed without quirks mode or CORS state. One is about M-1 provenance: the research notes cite engine source files, and several structures in Section 6 have the same shape as those files and are not derived in Section 6.18. Eight findings are High, 16 Medium and 11 Low.

---

## High

### 1. High: M-1 provenance. The design is partly sourced from engine source files, and several structures match those files without a derivation

**Section:** 6.2 depth cap (line 234), 6.4 yields (line 258), 6.6 lazy parsing (line 285), 6.7 rule index and ancestor filter (lines 293–295), 6.7.2 `:has()` memo (line 307), 6.9 match keys and custom-property chains (lines 343, 359), 6.18 (lines 503–516), 7 (line 520).

*0.3.1 note: the source files this table named are no longer cited anywhere in the notes; HGBA-2 2.3.1 makes engine source never evidence, and [source_evidence_pass.md](source_evidence_pass.md) logs each row's replacement or removal.*

**Problem.** M-1 part 2 says that interoperability evidence is a **black-box observation**: "test case plus each engine's observed result, never a link to its source". Part 3 lets another engine's design be cited only "as evidence ... from a published source, never as a template". Part 4 forbids "reading its code" to settle a disagreement. Part 1 applies to "people or AI tools". The research notes this spec was synthesized from read engine source files directly, and several of the resulting designs have the same shape as those files:

| Design element in the spec | Source file read in the notes |
|---|---|
| Rule-index buckets "id, class, attribute name, tag, then a few pseudo-class buckets (`:host`, `::slotted`, `::part`, links), then universal" (line 293) | a Blink source file (old PI 3.5; row removed 0.3.1) |
| Per-worker ancestor filter "rebuilt to the common ancestor after taking work" (line 295) | a Stylo source file (old PI 3.3; row removed 0.3.1) |
| Invalidation positions subject, ancestor, sibling, `:has()` argument (line 320 table) | a Blink source file (old PI 7.2; row removed 0.3.1) |
| Match key plus computed-style memo (lines 343, 349) | a Blink source file (old PI 4.4; row removed 0.3.1) |
| Per-pass `:has()` memo (line 307) | a Blink source file (old PI 7.4; now replaced by Lee 2022, Igalia) |
| Lazy declaration parsing (ST-D19), whose only evidence was a Blink source file (old PI 10.1) | that file (row removed 0.3.1) |
| A token budget plus a time budget for parser yields (line 258) | a Blink source file (old PI 8.4; row removed 0.3.1) |
| HTML depth cap 512 (line 234) | [SP 8.3][SP] is labelled "black-box behaviour" but says "engine sources read for the constant only"; the constants were quoted from engine source (removed 0.3.1; now WebKit bug 63856 and an ST-M0 black-box test) |

Section 7's sentence "where research notes read engine source files, they read them for numbers and comments, never as designs" is an assertion. M-1 does not allow that exception, and nothing in the process enforces it. Section 6.18 derives six structures from first principles. It does not derive the pseudo-class bucket set, the per-worker filter rebuild, the `:has()` memo scope, lazy declaration parsing, the custom-property chain with a flatten bound, or the two-budget yield policy. These are the ones whose only cited evidence is a source file.

**Evidence.** HGBA-2 M-1 parts 1–4 (lines 70–77). In the notes, source links appear in `performance_invalidation.md` (11 Chromium source files), `history_postmortems.md` (Stylo, WebKit and Blink source), `gaps_opportunities.md` (Stylo source) and `specs_semantics.md` row 8.3.

**Required change.**
- Remove every engine-source citation from the report. Where a number is needed (depth cap, filter size, budgets), replace it with a **black-box test** run at ST-M0 (Q-ST-3 already plans this for the depth cap; cite only its result), a published document (blog post, paper, bug discussion), or a † first-principles value.
- Ask the notes' owner to strike or re-source the source-file rows, and record the change in the spec header, as the JS spec did for its fact-check fixes.
- Extend Section 6.18 with a first-principles derivation and a deciding measurement for each structure in the table above. Where none can be given, choose the representation by measurement on Izcal's corpus (for example, derive pseudo-class buckets from corpus selector statistics, not from Blink's list).
- Add a provenance step to ST-M0's design review: a reviewer who has not read the cited files checks Section 6 against 6.18. Add to the research brief in the skill that engine source is not to be read, even for constants.

### 2. High: The speculative parser is a determinism hole, and its worker would intern into partition tables

**Section:** 6.4 (lines 260–264), ST-9, ST-14, ST-41.

**Problem.** Line 264 claims that "the set of speculative fetches is a function of the bytes, the environment snapshot and the recorded position at which the parser blocked." It is not. The speculative parser runs on a worker **while** the blocking script runs, and line 262 discards it when the script finishes. How far it gets before then depends on the relative speed of two threads. So the set of speculative requests depends on timing, and the requests are observable in three ways:
1. **To script.** Resource Timing entries, the timing and cache state of the later real fetch, and `load` ordering of preloaded subresources.
2. **To the network fixture.** A replay that issues a request the recording never made, or omits one it made, does not match the recorded network bytes. B1 records responses, so this breaks DET-1.
3. **To servers.** This is acceptable, but it is the same nondeterminism.

JS-D15 exists for exactly this case: work whose timing depends on another thread. The spec makes parser yields and sheet installs decision points but leaves out the speculative parser's progress.

Second, the speculative parser runs the real tokenizer and tree builder on a worker. Unknown tag and attribute names become atoms. Section 9.5 proposes that dynamic atoms are "the JS engine's atoms, under JS-17's partition rule". A worker interning into that table mutates shared state from off the script thread, which JS-D4 and Stage 1 3.3 forbid. It also makes atom IDs depend on timing, and atom IDs feed attribute-set hashing, rule-index buckets and FP-24's canonical bytes.

**Required change.**
- Make speculation a **recorded decision point** (JS-D15 (b)). In production, the token position the speculative parser reached when the block ended, or each speculative request issuance, is logged when recording. On replay, speculation is re-run to exactly that position. In simulation, the worker runs on the seeded scheduler. Restate line 264 accordingly and add speculation to ST-41.
- State how the real parser's fetch reuses a speculative response, and make that reuse part of the recorded decision.
- Give `MockSink` and the speculative tokenizer a **private, discardable atom scope**. Names are compared by string or by a scope-local ID, and never interned into the partition or JS atom tables. Add a G-rule or type rule that the speculative path cannot reach the partition intern tables.

### 3. High: Partition-wide style tables are shared by agents that may run on different threads

**Section:** 6.9 "Deterministic identities" (line 351), 6.11 "Arenas, partitions and lifetime" and "Sharing scope" (lines 381, 385), ST-5, ST-33.

**Problem.** Interned groups, computed styles, match keys, declaration blocks and custom-property sets live in **per-partition** arenas, and the merge interns new entries "into the partition's tables" in document order. JS 6.0 and 6.12 say a process holds several agents of the same (site, container), and that "window agents of the same (site, container) in one process **may** share the script thread". They may also run on different script threads (for example, two tabs of one site in one container, which form one partition). Then:
- two documents can flush **concurrently**, each with its own phase pool and merge, both writing one partition's intern tables. That is shared mutable state across phases, the hazard Stage 1 3.3 names;
- ID assignment, and so FP-24's canonical bytes and ST-5's style IDs, depends on how the two documents' flushes interleave, which is not recorded per document;
- the reclamation mark phase at "the end of a flush" (line 381) runs from **one** document's style columns, rule index and memo. It either frees objects another document still references, or it must stop the other document's script thread, which nothing specifies.

FP §6.4 allows sharing across "documents of one partition in the process". The JS spec allows those documents to run on separate threads. This spec has to reconcile the two.

**Required change.**
- Choose and record a decision (ST-D7 is the place). Options: (a) intern tables **per agent** (per event loop), with only immutable, merge-published data shared across agents of a partition; (b) partition tables with a single writer per partition, where the merge publishes through a partition-level sequencer whose order is a recorded B1 decision point; (c) per-document tables. State what each costs in FP-11 bytes. Recommend (a) unless measurement shows that cross-agent sharing pays.
- Reclamation must take roots from **every** document that can reference the table, at a point that is deterministic or recorded, with the trigger in canonical bytes (not "measured garbage").
- Extend ST-5 and ST-33 with a two-agent, two-thread case.

### 4. High: Deterministic identities do not exist when the parallel phase needs them

**Section:** 6.8 "Early cutoff" (line 330), 6.9 groups, memo and "Deterministic identities" (lines 345–351), 6.11 "What style writes" (line 379), 6.12 (lines 389–391), ST-D7.

**Problem.** IDs are assigned by "a merge step in document order ... after the parallel phase" (line 351). The parallel top-down traversal needs those IDs **during** the phase:
- **Inheritance.** "Inherited groups with no declaration share the parent's group" (line 346). A child styled in the same pass as its parent needs the parent's group identity before the merge has assigned it.
- **Memo.** The memo key includes the "parent computed-style ID" (line 349).
- **Early cutoff.** "Equality with the old style is one integer comparison" (line 330), but the new style has no integer until the merge, so the descent decision cannot be made in the parallel phase.
- **Column writes.** Workers write "disjoint slots of these columns" (line 379) and hold "`&mut` only to their own output slices" (line 389). The final value to write is the merged ID, which does not exist yet. Also, node slots are allocated in creation order, not tree order, so a subtree task's elements are not a contiguous slice of a slot-indexed column. "Disjoint `&mut` output slices", as written, cannot be expressed in safe Rust over slot-indexed columns. The phase-purity-by-types claim therefore does not hold as specified.

**Required change.**
- Specify **provisional identities** for the parallel phase: content hashes (with full-content comparison on a match, see finding 22), or worker-local IDs (worker, index) that the merge maps. Restate inheritance, the memo key and early cutoff in terms of them, and show that the merge's mapping cannot change any decision taken under provisional IDs.
- Workers write results to **task-local output buffers** keyed by slot. The merge, which already exists, scatters final IDs into the style columns sequentially. Restate line 389's type argument to match.
- Add a fourth ST-D7 option that removes the merge: wide (64- or 128-bit) content-hash identities with collision checks. Cost it in FP-11 bytes, so that the merge is chosen by measurement (Q-ST-8), not by default.

### 5. High: Lazy declaration parsing writes to shared, "immutable" sheets during a parallel flush

**Section:** 6.6 "Parsed sheets" and "Lazy declaration parsing (ST-D19)" (lines 283–285), 6.12, ST-D19.

**Problem.** Sheets are immutable `SheetContents`, shared across documents of one partition (FP §6.4: "immutable after parse"). ST-D19 parses a declaration block "on the first match". First matches happen inside the parallel style phase, where workers hold `&` to sheets (line 389). So either:
- workers mutate shared sheet contents. That is a data race, breaks phase purity (Stage 1 3.3), and makes the interned-value IDs created during parsing depend on which worker parsed first (FP-24, ST-5); or
- parsing is deferred to a sequential step, which the spec does not describe.

"Invisible by construction" (line 285) covers only the CSSOM's view. It does not cover memory, IDs or thread safety.

**Required change.** Choose one design and state it:
- (a) workers parse into task-local buffers, and parsed blocks are published in the deterministic merge, in document order, before IDs are assigned;
- (b) a sequential pre-pass parses every block the rule index selected for the dirty set before the parallel phase;
- (c) eager parsing.

Record that the published block is immutable thereafter, so FP §6.4 holds. ST-D19's reversal trigger should include the cost of (a) or (b). Its evidence also needs a non-source citation (finding 1).

### 6. High: The class (a) style arena holds state that cannot be rebuilt, so FP-12 cannot be met

**Section:** 6.9 inline style and the cascade (lines 343–357), 6.10 "Animations and transitions" (line 373), 6.11 "Arenas, partitions and lifetime" (line 381), ST-33.

**Problem.** Line 381 puts interned groups, computed styles, match keys, **declaration blocks** and custom-property sets in a class (a) derived store. FP §6.4 lets the memory manager drop such a store on a **running** page (Critical rung 5, after the safepoint handshake). FP-12 and ST-33 require bit-identical DET-1 hashes afterwards. Three kinds of content in that arena are not derived:
1. **Before-change style.** CSS Transitions starts a transition by comparing each element's before-change style with its after-change style. `@starting-style` applies when an element "had no computed style in the previous style change event". After a drop, every element has no previous style. Transitions that should start do not, every element looks newly styled, and `@starting-style` transitions fire across the page. Script observes this (`transitionrun`, `getAnimations()`), so DET-1 hashes diverge.
2. **CSSOM-mutated and constructed sheets.** After `insertRule`, `replaceSync` or `style.setProperty` on a rule, the mutated declaration blocks are the **only** copy (line 283, copy-on-write). They are primary data.
3. **Inline-style blocks** referenced from JS-D4's inline-style column (line 377: "a declaration block in the partition's style arena"). Dropping the arena leaves a column in `hg-dom`, which is primary data, pointing into freed memory, or forces a reparse whose result depends on whether CSSOM had edited the block.

**Required change.**
- Split the store. Make sheet contents after CSSOM mutation, constructed-sheet contents and inline-style blocks **primary data** (not class (a)), owned by `hg-cssom` or `hg-dom`.
- Keep **before-change styles** of elements with a non-`none` `transition-property`, or with live animations, outside class (a), or snapshot them before a drop. Define what `@starting-style` sees after a rebuild.
- Register only truly derived parts (groups, computed styles, match keys, memo, indexes) as class (a). Extend ST-33's DST to pages with running transitions, `@starting-style` and CSSOM-mutated sheets.

### 7. High: The parsed-sheet sharing key is incomplete, with a correctness bug and a cross-origin read

**Section:** 6.6 "Parsed sheets" (line 283), 6.11 "Sharing scope" (line 385), 6.14 "Exfiltration" row, ST-13.

**Problem.** Sheets are shared by "(partition, URL, content hash)". The parse result and its CSSOM exposure also depend on:
- **quirks mode** of the referring document. Quirks-mode parsing accepts unitless lengths and hashless colours, so one byte stream parses differently in a quirks and a no-quirks document of the same partition;
- **CORS mode and response type**, which set the sheet's **origin-clean flag** (CSSOM: `cssRules` throws `SecurityError` for non-origin-clean sheets). If one document loads `https://cdn.example/x.css` with `crossorigin` and another without it, a shared `SheetContents` that carries origin-clean lets the second document read a cross-origin sheet's rules. That is a cross-origin data exfiltration, an ATK-1 case;
- the **environment encoding** used to decode bytes without a BOM or `@charset` (line 209). This is mostly covered by hashing decoded content, but the key should then say so;
- the **base URL** for relative `url()`, for inline `<style>` (resolved against the document base, which `<base>` changes).

**Required change.** Key shared contents by (partition, URL, decoded-content hash, parsing mode, encoding), and keep the **origin-clean flag, CORS mode and base URL outside** the shared contents, on the per-document `CSSStyleSheet`. Add two cases to ST-13 and the mXSS/differential suite: a quirks/no-quirks pair, and a CORS/no-CORS pair on the same URL.

### 8. High: `:visited` is partitioned without the container

**Section:** 6.14 "`:visited` (ST-D15)" (line 443), ST-D15, ST-13.

**Problem.** Izcal's partition is **(top-level site, container)** (FP §6.4, I-1, JS-17's code-cache key, the networking spec's keys). ST-D15 keys visited state by "(link URL, top-level site, frame origin)", which is Chrome 136's triple. Chrome has no containers. Under that key, a link visited in the user's "Work" container makes the same link render as visited in the "Personal" container on the same top-level site. Containers exist to prevent exactly that linkage. This is a cross-container history channel in a P1 feature.

**Required change.** Key visited state by (link URL, top-level site, frame origin, **container**), and state how the broker builds that store at navigation time. Say what "salted per partition" means and how the salt is kept out of recordings (Stage 1 3.4, SEC-21). Visited-set updates delivered after a navigation are B1 inputs applied at a task boundary, and should be named so. Add a cross-container probe to ST-13.

---

## Medium

### 9. Medium: Atoms and attribute values must be readable by workers without touching the cage

**Section:** 6.11 (line 377), 9.5 JS-spec amendment (row "JavaScript engine spec 6.9, 6.15").

**Problem.** The amendment makes dynamic tag, attribute and CSS-name atoms "the JS engine's atoms". The JS spec does not say where atom text lives. If atoms or attribute values are JS strings in the cage, style workers that compare text (case-insensitive attribute selectors in HTML documents, `[attr^=…]`, `:lang()`, `:dir()`, `attr()` substitution) read cage memory. JS-D4 forbids that, and H-2 would make every such read untrusted.

**Required change.** State that atom text, attribute-set contents and inline-style source live **outside the cage**, in `hg-dom` or `hg-types` storage that workers read during a flush, with JS strings created from them as external strings (as for DOM text). Make this a requirement in the JS-spec amendment and a G-rule (style crates cannot name `hg-heap` types).

### 10. Medium: State sources write element-state bits without a stated writer, thread or timing

**Section:** 6.7.1 (line 301).

**Problem.** JS-D4 says only `hg-dom`, on the script thread, writes the columns. About 35 pseudo-classes have "a named state source that writes an element-state bit", but the section does not say which thread or when. Some sources are not script-thread events: media playback state (`:playing`, `:paused`, `:buffering`, `:muted`), fullscreen, `:target` after navigation, `:autofill` from the shell, `:popover-open`. `:hover` also changes when layout or scrolling moves content under a stationary pointer. Engines recompute it after layout, often on a timer.

**Required change.** Require every state source to post a script-thread task (or to run in "update the rendering" at a defined step) that writes through `hg-dom`'s API and logs a mutation. Define hover recomputation as a function of layout output and the recorded pointer position at a named rendering step. List each state source with its writer and its B1 input.

### 11. Medium: Demand-driven `getComputedStyle` is wrong inside size containers

**Section:** 6.10 "Demand-driven style for script queries" (line 367), ST-4.

**Problem.** "`getComputedStyle` on a property that needs no layout restyles only the element and the dirty part of its ancestor chain." Inside a size container whose query results are stale, **any** property set inside `@container`, and any length using container-relative units (`cqw`, `cqi` and so on), needs the container's layout first. The rule as written returns a value a full flush would not, and FLUSH-1's "bit-identical" requirement fails.

**Required change.** The demand path checks the element's container chain. If any container's size is dirty and the element's matched rules or units depend on it, it falls back to a layout flush. Add container cases to ST-4's demand-driven comparison, and tie the path to FLUSH-1, which the spec cites but never maps to a target.

### 12. Medium: Sheet loading, blocking and off-thread installation are undefined

**Section:** 6.6, 6.12 (line 393), ST-41.

**Problem.** ST-41 and line 393 mention "the install of a sheet parsed off-thread", but no section designs off-thread sheet parsing: which thread, what it may intern (finding 3), and when the install happens. The spec also does not cover the HTML rules that decide script execution order: **script-blocking style sheets** ("has a style sheet that is blocking scripts"), `blocking=render`, render-blocking `<link>`, `@import` loading and `load`/`error` event order. These are timing-sensitive decisions.

**Required change.** Add a subsection that covers the sheet pipeline (fetch, decode, parse location, install as a JS-D15 decision point), the anchored blocking rules, and events, with targets for each in ST-41.

### 13. Medium: The CSP enforcement split contradicts the networking spec

**Section:** 6.4 (line 260), 6.14 "Exfiltration" row, 6.16 integrations, ST-14, 9.5 networking row.

**Problem.** The spec says "the network service, not the renderer, enforces CSP" on speculative and style-initiated fetches. The networking draft (Section 6, "Where each part of Fetch runs") has the **renderer** run CSP's pre-request checks that need the document. The network service re-checks only the connect, script and frame directives the broker gives it. `style-src`, `img-src` and `font-src`, and any `<meta>` CSP (known only to the renderer, and applied by the speculative parser here), are renderer checks there.

**Required change.** Agree one split with the networking spec. Either the renderer enforces document-derived CSP (including `<meta>`) for speculative and style fetches, or the network service gets the full policy container per document, including `<meta>` additions, through a typed message. Restate ST-14's measurement point to match.

### 14. Medium: The reference style engine is less independent, and costs more, than claimed

**Section:** G20 (line 476), ST-D12, ST-4, ST-6, 9.3 "Reference engine and verification" row, Section 10 risk 2.

**Problem.** G20 lets `hg-style-reference` share the generated `hg-css-props`. Per line 281 and the crate table, that crate contains the **value parsers, computed-value types and conversions, serialization and interpolation types** for about 760 properties, which is most of the computed-value code. ST-4 and ST-6 therefore cannot detect a conversion bug. The risk table names only "a wrong overlay entry" as common-mode. The trait implementation over JS-D4's columns (`hg-tree-traits`) is also shared code. In the other direction, G20 forbids sharing `hg-css-syntax`, so the reference needs its own Syntax 3 parser, selector parser, cascade, substitution interpreter, container queries and `:has()`: a second style engine. That is staffed at 3–4 engineers who also own the round-trip, mXSS, fuzzing and WPT-triage work. Running the spec-literal engine on every flush of the mutation corpus, plus 10k DST seeds in T1 and 1M in T2, also has a CI compute cost that the spec does not estimate.

**Required change.** Either let the reference compute values with spec-literal conversions generated from webref **independently** (a second generator, or an interpreter over the value-definition grammar), or state that ST-4 and ST-6 do not verify computed-value conversion and name its oracle (WPT plus black-box differential). Re-estimate the verification track. Add the CI compute budget to 9.3.

### 15. Medium: The footprint accounting has gaps against FP-11 and the JS-D4 column budget

**Section:** 6.11 "Bytes per element" (line 383), 9.5 JS-spec amendment, ST-30.

**Problem.** The amendment asks JS-D4's element-state column to hold "at least 64 bits of pseudo-class state". JS-D4's 48 B† core holds five 32-bit links (20 B), tag, attribute-set reference, inline-style reference, rare-data index and generation (about 20 B), plus "flags and element state". 64 bits of state alone use the remaining 8 B, leaving nothing for node flags. Node flags are not style's to give away. ST-30's 80 B† also does not say whether pseudo-element rare entries and the per-element custom-property reader sets of ST-D14 are inside it, and the remaining 432 B for layout and paint is not stated as an allocation.

**Required change.** Show the 48 B column layout with the proposed state width (a CS-PF-1 assert), or move rarely used state bits into rare data. List ST-30's components explicitly, including reader sets. Record the split of FP-11's 512 B among DOM, style, layout and paint, so the layout spec inherits a number.

### 16. Medium: The per-process fixed cost of the style system is not stated (FP-2)

**Section:** 6.6 UA sheet (line 287), 6.11, 6.7 ancestor filter, 6.16 user style sheets.

**Problem.** FP §6.3 gives "Document, DOM and style state for `about:blank`" **0.25 MiB** of the 4 MiB empty-process budget, and F_agent repeats it per window agent. This spec adds per-process and per-agent structures but gives no line: initial capacities of the partition intern tables, the memo and the arenas; per-worker ancestor filters and scratch arenas; the environment snapshot; and the **global user style sheet** (ST-D17), which every process would parse and index. ST-32's "UA style data 0 private dirty bytes" covers only the image. Also, the UA cascade has environment-dependent parts (`forced-colors`, `prefers-color-scheme` and quirks variants), so it is unclear whether those precomputed groups are in the image or built per process.

**Required change.** Add a line-item table for the style system's share of the 0.25 MiB `about:blank` line and of F_agent. Make tables and arenas lazy, created at first style use beyond the UA initial style. State how the global user sheet is delivered: parsed once into a shared read-only region from a utility or shell renderer (IPC-R15 forbids parsing in the broker), or parsed per process and counted.

### 17. Medium: Hidden schedule staging in the performance thresholds and milestones

**Section:** 8.3 (ST-21 to ST-23), 9.1 (lines 618–631), 9.2 ST-M1/ST-M2.

**Problem.** Section 9.1 says every ordered or held item is listed with a first-principles reason, and that "nothing else is staged". Two kinds of staging are not listed:
- ST-21, ST-22 and ST-23 accept **≤ 1.25×† at Stage 1 exit and ≤ 1.0×† at Stage 2**. The design is the same at both stages. The difference is time to optimize, which is schedule. This is the same pattern the JS-spec critique raised for JS-D8.
- ST-M1 ships "Selectors 4 without `:has()`" and "computed values without registered properties". As a build order inside Stage 1 this is defensible, but it is not in 9.1's table.

**Required change.** Either set one threshold per target (the Stage 1 exit is just when it is first measured), or give the first-principles reason for the interim ratio (for example, that the corpus measurements that set group boundaries and filter sizes exist only from ST-M0, which is measurement) and list it in 9.1. List the ST-M1/ST-M2 split in 9.1 as build order, not as a hold.

### 18. Medium: ST-22 and ST-23 compare numbers that are not the same quantity

**Section:** ST-22, ST-23 (lines 583–584).

**Problem.** "Full-style time ... measured through each engine's own trace categories". Blink's "Recalculate Style", Gecko's Stylo traversal markers and WebKit's style-resolution events cover different work (invalidation, pseudo-elements, layout-triggered restyles, lazy declaration parsing). A ratio of unlike quantities is not falsifiable. M-1 part 4 also prefers black-box measurement to trusting engine internals.

**Required change.** Define the metric black-box. For example: the time from a mutation to the next `getComputedStyle` answer on an element that forces only style (with layout excluded by `display: none` containers or by a style-only property), or end-to-end interaction latency (ST-23 at INP level), with Izcal's internal phase split published separately. Keep the trace-category comparison as information only.

### 19. Medium: Exposure to DevTools and agents is not specified for sensitive style data

**Section:** 6.13 (lines 395–409), 6.16 developers row.

**Problem.** The cascade trace contains declarations from **non-origin-clean sheets** (which the page itself cannot read) and the matched `:visited` rules for each link, which reveal history. Line 409 hands "answers" to the agent principal (B9). DevTools G10 says clients depend only on `hg-inspect-schema`, G11 says only `Redacted<T>` leaves IIP, and the networking spec gates `FailureReason` on "DevTools sessions with read capability on that origin". This spec states no schema, capability or redaction rule for the trace, restyle causes, the parse-decision log or the selector statistics.

**Required change.** Define the IIP types for these outputs, gated by the session's origin capability. Agents see non-origin-clean declarations and visited state only when the user grants it, and never by default. Mark sheet text and visited state as `Redacted<T>` for export. Add the cases to ST-13.

### 20. Medium: User style sheets are a fingerprinting surface

**Section:** 6.16 (line 482), ST-D17.

**Problem.** A **global** user sheet changes computed styles on every site, and `getComputedStyle`, layout geometry and `matchMedia` expose them. A distinctive user sheet is a cross-site identifier, which PRIV-2 (a) measures and D-S1 tries to remove. ST-D17 records no privacy analysis and no reversal trigger.

**Required change.** Add a privacy analysis to ST-D17. Options include: a warning in the shell that a global sheet is observable; per-site sheets as the default; applying user-origin values with the page seeing initial values, as for `:visited`, where that is feasible. Add a PRIV-2 case. Give ST-D17 a reversal trigger.

### 21. Medium: The named limits are incomplete (DoS)

**Section:** 6.14 "Named limits" (lines 431–441), 6.5 XML, ST-12.

**Problem.** Missing limits:
- `@import` **fan-out and total count**. The depth limit is 16, but a sheet importing the same URL twice at each level creates 2¹⁶ sheet instances, each with CSSOM objects;
- selector program size and nesting of `:is()`, `:not()` and `:where()`, and selector-list length;
- invalidation-index size, which grows with `:has()` arguments times features;
- attributes per element (ST-12 tests 10⁵ but sets no limit), declarations per block, and `@layer`, `@container` and `@scope` nesting;
- **XML internal DTD subset entities**. Section 6.5 maps the 11 public identifiers and forbids external entities, but does not say whether internal-subset entity declarations are honoured. If they are, this is the billion-laughs case. If they are not, XHTML and SVG content that uses them breaks. Neither choice is recorded.

**Required change.** Add each limit with a value (†), an overflow behaviour, an anchor and tests at limit−1, limit and limit+1. Record a decision on XML internal entities, with an expansion budget if they are supported. Make ST-12's "published canonical work bound" an actual number per case.

### 22. Medium: Hashing for content addressing is unspecified (determinism versus HashDoS)

**Section:** 6.9 "Deterministic identities", 6.11 attribute sets and intern tables, 6.6 UA image.

**Problem.** Interning depends on hashes of attacker-controlled content. An unkeyed hash invites engineered collisions: slow intern tables, or, if equality is decided by hash, one element's style handed to another. A keyed hash with a per-run key changes table layout, which FP-24 counts. JS 6.9 already resolves this for JS ("one key per process from B1 entropy (CS-AS-2); image tables are perfect hashes"), but this spec says nothing.

**Required change.** Adopt CS-AS-2's keyed process hash from B1 entropy for all style intern tables. Use full-content comparison on every hash match. Use fixed perfect hashes for image-resident UA tables and ancestor-filter keys. State that canonical bytes do not depend on bucket order, or that the key is recorded.

### 23. Medium: The layout interface depends on a spec that does not exist, on the critical path

**Section:** 6.10 container re-entry (line 369), 9.2 ST-M2, 9.5.

**Problem.** ST-M2's exit needs "container queries with the layout spec's re-entry". Under 2.3.0, Taffy and Parley are excluded and the layout spec is "a later spec", so there is no layout to re-enter. The spec does not say whether `style_container_subtree` runs on a layout worker or the script thread, whether it runs its own deterministic merge (finding 4), or how the "affects" mask's categories are agreed with layout.

**Required change.** Name the minimum layout this track needs by ST-M2 (a reference block layout and size containment, for example) and who builds it. Specify the thread and merge semantics of re-entry. Add the dependency to the critical path in 9.2 and the risks in Section 10.

### 24. Medium: The `:visited` reversal trigger and the defence it drops

**Section:** ST-D15, 9.1 row 5.

**Problem.** "A release cycle with no finding: drop the restrictions" treats absence of reports as evidence. Constant work and the 2010 restrictions also protect against same-partition timing differences that partitioning alone does not change (a site learning which of its own links a user clicked through another context in the same partition, including cross-origin frames within the key's frame origin). Dropping them needs an analysis, not just elapsed time.

**Required change.** Make the trigger a positive one: a published analysis showing that every observation possible without restrictions is one the site could make itself under the full key (including the container), plus ST-13 green. Keep constant work regardless unless ST-23 shows its cost.

---

## Low

### 25. Low: G-rule numbering and references

G19–G22 are numbered "after the JS spec's G13–G18", while the networking and fonts specs are being written in parallel and may add rules. ST-8 (a) cites "graph check, G22", but G22 is the fast-path differential rule. The graph check needs its own rule (for example, "only `hg-html-syntax` implements HTML tokenization"). G20 cites "G2 ... the projection oracle", which JS-D4 retired to a debug and Nightly check. **Required change:** reserve the numbers in a shared registry (Stage 1 2.2), add the missing graph rule, and cite G1's dual-implementation rationale instead of G2.

### 26. Low: Crate layering

`hg-cssom` (L3) serves `getComputedStyle`, which needs `hg-style` (L4), an upward dependency. The parsers and `hg-selectors` sit in L2, "contracts", which Stage 1 reserves for interfaces (`hg-tree-traits`, `hg-ipc-gen`, `hg-types`). `hg-encoding` and `hg-math` are built for tier P as well, so they must meet CS-ER-3's "never panics" lint set, which the spec does not say. **Required change:** state the dependency direction (a trait in L3 implemented by `hg-style`, or move resolved-value code into L4), add an "L2p parsers" layer or justify L2, and state that dual-tier crates are written to tier P rules.

### 27. Low: Inconsistent parallel-gain thresholds

ST-26 fails below 1.5×† at 4 workers. ST-D13's reversal trigger is below 1.2×† at ST-M3. All the evidence cited is about 1.3× at 6 threads. **Required change:** use one threshold, justified against [PI 6.2][PI], and one failure action.

### 28. Low: Unfalsifiable phrases in targets

ST-13 "0 detectable differences above noise" (also JS-17 and FP-36) needs a stated test: a statistical test, trial count and significance. ST-12's "published canonical work bound" has no number (see finding 21). ST-20's "first node within 1 ms of the first byte reaching the renderer" depends on what else the event loop is doing. Measure from the start of the parser task, and separately the task-queue delay. **Required change:** define each.

### 29. Low: Over-invalidation metric definition

Line 333 says the reference engine is needed to count "restyled, no group changed". That count only needs old and new style IDs. What the reference adds is **under**-invalidation (elements that changed but were not restyled), which ST-4 already catches. **Required change:** state each metric with its oracle. Also map INC-2 (no frame class slower than from-scratch by more than 1.2×) and INC-4 (fallback rate ≤ 50%†, including `:has()`) to ST targets. The spec cites them but does not adopt them.

### 30. Low: Style columns and slot reuse

Style columns are indexed by node slot. When a slot is freed and reused, its style column, last-match flags and side-table entries must be cleared or generation-checked, and the release-by-ID path (JS 6.5) must notify `hg-style`. **Required change:** state the rule and add a planted-bug case to ST-11.

### 31. Low: Decisions without reversal triggers

ST-D5 ("— classification"), ST-D12 ("—") and ST-D17 ("—") have none. The method asks every decision for the evidence that would reverse it. **Required change:** for example, ST-D12: the reference finds fewer than N† divergences per quarter for a year and its CI cost exceeds X†. ST-D5: an XML-related CVE class in the own parser above a threshold.

### 32. Low: Fonts interface omissions

The fonts spec's interface table expects style to supply `@font-face` **and `@font-feature-values`** rules, and `@font-palette-values` is in CSS Fonts 4. The spec mentions only `@font-face`. The spec also calls font metrics "B1 inputs". The fonts spec makes metrics a deterministic function of the installed Izcal set and the recorded web-font installs. **Required change:** list all three at-rules in the interface and describe metrics as derived from recorded font state.

### 33. Low: Animation interface gaps

The cascade-input interface does not mention scroll-driven animations (timelines read scroll positions from the post-layout snapshot) or compositor-run animations (whose values style must not see as main-thread inputs). **Required change:** name both as inputs the animation spec will define, with their B1 status.

### 34. Low: Staffing realism notes

The observability track (2) owns the cascade trace, restyle causes, selector statistics, IIP integration **and** a bounded selector-intersection solver (an NP-complete problem in general). The reference-engine track is discussed in finding 14. The engineer-year arithmetic checks (115–190). **Required change:** size the static-query solver separately, or move it to a measured follow-up with a reason (measurement), and list it in 9.1.

### 35. Low: Recordings and visited state

Visited sets delivered to a renderer are B1 inputs, so a recording contains a slice of the user's history. Stage 1 3.4 keeps recording off by default because logs contain page content. **Required change:** state that visited inputs in recordings are redacted or replaced by the salted membership answers, in line with SEC-21.

---

## Summary

| Severity | Count | Findings |
|---|---|---|
| High | 8 | 1 (M-1 provenance: engine-source evidence and underived structures), 2 (speculative parser not a recorded decision point; worker interning), 3 (partition-wide tables shared across agents on different threads), 4 (IDs absent during the parallel phase; disjoint `&mut` slices impossible), 5 (lazy declaration parsing mutates shared sheets in parallel), 6 (class (a) arena holds non-derivable state: before-change styles, CSSOM-mutated sheets, inline style), 7 (sheet-sharing key lacks quirks mode and CORS/origin-clean), 8 (`:visited` keyed without container) |
| Medium | 16 | 9–24 |
| Low | 11 | 25–35 |
| **Total** | **35** | |

[HP]: history_postmortems.md
[SP]: specs_semantics.md
[PI]: performance_invalidation.md
[SR]: security_robustness.md
[GO]: gaps_opportunities.md
