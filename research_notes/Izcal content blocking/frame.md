# Izcal content blocking: frame

Compiled 19 October 2026 by the synthesis author (phase 1 of the first-principles-spec method). Sources are Izcal's own specifications; IDs are quoted from them. No web research in this file.

## 1. The problem and who has it

| User | Task, in their words | What content blocking must give them |
|---|---|---|
| People browsing (default settings) | "Sites should not follow me around, ads should not eat my battery, and pages should still work" | Default-on protection against cross-site tracking, fingerprinting scripts, malvertising and cryptominers; no visible breakage; nothing to configure |
| People who know what they want | "Block more here, block nothing there, add the list I trust" | Per-site and per-container controls, user-added third-party lists in the syntax they already use, a way to see what was blocked and why |
| People reporting breakage | "This site broke; fix it without making me send you my browsing" | A one-click, consented, minimal report; a fast published fix; a working site now (per-site off) |
| Web developers and site operators | "Why did my request fail in Izcal? How do I get a false positive removed?" | A typed reason in DevTools ("blocked by rule R of list L, entry E, evidence X"), a published entry with its evidence, a public process to contest it |
| List curators (Izcal staff) | "Find what tracks, prove it, publish it, and catch evasion quickly" | A crawler that is Izcal itself, with the engine's own attribution (script behind each API call, CNAME chain, partition), a review queue and breakage comparison |
| Izcal's engineers | "The matcher must be fast, small, deterministic, never panic on a hostile list, and replay exactly" | A compiled, sealed automaton; zero allocations per match; recorded list versions; tier-P parsing in a utility process |

## 2. What Izcal's specs already decide or constrain

