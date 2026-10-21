# Source-evidence pass, HGBA-2 2.3.3

*20 October 2026. Per-item log of the evidence pass that HGBA-2 revision 2.3.3 requires. M-1 part 3 now says **no third-party project's source code is evidence**: not a browser engine, a library, a tool, assistive technology, an editor or an OS component. No spec or research note links to code files, headers, commits that change code, diffs, pull-request file views or code reviews, and no constant, data layout or algorithm is read from them. This log names removed sources in plain text only and does not link to them.*

## Scope and method

**In scope:** the reports and research notes of the twelve component specifications (JavaScript engine, resource footprint, HTML and CSS parsing and style, fonts and text, networking, layout, images and media, graphics and rendering, content blocking, DOM, bindings and event loop, accessibility, PDF), plus `research_notes/Phase 2 reconciliation/` and `research_notes/Phase 3 amendments/`. **Out of scope, hits listed only:** HGBA-2's superseded passages and notes, HGBA-3, the Stage 1 design, the Stage 1 design facts and the essay reviews (last section).

**Search.** Every URL into a code host was extracted by script: GitHub, raw GitHub content, GitLab instances, googlesource, Codeberg, Bitbucket, SourceForge, crates.io, docs.rs, Searchfox and other source browsers, patchwork and mailing-list patch views, Gerrit and Phabricator. Each URL was then sorted by path into commit or diff, pull request, code file, docs.rs source, licence, prose, issue, crate page or repository landing page. A second scan looked for source-browser and patch paths on other hosts. A third scan looked for unlinked claims read from source, such as "X's code does Y", file names with code extensions and header names. Each slice owner then reviewed every hit by hand.

**Classification.**
- **Allowed:**
  - specification repositories (WHATWG, TC39, W3C, WICG, WebAssembly, gpuweb, IETF working groups, Khronos specification text and registries, the RISC-V ISA and profile documents, Unicode), including their commits and pull requests;
  - prose documents hosted in repositories (design documents, READMEs, `docs/*.md`, wikis) when no code file was opened (OD-7);
  - generated documentation (docs.rs API pages, not `/src/`) and crates.io crate pages;
  - issue and bug discussion text, release notes, advisories and changelogs;
  - licence files;
  - standards data, including Adobe's CMaps, CID-to-Unicode tables and the Adobe Glyph List (PD-D8);
  - MDN's browser-compat-data (documentation data);
  - test-suite data and descriptions (WPT, test262 results, html5lib-tests);
  - repository landing pages, when the claim is about the project and not its internals.
- **Allowed and flagged for the owner:** the Chrome DevTools Protocol definition JSON. It is kept as a published protocol definition under DT-D11, the same reading as the DevTools spec.
- **Platform SDK headers:** used only to call the OS interface they declare (M-1 part 1), never as evidence.
- **Disallowed:** everything else from a project's repository that is code, for example code files, headers, build manifests, OS configuration files, commits, patches, pull requests and source-browser views.

## Summary

| Spec or note set | Items checked | Allowed | Replaced | Measurement | Removed | Revision |
|---|---|---|---|---|---|---|
| JavaScript engine | 131 | 120 | 4 | 6 | 1 | 0.4 → 0.4.1 |
| Resource footprint | 75 | 68 | 1 | 5 | 1 | 0.4 → 0.4.1 |
| HTML and CSS parsing and style | 161 | 161 | 0 | 0 | 0 | 0.3.3 → 0.3.4 (evidence-rule wording) |
| Fonts and text | 105 | 103 | 1 | 1 | 0 | 0.3.4 → 0.3.5 |
| Networking | 84 | 77 | 3 | 2 | 2 | 0.3.3 → 0.3.4 |
| Layout | 51 | 51 | 0 | 0 | 0 | 0.4 → 0.4.1 (evidence-rule wording) |
| Images and media | 33 | 33 | 0 | 0 | 0 | 0.4, unchanged |
| Graphics and rendering | 145 | 142 | 1 | 1 | 1 | 0.4 → 0.4.1 |
| Content blocking | 57 | 55 | 0 | 0 | 2 | 0.3 → 0.3.1 |
| DOM, bindings and event loop | 110 | 108 | 2 | 0 | 0 | 0.2 → 0.2.1 |
| Accessibility | 138 | 124 | 6 | 5 | 3 | 0.2 → 0.2.1 (and AX-D25) |
| PDF | 45 | 45 | 0 | 0 | 0 | 0.2 → 0.2.1 (PD-D8 decided) |
| Phase 2 reconciliation | 37 (34 links, 3 claims tied to them) | 31 | 2 | 1 | 0 | notes, no version |
| Phase 3 amendments | 0 code-host links | — | — | — | — | notes, unchanged |
| **Total** | **1,172** | **1,118** | **20** | **21** | **10** | |

**Decisions whose support changed.** NW-D9's rationale loses a pull-request example and now rests on measurement and CDN77's published description; the decision is unchanged. GR-D8's DXBC fallback, already unproven, now depends entirely on black-box derivation against the in-box compiler (spike GR-S1). The macOS purge path in the footprint spec has no published prose and rests on FP-M0's measurement, which the spec already required. AX-D25 is decided by the owner against the recommendation (below). PD-D8 is decided as recommended.

**Thresholds.** None changed. NW-4's case count becomes 21† (read from a test tool's code file, now confirmed from the runner's output at NW-M3); its pass condition is unchanged. The accessibility counts of macOS 26 web attributes (59†, 32†, 42†) become AX-M0 measurements.

