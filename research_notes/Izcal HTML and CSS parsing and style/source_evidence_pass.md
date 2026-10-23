# Izcal HTML and CSS parsing and style: source-evidence pass (0.3.1)

Done 19 October 2026 for `reports/Izcal HTML and CSS parsing and style.md` (draft 0.3 → 0.3.1) and every note in this folder.

**Rule.** HGBA-2 2.3.1, M-1 part 3: another engine's source code (V8, Chromium, Blink, Gecko/Firefox, WebKit, Servo/Stylo, Ladybird and engine libraries; their code files, headers, commits, diffs, code reviews) is never evidence. No spec or research note links to it or takes a constant, data layout or algorithm from it. Allowed: published prose (papers, documentation and prose design documents even when hosted in a repository, talks, blog posts, release notes, bug-tracker discussion text, standards threads) and black-box observation.

**Method.** Every engine-source link (Gitiles, GitHub blob/tree/commit/pull pages, raw source files, rendered source in rustdoc, code reviews, Gerrit changes) and every unlinked claim read from source was found by pattern search and by reading the rows. Each was (1) replaced by a published prose source that states the same fact, checked by fetching it; or (2) for a value or behaviour, turned into a black-box measurement with a milestone and, where a number is needed now, a † value with its reasoning; or (3) removed, with the dependent argument checked. Engine sources are named below in plain text, never linked.

**Counts.** 80 items. Report: 13 (5 links, 8 unlinked claims): 9 replaced, 4 measurements, 0 removed. Notes: 67: 23 replaced, 3 measurements, 37 removed, 4 kept (prose design documents or standards records). Two further note edits rewrote method and provenance text (items 88–89). Totals: **32 replaced, 7 measurements, 37 removed, 4 kept**.

**Decisions and targets.** No decision's recommendation changed and no target threshold changed. Support weakened in two places, both recorded in the report's open questions: the XML nesting cap is now **5,000†** pending a black-box test at ST-M0 (its only source was a Firefox test file), and the HTML cap's overflow behaviour (new elements attached as siblings at depth 512) now rests on the ST-M0 black-box test (Q-ST-3, Q-ST-18). ST-D2 (foreground parser on the script thread) keeps its argument (JS-D4 and the published 2020 proposal); only the historical claim that Blink deleted its background parser became a black-box check (Q-ST-18).

## Report items

