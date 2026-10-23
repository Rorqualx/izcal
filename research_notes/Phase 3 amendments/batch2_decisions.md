# Phase 3, batch 2: record of the owner's decisions of 20 October 2026

Written 20 October 2026, on branch `spec/phase2c`. This file records what the owner decided on 20 October 2026 and where each decision is applied. It is a decision record, not a research note: it cites the documents it changes, and every change it names carries its own dated revision line in the document it lands in.

## 1. Every batch-2 recommendation is accepted

The owner accepted, in full and as written, every recommendation in the decision sections of the three batch-2 specifications:

- **`reports/Izcal SVG.md` §9.4, SV-D1 to SV-D16** (spec now draft 0.3). Notable consequences: SV-D1 answers reconciliation 2's Q-MD-18 **yes** — the DOM can exist without a JS agent heap, over the scriptless document host on the shared column store, which is what lets the media process rasterize opaque cross-site SVG images; SV-D2 fixes SVG geometry on IEEE-strict binary64 with a closed operation list (G122); SV-D4 confirms the SMIL split with the animation spec's AN-D6; SV-D6 replaces independent caps with one weighted ledger per document; SV-D9 opens local SVG files in the scriptless interactive mode.
- **`reports/Izcal animation.md` §9.4, AN-D1 to AN-D13** (spec now draft 0.3). Notable consequences: AN-D2's exact integer numeric contract (i128 nanoseconds, floored Q32.32 progress, fixed-point interpolation in `hg-interp`); **AN-D3 amends the DOM spec's DM-D10** — one recorded tick per frame, each document's frame time relative to its own time origin and quantized to its grid without jitter, `performance.now()` never below it, dedicated workers deriving from the same tick (the per-realm jitter no longer applies to frame time); AN-D4 keeps colour and other uniform-paint classes held behind a flag because the published evidence conflicts; AN-D7 puts View Transition snapshots only in the content GPU process or compositor and the cross-document hand-off in the broker; AN-D10's graded motion controls never remove an animation.
- **`reports/Izcal editing.md` §9.4, ED-D1 to ED-D22** (spec now draft 0.3, its "owner decisions pending" line updated). Notable consequences: **ED-D5 amends the fonts spec's TX-D23** — on Windows, TSF and its text store run in the window-owning UI host (AX-D9 (b)), so third-party input-method DLLs leave the broker; **ED-D22 closes the spellcheck question C-22**, and **ED-D7 departs from ledger row A-S1-75's utility-process placement**: spelling and grammar checking is Izcal's own finite-state engine and lexicons in the renderer, with an interim tier on the OS's local spelling service in a utility process keyed by renderer key for languages without an Izcal lexicon (ED-D8); ED-D10/D11 give the clipboard per-read Paste confirmation, provenance and the command-text warning; ED-D21 routes agent, AT and extension edits through one intent channel that never mints activation.

Each spec's §9.4 carries the stamp "**Owner decision (20 October 2026): every recommendation below is accepted.**", in the form `reports/Izcal JavaScript engine.md` §9.4 uses, and each has a dated revision line (0.2 → 0.3, a notable revision under `CONTRIBUTING.md`).

## 2. The CDP protocol-definition reversal (ground up)

