# Measurements — the program that confirms or replaces every † number

Compiled 21 October 2026, the day the document phase closed. From here, the corpus's †-thresholds and C-6R's Ledger B are confirmed or corrected by measurements, per each spec's own milestone definitions. This folder holds the program's data; the specs own their targets and thresholds.

## The calendar (first wave, in dependency order)

| Milestone | What it measures | Needs | Specs' pass condition |
|---|---|---|---|
| **FP-M0** | Real footprint baselines: process rows, idle wakeups, the 60 MiB-class budgets | A running substrate + renderer skeleton on one OS | The first budget rows measured instead of † |
| **GR-M0** | GPU matrix, integer-pixel identity, the compositor process costs | GPU lab (the GPU matrix), the graphics spec's M0 build | GR-D3's identical-output claim, first devices |
| **JS-M1** | JS engine velocity: WPT subtests per engineer-month, the cage overhead | `hg-js` with the interpreter tier, WPT harness | Replaces the 48–60-month Stage 1 placeholder with measured velocity (Stage 1 8.3) |
| **DV-M0** | The device-access seam, OS-stack grants, the six-family footprint | One desktop OS, the DV-M0 build | DV-D3's OS-stack choice confirmed on real subsystems |
| **AH-M0** | The component-host configuration, canonical-ABI overhead, the app corpus | `hg-js` Wasm + the app host M0 build | AH-D4's WASI 0.2.12 profile on real components |
| **AX-M0** | The accessibility mirror latency, the UI-host placement | The AT lab machines + screen readers | AX-D4's broker-mirror baseline (A-SH-10) and AX-D9's placement (A-SH-11) |

Then the M1/M2/M3 waves each spec defines; C-6R's reversal trigger fires on the first per-activity-class milestones (FP-M0, GR-M0, JS-M1 and the first phase-8 cycle on agentic output).

## Hardware the specs already commit to (shopping list)

- **AT lab** (accessibility spec): at least 8 Windows machines, 3 Macs, 2 Linux, a riscv64 board, an Android set, plus contracted user studies per release; two more Windows machines from AX-D25's decision.
- **riscv64 pool** (JS spec, JS-D22): SpacemiT K3-class RVA23 boards for CI and the cross-ISA identity tests; no hosted CI offers RVA23.
- **GPU matrix** (graphics spec, GR-M0): the device set GR-D3's identical-pixel claim is tested on.
- **Rig policy** (house rules): all heavy or exclusive runs book the rig first (`rig book`), with `rig check --kind exclusive` before every timing-sensitive run; the per-host game-run lock stays in force for game runs. No timing run shares a rig with another heavy job.

## Agentic measurements (C-6R, Ledger B)

`agentic/` holds the recorded agentic-usage rows — tokens, tool calls and wall time per activity class (I implementation, V review) — the data the reversal trigger compares against Ledger B's envelope. The first rows are `agentic/2026-10-21-spec-authoring.md` (the spec-authoring session itself). Every future agentic batch appends its own dated file; when two consecutive components measure outside the envelope on the same side, C-6R reopens and the class ranges are restated from these files, not from the literature.

## Rules

- A measurement that contradicts a † threshold does not silently change it: the spec's own reversal trigger decides, and the owner's decision log records it.
- Every measurement file states what was measured, on what hardware, with what build, and the date — the M0-results discipline from the corpus's `m0/` folder.
- No third-party source code is read to build any measurement (M-1 part 3, HGBA-2 2.4.3).
