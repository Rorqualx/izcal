# Phase 3 amendment ledger

Compiled 19 October 2026 on branch `spec/hgba2-2.4`, read-only over every source. No spec was edited. Sources: Section 9.5 (9.4 for the footprint spec) of the nine accepted component specs and of the DevTools spec; [reconciliation 1][RN] (P1–P27, OD-1 to OD-4), [reconciliation 2][RN2] (Q1–Q32, OD-5 to OD-8) and the [licence audit][LA] (OD-L1 to OD-L9); the [security scope][SEC] 9.4 (branch `spec/security-scope`, draft 0.3) and the [coding standard][CS] 9.4 (branch `spec/cs-amendments`, draft 0.3.1, the newest of the three coding-standard branches), both read from their worktrees.

**Source decision status.** Every component spec's decisions are accepted: JS 0.3 (JS-D1–D21), FP 0.3 (FP-D1–D19), ST 0.3.3 (ST-D1–D20), TX 0.3.4 (TX-D1–D25), NW 0.3.3 (NW-D1–D26), LY 0.4 (LY-D1–D22), MD 0.4 (MD-D1–D32), GR 0.4 (GR-D1–D28), CB 0.3 (CB-D1–D24), DevTools 0.3 (Section 9.4 decisions of 18 October; DT-D10 Option B, DT-D11 (a)), and OD-1 to OD-8 and OD-L1 to OD-L9. The security scope's decisions are accepted except SEC-D6, SEC-D8 and SEC-D9, which its 0.3 line does not list, but the document is **not merged**. The coding standard's decisions are taken and its 9.4 is "applied in 0.3.1", **but only on its unmerged branch**. The State column says whether a row is still to do.

**ID scheme.** `A-<target>-<n>`. Targets: H2S (HGBA-2 superseded sections, 2.4.0), H2 (other HGBA-2 changes), H3 (HGBA-3), S1 (Stage 1 design), JS, FP, ST, TX, NW, LY, MD, GR, CB (the nine component specs), DT (DevTools), UI (shell UI design), SEC (security scope), CS (coding standard), SKILL (first-principles-spec skill), CON (`CONTRIBUTING.md`), LA (licence audit), RN (reconciliation notes), FUT (unwritten specs). Where several specs ask for the same change, one row lists every source. Conflicts are `C-n`, gaps `G-n`.

## Summary

