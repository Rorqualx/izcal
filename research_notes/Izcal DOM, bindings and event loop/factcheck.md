# Izcal DOM, bindings and event loop: fact-check (phase 8)

Checked 19 October 2026 against `reports/Izcal DOM, bindings and event loop.md` (draft 0.1, 930 lines as of 23:38 local, which includes the content-blocking paragraph in 6.7) and its seven research notes, by a checker who wrote neither. Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8.

**What was re-read.** The WHATWG DOM, HTML (webappapis, timers-and-user-prompts, document-lifecycle, browsers, document-sequences, nav-history-apis, structured-data, workers, interaction, custom-elements, input, form-control-infrastructure, indices, rendering) and Web IDL standards, hr-time, Prioritized Task Scheduling, requestIdleCallback, Page Lifecycle and Service Workers, all downloaded on 19 October 2026 and searched by script. Also whatwg/html and whatwg/dom commit metadata (messages and dates only), `@webref/idl` 3.85.2 (re-parsed with webidl2 to recount the corpus), and MDN browser-compat-data 8.1.6 JSON. Primary sources: the papers as PDFs (Reis et al., PMForce, Loophole, Fuzzyfox, DOM clobbering, BLeak, EventRacer, ESEM 2013, the flaky-tests study); Project Zero posts and RCAs; Chrome, web.dev, WebKit, V8, Mozilla Research and Microsoft Learn pages; WebKit Bugzilla and Mozilla Bugzilla through their XML and REST endpoints (comment text only); the GitHub API (PR 6315; the rustwasm and wasm-bindgen organisations; WPT `interfaces/`); wpt.fyi per-run APIs and summary files; the Interop 2026 CSV; chromestatus popularity data; the Web Almanac 2024 and 2025; the State of HTML 2023–2025 results; crates.io and the npm registry. Finally, every Izcal spec the draft cites (JS 0.3.1, FP 0.3.1, ST 0.3.3, LY 0.4, GR 0.4, NW 0.3.3, MD 0.4, AX 0.2, CB 0.3, DT 0.3, Stage 1 design 0.11, HGBA-2, the Phase 3 ledger).

**Budget and method notes.** The claim checks were split across five parallel fetch workers plus the checker, using about 230 page fetches and no web searches. The webdriver-bidi editor's draft returned GitHub error pages, so its `index.bs` was read instead. The wpt.fyi `ids=` query ignores its parameter, so runs were read one at a time. No engine source code was opened: Chromium, Firefox and Ladybird pages were read only where they are prose documents (`.md` design docs, firefox-source-docs, newsletters), and Project Zero RCAs only for their prose. No personal data was sent in any request.