**Owner decisions recorded.** AX-D25 (accessibility): **no third-party source, ever**. This is the owner's choice and not the spec's recommendation. Screen-reader behaviour is learned from documentation, developer guides, issue discussions and black-box testing in the AT lab. The cost is 1–2 more engineers† and two more Windows lab machines (details in the accessibility section). PD-D8 (PDF): Adobe's CMaps, CID-to-Unicode tables and the Adobe Glyph List are standards registry data, as recommended.

**Owner may want to look at.**
- **CDP definition JSON (DT-D11):** whether the protocol definition JSON stays allowed.
- **MD-D4 (a):** the images and media spec still forbids reading the RFC 6386 and RFC 6716 code components. Under 2.3.3 that code is IETF specification text, so the decision is stricter than the rule. **The owner was asked on 20 October 2026 and chose to keep it strict**: the RFCs' code components are never read; the narrative, the test vectors and the black-box reversal route stand as written.
- **Paul Irish's "What forces layout" gist:** kept as third-party prose. It is derived from Blink source, and the layout spec already excludes it as an input.

## Izcal JavaScript engine (draft 0.4 → 0.4.1)

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| JS-1 | `reports/Izcal JavaScript engine.md` Section 6 "A shadow stack per segment" (x86 sentence) | Linux x86 shadow-stack kernel patch posting (LKML archive, July 2023) | OS component patch | Replaced | Kernel x86 shadow-stack documentation and arm64 GCS documentation (`map_shadow_stack`, `SHADOW_STACK_SET_TOKEN`); restore token and RSTORSSP from the Intel SDM. That x86's `map_shadow_stack` writes the restore token is checked black-box at JS-M0 |
| JS-2 | `reports/Izcal JavaScript engine.md` Section 12 unverified list, "Linux" item | Same x86 kernel patch posting | OS component patch | Replaced | Same kernel documentation and Intel SDM; x86 token placement listed as untested |
| JS-3 | `riscv64.md` 2.2 | linux-riscv list message carrying the "RISC-V: mm: Update documentation and include test" patch (July 2023) | OS component patch | Measurement | Kernel's default riscv64 `mmap` placement relative to 2^47 observed black-box at JS-M0. Spec Section 6.18 "Address space" now says the engine places its reservations below 2^47 itself with explicit hints (first principles), so nothing depends on the kernel default |
| JS-4 | `riscv64.md` 2.2 | Sv57 documentation patch (LKML archive, January 2024) | OS component patch | Measurement | As JS-3 |
| JS-5 | `riscv64.md` 2.2 | Linux kernel commit for v6.11 (mmap hint search), unlinked hash | OS component commit | Measurement | As JS-3 |
| JS-6 | `riscv64.md` 3.6 | riscv user-CFI series v16 patch (linux-riscv list, May 2025) | OS component patch | Replaced | LWN, March 2025, reporting the series' cover letter: riscv implements `map_shadow_stack` |
| JS-7 | `riscv64.md` 3.6 | riscv user-CFI series v26 patch "Implement map_shadow_stack() syscall" (LKML archive, December 2025) | OS component patch | Measurement | "Kernel needs no token / flag places one at the base" checked black-box at JS-M0; `ssamoswap` from the ISA manual, tokens as software convention from the kernel Zicfiss documentation |
| JS-8 | `riscv64.md` 3.8 | GCC Zicfiss code-generation patch (sourceware patchwork, January 2025) | Toolchain patch | Measurement | No prose source found (GCC 15 release notes do not mention Zicfiss/Zicfilp); toolchain CFI status checked black-box at JS-M0 (CFI-marked test binary, ELF property, run under kernel user CFI). Affects only C libraries a renderer links, not generated code |
| JS-9 | `riscv64.md` 3.8 | glibc `--enable-cfi` v5 series (sourceware patchwork, June 2026) | C library patch | Measurement | As JS-8 |
| JS-10 | `riscv64.md` "Could not verify" | Unlinked: "patches from September 2024 call them ratified" | OS/toolchain patches (unlinked claim) | Replaced | The RISC-V CFI specification's v1.0 document, marked Ratified (specification text) |
| JS-11 | `history_postmortems.md` 10.9 and its open-items line | webkit-changes commit-list message (July 2023) for JSC's IPInt | Engine commit | Removed | Row keeps WebKit bug 260083 (filed 11 August 2023: in-place interpreter OSR to BBQ and OMG); the "commits from July 2023" date is dropped; argument (Wasm tiering convergence, W0 design) unaffected — the spec's W0 row cites Titzer's paper and WebKit's JetStream 3 post |

