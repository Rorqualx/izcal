# Phase 3 amendment ledger

Compiled 19 October 2026 on branch `spec/hgba2-2.4`, read-only over every source. No spec was edited. Sources: Section 9.5 (9.4 for the footprint spec) of the nine accepted component specs and of the DevTools spec; [reconciliation 1][RN] (P1–P27, OD-1 to OD-4), [reconciliation 2][RN2] (Q1–Q32, OD-5 to OD-8) and the [licence audit][LA] (OD-L1 to OD-L9); the [security scope][SEC] 9.4 (branch `spec/security-scope`, draft 0.3) and the [coding standard][CS] 9.4 (branch `spec/cs-amendments`, draft 0.3.1, the newest of the three coding-standard branches), both read from their worktrees. **Updated 20 October 2026 (batch 2):** the SVG, animation and editing specs exist (each draft 0.3) and the owner accepted every one of their Section 9.4 decisions; the batch-2 section near the end of this file records the closures, the new A- rows and the CDP protocol-definition reversal. **Updated 21 October 2026 (batch 3):** the WebCrypto, storage and extensions specs exist (each draft 0.3, storage 0.3.1 after its receiver rows) and the owner accepted every decision in each (WC-D1–D10, SG-D1–D12, EX-D1–D16); G-18, G-19 and G-20 close, Q-NW-14 closes, C-6 gains its batch-3 addendum and C-5's registry line is rewritten; the batch-3 section near the end of this file records every A- row, and the decision record is `batch3_decisions.md` in this folder. **Updated 21 October 2026 (batch 4):** the ML service, shell and phishing protection specs exist (each draft 0.3) and the owner accepted every decision in each (ML-D1–D12, SH-D1–D30, PH-D1–PH-D15, with PH-D14 resolved as its recommendation (a)); **G-15, G-21, G-22 and Q-CB-5 close**, C-6 gains its batch-4 addendum, G180–G209 are registered (ML service, phishing protection and shell), and the batch-4 section near the end of this file records every A- row; the decision record is `batch4_decisions.md` in this folder. **Updated 21 October 2026 (batch 5):** the devices spec exists (draft 0.3, DV-D1–DV-D15) and the owner accepted every recommendation in its Section 9.4; **G-26 closes**, **G210–G219 are registered** (devices), C-6 gains its batch-5 addendum (about 613–809† people† at full strength; Stage 1 67.5–96.5† engineers), and the batch-5 section near the end of this file records every A-DV row; the decision record is `devices_decisions.md` in this folder. **Updated 21 October 2026 (batch 6):** the app host spec exists (draft 0.3, AH-D1–AH-D17) and the owner accepted every recommendation in its Section 9.4; **the WASI door closes as an ownership and scope question** — the Stage 1 refusal is narrowed and time-scoped (A4), the Stage 3 backlog row is promoted to name the app host and AH-M0–AH-M5 (A5, A31), and the HGBA-2 X-3 row is restated (A1; A2 subsumed) — **G220–G229 are registered** (app host), C-6 gains its addendum 5 (about 635–845† people† at full strength; Stage 1 unchanged), and the batch-6 section near the end of this file records every A-AH row; the decision record is `app_host_decisions.md` in this folder.

**Source decision status.** Every component spec's decisions are accepted: JS 0.3 (JS-D1–D21), FP 0.3 (FP-D1–D19), ST 0.3.3 (ST-D1–D20), TX 0.3.4 (TX-D1–D25), NW 0.3.3 (NW-D1–D26), LY 0.4 (LY-D1–D22), MD 0.4 (MD-D1–D32), GR 0.4 (GR-D1–D28), CB 0.3 (CB-D1–D24), DevTools 0.3 (Section 9.4 decisions of 18 October; DT-D10 Option B, DT-D11 (a)), and OD-1 to OD-8 and OD-L1 to OD-L9 — **plus, from 20 October 2026: SVG 0.3 (SV-D1–D16), animation 0.3 (AN-D1–D13) and editing 0.3 (ED-D1–D22), all accepted in full**, and DevTools DT-D12 (the CDP protocol-definition reversal, 20 October 2026). The security scope's decisions are accepted except SEC-D6, SEC-D8 and SEC-D9, which its 0.3 line does not list, but the document is **not merged**. The coding standard's decisions are taken and its 9.4 is "applied in 0.3.1", **but only on its unmerged branch**. The State column says whether a row is still to do.

**ID scheme.** `A-<target>-<n>`. Targets: H2S (HGBA-2 superseded sections, 2.4.0), H2 (other HGBA-2 changes), H3 (HGBA-3), S1 (Stage 1 design), JS, FP, ST, TX, NW, LY, MD, GR, CB (the nine component specs), DT (DevTools), UI (shell UI design), SEC (security scope), CS (coding standard), SKILL (first-principles-spec skill), CON (`CONTRIBUTING.md`), LA (licence audit), RN (reconciliation notes), FUT (unwritten specs). **From 20 October 2026 (batch 2):** DM (DOM, bindings and event loop spec), AX (accessibility spec), AN (animation spec), SVG (SVG spec), PD (PDF spec), and ED (editing spec) where a row targets one of the new specs directly. Where several specs ask for the same change, one row lists every source. Conflicts are `C-n`, gaps `G-n`.

## Summary

### Counts per target (snapshot of 19 October 2026; superseded by the Phase 3 completion note at the top of the batch-4 section)

| Target | Rows | Still to apply | Already applied | Superseded or conditional |
|---|---|---|---|---|
| HGBA-2, superseded-section rewrites for 2.4.0 (A-H2S) | 20 | 20 | 0 | 0 |
| HGBA-2, other changes (A-H2) | 25 | 16 | 0 | 9 (security-scope and coding-standard rows; see C-8 to C-11) |
| HGBA-3 (A-H3) | 30 | 30 | 0 | 0 |
| Stage 1 design (A-S1) | 96 | 90 (9 of them security-scope rows taken as Phase 4 input) | 2 (DevTools rows, Stage 1 0.11) | 4 (coding-standard branch rows: three to redo, one superseded) |
| JavaScript engine spec (A-JS) | 10 | 10 | 0 | 0 |
| Resource footprint spec (A-FP) | 40 | 38 | 2 (JS rows, FP 0.2 and 0.3) | 0 |
| HTML and CSS parsing and style spec (A-ST) | 7 | 5 | 2 (reconciliation 1) | 0 |
| Fonts and text spec (A-TX) | 6 | 4 | 2 (reconciliation 1) | 0 |
| Networking spec (A-NW) | 12 | 10 | 2 (reconciliation 1) | 0 |
| Layout spec (A-LY) | 4 | 0 | 4 (layout 0.2–0.3, reconciliation 2) | 0 |
| Images and media spec (A-MD) | 2 | 1 | 1 (reconciliation 2) | 0 |
| Graphics and rendering spec (A-GR) | 4 | 1 | 3 (graphics 0.3, reconciliation 2) | 0 |
| Content blocking spec (A-CB) | 0 | 0 | 0 | 0 (no spec amends it; stale text is in C-17) |
| DevTools spec (A-DT) | 12 | 11 | 0 | 1 (coding-standard branch row) |
| Shell UI design (A-UI) | 7 | 7 | 0 | 0 |
| Security scope (A-SEC) | 25 | 24 | 1 (coding standard, security scope 0.3) | 0 |
| Coding standard (A-CS) | 6 | 6 | 0 | 0 |
| First-principles-spec skill (A-SKILL) | 2 | 1 | 1 (HGBA-2 2.3.1 commit) | 0 |
| `CONTRIBUTING.md` (A-CON) | 4 | 2 | 0 | 2 (one conditional, one on the coding-standard branch) |
| Licence audit (A-LA) | 3 | 3 | 0 | 0 |
| Reconciliation notes (A-RN) | 3 | 2 | 1 | 0 |
| Unwritten specs: SVG, PDF, animation, editing, ML, phishing (A-FUT) | 7 | 7 (when each spec exists) | 0 | 0 |
| **Total** | **325** | **288** | **21** | **16** |

Stage 1 is amended by every spec and by both unmerged documents (96 rows touching every section but 4.2, 4.3 and 9), so it is a redesign, not an amendment pass. The footprint spec is the second hub (40 rows from eight sources), the security scope the third (25 rows).

### Conflicts between amendments

Settled means a later accepted owner decision or reconciliation item decides it; the editor applies the winning text. Unresolved means no decision covers it.