| Source | ID | Constraint on content blocking |
|---|---|---|
| HGBA-2 2.3.2 | **M-1 part 1** | No library that does browser-engine work: Brave's `adblock` is dropped by name (2.3.0); content blocking is listed among what Izcal builds. Everything shipped, code and data, MIT-available; exceptions: standards data, test-only inputs, OS interfaces and toolchain |
| HGBA-2 2.3.2 | **M-1 parts 2–4** | Web-visible behaviour anchored to pinned specifications; where none governs (filter syntax), documented interoperability evidence recorded as black-box observations; other engines' and blockers' source code never evidence; oracles decide correctness |
| HGBA-2 | **P1–P6, I-1, I-2** | Security and privacy first, compatibility second; never two sites' data in one process; renderers have no sockets |
| HGBA-3 | **P2 scope note** | Default-on interventions are allowed only as class (c): privacy protections with a recorded P1 analysis, publishing compatibility cost per release; class (b): user overrides per site or globally |
| HGBA-3 | **B1, DET-1** | Every decision that changes what content observes is a recorded input; replay is exact |
| HGBA-3 | **B9, B10, AG-0** | Agents are a distinct principal; AI off means zero AI requests; security classifiers only in the published ML inventory |
| HGBA-3 | **PRIV-1, PRIV-2** | CAPTCHA cost of content-blocking defaults measured before shipping and every release; fingerprinting defences measured for linkability, residual entropy and compatibility |
| Stage 1 | **Section 6, "Built-in content blocking"** | Engine hook in network service and style; shell owns lists; not built on an extension API; cosmetic filters as a user-origin sheet (WPT-neutral). Its `adblock` dependency is superseded (2.3.0) |
| Stage 1 | **Section 6, "Containers"** | Block-list set is per-container policy |
| Stage 1 | **Section 6, "Cookie-banner handling"** | Stage 2; Izcal's own rule format; never auto-accept; third-party rules only as data after licence review |
| Stage 1 | **D-S1** | Anti-fingerprinting: seeded per (site, container, session) noise on readback surfaces; discrete surfaces normalized; full uniformity opt-in. Amended by TX-D4 (no `measureText` noise), GR-D12 (no 2D canvas noise; WebGL/WebGPU noise applied once where pixels leave), MD-D14 (no audio noise for Izcal-rendered audio), LY-D11 (geometry normalized by construction) |
| Stage 1 | **D-S3** | No extensions in Stage 1; content blocking is built in; extensions arrive in Stage 2 via Wasmtime |
| Stage 1 | **D-S10** | No code from another browser, whatever its licence |
| Networking 0.3.3 | **6.15, NW-D26** | Request-matching semantics, the matcher (`hg-content-block`, L5, tier P), the cosmetic-rule compiler and Izcal's own lists are owned by networking. Tracking list by measured behaviour; ads list; mostly generic cosmetic rules; every entry reviewed and published with evidence; CNAME uncloaking by `hg-dns`; breakage reports send only URL and rules fired; triage ≤ 7 days†; users may add third-party lists by URL per container; third-party lists used in the lab only to measure coverage |
| Networking 0.3.3 | **6.15 rule syntax** | Izcal writes a rule-syntax page from public documentation of common formats: URL patterns, anchors, `$third-party`, `$domain`, resource types, exceptions, `$important`, `$redirect` to built-in neutral resources, `$removeparam` |
| Networking 0.3.3 | **6.15 partition semantics** | "Third-party" = request site differs from the initiating frame's site (F); `$domain` matched against T and F |
| Networking 0.3.3 | **6.15 placement** | Utility process compiles lists into a sealed automaton file mapped read-only by every partition service; ≤ 256 KiB† charged state per partition service; zero allocation per match; p99 ≤ 20 µs† (FP-28); blocked request is a network error `FailureReason::ContentBlocked`, not page-distinguishable (NW-22) |
| Networking 0.3.3 | **NW-D3, NW-D7** | Partition service per (T, C); partition key (T, C, F, X, N, R, n) |
| Networking 0.3.3 | **NW-D17** | Izcal's own traffic only from the system service; published endpoint list includes Izcal's lists and a third-party list URL only after the user added it; user-specific requests through an OHTTP relay (Q-NW-6); public files (lists) fetched plainly |
| Networking 0.3.3 | **NW-36, NW-54** | Quiet browser (only published endpoints); lists useful and honest (every entry with evidence; ≥ 90%† of classifier-flagged third-party requests blocked; ≤ 1%† corpus breakage; triage ≤ 7 days†) |
| Networking 0.3.3 | **9.3 Lists and Content blocking tracks** | Lists 4–7† (HSTS list included; content-blocking share 3–6†); content-blocking matcher and compilers 2–3†. Counted only in NW |
| Networking 0.3.3 | **6.18, 6.19** | Decision log records the content-blocking rule; "who received what" view; extension request interception (Stage 2) only through the scoped, audited IIP path |
| Networking 0.3.3 | **6.17** | Filter-list checks on a daily† schedule aligned to the wake grid, deferred on battery saver |
| Style 0.3.3 | **6.16, P21** | Cosmetic rules arrive as a user-origin sheet ≤ 256 KiB† per site process, parsed by `hg-css-syntax`, shared per partition; container list set is engine configuration, recorded, covered by PRIV-2's global-sheet case |
| Footprint 0.3 | **6.11, FP-28** | Compiled automata in a strictest-tier utility process; regenerable-cache class (FP-D16); blocked frames start no process; per-site savings measured and published; FP-28 to be amended to per service (reconciliation P-row) |
| Footprint 0.3 | **6.14, FP-D7** | Script-attributed interventions on ad scripts and frames classified by the blocking lists, on sites where blocking is off; recorded B1 inputs; announced |
| Reconciliation | **P21; licence audit 2.5; OD-L5** | Cosmetic filters compiled by NW, applied by ST; EasyList and EasyPrivacy GPL-3.0 or CC BY-SA, not shipped; build own lists |
| Shell UI design | **4.3 site panel** | "Blocked here: 23 trackers, 4 ads [list]" |
| Rule registry | — | G1–G78 are taken; this spec takes **G80 onward** |

## 3. What success means (measurable)

1. Default users get protection that is measured, not asserted: share of cross-site tracking requests (by Izcal's behavioural definition) blocked on a held-out crawl, fingerprinting scripts neutralised, malvertising and cryptomining endpoints blocked.
2. Breakage is measured and small: corpus pages with user-visible breakage from Izcal's defaults ≤ NW-54's 1%†, every report triaged ≤ 7 days†.
3. Every entry is explainable: the list, the entry, its evidence and its review are published; DevTools and the site panel say which rule blocked a request.
4. Performance and memory meet FP-28 as amended per service, with zero allocations per match and no measurable page-load regression from matching.
5. No new fingerprinting surface: the user's list set and per-site settings are not observable cross-site beyond what PRIV-2 allows.
6. Determinism: replay reproduces every blocking decision from the recorded list version and settings.
7. Licence and M-1: zero shipped bytes from third-party lists or blocking engines; zero citations of engine or blocker source code.
