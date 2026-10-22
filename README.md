# Izcal

Izcal is a new Rust-first browser engine, written from the web specifications and first principles (HGBA-2's method constraint M-1), built **from the ground up with no engine dependencies** (HGBA-2 2.4.3, 21 October 2026; the earlier V8-fork plan of this README is superseded history). The accepted specification corpus lives on branch `spec/phase2c`. **HGBA** (Hardware-Growth Browser Architecture) is the name of its architecture and specifications; the product name is Izcal (Stage 1 design, D-S9). The project is at the end of milestone M0: specifications, spikes and build infrastructure, with no engine code yet.

## Specifications

The documents cite each other by ID. This table maps the IDs to files.

| ID | File | Current revision |
|---|---|---|
| HGBA-2 (spec; targets S-, C-, P-, R-, I-, decisions D1–D13) | `reports/HGBA 2 browser architecture spec.md` | 2.2.1, frozen |
| HGBA-3 (direction: oracles, data plane, determinism; B-, DET-, AG-) | `reports/Future software browser engine.md` | 3.2.0, frozen |
| Stage 1 design (process map, crates, IPC, milestones; owner decisions D-S1–D-S11) | `reports/HGBA Stage 1 architecture design.md` | draft 0.12 |
| DevTools spec (native inspection protocol, `hg-devtools`, adapters; targets DT-) | `reports/Izcal DevTools spec.md` | draft 0.3.1 |
| Shell UI design | `reports/HGBA shell UI design.md` | draft 0.3 |

Reviews that fed the specs are also in `reports/`; the evidence behind every report is in `research_notes/<report name>/`.

## Layout

| Path | Contents |
|---|---|
| `reports/`, `research_notes/` | Specifications, reviews and their sources |
| `m0/` | Milestone M0. Start with `m0/M0_SUMMARY.md` |
| `m0/` | M0 measurement results and logs, plus `m0/spike-archive/` for the spikes' results. The fork and the spike code were removed from the tree on 21 October 2026 (they remain in git history) |
| `m0/spike-sandbox`, `m0/v8-cppgc-ext-sandbox`, `m0/determinism-probe`, `m0/fork-api-verify` | M0 spikes and probes, built against the fork |
| `m0/linux`, `m0/windows` | Per-platform M0 results and logs |
| `m0/ci` | CI notes and the local runner used to test the workflow |
| `cppgc-spike/`, `v8-cppgc-ext/` | Pre-M0 originals (sandbox off, crates.io V8). `v8-cppgc-ext` is retired (HGBA-2.1.6 S-4e) |

## Getting the code

```
git clone https://github.com/Rorqualx/izcal.git
cd izcal && tools/setup-hooks.sh
```

Branches, worktrees, commits, checks, code review, merging and pushing are covered in [CONTRIBUTING.md](CONTRIBUTING.md).

Both repositories are public. The V8 archives built in M0 are not in git (see `.gitignore`); their checksums are in `m0/archives/SHA256SUMS` and `m0/linux/SHA256SUMS`.

## License

MIT ([LICENSE](LICENSE)), for code and documents (Stage 1 design, D-S6). Everything Izcal ships, code and data, must be available under MIT (HGBA-2 2.4.3); the exceptions are in CONTRIBUTING.md, "License".
