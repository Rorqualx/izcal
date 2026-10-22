# FP-M0 decision log

Started 22 October 2026. Every entry that sets or moves a † threshold is recorded here item by item, per the footprint spec's rule — never as a lump.

| Date | Budget or question | Finding | Old value | New value / effect |
|---|---|---|---|---|
| 2026-10-22 | Smoke run, harness check (not a threshold) | Chrome 155, macOS, main process only, one blank tab, 12 samples over 1 min: phys_footprint 120–126 MB, peak 141 MB. Caveat: main process only; the tree-wide matrix (all browser processes) is the next step, and the harness will gain a tree-sum mode before the real matrix runs | — | Harness works; tree-sum mode queued |
| 2026-10-22 | Harness correctness (review findings, not a threshold) | The advisory AI review found three harness defects: an unreachable pgrep fallback, a README usage mismatch, and TSV columns not matching the header. All three fixed (sample-footprint.sh v2): the fallback runs, the README states app-name/PID usage and the single-process scope, and the TSV carries five declared columns (t, sample, pid, phys_footprint_MB, peak_MB). Validated: 4 samples at 126/168 MB, clean columns | — | v2 harness is the canonical sampler; tree-sum mode still queued |
