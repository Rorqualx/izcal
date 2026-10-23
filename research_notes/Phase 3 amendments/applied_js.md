# Phase 3, wave 1: ledger rows applied to the JS engine spec

Applied 19 October 2026 on branch `spec/hgba2-2.4` to [the JS engine spec][JS], which moves from 0.3.1 to **draft 0.4**. Source: the [Phase 3 amendment ledger][LDG], section "JavaScript engine spec (A-JS)". Each row was checked against the 9.5 row of the spec that raised it and against the reconciliation item that settles it ([RN][RN] P3, P4, P9; [RN2][RN2] Q15, Q16, Q29). Owner decisions were applied as final. The only other files touched are `README.md` (the JS row) and a new research note, [riscv64.md][RV].

## Rows

| Ledger row | Source and settling decision | What changed in the JS spec | Sections |
|---|---|---|---|
| A-JS-1 | TX 9.5; **OD-1**, P3 | `hg-ucd` and the UAX 29, UAX 15 and case-mapping code now belong to the fonts and text spec in L2s (tier P). `hg-intl`, `hg-tz` and `hg-cldr-build` stay here, and `hg-cldr-build` now compiles only CLDR and tz. `Intl.Segmenter`, `normalize` and the case builtins call `hg-unicode`, and one Unicode version is pinned per build. G6 now covers L2e only. G18 now covers values derived from font bytes and the session image (TX's G27). The Intl track drops from 4–5† to 3–4† | 6.13, 6.15 (crate table, intro, G6, G18), 9.3 |
| A-JS-2 | ST 9.5; P9 | The 48 B† core is placed inside the DOM's 72 B†. Its 8 B hold 32 bits of node flags and 32 of element state; the other state bits and `:state()` names go in rare data. Atom text, attribute-set contents (class and id atoms, filter hashes, presentational hints) and inline-style source are stored outside the cage. Style output lives in `hg-style`'s columns. Slot release notifies `hg-style`. The text arena adopts parser buffers without copying | 6.2, 6.5 |
| A-JS-3 | ST 9.5; P4 | New L2s row for `hg-math`: owned here, tier-P build, one implementation per function, no FMA, no libm. One per-agent atom table serves keys and the DOM's tag, attribute and CSS names; only the script thread writes to it. Workers get private atom scopes (ST's G23) | 6.9, 6.13, 6.15 |
| A-JS-4 | NW 9.5 (NW 6.13) | The proxy-resolver utility process (PAC through `hg-pac-host`) is listed with the code-cache builder and the component host as a jitless embedder with no DOM. G6 lets the DOM-less host crates depend on `hg-js-host` (but not on `hg-heap`'s API). G14 now names the builder and the proxy resolver | 6.0, 6.15 |
| A-JS-5 | MD 9.5 (R11); **MD-D31**, MD-S8 | New 6.12 item for real-time agents, with four guarantees: no shared safepoint and collections only at quantum boundaries; allocation without locks; no code installation or W^X change on the thread; joining the OS real-time mechanism through the broker. New target **JS-43** (p99.9 ≤ 0.5 ms† per quantum, plus lock and syscall probes), a JS-10 planted bug, a 9.1 row (per-OS readiness), JS-M2 measurement and Q-JS-22. `hg-math` gains f32 entry points, defined as the f64 result rounded once | 6.9, 6.12, 6.15, 8.4, 9.1, 9.2, 11.1 |
| A-JS-6 | GR 9.5; **Q15** | Graphics' decision points are in a new consolidated table in 6.9. `hg-math` f64 now serves feTurbulence and colour tables. There is one whole-renderer budget (≤ 1%† of p95 frame time) with per-subsystem shares, and JS-52 (a) is restated to match. Q15 names layout, graphics and media, so media's points (MD 6.13) and text's (TX 6.11) are also in the table | 6.9, 6.15, JS-52 |
| A-JS-7 | LY 9.5; **Q16** | Release-by-ID also clears `hg-box-tree`'s node-to-box entry. The stack segments now live in the substrate as one inventoried implementation shared with layout, with ownership split three ways: mechanism (JS), placement (substrate), budget (layout). On any OS where JS-D19 (c) is adopted, LY-D15 falls back to (c) as well. The `hg-wasm` row no longer owns segments. A 9.5 row amends Stage 1 3.1 | 6.5, 6.6, 6.15, 9.5 |
| A-JS-8 | LY 9.5; **Q15** | Layout's points, including the visual viewport, are in the same consolidated table, under the same budget | 6.9, JS-52 |
| A-JS-9 | CB 9.5; **CB-D10** | New 6.10 row for API-call attribution: the responsible script is the innermost non-builtin frame, and code created by `eval`, `Function`, string timers, `javascript:` URLs, inserted inline scripts and same-origin child documents inherits the creator's restriction. The attribution record is held outside the cage. JS-60 checks it against CB-27 within the same overhead budget, and JS-10 gains a planted bug | 6.10, JS-10, JS-60 |
| A-JS-10 | FP 9.4 (FP 11.3) | FP 11.3 #1–6 were already resolved in JS 0.2, and #7 stands. This pass updates the stale references: the header cites footprint 0.3, FP-D8 is now "accepted" (6.7), and FP-D13 (a) is now accepted, with the (b) wording removed (6.13) | Header, 6.7, 6.13 |

## Owner decision: riscv64 from the start (19 October 2026)

This settles ledger gap **G-4**: X-1 is kept, and the engine meets it. The design work is recorded as **JS-D22** (Section 6.18), and no owner question remains open. It changes the following sections:
- 4.2: the platform primitives now include riscv64.
- 6.1: decompression with `sh2add.uw`.
- 6.2a: an explicit overflow check, since riscv64 has no flags register.
- 6.3: T0 on three ISAs.
- 6.4: Zicfiss, and Sail semantics for rule verification.
- 6.6: Linux riscv64 shadow-stack switching.
- 6.7: Zicfilp landing pads.
- 6.9: the canonical-NaN row.
- 6.17: a new alternatives row.
- 6.18: the whole riscv64 design.
- 7: the anchor row and Sail as an oracle under JS-D9's class.
- 8: JS-1, JS-11, JS-15 and JS-50 now cover three ISAs, and new target JS-28.
- 9.1: the "RISC-V later" row is replaced by a row for CFI silicon.
- 9.2: riscv64 is added to JS-M0 to JS-M3, and JS-M5 no longer carries a RISC-V backend.
- 9.3: +5–7† engineers (65–82† in total), a ramp of 22–27† to about 43†, and about 285–500 engineer-years†.
- 9.4: JS-D22.
- 9.5: rows for HGBA-2 X-1 and I-1, Stage 1 7.x, 13.2 and 3.1, the security scope, the footprint spec and the coding standard.
- 10: two new risks.
- 11: Q-JS-4 is narrowed, Q-JS-19 to Q-JS-21 are added, and a block of 0.4 unverified claims is added.

Evidence is in [riscv64.md][RV].

## Conflicts and items not resolved here

*(State at application time, 19 October 2026. **Update, 22 October 2026:** C-6 is settled — the ledger's about 635–845 people† full strength, with the JS figure at 65–82†; and C-11 was settled on 21 October 2026 as recommended (HGBA-2 2.4.1).)*

- **C-6, staffing totals:** still unresolved. The JS figure in that total changes from 61–76† (or 60–75† after OD-1) to **65–82†** with riscv64, so the Stage 1 8.3 statement must use the new figure.
- **C-11, "jitless by default":** unchanged. The JS spec still says what it said before: Izcal-only renderers and the banking container are jitless, and other renderers default to full. The security scope's rule is still to be re-decided when that scope is rebased.
- **C-7 / Q29:** the JS spec now states that Stage 1 exit is 48–60 months† and that JS-M3 has about 9 months of float. The riscv64 tiers are planned inside that float, staffed separately, and not added after it.
- **HGBA-2 X-1 timing:** X-1 is a Stage 3 target in HGBA-2. JS-D22 reaches its threshold at JS-M3, at Stage 1 exit. The 9.5 row asks HGBA-2 2.4.0 to restate X-1, and that row is not applied here.

[JS]: ../../reports/Izcal%20JavaScript%20engine.md
[LDG]: ledger.md
[RN]: ../Phase%202%20reconciliation/reconciliation.md
[RN2]: ../Phase%202%20reconciliation/reconciliation_2.md
[RV]: ../Izcal%20JavaScript%20engine/riscv64.md