| # | file:line (before) | Engine source cited | Claim | Action | New source or target / † value | Dependent argument |
|---|---|---|---|---|---|---|
| 1 | report:11 | Chromium commit bc34c4e (2020) | Blink's foreground parser was expected to be "less complicated" with "a faster time-to-first paint" | Replaced | Chromium dom-dev thread "Removing the BackgroundHTMLParser" (April 2020): "could simplify the document construction / loading pipeline"; Pinpoint timeToFirstPaint −8.9% | Summary's pattern ("what was removed depended on thread assignment") still stands |
| 2 | report:11 | Chromium commit 4dbb354 (2022) | Blink made it the default and deleted the background parser on 2 February 2022 | Measurement | ST-M0 black-box: a Chrome performance trace of a large document shows which thread tokenizes (Q-ST-18) | ST-D2 rests on JS-D4, not on this date |
| 3 | report:19 | Chromium fix commit e045399a1e | CVE-2026-2441's root cause: stale iteration over a CSSOM map that script mutated | Replaced | Wiz vulnerability database entry for CVE-2026-2441 ("the map is mutated (via `delete()` or `set()`) during iteration") | 6.14's mitigation unchanged |
| 4 | report:20 | ChromeDevTools devtools-protocol definition file (browser_protocol.json) | CDP returns matched rules without a winner or reason | Replaced | CDP documentation, CSS domain (chromedevtools.github.io) | Section 6.13's case unchanged |
| 5 | report:51 | Same commit as item 2 (unlinked) | "deleted 2022-02-02 … became the default" | Measurement | Q-ST-18 | Row now says "proposed for replacement in 2020" |
| 6 | report:81 | DevTools frontend commit 0e0de1d (unlinked date) | Microsoft added a per-selector invalidation count in July 2025 | Replaced | Edge DevTools 140 release notes (September 2025) | — |
| 7 | report:134 | Commits of items 1–2 (via HP 2.7–2.8) | "Blink replaced its background parser" | Replaced | dom-dev thread (item 1) | Keep-or-break row unchanged ("Keep") |
| 8 | report:158 | Commit of item 2 | "then deleted when unused" | Measurement | Q-ST-18 | — |
| 9 | report:240, 472 | WebKit layout-test expectation file for the parser nesting cap (via SR 4.2) | Past 512 open elements, new elements become children of the 512th | Replaced + measurement | WebKit bug 63856 ("Reduce parser nesting cap to 512", 2011); overflow behaviour and other engines black-box at ST-M0 (Q-ST-3) | Cap value unchanged; behaviour rests on ST-M0 (weakened) |
| 10 | report:240, 473 | Firefox test file for XML parse depth (via SR 4.4) | Most engines cap XML nesting at 5,000 | Measurement | ST-M0 black-box (Q-ST-18); **5,000†**: far above any real document's depth, and Izcal's walks use explicit stacks, so only interoperability depends on the value | Limit unchanged in value, now † (weakened) |
| 11 | report:460 | Chromium fix commit e045399a1e | Quote of the fix's title | Replaced | Wiz entry (item 3) | — |
| 12 | report:851 | Fix commit; Blink/WebKit/Gecko constants | Fact-check confirmations | Replaced + measurement | Wiz entry; WebKit bug 63856; ST-M0 | — |
| 13 | report:852 | Commits of items 1–2 | "two commits, two reasons" | Replaced | dom-dev thread | — |

## Research-note items

HP = `history_postmortems.md`, PI = `performance_invalidation.md`, GO = `gaps_opportunities.md`, SP = `specs_semantics.md`, SR = `security_robustness.md`, FC = `factcheck.md`, CR = `critique.md`. Removed rows keep their number and say so.