**Verdicts.** **Correct**; **Partly** (the substance holds but a number, scope, citation, section or wording is off); **Wrong**; **Unverifiable** (the cited source does not contain the claim and no other source was found); **Not checked** (left at the note's own grade). "Must fix" marks a finding that a decision (DM-D1 to DM-D21), a target or a rule (G90–G99) depends on, a broken reference, an internal inconsistency, or an arithmetic error. Every other fix is recommended.

## Summary

There are 194 claim rows: 169 about external facts (Section 3) and 25 about other Izcal specs (Section 4). A row that bundles sub-claims is counted once, at its worst verdict. The 20 internal-consistency checks are counted separately, in Section 5.

| Verdict | Count |
|---|---|
| Correct | 148 |
| Partly | 41 |
| Wrong | 3 |
| Unverifiable | 1 |
| Not checked | 1 |
| **Total** | **194** |

**Links.** All 47 distinct inline URLs return HTTP 200 with the expected page. All 13 `#fragment`s exist on the fetched pages. One of the 24 reference-style definitions is broken: `[IF]`, `izcal_interfaces.md`, does not exist (Section 6). All 322 `[XX n.n]` note citations name rows, sections or open questions that exist.

**M-1 part 3.** No link in the spec, and none of the 492 distinct URLs in its notes, points to another engine's source code. Every `chromium.googlesource.com` and GitHub `blob/` link is a prose `.md` document, MDN BCD data or an Interop README. Every `/commit/` and `/pull/` link is to whatwg/html or whatwg/dom. Bug-tracker pages are discussion pages. One borderline case is listed under Recommended.

**Licences.** These match the registries: `proc-macro2` 1.0.107 and `quote` 1.0.47 (MIT OR Apache-2.0); `weedle` 0.13.1 and `weedle2` 5.0.0 (MIT); `@webref/idl` 3.85.2 (MIT). The `webidl2` conflict is confirmed: npm 24.5.0 says `"W3C"` and the repository's LICENSE is MIT. The spec does not name `web-features` (npm 3.43.0, Apache-2.0) or `@mdn/browser-compat-data` (8.1.6, CC0-1.0) as dependencies. Both are cited only as evidence, which is fine; neither is MIT, so neither may be shipped as data.

**What holds.** The claims that carry the design are confirmed:
- **Spec contracts and anchors.** The DOM insertion and moving contracts, "atomically with respect to script", `moveBefore`'s steps and its 7 March 2025 date, task queues as sets, "update the rendering" as a queued rendering-source task, the timer clamp text, the pause, idle and activation wording, "only ever gains entries", "transferring has side effects", the 13 + 2 cross-origin names, the shared-worker timeouts, the four lifetime predicates, the 33 DOMException names, the 11 `[Legacy*]` attributes and 196 uses, the scoped-registry commit of 14 April 2025, and the July 2026 realm commit.
- **History.** Mutation-event removal (Chrome 127 on 23 July 2024; BCD Firefox 140 and Safari 26), Belshe's clamp story, WebKit 221124's 1–1.7%, Oilpan's 37% and Chrome 66, PR 6315's 469 commits over 21 months, and DOMJIT's 40%.
- **Site Isolation.** 9–13%, under 2.25%, about 450k lines, 56%, 94 UXSS-like bugs, and PMForce's 219.
- **bfcache.** web.dev's 1 in 10 and 1 in 5, and Chrome's 17%/7%, 3 min and 2025 rollout.
- **Usage and conformance numbers.** Every chromestatus counter (36.6, 2.84, 57.9, 0.73, 0.66, 1.19%); the WPT runs and pass rates of Section 3; the Interop 2026 scores (1.9% and 55%); the 1,142 / 8,208 / 1,807 IDL counts; BO's black-box byte figures; MiraclePtr's 57%; Project Zero's 25% variants.

## Must fix

1. **The `[IF]` reference is broken.** The header lists `izcal_interfaces.md` among the research notes, Section 1 says "the full digest is [IF][IF]", the citation key names **IF**, and the definition points to `../research_notes/Izcal%20DOM%2C%20bindings%20and%20event%20loop/izcal_interfaces.md`. That file does not exist; the folder holds `frame.md`, the seven strand notes and `critique.md`. `frame.md` also refers to it twice. Either write the digest, or remove `[IF]`, the "IF" entry in the citation key and the file from the header list, and point Section 1 to `frame.md`.

2. **9.3's engineer-years do not follow from 9.3's own figures.** The spec says "about **230–310†** (about 30 in year one, 55 in year two, then 60–78 a year to Stage 1 exit at 48–60 months†)". The stated components give 30 + 55 + 2 × 60 = **205** at 48 months and the minimum rate, and 30 + 55 + 3 × 78 = **319** at 60 months and the maximum. A fixed "55 in year two" also cannot hold for both ends of a ramp "to full strength by month 18†": with 78 engineers by month 18, year two averages about 65–70. Correct statement: "about **205–320†** engineer-years (about 30 in year one, 55–70 in year two, then 60–78 a year to Stage 1 exit at 48–60 months†)". The other totals add up: tracks 60–78, and 69–90 people at ×1.15.

3. **Section 1 misstates FP-D8, and 6.8 contradicts it.** The Section 1 bullet says "One locked renderer per (site, container), origin-keyed above a RAM threshold (FP-D8)". FP-D8 rejected the RAM threshold: its option (b), "a RAM threshold, as HGBA-2 I-1 (c) now says", lost to option (a), "On for every desktop class, regardless of FP-7" (FP 9.3). Section 6.8 of this spec says it correctly ("(origin, container) wherever FP-D8 keys by origin (every desktop class)"), and DM-D6's argument depends on that reading. Correct statement: "One locked renderer per (site, container), origin-keyed on every desktop class (FP-D8); below I-1's RAM threshold, reduced isolation may co-locate sites (JS-D20)."

4. **DM-D2's "`weedle`'s organisation was archived in 2025" (6.3, 9.4) is misleading.** The rustwasm organisation was archived on 25 August 2025 (GitHub API `archived_at`), ahead of the September plan in the [Inside Rust post](https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org/). But `weedle` was transferred to the active **wasm-bindgen** organisation (`github.com/wasm-bindgen/weedle`, not archived, pushed 20 October 2026), and `weedle` 0.13.1 (MIT) was published on 26 February 2026, after the move ([crates.io](https://crates.io/api/v1/crates/weedle)). Drop this reason from DM-D2's recommendation and from 6.3. Keep the others: a small grammar, typed Izcal attributes and spec-anchored errors. The recommendation itself still stands on M-1 grounds.

5. **DM-D21's two cross-checks are not independent, and the WPT quotation is not verbatim (6.3).** WPT's `interfaces/README.md` says the `.idl` files "are copied from @webref/idl by a workflow that tries to sync the files daily". Comparing Izcal's extraction with both `@webref/idl` and WPT `interfaces/` is therefore one oracle checked twice, not two. The idlharness page says "Web IDL is automatically scraped from specifications and added to the /interfaces/ directory"; it does not contain "scraped automatically from specifications". State that WPT `interfaces/` mirrors `@webref/idl`, keep one of them as the cross-check, and fix the quotation.

6. **The DM-D5 headline rests on three bugs, one of them only partly read.** The summary, 6.16 and DM-D5 say "Every in-the-wild DOM use-after-free read in full for this spec needed author script to run in the middle of an engine algorithm", and that DM-D5 "closes the class behind every in-the-wild DOM UAF read for this spec". SR 3 covers three bugs (3.1, 3.3, 3.4):
   - CVE-2021-26411 (3.4) is known from one sentence of Project Zero's 2021 review, not a full root-cause analysis.
   - CVE-2021-1879's RCA classes it as a plugin script object "not properly tracked by the garbage collector"; the mutation-event listener is the proof of concept's trigger.

   Correct statement: "All three in-the-wild DOM use-after-frees whose analyses were read (CVE-2022-22620, CVE-2021-1879, CVE-2021-26411) were triggered by author script running in the middle of an engine algorithm." Also restate 6.2's "CVE-2021-1879 a mutation-event listener" as "triggered from a mutation-event listener".

7. **G91 and 6.11 credit CVE-2022-22620 to the wrong lost property.** 6.11 says deserialization running no author code "is the property whose loss caused CVE-2022-22620", and G91's rationale names "CVE-2022-22620's regression class". Project Zero's "Autopsy on a Zombie" shows the December 2016 refactor changed a reference-counted argument to a raw pointer ("No one was passing ownership"). The 2022 trigger was a `blur` event during a same-document traversal, a path the 2013 deserialization hardening "did not cover". The lost property is **a strong reference held across a script point**, which is Rule 3 of 6.2, not script-free deserialization. Fix 6.11's sentence. G91 can keep the CVE as an example of a refactor that silently reintroduces a script point; G93's rule ("nothing durable held across a script point except IDs") is the one this CVE motivates.

8. **6.1, 6.2 and G91 extend DOM's script-free contract to the removing steps, but DOM does not.** 6.1 says "the insertion, removing and moving steps 'must not modify the node tree …, create browsing contexts, fire events, or otherwise execute JavaScript'". In [DOM](https://dom.spec.whatwg.org/#concept-node-insert-ext), that sentence belongs to the insertion steps, and the moving steps repeat it ("Like the insertion steps, these steps must not …"). The **removing steps** definition has no such clause; it says only "Specifications may define removing steps … passed a node removedNode, a boolean isSubtreeRoot, and a node oldAncestor". The design choice can stand, since making removing steps script-free is stricter and the remove algorithm queues its script-observable effects. But 6.1, 4.2 and G91's "Why" must not attribute it to the spec. Correct statement: "DOM requires insertion and moving steps not to execute JavaScript; Izcal applies the same rule to removing steps (an Izcal restriction, checked against every specification's removing steps at its pin)".

9. **DM-D1's class 3 contradicts the report it cites.** 6.7 says "Scheduler effective priority 4–5 … as the scheduling report suggests: above most task sources", and class 4 puts effective priority 3 (`user-visible` continuations, including every `scheduler.yield()` continuation by default) among the normal sources. [Prioritized Task Scheduling](https://wicg.github.io/scheduling-apis/) suggests elevating tasks of "effective priority of 3 or higher". Either move effective priority 3 into class 3, or keep it in class 4 and record that as an Izcal deviation from the report's suggestion, with its reason. DM-D1 and DM-41's "orders HTML permits" checker depend on it.

10. **The cross-origin `Window` surface misses named child navigables (6.4; also G92, DM-11, 6.16).** The spec says the 13 properties "plus indexed child frames" and that "everything else throws `SecurityError`". HTML's WindowProxy `[[GetOwnProperty]]` also exposes, cross-origin, any P in the window's "document-tree child navigable target name property set", returning the named child's WindowProxy ([HTML, nav-history-apis](https://html.spec.whatwg.org/multipage/nav-history-apis.html#crossoriginproperties-(-o-)) and the WindowProxy internal methods). Add named child navigables to the remote stub's operations (the frame tree the broker replicates already has the names) and to DM-11 (c)'s fuzzed member set. Otherwise the generated allow-list will be wrong on purpose.

11. **The Section 1 INP figures compare different percentiles, and DM-D1's rationale cites them.** Section 1 says "Input delay dominates poor INP: 155 ms at p90 against 56 ms of processing at p75". The [Web Almanac 2024](https://almanac.httparchive.org/en/2024/performance) says: "At the 75th percentile, input delay reaches 37 milliseconds and processing delay 56 milliseconds", and at p90 "input delay jumps to 155 milliseconds, which makes it the biggest contributor to poor INP", with processing and presentation each over 100 ms. Correct statement: "At p90, input delay (155 ms) is the biggest of INP's three parts; processing and presentation each exceed 100 ms (at p75: 37 ms against 56 ms)". "Dominates" in 6.7's class-1 rationale should become "is the largest part of poor INP".

12. **DM-D14's evidence for shared workers and BroadcastChannel is not in the source (6.9 table).** The row "Shared worker clients, BroadcastChannel, open IndexedDB connections that block another context, Web Locks held" says "Chrome's freezing exemptions show these are the cases that break". [Chrome's freezing post](https://developer.chrome.com/blog/freezing-on-energy-saver) lists calls and capture, open RTCDataChannel or live tracks, WebUSB, Bluetooth, HID and Serial, and "Hold a Web Lock or an IndexedDB connection that blocks operations outside the group". It names neither shared workers nor BroadcastChannel. Cite the source for Web Locks and IndexedDB only, and mark the other two as Izcal's first-principles choice (†).

13. **"Chrome no longer fires `unload` at all" (4.4; DM-D9 (b) "Never fired (Chrome 155)"; 6.9 "Chrome's 'never fire'").** [Chrome's deprecation page](https://developer.chrome.com/docs/web-platform/deprecating-unload) confirms the 100% rollout in Chrome 155 on 6 October 2026, but says `unload` "may still fire on iOS devices", and sites can opt back in. Correct statement: "Chrome no longer fires `unload` by default (Chrome 155, 6 October 2026, except on iOS and for sites that opt in)". DM-D9's comparison of options depends on this.

14. **"Two memory managers for one DOM-and-JS graph failed in four engines" (summary paragraph).** HP 8.1 builds this from IE, Gecko, Blink before Oilpan and Servo. But Servo never had two managers: it gave the JS collector "full responsibility for managing those native-code DOM objects" ([Mozilla Research](https://research.mozilla.org/2014/08/26/javascript-servos-only-garbage-collector/)), and its problem was rooting soundness. Say "**three** engines (IE, Gecko, Blink before Oilpan)", and cite Servo separately for "one collector, with rooting checked only by a lint".

15. **"WPT's `interfaces/` (348 IDL files)" (6.3) is wrong.** The [directory](https://api.github.com/repos/web-platform-tests/wpt/contents/interfaces) has 348 entries: **346 `.idl` files** plus `META.yml` and `README.md`. Say "346 IDL files".

16. **"'Need to detail the native appearance', 11 open markers" (6.12) is wrong.** The live HTML source has **10** `XXX` markers on this topic. Seven read "Need to detail the native appearance and primitive appearance" and three read "Need to detail the primitive appearance". SP 16.6 carries the same error.

## Recommended (not blocking)

**Spec text**
- **4.2 and 6.7, "'update the rendering' … with 17 ordered steps".** The algorithm has 23 top-level steps; 17 are the per-document "For each doc of docs" steps after the filters, with the style-and-layout loop counted as one. Say "17 per-document steps".
- **4.2 and 6.8, "COEP's isolation mode ('none', 'logical', 'concrete')".** These are the values of the **browsing context group's cross-origin isolation mode**. COEP's values are `unsafe-none`, `require-corp` and `credentialless`.
- **6.3, the Legacy behaviour row.** It claims "Every one of the 11 `[Legacy*]` attributes" but lists ten; add `[LegacyUnforgeable]` (19 uses).
- **6.11, "the 33 `[Serializable]` and 16 `[Transferable]` interfaces".** These are 33 `[Serializable]` annotations on **32** distinct interfaces (`FileSystemHandle` is annotated twice in `@webref/idl` 3.85.2). The 16 is right.
- **6.7, "The monotonic clock keeps ticking across system sleep …, as hr-time requires".** hr-time never mentions sleep or suspend. It says only that clocks "attempt to count 1 millisecond of clock time per 1 millisecond of real-world time". Say "as hr-time's clock model implies". The "only Windows engines" part is MDN's hedged "It appears that only browsers on Windows keep ticking during sleep".
- **6.3, overloads "compiled to one type test at the distinguishing index per arity".** At the distinguishing argument index, resolution is an ordered series of type-category tests across the candidate overloads. Say "a type dispatch at the distinguishing index".
- **6.3 and DM-D4, "Gecko and Blink both put side-effect facts in IDL so their JITs can hoist and eliminate DOM calls, and Blink also uses them for DevTools".** That holds for Gecko ([firefox-source-docs](https://firefox-source-docs.mozilla.org/dom/webIdlBindings/index.html): used "by the JIT to perform loop-hoisting and common subexpression elimination"). Blink's documented `[Affects=Nothing]` serves DevTools' `throwOnSideEffect` only. Say "Gecko puts effect facts in IDL for its JIT; Blink puts them in IDL for DevTools' side-effect-free evaluation".
- **6.4, DM-D16 and 6.19, "~20 B per touched node".** That is the V8-heap figure. The same paragraph gives "about 70 B of process memory" (BO 1.9: 797 → 870 B), which is the stake for DM-30's byte target. State both.
- **6.5, "80% of blocking touch listeners never cancel".** The explainer says 80% of touch **events** that block scrolling never prevent it; Chrome's post says about 80% of root-target **listeners** are conceptually passive. Pick one and word it to match.
- **6.12, UA shadow trees "built lazily on first render as WebKit does for `input`".** WebKit says "construct their user-agent shadow tree lazily during construction and cloning". The "first render" trigger is Izcal's choice.
- **Section 3, LoAF "frames ≥ 50 ms".** The source says "greater than 50 milliseconds".
- **Section 3, EventRacer "45.3 filtered races per site".** 45.3 is **variables** with uncovered races per site (down from 634.6). "75 harmful races" is right.
- **6.9, `unload` "11% of all sites and 28% of the top 1,000".** Both figures are desktop (mobile: 10% and 20%). Add "desktop".
- **Section 1, "in-process DOM bugs remain 15–20% of in-the-wild browser 0-days".** Project Zero gives that as the share "in previous years" and notes none were found in WebKit in 2021. Say "were historically 15–20%".
- **2.2, MiraclePtr "is not used in renderers".** True of the 2024 post, which also says "we plan to enable BRP on the renderer process". Add "as of January 2024".
- **Section 5, Fuzzyfox and Fermata: "the only design that closes timing channels".** The paper says browsers "can be architected to mitigate all possible timing side channels" by complete mediation. It does not say this is the only design. Say "a design that closes them".
- **Section 5 and 6.16, fuzzers.** Domino does not appear in SR 7, and Minerva's results are an open question in SR (Q1). Drop Domino or cite it, and drop Minerva from "what made … effective".
- **6.16, "WebAudio and Cache actor UAFs" as cross-thread lifetime.** The Cache bug (CVE-2020-6820) is an IPC actor deleting itself during a close loop, not a cross-thread race. Keep WebAudio, and move Cache to the re-entrancy row or drop it.
- **6.5, user activation, "Chrome's post-Spectre threat model says a gesture tracked in the renderer 'can be forged …'".** The quote is exact, but the document's context is secure gestures for autofill and passwords. Say that the spec generalises it to activation.
- **2.1, Gecko CC "seconds on leaky pages; a decade of pause work".** Neither Bugzilla 698919 nor the CC wiki gives a seconds figure ("unpleasant pause times, especially when things get leaky"). The 2024 closing comment says "No new bugs blocking this one in a decade", so the work ended about 2014. Say "40–120 ms typical pauses, worse on leaky pages; pause work through about 2014".
- **2.1, "Chrome 1.0 timers".** It was the Chrome 1.0 **beta** ([Belshe](https://www.belshe.com/2010/06/04/chrome-cranking-up-the-clock/)).
- **2.1, mutation events "removed in … Firefox 140 and Safari 26".** The inline Chrome link names neither; cite BCD (`api/MutationEvent.json`) beside it.
- **2.1 and 5, service workers "boot costs 50 ms desktop, 250 ms mobile"; "50–500 ms boot on every controlled navigation".** web.dev says boot happens only when the worker "isn't already running", that it is "usually around 50ms", and that on mobile "it's more like 250ms", with 500 ms in extreme cases; "desktop" is not in the source. Say "when the worker is not running: usually about 50 ms, about 250 ms on mobile, over 500 ms in extreme cases".
- **6.8, "Safari removed SharedWorker for seven years".** BCD has it gone from Safari 7 (October 2013) to 16 (September 2022): about **nine** years. 2.1's "(7–15)" is right.
- **2.1, Paris bindings: "each redesign had the same two drivers".** HP 2.1–2.5 grade this Medium, inferred from bug titles (HP Q1). List it in 11.2 or soften it.
- **11.2, "Conflicting: `scheduler.yield()` support in Firefox".** BCD gives Firefox 142 and the Chrome post's badges agree; only the post's body text is stale. Resolve it as "Firefox 142" and drop it from the conflicting list. The `Origin-Agent-Cluster` 88-or-90 conflict is real (web.dev: "Chrome 88 onward"; BCD: 90).

**Cross-references to other Izcal specs**
- **FP section numbers.** "bfcache budgeted in bytes" and "evicted before live tabs are discarded (BF-1, FP 6.9)" belong to **FP 6.10** ("Process lifecycle and bfcache budgets": total ≤ 5%† of RAM and ≤ 256 MiB†, 64 MiB† per (site, container)). FP 6.9 is suspend-to-disk. 9.5's row "Resource-footprint spec 6.9 | Freeze per browsing context group …" should target **FP 6.8** (background policy, where FP-D4's freeze lives). The header's "Sections 6.4, 6.9" should read 6.4, 6.8 and 6.10.
- **NW section numbers.** Service-worker routing is **NW 6.7** ("Fetch in the network service": "A request from a controlled client goes to its partition service, which owns service-worker routing …"); NW 6.8 is partitioning, and `document.cookie` is NW 6.9. NW mentions neither navigation preload nor static routes. 6.10's "navigation preload and static routes are the partition service's (NW 6.8 …)" is a new allocation, so add it to 9.5's networking row.
- **MD 6.6.** The quotation "selection algorithms are HTML's, in `hg-dom`" is in MD's coverage table (**MD 6.20**), which points to 6.6. Cite MD 6.20.
- **GR 9.3 and MD 9.3, "before the DOM, bindings, shell, storage, security, infrastructure and DevTools tracks".** The quoted words are in MD 9.3 only; GR 9.3 gives the same 437–560† total without them.
- **JS 9.2, "Mock host from DM-M0 (JS 9.2's mitigation)"; "as JS 9.2 planned for the bindings track".** JS 9.2's mitigation is that "`hg-js-host` is designed first … and frozen early, so bindings work proceeds against it". A mock host is this spec's addition, so say so.
- **Header versions are stale.** Layout is now **0.4**, graphics and rendering **0.4**, images and media 0.4, accessibility **0.2**, and content blocking 0.3. JS 0.3.1, ST 0.3.3, NW 0.3.3 and DT 0.3 are right. The cited IDs are unchanged.
- **6.12, LY-D10 and single-engine features.** The row admits Page Lifecycle `freeze`/`resume`, and 6.9 and 6.15 rely on LoAF and `notRestoredReasons`. All three are Chromium-only, so LY-D10's rule alone would hold them. Layout handled the same situation with an explicit **owner exception** (LY 6.15, "Admission exceptions"). List these three as owner exceptions in 9.4 (or under DM-D15) instead of "admitted".

**M-1 (borderline, not a violation)**
- **2.1's DOMJIT link, [WebKit bug 162544](https://bugs.webkit.org/show_bug.cgi?id=162544).** This bug is the patch-review thread that landed DOMJIT. The 40% figure is in the patch author's prose comment, and only comment text was read. Bug pages are accepted as discussion in the other specs' fact-checks, but code review is excluded by M-1 part 3. Prefer a prose source for DOMJIT if one exists (BO notes that webkit.org has no DOMJIT post), or keep the link with a note that only the description comment is cited.

**Note fixes (the spec does not repeat these)**
- SP 16.6: 11 markers → 10.
- SP 2.4: "17 steps" → 17 per-document steps of 23.
- NA 2.7: Chrome 106 was named in the 4 February 2022 update, not the January announcement.
- NA 5.7: the BroadcastChannel partitioning versions (Chrome 115, Firefox 103) come from [Chrome's storage-partitioning page](https://developer.chrome.com/docs/privacy-sandbox/storage-partitioning) and MDN's State Partitioning guide, not from BCD.
- HP 6.3: "desktop" is not in the source.
- BO 5.5: rustwasm archived on 25 August 2025; weedle moved to wasm-bindgen.
- BO 4.5: Blink's doc does not say one IDL fact serves the JIT and the debugger.
- SR 3 implications: three bugs, with 3.4 not read in full.
- GO 5.6: variables, not races.

## 3. Claim table: external facts

| # | Claim (section) | Source checked | Verdict | Note |
|---|---|---|---|---|
| 1 | "Two memory managers … failed in four engines" (summary) | HP 8.1; Mozilla Research 2014 | Partly | Three: Servo had one collector (Must fix 14) |
| 2 | Mutation events "Deprecated 2011" (2.1) | HP 3.1; Chrome blog | Correct | "deprecated since 2011" |
| 3 | Removed in Chrome 127 (July 2024) (2.1) | HP 3.2; Chrome blog | Correct | Stable on 23 July 2024 |
| 4 | Removed in Firefox 140 and Safari 26 (2.1) | HP 3.2; BCD MutationEvent.json | Partly | Right per BCD; the linked Chrome post names neither |
| 5 | Quotes "slow", "fire too often", "event listeners can change the entire DOM underneath a running DOM operation" (2.1) | Chrome blog | Correct | Verbatim |
| 6 | "Removal took 13 years" (5) | HP 3.2 | Correct | 2011 → 2024 in Chrome; about 14 in Firefox and Safari |
| 7 | IE6 leaks until process exit, IE7 until navigation; IE8 lets JScript own DOM objects (2.1) | HP 1.1–1.4; Microsoft Learn dd361842 | Correct | Quotes match |
| 8 | Gecko CC 40–120 ms typical pauses (2.1) | HP 1.6; Bugzilla 698919 REST | Correct | "40ms to 120ms ish cycle collector time" |
| 9 | Gecko CC "seconds on leaky pages", "a decade of pause work" (2.1) | HP 1.6; 698919; CC wiki | Partly | No seconds figure; "No new bugs blocking this one in a decade" (Recommended) |
| 10 | Chrome 1.0 dropped 10 ms → 1 ms, broke NYT, settled on 4 ms (2.1) | HP 4.1–4.2; Belshe 2010 | Correct | It was the 1.0 beta |
| 11 | 4 ms nesting clamp survives in HTML (2.1) | HTML timers | Correct | "nestingLevel is greater than 5, and timeout is less than 4" |
| 12 | 1 ms clamp reordered timers; 1–1.7% on Speedometer (2.1, 6.7) | HP 4.4; WebKit 221124 XML | Correct | About 1% (M1, 2021); 1.7% (M1 Air) and 1.2% (Intel) in 2022 |
| 13 | Safari removed SharedWorker (7–15), restored in 16 (2022) during multiprocess bring-up (2.1) | HP 6.2; WebKit 149850; BCD | Correct | "removed temporarily during multiprocess bring-up" |
| 14 | "Safari removed SharedWorker for seven years" (6.8) | BCD; WebKit 140344 | Partly | About nine years for users (2013–2022) |
| 15 | WebKit2 and Paris bindings: "same two drivers" (2.1) | HP 2.1–2.5 (Medium) | Not checked | Inferred from bug titles in the note |
| 16 | History API 2010; state lost, joint history, no interception (2.1) | HP 5.1; WICG README; BCD | Correct | Quotes match |
| 17 | Navigation API 2022–2026 in all three engines (2.1) | HP 5.2; BCD | Correct | Chrome 102, Firefox 147, Safari 26.2 |
| 18 | rAF 2011 (2.1) | HP 4.7; BCD | Correct | Prefixed in Firefox 4 and Chrome 10 |
| 19 | MutationObserver 2012; AppCache removed 2021 (2.1) | HP 3.2, 3.7; BCD; web.dev | Correct | Removal complete 5 October 2021 |
| 20 | Oilpan: 37% of collections over 50 ms on a Nexus 7, the 2014 blocker (2.1) | HP 1.8; Oilpan deck | Correct | "37% … take more than 50 ms … Looks unacceptable" |
| 21 | Exact retaining paths only from Chrome 66; object grouping leaked (2.1) | HP 1.11; v8.dev | Correct | "As of Chrome 66" |
| 22 | Servo lint "will miss some mistakes"; performance left open (2.1, 5) | HP 1.12; Mozilla Research | Correct | Verbatim |
| 23 | Service-worker boot 50 ms desktop, 250 ms mobile, over 500 ms (2.1) | HP 6.3; web.dev navigation preload | Partly | "Desktop" is not in the source; boot only if not running |
| 24 | "50–500 ms boot on every controlled navigation" (5) | NA 4.4; same | Partly | Only when the worker is not running |
| 25 | rIC capped at 50 ms; keep input under 100 ms; not in Safari (2.1) | HP 4.8; Chrome blog; BCD | Correct | Chrome 47, Firefox 55 |
| 26 | `Object.observe` withdrawn because frameworks moved on (2.1) | HP 3.3; es-discuss | Correct | Klein, 2 November 2015 |
| 27 | DOMJIT about 40% on a micro-benchmark; `firstChild` is a field read (2.1) | HP 2.6; WebKit 162544 | Correct | "super stupid tiny micro benchmark … 40%" (see M-1 note) |
| 28 | Site Isolation 9–13% memory, under 2.25% load latency (2.1) | NA 1.1; Reis et al. | Correct | Abstract |
| 29 | 450k lines over five years (2.1) | NA 1.10; Reis et al. §4 | Correct | "approximately 450,000 lines of code" |
| 30 | 56% of navigations cross processes (2.2) | NA 1.3; Reis et al. §5.3.1 | Correct | "from 5.73% to 56.0%" |
| 31 | 94 UXSS-class bugs moot by construction, 2014–2018 (1, 5, 6.16) | SR 5.1; Reis et al. §5 | Correct | "94 UXSS-like bugs" |
| 32 | `isInputPending` no longer recommended; can miss input; one engine (2.1, 6.7) | HP 4.11; web.dev; BCD | Correct | Chrome 87 only |
| 33 | `postTask`/`yield` in Chrome and Firefox, not Safari; LoAF Chromium-only (2.1) | HP 4.12–4.13; BCD | Correct | postTask 94/142; yield 129/142; LoAF 123 |
| 34 | PR 6315 merged after 469 commits and 21 months (2.1, 5) | HP 5.3; GitHub API | Correct | 2021-01-20 → 2022-10-31 |
| 35 | Chrome 115: `document.domain` no-op by default; slipped four times (2.1) | NA 2.7; Chrome blog (immutable-document-domain) | Correct | 106 → 109 → 112 → suspended → 115 |
| 36 | `?0` opt-out; Chrome 115 setter no-op (6.8) | NA 2.6; Chrome blog (setter deprecation) | Correct | "As a last resort, send the Origin-Agent-Cluster: ?0 header" |
| 37 | WHATWG text keys by site by default; only `?1` counts (2.1, 6.8, 11.2) | SP 6.3, 6.6; HTML | Correct | "values that are not … `?1` … will be ignored" |
| 38 | `Origin-Agent-Cluster` Chrome 88 (web.dev) or 90 (BCD) (11.2) | NA 2.4; web.dev; BCD | Correct | The conflict is real |
| 39 | `unload` deprecation: top-50 sites from 2025, 100% in Chrome 155 on 6 October 2026 (2.1) | SP 10.2, NA 3.4; Chrome docs | Correct | Chrome 135–142 for top sites |
| 40 | "Chrome no longer fires it" / "never fire" (2.2, 4.4, DM-D9) | Chrome docs | Partly | iOS exception and opt-in (Must fix 13) |
| 41 | `moveBefore`, scoped registries, Sanitizer, Navigation API are Interop 2025–2026 focus (2.1) | GO 2.9; Interop READMEs | Correct | — |
| 42 | Back/forward are 1 in 10 (desktop) and 1 in 5 (mobile) navigations (1) | NA 3.1; web.dev bfcache | Correct | Verbatim |
| 43 | `no-store` blocked 17% mobile and 7% desktop history navigations (1, 6.9) | NA 3.2; Chrome bfcache-ccns | Correct | Verbatim |
| 44 | Chrome's no-store eviction rules at 100% in 2025; 3 min (6.9, 11.2) | NA 3.2; same | Correct | "March and April 2025"; 3 min "from 10 minutes" |
| 45 | `notRestoredReasons` Chrome 123, cross-origin masked (3) | NA 3.5; Chrome docs | Correct | — |
| 46 | Firefox desktop: an `unload` listener blocks bfcache (6.9) | NA 3.3; web.dev | Correct | Verbatim |
| 47 | Prerendering runs script in a hidden context (6.9) | NA 3.8; Chrome prerender docs | Correct | "invisible background tab" |
| 48 | `:visited` partitioned by (link URL, top-level site, frame origin), Chrome 136 (6.9) | NA 3.11; Chrome blog | Correct | — |
| 49 | Chrome places SWs "in the same process as a document that is likely to rely on them"; Firefox `webServiceWorker=$SITE` (6.8) | NA 4.8; Chromium process-model doc (prose); firefox-source-docs | Correct | Verbatim |
| 50 | Static routes: at most 1,024 conditions, depth 10 (6.10) | NA 4.5; Service Workers ED | Correct | "cannot exceed 1024 … limited to 10 levels" |
| 51 | Chrome documents 30 s SW idle only for extensions (6.10) | NA 4.7; Chrome extensions docs | Correct | Absence for web SWs not proven |
| 52 | PMForce: 219 sites trivially exploitable (5, 6.16) | NA 5.2; PMForce PDF | Correct | "219 sites can be trivially exploited" |
| 53 | Chrome relays cross-process `postMessage` through the browser process (6.11) | NA 5.6; Reis et al. | Correct | "extra IPC hop through the browser process" |
| 54 | BroadcastChannel partitioned in Chrome 115 and Firefox 103 (6.11) | NA 5.7; Chrome and MDN pages | Correct | Not from BCD as the note says |
| 55 | Freezing per browsing context group, as Chrome (6.14) | NA 7.4, 7.6; Chrome blog | Correct | "Freezing will operate on browsing context groups" |
| 56 | DM-D14: shared workers, BroadcastChannel, IDB, Web Locks per Chrome's freezing exemptions (6.9) | NA 7.4; Chrome blog | Partly | Only Web Locks and blocking IDB are listed (Must fix 12) |
| 57 | WebKit site isolation status 2026 (Q-DM-14) | NA 1.18 (Low); Safari 26.6 and 27.0 posts | Unverifiable | Neither post mentions it; correctly left open |
| 58 | Surma: payload thresholds; no isolated measure short of a trace (3, DM-25) | NA 5.4, GO 5.12; surma.dev | Correct | "short of running a trace" |
| 59 | Insertion, removing and moving steps "must not … execute JavaScript" (4.2, 6.1, G91) | SP 11.2; DOM | Partly | Not stated for removing steps (Must fix 8) |
| 60 | Batch inserted "atomically with respect to script"; script in post-connection steps (6.1) | SP 11.2; DOM | Correct | Verbatim |
| 61 | `moveBefore` runs pre-remove and moving steps, `connectedMoveCallback`; 7 March 2025 (6.1) | SP 11.3; DOM #move; whatwg/dom eaf2ac7044 | Correct | Commit dated 2025-03-07 |
| 62 | Agent cluster "an idealized 'process boundary'", stricter than real processes (4.2, 6.8) | SP 6.2; HTML | Correct | Verbatim |
| 63 | Task queues are sets (4.2, 6.7) | SP 1.2; HTML | Correct | Verbatim |
| 64 | Several window event loops may share a thread; `[[CanBlock]]` agents need threads (6.0, 6.7) | SP 1.1; HTML | Correct | — |
| 65 | "Update the rendering" queued on the rendering task source (6.7) | SP 2.2; HTML | Correct | Verbatim |
| 66 | "17 ordered steps" (4.2) | SP 2.4; HTML | Partly | 23 top-level; 17 per-document steps |
| 67 | Microtask checkpoint: rejected promises, IDB cleanup, ClearKeptObjects (6.7) | SP 3.1; HTML | Correct | — |
| 68 | Microtasks between listeners of user-dispatched, not script-dispatched, events (6.7) | SP 3.2; HTML "clean up after running script" | Correct | Follows from the empty-stack rule |
| 69 | Spin the event loop queues a continuation on the same task source (6.7) | SP 3.3; HTML | Correct | — |
| 70 | Pause is "highly detrimental"; alternatives permitted (6.7) | SP 3.4; HTML | Correct | Verbatim |
| 71 | Timers: negative → 0; nesting > 5 and < 4 ms → 4 ms (6.7) | SP 4.1; HTML | Correct | Verbatim |
| 72 | Optional power padding; "fully active for N ms" (6.7, 6.14) | SP 4.3; HTML | Correct | "fully active for a further milliseconds milliseconds" |
| 73 | Idle period deadline capped at 50 ms (6.7) | SP 2.7; HTML; rIC | Correct | Both specs |
| 74 | Transient activation "at most a few seconds" (6.5, Q-DM-4) | SP 4.7; HTML | Correct | Verbatim |
| 75 | Activation-triggering event list (6.5) | SP 4.7; HTML | Correct | Verbatim |
| 76 | Historical key map "only ever gains entries" (6.8) | SP 6.5; HTML | Correct | Verbatim |
| 77 | Site-keyed unless `?1`, cross-origin isolated or historical map (6.8) | SP 6.3, 6.6; HTML | Correct | — |
| 78 | "COEP's isolation mode ('none', 'logical', 'concrete'); only concrete grants" (6.8) | SP 6.5, 6.9; HTML | Partly | It is the BCG's cross-origin isolation mode |
| 79 | `unload` fires only if not salvageable; `pagehide` persisted (4.4, 6.9) | SP 10.2; HTML | Correct | Verbatim |
| 80 | Child navigables unloaded "in what order?" (6.9, Q-DM-12) | SP 10.4; HTML | Correct | Three places |
| 81 | beforeunload prompt needs five conditions incl. sticky activation (6.9) | SP 9.5; HTML | Correct | — |
| 82 | Cross-origin Window: 13 names plus indexed frames; Location `href` setter and `replace`; "everything else throws" (6.4) | SP 7.1; HTML | Partly | Named child navigables also exposed (Must fix 10) |
| 83 | `then`, `@@toStringTag`, `@@hasInstance`, `@@isConcatSpreadable` return undefined (6.4) | SP 7.2; HTML | Correct | Verbatim |
| 84 | Shared worker between-loads timeout "might be 5 seconds" (6.10) | SP 17.2; HTML | Correct | Verbatim |
| 85 | Extended lifetime 10 s to 5 min, not above the SW limit (6.10) | SP 17.2; HTML; whatwg/html 9c009049e4 | Correct | Commit 2026-03-06 |
| 86 | Four worker-lifetime predicates (6.10) | SP 17.1; HTML | Correct | — |
| 87 | Service Workers spec sets no idle number (6.10) | SP 17.4; SW ED | Correct | — |
| 88 | Worklets: at least two global scopes (6.7) | SP 17.5; HTML | Correct | Applies to specs that allow implementation-defined behaviour |
| 89 | "Transferring has side effects" (6.11) | SP 8.2; HTML | Correct | Verbatim |
| 90 | 33 `[Serializable]` and 16 `[Transferable]` interfaces (6.11) | SP 8.3; @webref/idl 3.85.2 | Partly | 33 annotations on 32 interfaces |
| 91 | 33 DOMException names (4.2, 6.3) | SP 15.2; Web IDL | Correct | 33 rows |
| 92 | 11 `[Legacy*]` attributes, ~196 uses (4.2, 6.3) | SP 15.3; Web IDL; webref | Correct | 196 counted |
| 93 | "Every one of the 11" in the legacy row's list (6.3) | Same | Partly | The list has 10; `[LegacyUnforgeable]` missing |
| 94 | Platform object realm "can change after it has been created" (6.3) | BO 3.11; Web IDL | Correct | Verbatim |
| 95 | HTML July 2026 "Create nodes in the realm of the document …" (6.3) | SP 15.7; whatwg/html 85f453b854 | Correct | 2026-07-17 |
| 96 | Scoped registries landed in HTML and DOM on 14 April 2025 (6.6) | SP 14.3; commits f2d8b88655, 388779bfbd | Correct | Both 2025-04-14 |
| 97 | 22 input types, four value modes, ten validity flags (4.1, 6.12) | SP 16.3–16.4; HTML | Correct | — |
| 98 | 113 HTML elements (6.3) | SP 16.1; HTML index | Correct | Excluding `math` and `svg` |
| 99 | "Need to detail the native appearance", 11 markers (6.12) | SP 16.6; HTML source | Wrong | 10 markers (Must fix 16) |
| 100 | Page Lifecycle: 2022 WICG report, one engine (6.12) | SP 5.5; WICG | Correct | "Draft Community Group Report, 9 June 2022" |
| 101 | Scheduler priorities 0–5; continuations above tasks of same priority (6.7) | SP 5.2–5.3; WICG scheduling | Correct | — |
| 102 | User-blocking = effective 4–5 "as the scheduling report suggests" (6.7) | SP 5.3; WICG scheduling | Partly | Report suggests ≥ 3 (Must fix 9) |
| 103 | hr-time coarsening 100 µs / 5 µs (6.7, DM-14) | SP 4.5; hr-time #dfn-coarsen-time | Correct | Verbatim |
| 104 | Monotonic clock ticks across sleep "as hr-time requires" (6.7) | EL 4.3; hr-time | Partly | hr-time does not mention sleep |
| 105 | Only Windows engines tick across sleep (6.7) | EL 4.3; MDN | Correct | MDN hedges "It appears" |
| 106 | Default-passive for touchstart, touchmove, wheel, mousewheel on window, document, root, body (6.5) | SP 13.2; DOM | Correct | — |
| 107 | `getElementById` returns the first in tree order (6.1) | BO 7.4; DOM | Correct | Verbatim |
| 108 | DOMString "might include unmatched surrogate code points" (6.1) | BO 7.8; Web IDL | Correct | Verbatim |
| 109 | WTF-8 as the rejected alternative (6.1) | BO 7.9; wtf-8.codeberg.page | Correct | — |
| 110 | Same `HTMLCollection` may be returned for the same (root, filter) (6.1) | BO 7.1–7.3; DOM #concept-getelementsbyclassname | Correct | — |
| 111 | Overloads "compiled to one type test at the distinguishing index" (6.3) | BO 6.2; Web IDL | Partly | An ordered series of type tests |
| 112 | WPT `interfaces/` is "scraped automatically from specifications" (6.3) | BO 5.11; idlharness docs; WPT README | Partly | Not verbatim; copied from @webref/idl (Must fix 5) |
| 113 | WPT `interfaces/` has 348 IDL files (6.3) | GitHub API | Wrong | 346 (Must fix 15) |
| 114 | `weedle`'s organisation (rustwasm) archived in 2025 (6.3, DM-D2) | BO 5.5; GitHub API; Inside Rust | Partly | weedle moved to wasm-bindgen; 0.13.1 after (Must fix 4) |
| 115 | Stage 1 5.3 chose `weedle` 0.13 (MIT) (6.3) | Stage 1 design 5.3; crates.io | Correct | Stage 1 does not state MIT; MIT is right |
| 116 | `webidl2.js`: MIT in repo, "W3C" in npm (6.3, 7, Q-DM-13) | BO 5.1; npm; LICENSE | Correct | Confirmed |
| 117 | `@webref/idl` MIT; `proc-macro2` and `quote` MIT OR Apache-2.0 (6.3, 7) | crates.io; npm | Correct | — |
| 118 | Chrome per-node cost: 89 B unrendered, ~800 B rendered, `id`+`class` +268 B (4.4, 6.4, 6.14) | BO 1.6–1.8 (researcher's black-box) | Correct | Internally consistent (797 B; 357 − 89) |
| 119 | Wrapper 17–22 B JS heap, ~70 B process (6.4) | BO 1.9 | Correct | 22 B includes a 4 B slot |
| 120 | "About 20 B per touched node" as the stake (6.4, DM-D16) | BO 1.9–1.10 | Partly | Process stake is ~70 B |
| 121 | Chrome collects unmodified wrappers (inferred) (6.4, 11.2) | BO 1.10 | Correct | Flagged as inferred |
| 122 | Blink: inline main-world slot and per-world maps (6.4) | BO 3.1–3.2; V8BindingDesign.md (prose) | Correct | — |
| 123 | Ladybird 2026 lazy wrappers and per-world wrappers (5, 6.4) | BO 3.9; Ladybird newsletter 2026-08-31 | Correct | Newsletter, not code |
| 124 | Gecko and Blink put effect facts in IDL for their JITs; Blink also for DevTools (6.3) | BO 4.1–4.5; firefox-source-docs; IDLExtendedAttributes.md | Partly | Blink's are documented for DevTools only |
| 125 | 80% of blocking touch listeners never cancel (6.5) | BO 8.1–8.2; WICG explainer; Chrome blog | Partly | Events, or root-target listeners |
| 126 | WebKit builds `input` UA shadow trees lazily on first render (6.12) | BO 2.4; WebKit Speedometer 3 post | Partly | "lazily during construction and cloning" |
| 127 | js-framework-benchmark: script 1.8 of 20.6 ms (5) | BO 10.3; krausest results, Chrome 154 | Correct | vanillajs-keyed create |
| 128 | No standard defines direct Wasm DOM access (9.1) | BO 9.3–9.4; Mozilla Hacks 2026-02; interface-types archived | Correct | — |
| 129 | Input delay 155 ms at p90 against 56 ms processing at p75 (1) | EL 3.3; Web Almanac 2024 | Partly | Mixed percentiles (Must fix 11) |
| 130 | Input delay "dominates" poor INP (6.7 class 1) | EL 3.3; Web Almanac 2024 | Partly | "the biggest contributor" |
| 131 | LoAF: frames ≥ 50 ms, invoker attribution, forced style and layout (3) | EL 3.4; Chrome docs | Partly | "> 50 ms" |
| 132 | `scheduler.yield()` Firefox support conflicting (11.2) | EL 1.6; BCD; Chrome blog | Partly | Resolvable: Firefox 142 |
| 133 | "Every in-the-wild DOM UAF read in full needed author script mid-algorithm" (summary, DM-D5, 6.16) | SR 3; Project Zero RCAs | Partly | Three bugs; one not read in full (Must fix 6) |
| 134 | In-process DOM bugs "remain" 15–20% of 0-days (1, 6.16) | SR 2.2; Project Zero 2022 | Partly | Historical share |
| 135 | MiraclePtr covers about half of UAFs, not in renderers (2.2) | SR 1.4–1.6; Google security blog | Correct | 57%; renderer planned |
| 136 | 70% of Chromium high-severity bugs memory safety, half UAF (6.16) | SR 1.1; chromium.org | Correct | — |
| 137 | CVE-2022-22620: 2013 fix undone by 2016 refactor to a raw pointer (5) | SR 3.1–3.2; Project Zero zombie post | Correct | "reintroduced 3 years later" |
| 138 | CVE-2022-22620 caused by loss of script-free deserialization (6.11, G91) | SR 3.1–3.2, 9.1 | Partly | Lost property is the held reference (Must fix 7) |
| 139 | CVE-2021-1879 "a mutation-event listener" (6.2) | SR 3.3; RCA | Partly | GC-tracking bug; listener is the trigger |
| 140 | CVE-2021-26411 "a user controlled callback in between two actions using an object" (6.2) | SR 3.4; Project Zero 2021 review | Correct | Verbatim |
| 141 | Renderer gesture "can be forged by web content that compromises the renderer" (6.5) | SR 5.5; Chromium side-channel threat model (prose) | Correct | Context is autofill gestures |
| 142 | Renderer-enforced sandbox flags bypassable (5, 6.0) | SR 5.2; Reis et al. | Correct | — |
| 143 | 25% of 2020's 0-days were variants (6.16) | SR 2.9; Project Zero | Correct | Verbatim |
| 144 | Loophole: 25 µs monitoring, 75% page identification (6.16) | SR 8.6; USENIX Sec 2017 | Correct | — |
| 145 | DOM clobbering: 9.8% of top 5K; 10.5% of pages would break (6.16) | SR 6.3–6.4; S&P 2023 | Correct | — |
| 146 | DOMPurify bypasses via depth and namespaces; 512 flattening in all three engines (5, 6.16, 11.2) | SR 6.1–6.2; mizu.re | Correct | Single source, flagged |
| 147 | Chrome "no longer pursuing" removal of cross-origin dialogs (6.12) | SR 10.7; chromestatus API | Correct | — |
| 148 | HTML permits killing scripts without `finally` (6.7) | SR 11.2; HTML | Correct | — |
| 149 | Fuzzyfox/Fermata: mediation "the only design" (5) | SR 8.10; USENIX Sec 2016 | Partly | Not "only" |
| 150 | Domato, FreeDom, Favocado, Domino, Minerva effective; coverage underperformed (5, 6.16) | SR 7 | Partly | Domino absent; Minerva unverified |
| 151 | Trusted Types: 130 services, no client-side XSS (11.2) | SR 6.7; Roth et al. | Correct | Second-hand, flagged |
| 152 | WebAudio and Cache actor UAFs as cross-thread lifetime (6.16) | SR 4.1, 4.3; RCA CVE-2020-6820 | Partly | Cache is not cross-thread |
| 153 | Unretained-pointer UAFs (G93) | SR 4.4; MiraclePtr post | Correct | — |
| 154 | Mobile INP good on 77% of sites, 63% of top 1,000 (1) | GO 4.6; Web Almanac 2025 | Correct | Verbatim |
| 155 | BLeak mirrored the DOM, wrapped `addEventListener` (1) | GO 3.3; BLeak §5.3.2 | Correct | — |
| 156 | 40% of flaky JS tests from concurrency and async waits (1) | GO 6.1; arXiv 2207.01047 | Correct | 144 of 365 = 40.3% |
| 157 | 65% of client-side JS faults involve the DOM (1) | GO 1.1; ESEM 2013 | Correct | Verbatim |
| 158 | Popover first interop blocker 2023–2025; web components top five 2025 (1) | GO 1.4–1.8; State of HTML | Correct | Web components 5th |
| 159 | Scoped registries 1.9%, dialogs and popovers 55% (October 2026) (1) | GO 2.10; Interop 2026 CSV 2026-10-17 | Correct | Stable-channel scores |
| 160 | IDL corpus: 1,142 interfaces, 8,208 members, 1,807 in DOM and HTML (1, 4.4) | GO 8.1–8.2; recount of @webref/idl 3.85.2 | Correct | "Members" includes constructors and iterable declarations |
| 161 | Playwright MCP waits only for text or a fixed time (1) | GO 7.4; playwright-mcp README | Correct | — |
| 162 | WebDriver BiDi: no DOM-mutation event, no quiescence (1, 6.15) | GO 6.8; webdriver-bidi `index.bs` | Correct | "mutation" occurs 0 times |
| 163 | EventRacer: 45.3 filtered races per site, 75 harmful (3) | GO 5.6; OOPSLA 2013 | Partly | 45.3 variables |
| 164 | WPT: `dom/` ~98% Servo and Ladybird; `html/browsers/` 63.6–86.2%; SW 7.2%/34.3%; navigation 0.5%/63.8% (3) | GO 8.5; wpt.fyi runs and summaries | Correct | All five run IDs exist, 19–20 October 2026 |
| 165 | Interop 2026 web-compat area names the event-loop processing model (3) | GO 2.9; interop 2026 README | Correct | Verbatim |
| 166 | `document.all` 36.6%, sync XHR 2.84%, `beforeunload` 57.9%, mutation listeners 0.73%, `document.domain` 0.66%, `execCommand` 1.19% (3, 4.4, 6.8, 6.12) | GO 8.7, 5.9, 8.8, 5.13; chromestatus | Correct | 2026-10-18 single-day values |
| 167 | `document.write` on 12% of mobile pages (3, 6.12) | GO 8.9; Web Almanac 2024 | Correct | 13% desktop |
| 168 | `unload` on 11% of sites, 28% of top 1,000 (6.9) | GO 5.8; Web Almanac 2025 | Partly | Desktop figures |
| 169 | Playwright two-stable-frames heuristic (3) | GO 6.5; Playwright docs | Correct | Verbatim |

## 4. Claims about other Izcal specs

| # | Claim (section) | Source checked | Verdict | Note |
|---|---|---|---|---|
| 170 | JS-D4: node columns outside the cage, lazy wrappers held strongly, text arena as external strings, release by ID at event-loop points (1, 6.1) | JS 9.4 JS-D4; JS 6.5 finalization | Correct | — |
| 171 | JS-D15: timing-dependent decisions are recorded B1 inputs (1, 6.7) | JS-D15 (b) | Correct | — |
| 172 | JS-D17 one cage per process, agent heaps in own blocks; JS-D18 branded handles; JS-D13 jitless Izcal-only renderers; JS-D20 reduced-isolation hardening; JS-D12 NaNs (1, 6.8, DM-40) | JS 9.4 | Correct | — |
| 173 | JS 6.2: `WindowProxy`, `Location`, legacy platform objects, `[[IsHTMLDDA]]` as shapes with host hooks; `hg-webidl-codegen` emits shape families and IC generators (1) | JS 6.2 "Host exotic objects are first-class" | Correct | — |
| 174 | JS 6.12: "Window agents of the same (site, container) in one process may share the script thread" (9.5) | JS 6.12 | Correct | Verbatim |
| 175 | JS 6.14: `hg-js-host`, host hooks, jobs owned by host, realms, typed serializer interface (1, 6.3, 6.11) | JS 6.14 | Correct | — |
| 176 | JS G6: only `hg-dom`, `hg-bindings`, `hg-script`, `hg-inspect-renderer` may depend on `hg-js-host` and `hg-heap` (1, 6.17) | JS G6 | Correct | — |
| 177 | JS G16 freshness, JS G18 checked arithmetic, JS-10 planted bugs and ≥ 20 variants, JS-52 ≤ 1%, JS 6.4 assumption ledger, JS 6.13 Latin-1/UTF-16 strings, CS-D3 `TokenCell` (6.2, 6.11, 6.17, 8) | JS 6.4, 6.5, 6.13, G16, G18, JS-10, JS-52 | Correct | — |
| 178 | JS 9.2's mitigation is a mock host (9.2, 10) | JS 9.2 | Partly | JS freezes `hg-js-host` early; mock host is this spec's |
| 179 | FP-11: columns ≤ 48 B† inside 72 B†; 512 B† total (1, 6.1, 6.14, DM-30) | FP-11; ST 6.11 split 72/80/360 | Correct | 72 + 80 + 360 = 512 |
| 180 | FP-D3 never splits an agent cluster; FP-D4 freeze after 30 s†; FP-34 768 KiB†; FP-4 64 KiB† per realm; FP-32 threads (1, 6.8, 6.14, 8) | FP 9.2–9.3 | Correct | — |
| 181 | FP-D8: "origin-keyed above a RAM threshold" (1) | FP-D8 | Wrong | Every desktop class (Must fix 3) |
| 182 | FP 6.3: control thread receives IPC, pressure and freeze; 256 KiB worker stack (6.0, 6.10) | FP 6.3 table | Correct | — |
| 183 | bfcache byte budget and eviction order at FP 6.9 (1, 6.9, 9.5) | FP 6.8–6.10 | Partly | FP 6.10 (bfcache), 6.8 (freeze) |
| 184 | ST: `DomSink`, `Yield` call-outs, ID re-validation (ST 6.3); 512 cap (ST 6.2); 32 bits element state, attribute sets per partition (ST 6.11); `:visited` source (ST 6.7.1); environment snapshot (ST 6.10); ST-D4; ST G21, G23; ST-8, ST-40 (1, 6.1, 6.5, 6.16, 6.17, DM-5) | ST 6.2–6.11, 9.4, rules, targets | Correct | — |
| 185 | LY: `hg-layout-query` owns CSSOM View, RO, IO, forcing table; `hg-scroll` scroll offsets; LY 6.14 hit testing; LY-D10 admission rule; LY-D11 UA control metrics (1, 6.5, 6.7, 6.12, 6.18) | LY 6.3, 6.14, 6.15, crate table, LY-D10/D11 | Correct | See Recommended on LY-D10 exceptions |
| 186 | GR-D15: compositor sends BeginFrame; process-independent throttling; hidden documents get no rendering opportunities (1, 6.7) | GR 6.9, GR-D15 | Correct | — |
| 187 | "437–560 people† … 'before the DOM, bindings, shell …'" (GR 9.3, MD 9.3) (9.3) | GR 9.3, MD 9.3 | Partly | Quote is MD's only |
| 188 | NW-D10 cookie region without sync IPC; NW-D3 partition service (1, 4.4, 6.13) | NW 9.4 | Correct | — |
| 189 | NW 6.8 owns SW routing, navigation preload, static routes (6.9, 6.10) | NW 6.7, 6.8 | Partly | NW 6.7; preload and routes not in NW |
| 190 | MD 6.6 says "selection algorithms are HTML's, in `hg-dom`" (6.12) | MD 6.20 coverage table | Partly | Quoted from MD 6.20 |
| 191 | AX G106 input-equivalence in the input crate this spec names; AX 6.2 one IDREF resolver; AX 6.6 `hg-ax-focus` (1, 6.5, 6.17) | AX G106, 6.1–6.2, 6.6, 9.5 | Correct | — |
| 192 | CB I5 (restrict a script) and I11 (heavy frames and scripts, FP-D7) run in renderer, bindings, scheduler (6.7) | CB 6.6 table; FP-D7 | Correct | — |
| 193 | Stage 1: `localStorage` reads in the sync-IPC allowlist (4.1); `hg-navigation` in the broker (4.3); "JS, bindings and DOM" track 5–6 (8.3); A-S1-35 (6.13, 9.3, 9.5) | Stage 1 design; ledger | Correct | — |
| 194 | Phase 3 ledger G-11, G-12, G-13, G-16, G-19, G-20 (header, 6.6, 6.12, 9.1, 9.5) | Ledger rows | Correct | — |

Rows 1–169 are external facts and 170–194 are claims about other Izcal specs: 148 Correct, 41 Partly, 3 Wrong (rows 99, 113, 181), 1 Unverifiable (57), 1 Not checked (15).

## 5. Internal consistency

| # | Check | Result | Fix |
|---|---|---|---|
| C1 | 9.3 track ranges sum to 60–78† | Correct: 7+8+5+8+8+6+3+6+3+3+3 = 60; 9+10+6+10+11+8+4+8+4+4+4 = 78 | — |
| C2 | 69–90 people† | Correct at ×1.15 (69.0, 89.7) | — |
| C3 | 230–310† engineer-years from "30, 55, then 60–78 a year to 48–60 months" | Wrong: 205–319 | Must fix 2 |
| C4 | Summary "60–78 engineers†", "Stage 1 staffed at 5–6" vs 9.3 and Stage 1 8.3 | Correct | — |
| C5 | FP budgets: 72 B† (6.1, 6.14, DM-30), 48 B† (1, 6.1), 768 KiB† (6.8, 6.10, DM-26), 64 KiB† (6.14 per realm, DM-32 per agent) | Consistent; ST's 48 B layout (20 + 20 + 8) matches 6.1's two 32-bit columns | — |
| C6 | Scheduler constants 100 ms†, 1 s†, 1,024† (6.7, 6.19, DM-D1, Q-DM-15) | Consistent | — |
| C7 | Timer values: 4 ms clamp, 1 s† hidden alignment, 30 s† freeze (6.7, 6.14, DM-D10, FP-D4) | Consistent | — |
| C8 | 5 s† activation (6.5, Q-DM-4); 10 s† stop-script (6.7, Q-DM-15); 1 s† beforeunload deadline (6.9); 3 min† no-store (6.9, DM-D14, 11.2) | Consistent | — |
| C9 | SW and shared-worker timeouts 5 s†, 30 s†, 30 s†, 5 min† (6.10) | Consistent; 30 s† extended lifetime within HTML's 10 s–5 min and not above SW idle | — |
| C10 | 100 µs / 5 µs (6.7, DM-14) | Consistent | — |
| C11 | Every DM-D referenced exists in 9.4 | Correct: DM-D1 to DM-D21 referenced and defined | — |
| C12 | Every DM-n target referenced exists in Section 8 | Correct: DM-1–5, 10–14, 20–28, 30–32, 40–42, 50–52 | — |
| C13 | G90–G99 defined once; references to G91–G99 in text match their statements | Correct | — |
| C14 | "13 + 2" cross-origin members (4.4, 6.4, 6.16, DM-11) | Consistent internally; incomplete against HTML | Must fix 10 |
| C15 | "~20 B" wrapper stake vs "about 70 B of process memory" (6.4) | Inconsistent framing | Recommended |
| C16 | Origin keying: Section 1 "above a RAM threshold" vs 6.8 "every desktop class" | Inconsistent | Must fix 3 |
| C17 | Stage 1 exit 48–60 months† (9.2, 9.3) vs GR 9.5 and RN2 | Consistent | — |
| C18 | Single-engine admission (6.12) vs use of LoAF and `notRestoredReasons` (6.7, 6.9, 6.15) | Inconsistent with LY-D10 unless excepted | Recommended |
| C19 | Q-DM-1 to Q-DM-15 each cite an existing note open question (HP Q4, SR Q4, NA Q1–Q3, Q6, BO Q1, EL Q-EL-4) | Correct | — |
| C20 | Header lists `izcal_interfaces.md`; citation key lists IF | File missing | Must fix 1 |

## 6. Link check

**Inline links (47 distinct, 50 occurrences).** All return HTTP 200 with the expected title (curl with redirects; PDFs checked for `%PDF`). The `#fragment`s were searched for as `id=` in the fetched HTML; all 13 exist:

| Page | Fragment | Result |
|---|---|---|
| dom.spec.whatwg.org | `#concept-node-insert-ext`, `#move`, `#dom-nonelementparentnode-getelementbyid`, `#concept-getelementsbyclassname` | All exist |
| html …/webappapis.html | `#event-loop-processing-model`, `#update-the-rendering` | Both exist |
| html …/custom-elements.html | `#cereactions` | Exists |
| html …/nav-history-apis.html | `#crossoriginproperties-(-o-)` | Exists |
| webidl.spec.whatwg.org | `#dfn-overload-resolution-algorithm`, `#idl-DOMString`, `#dfn-distinguishing-argument-index`, `#es-platform-objects` | All exist |
| w3c.github.io/hr-time | `#dfn-coarsen-time` | Exists |

The 2.1 Chrome 115 row cites `developer.chrome.com/blog/immutable-document-domain` and 6.8 cites `…/document-domain-setter-deprecation`. Both resolve, and both support their claims.

**Reference-style links (24).** 23 resolve to files on disk: the seven strand notes; `HGBA 2 browser architecture spec.md`, `Future software browser engine.md`, `HGBA Stage 1 architecture design.md`; the JS, FP, ST, LY, GR, NW, MD, TX, DT, AX and CB specs; and `research_notes/Phase 3 amendments/ledger.md`. **`[IF]` → `izcal_interfaces.md` does not exist** (Must fix 1). Every key used is defined and every definition is used.

**Note citations.** All 322 citation parts in 295 `[XX …]` links resolve to an existing row, section or open question in the named note.

**Note links (for M-1 only).** All 492 distinct URLs in the seven notes were classified, and none is engine source. The `chromium.googlesource.com` links are all prose `.md` files: security FAQ, rule-of-2, severity guidelines, side-channel threat model, URL display guidelines, process model, `document-domain.md`, and Blink's `V8BindingDesign.md`, `IDLCompiler.md`, `IDLExtendedAttributes.md` and bindings `README.md`. The GitHub `blob/` links are BCD JSON, READMEs and explainers. The only `/commit/` and `/pull/` links are to whatwg/html and whatwg/dom. Ladybird is cited only through its newsletters and `Documentation/AddNewIDLFile.md`.
