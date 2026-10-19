# Izcal resource footprint: frame

Written 19 October 2026 (phase 1 of the first-principles-spec method).

## The problem and who has it

Browsers are among the largest consumers of memory, CPU wakeups and battery on users' machines, and users notice: they keep many tabs open, run the browser alongside heavy apps, and use low-memory laptops and phones. Izcal now owns every component that does engine work (HGBA-2 2.3.0, M-1), so footprint can be designed in from the start instead of reclaimed afterwards. The owner's direction: no shortcuts for time or complexity; the best design from first principles. This spec designs **minimal memory, CPU, energy, disk and GPU-memory use across the whole browser**, and sets absolute budgets.

Who has the problem:
- **Users**: many tabs on 8–16 GB machines, battery life, responsiveness when memory is tight, background tabs that keep costing.
- **Izcal's engineers**: need budgets per component and per process that a PR can be measured against, and attribution that says where memory goes.
- **Site isolation** (HGBA-2 P1, I-1): one process per site is the security boundary and the largest fixed memory cost; this spec must reduce its cost without weakening it.

## What Izcal's specs already decide or constrain (read these)

- HGBA-2 (`reports/HGBA 2 browser architecture spec.md`, 2.3.0): P1 > P2 > P3 > P4 (performance and energy) > P5 > P6; I-1 (process per site), I-2; Section 4.6 (advisory memory manager: hot/warm/cold/frozen classes, madvise, discard, bfcache, snapshots, broker-allocated shared memory; io_uring excluded) — its V8-specific examples are superseded; X-4 energy.
- Stage 1 design (`reports/HGBA Stage 1 architecture design.md`): process map (Section 1), substrate (Section 3), renderer pipeline (Section 5), bfcache policy BF-1, 13.3 resource targets RES-1 to RES-5 — currently relative to Chrome (RES-3 memory ≤ 1.2× Chrome) and set when Izcal would inherit V8/Stylo/WebRender footprints; to be replaced by absolute budgets.
- The JS engine spec in progress (`research_notes/Izcal JavaScript engine/`): the JS heap and GC are the largest single consumer; this spec's budgets feed it.
- Security scope 0.3 (PR #8): process list and per-process threat models (6.3a), which fix how many processes exist.
- Coding standard 0.3 (PR #9): CS-PF rules (type-size budgets, allocation budgets, instruction-count gate).

## What success looks like (measurable)

- Absolute memory budgets: browser baseline (no tabs), per additional site process, per typical page, for a fixed N-tab corpus scenario — on each reference device, with a stated measurement method (PSS/USS on Linux, phys_footprint on macOS, private working set/commit on Windows).
- Energy: per-corpus-run energy and idle background cost (wakeups/s per background tab).
- Disk: installer and on-disk profile/cache size.
- GPU memory per tab.
- Start-up time and memory under pressure (no correctness loss, P3).
- Attribution: every byte charged to a site, component and cause, visible to users and developers.
