# Izcal feature inventory audit

*Planning document: does Izcal's design cover every feature a browser ships in 2026, and, where it does not, what work closes each gap. Date: 24 October 2026. Status: draft 0.1. Evidence: `research_notes/Izcal feature inventory audit/` (four inventories, four maps, five checks, one fact-check, and the final per-row `ledger.md`; Appendix A). Relates to: HGBA-2 [M-1, 2.3.2's MIT-only rule and its standards-data exception]; HGBA-3 [§7, B9, B10]; the Stage 1 design [§1 process map, §3 substrate, §4 IPC, §6, 7.8, 8.1 milestones, §9 backlog, 13.2]; the 24 component specs and the shell UI design; the security scope [6.15, 11.4, SEC-D14].*

*Revision 0.2.1 (29 October 2026): FI-11 is recorded as a **known gap**: M1 built its shared recording and crash-report header, but FI-11 still has no spec. No work item, decision or count changed.*

*Revision 0.2 (24 October 2026): **the owner decided FI-D1 to FI-D17** (Section 7.1); the work items they change (FI-9, FI-29, FI-31, FI-32, FI-35, FI-37) are updated, and FI-D14 records a project-wide rule: open protocols first. FI-D1 lifts the engine-code hold for M1.*

*Revision 0.1 (24 October 2026): first draft, written by the synthesis author from the evidence notes. No existing report was edited. Every count in this document is computed by a script from the ledger's rows.*

The owner paused engine code on 24 October 2026 because the specs might not cover every feature a browser ships. This audit tested that suspicion against **1,117 features** listed from published prose in four inventories: web platform APIs, CSS, graphics and media, browser product features, and OS integration and operations. **The suspicion is right, but the gaps are not where the engine is.** 556 rows (448 Designed, 108 Owned) are covered, 101 are held by a recorded admission rule, 42 are refused, and 20 are deferred with an owner. **398 rows are gaps**: 174 Missing, 163 Partial, 41 Mentioned and 20 deferred with no owner. The engine-facing inventories are well covered: only 69 of 333 web API rows and 41 of 277 CSS, graphics and media rows are gaps. The product and operations inventories are not: 160 of 270 product rows and 128 of 237 operations rows are gaps. The corpus specifies an engine and a security model in depth, but it barely specifies how the browser is installed, updated, managed, localised, made to comply with the law, or extended into identity, installed apps, real-time media and speech. **481 rows go into 38 work items** (the 398 gaps, 77 holds that rest on the rule alone and 6 implicit refusals): 13 new specs, 20 amendments to named specs and 5 decisions in draft 0.1; after the owner's decisions of 24 October 2026 (FI-D1 to FI-D17, Section 7.1) they are 15 new specs, 22 amendments and 1 decision still open (FI-34, the embedding API). The urgent ones are a legal duty already in force (the EU Cyber Resilience Act's reporting duty, since 11 September 2026, from the first EU distribution) and the ship blockers: installers and updates, enterprise policy delivery, default-browser and OS integration, script dialogs and file pickers, localisation, headless mode and an owner for the print UI. **No gap changes M1's substrate traits or its IPC wire format as built in phases A to D2.** Three small additions belong at M1's exit, and two process-model decisions belong before M5 (Section 8), so the hold can lift for M1 now.

Abbreviations follow the maps: **AH** `reports/Izcal app host.md`; **AN** animation; **AX** accessibility; **CB** content blocking; **DM** `reports/Izcal DOM, bindings and event loop.md`; **DT** DevTools spec; **DV** devices; **ED** editing; **EX** extensions; **FP** resource footprint; **GR** graphics and rendering; **JS** JavaScript engine; **LY** layout; **MD** images and media; **ML** ML service; **NW** networking; **PD** PDF; **PH** phishing protection; **SEC** `reports/Izcal security scope.md`; **SG** storage; **SH** `reports/Izcal shell.md`; **SHUI** `reports/HGBA shell UI design.md`; **ST** HTML and CSS parsing and style; **SV** SVG; **TX** fonts and text; **WC** WebCrypto; **H2** `reports/HGBA 2 browser architecture spec.md`; **H3** `reports/Future software browser engine.md`; **S1** `reports/HGBA Stage 1 architecture design.md`. Each component abbreviation is the file `reports/Izcal <name>.md`. "S1 M5" is a Stage 1 milestone; "SH-M3" and the like are the specs' own milestones. Row IDs (WA-, CG-, BF-, PO-) are the inventories'; the ledger gives each one's final status, owner, stage and the verdicts behind it.

## 1. Frame

**Why the audit.** On 24 October 2026 the owner paused engine code. M1's substrate and IPC skeleton had merged through phase D2 (typed endpoints and the simulated transport). The concern was that 24 component specs, written component by component, might each be sound while the set misses features every shipping browser has. A gap found after M1 could force rework in the substrate, the IPC schema or the process model, which every later milestone builds on.

**The question.** For every feature a 2026 browser ships, does an Izcal spec own it?

**What counts as covered.** A row is **covered** when a spec designs Izcal's behaviour for it (**Designed**), or names it, or a standard that contains it, as in scope with an owner and a milestone, WPT floor or target (**Owned**). A recorded **refusal**, a **deferral** to a named stage with an owner spec, and a **hold** under a recorded admission rule are answers, not gaps. A name with no owner (**Mentioned**), a partial design (**Partial**), a later stage with no owning spec (**Deferred, no owner**) and silence (**Missing**) are gaps.

**The success test.** Every feature a 2026 browser ships has an owner, a stage, or a recorded refusal. By that test the corpus fails today on 398 rows, or 398 plus 77 if each single-engine hold needs its own record (FI-D7). This document makes the test pass on paper: each gap row is assigned to exactly one work item, and each work item names its owner, its stage and what it depends on.

## 2. Method

### 2.1 Inventories from published prose

Four researchers each compiled one inventory on 24 October 2026, from published prose only: specifications and standards indexes, vendor help centres and feature pages, release notes, OS developer documentation, regulation text and licensing bodies' pages. **No source code, commit, code review or repository data file was opened** (M-1 part 3); that includes the Public Suffix List file and the web-features and browser-compat-data sets. The four inventories:

| Inventory | Rows | Grain | Main sources |
|---|---|---|---|
| Web platform APIs (WA) | 333 | Per spec or feature group, 22 groups | WHATWG standards list, MDN's API index, [Interop 2026](https://web.dev/blog/interop-2026), Chrome 141–154, Firefox 141–158 and Safari 26.0–27.1 release notes |
| CSS, graphics and media (CG) | 277 | Per module or format, 16 groups | CSSWG and W3C specs, codec and licensing bodies' pages |
| Browser product features (BF) | 270 | Per user-facing feature, 21 groups; [TS] table stakes, [Diff] differentiator | Feature pages of Chrome, Firefox, Edge, Safari, Vivaldi, Brave, Opera, Zen, Orion, Dia and Arc |
| OS integration and operations (PO) | 237 | Per capability, 17 groups; [SB] ship blocker | Edge and Firefox policy docs, Chrome Enterprise channel docs, Apple's EU engine rules, WebDriver BiDi, OS docs |

### 2.2 Mapping

A mapper per inventory searched the files in `reports/` (30 component and design documents; the three review documents were excluded) for each row with several synonyms, read the matching section, and gave a status: Covered, Partial, Excluded (refused, held or deferred), Deferred or Missing, with the owner, the stage and the evidence.

### 2.3 Five checks, and how often the first mapping was wrong

Five independent passes then checked the maps. A later check overrides an earlier one, in this order: map, the BF and PO Missing check, the WA and CG Missing check, the Covered and Excluded spot-check, the BF and PO re-grade, the WA and CG re-grade. The ledger records every verdict a row received, and where checks disagree it says which one won and why.

| Check | Rows | What it found | Error rate of the earlier verdict |
|---|---|---|---|
| **Missing rows, BF and PO** (`verify_missing_bf_po.md`) | 155 | **140 confirmed Missing**; 11 partly designed or named; 4 refused or deferred; none covered | 15 of 155 changed (10%) |
| **Missing rows, WA and CG** (`verify_missing_wa_cg.md`) | 101 | **27 confirmed Missing**; 62 held by the corpus's admission rule (DM 6.3, DM 9.1, AH 6.15; ST-D18 for two); 6 excluded and 4 partial on spec text; 2 covered | 12 of 101 changed on evidence (12%); 62 more changed on the rule |
| **Covered spot-check** (`verify_covered_excluded.md`), strict standard | 100 (72 chosen for thin evidence, 28 random) | 67 confirmed; 32 only partial; 1 not there | **33%** overall: 44% of thin rows, **4% of random rows** |
| **Excluded and Deferred check** (same file) | 127 | 108 confirmed; 19 misread; 28 more mislabelled (deferrals and holds filed as refusals) | BF 21%, PO 25%, WA 23%, CG 0% |
| **Re-grade, BF and PO** (`regrade_bf_po.md`) | 199 Covered, 54 Excluded or Deferred | 187 of 199 still covered; 12 Mentioned. Of the 54: 16 refused, 12 deferred, 14 deferred with no owner, 12 misread | **6.0%** of Covered rows; 22% of Excluded rows |
| **Re-grade, WA and CG** (`regrade_covered_wa_cg.md`) | 384 Covered | 361 still covered (271 Designed, 90 Owned); 21 Mentioned; 1 Held; 1 Not there | **6.0%** (WA 9.5%, CG 2.2%) |

**What the rates mean.** The mappers' **Missing** verdicts were reliable for the product and operations inventories (90% confirmed). For the engine inventories most "Missing" rows were single-engine features that a recorded rule already holds. The **Covered** verdicts were right about 94% of the time across the full re-grades. The spot-check's 33% came from a sample weighted toward thin evidence and a strict standard; on random rows it found one error in 28. The **Excluded** label was the least reliable: about one in five was a deferral or a hold called a refusal, or a citation that did not say what the mapper thought. Errors cluster where evidence is thin: a scoping sentence, a claim by reference, a single mention.

### 2.4 Confidence limits

