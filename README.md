# Izcal

Izcal is a new Rust-first browser engine, written from the web specifications and first principles (HGBA-2's method constraint M-1). It is written from the ground up: Izcal builds its own JavaScript engine, style system, layout, text stack and renderer, uses no other browser's code or components, and depends only on generic infrastructure (HGBA-2 2.3.0). The specifications written before 2.3.0 still describe a V8-, Stylo- and WebRender-based design in places; those sections are marked superseded and are being rewritten. **HGBA** (Hardware-Growth Browser Architecture) is the name of its architecture and specifications; the product name is Izcal (Stage 1 design, D-S9). The project is at the end of milestone M0: specifications, spikes and build infrastructure, with no engine code yet.

## Specifications

The documents cite each other by ID. This table maps the IDs to files.

| ID | File | Current revision |
|---|---|---|
| HGBA-2 (spec; targets S-, C-, P-, R-, I-, decisions D1–D13) | `reports/HGBA 2 browser architecture spec.md` | 2.3.4, frozen (V8-era sections superseded) |
| HGBA-3 (direction: oracles, data plane, determinism; B-, DET-, AG-) | `reports/Future software browser engine.md` | 3.2.0, frozen |
| Stage 1 design (process map, crates, IPC, milestones; owner decisions D-S1–D-S12) | `reports/HGBA Stage 1 architecture design.md` | draft 0.12 |
| DevTools spec (native inspection protocol, `hg-devtools`, adapters; targets DT-) | `reports/Izcal DevTools spec.md` | draft 0.3.2, decisions accepted |
| JavaScript engine spec (Izcal's own JS and Wasm engine and the GC shared with the DOM, on x86-64, AArch64 and riscv64; targets JS-) | `reports/Izcal JavaScript engine.md` | draft 0.4.2, decisions accepted; riscv64 first-class (JS-D22) |
| Resource footprint spec (memory, wakeups, energy, disk and GPU budgets, per ISA on x86-64, AArch64 and riscv64; targets FP-) | `reports/Izcal resource footprint.md` | draft 0.4.2, decisions accepted |
| HTML and CSS parsing and style spec (HTML, XML and CSS parsers, selectors, cascade, invalidation; targets ST-) | `reports/Izcal HTML and CSS parsing and style.md` | draft 0.3.6, decisions accepted |
| Fonts and text spec (font parsing, shaping, Unicode, rasterization, text input, Izcal's own font set; targets TX-) | `reports/Izcal fonts and text.md` | draft 0.3.6, decisions accepted |
| Networking spec (HTTP/1.1, HTTP/2, HTTP/3 and QUIC, TLS and cryptography, the root program, DNS, Fetch, cookies, HTTP cache; targets NW-) | `reports/Izcal networking.md` | draft 0.3.5, decisions accepted |
| Layout spec (box tree, formatting contexts, fragmentation, incremental layout, hit testing; targets LY-) | `reports/Izcal layout.md` | draft 0.4.2, decisions accepted |
| Images and media spec (image decoders, colour and HDR, codecs, audio, WebCodecs, capture; targets MD-) | `reports/Izcal images and media.md` | draft 0.4.1, decisions accepted |
| Graphics and rendering spec (GPU abstraction, shaders, rasterizer, compositor, WebGPU, WebGL, Canvas; targets GR-) | `reports/Izcal graphics and rendering.md` | draft 0.4.2, decisions accepted |
| Content blocking spec (protection layers, matcher, Izcal's own lists and their governance; targets CB-) | `reports/Izcal content blocking.md` | draft 0.3.2, decisions accepted |
| DOM, bindings and event loop spec (single DOM copy, Web IDL bindings, the event loop, agents; targets DM-) | `reports/Izcal DOM, bindings and event loop.md` | draft 0.2.3, decisions accepted |
| Accessibility spec (accessibility tree, OS adapters, Izcal's own, AT lab; targets AX-) | `reports/Izcal accessibility.md` | draft 0.2.2, decisions accepted |
| PDF viewer and writer spec (sandboxed PDF process, print-to-PDF; targets PD-) | `reports/Izcal PDF.md` | draft 0.2.1, decisions accepted |
| SVG spec (SVG document model, scriptless host, SMIL split; targets SV-) | `reports/Izcal SVG.md` | draft 0.3, decisions accepted |
| Animation spec (Web Animations, transitions, SMIL timing, View Transitions; targets AN-) | `reports/Izcal animation.md` | draft 0.3, decisions accepted |
| Editing spec (editing model, IME, Izcal's own spelling; targets ED-) | `reports/Izcal editing.md` | draft 0.3.1, decisions accepted |
| Storage spec (storage service, Izcal's own storage engine, quotas; targets SG-) | `reports/Izcal storage.md` | draft 0.3.1, decisions accepted |
| WebCrypto spec (SubtleCrypto surface, entropy, constant-time additions; targets WC-) | `reports/Izcal WebCrypto.md` | draft 0.3, decisions accepted |
| Extensions spec (WebExtensions surface, MV3, published trust process; targets EX-) | `reports/Izcal extensions.md` | draft 0.3, decisions accepted |
| Shell UI design | `reports/HGBA shell UI design.md` | draft 0.3 |

Reviews that fed the specs are also in `reports/`; the evidence behind every report is in `research_notes/<report name>/`.

## Layout

| Path | Contents |
|---|---|
| `reports/`, `research_notes/` | Specifications, reviews and their sources |
| `m0/` | Milestone M0. Start with `m0/M0_SUMMARY.md` |
| `m0/rusty_v8` | History (M0, superseded by HGBA-2 2.3.0). The `rusty_v8` fork (submodule, [Rorqualx/rusty_v8-izcal](https://github.com/Rorqualx/rusty_v8-izcal), branch `hgba`). It carries the `cppgc` APIs and the predictable platform, and its CI builds V8 from source (`.github/workflows/izcal-v8-build.yml`) |
| `m0/spike-sandbox`, `m0/v8-cppgc-ext-sandbox`, `m0/determinism-probe`, `m0/fork-api-verify` | M0 spikes and probes, built against the fork |
| `m0/linux`, `m0/windows` | Per-platform M0 results and logs |
| `m0/ci` | CI notes and the local runner used to test the workflow |
| `cppgc-spike/`, `v8-cppgc-ext/` | Pre-M0 originals (sandbox off, crates.io V8). `v8-cppgc-ext` is retired (HGBA-2.1.6 S-4e) |

## Getting the code

```
git clone --recurse-submodules https://github.com/Rorqualx/izcal.git
cd izcal && tools/setup-hooks.sh
```

Branches, worktrees, commits, checks, code review, merging and pushing are covered in [CONTRIBUTING.md](CONTRIBUTING.md).

Both repositories are public. The V8 archives built in M0 are not in git (see `.gitignore`); their checksums are in `m0/archives/SHA256SUMS` and `m0/linux/SHA256SUMS`.

## License

MIT ([LICENSE](LICENSE)), for code and documents (Stage 1 design, D-S6). The `rusty_v8` fork is MIT too. Everything Izcal ships, code and data, must be available under MIT (HGBA-2 2.3.2); the exceptions are in CONTRIBUTING.md, "License".
