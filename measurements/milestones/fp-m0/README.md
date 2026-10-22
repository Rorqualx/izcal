# FP-M0 — the footprint floor and the incumbent observations

Milestone folder for the resource-footprint spec's FP-M0, started 22 October 2026. The spec's own definition (footprint 0.4.5, Section 9):

> FP-M0 builds an empty process on Izcal's substrate, with the full sandbox of security scope 6.7, on all three OSes to measure the floor that Izcal does not control (page tables, loader, sandbox set-up, OS framework init) before any engine code exists. **FP-2's ship blocker is then set as that measured floor plus Izcal's own line items**, recorded in the decision log item by item, never as a lump.

Plus the black-box observations 0.3.1 added: which OS counters each incumbent's reported footprint follows, per-element memory cost, reaction to Linux PSI, macOS accounting after `MADV_FREE_REUSABLE`, HTTP-cache sizing (open question 17), and spawn latency of a sandboxed child on macOS and Windows (the spare-process gate: keep a spare only while spawn-to-ready exceeds 10 ms†).

## Two halves

**A. Incumbent black-box observations** — runnable now, no Izcal code. Incumbents are signals, never sources (M-1 part 2): launch Chrome, Firefox and Safari with controlled loads, sample the OS counters, and record which reported figure each counter feeds. Runs on this Mac first (rig `local`), then the Linux (.185) and Windows (.253) rigs.

**B. The substrate floor** — needs the first code milestone: an empty process on Izcal's substrate with the security scope 6.7 sandbox, on macOS first, then Linux and Windows. Gated on the substrate skeleton; the harness here is reused as-is (it samples any PID).

## The harness

`harness/sample-footprint.sh <app-name|pid> <url-or-empty> <minutes> [label]` — single-process sampler (the smoke-validation tool; superseded for browsers by the tree sampler). `harness/sample-tree.sh <pgrep-patterns> <minutes> <label>` — the tree sampler: roots match the patterns, then the walk follows parentage breadth-first with a final dedup; columns are t, sample, nprocs, total_phys_MB, total_peak_MB, max_single_MB. `harness/run-matrix.sh` — the macOS incumbent matrix (Chrome via its DevTools `/json/new` endpoint with verified page counts; Firefox by direct invocation; both in disposable profiles). Both samplers are bash-3.2-portable (no `local -A`, no `seq`). Linux and Windows flavours of the samplers come with their rig runs.

## Decision-log discipline

Every FP-M0 finding that sets or moves a † threshold is recorded in `results/decision-log.md` item by item (which budget, old value, measured value, date, hardware) — never as a lump — per the spec's rule. A measurement that contradicts a threshold does not change it silently: the spec's reversal trigger and the owner's decision log decide.

## Order of work

1. Harness + smoke run on one incumbent here (22 October 2026).
2. Full incumbent matrix on macOS (3 browsers × idle / 1 tab / 10 tabs / 30 tabs).
3. The same matrix on Linux and Windows rigs (booked per the rig rules).
4. Substrate floor once the empty-process milestone builds (macOS first).
5. Spawn-latency and `MADV_FREE_REUSABLE` probes (small standalone binaries, no engine needed).
