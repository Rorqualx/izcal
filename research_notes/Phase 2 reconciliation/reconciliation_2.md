# Phase 2 reconciliation 2: layout, images and media, graphics and rendering

Compiled 19 October 2026 by the reconciliation reviewer, who wrote none of the three specs. Method: phase 8 of the first-principles-spec skill (`.claude/skills/first-principles-spec/SKILL.md`), following the precedent of the [first Phase 2 reconciliation][RN] (P1–P27): for each conflict, one spec owns the design and the others cite it.

**Documents reconciled.** Draft 0.2 of [layout][LY] (LY), draft 0.2.1 of [images and media][MD] (MD; MD-D3, royalty-free codecs, is an owner decision and is not changed here) and draft 0.2 of [graphics and rendering][GR] (GR), against the accepted [JavaScript engine][JS] (JS 0.3.1), [resource footprint][FP] (FP 0.3.1), [HTML and CSS parsing and style][ST] (ST 0.3.3), [fonts and text][TX] (TX 0.3.4) and [networking][NW] (NW 0.3.3) specs, all with owner decisions recorded, [HGBA-2 2.3.2][H2] M-1 (MIT only; another engine's source is never evidence; engine work is all Izcal's), the [first reconciliation][RN] (P1–P27, the rule registry, staffing) and the [licence audit][LA]. The accepted specs, HGBA-2, HGBA-3 and Stage 1 are not edited; every amendment they need is listed in the amendments section (9.5) of the new spec that owns it and summarized in Section 5 below. LY and GR gain a revision 0.3 line, MD a revision 0.3 line, and each an Appendix A section for this pass.

**Result.** 32 items (Q1–Q32). Every item is applied to the three new specs. Four need an owner decision (Section 6): OD-5 (the content GPU process's start and device rules, and FP-7's scope for per-key service processes), OD-6 (one D-S1 amendment for canvas, text, audio and video readback), OD-7 (prose documents hosted in code repositories under M-1 part 3) and OD-8 (one list of I-1 shared-process exceptions). **The footprint arithmetic breaks with GR as written**: the shell's content GPU process alone puts FP-1 at about 70.5 MiB against 64, a visible iframe-heavy tab puts FP-S30 about 100 MiB over 1.25 GiB, and FP-7's iframe-heavy limit fails for any per-key raster process above about 0.4 MiB. With OD-5, FP-1, FP-S30 and FP-6 hold on paper and FP-7 needs its scope restated as OD-2 did for networking (Section 3). The eight specs need about **437–560 people†** at full strength, **384–502 of them engineers†** (Section 4). **Stage 1 exits at 48–60 months†**, set by layout, with graphics placed in the same window and the JS engine's 39–51 months no longer the critical path (Section 4).

## 1. Summary

| ID | Topic | Kind | Owner after reconciliation | Owner decision |
|---|---|---|---|---|
| Q1 | Identity of paint output: no fragment-store ids in property trees or display items | Gap (new specs) | GR (paint); LY (G45) | — |
| Q2 | `hg-paint-order`: one owner; its data stays in the renderer | Conflict (new specs) | GR (crate); LY (reference builder) | — |
| Q3 | Per-node bytes: 200 B / 160 B, and one failure order | Conflict (new specs) | LY and GR (split); FP (FP-11) | — |
| Q4 | Clamping cross-process hit-test regions | Conflict (new specs) | LY (message); GR (routing) | — |
| Q5 | Scroll ownership and the visual viewport | Consistency | LY (protocol); GR (compositor side) | — |
| Q6 | Visibility contract (LY-D22) | Consistency | LY, with GR's rule | — |
| Q7 | Keying of media, hardware-decode and CDM processes | Conflict (MD with FP-D8 and GR) | MD | — |
| Q8 | Content GPU processes against FP-1, FP-S30, FP-6, FP-7 | Conflict (FP) | GR (process); FP (budgets, amended) | **OD-5** |
| Q9 | Hardware decode placement after GR-D2 | Conflict (new specs) | MD | — |
| Q10 | One list of I-1 shared-process exceptions | Conflict (new specs, HGBA-2) | HGBA-2 owner | **OD-8** |
| Q11 | Raster of opaque images: "the GPU service" | Conflict (new specs) | GR (where); MD (G56) | — |
| Q12 | One colour contract in two forms; class W into E16 | Conflict (new specs) | MD (`hg-colour`); GR co-signs | — |
| Q13 | Display conversion, HDR and readback | Consistency | MD (rule); GR (compositor) | — |
| Q14 | One D-S1 amendment (2D canvas, `measureText`, audio, video) | Conflict (new specs, TX, Stage 1) | GR (text of the amendment); MD, TX cite | **OD-6** |
| Q15 | Decision points and one recording budget | Consistency | Each spec its points; JS-D15 the rule | — |
| Q16 | Layout's stack segments (LY-D15) against JS-D19 | Conflict (JS) | JS (mechanism); LY (budget) | — |
| Q17 | Threads and OS callbacks in graphics | Gap | Stage 1 substrate; MD requests, GR cites | — |
| Q18 | Layers: `hg-path` format, `hg-simd`, L2g and L2m | Conflict (new specs, TX) | GR (`hg-path`); MD (`hg-simd`) | — |
| Q19 | Rule registry G37–G72 | Conflict (new specs) | Stage 1 2.2 (registry); each spec its rules | — |
| Q20 | One owner per shared crate and object | Consistency | As tabled | — |
| Q21 | Video path and script-visible frames | Conflict (new specs) | MD (frames, rule); GR (layers, routing) | — |
| Q22 | Canvas export and the media process's encoders | Conflict (new specs) | MD (encoders); GR (pixels) | — |
| Q23 | WebCodecs `VideoFrame` from non-canonical sources | Gap | MD, with GR's rule | — |
| Q24 | Print, vector output and PDF | Conflict (new specs, Stage 1) and gap | LY (paged layout); GR (raster, vector); PDF spec unowned | — |
| Q25 | SVG ownership and the media process's SVG host | Gap | GR, LY, MD as tabled; SVG spec unowned | — |
| Q26 | Screen and tab capture | Gap | MD (OS capture); GR (compositor capture) | — |
| Q27 | Energy targets: absolute, one playback method | Conflict (FP, new specs) | MD (playback); GR (active energy) | — |
| Q28 | One list of cross-origin rendering channels | Consistency | GR, with MD's rows | — |
| Q29 | Stage 1 milestone order and the critical path | Conflict (Stage 1, JS) | Stage 1 owner | — |
| Q30 | Staffing and double counting | Conflict (new specs) | Each spec its tracks | — |
| Q31 | Licence and M-1 residuals | Consistency | Each spec; owner for classifications | **OD-7** |
| Q32 | Stale cross-references | Consistency | Each spec | — |

## 2. Items

