# Izcal

Izcal is a new Rust-first browser engine, written from the web specifications and first principles (HGBA-2's method constraint M-1). It is written from the ground up: Izcal builds its own JavaScript engine, style system, layout, text stack and renderer, uses no other browser's code or components, and depends only on generic infrastructure. **HGBA** (Hardware-Growth Browser Architecture) is the name of its architecture and specifications; the product name is Izcal (Stage 1 design, D-S9). The document phase is closed: **24 component specifications** exist with every decision accepted, the architecture trio stands at HGBA-2 2.4.4 (frozen), HGBA-3 3.3.3 (direction) and Stage 1 design 0.14.5 (draft), and the project is at the end of milestone M0, with no engine code yet.

## Specifications

The documents cite each other by ID. This table maps the IDs to files.

| ID | File | Current revision |
|---|---|---|
| HGBA-2 (spec; targets S-, C-, P-, R-, I-, decisions D1–D13) | `reports/HGBA 2 browser architecture spec.md` | 2.4.4, frozen (no superseded section) |
| HGBA-3 (direction: oracles, data plane, determinism; B-, DET-, AG-) | `reports/Future software browser engine.md` | 3.3.3, direction document |
| Stage 1 design (process map, crates, IPC, milestones; owner decisions D-S1–D-S12) | `reports/HGBA Stage 1 architecture design.md` | draft 0.14.5 |
| DevTools spec (native inspection protocol, `hg-devtools`, adapters; targets DT-) | `reports/Izcal DevTools spec.md` | draft 0.3.7, decisions accepted |
| JavaScript engine spec (Izcal's own JS and Wasm engine and the GC shared with the DOM, on x86-64, AArch64 and riscv64; targets JS-) | `reports/Izcal JavaScript engine.md` | draft 0.4.4, decisions accepted; riscv64 first-class (JS-D22) |
| Resource footprint spec (memory, wakeups, energy, disk and GPU budgets, per ISA on x86-64, AArch64 and riscv64; targets FP-) | `reports/Izcal resource footprint.md` | draft 0.4.5, decisions accepted |
| HTML and CSS parsing and style spec (HTML, XML and CSS parsers, selectors, cascade, invalidation; targets ST-) | `reports/Izcal HTML and CSS parsing and style.md` | draft 0.3.7, decisions accepted |
| Fonts and text spec (font parsing, shaping, Unicode, rasterization, text input, Izcal's own font set; targets TX-) | `reports/Izcal fonts and text.md` | draft 0.3.7, decisions accepted |
| Networking spec (HTTP/1.1, HTTP/2, HTTP/3 and QUIC, TLS and cryptography, the root program, DNS, Fetch, cookies, HTTP cache; targets NW-) | `reports/Izcal networking.md` | draft 0.3.8, decisions accepted |
| Layout spec (box tree, formatting contexts, fragmentation, incremental layout, hit testing; targets LY-) | `reports/Izcal layout.md` | draft 0.4.2, decisions accepted |
| Images and media spec (image decoders, colour and HDR, codecs, audio, WebCodecs, capture; targets MD-) | `reports/Izcal images and media.md` | draft 0.4.3, decisions accepted |
| Graphics and rendering spec (GPU abstraction, shaders, rasterizer, compositor, WebGPU, WebGL, Canvas; targets GR-) | `reports/Izcal graphics and rendering.md` | draft 0.4.4, decisions accepted |
| Content blocking spec (protection layers, matcher, Izcal's own lists and their governance; targets CB-) | `reports/Izcal content blocking.md` | draft 0.3.5, decisions accepted |
| DOM, bindings and event loop spec (single DOM copy, Web IDL bindings, the event loop, agents; targets DM-) | `reports/Izcal DOM, bindings and event loop.md` | draft 0.2.7, decisions accepted |
| Accessibility spec (accessibility tree, OS adapters, Izcal's own, AT lab; targets AX-) | `reports/Izcal accessibility.md` | draft 0.2.7, decisions accepted |
| PDF viewer and writer spec (sandboxed PDF process, print-to-PDF; targets PD-) | `reports/Izcal PDF.md` | draft 0.2.5, decisions accepted (PD-D8 decided 20 October 2026) |
| SVG spec (SVG document model, scriptless host, SMIL split; targets SV-) | `reports/Izcal SVG.md` | draft 0.3, decisions accepted |
| Animation spec (Web Animations, transitions, SMIL timing, View Transitions; targets AN-) | `reports/Izcal animation.md` | draft 0.3.1, decisions accepted |
| Editing spec (editing model, IME, Izcal's own spelling; targets ED-) | `reports/Izcal editing.md` | draft 0.3.2, decisions accepted |
| Storage spec (storage service, Izcal's own storage engine, quotas; targets SG-) | `reports/Izcal storage.md` | draft 0.3.3, decisions accepted |
| WebCrypto spec (SubtleCrypto surface, entropy, constant-time additions; targets WC-) | `reports/Izcal WebCrypto.md` | draft 0.3, decisions accepted |
| Extensions spec (WebExtensions surface, MV3, published trust process; targets EX-) | `reports/Izcal extensions.md` | draft 0.4.3, decisions accepted |
| ML service spec (sandboxed inference service, runtime and kernels, Stage 1 models and their grants; targets ML-) | `reports/Izcal ML service.md` | draft 0.3.1, decisions accepted |
| Shell spec (shell processes, prompt platform, UI host and recovery window, settings; targets SH-) | `reports/Izcal shell.md` | draft 0.3.7, decisions accepted |
| Phishing protection spec (URL reputation, the phishing list and its governance, warnings, download checks; targets PH-) | `reports/Izcal phishing protection.md` | draft 0.3.1, decisions accepted |
| Devices spec (WebUSB, WebHID, Web Serial, Web MIDI, Web Bluetooth and Web NFC behind one device-access boundary; targets DV-, rules G210–G219) | `reports/Izcal devices.md` | draft 0.3.2, decisions accepted |
| App host spec (WASI 0.2.12 component host, `app://<id>` identities, broker-minted capability tickets, sealed HTML UI with typed intents, recorded host calls that replay; targets AH-, rules G220–G229) | `reports/Izcal app host.md` | draft 0.3, decisions accepted |
| Shell UI design | `reports/HGBA shell UI design.md` | draft 0.5 |

Reviews that fed the specs are also in `reports/`; the evidence behind every report is in `research_notes/<report name>/`.

## Decisions and staffing

**Every decision is accepted.** The records are in `research_notes/Phase 3 amendments/`: `ledger.md` (the conflict C- rows and the gap G- rows, with the batch 2–6 addenda), `c6r_decision.md` (the two-ledger staffing), `residue_decisions.md` (the phase 3 residue: C-11, C-19, C-21, G-4, Q-SH-3 and the accessibility rows), `batch2_decisions.md` and `batch3_decisions.md` (SVG, animation and editing; storage, WebCrypto and extensions), `batch4_decisions.md` (ML service, shell and phishing protection), `devices_decisions.md` and `app_host_decisions.md`, the receiver notes `applied_js.md` and `applied_footprint.md`, plus `source_evidence_2.3.3.md` for the M-1 part 3 pass.

- **C-6 is settled (22 October 2026): two ledgers.** **Ledger A** is the published, human-derived planning and audit basis: about **635–845 people†** at full strength (about 628–830 engineers† plus 7.5–14.5† non-engineers), of which Stage 1 8.3 allocates **67.5–96.5 engineers† plus 5.5–10.5† non-engineers**. **Ledger B** is the internal agentic envelope (about 1,700–3,400 engineer-years†, 3.1–6.1M agent-hours, 15–31T processed tokens, $6M–$350M, Stage 1 exit floor 40–60 months†), adopted internal under Q-AE-1. Reversal trigger: at FP-M0, GR-M0, JS-M1 and the first phase-8 cycle, if two consecutive components measure outside Ledger B's envelope on the same side, the class ranges are restated from the measurements (`c6r_decision.md`).
- **Q-SH-3 is decided (21 October 2026):** 90 days† of full visits and 12 months† of the `aggregate` record, user-configurable (shell 0.3.6).
- **Open by design, not by omission:** the devices spec's **DV-E1 to DV-E6 stay PROPOSED** (not granted); the specifications' Q- lists are measurement, product or audit questions and stay open; the app host's **Q-AH-1 to Q-AH-9** stay open. The ledger's other open rows are **A-EX-12** (until the Stage 2 extension surfaces exist), **A-CON-2** (conditional on an owner authorisation) and **A-SEC-26** (Phase 5).

## The archive

The V8 fork, the spike code and the M0 sandboxes were removed from the tree on 21 October 2026 (PR #14 on `main`); git history keeps them. Six tags mark the archive: `archive/hgba2-2.4-2026-10-21`, `archive/phase2-engine-2026-10-21`, `archive/phase2-layout-media-2026-10-21`, `archive/ground-up-intent-2026-10-21`, `archive/coding-standard-2026-10-21` and `archive/cs-q56-2026-10-21`. The V8 archives built in M0 are not in git; their checksums are in `m0/archives/SHA256SUMS` and `m0/linux/SHA256SUMS`.

## Worktrees

Three worktrees are live, one branch each (rules in [CONTRIBUTING.md](CONTRIBUTING.md)):

| Worktree | Branch | State |
|---|---|---|
| `izcal-wt/phase2c` | `spec/phase2c` | The accepted corpus: the specifications, research notes, measurements and this README |
| `izcal-wt/security-scope` | `spec/security-scope` | The security scope at 0.3.8; unmerged, awaiting its pull request |
| `izcal-wt/cs-amendments` | `spec/cs-amendments` | The coding standard at 0.3.7; unmerged, awaiting its pull request |

## Measurements

`measurements/` holds the program that confirms or replaces every † number. `measurements/README.md` is the calendar (FP-M0, GR-M0, JS-M1, DV-M0, AH-M0, AX-M0, then each spec's own waves) and the rules; `measurements/agentic/` holds the Ledger B data rows, beginning with `2026-10-21-spec-authoring.md`. The specs own their targets; the measurements confirm or correct them, and C-6R's reversal trigger reads these files.

## Layout

| Path | Contents |
|---|---|
| `reports/` | Specifications, designs and reviews |
| `research_notes/` | The evidence behind every report, and the Phase 3 decision records under `research_notes/Phase 3 amendments/` |
| `measurements/` | The measurement program: the calendar, the rules and the agentic data rows |
| `m0/` | M0 results only: `M0_SUMMARY.md`, `M0_RESULTS.md`, per-platform results in `m0/linux` and `m0/windows`, the local CI runner in `m0/ci`, spike results in `m0/spike-archive/`, and the V8 archive checksums in `m0/archives/SHA256SUMS` |
| `tools/`, `.githooks/`, `.github/` | Checks, hooks and CI |

## Getting the code

```
git clone https://github.com/Rorqualx/izcal.git
cd izcal && tools/setup-hooks.sh
```

Branches, worktrees, commits, checks, code review, merging and pushing are covered in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT ([LICENSE](LICENSE)), for code and documents (Stage 1 design, D-S6). Everything Izcal ships, code and data, must be available under MIT (HGBA-2 2.4.4); the exceptions are in CONTRIBUTING.md, "License".