### Counts per target

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
| C-6 | Staffing totals | ST (231–290† for five specs), MD and GR (437–560† for eight), CB (+5–9.5†), NW's own Stage 1 row (46–61†, stale: NW is 58–81† since 0.3.2), DevTools (4–5† + 1–1.5†, applied), security scope (Security track 3–4†, SEC-D2) | RN2 Section 4 covers eight specs only | **Unresolved.** Recommendation: one statement for Stage 1 8.3 of about **442–570 people†** for the nine component specs (437–560† + CB's 5–9.5†), plus DevTools 5–6.5† and the Security track 3–4†, with the uncounted list of G-11 to G-21 named beside it |
| C-7 | Stage 1 dates | JS (Stage 1 exit 39–51 months†, cited by ST 9.3 and NW-M5), LY (48–60†) | **Q29** (milestones re-dated from the specs; M6 and M7 readiness points, not an order) | Settled. HGBA-2's own Stage 1 heading has no row (G-3) |
| C-8 | HGBA-2 S-5 library sandboxing | SEC (S-5 library sandbox for codecs and PDF as a Stage 1 target, SEC-12, SEC-D3), MD (S-5 superseded for media; SEC-12's metric rewritten) | HGBA-2 2.3.0 / 2.3.2 and **MD-D1** (accepted after security scope 0.3) | Settled for media: MD's text. PDF has no owner (G-14). Recommendation: restate S-5 and D6 together (G-8) |
| C-9 | HGBA-2 4.8 FFI rules | SEC (add FFI-R1 to FFI-R4), JS (FFI-R references give way to the unsafe-core rules, JS-14; FFI-R1–R4 replaced by H-1, `Field`, `Root`, JS-10) | HGBA-2 2.3.0 and the JS decisions | Settled: JS's text. The security scope keeps FFI rules only for OS-interface FFI when rebased |
| C-10 | HGBA-2 D2, S-4e and the fork | SEC (SEC-1 clock, Chrome-patched-branch rule SEC-D12, S-4e force-push), coding-standard branch (HGBA-2 "2.2.1": S-4e range-unwrap binding for CS-V8-1; Stage 1 5.3 wrapper tags; `v8-audit` check), JS (S-4e and S-4f retire; SEC-D12 retires; JS-16 replaces SEC-1's V8 clock) | HGBA-2 2.3.0 (V8 excluded) and the JS decisions | Settled: drop on rebase. The branch's revision number 2.2.1 collides with main's 2.3.x line and must not be merged as is |
| C-11 | HGBA-2 S-4a Hardened profile | JS (jitless tier ceiling with W0; Wasm conformance "identical"), SEC (CET/BTI "where hardware and OS support it"; ACG on Windows; "Izcal-only renderers jitless by default") | Partly: JS decides the ceilings | **Unresolved** for "jitless by default", which JS's Standard and Hardened tier ceilings (JS 6.8) do not state. Recommendation: S-4a takes JS's text plus SEC's CET/BTI and ACG wording; the default tier stays JS's, and the security scope's rule is re-decided at its rebase |
| C-12 | FP-D16 regenerable-cache class: binding and partition | JS and FP (bound to the engine build, partitioned per (top-level site, container); members include compiled filter automata), NW (HTTP cache bound to format version and security epoch), CB (filter files keyed by list hashes and compiler version, user lists per container), ST and TX (compiled user sheets, font-hash cache) | **P19** (members), **CB-D11** (accepted after P19) | Settled for members. The class definition must state binding and partition per member (recommendation: "bound to its format version and writer; partitioned per (T, C) whenever it holds user- or site-derived state") |
| C-13 | Content-blocking list ownership | NW (FP 6.11: "their production is this spec's", NW-D26), CB (FP 6.11: governance "belong to the content-blocking spec"; NW 9.3: second review, importers, log, witnesses and entity data counted in CB) | CB 9.3 (production stays in NW's lists track; governance, second review and log in CB), CB-D5, CB-D16 | Settled. FP 6.11 text: Izcal's own lists (NW-D26), produced by NW's lists track under CB's governance |
| C-14 | Matcher budget FP-28 | NW (≤ 256 KiB† charged per partition service; sealed automaton file uncharged), CB (compiled default bundle ≤ 8 MiB†, clean and shared; same per-service line; p99 ≤ 20 µs†; compile budgets) | CB is later and compatible | Settled: merge (CB's cap on the clean file plus NW's per-service line) |
| C-15 | HGBA-3 B7 | FP (frozen tabs skip style and layout; hidden running tabs stay pixel-optional), GR (B7 replaced; "skips paint" made one text with Stage 1 5.7's "stops after chunk generation") | GR owns rendering text (Q20 table) | Settled: GR writes B7 with FP's freeze rule |
| C-16 | Keying of decoder and media processes in Stage 1 | FP (decoder process per (site, container), FP-D11; Stage 1 M7 row), MD's own Stage 1 rows ("media process per (site, container)") | **Q7**: every per-site service process keyed exactly like its renderer (origin under FP-D8) | Settled. FP's and MD's rows are stale; A-MD-2 fixes MD |
| C-17 | Extensions host | HGBA-2 D5, W-1, X-3 and Stage 1 D-S3 (Wasmtime), CB 6.15 and NW 6.19 ("Stage 2, Wasmtime"), CB's D-S3 row (three tiers), JS (JS-D10: Izcal's own engine in a component host) | **JS-D10** | Settled. Stale "Wasmtime" text in CB 6.15, NW 6.19, HGBA-2 W-1 and X-3 (G-5) |
| C-18 | Stage 1 utility processes and WebRTC placement | SEC ("WebRTC (network service and utility)"; seventeen process kinds), NW-D3 (WebRTC transport in the partition service), MD (WebRTC media in the media process), TX (font service), FP (font and filter-list utility jobs) | NW-D3, MD-D5 (accepted after security scope 0.3) | Settled; the security scope's count of seventeen kinds is stale and is recounted from the Phase 4 process map |
| C-19 | DevTools replay unit | DevTools (Stage 1 3.4: one process per replay host), GR (a multi-process replay unit: renderer, its content GPU process and the tab's compositor inputs), NW-D2 (network recording joined by request ID, P20), SEC (replay host at the recorded renderer tier, REC-R6) | None: DevTools was never reconciled with the Phase 2 specs | **Unresolved; needs the DevTools owner.** Recommendation: recordings are per process and joined by IDs (P20's rule); a replay host runs one process kind, and GR's unit replays as a group of hosts with the compositor inputs recorded at the broker boundary |
| C-20 | DevTools JS domain | DevTools (D-S11 text applied in Stage 1 0.11: "V8's inspector passed through"), SEC (`script.v8` as opaque bytes, IPC-R16), JS (native IIP script domains from `hg-js-observe`, JS-63) | HGBA-2 2.3.0, JS-63 | Settled: JS's text; Stage 1 D-S11 and the 13.2 DevTools row are rewritten in Phase 4 |
| C-21 | PDF and print in Stage 1 13.2 | TX (print-to-PDF subsets fonts), LY (paged layout Stage 1; print UI and PDF writer Stage 2), GR (print raster Stage 1, GR-M3; PDF row reconciled with media's "PDF process" by the future PDF spec), MD (PDF process), Stage 1 13.2 (utility process) | **Q24** for layout and raster | Partly settled. **PDF writer, viewer and process have no owner** (G-14). Recommendation: the owner names a PDF spec owner before Phase 4 fixes 13.2 |
| C-22 | Spellcheck | TX (Stage 1 13.2: spellcheck in a utility process, 6.8a), RN1 Section 3 (spellcheck utility is Stage 2, budgeted by the editing spec) | None | **Unresolved, minor.** Recommendation: keep TX's placement and record the stage and budget as the editing spec's (G-16) |

**Final G-rule registry** (C-5): G1–G9 Stage 1; G10–G12 DevTools; G13–G18 JS; G19–G24 ST; G25–G29 TX; G30–G36 NW; G37–G45 LY; G46–G49 unassigned; G50–G57 MD; G58–G59 unassigned; G60–G72 GR; G73–G79 unassigned; G80–G87 CB; G88 onward the next spec.

### Gaps

**Superseded HGBA-2 text with no replacement row**

| ID | Superseded text | What exists | Recommendation |
|---|---|---|---|
| G-1 | Section 3 diagram: the renderer box ("style (stylo crate)", "JS+Wasm: V8 (rusty_v8) + V8 sandbox") and the network box | GR rewrites only the GPU box | The 2.4.0 editor redraws the whole diagram from JS, ST, NW, GR and MD |
| G-2 | Section 6, Stage 1: the `cppgc` spike narrative, start conditions (i)–(iii), and the F1, F2 and F3 fallback shapes | JS's S-4c and S-4e rows; JS-D4 retires the mirror (A-H3-2) | Move to a history appendix, as the status line already treats them |
| G-3 | Section 6 Stage 1 heading ("provisionally months 0–18; planning-pass placeholder 24–36") | RN2 Q29: Stage 1 exit 48–60 months† | Restate from Q29 in 2.4.0 |
| G-4 | X-1 riscv64 support tier and its V8-grounded prose | The JS spec never mentions riscv64 (JS-D12 covers x86-64 and AArch64) | **Owner decision needed** for the JS spec: keep X-1 (riscv64 back end for Izcal's engine) or retire it |
| G-5 | W-1 and X-3 (Wasmtime host; native Component Model "in the engine") | JS-D10 covers D5 only | Restate W-1 and X-3 from JS-D10 |
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
| G-12 | SVG (document model, SMIL, SVG DOM, geometry APIs) | RN2 Q20 and Q25, GR 6.16 and 9.5, LY 6.5.10 and 9.5, MD-D20 | GR's and LY's SVG interfaces |
| G-13 | Animations (Web Animations, CSS transitions and animations, owner of `hg-interp`) | RN2 Q20 and Section 4, GR 6.28 and 9.5 (owner to be named before GR-M1), LY 9.5, GR's ST row | GR-M1 |
| G-14 | PDF (writer, viewer, PDF process) | RN2 Q24, GR 6.27 and 9.5, LY 6.7, MD 9.5 (R18), TX (Stage 1 13.2 print-to-PDF; PDF viewer links the text stack) | Stage 1 13.2 (C-21) |
| G-15 | ML service (MD-D25's caption and image-description models; HGBA-2 4.5, D4, R-3) | MD 9.5 (R19), MD 9.3, RN2 Section 4 | MD-D25 |
| G-16 | Editing (caret, selection, editing commands, spellcheck budget) | LY 9.5, TX 6.8a, RN1 Section 3, RN2 Section 4 | C-22 |
| G-17 | Accessibility (AccessKit replacement, OS adapters) | JS-D11, RN2 Section 4 ("beyond each spec's share"), LY 6.x ("A11Y track") | G-10 |
| G-18 | WebCrypto owner, and whether it links `hg-crypto` (Q-NW-14) | NW 6.5 and 11.1, RN1 Sections 7 and 8 (still open) | Staffing (counted nowhere) |
| G-19 | Storage (storage service, IndexedDB, Cache API, quotas) | RN1 Section 4, RN2 Section 4; used by FP-D18, ST-D17 and NW | Stage 1 process map, security scope |
| G-20 | Extensions (Stage 2 surface, JS-D10 component host) | CB-D15, CB 6.15, NW 6.19, Stage 1 D-S3 | Stage 2 only |
| G-21 | Shell (as a staffed track; the shell UI design is a design, not a spec) | RN1 Section 4, RN2 Section 4, TX ("Shell spec"); A-UI rows land in a design document | Phase 3 A-UI edits are design edits |
| G-22 | Phishing and URL reputation (Q-CB-5) | CB-D23 | Unowned at CB-M3 |

**Other gaps found**

- G-23. No coding-standard tier rows are proposed for GR's L2g crates and back ends, LY's crates (L4) or CB's `hg-cb-*` crates (only a security-scope fuzz row for two of them). Add at the coding-standard rebase.
- G-24. The coding standard's Q-CS-4 cites a V8 source file (`sweeper.cc`), which HGBA-2 2.3.1 forbids; the standard has had no 2.3.1 source-evidence pass. Its CS-V8 rules, CS-D3 spike and `v8-audit` check are all V8-specific.
- G-25. Stage 1 Section 12 (amendments from the four verified facts, all about Servo, WebRender and V8) is history; facts 1–4 at the top of the document are obsolete. Retire in Phase 4.

### Order of application, and the recommended phase split

The evidence supports the proposed split with four adjustments: a short **step 0** before Phase 3 (the conflicts no decision covers, and the content-blocking spec, which no reconciliation pass covered); **HGBA-3 needs a 2.4.0-style rewrite**, not just amendments (G-9); the **Stage 1 redesign should take the security scope's and coding standard's Stage 1 rows** that are not V8-specific, so Stage 1 is rewritten once; and the **DOM-and-bindings and accessibility specs** (or an explicit owner placeholder) should exist before Phase 4, because Stage 1 5.1–5.4 and 5.8 have no source text otherwise.

| Step | Targets | Depends on | Parallel with |
|---|---|---|---|
| **Phase 3, step 0** | Owner decisions: C-6 (one staffing statement), C-11, C-19 (DevTools owner), C-22, G-4 (riscv64); owners named for G-11, G-14, G-17 at least. A short reconciliation addendum covering CB (C-12, C-13, C-14, C-17) and the final registry | — | — |
| **Phase 3, wave 1 (hubs)** | FP (A-FP, 38 rows, one editor), JS (A-JS) | Step 0 for C-6 wording only | Each other |
| **Phase 3, wave 2** | HGBA-2 2.4.0 (A-H2S, then A-H2 rows from merged specs); ST, TX, NW, MD, GR (pending rows); DevTools (A-DT, after C-19); shell UI (A-UI); skill (A-SKILL-2); licence audit (A-LA); `CONTRIBUTING.md` (A-CON-1); reconciliation notes (A-RN) | HGBA-2's I-1, S-3, 4.6 and X-4 text cite FP-7, FP-8, FP-D16 and FP-22 as amended in wave 1 | All of wave 2 in parallel; component specs touch disjoint files |
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
| A-S1-75 | 13.2 "Everyday features": find in page uses `hg-unicode` caseless and canonical-equivalence matching; print-to-PDF honours `fsType`; spellcheck in a utility process | TX 9.5 (R6) (C-22) |
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
| A-UI-7 | UI-5, UI-10, 4.5, UI-9 | Security prompts drawn by the prompt renderer, approved only on broker-observed input; phishing interstitials depend on SEC-D7 | SEC 9.4; CON-R1 to CON-R5 |

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

Targets the [coding standard][CS], draft 0.3.1 on `spec/cs-amendments`. All rows are Phase 5, with G-23 and G-24.

| ID | Section | Change | Source |
|---|---|---|---|
| A-CS-1 | CS-V8-1, -3, -5, -9; CS-FFI-1; CS-TY-10; CS-TY-12; CS-ST-10; CS-D3; tiers; CS-PF-4 | CS-V8-1 becomes a brand check on shape families; CS-V8-3 becomes H-1 via `Field` types; CS-V8-5 stands for `hg-js-host`; CS-V8-9 moves to `hg-heap`'s `Trace` derive; CS-FFI-1 retires; CS-TY-10 adds the `Mutator` token; CS-D3 re-run at JS-M0; L2e crates (tier R), tier-P component-host build, tools (tier T); CS-TY-12 extends to engine parsers (G18); H-2 lint as G17; CS-PF-4 adds interpreter handlers, IC programs and GC loops | JS 9.5 (C-10) |
| A-CS-2 | CS-PF | Canonical-footprint gate with a `Footprint:` trailer per ISA; caches register with the memory manager; no pointer-bearing static data; no periodic timers; hot-type list adds node columns and rare data; field-reorder lint; threads only through the substrate; no release target enables the co-locating feature; CS-PF-5 and CS-PF-6 cite FP targets | FP 9.4; FP-3, FP-18, FP-24, FP-D5 |
| A-CS-3 | New rule | A rule matching G21 (no recursion on content-controlled depth); fallible allocation with defined outcomes in parsers and the substitution interpreter | ST 9.5 |
| A-CS-4 | Tier table, G18 | L2t crates tier R; `hg-ucd`, `hg-unicode`, `hg-glyph-atlas` tier P; tools tier T; G18 covers font-derived values | TX 9.5 (R18); P3 |
| A-CS-5 | Tier table, `tools/tiers.toml`, CS-AS-1 | NW 6.21's crates tier P; L2s crates as linked by renderers; `hg-net-replay` in the replay binary; G32 (no CS-AS-4 entry); G35's wire-identifier lint; G36 and the constant-time rule for `hg-crypto` | NW 9.5 |
| A-CS-6 | `tiers.toml`, `unsafe-inventory.toml`, `hg-substrate-os` | L2m with its lint profile (G53); inventory entries for `hg-simd`, `hg-media-os`, `hg-hwdecode-host`, `hg-audio-service`, `hg-capture-service`; a real-time thread interface and OS-callback bridge (tier O) | MD 9.5 (R10); Q17 |

## First-principles-spec skill (A-SKILL)

| ID | Change | Source | State |
|---|---|---|---|
| A-SKILL-1 | The research brief says engine source files are not read, even for constants; numbers come from black-box tests or published documents | ST 9.5 (critique C1) | Applied (skill line 83; commit 04492ac, HGBA-2 2.3.1) |
| A-SKILL-2 | Add OD-7's three conditions for prose hosted in code repositories (no code file, header, diff or review opened; an outside copy cited where one exists; no constant taken that appears only beside code) | RN2 Q31; **OD-7** (via GR 9.5's decision-log row) | Pending (wave 2) |

## Other targets (A-CON, A-LA, A-RN, A-FUT)

| ID | Target | Change | Source | State |
|---|---|---|---|---|
| A-CON-1 | `CONTRIBUTING.md`, code review | One carve-out for hand-rolled cryptography: `hg-crypto`, under the external audit (NW-52), NW-49 and NW-50 | NW 9.5; NW-D23 | Pending (wave 2) |
| A-CON-2 | `CONTRIBUTING.md`, `provenance` check | If the owner ever authorises reading RFC code under MD-D4's reversal, the logging rule and the check's scope | MD 9.5 (R22) | Conditional |
| A-CON-3 | `CONTRIBUTING.md` | Rulesets instead of client-side tag rules; checks from the base branch; AI review documented as advisory; self-approvals recorded (S-4e force-push and V8 hash pins are superseded) | SEC 9.4 | Phase 5 |
| A-CON-4 | `CONTRIBUTING.md`, `tools/checks.sh` | "Coding standard" section, PR trailers, `cs-allow` markers, agent rules; the `v8-audit` check | CS 9.4 (applied on branch) | Phase 5: keep the section, drop `v8-audit` (C-10) |
| A-LA-1 | [Licence audit][LA] | AOM patent notice, codec constant tables by publisher, test-only P.863, Argon and ISO material, energy-reference players; no table taken from RFC code components (recommended); OD-7 for prose in code repositories | MD 9.5 (R24); Q31 | Pending (owner confirms the table rule) |
| A-LA-2 | Licence audit | Vulkan loader as an OS interface on G67's condition; SPIR-V grammar as standards data; `ash` removed; Agility SDK, `dxil.dll`, DXC, Swappy and the Metal shader converter never shipped; class-5 "Renderer" row corrected (I-2) | GR 9.5; Q31 | Pending (owner confirms) |
| A-LA-3 | Licence audit 2.5 | Non-commercial datasets unused even test-only until Q-CB-1; Tranco's licence unstated; entity-data inputs licence-reviewed | CB 9.5; Q-CB-1, Q-CB-2 | Pending |
| A-RN-1 | [Reconciliation 1][RN] P7 | Registry statement lists G10–G12 and G37–G45 and the full registry (C-5) | LY 9.5; Q19 | Pending (an addendum line) |
| A-RN-2 | Reconciliation 1 P18 | Projection assumes F fixed per process; holds because content GPU processes are keyed like renderers | GR 9.5 | Pending (an addendum line) |
| A-RN-3 | Reconciliation staffing | 384–502 engineers† (437–560 people†) for eight specs after removing one double-counted overlay engineer | GR 9.5 | Applied (RN2 Section 4); superseded for nine specs by C-6 |
| A-FUT-1 | SVG, editing, animation specs | LY 6.5.10 (SVG boundary and text); caret and selection from fragments; scroll ranges, view-transition geometry and `interpolate-size` through the intrinsic memo | LY 9.5 | When written (G-12, G-13, G-16) |
| A-FUT-2 | PDF spec | Uses `hg-jpx`, `hg-jbig2`, `hg-ccitt` (built by MD) inside the PDF process under G50–G53 | MD 9.5 (R18) | When written (G-14) |
| A-FUT-3 | ML service spec | Owns and sizes MD-D25's caption and image-description models | MD 9.5 (R19) | When written (G-15) |
| A-FUT-4 | Animation spec | Owns `hg-interp`, the compositor-eligible record (transform and opacity only), events and `getAnimations()` state; owner named before GR-M1 | GR 9.5 | When written (G-13) |
| A-FUT-5 | SVG spec | Typed render tree consumed by `hg-svg-paint`; the opaque-SVG path uses the same crates | GR 9.5 | When written (G-12) |
| A-FUT-6 | PDF spec | Which raster and display list PDF rendering and the writer use; reconcile Stage 1 13.2's utility process with media's "PDF process" | GR 9.5 | When written (G-14, C-21) |
| A-FUT-7 | Phishing / URL-reputation spec | URL reputation is a separate system with its own owner | CB 9.5; CB-D23, Q-CB-5 | When written (G-22) |

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
