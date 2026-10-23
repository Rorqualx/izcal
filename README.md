# Izcal

Izcal is a new Rust-first browser engine, written from the web specifications and first principles (HGBA-2's method constraint M-1), built **from the ground up with no engine dependencies** (HGBA-2 2.4.3, 21 October 2026; the earlier V8-fork plan of this README is superseded history). The corpus of accepted specifications lives on branch `spec/phase2c`. **HGBA** (Hardware-Growth Browser Architecture) is the name of its architecture and specifications; the product name is Izcal (Stage 1 design, D-S9). Milestone M0 (specifications, spikes, build infrastructure) is complete, and engine code has begun: the substrate crate `hg-substrate-os` and the FP-M0 half-B empty-process probe (measurements/milestones/fp-m0/).

## Specifications

The documents cite each other by ID. This table maps the IDs to files.

| ID | File | Current revision |
|---|---|---|
| HGBA-2 (spec; targets S-, C-, P-, R-, I-, decisions D1–D13) | `reports/HGBA 2 browser architecture spec.md` | 2.2.0, frozen |
| HGBA-3 (direction: oracles, data plane, determinism; B-, DET-, AG-) | `reports/Future software browser engine.md` | 3.2.0, frozen |
| Stage 1 design (process map, crates, IPC, milestones; owner decisions D-S1–D-S11) | `reports/HGBA Stage 1 architecture design.md` | draft 0.11 |
| DevTools spec (native inspection protocol, `hg-devtools`, adapters; targets DT-) | `reports/Izcal DevTools spec.md` | draft 0.3 |
| Shell UI design | `reports/HGBA shell UI design.md` | draft 0.3 |

Reviews that fed the specs are also in `reports/`; the evidence behind every report is in `research_notes/<report name>/`.

## Layout

| Path | Contents |
|---|---|
| `reports/`, `research_notes/` | Specifications, reviews and their sources |
| `m0/` | Milestone M0: start with `m0/M0_SUMMARY.md`. The fork and the spike code were removed from the tree on 21 October 2026 (they remain in git history; the V8 archive checksums stay in `m0/archives/SHA256SUMS`) |
| `m0/linux`, `m0/windows` | Per-platform M0 results and logs |
| `m0/ci` | CI notes and the local runner used to test the workflow |

## Getting the code

```
git clone https://github.com/Rorqualx/izcal.git
cd izcal && tools/setup-hooks.sh
```

Branches, worktrees, commits, checks, code review, merging and pushing are covered in [CONTRIBUTING.md](CONTRIBUTING.md).

The V8 archives built in M0 are not in git (see `.gitignore`); their checksums are in `m0/archives/SHA256SUMS` and `m0/linux/SHA256SUMS`.

## License

MIT ([LICENSE](LICENSE)), for code and documents (Stage 1 design, D-S6). Everything Izcal ships, code and data, must be available under MIT (HGBA-2 2.4.3); the exceptions are in CONTRIBUTING.md, "License".