| # | file:line (before) | Engine source cited | Claim | Action | New source or target / † value | Dependent argument |
|---|---|---|---|---|---|---|
| 21 | HP:70 (2.3) | Gecko header nsHtml5StreamParser.h (comments) | Gecko tokenizes and builds off the main thread, producing tree operations | Replaced | Mozilla Hacks 2010-05 and 2017-09 | — |
| 22 | HP:71 (2.4) | Same header | Gecko stops speculating after 100 failures; a mutex guards tokenizer state | Removed | Izcal measures its own speculation failure rate (ST-25) | ST-D2 (b) is decided by measurement, not by Gecko's limit |
| 23 | HP:74 (2.7); FC:53, 194 | Chromium commit bc34c4e | Foreground parser rationale | Replaced | dom-dev thread (item 1) | — |
| 24 | HP:75 (2.8); FC:43, 53, 194 | Chromium commit 4dbb354 | Default since M96; deleted 2022-02-02 | Measurement | Q-ST-18 | — |
| 25 | HP:58 (1.7); GO:97 (6.1) | Firefox in-tree README for the Java parser translator | Firefox regenerates its parser from Java | Kept: prose doc | — | — |
| 26 | HP:98 (4.4) | Chromium code review 566483002; Blink tokenizer source file header | Blink's 2014 plan for a CSS Syntax 3 parser | Replaced | WebKit bug 161174 (reviewer: "this comes from Blink"); the 2014 dating is Low | — |
| 27 | HP:99 (4.5) | WebKit CSSTokenizer.cpp copyright line | WebKit's parser was ported from Blink | Replaced | WebKit bug 161174, comment 3 | — |
| 28 | HP:114 (5.6) | WebKit header PlatformEnable.h | Selector JIT enabled on x86_64 and ARM64 | Replaced | WebKit bugs 275971 and 307542 (activity; per-architecture enablement removed) | ST-D9 argues from jitless renderers, unchanged |
| 29 | HP:110 (5.2) | Stylo bloom.rs doc comment | Stylo has a thread-local ancestor Bloom filter | Removed | WebKit bug 53880 carries the row | — |
| 30 | HP:126 (6.3) | Stylo sharing module doc comment | Sharing-cache checks and revalidation selectors | Replaced | Lin Clark, Mozilla Hacks 2017 | 4.1's "Break" row unchanged |
| 31 | HP:127 (6.4); FC:54, 195 | Chromium code review 2887593002; Chromium commit fd7f99c9125b | Style sharing disabled in M61, deleted 2017-07-24 | Removed (dates) | style-dev thread kept for the reasons | — |
| 32 | HP:128 (6.5), HP 12, HP open questions; FC:75, 207; GO:102 (count) | Blink css_properties.json5 | ~950 / 835 property entries | Removed | Blink's ComputedStyle.md (prose design doc) for the generation fact; no count | Report cites no count (as at 0.2) |
| 33 | HP:129 (6.6) | Blink css_group_config.json5, make_computed_style_base.py; code review 2841413002 | Field groups by usage CDF; bit packing | Removed | Izcal's groups derived in report 6.18 (ST-30, Q-ST-5) | Unchanged |
| 34 | HP:145 (7.7) | Stylo driver.rs, parallel.rs | Sequential start; 53 recursion levels | Replaced (work stealing) / removed (rest) | Lin Clark 2017 | — |
| 35 | HP:157 (8.3) | Chromium code reviews 199633020, 222643002, 1333863002 | Invalidation-set dates 2014–15 | Removed (dates) | Blink style-invalidation.md kept (prose doc) | — |
| 36 | PI:40 (3.2) | Blink selector_filter.h | 1 KiB one-hash filter; 60–70% rejection | Removed | ST-22, Q-ST-6 measure Izcal's filter | 6.18 derivation unchanged |
| 37 | PI:41 (3.3) | Stylo bloom.rs (rendered source pages) | 4 KiB counting filter; per-thread rebuild | Removed | — | Izcal's per-task filter is derived in 6.18 |
| 38 | PI:42 (3.4) | WebKit SelectorFilter.h | 12-bit key, ~0.2% false positives | Replaced (fact) / removed (numbers) | WebKit bug 53880 | — |
| 39 | PI:43 (3.5) | Blink rule_set.h | Subject filter; bucket-covered flag | Removed | Report 6.18 derives the subject-covered flag | — |
| 40 | PI:57 (4.3) | Gecko StaticPrefList.yaml | Work-unit pref value and comment | Removed | Mozilla bug 1385982 kept for the measured values | — |
| 41 | PI:58 (4.4) | Blink matched_properties_cache.cc | Cache pruned 500 → 300; 1% sampling | Removed | ST-31 publishes Izcal's hit rate | — |
| 42 | PI:59 (4.5) | Chromium code review 30453002 | Older sharing finder used an LRU list | Removed | Lin Clark; style-dev thread | — |
| 43 | PI:69 (5.2) | Chromium code review 1858553002 | Root-level variables were copied per node until 2016 | Removed | — | Report ST-D14 argues from Web Almanac shape data |
| 44 | PI:71 (5.4) | WebKit StyleCustomPropertyData.cpp; Stylo custom_properties_map.rs | Chain limit 4; copy threshold 8 | Removed | Bound chosen at ST-M0 (ST-24) | 6.18 derivation unchanged |
| 45 | PI:82 (6.2) | Stylo STYLO_MAX_THREADS rustdoc constant | Thread pool capped at 6 | Replaced | Mozilla bug 1431285 ("Cap the number of stylo threads at six", Firefox 63) | — |
| 46 | PI:84 (6.4) | Stylo driver.rs; Gecko pref file | Adaptive start; work-unit constants | Removed | — | — |
| 47 | PI:87 (6.7) | Stylo parallel.rs (rendered source) | Stack sizes; 53 levels | Removed | Report 6.14: explicit stacks | — |
| 48 | PI:95 (7.1) | Blink style-invalidation.md | Invalidation sets mechanism | Kept: prose doc | — | — |
| 49 | PI:96 (7.2) | Blink rule_invalidation_data.h | Sibling-invalidation fallbacks | Removed | — | — |
| 50 | PI:98 (7.4) | Blink check_pseudo_has_cache_scope.h | `:has()` result cache flags | Replaced | Byungwoo Lee, "How blink tests :has() pseudo class?" (Igalia, 2022) | Report's per-pass memo is derived in 6.18 |
| 51 | PI:99 (7.5) | WebKit HasSelectorFilter.cpp | 4096-slot subtree filter | Removed | WebKit blog 13096 kept for the claim | — |
| 52 | PI:100 (7.6) | Stylo relative_selector.rs, state_and_attributes.rs | `:has()` invalidation order; snapshots | Removed | — | Report uses its own mutation log (B4) |
| 53 | PI:102 (7.8) | Stylo invalidation/stylesheets.rs; Blink style_engine.cc | Full-restyle fallbacks on sheet changes | Removed | — | 12.4 rewritten to cite only Ladybird's issue |
| 54 | PI:103 (7.9) | Blink style_recalc_change.h | Container-unit over-recalc | Removed | CSS Containment 3 cited for context | — |
| 55 | PI:112 (8.1) | WebKit pull request 28251; Chromium change 5538407 | SIMD scanning in both engines | Removed (links) | Lemire's paper carries the row | Report 6.4 unchanged |
| 56 | PI:113 (8.2) | Blink html_document_parser_fastpath.cc | SIMD threshold 64 / 16 | Removed | — | — |
| 57 | PI:114 (8.3) | Same file | Fast-path subset and bail-out list | Replaced (existence) / removed (bail-outs) | Chromium blog 2023-04-13; Macworld | ST-D1 rejects a subset parser on the mXSS record, unchanged |
| 58 | PI:115 (8.4) | Blink html_document_parser.cc | Token and time yield budgets | Removed | Izcal sets budgets from its corpus at ST-M1 (report 6.4) | Two-budget policy derived from first principles, unchanged |
| 59 | PI:116 (8.5) | Same file | Threaded preload scanner flag | Removed | — | — |
| 60 | PI:138 (10.1) | Blink css_lazy_parsing_state.h | Lazy declaration parsing | Removed | Q-ST-7 decides ST-D19 | 6.18 derivation unchanged |
| 61 | PI:139 (10.2) | Blink style_engine.cc | Text-to-sheet cache | Removed | compact_engine_data kept | — |
| 62 | HP:109 (5.1), HP:157, GO:55 (3.6), HP:128 | Blink style-calculation.md, style-invalidation.md, ComputedStyle.md | Mechanisms and generation | Kept: prose docs | — | — |
| 63 | GO:52 (3.3); FC:63 | DevTools frontend InvalidationsHandler.ts, Configuration.ts | Invalidation tracking is an experiment capped at 20 events | Measurement | ST-M0 black-box: enable the experiment, record a trace, list what events carry | GO opportunity row unchanged in direction |
| 64 | GO:53 (3.4) | DevTools frontend TraceEvents.ts | Event fields; 2 style reasons, 8 layout reasons | Removed | Covered by item 63's observation | — |
| 65 | GO:51 (3.2); FC:59, 209 | DevTools frontend commit 0e0de1d | Invalidation-count column, July 2025 | Replaced | Edge DevTools 140 release notes | — |
| 66 | GO:58 (3.9) | DevTools frontend CSSMatchedStyles.ts | Frontend re-implements the cascade | Removed (code fact); inference kept | CDP documentation (item 79) | Report's "explanations outside the engine drift" now rests on the protocol's missing fields |
| 67 | GO:101 (6.5) | Ladybird commit history; file sizes | Rust HTML tokenizer, 2026 | Removed | — | — |
| 68 | GO:102 (6.6) | Blink css_properties.json5 | Generated property code; parameters; counts | Replaced (fact) / removed (counts) | Blink ComputedStyle.md | ST-D6 rests on webref (GO 6.9), unchanged |
| 69 | GO:103 (6.7) | Stylo longhands.toml | Property data file, 433 entries | Removed | — | — |
| 70 | SP:106 (8.3); FC:128 | Blink, WebKit and Gecko constants (html_construction_site.h, SettingsBase.h, TreeBuilder.java) | All three engines cap at 512 | Replaced + measurement | WebKit bugs 63856, 63219, 157230; Q-ST-3 | See item 9 |
| 71 | SR:87 (4.2) | WebKit layout-test expectation file (webkit.googlesource) | WebKit attaches elements as siblings at 512 | Replaced + measurement | WebKit bugs 63856, 63219; Q-ST-3 | See item 9 |
| 72 | SR:89 (4.4) | Firefox test file (searchfox) | XML depth 5,000 in most engines | Measurement | Q-ST-18; 5,000† | See item 10 |
| 73 | SR:146 (8.2) | Stylo rustdoc pages rendering source constants | Style-thread stack 8,192 KB | Removed | — | — |
| 74 | SR:147 (8.3) | Stylo rustdoc page | AtomicRefCell panics; one unsafe site | Replaced (pattern) / removed (count) | Goregaokar, Rust blog 2017 | — |
| 75 | GO:104 (6.8) | Ladybird LibWeb/CSS source tree | 15 JSON generator tables | Replaced | Ladybird Documentation/CSSGeneratedFiles.md (12 documented tables) | — |
| 76 | GO:132 (8.3) | Stylo sharing module, context.rs | Per-thread LRU of 32, 8 levels | Replaced | Mozilla bugs 1397380, 1385982 | Report's deterministic-sharing argument unchanged |
| 77 | GO:133 (8.4) | Same (code comment "very important") | Sharing correctness invariant | Replaced | Lin Clark 2017 (inference from the described checks) | — |
| 78 | GO:144 (9.3) | Chromium code review 66383005 | Chrome removed user style sheets around 33 | Replaced | public-uaag2-comments, March 2014: "User stylesheet support removed in Chrome 33" | — |
| 79 | GO:57 (3.8) | devtools-protocol definition file | CSSProperty fields; no overridden flag | Replaced | CDP documentation, CSS domain | — |
| 80 | HP:113 (5.5) | WebKit commit messages | JIT used for exact invalidation checks | Replaced | WebKit bug 128321 title and description | — |
| 81 | HP:116 (5.7) | WebKit commit messages | Internal flag to compare matchers | Replaced | WebKit bugs 297655 and 301090 text | Report G22 unchanged |
| 82 | HP:54 (1.4) | html5lib-python commit history | Start dates 2006-12-01/07 | Removed (link; dates now Low) | html5lib-tests repository kept | — |
| 83 | CR:23–30 | Blink, Stylo source file names (unlinked) | Finding 1's table | Removed (names) | Descriptions plus a 0.3.1 note | Critique finding unchanged |
| 84 | PI:162 (12.4) | Stylo and Blink identifiers from source | Named full-restyle fallbacks | Removed | — | — |
| 85 | PI §3 implications | Numbers from items 36–38 | Filter sizes and rejection rate | Removed | — | — |
| 86 | PI:68 (5.1) | Chromium revision r765278 | Revision number of a Chrome 84 change | Removed | Igalia blog carries the row | — |
| 87 | FC, SP (several) | whatwg/html commits; WPT, html5lib-tests, webref repositories | Standard and test-suite history | Kept: standards records and test data, not engine source | — | — |
| 88 | GO:14 | Method note listing source files read | — | Updated text | — | — |
| 89 | HP:3, 6; PI:3, 6; SP:6; GO:6; SR:6 | Provenance and method notes | "rows marked, kept as a record" | Updated text: rows now replaced or removed | — | — |