**The owner reversed how `hg-cdp` gets its protocol description.** Izcal does not copy, vendor or pin any file from the ChromeDevTools/devtools-protocol repository — not `browser_protocol.json`, not `js_protocol.json`, nor a pinned revision snapshot of them. The 0.3 text had made those JSON files `hg-cdp`'s pinned input, with black-box observations settling only what they left implicit; that is reversed. `hg-cdp`'s protocol description is **authored by Izcal**, compiled from the CDP's published prose documentation and recorded black-box observations of Chrome's wire protocol in `compat/observations/` (a test case and each engine's observed result, never source). The JSON files are neither an input nor an oracle.

- **DT-D11 ruling (a) of 18 October 2026 stays in force** and is what makes the reversal workable: observations are the oracle for this compatibility adapter's behaviour where the prose documentation is silent.
- **Recorded as DT-D12** in `reports/Izcal DevTools spec.md` (draft 0.3.1; the next free number in its decision section; nothing renumbered). Section 7's anchor table, 6.3's integration row and Section 1's D-S11 bullet are restated; the Section 3 history-table links to the protocol's publication are kept as history evidence.
- **`reports/HGBA Stage 1 architecture design.md` 0.11.1**: D-S11's clause "written from the published protocol definition" is replaced, and D-S12 is clarified (the JSON files are neither input nor oracle); the status line and the 13.2 DevTools row carry the same statement.
- **Ledger**: the batch-2 section records the reversal; A-DT-1's "CDP JS domains translated by `hg-cdp`" is unaffected (it never named the JSON files).

The reason, recorded in DT-D12: a pinned third-party protocol-definition file is another project's machine-readable specification treated as an input, which puts another project's artefact inside Izcal's build and its revision pin inside Izcal's supply chain. Authored-from-prose-and-observation keeps the adapter's provenance Izcal's own, consistent with D-S10's standing rule for everything else DevTools is built from. Client-visible behaviour is unchanged: DT-6 still floors the profile with third-party client suites run as black-box drivers.

## 3. MD-D4 kept strict (no decision)

The owner kept **MD-D4 strict** (media tables are taken only from the standards bodies' own documents, never from inside RFC code components). This was already recorded in `source_evidence_2.3.3.md`; **the images and media spec needs no edit for it**, and none was made. The media spec's only change in batch 2 is the SVG spec's R8 amendment (revision 0.4.1).

## 4. Where the accepted decisions are applied

With the source decisions accepted, the §9.5 amendments follow from them. Applied on 20 October 2026, each with its own dated revision line:

| Document | Version | What landed |
|---|---|---|
| `reports/Izcal fonts and text.md` | 0.3.6 | The editing spec's amendments: TX-D23 (TSF into the UI host), TX-16 scope restatement, TX-D14 and 9.3 (language-data track +10–16†), TX-M1b counting rule, 6.8a and 9.5 R6 spellcheck placement |
| `reports/Izcal DOM, bindings and event loop.md` | 0.2.2 | The editing spec's DOM rows, the animation spec's AN-D3 amendment to DM-D10 and its 6.7/6.15/6.9 rows, the SVG spec's R4 |
| `reports/Izcal HTML and CSS parsing and style.md` | 0.3.5 | The SVG spec's R5 and the animation spec's style rows |
| `reports/Izcal graphics and rendering.md` | 0.4.2 | The SVG spec's R7 and the animation spec's graphics rows (compositor track −1†) |
| `reports/Izcal layout.md` | 0.4.2 | The SVG spec's R6 and the animation spec's layout rows |
| `reports/Izcal images and media.md` | 0.4.1 | The SVG spec's R8 (Q-MD-18 answered) |
| `reports/HGBA Stage 1 architecture design.md` | 0.11.1 | The CDP reversal (D-S11, D-S12) |
| `reports/Izcal DevTools spec.md` | 0.3.1 | The CDP reversal (DT-D12) |
| `research_notes/Phase 2 reconciliation/reconciliation_2.md` | addendum, 20 October 2026 | Q25's "unowned" resolved; C-22 closed by the owner's acceptance of ED-D22/ED-D7; the A-S1-75 departure recorded |
| `research_notes/Phase 3 amendments/ledger.md` | updated 20 October 2026 | G-12, G-13, G-16 closed; C-22 closed; A-S1-75 superseded in part; A-FUT-1, A-FUT-4, A-FUT-5 fulfilled; C-6 staffing addendum; G-18, G-19, G-20 marked as being written (batch 3, started 20 October 2026); batch-2 A- rows added |

The remaining §9.5 amendments of the three specs (to the JavaScript engine, resource-footprint, accessibility, networking, security-scope, content-blocking, DevTools and PDF documents, the shell UI design, the Stage 1 sections outside the CDP item, and HGBA-3) are queued as batch-2 A- rows in the ledger for their owners. The editing spec's filings to upstream working groups are outside Izcal's documents and are not tracked as rows.

## 5. Staffing, as the batch recomputes it

The running total (ledger C-6, addendum of 20 October 2026): the nine component specs' recommendation stands at about **442–570 people†**; batch 2 adds SVG **16–18†**, animation **29–39†** (about **105–175 engineer-years†** to Stage 1 exit, including the about 1† of `hg-interp` work transferred out of graphics's compositor track, which becomes 98–131†), and editing **34–46† plus the 10–16† language-data extension** (the extension sits in the fonts spec's language-data track, whose 4–6† becomes 14–22†; its engineer-years are budgeted by the editing spec, so nothing is counted twice): about **531–689 people†** over the batch, before the DOM spec's 72–92† and the accessibility, DevTools, shell, storage, security, infrastructure and Security-track figures, which are named beside the total, still outside it.
