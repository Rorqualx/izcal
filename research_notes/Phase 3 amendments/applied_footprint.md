# Phase 3 wave 1: footprint spec rows applied

19 October 2026, branch `spec/hgba2-2.4`. Target: [resource footprint spec][FP], draft 0.3.1 → **0.4**. Source: the [ledger][LG], section "Resource footprint spec (A-FP)", 40 rows; the source specs' 9.5 (9.4 for FP) rows; [reconciliation 1][RN] (P9–P19, OD-2 to OD-4) and [reconciliation 2][RN2] (Q3, Q7, Q8, Q11, Q27, OD-5); the owner decision of 19 October 2026 that riscv64 is a target from the start (the ledger's G-4). Only the footprint spec, the README's footprint cell and this file were edited.

## Rows

| Row | Source; decision | What changed in FP 0.4 | Sections |
|---|---|---|---|
| A-FP-1 | JS-D21 | Nothing: already applied at 0.3 (FP-39, code-cache builder); confirmed | 6.12, 8.2, FP-39 |
| A-FP-2 | JS 0.1 rows | Nothing: already applied at 0.2; confirmed | 6.3, 6.4, 6.9 |
| A-FP-3 | ST 9.5; P9, P18, P19 | Computed style is content-addressed groups and match keys in per-agent tables; unmodified parsed sheets are class (a), keyed (partition, URL, hash), keep their decoded bytes and never refetch; CSSOM-mutated and constructed sheets, inline style and transition state are primary; FP-11 split 72 / 80 / 360; ST-34's 64 KiB inside the `about:blank` line; partition = (T, C, X, n) | 6.3, 6.4, 8.2, FP-11, FP-12 |
| A-FP-4 | ST-D17; P19 | Compiled user style sheets are an FP-D16 member (shell renderer writes; never the only copy) | FP-D16 |
| A-FP-5 | ST-13 | Parsed sheets, rule indexes, style arenas, attribute sets and `:visited` in 6.18.1 and FP-36 | 6.18.1, FP-36 |
| A-FP-6 | TX 9.5; P12 | Inventory: session image in two parts, owners `hg-image` (build time) and `hg-font-service` (per session), format `.hgfont`, ready within TX-43; Izcal font set row; rule 5 rewritten as "hashes against manifests, never parses" | 6.3 |
| A-FP-7 | TX 9.5; P12 | Font service as a utility job (peak ≤ 8 MiB†, 0 idle, outside FP-1's idle sum); granted sealed copies charged to the granting (site, container); Linux "interim OS fonts" ledger line; 8.2 row | 6.3, 8.2, FP-1 |
| A-FP-8 | TX 9.5; P13 | New ledger category "attributed OS code": third-party IME in the broker, outside FP-1, inside FP-10; FP-1 measured with no third-party IME | 6.2, 8.2, FP-1, FP-10 |
| A-FP-9 | TX 9.5; P9 | FP-11 excludes the text-shaping category (glyph runs wherever held, including layout's item lists) | 6.4 (Text), FP-11 |
| A-FP-10 | TX 9.5; P19 | Font-hash cache is an FP-D16 member | FP-D16 |
| A-FP-11 | TX 9.5; OD-4, P18, P19 | Canonical-exempt sub-class of class (a) with four rules (fixed cap, transparency, own ledger line, class (a) release), outside CT, CC and FP-24; FP-33 adds the caps; members: shaping cache ≤ 512 KiB†, raster cache ≤ 1 MiB†, variation instances (cap open, Q18); sharing exception for the Izcal set and OS font files on protected volumes | 6.2, 6.4, FP-24, FP-33 |
| A-FP-12 | TX 9.5 (R10); P15 | Atlas as A8 and RGBA8 pools, each its own line, fixed per build and identical on every device, sized by TX-52; together ≤ 2 MiB† until then (held item in 9.1) | 6.6, 8.2, FP-14, 9.1 |
| A-FP-13 | TX 9.5 (R11); OD-3, P14 | Font set in FP-D13's data set as one signed package; FP-25 lines ≤ 40 MiB† (TX-W1), ≤ 350 MiB† installed, ≤ 200 MiB† compressed; OS interim fonts outside; FP-D13 (a) kept; FP-D13's reversal trigger noted as fired and answered by OD-3, its second half now measuring the data set without the font package | 6.3, 6.12, 8.2, FP-25, FP-D13 |
| A-FP-14 | TX 9.5 (R9); TX-D21 | Closed: raster cache, variation instances, lazy validation; bounded: font-file residency, ≤ 1 bit/s† (TX-21) | 6.18.1, FP-36 |
| A-FP-15 | NW 9.5; P10, P11 | FP-1's network line itemized (system ≤ 4, spare ≤ 1.5, default container service ≤ 2 from first navigation, within 8); 8.2 rows for container, partition (2 / 3 / 1.25), OS adapter, proxy resolver, route process | 8.2, FP-1 |
| A-FP-16 | NW 9.5; P10 | Model term S_net (and N_net); B keeps the system service and spare | Summary, 6.1 |
| A-FP-17 | NW 9.5; OD-2 | FP-7 measures renderer isolation; network runtime reported separately under NW-35 | 6.10, 8.2, FP-7 |
| A-FP-18 | NW 9.5; P10 | Network totals published beside FP-6: 90 MiB at settle, 39.25 after 10 min | 8.1, 8.2, FP-6 |
| A-FP-19 | NW 9.5; P10; C-14 | Merged with A-FP-39 (below) | 6.11, 8.2, FP-28 |
| A-FP-20 | NW, ST, TX 9.5; P18; C-2 | Every in-process cache, every per-key service process's caches and the atlas key by the (T, C, X, n) projection | 4.3, 6.2, 6.4, 6.6, 6.18.1, FP-D14 |
| A-FP-21 | NW-D11; C-12 | HTTP cache is an FP-D16 member, bound to format version and a broker-issued security epoch; crash-discard kept | 6.12, FP-D16 |
| A-FP-22 | NW 9.5 | Frozen pages keep WebSocket and WebTransport sessions without wakeups (closed only under pressure); PING only with a request; WebRTC consent checks only while live; Web Push under a grant; NW-33; FP-17's suite. Suspend eligibility now excludes pages with open sessions instead of saying frozen pages close them | 6.8, 6.9, FP-17 |
| A-FP-23 | NW 9.5 | 6.18.1 rows: connection pools and socket budgets, mint queue, receive buffer and decryption queue, HTTP-cache eviction | 6.18.1 |
| A-FP-24 | NW 9.5; NW-D26; C-13 | Lists are Izcal's own, produced by NW's lists track under CB's governance; shell keeps the per-container choice; user lists compiled in the utility process | 6.11 |
| A-FP-25 | MD 9.5 (R14); Q7, Q11 | Media, hardware-decode and CDM processes per renderer key; FP-38 renamed and restated (pixels reach the content GPU process of the embedding key); FP-D11 annotated | 6.1, 6.5, FP-38, FP-D11 |
| A-FP-26 | MD 9.5 (R14); Q8 | 8.2 rows: media process while decoding (MD-53 formula), hardware-decode ≤ 8 MiB†, audio ≤ 4 MiB† / 0, capture ≤ 6 MiB† + frames, CDM as attributed OS code (MD-S5); lab measures every kind | 6.2, 6.16, 8.2 |
| A-FP-27 | MD 9.5 (R14); G56 | Opaque encoded bytes once per partition in the media process; class W at 8 B/px in FP-13; intrinsic-size decodes for readback charged as canvas readback; "video" kind carries hardware frames | 6.5, 6.6, FP-13, FP-14 |
| A-FP-28 | MD-D23 | Audible-media reason: video decode suspended while hidden; hidden muted media suspended without events, page frozen | 6.8 |
| A-FP-29 | MD 9.5; Q27 | MD-54 and MD-55 as absolute FP-22-style gates; 8.2 rows | 6.13, 8.2, FP-22 |
| A-FP-30 | MD-D3 | FP-D13 gains codec tables, the HRTF set and named OS-dependent exceptions (H.264 and AAC by OS family, Windows HEVC and N/KN bits); FP-25 counts the AOM notice | 6.12, FP-25, FP-D13 |
| A-FP-31 | GR 9.5; OD-5, Q8 | S_key term; content GPU process lifecycle row (opportunity start, device-less ≤ 2 MiB†, ≤ 12 MiB† with device, volatile hidden, exits 30 s†, device-less spare); shell's device-less process in FP-1 | 6.1, 6.6, 8.2, FP-1 |
| A-FP-32 | GR, MD 9.5; OD-5 | FP-7: per-key service processes on their own line, never co-located | 6.10, 8.2, FP-7 |
| A-FP-33 | GR 9.5; RN2 Section 3; C-3 | FP-S30 recomputed and recorded as an open measurement (9.1, 10, Q20) | 8.2, 9.1, 10, 11.1 |
| A-FP-34 | GR 9.5 | View Transition snapshots, OffscreenCanvas surfaces, shared-tile pinning on their own lines; ≤ 16† compositor-mutable nodes; prepaint inside 2.5 viewports | 6.6, 8.2, FP-14 |
| A-FP-35 | GR 9.5 | Adaptive swapchain rule on desktop; Android after Q-GR-15 | 6.6 |
| A-FP-36 | GR-D23; Q3; C-4 | Display items in the content GPU process count in paint's 160 B† (about 72 + ≤ 88); one failure order in FP-11 | 6.4, FP-11 |
| A-FP-37 | GR 9.5; Q11, Q27 | Atlas per renderer key and partition; FP-27 and 8.2 gain the content GPU spawn row (GR-27, ≤ 50 ms† p95); 6.5's region owner is the content GPU process of the embedding key; GR-43 an absolute gate | 6.5, 6.6, 8.2, FP-14, FP-22, FP-27 |
| A-FP-38 | LY 9.5 | Layout store table (box columns, fragment store, caches and memos, item lists, `hg-layout-state` primary); safepoint outside a flush; (constraint-space id, break-token id) keying with pass-count eviction; (id, epoch) handles; live records only; 200 / 160 B† no longer provisional; LY-31 | 6.3, 6.4, 6.9, 8.2, FP-11, FP-12 |
| A-FP-39 | CB-18, CB-23; C-13, C-14 | FP-28 restated: bundle ≤ 8 MiB† clean and shared, ≤ 256 KiB† charged per partition service, p99 ≤ 20 µs† all phases, cold start ≤ 250 µs†, CB-23 compile budgets; governance to CB | 6.11, 6.12, 8.2, FP-28, 10 |
| A-FP-40 | CB-D10, CB-D11; C-12 | Compiled filter files keyed by source-list hashes and compiler version (user lists per container); restricted scripts under FP-D7's caps without a report | 6.3, 6.14, FP-D7, FP-D16 |

**riscv64 (owner decision, 19 October 2026).** Lab: an RVA23 riscv64 reference device of the SpacemiT K3 class (HGBA-2 X-1) and riscv64 CI runners at FP-M0 (8.1, FP-M0). CI: FP-24 identity per ISA on three ISAs (Linux only on riscv64) and across ISAs (6.2, 6.16, FP-24); FP-33 per ISA. ISA-dependent budgets marked **[ISA]** in 8.2: FP-5 and FP-35 heaps (machine code), FP-25 installed code, FP-21, FP-22, MD-54, GR-43 (energy), FP-27 and GR-27 (start-up and spawn), CB-18 and CB-23 (match and compile time). Sv39: reservations follow JS 0.4's guarded-reservation count (JS-D22, JS 6.18), so FP-D19's identical limits hold; cost measured at FP-M0 (6.20.6, Q19). Items for the JS spec in FP 11.3 (8–10). These citations rely on the JS engine spec's draft 0.4, being written in the same worktree in parallel.

## Budget arithmetic after the pass (FP 8.2)

| Budget | Result |
|---|---|
| FP-1 ≤ 64 | Line budgets 20 + 14 + 12 + 2 + 8 + 4 + 2 + 1 = 63; at idle 60.5 (network 5.5). Holds |
| FP-S30 at settle ≤ 1,280 (10 frames per iframe-heavy page) | 64 + 960 + 120 + 90 + 14 = 1,248; iframe-heavy visible 1,268. Holds |
| FP-S30 at 15 frames | 1,308–1,338. Breaks by 28–58; holds to ≤ 12 frames (median visible) or ≤ 10 (iframe-heavy visible). Open measurement, C-3 |
| FP-6 ≤ 768 | 617–637 at 10 frames; 677–707 at 15. Holds |
| FP-7 corpus ≤ 5% (renderer only) | Allowance about 5.6 MiB of about 111: ≤ 1.4 cross-site frame keys per page. At risk until FP-M3 |
| FP-7 iframe-heavy ≤ 25% | Allowance about 43.8 of 175: holds to 10 frames (22.9%), fails at 11 (25.1%) and 15 (34.3%). Open measurement (new finding of this pass: RN2 checked 10 frames only) |
| FP-11 | 72 + 80 + 200 + 160 = 512. Holds at expected values |
| FP-28 | 30 × 0.25 = 7.5 MiB charged at FP-S30, inside the partition services' lines; bundle clean. Holds |
| FP-25 installed per architecture | 120 + 24 + font set: 184 (TX-W1) to 494 (complete) |

## Unresolved and open items

- **No unresolved conflict between amendments.** Every overlap (A-FP-3/9/36/38 on FP-11, A-FP-19/39 on FP-28, A-FP-24/39 on 6.11, A-FP-21/40 with FP-D16's partition rule, A-FP-17/32 on FP-7, A-FP-12/20/37 on the atlas) is settled by P9, Q3, C-14, C-13, C-12 (P19, NW-D11, CB-D11), OD-2, OD-5, P18 or Q11.
- **C-12's per-member binding** is applied as the ledger recommends ("bound to its format version and writer; partitioned per (T, C) whenever it holds user- or site-derived state"); the ledger marks this a recommendation, not an owner decision. Each member's binding comes from its own accepted spec.
- **Variation-instance cache cap** (OD-4 needs a fixed cap; TX states none): FP Q18. Interim placement: counted in CT and CC.
- **Interim OS fonts line** (Linux, TX-D25): own ledger line, no budget: FP Q21.
- **CDM process memory**: attributed OS-code line, unmeasured until MD-S5.
- **FP-S30 and FP-7 at iframe-heavy counts** above 10–12 frames and FP-7's corpus limit: open measurement for FP-M3 (C-3, FP Q20); no owner decision exists, and the pass did not set one.
- **FP-D13's reversal trigger**: its second half fired with the font set (P14) and OD-3 kept (a); 0.4 records that the trigger now measures the data set without the font package. This reads OD-3; if the owner meant otherwise, only the trigger wording changes.

[FP]: ../../reports/Izcal%20resource%20footprint.md
[LG]: ledger.md
[RN]: ../Phase%202%20reconciliation/reconciliation.md
[RN2]: ../Phase%202%20reconciliation/reconciliation_2.md
