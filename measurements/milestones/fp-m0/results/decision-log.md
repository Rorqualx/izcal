# FP-M0 decision log

Started 22 October 2026. Every entry that sets or moves a † threshold is recorded here item by item, per the footprint spec's rule — never as a lump.

| Date | Budget or question | Finding | Old value | New value / effect |
|---|---|---|---|---|
| 2026-10-22 | Smoke run, harness check (not a threshold) | Chrome 155, macOS, main process only, one blank tab, 12 samples over 1 min: phys_footprint 120–126 MB, peak 141 MB. Caveat: main process only; the tree-wide matrix (all browser processes) is the next step, and the harness will gain a tree-sum mode before the real matrix runs | — | Harness works; tree-sum mode queued |
