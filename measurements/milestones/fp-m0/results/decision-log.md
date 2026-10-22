# FP-M0 decision log

Started 22 October 2026. Every entry that sets or moves a † threshold is recorded here item by item, per the footprint spec's rule — never as a lump.

| Date | Budget or question | Finding | Old value | New value / effect |
|---|---|---|---|---|
| 2026-10-22 | Smoke run, harness check (not a threshold) | Chrome 155, macOS, main process only, one blank tab, 12 samples over 1 min: phys_footprint 120–126 MB, peak 141 MB. Caveat: main process only; the tree-wide matrix (all browser processes) is the next step, and the harness will gain a tree-sum mode before the real matrix runs | — | Harness works; tree-sum mode queued |
| 2026-10-22 | Harness correctness (review findings, not a threshold) | The advisory AI review found three harness defects: an unreachable pgrep fallback, a README usage mismatch, and TSV columns not matching the header. All three fixed (sample-footprint.sh v2): the fallback runs, the README states app-name/PID usage and the single-process scope, and the TSV carries five declared columns (t, sample, pid, phys_footprint_MB, peak_MB). Validated: 4 samples at 126/168 MB, clean columns | — | v2 harness is the canonical sampler; tree-sum mode still queued |

## The first matrix (22 October 2026, macOS, rig local) — v4, shipped

Methodology: fresh disposable profiles per browser; Chrome tabs created through its DevTools endpoint (`/json/new`, renderer-backed targets — direct launches create no windows in this environment, which invalidated v1) and **verified by page count** (1, 11 and 41 pages for the 1/10/30-tab loads); Firefox tabs by direct invocation with the process-tree growth from v1 as the tab-opening evidence, its page-count check queued (Firefox's debugger endpoint does not serve Chrome-style `/json/list` under the 2026 flags tried); tree sampling walks the process tree by parentage with a final `sort -u` dedup and **normalizes the footprint unit** (the CLI prints KB/B for small processes; v3 dropped the unit and inflated rows — the v3 TSVs are discarded and the v3 "startup spike" was a misparse, not a spike); 2-minute samples, steady state = the last 12 samples. All loads are example.com tabs.

| Load | Chrome 155 | Firefox |
|---|---|---|
| Blank | **162 MB** (5 procs) | **639 MB** (11 procs) |
| 1 tab | 276 MB (7) | 703 MB (12) |
| 10 tabs | 644 MB (17) | 782 MB (15) |
| 30 tabs | **1,729 MB** (47) | **840 MB** (15) |

**Reading (first, provisional):** Chrome starts ~4x cheaper but pays ~52 MB per same-site tab (one renderer per tab); Firefox's floor is ~640 MB with same-site tabs nearly free (~7 MB each, shared content processes). For FP-1's budgets the load-bearing rows are the **blank floor** (the incumbent floor Izcal's own floor must be argued against) and the **30-tab slope** (FP-S30's target). Neither is a threshold — they are FP-M0's incumbent observations, the baseline the substrate floor will be compared against.

**Caveats recorded:** same-site-only tabs (cross-site mixes come later); one machine (Apple silicon, macOS 26); Firefox's page-count verification queued (its rows rest on v1's process-growth evidence plus the unit-corrected sampler); Safari queued (single-instance model). **Discarded versions:** v1 (Chrome opened no windows; invalid loads), v2 (`local -A` is not bash-3.2), v3 (unit-suffix drop corrupted KB-scale rows; the data was discarded, the methodology kept).

## MADV_FREE_REUSABLE probe (22 October 2026, macOS 26, Apple silicon)

FP-M0 open question 17, the macOS part. A standalone 30-line probe (task_info's phys_footprint, no third-party source) allocates 256 MB, touches it, applies the advice values and reports after each step:

| Step | phys_footprint |
|---|---|
| After touch (256 MB resident) | 263,472 KB |
| After `MADV_FREE_REUSABLE` | **1,328 KB** |
| After re-touching the same region | 1,328 KB (unchanged) |
| After `MADV_FREE` (comparison) | 1,328 KB |

**Finding:** `MADV_FREE_REUSABLE` releases the `phys_footprint` charge on this OS — the spec's FP-1 macOS release path works as intended. The re-touch observation (no re-charge within the run) is recorded as observed, not explained; a longer-observation variant is queued if any budget depends on the re-charge rate. Probe: `harness/madvise-probe.c` (build: `clang -O2`).

## Cross-site mix (22 October 2026, macOS, rig local)

30 tabs across 10 distinct domains (example.com, example.org, iana.org, wikipedia.org, archive.org, openstreetmap.org, gutenberg.org, ietf.org, w3.org, python.org — three tabs each), Chrome's page count verified (30), Firefox by direct invocation with the queued page-count caveat, same sampler and steady-state rule as the v4 matrix.

| Load | Chrome | Firefox |
|---|---|---|
| 30 same-site tabs (v4) | 1,729 MB (47 procs) | 840 MB (15 procs) |
| 30 cross-site tabs | **1,830 MB (36 procs)** | **2,066 MB (41 procs)** |

**Reading:** Firefox's same-site economy disappears across sites (+1,226 MB, 15→41 processes — each distinct domain gets its own content process); Chrome's cross-site cost is mild (+101 MB) because its per-tab process model was already paying. The FP-S30 budget's worst case is the cross-site row, not the same-site one — the corpus's FP-S30 caveat ("≤ 10–12 cross-site frames per iframe-heavy page") now has its first incumbent baseline. Neither row is a threshold; both are FP-M0 observations.