- **Ship-status cells are often Medium or Low.** Of the 1,117 inventory rows, 443 are graded High, 543 Medium and 131 Low. CG has 59 Low rows and BF 57 (48 BF rows were "not re-checked" against a vendor page).
- **webstatus.dev could not be read.** Its pages are script-rendered and every fetch failed, so Baseline labels come from release notes (High) or earlier published status (Medium). No row cites webstatus.dev.
- **Release cadences changed.** Chrome moved to a two-week cadence from Chrome 153 (8 September 2026; Chrome 155 reached stable about 6 October 2026), according to third-party calendars only ([piunikaweb](https://piunikaweb.com/2026/10/01/chrome-155-rollout-desktop-android-ios/), Low). The WA inventory says Firefox moved to two weeks from Firefox 154, while the PO inventory's PO-3.5 (High, from the Firefox release calendar) says about four weeks; the two disagree (Section 9). Any conversion from a version number to a date that assumed four weeks is suspect, and the inventory's "Chrome 144–153" range for Privacy Sandbox removals could not be verified.
- **Not read:** Safari 26.1 and 26.3 posts; Chrome and Firefox notes before 141; the Chrome 155 notes (404 on 24 October 2026); Chrome's policy list (script-rendered); the Firefox help centre (error).
- **The inventories are coarse.** One row per module or feature group hides split sub-features (Text 4, Canvas 2D, Web Audio, single WebAuthn extensions). A Covered module can still miss a property. Each inventory lists what it left out (its "Gaps in this inventory" section), for example ChromeOS and mobile enterprise management, Opera GX, Comet and Atlas, `<marquee>`, overlay scrollbars and tap highlight.
- **The test is relative to these inventories.** A feature none of the four researchers listed is not tested. The four lists overlap at their edges (Document PiP, EyeDropper, Shape Detection, HTML Media Capture appear twice), which suggests reasonable reach, not completeness.

## 3. Results

### 3.1 Totals

| Inventory | Rows | Designed | Owned | Held | Refused | Deferred | Deferred, no owner | Mentioned | Partial | Missing | Covered | Gaps |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Web platform APIs (WA) | 333 | 121 | 63 | 61 | 12 | 7 | 1 | 18 | 28 | 22 | 184 (55%) | 69 (21%) |
| CSS, graphics and media (CG) | 277 | 151 | 30 | 40 | 14 | 1 | 1 | 4 | 28 | 8 | 181 (65%) | 41 (15%) |
| Browser product features (BF) | 270 | 89 | 2 | 0 | 12 | 7 | 12 | 10 | 48 | 90 | 91 (34%) | 160 (59%) |
| OS integration and operations (PO) | 237 | 87 | 13 | 0 | 4 | 5 | 6 | 9 | 59 | 54 | 100 (42%) | 128 (54%) |
| **All** | **1117** | **448** | **108** | **101** | **42** | **20** | **20** | **41** | **163** | **174** | **556 (50%)** | **398 (36%)** |

**Held** rows: 77 of the 101 rest on the admission rule alone, with no per-feature record. **Refused** rows: 6 are implicit (a general rule covers them without naming them). Both kinds are answers today, but each should become a one-line record (FI-37, FI-38).

| Tag | Rows | Covered | Held | Refused | Deferred | Deferred, no owner | Mentioned | Partial | Missing | Gaps |
|---|---|---|---|---|---|---|---|---|---|---|
| Ship blockers [SB] (PO) | 55 | 29 | 0 | 0 | 2 | 2 | 2 | 13 | 7 | 24 |
| Table stakes [TS] (BF) | 82 | 55 | 0 | 0 | 3 | 3 | 4 | 10 | 7 | 24 |

Of the 55 ship-blocker rows, 29 are covered and 24 are gaps. Seven ship blockers are Missing outright: uninstall (PO-1.13), SmartScreen reputation (PO-2.2), the four policy-delivery rows (PO-6.1, 6.2, 6.3, 6.5) and the Cyber Resilience Act (PO-15.9).

### 3.2 What is well covered

- **The engine.** Video and audio formats (CG-13, no gaps), HTML elements and metadata (WA-2, none), the event loop, workers and messaging (WA-7, none), SVG and MathML (CG-10, none), transforms, animation and effects (CG-7, one), interactive UI and dialogs at the DOM level (WA-4, one) and layout modes (CG-4, two of 33). Each has a dedicated design section, WPT floors and a milestone.
- **Components and data pushed outside the binary** (PO-4: 15 of 17 covered): the TUF component channel, the root program, CT, revocation, the PSL, Izcal's own lists and fonts.
- **Privacy APIs** (WA-14): no gaps, because partitioning is designed (NW 6.8, 6.9) and the retired Privacy Sandbox APIs are held by the rule. They still need a recorded refusal (FI-12).
- **Accessibility** (BF-14: 8 of 9 covered), **security UI basics** (downloads, security indicator, site panel), **content blocking**, **PDF**, **WebDriver and CDP**, **code signing** and **platform sandboxes**.

### 3.3 Where the gaps are

| Inventory group | Gaps / rows |
|---|---|
| PO 12. Web apps installed into the OS | 7 / 7 |
| BF 20. Enterprise, family and special modes | 6 / 6 |
| BF 4. Profiles, accounts, sync and cross-device | 10 / 11 |
| PO 6. Enterprise policy and management | 18 / 20 |
| PO 1. Packaging and installation | 15 / 17 |
| BF 16. Built-in apps and services | 12 / 14 |
| BF 21. Mobile-specific product features | 5 / 6 |
| PO 8. Default browser and OS shell integration | 11 / 14 |
| BF 15. AI features and their off switches | 14 / 18 |
| BF 10. Content, reading and page tools | 14 / 18 |
| PO 15. Legal, regulatory and compliance | 10 / 13 |
| PO 14. Localisation and the browser's own accessibility | 6 / 8 |
| PO 3. Updates and release channels | 11 / 15 |

Three patterns explain most of them:

1. **Whole layers have no spec.** Distribution and updates, enterprise policy delivery, compliance, OS integration, installed web apps, identity and payments, privacy, WebRTC's API, speech, localisation and the mobile shell are each consumed by several specs and owned by none. Several specs point at a spec or design that does not exist: SEC 6.15 at "a privacy spec", MD 6.20 at "a separate spec" for speech, EX at "the OS-integration design", DV at "the credential subsystem".
2. **The shell names features without designing them.** Pinned tabs, mute, save page, clear browsing data, view source, the certificate viewer and the default search engine are named, sometimes with a state field, but their behaviour is not specified.
3. **Script-facing APIs with an owner crate but no floor or milestone.** Streams, server-sent events, Compression Streams and four timing APIs are listed in DM 6.18 under `hg-webapi` with no design, no WPT floor and no milestone.

## 4. The gaps, as work items

Each work item is a **new spec**, an **amendment to a named spec**, or a **decision needed from the owner**. Items are ordered by urgency: legal duties and ship blockers first, then inputs that could change Stage 1 design or schemas, then table stakes, then Stage 2 and records. Row lists are complete; the ledger gives each row's status and the evidence.

| ID | Work item | Type | Rows | Of which gaps | Urgency | Suggested stage |
|---|---|---|---|---|---|---|
| FI-1 | Compliance and security-response duties | New spec | 14 | 14 | Legal duty | Spec now; in place before the first EU distribution |
| FI-2 | Distribution, installers, channels and updates | New spec | 27 | 27 | Ship blocker | Spec before M5; complete before the first public binary |
| FI-3 | Enterprise policy delivery and management | New spec | 29 | 29 | Ship blocker (enterprise) | Spec before M5; delivery in Stage 2 |
| FI-4 | OS integration (default browser, handlers, share, taskbar, lifecycle) | New spec | 22 | 22 | Ship blocker | Spec before M5; lands M6–M8 |
| FI-5 | Script dialogs, form pickers and the file chooser | Amendment (shell 6.4, LY-D11, SHUI) | 9 | 9 | Ship blocker; Stage 1 web compatibility | Before SH-M2 (M3) |
| FI-6 | Localisation of the UI | Amendment (shell 4.4, 6.17) | 5 | 5 | Ship blocker | Stage decided now; RTL rules before SH-M1 (M2) |
| FI-7 | Automation, headless and testing builds | Amendment (DevTools) | 6 | 6 | Ship blocker (headless) | Headless before M2's WPT runs; the rest with DT's milestones |
| FI-8 | Print UI and the OS print path: name one owner | Amendment (shell 6.19 or PD 9) | 1 | 1 | Ship blocker (Stage 2) | Owner named now; UI in Stage 2 |
| FI-9 | Installed web apps: the full PWA platform (FI-D8) | New spec | 23 | 13 | Process-model input | Spec before Stage 2; build in Stage 2 |
| FI-10 | Profiles, guest mode and shared devices | Amendment (shell; SG) | 6 | 6 | Process-model input | Decision before M5 |
| FI-11 | Crash, telemetry, feedback and experiment operations | Amendment (S1 7.8) plus a short new spec | 9 | 9 | M1-exit input | Record format at M1's exit; server before the first public binary |
| FI-12 | Privacy spec (the one the security scope points to) | New spec | 16 | 5 | Stage 1 design input | Before DM-M2 fixes the environment record (S1 M5) |
| FI-13 | Identity, credentials and payments | New spec | 19 | 13 | Stage 1 commitment | WebAuthn API in Stage 1 (by M7); the rest Stage 2 |
| FI-14 | DOM web-API table: owners, WPT floors and milestones | Amendment (DM 6.18, DM 8, DM 9) | 17 | 17 | Stage 1 web compatibility | DM-M2–DM-M3 (S1 M5–M7) |
| FI-15 | Web security policy: CSP 3, Permissions Policy, X-Frame-Options | Amendment (security scope; NW 6.7) | 4 | 3 | Stage 1 web compatibility | Before M5 (NW-M2, NW-M3) |
| FI-16 | Networking: headers, loading checks and enterprise network edges | Amendment (NW) | 12 | 7 | Stage 1 web compatibility | NW-M2–NW-M4 |
| FI-17 | Tabs, sessions, library and the side panel | Amendment (shell 6.7, 6.8; panel design for EX-D15) | 24 | 24 | Schema input | Before SH-M3 (M4) |
| FI-18 | Address bar and search settings | Amendment (shell 6.8, 6.9) | 6 | 6 | Table stakes | SH-M4 (M5) |
| FI-19 | Privacy, permission and security surfaces | Amendment (shell 6.4, 6.6, 6.9; CB, PH, NW, MD) | 14 | 14 | Table stakes | SH-M2–SH-M4 (M3–M5) |
| FI-20 | Page tools and media surfaces | Amendment (shell menus; MD 6.12) | 10 | 10 | Table stakes | SH-M4–SH-M6 (M5–M7) |
| FI-21 | Performance controls and customisation | Amendment (FP; shell 6.2, 6.9, 6.16) | 11 | 11 | Table stakes; Stage 2 | SH-M4–SH-M6; mods in Stage 2 |
| FI-22 | Help, support and what's new | Amendment (shell 6.11, 6.14) | 3 | 3 | Table stakes | SH-M5 (M6) |
| FI-23 | CSS completeness and the Compat Standard | Amendment (ST 4.2, 7, ST-3) | 14 | 14 | Stage 1 web compatibility | ST-M1–ST-M2, LY-M1 (M2–M4) |
| FI-24 | Paint and GPU API coverage | Amendment (GR 4.2, 7; LY) | 7 | 7 | Stage 1 web compatibility | GR-M1–GR-M2, LY-M1 (M4–M7) |
| FI-25 | Media API coverage | Amendment (MD 6.20, MD-D1) | 4 | 3 | Stage 1 web compatibility | MD-M2 (M7) |
| FI-26 | JS and Wasm proposal-stage rule | Amendment (JS 6.14, JS 9.1) | 10 | 10 | Stage 1 web compatibility | JS-M2 (M3) |
| FI-27 | WebRTC API layer and real-time media | New spec | 12 | 8 | Table stakes (calls) | MD-M2 (M7) |
| FI-28 | Sensors, screen and input APIs | New spec | 24 | 11 | Stage 1 web compatibility | Desktop APIs Stage 1; mobile sensors Stage 2 |
| FI-29 | Speech: synthesis (OS voices) and recognition | New spec (MD 6.20 names it) | 7 | 6 | Accessibility; Stage 2 | OS-voice synthesis Stage 1, outside the AI switch (FI-D5); recognition Stage 2 |
| FI-30 | Reading tools: reader view, read aloud, translation | New spec | 5 | 5 | Table stakes (Stage 2) | Stage 2 |
| FI-31 | AI features with the switch on, incl. a local voice-generation model (FI-D4, FI-D5) | New spec (under ML, HGBA-3 §7, B10) | 21 | 20 | Stage 2–3 | Before ML-M5 (the Stage 2 gate) |
| FI-32 | Casting and remote playback: open protocols first (FI-D14) | New spec (small) | 4 | 2 | Stage 2 | Stage 2 at the earliest |
| FI-33 | Mobile shell (Android, Stage 2) | New spec (Stage 2) | 17 | 16 | Stage 2 | Start of Stage 2 |
| FI-34 | Embedding API and shared runtime | Decision, then a Stage 2 spec | 2 | 2 | Stage 2 | Stage 2 |
| FI-35 | Sync and accounts, end-to-end encrypted (FI-D2) | New spec | 12 | 11 | Stage 2 | Spec before SH-M3 fixes the schema (M4); build in Stage 2 |
| FI-36 | XR: a recorded non-goal for Stages 1 and 2 (FI-D3) | Amendment (record the non-goal in H2 §8) | 3 | 2 | Record | Now, one amendment |
| FI-37 | Admission table for single-engine features, and Izcal's own origin trials (FI-D7, FI-D16) | Amendments (DM 6.3, ST 9, LY 9) | 24 | 3 | Record | With FI-14 (DM-M2) |
| FI-38 | Product non-goals record | Amendment (shell 4.3 or UI 8) | 28 | 24 | Record | Now, one amendment |
| | **All** | | **481** | **398** | | |

### 4.1 Legal duties and ship blockers

#### FI-1 Compliance and security-response duties

**Type:** new spec, `reports/Izcal compliance.md`. **Closes 14 rows** (7 Missing, 6 Partial, 1 Mentioned): PO-3.15, 14.6, 15.1, 15.3, 15.4, 15.5, 15.6, 15.7, 15.9, 15.10, 15.11, 16.3, 16.6, 16.10. Ship blockers: PO-14.6, 15.1, 15.9.

**Why.** The EU Cyber Resilience Act lists "standalone and embedded browsers" as Class I important products ([Annex III, item 2](https://www.springlex.eu/en/packages/cra/cra-regulation/annex-3/)). Its reporting duty, Art. 14, **has applied since 11 September 2026**, including to products placed on the market before full application on 11 December 2027 ([Arts. 14, 69(3), 71](https://eur-lex.europa.eu/legal-content/EN/TXT/HTML/?uri=CELEX:32024R2847)): an early warning within 24 hours of becoming aware of an actively exploited vulnerability, a notification within 72 hours and a final report within 14 days of a fix, sent to the coordinating CSIRT and ENISA through the single reporting platform. The support period must be **at least five years** (Art. 13(8)), and Class I conformity is self-assessment only if a harmonised standard is applied in full; otherwise a notified body is involved (Art. 32(2)). Izcal has no public binary, so the duty starts with its first EU distribution; whether a Nightly counts is a legal question (FI-D11). S1 13.2's "Security response" and "Privacy and legal" rows cover a disclosure policy, advisories, GDPR handling and export control, but no spec owns the CRA runbook, the support period, conformity, CVE assignment (PO-16.3), a severity scale (PO-16.6), terms of use (PO-15.1), access and deletion for server-held data (PO-15.3, 15.4), trademark rules (PO-15.11) or a published accessibility conformance report (PO-14.6, a ship blocker for public-sector buyers). The **DMA does not oblige Izcal to show a choice screen**: only designated gatekeeper browsers (Chrome, Safari) must ([Art. 6(3)](https://eur-lex.europa.eu/legal-content/EN/TXT/HTML/?uri=CELEX:32022R1925); [gatekeepers](https://digital-markets-act.ec.europa.eu/gatekeepers_en)). PO-15.5 is therefore eligibility to appear on Apple's and Google's screens, which needs an iOS build and about 5,000 EU iPhone users a year ([Apple](https://developer.apple.com/support/browser-choice-screen/)), so it waits on the iOS refusal in S1 13.2. The **European Accessibility Act** never names browsers; Izcal's exposure is indirect ([Directive (EU) 2019/882](https://eur-lex.europa.eu/legal-content/EN/TXT/HTML/?uri=CELEX:32019L0882)). Age assurance (PO-15.7) is moving fast and needs its own legal read. PO-3.15 (end-of-support handling) joins here because the CRA support period sets it.

**Stage.** Spec now. The reporting runbook and support-period statement must exist before the first EU distribution; S1 13.2 already puts the privacy policy before M7.

**Depends on.** D-S8 (funding decides whether Izcal is a "manufacturer" or an open-source steward under Art. 24; FI-D11); FI-2 (support period and channels); FI-11 (data handling for crash reports and telemetry); GR-D21 (supported OSes).

#### FI-2 Distribution, installers, channels and updates

**Type:** new spec, `reports/Izcal distribution.md`. **Closes 27 rows** (10 Missing, 12 Partial, 5 Mentioned): BF-18.7, PO-1.1, 1.3, 1.4, 1.5, 1.6, 1.8, 1.9, 1.10, 1.11, 1.12, 1.13, 2.2, 2.6, 3.1, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.13, 3.14, 4.9, 11.2, 11.5, 16.9. Ship blockers: PO-1.1, 1.4, 1.6, 1.13, 2.2, 2.6, 3.1, 3.4.

**Why.** S1 7.8 and SEC 6.16 specify the trust chain (TUF, signing, attestations, SBOM), and S1 13.2 lists the package formats, but nothing specifies the path from "a signed binary exists" to "it is on a machine and stays current". Missing: the Windows installer forms and repair (PO-1.1), the macOS `.pkg` for MDM (PO-1.4), signed APT and YUM repositories (PO-1.6, 2.6), **uninstall and leftover cleanup** (PO-1.13), **SmartScreen reputation**, which needs lead time for a new signing identity before launch day (PO-2.2), updating while the browser is closed (PO-3.1), what a channel is and how users move between channels (PO-3.4, 1.10, BF-18.7), cadence (PO-3.5), staged rollout and halt (PO-3.7, 16.9), delta updates (PO-3.3), version pinning and extended support (PO-3.8, 3.6), release notes (PO-3.14), the ARM64 targets (PO-11.2, 11.5; the coding standard's target list has no `aarch64-unknown-linux-gnu`) and the extension blocklist's format (PO-4.9). It must also settle a contradiction: S1 13.2 ships `.deb` and `.rpm`, which a system package manager normally updates, while SEC-D14 and UPD-R2 assume Izcal's per-user TUF updater writes the install directory, and SEC 6.5 leaves it Open (FI-D10).

**Stage.** Spec before M5, when the Windows process launch and sandbox land; complete before the first public binary.

**Depends on.** FI-1 (support period); FI-3 (admin update policies, per-machine installs); SEC-D14; FI-D10.

#### FI-3 Enterprise policy delivery and management

**Type:** new spec, `reports/Izcal enterprise policy.md`. **Closes 29 rows** (19 Missing, 8 Partial, 1 Mentioned, 1 deferred with no owner): BF-20.1, 20.2, PO-1.2, 1.16, 1.17, 3.9, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 6.10, 6.11, 6.12, 6.13, 6.14, 6.15, 6.16, 6.17, 6.19, 7.3, 7.9, 8.5, 12.5, 13.11. Ship blockers: PO-1.2, 6.1, 6.2, 6.3, 6.5.

**Why.** About fifteen specs consume "enterprise policy": SH 6.9's "can be set by policy" rows and "Set by your organisation" display, NW 6.14's policy list, DV 6.10's keys, AH 6.7's and DV's "signed policy", B10's switches, the `hg-policy` crate (S1 2.1), and S1 §1 gives the broker "profile and policy". **No spec says how a policy reaches the broker**: Windows Group Policy templates (ADMX/ADML), MDM and OMA-URI, macOS configuration profiles, Linux policy files, a cross-platform file, precedence between them, one schema that generates every template, and the policy page (whose name already conflicts, Section 5). SEC-D14 refuses per-machine installs for Stage 1 and names "Enterprise deployment (Stage 2)" as its reversal trigger, so the Stage 2 MSI (PO-1.2) has a trigger but no owner. The item also decides kiosk mode, managed bookmarks, enterprise reporting, DLP connectors, IE-mode-style legacy switching, cloud management, TLS version policy, OS single sign-on to cloud identity (PO-7.3), default-browser assignment, force-installed web apps and a DevTools switch. Several of these may be refusals; each needs a line.

**Stage.** Spec before M5, because policy is a broker input that recordings must capture (Section 8). Delivery in Stage 2 with per-machine installs.

**Depends on.** FI-2 (MSI and `.pkg`); FI-9 (force-installed apps); FI-4; SEC-D14.

#### FI-4 OS integration

**Type:** new spec, `reports/Izcal OS integration.md`. **Closes 22 rows** (9 Missing, 13 Partial): WA-12.6, BF-10.9, 12.7, 16.14, 17.6, PO-8.1, 8.2, 8.3, 8.6, 8.7, 8.9, 8.11, 8.12, 8.13, 9.8, 9.9, 10.10, 10.16, 10.17, 10.18, 13.9, 13.10. Ship blockers: PO-8.1, 8.2, 8.3, 10.10.

**Why.** EX defers `protocol_handlers` and file-handler keys "to Stage 3, with the OS-integration design"; that design does not exist. Every row here is three OS integrations, all broker work: default-browser registration on Windows, macOS and Linux (PO-8.1 to 8.3, ship blockers), protocol and file associations (PO-8.6), jump lists, dock menus, badges and progress (PO-8.7), the download scan through AMSI or IAttachmentExecute (PO-8.9), background mode and start at login (PO-8.11, BF-12.7), OS scripting hooks (PO-8.12), the OS share sheet and the Web Share API, which meets the two-engine bar (PO-9.9, BF-17.6, BF-10.9, WA-12.6), system media-control adapters (PO-9.8), per-monitor DPI and mixed-DPI moves (PO-10.10, a ship blocker), window-manager integration, touch and pen gestures and multi-monitor placement (PO-10.16 to 10.18), supported command-line switches (PO-13.9) and **single instance**: a second launch handing its URL to the running browser (PO-13.10).

**Stage.** Spec before M5, which builds process launch; the integrations land M6–M8.

**Depends on.** FI-9 (app shortcuts and badges share the plumbing); FI-2 (install-time registration); FI-3 (default-browser policy).

#### FI-5 Script dialogs, form pickers and the file chooser

**Type:** amendment to `reports/Izcal shell.md` §6.4 (prompt classes) and `reports/HGBA shell UI design.md`, with LY-D11 (`reports/Izcal layout.md`). **Closes 9 rows** (4 Missing, 2 Partial, 3 Mentioned): WA-1.16, 3.6, 3.8, 3.9, 3.11, 9.9, CG-15.13, BF-5.11, PO-9.16. Ship blockers: PO-9.16.

**Why.** Every engine ships `alert`, `confirm`, `prompt`, `beforeunload` prompts, the select, date and colour pickers and the file chooser. DM 6.7 designs the pause and hands drawing to the broker, but shell 6.4's prompt-class table has no class for script dialogs, and LY-D11 says pickers "are browser-process UI" with no owning design. The file chooser needs the OS dialog or the Linux portal (PO-9.16, a ship blocker), directory upload (WA-3.8, three engines), the `capture` attribute (WA-3.9, CG-15.13) and the File System Access pickers, whose permission design S1 13.2 promises "before M6" (WA-9.9).

**Stage.** Design with SH-M2 (the prompt layer, S1 M3), so the surfaces exist before DM-M2 (S1 M5), whose WPT floors exercise these dialogs.

**Depends on.** SH-M2; FI-4 (portal and native dialogs).

#### FI-6 Localisation of the UI

**Type:** amendment to `reports/Izcal shell.md` §4.3 (assumptions kept and broken) and §6.17 ("Strings"). **Closes 5 rows** (1 Missing, 3 Partial, 1 Mentioned): PO-14.1, 14.2, 14.3, 14.4, 14.5. Ship blockers: PO-14.1.

**Why.** Strings are Fluent-externalized so that translation stays possible (SH 4.3 keeps English-only for Stage 1 on that condition), but no stage, locale count, right-to-left mirroring rule, translator pipeline, string freeze or regional default is set. Public-sector and most non-English buyers cannot use an English-only browser, so PO-14.1 is a ship blocker. Mirroring is the costly part to retrofit.

**Stage.** Decide the stage and locale set now; write the right-to-left rules before SH-M1 (S1 M2) fixes the shell document's layout; translations in Stage 2.

**Depends on.** TX (bidi and fonts); FI-1 (accessibility conformance report).

#### FI-7 Automation, headless and testing builds

**Type:** amendment to `reports/Izcal DevTools spec.md`. **Closes 6 rows** (4 Missing, 2 Partial): WA-22.3, 22.10, 22.12, BF-18.3, PO-13.7, 13.8. Ship blockers: PO-13.7.

**Why.** WebDriver classic, BiDi and CDP are well covered (DT 6.3, DT-5). The edges are not: a **headless mode** (PO-13.7, a ship blocker for CI and automation users), pinned non-updating testing builds (PO-13.8), the WebDriver extension commands that WPT's testdriver uses (permissions, virtual authenticators, sensors; WA-22.3), `TestUtils.gc()` (WA-22.10), whether Izcal runs origin trials at all (WA-22.12; FI-D16) and DevTools device mode, which web developers treat as table stakes (BF-18.3).

**Stage.** Headless before M2 starts running WPT through the WebDriver server; the rest with DT's milestones.

**Depends on.** DT-5; FI-13 (virtual authenticator); FI-28 (sensor commands).

#### FI-8 Print UI and the OS print path: one owner

**Type:** amendment to `reports/Izcal PDF.md` §9 or `reports/Izcal shell.md` §6.19. **Closes 1 row**: PO-9.14 (ship blocker).

**Why.** Paged layout, `@page`, `window.print()` and the preview raster are Stage 1 designs (LY 6.7, GR 6.27), and the PDF writer is designed (PD 6.19). The print UI and the OS print path are Stage 2 (S1 13.2), but SH 6.19 lists the print UI under PD while PD-M4 treats it as a dependency, so each points at the other. Printing is a ship blocker in the PO inventory.

**Stage.** Name the owner now (recommendation: PD, which already owns `hg-pdf-print`); design in Stage 2.

**Depends on.** FI-4 (OS print dialogs).

### 4.2 Inputs that could change Stage 1 design

#### FI-9 Installed web apps

**Owner decision (FI-D8, 24 October 2026):** build the full PWA platform, not the minimal set; single-engine app APIs still pass the admission rule (FI-37). Apps run in ordinary (site, container) keys (FI-D17).

**Type:** new spec, `reports/Izcal installed web apps.md` (or an amendment to `reports/Izcal app host.md`). **Closes 23 rows** (8 Missing, 4 Partial, 1 deferred with no owner, 10 held by the rule): WA-10.6, 10.8, 10.9, 10.11, 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8, 11.11, 11.12, BF-17.1, 17.2, 17.3, PO-12.1, 12.2, 12.3, 12.4, 12.6, 12.7.

**Why.** Two accepted specs assume installed sites exist: PH 6.4.9 checks "an installed web app … in a window with no address bar", and AH contrasts its app windows with "an installed site's standalone window". No spec owns the Web App Manifest (WA-11.1, multi-engine), install and uninstall through the OS (PO-12.1 to 12.3), standalone windows, shortcuts, badging (WA-11.6, two engines), protocol handlers (WA-11.4, two engines) or management (BF-17.2). The single-engine APIs (install prompts, launch handler, window controls overlay, window management, related apps, origin migration, share target, periodic background sync, content index) are held by the rule and need only a record. **Whether to build installed web apps at all is an owner decision** (FI-D8), and so is one process-model question: whether an installed app runs in the profile's ordinary (site, container) keys or gets a container of its own (Section 8).

**Stage.** Decision now; spec before Stage 2; the keying decision before M5.

**Depends on.** FI-4; FI-3 (force-install); FI-10 (profiles); push and background sync (S1 13.2, Stage 2).

#### FI-10 Profiles, guest mode and shared devices

**Type:** amendment to `reports/Izcal shell.md`, with `reports/Izcal storage.md`. **Closes 6 rows** (2 Missing, 4 Partial): BF-4.1, 4.2, 4.8, 7.2, 20.6, PO-1.15.

**Why.** "Profiles later" (SH 4.1) is referenced by ML's, EX's and B10's per-profile switches, and the process map counts some services per profile (the device arbiter), yet no spec says what a profile is next to a container, how the user creates, switches or picks one (BF-4.1, PO-1.15, table stakes), whether guest mode exists (BF-4.2), or how a shared device works (BF-20.6). Whether several profiles share one broker process, or each runs its own, changes the process map (Section 8).

**Stage.** The process decision before M5; the surfaces with SH-M4.

**Depends on.** SG (per-profile store); FI-3 (managed work profile, PO-6.16).

#### FI-11 Crash, telemetry, feedback and experiment operations

**Type:** amendment to `reports/HGBA Stage 1 architecture design.md` 7.8, plus a short new spec for the back end. **Closes 9 rows** (2 Missing, 7 Partial): WA-17.17, PO-2.8, 5.3, 5.4, 5.7, 5.8, 5.9, 5.12, 17.8.

**Why.** D-S4 decides consent and the crash-handler process is in the process map, but there is no ingestion server, symbol store, grouping or regression flag (PO-5.3, 17.8), no Windows Error Reporting or OS hang integration (PO-5.4), no metric set (PO-5.7), no deletion path or regional defaults (PO-5.8), no feedback form (PO-5.12), no crash-driven blocklist of injected modules (PO-2.8) and no decision on server-side experiments (PO-5.9; FI-D12). Crash reports and production recordings should share one header (build ID, data versions, process kind, policy digest), which is cheapest to fix at M1's exit (Section 8).

**Stage.** Record header at M1's exit; server and pipeline before the first public binary.

**Depends on.** FI-1 (GDPR duties); FI-2 (symbols per build and channel).

**Status (29 October 2026): known gap, no spec.** M1 built the record header this item asked for at M1's exit, `RecordingHeader` (magic `IZRH`: build ID, data versions, decision-point log, OS version, process kind, sandbox tier, policy digest or flags, profile identity), placed in `hg-types` so recordings and crash reports share it ([`hg-types`' `recording` module](../crates/hg-types/src/recording.rs)). Because FI-11 has no spec, the header's own sizes not fixed by another spec are the crate's and marked †, and the amendment to Stage 1 7.8 and the short back-end spec are still unwritten. The header's layout becomes that spec's input, not its decision.

#### FI-12 Privacy spec

**Type:** new spec, `reports/Izcal privacy.md`, the one SEC 6.15 and 11.4 already point to. **Closes 16 rows** (1 Missing, 4 Partial, 11 held by the rule): WA-1.8, 1.9, 8.19, 8.20, 12.12, 14.5, 14.6, 14.7, 14.8, 14.9, 14.10, 14.11, 14.12, 14.16, 16.5, BF-7.6.

**Why.** SEC 6.15 sends "CHIPS and FedCM versus the retired Privacy Sandbox APIs" to "a privacy-spec question, out of scope here", and no privacy spec exists. On 17 October 2025 Google announced the retirement of Attribution Reporting, IP Protection, Private Aggregation, Protected Audience, Related Website Sets, selectURL, Topics and others, and said it keeps CHIPS, FedCM and Private State Tokens ([Privacy Sandbox update](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies)). The spec should record those refusals, decide the anti-fraud tokens (WA-14.11, 14.12) and Private Click Measurement (WA-14.10), and fix which fingerprintable signals the DM 6.16 environment record exposes: client hints and `userAgentData` (WA-8.19), Network Information and Save-Data (WA-8.20), `deviceMemory` (WA-16.5, exposed today without an admission record), plus credentialless iframes and fenced frames (WA-1.8, 1.9), the permission-prompt intervention (WA-12.12) and a user-facing third-party cookie setting (BF-7.6). Apple's EU engine rules require cross-site cookies blocked by default ([Apple](https://developer.apple.com/support/alternative-browser-engines/)); whether partitioning satisfies "block" is worth stating.

**Stage.** Before DM-M2 fixes the environment record (S1 M5).

**Depends on.** D-S1 and OD-6; CB; FI-13 (FedCM); FI-28 (sensor surfaces).

#### FI-13 Identity, credentials and payments

**Type:** new spec, `reports/Izcal identity and payments.md`; SH 6.20's reversal column already names "a dedicated password-manager spec" as the way out. **Closes 19 rows** (5 Missing, 5 Partial, 3 Mentioned, 6 held by the rule): WA-15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7, 15.8, 15.9, 15.10, 15.11, BF-4.10, 6.2, 6.3, 6.7, 6.10, 6.11, PO-7.17, 9.11.

**Why.** S1 §6 commits Stage 1 to "WebAuthn through platform authenticators" (A-S1-51), but no spec designs the API or the per-OS authenticator bridge (WA-15.2: conditional mediation, hybrid transport, PRF, related origins). DV says FIDO and CTAP "remain in the credential subsystem (WebAuthn)", which no spec defines, and Linux passkeys need CTAP2 over USB, HID and BLE (PO-9.11). The `navigator.credentials` container (WA-15.1), Digital Credentials (WA-15.4, two engines) and Payment Request (WA-15.8, two engines) share one browser-mediated chooser. FedCM (WA-15.3) is single-engine but kept by Google, so it needs a recorded decision with the privacy spec. Product rows: the password generator and breach and weak-password checks (BF-6.2, 6.3, table stakes), third-party password managers (BF-6.7), the wallet sheet (BF-6.11), virtual cards (BF-6.10, likely a refusal) and enterprise password-reuse alerts (PO-7.17). The Stage 2 address and payment autofill (BF-6.8, 6.9, deferred with an owner) should be designed here too.

**Stage.** WebAuthn API and bridge in Stage 1 (by M7); the rest Stage 2.

**Depends on.** DV (transports); FI-12; SH 6.12.

#### FI-14 DOM web-API table: owners, WPT floors and milestones

**Type:** amendment to `reports/Izcal DOM, bindings and event loop.md` §6.18, §8 and §9. **Closes 17 rows** (3 Missing, 4 Partial, 10 Mentioned): WA-1.11, 6.15, 6.25, 6.26, 8.6, 8.7, 8.12, 8.13, 8.16, 9.11, 17.2, 17.3, 17.5, 17.7, 17.8, 22.7, CG-7.18.

**Why.** DM 6.18 says "Any web API not in this table or another spec's is recorded in the ledger as a gap before it is admitted". That ledger is the Phase 3 gap ledger (G-1 to G-26), so these rows should be filed there as G-27 onward, or the owner should record that this audit's ledger replaces it. Streams, server-sent events, Compression Streams, Performance Timeline, Navigation, User and Paint Timing have an owner crate (`hg-webapi`) but no design, floor or milestone. LCP (Baseline since December 2025), Beacon, URLPattern (which DM 6.10's static service-worker routes need), the Entries API, an XPath evaluator, Geometry interfaces and Visual Viewport have no owner. The Origin API depends on an HTML pin date no report records, and Infra is named nowhere. Adding the WPT directories `streams/`, `eventsource/`, `compression/`, `performance-timeline/`, `navigation-timing/`, `user-timing/`, `paint-timing/`, `css/geometry/` and `visual-viewport/` to DM-1, with milestones, makes most of them Owned at little cost.

**Stage.** DM-M2 to DM-M3 (S1 M5–M7).

**Depends on.** NW (fetch side of SSE and Beacon); recording the HTML and DOM pin dates (Section 9).

#### FI-15 Web security policy: CSP 3, Permissions Policy, X-Frame-Options

**Type:** amendment to `reports/Izcal security scope.md`, with NW 6.7. **Closes 4 rows** (2 Partial, 1 Mentioned, 1 held by the rule): WA-12.2, 12.3, 13.2, 13.10.

**Why.** NW pins only "CSP (fetch directives)". CSP's document-side directives (inline script and style, `frame-ancestors`, `sandbox`, reporting), the `Permissions-Policy` header with `allow` inheritance and the feature registry, and X-Frame-Options are consumed by NW, ST, DV, MD and AX but parsed and enforced by no named owner. S1 13.2 already promises a permission design "before M6".

**Stage.** Before M5, when NW-M2 and NW-M3 enforce CSP.

**Depends on.** NW 6.7; DM 6.9 (sandbox flags).

#### FI-16 Networking: headers, loading checks and enterprise network edges

**Type:** amendment to `reports/Izcal networking.md`. **Closes 12 rows** (4 Missing, 3 Partial, 5 held by the rule): WA-5.9, 5.11, 5.12, 5.13, 8.2, 8.9, 13.5, 13.13, 17.6, PO-7.6, 7.12, 7.16.

**Why.** No-Vary-Search (two engines), Integrity-Policy (Baseline), upload streaming (an Interop 2026 area) and Server-Timing (Baseline) are Fetch-layer checks that belong in the partition service beside CORS and ORB. Web bundles, signature-based SRI, Document-Isolation-Policy, Connection Allowlists and WebSocketStream are single-engine and need records. The enterprise edges are PKCS#11 and smart cards (PO-7.6), intranet single-word navigation between the address bar and NW (PO-7.12), and per-app and Always On VPN routing (PO-7.16).

**Stage.** NW-M2 to NW-M4.

**Depends on.** FI-18 (address bar resolution); FI-3 (enterprise policy).

#### FI-17 Tabs, sessions, library and the side panel

**Type:** amendment to `reports/Izcal shell.md` §6.7 and §6.8, plus the shell panel design EX-D15 waits for. **Closes 24 rows** (12 Missing, 8 Partial, 3 Mentioned, 1 deferred with no owner): BF-1.3, 1.11, 1.12, 1.15, 1.17, 1.18, 1.19, 1.20, 1.22, 3.2, 3.4, 3.5, 3.6, 3.8, 3.10, 3.11, 3.12, 3.13, 3.14, 5.4, 10.14, 16.1, 16.2, 16.6.

**Why.** SH-D8's one-store schema freezes at SH-M3 (S1 M4). Record types that are not in it then are a migration later: reading list (BF-3.5), tags (BF-3.2), collections (BF-3.6), notes (BF-10.14), named sessions (BF-1.15), feeds (BF-16.6) and the tab-group model (Q-SH-9). Table-stakes behaviour is named but not specified: pinned tabs (BF-1.3), mute and the audible indicator (BF-1.12) and save page as a file (BF-5.4). EX-D15 stages the extension sidebar until "the shell's panel design lands", and shell 9.1 and 9.2 schedule no panel design (BF-16.1, 16.2).

**Stage.** Record types before SH-M3 (M4); surfaces in Stage 1 and Stage 2.

**Depends on.** SG; EX-D15.

### 4.3 Table stakes in the shell

#### FI-18 Address bar and search settings

**Type:** amendment to `reports/Izcal shell.md` §6.8 and §6.9. **Closes 6 rows** (3 Missing, 3 Partial): BF-2.3, 2.4, 2.5, 2.6, 2.14, PO-15.13.

**Why.** Three specs assume "the chosen search engine", and EX 6.9 refuses the search override because "Izcal's search settings are the user's", but no spec designs the setting, the default list or adding an engine (BF-2.3). The fact-check removes the legal pressure: no DMA choice screen is owed (FI-1), so BF-2.4 becomes a first-run product choice. Keywords (BF-2.5) and calculator answers (BF-2.6) are ranked providers in SH 6.8's existing autocomplete model. Search agreements (PO-15.13) tie into D-S8 funding and CB-D16's ban on revenue from listed parties.

**Stage.** SH-M4 (S1 M5).

**Depends on.** D-S8; FI-16 (intranet names).

#### FI-19 Privacy, permission and security surfaces

**Type:** amendment to `reports/Izcal shell.md` §6.4, §6.6 and §6.9, with CB, PH, NW and MD. **Closes 14 rows** (2 Missing, 11 Partial, 1 Mentioned): BF-7.3, 7.9, 7.13, 7.18, 8.2, 8.3, 8.6, 8.8, 8.9, 8.11, 9.7, 9.9, 9.10, 9.11.

**Why.** Each is a surface over facts the broker already holds: the clear-browsing-data dialog (BF-7.13, table stakes), the popup-blocked indicator (BF-9.7, table stakes), the certificate viewer behind SH's "certificate details expander" (BF-8.6, table stakes), in-use indicators with a stop control (BF-9.9), a tab picker and "sharing this tab" bar (BF-9.10), per-site content toggles (BF-9.11), the lookalike-domain warning's owner (Q-UI-5 and Q-PH-16 are open; BF-8.11), the scareware exit (BF-8.3), a safety check (BF-8.8), an enhanced-protection setting over JS-D13's jitless tier (BF-8.2), a privacy report (BF-7.9), an element picker (BF-7.18) and opening a private window on a Tor route (BF-7.3).

**Stage.** SH-M2 to SH-M4 (S1 M3–M5).

**Depends on.** CB 6.12; PH; MD 6.10 (capture).

#### FI-20 Page tools and media surfaces

**Type:** amendment to `reports/Izcal shell.md` (menus and context menus) and `reports/Izcal images and media.md` §6.12. **Closes 10 rows** (3 Missing, 7 Partial): BF-10.3, 10.10, 10.11, 10.12, 10.13, 10.18, 11.2, 11.3, 11.10, 18.2.

**Why.** Small shell features over capabilities the engine already has: text-only zoom and minimum font size over AX-D19 and ST-D17 (BF-10.3), creating a text-fragment link (BF-10.10), a QR code (BF-10.11), a user screenshot over GR 6.23's capture (BF-10.12), an eyedropper (BF-10.13), **`view-source:`**, which PH 6.2 and AGT-R3 assume and nobody owns (BF-10.18, table stakes), F12 and Inspect (BF-18.2, table stakes), automatic picture-in-picture (BF-11.2), a media hub over Media Session (BF-11.3) and a microphone level control (BF-11.10).

**Stage.** SH-M4 to SH-M6 (S1 M5–M7).

**Depends on.** FI-4 (media keys); DT (entry points).

#### FI-21 Performance controls and customisation

**Type:** amendment to `reports/Izcal resource footprint.md` and `reports/Izcal shell.md` §6.2, §6.9 and §6.16. **Closes 11 rows** (3 Missing, 6 Partial, 2 Mentioned): BF-2.15, 12.3, 12.5, 12.6, 12.8, 13.11, 13.12, 13.14, 13.15, 14.8, PO-10.3.

**Why.** FP-D15 already holds the data; what is missing is the controls: an energy mode (BF-12.3), a task-manager view with "end process" (BF-12.5), a preload setting (BF-12.6), a hardware-acceleration setting beyond the recovery window (BF-12.8, PO-10.3), toolbar layout (BF-13.11), editable shortcuts (BF-13.14), default fonts (BF-14.8), macros and mouse gestures (BF-13.15, 2.15) and deep mods within D-S2's W-1 rule (BF-13.12).

**Stage.** SH-M4 to SH-M6; mods in Stage 2.

**Depends on.** SH-D15 (settings registry); FP.

#### FI-22 Help, support and what's new

**Type:** amendment to `reports/Izcal shell.md` §6.11 and §6.14. **Closes 3 rows** (1 Missing, 2 Partial): BF-19.3, 19.7, PO-17.9.

**Why.** A what's-new page (BF-19.3) is mentioned only in SH 6.11; help content (BF-19.7) and a support site (PO-17.9) do not exist. Local help fits the sealed shell image and the no-network rule.

**Stage.** SH-M5 (S1 M6).

**Depends on.** FI-2 (release notes); FI-6 (translated help).

### 4.4 Engine coverage amendments

#### FI-23 CSS completeness and the Compat Standard

**Type:** amendment to `reports/Izcal HTML and CSS parsing and style.md` §4.2, §7 and ST-3. **Closes 14 rows** (1 Missing, 12 Partial, 1 Mentioned): WA-22.9, CG-1.2, 1.9, 2.8, 2.17, 2.19, 3.16, 4.23, 6.19, 6.20, 6.21, 8.3, 9.1, 9.2.

**Why.** Module coverage is assembled from four anchor tables (ST, LY, GR, TX), so modules fall between them: CSS Namespaces (a three-engine REC), Logical Properties, Counter Styles, Content 3, `zoom` (Interop 2025–2026), Media Queries 5's newer features, `env()`'s variables, Basic UI 3 and 4, Lists and Counters, Typed OM (Q-ST-14) and the Compat Standard's `-webkit-` aliases and `window.orientation` (WA-22.9, an Interop 2026 area). One scoping sentence closes most: **every module in CSS Snapshot 2026 plus the Compat Standard is in scope**, each with an owner and a WPT floor (`css-namespaces`, `css-logical`, `css-counter-styles`, `css-zoom`, `css-lists`, `compat`). The predefined counter-style data needs a licence check (Section 6).

**Stage.** ST-M1 and ST-M2, LY-M1 (S1 M2–M4).

**Depends on.** LY and GR anchor tables.

#### FI-24 Paint and GPU API coverage

**Type:** amendment to `reports/Izcal graphics and rendering.md` §4.2 and §7, and `reports/Izcal layout.md`. **Closes 7 rows** (7 Partial): CG-3.8, 4.28, 5.4, 5.5, 6.3, 6.4, 11.9.

**Why.** GR scopes Backgrounds 3 only, so `background-clip: text` is undesigned (CG-3.8); text decoration and skip-ink painting (CG-6.3, 6.4), overscroll effects (CG-5.4) and `scrollbar-color` (CG-5.5) have no design or floor. `shape()` is an Interop 2026 area that LY-2 already makes binding, with no design (CG-4.28). The exposed WebGL extension list per GR-D11 bucket is both a compatibility list and a fingerprint (CG-11.9).

**Stage.** GR-M1 to GR-M2, LY-M1 (S1 M4–M7).

**Depends on.** FI-12 (fingerprinting of the extension list).

#### FI-25 Media API coverage

**Type:** amendment to `reports/Izcal images and media.md` §6.20 and MD-D1. **Closes 4 rows** (2 Partial, 1 Mentioned, 1 held by the rule): CG-12.16, 14.5, 14.11, 14.19.

**Why.** Image density correction is one clause with no Exif rules or floor (CG-12.16); Media Fragments `#t=` (CG-14.5) and `getVideoPlaybackQuality()` (CG-14.11) have no row; Audio Session (CG-14.19) is held. The same amendment should give MD 6.20 one line per media API, designed or recorded as not admitted, and record the AC-3, E-AC-3 and Dolby Vision licence route (Section 6).

**Stage.** MD-M2 (S1 M7).

**Depends on.** FI-D13 (codec licences).

#### FI-26 JS and Wasm proposal-stage rule

**Type:** amendment to `reports/Izcal JavaScript engine.md` §6.14 and §9.1. **Closes 10 rows** (2 Missing, 5 Partial, 3 Mentioned): WA-18.4, 18.14, 18.17, 18.18, 18.21, 18.23, 18.24, 18.25, 19.4, 20.11.

**Why.** JS-1 covers every feature at Stage 4 at the pinned ECMA-262 commit, but the reports never record a proposal's stage except for six named ones, and give no rule for Stage 3 proposals that already ship in two engines (explicit resource management, iterator helpers' new methods). Whether Upsert and `JSON.parse` source text are in ES2026 rests on secondary sources. Proper tail calls need a decision that interacts with the debugger and JS-D19's stack segments. `Error.captureStackTrace` is a non-standard compatibility surface. Resizable Wasm buffers depend on the Wasm JS API at the pin.

**Stage.** JS-M2 (S1 M3).

**Depends on.** Recording the ECMA-262 and Wasm pins (Section 9).

### 4.5 New Stage 1 and Stage 2 platform specs

#### FI-27 WebRTC API layer and real-time media

**Type:** new spec, `reports/Izcal WebRTC.md`. **Closes 12 rows** (3 Missing, 3 Partial, 2 Mentioned, 4 held by the rule): CG-15.2, 15.3, 15.4, 15.6, 15.9, 15.10, 15.12, 15.14, 15.17, 15.18, 15.19, 15.20.

**Why.** NW 6.12 owns the transport and MD 6.10 the media plumbing (`hg-voice`, jitter buffers, MD-D30's codecs), but nobody owns the API between them: `RTCPeerConnection`, JSEP and SDP, transceivers, simulcast and SVC, DTMF and statistics (CG-15.14), congestion control and bandwidth estimation (CG-15.20), Encoded Transform and insertable streams (CG-15.17, 15.10), content hints (CG-15.12), capture extensions and `captureStream()` (CG-15.2, 15.4). NW-42's RFC 7742 interoperability target depends on this layer. Video calls are something users expect to work in a browser.

**Stage.** MD-M2 (S1 M7).

**Depends on.** NW 6.12; MD 6.10; the H.264-on-Linux licence decision (MD-D3).

#### FI-28 Sensors, screen and input APIs

**Type:** new spec, `reports/Izcal sensors and input.md`. DV 3.5 says the devices spec "owns only the six device APIs", so nothing else owns these. **Closes 24 rows** (5 Missing, 6 Partial, 13 held by the rule): WA-4.15, 4.16, 12.7, 12.8, 16.1, 16.2, 16.3, 16.7, 16.8, 16.9, 16.10, 16.11, 16.12, 16.19, 16.20, 16.21, 16.23, 16.24, CG-9.10, 11.13, PO-9.20, 9.23, 9.24, 17.6.

**Why.** Geolocation's location provider (WA-16.1, PO-9.20, PO-17.6), device orientation and motion and Screen Orientation (three engines each), Screen Wake Lock (Baseline), Gamepad (Baseline widely available), Pointer Lock (three engines) and Vibration all need OS backends, permission grants, recording and fingerprinting rules, and none has them. Keyboard Lock, the Generic Sensor family, Idle Detection, Device Posture, Viewport Segments, Shape Detection, Contact Picker, EyeDropper and the other single-engine APIs need records. A network location provider is a service Izcal would have to run or license (Section 6).

**Stage.** Desktop APIs in Stage 1; mobile sensors with the Android shell in Stage 2.

**Depends on.** DV (backends); FI-12; FI-7 (sensor test commands).

#### FI-29 Speech: synthesis and recognition

**Owner decision (FI-D5, 24 October 2026):** read aloud through OS voices sits outside the AI switch; recognition stays under the ML grant; a generative voice model belongs to FI-31.

**Type:** new spec, `reports/Izcal speech.md`, the "separate spec" MD 6.20 names. **Closes 7 rows** (1 Missing, 3 Partial, 1 Mentioned, 1 deferred with no owner, 1 held by the rule): WA-12.10, CG-1.21, 16.3, 16.4, BF-2.8, PO-9.22, 17.7.

**Why.** Speech synthesis ships in all three engines and recognition in Chromium and WebKit (WA-12.10, CG-16.3, 16.4). MD 6.20 hands synthesis and recognition to a spec that does not exist, and EX 9.1 names TTS for Stage 3 with no owner (PO-9.22). Synthesis needs a choice between OS voices and Izcal's own, with a fingerprinting rule for voice lists; recognition belongs under B10 with the ML service, which already runs ML-ASR for captions. Read aloud (FI-30) and voice search (BF-2.8) depend on it.

**Stage.** Synthesis in Stage 1 (it is also an accessibility path); recognition in Stage 2 under the ML grant (FI-D5).

**Depends on.** ML; AX; FI-12.

#### FI-30 Reading tools: reader view, read aloud, translation

**Type:** new spec, `reports/Izcal reading tools.md`. **Closes 5 rows** (2 Missing, 1 Partial, 2 deferred with no owner): CG-16.11, BF-10.4, 10.5, 10.6, 10.17.

**Why.** Reader view is table stakes, named for Stage 2 by S1 §6 ("Reader rendering on top") and PD 9.1, and owned by no spec (BF-10.4, CG-16.11). Page translation is table stakes (BF-10.6): ML-D2 defers only the web Translator API, so no product decision exists, and putting translation behind `ai.enabled` may not match what users expect. Read aloud for pages (BF-10.5) exists only for PDFs, and a dark mode for web content (BF-10.17) exists only for the shell and PDF.

**Stage.** Stage 2.

**Depends on.** FI-29; FI-31; ML-D2; FI-D5.

#### FI-31 AI features with the switch on

**Owner decisions (FI-D4, FI-D5, 24 October 2026):** the focused set (agent task UI, summarise, multi-tab questions), local models only, plus a local voice-generation model and similar generative speech tools behind the AI switch. Image generation, virtual try-on and AI-built extensions and themes are refused.

**Type:** new spec, `reports/Izcal AI features.md`, under ML, H3 §7 and B10. **Closes 21 rows** (15 Missing, 3 Partial, 1 Mentioned, 1 deferred with no owner, 1 refused implicitly): BF-2.9, 2.12, 5.9, 5.10, 10.15, 13.7, 13.9, 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7, 15.9, 15.10, 15.11, 15.12, 15.13, 15.16, 15.18.

**Why.** ML-D2 schedules Stage 2 task APIs with no product surface, and H3 §7 bets on an agent runtime rather than a consumer AI browser. With the switch off there are zero surfaces (B10, SH-D16); with it on, nothing is designed. One document should list what Izcal builds (the agent task UI over B9 and AGT-R, summarise, multi-tab questions), what it refuses (image generation, virtual try-on, AI-built extensions and themes), whether cloud providers are allowed at all (BF-15.2; ML-D5's service is network-less), and how memory and personal-data access work beyond AGT-R2 and AGT-R6.

**Stage.** Before ML-M5, the Stage 2 gate.

**Depends on.** FI-D4; ML; B9; B10.

#### FI-32 Casting and remote playback

**Owner decision (FI-D14, 24 October 2026):** open protocols first, Izcal's own implementation wherever possible, a proprietary protocol only where no open route works. This is now the project's general rule for protocol choices.

**Type:** owner decision, then a small spec. **Closes 4 rows** (2 Missing, 2 held by the rule): WA-16.22, CG-14.16, 14.17, BF-11.4.

**Why.** Casting is table stakes on Chrome (BF-11.4). Remote Playback ships in two engines in part (CG-14.16) and the Presentation API in one (WA-16.22, CG-14.17). The protocols are per ecosystem and some are proprietary, so licensing comes before design (FI-D14).

**Stage.** Stage 2 at the earliest.

**Depends on.** FI-D14; MD.

#### FI-33 Mobile shell (Android)

**Type:** new spec, Stage 2. **Closes 17 rows** (13 Missing, 2 Partial, 1 deferred with no owner, 1 held by the rule): CG-2.18, 6.5, 9.9, BF-1.24, 2.11, 10.16, 13.6, 17.8, 17.10, 17.12, 20.5, 21.1, 21.2, 21.3, 21.5, 21.6, PO-11.10.

**Why.** S1 schedules the Android engine for Stage 2 (H2 S-8) and refuses iOS for Stages 1 and 2, and AX, NW and DV already carry Android rows, but no spec covers a phone shell: the tab grid, swipe gestures and pull to refresh (BF-21.1 to 21.3, table stakes), address bar position, request desktop site, widgets, autofill into other apps, camera search, data saver and the mobile-only CSS (viewport meta, `text-size-adjust`, viewport segments).

**Stage.** At the start of Stage 2 (FI-D6).

**Depends on.** H2 S-8; FI-13 (autofill service); FI-6.

#### FI-34 Embedding API and shared runtime

**Type:** owner decision, then a Stage 2 spec. **Closes 2 rows** (1 Missing, 1 deferred with no owner): PO-13.1, 13.2.

**Why.** S1 13.2 defers the embedding API to Stage 2 in one row, and no spec designs it (PO-13.1). A shared runtime that other apps use and Izcal updates, as WebView2 is, is never mentioned (PO-13.2). The choice changes the updater and the support period.

**Stage.** Stage 2.

**Depends on.** FI-2; FI-1.

### 4.6 Owner decisions and records

#### FI-35 Sync and accounts

**Type:** new spec, decided by the owner on 24 October 2026 (FI-D2: plan end-to-end-encrypted sync for Stage 2). **Closes 12 rows** (1 Mentioned, 10 deferred with no owner, 1 refused implicitly): BF-1.5, 1.23, 4.3, 4.4, 4.5, 4.6, 4.7, 4.9, 17.7, 19.8, PO-8.14, 17.3.

**Why.** Shell 9.1 says "Stage 3 or never", SH-D20's reversal trigger says that if sync is wanted it "is commissioned as its own spec", and Q-SH-5 ("is sync ever in scope?") is open. Sync and sign-in are table stakes in every major browser (BF-4.3, 4.4). Until the owner answers, these rows have neither an owner nor a refusal.

**Stage.** Spec written before SH-M3 fixes the history and bookmark schema (M4); built in Stage 2. End-to-end encrypted from its first version; Izcal runs the sync servers, so the spec covers the account system and the server, under the open-protocols-first rule (FI-D14).

**Depends on.** SH-M3's schema (stable record IDs and tombstones), FI-13 (account sign-in and passkeys), FI-1 (data-protection duties for a service Izcal runs).

#### FI-36 XR

**Owner decision (FI-D3, 24 October 2026):** a non-goal for Stages 1 and 2, with the reversal trigger.

**Type:** decision needed from the owner (FI-D3). **Closes 3 rows** (2 Missing, 1 held by the rule): WA-2.16, CG-16.1, 16.2.

**Why.** WebXR ships in Chromium and in visionOS WebKit, so the two-engine rule may not hold it (CG-16.1, 16.2). HGBA-2's non-goals do not name XR. The `<model>` element is WebKit-only and held (WA-2.16). A WebXR spec would touch GPU, devices, the HAL and permissions; a non-goal costs one line.

**Stage.** Decision now.

**Depends on.** GR; DV.

#### FI-37 Admission records for single-engine features

**Owner decisions (FI-D7, FI-D16, 24 October 2026):** one admission table owned by DM 6.3; other vendors' origin-trial tokens are refused; where possible, Izcal builds its own origin-trial system for its experimental features (site-opted, token-based; distinct from the remote field trials FI-D12 refuses).

**Type:** decision (FI-D7), then amendments to DM 6.3, ST §9 and LY §9. **Closes 24 rows** (3 Partial, 21 held by the rule): WA-2.3, 3.5, 3.10, 3.13, 4.6, 4.11, 6.11, 6.23, 9.10, 17.12, 17.13, 17.18, CG-1.15, 2.6, 3.13, 3.15, 4.22, 4.29, 4.30, 4.32, 5.7, 8.4, 8.6, 9.7.

**Why.** The corpus's admission rule (LY-D10 and ST-D18, applied to web APIs by DM 6.3, DM 9.1 and AH 6.15) holds 77 rows without naming them, and some topical work items above take theirs. These 24 have no topical home. A one-line record per feature (held, admitted with a reason, or refused) in one admission table stops the next audit reopening them. The same table should settle three inconsistencies: customizable `<select>` is held by LY-D10 though Chrome 135 and Safari 27.0 ship it (WA-3.5), Element Timing is listed by DM 6.18 with no DM-D15 exception (WA-17.12), and Reference Target is mapped by AX with no admission (WA-6.11). Section 5 lists the single-engine features already designed without a record.

**Stage.** With FI-14 (DM-M2).

**Depends on.** FI-D7.

#### FI-38 Product non-goals record

**Type:** amendment to `reports/Izcal shell.md` §4.3 or `reports/HGBA shell UI design.md` §8. **Closes 28 rows** (24 Missing, 4 refused implicitly): BF-1.6, 2.16, 2.17, 3.9, 6.12, 6.13, 6.14, 6.15, 11.6, 11.7, 11.8, 12.4, 13.16, 13.19, 13.20, 16.3, 16.4, 16.5, 16.9, 16.10, 16.11, 16.12, 16.13, 17.9, 17.11, 20.3, 20.4, PO-9.27.

**Why.** These conflict with the lean-core, privacy and no-network posture, or belong to the OS: messengers, mail, calendar, video calling, productivity shortcuts, remote desktop, map cards, shopping, store reviews, a crypto wallet, creator rewards, a music player and volume booster, games, peripheral lighting, the Touch Bar, a desktop search bar, a bundled search engine, an in-car browser, and parental controls and kids mode, where the OS family controls are the better owner. Writing them down, or naming mail and calendar as AH Stage 3 app candidates, closes them honestly. Four rows are already refused implicitly (BF-1.6, 3.9, 12.4, 17.11) and only need the line.

**Stage.** Now, one amendment.

**Depends on.** FI-D15 (family controls).

## 5. Contradictions between specs

Found while checking rows. No report was edited; each needs an amendment in the specs named.

| # | Contradiction | Specs involved | Proposed resolution |
|---|---|---|---|
| C1 | **Policy page name.** The shell calls it `about:policies`; NW calls it `izcal://policy` | SH 6.9, 6.14 vs NW 6.14 | The shell owns about pages: NW 6.14 adopts `about:policies` (FI-3) |
| C2 | **PDF annotation stage.** S1 says "Stage 2 adds local annotate, sign and redact" | S1 §6 vs PD 9.5 (annotation authoring at PD-M3, Stage 1) | Apply PD 9.5's pending amendment to S1 §6 |
| C3 | **Picture-in-picture stage.** S1 lists PiP under Stage 2 | S1 13.2 "Everyday features" vs MD 6.12 and MD-M2 (video and Document PiP in Stage 1) | Keep video PiP in Stage 1 (MD); give Document PiP an admission record, because the two inventories disagree on whether one or two engines ship it (FI-37); amend S1 13.2 |
| C4 | **H.264 on Linux.** S1 says OS decoders "(Media Foundation, VideoToolbox or GStreamer)" | S1 13.2 "Codecs" vs MD-D3 (owner decision, 19 October 2026: no H.264 on Linux until a licence decision, no system codec framework) | Apply MD 9.5's pending amendment to S1 13.2 |
| C5 | **Prerender stage.** DM-M3 lists prerendering in Stage 1 (S1 M7) | DM 6.9, DM-M3 vs S1 §9 ("Speculation Rules and prerender", Stage 2) | Keep the activation seam in DM-M3 and move prerendering to Stage 2 as S1 says, unless the owner wants it in Stage 1 |
| C6 | **Customizable `<select>`.** DM staffs it and AX-O12 maps its roles; LY holds `appearance: base-select`, and the hold's trigger now looks met | DM 9.3, AX-O12 vs LY 6.3, LY-D10 | Re-run LY-D10's test against Chrome 135 and Safari 27.0; admit or re-record the hold (FI-37) |
| C7 | **`view-source:` has no owner.** Two specs assume it exists | PH 6.2, SEC AGT-R3 vs no owner | The shell owns it (FI-20) |
| C8 | **Sidebar.** The extension sidebar waits on a shell panel design that is not scheduled | EX-D15 vs shell 9.1, 9.2 | Schedule the panel design in shell 9.2 (FI-17) |
| C9 | **Element Timing and `deviceMemory` exposed without admission records.** Both are Chromium-only | DM 6.18 (Element Timing), DM 6.16 (environment record) vs DM-D15 (exceptions name only Page Lifecycle, LoAF and `notRestoredReasons`) | Add DM-D15 exceptions with reasons, or drop them; recommended: hold Element Timing, do not expose `deviceMemory` (FI-12, FI-37) |
| C10 | **Single-engine features designed with no admission record:** `revert-rule` (CG-1.7), `sibling-index()` (CG-2.5), `corner-shape` (CG-3.9), Document PiP (CG-14.15), Region Capture (CG-15.7), Element Capture (CG-15.8) | ST 6.9, GR-D17, LY 6.14, MD 6.12, MD 6.10 vs ST-D18, LY-D10, DM 6.3 | Each gets a line in the admission table: admitted with a reason, or held with its design kept ready (FI-37) |
| C11 | **OS spellchecker restaging.** S1 still puts the interim OS spelling tier in Stage 1 | S1 13.2 "Text and input" and "Everyday features" vs ED-D8 and ED 6.8 (restaged to Stage 2 by the 21 October 2026 owner decision) | Amend S1 13.2; the ledger already lands BF-10.7, PO-4.12 and PO-9.13 in Stage 2 |
| C12 | **Linux updates.** S1 ships `.deb` and `.rpm`; the updater assumes it writes the install directory | S1 13.2 "Packaging" vs SEC-D14, UPD-R2; SEC 6.5 "Open" | FI-D10 |
| C13 | **Print UI ownership loop.** Each spec points at the other | SH 6.19 vs PD-M4 | PD owns it (FI-8) |
| C14 | **Vertical tabs and split view (wording only).** S1 §6's own row puts them in Stage 2 as table stakes, but its out-of-scope list excludes "UI chrome as differentiation", which the maps read as excluding them | S1 §6 (both places) vs SH-D27 | Not a real conflict; tidy the out-of-scope wording so it names differentiation, not the features |
| C15 | **Shell milestone mapping.** Two different maps of SH-M2 to SH-M5 onto S1 milestones | SH 9.2 vs S1 8.1 | One table; this document uses S1 8.1's |
| C16 | **Revocation filters' stage.** CRLite-style filters | S1 13.2 ("Stage 2") vs NW-D21 (NW-M2, Stage 1) | Apply NW 6.5's pending amendment ("becomes both from NW-M2") to S1 13.2 |
| C17 | **Phantom references.** Specs point at documents that do not exist | SEC 6.15 and 11.4 (privacy spec), MD 6.20 (speech spec), EX (OS-integration design), DV (credential subsystem) | FI-12, FI-29, FI-4 and FI-13 create them. (SH 6.20's password-manager spec and SH-D20's sync spec are reversal conditions, not references, and are not counted here.) |
| C18 | **Undefined worklet kinds.** DM's agent table lists "audio, paint, layout, animation" worklet agents | DM 6.8 vs ST-D18 and LY-D10 (Paint and Layout APIs held) and AN-D9 (Animation Worklet refused) | Mark paint and layout held and animation refused in DM 6.8 |
| C19 | **Held features worded as admitted.** "admitted per LY-D10" and "`column-wrap` and `::column` run in ordinary browsing" | LY 6.10, LY 6.7 vs LY-D10's holds (CG-5.2, 5.9, 4.13) | Reword as "if admitted" |
| C20 | **Stale wording.** DT 6.3 still calls attach consent the "trusted-region prompt"; GR 6.27 says the PDF writer has "no owner yet" | DT 6.3 vs SH-D2; GR 6.27 vs C-21 (settled) | Tidy at the next revision of each |

## 6. Licence and data decisions

HGBA-2 2.3.2's rule: everything Izcal ships, code and data, must be available under MIT; anything else Izcal builds itself. Its exceptions are Unicode data, other standards bodies' own data (WHATWG tables, IANA registries, tz, the OpenType registries, the Public Suffix List, the CT log list), test-only inputs, OS interfaces and the toolchain. Patents are a separate question from licences: an MIT decoder Izcal writes can still need a patent licence.

| Item | Licence or claim | What the specs decide | Status |
|---|---|---|---|
| **H.264 / AVC** | Via LA pool; the page states no patent expiry ([Via LA](https://www.via-la.com/licensing-programs/avc-h-264/)) | OS decoders and encoders on macOS and Windows; Izcal's own decoder ships on Linux only if the owner takes the AVC licence before the first public release; until then Linux has no H.264 and WebRTC negotiates VP8, VP9 or AV1 (MD-D3, MD-D30); no OpenH264 | **Decided**, with a scheduled owner decision; S1 13.2 contradicts it (C4) |
| **AAC** | Via LA pool, no stated expiry ([Via LA](https://www.via-la.com/licensing-programs/aac/)) | AAC-LC through Izcal's own decoder "wherever its patents are confirmed expired"; HE-AAC through OS decoders only (MD-D3) | **Open**: the legal review must confirm expiry; no pool page states it |
| **HEVC, HEIC** | Access Advance and Via LA pools; no software exemption stated ([Access Advance](https://accessadvance.com/licensing-programs/hevc-advance/)) | OS full decoders only, never Izcal code (MD-D3, G57); HEIC not exposed (MD-D1) | Decided |
| **VP9 (and VP8)** | Google's WebM grant covers only "the copyrightable works that implement the WebM codecs distributed by Google", not an independent decoder; Sisvel runs a VP9 pool ([WebM grant](https://www.webmproject.org/license/additional/); [Sisvel](https://www.sisvel.com/licensing-programmes/audio-and-video-coding-decoding/video-coding-platform/)) | Izcal's own VP8 and VP9 decoders, filed as royalty-free (S1 13.2, MD 6.2) | **Open**: VP9 needs its own legal review. The fact-check read the grant for VP9; the same text appears to cover VP8, which was not checked (flagged) |
| **AV1, Opus** | AOM licence covers any implementation ([AOM](https://aomedia.org/license/patent-license/)); Opus grants royalty-free ([Opus](https://opus-codec.org/license/)); Sisvel claims AV1 patents | Izcal's own decoders (MD 6.2) | Decided; Sisvel's claim is a business risk to record |
| **VVC / H.266** | Pools exist | Not exposed: outside MD-D1's closed set | Decided; add it to MD 6.2's "not exposed" list |
| **AC-3, E-AC-3, Dolby Vision** | Licence-encumbered (inventory, not re-read) | Outside MD-D1's closed set, so not exposed; but Chromium in part plus WebKit could meet MD-D1's add trigger, and no licence or OS-decoder route is recorded | **Open** (FI-25, FI-D13) |
| **ALAC** | — | Not exposed; WebKit-only through the OS (MD-D1) | Decided |
| **JPEG XL** | 2022 rANS-related patent question | Legal review before MD-D2 ships. Chrome is "shipping" it from Chrome 155 ([Chrome](https://developer.chrome.com/blog/jpeg-xl-in-chrome)); that it is on by default rests on a secondary source (Low). If so, it meets a two-engine bar | **Open**: the patent review gates it |
| **Voice processing (AEC, noise suppression)** | Patent search | Legal review before MD-M2 (MD 6.2) | Open, scheduled |
| **DRM CDMs** | Commercial | No unlicensed CDM ever; Clear Key built; PlayReady natively on Windows; Widevine and FairPlay only through protected media mode (D-S7, MD-D32) | Decided |
| **Public Suffix List** | MPL-2.0 according to downstream attribution pages only; publicsuffix.org states no licence ([publicsuffix.org](https://publicsuffix.org/)) | Shipped under the standards-data exception (H2 2.3.2; S1 13.2; NW-D7) | Decided; **confirm the MPL attribution from a primary statement** before release. MPL is file-level copyleft, so Izcal's code is unaffected |
| **Unicode, CLDR, UCD** | Unicode License v3, permissive with a notice ([Unicode](https://www.unicode.org/copyright.html)) | Shipped as standards data; Izcal's own algorithms (`hg-ucd`, `hg-unicode`), no ICU | Decided |
| **tz (IANA time-zone database)** | Public domain (not re-checked in the fact-check) | Shipped under H2 2.3.2's standards-data exception, which names it | Decided |
| **CT log list** | Not checked | Shipped as standards data (H2 2.3.2) | Open (minor): check the terms |
| **HSTS preload, root stores, filter lists, safe-browsing lists, GPU blocklist, hyphenation, dictionaries, fonts, HRTF, ICC** | Vendor terms, GPL, CC BY-SA, mixed | Izcal builds its own (NW-D25, NW-D5, NW-D26, CB-D13, PH-D14, GR-D19, TX-D14, ED-D7, TX-D24, MD-D15, MD-D8); Hunspell dictionaries excluded | Decided |
| **Phishing sources** | Phishunt CC0; CERT Polska and URLAbuse unknown | Phishunt ships; the others only if PH-D14's MIT audit allows | **Open**: whether CC0 satisfies "available under MIT" is the owner's call (flagged) |
| **Emoji** | Microsoft Fluent Emoji, MIT | Compiled into Izcal's emoji font (TX-D25) | Decided |
| **On-device ML models and data** | Mixed | Every weight and data set MIT-compatible, tracked in a data ledger; an unledgered weight fails the build (ML-20, ML-23) | Decided; translation, speech and recognition models (FI-29, FI-30) will need ledger rows |
| **Predefined counter styles** | W3C Note data | Not addressed (CG-6.20) | **Open** (FI-23): apply the standards-data test |
| **Services Izcal would run or license** | — | Push: a self-hosted autopush server (S1 13.2; Q-SEC-12 open). Geolocation: no provider (PO-17.6). Default search: a commercial agreement, bounded by CB-D16 (PO-15.13). Casting: proprietary protocols (FI-32) | **Open**: whether the MIT-only rule governs server software Izcal operates but does not ship (FI-D13); a location provider and casting need licensing decisions |

## 7. Decisions for the owner

Each has a recommended default. A bare "yes" means the recommendation.

| # | Decision | Recommended default | Why |
|---|---|---|---|
| **FI-D1** | **Lift the engine-code hold for M1?** | **Yes, now**, with the three M1-exit additions and two pre-M5 decisions of Section 8, and with M0's open start conditions (D-S8 funding, M1–M2 staffing) formally waived or met, since M1 depends on M0 (S1 8.1) | No gap found changes the substrate traits or the IPC wire format; the gaps add specs, not rework |
| **FI-D2** | **Sync and accounts (Q-SH-5): reopen, or confirm "never"?** | Confirm **no account and no sync in Stages 1 and 2**; keep Stage 3 open on one condition: any future sync is end-to-end encrypted from its first version. Reserve stable record IDs and tombstones in SH-M3's schema so that sync stays possible without a migration | Sync is table stakes elsewhere, but it needs servers and accounts that conflict with the no-account posture; the schema reservation is cheap and the reversal is expensive |
| **FI-D3** | **XR** | Record WebXR as a **non-goal for Stages 1 and 2**, with a reversal trigger: a desktop second engine ships it, or it becomes an Interop focus area | The cost is GPU, devices and HAL work for a small audience; silence is the worst answer |
| **FI-D4** | **AI features with the switch on** | Design the **agent task UI, page summarise and multi-tab questions** in Stage 2; refuse image generation, virtual try-on and AI-built extensions and themes; **local models only** in Stage 2 (ML-D5), with cloud providers revisited alongside H3 §7's agent runtime | Matches H3 §7 and B10; keeps the ML service network-less |
| **FI-D5** | **Translation and read aloud: behind `ai.enabled`?** | Read aloud through **OS voices is an accessibility feature outside the AI switch**; page translation and speech recognition stay under the ML grant | Users treat read aloud and translation as reading tools; OS voices need no model, translation does |
| **FI-D6** | **Mobile** | Write the Android shell spec at the **start of Stage 2**; keep iOS refused (S1 13.2) | The engine is Stage 2 already; a phone shell spec written now would go stale |
| **FI-D7** | **Do held single-engine features need individual ledger entries?** | **Yes**: one line each in a single admission table owned by DM 6.3, generated from this ledger (77 rows) | It turns 77 silent holds into answers for the price of one table, and the next audit will not reopen them |
| **FI-D8** | **Installed web apps** | **Build a minimal set in Stage 2**: manifest, install, standalone window, OS shortcuts, badging, uninstall; hold the single-engine APIs | Two accepted specs assume them, and every desktop engine ships them in some form |
| **FI-D9** | **Enterprise** | Write the policy-delivery spec **before M5**; ship per-machine installs, ADMX, MDM, configuration profiles and Linux policy files in **Stage 2**, as SEC-D14's trigger says | The consumers exist already; delivery is what makes them real |
| **FI-D10** | **Linux packages vs the per-user updater** | Packages from Izcal's own signed APT and YUM repositories are **updated by the system package manager**, into **versioned install directories** so a running browser keeps launching children from its own build; the per-user TUF updater stays off for them; the tarball uses the TUF updater | It is what Linux users and administrators expect, it removes the conflict in C12, and versioned directories keep the build-ID check from breaking new tabs after an upgrade |
| **FI-D11** | **CRA role and timing** | Plan as a **manufacturer** (the stricter role) until legal advice says otherwise; get that advice, including whether Nightly and Beta count, **before the first EU distribution** | The Art. 14 duty already applies to products on the market; the steward route depends on D-S8's funding model |
| **FI-D12** | **Telemetry and experiments** | Refuse remote field trials and server-side experiments; keep local flags; usage telemetry opt-in only, under D-S4 | Consistent with D-S4 and B10; a refusal costs one line |
| **FI-D13** | **Codec and service licences** | Commission the **VP9 (and VP8) legal review** alongside JPEG XL and AAC-LC; keep AC-3, E-AC-3 and Dolby Vision **not exposed**; state that the MIT-only rule covers what Izcal ships, not server software it runs | The VP9 grant does not cover Izcal's decoder; the other answers follow MD-D1 |
| **FI-D14** | **Casting** | Not in Stage 1; a Stage 2 decision after a licensing review of the casting protocols | Proprietary protocols cannot be assumed licensable |
| **FI-D15** | **Parental controls and kids mode** | Record as **non-goals for the browser itself**; honour OS family controls and, where an OS offers one, its age signal | The OS owns family settings; age assurance is a legal item (FI-1) |
| **FI-D16** | **Origin trials** | **Refuse**: the admission rule is the only path a feature takes into Izcal | Origin trials are another vendor's mechanism; Izcal has its own gate |
| **FI-D17** | **Installed-app and profile keying** | An installed web app runs in the profile's ordinary (site, container) keys; several profiles share one broker, each profile with its own storage service instance and per-profile services as the process map already counts them | Keeps I-1's key unchanged and avoids a broker per profile (Section 8) |

### 7.1 Owner decisions of 24 October 2026

The owner decided all seventeen on 24 October 2026. Where the answer differs from the recommended default, the difference is in bold.

| # | Decision | Outcome |
|---|---|---|
| FI-D1 | Engine-code hold | Lift for M1 now, with Section 8's three M1-exit additions and two pre-M5 decisions; M0's open start conditions to be formally waived or met |
| FI-D2 | Sync and accounts | **Plan end-to-end-encrypted sync for Stage 2**: a sync-and-accounts spec is written (FI-35), encrypted from its first version; this reverses shell 9.1's "Stage 3 or never" and answers Q-SH-5 |
| FI-D3 | XR | Non-goal for Stages 1 and 2, with the reversal trigger as recommended |
| FI-D4 | AI features with the switch on | Focused set, local models only, as recommended |
| FI-D5 | Read aloud and translation | Split as recommended (OS-voice read aloud outside the AI switch; translation and recognition under the ML grant), **plus an AI feature: a local voice-generation model and similar generative speech tools, behind the AI switch** (FI-29, FI-31) |
| FI-D6 | Mobile | Android shell spec at the start of Stage 2; iOS stays refused |
| FI-D7 | Single-engine holds | One admission table, owned by DM 6.3, generated from the ledger |
| FI-D8 | Installed web apps | **The full PWA platform** (manifest, install, standalone window, OS shortcuts, badging, uninstall, file and protocol handlers, window controls overlay, related apps), subject to the admission rule for single-engine APIs (FI-9) |
| FI-D9 | Enterprise | Spec before M5; ship in Stage 2 |
| FI-D10 | Linux updates | System package manager from Izcal's own signed repositories, into versioned install directories; Izcal's updater for the tarball |
| FI-D11 | Cyber Resilience Act | Plan as manufacturer; legal advice before the first EU distribution |
| FI-D12 | Telemetry | Opt-in usage telemetry only; no remote field trials or server-side experiments; local flags kept |
| FI-D13 | Licences | VP9 and VP8 legal review; AC-3, E-AC-3 and Dolby Vision not exposed; the MIT-only rule covers what Izcal ships, not servers it runs |
| FI-D14 | Casting | **Open protocols first; Izcal writes its own implementation wherever possible; a proprietary protocol is considered only where no open route works.** The owner made this **the project's general rule** for every protocol choice, not only casting (FI-32) |
| FI-D15 | Parental controls | Rely on the OS; non-goals for the browser itself |
| FI-D16 | Origin trials | Refuse other vendors' trial tokens, **and, where possible, build an origin-trial system for Izcal's own experimental features** (FI-37) |
| FI-D17 | Profiles and installed apps | Ordinary (site, container) keys; profiles share one broker |

## 8. What this means for the engine-code hold

M1 is the substrate and IPC skeleton (S1 8.1). Phases A to D2 have merged: the substrate traits (`Clock`, `Entropy`, `Executor`, `Memory`, `Ipc`), the OS and simulation substrates, the DST runner, the wire format with its `.hgidl` generator and Kani proofs, and typed endpoints over the simulated transport. The test is whether any gap would change those, or the process model they serve.

**Gaps that touch M1's design. Small additions, no rework:**

1. **Recording and crash-report header (FI-11, FI-3).** M1's exit requires recordings to carry "the engine build ID, the data versions and the decision-point log". Add three fields now: a **policy snapshot digest**, the **profile and channel identity**, and the **process kind**. Crash reports should use the same header, so a crash and a recording of it join by ID. Adding these after recordings exist in the field would mean a format version bump.
2. **OS-sourced inputs enter as recorded broker inputs (FI-3, FI-4, FI-9).** Enterprise policy (registry, managed preferences, `/etc` files), launch requests from the OS (file and URL opens, a second instance's command line), default-browser status and OS settings are inputs from outside the simulation. Decide the pattern before M1 exits: either a substrate trait that the simulator records, or the OS-adapter process pattern NW already uses for proxy and DNS configuration, delivering typed snapshots over IPC. Either fits the current traits: the `Substrate` struct already grows by milestone (network and storage join later), so this is an addition, not a change.
3. **`.hgidl` is same-build only (FI-4, FI-2).** The wire format has no self-describing schema (VER-1), and the code already enforces one build: `process.hgidl` sends a `BuildId` in `ready` and refuses a child from another build (`ExitReason::BuildMismatch`). Single-instance hand-off (PO-13.10) and side-by-side channels (PO-1.10) mean a launcher of one version talking to a running browser of another, so the launch hand-off needs its own tiny, versioned message, designed in FI-4. The same check matters for updates: if a package manager or updater replaces the binaries under a running browser, every child launched afterwards fails the build check, and no new tab opens until relaunch. Process launch (M5) therefore needs versioned install directories, a spawner that holds its binary open, or a forced-relaunch flow; FI-2 must pick one.

**Process-model decisions to make before M5, which builds process launch, the Windows sandbox and site-per-process (S1 8.1):**

4. **Profiles (FI-10, FI-D17).** The process map has one broker and counts some services per profile. Whether several profiles share one broker or each has its own changes the map, the storage service count and the always-on footprint (FP-1).
5. **Installed web apps (FI-9, FI-D17).** If an installed app ran in a container of its own, every partition key and site-to-process rule would gain an axis. Running it in the ordinary (site, container) keys, as recommended, changes nothing in I-1.

**Gaps that only add later specs.** Everything else: compliance, distribution, enterprise delivery (beyond the two points above), localisation, the shell surfaces, identity, privacy exposure decisions, WebRTC's API, sensors, speech, reading tools, AI features, mobile, casting, embedding, sync and XR. Each sits on capabilities the process map already has: WebRTC in the renderer's worker threads with broker-granted sockets, devices in the device-access service, credentials in the broker over OS stores, speech in the ML service or the broker. The engine inventories' gaps (WA, CG) are owner, floor and milestone fixes in specs whose milestones start at M2 or later.

**Honest limits of this conclusion.** It rests on the specs' process map, not on measurement. Two risks remain. Speech synthesis outside the AI switch (FI-D5) needs a home that is not the ML service, whose count is zero by default (AG-0): a broker-side OS-voice call or a new utility configuration. And background mode for push (Stage 2, PO-8.11) keeps the broker alive with no window, which FP-1's always-on budget must absorb. Neither touches M1. One more fact bears on the hold but is not a gap: M1 depends on M0 (S1 8.1), whose exit includes "D-S8 funding decided; team for M1–M2 staffed", and S1 13.1 says "Stage 1 (M1 onward) cannot be staffed until D-S8 is decided". M1 phases A to D2 have merged anyway, so the record should state that M0's open conditions are waived for this work, or how else it reconciles.

## 9. Open questions and flagged claims

**Open questions.**

- **Q-FI-1.** Is a Nightly or Beta build "made available on the market" under the CRA, and is Izcal a manufacturer or an open-source steward (Art. 24)? Needs legal advice (FI-D11). Owner: compliance. Before the first EU distribution.
- **Q-FI-2.** Is ETSI EN 304 617 (the harmonised browser standard, in draft) finished? It decides whether Class I self-assessment is available. Not checked.
- **Q-FI-3.** Which HTML, DOM, ECMA-262 and Wasm commits are pinned? No report records the pin dates, so the Origin API, Upsert, `JSON.parse` source text and resizable buffers cannot be graded (FI-14, FI-26).
- **Q-FI-4.** Does partitioning satisfy Apple's EU rule to "block cross-site cookies by default"? Matters only if iOS is reopened (FI-12).
- **Q-FI-5.** Does Document PiP ship in one engine or two? The WA inventory says Chrome 116 and Firefox 151 (desktop); the CG inventory says Chromium only (C3).
- **Q-FI-7.** What is Firefox's release cadence? The WA inventory says two weeks from Firefox 154; PO-3.5 says about four weeks, citing the release calendar. Re-check before converting Firefox versions to dates.
- **Q-FI-6.** Does CC0 data (Phishunt) satisfy "available under MIT"? Owner's call under H2 2.3.2.

**Flagged claims.**

- **Chrome's two-week cadence** from Chrome 153 rests on third-party calendars (Low). The inventory's "Chrome 144–153" Privacy Sandbox removal range is unverifiable and should be dropped or replaced with Chrome Platform Status milestones.
- **The PSL's MPL-2.0 licence** comes from downstream attribution pages only; publicsuffix.org states none.
- **VP8** is assumed to share VP9's grant problem because the grant text names the WebM codecs; only VP9 was checked.
- **The PO inventory's header says 57 ship blockers**; its table tags 55. This document uses 55.
- **"An EU choice screen is a legal requirement"** (the BF map's group G) is wrong for Izcal and corrected here (FI-1, FI-18).
- **The inventories' ship cells** for features that moved between June and October 2026 (`if()`, `random()`, `sibling-index()`, `contrast-color()`) were recalled, not re-read, and are Low.
- **Firefox AI Controls, Safari 27's AI features and Edge Copilot Actions** were confirmed only through news coverage.
- **Age-assurance law** (PO-15.7) was not researched beyond naming; it needs a legal read per jurisdiction.

## Appendix A. Evidence

All files are in `research_notes/Izcal feature inventory audit/`.

| File | What it is |
|---|---|
| [`ledger.md`](../research_notes/Izcal%20feature%20inventory%20audit/ledger.md) | **The final status of all 1,117 rows**, with owner, stage, every verdict received and the work item; count tables computed from its rows |
| `web_platform_apis.md`, `css_graphics_media.md`, `browser_product_features.md`, `os_integration_operations.md` | The four inventories |
| `map_web_platform_apis.md`, `map_css_graphics_media.md`, `map_browser_product_features.md`, `map_os_integration_operations.md` | The first mappings against `reports/` at 217e02d |
| `verify_missing_bf_po.md`, `verify_missing_wa_cg.md` | Independent checks of every Missing row |
| `verify_covered_excluded.md` | Spot-check of 100 Covered rows and every Excluded or Deferred row |
| `regrade_bf_po.md`, `regrade_covered_wa_cg.md` | Full re-grades of every Covered row (and the BF and PO Excluded rows) |
| `factcheck.md` | External fact-check: CRA, Privacy Sandbox, Apple's EU engine rules, DMA choice screens, EAA, codec licensing, PSL and Unicode licences, JPEG XL and Chrome's cadence |