| ID | Conflict | Sources | Settled by | Status |
|---|---|---|---|---|
| C-1 | Stage 1 D-S1 noise rule | TX-D4 / TX-20 (`measureText`), GR-D12 (2D canvas, WebGL, WebGPU), MD-D14 (audio), MD 6.4 (export kept noise), LY-D11 (geometry), CB-D3 (per-script uniformity) | **OD-6** (RN2 Q14), GR holds the text | Settled for noise. LY-D11 and CB-D3 are additions OD-6's text does not carry: fold them into the one D-S1 rewrite (recommendation) |
| C-2 | HGBA-2 I-1: key, cache partition, shared-process exceptions, reduced mode | JS-D20, FP (FP-8, FP-D8, "(top-level site, container)" caches), NW-D7 (F, R, n; static global exceptions), GR and MD (OD-8 list), MD (Q7 keying) | **OD-8**, **P18**, **Q7** | Settled. FP's row is stale (P18 keys in-process caches by (T, C, X, n)); one I-1 rewrite with five parts (recommendation) |
| C-3 | FP-7 scope | FP 9.4 (≤ 5%† corpus, ≤ 25%† iframe-heavy, origin keying), NW (network placement), GR and MD (per-key service processes) | **OD-2**, **OD-5** | Settled for scope. Open measurement left: FP-7's corpus limit is at risk and FP-S30 breaks at 15 frames per iframe-heavy page even without graphics (RN2 Section 3); no owner decision until FP-M3 measures it |
| C-4 | FP-11 per-node split | ST (72 / 80 / 360, P9), LY (200 / 160 "provisional"), GR (paint counted across processes: about 72 B renderer plus ≤ 88 B† display items), TX (text-shaping excluded), JS-D4 (48 B† columns inside DOM's 72) | **P9**, **Q3** (one failure order; split co-signed) | Settled. Drop "provisionally" from ST's and LY's rows |
| C-5 | G-rule registry | ST ("G30–G35 networking, G36 onward"), NW (G30–G36), MD ("after G37–G43"), LY (G37–G45), GR (G60–G72), CB (G80–G87, G73–G79 unassigned) | **P7**, **Q19**, CB critique C31 (owner directed CB to start at G80) | Settled. Final registry below |
| C-6 | Staffing totals | ST (231–290† for five specs), MD and GR (437–560† for eight), CB (+5–9.5†), NW's own Stage 1 row (46–61†, stale: NW is 58–81† since 0.3.2), DevTools (4–5† + 1–1.5†, applied), security scope (Security track 3–4†, SEC-D2) | RN2 Section 4 covers eight specs only | **Settled 22 October 2026.** The addenda below (batches 2–6) *are* the statement: **Ledger A** is the published, human-derived planning and audit basis at about **635–845 people†** full strength (about 628–830 engineers† plus 7.5–14.5† non-engineers), and **Ledger B** is the internal agentic envelope adopted under C-6R with its reversal trigger. No further owner action is needed; the recommendation kept below is the historical starting point. **Recommendation:** one statement for Stage 1 8.3 of about **442–570 people†** for the nine component specs (437–560† + CB's 5–9.5†), plus DevTools 5–6.5† and the Security track 3–4†, with the uncounted list of G-11 to G-21 named beside it. **Addendum (20 October 2026, batch 2):** the running total recomputes as follows. The nine component specs' recommendation stands at about **442–570 people†**. Batch 2 adds, counted nowhere before: **SVG 16–18†** (its own 9.3), **animation 29–39†** (its own 9.3, about **105–175 engineer-years†** to Stage 1 exit, including the about 1† of `hg-interp` work transferred out of graphics's compositor track, which becomes 98–131†), and **editing 34–46† plus the 10–16† language-data extension** (the extension is shared with the fonts spec's language-data track, whose 4–6† becomes 14–22†; the extension's engineer-years are budgeted by the editing spec, so nothing is counted twice). Batch-2 full-strength people, over the nine-spec recommendation: about **531–689 people†** (442–570 + SVG's 16–18 + animation's 29–39 + editing's 34–46 + the 10–16† language-data extension, which is new relative to the fonts figure the nine-spec base carries). The DOM spec's 72–92† (G-11's spec, accepted with its own decisions) and the accessibility, DevTools, shell, storage, security, infrastructure and Security-track figures are named beside this in the running total, still outside it. **Addendum (21 October 2026, batch 3):** storage, WebCrypto and extensions are now written and accepted, and the running total includes them. Batch 3 adds **storage 7–10†** (its own 9.2), **WebCrypto 3.5–5.5†** (its own 9.3, additive to the networking spec's cryptography track with **1–2† shared and not double-counted** — the NW 9.3 row says so), and **extensions 9–14† plus 2–4† non-engineering review and directory operations** (the extensions spec's 9.3; Stage 2, so it is off Stage 1's critical path but is part of full strength). Batch-3 full-strength people, over the batch-2 figure: about **551–719 engineers†** (531–689 + 7–10 + 3.5–5.5 + 9–14, rounded) plus **2–4† non-engineers**, about **553–723 people†**. The DOM, accessibility, DevTools, shell, security, infrastructure and Security-track figures stay named beside this in the running total outside it. **Addendum (21 October 2026, batch 4):** the ML service, shell and phishing protection specs are written and accepted, and the running total includes them. Batch 4 adds **ML service 14–20.5† engineers plus 0.5–1† non-engineering** (its 9.3), **shell 14–20.25† engineers plus 1.5–2.5† design/research** (its 9.2) and **phishing protection 5–8† engineers plus 1.5–3† non-engineers** (its 9.2; the non-engineers are a marginal load on content blocking's shared list staff, not a second governance team, and are not double-counted — the content-blocking figure already carries the lists staff). Batch-4 full-strength people, over the batch-3 figure: about **584–768 engineers†** (551–719 + 14–20.5 + 14–20.25 + 5–8, rounded) plus **5.5–10.5† non-engineers** (2–4 + 0.5–1 + 1.5–2.5 + 1.5–3), about **590–778 people†**. **Stage 1 8.3's own total is 59.5–84.5 engineers† plus 5.5–10.5† non-engineers** after its rows are re-cut (the ML service's 2–3† move out of the accessibility row, as the ML spec's Stage 1 6 row records). The DOM, accessibility, DevTools, security, infrastructure and Security-track figures stay named beside this in the running total outside it; **the shell is no longer outside it.** **C-6R, adopted 21 October 2026.** The owner adopted the two-ledger staffing statement: **Ledger A** (the figures above) remains the published, human-derived planning and audit basis; **Ledger B** — the agentic envelope of `research_notes/Agentic engineering economics/synthesis.md` (compressed effort about 1,700–3,400 engineer-years†, 3.1–6.1M agent-hours, 15–31T processed tokens, $6M–$350M at October 2026 prices, Stage 1 exit floor about 40–60 months set by the measurement calendar and decision cadence) — is adopted as an **internal planning artifact** (Q-AE-1: internal). Reversal trigger: at the first real milestones per activity class (FP-M0, GR-M0, JS-M1, the first phase-8 cycle on agentic output), if two consecutive components measure outside Ledger B's envelope on the same side, the class ranges are restated from the measurements, not the literature. **Addendum (21 October 2026, batch 5):** the devices spec is written and accepted. Its **23–31† engineers at peak** (its 9.3) fold into the running total, and its **9–13† gross / 8–12† net** Stage 1 share folds into Stage 1 8.3's track table. C-6's full-strength total is **not unsupported and not withdrawn**: C-6's figure is about **590–778 people†** (the nine-spec base of 442–570† plus the batch-2, batch-3 and batch-4 folds), and the device track takes it to about **613–809† people†** (590–778 + 23–31). Stage 1 8.3's total rises from **59.5–84.5†** to **67.5–96.5† engineers** (59.5–84.5 + 8–12 net, with 1† shared security and fuzzing work), and the **5.5–10.5† non-engineers** stand. C-6's Stage 1 statement stays as it is — 442–570† plus DevTools 5–6.5† and Security 3–4† — with the device addition to that Stage 1 figure being the 8–12† net share. **Addendum 5 (21 October 2026, batch 6 — *Izcal app host*):** the app host is written and accepted. Its **15–21† engineers at peak** and **2–4† non-engineers** (its 9.3) fold into the running total; **16–39† of its 22–42† engineer-years are incremental**, with 2–3† shared with the JavaScript engine's Wasm work and already named in that track, so nothing is double-counted. C-6's full-strength total becomes about **628–830 engineers† plus 7.5–14.5† non-engineers, about 635–845 people†** (613–809 + 15–21 engineers; 5.5–10.5 + 2–4 non-engineers). Stage 1 8.3 is unchanged: the app host is a Stage 3 track, and its Stage 2 seams (AH 6.19) are named in the Stage 2 backlog without staffing. C-6's Stage 1 statement stays 442–570† plus DevTools 5–6.5† and Security 3–4†, with the device addition being the 8–12† net share and no app-host addition |
| C-7 | Stage 1 dates | JS (Stage 1 exit 39–51 months†, cited by ST 9.3 and NW-M5), LY (48–60†) | **Q29** (milestones re-dated from the specs; M6 and M7 readiness points, not an order) | Settled. HGBA-2's own Stage 1 heading has no row (G-3) |
| C-8 | HGBA-2 S-5 library sandboxing | SEC (S-5 library sandbox for codecs and PDF as a Stage 1 target, SEC-12, SEC-D3), MD (S-5 superseded for media; SEC-12's metric rewritten) | HGBA-2 2.3.0 / 2.3.2 and **MD-D1** (accepted after security scope 0.3) | Settled for media: MD's text. PDF has no owner (G-14). Recommendation: restate S-5 and D6 together (G-8) |
| C-9 | HGBA-2 4.8 FFI rules | SEC (add FFI-R1 to FFI-R4), JS (FFI-R references give way to the unsafe-core rules, JS-14; FFI-R1–R4 replaced by H-1, `Field`, `Root`, JS-10) | HGBA-2 2.3.0 and the JS decisions | Settled: JS's text. The security scope keeps FFI rules only for OS-interface FFI when rebased |
| C-10 | HGBA-2 D2, S-4e and the fork | SEC (SEC-1 clock, Chrome-patched-branch rule SEC-D12, S-4e force-push), coding-standard branch (HGBA-2 "2.2.1": S-4e range-unwrap binding for CS-V8-1; Stage 1 5.3 wrapper tags; `v8-audit` check), JS (S-4e and S-4f retire; SEC-D12 retires; JS-16 replaces SEC-1's V8 clock) | HGBA-2 2.3.0 (V8 excluded) and the JS decisions | Settled: drop on rebase. The branch's revision number 2.2.1 collides with main's 2.3.x line and must not be merged as is |
| C-11 | HGBA-2 S-4a Hardened profile | JS (jitless tier ceiling with W0; Wasm conformance "identical"), SEC (CET/BTI "where hardware and OS support it"; ACG on Windows; "Izcal-only renderers jitless by default") | Partly: JS decides the ceilings | **Settled 21 October 2026** as recommended: S-4a takes JS's text plus SEC's CET/BTI and ACG wording; the default tier stays JS-D13's (Full by default, Jitless for Izcal-only renderers and the banking container) and the security scope's "jitless by default" rule is re-decided at its rebase to match (HGBA-2 2.4.1) |
| C-12 | FP-D16 regenerable-cache class: binding and partition | JS and FP (bound to the engine build, partitioned per (top-level site, container); members include compiled filter automata), NW (HTTP cache bound to format version and security epoch), CB (filter files keyed by list hashes and compiler version, user lists per container), ST and TX (compiled user sheets, font-hash cache) | **P19** (members), **CB-D11** (accepted after P19) | Settled for members. The class definition must state binding and partition per member (recommendation: "bound to its format version and writer; partitioned per (T, C) whenever it holds user- or site-derived state") |
| C-13 | Content-blocking list ownership | NW (FP 6.11: "their production is this spec's", NW-D26), CB (FP 6.11: governance "belong to the content-blocking spec"; NW 9.3: second review, importers, log, witnesses and entity data counted in CB) | CB 9.3 (production stays in NW's lists track; governance, second review and log in CB), CB-D5, CB-D16 | Settled. FP 6.11 text: Izcal's own lists (NW-D26), produced by NW's lists track under CB's governance |
| C-14 | Matcher budget FP-28 | NW (≤ 256 KiB† charged per partition service; sealed automaton file uncharged), CB (compiled default bundle ≤ 8 MiB†, clean and shared; same per-service line; p99 ≤ 20 µs†; compile budgets) | CB is later and compatible | Settled: merge (CB's cap on the clean file plus NW's per-service line) |
| C-15 | HGBA-3 B7 | FP (frozen tabs skip style and layout; hidden running tabs stay pixel-optional), GR (B7 replaced; "skips paint" made one text with Stage 1 5.7's "stops after chunk generation") | GR owns rendering text (Q20 table) | Settled: GR writes B7 with FP's freeze rule |
| C-16 | Keying of decoder and media processes in Stage 1 | FP (decoder process per (site, container), FP-D11; Stage 1 M7 row), MD's own Stage 1 rows ("media process per (site, container)") | **Q7**: every per-site service process keyed exactly like its renderer (origin under FP-D8) | Settled. FP's and MD's rows are stale; A-MD-2 fixes MD |
| C-17 | Extensions host | HGBA-2 D5, W-1, X-3 and Stage 1 D-S3 (Wasmtime), CB 6.15 and NW 6.19 ("Stage 2, Wasmtime"), CB's D-S3 row (three tiers), JS (JS-D10: Izcal's own engine in a component host) | **JS-D10** | Settled. Stale "Wasmtime" text in CB 6.15, NW 6.19, HGBA-2 W-1 and X-3 (G-5) |
| C-18 | Stage 1 utility processes and WebRTC placement | SEC ("WebRTC (network service and utility)"; seventeen process kinds), NW-D3 (WebRTC transport in the partition service), MD (WebRTC media in the media process), TX (font service), FP (font and filter-list utility jobs) | NW-D3, MD-D5 (accepted after security scope 0.3) | Settled; the security scope's count of seventeen kinds is stale and is recounted from the Phase 4 process map |
| C-19 | DevTools replay unit | DevTools (Stage 1 3.4: one process per replay host), GR (a multi-process replay unit: renderer, its content GPU process and the tab's compositor inputs), NW-D2 (network recording joined by request ID, P20), SEC (replay host at the recorded renderer tier, REC-R6) | None: DevTools was never reconciled with the Phase 2 specs | **Settled 21 October 2026** by the owner, exactly as recommended: recordings are per process and joined by IDs (P20's rule); a replay host runs one process kind, and GR's unit replays as a group of hosts with the compositor inputs recorded at the broker boundary (HGBA-2 2.4.1, Stage 1 0.14.2, DevTools 0.3.5) |
| C-20 | DevTools JS domain | DevTools (D-S11 text applied in Stage 1 0.11: "V8's inspector passed through"), SEC (`script.v8` as opaque bytes, IPC-R16), JS (native IIP script domains from `hg-js-observe`, JS-63) | HGBA-2 2.3.0, JS-63 | Settled: JS's text; Stage 1 D-S11 and the 13.2 DevTools row are rewritten in Phase 4 |
| C-21 | PDF and print in Stage 1 13.2 | TX (print-to-PDF subsets fonts), LY (paged layout Stage 1; print UI and PDF writer Stage 2), GR (print raster Stage 1, GR-M3; PDF row reconciled with media's "PDF process" by the future PDF spec), MD (PDF process), Stage 1 13.2 (utility process) | **Q24** for layout and raster | **Settled 21 October 2026.** The PDF spec (accepted, batch 1) is the named owner of the writer, viewer and process; the pagination contract sits between layout's paged layout and `hg-pdf-write` (Stage 1 0.14.2, 11.1). G-14 closes with it |
| C-22 | Spellcheck | TX (Stage 1 13.2: spellcheck in a utility process, 6.8a), RN1 Section 3 (spellcheck utility is Stage 2, budgeted by the editing spec) | **ED-D22 and ED-D7**, accepted by the owner 20 October 2026 | **Closed 20 October 2026 by the owner's acceptance of ED-D22/ED-D7.** Spelling and grammar checking is Izcal's own engine and lexicons in the renderer (`hg-spell`, `hg-lexc`), with results applied at a registered decision point, and an interim tier on the OS's local spelling service in a utility process keyed by renderer key for languages without an Izcal lexicon (ED-D8). This departs from this row's and A-S1-75's utility-process placement and from Stage 1's "spellcheck through OS services (Stage 2)"; the fonts spec (0.3.6, 6.8a and its 9.5 R6 restated) and the Stage 1 row carry the new text |

**Final G-rule registry** (C-5): G1–G9 Stage 1; G10–G12 DevTools; G13–G18 JS; G19–G24 ST; G25–G29 TX; G30–G36 NW; G37–G45 LY; G46–G49 unassigned; G50–G57 MD; G58–G59 unassigned; G60–G72 GR; G73–G79 unassigned; G80–G87 CB; G88–G109 unassigned (the next spec or specs); G110–G119 PDF; G120–G129 SVG; G130–G139 animation; G140–G149 editing; **G150–G159 storage; G160–G169 WebCrypto; G170–G179 extensions (batch 3, all three registered 21 October 2026)**; **G180–G189 ML service; G190–G199 phishing protection; G200–G209 shell (batch 4, all three registered 21 October 2026)**; **G210–G219 devices (batch 5, registered 21 October 2026)**; G220 onward the next specification. The 20 October 2026 wording ("G88 onward the next spec") is superseded by these allocations.

### Gaps

**Superseded HGBA-2 text with no replacement row**

| ID | Superseded text | What exists | Recommendation |
|---|---|---|---|
| G-1 | Section 3 diagram: the renderer box ("style (stylo crate)", "JS+Wasm: V8 (rusty_v8) + V8 sandbox") and the network box | GR rewrites only the GPU box | The 2.4.0 editor redraws the whole diagram from JS, ST, NW, GR and MD |
| G-2 | Section 6, Stage 1: the `cppgc` spike narrative, start conditions (i)–(iii), and the F1, F2 and F3 fallback shapes | JS's S-4c and S-4e rows; JS-D4 retires the mirror (A-H3-2) | Move to a history appendix, as the status line already treats them |
| G-3 | Section 6 Stage 1 heading ("provisionally months 0–18; planning-pass placeholder 24–36") | RN2 Q29: Stage 1 exit 48–60 months† | Restate from Q29 in 2.4.0 |
| G-4 | X-1 riscv64 support tier and its V8-grounded prose | The JS spec covers riscv64 since 0.4 (JS-D22, the third ISA) | **Settled 21 October 2026:** X-1 stays; riscv64 rides the JS, bindings and DOM track per JS-D22 (5–7† inside the JS spec's 65–82†), recorded in Stage 1 8.3 (0.14.2); references no longer conditional |
| G-5 | W-1 and X-3 (Wasmtime host; native Component Model "in the engine") | JS-D10 covers D5 only | **Closed 21 October 2026:** W-1 is restated by A-EX-1 (HGBA-2 2.3.4) and X-3 is replaced by A1 (HGBA-2 2.4.3), which states the component host structurally and names the app host as the Stage 3 deliverable (*Izcal app host* 6.15, A1/A2) |
| G-6 | Section 9.1 "V8 on RISC-V", Section 9.3 rows (`rusty_v8`, `cppgc` spike, Stylo "2x–18x"), Section 10 changelog row for D2 | JS retires Q-1 only | Mark historical in 2.4.0 |
| G-7 | S-6 "network service sandboxed" (one service) | NW-D3 makes system, container and partition services; only the unmerged security scope proposes S-6 text | Restate S-6 for every network process kind (NW) |
| G-8 | D6 and S-5's wasm2c library sandbox in general | MD retires it for media only; M-1 2.3.0 leaves no C library but OS frameworks | Restate D6 and S-5 once (with C-8) |

**HGBA-3 and Stage 1 text no spec rewrites**

| ID | Text | Recommendation |
|---|---|---|
| G-9 | HGBA-3 has no superseded status line, yet Section 3's rows "JS engine is V8 via `rusty_v8`: Kept" and "DOM objects are `cppgc`-managed: Kept", the Section 4 diagram, Section 5's dependency table (V8, Stylo, Taffy, Parley, html5ever, WebRender, wgpu), Section 6's oracle rows (V8 heap verifier, Stylo-based projection definition), D1′, Section 10's non-goal "does not write its own JavaScript engine" and Section 11 contradict HGBA-2 2.3.0 | HGBA-3 3.3.0 in Phase 3: add a superseded status line like HGBA-2's, apply A-H3, and rewrite these from the component specs |
| G-10 | HGBA-3 B8 and Stage 1 5.8 rely on AccessKit, which JS-D11 classifies as engine work | Needs the accessibility spec (G-17); until then B8 and 5.8 state the AccessKit dependency as superseded |

**Components with no spec**

| ID | Component | Named as missing by | Blocks |
|---|---|---|---|
| G-11 | DOM, WebIDL bindings and event loop | JS 9.3 ("the DOM and bindings track cannot generate interfaces until `hg-js-host` exists"), RN1 Section 4, RN2 Section 4, LY 9.3 | Stage 1 5.1–5.4 (Phase 4) |
| G-12 | SVG (document model, SMIL, SVG DOM, geometry APIs) | RN2 Q20 and Q25, GR 6.16 and 9.5, LY 6.5.10 and 9.5, MD-D20 | **Closed 20 October 2026**: `reports/Izcal SVG.md` (draft 0.3) exists and the owner accepted SV-D1–D16 in full; it owns `hg-svg-dom`, `hg-svg-instances`, `hg-svg-model`, `hg-svg-geom`, `hg-svg-interp` and `hg-svg-syntax`, staffs 16–18†, and answers Q-MD-18 yes (SV-D1). GR's, LY's, MD's, ST's, TX's and DM's SVG interfaces are settled in those specs (batch-2 rows below) | GR's and LY's SVG interfaces |
| G-13 | Animations (Web Animations, CSS transitions and animations, owner of `hg-interp`) | RN2 Q20 and Section 4, GR 6.28 and 9.5 (owner to be named before GR-M1), LY 9.5, GR's ST row | **Closed 20 October 2026**: `reports/Izcal animation.md` (draft 0.3) exists and the owner accepted AN-D1–D13 in full; it owns `hg-anim`, `hg-anim-values`, `hg-anim-time`, `hg-view-transition`, `hg-interp` (from AN-M0, split from `hg-interp-geom`), staffs 29–39† (about 105–175 engineer-years†), and GR's owner-named-before-GR-M1 condition is met | GR-M1 |
| G-14 | PDF (writer, viewer, PDF process) | RN2 Q24, GR 6.27 and 9.5, LY 6.7, MD 9.5 (R18), TX (Stage 1 13.2 print-to-PDF; PDF viewer links the text stack) | Stage 1 13.2 (C-21) |
| G-15 | ML service (MD-D25's caption and image-description models; HGBA-2 4.5, D4, R-3) | MD 9.5 (R19), MD 9.3, RN2 Section 4 | **Closed 21 October 2026.** `reports/Izcal ML service.md` (draft 0.3) exists and the owner accepted ML-D1–D12 in full. It owns the ML service, runtime and Stage 1 models (`hg-ml`, `hg-ml-loader`, `hg-ml-service`, `hg-ml-tasks`, `hg-ml-observe`, `hg-ml-api`), registers **G180–G189**, and staffs **14–20.5† engineers plus 0.5–1† non-engineering**. **A-FUT-3 is discharged**; ML-D5's fourth I-1 shared-process exception is applied in HGBA-2 2.3.5, the licence regime is the licence audit's class 6, and the security scope carries the process, tier-P loader, denial tests, model supply chain and crash-report exclusion |
| G-16 | Editing (caret, selection, editing commands, spellcheck budget) | LY 9.5, TX 6.8a, RN1 Section 3, RN2 Section 4 | **Closed 20 October 2026**: `reports/Izcal editing.md` (draft 0.3) exists and the owner accepted ED-D1–D22 in full; it owns the edit engine (`hg-edit`), `hg-caret`, `hg-spell`, `hg-lexc`, `hg-clipboard`, `hg-dnd` and `hg-find`, staffs 34–46† plus the 10–16† language-data extension (budgeted with the fonts spec's track), and closes C-22 (ED-D22) | C-22 |
| G-17 | Accessibility (AccessKit replacement, OS adapters) | JS-D11, RN2 Section 4 ("beyond each spec's share"), LY 6.x ("A11Y track") | G-10 |
| G-18 | WebCrypto owner, and whether it links `hg-crypto` (Q-NW-14) | NW 6.5 and 11.1, RN1 Sections 7 and 8 | **Closed 21 October 2026.** `reports/Izcal WebCrypto.md` (draft 0.3) exists, the owner accepted WC-D1–D10 in full, and WC-D1 settles ownership: the WebCrypto spec owns `hg-webcrypto`, while renderers link the networking spec's `hg-crypto` for the primitives; **Q-NW-14 is answered and closed** (NW 11.1). Staffing is now counted: **3.5–5.5†**, additive to NW's cryptography track with 1–2† shared and not double-counted (C-6 addendum) |
| G-19 | Storage (storage service, IndexedDB, Cache API, quotas) | RN1 Section 4, RN2 Section 4; used by FP-D18, ST-D17 and NW | **Closed 21 October 2026.** `reports/Izcal storage.md` (draft 0.3, 0.3.1 with the batch-3 receiver rows) exists and the owner accepted SG-D1–D12 in full. It owns the storage service and its engine (`hg-store-engine`, `hg-store-blob`, `hg-store-cacheapi`, `hg-store-opfs`, `hg-store-crypto`, `hg-store-observe`, `hg-store-format`, `hg-store-quota`), registers **G150–G159**, and staffs **7–10†**. The Stage 1 process map and the security scope's storage row now have an owner; the Stage 1 13.2 storage row, the six stale "SQLite (WAL)" sentences, the DOM bucket-FS and blob rows and the footprint staging-region accounting are applied (batch-3 rows below) |
| G-20 | Extensions (Stage 2 surface, JS-D10 component host) | CB-D15, CB 6.15, NW 6.19, Stage 1 D-S3 | **Closed 21 October 2026.** `reports/Izcal extensions.md` (draft 0.3) exists and the owner accepted EX-D1–D16 in full. Stage 2, on the component host (JS-D10), one process per extension origin; **G170–G179** registered; staffing **9–14† engineers plus 2–4† non-engineering review and directory operations**, now counted in Stage 1 8.3 and C-6's addendum |
| G-21 | Shell (as a staffed track; the shell UI design is a design, not a spec) | RN1 Section 4, RN2 Section 4, TX ("Shell spec"); A-UI rows land in a design document | **Closed 21 October 2026.** `reports/Izcal shell.md` (draft 0.3) exists and the owner accepted SH-D1–SH-D30 in full; it owns the shell track, registers **G200–G209**, and staffs **14–20.25† engineers plus 1.5–2.5† design/research**. The shell UI design is draft 0.4. It takes the extensions spec's shell UI handoff, and **A-EX-12 stays open** until those Stage 2 surfaces exist. Its 9.5 rows are applied except the accessibility placement/AT-approval rows (**AX-D3/AX-D4, AX-D9, AX-D11**, queued) and the **FP-1 Windows always-on ceiling**, which is an explicit owner decision, not a silent raise |
| G-22 | Phishing and URL reputation (Q-CB-5) | CB-D23 | **Closed 21 October 2026.** `reports/Izcal phishing protection.md` (draft 0.3) exists and the owner accepted PH-D1–PH-D15 in full, with **PH-D14 resolved as its recommendation (a)** (recorded in the phishing spec's 9.3 and the licence audit's feed table). It owns the reputation system, list, pipeline and surfaces, registers **G190–G199**, and staffs **5–8† engineers plus 1.5–3† non-engineers**, a marginal load on content blocking's shared list staff. **Q-CB-5 is answered and closed**: reputation is not a seventh content-blocking list (CB-D23, content blocking 0.3.3) |

**Other gaps found**

- G-23. No coding-standard tier rows are proposed for GR's L2g crates and back ends, LY's crates (L4) or CB's `hg-cb-*` crates (only a security-scope fuzz row for two of them). Add at the coding-standard rebase.
- G-24. The coding standard's Q-CS-4 cites a V8 source file (`sweeper.cc`), which HGBA-2 2.3.1 forbids; the standard has had no 2.3.1 source-evidence pass. Its CS-V8 rules, CS-D3 spike and `v8-audit` check are all V8-specific.
- G-25. Stage 1 Section 12 (amendments from the four verified facts, all about Servo, WebRender and V8) is history; facts 1–4 at the top of the document are obsolete. Retire in Phase 4.
- G-26. Six device APIs (WebUSB, WebHID, Web Serial, Web MIDI, Web Bluetooth, Web NFC) have no owner, no process, no permission model, no recording class and no threat model. **Closed 21 October 2026** by `reports/Izcal devices.md` (draft 0.3, DV-D1–DV-D15): the device-access service `hg-device-main` in its arbiter, session and enumerate configurations joins the security scope's 37-kind map as its 37th kind, the six APIs enter the DOM spec's 6.18 table (11 → 17 rows), the extensions Ring C refuses the device namespaces and the six web APIs, and the security scope gains ATK-10, TB-14, CON-R6–R9, REC-R7 and `device-access.md`. G210–G219 are its rule range (batch-5 section below).

### Order of application, and the recommended phase split

The evidence supports the proposed split with four adjustments: a short **step 0** before Phase 3 (the conflicts no decision covers, and the content-blocking spec, which no reconciliation pass covered); **HGBA-3 needs a 2.4.0-style rewrite**, not just amendments (G-9); the **Stage 1 redesign should take the security scope's and coding standard's Stage 1 rows** that are not V8-specific, so Stage 1 is rewritten once; and the **DOM-and-bindings and accessibility specs** (or an explicit owner placeholder) should exist before Phase 4, because Stage 1 5.1–5.4 and 5.8 have no source text otherwise.

| Step | Targets | Depends on | Parallel with |
|---|---|---|---|
| **Phase 3, step 0** | Owner decisions: C-6 (one staffing statement), C-11, C-19 (DevTools owner), C-22, G-4 (riscv64); owners named for G-11, G-14, G-17 at least. A short reconciliation addendum covering CB (C-12, C-13, C-14, C-17) and the final registry | — | — |
| **Phase 3, wave 1 (hubs)** | FP (A-FP, 38 rows, one editor), JS (A-JS) | Step 0 for C-6 wording only | Each other |
| **Phase 3, wave 2** | HGBA-2 2.4.0 (A-H2S, then A-H2 rows from merged specs); ST, TX, NW, MD, GR (pending rows); DevTools (A-DT, after C-19); shell UI (A-UI); skill (A-SKILL-2); licence audit (A-LA); `CONTRIBUTING.md` (A-CON-1, applied 21 October 2026); reconciliation notes (A-RN). **Batch 3 applied its HGBA-2 rows in 2.3.4** (the storage, WebCrypto and extensions amendments), so 2.4.0 carries only A-H2S and any A-H2 rows still pending | HGBA-2's I-1, S-3, 4.6 and X-4 text cite FP-7, FP-8, FP-D16 and FP-22 as amended in wave 1 | All of wave 2 in parallel; component specs touch disjoint files |
| **Phase 3, wave 3** | HGBA-3 3.3.0 (A-H3 and G-9) | HGBA-2 2.4.0 (B7, B4 and D1′ cite it) | — |
| **Phase 4** | Stage 1 redesign (A-S1: 90 rows, plus the non-V8 parts of A-S1 security and coding-standard rows, D-S1 rewrite of C-1 with one owner signature, G-25) | Phase 3 complete (registry, staffing, dates, process kinds, D-S1 text); G-11 and G-17 specs or placeholders; PDF owner (C-21) | — |
| **Phase 5** | Security scope rebase (A-SEC, its own 9.4 re-cut against 2.4.0, C-8 to C-11 and C-18), then coding standard rebase (A-CS, G-23, G-24, C-10; drop the branch's HGBA-2 2.2.1, Stage 1 and DevTools 0.3.1 edits and redo them against Phase 4's text) | Phase 4 (process map, layers and tiers); the coding standard cites security-scope IDs, so security goes first | Security and coding standard can overlap once the security scope's IDs are fixed |

Unwritten specs (A-FUT) receive their rows when each is written.

## HGBA-2: superseded-section rewrites for 2.4.0 (A-H2S)

All rows target [HGBA-2][H2] text the 2.3.2 status line lists as superseded, or text naming V8, `rusty_v8`, `cppgc`, Stylo, WebRender or wgpu. State: pending for all.

| ID | Section | Change | Source |
|---|---|---|---|
| A-H2S-1 | Section 3 diagram, GPU-service box | The GPU-service box becomes a compositor GPU service plus content GPU processes keyed like renderers (origin under FP-D8) | GR 9.5; GR-D2 |
| A-H2S-2 | 4.2 | Rewrite: Izcal's own engine; Standard and Hardened become tier ceilings (JS 6.8); page Wasm stays in the engine; DrumBrake and the V8 sandbox text give way to W0 and the cage; JS-in-Wasm only as an isolation tool | JS 9.5 |
| A-H2S-3 | 4.2 internal-components paragraph, D5 | Internal components run on Izcal's engine in a component host instead of Wasmtime | JS 9.5; JS-D10 |
| A-H2S-4 | D2 and "Why V8" | Replace with "D2: Izcal's own engine"; the `rusty_v8` ownership, Mozilla roadmap and V8-specific bindings bullets retire | JS 9.5 |
| A-H2S-5 | S-4a | Hardened is the jitless ceiling with W0 for Wasm; "within 1% of Standard" on Wasm conformance becomes "identical" | JS 9.5; JS-1, JS-3 (see C-11) |
| A-H2S-6 | S-4b | Replaced by H-1, H-2, the cage and the trusted region, with JS-12, JS-15 and JS-27's overhead accounting | JS 9.5 |
| A-H2S-7 | S-4c | Phase workers never read `hg-heap` memory; the token API (JS 6.5); heap verifier and GC stress mode in CI | JS 9.5 |
| A-H2S-8 | S-4d, C-1 | Engine-only suites are Izcal's own (JS-1 to JS-3); "test262 equal to the embedded engine's upstream shell" becomes JS-1 | JS 9.5 |
| A-H2S-9 | S-4e, S-4f | Retire (no V8 fork, no V8 platform teardown) | JS 9.5 (see C-10) |
| A-H2S-10 | I-3 | The JS heap sandbox is Izcal's cage with its trusted region and H-2; library-sandbox and MTE layers unchanged | JS 9.5 |
| A-H2S-11 | 4.8 | The V8 C++ FFI paragraph and FFI-R references give way to the engine's unsafe-core rules | JS 9.5; JS-14 (see C-9) |
| A-H2S-12 | Q-1, Q-1a–Q-1c, Section 9 | Retire | JS 9.5 |
| A-H2S-13 | 4.6 | Replace V8 startup snapshots, V8 code cache and SpiderMonkey JSBC with Izcal's image inventory and own code cache; define charged memory per OS, with resident memory beside it; "reclaim never needs memory"; per-OS release mapping; hot, warm, cold and frozen mapped to FP 6.7's states; define the **regenerable-cache class** and its members (code cache, B1 checkpoints, heap export files, live-page snapshots, compiled filter automata) | FP 9.4 and JS 9.5 jointly; FP-D16 (see C-12) |
| A-H2S-14 | 4.3 (style) | Izcal's own style system replaces "uses the `stylo` crate"; adaptive parallelism and bit-identical output kept, now including canonical bytes; HiPC 2016 kept; Stylo's published results added as evidence | ST 9.5; ST-5 |
| A-H2S-15 | 4.3 (layout), P-1 | Layout parallelism is LY-D6; "a parallel layout path that changes output on any WPT test is disabled" becomes LY-4's byte identity; P-1 becomes the joint ST-26 plus LY-26 target with LY-D6's failure action (threshold rises, never demotion to untested opt-in code) | LY 9.5; LY-D6 |
| A-H2S-16 | 4.4, D1 note | Rewritten from GR: Izcal's HAL on Metal, D3D12 and Vulkan; one render graph per process; canonical integer raster; "CPU page rendering is the floor; WebGL and WebGPU fail cleanly"; WebGPU-compat as the compat feature level. Nothing in 4.4 applies to fonts or text | GR 9.5 (GR-D1, GR-D3, GR-D10, GR-D11); TX 9.5 |
| A-H2S-17 | D3, P-2, R-1, R-6 | D3 restated as GR-D1; P-2 retired, replaced by GR-30 and GR-34; R-1 and R-6 mapped to GR-38 and GR-39 | GR 9.5 |
| A-H2S-18 | 5.1, 5.3, 5.5, 5.6, R-2 | `Gpu` is an `hg-gpu-hal` device, not a wgpu device; 5.3 met by in-renderer canvas readback and CPU-observed foreign fences (G72); 5.5 gains GR-D19's short submissions, attribution and per-site suspension, a class rule counting only unattributed losses, and "no fallback to a less sandboxed process"; R-2 becomes "one repaint (chunk resend), no tab reload"; 5.6's reftest match holds by construction | GR 9.5; GR-D12, GR-D19 |
| A-H2S-19 | 4.7 | Rewrite: Izcal's own QUIC, HTTP/3, HTTP/2 and WebRTC transport including DTLS with hybrid key exchange; own TLS on `hg-crypto`, X25519MLKEM768 by default; GSO/GRO probing; services per (top-level site, container) with broker-minted sockets | NW 9.5; NW-D3, NW-D4, NW-D22, NW-D23 |
| A-H2S-20 | 4.7 (crypto) | CPU crypto instructions reached through `hg-crypto`'s per-ISA back ends; crypto agility through `hg-tls`'s group and signature registries, not a provider interface | NW 9.5; NW-D23 |

## HGBA-2: other changes (A-H2)

| ID | Section | Change | Source | State |
|---|---|---|---|---|
| A-H2-1 | I-1 | One list of shared-process exceptions: compositor GPU service (parses no display list, path or shader, G63), audio service and capture service (parse nothing), each with a threat model; content GPU, media, hardware-decode and CDM processes keyed exactly like renderers; no memory-driven co-location | GR 9.5, MD 9.5; **OD-8**, Q7, Q10 | Pending |
| A-H2-2 | I-1 | The reduced-isolation mode names the engine's duties: per-site cages and caches, Spectre hardening, a report on `izcal://sandbox` | JS 9.5; JS-D20 | Pending |
| A-H2-3 | 4.1, I-1 | Reduced-isolation threshold measured by FP-8 on DC-4 and DC-2; origin keying on every desktop class; in-process caches keyed per partition, extending I-1 (b) (key per P18: (T, C, X, n)) | FP 9.4; FP-D8 (see C-2) | Pending |
| A-H2-4 | I-1 (b) | Add F, R and n to the key; name the static global exceptions; shared limits are bounded, not closed, channels | NW 9.5; NW-D7 | Pending |
| A-H2-5 | I-2 | A broker-granted video-only device in the hardware-decode process is a per-OS-recorded exception to "must not open devices" | MD 9.5 (R8); MD-D10 | Pending |
| A-H2-6 | I-6 | WGSL atomics are a high-resolution timer: WebGPU on by default only in cross-origin-isolated contexts, behind a permission elsewhere | GR 9.5; GR-D11 | Pending |
| A-H2-7 | S-7 | MTE where present in both GPU process kinds | GR 9.5 | Pending |
| A-H2-8 | Section 6, S-3 | Tighten to FP-7 (≤ 5%† corpus, ≤ 25%† iframe-heavy, origin keying on); keep "never across sites"; scope per OD-2 and OD-5 | FP 9.4; **OD-2**, **OD-5** (see C-3) | Pending |
| A-H2-9 | Section 6, X-4 | Energy moves from Stage 3 to Stage 1 as FP-22; MD-54, MD-55 and GR-43 are FP-22-style absolute gates | FP 9.4; Q27 | Pending |
| A-H2-10 | Section 5 HAL | `MediaCodec`: split decode with a video-only device in the hardware-decode process, full-decoder sessions, MD-42's conformance check and device-loss fallback, capability-graph costs; a colour-output description per window | MD 9.5 (R7) | Pending |
| A-H2-11 | S-5 | For media, superseded: no C codec library remains; OS decoders confined by G57 and the hardware-decode process | MD 9.5; MD-D1 (see C-8) | Pending |
| A-H2-12 | C-1 | Per-directory floors for `domparsing` and `sanitizer-api` use "≥ the best shipping engine" while all engines are below 70% | ST 9.5; ST-2 | Pending |
| A-H2-13 | M-1 | TX-D9's black-box oracles join the test-only oracle class accepted with JS-D9 | TX 9.5; P25 | Pending |
| A-H2-14 | M-1 | rustls-webpki, rustls, BoringSSL, OpenSSL, NSS, AWS-LC, Go's TLS and the dalek crates join the test-only oracle class; none linked | NW 9.5; P25, NW-D19, NW-D22 | Pending |
| A-H2-15 | M-1 part 3 (and Stage 1 decision log) | Prose documents hosted in code repositories count as prose under OD-7's three conditions | LY 9.5, GR 9.5; **OD-7** | Pending |
| A-H2-16 | M-1 / licence classes | The Vulkan loader as an OS interface on G67's condition; the SPIR-V grammar as standards data; `dxil.dll` and `d3dcompiler_47.dll` as test-only black-box oracles | GR 9.5, RN2 Q31 (owner confirmations) | Pending (owner confirms) |
| A-H2-17 | S-2 | Extend to every sandboxed process type; architecture check; jitless `PROT_EXEC` rule | SEC 9.4; SEC-9 | Phase 5 |
| A-H2-18 | S-4a | CET/BTI "where hardware and OS support it, reported per device"; ACG on Windows for the jitless tier; Izcal-only renderers jitless by default | SEC 9.4 | Phase 5 (C-11) |
| A-H2-19 | S-5 | Library-sandbox design for codecs and PDF becomes a Stage 1 target | SEC 9.4; SEC-12, SEC-D3 | Superseded for codecs (C-8) |
| A-H2-20 | S-6 | Required at M8 on Linux and macOS | SEC 9.4; SEC-D13 | Phase 5 (G-7) |
| A-H2-21 | Section 3 | Record the macOS Seatbelt risk and the backend trait | SEC 9.4 | Phase 5 |
| A-H2-22 | 4.8 | Add FFI-R1 to FFI-R4 | SEC 9.4 | Superseded (C-9) |
| A-H2-23 | I-2 note | Add SH-R4's no-bridge, separate-process, restart and minimum-version rules for the protected-media host | SEC 9.4 | Phase 5 |
| A-H2-24 | D2, S-4e | SEC-1's severity bounds and clock; Chrome-patched-branch rule; SEC-2 drills | SEC 9.4; SEC-D12 | Superseded (C-10; JS-16 replaces the clock) |
| A-H2-25 | S-4e ("2.2.1" on the coding-standard branch) | Fork gains a range-unwrap binding for CS-V8-1 | CS 9.4 (applied on branch) | Superseded (C-10) |

## HGBA-3 (A-H3)

Targets [HGBA-3][H3], frozen at 3.2.0. See G-9 for text no row covers. State: pending, except security rows (Phase 5).

| ID | Section | Change | Source |
|---|---|---|---|
| A-H3-1 | B1 risk, D10, Q-3 | V8's GC and tiering timing are no longer outside Izcal's control; D10's "fork's predictable platform" becomes "the engine's deterministic decision points" | JS 9.5; JS 6.9 |
| A-H3-2 | B4, D8 | "`cppgc` DOM" becomes one copy of the DOM (node columns outside the cage, traced by `hg-heap`); the mirror retires; workers read columns only during a flush with the script thread blocked; mutation log and stable IDs stay; projection equality becomes a debug and Nightly check; spike items 3–5 replaced by JS-M0's spikes | JS 9.5; JS-D4 |
| A-H3-3 | B4 | FP-11's per-node budgets across every copy (with A-H3-2, one copy) | FP 9.4 |
| A-H3-4 | PRIV-2 | Add JS-17's cross-partition cache-probing suite | JS 9.5 |
| A-H3-5 | DET-1 failure action | Remove "if the source is V8 and cannot be injected" | JS 9.5 |
| A-H3-6 | B7 | Frozen tabs skip style and layout; hidden running tabs stay pixel-optional | FP 9.4 (see C-15) |
| A-H3-7 | INC-3 | Re-base on FP-11 once the mirror is retired | FP 9.4 |
| A-H3-8 | PRIV-2 | Add FP-36's cross-partition footprint probing suite | FP 9.4 |
| A-H3-9 | B5 | "Style keeps Stylo's own invalidation and sharing" becomes feature-indexed invalidation with old-value checks, recorded causes, content-addressed sharing and the reference style engine; INC-4's fallback rates include `:has()` and container re-entry | ST 9.5; ST-D7, ST-D10 |
| A-H3-10 | B6 | Computed style becomes content-addressed style groups with deterministic identities and per-agent tables; per-property struct-of-arrays stays rejected | ST 9.5; ST-D7, ST-D8 |
| A-H3-11 | D9 | "Stylo invalidation" becomes "Izcal's invalidation index" | ST 9.5; ST-D10 |
| A-H3-12 | VER-1 | "Font container headers" becomes the whole generated font reader and the session-image reader, with Kani proofs | TX 9.5; TX-11 |
| A-H3-13 | PRIV-2 | Add TX-20's metrics, `local()` and pre-noise canvas-text probes per UI locale and setting, and TX-22's interim-script probe counted in discrete-surface entropy | TX 9.5 (R19) |
| A-H3-14 | DET-1 | Text scope: pages whose resolved faces are web fonts or Izcal faces; interim-script pages replayed exactly only on hosts with the same OS family and manifests; GPU-path glyphs outside pixel hashes | TX 9.5 (R20); TX-30 |
| A-H3-15 | B5, INC-1, INC-2, INC-4, FLUSH-1 | `layout(node, constraint_space) → fragment` stands, with break tokens and a style-version id; fragments are arena records, not `Arc`; B5's hard cases handled by LY 6.4, 6.7, 6.9; INC targets and FLUSH-1 measured at Stage 1 exit, binding at LY-M4 | LY 9.5; LY-22, LY-25 |
| A-H3-16 | B6 | Phase outputs: fragment records with physical coordinates, flat inline item lists and line records | LY 9.5 |
| A-H3-17 | DET-1, PRIV-2 | DET-1's fragment-tree hash is LY 6.13's id-free canonical tree with break tokens, `hg-layout-state` hashed beside it; PRIV-2 adds LY-14's geometry probes and counts the scrollbar-preference bit and localized form-control strings | LY 9.5 |
| A-H3-18 | DET-1 | Scope extended to the software media path; OS-decoded tolerance codecs and hardware paths recorded by hash | MD 9.5 (R20); MD-41 |
| A-H3-19 | VER-1 | Tier 1 parsers include every `.hgbits`-generated parser; Kani covers `hg-simd`'s index arithmetic | MD 9.5 (R21); MD-21 |
| A-H3-20 | PRIV-2 | Adds MD-30's media surfaces, Windows HEVC and N-edition bits, key-system answers, HDCP status, `color-gamut` and `dynamic-range` buckets | MD 9.5 (R20) |
| A-H3-21 | B7, DET-1, PRIV-2 | B7's "WebRender in Stage 1, Vello Hybrid in Stage 2, wgpu" replaced by GR, its "skips paint" made one text with Stage 1 5.7; DET-1's pixel scope extended to the canonical image; PRIV-2 gains GR-20 to GR-24 and GR-44; PRIV-2 (a) restated as distinguishability | GR 9.5; GR-51, **OD-6** |
| A-H3-22 | DET-1 | One whole-renderer recording budget (≤ 1%† of p95 frame time) with per-subsystem shares | RN2 Q15; LY, MD, GR 9.5 |
| A-H3-23 | PRIV-2 | Add a surface inventory, the list-set probes (CB-8) and the learned-state attack classes (CB-9) | CB 9.5 |
| A-H3-24 | PRIV-1 | Measure with protection on and off from the same IP and session, per challenge provider | CB 9.5; CB-14 |
| A-H3-25 | B10 | Session-scope enforcement for automation clients is not agent-only computation | DT 9.5 |
| A-H3-26 | AG-0 | UI-surface list adds `hg-devtools` surfaces | DT 9.5; DT-12 |
| A-H3-27 | B2 (and Stage 1 Section 4) | Add IPC-R1 to IPC-R17, including bound and free endpoints, per-frame endpoints and the nested-sync rule | SEC 9.4 (Phase 5) |
| A-H3-28 | VER-1 | Name the three proven properties; add the per-backend handle harness | SEC 9.4 (Phase 5) |
| A-H3-29 | DST-1 | Add the adversarial-peer measurement and the planted-bug benchmark | SEC 9.4; SEC-5 (Phase 5) |
| A-H3-30 | B9, AG-3 | Add AGT-R1 to AGT-R6 | SEC 9.4; SEC-18, SEC-19 (Phase 5) |

## Stage 1 design (A-S1)

Targets the [Stage 1 design][S1], draft 0.11. State: pending (Phase 4) unless marked.

**Section 1, process map**

| ID | Change | Source |
|---|---|---|
| A-S1-1 | Remove the V8 facts (3 and 4) and the Section 1 V8 text; renderer row holds "agent heaps and the engine"; add `hg-renderer-jitless-main` | JS 9.5 |
| A-S1-2 | FP-D17 wording; spare-process row; Linux spawner row; decoder-process row (now the media process per renderer key, C-16); threads column (FP-32); owns lists rewritten for own components | FP 9.4; FP-D11, FP-D17 |
| A-S1-3 | Utility row's "font parsing" becomes the font service (hashing and sealed copies, no parsing); web fonts parsed in the renderer; PDF viewer's process links the text stack (Type 1 and Type 3 for PDF only) | TX 9.5 (R2); TX-D2 |
| A-S1-4 | Network service row becomes system service (1), container service (1 per active container), partition service (1 per (top-level site, container), plus spares); add proxy-resolver utility, route process, OS-adapter process | NW 9.5; NW-D3 |
| A-S1-5 | Media process at the strictest tier (MD-D5), keyed by renderer key (Q7); new rows: hardware-decode process (MD-D10), audio service, capture service, CDM process (SEC-33) | MD 9.5 (R2) (see C-16) |
| A-S1-6 | GPU service row becomes the compositor GPU service (no WebRender, wgpu or WebGPU); content GPU process per visible renderer key, device-less until needed, exiting 30 s† after hidden; prompt layer placed by the broker | GR 9.5; GR-D2, GR-D28, OD-5 |
| A-S1-7 | Add workspace and pane renderers, replay host and automation gateway; a recorded browsing context group runs in dedicated, freshly launched renderers | DT 9.5 |
| A-S1-8 | Add DevTools process kinds, crash handler, prompt renderer, updater; place WebRTC and the native CDM process; the broker owns native windows and input routing (CON-R3); GPU and network sandbox tiers per OS | SEC 9.4 (Phase 4 input; WebRTC placement per C-18) |

**Section 2, crate graph and rules**

| ID | Change | Source |
|---|---|---|
| A-S1-9 | L0 loses `v8`; add layer L2e and JS 6.15's crates; rewrite G6; add G13 to G18 | JS 9.5 |
| A-S1-10 | L4 `hg-style (Stylo adapter)` becomes `hg-style`; sub-layer L2s with `hg-encoding`, `hg-url`; layer L2p with the parser crates; L3 gains `hg-html-driver`, `hg-cssom`; L0 loses `stylo`, `selectors`, `html5ever`; G1 restated; G19–G24 | ST 9.5; P1, P7 |
| A-S1-11 | L0 loses `parley`; layer L2t; TX's L2s crates; generators `hg-fontgen`, `hg-scriptgen` on G9's list; G25–G29 (G27 extends G18) | TX 9.5 (R4) |
| A-S1-12 | NW 6.21's crates, with `hg-net-wire`, `hg-compress`, `hg-cookie-parse`, `hg-psl`, `hg-crypto` in L2s; G30–G36 | NW 9.5; P1, P7 |
| A-S1-13 | L0 loses `taffy` and `parley`; L4 layout crates; layout tools; `hg-types` geometry module (LY-owned); G9 list; G37–G45; the full registry statement | LY 9.5 |
| A-S1-14 | Layer L2m; `hg-colour` and `hg-simd` in L2s; G50–G57; G46–G49 and G58–G59 unassigned | MD 9.5 (R4); Q18, Q19 |
| A-S1-15 | L0 loses `webrender` and `wgpu`; layer L2g including `hg-interp`; L4 and L5 crates; G9 list gains the paint, SVG-paint, canvas, WebGL, WebGPU and shader crates; G60–G72 | GR 9.5 |
| A-S1-16 | G80–G87 registered to CB; G73–G79 unassigned | CB 9.5 (C-5) |
| A-S1-17 | DevTools 6.1 crates and rules G10–G12 | DT 9.5 |
| A-S1-18 | G3: `hg-sandbox` inside `hg-substrate-os`; `hg-update` uses files and network only through it | SEC 9.4 (Phase 4 input) |
| A-S1-19 | G3 binds test targets; adds `std::env`, `std::os::`, `num_cpus`; tier T tools outside it | CS 9.4 (applied on the unmerged branch; redo in Phase 4) |

**Section 3, substrate and determinism**

| ID | Change | Source |
|---|---|---|
| A-S1-20 | One memory trait set shared with FP (`Memory` with ledger tags and `purge`, `CodeMemory`); JS determinism rows follow JS 6.9 with decision points as recorded inputs; the worker-task-mode field is replaced by the engine build ID, data versions and decision-point log | JS 9.5; JS-D15 |
| A-S1-21 | Same memory trait set; timer tolerance on `post`; simulation counts canonical touched and committed bytes and wakeups | FP 9.4 |
| A-S1-22 | 3.3 style row risk closed by ST-D7's merge; check becomes ST-5 | ST 9.5 |
| A-S1-23 | 3.3 text rows: cache insertions under phase purity outside results; font installs at task boundaries; session-image identity and manifests as recorded inputs | TX 9.5 (R3) |
| A-S1-24 | 3.3 layout rows: phase purity by types, provisional fragment ids, single-writer cache slots, `hg-layout-state` written only by the script thread | LY 9.5 |
| A-S1-25 | 3.3: software media path in scope; hardware and OS decoders recorded by frame hash | MD 9.5; MD-D12, MD-41 |
| A-S1-26 | 3.1, 3.3, 5.1: vsync as the compositor's BeginFrame from a per-OS source; canonical rendering in scope through the CPU back end; WebGL, WebGPU and video display out of scope | GR 9.5; GR-D15 |
| A-S1-27 | 3.1: `NetTransport::connect` becomes "request a socket from the broker"; wire identifiers from `Entropy::crypto()` | NW 9.5; NW-D4 |
| A-S1-28 | 3.2, 3.4: `hg-netsim`; network recordings per NW-D2 | NW 9.5; NW-D2 |
| A-S1-29 | 3.2: protocol-level faults for what-if runs | DT 9.5 |
| A-S1-30 | 3.4: recording starts at launch ("Reload to record"), covers the browsing context group in dedicated processes, never holds credential input, one process per replay host | DT 9.5 (C-19) |
| A-S1-31 | 3.1 and Section 8: CS-D8 task model; Q-CS-1 answered in M1's exit | CS 9.4 (applied on branch; redo) |

**Section 4, IPC**

| ID | Change | Source |
|---|---|---|
| A-S1-32 | 4.1: remove `document.cookie` from the sync allowlist once NW-2's ordering cases pass | NW 9.5; NW-D10 |
| A-S1-33 | Section 4: IPC-R1 to IPC-R17 (with A-H3-27) | SEC 9.4 (Phase 4 input) |

**Section 5, renderer pipeline**

| ID | Change | Source |
|---|---|---|
| A-S1-34 | 5.1: microtasks handed to the host by the engine | JS 9.5 |
| A-S1-35 | 5.2, 5.4: one-copy DOM (node columns outside the cage, `hg-heap`'s node space, lazy wrappers, text arena); 5.4's data plane is those columns; release-by-ID replaces prefinalizers; Izcal's atoms replace `markup5ever` | JS 9.5 and FP 9.4 (same row); JS-D4 |
| A-S1-36 | 5.3: bindings target `hg-js-host`, emitting shape families and IC generators; the fast-call rule retires | JS 9.5 |
| A-S1-37 | 5.3: CS-V8-1 wrapper tags and range unwrap; CS-V8-9 payload and prefinalizer rules | CS 9.4 (applied on branch) — superseded by A-S1-35/36 |
| A-S1-38 | 5.4: attribute log entries are pairs of attribute-set IDs, kept alive until the flush; state, neighbour, text-emptiness, sheet and environment-snapshot entries | ST 9.5 |
| A-S1-39 | 5.5: rewrite from ST 6.6–6.12 | ST 9.5 |
| A-S1-40 | 5.6: "Inline via Parley" becomes TX-M1's text interface | TX 9.5 (R1) |
| A-S1-41 | 5.6 rewritten: no Taffy or Parley; LY's contract, units, store and oracles; arena instead of `Arc<Fragment>`; Spineless queue built from LY-M2 | LY 9.5 |
| A-S1-42 | 5.7: paint consumes `hg-fragment` and `hg-scroll`; chunk order is `hg-hit`'s walk order; paint owns stacking contexts, ink overflow and snapping | LY 9.5; LY-D14 |
| A-S1-43 | 5.7 rewritten from GR 6.3: no WebRender, no "two GPU stacks"; pixel-optional stops after chunk generation | GR 9.5 |
| A-S1-44 | 5.8: semantic plane takes per-character bounds, word boundaries and the glyph-to-character map from `hg-text` | TX 9.5 (R24) |
| A-S1-45 | 5.8: semantic plane reads bounds, line records, `reading-flow` order and the `content-visibility` rule from LY | LY 9.5 (with G-10) |

**Section 6, shell and browser features**

| ID | Change | Source |
|---|---|---|
| A-S1-46 | Rows "Per-site resource meter", "Tab memory and bfcache", "Built-in content blocking": attribution, history rules and interventions; FP-D15 replaces "engine defaults, not user budgets"; reclaim ladder and bfcache byte budgets; own matcher compiled in a utility process | FP 9.4 |
| A-S1-47 | "Built-in content blocking": Izcal's matcher and lists, six layers, one default list set in every named container | CB 9.5; CB-D1, CB-D13 |
| A-S1-48 | "Containers": block-list set per container; named containers share one default set | CB 9.5; CB-D13 |
| A-S1-49 | "Extensions" and D-S3 note: Stage 2's three tiers; no blocking interception API (host per JS-D10, C-17) | CB 9.5; CB-D15 |
| A-S1-50 | "Cookie-banner handling": reuses the list pipeline; hide-first; no clicks in the first version | CB 9.5; CB-D22 |
| A-S1-51 | GPC row (after legal review) and "credentials stored only in the OS keychain" | SEC 9.4 (Phase 4 input) |

**Section 7, build and test**

| ID | Change | Source |
|---|---|---|
| A-S1-52 | 7.1–7.3, 7.5, 7.7: remove the V8 build farm, debug-V8 and V8 update bot; T1 runs `hg-test262` and `hg-wast`; T0 adds `hg-rulecheck`; heap verifier and GC stress builds; Fuzzilli targets the engine; V8 upstream row removed | JS 9.5 |
| A-S1-53 | 7.3, 7.5, 7.7: reference style engine, reference stepper, round-trip suite; ST 6.14's fuzz targets; remove Stylo, `selectors`, html5ever rows | ST 9.5 |
| A-S1-54 | 7.2, 7.5: Kani for changed `.hgfont` accessors; split-and-compare and caches-off; TX-16's broker string test; fuzzing of every `.hgfont` table and Unicode algorithm | TX 9.5 (R5) |
| A-S1-55 | 7.6: device classes DC-16, DC-8, DC-4, DC-2; charged and resident time series; meters; FP-S30 | FP 9.4 |
| A-S1-56 | 7.1: release builds use no `sccache`; downloads hash- or version-pinned | SEC 9.4 (Phase 4 input) |
| A-S1-57 | 7.5: adversarial-peer, transport, GPU and display-list, WebRTC and protocol fuzz campaigns (the "Rust↔V8" campaign is superseded) | SEC 9.4 (Phase 4 input) |
| A-S1-58 | 7.8 and 13.2 (Updates, Packaging): attestations, immutable releases, TUF roles and emergency class, data channels as TUF targets, updater process, key custody, drills (the "reproducible V8 archive" is superseded) | SEC 9.4; SEC-13 to SEC-17, UPD-R1 to UPD-R5 (Phase 4 input) |
| A-S1-59 | 7.1, 7.2: `checked` profile; shipping and DST builds abort; T0 gains the coding-standard jobs; clippy on three OSes | CS 9.4 (applied on branch; redo) |

**Section 8, milestones and team**

| ID | Change | Source |
|---|---|---|
| A-S1-60 | 8.1–8.3, 13.3: JS-M0 to JS-M3 in the milestone table and critical path; engine track; duration re-estimate (JS's 39–51 months†, replaced by 48–60 per C-7); RES-1 to RES-5 replaced by FP's targets | JS 9.5 |
| A-S1-61 | 8.1 M7: images in decoder processes (now media processes per renderer key, C-16); 8.3: footprint track 3–4† plus 2–3† in other tracks; 13.3: RES-1 to RES-5 replaced per FP 8.4 | FP 9.4 |
| A-S1-62 | 8.1, 8.3: M2 and M3 exits from ST-M1 and ST-M2; style track 31–40†; combined staffing (superseded by C-6); unowned layout spec (now written) | ST 9.5 |
| A-S1-63 | 8.1, 8.3: M6 gains TX-M1b; M7's fonts item becomes TX-M2; text track 34–45† and type-design track about 45†; text on the critical path at M4 and M8; double counting removed | TX 9.5 (R1, R25) |
| A-S1-64 | 8.1–8.3: M5 becomes NW-M2 and NW-M3; M7 gains NW-M4; networking off the WPT critical path; network staffing (row says 46–61†; NW is 58–81† since 0.3.2, C-6) | NW 9.5 |
| A-S1-65 | 8.3, D-S8: fund the ongoing root program and lists (8–13†), external cryptography audits (NW-52), CA/Browser Forum and CCADB participation | NW 9.5; NW-D5, NW-D25, NW-D26 |
| A-S1-66 | 8.1–8.3: M4's exit is LY-M1 with TX-M1 and the graphics raster; M6 from LY-M2; Stage 1 exit (M8) 48–60 months†; M6 and M7 readiness points; layout track 52–66† | LY 9.5; Q29 |
| A-S1-67 | 8.1: M4 includes MD-M1; M7 via media processes with MD-M2's contents | MD 9.5 |
| A-S1-68 | 8.3, D-S8: media tracks 55–72†; combined 437–560† for eight specs; funding for codec licences, ISO texts, HRTF, P.863, listening panels, audit | MD 9.5 (R3) (C-6) |
| A-S1-69 | 8.1, 8.3, 10, 13.2: M4's WebRender item becomes GR-M1; graphics track 99–132†; "Two GPU stacks" risk removed; 13.2 rendering row restated; PDF row left to the PDF spec | GR 9.5 (C-21) |
| A-S1-70 | 8.3, D-S8: fund CB's 5–9.5†, crawler compute, hosting, witnesses, legal opinions; funding from a party the lists name conflicts with CB-D16 | CB 9.5 |
| A-S1-71 | 8.3 and decision log: DevTools 4–5† plus 1–1.5†; DT-D11 as D-S12 | DT 9.5 — **applied (Stage 1 0.11)** |
| A-S1-72 | 8.3: Security track 3–4† with a named owner | SEC 9.4; SEC-D2 (Phase 4 input; C-6) |

**Section 13 and the decision log**

| ID | Change | Source |
|---|---|---|
| A-S1-73 | D-S6: consequence (a), which kept MPL-2.0 dependencies, is superseded by HGBA-2 2.3.2; no open-licensed font ships; release notices list no third-party font licence | ST, TX, NW 9.5; OD-L9 |
| A-S1-74 | 13.2: shipped standards data (WHATWG and W3C UA sheets, `entities.json`, Encoding index files) and `@webref/css` as MIT | ST 9.5 |
| A-S1-75 | 13.2 "Everyday features": find in page uses `hg-unicode` caseless and canonical-equivalence matching; print-to-PDF honours `fsType`; spellcheck in a utility process | TX 9.5 (R6) (C-22). **Superseded in part, 20 October 2026: the spellcheck item is replaced by the owner's acceptance of ED-D7/ED-D8** — spellchecking is Izcal's own engine and lexicons in the renderer per language from ED-M2, with the interim OS-service tier in a utility process keyed by renderer key (C-22 closed); the fonts spec's 9.5 R6 row (0.3.6) carries the restated Stage 1 text. The find-in-page and print-to-PDF items stand |
| A-S1-76 | 13.2 "Text and input": "all scripts render" via Izcal faces and the interim OS tier; text differs by OS until a script's face ships; Linux last-resort glyphs; this spec and the broker own IME composition, caret and editing services | TX 9.5 (0.3.3, R1) |
| A-S1-77 | D-S1: remove `measureText`-class metrics from the noise list; font enumeration normalization is TX-D1's OS-family tier; TX-20 compares canvas text before noise | TX 9.5 (R19) (C-1) |
| A-S1-78 | D-S1: scrollbar and form-control geometry normalized by construction, not noise | LY 9.5; LY-D11 (C-1) |
| A-S1-79 | D-S1: audio noise dropped for Izcal-rendered audio once MD-30 passes; kept for OS tolerance codecs; video from OS decoders noised unless MD-42 passed | MD 9.5; MD-D14, OD-6 (C-1) |
| A-S1-80 | D-S1 and decision log: one D-S1 amendment for every readable surface (OD-6's text); the log records OD-6 with the owner's signature and OD-7 | GR 9.5; **OD-6**, **OD-7** (C-1) |
| A-S1-81 | D-S1: per-script uniformity for restricted scripts, gated by CB-D3's measurements | CB 9.5 (C-1) |
| A-S1-82 | 13.2 printing: print UI and PDF writer stay Stage 2; paged layout, `@page` and `window.print()`'s layout are Stage 1; find in page lays out skipped content synchronously | LY 9.5; LY-D8, Q24 (C-21) |
| A-S1-83 | 13.2: TLS, HTTP, DNS (no rustls, neqo, quinn, h3, hickory-dns); certificates (Izcal verifier and root program); content blocking (Izcal matcher and lists); UA row cross-referenced to NW-D12 | NW 9.5 |
| A-S1-84 | 13.2: add a content-blocking row (TUF bundle with a witnessed log; reports through OHTTP) | CB 9.5; CB-D17, CB-D18 |
| A-S1-85 | 13.2 "Codecs": dav1d, libvpx, OpenH264 removed; VA-API/GStreamer removed for H.264; MD-D1 and MD-D3 replace the row (reverses a "Decided" row: owner line needed); legal review before M7 | MD 9.5 (R1) |
| A-S1-86 | D-S7 / 13.1a: key-system answer policy and the protected-playback affordance (MD-D32); spike MD-S5 a dependency of MD-M2's PlayReady exit | MD 9.5 (R5) |
| A-S1-87 | 13.1a / D-S7: the protected-media host's blocking (`WKContentRuleList` on macOS; WebView2 interception routed to a partition service) | CB 9.5; CB-D24 |
| A-S1-88 | D-S11 and 13.2 DevTools row: IIP backend, CDP opt-in profile, no listening port by default | DT 9.5 — **applied (Stage 1 0.11)**; its "V8's inspector passed through" is superseded (C-20) |
| A-S1-89 | 13.2 threat models: add the four DevTools process kinds | DT 9.5 |
| A-S1-90 | 13.2 threat model: list every process kind and the two spanning components; the security scope is the template | SEC 9.4 (count per C-18) |
| A-S1-91 | 13.2 "Everyday features": importer per SH-R2; out-of-DOM autofill; downloads, external protocols and File System Access per SH-R6 to SH-R9 | SEC 9.4; SEC-23 (Phase 4 input) |
| A-S1-92 | BF-1 policy: byte budgets with per-(site, container) sub-budgets and trimmed entries | FP 9.4; FP-16 |
| A-S1-93 | 13.2: rendering row restated (with A-S1-69) and the WebRender, wgpu rows removed | GR 9.5 |
| A-S1-94 | Section 6 and 13.2 TLS, certificates and content-blocking rows retire Brave's `adblock` and third-party default lists | NW 9.5; NW-D26 |
| A-S1-95 | 13.2 "Text and input" (R25): text row cites TX; double counting with the Intl and Unicode track and the security track removed | TX 9.5 (R25) |
| A-S1-96 | Section 12 and facts 1–4 retired as history | Ledger finding (G-25) |

## JavaScript engine spec (A-JS)

Targets the [JS spec][JS], accepted 0.3.1. State: pending for all.

| ID | Section | Change | Source |
|---|---|---|---|
| A-JS-1 | 6.13, 6.15, 9.3 (G6, G18) | `hg-ucd`, UAX 29, UAX 15 and case mapping move to L2s owned by TX, tier P; `hg-intl`, `hg-tz`, `hg-cldr-build` stay; G6 no longer covers the moved crates; `Intl.Segmenter` and case builtins use `hg-unicode`; Intl and Unicode track 4–5† → 3–4†; G18 covers font- and session-image-derived values | TX 9.5; **OD-1**, P3 |
| A-JS-2 | 6.2 (JS-D4) | The 8 B left in the 48 B† columns hold 32 bits of node flags and 32 of element state; other state and `:state()` names in rare data; atoms, attribute sets and inline-style source outside the cage; style output in `hg-style`'s columns; slot release notifies `hg-style`; text arena takes parser buffers without copying | ST 9.5 |
| A-JS-3 | 6.9, 6.15 | `hg-math` owned by JS in L2s with a tier-P build; one per-agent atom table for tag, attribute and CSS names, written only on the script thread; workers use private atom scopes (G23) | ST 9.5; P4 |
| A-JS-4 | 9.5 | The proxy-resolver process is a new jitless embedder with no DOM | NW 9.5 |
| A-JS-5 | AudioWorklet, `hg-math` | AudioWorklet real-time agent requirements and a worklet GC-pause target; `hg-math` f32 entry points (f64 result rounded once) | MD 9.5 (R11); MD-D31, MD-S8 |
| A-JS-6 | JS-D15, `hg-math` | Decision points: canvas command-log snapshots and readback completion, switch to content-process-only, chunk resend, listener timeout, `will-change` expiry, `requestVideoFrameCallback` metadata; `hg-math` f64 for feTurbulence and colour tables; consolidated list and one recording budget | GR 9.5; Q15 |
| A-JS-7 | 6.5, 6.9 (JS-D19) | Release-by-ID also reaches `hg-box-tree`; JS-D19's stack segments move into the substrate so layout recursion runs on them; LY-D15's reversal fires per OS where JS adopts JS-D19 (c) | LY 9.5; Q16 |
| A-JS-8 | JS-D15 | Layout's decision points, including the visual viewport, join the consolidated list; one whole-renderer recording budget with per-subsystem shares | LY 9.5; Q15 |
| A-JS-9 | 6.10, JS-60 | API-call attribution: the innermost non-builtin frame's script is responsible; `eval`, `Function`, string timers, `javascript:` URLs, inserted inline scripts and same-origin child documents inherit the creator's restriction; held outside the cage; within JS-60's budget | CB 9.5; CB-D10 |
| A-JS-10 | Next revision | Items listed in FP 11.3 | FP 9.4 |

## Resource footprint spec (A-FP)

Targets the [footprint spec][FP], accepted 0.3.1. One editor should apply all rows in one pass. State: pending unless marked.

| ID | Section | Change | Source |
|---|---|---|---|
| A-FP-1 | Process kinds, 6.12 | Code-cache builder as a process kind; only the builder writes code-cache entries; footprint 11.3 #6 resolved | JS 9.5; JS-D21 — **applied (FP 0.3: FP-39 row)** |
| A-FP-2 | FP-D2, F_fixed, sharing table, FP-D3, engine image, stack row, code writer, derived-data classes, suspend spike, FP-24, time accounting | JS's 0.1 rows | JS 9.5 — **applied (FP 0.2)** |
| A-FP-3 | 6.3, 6.4, FP-11 | Computed-style row as content-addressed groups and match keys; parsed sheets class (a) keyed per partition; CSSOM-mutated sheets, inline style and transition state primary; FP-11 split DOM 72 / style 80 / layout and paint 360; ST-34 inside the `about:blank` item; partition per (T, C, X, n); sheet rebuild uses kept bytes | ST 9.5; P9, P18, P19 (C-4) |
| A-FP-4 | FP-D16 | Add compiled user style sheets as a member | ST 9.5; ST-D17, P19 |
| A-FP-5 | FP-36 | Add parsed sheets, rule indexes, style arenas, attribute sets and `:visited` to the probing suite | ST 9.5; ST-13 |
| A-FP-6 | 6.3 | Session image owned by `hg-font-service` and `hg-image`; format in `.hgfont`; start-up budget TX-43; rule 5 "hashes against manifests, never parses"; the Izcal font set in the image inventory | TX 9.5; P12 |
| A-FP-7 | 8.2, FP-1 | Font service as a utility job (peak ≤ 8 MiB†, exits); sealed copies charged to the granting (site, container); "interim OS fonts" line | TX 9.5; P12 |
| A-FP-8 | FP-1, FP-10 | Third-party IME code in the broker on its own attributed line, outside the broker's ≤ 20 MiB†; FP-1 measured with no third-party IME | TX 9.5; P13 |
| A-FP-9 | FP-11 | "Text excluded" covers the text-shaping category, including layout's inline item lists | TX 9.5; P9 |
| A-FP-10 | FP-D16 | Add the font-hash cache as a member | TX 9.5; P19 |
| A-FP-11 | 6.4, FP-24, FP-D14 | Text caches and the atlas are class (a) keyed by (T, C, X, n); new canonical-exempt sub-class of class (a) (fixed cap, transparent, own ledger line, outside FP-24); sharing exception for the installed Izcal set and protected OS font files | TX 9.5; **OD-4**, P18, P19 |
| A-FP-12 | 6.6, 8.2 (FP-14) | Atlas as A8 and RGBA8 pools sized from TX-52, with no web-observable device dependence | TX 9.5 (R10); P15 |
| A-FP-13 | 6.12, 8.2 (FP-25), FP-D13 | The Izcal font set joins FP-D13's data set as one signed package; FP-25 lines ≤ 40 MiB† at TX-W1, complete set ≤ 350 MiB† installed and ≤ 200 MiB† compressed; OS interim fonts outside the data set; FP-D13 (a) kept | TX 9.5; **OD-3**, P14 |
| A-FP-14 | 6.18.1, FP-36 | Raster cache, variation instances and lazy validation state as closed channels; font-file residency channel bounded and pre-faulted | TX 9.5 (R9); TX-D21 |
| A-FP-15 | FP-1, 8.2 | Network line itemized (system ≤ 4 MiB†, spare ≤ 1.5 MiB†, within 8 MiB†); 8.2 rows: container service, partition service (idle, settled, frozen), OS-adapter, proxy resolver, route process | NW 9.5; P10, P11 |
| A-FP-16 | 6.1 | Model gains network services, one per open (top-level site, container) and per active container, plus spares; B keeps only the system service and its spare | NW 9.5; P10 |
| A-FP-17 | FP-7 | Scope: renderer isolation; network services' duplicated runtime reported separately and bounded by NW-35 | NW 9.5; **OD-2** (C-3) |
| A-FP-18 | FP-6, FP-S30 | Network services' totals published beside FP-6 (about 90 MiB at FP-S30's settle, about 39 MiB after 10 minutes) | NW 9.5; P10 |
| A-FP-19 | FP-28, 6.11 | Matcher budget per partition service (≤ 256 KiB† charged); the 8 MiB† line becomes a sealed automaton file (clean, uncharged) | NW 9.5; P10 (C-14) |
| A-FP-20 | 6.4, FP-D14 | In-process caches and the atlas key by the (T, C, X, n) projection of NW-D7 | NW, ST, TX 9.5; P18 |
| A-FP-21 | FP-D16 | Add the HTTP cache, bound to its format version and a broker-issued security epoch | NW 9.5; NW-D11 (C-12) |
| A-FP-22 | 6.8 | Frozen pages' WebSocket and WebTransport sessions stay open without wakeups; PING only on request; WebRTC consent checks only when live; Web Push under a grant; FP-17's background suite | NW 9.5 |
| A-FP-23 | 6.18.1 | Rows: connection pools and socket budgets, broker mint queue, receive buffer and decryption queue, HTTP-cache eviction across top-level sites | NW 9.5 |
| A-FP-24 | 6.11 | Default lists are Izcal's own (NW-D26); the shell keeps the per-container choice; user-added lists compiled in the utility process | NW 9.5 (C-13) |
| A-FP-25 | FP-D11, FP-38 | Media process (MD-D5), hardware-decode and CDM processes keyed by renderer key; "pixels reach the GPU service" means the content GPU process of the embedding key | MD 9.5 (R14); Q7, Q11 |
| A-FP-26 | 8.2, 6.16 | Rows for the media process while decoding (MD-53), hardware-decode process, audio service, capture service; CDM process as an attributed OS-code line | MD 9.5 (R14); Q8 |
| A-FP-27 | 6.5, FP-13, FP-14 | Opaque encoded bytes kept once per partition in the media process; FP-13 class W rule (8 B/px) and intrinsic-size decodes charged as canvas readback; FP-14's "video" kind carries hardware-path frames | MD 9.5 (R14); G56 |
| A-FP-28 | 6.8 | Audible-media reason amended: video decode suspended when hidden; hidden muted media suspended without events | MD 9.5; MD-D23 |
| A-FP-29 | FP-22 rule | New energy targets MD-54 and MD-55 as absolute FP-22-style gates | MD 9.5; Q27 |
| A-FP-30 | FP-D13, FP-25 | Named OS-dependent codec exceptions (H.264 and AAC by OS family, Windows HEVC and N-edition bits), codec tables, HRTF set; FP-25 counts the AOM licence notice | MD 9.5; MD-D3 |
| A-FP-31 | 6.1, 6.6, FP-1 | Content GPU process as a model term and process kind: started with a rendering opportunity, device-less ≤ 2 MiB† until it needs a device, ≤ 12 MiB† with one, volatile when hidden, exits 30 s† later, device-less spare; FP-1 gains the shell's device-less process (≤ 2 MiB†; FP-1 ≈ 60.5 of 64) | GR 9.5; **OD-5**, Q8 |
| A-FP-32 | FP-7 | Scope: renderer isolation; per-key service processes (content GPU, media, hardware-decode, CDM) on their own line with absolute budgets, never co-located | GR 9.5, MD 9.5; **OD-5** (C-3) |
| A-FP-33 | FP-S30 | Open measurement: three iframe-heavy pages at 15 cross-site frames exceed the headroom P10 left | GR 9.5; RN2 Section 3 (C-3) |
| A-FP-34 | Ledger lines | View Transition snapshots, OffscreenCanvas surfaces, shared-tile pinning on their own lines; cap of 16† compositor-mutable nodes; prepaint inside the 2.5-viewport budget | GR 9.5 |
| A-FP-35 | 6.6 | Adaptive swapchain rule kept on desktop, Android after Q-GR-15 | GR 9.5 |
| A-FP-36 | 6.4, FP-11 | Display list held by the content GPU process and counted in FP-11's paint share (about 72 B renderer plus ≤ 88 B† items); 200 B† / 160 B† split co-signed with one failure order | GR 9.5; GR-D23, Q3 (C-4) |
| A-FP-37 | FP-14, FP-27, 6.5, 8.2, FP-22 | Atlas per renderer key and partition; FP-27 content GPU spawn row; 6.5's region owner is the content GPU process of the embedding key; 8.2 rows for both GPU process kinds; GR-43 as an absolute gate | GR 9.5; Q11, Q27 |
| A-FP-38 | 6.4, 8.2, FP-11, FP-12, FP-24 | Rows for box columns, fragment store, caches and item lists (class (a), discarded only at a safepoint) and `hg-layout-state` (primary); cache keyed by (constraint-space id, break-token id) with pass-count eviction; handles (id, epoch); canonical bytes count live records; layout 200 B† / paint 160 B†; layout fixed cost ≤ 64 KiB† | LY 9.5 |
| A-FP-39 | 6.11, FP-28 | Compiled default bundle ≤ 8 MiB†, clean and shared; charged matcher state ≤ 256 KiB† per partition service; p99 ≤ 20 µs† per request plus a cold-start case; compile budgets; list governance, breakage and default choice belong to the CB spec | CB 9.5; CB-18, CB-23 (C-13, C-14) |
| A-FP-40 | FP-D16, 6.14, FP-D7 | Compiled filter files keyed by source-list hashes and compiler version, not (T, C); user-list files per container; FP-D7's CPU and wakeup budget applies to restricted scripts without a Reporting API report | CB 9.5; CB-D10, CB-D11 (C-12) |

## Component specs amended by later specs (A-ST, A-TX, A-NW, A-LY, A-MD, A-GR)

Rows from ST, TX and NW to one another were applied by reconciliation 1; rows among LY, MD and GR by reconciliation 2. Rows from LY, MD, GR and CB to ST, TX and NW (accepted earlier) were not.

| ID | Target | Change | Source | State |
|---|---|---|---|---|
| A-ST-1 | [ST][ST] | Metrics ratios, font installs as mutation-log entries, descriptors parsed here and handed over typed, `text-transform` in `hg-unicode`, quantization as a used-value rule, `hg-url` uses L2s for UTS 46 | TX 9.5 (R12); P2, P8, P17 | Applied (ST 0.3) |
| A-ST-2 | ST | `hg-url` and `hg-encoding` owned here; CSP split; speculative fetches as ordinary fetches; cosmetic filters as user-origin sheets | NW 9.5; P6, P16, P21, P22 | Applied (ST 0.3) |
| A-ST-3 | ST | Computed colours stay f64 via `hg-math`; `hg-colour` is the pixel path with one hand-off; gamut mapping chosen jointly; `dynamic-range-limit` to the GPU service; `@color-profile` bytes to the media process's ICC parser; `::cue` hand-off | MD 9.5 (R12); MD-D28 | Pending |
| A-ST-4 | ST | Forced-colours backplates owned by `hg-paint`; `will-change` per GR 6.3; the "affects" mask's paint-only category feeds chunk invalidation; compositor-eligible properties interpolate through `hg-interp`'s integer form | GR 9.5 | Pending |
| A-ST-5 | ST 6.10, 6.12, 6.15, 9.2, 9.5 | Container re-entry at a scheduler barrier with no layout worker running; "once per (container, distinct queried size) per pass" with a pass-local overlay; the "affects" mask's five categories plus scroll-anchoring flag; ST-M2's reference block layout from `hg-layout-reference` (by month 21†); `getComputedStyle` via `hg-layout-query`; scroll-state snapshot after ResizeObserver; 360 B† split 200 / 160; registry text replaced by the full registry | LY 9.5; LY-D12 (C-4, C-5) | Pending |
| A-ST-6 | ST 9.3 | "Stage 1 exit (39–51 months)" becomes 48–60 months†; ST-M3 follows | LY 9.5; Q29 (C-7) | Pending |
| A-ST-7 | ST 6.16 | User-origin-only procedural pseudo-classes; cosmetic subsets as re-serialized selectors with one fixed declaration (G81); background-image and font eligibility of user-hidden elements from a style-only counterfactual | CB 9.5; CB-D12, CB-D13 | Pending |
| A-TX-1 | [TX][TX] | Interfaces as reconciled (metrics, descriptors, `@font-face` rules, web-font installs, `hg-url` uses `hg-ucd` and `hg-unicode`) | ST 9.5; P8, P17 | Applied (TX 0.3) |
| A-TX-2 | TX Q-TX-11 | Answered: Izcal's own Brotli and zlib in `hg-compress`, tier P, L2s, delivered at NW-M1 | NW 9.5; NW-D6, P5 | Applied (TX 0.3) |
| A-TX-3 | TX | `sbix` and `CBDT` images decoded by `hg-png`, `hg-jpeg`, `hg-tiff` in the media process; OT-SVG embedded rasters in the media process or disallowed; read-only font endpoint for the opaque-SVG host | MD 9.5 (R15) | Pending |
| A-TX-4 | TX 6.1, 6.10, 6.15 | `hg-glyph-atlas` in the content GPU process of its key; bounded path types in `hg-display-list` (L2), flattened by `hg-path`; one D-S1 amendment; TX decides masks versus paths; bounded COLRv1 item; mask compositing in encoded sRGB; TX-D13's GPU outline path only on display-only fallback devices | GR 9.5; Q11, Q18, OD-6 | Pending |
| A-TX-5 | TX 6.8, 9.5 (R22) | TX-D4 unchanged; the line-breaking helper provides break opportunities with cumulative positions and reshape on demand; layout decides and converts once; per-glyph ink bounds and per-character bounds through `hg-text`; SVG text per-character positions from layout | LY 9.5 | Pending |
| A-TX-6 | TX 9.2, 9.3 | Stage 1 exit citations become 48–60 months†; TX-M3 follows | LY 9.5; Q29 (C-7) | Pending |
| A-NW-1 | [NW][NW] | Interfaces as reconciled (speculative and style-initiated requests, CSP split, ORB and strict MIME for style, `hg-encoding` and `hg-url` owned by ST, cosmetic filters) | ST 9.5; P2, P6, P16, P21, P22 | Applied (NW 0.3) |
| A-NW-2 | NW | `hg-compress` serves WOFF2 under a 128 MiB† per-font budget, delivered at NW-M1; font and IFT fetches as destination `font`; text caches use (T, C, X, n) | TX 9.5 (R13); P5, P18, P23 | Applied (NW 0.3) |
| A-NW-3 | NW 6.3 | `hg-layout-query` publishes per-frame viewport-intersection classes to the loader; each priority update and lazy-load start is a recorded decision point; natural sizes from the first decoded header | LY 9.5 | Pending |
| A-NW-4 | NW 9.2 | NW-M5's alignment with Stage 1 exit becomes 48–60 months† | LY 9.5; Q29 (C-7) | Pending |
| A-NW-5 | NW | Opaque bodies delivered by the partition service directly to the media process; ORB's subsequent-range marker set only by the partition service; typed endpoint for decrypted RTP and RTCP; `hg-crypto` gains AES-CTR and `cbcs`; `hg-compress`'s deflate encoder bit-identical; partition-scoped throughput estimate; range-request caching | MD 9.5 (R13); G56, G55 | Pending |
| A-NW-6 | NW 6.8 | P18's projection assumes F fixed per process, which holds because content GPU processes are keyed like renderers | GR 9.5; GR-D2, G68 | Pending (one sentence) |
| A-NW-7 | NW 6.15 | Semantics per CB 6.2 (request context, scoped exceptions, total order, phases A–C, popups, `restrict` and `permissions`, `csp=`, linear-time regex subset); first-party scope; CNAME at every hop and IP ranges; "collapse" becomes paint-only suppression; DNR and Safari JSON import; CB 6.8's list set | CB 9.5; CB-D6, CB-D9, CB-D11, CB-D13, CB-D14 | Pending |
| A-NW-8 | NW 6.15, NW-54 | Report contents per CB-D18; NW-54 counts interaction-triggered and cosmetic breakage; denominator excludes Anti-fraud and Restrict entries | CB 9.5; CB-D18 | Pending |
| A-NW-9 | NW 6.9 | Faster bounce-tracker rule beside the 7-day backstop in `hg-nav-protect`; cookie-lifetime caps for cloaked responses and restricted scripts | CB 9.5; CB-D8, CB-D9, CB-D10 | Pending |
| A-NW-10 | NW 6.13, NW-D17, NW-36 | Bundle as a TUF target with a witnessed log; third-party list URLs fetched by each container's service through its route; report endpoint through OHTTP | CB 9.5; CB-D17, CB-D18 | Pending |
| A-NW-11 | NW 6.21 | New crates `hg-cb-*` and `hg-nav-protect`; rules G80–G87 | CB 9.5 (C-5) | Pending |
| A-NW-12 | NW 9.3 | Lists track size confirmed at CB-M2 from measured review throughput; second review, importers, log, witnesses and entity data counted in CB | CB 9.5 (C-6, C-13) | Pending |
| A-LY-1 | [LY][LY] | Inherits 360 B† for layout and paint; owns container re-entry scheduling, the reference block layout and the "affects" categories | ST 9.5 | Absorbed (LY 6.10, LY-D12) |
| A-LY-2 | LY | Numeric contract and `break_reshapes` / `join_reshapes` contract | TX 9.5 (R22) | Absorbed (LY 6.5.2) |
| A-LY-3 | LY 6.0 | Intrinsic sizes from MD's typed headers; cue box layout; the media process is the one non-renderer linking layout crates | MD 9.5 (R16); MD-D20, Q25 | Applied (RN2) |
| A-LY-4 | LY | `hg-paint-order` production from GR-M1; snapping and the device-pixel box function; integer layer origins; hit-test regions clamped by the broker; IO v2 rule; event-time scroll offsets; print and PDF vector output consume the display list; 200 / 160 co-signed | GR 9.5; Q1–Q6, Q24 | Applied (RN2) |
| A-MD-1 | [MD][MD] | Routing into GPU processes, one colour contract, frame selection, channel list, keying, `hg-simd`, class W, origin-clean frames, `VideoFrame` noise, capture split, GR-41 replaced by MD-54 | GR 9.5; Q7, Q11, Q12, Q18, Q21, Q23, Q26–Q28 | Applied (RN2) |
| A-MD-2 | MD 9.5 Stage 1 rows | "Media process per (site, container)" in its Section 1 and 8.1 rows becomes "per renderer key" (Q7) | Ledger finding; Q7 (C-16) | Pending |
| A-GR-1 | [GR][GR] | Mask compositing colour space, bounded path format, GPU outline tolerance, CPU compositing of canvas 2D text | TX 9.5 (R21) | Absorbed (GR 0.3, Q18) |
| A-GR-2 | GR | Owner of graphics; co-signs 200 / 160; owns `hg-paint-order`; region routing; scroll delta protocol; snapping; paint identities without fragment ids | LY 9.5; Q1–Q6 | Applied (GR 0.3) |
| A-GR-3 | GR | Opaque-image raster in the content GPU process; display conversion in the compositor; video layers; overlays; one list of cross-origin rendering channels | MD 9.5 (R16); Q11, Q28 | Applied (GR 0.3) |
| A-GR-4 | GR, display-list generation | Paint suppression of elements whose load content blocking refused: not drawn, layout unchanged, flag exposed to no API (IO v2 included); clicks to a blocked link target not followed | CB 9.5; CB-D13 | Pending |

## DevTools spec (A-DT)

Targets the [DevTools spec][DT], draft 0.3. State: pending (wave 2, after C-19) unless marked.

| ID | Section | Change | Source |
|---|---|---|---|
| A-DT-1 | DT-D1, DT-D2, Section 7, Q-DT-11, M2, M7 | `script.v8` replaced by native IIP script domains from `hg-js-observe` (evaluation, console, exceptions at JS-M1; debugger and profiler at JS-M2); DT-4(d) exemption ends; recording configuration is production; progress counter answers Q-DT-11; CDP JS domains translated by `hg-cdp` | JS 9.5; JS-50, JS-63 (C-20) |
| A-DT-2 | Stage 2 views, 6.5 | Per-site memory view from the ledger moves to Stage 1's late milestones; deterministic footprint replay; recorded-session processes budgeted as site processes; scoped principals see out-of-set frames as opaque | FP 9.4 |
| A-DT-3 | Q-DT-4, M3 | Cascade trace is engine output; Stylo spike and contingent explainer (2 + 8) removed; Style domain carries it; parse decisions, restyle causes, selector statistics, static selector queries, with `Redacted<T>` for non-origin-clean data | ST 9.5 |
| A-DT-4 | IIP | Add the `text.*` domain | TX 9.5 |
| A-DT-5 | DT-D2, DT-D5, M5, Q-DT-5 | Network recording unit per NW-D2; qlog from `hg-quic`; Q-DT-5 answered; live link model for a granted partition; 8 engineer-months move to NW | NW 9.5; NW-D2 (C-19) |
| A-DT-6 | Section 3 table, Q-DT-5 | "Own stack (rustls, neqo, hickory-dns)" becomes Izcal's own TLS, QUIC and DNS; `hg-tls` emits DT-D5's events | NW 9.5; NW-D22 |
| A-DT-7 | DT-D3, DT-D4, IIP | Layout is "anchored", not "mapped"; `layout.*` domain; B5 no longer gates layout why-queries | LY 9.5 |
| A-DT-8 | IIP | `media.*` domain with hardware-path frame hashes and "not exact" replay marking | MD 9.5 (R23); MD-60 |
| A-DT-9 | IIP, replay | Rendering domain (frame records, layers, repaint causes, GPU attribution, colour explainer, canonical captures); multi-process replay unit | GR 9.5; GR-D24 (C-19) |
| A-DT-10 | Network view, IIP | Typed reason for blocked requests; `Protections` domain and view, read by agents only for their origin set | CB 9.5; CB-D20 |
| A-DT-11 | 6.1, 6.5 rules 4, 8, 10 | Replay host at the recorded renderer tier; broker owns the loopback listener; imported bundles, OS-bound key, retention, timestamp quantization; launch-flag rule (the `script.v8` part is superseded) | SEC 9.4; REC-R1 to REC-R6, AGT-R8 (Phase 5) |
| A-DT-12 | G12 | `hg-substrate-sim` follows tier R and G3 | CS 9.4 — applied as DevTools 0.3.1 on the unmerged branch; redo at rebase |

## Batch 2: the SVG, animation and editing specs (added 20 October 2026)

Three new specs exist, each draft 0.3, and **the owner accepted every recommendation in each spec's Section 9.4 on 20 October 2026**: SVG (SV-D1–D16, G-12), animation (AN-D1–D13, G-13) and editing (ED-D1–D22, G-16; ED-D22 closes **C-22**). The owner also reversed the CDP protocol-definition input (**DT-D12**, DevTools spec 0.3.1, Stage 1 design 0.11.1): Izcal does not copy, vendor or pin any file from the ChromeDevTools/devtools-protocol repository (`browser_protocol.json`, `js_protocol.json`, or a pinned revision snapshot of them); `hg-cdp`'s protocol description is authored by Izcal from the CDP's published prose documentation and recorded black-box observations (`compat/observations/`, per DT-D11 ruling (a), which stays in force); the JSON files are neither an input nor an oracle. A-DT-1's "CDP JS domains translated by `hg-cdp`" is unaffected. Section 3's history-table links in the DevTools spec stay as history evidence. A decision record for all of this is `batch2_decisions.md` in this folder; MD-D4 was kept strict (source_evidence_2.3.3.md), so the images and media spec needed no edit beyond the SVG amendment below.

Rows follow the ID scheme; where the amendment is already applied, the target document's revision line of 20 October 2026 is the record.

| ID | Target | Change | Source | State |
|---|---|---|---|---|
| A-ST-8 | Style spec (R5; animation 6.9, 6.10, 6.12, 6.15) | `StaticSink`; computed-length type fixed binary64; per-style-scope tables (agent or static document); `ResourceLoader` traits; SVG presentational hints from `hg-svg-model`; svg-geometry and svg-resource "affects" categories; overlay values through `hg-tree-traits`; the four SVG crates in L2p under G24; G23 covers the SVG micro-grammars; `AnimationSource` trait; effect-stack snapshot read during the parallel pass; transition candidates written by workers; transition creation as a sequential post-pass; base value and "display ignoring animation origins"; one style change event per flush, a discard rebuild is not one; animation-type and motion-inducing overlay columns; static transform trigonometry through `hg-interp`; the two open inputs defined | SVG 0.3 (R5), animation 0.3 | **Applied (style 0.3.5, 20 October 2026)** |
| A-TX-7 | Fonts and text spec | TX-D23 amended (ED-D5 (b): TSF and its text store in the window-owning UI host); TX-16 scope restated (ED-D6: only the bounded snapshot window reaches an OS input-method interface; fuzzing covers the snapshot path); TX-D14's language-data track gains spelling lexicons, morphology, error models, grammar rules and autocorrect thresholds (10–16†, ED-40); TX-M1b counting rule (relay and caret/selection movement counted in the editing spec); 6.8a and 9.5 R6 spellcheck placement (ED-D7 renderer engine, ED-D8 interim OS tier) | Editing 0.3 (ED-D5 to ED-D8, ED-D22) | **Applied (fonts 0.3.6, 20 October 2026)** |
| A-GR-5 | Graphics spec (R7; animation rows) | The SVG paint contract from the SVG spec's 6.4 and 6.5 (per-path exponent, per-record f64 CTM, content-hash path references, `url()` resolver, filter resolution never reduced, Q-SV-5 filed, canvas origin-clean on SV-S3, per-record taint and transparent-black substitution, SV-16 joins GR-14); the bounded animation hand-off record; one tick per frame sampling; per-renderer BeginFrame decimation and the 30 Hz†/Hardened-ceiling rules; compositor snapshot effect limits; flash protection as a motion setting; `hg-interp` owned by the animation spec from AN-M0; G61 gains `hg-anim-time`; G63 restated; compositor track −1† | SVG 0.3 (R7), animation 0.3 | **Applied (graphics 0.4.2, 20 October 2026)** |
| A-LY-5 | Layout spec (R6; animation 6.10, 6.19) | Q-MD-18 answered via SV-D1 (media process links through the scriptless host); forcing-table entries for SVG geometry and `use` shadow-root reads; `SvgGeometry` through `hg-layout-num`; G39 defers to the SVG spec's G122; SVG and `foreignObject` roots one work list (G43); fragment store per style scope; G1 includes `hg-svg-layout`; chunk-to-SVG-record map; view-timeline subject geometry and scroll ranges per layout; stale-timeline re-run as one more loop pass | SVG 0.3 (R6), animation 0.3 | **Applied (layout 0.4.2, 20 October 2026)** |
| A-MD-3 | Images and media spec (R8) | Q-MD-18 answered (6.1); MD-S10 joined with SV-S1; `data:` fonts in opaque SVG images parsed in the media process under TX's fuel; the pull-only opaque-image raster protocol; SV-43 re-raster target; L2m profile for the SVG crates; static-document keys include request and credentials mode; reduce-motion shows the t = 0 sample paused | SVG 0.3 (R8) | **Applied (media 0.4.1, 20 October 2026)** |
| A-DM-1 | DOM spec (editing's two DOM rows) | `hg-input` hands every intent to `hg-edit` (G141); post-connection steps and `[CEReactions]` as one declared script point; `execCommand('insertHTML')` Trusted Types at the sink; `TextEdit` mutation-log record; register entries for spelling/grammar results and interim OS spelling results; `selectionchange` per-target bit; key-event timestamps on the coarsened clock; agent intents never mint activation (DM-D20); autofill door, `hg-forms` traits, script `value` writes, the "last changed by user edit" bit; Clipboard counting rule | Editing 0.3 | **Applied (DOM 0.2.2, 20 October 2026)** |
| A-DM-2 | DOM spec (animation's DOM rows) | AN-D3 amends DM-D10 (one recorded tick per frame, per-document time origin, quantized without jitter; `performance.now()` never below the frame time; dedicated workers derive from the same tick); "update animations" step takes `hg-anim::update`'s change list (`hg-anim-events` removed at AN-M1); view-transition operations owned by `hg-view-transition`; analytic wake-ups request a rendering opportunity; listener-existence summary and the settled signal's animation clause in 6.15; `hg-navigation` gains View Transition eligibility and snapshot re-grant (G94) | Animation 0.3 (AN-D3, 9.5) | **Applied (DOM 0.2.2, 20 October 2026)** |
| A-DM-3 | DOM spec (R4) | Instance building as a core-phase step outside clone; listener dispatch on instances from the source's list; `use` shadow-root reads resolve pending builds; shadow-root mode as one flag; `hg-html-render` (L3) factored out of `hg-html`; SVG rows of 6.12/6.18 point to `hg-svg-dom` and `hg-svg-instances`, SMIL to `hg-anim`; navigation opens local SVG in the scriptless interactive mode and blocks top-level `data:` navigation with no SVG exemption; a planted-bug case in DM-10's suite | SVG 0.3 (R4) | **Applied (DOM 0.2.2, 20 October 2026)** |
| A-JS-11 | JavaScript engine spec (SVG R1–R3; animation `hg-math` row) | Columns, attribute-set references, text arena, untraced rare data and `hg-tree-traits` move to `hg-node-store`; H-2's wording becomes "written only through `StoreWriter<TracedBarrier>`, constructed only in `hg-dom`"; an SVG row in the consolidated decision-point list; degree-native `sin_deg`/`cos_deg`/`tan_deg` exact at multiples of 90° after black-box measurement; correctly rounded binary64 functions, degree variants of `sin`/`cos`/`atan2` and the floating-point control-register assertion at frame start | SVG 0.3 (R1–R3), animation 0.3 | Pending (JS owner, with DM) |
| A-FP-41 | Resource footprint spec (animation) | An "animation state" ledger category with the animation spec's 6.15 budgets, outside FP-11's per-node split; compositor animation-record bytes per renderer key in FP 6.6 and FP-1's compositor line; FP 6.6's "beyond the cap, re-raster" corrected to demotion to the main thread; FP-D15's "display's lowest refresh" replaced by 30 Hz† with user input exempt | Animation 0.3 | Pending (FP owner) |
| A-FP-42 | Resource footprint spec (SVG R11) | SVG render records in FP-11's layout share; path geometry as an excluded category with its own ledger line; a static-document line (SV-51) including its style scope; a per-process cap for scriptless documents (256 MiB†); SV-21 joins FP-36's probes | SVG 0.3 (R11) | Pending (FP owner) |
| A-FP-43 | Resource footprint spec (editing) | A total lexicon line in FP-D13's data set (ED-40); rows for the sanitizer utility and the interim spell utility (keyed by renderer key, exit when idle); the Windows UI host as an always-on process kind with an FP-1 line; undo (per history and 64 MiB† per renderer) and snapshot bounds as class (a) primary state | Editing 0.3 | Pending (FP owner) |
| A-AX-1 | Accessibility spec (animation 6.11; SVG R12; editing row) | `hg-ax-prefs` carries the motion level per site (R, S, X), the Android duration scale and the flash-protection setting; R0 holds in the Hardened profile and only the opt-in uniformity profile reports defaults; `SvgSemanticSource` and the semantic tree for a top-level scriptless document; inline SMIL under reduce-motion like `marquee`; editable-text events from `TextEdit` records; piece-table text leaves; spelling and grammar text attributes; composition events from the IME protocol; caret browsing on `hg-caret`; the UI host also hosts TSF and the AX-M0 measurement includes input methods | Animation 0.3, SVG 0.3 (R12), editing 0.3 | Pending (AX owner) |
| A-NW-13 | Networking spec (SVG R13; editing lists track) | Gzip-magic `image/svg+xml` bodies and local `.svgz` decoded in the partition service under the content-coding budget (1:1,024† and 64 MiB† proposed); the command-text pattern list maintained in the lists track, authored from published prose reports, with the blocking lists' release process | SVG 0.3 (R13), editing 0.3 (ED-D11) | Pending (NW owner) |
| A-DT-13 | DevTools spec (SVG R16; animation domain; editing 6.5) | The `svg.*` domain and overlays, `svg.imageInfo` through the broker; an animation domain (decision records, A1–A8, the recorded-session animation view, CDP Animation domain in the compatibility profile, BiDi `animation` proposals); the IIP `editing` domain with credential placeholders and origin-set filtering; the virtual IME in the automation profile | SVG 0.3, animation 0.3, editing 0.3 | Pending (DevTools owner) |
| A-UI-8 | Shell UI design (SVG R14; animation motion; editing rows) | The SV-D9 bar and "run with script"; play controls for SVG images and inline SMIL under reduce-motion; `title` tooltips; motion settings (level per site, X command and shortcut, rAF throttle offer, flash protection) and the uniformity-profile warning; the Paste popup, autofill preview, UI-10 amendment for in-page-area OS windows, clipboard panel and notices, command-text warning, find options, spelling menu, autofill chooser, caret-blink setting | SVG 0.3, animation 0.3, editing 0.3 | Pending (shell owner; design edits, G-21) |
| A-SEC-26 | Security scope (SVG R17; animation fuzzing; editing threat rows) | Threat rows: the static host as a shared parse across agents of one partition; the opaque-image protocol's channels; the scriptless interactive mode's `file:` directory grant; `hg-anim-fuzz` and the SMIL fuzzer as top-tier targets; threat-model rows for the extended compositor record and the per-key compositor byte caps; the clipboard, drag, sanitizer, interim spell and UI-host input-method rows (no code-integrity relaxation without an owner decision); IPC-R17 suites for clipboard, drag and IME; OS-sandbox denial tests | SVG 0.3, animation 0.3, editing 0.3 | Phase 5 (rebase) |
| A-S1-89 | Stage 1 design (SVG R15) | Crates of the SVG spec's 6.18 in L2, L2p, L3, L4 and tools (`hg-svg-instances` and `hg-html-render` in L3; the four L2p crates); G9's list gains the SVG crates; rules **G120–G129** registered after the PDF spec's G110–G119 | SVG 0.3 (R15) | Pending (Stage 1 owner) |
| A-S1-90 | Stage 1 design (animation rows) | Crates and layers of the animation spec's 6.16; **G130–G139** in the registry; `hg-anim`, `hg-anim-values` and `hg-view-transition` in G9's anchor list | Animation 0.3 | Pending (Stage 1 owner) |
| A-S1-91 | Stage 1 design (editing rows: 13.2 A-S1-75/A-S1-76 restated, 8.1, 8.3) | "Spellcheck through OS services" (Stage 2) and "spellcheck in a utility process" become Izcal's spellchecking in the renderer per language from ED-M2 with the interim tier (Stage 1); ED-M0 to ED-M4 join the milestone table; the editing tracks join 8.3 | Editing 0.3 (ED-D7, ED-D22) | Pending (Stage 1 owner) |
| A-H3-31 | HGBA-3 (animation: DET-1, PRIV-2, B10) | Frame ticks named among the recorded inputs; PRIV-2 rows for level S's discreteness, X's pause states, the rAF throttle and per-renderer cadence; no R0 normalization outside the opt-in uniformity profile | Animation 0.3 | Pending (HGBA-3 editor, wave 3) |
| A-H3-32 | HGBA-3 (editing: B9; B1 and DET-1) | Agent edit intents require the frame's origin in the agent's read-write set; agent intents never mint activation; IIP `editing` records filtered by origin set; the editing recorded inputs join the renderer's list under the one recording budget | Editing 0.3 | Pending (HGBA-3 editor, wave 3) |
| A-CB-1 | Content blocking spec (editing 6.2, 6.3) | The linear-time regular-expression engine moves to the shared L2 crate `hg-regex-linear`, which the compiler and `hg-find` both use | Editing 0.3 | Pending (CB owner) |
| A-PD-1 | PDF spec (editing 6.19) | Field editing through `hg-edit-model`'s text-layout and backing traits; credential mode for PDF password fields; copy from PDF through the viewer renderer to the broker | Editing 0.3 | Pending (PDF owner, G-14) |
| A-AN-1 | Animation spec (its 9.5 SVG row, R10's request side) | Agreed SMIL split recorded on both sides: timing in `hg-anim`, elements/attributes/geometry/`hg-svg-interp` in the SVG spec; the `InterpolateValue` trait, `SmilControl`, paced offsets, image timelines and the composite-position measurement (Q-SV-7, Q-AN-1) before deciding | SVG 0.3 (R10), animation 0.3 | **Agreed and recorded in both specs (20 October 2026); nothing further to apply** |
| A-RN-4 | Reconciliation notes (RN2 Q20, Q25, Section 4) | Owner rows for `AnimTime`, `hg-anim-time`, `hg-interp` and `hg-interp-geom`; the animation total added with the graphics transfer subtracted; Q25's unowned status resolved; C-22 closed | SVG 0.3 (R18), animation 0.3, editing 0.3 | **Applied (reconciliation 2 addendum of 20 October 2026)** |

Not tracked as rows: the editing spec's filings to upstream working groups (Input Events, Selection API, Clipboard API, the Editing WG, CSS, WebDriver BiDi) — outside Izcal's documents — and its shell/DevTools/security/stage rows, which the batch-2 rows above carry.

## Batch 3: the WebCrypto, storage and extensions specs (added 21 October 2026)

Three new specs exist: **WebCrypto** (draft 0.3, WC-D1–D10), **storage** (draft 0.3; 0.3.1 adds the receiver rows WebCrypto and extensions require) and **extensions** (draft 0.3, EX-D1–D16). The owner accepted every recommendation in each decision section on 21 October 2026; **G-18, G-19 and G-20 close**, **Q-NW-14 closes** (WebCrypto owns `hg-webcrypto`; renderers link `hg-crypto`), and the three spec ranges are registered: **G150–G159 storage, G160–G169 WebCrypto, G170–G179 extensions**. The source specs' own 9.5 tables number the applications A-SG-1 to A-SG-13, A-WC-1 to A-WC-21 and A-EX-1 to A-EX-15; the rows below use those IDs, and the target documents' revision lines of 21 October 2026 are the record. The decision record and the recomputed staffing are in `batch3_decisions.md`. **HGBA-2 2.3.4 is a revision of the frozen spec, not the 2.4.0 rewrite**: the batch-3 HGBA-2 rows are applied there with a dated revision line, while the superseded-section rewrites (A-H2S) still land in 2.4.0.

**Storage (A-SG-1 to A-SG-13).**

| ID | Target | Change | State |
|---|---|---|---|
| A-SG-1 | HGBA-2 4.6 and the Section 3 UTILITY SERVICES box | The SQLite sentence and the diagram's "storage (SQLite)" become Izcal's store engine | **Applied (HGBA-2 2.3.4)** |
| A-SG-2 | Stage 1 Section 1, Storage row | The storage service's ownership list names the store engine and every store class; strictest tier | **Applied (Stage 1 0.12)** |
| A-SG-3 | Stage 1 Section 2 | `hg-store-format` and `hg-store-quota` at L2s; L3s service-internals layer; **G150–G159** registered | **Applied (Stage 1 0.12)** |
| A-SG-4 | Stage 1 13.2, Storage row | Storage-key quota, whole-bucket eviction and the eviction decision log | **Applied (Stage 1 0.12)** |
| A-SG-5 | Stage 1 13.2, Everyday features | Imported history and bookmarks persist through the storage service | **Applied (Stage 1 0.12)** |
| A-SG-6 | Networking 6.9 and the architecture diagram | "(SQLite WAL)" becomes "(Izcal's store engine)" in both places | **Applied (networking 0.3.5)** |
| A-SG-7 | DOM 6.18 | The bucket file system row; large blobs = ≥ 256 KiB† | **Applied (DOM 0.2.3)** |
| A-SG-8 | DOM 6.13 | Snapshot staging regions charged to the mapping renderer | **Applied (DOM 0.2.3)** |
| A-SG-9 | Footprint 8.2, FP-1 | The staging-region accounting sentence | **Applied (footprint 0.4.2)** |
| A-SG-10 | Footprint 6.4, FP-11 | Renderer-held sub-threshold blob backing on its own charged-content line | **Applied (footprint 0.4.2)** |
| A-SG-11 | DevTools 3.2 and 6.2 | Storage keys come from `hg-store-observe`; no DevTools-side capture | **Applied (DevTools 0.3.2)** |
| A-SG-12 | Security scope, storage-service threat model | Assets, capabilities and key discipline per SG-D7; the attribution-history asset named | Pending (security-scope merge; 6.3a already indexes the storage service and its `storage-service.md` model) |
| A-SG-13 | HGBA-3, PRIV-2 | A storage-probing case (SG-D5's reversal trigger measures it) | Pending (HGBA-3 3.3.0, wave 3) |

**WebCrypto (A-WC-1 to A-WC-21).**

| ID | Target | Change | State |
|---|---|---|---|
| A-WC-1 | Networking 6.5, NW-D23 | Extended pin set (`ctr`, `cbc`, `aes-kw`, `pbkdf2`; AES-CTR jointly with A-NW-5, landing once); Izcal-written RSA private-key, ECDSA P-384, ECDH P-384 and Ed25519; `expose_bytes(sink)`; the full secret-dependent set | **Applied (networking 0.3.5)** |
| A-WC-2 | Networking 6.21, G36 | `hg-crypto`'s role names the WebCrypto primitives | **Applied (networking 0.3.5)** |
| A-WC-3 | Networking NW-50 | Taint check on x86-64, arm64 and riscv64, per link configuration | **Applied (networking 0.3.5)** |
| A-WC-4 | Networking NW-51 | The throughput bar covers the renderer link of the AEADs | **Applied (networking 0.3.5)** |
| A-WC-5 | Networking NW-52 | Audit scope names `hg-crypto`'s WebCrypto additions and `hg-webcrypto`; registry additions join the re-audit trigger | **Applied (networking 0.3.5)** |
| A-WC-6 | Networking 9.3 | The cryptography track notes 1–2† shared with the WebCrypto track | **Applied (networking 0.3.5)** |
| A-WC-7 | JavaScript engine 6.9 and the heap verifier | `crypto.getRandomValues` draws from `Entropy::crypto()`; the key-custody assertion pass (WC-9) | **Applied (JS 0.4.2)** |
| A-WC-8 | DOM 6.18 | The WebCrypto row points at `hg-webcrypto` with `hg-crypto` behind it | **Applied (DOM 0.2.3)** |
| A-WC-9 | DOM 6.11, DM-D12 | `CryptoKey` clone-wire record kinds; same-agent handle-move fast path; per-sink accepted kinds; history and Navigation sink refusal | **Applied (DOM 0.2.3)** |
| A-WC-10 | DOM 9.3 | The web-API track counts the `SubtleCrypto` bindings | **Applied (DOM 0.2.3)** |
| A-WC-11 | Stage 1 2.1 | `hg-webcrypto` and `hg-webcrypto-observe` at L4; G160–G169; `hg-crypto-check` vector runners | **Applied (Stage 1 0.12)** |
| A-WC-12 | Stage 1 2.2, G9 | `hg-webcrypto` joins the anchor rule | **Applied (Stage 1 0.12)** |
| A-WC-13 | Stage 1 8.1 and 8.3 | WC-M0 to WC-M4 milestones; the web-crypto track, additive to networking's cryptography track | **Applied (Stage 1 0.12)** |
| A-WC-14 | Footprint 8.2 | Renderer that has used WebCrypto ≤ 256 KiB†, 0 when unused | **Applied (footprint 0.4.2)** |
| A-WC-15 | DevTools 6.1 | IIP `crypto` domain; "Web crypto" security section; DT-D2's sensitive-draw class | **Applied (DevTools 0.3.2)** |
| A-WC-16 | DevTools DT-D2, "Never recorded" | Engine-held key material, derived keys and entropy seeds; sealed records are ciphertext | **Applied (DevTools 0.3.2)** |
| A-WC-17 | Security scope NET-R5, SEC-25 | The extended pin set, the `hg-crypto` additions and `hg-webcrypto` in the advisory and audit scope | **Applied (security scope 0.3.1, branch)** |
| A-WC-18 | Security scope REC-R2 | Sealed `CryptoKey` clone-wire records under the at-rest rule; renderer inventory entry points | **Applied (security scope 0.3.1, branch)** |
| A-WC-19 | Storage 6.3 and 6.9 | The sealed-record receiver rule; stored, served, never decrypted, indexed or logged; IndexedDB path feature-gated | **Applied (storage 0.3.1)** |
| A-WC-20 | Coding standard, tier table and rule table | `hg-webcrypto` and `hg-webcrypto-observe` tier P; G160–G169 join the lint set; `tools/tiers.toml` takes the same entries | **Applied (coding standard 0.3.2, branch)** |
| A-WC-21 | `CONTRIBUTING.md`, code review | A-CON-1's carve-out names the WebCrypto additions as inside the same audit and gates | **Applied (branch)** |
| — | DevTools DT-D2 worker-replay clause | Pre-written contingency; not applied until a DevTools owner decision | Pre-written |
| — | Key service process kind (Stage 1, footprint, security scope) | Pre-written contingency on WC-D3's reversal trigger | Pre-written |

**Extensions (A-EX-1 to A-EX-15).**

| ID | Target | Change | State |
|---|---|---|---|
| A-EX-1 | HGBA-2 W-1 and X-3 | The component host (JS-D10) replaces Wasmtime; the rule restated structurally | **Applied (HGBA-2 2.3.4)** |
| A-EX-2 | Stage 1 D-S3, Section 6 and Section 9 Stage 2 row | "via Wasmtime" becomes "via the component host (JS-D10)"; EX-M0's seams verified in Stage 1 integration tests | **Applied (Stage 1 0.12)** |
| A-EX-3 | Stage 1 8.3 | Extensions row: 9–14† + 2–4† non-engineers; EX-M0 counted in existing rows | **Applied (Stage 1 0.12)** |
| A-EX-4 | JavaScript engine 6.0, G6 and G14 | `hg-ext-host` joins the allowed dependents and the embedder list; `hg-ext-host-main` in G14's closure | **Applied (JS 0.4.2)** |
| A-EX-5 | Networking 6.22 | Extensions on the component host; tier-3 as the only read path; `webRequest` as an NW 6.19 client | **Applied (networking 0.3.5)** |
| A-EX-6 | Content blocking 6.15 | Component-host wording; tier-1 list updates on the container route; packages on the uniform grid | **Applied (CB 0.3.2)** |
| A-EX-7 | DevTools 6.1 and Section 6 | IIP `extensions` domain; about:debugging equivalent; DM-D18 cost classes | **Applied (DevTools 0.3.2)** |
| A-EX-8 | Storage 6.3 and 6.8 | The extension-storage mapping and the split-mode key rule | **Applied (storage 0.3.1)** |
| A-EX-9 | HTML and CSS parsing and style 6.1 | `hg-url` registers `ext://<id>/`; one origin = one site key; not network-fetchable | **Applied (style 0.3.6)** |
| A-EX-10 | HGBA-2 I-1 | Extension origins are their own site keys, including in reduced isolation | **Applied (HGBA-2 2.3.4)** |
| A-EX-11 | Security scope | The per-extension-origin component host process kind (eighteen kinds, twenty files), IPC-R17 harness, `hg-ext-package-main` utility parser, native-host registration paths, CON-R extension consents | **Applied (security scope 0.3.1, branch)** |
| A-EX-12 | Shell UI design | Extensions page and slots | **Handed to the shell owner (batch 4)**: the shell spec 6.15 and shell UI design 0.4 accept the handoff and put every Ring A surface, the new-tab/homepage override prompts, the recommendation badge and the command editor in Stage 2 (EX-D15, A-SH-37); **open** until EX-D15 lists those rows |
| A-EX-13 | Footprint 8.2 and FP-1 | Extension host budget, offscreen extension-page renderer, N-extension scenario | **Applied (footprint 0.4.2)** |
| A-EX-14 | Editing 6.16 | The extension client of ED-D21, with an `extension` principal label | **Applied (editing 0.3.1)** |
| A-EX-15 | Accessibility 6.12 | Extension UI in the AT bar; extension-provided names under AX-32; no AT client status | **Applied (accessibility 0.2.2)** |

Not tracked as rows: the extensions spec's WECG position statement (filed upstream, not an Izcal document) and the three source specs' own revision lines.

## Phase 3 completion (21 October 2026)

The wave that closes the ledger ran on 21 October 2026, after all 22 component specs existed with accepted decisions. **HGBA-2 is 2.4.0** (no superseded section remains; the 2.2.x record moved to Appendix B), **HGBA-3 is 3.3.1**, **Stage 1 is 0.14.1** (stale V8/Stylo/WebRender/AccessKit tracks replaced, settled conflicts folded), and the **sweep** applied the remaining receiver rows. Sweep totals over 323 assigned A-rows: **80 applied in the sweep, 232 already present from earlier waves, 11 not applied** (reasons below). Rows the sweep lists as obsolete or superseded are marked accordingly in their row text where it mattered.

**Not applied, by design (the honest residue):**
- **C-11** — "Izcal-only renderers jitless by default" — stated unresolved in HGBA-2 9.2/S-4a/D2 and the security scope; **settled 21 October 2026 as recommended** (HGBA-2 2.4.1: S-4a takes JS's text plus the security scope's CET/BTI and ACG wording; JS-D13's Full-by-default stands, and the security scope re-decides its "jitless by default" rule at its rebase — `residue_decisions.md`).
- **C-19** — the replay host — stated unresolved in HGBA-2 9.2; DevTools owns the surface; **settled 21 October 2026** as the ledger's recommendation: recordings are per process and joined by IDs, and a replay host runs one process kind (HGBA-2 2.4.1, DevTools 0.3.5).
- **C-21** — the PDF pagination contract between layout and `hg-pdf-write` — **settled 21 October 2026**: the PDF spec (accepted, batch 1) is the named owner of writer, viewer and process, and the contract is resolved between the two owners under the Stage 1 13.2 row; G-14 closes with it.
- **G-4** — the JS spec's missing riscv64 row in Stage 1 8.3 — **settled 21 October 2026**: X-1 stays, and riscv64 is covered by the JS, bindings and DOM track per JS-D22, with the 5–7† riscv64 engineers inside the JavaScript engine spec's 65–82†; no separate 8.3 row exists and Stage 1's riscv64 references are no longer conditional.
- **A-EX-12** — open until the Stage 2 extension surfaces exist (EX-D15 says so).
- **A-SH-10/11/12** — **settled 21 October 2026**: AX-D4's mirror is the baseline, AX-D9 confirmed provisional to AX-M0, AX-D11's person-present rule adopted by the security owner (accessibility 0.2.4, security scope 0.3.6). **A-SH-39 — accepted 21 October 2026: the ~74 MiB† Windows ceiling, measured at SH-M0 (shell 0.3.2, FP-1 amended).**
- **A-LA-1/2/3** — **confirmed 21 October 2026 as recommended (licence audit, dated note).**
- **A-CON-2** — conditional on the owner authorising RFC code reading under MD-D4's reversal; untriggered.
- **A-SEC-26** — Phase 5 (security-scope rebase).
- A-SG-13 is **applied** (HGBA-3 3.3.0 carried it, PRIV-2); the sweep's report predates that agent's edit.

**Done elsewhere in this wave:** A-H2S-1..20 and A-H2-1..16 (HGBA-2 2.4.0; A-H2-17..25 deferred or superseded by design); all 30 A-H3 rows plus A-H3-31/32 (HGBA-3 3.3.0); the A-S1 redesign (Stage 1 0.14); the sweep's receiver revisions (style 0.3.7, fonts 0.3.7, networking 0.3.7, media 0.4.3, graphics 0.4.4, JS 0.4.3, accessibility 0.2.4, DevTools 0.3.4, shell UI design 0.5, content blocking 0.3.4, PDF 0.2.4, footprint 0.4.3, reconciliation addenda, skill A-SKILL-1/2, CONTRIBUTING/checks.sh A-CON-3/4, coding standard 0.3.4 A-CS-1..6, security scope 0.3.4 A-SEC-1..24 and A-SG-12); C-8, C-9 and C-10 settled and applied; the 24–36-month Stage 1 placeholder is retired everywhere (C-7's 48–60 months† stands).

## Batch 4: the ML service, shell and phishing protection specs (added 21 October 2026)

Three new specs exist: **ML service** (draft 0.3, ML-D1–D12), **shell** (draft 0.3, SH-D1–SH-D30) and **phishing protection** (draft 0.3, PH-D1–PH-D15). The owner accepted every recommendation in each decision section on 21 October 2026; **PH-D14 is resolved as its recommendation (a)**, recorded in the phishing spec's 9.3 and the licence audit's feed table. **G-15, G-21, G-22 and Q-CB-5 close**; **G180–G189 (ML service), G190–G199 (phishing protection) and G200–G209 (shell) are registered**; the shell takes the extensions spec's shell UI handoff; and the three tracks join C-6 (its batch-4 addendum above). The source specs' 9.5 tables are recorded as **A-ML-1 to A-ML-26**, **A-SH-1 to A-SH-45** and **A-PH-1 to A-PH-39**; the target documents' revision lines of 21 October 2026 are the record, and the security-scope and coding-standard rows land on `spec/security-scope` and `spec/cs-amendments` respectively. The decision record and the recomputed staffing are in `batch4_decisions.md`.

**ML service (A-ML-1 to A-ML-26).**

| ID | Target | Change | State |
|---|---|---|---|
| A-ML-1 | Phase 3 ledger | G-15 closed by the ML spec (ML-D1); A-FUT-3 discharged; the track joins C-6; G180–G189 registered; the A-ML rows record the applications | **Applied (this section; G-15 and C-6 above)** |
| A-ML-2 | HGBA-2 I-1 (OD-8's list) | A **fourth shared-process exception**: the ML service, with an empty capability set, no per-site state, no content-keyed cache, one job at a time, no page-facing request path and a caption session's bounded decoder state; its threat model is in the security scope | **Applied (HGBA-2 2.3.5)** |
| A-ML-3 | HGBA-2 Section 4.5 | Replacement anchor naming ML-D1/ML-D5/ML-D9; the Web-exposure paragraph scopes browser-owned task APIs to Stage 2 under the AI grant; D4 and R-3 point at ML-D9 and ML-18 | **Applied (HGBA-2 2.3.5)** |
| A-ML-4 | HGBA-3 B10 | The accessibility model is a grant with a model set; each model downloads at enable and is deleted unconditionally on disable (B10's keep option remains for AI-grant models); the two grants are independently testable; S_A is defined here | **Applied (HGBA-3 3.2.1)** |
| A-ML-5 | HGBA-3 AG-0 | Model files are a watched category; "zero model files other than inventory entries" becomes "zero model files with both grants off"; a post-opt-in cycle returns to the default state; the service's own contribution is 0.1%† | **Applied (HGBA-3 3.2.1)** |
| A-ML-6 | Stage 1 design Section 1, process map | The ML service row: launched only with the accessibility grant (or, Stage 2, the AI switch), exits when idle; owns the runtime, models and task front ends | **Applied (Stage 1 0.13)** |
| A-ML-7 | Stage 1 design 2.1, build line | `hg-ml-main` built always; generative task modules only with feature `ai` | **Applied (Stage 1 0.13)** |
| A-ML-8 | Stage 1 design 2.1, G4 | Named exception: `hg-ml-main` may depend on `hg-ml*`; nothing else may | **Applied (Stage 1 0.13)** |
| A-ML-9 | Stage 1 design 2.1 crate table and 2.2 G9 | `hg-ml`, `hg-ml-loader`, `hg-ml-service`, `hg-ml-tasks`, `hg-ml-observe` and `hg-ml-api` join the table; `hg-ml*` join G9's anchor list; `hg-ml-main` joins L6 | **Applied (Stage 1 0.13)** |
| A-ML-10 | Stage 1 design Section 6, accessibility model opt-in row | Scope change to shell settings and repair-overlay integration only; the service's 2–3† move to the ML spec's 9.3 and are recorded as not counted twice | **Applied (Stage 1 0.13)** |
| A-ML-11 | Stage 1 design 8.1 and 8.3 | ML-M0–ML-M5 join the milestone table; the ML track (14–20.5† + 0.5–1†) joins 8.3 additive to the media, accessibility and PDF tracks | **Applied (Stage 1 0.13)** |
| A-ML-12 | Images and media 6.12 | Generated captions and image descriptions are behind the accessibility model grant (ML-D4), off by default, on device, marked generated, never exposed to the page | **Applied (images and media 0.4.2)** |
| A-ML-13 | Images and media 9.5 R19 | Replaced: ML-IMG ≤ 200 MB†, ML-OCR ≤ 80 MB† per script, ML-ASR ≤ 150 MB†; the media spec counts no ML staffing | **Applied (images and media 0.4.2; A-FUT-3 discharged)** |
| A-ML-14 | Images and media 9.3 | The double-counting note names the ML spec's 9.3 rows for all three models | **Applied (images and media 0.4.2)** |
| A-ML-15 | PDF OCR paragraph and PD-D11 | ML-OCR gates: raster from the content GPU process, confidence and reading order, labelled recognised, accessibility grant only; OCR starts on a user action or an AT read, never on document open (ML-12); the model is the ML service's | **Applied (PDF 0.2.2)** |
| A-ML-16 | Accessibility R4/Q-10 and 9.3 | Generated-label input is `generated: true` plus model id; R4 rows cite ML-IMG; captions record the grant-state signal (Q-ML-14) and its ML-M3 measurement | **Applied (accessibility 0.2.3)** |
| A-ML-17 | Editing ED-D9 | The grammar model is Stage 2 under the AI grant, the `ai` feature and the person's choice; Stage 1 has no model | **Applied (editing 0.3.2)** |
| A-ML-18 | Animation AN-D12 | No text change; the note records that no ML crate is reachable from `hg-anim-observe` (G186) | **Applied (animation 0.3.1; no text change)** |
| A-ML-19 | DOM DM-D18 | The observability classes gain the `ml` domain's rows; no class changes | **Applied (DOM 0.2.4)** |
| A-ML-20 | DevTools 6.1, DT-D2 and DT-D6 | IIP gains the experimental `ml` domain served by `hg-ml-observe`; recordings gain `ml.job` values with raw inputs excluded; the ML panel and model inventory join the Stage 1 frontend | **Applied (DevTools 0.3.3)** |
| A-ML-21 | Networking NW-D17 and NW-36, and the Stage 1 update client | Model files are update-channel payloads verified by a second chain (the model-signing role, keys separate from the release key); the client gains the model-manifest hook and fails closed; if it cannot carry a second chain by ML-M3, models ship with the release | **Applied (networking 0.3.6)** |
| A-ML-22 | Resource footprint 8.2, FP-25 and FP-D13 | One conditional row: service ≤ 4 MiB† base, activations ≤ 192 MiB† per task, weights clean and shared; the installer's ≤ 4 MiB† compressed allocation inside FP-25; user-requested downloads; FP-22 gains the per-task energy baseline | **Applied (resource footprint 0.4.2)** |
| A-ML-23 | Security scope | ML service in the process inventory as a trusted, sandboxed, network-less process; `hg-ml-loader` in the tier-P parser inventory; per-OS denial tests; recordings are page-content recordings; data ledger, model-signing chain and build provenance in the supply chain; the training-data leakage check; crash-report exclusions | **Applied (security scope 0.3.2, branch)** |
| A-ML-24 | Licence audit | **Class 6**: model weights and training data, D0–D5; shipped weights MIT; D2 only under Q-ML-9; a third-party MIT weight needs Q-ML-11; model cards and licence files are read as data; `memmap2` and `bytemuck` are the only generic crates | **Applied (licence audit, 21 October 2026)** |
| A-ML-25 | Coding standard, tier table and `tools/tiers.toml` | `hg-ml`, `hg-ml-loader`, `hg-ml-service`, `hg-ml-tasks` and `hg-ml-observe` tier P; G180–G189 join the lint set | **Applied (coding standard 0.3.3, branch)** |
| A-ML-26 | `CONTRIBUTING.md`, code review | A model change is an audit event: the data-ledger digest and model card are in the review, and a weight file without a ledger row is rejected at the build | **Applied (cs-amendments worktree)** |
| — | Security scope, AG-0 and the ML spec | Pre-written contingency: a security classifier inside the signed package, under 50 MB, embedded through `hg-ml`'s loader with no service, with an inventory entry, its own G4 exception and amendments to G182/G187 | **Pre-written; not applied** |

**Shell (A-SH-1 to A-SH-45).**

| ID | Target | Change | State |
|---|---|---|---|
| A-SH-1 | Shell UI design R1–R21 | Each design decision mapped: adopted, amended (SH-D22's five actions, SH-D17's Stage 2 notifications, SH-D2/6.5's separate owned prompt windows, 6.6's neutral site-controls button), or left a design detail; each becomes a spec decision or target | **Applied (shell UI design 0.4)** |
| A-SH-2 | Shell UI design §4.3, §4.4, §5 UI-1–UI-11, §6, §7 | Site-panel language amended; UI-1–UI-11 map to SH-2–SH-13 with the stricter target governing; staging amended by 9.1; Q-UI-2 resolved, Q-UI-1 open to SH-M4, Q-UI-3 deferred, Q-UI-4 unchanged | **Applied (shell UI design 0.4)** |
| A-SH-3 | Security scope CON-R3 | Confirmed and clarified: the broker creates and owns every prompt window on every platform and receives its input directly; on Windows the UI host owns content windows only and hosts TSF/IME and UIA/IA2; the earlier reading is withdrawn | **Applied (security scope 0.3.2, branch)** |
| A-SH-4 | Security scope CON-R1, CON-R2 and CON-R5 | Confirmed: hit-test against the broker-owned prompt template geometry, abort on divergence, software raster inside the prompt renderer, options (a)+(b) with (c) fallback unchanged | **Applied (security scope 0.3.2, branch)** |
| A-SH-5 | Security scope GR-D28 | Tightened: the shell content GPU must never rasterise prompt pixels; the device-less prompt-tile path is not used | **Applied (security scope 0.3.2, branch)** |
| A-SH-6 | Security scope SH-R1–SH-R9 | No text change; implementation mapped to 6.1, 6.4, 6.5, 6.6 and 6.18 and to SH-7, SH-11, SH-14 and SH-16 | **Applied (security scope 0.3.2, branch; no text change)** |
| A-SH-7 | Security scope process kinds | The recovery window joins as a native process kind with no engine, no page data and no network (twenty kinds, twenty-two threat-model files) | **Applied (security scope 0.3.2, branch)** |
| A-SH-8 | Security scope prompt renderer | Confirmed jitless, project JS only, no page strings except the broker-formatted origin, one process per prompt, and **expiry per class**: low-risk 30 s† with no decision, high-risk until decided or the window closes; the fixed 30 s statement is removed | **Applied (security scope 0.3.2, branch)** |
| A-SH-9 | Security scope CON-R4 | No change; 6.3 and 6.18 restate it and `shell.password` returns usernames only | **Applied (security scope 0.3.2, branch; no text change)** |
| A-SH-10 | Accessibility AX-D3 / AX-D4 | **Settled 21 October 2026.** Baseline is AX-D4's validated mirror in the broker, read by the UI host through a read-only client; the mirror client is the only seam if AX-M0 chooses otherwise (accessibility 0.2.4) | **Applied** |
| A-SH-11 | Accessibility AX-D9 | **Confirmed provisional 21 October 2026.** ED-D5(b) settles TSF, not AT; the UI host's AT adapter/mirror placement and its FP row wait on AX-M0 (accessibility 0.2.4) | **Applied (provisional stands)** |
| A-SH-12 | Accessibility AX-D11 | **Settled 21 October 2026: the security owner adopted it** — high-risk prompt classes need person-present evidence for an AT-invoked approval; until implemented, high-risk AT approval is not accepted and the AT action moves focus only (accessibility 0.2.4, security scope 0.3.6) | **Applied** |
| A-SH-13 | Accessibility AX-D12 | The recovery window and the shell use AX-D12's sandboxed AT-SPI bridge process (b) on Linux, including when the engine is down | **Applied (accessibility 0.2.3)** |
| A-SH-14 | Accessibility diagnostics | The shell's `about:diagnostics` includes the AT bridge state and the last 20 accessibility events; no page can read it | **Applied (accessibility 0.2.3)** |
| A-SH-15 | Editing ED-D5(b) | TSF and third-party IME DLLs run in the UI host on Windows; the UI host is an always-on Windows process kind with its own FP row | **Applied (editing 0.3.2)** |
| A-SH-16 | Editing ED-D10 | The Paste button and confirmation are drawn by the prompt renderer; the 5 s† dismissal, 1 s† enable delay and OS change counter are unchanged, the timer suspends on focus or hover and can be extended with Enter or Space | **Applied (editing 0.3.2)** |
| A-SH-17 | Editing ED-D6 | The UI host's TSF text store exposes only the focused, non-credential editable's ±2,048 UTF-16 window; never hidden tabs, never credential fields | **Applied (editing 0.3.2)** |
| A-SH-18 | Editing ED-D8 | OS spellcheck through system services is Stage 2; the shell's Language and writing group stays hidden until it ships, then names the OS service | **Applied (editing 0.3.2)** |
| A-SH-19 | Editing ED-D17 | The find bar is shell UI; the broker coordinates matching; the shell draws counts and navigation | **Applied (editing 0.3.2)** |
| A-SH-20 | Editing clipboard rules | No change; the shell's Paste UI is a client of the broker's clipboard endpoint | **Applied (editing 0.3.2; no text change)** |
| A-SH-21 | Storage Q-SG-6 | **Closed** by 6.7: the shell owns the history and bookmark schema, index definitions, migrations and retention; the storage service holds and encrypts the data | **Applied (storage 0.3.2)** |
| A-SH-22 | Storage SG-D11 | Typed catalogue interface (`range`, `prefix`, `postings`, `intersect`, `get`, `put`, `delete`, `count`) with typed keys and bounded results; no raw page access, no shell-side file | **Applied (storage 0.3.2)** |
| A-SH-23 | Storage SG-D5 | The shell shows the pre-eviction notice and the persist prompt class; eviction stays whole-bucket and deterministic | **Applied (storage 0.3.2)** |
| A-SH-24 | Storage SG-D6 | The `persist()` prompt is class `site-data-persist`, per (site, container), with the standard timing and expiry | **Applied (storage 0.3.2)** |
| A-SH-25 | Storage SG-D7 | **Documented exception**: prompt-ledger entries stay byte-encrypted under their container keys while a profile-scoped encrypted id/time index supports cross-container listing; clearing a container deletes its rows and index entries | **Applied (storage 0.3.2)** |
| A-SH-26 | Storage FP-D18 retention | History defaults: 90 days† of full visits and 12 months† of the aggregate record (container, registrable site, day, count, last visit); user-configurable, container-clearable, private windows never stored | **Applied (storage 0.3.2)** |
| A-SH-27 | Storage FP-37 | No change; 6.7's queries return rows and cursors, never a full-store scan to the shell | **Applied (storage 0.3.2; no text change)** |
| A-SH-28 | Networking NW-D24 | The site panel carries the trust-store row while the OS store is in use; `about:build` records the mode | **Applied (networking 0.3.6)** |
| A-SH-29 | Networking 6.18 error pages | New decision **NW-D27**: error pages are shell documents with broker-formatted codes and the typed `FailureReason`, no page-supplied error text; not NW-53 | **Applied (networking 0.3.6; NW-D27)** |
| A-SH-30 | Content blocking CB-D19 | The narrow-first site controls live in the site panel's Blocking section, with the same defaults and the same one-click revert | **Applied (content blocking 0.3.3)** |
| A-SH-31 | Content blocking CB-D20 | SH-10's ≥ 80%† comprehension and ≤ 5%† false-alarm targets cover the warning and blocking surfaces; CB's decision log supplies the report-flow evidence | **Applied (content blocking 0.3.3)** |
| A-SH-32 | Content blocking reports | The report flow is entered from the panel and the prompt layer; the shell shows the recognizability label and never silently uncloaks | **Applied (content blocking 0.3.3)** |
| A-SH-33 | Animation AN-D10 | The shell owns the motion settings UI (R/S/X), maps the OS preference to R, offers "stop motion now" in the panel and by keyboard; shell transitions ≤ 150 ms† | **Applied (animation 0.3.1)** |
| A-SH-34 | DevTools DT-D2 | An explicitly started, navigation-scoped recording may include the shell's UI state and typed event stream, joined by session id; the shell is not a second recording engine; a content-free diagnostic ring runs from process launch | **Applied (DevTools 0.3.3)** |
| A-SH-35 | DevTools DT-D10 Option B | The shell contributes the `shell.*` IIP domains, a shell inspector and the UI-state recorded unit; the 3 engineer-months are an internal allocation in T4 and T10, not added to the program total | **Applied (DevTools 0.3.3)** |
| A-SH-36 | Extensions EX-D5 | Extension install and growth grants are requested by the extension, shown in the prompt layer and approved only by the broker; the shell shows a persistent pending marker | **Applied (extensions 0.4)** |
| A-SH-37 | Extensions EX-D15 | Ring A surfaces (toolbar action, popup host, options page, badge, per-site grant row) and the new-tab/homepage override prompts, recommendation badge and command editor are all Stage 2; the handoff is accepted; **A-EX-12 stays open** until EX-D15 lists the rows | **Applied (extensions 0.4); A-EX-12 open** |
| A-SH-38 | PDF PD-D2 | The viewer is an HTML app in the PDF key's renderer, not shell UI; the shell owns the download handoff and "open with another application"; the document-password prompt is the shell's `document-password` class (6.4), broker-owned, high risk, the secret goes straight to the document process and is never recorded | **Applied (PDF 0.2.3)** |
| A-SH-39 | Resource footprint FP-1 | Three process rows: prompt renderer ≤ 8 MiB† on demand, UI host ≤ 10 MiB† always-on on Windows as its base, recovery window ≤ 6 MiB† on demand; the Windows always-on sum would be ~74 MiB† against the ≤ 64 MiB† ceiling, so SH-25 is not claimed on Windows until the owner accepts the measured trade-off or a reworked UI host fits; the IME/AT attributed line moves to the UI host | **Accepted 21 October 2026** (the residue decisions): the ~74 MiB† Windows always-on ceiling, measured at SH-M0 with the stated trade-off; FP-1 carries the dated amendment |
| A-SH-40 | Resource footprint 8.2 | The shell content GPU row stays ≤ 2 MiB† and device-less and is never on the prompt pixel path; the prompt renderer rasterises in software | **Applied (resource footprint 0.4.2)** |
| A-SH-41 | Stage 1 D-S3 and 13.2 | Confirmed, not changed: extensions are None in Stage 1; containers are 1 key design + 2 UI plus the read-only chip; autofill beyond passwords and OS spellcheck are Stage 2; notification delivery and quiet UI are Stage 2 | **Applied (Stage 1 0.13)** |
| A-SH-42 | Stage 1 Section 1 and Section 6 | The shell's Stage 1 scope as in 9.1; the UI host, prompt renderer and recovery window join the process map and feature table; the notification row reads "permission prompt and quiet UI in Stage 2; Stage 1 denies honestly"; the document-password class is added | **Applied (Stage 1 0.13)** |
| A-SH-43 | Stage 1 Section 8.3 | The shell row's 1–2 becomes **6–8, 168–243 engineer-months†, peak 10–12†**, plus 1.5–2.5 design/research FTE; the program minimum rises to 40.5–56 engineers† | **Applied (Stage 1 0.13)** |
| A-SH-44 | Stage 1 M1 (gate before M2) | SH-M0 platform measurements: prompt window ordering and occlusion on four window systems, OS input provenance and synthesized-input detection, input-to-paint baseline, jitless UI on three ISAs, UI-host AT/TSF latency; M0 is not reopened and missing measurements are recorded | **Applied (Stage 1 0.13)** |
| A-SH-45 | Stage 1 M2–M8 | SH-M1–SH-M7 exit criteria join the corresponding milestones; M8 is SH-M7, Stage 1 exit | **Applied (Stage 1 0.13)** |

**Phishing protection (A-PH-1 to A-PH-39).**

| ID | Target | Change | State |
|---|---|---|---|
| A-PH-1 | Security scope SEC-D7 | Marks the client, protocol and privacy analysis this spec specifies; the source list stays as chosen under PH-D14; matching distinguishes a URL-feed entry (never widened to a host suffix) from a domain feed expressed as a host suffix | **Applied (security scope 0.3.2, branch)** |
| A-PH-2 | Security scope Section 8 and SEC-29 | The phishing target points at PH-1–PH-22; SEC-29's surface inventory gains the interstitial and its "0 page-derived strings" rule gains the broker-formatted URL and Izcal evidence exception | **Applied (security scope 0.3.2, branch)** |
| A-PH-3 | Security scope CON-R2 | The prompt renderer may display the broker-formatted target URL and Izcal-authored list evidence, bounded and as text; no page-derived markup or strings | **Applied (security scope 0.3.2, branch)** |
| A-PH-4 | Security scope SH-R6, SH-R7 and 6.14 | Download reputation is this spec's URL check (final URL and every redirect hop) plus SH-R7's type rules; the malware class is a documented gap; `file:` and deeplink prompts show the verdict | **Applied (security scope 0.3.2, branch)** |
| A-PH-5 | Security scope SH-R6 | The OS origin-mark rule extends from downloads to every file the browser writes, including a saved page; the broker sets the mark itself | **Applied (security scope 0.3.2, branch)** |
| A-PH-6 | Security scope UPD-R1 and SEC-17 | The bundle is a delegated targets role in the shared TUF repository with a 15-minute† manifest poll, daily full sync and seed; an emergency entry is a new bundle version, not UPD-R5; SEC-17's 3-day† expiry is a ceiling and a per-channel shorter expiry is allowed | **Applied (security scope 0.3.2, branch)** |
| A-PH-7 | Content blocking CB-D23 and Q-CB-5 | Q-CB-5 answered: phishing and malware URL reputation are this spec's system, not a seventh content-blocking list; the Security list stays crawl-observed; the lists share no compiled file or match | **Applied (content blocking 0.3.3)** |
| A-PH-8 | Content blocking 6.8 crawler | The crawler records the page URL and referring URL of each visit under its existing retention, so the top-10k allowlist ranking can be computed from Izcal's own observations; Tranco's rank never ships | **Applied (content blocking 0.3.3)** |
| A-PH-9 | Content blocking CB-D16 and CB-21 | A named, published **source-level governance exception**: sources are vetted and re-vetted, feed entries auto-accepted with a 2.5%† sampling audit, and two-person review kept for allowlist, human, exception and top-10k entries; PH-D9 accepts or rejects it | **Applied (content blocking 0.3.3)** |
| A-PH-10 | Content blocking 6.9, 6.12, 6.16 and 9.3 | A second TUF target on the same update client and witness log; the per-site override does not apply (a phishing continue is per navigation); the shared lists staff adds 1.5–3† non-engineers, not a second governance team | **Applied (content blocking 0.3.3)** |
| A-PH-11 | Content blocking CB-D16 and 6.8 | Phishing entries use source-driven expiry (the feed's retention plus a 7-day† absence rule), not CB-D16's 12-week revalidation | **Applied (content blocking 0.3.3)** |
| A-PH-12 | Content blocking CB-D17 and 6.9 | The diff is Izcal's version-addressed additions/removals format with a whole-list SHA-256 and a full-sync fallback; the TUF signer joins CB-D17's automated signer and on-call, timestamp/snapshot re-signed at most 96 times a day; CB-D17's UPD-R5 sentence is amended | **Applied (content blocking 0.3.3)** |
| A-PH-13 | Content blocking 6.13 and the log schema | A `Reputation` intervention kind plus the check-result fields (`check_id`, canonical URL, surface, result, bundle hash, entry, decision, `caused_by`) join NW 6.18's decision log; the page sees nothing | **Applied (content blocking 0.3.3)** |
| A-PH-14 | Content blocking 6.14 and CB-D21 | The bundle version/hash, check result and surface are recorded B1 inputs; replay reproduces the block and can run with and without the entry | **Applied (content blocking 0.3.3)** |
| A-PH-15 | Content blocking 6.5 and CB-D8 | A debounced redirector's destination is checked before the navigation proceeds; the record shows the redirector was skipped | **Applied (content blocking 0.3.3)** |
| A-PH-16 | Content blocking CB-22 | The phishing bundle is a target in the same witness-cosigned transparency log; no new witness operator | **Applied (content blocking 0.3.3)** |
| A-PH-17 | Networking NW-D17 and 6.13 | The manifest, diff and bundle join the published endpoint list (public, plain fetch, 15-minute poll, daily full sync); no per-user query; the report stays OHTTP; no prefix mode in Stage 1 | **Applied (networking 0.3.6)** |
| A-PH-18 | Networking 6.2, G31 and NW-18 | `ReputationCanonicalUrl` is one function over `hg-url`, shared by the build and the client, with a golden corpus and one index hash | **Applied (networking 0.3.6)** |
| A-PH-19 | Networking 6.7 and `FailureReason` | URL reputation joins the per-hop checks before socket creation, with a `reputation` failure layer carrying the structured payload; the page sees a generic failure | **Applied (networking 0.3.6)** |
| A-PH-20 | Networking 6.14 | The download path checks the initial URL before connecting and each redirect target before the next request; a malware match writes nothing; the record carries the verdict and check time | **Applied (networking 0.3.6)** |
| A-PH-21 | Networking 6.18 | The "why did this not load" page gains a reputation case with entry, source, date and contest action; the site panel gains list age and last check | **Applied (networking 0.3.6)** |
| A-PH-22 | Networking 6.19 | Phishing checks run before any agent or automation interception and cannot be mocked, modified or observe-then-allowed by an automation principal | **Applied (networking 0.3.6)** |
| A-PH-23 | Networking NW-D26 and 6.15 | The phishing list is Izcal's own, separate from content blocking's lists, not user-addable, not a filter-list format; no user or extension allow entry overrides it | **Applied (networking 0.3.6)** |
| A-PH-24 | Graphics GR-D28 and GR-19 | The full-page interstitial is a prompt-renderer surface in the broker-placed OS layer; GR-D28's property is normative (broker-placed, unoccludable, broker-hit-tested); the shell's owned window is one conforming implementation; GR-19's suite tests the property | **Applied (graphics 0.4.3)** |
| A-PH-25 | Shell UI design 4.3, 4.5, UI-9 and UI-10 | The site panel gains the Phishing protection row; 4.5's interstitials are this spec's surfaces; UI-9's study is PH-11; UI-10's trusted-region property covers the interstitial; the download warning is two-level | **Applied (shell UI design 0.4)** |
| A-PH-26 | Shell 6.6, address bar, downloads and SH-D11 | The site panel gains state, bundle age, last verdict, report link, no off switch, the `no-bundle` state and the "You chose to visit a listed site — Leave" state; paste-and-go checks before the request; the download warning is the prompt layer's two-level surface; SH-D11's ledger carries a durable summary linked by `check_id` | **Applied (shell 0.3)** |
| A-PH-27 | Shell UI design and shell, password manager and form fill, SH-10 and SH-11 | The password manager consults the check record and does not autofill on a `listed` verdict or a `no-bundle` state without a fresh user action; the lookalike detector stays Q-PH-16 (the shell), not a reputation entry | **Applied (shell 0.3; shell UI design 0.4)** |
| A-PH-28 | Editing ED-D11 and A-NW-13 | ED-D11 owns command-text warnings and the pattern list; this spec owns URL checks and paste-and-go; the lists track maintains both; no duplicate warning on one event | **Applied (editing 0.3.2)** |
| A-PH-29 | Extensions EX-D7 and 6.7 | An extension-initiated navigation is checked; no API exposes, suppresses or alters a verdict; an extension cannot add an allowlist or removal; `ext://` pages are labelled and governed by EX-D10, with embedded remote content checked as a subframe | **Applied (extensions 0.4)** |
| A-PH-30 | DevTools 6.1–6.2, DT-D1 and DT-D5 | A `Reputation` IIP domain or "Site reputation" section with the check-record schema; the Network view shows a policy decision, not a fake request | **Applied (DevTools 0.3.3)** |
| A-PH-31 | DevTools 6.1 and DT-D2 | A pre-renderer decision record is recordable and replayable; a navigation blocked before a renderer exists is not a gap; replay can ask "would this have loaded without the entry?" | **Applied (DevTools 0.3.3)** |
| A-PH-32 | DevTools 6.5 rule 9 | Reputation verdicts are audit events in the agent's session, queryable within its granted origins; the agent cannot read another session's | **Applied (DevTools 0.3.3)** |
| A-PH-33 | Storage 6.3, SG-D8 and FP-D18 | A bounded local check-record log: 24 h† of detail plus the prompt-ledger summary, cleared with history, memory-only in private windows, deleted with the container, never sent | **Applied (storage 0.3.2)** |
| A-PH-34 | Resource footprint FP-28 and 8.2 | The phishing matcher line: mapped bundle ≤ 4 MiB†, charged state ≤ 128 KiB† per partition service, ≤ 12 MiB† clean and shared with content blocking, p99 ≤ 100 µs† once per navigation on x86-64, AArch64 and riscv64; the 96† polls a day are in the update channel's budget | **Applied (resource footprint 0.4.2)** |
| A-PH-35 | HGBA-2 2.3.2 | The MIT-only rule covers list data, not only code; **no existing row (OD-L2, OD-L5, OD-L6) authorises shipping the feeds**; PH-D14's resolution is (a), recorded with (b) and (c) not assumed | **Applied (HGBA-2 2.3.5)** |
| A-PH-36 | Stage 1 Section 1, Section 2 and 13.2 | No new process: the check runs in the partition service, the broker's `hg-navigation`, the download manager and the prompt renderer; new crates `hg-phish`, `hg-phish-match`, `hg-phish-format`, `hg-phish-observe` and server-side `hg-phish-ingest`, `hg-phish-compile`; the 13.2 row and 8.3 track | **Applied (Stage 1 0.13)** |
| A-PH-37 | HGBA-3 B9, B10 and PRIV-2 | Agent tasks require the refusal path; B10's classifier carve-out is not used; PRIV-2 gains the "no URL-dependent request before the verdict" probe | **Applied (HGBA-3 3.2.1)** |
| A-PH-38 | Licence audit | Rows for CERT Polska, Phishunt, URLAbuse, Phishing.Database, PhishTank, URLhaus and Spamhaus, APWG eCX and Netcraft, each MIT-compatible, test-only or exception-only, with terms, the MIT-redistribution test and attribution recorded; the bundle carries expressions and provenance only and ships a NOTICE / LICENSE-DATA file; **no existing row authorises the feeds** | **Applied (licence audit, 21 October 2026)** |
| A-PH-39 | Phase 3 ledger | G-22 closed by this spec's milestones and staffing; A-FUT-7 applied; A-UI-7's "phishing interstitials depend on SEC-D7" becomes "depend on this spec" | **Applied (this section; G-22 and A-FUT-7 above)** |

Not tracked as rows: the three source specs' own revision lines and the phishing spec's open questions Q-PH-1 to Q-PH-16, except Q-PH-7 (answered by PH-D14 (a)) and Q-PH-16 (the lookalike detector, assigned to the shell). Open owner decisions stay recorded: none — **Q-SH-3's retention defaults were decided on 21 October 2026** (90 days† full visits, 12 months† aggregate, user-configurable; shell 0.3.6), and the accessibility rows were settled the same day (above). C-6R/Q-AE-1 and FP-1's Windows ceiling (A-SH-39) were settled on 21 October 2026 (residue_decisions.md).

## Batch 5: the devices spec (added 21 October 2026)

The corpus's 23rd spec exists: **devices** (draft 0.3, DV-D1–DV-D15). The owner accepted every recommendation in its Section 9.4 on 21 October 2026; **the six proposed owner exceptions in its 6.15 (DV-E1 to DV-E6) are not granted** and stay proposed, each with its 12-month† expiry after grant and its re-decision at DV-M5. **G-26 closes**; **G210–G219 are registered**; the device track joins C-6 (its batch-5 addendum above) and Stage 1 8.3's total becomes **67.5–96.5† engineers** (8–12† net). The source spec's 9.5 tables are recorded as **A-DV-1 to A-DV-15** below; the target documents' revision lines of 21 October 2026 are the record, and the security-scope, coding-standard and DOM/accessibility rows land on `spec/security-scope`, `spec/cs-amendments` and `spec/phase2c` respectively. The decision record and the recomputed staffing are in `devices_decisions.md`.

| ID | Target | Change | State |
|---|---|---|---|
| A-DV-1 | Devices spec 0.3 | The owner stamp (21 October 2026), Revision 0.3, the C-6 correction in 9.3/9.5.11/F2, the applied 9.5 note, the C10 disposition, the relates-to versions and the F2/C10 ledger text | **Applied (devices 0.3)** |
| A-DV-2 | Security scope 0.3.7 (`spec/security-scope`) | 37 process kinds / 39 threat-model files; the `hg-device-main` row and 6.3a index; ATK-10 and TB-14 with the matrix; CON-R6–R9; `DeviceHandle` and the IPC-R1 ring caps; the device sandbox row; the `device-access.md` model; REC-R7; SEC-24 parser targets and the 14†-day condition; SEC-26 and M1 counts; Section 3.5's G-26 closure; Appendix B ID changes | **Applied (security scope 0.3.7, branch)** |
| A-DV-3 | Stage 1 design 0.14.3 | 35 kinds / 36 rows; Section 1 device row; 2.1 crate table and 2.2 G9/registry (G210–G219); 4.1 I-2 note; Section 6 feature row; 8.1 DV-M0–DV-M5; 8.3 total 67.5–96.5† and the Devices track row (9–13† gross, 8–12† net); 13.2 rows | **Applied (Stage 1 0.14.3)** |
| A-DV-4 | Shell 0.3.4 | 6.4 prompt classes 11 → 14 (`device-pairing`, `device-connection`, `device-write`, all high-risk), the queue and the ledger; 6.6 site-panel Devices section and device indicator; 6.9 Devices settings; SH-35 | **Applied (shell 0.3.4)** |
| A-DV-5 | DevTools spec 0.3.6 | The `devices` IIP domain; DT-D2 device recording classes, sensitive/never table and redaction; DT-1 replay claims; DT-13 agent scoping; DT-15 cost classes; DT-16 virtual-device label; 6.5 export redaction; the 6.13 command and event set | **Applied (DevTools 0.3.6)** |
| A-DV-6 | Resource footprint 0.4.4 | The eight 8.2 device rows; FP-1 lazy-service note and the Windows always-on ceiling note; FP-22 device energy; FP-D8 and FP-D13; FP-25 installer | **Applied (resource footprint 0.4.4)** |
| A-DV-7 | Coding standard 0.3.6 (`spec/cs-amendments`) | Tier rows (13 device crates tier P, `hg-device-api` tier R, the tools tier T, the `hg-substrate-os` device modules tier O); G210–G219 registered and G9's anchors; CS-ER-19, CS-TY-12 and CS-TY-13 coverage; G218 log redaction; the 66 rules unchanged | **Applied (coding standard 0.3.6, branch)** |
| A-DV-8 | DOM 0.2.5 | 6.18's six device rows (11 → 17); G-26 closed; the bindings inventory and the `[SecureContext]`/`[Exposed=Window]` policy token, with no worker exposure | **Applied (DOM 0.2.5)** |
| A-DV-9 | Extensions 0.4.1 | Ring C device namespaces; the separate refusal of the six device web APIs to extension pages, content scripts and workers; EX-D5's closed-list rule | **Applied (extensions 0.4.1)** |
| A-DV-10 | Accessibility 0.2.5 | The AX-D11 row (three device classes high-risk, AT approval refused until person-present evidence, the OS-attested path, the keyboard-operable chooser) and 6.9's chooser requirements | **Applied (accessibility 0.2.5)** |
| A-DV-11 | HGBA-2 2.4.2 | The I-2 device note (broker-side service; only the arbiter calls the OS; no renderer, content GPU, media, network or session opens a device) and the HAL note (device backends are `hg-substrate-os` OS-interface adapters, not an engine HAL class) | **Applied (HGBA-2 2.4.2)** |
| A-DV-12 | HGBA-3 3.3.2 | B9's `device.request` typed intent; PRIV-2's device linkability probe (DV-6); DET-1's device input as a recorded class at the device-service boundary | **Applied (HGBA-3 3.3.2)** |
| A-DV-13 | `README.md` | The devices spec row (draft 0.3) and the bumped revisions | **Applied (README, 21 October 2026)** |
| A-DV-14 | Phase 3 ledger | G-26 closed; the C-6 batch-5 addendum; G210–G219 registered; the A-DV rows record the applications | **Applied (this section; G-26, C-6 and the registry above)** |
| A-DV-15 | `devices_decisions.md` | The acceptance, the C-6 correction, the application map and the items left open (DV-E1–E6; Q-DV-1, -2, -3, -8, -9, -15, -16 and -17) | **Applied (this folder)** |

Not tracked as rows: the devices spec's own open questions, except where a decision answered one; **Q-DV-1, Q-DV-2, Q-DV-3, Q-DV-8, Q-DV-9, Q-DV-15, Q-DV-16 and Q-DV-17 stay open** with the owners and needed-by dates in the spec's Section 11. The eight **owner-facing** decisions are flagged for the owner in the decision record: **DV-D3, DV-D4, DV-D5, DV-D8, DV-D9, DV-D12, DV-D13 and DV-D15**.

## Batch 6: the app host spec (added 21 October 2026)

The corpus's 24th spec exists: **app host** (draft 0.3, AH-D1–AH-D17). The owner accepted every recommendation in its Section 9.4 on 21 October 2026; the decision record is `app_host_decisions.md` in this folder. Its §9.5 rows are stamped **applied, 21 October 2026**, and every receiver below carries a dated revision line and a status bump.

**The WASI door closes (21 October 2026).** The door had three parts: HGBA-2's X-3 row offered native Component Model support as optional and said nothing about running a program; Stage 1's out-of-scope bullet refused "the browser as a window manager for local apps"; and Stage 1's Stage 3 backlog row offered "native Component Model (optional)" with nothing behind it. All three now name the same thing:
- **HGBA-2 2.4.3 (A1; A2 subsumed).** The X-3 row is replaced: native CM counts toward the "two engines" gate for web exposure only, the app host is the Stage 3 deliverable, app-host components do not count toward the gate, and the app host does not depend on that row. A2 proposed a second replacement of the same row; it is recorded as **subsumed by A1** and no second edit is applied.
- **Stage 1 0.14.4 (A4).** The refusal is now **time-scoped to Stages 1–2** and **object-scoped to arbitrary native local apps**, and the section names the capability-sandboxed WASI app host as the spec'd Stage 3 path: signed components in browser-owned windows under the broker's consent. The refusal's object, arbitrary native local apps, is unchanged.
- **Stage 1 0.14.4 (A5, A31).** The Stage 3 backlog row becomes "native Component Model (optional) **and the app host (AH-M0–AH-M5; *Izcal app host*)**", with the app host's milestones, **15–21† engineers at peak**, **16–39† incremental engineer-years**, **2–4† non-engineers** and C-6 addendum 5. The door's closure is therefore a promotion with an owner, a gate (**AH-D8**, at most 20† named apps), a cost and an exit audit (AH-M5), not a new permission: no app model is added to Stages 1–2.

**G220–G229 are registered** (app host) in Stage 1 2.2 and the coding standard; the registry now reads `G210–G219 devices; G220–G229 the app host (*Izcal app host* 6.17); **G230 onward: the next specification**`.

| ID | Target | Change | State |
|---|---|---|---|
| A-AH-1 | HGBA-2 2.4.3 | X-3 row replaced by A1's text (native CM gate for web exposure only; app host the Stage 3 deliverable; components do not count; no dependency); status 2.4.2 → 2.4.3 | **Applied (HGBA-2 2.4.3)** |
| A-AH-2 | HGBA-2 2.4.3 | A2 recorded as subsumed by A1 in the Revision 2.4.3 line; no second edit of the X-3 row | **Applied (HGBA-2 2.4.3)** |
| A-AH-3 | HGBA-2 2.4.3 | M-1 record: app host §7 compliance and the Component Model/WASI suites as test-only oracles | **Applied (HGBA-2 2.4.3)** |
| A-AH-4 | Stage 1 0.14.4 | §6 out-of-scope bullet: narrowed, time-scoped refusal and the app host as the Stage 3 path (A4) | **Applied (Stage 1 0.14.4)** |
| A-AH-5 | Stage 1 0.14.4 | §9 Stage 3 backlog row and app host paragraph (AH-M0–AH-M5, 15–21† engineers, 16–39† incremental, 2–4† non-engineering, C-6 addendum 5); A31 part 2 | **Applied (Stage 1 0.14.4)** |
| A-AH-6 | Stage 1 0.14.4 | 2.1 crates: `hg-app-host-main` L6; six L5 app crates; `hg-wit`, `hg-app-manifest`, `hg-app-ticket` L2s; `hg-component` L2e; naming G10–G229 | **Applied (Stage 1 0.14.4)** |
| A-AH-7 | Stage 1 0.14.4 | 2.2 registry: G220–G229 the app host; G230 onward reserved | **Applied (Stage 1 0.14.4)** |
| A-AH-8 | Stage 1 0.14.4 | Stage 2 backlog Engine cell and the Stage 2 seams paragraph (AH 6.19) | **Applied (Stage 1 0.14.4)** |
| A-AH-9 | Security scope 0.3.8 | 6.3/6.3a component-host configurations and `app://<id>` keying; `component-host.md`; 6.5 app-profile subsection (A9) | **Applied (security scope 0.3.8)** |
| A-AH-10 | Security scope 0.3.8 | `app.intent`, `app.log`, `app.control`, `app.invoke` endpoints and their three rules (A10) | **Applied (security scope 0.3.8)** |
| A-AH-11 | Security scope 0.3.8 | Ten `app-*` prompt classes and the A-SH-12 extension to all high classes (A11) | **Applied (security scope 0.3.8)** |
| A-AH-12 | Security scope 0.3.8 | REC-R8 `app.wasi` class with retention and the two-record join (A12) | **Applied (security scope 0.3.8)** |
| A-AH-13 | Security scope 0.3.8 | AGT-R3 refuses `app://`; agents use `app.invoke` with `agent:<task>`; AGT-R6 counts state-changing app intent (A13) | **Applied (security scope 0.3.8)** |
| A-AH-14 | Security scope 0.3.8 | SEC-18/SEC-19: 0 `app.invoke` from web, 0 `app://` navigations from agents, person-present app-intent confirmation (A14) | **Applied (security scope 0.3.8)** |
| A-AH-15 | Coding standard 0.3.7 | G220–G229 registered; tier-P app crates and tier-T tools named (A15) | **Applied (coding standard 0.3.7)** |
| A-AH-16 | Coding standard 0.3.7 | CS-ST-10 ceiling, replay, verify, G224 and the tier-P closure check (A16) | **Applied (coding standard 0.3.7)** |
| A-AH-17 | JavaScript engine 0.4.4 | Five embedders: app, `--replay` and `--verify` configurations in §6.0 and JS-D10 (A17) | **Applied (JavaScript engine 0.4.4)** |
| A-AH-18 | JavaScript engine 0.4.4 | G6 entry point; G14 jitless loaded-library closure includes `hg-app-host-main` (A18) | **Applied (JavaScript engine 0.4.4)** |
| A-AH-19 | JavaScript engine 0.4.4 | JS-D13 app tier ceilings (Baseline default, Full per app/policy, Jitless selectable; G224) (A19) | **Applied (JavaScript engine 0.4.4)** |
| A-AH-20 | Devices 0.3.1 | DV-D2 app-id-bound tickets; DV-D4 chooser accepts `app://<id>`; DV-D8 virtual device; DV-D9 named-app gating (A20) | **Applied (devices 0.3.1)** |
| A-AH-21 | Storage 0.3.3 | SG-D4 app bucket; §6.14 app directories and the five path-resolution rules; G150 (A21) | **Applied (storage 0.3.3)** |
| A-AH-22 | Networking 0.3.8 | NW-D4 LNA tickets for app ids; NW-D7 app partition tuple; §6.15 app HTTP out of scope (A22) | **Applied (networking 0.3.8)** |
| A-AH-23 | Extensions 0.4.2 | EX-D10 shared directory/signing/review plus mechanical capability diff; EX-D12 no telemetry; §6.12 (A23) | **Applied (extensions 0.4.2)** |
| A-AH-24 | Shell 0.3.5 | `shell.apps`; app windows and their chrome; ledger app entries; SH-D11 (A24) | **Applied (shell 0.3.5)** |
| A-AH-25 | DevTools 0.3.7 | `app` IIP domain (6.1; DT-D5); `app.wasi` class and the two-record join (DT-D2) (A25) | **Applied (DevTools 0.3.7)** |
| A-AH-26 | Phishing protection 0.3.1 | §6.4.9 app window vs installed site's standalone window; no-chrome-imitation review refusal (A26) | **Applied (phishing protection 0.3.1)** |
| A-AH-27 | Resource footprint 0.4.5 | FP-1 exclusion; app host base ≤ 4 MiB† and per live app ≤ 8 MiB†; FP-D8 app origins; FP-D9 no app ids (A27) | **Applied (resource footprint 0.4.5)** |
| A-AH-28 | Content blocking 0.3.5 | Scope: app HTTP out of scope, with AH-D10's reason and AH-18's reversal trigger (A28) | **Applied (content blocking 0.3.5)** |
| A-AH-29 | Accessibility 0.2.6 | §6.12 app surfaces: chrome meets shell targets, sealed UI inherits engine checks, component has no accessibility surface (A29) | **Applied (accessibility 0.2.6)** |
| A-AH-30 | DOM 0.2.6 | §6.9 app navigation rules: web-to-`app://` refused, `app://` is not web, in-place replacement refused (A30) | **Applied (DOM 0.2.6)** |
| A-AH-31 | Licence audit and the Stage 3 plan | Component Model WAST suite and `wasi-testsuite` recorded as Apache-2.0 test-only oracles (revision + class-3 rows); AH-M0–AH-M5 in the Stage 3 plan (see A-AH-5) | **Applied (licence audit; Stage 1 0.14.4)** |

Not tracked as rows: the app host's own open questions. **Q-AH-1 to Q-AH-9 stay open** with the owners and needed-by dates in the spec's Section 11; the owner-facing exceptions are **AH-D8's named-app gate and per-capability owner exceptions**, which stand as written and expire after 12 months†, and no new exception is granted.

## Shell UI design (A-UI)

Targets the [shell UI design][UI], draft 0.3. State: pending.

| ID | Section | Change | Source |
|---|---|---|---|
| A-UI-1 | R13 | Resource meter becomes the causal view of FP 6.14; background and battery controls per site | FP 9.4; FP-D15 |
| A-UI-2 | 4.10 | "System UI fonts" is the OS UI face on macOS and Windows and Izcal's sans alias on Linux, with the interim tier; native-surface strings follow TX-16 | TX 9.5 (R23) |
| A-UI-3 | Media | Native media controls' shell parts, fullscreen exit, Picture-in-Picture and Document PiP windows, protected-playback affordance, per-site autoplay and animation settings, caption preferences, media-crash label, per-site media energy | MD 9.5 (R17); MD-D32 |
| A-UI-4 | Site panel, prompts | States for suspended GPU contexts, unavailable WebGL and WebGPU, the WebGPU permission; prompt surface as a separate OS layer; reduce-motion, reduce-transparency, flash protection | GR 9.5; GR-D11, GR-D28, GR-S11 |
| A-UI-5 | Settings | Overlay-or-classic scrollbar preference defaulting to the OS accessibility setting; page-zoom levels as recorded inputs; form-control pickers anchored by `hg-layout-query` rectangles | LY 9.5 |
| A-UI-6 | 4.3 site panel | Counts of parties and organisations, graduated controls, the report flow, the "recognizability" label, the Security interstitial | CB 9.5; CB-D13, CB-D19 |
| A-UI-7 | UI-5, UI-10, 4.5, UI-9 | Security prompts drawn by the prompt renderer, approved only on broker-observed input; phishing interstitials depend on SEC-D7 | SEC 9.4; CON-R1 to CON-R5 | **Applied (shell UI design 0.4, graphics 0.4.3, phishing 0.3, security scope 0.3.2):** prompts are separate broker-owned OS windows, software-rasterised, never on the content GPU; GR-D28's property is normative; the phishing/malware interstitial is the phishing spec's PH-D6/PH-12 surface (A-SH, A-PH rows) |

## Security scope (A-SEC)

Targets the [security scope][SEC], draft 0.3 on `spec/security-scope`. All rows are Phase 5 (rebase), except A-SEC-25.

| ID | Section | Change | Source |
|---|---|---|---|
| A-SEC-1 | 6.8, TB-4, SEC-1, SEC-2, SEC-11, SEC-D1, SEC-D12 | TB-4 becomes the cage boundary; JS-16 replaces SEC-1's V8 clock; SEC-2 rehearses the engine's emergency release; FFI-R1–R4 replaced by H-1, `Field`, `Root` and JS-10; SEC-D12 retires | JS 9.5 (C-9, C-10) |
| A-SEC-2 | 6.7, UPD-R5, REC-R6, renderer threat model | "DrumBrake off" becomes "Wasm interpreter off"; jitless replay of JIT recordings by default with an engine-triage mode; residual risk names engine logic bugs inside the cage; persistence through engine images bounded by FP-D16; code-cache entries written only by the builder, which joins the utility list at the strictest tier | JS 9.5; JS-51, JS-D21 |
| A-SEC-3 | 6.3, 6.3a | Rows for the Linux spawner, spare process, font and filter-list utility jobs, image decoder process, the measurement-only co-locating build, and attribution history as a storage-service asset | FP 9.4 |
| A-SEC-4 | Renderer and GPU-service threat models | Persistence through engine images; FP 6.18.1's shared-resource channels; glyph atlases partitioned | FP 9.4 |
| A-SEC-5 | 6.7, 6.13, SEC-34, SEC-16 | Suspend files and attribution history under REC-R2's key rule; image inventory under SEC-16; cgroups and job objects as resource limits, not boundaries; no `RLIMIT_AS` on renderers | FP 9.4 |
| A-SEC-6 | Tier 1, ATK-1, SEC-21, SEC-24 | ST 6.14's fuzz targets; one parser, sanitize-while-parsing, `attr()` taint, `:visited` partitioned with the container, sheet sharing without leakage; visited state in recordings only as salted answers; CVE classes in SEC-24's suite | ST 9.5 |
| A-SEC-7 | Supply chain | Licence gate: `tools/deny.toml`'s MIT allow list; `hashbrown` without its default hasher | ST 9.5; OD-L8, OD-L9 |
| A-SEC-8 | 6.3, 6.5, 6.10, Q-SEC-13 | Fonts are Izcal's own; Q-SEC-13 answered for fonts; the font service parses no font bytes | TX 9.5 (R17) |
| A-SEC-9 | IPC-R7, IPC-R17, GPU-service threat model | Mask transfer with receiver-fixed sizes and per-partition upload budget; bounded path format; session image read as untrusted input | TX 9.5 (R14) |
| A-SEC-10 | Broker threat model, 6.7 | No page-controlled string reaches an OS text API in the broker; OS UI frameworks inventoried; third-party IME DLLs recorded | TX 9.5 (R15); TX-16, TX-D23 |
| A-SEC-11 | Shell-renderer threat model, ATK-2 | Title and string length caps; the shell text path's fuzz bar | TX 9.5 (R16); TX-12a |
| A-SEC-12 | Renderer threat model | Web fonts parsed in the renderer over immutable bytes; residency channel; no Local Font Access; Linux interim faces read only as sealed copies through the proved reader | TX 9.5 (0.3.3); TX-D2, TX-D25 |
| A-SEC-13 | 6.3, 6.3a, 6.5 | Network services per (top-level site, container): a compromise yields that site's partitions and grants; container and system services as rows; no neqo or hickory-dns; residuals | NW 9.5; NW-D3 |
| A-SEC-14 | 6.6 IPC-R1, IPC-R2 | New handle kinds (connected and connecting TCP, connected and interface-bound UDP, producer-writable consumer-read-only shared memory); broker-issued LNA tickets | NW 9.5; NW-D4, NW-D10 |
| A-SEC-15 | 6.3, 6.7 | OS-adapter utility process with its sandbox row; the broker does none of resolver fallback, SSPI/GSSAPI, client-key signing; network rows: no socket creation, no certificate-store access | NW 9.5 |
| A-SEC-16 | 6.10a | Point to NW 6.12 (own ICE, DTLS, SRTP, SCTP; interface-bound sockets; no local candidates without a ticket) | NW 9.5 (C-18) |
| A-SEC-17 | 6.11 NET-R5–R7, Q-SEC-5, SEC-D13 | NET-R5 lists `hg-tls` and `hg-crypto`; NET-R6 per NW-D5 with the distrust process (Q-SEC-5 answered); NET-R7 at the broker with tickets; SEC-D13's Windows question answered by Q-NW-2's spike | NW 9.5; NW-D5 |
| A-SEC-18 | 9.2 costs | 5 engineer-months of protocol fuzzing, caps and the LNA suite move to NW's verification track; on-call shared | NW 9.5 (C-6) |
| A-SEC-19 | SEC-25, supply chain | Advisory scope: `hg-crypto`'s RustCrypto pins and Izcal's own TLS; the licence gate; G36 | NW 9.5; NW-D22, NW-D23 |
| A-SEC-20 | 6.5, trust data | Root store as an Izcal policy output with the security owner in review; the trust-data channel carries Izcal's store and HSTS list; OS-anchor interim and NW-D24's release trigger as residuals | NW 9.5; NW-D5, NW-D24, NW-D25 |
| A-SEC-21 | 6.10, SEC-12, SEC-30, TB-6, 6.3a, IPC-R7, Q-SEC-4, Q-SEC-13, 6.10a | 6.10 rewritten: no C decoders or OpenH264; SEC-12's metric is L2m crates only in media or PDF processes; SEC-30 (a) names media and hardware-decode processes; TB-6 rewritten; rows and threat models for media, hardware-decode, audio, capture and CDM processes; Q-SEC-4 answered by MD-S1, MD-S2, MD-S4; Hardened profile's media setting | MD 9.5 (R9); MD-D1, MD-D5, MD-D27 (C-8) |
| A-SEC-22 | 6.9, GPU-R2 to GPU-R15, IPC-R7, IPC-R15, IPC-R16, TB-5, SEC-10, Q-SEC-14, Q-SEC-15 | 6.9 rewritten for two GPU process kinds; Izcal's WebGPU validation, WebGL on the HAL, generated display lists, translation validation; new GPU-R9 to GPU-R15; prompt rule meeting CON-R1, CON-R3, CON-R5; IPC-R16 entry 1 removed; threat models for both GPU kinds; Q-SEC-14 and Q-SEC-15 answered | GR 9.5; GR-D6, GR-D9, GR-D11, GR-D28 |
| A-SEC-23 | Tier 1, renderer threat model | Layout crates are Tier 1 fuzz surfaces; no re-entrancy (G37); depth and fuel limits; IO v2 and the frame-clip invariant | LY 9.5; LY-15 |
| A-SEC-24 | IPC-R15, fuzzing, UPD-R5 | List text parsed only in the utility process; `hg-cb-syntax` and `hg-cb-compile` tier-P fuzz targets; Security-list additions ride the emergency channel | CB 9.5; G80, CB-D17 |
| A-SEC-25 | IPC-R5, NET-R3, SEC-28, SEC-7 | Abort-on-panic everywhere; the tier-P exceptions file named | CS 9.4 — **applied (security scope 0.3 and commit d30b95b)** |

## Coding standard (A-CS)

Targets the [coding standard][CS], draft 0.3.2 on `spec/cs-amendments`. All rows are Phase 5, with G-23 and G-24, except **A-CS-8**, applied 21 October 2026 with batch 3.

| ID | Section | Change | Source |
|---|---|---|---|
| A-CS-1 | CS-V8-1, -3, -5, -9; CS-FFI-1; CS-TY-10; CS-TY-12; CS-ST-10; CS-D3; tiers; CS-PF-4 | CS-V8-1 becomes a brand check on shape families; CS-V8-3 becomes H-1 via `Field` types; CS-V8-5 stands for `hg-js-host`; CS-V8-9 moves to `hg-heap`'s `Trace` derive; CS-FFI-1 retires; CS-TY-10 adds the `Mutator` token; CS-D3 re-run at JS-M0; L2e crates (tier R), tier-P component-host build, tools (tier T); CS-TY-12 extends to engine parsers (G18); H-2 lint as G17; CS-PF-4 adds interpreter handlers, IC programs and GC loops | JS 9.5 (C-10) |
| A-CS-2 | CS-PF | Canonical-footprint gate with a `Footprint:` trailer per ISA; caches register with the memory manager; no pointer-bearing static data; no periodic timers; hot-type list adds node columns and rare data; field-reorder lint; threads only through the substrate; no release target enables the co-locating feature; CS-PF-5 and CS-PF-6 cite FP targets | FP 9.4; FP-3, FP-18, FP-24, FP-D5 |
| A-CS-3 | New rule | A rule matching G21 (no recursion on content-controlled depth); fallible allocation with defined outcomes in parsers and the substitution interpreter | ST 9.5 |
| A-CS-4 | Tier table, G18 | L2t crates tier R; `hg-ucd`, `hg-unicode`, `hg-glyph-atlas` tier P; tools tier T; G18 covers font-derived values | TX 9.5 (R18); P3 |
| A-CS-5 | Tier table, `tools/tiers.toml`, CS-AS-1 | NW 6.21's crates tier P; L2s crates as linked by renderers; `hg-net-replay` in the replay binary; G32 (no CS-AS-4 entry); G35's wire-identifier lint; G36 and the constant-time rule for `hg-crypto` | NW 9.5 |
| A-CS-6 | `tiers.toml`, `unsafe-inventory.toml`, `hg-substrate-os` | L2m with its lint profile (G53); inventory entries for `hg-simd`, `hg-media-os`, `hg-hwdecode-host`, `hg-audio-service`, `hg-capture-service`; a real-time thread interface and OS-callback bridge (tier O) | MD 9.5 (R10); Q17 |
| A-CS-8 | Tier table, rule table, `tools/tiers.toml` | `hg-webcrypto` and `hg-webcrypto-observe` tier P; **G160–G169** join the lint set (A-WC-20). **Applied 21 October 2026 (coding standard 0.3.2, `spec/cs-amendments`)** | WebCrypto 9.5 |

## First-principles-spec skill (A-SKILL)

| ID | Change | Source | State |
|---|---|---|---|
| A-SKILL-1 | The research brief says engine source files are not read, even for constants; numbers come from black-box tests or published documents | ST 9.5 (critique C1) | Applied (skill line 83; commit 04492ac, HGBA-2 2.3.1) |
| A-SKILL-2 | Add OD-7's three conditions for prose hosted in code repositories (no code file, header, diff or review opened; an outside copy cited where one exists; no constant taken that appears only beside code) | RN2 Q31; **OD-7** (via GR 9.5's decision-log row) | Pending (wave 2) |

## Other targets (A-CON, A-LA, A-RN, A-FUT)

| ID | Target | Change | Source | State |
|---|---|---|---|---|
| A-CON-1 | `CONTRIBUTING.md`, code review | One carve-out for hand-rolled cryptography: `hg-crypto`, under the external audit (NW-52), NW-49 and NW-50; **extended 21 October 2026** to name `hg-webcrypto` and the WebCrypto additions to `hg-crypto`, in the same audit and gates (A-WC-21) | NW 9.5; NW-D23; WebCrypto 9.5 | **Applied 21 October 2026 (cs-amendments worktree)** |
| A-CON-2 | `CONTRIBUTING.md`, `provenance` check | If the owner ever authorises reading RFC code under MD-D4's reversal, the logging rule and the check's scope | MD 9.5 (R22) | Conditional |
| A-CON-3 | `CONTRIBUTING.md` | Rulesets instead of client-side tag rules; checks from the base branch; AI review documented as advisory; self-approvals recorded (S-4e force-push and V8 hash pins are superseded) | SEC 9.4 | Phase 5 |
| A-CON-4 | `CONTRIBUTING.md`, `tools/checks.sh` | "Coding standard" section, PR trailers, `cs-allow` markers, agent rules; the `v8-audit` check | CS 9.4 (applied on branch) | Phase 5: keep the section, drop `v8-audit` (C-10) |
| A-LA-1 | [Licence audit][LA] | AOM patent notice, codec constant tables by publisher, test-only P.863, Argon and ISO material, energy-reference players; no table taken from RFC code components (recommended); OD-7 for prose in code repositories | MD 9.5 (R24); Q31 | Pending (owner confirms the table rule) |
| A-LA-2 | Licence audit | Vulkan loader as an OS interface on G67's condition; SPIR-V grammar as standards data; `ash` removed; Agility SDK, `dxil.dll`, DXC, Swappy and the Metal shader converter never shipped; class-5 "Renderer" row corrected (I-2) | GR 9.5; Q31 | Pending (owner confirms) |
| A-LA-3 | Licence audit 2.5 | Non-commercial datasets unused even test-only until Q-CB-1; Tranco's licence unstated; entity-data inputs licence-reviewed | CB 9.5; Q-CB-1, Q-CB-2 | Pending |
| A-RN-1 | [Reconciliation 1][RN] P7 | Registry statement lists G10–G12 and G37–G45 and the full registry (C-5) | LY 9.5; Q19 | Pending (an addendum line) |
| A-RN-2 | Reconciliation 1 P18 | Projection assumes F fixed per process; holds because content GPU processes are keyed like renderers | GR 9.5 | Pending (an addendum line) |
| A-RN-3 | Reconciliation staffing | 384–502 engineers† (437–560 people†) for eight specs after removing one double-counted overlay engineer | GR 9.5 | Applied (RN2 Section 4); superseded for nine specs by C-6 |
| A-FUT-1 | SVG, editing, animation specs | LY 6.5.10 (SVG boundary and text); caret and selection from fragments; scroll ranges, view-transition geometry and `interpolate-size` through the intrinsic memo | LY 9.5 | **Fulfilled 20 October 2026**: all three specs exist (batch 2); the layout spec 0.4.2 carries the SVG boundary and view-timeline text, and the caret/selection row is superseded by the editing spec's ownership (A-DM-1) |
| A-FUT-2 | PDF spec | Uses `hg-jpx`, `hg-jbig2`, `hg-ccitt` (built by MD) inside the PDF process under G50–G53 | MD 9.5 (R18) | When written (G-14) |
| A-FUT-3 | ML service spec | Owns and sizes MD-D25's caption and image-description models | MD 9.5 (R19) | **Discharged 21 October 2026 (ML service 0.3)**: 9.3 owns and sizes ML-IMG (≤ 200 MB†), ML-OCR (≤ 80 MB† per script) and ML-ASR (≤ 150 MB†); MD 9.5 R19 is replaced (A-ML-13) |
| A-FUT-4 | Animation spec | Owns `hg-interp`, the compositor-eligible record (transform and opacity only), events and `getAnimations()` state; owner named before GR-M1 | GR 9.5 | **Fulfilled 20 October 2026**: the animation spec exists (draft 0.3, owner decisions accepted) and owns `hg-interp` from AN-M0; the record is the bounded hand-off record of its 6.5 — content is no longer "transform and opacity only" (class G′, U and filter/clip-path classes exist), though the compositor still samples transform and opacity only |
| A-FUT-5 | SVG spec | Typed render tree consumed by `hg-svg-paint`; the opaque-SVG path uses the same crates | GR 9.5 | **Fulfilled 20 October 2026**: the SVG spec exists (draft 0.3, owner decisions accepted); the graphics spec 0.4.2 consumes its typed render tree (its 6.4 and 6.5) and the opaque-SVG path uses the same crates |
| A-FUT-6 | PDF spec | Which raster and display list PDF rendering and the writer use; reconcile Stage 1 13.2's utility process with media's "PDF process" | GR 9.5 | When written (G-14, C-21) |
| A-FUT-7 | Phishing / URL-reputation spec | URL reputation is a separate system with its own owner | CB 9.5; CB-D23, Q-CB-5 | **Applied 21 October 2026 (phishing 0.3)**: the spec owns the system and closes G-22 and Q-CB-5; content blocking 0.3.3 carries CB-D23's answer and the source-governance exception |

[H2]: ../../reports/HGBA%202%20browser%20architecture%20spec.md
[H3]: ../../reports/Future%20software%20browser%20engine.md
[S1]: ../../reports/HGBA%20Stage%201%20architecture%20design.md
[DT]: ../../reports/Izcal%20DevTools%20spec.md
[UI]: ../../reports/HGBA%20shell%20UI%20design.md
[JS]: ../../reports/Izcal%20JavaScript%20engine.md
[FP]: ../../reports/Izcal%20resource%20footprint.md
[ST]: ../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[TX]: ../../reports/Izcal%20fonts%20and%20text.md
[NW]: ../../reports/Izcal%20networking.md
[LY]: ../../reports/Izcal%20layout.md
[MD]: ../../reports/Izcal%20images%20and%20media.md
[GR]: ../../reports/Izcal%20graphics%20and%20rendering.md
[CB]: ../../reports/Izcal%20content%20blocking.md
[RN]: ../Phase%202%20reconciliation/reconciliation.md
[RN2]: ../Phase%202%20reconciliation/reconciliation_2.md
[LA]: ../Phase%202%20reconciliation/licence_audit.md
[SEC]: ../../../security-scope/reports/Izcal%20security%20scope.md
[CS]: ../../../cs-amendments/reports/Izcal%20coding%20standard.md