Allowed (120: 20 in the spec, 100 in the notes): 62 specification repositories (tc39/ecma262 incl. `esmeta-ignore.json` and its workflow, WebAssembly, riscv-isa-manual and riscv-profiles), 8 prose documents in repositories (V8 sandbox README, Hermes design docs, xsnap README, v8-riscv wiki), 7 issues and security advisories, 31 repository landing pages and project metadata (claims about the project, not its internals), 9 test262.fyi result-data files (test-only data), 3 prose mailing-list messages (a Squeak vm-dev quote; Röttger's LKML reply "PKU usage improvements for threads", twice, which has no diff).

## Izcal resource footprint (draft 0.4 → 0.4.1)

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| FP-1 | `reports/Izcal resource footprint.md` Section 4.2, "Release memory so the charge drops" row | xnu kernel header `sys/mman.h` (Apple open-source mirror) | OS component header | Measurement | Advice values named as the macOS SDK's `madvise` interface; Apple publishes no prose on their semantics; effect on `phys_footprint` measured black-box at FP-M0 (already required since 0.3.1) |
| FP-2 | `reports/Izcal resource footprint.md` Section 6.7, macOS discardable row | Same xnu header | OS component header | Measurement | As FP-1; the macOS path stays gated on FP-M0 |
| FP-3 | `factcheck.md` macOS `MADV_FREE_REUSABLE` row | Same xnu header | OS component header | Measurement | As FP-1 |
| FP-4 | `os_mechanisms.md` 5.3 | Same xnu header | OS component header | Measurement | As FP-1 |
| FP-5 | `source_evidence_pass.md` (0.3.1 log) S4, plus plain-text mentions in F4 and O4 | Same xnu header | OS component header | Measurement | Link removed; the 0.3.1 rows note "link removed at 2.3.3" |
| FP-6 | `os_mechanisms.md` 2.4 (and the 0.3.1 log's closing note, which had kept it) | dyld header `fixup-chains.h` (third-party source browser) | OS component header | Replaced | Apple WWDC22 "Link fast: Improve build and launch times" (page-in linking: the kernel applies chained fixups to DATA pages on page-in; chained-fixup binaries only); Emerge Tools' iOS 15 post keeps the page counts |
| FP-7 | `compact_engine_data.md` 11.3 | Zig compiler commit (mirror) and its figure "8-byte cut to IrInstruction: 3.51 → 3.41 GiB" | Compiler commit | Removed | Figure and link removed; the data-oriented-design row stands on the talk summary (−40 % peak memory, secondhand, Low); the spec cites only the talk percentages |

Allowed (68: 15 in the spec, 53 in the notes): 62 prose documents in repositories (Chromium `docs/memory`, `memory-infra`, `process_model_and_site_isolation`, `linux/zygote`, `how_cc_works`, `android_native_libraries`, PartitionAlloc.md, binary-size explainer and README and seatbelt design and orderfile docs pinned at a revision, jemalloc TUNING.md, Ladybird ProcessArchitecture.md and README), 1 docs.rs API page (`string_cache`), 1 specification-repository link, 4 prose mailing-list and forum messages (two webkit-dev threads, an Ubuntu bug-list message, a blink-dev thread). Android `source.android.com/docs` pages are documentation, not source.

## Izcal HTML and CSS parsing and style

Draft 0.3.3 -> 0.3.4. 161 items checked (95 code-host links, 66 unlinked-claim candidates; 26 in the report, 135 in the notes). No disallowed item.

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| — | — | none | — | — | — |

Allowed: 161. By class (approximate tallies): specification repositories (whatwg/html commits and issues, w3c/webref, csswg) 47; prose design documents in repositories (Blink style-invalidation, style-calculation and ComputedStyle docs; Ladybird CSSGeneratedFiles doc; Firefox parser README) 11; repository landing pages and READMEs for project facts (html5lib-tests, html5ever, rust-cssparser, lightningcss, lexbor, Domato, SatCSS, Stylo, validator htmlparser, MDN browser-compat-data, visited-links explainer) 20; issues (Ladybird issue 2146) 3; test-suite data 14; unlinked-claim candidates that are evidence-rule statements, Izcal's own tooling or records of the 0.3.1 pass 66. Wording only (not counted as items): the report's Section 1 M-1 item and Section 7 evidence sentence widened to every third-party project; Section 7 now accepts commit messages only from specification repositories.

## Izcal fonts and text

Draft 0.3.4 -> 0.3.5. 105 items checked (83 code-host links, 22 unlinked-claim candidates; 13 in the spec, 92 in the notes). Two disallowed items, both in the notes; nothing in the spec rested on them.

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| F1 | research_notes/Izcal fonts and text/gaps_opportunities.md:77 (GO 5.6), method line 10 | Android's font configuration file (fonts.xml) in the Android frameworks source repository: family, entry and alias counts and the fallback matching rule read from it | OS component config file in a source repository | Measurement (plus replacement for the deprecation fact) | Deprecation now cites Android's "Implement custom fonts" documentation (verified: fonts.xml "is being deprecated", font_fallback.xml from Android 15) and the SystemFonts API reference (verified: "Returns all available font files in the system"); inventory and matching rule measured black-box on Android reference devices at the Android port (S-8). The 0.3.1 log's "kept as OS data" line is annotated as withdrawn |
| F2 | research_notes/Izcal fonts and text/formats_specs.md:25 (FS 1.8) | Kaitai Struct TrueType definition file (.ksy) and its line count | Third-party code file (format-definition DSL) | Replaced | Kaitai format gallery's TrueType page (verified: MIT, lists classic TrueType tables only, no GSUB, GPOS or CFF); the line count is dropped. The "no official machine-readable description" argument stands on the Microsoft and ISO prose |

Allowed: 103. By class (approximate tallies): issues and release notes (HarfBuzz issue 2707 and releases 2.0.0 and 2.7.3, WebKit and Mozilla standards positions, Noto CJK releases) 17; repository landing pages and READMEs for project facts (fontations, OTS, rustybuzz, HarfRust, Allsorts, swash, Parley, cosmic-text, font-rs, Pathfinder, Vello, fontdue, etagere, wuffs, BudouX, Graphite, llama.ttf) 46; prose documents in repositories (HarfBuzz wasm-shaper doc, Chromium IDN doc) 2; HarfBuzz DocBook manual source (prose; link switched to the rendered "Why is it called HarfBuzz?" page, which states the FreeType origin) 1; licence file (Noto CJK) 1; repository metadata (contributor graphs) 3; specification-owner data (Microsoft's USE override data files) 4; third-party data, not code (TeX hyph-utf8 pattern files: counts and per-file licence headers, no algorithm read) 1; docs.rs crate page (vello_cpu) 1; distribution changelog (openSUSE HarfBuzz changes) 1; unlinked-claim candidates (evidence-rule statements, the 0.3.1 log, Izcal's own tooling) 22, less the two above counted as disallowed. Wording only: the history note's method line no longer lists commit messages of non-specification projects as a source; the spec's Section 1 M-1 item widened.

## Izcal layout

Draft 0.4 -> 0.4.1. 51 items checked (26 code-host links, 25 unlinked-claim candidates; 7 in the spec, 44 in the notes). No disallowed item.

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| — | — | none | — | — | — |

Allowed: 51. By class (approximate tallies): specification repositories (CSSWG drafts issues, WPT) 14; Servo wiki Layout Engines Report (prose) 6; Readium CSS documentation (prose Markdown) 2; Paul Irish's "What forces layout/reflow" gist 3 (flagged: third-party prose built by its author from Blink source and linking into it; allowed as prose, its source links never followed, and the spec already excludes it as an input to the forcing table); unlinked-claim candidates (M-1 notes, the Gecko app-unit range already moved to a black-box measurement, Izcal's own tooling) 25. Wording only: Section 1's summary of M-1 and Section 7's evidence paragraphs widened to every third-party project.

## Izcal networking (0.3.3 -> 0.3.4)

84 items checked (80 links, 4 unlinked claims): 77 allowed, 3 replaced, 2 measurement, 2 removed.

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| N1 | research_notes/Izcal networking/performance_transport.md:47 (PT 3.2) | moq-dev pull request 2468 (a BBRv3 overflow panic in a Rust MoQ library) | Pull request (code review) | Removed | The sentence and link are gone; the row keeps the IETF CCWG draft and the CDN77 blog |
| N2 | reports/Izcal networking.md, NW-D9 rationale | "BBR's complexity has produced panics elsewhere", resting on N1 | Unlinked claim dependent on removed source | Removed | Rationale now: measured-safe, and BBR "much more computationally complex" in CDN77's QUIC measurements (PT 3.2). NW-D9 unchanged; its support is slightly thinner |
| N3 | research_notes/Izcal networking/performance_transport.md:112 (PT 8.3) | quic-interop-runner test-definition file (testcases_quic.py): link delay, bandwidth, queue, long-RTT, loss and corruption rates, goodput and cross-traffic parameters | Test tool source file | Measurement | Design facts (containers, ns-3 simulation, both roles) now from Seemann and Iyengar, EPIQ 2020 and the IETF blog; the parameter values are recorded black-box from the runner's run logs at NW-M3 (NW-4) |
| N4 | research_notes/Izcal networking/protocols_rfcs.md:102 (PR 3.7) | same test-definition file (raw view): the list of 21 test cases and 2 measurements | Test tool source file | Replaced | The runner's published QUIC test descriptions (quic.md, prose in the repository) and the EPIQ 2020 paper; 13 documented run cases, versionnegotiation disabled; the current case list is observed from the runner's output at NW-M3 |
| N5 | reports/Izcal networking.md, NW-4 | "every run test case (21)": count read from the same file | Unlinked value from source | Measurement | 21†, counted from the runner's output at NW-M3; the target (every run case passes against at least 5† stacks) is unchanged |
| N6 | research_notes/Izcal networking/protocols_rfcs.md:3 | Method note "test-suite source trees" | Method statement implying source reading | Replaced | Now "test suites' published descriptions", with a 2.3.3 note |
| N7 | research_notes/Izcal networking/fetch_privacy_partitioning.md:6 | Method note "Browser source code was read only for stated rationale" | Method statement implying source reading | Replaced | Now states that no third-party source is evidence and points to the 0.3.1 withdrawal at row 1.9 |

Allowed (77 links): 28 prose (Chromium docs/*.md and FAQ prose, Lighthouse and msquic docs, webpagereplay README, docs.rs API pages for rustls), 25 repository landing pages (explainers, QUIC stacks named as projects, test tools), 11 issues, release notes and advisories, 5 docs.rs crate and API pages, 6 specification-repository links (whatwg/fetch, whatwg/websockets, w3c/webdriver-bidi), 2 Chrome DevTools Protocol definition JSON links (gaps_opportunities.md 4.3 and 8.3), kept as a published protocol definition under DT-D11 and **flagged** for the owner. The existing 0.3.1 log (source_evidence_pass.md) names removed Chromium files in plain text only; a 2.3.3 resolution note was added to it.

## Izcal content blocking (0.3 -> 0.3.1)

57 items checked (56 links, 1 unlinked reference): 55 allowed, 0 replaced, 0 measurement, 2 removed.

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| C1 | research_notes/Izcal content blocking/critique.md:278 | urlhaus-filter commit page (a filter-list project's commit labelling Tranco "MIT") | Commit (shows a diff) | Removed | Link replaced by a plain-text mention; the claim was already dropped from LG 2.5 in 0.2 and no decision used it |
| C2 | research_notes/Izcal content blocking/factcheck.md:168 | same commit page, named by its address in code format | Unlinked reference to a commit | Removed | Now "urlhaus-filter commit page (address removed in the 2.3.3 pass)" |

Allowed (55 links): 36 prose (uBlock Origin, uBOL and Brave wiki pages, DuckDuckGo's data-model document), 12 repository landing pages of filter lists and list tools (list data, read for README prose), 5 issue lists and release notes, 1 organisation page, 1 project-index page.

## Phase 2 reconciliation (reconciliation.md, reconciliation_2.md, licence_audit.md)

37 items checked (34 links, 3 claims tied to them): 31 allowed, 2 replaced, 1 measurement, 0 removed. reconciliation.md and reconciliation_2.md had no code-host links and no source-derived claims.

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| R1 | research_notes/Phase 2 reconciliation/licence_audit.md:71 (2.4) | Chromium's HSTS preload list file under src/net, its licence read from the file header | Engine source file | Replaced | hstspreload.org (Chrome maintains the list; Firefox, Safari and Edge base theirs on it) and Chromium's LICENSE file (BSD-3-Clause); the file is never opened. NW-D25 (Izcal's own list) is unchanged |
| R2 | research_notes/Phase 2 reconciliation/licence_audit.md:69 (2.2) | rustls build manifest (Cargo.toml) on the main branch: webpki optional, subtle mandatory in 0.24-dev | Build manifest | Measurement | Restated as an expectation for rustls 0.24 (pre-releases dev.0 to dev.2 on crates.io), to be confirmed from crates.io dependency metadata of the first stable 0.24 release. No decision depends on it: NW-D22 already makes TLS Izcal's own |
| R3 | research_notes/Phase 2 reconciliation/licence_audit.md:74 (2.7) | hashbrown build manifest: `default-hasher` enables `foldhash` | Build manifest | Replaced | docs.rs feature-flag page for hashbrown (verified: `default` includes `default-hasher`, which enables `foldhash` ^0.2) |

Allowed (31 links): 30 licence files (including Rust's COPYRIGHT file and specification repositories' LICENSE files), 1 issue-tracker landing page.

## Phase 3 amendments (ledger.md, applied_js.md, applied_footprint.md)

0 links to code hosts. One plain-text mention (ledger G-24 names a V8 source file that the coding standard cites) is a record of a violation in another document, not evidence; allowed. No change.

Note: the session's web-search budget ran out during this slice; every replacement above was verified by direct fetch (hstspreload.org, Chromium LICENSE, crates.io API, docs.rs features, the runner's quic.md, the IETF blog), and the EPIQ paper link was taken from a search result page, not re-fetched.

## Izcal graphics and rendering (0.4 -> 0.4.1)

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| GR-1 | research_notes/Izcal graphics and rendering/compositing_scheduling.md:76 (row 3.5) | Raw link to `protocol/wayland.xml` in the Wayland implementation repository (freedesktop GitLab) | Protocol XML in an implementation repository; a rendered protocol document exists | Replaced | The Wayland protocol specification, Appendix A, on wayland.freedesktop.org, and Wayland Explorer's wl_surface page. Checked 20 October 2026: commit assigns pending damage as current damage, the server clears it as it repaints, damage_buffer takes buffer coordinates and is "since 4", frame callbacks "should avoid signaling ... if the surface is not visible in any way", and the opaque region is "an optimization hint for the compositor". Same facts as before. |
| GR-2 | reports/Izcal graphics and rendering.md 6.12 (GR-D8), Sections 8, 9.4, 10 and 11; research_notes/.../gpu_apis_platforms.md row 3.14; critique.md E2 | DXBC token format, which is published only in a Windows Driver Kit header. The critique had called this header "fine to read for the token layout". | Unlinked; OS component's header (source) | Measurement | Spike GR-S1 at GR-M0 now derives the DXBC token encoding black-box, alongside the checksum, from the in-box `d3dcompiler_47.dll` used as a test-only oracle (inputs and outputs only). The header is not opened. No † number is needed. GR-D8's DXBC reversal route was already marked unproven and is now less supported. GR-D8 itself (self-hashed DXIL) is unchanged. |
| GR-3 | reports/Izcal graphics and rendering.md Section 7, dependency table row | "Khronos `vk.xml`, Vulkan headers" given as inputs to the generated Vulkan bindings | Unlinked use of Khronos's generated C headers | Removed | The row now names `vk.xml` (the registry) alone. The headers are generated from it, so the argument still holds: the bindings are generated from standards data under Apache-2.0 OR MIT. |

Allowed (142): specification repositories and registries 13 (gpuweb, w3c, WICG, whatwg issue, WPT tree and licence, Khronos Vulkan-Docs `vk.xml` and licence), SPIR-V grammar JSON as Khronos standards data 1, prose documents in repositories 44 (Chromium docs/*.md under OD-7, Vulkan loader docs, `DXIL.rst`, INF-0004, chrome-devtools-mcp docs, X.org Present protocol text, WebGL tests README, Rust-GPU README), crates.io crate pages and API metadata 55, docs.rs crate pages (not /src/) 3, licence files 21, issues 1, repository landing pages 2 (wayland-protocols, playwright-mcp README), Fuchsia documentation 2. Unlinked mentions of engine files in the notes' "Sources not used" lists are records of sources that were not opened, so they are kept. Test-suite file counts (WebGPU CTS spec files, WebGL CTS HTML files, WPT fuzzy annotations) come from test-only inputs and are allowed.

## Izcal images and media (0.4, unchanged)

No disallowed items: 32 links and 1 unlinked use, all allowed. Links: specification repositories and issues 5 (csswg-drafts, ColorWeb-CG, WPT interop), prose documents in repositories 9 (Chromium Rule of 2 and audio-service README, WebRTC NetEq g3doc, castLabs and xbmc wikis, MNG spec page), repository landing pages and READMEs 15 (Wuffs, Compact-ICC-Profiles, iccDEV, intel media-driver, servo/media, oss-fuzz, MicrosoftEdge videotest, ORB explainer), issues 2, and a docs.rs crate page 1 (rav1d-safe, README prose).

The unlinked use is the code components inside RFC 6386 (VP8) and RFC 6716 (Opus) under MD-D4. These are IETF specification text, which 2.3.3 allows. MD-D4 (a), accepted by the owner, already rules out reading them except through its logged reversal route. That is stricter than 2.3.3 requires, and the decision is unchanged.

## Izcal DOM, bindings and event loop (draft 0.2 -> 0.2.1)

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| DM-1 | research_notes/Izcal DOM, bindings and event loop/bindings_object_model.md:83 (row 4.6) | WebKit DOMJIT: "declared read/write heap-effect description" of DOM getters, from researcher recollection of engine internals | Unlinked claim from engine source knowledge | Replaced | WebKit bug 163657 discussion text (comments 1 and 4: nodeType Pure, firstChild NoSideEffect, appendChild SideEffect; a proposed hierarchy-based abstract heap); no patch or commit opened. Confidence Low -> Medium |
| DM-2 | research_notes/Izcal DOM, bindings and event loop/bindings_object_model.md:84 (row 4.7) | SpiderMonkey JSJitInfo: "Ion could call DOM getters directly through per-member JIT info", from recollection of engine internals | Unlinked claim from engine source knowledge | Replaced | Bugzilla 747288 discussion text (Generate faster jitcode for DOM getters/setters: readyState x 10^7 about 125 ms vs 137 ms native getter, about 97-100 ms later, Safari about 90 ms) and Firefox's Web IDL bindings prose docs; the per-member JIT-info mechanism is no longer claimed. Open question 5 in the same note reworded |

Allowed (108 links): 34 specification-repository links (WHATWG HTML source, DOM, Web IDL, w3c and tc39 repositories, including 6 raw WHATWG HTML source links); 39 prose documents in repositories under OD-7 (Blink bindings README, IDLCompiler.md, V8BindingDesign.md, IDLExtendedAttributes.md, Chromium security and process-model docs, Ladybird AddNewIDLFile.md, webidl2js and js-framework-benchmark READMEs and wiki, Playwright MCP README, web-features baseline doc, js-string-builtins overview); 8 MDN browser-compat-data JSON files (documentation data, not code); 2 WTF-8 specification pages; 1 WPT interfaces listing and 1 Interop 2026 results CSV (test-suite data); 8 crates.io pages; 7 repository landing pages (BCD, Minerva); 1 release-notes page (Playwright 1.45); 1 licence file (webidl2.js LICENSE). Flagged, allowed: js-framework-benchmark's Chrome 154 results were "parsed from the published bundle" (results data shipped in the results page, not benchmark code). Other unlinked statements checked and kept: webidl2.js used only as a test-time oracle (test tool, not evidence); history rows 4.10 (MessageChannel yield hack), 5.12 and 7.11 are researcher recollection of public behaviour, not source reading, and stay Low confidence. Version references: Section 1's M-1 line and izcal_interfaces.md row 1.1 now cite HGBA-2 2.3.3.

## Izcal PDF (draft 0.2 -> 0.2.1)

No disallowed item found.

Allowed (45 links, all in the research notes; the spec itself links no code host): 27 crates.io pages (licence and version facts for excluded or generic crates); 13 repository landing pages (krilla, pdf-writer, subsetter, polyfile, daedalus, URW base35 fonts, Arlington PDF model as ISO-derived specification data used test-only, veraPDF corpus as test-only input, Adobe cmap-resources and agl-aglfn as standards registry data under PD-D8); 3 pdf.js wiki FAQ pages (prose); 2 licence files (Liberation fonts, Google Fonts Arimo directory). Unlinked claims checked: Acrobat's bundled form scripts and other viewers' AF* implementations are never read (spec 6.17, critique C14/E3); values only viewers' source would state stay black-box questions (SE 7.4, WP 11, RT 13, GO 12). Writer tests that inspected Chrome's and WebKit's output PDF bytes are black-box output, not source.

Owner decision recorded: PD-D8 (a), as recommended. Adobe CMaps, CID-to-Unicode tables and the Adobe Glyph List are standards registry data (HGBA-2 2.3.3) and ship with the BSD-3 notice. Edited: status line, PD-D8 summary bullet, 6.8 option verdicts, Section 7 built-new and standards-data paragraphs, 9.3 staffing row, 9.4 decision row and "most needs to see" list, the legal-review list, the risk row, Q-PD-2 (now moot).

## Izcal accessibility (draft 0.2 → 0.2.1)

| # | Where (file:line) | Item (plain-text name, no URL) | Class | Action | Replacement / measurement / note |
|---|---|---|---|---|---|
| AX-1 | spec 2.1 history table (NSAccessibility row) | macOS 26 web-attribute, parameterised-attribute and search-key counts (59, 32, 42) counted from the macOS SDK headers `AXWebConstants.h` and `NSAccessibilityConstants.h` | OS component header as evidence | Measurement | † counts; `hg-ax-probe` lists attribute and parameterised-attribute names on Safari web areas over the corpus at AX-M0 |
| AX-2 | spec 7, Anchors | "Apple's … documentation and SDK headers" as a behaviour anchor | OS component header as evidence | Replaced | Apple documentation; SDK headers only to call the interfaces they declare (AX-D8) |
| AX-3 | spec 9.4 AX-D25, 6.10, critique item 6 | Option (a): read NVDA, Orca and TalkBack source for in-process expectations | Screen-reader source | Removed | Owner decision (b): documentation, issue discussions, vendor liaison and the AT lab's characterisation suite |
| AX-4 | platform_apis 2.5 | "NVDA issue/commit discussion" for injection failure, IA2 proxy registration and in-process crashes | Screen-reader commit | Replaced | Microsoft Learn "COM Proxy" (per-interface proxies implement the marshaling code, loaded from their DLL); NVDA 2026.2 User Guide (UIA only when NVDA cannot inject); Bugzilla 1532603 ("because it is in-process, it takes down Firefox with it"), all verified 20 October 2026 |
| AX-5 | platform_apis 4.3 | `kAXErrorCannotComplete` value from SDK `AXError.h` | OS component header | Replaced | Apple documentation, AXError.cannotComplete = −25204 (verified) |
| AX-6 | platform_apis 4.5 | "SDK headers carry no main-thread annotation" | OS component header | Removed | Row stays recollection, Medium |
| AX-7 | platform_apis 4.6 | Text-marker C API from SDK `HIServices/AXUIElement.h` | OS component header | Replaced | Apple documentation, AXTextMarker (macOS 12+); function list confirmed by `hg-ax-probe` at AX-M0 |
| AX-8 | platform_apis 4.7 | 59 web attributes, 5 attributed-string keys from `AXWebConstants.h` | OS component header | Measurement | † counts, AX-M0 black-box enumeration |
| AX-9 | platform_apis 4.8 | 32 parameterised attributes, 8 notifications, 56 subroles from the same header | OS component header | Measurement | † counts, AX-M0 |
| AX-10 | platform_apis 4.9 | 42 search keys from `NSAccessibilityConstants.h` | OS component header | Measurement | † count, AX-M0 probe of VoiceOver's search requests |
| AX-11 | platform_apis 4.10 | 57 symbols new in macOS 26 counted in the header | OS component header | Measurement | † count from Apple's AppKit documentation at AX-M0 |
| AX-12 | platform_apis 4.11 | SDK `AXAttributeConstants.h` beside Apple's announcementRequested page | OS component header | Removed | Apple documentation page alone carries the row |
| AX-13 | platform_apis 4.15 | SDK `NSApplication.h` for `isFullKeyboardAccessEnabled` | OS component header | Replaced | Apple documentation, NSApplication.isFullKeyboardAccessEnabled (macOS 10.6, verified) |
| AX-14 | platform_apis 4.16 | Caption functions from SDK `MACaptionAppearance.h` | OS component header | Replaced | Apple documentation, Media Accessibility: Captions (function list verified) |

Allowed (124 link occurrences): 36 specification repositories (w3c/aria, html-aam, core-aam and similar), 59 prose documents in repositories (Chromium `docs/accessibility/*.md`, at-spi2-core developer documentation, MSEdgeExplainers, NVDA's `nvdaHelper` readme, AccessKit, axe-core and chrome-devtools-mcp READMEs and docs), 12 project landing pages, 7 crate pages, 3 issues (NVDA issue 19276, read for the maintainer's comments; a Tor Browser issue), 1 licence file, and 6 citations of the Chrome DevTools Protocol definition JSON in gaps_opportunities 3.1–3.6, **flagged**: kept as a published protocol definition under DT-D11, the same reading as the DevTools spec. Searchfox's `nsCSSFrameConstructor.cpp` in performance_architecture 1.11 names a test page (a 12,000-row table) used in Mozilla's published measurement, not code that was read.

**AX-D25.** The owner chose "no third-party source, ever", not the spec's recommendation (a). The decision row, 6.10, Section 7's evidence rule, critique item 6, Appendix A's finding 22 and engine-source item 6 now say this. Screen-reader behaviour comes from documentation, developer guides, vendor liaison, issue discussions (text only) and the AT lab. `hg-ax-probe` enumerates Chrome's and Firefox's platform output, and a new **characterisation suite** serves ablation pages that add or remove one attribute, event or convention at a time, then records each reader's speech, braille and browse-mode output. The results feed AX-7, AX-12 and AX-13. Cost: the reference and verification track grows from 4–5† to 5–7† engineers, and full strength from 30–38† to 31–40† (33–41† to 34–43† with Android). Engineer-years to AX-M3 go from about 105–170† to about 108–180†. The AT lab grows from six to eight Windows machines. AX-D8, which treats the IA2 definitions and Apple constants as OS-interface definitions used only to call those interfaces, is unchanged. No target or threshold changed.

## Out of scope: hits listed only (no edits)

HGBA-2's superseded passages and research notes, HGBA-3, the Stage 1 design, the Stage 1 design facts and the essay reviews are outside this pass. Their links to non-specification code are listed here only; they are removed when those documents are rewritten (HGBA-2 2.4.0 and the Stage 1 rewrite). A scan of the Stage 1 design and the Feature design essays review found no hits.

| # | Where (file:line) | Item (plain-text name) |
|---|---|---|
| O-1 | reports/HGBA 2 browser architecture spec.md:136 | V8 file code-memory-access-inl.h |
| O-2 | reports/HGBA 2 browser architecture spec.md:337 | V8 file v8-embedder-heap.h |
| O-3 | reports/HGBA 2 browser architecture spec.md:427 | Chromium file rusty_png_feature.h |
| O-4 | reports/Future software browser engine.md:169 | Taffy file taffy_tree.rs |
| O-5 | reports/Future software browser engine.md:169 | Blitz file document.rs |
| O-6 | reports/Future software browser engine.md:185 | commit in a git.joshthomas.dev repository |
| O-7 | reports/Future software browser engine.md:294 | Blitz file document.rs |
| O-8 | reports/Future software browser engine.md:456 | Blitz file document.rs |
| O-9 | research_notes/Stage 1 design facts/facts.md:49 | rusty_v8 file build.rs |
| O-10 | research_notes/Stage 1 design facts/facts.md:111 | rusty_v8 file platform.rs |
| O-11 | research_notes/Stage 1 design facts/facts.md:112 | rusty_v8 file binding.cc |
| O-12 | research_notes/Stage 1 design facts/facts.md:113 | V8 file flag-definitions.h |
| O-13 | research_notes/Stage 1 design facts/facts.md:114 | Deno file setup.rs |
| O-14 | research_notes/Stage 1 design facts/facts.md:144 | servo/servo pull request 44114 |
| O-15 | research_notes/Stage 1 design facts/facts.md:145 | servo/servo pull request 44454 |
| O-16 | research_notes/Stage 1 design facts/facts.md:146 | Servo file Cargo.toml |
| O-17 | research_notes/Stage 1 design facts/facts.md:199 | Stylo file dom.rs |
| O-18 | research_notes/Stage 1 design facts/facts.md:200 | Blitz file stylo.rs |
| O-19 | research_notes/Stage 1 design facts/facts.md:210 | html5ever file tree_builder.rs |
| O-20 | research_notes/Stage 1 design facts/facts.md:234 | Taffy file traits.rs |
| O-21 | research_notes/Stage 1 design facts/facts.md:343 | WPT tooling file products.py |
| O-22 | research_notes/Stage 1 design facts/facts.md:344 | WPT tooling file ladybird.py |
| O-23 | research_notes/Stage 1 design facts/facts.md:346 | V8 file testcfg.py |
| O-24 | research_notes/Stage 1 design facts/facts.md:373 | Fuzzilli file libreprl.h |
| O-25 | research_notes/Stage 1 design facts/scope_completion_factcheck.md:37 | Servo file prefs.rs |
| O-26 | research_notes/HGBA 2 browser architecture spec/emerging_hardware.md:55 | commit in a chromium.googlesource.com repository |
| O-27 | research_notes/HGBA 2 browser architecture spec/emerging_hardware.md:55 | Firefox (via Searchfox) file riscv.gni |
| O-28 | research_notes/HGBA 2 browser architecture spec/engines_and_languages.md:84 | Chromium file features.cc |
| O-29 | research_notes/HGBA 2 browser architecture spec/engines_and_languages.md:87 | Chromium file rusty_png_feature.h |
| O-30 | research_notes/HGBA 2 browser architecture spec/gpu_npu_compute.md:60 | webmachinelearning/webnn-docs pull request 32 |
| O-31 | research_notes/HGBA 2 browser architecture spec/gpu_npu_compute.md:164 | commit in a dawn.googlesource.com repository |
| O-32 | research_notes/HGBA 2 browser architecture spec/isolation_security.md:40 | V8 file code-memory-access-inl.h |
| O-33 | research_notes/HGBA 2 browser architecture spec/memory_storage_network.md:130 | Chromium mirror file initialize_mojo_core.cc |
| O-34 | research_notes/HGBA 2 browser architecture spec/memory_storage_network.md:153 | commit in a quiche.googlesource.com repository |
| O-35 | research_notes/HGBA 2 browser architecture spec/wasm_component_model.md:125 | fastly/js-compute-runtime pull request 541 |
| O-36 | research_notes/HGBA 2 browser architecture spec/wasm_component_model.md:150 | bytecodealliance/wasmtime pull request 6240 |
| O-37 | research_notes/HGBA 2 browser architecture spec/wasm_component_model.md:151 | cobalt.googlesource.com file riscv.gni |
| O-38 | research_notes/Future software browser engine/data_oriented_ui.md:19 | commit in a skia.googlesource.com repository |
| O-39 | research_notes/Future software browser engine/data_oriented_ui.md:21 | linebender/vello pull request 1786 |
| O-40 | research_notes/Future software browser engine/data_oriented_ui.md:21 | linebender/vello pull request 1926 |
| O-41 | research_notes/Future software browser engine/data_oriented_ui.md:28 | Taffy file taffy_tree.rs |
| O-42 | research_notes/Future software browser engine/data_oriented_ui.md:76 | Blitz file document.rs |
| O-43 | research_notes/Future software browser engine/data_oriented_ui.md:76 | Blitz file node.rs |
| O-44 | research_notes/Future software browser engine/data_oriented_ui.md:77 | Blitz file Cargo.toml |
| O-45 | research_notes/Future software browser engine/data_oriented_ui.md:131 | Taffy file taffy_tree.rs |
| O-46 | research_notes/Future software browser engine/data_oriented_ui.md:132 | Blitz file node.rs |
| O-47 | research_notes/Future software browser engine/data_oriented_ui.md:159 | Blitz file document.rs |
| O-48 | research_notes/Future software browser engine/data_oriented_ui.md:202 | commit in a skia.googlesource.com repository |
| O-49 | research_notes/Future software browser engine/incremental_computation.md:18 | commit in a git.joshthomas.dev repository |
| O-50 | research_notes/Future software browser engine/incremental_computation.md:108 | Blink file layout_ng_mixin.cc |
| O-51 | research_notes/Future software browser engine/incremental_computation.md:131 | commit in a git.joshthomas.dev repository |
| O-52 | research_notes/Composable platform essay review/factcheck_A.md:90 | uBlock Origin file mode-manager.js |