### Q1. Identity of paint output

**Gap.** LY makes fragment ids history-dependent store indices, never hashed, and forbids primary state from holding them ([LY 6.13, LY-D18, G45][LY]). GR keys property-tree nodes "by a stable id" and hashes display items for DET-1 ([GR 6.3][GR]) without saying where the ids come from. If they came from fragment ids, an incremental pass and a from-scratch pass would produce different display-list hashes for identical pixels, and GR-50 and LY-3 would disagree.
**Resolution.** Property-tree node ids and display-item identities are derived from the node slot (or the anonymous box's path from its nearest node), the pseudo-element and the fragment's index within its box, never from fragment-store ids or epochs. DET-1's display-list hash is computed over that id-free form, as LY's canonical fragment tree is. **Owner.** GR. **Changed.** GR 6.3, 6.21; LY 6.13.

### Q2. `hg-paint-order`: one owner, and its data stays in the renderer

**Conflict.** LY defines the crate "owned jointly with the graphics track" ([LY 6.14, 6.18][LY]); GR "co-owns" it and supplies the production implementation ([GR 6.3, 6.24][GR]). Separately, GR's renderer keeps only chunk hashes after the content GPU process acknowledges a frame ([GR 6.3][GR]), while `hg-hit` walks chunks synchronously in the renderer ([LY 6.14][LY]).
**Resolution.** **GR owns `hg-paint-order`** and its production implementation; LY owns the requirements it wrote (6.14) and the Appendix-E reference builder, kept as a test oracle (LY-7, GR-5); a contract change needs both owners' sign-off. Everything the contract serves (chunk order, chunk-to-fragment map, property-tree nodes, ink overflow per chunk) stays in the renderer after acknowledgement, inside GR's 72 B† renderer share (GR 6.22's rows already count it), so hit testing never waits on another process. **Changed.** LY 6.14, 6.18; GR 6.3, 6.24.

### Q3. Per-node bytes and one failure order

**Conflict.** The split is consistent: ST gives layout and paint 360 B† of FP-11's 512 B† ([ST 6.11][ST]); LY takes 200 B† (expected; high column 276 B) ([LY 6.13][LY]); GR co-signs 160 B†, counted across processes as about 72 B in the renderer and ≤ 88 B† of display items in the content GPU process ([GR 6.22, GR-D23][GR]). The failure orders disagree: LY re-splits "inside 360 B†, never FP-11 itself"; GR asks FP and layout for a re-split "inside FP-11's 512 B†".
**Resolution.** One order for both: (1) representation work on the side that is over; (2) a joint re-split of the 360 B† between LY and GR; (3) FP-11's own failure action, which FP owns; ST's 72 B† and 80 B† shares change only with ST's agreement. Never a separate ledger line. LY's "provisional until graphics co-signs" is now co-signed. FP-11 counting paint data across processes is an FP amendment (listed in GR 9.5). **Changed.** LY 6.13; GR 6.22, GR-D23.

### Q4. Clamping cross-process hit-test regions

**Conflict.** LY has the browser clamp each region "to the frame's composited surface bounds" ([LY 6.14][LY]), which is compositor geometry; GR-D28 has the broker clamp with parent-supplied geometry and never trust compositor geometry for consent ([GR 6.1][GR]).
**Resolution.** GR's rule: the broker clamps each child frame's region to the region its embedder renderer reported for that frame, recursively up to the top-level renderer, whose regions the broker clamps to the window; compositor geometry is only a cross-check. **Owner.** LY (the message); GR (routing). **Changed.** LY 6.14.

### Q5. Scroll ownership and the visual viewport

**Consistency.** GR implements LY-D13's delta protocol exactly ([GR 6.8][GR]). Missing: LY's complete list of recorded inputs ([LY 6.1][LY]) does not name the visual viewport's scale and offset, which GR's compositor owns during a pinch and returns as deltas ([GR 6.28][GR]). **Resolution.** LY 6.1 adds them. GR owns the compositor-scrollable predicate; LY owns offsets as primary state. **Changed.** LY 6.1.

### Q6. Visibility contract (LY-D22)

**Consistency.** GR accepts LY-D22 and adds a rule: a target is not provably visible under a compositor-mutable opacity or transform ancestor, a filter, or a GR-D18 clamp or substitution, computed in the renderer ([GR 6.28][GR]). **Resolution.** LY-D22 accepted as amended; LY 6.15's per-frame visibility bit applies GR's rule. **Changed.** LY 6.15, 9.4.

### Q7. Keying of media, hardware-decode and CDM processes

**Conflict.** GR keys the content GPU process exactly like the renderer, by origin wherever FP-D8 keys renderers by origin (every desktop class) ([GR 6.1][GR]). MD keys the media, hardware-decode and CDM processes by (site, container) ([MD 6.1][MD]). Under FP-D8, `a.example.com` and `b.example.com` have separate renderers but would share one media process; a compromised `a` renderer that exploits that process (MD names the renderer→media endpoint an escalation boundary) would reach `b`'s images and the opaque media `b` embeds, which FP-D8 separates.
**Resolution.** Every per-site service process is keyed exactly like the renderer it serves: the media, hardware-decode and CDM processes, as the content GPU process already is. FP-D11 and FP-38 are restated with "renderer key" (listed in MD 9.5). **Owner.** MD. **Changed.** MD summary, 6.0, 6.1, 9.5.

### Q8. Content GPU processes against FP-1, FP-S30, FP-6 and FP-7

**Conflict.** GR-D2 adds a content GPU process per visible renderer key at ≤ 12 MiB† charged, a spare, and the shell's own content GPU process for prompts (GR-D28) ([GR 6.1, 6.22][GR]). FP's model has one GPU service in the baseline ([FP 6.1, 8.2][FP]). Section 3's arithmetic shows: FP-1 breaks (about 70.5 MiB against 64), FP-S30 breaks when the visible tab is iframe-heavy (about 1,378 MiB against 1,280 at 10 cross-site frames), and FP-7's iframe-heavy limit fails by a factor of about four. GR's own failure response is "reduce per-process cost, never co-locate".
**Resolution, applied to GR and pending OD-5.** (a) **Start with a rendering opportunity.** A key's content GPU process starts only when one of its surfaces has a rendering opportunity (a visible document with a frame intersecting the viewport plus the prepaint margin); throttled off-screen frames start none, by the same process-independent rule as GR-D15's throttling. (b) **Device on demand.** It starts **device-less**: canonical raster on the CPU gives the same bits (GR-D3), so it opens a GPU device only when the key uses WebGL, WebGPU or an accelerated canvas, draws video under a page effect, or its visible raster work exceeds a threshold the cost model sets from GR-S2 and GR-S3; it closes the device after 30 s† without such use. Budgets: device-less ≤ **2 MiB†** charged at idle (FP-38's figure, a process with no JS agent); with a device ≤ **12 MiB†** (GR-47). (c) The shell's content process (GR-D28's prompts) and the spare are device-less. (d) **FP-7's scope**, as OD-2 did for networking: FP-7 measures renderer isolation; per-key service processes (content GPU, media, hardware-decode, CDM) are reported on their own isolation line with their absolute budgets, never co-located. Merging the device-less raster into the key's media process (same authority by MD's own option (d) argument) is kept as a measured alternative for GR-M0 (GR-S2 with MD-S10), not adopted. **Owner.** GR (process rules), FP (budget amendments). **Owner decision OD-5.** **Changed.** GR summary, 6.1, 6.21, 6.22, GR-47, GR-D2, GR-D23, 9.5.

### Q9. Hardware decode placement after GR-D2

**Conflict.** MD-D10 rejects option (b), decode submitted by "the GPU service", because it would put slice-data driver bugs "in the one process that holds every site's pixels" ([MD 6.9][MD]); under GR-D2 that process no longer exists, and MD's reversal (b) still names it.
**Resolution.** MD-D10 (c) stands: the content GPU process of a key parses display lists and shaders from a possibly compromised renderer, and adding OS decoder grants (VideoToolbox Mach services, Media Foundation's D3D11 video device) to it would widen the sandbox of a parsing process, against P1; the hardware-decode process runs no parser. MD-D10's reversal (b) now means the **content GPU process of the same key**, which holds only that key's data. **Owner.** MD. **Changed.** MD 6.9, MD-D10.

### Q10. One list of I-1 shared-process exceptions

**Conflict.** MD records the audio and capture services as I-1 exceptions "beside the GPU service" ([MD 9.5][MD]); GR records the compositor as an exception "beside media's audio and capture services" ([GR 9.5][GR]); each lists its own row for HGBA-2 I-1.
**Resolution.** One HGBA-2 I-1 amendment with one list: the **compositor GPU service** (composited pixels of every site, parses no display list or shader, G63), the **audio service** (output-ready PCM and granted microphone streams, parses nothing) and the **capture service** (granted capture frames, parses nothing), each with a threat model in the security scope. **Owner.** HGBA-2 owner. **Owner decision OD-8**, because it amends an invariant. **Changed.** GR 9.5; MD 9.5.

### Q11. Raster of opaque images: "the GPU service"

**Conflict.** MD requires every raster of content containing an opaque image to happen "in the GPU service", a requirement on an unwritten graphics spec ([MD 6.1, 9.5][MD]); GR accepts it in the content GPU process of the embedding key, which holds that key's decoded images ([GR 6.1, 6.11][GR]). MD still says "GPU service" in 26 places, including FP 6.5's "GPU-service-fixed regions".
**Resolution.** GR's placement. In MD, "GPU service" now means the **compositor GPU service** for display conversion, tone mapping, gain maps, video layers, frame selection and overlays, and the **content GPU process of the embedding renderer key** for raster (opaque images included, on its device-less CPU path too), decoded-image regions and script-visible frame uses. MD 6.0 states it once; the key sentences are rewritten. **Owner.** GR (where), MD (G56). **Changed.** MD summary, 6.0, 6.1, 6.6, 9.5, Section 10.

### Q12. One colour contract in two forms; class W into E16

**Conflict.** MD's conversion contract evaluates matrix-shaper profiles per pixel in IEEE f32 at decode ([MD 6.5][MD]); GR builds tables with `hg-colour`'s f32 contract and applies them by integer lookup, because G61 forbids floating point in canonical raster ([GR 6.5][GR]). Class W is linear FP16; GR's E16 is signed fixed point holding encoded extended sRGB. Neither says how a W texel becomes E16 without float per-pixel work in `hg-raster`, nor which form a given conversion uses.
**Resolution.** One contract, two forms: **per-pixel f32 at decode**, in the media process under G55 (bit-identical across ISAs); **tables built on the CPU with the same f32 contract and applied by integer lookup and interpolation at raster**, under G61. Each (source, destination) conversion has exactly one form, named in `hg-colour`'s table of conversions, so no operation has two answers. A class W texel enters E16 or S8 through a table indexed by its half-float bit pattern (65,536 entries, built once per destination encoding). **Owner.** MD (`hg-colour`, MD-D28); GR co-signs. **Changed.** MD 6.5; GR 6.5.

### Q13. Display conversion, HDR and readback

**Consistency.** Display conversion, tone mapping and gain maps happen only in the compositor (GR 6.5) or the OS compositor, and no readable pixel passes through them (MD-32). Readback tone mapping at headroom 1.0 uses Q12's table form. MD's "GPU service or OS compositor" becomes "compositor GPU service or OS compositor". **Changed.** MD summary, 6.5.

### Q14. One D-S1 amendment

**Conflict.** Three specs amend Stage 1's owner-signed D-S1 (seeded readback noise) separately and one contradicts another: TX-D4 drops noise on `measureText` (accepted) ([TX 6.8, TX-D4][TX]); GR-D12 drops it on 2D canvas readback and applies it once where WebGL and WebGPU pixels leave for a readable surface ([GR 6.15][GR]); MD-D14 drops it on Izcal-rendered audio and keeps it on OS-decoded audio ([MD 6.8][MD]); but MD 6.4 says canvas export keeps D-S1's noise "this spec does not change it", against GR-D12. Gap: readback of video frames decoded by OS decoders (H.264 and HEVC on macOS and Windows), whose pixels Izcal does not control, is covered by no rule.
**Resolution.** One amendment, one rule: **noise is applied once, where pixels or samples leave for a script-readable surface, to exactly the outputs that are not canonical**: WebGL and WebGPU pixels (GR-D12, including laundering through 2D canvases, ImageBitmaps and VideoFrames); audio derived from OS-decoded tolerance codecs (MD-D14); and video frames from OS decoders that have not passed MD-42's conformance-hash check for that codec and profile on that device (exact-output codecs that pass are canonical). No noise on 2D canvas, OffscreenCanvas 2D, canvas export (`toBlob`, `toDataURL`, `convertToBlob`, whose encoders are deterministic), `measureText`, Izcal-rendered audio, or Izcal-decoded images and video. Recordings keep pre-noise data and the seed. PRIV-2 (a) is restated as distinguishability (GR's proposal). GR holds the amendment's text; MD and TX cite it. **Owner decision OD-6**, because D-S1 is owner-signed. **Changed.** GR 6.15, 9.5; MD 6.4, 6.7, 6.8, 9.5.

### Q15. Decision points and one recording budget

**Consistency.** LY lists its recorded inputs as complete ([LY 6.1][LY]); GR adds BeginFrame ticks, chunk resend, canvas snapshots, the listener timeout, `will-change` expiry and `requestVideoFrameCallback` metadata ([GR 6.21][GR]); MD adds header results, decode completion, cache hits, frame selection and hardware choice ([MD 6.13][MD]). Each states a recording cost: LY-40 "≤ 1%†", MD against "JS-D15's 1%", GR none. HGBA-3 DET-1 and JS-52 set **one** whole-renderer budget (≤ 1%† of p95 frame time).
**Resolution.** The three lists stand, each spec owning its points; the JS-D15 amendment rows in each 9.5 together form the consolidated list. LY-40's 1% and MD's measurement are **shares of one whole-renderer budget**, measured jointly with JS-52 and attributed per subsystem; GR reports its share. **Changed.** LY 6.1, LY-40; GR 6.21; MD 6.13.

### Q16. Layout's stack segments against JS-D19

**Conflict.** LY-D15 (b) runs layout recursion on JS-D19's guard-paged segments, moved into the substrate ([LY 6.17][LY]). JS-D19's own fallback (c) is "runtime calls switch to the OS stack" on an OS where Rust frames on segments fail verification ([JS 6.4, JS-D19][JS]); layout is all Rust frames, so where JS falls back, LY-D15 (b) fails too.
**Resolution.** LY-D15 (b) stands as an amendment request to JS (listed in LY 9.5); its reversal now also fires per OS where JS adopts JS-D19 (c): layout then uses (c), explicit work stacks, on that OS, with the same canonical budget and depth limit. **Owner.** JS (mechanism), substrate (placement), LY (budget). **Changed.** LY 6.17, LY-D15.

### Q17. Threads and OS callbacks in graphics

**Gap.** Stage 1 G3 lets only the substrate create threads or touch ambient OS effects. MD asks `hg-substrate-os` for a real-time thread interface and an OS-callback bridge ([MD 6.8][MD]); GR runs raster threads, a broker presentation thread, `AChoreographer` loopers and Metal presented handlers without naming the substrate ([GR 6.9][GR]).
**Resolution.** GR's threads come from the substrate's pool and its OS callbacks pass through the same OS-callback bridge, recorded under B1; the substrate interface has one owner (Stage 1), with MD as requester and GR as second user. **Changed.** GR 6.9; MD 6.8.

### Q18. Layers: `hg-path`'s format, `hg-simd`, L2g and L2m

**Conflict.** (a) TX emits large glyphs as paths "in the graphics spec's bounded path format" ([TX 9.5][TX]); GR puts that format in `hg-path`, in L2g ([GR 6.3, 6.24][GR]); siblings L2t and L2g may not depend on each other (P1's rule). (b) GR's CPU fine raster is "SIMD, integer" while G60 makes every graphics crate but the back ends `#![forbid(unsafe_code)]`; pointer-based SIMD loads and calls into `#[target_feature]` functions are `unsafe` in Rust, which is why MD's `hg-simd` is MD's one unsafe L2m crate ([MD 6.4, G54][MD]). Two SIMD layers would mean two audits of the same kind of code.
**Resolution.** (a) The bounded path **types** move to `hg-display-list` (L2, owned by GR); flattening, stroking and dashing stay in `hg-path` (L2g); TX depends only on the L2 types. (b) **One `hg-simd`, in L2s, owned by MD**: per-ISA dispatch and slice-based load and store wrappers, the only `unsafe`, inventoried and Kani-proved; kernels in `hg-dsp` (L2m) and `hg-raster` (L2g) use its safe API and stay free of `unsafe`; G54 and G60 are restated accordingly. L2s, L2e, L2p, L2t, L2m and L2g are siblings above L2. **Changed.** GR 6.3, 6.4, 6.24, G60; MD 6.4, 6.18, G54, MD-D7.

### Q19. Rule registry

**Conflict.** MD registers G50–G57 "after the layout spec's G37–G43; G44–G49 stay unassigned" ([MD 6.18, 9.5][MD]); LY now uses G37–G45 ([LY 6.18][LY]); GR records G46–G49 and G58–G59 as unassigned ([GR 6.24][GR]).
**Resolution.** No renumbering (each spec's numbers are already cited across the others):

| Rules | Owner |
|---|---|
| G1–G9 | Stage 1 |
| G10–G12 | DevTools spec |
| G13–G18 | JS |
| G19–G24 | ST |
| G25–G29 | TX |
| G30–G36 | NW |
| **G37–G45** | LY |
| G46–G49 | Unassigned (reserved, recorded) |
| **G50–G57** | MD |
| G58–G59 | Unassigned (reserved, recorded) |
| **G60–G72** | GR |
| G73 onward | Next spec, assigned in Stage 1 2.2 |

**Changed.** MD 6.18, 9.5; LY 9.5 (registry row); GR 6.24.

### Q20. One owner per shared crate and object

| Crate or object | Owner | Users | Layer |
|---|---|---|---|
| `hg-colour` (descriptors, transfer functions, tables, EETF, gain maps) | MD (MD-D28); GR co-signs the numeric contract | Media process, both GPU process kinds, renderer canvas | L2s |
| `hg-simd` | MD | `hg-dsp`, `hg-raster` | L2s (Q18) |
| `hg-types` geometry module | LY | Paint, GPU validation, DevTools | L2 |
| `hg-display-list` (display list, compositor frame, canvas commands, bounded path types) | GR | Renderer, content GPU process, compositor, TX | L2 |
| `hg-path`, `hg-raster`, `hg-layerize` | GR | Renderer (canvas), both GPU kinds, media process (SVG images) | L2g |
| `hg-interp` | Future animation spec; GR writes it until that spec has an owner | Compositor, main thread | L2g |
| `hg-paint-order` | GR; LY keeps the reference builder | `hg-hit`, `hg-layout-query` | L4 |
| Image cache (keys, discardable policy, `hg-imgpipe`) | MD | Renderer; regions allocated by the content GPU process (GR) | L3 |
| `VideoFrame`, frame-selection rule, decoded frames | MD | Compositor video layers (GR implements the rule) | — |
| Video layers, overlays, direct scanout | GR | MD (energy) | L5 |
| SVG: `hg-svg-paint` | GR | Renderer, media process | L4 |
| SVG: outer `<svg>` sizing, `foreignObject`, SVG text interface | LY | SVG spec | L4 |
| SVG: opaque-image host (`hg-media-host`) | MD | — | L4 |
| SVG document model, SMIL, SVG DOM, geometry APIs | **Unowned** (future SVG spec; Q25) | — | — |
| `hg-raster-ref`, `hg-reftest-raster` | GR; LY until GR-M1 retires it | Tests only | tools |

**Changed.** GR 6.24 cites this table; LY 6.18 and MD 6.18 carry their rows.

### Q21. Video path and script-visible frames

**Conflict.** GR routes frames on demand to the content GPU process and says "for a CPU-placed canvas the media process supplies the converted pixels" ([GR 6.11][GR]); but GR 6.15 makes the **renderer** the authoritative raster of every origin-clean canvas, whatever its display placement, so the renderer needs the frame's pixels for every `drawImage(video)` into such a canvas. MD's frame for script reads is the one at the official playback position, supplied by the media process ([MD 6.7][MD]).
**Resolution.** For an origin-clean video source, the media process supplies the frame at the official playback position as canonical RGBA (MD-32's integer YUV path) to the renderer for the authoritative replay, and the same pixels to the content GPU process for the display copy; hardware-decoded frames are read back to the media process first (MD 6.1). Opaque or tainting sources go only to the content GPU process (G56). **Owner.** MD (frames, rule); GR (routing). **Changed.** GR 6.11, 6.15; MD 6.7.

### Q22. Canvas export and the media process's encoders

**Conflict.** MD runs the canvas encoders in the media process and keeps D-S1's noise on their input ([MD 6.4][MD]); GR's readable-surface table does not say where `toBlob` encodes ([GR 6.15][GR]). **Resolution.** Export encodes in the media process of the key from canonical pixels (the renderer's for origin-clean canvases; tainted canvases cannot export, as HTML requires), with Q14's noise rule; encoders are deterministic, so export is a function of the canvas's pixels only. **Changed.** MD 6.4; GR 6.15.

### Q23. WebCodecs `VideoFrame` from non-canonical sources

**Gap.** GR noises WebGL and WebGPU pixels when they become a `VideoFrame` ([GR 6.15][GR]); MD's WebCodecs section does not mention it ([MD 6.7][MD]). **Resolution.** `new VideoFrame(canvas)` and `MediaStreamTrackProcessor` frames from a WebGL or WebGPU canvas carry GR's noised pixels; `VideoFrame.copyTo` of frames from OS decoders follows Q14. **Owner.** MD, with GR's rule. **Changed.** MD 6.7.

### Q24. Print, vector output and PDF

**Conflict and gap.** LY makes paged layout Stage 1 work and says "the PDF writer takes its structure from the fragment tree and the semantic plane, not from paint output" ([LY 6.7, LY-D8][LY]); GR takes appearance from the display list and asks LY to amend that sentence; GR-M3 lists "print, capture and export" at Stage 1 exit ([GR 6.27, 9.2][GR]), while Stage 1 13.2 keeps the print UI and print-to-PDF in Stage 2. No spec owns the PDF writer or PDF viewing; Stage 1 13.2's utility process and MD's "PDF process" are not reconciled.
**Resolution.** Paged layout and `window.print()`'s layout: Stage 1, LY. Print raster and print preview raster: Stage 1, GR (GR-M3), so print reftests run (GR-28). Vector print output: appearance from the display list, structure (tags, links, reading order) from fragments and the semantic plane, LY 6.7 amended; it ships with Stage 2's print UI. The PDF writer and PDF viewing belong to a **future PDF spec, unowned and unstaffed**, which also names the PDF process; MD's PDF-only image decoders are staffed in MD. **Changed.** LY 6.7; GR 6.27, GR-M3.

### Q25. SVG ownership and the media process's SVG host

**Gap.** MD-D20 rasterizes opaque cross-site SVG images in the media process with "the scriptless SVG, XML, style, layout and text crates" ([MD 6.1][MD]); LY says "the broker, the GPU service and the network service run no layout" ([LY 6.0][LY]) and its G1 assumes JS-D4's node columns, which MD-S10 must show can exist without an agent heap; the SVG document model, SMIL, SVG DOM and geometry APIs have no spec. **Resolution.** Q20's table; LY 6.0 names the media process as the one non-renderer that links layout crates, for MD-D20 only, under MD-S10; TX G26 is unaffected (the media process is not listed and reaches fonts through the read-only endpoint TX 9.5 already received). The SVG spec is recorded as **unowned**, like layout was in P23; its staffing is counted nowhere. **Changed.** LY 6.0; GR 6.16.

### Q26. Screen and tab capture

**Gap.** GR's compositor captures tabs, regions and elements for `getDisplayMedia`, `cropTo` and `restrictTo` ([GR 6.27][GR]); MD's capture service opens screen capture through the OS pickers ([MD 6.1, 6.10][MD]). Neither says which handles which. **Resolution.** A capture of an Izcal tab or element is composed by the compositor; a capture of a screen, another application's window, or a browser window as a whole goes through the capture service and the OS picker. Both deliver frames to the media process of the capturing key, and both prompts are in the broker's own layer (GR-D28). **Changed.** GR 6.27; MD 6.10.

### Q27. Energy targets

**Conflict.** FP's owner-accepted rule makes memory and energy thresholds absolute, with comparisons published as information; MD follows it (MD-54, MD-D24) ([MD 6.16][MD]); GR-41 sets video energy at "≤ 1.1× the OS's own player" and GR-43 active energy at "≤ the best shipping engine × 1.1" ([GR 8.5][GR]). **Resolution.** GR-41 is replaced by MD-54's method (MD owns playback energy; GR's overlay work is measured by it); GR-43 becomes an absolute FP-22-style gate per device class set at FP-M3, with the ratio to the best shipping engine published as information. GR-34's 1.15× comparison against Izcal's own float prototype stays (it is a design trigger, not a budget). **Changed.** GR-41, GR-43.

### Q28. One list of cross-origin rendering channels

**Consistency.** GR-D18 and MD-31 (b) agree to keep one list "jointly" ([GR 6.7][GR]; [MD 6.14][MD]). **Resolution.** GR owns the list (filters, framebuffer compression, frame timing, compositor slices); MD contributes the decode-timing and `requestVideoFrameCallback` rows. **Changed.** GR 6.7; MD 9.5.

### Q29. Stage 1 milestone order and the critical path

**Conflict.** Stage 1's milestones are now dated by the specs: M4 first pixels (LY-M1, GR-M1, MD-M1, TX-M1) at 18–24 months†; M6 (LY-M2) at 36–54; M7 (MD-M2) at 27–33; Stage 1 exit (LY-M3, GR-M3) at 48–60. So M7's media breadth is ready before M6's layout breadth, and the JS spec's 39–51-month Stage 1 exit, which ST and NW cite, is no longer the critical path. **Resolution.** Stage 1 8.1 is re-dated from the specs (listed in LY 9.5); M6 and M7 are expected-readiness points, not an order. See Section 4 for the combined path. **Changed.** LY 9.3, 9.5; GR 9.2 (GR-M3); MD 9.3.

### Q30. Staffing and double counting

See Section 4. Checked: **SVG** (LY's boundary 2†, GR's painting 3–4†, MD's opaque-image host glue inside its 2–3† track: no overlap; the SVG document model is uncounted); **print** (LY's paged layout, GR's print raster and vector interface: no overlap; the PDF writer is uncounted); **colour** (MD's `hg-colour` and maths 3–4†; GR's compositor colour output 3–4†: MD already excludes the GPU-side implementation, no overlap); **video compositing and overlays** (MD's OS media integration track listed "overlays with graphics" and GR's colour and video compositing track lists "video layers and overlays": one engineer counted twice; MD's track goes from 5–6† to **4–5†** and MD from 56–73† to **55–72†**); **reference engines** (ST's style reference, LY's layout reference 8–12†, GR's `hg-raster-ref`: distinct oracles, no overlap); **energy lab** (MD's 1–2† share, GR's 3–4† performance and energy lab, FP's 3–4†: MD counts only its share, GR's lab is graphics-specific, no overlap found); **observability** (each spec's track builds its own IIP domain with the DevTools track; DevTools' own staffing is outside the eight specs). **Changed.** MD 9.3; LY 9.3; GR 9.3.

### Q31. Licence and M-1 residuals

**Consistency.** No shipped dependency without an MIT choice was found in the three specs: LY uses only `smallvec`, `bitflags` and `hashbrown` without `foldhash`; MD only generic crates and its own D-Bus client unless Q-MD-16 finds an MIT tree; GR only MIT-choice binding crates, with Vulkan and SPIR-V bindings generated by Izcal and `libloading` (ISC) and `spirv` (Apache-2.0 only) excluded. No link to engine source code was found in any of them (checked: the only `github.com` links are a Servo wiki page, CSSWG issues, the Vulkan loader's documentation and Microsoft's INF-0004 proposal, all prose). Items for the owner: (a) **all three specs cite prose documents hosted in code repositories** (Mozilla's source docs, Chromium's Markdown design documents, the Vulkan loader's documentation, INF-0004, the ORB explainer); GR asks for one decision-log line, which this note makes **OD-7**; (b) classifications GR asks the owner to confirm: the Vulkan loader as an OS interface on G67's condition, the SPIR-V grammar under the standards-data exception, and `dxil.dll` and `d3dcompiler_47.dll` as test-only black-box oracles of JS-D9's class; (c) MD-D4's codec tables that exist only inside RFC code components (VP8, Opus): RFC 6386's code is a codec library's code, so this note recommends that no table be taken from it, and that such tables come from the narrative, formulas or black-box derivation, or the owner decides per table; the AOM patent-licence notice is a notice, not a dependency (owner confirms inside MD-D4); (d) LY's Q-LY-19 (MathML operator dictionary as standards data). The security-scope and coding-standard links in MD and GR follow the accepted specs' convention: they resolve when `spec/security-scope` and the coding-standard branch merge. **Owner decision OD-7.** **Changed.** GR 9.5; LY 9.5; MD 9.5.

### Q32. Stale cross-references

MD cites the graphics spec as "not yet written" and LY's rules as G37–G43; LY cites a "future graphics spec"; GR cites MD 0.2.1 and LY 0.2. Each header now cites the others at 0.3 and this note. **Changed.** Headers and 9.5 rows of LY, MD and GR.

## 3. Footprint budget results

Charged memory, settled, from FP 8.2 and the specs as reconciled, in MiB. "Holds" means the arithmetic of the stated budgets fits; nothing is measured. Two columns: GR 0.2 as written (every content GPU process ≤ 12 MiB†, a spare and the shell's process with devices), and with OD-5 (a key's process starts with a rendering opportunity, device-less ≤ 2 MiB† until it needs a device, ≤ 12 MiB† with one). Page sizes are FP's: median page 32 (FP-5), p90 96, iframe processes at F_fixed 4 (FP-2); whole-browser size of one visible corpus page about 99 + 12 = 111; of one iframe-heavy page about 64 + 96 + 3 + 12 = 175.

| Budget (FP) | After P10 (reconciliation 1) | GR 0.2 as written | With OD-5 | Result |
|---|---|---|---|---|
| FP-1 baseline ≤ 64 | Broker 20, shell 14, GPU service CPU side 12 (now the compositor), network 4 + spare 1.5, storage 4, crash 2, spawner 1 = **58.5** | + shell's content GPU process 12 = **70.5** | + device-less shell process 2 = **60.5** | **Breaks as written; holds with OD-5** (3.5 MiB left). Audio service and capture service: 0 at idle (exit when unused) |
| FP-S30 at settle ≤ 1,280 | 30 × 32 + 64 + 30 partition services × 3 = **1,114**; 166 left for the 3 iframe-heavy pages' frame processes | Visible tab median: + spare 12 + main 12 = 1,138; 3 iframe-heavy pages × 10 frames × 4 = 120 → **1,258**. Visible tab iframe-heavy (10 frames): + 10 × 12 more → **1,378** | Visible median: + spare 2 + main 12 + 120 = **1,248**; visible iframe-heavy (10 frames): + 10 × 2 = **1,268** | **Breaks as written by about 100 when the visible tab is iframe-heavy; holds with OD-5** (12–32 left). Hidden tabs' content GPU processes have exited (30 s after hidden), media processes exited (10 s idle), no hardware decode on a static corpus |
| FP-S30, iframe-heavy pages at 15 frames | — | 3 × 15 × 4 = **180** for frame renderers alone, above the 166 left after P10 | Same | **Breaks before graphics is counted**: P10 left this unexamined; FP-S30 holds only if iframe-heavy pages average ≤ 13 cross-site frames (166 ÷ 12 renderers) or their frame processes settle below F_fixed. Recorded as an open measurement for FP-M3 |
| FP-6 after 10 min idle ≤ 768 | 29 × 12 + 32 + 64 + network 39 = **483** (frame renderers not counted) | + 24 + 30 frozen frame renderers × 4 = **627**; visible iframe-heavy: **747** | + 14 + 120 = **617**; visible iframe-heavy: **637** | Holds either way (21–151 left) |
| FP-7 corpus ≤ 5% | Renderer F 4 per cross-site frame; allowance about 5.5 per page (5% of 111) → ≤ 1.4 visible cross-site keys per page on average | + 12 per visible key → ≤ **0.34** keys per page | + 2 per visible key → ≤ **0.9** keys per page | **At risk**: the corpus's average of visible cross-site frames per page is unmeasured (FP-M3) |
| FP-7 iframe-heavy ≤ 25% | 10 frames × 4 = 40 of an allowance of about 44 (25% of 175): holds barely | 10 × (4 + 12) = **160** | 10 × (4 + 2) = **60** | **Breaks with any per-key out-of-renderer raster process above about 0.4 MiB**; G56 forbids raster of opaque images in the renderer, so OD-5 (d) restates FP-7's scope as OD-2 did |
| FP-38 media process ≤ 2 idle | Per (site, container) | — | Per renderer key (Q7); pixels reach the content GPU process with 0 extra copies | Holds; more processes only where FP-D8 splits subdomains |
| FP-2 empty site process ≤ 4 | — | — | — | Unaffected (no graphics, layout or media fixed cost beyond LY-31's ≤ 64 KiB† inside the 0.25 MiB document line) |
| FP-5 median page ≤ 32 | — | Display items leave the renderer (≤ 88 B† per node, counted in FP-11) | Same | Holds by construction of the lines |
| FP-11 512 B† per node | 72 + 80 + 360 | Layout 200 (expected; 276 high) + paint 160 (72 renderer + 88 items) | Same | Holds at expected values; failure order Q3 |
| FP-14 GPU memory | Atlas per (site, container, partition) | Atlas per renderer key and partition, in the content GPU process; new lines: View Transition snapshots, OffscreenCanvas, shared-tile pinning; hardware-decode frames on the "video" kind | Same | Rows to add (listed in GR 9.5 and MD 9.5) |
| New process kinds not in FP 8.2 | — | Content GPU process ≤ 12 (GR-47); hardware-decode ≤ 8 while decoding (MD-53); audio service ≤ 4 with a stream, 0 exited; capture service ≤ 6 + frames while capturing; **CDM process (Windows, during EME): unbudgeted** | Content GPU process ≤ 2 device-less, ≤ 12 with a device; CDM process memory reported as an attributed OS-code line, as P13 did for IME DLLs, measured by MD-S5 | Rows to add (GR 9.5, MD 9.5) |

**What breaks, in one sentence.** With GR 0.2 as written, FP-1 and FP-S30 (iframe-heavy visible tab) break and FP-7 fails by a factor of about four on the iframe-heavy set; with OD-5, FP-1, FP-S30 and FP-6 hold on paper, FP-7's corpus limit is at risk until the corpus is measured, and FP-7's iframe-heavy limit cannot hold with G56 unless its scope is restated; FP-S30 at 15 frames per iframe-heavy page breaks even without graphics, a finding against P10 recorded for FP-M3.

**FP amendments needed, by the spec that lists them.** GR: FP 6.1 model (content GPU processes as a term, by rendering opportunity); FP-1 (the shell's device-less content process, 2 MiB†); FP 8.2 rows (content GPU process device-less and with device; spare); FP-7's scope (OD-5); FP-11 across processes; FP-14 additions; FP-27 spawn row; FP 6.5's region owner. MD: FP-D11 and FP-38 by renderer key; FP 8.2 rows (hardware-decode, audio, capture, CDM line); FP-13 class W; FP 6.8 hidden media; MD-54 and MD-55 as FP-22-style gates. LY: FP 6.4 cache eviction, store rows, LY-31's line.

## 4. Staffing and the Stage 1 critical path

Full-strength staff† as each spec states it after this pass. People include TX's type designers and NW's root-program and list staff; engineers exclude them.

| Spec | Full strength† | Of which engineers† | Start (M0)† | Month 12† | Person-years to parity† | Parity† |
|---|---|---|---|---|---|---|
| JavaScript engine (JS) | 60–75 | 60–75 | 20–25 | about 40 | about 265–460 | 5–7 years |
| Resource footprint (FP) | 3–4 | 3–4 | 3–4 | 3–4 | about 15–25 | With Stage 1 exit |
| Parsing and style (ST) | 31–40 | 31–40 | 14–17 | about 27 | about 130–215 | 4.5–6 years |
| Fonts and text (TX) | 79–90 | 34–45 (plus about 45 type design) | 18–23 | about 40 | about 115–185, plus 150–400 designer-years | 4–5 years (font set about 5–10) |
| Networking (NW) | 58–81 | about 50–68 (plus 8–13 root program and lists) | 21–26 | about 45 | about 200–340 | 4–5 years |
| **Five accepted specs** | **231–290** | **178–232** | **76–95** | **about 155** | **about 725–1,225**, plus 150–400 designer-years | — |
| Layout (LY) | 52–66 | 52–66 | 18–22 | about 40 | about 260–415 | 5.5–7 years |
| Images and media (MD) | **55–72** (was 56–73; Q30) | 55–72 | 18–22 | about 40 | about 240–435 | 5–7 years |
| Graphics and rendering (GR) | 99–132 | 99–132 | 25–30 | about 60 | about 420–780 | 5–7 years |
| **Eight specs** | **437–560** | **384–502** | **137–169** | **about 295** | **about 1,645–2,855 engineer-years**, plus 150–400 designer-years | — |

**Honest reading.** The eight components need about **14 to 24 times** Stage 1 8.3's whole pre-2.3.0 plan of 23–30.5 engineers (437 ÷ 30.5 ≈ 14.3; 560 ÷ 23 ≈ 24.3), and start with 137–169 specialists at M0, rising to about 295 within a year; hiring (JIT, GC, shaping, QUIC, layout, codec, GPU-compiler and driver specialists, and type designers) is the largest schedule risk every spec names. **Not counted anywhere:** the DOM, bindings and event loop (Stage 1's 5–6), shell, storage, security, infrastructure and DevTools tracks, and the unowned specs this pass found: **SVG** (document model, SMIL, SVG DOM, geometry APIs), **PDF** (writer and viewer), **animations** (Web Animations, CSS transitions and animations, owner of `hg-interp`), **editing**, the **ML service** (MD-D25's models), and accessibility beyond each spec's share. Four tracks continue after parity: NW's root program and lists (8–13†), TX's type staff (6–10†), MD's codec maintenance (20–25†) and GR's device lab (4–6†). D-S8 (funding) must cover all of it, plus MD-D3's licence option, ISO texts, the HRTF lab, listening panels and external audits.

**Stage 1 critical path.** Combined Stage 1 exit is **48–60 months†**, the latest of the specs' Stage 1 exits:

| Spec | Stage 1 exit† | What sets it |
|---|---|---|
| NW (NW-M5) | 36–48 | Own path, in parallel |
| JS (JS-M3) | 39–51 | T2 and W2 through their gates |
| ST, TX, MD (ST-M3, TX-M3, MD-M3) | "With Stage 1 exit" | Not on the critical path: their last milestones before exit land at 24–33 months |
| **LY (LY-M3)** | **48–60** | LY-M2's breadth: every formatting context fragmentable, re-entry, anchors, the reference engine for every context |
| **GR (GR-M3)** | **48–60** | Placed in layout's window; its own chain (HAL and shader compiler → WebGL at GR-M2, 30–42 → WebGPU through its gates) is not shown to finish earlier |

The path: **JS-M1, ST-M1 and TX-M1 (15–21) → LY-M1 (18–24, at least 3 months after them) → GR-M1 with LY-M1 (first pixels, Stage 1 M4, 18–24) → LY-M2 (36–54) → LY-M3 and GR-M3 (Stage 1 exit, 48–60)**. JS keeps about 9 months of float and NW about 12. ST and NW cite the JS spec's 39–51 months as Stage 1 exit; that becomes 48–60 (listed in LY 9.5). Stage 1 8.1's milestone dates are replaced by the specs' (Q29).

## 5. Amendments to the accepted specs, HGBA-2, HGBA-3 and Stage 1 (not applied)

Each row is listed in the amendments section (9.5) of the spec named in the last column.

| Document | Amendment | Item | Listed in |
|---|---|---|---|
| FP 6.1, 8.2, FP-1, FP-27 | Content GPU processes as a model term, started by rendering opportunity; device-less ≤ 2 MiB† and with-device ≤ 12 MiB† rows; the shell's device-less content process in FP-1; spare; spawn row | Q8 (OD-5) | GR |
| FP-7 | Scope: renderer isolation; per-key service processes (content GPU, media, hardware-decode, CDM) on their own reported line with absolute budgets, never co-located | Q8 (OD-5) | GR |
| FP-11 | Paint data counted across processes; 200 B† / 160 B† co-signed; one failure order | Q3 | GR, LY |
| FP-14, FP 6.5, FP 6.6 | Atlas per renderer key and partition in the content GPU process; regions fixed by the content GPU process; new GPU lines | Q11, Q8 | GR |
| FP-D11, FP-38 | Media process per renderer key (origin under FP-D8); hardware-decode and CDM likewise | Q7 | MD |
| FP 8.2 | Rows: hardware-decode, audio service, capture service; CDM process as an attributed OS-code line | Q8 | MD |
| FP-S30 | Open measurement: frame processes of iframe-heavy pages at 15 frames exceed P10's headroom | Q8 | GR |
| FP-22 rule | GR-43 and MD-54 as absolute FP-22-style gates | Q27 | GR, MD |
| JS-D15 | Consolidated decision points of LY, GR, MD; one whole-renderer recording budget with per-subsystem shares | Q15 | LY, GR, MD |
| JS-D19 | Segments in the substrate; layout as a second user; per-OS fallback interplay with LY-D15 | Q16 | LY |
| JS `hg-math` | f32 entry points (MD); f64 for feTurbulence and colour tables (GR) | — | MD, GR |
| TX 9.5 R21 | The bounded path types are in `hg-display-list` (L2), not L2g | Q18 | GR |
| TX 6.10, FP-D14 | `hg-glyph-atlas` in the content GPU process of its renderer key | Q11 | GR |
| ST, TX, NW Stage 1 exit citations | 39–51 months becomes 48–60 | Q29 | LY |
| Stage 1 D-S1 and decision log | One noise rule (Q14), owner signature | Q14 (OD-6) | GR (MD, TX cite) |
| Stage 1 decision log, M-1 part 3 | Prose documents in code repositories count as prose under conditions | Q31 (OD-7) | GR |
| HGBA-2 I-1 | One list: compositor GPU service, audio service, capture service | Q10 (OD-8) | GR, MD |
| HGBA-3 PRIV-2 (a) | Distinguishability instead of identical identifiers | Q14 | GR |
| HGBA-3 DET-1 | One recording budget; pixel scope (GR-51); media scope (MD-41) | Q15 | GR, MD |
| Stage 1 1, 2.1, 2.2, 3.3, 8.1, 8.3 | Process map with all new kinds keyed by renderer key; layers L2m, L2g, `hg-simd` in L2s; registry G37–G72 with gaps recorded; milestones re-dated; eight-spec staffing | Q7, Q18, Q19, Q29, Q30 | LY, MD, GR |
| Stage 1 13.2 | Print UI and PDF writer stay Stage 2; print raster and paged layout Stage 1; PDF spec to be owned | Q24 | GR, LY |
| Licence audit | Classifications of Q31 (b) and (c) | Q31 | GR, MD |

## 6. Decisions for the owner

A bare "yes" means the recommendation.

| ID | Decision | Recommendation | Why |
|---|---|---|---|
| **OD-5** | (a) Content GPU process starts only with a rendering opportunity; (b) starts device-less, CPU raster with identical bits, device on demand, ≤ 2 MiB† device-less and ≤ 12 MiB† with a device; (c) shell and spare processes device-less; (d) FP-7 measures renderer isolation, with per-key service processes on their own reported line and absolute budgets, never co-located | **Yes** | As written, GR-D2 breaks FP-1 and FP-S30 and fails FP-7 by about four times; (a)–(c) cost nothing visible because canonical pixels are placement-independent; (d) is OD-2's precedent, and G56 makes per-key raster outside the renderer a security property that FP-7 must not trade |
| **OD-6** | One D-S1 amendment: noise only where non-canonical output leaves for a readable surface (WebGL and WebGPU pixels, audio from OS tolerance decoders, video from OS decoders not passing MD-42 on that device); none on 2D canvas, export, `measureText`, Izcal-rendered audio or Izcal-decoded media; recordings keep pre-noise data; PRIV-2 (a) as distinguishability | **Yes** | Canonical output removes device entropy without noise, noise is detectable and breaks pages, and the three specs otherwise contradict each other (MD 6.4 against GR-D12) |
| **OD-7** | Prose documents hosted in a code repository (Markdown design documents, generated documentation sites, specification proposals) count as prose under M-1 part 3, on three conditions: no code file, header, diff or review is opened; a copy outside the repository is cited where one exists; no constant that appears only beside code is taken | **Yes** | All three new specs, and earlier ones, rely on such documents; one rule removes the per-spec grey zone |
| **OD-8** | HGBA-2 I-1 records three shared-process exceptions in one list: the compositor GPU service, the audio service and the capture service, each parsing no page payload, each with a threat model | **Yes** | Each must see every site's output or grants by function; none parses, and the alternatives give renderers devices (I-2) or put every site's pixels in a parsing process |

Owner confirmations, not decisions (Q31): the Vulkan loader as an OS interface on G67's condition; the SPIR-V grammar as standards data; `dxil.dll` and `d3dcompiler_47.dll` as test-only black-box oracles; MD-D4's tables only inside RFC code (recommendation: not used); Q-LY-19.

[LY]: ../../reports/Izcal%20layout.md
[MD]: ../../reports/Izcal%20images%20and%20media.md
[GR]: ../../reports/Izcal%20graphics%20and%20rendering.md
[JS]: ../../reports/Izcal%20JavaScript%20engine.md
[FP]: ../../reports/Izcal%20resource%20footprint.md
[ST]: ../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[TX]: ../../reports/Izcal%20fonts%20and%20text.md
[NW]: ../../reports/Izcal%20networking.md
[H2]: ../../reports/HGBA%202%20browser%20architecture%20spec.md
[RN]: reconciliation.md
[LA]: licence_audit.md

**Addendum (19 October 2026).** The owner accepted OD-5 to OD-8 and every recommendation of the layout, images and media, graphics and rendering, and content blocking specs.
