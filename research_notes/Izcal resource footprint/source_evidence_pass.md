# Izcal resource footprint: source-evidence pass (M-1 2.3.1)

Done 19 October 2026 for spec revision 0.3.1. HGBA-2 2.3.1 adds M-1 part 3: another engine's source code (code files, headers, commits, diffs, code reviews) is never evidence. No spec or research note links to it or takes a constant, data layout or algorithm from it. Evidence is published prose (papers, documentation and prose design documents, even when hosted in a repository, talks, blog posts, release notes, bug-tracker and standards discussions) and black-box observation.

**Method.** Every link in `reports/Izcal resource footprint.md` and in this folder was matched against engine source hosts and paths (`chromium.googlesource.com` files other than prose `.md` documents, `source.chromium.org`, `crrev.com`, `codereview.chromium.org`, `chromium-review`, `searchfox.org`, `reviewboard.mozilla.org`, `trac.webkit.org` changesets, `webkit.googlesource.com`, `lists.webkit.org/webkit-reviews`, `github.com/WebKit/…/blob`, `raw.githubusercontent.com/WebKit`, `mojo.googlesource.com`, `skia.googlesource.com`, `doc.servo.org/src`, and a Chromium source mirror on `gitcode.com`). The text was also read for claims without links that came from source (constants, per-OS formulas, "Chromium uses a private struct"). For each item:

1. **Prose**: replaced with a published prose source that states the same fact, read to confirm it.
2. **Measurement**: a value or behaviour with no prose source became a black-box observation with a named milestone, and where Izcal needed a number now, a first-principles value with its reasoning.
3. **Removed**: the claim was dropped, and any argument that relied on it was checked to stand from first principles.

**Totals.** 70 links: 18 replaced with prose, 20 turned into black-box measurements, 32 removed.

| File | Links | Prose | Measurement | Removed |
|---|---|---|---|---|
| `reports/Izcal resource footprint.md` | 13 | 4 | 6 | 3 |
| `compact_engine_data.md` | 29 | 2 | 5 | 22 |
| `factcheck.md` | 10 | 4 | 5 | 1 |
| `os_mechanisms.md` | 9 | 3 | 3 | 3 |
| `history_postmortems.md` | 7 | 4 | 0 | 3 |
| `breakdown_measurement.md` | 1 | 0 | 1 | 0 |
| `gaps_opportunities.md` | 1 | 1 | 0 | 0 |
| **Total** | **70** | **18** | **20** | **32** |

Unlinked claims from source were handled as well (listed below with a "u" suffix). No decision (FP-D1 to FP-D19) changed and no target threshold changed. One design rationale is now a hypothesis until FP-M0 measures it (see the end of this log).

**New black-box measurements.** FP-M0 now includes the checks of the spec's new **open question 17**: which OS counters each incumbent's reported footprint follows, including whether shared memory is charged; memory charged per element and per attribute set when a page creates a million of them; whether Chrome and Firefox react to Linux PSI pressure before the OOM killer; macOS `phys_footprint` after `MADV_FREE_REUSABLE` and after `MADV_FREE_REUSE` on Izcal's own probe; each browser's HTTP-cache size against free disk; Chrome's soft process limit on each lab device. FP-M3 now measures the attribute-set sharing hit rate on the corpus.

## Spec: `reports/Izcal resource footprint.md`

| # | Where | Was | Action | Now | Effect on the design |
|---|---|---|---|---|---|
| S1 | Section 2, Never-Slow Mode row | Gerrit CL 1265506: "PROTOTYPE -- DO NOT COMMIT", auto-abandoned 2021-04-07, the author's quoted caveat | Prose | [BleepingComputer, 2019](https://www.bleepingcomputer.com/news/google/google-working-on-chrome-never-slow-mode-for-faster-browsing/): a prototype behind a flag, its budgets, and the author's warning that it could silently break content. Dates from the CL removed; the row is dated 2019 | None. Section 4.4 already derives the byte-budget lesson from first principles. |
| S2 | Section 3, "a different quantity per OS" | `queued_request_dispatcher.cc`: Chrome's PMF per OS and "not implemented on iOS" | Measurement | What each OS bills, from OS documentation ([WWDC 2018](https://developer.apple.com/videos/play/wwdc2018/416/), [proc](https://docs.kernel.org/filesystems/proc.html)); Chrome's metric named from its [memory docs](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/memory/README.md); which counters it follows is observed black-box at FP-M0 (Q17) | None. Section 6.2's per-OS definitions are derived from the OS definitions. |
| S3 | Section 3, PSI unused | `system_memory_pressure_evaluator.cc` (no Linux evaluator) | Measurement | Restated as "absent from published browser designs"; whether Chrome and Firefox react to PSI is tested black-box at FP-M0 (Q17) | None. Using PSI rests on the kernel's documentation of PSI. |
| S4 | Section 4.2, macOS release row | `discardable_shared_memory.cc` (`MADV_FREE_REUSABLE`) | Measurement | Advice values cited from the OS interface (the macOS SDK's `madvise` advice values; the kernel-header link was removed in the 2.3.3 pass); their effect on `phys_footprint` measured at FP-M0 | See "Rationale now a hypothesis" below. |
| S5 | Section 4.2, pressure row | `system_memory_pressure_evaluator_mac.cc` (3 levels) | Prose | [Apple: `DispatchSource.MemoryPressureEvent`](https://developer.apple.com/documentation/dispatch/dispatchsource/memorypressureevent) (normal, warning, critical) | None |
| S6 | Section 4.2, background CPU row | `process_mac.cc` (Chromium uses a private struct) | Removed | Replaced by App Nap's per-app scope ([Apple](https://developer.apple.com/library/archive/documentation/Performance/Conceptual/power_efficiency_guidelines_osx/AppNap.html)) | None. Izcal's public-QoS-only rule never depended on what Chromium does. |
| S7 | Section 5, WebKit pressure handler | `MemoryPressureHandler.cpp` ("die only if shrinking fails", 30 s poll) | Prose | [WebKitGTK `MemoryPressureSettings`](https://webkitgtk.org/reference/webkit2gtk/stable/struct.MemoryPressureSettings.html): conservative and strict policies at 0.33 and 0.5 of a memory limit by default, 30 s poll by default, optional kill threshold. "Die only if shrinking fails" had no prose source and is dropped | None. "Graded response, kill last" holds: the kill threshold sits above the strict one. |
| S8 | Section 5, WebKit shared attribute sets | Changeset r127438 ("nearly 80%" hit rate) | Measurement | The mechanism from [WebKit bug 102845](https://bugs.webkit.org/show_bug.cgi?id=102845) (bug discussion); the hit rate on Izcal's corpus measured at FP-M3 | None. Hash-consed attribute sets rest on "parsed markup repeats". |
| S9 | Section 6.7, macOS discardable row | `discardable_shared_memory.cc` (`MADV_FREE_REUSE` keeps `phys_footprint` accurate) | Measurement | Stated as expected behaviour, measured black-box at FP-M0 before the macOS path is fixed | See below. |
| S10 | Section 6.7, macOS pressure row | `system_memory_pressure_evaluator_mac.cc` (Chromium's 250 MiB disk floor as the prior) | Removed | The floor is set at FP-M3 from the largest single write the ladder may need (Section 7.1's derivation, unchanged). No number was set before, so no number is needed now | None |
| S11 | Section 6.7, runaway pages | `MemoryPressureHandler.cpp` (WebKit's kill thresholds) | Prose | [WebKitGTK](https://webkitgtk.org/reference/webkit2gtk/stable/struct.MemoryPressureSettings.html): an optional kill threshold, a fraction of a configured limit | None |
| S12 | Section 6.8, macOS background policy | `process_mac.cc` (struct copied from a private header) | Removed | App Nap applies per app, so children need their own policy; open question 4 reworded ("beyond QoS classes", FP-M4) | None |
| S13 | Section 6.12, HTTP cache | `cache_util.cc` (80 MiB, 200 MiB, 1% of free space) | Measurement | Incumbent rules observed black-box at FP-M0 (Q17). Izcal's 1% rule now states its derivation: 1% of 25 GiB† is 256 MiB†, so the rule is continuous at the switch and never takes more than 1% of a nearly full disk | None; values unchanged. |
| S1u | Section 6.2, Linux row | "The same arithmetic as Chromium's private footprint" | Removed | "Derived from the kernel's definitions above, not from any browser's metric" | None |
| S2u | Section 6.4, node-size evidence | WebKit ≈88 B and Blink ≈56 B, inferred from source member lists | Measurement | Per-element cost of Chrome, Firefox and Safari measured black-box at FP-M0 (Q17); FP-11's 48 B† is unchanged and was always set from JS-D4's columns | None |
| S3u | Section 6.6, visible-tab tiles | WebRender 1024 × 512 tiles; Chrome 256² to 512² (from source) | Prose | [How cc Works](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/how_cc_works.md): about 256 × 256 px for software raster, about viewport width × ¼ height for GPU raster | None. Tile size is chosen from measured invalidation waste. |
| S4u | Section 7.1 priors | Linux charged = Chromium's arithmetic; Chrome's 600 s; Chromium's 250 MiB; Chrome's Android ranking; Firefox's 3 and the spare process unlinked | Mixed | Linux: measurement (FP-M0); bfcache TTL: prose ([Chrome](https://developer.chrome.com/docs/web-platform/bfcache-ccns), 10 minutes); disk floor: removed; kill ranking: prose ([Android process lifecycle](https://developer.android.com/guide/components/activities/process-lifecycle)); Firefox's 3: prose ([Mozilla wiki](https://wiki.mozilla.org/DocShell/Fastback)); spare process: prose ([Reis et al. 2019](https://www.usenix.org/system/files/sec19-reis.pdf) §4.1.3) | None. Every constant's derivation and milestone is unchanged. |
| S5u | Section 7 text and anchor table | "Chromium, Firefox and WebKit sources are cited as evidence of cost" | Edited | Cited only through published prose and black-box observation (M-1 part 3) | — |
| S6u | Sections 5 and 11, Appendix FC-2 | "auto-abandoned as stale" (from the CL) | Edited | "No published outcome"; FC-2 records the re-sourcing | — |
| S7u | Section 11.2 | Inferred node sizes; "Chrome's bfcache constants (6 entries, 600 s) … from a mirror of Chromium source" | Edited | Node sizes measured at FP-M0; TTL from Chrome's documentation, entry count removed | — |
| S8u | Header | "HGBA-2 2.3.0" | Edited | 2.3.1. References to 2.3.0 as the revision that introduced M-1 part 1 are history and stay | — |

**Kept in the spec (prose documents in Chromium's repository, not code):** `docs/process_model_and_site_isolation.md`, `docs/linux/zygote.md`, `docs/memory-infra/memory_benchmarks.md`, `docs/memory/README.md`, `docs/memory/key_concepts.md`, `docs/memory/achievements.md`, `docs/how_cc_works.md`, `base/allocator/partition_allocator/PartitionAlloc.md`. jemalloc's `TUNING.md` (prose, and jemalloc is a general-purpose allocator).

## Research notes

### `compact_engine_data.md` (29 links)

| # | Row | Was | Action | Now |
|---|---|---|---|---|
| C1 | Header and M-1 paragraph | "engine source files and their size assertions, commit messages" as preferred sources | Edited | Engine source, commits and code reviews are not evidence; points to this log |
| C2 | 1.1 | WebKit `Node.cpp`, `Element.cpp` (raw), Blink `node.cc`, crrev 2231953003 (4) | Removed | The lesson restated from WebKit bug 186360 (reviewers objected on size grounds) and from first principles (the build fails on a type-size regression) |
| C3 | 1.2 | WebKit `Node.cpp`, `Element.cpp` (2) | Measurement | Per-element cost of each browser, FP-M0 (Q17) |
| C4 | 1.3 | Blink `node.cc` (1) | Measurement | As C3 |
| C5 | 1.4 | WebKit commit "Reduce Node object size", webkit-reviews thread (2) | Removed | Only the arithmetic kept (8 B × 300,000 nodes = 2.4 MB) |
| C5u | 1.6 | "≈325k nodes", derived from 1.4 | Measurement | Arithmetic at 300,000 nodes; the HTML spec's node count counted black-box at FP-M0 |
| C6 | 2.1 | Changeset r123636 (1) | Measurement | Inline attribute storage kept as an option; its saving measured at FP-M3 |
| C7 | 2.2 | r127438 (1) | Measurement | Hit rate measured at FP-M3; mechanism from [WebKit bug 102845](https://bugs.webkit.org/show_bug.cgi?id=102845) |
| C8 | 2.2 | r136334 (1) | Removed | The 10 s cache-retention detail and its saving are gone |
| C9 | 2.2 | `ElementData.h` (Sky snapshot) (1) | Removed | — |
| C10 | 2.3 | `ElementData.h` (Sky snapshot) (1) | Removed | Restated as an Izcal option from first principles |
| C11 | 4.2 | Stylo `style/sharing/mod.rs` on doc.servo.org/src (1) | Removed | The cache exists per 4.1's prose source (Mozilla Hacks); Izcal sizes its own from a measured hit rate |
| C12 | 4.4 | Blink `computed_style.cc`, crrev 2117143003, crrev 2656733002 (3) | Removed | The idea (groups shared until written) is in 4.1's prose source |
| C13 | 4.5 | Searchfox `GlobalStyleSheetCache.h` (1) | Removed | Bug 1474793 and Nethercote's posts carry the row |
| C14 | 7.1 | Blink `layer_tree_settings.cc` (1) | Removed | How cc Works (prose) carries the tile sizes; per-platform defaults dropped |
| C15 | 7.2 | `layer_tree_settings.cc`, cc `layer_tree_settings.h` (2) | Removed | Chrome's GPU budget formula dropped; Izcal's GPU budgets are its own (footprint 6.6) |
| C16 | 7.3 | WebRender `tile_cache/mod.rs` (1) | Removed | — |
| C17 | 7.4 | WebRender `texture_cache.rs` (1) | Removed | — |
| C17u | Section 7 implications | WebRender tile cost and Chrome's budget formula as a prior | Removed | Tile size from measured invalidation waste; GPU memory measured on Izcal's renderer |
| C18 | 8.5 | WebKit `DiscardablePixelRef` commit (1) | Removed | Chrome's GPU discardable decode cache cited from How cc Works; Firefox from bug 661304 |
| C19 | 9.2 | Skia `SkGlyphCache_Globals.h` (1) | Removed | Izcal's atlas sizes are its own (footprint 6.6, FP-D14) |
| C20 | 13.1 | Chromium `back_forward_cache_impl.cc` mirror on gitcode (1) | Prose | [Chrome developer documentation](https://developer.chrome.com/docs/web-platform/bfcache-ccns): 10 minutes, 3 minutes for `no-store` pages. Entry count, foreground limits and pressure behaviour removed |
| C21 | 13.2 | Searchfox `nsSHistory.cpp` (1) | Prose | [DocShell/Fastback wiki](https://wiki.mozilla.org/DocShell/Fastback): "a maximum of 3 cached content viewers" per tab; global limit "a logarithmic function of the memory size, with a maximum of 8" |
| C22 | 13.2 | Reviewboard review 241862 (1) | Removed | The Android detail is gone |
| C23u | Sections 1, 2, 9 implications and Section 16 | "same order as WebKit's Node", "WebKit's 80 %", "which WebRender already does", inferred sizes 1.2, 1.3, 4.4 | Edited | Consistent with the rows above |

### `factcheck.md` (10 links)

| # | Row | Was | Action | Now |
|---|---|---|---|---|
| F1 | Never-Slow Mode | CL 1265506 via its REST API | Prose | BleepingComputer, 2019; the claim cell's quote from the CL is redacted |
| F2 | Chrome PMF per OS | `queued_request_dispatcher.cc` | Measurement | Verdict "Not evidenced (0.3.1)"; FP-M0 black-box (Q17) |
| F3 | No Linux pressure evaluator | `system_memory_pressure_evaluator.cc` | Measurement | Verdict "Not evidenced (0.3.1)"; FP-M0 PSI test |
| F4 | `MADV_FREE_REUSABLE` / `MADV_FREE_REUSE` | `discardable_shared_memory.cc` | Measurement | xnu header for the interface (link removed at 2.3.3); effect measured at FP-M0 |
| F5 | Dispatch pressure and 250 MiB | `system_memory_pressure_evaluator_mac.cc` | Prose | Apple documentation for the three levels; the 250 MiB half "not evidenced" and removed from the spec |
| F6 | Private struct for background policy | `process_mac.cc` | Removed | Chromium half "not evidenced"; spec cites App Nap instead |
| F7 | WebKit pressure handler | `MemoryPressureHandler.cpp` | Prose | WebKitGTK documentation; verdict "Partly (re-sourced)" because "die only if shrinking fails" has no prose source |
| F8 | WebKit shared attributes ~80% | r127438 | Measurement | Verdict "Not evidenced (0.3.1)"; FP-M3 |
| F9 | Firefox 3 per tab | Searchfox `nsSHistory.cpp` | Prose | Mozilla wiki, which states both the per-tab 3 and the global cap of 8 |
| F10 | Chromium HTTP cache | `cache_util.cc` | Measurement | Verdict "Not evidenced (0.3.1)"; FP-M0 |
| F1u | Link check (§0) | Mentioned the `nsSHistory.cpp` link's 406 | Edited | Notes the link was removed |
| F2u | Chrome bfcache 600 s and 6 entries | "Unverifiable", mirror source | Prose | TTL from Chrome's documentation; entry count removed |

### `os_mechanisms.md` (9 links)

| # | Row | Was | Action | Now |
|---|---|---|---|---|
| O1 | Header | "Chromium source read at `HEAD` … as evidence only" | Edited | Not evidence; points to this log |
| O2 | 1.6 | crrev `sandboxed_process_launcher_delegate.h` | Prose | [Chromium zygote doc](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/linux/zygote.md): "In the model used on Windows and Mac, renderers are exec'ed as needed from the chrome binary"; the Mac Sandbox V2 design document kept |
| O3 | 2.2 | `platform_shared_memory_region_posix.cc` | Prose | The OS facts from [memfd_create(2)](https://man7.org/linux/man-pages/man2/memfd_create.2.html) and [fcntl(2)](https://man7.org/linux/man-pages/man2/fcntl.2.html); Chromium's exact flags removed |
| O4 | 5.3 | `discardable_shared_memory.cc` | Measurement | Purge mechanisms per OS from OS documentation and the xnu header (link removed at 2.3.3); `phys_footprint` effect at FP-M0; Chromium's choices and its Windows 10 caveat removed |
| O5 | 6.7 | `system_memory_pressure_evaluator_mac.cc` | Prose | Apple's documentation for the levels; Chromium's mapping and 250 MiB removed |
| O6 | 6.8 | `threshold_memory_pressure_evaluator.cc` | Removed | Chromium's 200 / 500 MiB commit thresholds removed; Microsoft's API documentation kept |
| O7 | 6.9 | `system_memory_pressure_evaluator.cc` | Measurement | FP-M0 PSI test (Q17) |
| O8 | 10.3 | `process_mac.cc` | Removed | Chromium's task-policy details removed; App Nap row kept |
| O9 | 10.5 | `process_win.cc` | Removed | Chromium's EcoQoS feature flag removed; Microsoft and Firefox sources kept |
| O10 | 12.1 | `cache_util.cc` | Measurement | FP-M0 (Q17) |
| O11u | Summary tables and open question 4 | "QoS + task policy (Chromium uses private struct)"; "Chromium no Linux pressure evaluator"; "Chromium relies on `task_policy_private.h`" | Edited | "QoS; App Nap per app"; "no published browser use of PSI"; "beyond QoS classes" |

The `dyld` `fixup-chains.h` link in 2.4 was Apple's operating-system source; under HGBA-2 2.3.3 (M-1 part 3 covers every third-party project's source) it was replaced by Apple's WWDC22 "Link fast" talk (`research_notes/Phase 3 amendments/source_evidence_2.3.3.md`).

### `history_postmortems.md` (7 links)

| # | Row | Was | Action | Now |
|---|---|---|---|---|
| H1 | Header and method note | "source code read for design rationale only"; "source files … read for their stated rationale and constants only" | Edited | Engine source is never evidence; points to this log |
| H2 | 2.3 | `render_process_host_impl.cc` (RAM / 2 / 85 MB, worked values) | Prose | [Process model doc, "Soft Process Limit"](https://chromium.googlesource.com/chromium/src/+/main/docs/process_model_and_site_isolation.md#process-reuse): a soft limit "based on the memory available"; formula removed; Chrome's limit per lab device observed at FP-M0 (Q17) |
| H3 | 2.4 | `render_process_host_impl.cc` (Android source comment, RAM/80 MB flag) | Prose | Same doc: "Chromium on Android does not set this soft process limit, and instead relies on the OS to discard processes" |
| H3u | Section 2 implications | "Chrome's 85 MB per-WebContents constant" | Edited | "A memory-based limit implies a per-process budget" |
| H4 | 4.4 | `ChildProcessRanking.java` | Removed | Chrome's ranking removed; Android's process-importance documentation added |
| H5 | 6.1 | `MemoryPressureHandler.cpp` (30 s; 0.33 / 0.5; iOS fractions; log strings) | Prose | WebKitGTK documentation (30 s, 0.33 / 0.5, optional kill threshold); iOS fractions and log strings removed |
| H6 | 6.2 | `MemoryPressureHandler.cpp` (7 / 15 GB + 1 GB per tab) | Prose | WebKitGTK's kill-threshold documentation; per-tab values removed |
| H7 | 9.3 | `chrome/browser/experiments` source tree at 64.0.3282.8 | Removed | Row kept with crbug 710048 (bug tracker), not re-read, graded Low |
| H8 | 9.3 | codereview 2810833002 | Removed | As H7; the GPU-memory trial detail removed |

The Ladybird `README.md` and `Documentation/ProcessArchitecture.md` (prose in a repository) and doc.servo.org's API documentation page for `profile_traits::mem` (generated documentation, not `/src/`) stay.

### `breakdown_measurement.md` (1 link)

| # | Row | Was | Action | Now |
|---|---|---|---|---|
| B1 | 1.1 | `queued_request_dispatcher.cc` (`CalculatePrivateFootprintKb`) | Measurement | PMF named from Chromium's memory docs; OS quantities from 1.3, 1.4, 1.6; which counters Chrome follows observed at FP-M0 (Q17) |
| B1u | 1.5 | Derived from 1.1's source reading | Measurement | Derived from the kernel's definitions only; checked black-box at FP-M0 |
| B2u | Header | "source code" among preferred sources | Edited | Not evidence |

### `gaps_opportunities.md` (1 link)

| # | Row | Was | Action | Now |
|---|---|---|---|---|
| G1 | 7.2 Never-Slow Mode | Gerrit CL 1265506 (budgets, abandonment, the author's caveat) | Prose | BleepingComputer, 2019 for the budgets and status; `document.write`/sync XHR detail and the abandonment date removed; the lesson labelled as first principles |

### `critique.md` (no links)

| # | Row | Was | Action | Now |
|---|---|---|---|---|
| R1u | Finding on derived constants | "exactly Chromium's private memory footprint"; Chrome's free-disk floor | Edited | Both marked as engine-source readings that are no longer evidence |

## Decisions, rationales and thresholds

- **No decision changed, and no decision lost its support.** FP-D1 (no fork server) rests on the zygote prose document and first principles. FP-D3, FP-D4, FP-D13, FP-D16 and the others never cited engine source.
- **Rationale now a hypothesis until measured:** Section 6.7's macOS discardable path (unlock plus `MADV_FREE_REUSABLE`, and `MADV_FREE_REUSE` on relock "so `phys_footprint` stays accurate") was supported only by Chromium source. It is now stated as expected behaviour of the OS interface and is measured black-box at FP-M0 before the path is fixed. If FP-M0 shows the charge does not drop, the macOS row of Section 6.7 changes (purgeable VM objects remain the other listed mechanism). This is a design detail inside the accepted pressure ladder, not a decision.
- **No target threshold changed.** No † value moved. The macOS free-disk Critical floor had no Izcal number before (only Chromium's 250 MiB as a prior) and is still set at FP-M3 from Section 7.1's derivation. The HTTP cache's 1% rule is unchanged and now states its derivation (continuity with 256 MiB† at 25 GiB†).
