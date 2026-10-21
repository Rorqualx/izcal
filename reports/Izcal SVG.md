# Izcal SVG

*First-principles specification of Izcal's own SVG engine: the SVG document model on the DOM's node storage (SVG 2 elements and attributes, the SVG DOM, presentation attributes into the cascade), SVG layout and geometry (viewports, `viewBox`, `preserveAspectRatio`, units, the layout spec's SVG boundary, text and `textPath` with the fonts and text stack), the render tree handed to the graphics spec, which paints (fill, stroke, markers, paint servers, clipping, masking, filters), `<use>` and its shadow trees with resource budgets, SVG as an image and the scriptless document host (MD-D20, reconciliation 2 Q25), SVG in CSS, SMIL with the animation spec, SVG fonts, SVGZ, inline SVG in HTML parsing, accessibility (SVG-AAM with the accessibility spec), hit testing and `pointer-events`, security, privacy, determinism, footprint and observability. Date: 20 October 2026. Status: draft 0.2, phase 8 applied. Research notes: `research_notes/Izcal SVG/` ([`frame.md`][FR], `history_postmortems.md`, `specs_standards.md`, `rendering_text_geometry.md`, `animation_smil.md`, `security_robustness.md`, `usage_interop.md`, `gaps_opportunities.md`; phase 8: [`factcheck.md`][FK], [`critique.md`][CR]). Pinned specification: [SVG 2, Candidate Recommendation Snapshot of 6 October 2026](https://www.w3.org/TR/2026/CR-SVG2-20261006/), checked against the Editor's Draft of 13 October 2026. Relates to: HGBA-2 2.3.3 [M-1, P1–P6, I-1, I-2, C-1, ANC-1][H2]; HGBA-3 [B1, B3, B4, B5, B8, B9, B10, DET-1][H3]; Stage 1 design [Sections 2 (G1–G9), 3, 5][S1]; JavaScript engine spec [JS-D4, JS-D12, JS-D15, JS-D17, JS 6.9, G6][JS]; resource-footprint spec [FP-4, FP-11, FP-13, FP-17, FP-34, FP-38, FP-D4, FP-D19, FP 6.3, 6.4][FP]; HTML and CSS parsing and style spec [ST-D5, ST-D18, ST-D20, ST 6.2, 6.8–6.11, G19–G24, Q-ST-19][ST]; fonts and text spec [TX-D4, TX-D7, TX 6.3, 6.8, G25–G29][TX]; layout spec [LY-D3, LY-D10, LY 6.0, 6.5.10, 6.14, 6.15, G37–G45][LY]; images and media spec [MD-D5, MD-D19, MD-D20, MD-S10, Q-MD-18, G50, G56, MD-56][MD]; graphics and rendering spec [GR-D3, GR-D13, GR-D17, GR-D18, GR 6.3, 6.4, 6.16, 6.28, G60–G72][GR]; DOM, bindings and event loop spec [DM 6.1, 6.5, 6.6, 6.12, 6.18, G90–G99][DM]; accessibility spec [AX 6.2, G100–G109][AX]; PDF spec [PD-D19, PD-D20][PD]; DevTools spec [IIP][DT]; [reconciliation 2 Q20, Q25, Section 4][RN2]; [Phase 3 ledger G-12, A-FUT-1, A-FUT-5][LG]; [animation spec](Izcal%20animation.md) draft 0.1 [AN-D6, 6.8, 6.14] for SMIL.*

*Revision 0.2 (20 October 2026): phase 8 applied, by a fact-checker and a design critic who did not write the spec. The [fact-check][FK] checked 146 claims (121 correct, 17 partly correct, 5 wrong, 3 unverifiable) plus 52 cross-references, screened 372 note URLs (6 dead, now replaced or cited by title; 127 fragments, none missing) and found no source-code link (three borderline repository and changeset links in the notes, now removed or pointed at README prose); its 11 must-fix and 24 recommended items are applied. The [design critique][CR] raised 36 findings (9 High, 16 Medium, 11 Low), 9 engine-source and licence items and 18 reconciliation items; all are applied, several in a different form (Appendix A). What changed: **`hg-node-store` writes only through a typed `StoreWriter<Barrier>`**, so JS-D4's write discipline holds for scripted documents (6.1); **static documents are their own style scopes**, because style, atom and fragment tables are per agent (6.1); **rendering-only HTML behaviour is factored into `hg-html-render`** so `foreignObject` renders the same in an image (6.1); **the instance builder is not DOM's clone algorithm**, instances dispatch to their source's listeners, and the `use` shadow root's mode is one flag because the Editor's Draft moved it from open to closed a week after the CR (6.7); **one weighted ledger in canonical units**, maintained as a prefix sum over document order, replaces independent caps, and filter work is counted in CSS pixels (6.13); **G122 lists the only allowed floating-point operations** and style's computed lengths must be binary64 (6.6); no crate depends upward (`hg-svg-instances` in L3, `hg-svg-geom` and `hg-svg-interp` in L2p, traits for L3-to-L4 calls); SVG effects substitute transparent black for opaque images (6.5); hit testing is the user-space geometry (6.11); local SVG files open in a **scriptless interactive** mode with links, find and same-directory files (6.13); reduced motion pauses SMIL at t = 0 (6.9); an opaque-image raster protocol that leaks nothing to the renderer (6.8); SMIL keeps firing events off-screen when observed (6.9); eight targets added or restated; staffing 16–18†; after the animation spec's report appeared, SV-D4 is aligned with its AN-D6 (SMIL timing in `hg-anim`).*

*Revision 0.1 (20 October 2026): first draft, written by the synthesis author from seven research notes (six strands by independent researchers in parallel, and the frame by the synthesis author).*

SVG is a majority feature of the web that nobody at Izcal owns. Chrome counts an inline `<svg>` on **68.5%** of page loads and at least one SVG image decoded on **57.5%** ([chromestatus 250](https://chromestatus.com/metrics/feature/timeline/popularity/250); [US 1.3–1.4][US]), yet the ledger lists the SVG document model, SMIL, the SVG DOM and the geometry APIs as gap **G-12**, reconciliation 2 says the SVG spec's staffing "is counted nowhere" (Q25), and five accepted specs already call into it: graphics paints SVG from "typed records" this spec must define (GR 6.28), layout sizes the outer `<svg>` and leaves everything inside to it (LY 6.5.10), media rasterizes cross-site SVG images in a sandboxed process only if "the DOM can exist without a JS agent heap" (MD-S10, Q-MD-18), the fonts stack renders OT-SVG glyphs with "Izcal's SVG engine" (TX-D7), and the DOM spec leaves "SVG DOM, SMIL" here (DM 6.18). The record teaches three things. **First, SVG is one of the least interoperable mature parts of the platform**: only 891 of the 1,405 WPT `svg/` files in Chrome's run (63%) pass in all three major engines ([wpt.fyi `svg/`](https://wpt.fyi/results/svg?label=stable)), `svg/path` passes 54.7% in Chrome and 38.3% in Safari, and vendors turned down whole-SVG Interop proposals four years running ([US 3.1–3.18, 5][US]); "SVG issues" was the top content pain point in State of HTML 2024 ([US 6.1][US]). **Second, the specification is unfinished, moving and, in places, wrong**: SVG 2 became a Candidate Recommendation again on 6 October 2026 after eight years, a week later its Editor's Draft closed the `use` shadow tree the CR calls open, its text-layout pseudo-code has five defects, it sets no limit on `<use>` expansion, and its two processing-mode documents disagree ([HP 1.5][HP]; [FK][FK] FC-M1; [RG 1.5][RG]; [SS 1.8, 4.7][SS]). **Third, SVG's dangerous parts are its dynamic ones**: an exploited Firefox use-after-free lived in SMIL timing (CVE-2016-9079, on CISA's exploited list since 22 June 2023), nested `<use>` and patterns are SVG's "billion laughs", filter timing leaked cross-origin pixels from 2013 to 2023 and filter logic over framed pages enabled clickjacking in 2025, and SVG attachments became a 2025 phishing vehicle ([CERT/CC VU#791496](https://kb.cert.org/vuls/id/791496); [SR 1–3, 6][SR]; [GO 4.4][GO]).

**The central idea: one SVG engine, two kinds of host, and explicit structure in between.** Every SVG document, inline or image, scripted or not, goes through the same parsers, cascade, layout pass and render tree; a scripted document lives in the DOM with its agent heap, and a scriptless one lives in a **static document host**, over the same column store with a typed write capability, that has no collector, no realm and no fetch and is its own style scope, which is what makes SVG images cheap, safe and possible in the media process. Between the tree and the painter sits an explicit **render tree** and an explicit **reference graph**, computed in the layout flush, never by script, so painting, hit testing, accessibility and the geometry APIs read one source of truth, and every resolved reference, instance and budget event is visible to developers. The decisions that matter most: (1) the **scriptless host over a GC-free column store** shared with the DOM, answering Q-MD-18 with yes (SV-D1); (2) **IEEE-strict binary64 geometry** with integers from the graphics boundary on, because SVG user units have no bounded range (SV-D2); (3) **SMIL on the animation spec's one effect model**, its timing in `hg-anim` as AN-D6 decided, its elements, attributes, geometry and value interpolation here (SV-D4); (4) **one deterministic weighted ledger per document** for `<use>`, references, filters and SMIL, a pure function of the tree (SV-D6); (5) **local SVG files open without script**, with links, find and sibling files, and script one action away (SV-D9). The work is about **16–18 engineers†** (Section 9), counted nowhere until now.

Note sources are cited as **HP** ([history and postmortems][HP]), **SS** ([specifications and standards][SS]), **RG** ([rendering, text and geometry][RG]), **AN** ([animation and SMIL][AN]), **SR** ([security and robustness][SR]), **US** ([usage and interoperability][US]) and **GO** ([gaps and opportunities][GO]), with the row number. Primary sources are linked inline for the claims a decision or target depends on.

## 1. The problem and who has it

| User | Task, in their words | What fails today |
|---|---|---|
| **People browsing** | "Icons, logos, charts and maps should be sharp, fast and safe; an image must not run code" | SVG attachments are now a phishing vehicle because the OS opens them in the browser, where they run as pages: Kaspersky counted 2,825 such emails in Q1 2025 and KnowBe4 put SVG at 6.6% of malicious attachments ([SR 6.1–6.3][SR]) |
| **Web developers and designers** | "Why is `getBBox` different in each browser?", "why doesn't my font load in `<img>` SVG?", "why does my `<use>` from a CDN fail?", "why is this filter slow?" | `getBBox` options ship only in Firefox; Chrome and Safari fail subtests in six of ten `getBBox` files ([RG 2.8][RG]; [US 4.10][US]); SVG images cannot load fonts or inherit `currentColor` ([US 6.3–6.4][US]); cross-origin `<use>` was deferred past SVG 2 ([GO 6.3][GO]); SVG filters are "long neglected" by an author who files their bugs ([US 6.5][US]) |
| **Animation authors** | "My SMIL animation works in Chrome but not Safari", "which animation is running?" | Safari fails 68 of 321 `svg/animations` files, clustered in syncbase and event timing ([AN 3.2][AN]); neither major animation inspector lists SMIL ([GO 1.1][GO]) |
| **Data-visualisation authors** | "Ten thousand points, hover and zoom" | Published SVG ceilings are "a few thousand individual marks" and no engine publishes element-count scaling ([GO 3.1–3.4][GO]) |
| **Accessibility users** | "Tell me what this graphic is; let me find and read the text in it" | Without ARIA, `title` and `desc` were almost unsupported in 2013; SVG inside links and buttons still fails in several pairings; SVG text is often not text to tools ([GO 2.1–2.7][GO]) |
| **Izcal itself** | "Render OT-SVG glyphs, our own UI icons and cross-site SVG images without a JS heap per image" | FP-34 puts each further agent heap, with an empty document, at ≤ 768 KiB†; a page with 50 SVG icons could pay up to about 37.5 MiB† if each image had one |
| **Agents and tests** (agents only with AI on, B9, B10) | "Read the values in this chart", "the screenshot must match" | Language models misread SVG chart markup and invent points ([GO 5.1][GO]); Playwright documents that screenshots vary by OS, hardware and power source ([GO 5.3][GO]) |

**What Izcal's specs already decide** (the full list is in the [frame][FR]):

- **[M-1][H2] (2.3.3).** No library from a browser project or doing engine work: resvg, usvg, librsvg, Batik and the rest are history and black-box oracles only. MIT only; W3C standards data allowed (the SVG UA sheet is confirmed, Q-ST-19). No third-party project's source code is evidence.
- **Node storage (JS-D4, DM 6.1).** One copy of the DOM, node columns outside the cage traced by `hg-heap` as its node space; style, layout and paint read it through `hg-tree-traits` only (Stage 1 G1, ST G1, LY G1, AX G100).
- **Painting (GR 6.16, 6.28, GR-D3).** Graphics owns `hg-svg-paint` and every pixel, in integer arithmetic; it consumes this spec's typed render tree; paths travel in the bounded path format (24.8 fixed point, 65,536 segments).
- **Layout (LY 6.5.10, LY-D3).** Outer `<svg>` sizing, `foreignObject` as a layout root, an inline-layout call returning per-character positions; SVG geometry for CSSOM View through a trait this spec answers; layout numbers are integers after a boundary.
- **Images (MD-D20).** Same-origin and CORS-approved SVG images render in the renderer; opaque cross-site ones in the media process, once MD-S10 answers Q-MD-18.
- **Text (TX-D4, TX-D7).** Font size quantized in the drawing context's local units, so SVG user units; OT-SVG glyphs in a restricted glyph mode of Izcal's SVG engine.
- **Determinism, footprint, privacy** (B1, DET-1, FP-11, FP-D19, D-S1): outputs bit-identical across runs, OSes and three ISAs; limits identical on every device.
- **Admission (ST-D18, LY-D10).** Features from drafts ship when a WG resolution, WPT tests and two shipping engines (or an Interop focus area) exist.

**Success, in measurable terms**: WPT `svg/` at or above the best shipping engine per directory; geometry APIs bit-identical to an independent reference; SVG image rasters identical whether made in the renderer or the media process; no script and no fetch in any secure mode, proved by construction and probed; every amplification attack bounded by a deterministic budget; no JS heap per SVG image; every resolved reference, instance and SMIL interval visible to developers (Section 8).

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1998 | **PGML** (Adobe, IBM, Netscape, Sun) and **VML** (Microsoft, Macromedia, HP, Autodesk, Vision); SVG WG chartered 24 July | Competing XML vector formats; SVG designed new, drawing on both ([HP 1.1–1.2][HP]) | XML was the W3C's answer to everything; PostScript's imaging model was the known quantity | VML shipped in IE5 and stayed Microsoft's only vector format until IE9, which kept SVG out of most browsers for a decade ([HP 2.4][HP]) |
| 1999–2009 | **Adobe SVG Viewer** | The de-facto SVG renderer for IE and Netscape, as a plugin ([HP 2.1][HP]) | No browser had native SVG | End of support 1 January 2009, never ported to Vista; no official reason; Adobe owned the competing Flash ([HP 2.1][HP]) |
| 2001–2003 | **SVG 1.0, SVG 1.1, SVG Tiny and Basic** | Full XML vector graphics with DOM, SMIL animation, SVG fonts, filters; mobile profiles mandated by 3GPP ([HP 1.3–1.4][HP]) | Phones without styling or scripting; a document format, not a web feature | Mobile profiles faded once phones ran desktop engines; Tiny 1.2 (2008) diverged and was never implemented whole ([HP 1.4][HP]) |
| 2001 | **SMIL Animation** | A timing model (begin/end lists, syncbase, events, sandwich) that SVG embedded ([AN 1.2][AN]) | Multimedia presentation semantics, generalized | Still the normative definition SVG animations defer to ([SS 6.1][SS]) |
| 2004–2011 | **Native support**: KSVG (2004), Opera 8 (2005), Firefox 1.5 (2005), WebKit (2006), **IE9 (2011)** | Engines implemented SVG 1.1 subsets ([HP 2.2][HP]) | Each engine built SVG as its own subsystem beside HTML | IE9 was the turning point; by 2011 every desktop browser had some SVG |
| 2010–2011 | **Inline SVG in HTML** via the HTML5 parser | Foreign content rules, case-fixing tables, integration points ([HP 2.3, 5.2][HP]) | Authors wrote HTML, not XHTML | The reason SVG became a majority feature; also the source of namespace-confusion mXSS ([SR 4.4][SR]) |
| 2013 | **Firefox SVG text on the HTML text engine** | 50 patches reused HTML's layout for SVG text: decorations, find, selection, bidi ([RG 1.15][RG]) | SVG text had been "greatly non-interoperable and somewhat underspecified" | The model every engine now follows: CSS lays out, SVG positions ([RG 1.1][RG]) |
| 2014 | **SVG fonts removed from Chrome 38** | — | Chrome's own reason is not documented; the SVG WG recalled in 2015 that the debate began when Microsoft said it would implement neither SVG fonts nor SMIL, and Mozilla objected in 2010 on low use ([HP 3.1][HP]) | Never in Firefox, IE or Edge; still in Safari; removed from SVG 2 ([US 4.25][US]) |
| 2014 | **SVG Integration** (first draft) | Defines processing modes for images, resources and fonts ([HP 1.9][HP]) | Images must not run script or fetch | Never advanced; SVG 2 now carries the modes and disagrees on resource documents ([SS 1.6–1.8][SS]) |
| 2015 | **Blink's intent to deprecate SMIL** (29 April) | Proposed removal at 0.0403% of page views ([blink-dev](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/59rZqirUQNwJ); [HP 3.2][HP]) | "Significant complexity", tear-offs, Oilpan integration; Web Animations as the replacement | **Suspended in August 2016**: "use cases ... that just don't have high-fidelity replacements yet" ([AN 1.10][AN]); SMIL is now on 2.90% of page loads, up from 0.58% in January 2019 ([AN 6.1][AN]) |
| 2016–2018 | **SVGPathSeg removed** (Chrome 48, Firefox 59) | The path-segment DOM went away ([HP 3.5][HP]) | Live segment objects were costly and little used | Its replacement `getPathData` shipped only in Firefox 137 (2025); Blink announced a prototype in June 2026 ([HP 3.5][HP]) |
| 2016–2020 | **librsvg's port from C to Rust** | A desktop SVG library rewritten in a memory-safe language ([HP 4.1][HP]) | Memory-unsafe parsers; a property model that "never ... matched the SVG/CSS specs" ([HP 4.2–4.3][HP]) | Bugs moved to logic (CVE-2023-38633's URL-parser disagreement) and C dependencies ([SR 7.2–7.3][SR]) |
| 2018 | **SVG 2 re-CR** | Mesh gradients, hatches, `solidcolor` cut for lack of implementations ([HP 1.7][HP]) | Specify only what ships | Then eight years without a publication |
| 2019–2023 | **`data:` in `use` and cross-origin `use`** | Cross-origin `use` deferred (2019); `data:` in `use` removed from Chrome 120 and Firefox 122 after Trusted Types bypasses ([GO 6.3][GO]; [SR 4.5][SR]) | Inserting foreign content into the host document is a script-injection vector | Same-origin `use` only |
| 2023–2024 | **WebKit's layer-based SVG engine** | Moved invalidation decisions into the style system after "the invalidation chain depended on the DOM order" ([RG 6.2][RG]) | Ad-hoc client lists and cycle checks | Not yet default as of its 2024 post |
| 2024–2026 | **SVG WG charters; SVG 2 CR (6 October 2026)** | The 2024 charter extended the group while a maintenance charter was drafted; the draft 2026 charter's mission is "to maintain SVG specifications"; a new CR with three at-risk features ([GO 6.1–6.2][GO]; [HP 1.5–1.6][HP]) | Too few participants for new features | Recommendation expected no earlier than 1 December 2026; the ED kept moving after it |

### 2.2 Technology mismatches: assumptions that no longer hold

| Assumption | Where it came from | Why it no longer holds | Evidence |
|---|---|---|---|
| SVG is an XML document format with its own subsystem | SVG 1.x's DTD, XLink, XML error handling | SVG is mostly inline HTML content, styled by CSS, laid out partly by CSS, with geometry as CSS properties | [HP 5.1, 5.4][HP]; [SS 2.2–2.4][SS] |
| Animated values must be visible to script as `animVal` | SVG 1.1's tear-offs | SVG 2 makes `animVal` an alias of `baseVal`; a WebKit participant called the tear-off support "costly to maintain" and Blink cited "significant complexity" | [SS 2.10][SS]; [HP 3.4, 5.3][HP] |
| SMIL is on its way out | Blink's 2015 intent | Use rose steadily from 0.58% of page loads (2019) to 2.90% (2026); with CSS animations, SMIL is the only animation inside SVG images | [AN 1.10, 3.8, 6.1][AN] |
| An SVG image is a small document that can share the page's machinery | Engines run SVG images as pages with their own scripting machinery switched off | A per-image agent heap costs up to 768 KiB† in Izcal (FP-34); cross-site image source in the renderer conflicts with G56 | FP-34; [MD 6.1][MD] |
| Single precision is enough | SVG 1.1's implementation notes | SVG 2 requires double precision for CTMs; precision-loss reftests still fail in an engine | [RG 3.3–3.5][RG] |
| Resource limits are an implementation detail | The specification sets none | Exponential `use` and pattern expansion are known attacks; Izcal's determinism needs the cut-off to be identical everywhere | [SS 4.7][SS]; [SR 2.1–2.4][SR] |
| Filters are a fixed-cost visual effect over the element's own pixels | SVG 1.1's filter model | Filter timing leaked cross-origin pixels from 2013 to 2023, and filter logic over a framed page enabled clickjacking in 2025 | [SR 3.1–3.8][SR]; [GO 4.4][GO] |
| An `.svg` file is an image | File managers and mail filters treat it so | Opened top-level it is a page with script | [SR 6.3][SR] |

## 3. Today: the landscape and its gaps

**Usage.** Inline `<svg>` grew from 10.1% of Chrome page loads on 1 January 2015 to 68.5% in October 2026; SVG is 6.4% of image requests on mobile; `filter: url()` is on 6.19% of page loads, `foreignObject` on 8.24% (from 0.32% in January 2025, cause unknown), external SVG resources on 4.49% ([US 1.3–1.5, 2.5–2.8][US]). Lighting, convolution and tile filters, `SVGPathSeg`, `xml:base`, SVG fonts and `<discard>` are near zero ([US 2.9–2.12, 4.14][US]).

**Interoperability.** On 19 October 2026, the 1,411-file `svg/` directory (1,405 files in Chrome's run, the base of the three-engine count) scored 87.2% in Chrome, 88.4% in Firefox and 73.3% in Safari (49.5% in Ladybird, 24.4% in Servo); path, text, linking and styling are the weak directories ([US 3.1–3.18][US]). The SVG 1.1 test suite is not automated in WPT ([US 3.19][US]). No engine implements all of SVG 2: Safari lacks the `d` property and `context-fill`; only Firefox ships `getBBox` options and `getPathData`; none wraps text ([US 4][US]; [RG 1.11][RG]).

**The standard.** SVG 2 is a Candidate Recommendation dated 6 October 2026 with three at-risk features (more than one `title` or `desc` for localisation, nested links, and `vector-effect` values other than `non-scaling-stroke`); SVG Animations Level 2 is an Editor's Draft that still defers to SMIL; SVG-AAM is a Working Draft dated 12 October 2026; SVG Integration never advanced; the WG is chartered for maintenance only ([SS 1.1–1.2, 6.1, 8.1][SS]; [GO 6.1–6.2][GO]). The specifications contradict each other on resource documents' processing mode ([SS 1.8][SS]), on mask luminance coefficients ([RG 4.5][RG]), and OpenType's SVG table still pins SVG 1.1 and discourages `context-fill`, which SVG 2's font style sheet uses ([SS 7.7][SS]).

### 3.1 What current tools cannot see or do, with evidence

| Gap | Evidence | What Izcal can do |
|---|---|---|
| SMIL is invisible to animation inspectors | Firefox and Chrome document CSS, transitions and Web Animations only ([GO 1.1][GO]) | A SMIL timeline beside the other animations, from `hg-anim`'s intervals (6.17) |
| SVG geometry has no inspector | `DOM.getBoxModel` returned nothing usable for SVG; no panel shows bbox kinds, CTM or viewport ([GO 1.3–1.4][GO]) | Overlays from the render tree (6.17) |
| Resolved paint servers, filter regions and intermediate filter results are not shown | The researcher's recollection, unverified ([GO 1.6][GO], Low) | `svg.references`, `svg.filterGraph` (6.17) |
| `use` instance provenance is invisible | The researcher's recollection, unverified ([GO 1.5][GO], Low) | `svg.instances` (6.17) |
| Why an SVG image did not load a font or resource | A recurring question ([US 6.3][US]) | `svg.imageInfo` (6.17) |
| Engines disagree with no way to tell which is right | 63% three-engine pass rate ([US 3.18][US]) | An independent reference implementation and spec-anchored algorithms (6.18, G129) |
| Screenshot tests differ by machine | Playwright's documentation ([GO 5.3][GO]) | Canonical rasters (GR-D3) |

## 4. First principles

### 4.1 Users' real tasks

1. **Draw what the author wrote, exactly, at any size.** Geometry, paint, text and effects as the specifications define them, re-rasterized at every zoom.
2. **Answer geometry questions exactly.** `getBBox`, lengths along paths, hit tests, with one answer on every machine.
3. **Animate declaratively, including inside images.** SMIL and CSS, without script.
4. **Reuse graphics.** Sprites, symbols, gradients and filters shared by reference, without unbounded cost.
5. **Use SVG as an image safely.** No script, no fetch, no user state, no cross-site bytes in the wrong process.
6. **Make graphics accessible and findable.** Names, roles, text that can be found, selected and read.
7. **Understand what happened.** Which reference resolved, which instance came from where, which animation won, which limit was hit.

### 4.2 Platform primitives

- **The specifications**: SVG 2 (CR 2026-10-06) with its element, attribute and IDL indexes ([SS 1.3, 2.1, 2.7][SS]); SVG Integration (processing and referencing modes); CSS Masking 1; Filter Effects 1; Compositing 1; CSS Transforms 1; SVG Animations Level 2 (ED) and SMIL Animation; Web Animations 1 and 2; SVG Paths (`getPathData`); SVG-AAM 1.0 and Graphics-AAM; HTML's parsing and rendering sections; Geometry Interfaces 1; the OpenType `SVG ` table. Each is pinned at a dated snapshot (M-1 part 2).
- **The engine's data**: the node columns and mutation log (JS-D4, DM 6.1); computed styles and the "affects" mask (ST 6.8); fragments and the inline-layout call (LY 6.5.10); shaped text with cluster maps (TX 6.8); property trees and paint chunks (GR 6.3); the environment snapshot (ST 6.10).

### 4.3 Izcal's principles that bind this design

P1 (hostile content; renderers hold no ambient authority; cross-site data never co-located), P2 (conformance gates every optimization), P3 (CPU fallback identical, GR-D3), P4 (measured performance and energy), P6 (evidence); M-1; I-1 and I-2; B1 and DET-1; FP-D19 (limits identical on every device); D-S1 (no new fingerprinting bits).

### 4.4 Assumptions kept and broken

| Assumption | Keep or break | Reason |
|---|---|---|
| SVG elements are DOM elements styled by the cascade | **Keep** | The specification says so; a second tree model diverges ([HP 4.2][HP]) |
| SVG images need a document with scripting machinery switched off | **Break** | A scriptless document needs no collector and no realm; the host can lack them by construction, over the same column store with a typed write capability (SV-D1) |
| SVG geometry lives in CSS boxes, or in ad-hoc per-element state | **Break** | An explicit render tree computed in the flush is what graphics, hit testing and accessibility all need (SV-D3) |
| Resource clients are tracked by lists inside resources | **Break** | Reverse edges derived from the tree with generation-checked ids and a from-scratch differential (6.5) |
| SMIL needs its own animation engine | **Break** | Its effects lower to the Web Animations model, and its timing graph shares the animation spec's core (SV-D4, AN-D6) |
| SMIL can be dropped | **Break** | Use is rising, and with CSS it is the only animation in images (P2) |
| Limits are private, per-engine and undocumented | **Break** | One published, deterministic ledger (SV-D6) |
| Floating-point geometry is non-deterministic | **Break** | IEEE-754 basic operations are exact; the hazards (FMA, reassociation, libm) are removable (SV-D2) |
| `use` must allow only same-origin, and SVG images must not fetch | **Keep** | Specification and security record ([SS 4.1][SS]; [SR 4.5, 5.1–5.2][SR]) |
| A canvas must be tainted by SVG images with `foreignObject` | **Break, after proof** | The scriptless host has no user state; staged until SV-S3 proves it (SV-D10) |
| A top-level `.svg` file runs script | **Break for local files** | The phishing record; links, find and sibling files keep working; one action restores script (SV-D9) |
| Hit testing for SVG is its own geometry code | **Break** | Hit testing is the geometry API evaluated at the pointer (6.11) |

## 5. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Adobe SVG Viewer | A single vendor's plugin carried SVG for a decade, then ended ([HP 2.1][HP]) | A platform feature cannot depend on one external renderer | SVG is engine work, built and owned (M-1) |
| SVG Tiny 1.2 | A profile diverged from the main spec and was never implemented whole ([HP 1.4][HP]) | Profiles that fork the DOM die | One engine; the glyph and image cases are processing modes of it, not profiles |
| Firefox SVG text (2013) | Reusing HTML's text engine gave SVG text find, selection, bidi and decorations ([RG 1.15][RG]) | One text pipeline | LY's inline call and TX's shaping for SVG text; SVG positions on top (6.10) |
| Blink's SMIL deprecation | Proposed at 0.0403%, suspended in 2016, use now 2.9% ([AN 1.8–1.10, 6.1][AN]) | Declarative features with no script equivalent in images do not go away | SMIL supported, on the shared effect model (SV-D4, AN-D6) |
| CVE-2016-9079 | SMIL timing container list mutated by a callback while walked; exploited against Tor Browser ([SR 1.1–1.2][SR]) | Never let callbacks run inside an engine iteration | `hg-anim`'s arena with generational handles (AN G131); no script in any SVG algorithm (G126) |
| `use` and resource bugs in C++ engines | Instance trees and client lists outliving their targets, as inferred from the bug record ([SR 1.7][SR], Low) | Lifetimes must be structural; engine algorithms must not run author code | Generation-checked ids, rebuilt from the tree, with a differential; an instance builder that is not DOM's clone (6.5, 6.7) |
| librsvg | Caps of 1,000,000 elements and 500,000 references but "no built-in limits on the total amount of memory or CPU time"; CVE-2019-20446 (nested patterns) and issue #323 (nested `use`) show the attack ([SR 2.1–2.4][SR]) | Caps without a total budget do not bound cost | The weighted ledger (SV-D6) |
| librsvg CVE-2023-38633 | Two URL parsers disagreed and read local files ([SR 7.3][SR]) | Parse once | `hg-url` only (NW G31); one parser per SVG micro-grammar (G121) |
| librsvg's property model | Never matched the SVG/CSS specs ([HP 4.2][HP]) | A home-grown cascade diverges | SVG uses the one cascade (6.2) |
| DOMPurify 2020 bypasses | Namespace confusion on reparse ([SR 4.4][SR]) | One parser; test round trips | ST's parser with this spec's SVG corpus (6.3) |
| SVG filter timing attacks, 2013–2023, and filter clickjacking, 2025 | Each fix closed one primitive, not the class ([SR 3.8][SR]; [GO 4.4][GO]) | Structural rules beat per-primitive fixes | GR-D18 and G69 with SVG's taint flags (6.5, 6.13) |
| `data:` in `use` | Trusted Types and Sanitizer bypasses; removed from Chrome 120 and Firefox 122 ([SR 4.5][SR]; [US 4.17][US]) | References that insert content are script sinks | Same-origin `use` only; G126 for animated URLs |
| SVGPathSeg and `getPathData` | A costly live API removed; its replacement took nine years ([HP 3.5][HP]) | Replace before removing | `getPathData` implemented by an admission exception (SV-D13, SV-D14) |
| WebKit's layer-based SVG engine | Invalidation depended on DOM order until moved into the style system ([RG 6.2][RG]) | Invalidation must be explicit and order-independent | Style's mask categories plus the reference graph (6.4, 6.5) |
| SVG 2's text algorithm | Five defects in the published pseudo-code ([RG 1.5][RG]) | A specification is evidence, not code | Corrected algorithm, black-box checked, filed (6.10) |
| SVG phishing wave (2025) | Attachments open as pages ([SR 6.1–6.3][SR]) | What users think is an image should not run code, but must stay a usable document | SV-D9's scriptless interactive mode |
## 6. Design

### 6.0 Architecture at a glance

```
 RENDERER per renderer key (site, container; origin under FP-D8)
  parse:  hg-html-syntax (inline <svg>, foreign content, ST) ─┐        hg-xml-syntax (image/svg+xml, ST-D5)
                                                              v                 │
          DomSink (hg-html-driver) ──► hg-dom                       StaticSink ──► hg-static-doc
          Store<Traced>: scripted documents (inline SVG,            Store<Arena>: scriptless documents (SVG
          top-level SVG, SVG in iframe/object/embed)                images, resource documents, OT-SVG glyph
          agent = style scope; hg-svg-dom (SVG DOM, bindings)       documents, local SVG files); own style scope;
              │                                                     no hg-heap, realm, event loop or fetch (G120)
              └──────── both over hg-node-store (columns; writes only through StoreWriter<Barrier>) ────────┘
                             v   reads through hg-tree-traits (G1); animated attributes visible there (6.9)
  L2p   hg-svg-syntax (micro-grammars, G121)  hg-svg-model (tables, presentational mapping)
        hg-svg-geom (IEEE-strict f64 geometry, G122)  hg-svg-interp (path, points, list interpolation)
  L3    hg-svg-instances (use and template instance builder, not DOM's clone)  hg-html-render (DM; rendering-
        only HTML element behaviour shared with the static host)
  L4    hg-style (cascade incl. presentational hints, SVG UA sheet, per style scope)
        hg-anim (animation spec: effect model and SMIL timing, scriptless core; this spec supplies attributes, SV-D4)
        hg-svg-layout (viewports, user spaces, geometry, markers, text placement via LY's inline call, resource
          resolution, reference graph, bounding boxes, spatial index -> SvgRenderTree, class (a) data)
        hg-svg-image (scriptless driver: rasters per (document key, size, scale, time, environment), SV-D12)
        hg-svg-observe (IIP svg.*)
     │ SvgRenderTree records (6.4, 6.5)             │ SvgGeometry, SvgSemanticSource, SvgGeometryQuery (traits)
     v                                              v
  hg-svg-paint (GR) -> display items         hg-hit, hg-layout-query (LY), hg-semantic (AX), hg-svg-dom

 CONTENT GPU PROCESS: rasterizes inline SVG display items and same-origin SVG image display lists
 MEDIA PROCESS per renderer key (MD-D20): hg-static-doc + hg-svg-image + style, layout, text, svg-layout,
   hg-anim's scriptless core, hg-svg-paint, hg-raster (CPU), all built with the L2m profile -> pixels; no JS, no fetch
```

**One SVG engine, two kinds of host.** Every SVG document Izcal renders goes through the same crates: the same grammar parsers, the same cascade, the same SVG layout pass, the same render tree, the same painting. What differs is the **host**: a **scripted document** lives in `hg-dom`, with wrappers, an agent heap and an event loop; a **scriptless document** lives in `hg-static-doc`, which has none of these and cannot name them (G120). SVG 2's processing modes ([SVG 2 CR §2.2–2.3](https://www.w3.org/TR/SVG2/single-page.html#conform-features); [SVG Integration](https://www.w3.org/TR/2014/WD-svg-integration-20140417/); [SS 1.4–1.8][SS]) become a type: a document's mode is fixed when its host is created, and the secure modes are the scriptless host. This answers the question the media spec left open (Q-MD-18) and reconciliation 2's Q25, and it is SV-D1 (6.1).

**Ownership, end to end.**

| Piece | Owner | This spec's part |
|---|---|---|
| Parsing inline SVG (foreign content, case-fixing tables) and SVG documents (`hg-xml-syntax`) | Style spec (ST 6.2, ST-D5) | `StaticSink`; the static element indices of SVG names (FP 6.3, 6.4) |
| Node storage | JS engine (JS-D4), DOM (DM 6.1) | Proposes `hg-node-store`, the column store both hosts share, with typed write capabilities (amendment, 9.5) |
| Generic element machinery, bindings, `HTMLOrSVGElement`, SVG `<a>` and `<script>` behaviour; rendering-only HTML element behaviour | DOM (DM 6.12) | SVG element behaviours and the SVG DOM in `hg-svg-dom`; asks DM to factor `hg-html-render` (6.8) |
| Cascade, presentational hints, SVG UA sheet | Style (ST 6.9, 6.11, Q-ST-19) | The SVG presentation-attribute mapping and the geometry-property rules (`hg-svg-model`); per-style-scope tables (6.1) |
| Outer `<svg>` as a replaced element, `foreignObject` layout root, inline-layout library call, the forcing table | Layout (LY 6.5.10, 6.1) | Everything inside the outer `<svg>`: `hg-svg-layout` |
| Hit testing walk, CSSOM View, ResizeObserver | Layout (LY 6.14, 6.15) | `SvgGeometry` for elements without CSS boxes; SVG hit geometry |
| Painting SVG: display items, paint servers, markers, clip, mask, filters, isolation | Graphics (GR 6.16) | `SvgRenderTree`, the typed records it consumes (GR 6.28) |
| Effect model, SMIL timing, CSS animation of SVG properties, frame sampling, compositor animation | Animation spec (AN-D6) | SMIL's elements and attributes, `hg-svg-interp`, the overlay, and the attribute-effect requirements (SV-D4) |
| Opaque SVG image host process, re-raster IPC | Media (MD-D20, `hg-media-host`) | `hg-svg-image`, the scriptless engine it links, and the raster protocol (6.8) |
| OT-SVG colour glyphs | Fonts and text (TX-D7, TX 6.3) | The glyph profile of the scriptless host (6.8) |
| SVG-AAM tables, semantic tree | Accessibility (AX 6.2, G103) | `SvgSemanticSource`: inclusion inputs, names' sources, bounds, text (6.12) |
| OT-SVG glyph export, SVG in print | PDF (PD-D19, PD-D20) | Nothing beyond the render tree |

### 6.1 The scriptless document host (SV-D1)

**The question.** MD-D20 rasterizes opaque cross-site SVG images in the media process with "the scriptless SVG, XML, style, layout and text crates", and asks (MD-S10, Q-MD-18) whether the DOM can exist there "without a JS agent heap" ([MD 6.1][MD]). JS-D4 stores the DOM once, as node columns outside the cage that `hg-heap` traces as its **node space**, and its soundness argument rests on who may write them: the columns "are written only by `hg-dom`'s Rust code", link writes go through `hg-dom`'s API, which performs the SATB barrier, and every write is a mutation-log record (JS 6.2; DM 6.1). JS G6 lets only `hg-dom`, `hg-bindings`, `hg-script` and `hg-inspect-renderer` depend on `hg-heap`. A scriptless document still needs everything style, layout and paint read: tags, attribute sets with their presentational-hint blocks, links, rare data such as the instance trees of `<use>`, and text ([ST 6.11][ST]).

**What a scriptless document needs from a collector: nothing.** In the secure modes no script runs, no wrapper exists, no event listener exists, and no external traced slot (JS 6.2) is ever filled. Nodes are created by parsing and by the engine's own instance builder (6.7); instance trees are the only part of the tree that ever changes after parsing, when a referenced element's attributes change under SMIL (6.9). So a scriptless document is an **arena** with **regions**: the parsed tree is one region, freed when the document is dropped, and each `use` or template instance tree is its own region, dropped as a whole (with its slots' generations bumped) when the instance tree is rebuilt. Memory is bounded by the current instance trees, never by their history, so a long-running animated image cannot grow (6.7, C6 of the critique).

**The design.**

- **`hg-node-store`** (L3, no `hg-heap` dependency, `#![forbid(unsafe_code)]`) holds the column layout, the attribute-set references, the text arena, the rare-data side tables without traced fields, and the `hg-tree-traits` implementation, moved unchanged from `hg-dom`. It exposes reads to everyone and **writes only through a `StoreWriter<B: Barrier>` capability**. `B` is a sealed trait with two implementations: `hg-dom`'s `TracedBarrier` (SATB barrier, mutation-log record with the old value, `ScriptToken`-checked) and `hg-static-doc`'s `ArenaBarrier`. A store is created with its barrier type (`Store<Traced>` or `Store<Arena>`), so a scripted document's store cannot be written through the arena capability, and `hg-graph-lint` allows `StoreWriter` construction only in those two crates (G120). The text arena never frees a buffer itself: in `Store<Traced>` freeing stays in `hg-dom`'s release step, where external-string liveness is known (JS 6.2), and in `Store<Arena>` buffers are freed only with their region, where no external string can exist. JS-D4's decision, one copy outside the cage and written only through `hg-dom`'s barrier for scripted documents, is unchanged (amendment R1, 9.5).
- **`hg-dom`** keeps everything that touches the engine: the node-space adapter that lets `hg-heap` trace a `Store<Traced>`, the external traced slots, wrappers, listeners, observers and the two-phase edit machinery.
- **`hg-static-doc`** (L3) builds a `Store<Arena>` from `hg-xml-syntax` through a `StaticSink`, never links `hg-dom` (G120), and is **its own style scope**. Style's intern tables, the atom table, and the fragment store are per agent in the accepted specs ([ST 6.6a, 6.9][ST]; JS 6.13; [LY 6.13][LY]); this spec asks that they be **per style scope**, a trait `hg-style`, `hg-text` and layout already need in shape, implemented by an agent and by a static document (amendment R5, 9.5). A static document never shares tables with an agent, so one static document can serve several agents of one partition without a writer from another thread. A static document whose instance trees change keeps a small mutation log of its own for invalidation (6.4).
- **Rendering-only HTML.** HTML inside `foreignObject` in an SVG image needs element behaviour that builds rendering structure (form-control UA shadow trees, `<details>`, `<summary>`, `<select>`'s rendered state, `<img>` with `data:` sources, `<picture>` selection, `<progress>`, `<meter>`), which lives in `hg-html` (L4). This spec asks the DOM spec to factor it into **`hg-html-render`** (L3, script-free, over `StoreWriter`), used by `hg-html` and `hg-static-doc` alike, so the same markup renders the same inline and in an image (amendment R4, 9.5; SV-7). Elements that need a browsing context, a media pipeline or script (`iframe`, `object`, `embed`, `video`, `audio`, `canvas`, custom elements) render their fallback or nothing in secure modes, as SVG 2's rule that `foreignObject` content gets "equivalent restrictions" implies ([SS 1.6][SS]); each is recorded per element.
- **Loaders.** Style's sheet loading and the text stack's font loading reach the static host only through `ResourceLoader` traits, which the static host implements as "`data:` and same-document only" (G120; amendments R5 and R9).

Style, layout, text, SVG layout and paint read the DOM only through `hg-tree-traits` and style's read API (G1, LY G1, ST G1, AX G100), so with per-style-scope tables they run unchanged over either host.

| Option | What | Cost | Verdict |
|---|---|---|---|
| (a) **Shared column store with typed write capabilities; scriptless arena host with its own style scope** | `hg-node-store`; `hg-static-doc`; `hg-dom` wraps `Store<Traced>` with the node space | One crate move, a sealed barrier trait, per-scope tables in style and layout | **Recommended** |
| (b) A full DOM with an agent heap that never runs script | `hg-dom` and `hg-heap` in the media process; one agent heap per SVG image | FP-34 puts a further agent heap, with an empty document, at ≤ 768 KiB† charged, so 50 SVG icons could cost up to about 37.5 MiB†, against SV-51's 48 KiB† per static document; the media process would link L2e crates and a cage reservation (JS-D17), against its strictest tier | Rejected |
| (c) A separate SVG-only tree built straight from XML | No DOM, no cascade over the DOM | A second document model that must agree with the first on selectors, cascade, inheritance and `<use>` styling; the divergence class G22 exists to prevent ([HP 4.2][HP]: librsvg's own property model "never ... matched the SVG/CSS specs") | Rejected |

**What the scriptless host is used for.** Every SVG document that runs in a secure mode, plus local files (SV-D9):

| Context | Mode ([SS 1.5–1.8][SS]) | Where | Timeline |
|---|---|---|---|
| `<img>`, `<picture>`, `<input type=image>`, `<video poster>`, CSS images (`background-image`, `border-image`, `list-style-image`, `content`, `mask-image` that is not a resource reference, `cursor`), canvas `drawImage`, `createImageBitmap`, `<image>` and `<feImage>` referencing an SVG file | Secure animated, or secure static where the context cannot animate (cursor) | Renderer if same-origin or CORS-approved; media process if opaque cross-site (MD-D20) | Child of the embedding document's timeline (6.9) |
| External resource documents (`<use href="file.svg#id">`, `fill`/`stroke`/`clip-path`/`mask`/`filter`/`marker` `url(file.svg#id)`, paint-server templates in other files) | **Secure static**, as SVG 2 says; SVG Integration's draft says secure animated and asks the question itself, so Izcal follows the Candidate Recommendation and records the choice ([SS 1.6, 1.8][SS]) | Renderer: `use` same-origin only; CSS references same-origin or CORS-approved in anonymous mode (SV-D11) | None |
| OT-SVG glyph documents | Secure static, with TX's glyph limits (TX 6.3) | Renderer | None (no animation, TX) |
| Favicons, notification images, Media Session artwork | Secure static | Media process of the key (MD 6.1) | Time 0 |
| Top-level SVG opened from a local file or the downloads list | Izcal's **scriptless interactive** mode (SV-D9, 6.13) | Renderer, opaque origin | Its own document timeline |
| Izcal's own UI icons (shell, DevTools, prompts), Izcal-authored and MIT | Secure static | Izcal-only renderers | None |

**Sharing.** One parsed static document serves every use of the same **(URL, fetch partition, request mode, credentials mode, response identity)** in one process: forty `<img src="icon.svg">` elements share one parse, one cascade and one SVG layout per distinct viewport, as FP §6.4 permits within a partition and never across one. A no-cors and a CORS-mode load of one URL are different documents, because servers may vary on `Origin` and because MD-D20 places them differently (G56; SV-52). What differs per use (concrete size, device scale, animation time, the environment snapshot's preferences) is a key of the raster cache (6.8).

**What is measured before it is committed (SV-S1, joined with MD-S10).** Start-up time of a static document, dirty bytes per document (including its style-scope tables) and per element against FP-38 and MD-53, and the latency of a re-raster at a new size and of each animation frame. MD-D20's reversal trigger stands; SV-D1 has its own: if the store move costs the DOM more than 2%† on the DOM benchmarks, the columns stay in `hg-dom` and `hg-static-doc` uses a shared column-type crate behind the same `hg-tree-traits`, checked by a differential (G22). A reversal of JS-D4 to "node in the cage" would also reverse SV-D1, and is recorded in R1.

### 6.2 The document model

**Elements and attributes as data (`hg-svg-model`, L2p).** SVG 2 defines 63 elements in its element index and its attributes in an attribute index, each with the content model, the attribute's grammar, its animatability, whether it is a presentation attribute and for which elements, and its DOM reflection ([SS 2.1–2.3][SS]). Izcal writes these facts once, in a description language of its own (`.hgsvg`), each row anchored to its spec section (G9), and `hg-svggen` generates: the static name indices FP 6.3 and 6.4 place in the engine image; the presentation-attribute table; the reference-attribute table (every attribute or property that holds a URL or IRI reference, with its valid target types, [SS 4.9][SS]); the animatable-attribute table with its security class (6.13); the element categories; and the reflection descriptors `hg-svg-dom` consumes. `hg-svg-model` holds tables only; the instance builder that uses them is `hg-svg-instances` (L3, 6.7). The description is Izcal's MIT text; the specification it restates is W3C standards data under HGBA-2's exception, as the SVG UA sheet already is (Q-ST-19).

**Presentation attributes into the cascade.** SVG 2 says presentation attributes "contribute to the author level of the cascade, followed by all other author-level style sheets, and have specificity 0", and parses the value with the property's grammar ([SVG 2 CR §6.6](https://www.w3.org/TR/SVG2/single-page.html#styling-PresentationAttributes); [SS 2.2][SS]). That is exactly the presentational-hint block style already computes once per interned attribute set ([ST 6.9, 6.11][ST]). `hg-svg-model` supplies the mapping `hints(element kind, attribute set) → declarations`, parsing each value with `hg-css-props`' parser for that property, plus the SVG-specific allowances (unitless lengths in geometry attributes; the `transform` attribute's own list grammar, parsed by `hg-svg-syntax`, G121). The **geometry properties** (`cx`, `cy`, `r`, `rx`, `ry`, `x`, `y`, `width`, `height`, `d`, and the new `path-length`) are presentation attributes only on the elements SVG 2 lists for each, and `x` on `text` or `r` on `radialGradient` are not ([SS 2.3–2.4][SS]); the table says which. `d` is `none | <string>`, parsed by `hg-svg-syntax`'s path grammar. The `style` attribute, `class` and `id` are ordinary DOM machinery.

**One parser per micro-grammar (`hg-svg-syntax`, L2p, G121).** Path data, transform lists, lengths and length lists, number lists, `viewBox`, `preserveAspectRatio`, `points`, angles, clock values and the SMIL begin/end value lists, `keyTimes`, `keySplines`, `keyPoints`, and the IRI forms of SVG are each parsed by exactly one function. Clock values parse exactly into the shared **time type**, an integer count of a fixed time quantum in `hg-types` (L2), owned by the animation spec (amendment R10), so `hg-svg-syntax` depends on nothing above L2. The SVG DOM's setters, the presentational-hint mapping, the CSS `d` and `transform` properties where they share a grammar, SMIL's value interpolation and Izcal's sanitizer all call it, and a differential checks every other caller against it (G22). Error handling is the specification's: path data renders up to the command containing the first error, and a command with too few parameters still renders its complete segments ([RG 2.1][RG]); a librsvg release line showed what a hand-rolled parser misreading compact arc flags costs ([HP 4.3][HP]).

**The SVG DOM (`hg-svg-dom`, L3, scripted documents only).** SVG 2's 60 interfaces and 5 mixins ([SS 2.7][SS]: `SVGElement`, `SVGGraphicsElement`, `SVGGeometryElement`, the element interfaces, the `SVGAnimated*` family, `SVGLength`, `SVGAngle`, `SVGNumber`, the list interfaces, `SVGTransform`, `SVGPreserveAspectRatio`, `SVGUseElementShadowRoot`, `ShadowAnimation`, and from SVG Animations `SVGAnimationElement` and the timing methods on `SVGSVGElement`) are generated by the DOM spec's `hg-webidl-codegen` from the pinned IDL, with reflection descriptors from `hg-svg-model`. Live-object rules are kept exactly, because scripts depend on identity: reflected objects are created on first access and synchronize "immediately after the value changed"; list items keep identity by index, surplus items detach, and `initialize`, `insert` and `replace` copy an object attached elsewhere ([SVG 2 CR §4.3.4–4.3.6](https://www.w3.org/TR/SVG2/single-page.html#types-ReflectingAttributes); [SS 2.11–2.12][SS]). Setting `baseVal` writes the attribute through `hg-dom`'s two-phase edit (DM 6.1, G91) and a mutation-log record, so style, layout and accessibility see one change. These objects live in rare-data side tables keyed by node slot, created on first access, so an element nobody scripts pays nothing (FP-11). Geometry methods and SMIL controls reach L4 through two L3 traits, **`SvgGeometryQuery`** (implemented by `hg-svg-layout`) and **`SmilControl`** (implemented by the animation spec's `hg-anim`), wired in `hg-renderer-main`, as `hg-cssom` reaches `hg-style` through `ComputedStyleSource` ([ST 6.15][ST]), so no crate depends upward.

**`animVal`.** SVG 2 makes it "an alias of baseVal" ([SVG 2 CR §4.6](https://www.w3.org/TR/SVG2/single-page.html#types-DOMInterfacesForReflectingSVGAttributes); [SS 2.10][SS]), while SVG 1.1 exposed the animated value. Izcal implements the alias; because the animated value lives in one place (the overlay, 6.9), exposing it instead is a reflection change, not a new store. Whether WPT `svg/animations` and the corpus still depend on animated `animVal` is measured black-box at SV-M0 (Q-SV-6) and decides SV-D14's last clause.

**Legacy and removed members (SV-D14).** Izcal implements SVG 2's DOM plus the members the web still reaches: `SVG1DOMUriReference` and `ObsoleteSVG1DOMFilter` counters are at 1.91% and 1.26% of page loads ([US 2.17][US]), so the SVG 1.1 members those counters cover are implemented where SVG 2 kept them or a second engine ships them. `SVGPathSeg` is not implemented (removed from Chrome 48 and Firefox 59, use counter about 0, [HP 3.5][HP]; [US 2.10][US]); the SVG Paths module's `getPathData`, `setPathData` and `getPathSegmentAtLength` are implemented, because they replace it, Firefox ships them, Blink announced an intent to prototype on 5 June 2026 and they are on the open Interop 2027 proposal ([HP 3.5][HP]; [US 4.11, 5.7][US]); this is an admission exception the owner is asked to record (SV-D13). `SVGElementInstance`, `instanceRoot` and `getTransformToElement` stay removed ([HP 3.6][HP]).

**`<a>`, `<script>`, `<style>` and event handlers** are the DOM spec's code, shared with HTML (DM 6.12). SVG `<script>` runs only in scripted documents; the scriptless host has no script element behaviour at all (G120).

### 6.3 SVG in HTML and XML parsing

Inline SVG reaches the DOM through ST's one tree builder: the foreign-content rules switch to the SVG namespace, apply HTML's 37 SVG tag-name and 58 attribute-name case adjustments, treat `foreignObject`, `desc` and `title` as HTML integration points, and break out on the listed HTML elements ([HTML parsing, foreign content](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign); [SS 7.1–7.2][SS]). The living standard now also creates processing-instruction tokens in foreign content ([SS 7.3][SS]); ST's pinned revision decides when Izcal follows. This spec adds nothing to the algorithm; it supplies `hg-svg-model`'s static name indices, and its differential corpus of namespace round trips (serialize and reparse inline SVG and MathML, compare trees) joins ST's sanitizer differential, because namespace confusion is how DOMPurify was bypassed in 2020 ([CVE-2020-26870](https://www.cve.org/CVERecord?id=CVE-2020-26870); [SR 4.4][SR]; ST G22).

SVG documents (`image/svg+xml`) are parsed by `hg-xml-syntax` with ST-D20's entity budget (1,048,576 characters, 64 nesting levels) and ST's 5,000† element-depth cap ([ST 6.2, ST-D20][ST]); a well-formedness error yields the `parsererror` document for scripted documents and a broken image for scriptless ones.

**SVGZ (SV-D8).** SVG 2 requires viewers to support gzip and deflate streams, and servers to send `Content-Encoding: gzip` for stored `.svgz` files ([SS 7.5][SS]); such bodies are decoded by the networking spec's partition service like any content coding, under its per-destination decompression budget (NW-D6). No specification defines what to do when an `image/svg+xml` body, or a local `.svgz` file, arrives gzip-compressed without the header, as misconfigured servers and editors produce. Izcal decodes such a body when its first bytes are the gzip magic number, in the partition service with the same budget, and treats a local `.svgz` the same way. The budget's values are the networking spec's; until it states them, this spec sets a ratio of 1:1,024† and a 64 MiB† absolute output cap for this case and asks NW to adopt or replace them (amendment R13). The rule is recorded as a deviation with a test (SV-17).

### 6.4 SVG layout and the render tree (SV-D3)

**Where SVG geometry is computed.** Elements inside an outer `<svg>` generate no CSS boxes (except `foreignObject`'s content, which LY lays out as a layout root); SVG's own geometry model applies ([SS 3][SS]). `hg-svg-layout` runs inside layout's flush, once per **SVG root**: an outer `<svg>` in an HTML document, the root of an SVG document, or the root of a scriptless document. It reads the tree through `hg-tree-traits` and computed values through style's read API, like every other layout crate (G1), and writes only its own store.

| Option | What | Verdict |
|---|---|---|
| (a) **A separate SVG layout pass producing an SVG render tree** | `hg-svg-layout` → `SvgRenderTree` (typed records), consumed by `hg-svg-paint`, `hg-hit`, `hg-layout-query`, `hg-semantic` | **Recommended**: SVG's model (user spaces, viewports, no flow, resources by reference) is not CSS's; GR 6.28 already asks for typed records |
| (b) SVG elements as CSS boxes in the box tree | Every SVG element a box with a special formatting context | Rejected: CSS box properties do not apply to SVG graphics elements ([GO 1.3][GO]), and the box tree's 32 B per box would buy nothing |

**Nesting with CSS layout.** `foreignObject` makes SVG layout and CSS layout mutually nested to any depth (an inline `<svg>` inside a `foreignObject` inside an SVG). SVG roots and `foreignObject` layout roots form **one work list ordered by tree position**: each SVG root's pass runs as a layout task after LY has sized it, writes a task-local `SvgRenderTree` merged deterministically like fragments (LY G43), and enqueues its `foreignObject` roots for LY; no step recurses, and nesting depth is charged to the ledger (6.13).

**The pass, in order** (each step anchored to its SVG 2 section, G9):

1. **Viewports and user spaces.** For the root and every nested `<svg>`, and every `<symbol>` instance, compute the viewport rectangle, the `viewBox` transform with `preserveAspectRatio` by SVG 2's explicit algorithm (scale e/vb per axis, meet or slice, alignment offsets; [RG 3.1][RG]), and the clip that `overflow` implies. The root's viewport comes from LY's replaced-element sizing (LY 6.5.10) or from the image host's concrete object size. Fragment identifiers (`#svgView(...)`, `<view>` elements) change the root's `viewBox` and `preserveAspectRatio` as SVG 2 specifies ([SS 3.5][SS]).
2. **Lengths.** Percentages resolve against the nearest viewport (or its `viewBox`): width, height, or the normalized diagonal √(w² + h²)/√2 for lengths in neither axis ([SS 3.1][SS]); font-relative units use the element's computed font through `hg-text`'s metrics; absolute units use CSS's fixed ratios.
3. **Transforms.** The element's `transform` (property, or the presentation attribute through the cascade; `patternTransform` and `gradientTransform` map to it too), `transform-origin` (UA sheet `0 0`) and `transform-box`, composed with the viewport transform and any `<use>` translation, give each element's local-to-parent matrix ([SS 2.3, 2.5, 3.3][SS]). `vector-effect: non-scaling-stroke` records the host coordinate system its stroke is computed in ([RG 2.11][RG]).
4. **Geometry.** Each shape becomes a path in user space: `rect` with rounded corners, `circle`, `ellipse`, `line`, `polyline`, `polygon`, `path` from the `d` property; `pathLength` and `path-length` record the author's length for dash and text-on-path scaling ([SS 2.4][SS]; [RG 2.3][RG]).
5. **`<use>` instances** (6.7) are expanded before their content is laid out, under the ledger (6.13).
6. **Text** (6.10) is placed by LY's inline-layout call and SVG's per-character positioning.
7. **Markers.** For every vertex with a `marker-start`, `marker-mid` or `marker-end`, compute the marker instance: position, orientation (`auto` bisects the incoming and outgoing directions, which "move past" zero-length segments; `auto-start-reverse` adds 180° at the start only), the scale from `markerUnits`, the marker's `viewBox` transform and its `overflow: hidden` clip ([RG 2.9][RG]).
8. **Resource resolution.** Every reference property (`fill`, `stroke` paint servers, `clip-path`, `mask`, `filter`, `marker-*`) and every `href` chain (gradient and pattern templates) is resolved through the **reference graph** (6.5) to a render-tree id, or to the property's specified result for an invalid reference ([RG 4.4][RG]). Units that depend on the referencing element (`objectBoundingBox`) are resolved here, against the element's fill bounding box, into explicit user-space transforms.
9. **Bounding boxes.** Each element's fill box is computed from its exact geometry: control points not on the curve do not contribute, curve extrema do; dashing, opacity and fill have no effect; non-rendered elements answer `getBBox` but do not contribute to ancestors ([RG 2.7][RG]). Stroke, marker and clipped boxes are computed on demand for `getBBox` options and paint's ink overflow.
10. **Hit-test index.** For an SVG root with more than 256† records, a spatial index of record bounds (6.11).

**The render tree (`SvgRenderTree`).** One store per SVG root, class (a) derived data (FP 6.4): dropping it and rebuilding gives bit-identical records (SV-32). Each record carries: the element's node slot (or instance node, 6.7), its kind, **one CTM composed in f64 relative to its nearest property-tree node** (6.6), geometry (the content hash of its parsed path, 6.16), stroke parameters with `vector-effect`, resolved fill and stroke (colour, `context-fill`/`context-stroke` resolved per marker or instance, or a paint-server instance id), `paint-order`, marker instances, clip, mask and filter instance ids, the isolation inputs (`opacity`, `mix-blend-mode`, `isolation`, filters, masks, 3D transforms), a **cross-origin taint flag** for records whose pixels come from an opaque image (6.5), `visibility`, `display`, `pointer-events`, its bounding boxes, which invalid-reference rule applied (if any), and the spec anchor of each derived field. The records are the contract GR 6.28 asked for; `hg-svg-paint` reads nothing else (G127).

**Property-tree nodes.** SVG creates property-tree nodes only for the outer `<svg>` and for elements that are compositor-mutable, isolated, 3D or scroll-related; every other element's transform is folded into its record's f64 CTM, converted once at the boundary (6.6). Nested `viewBox` scales therefore never compose in Q32.

**Invalidation.** `hg-svg-layout` reads the style scope's mutation log, style's per-element "affects" mask and the animated-attribute overlay's list of changed values (6.9). This spec asks style for two more mask categories, **svg-geometry** (geometry properties, `transform`, `vector-effect`, marker properties) and **svg-resource** (properties of paint servers, clip paths, masks, filters, markers and patterns, and anything inside them), so a change inside a gradient invalidates exactly the records that reference it (amendment R5). WebKit's history is the warning: its legacy SVG invalidation ran "through layout", so "the invalidation chain depended on the DOM order", until style decided ([RG 6.2][RG]). Every incremental result has a CI differential against a from-scratch pass (G127, SV-32).

### 6.5 The reference graph and the hand-off to graphics

**The graph.** Forward edges run from a referencing element to the resource it names; reverse edges from a resource to its clients. Both are **derived from the tree** in the layout pass, stored as node-slot pairs with generations (DM 6.1), never as owning pointers, and rebuilt for a subtree whenever the mutation log touches an `id`, a reference attribute or property, or a resource's subtree. Engines' SVG bug history includes resource client lists that outlive a client and instance trees that outlive their source, a pattern the security note infers from the record rather than reads in a published root-cause study ([SR 1.7][SR], Low); in Izcal a stale entry is caught by its generation and the from-scratch differential (G127). Every walk over the graph carries a visited set and the specification's rule that a reference cycle is an error, uses an explicit stack, and is charged to the ledger (G123, 6.13).

**What graphics receives.** `hg-svg-paint` (GR 6.16) receives, per SVG root, the render-tree records in paint order (document order within the SVG root; SVG has no `z-index`, which SVG 2 deferred, [HP 1.7][HP]) and these **resolved resource instances**:

| Resource | Resolved by this spec | Graphics does |
|---|---|---|
| Linear and radial gradients | Stops after `href` template inheritance, offsets clamped and ordered, colours as computed values, `gradientUnits` folded into a user-space transform, `spreadMethod`, focal point and `fr` (focal circle outside the end circle forms a cone, [RG 4.1][RG]) | Builds the ramp through `hg-colour` and rasterizes (GR 6.4) |
| Patterns | Tile rectangle in user space, the content transform (`patternUnits`, `patternContentUnits`, `viewBox`), the render subtree id of its content, nesting depth | Renders the tile once per distinct device scale and tiles it |
| Clip paths | Member records (shapes, text, `use` that directly references a shape or text) with `clip-rule`, `clipPathUnits` folded, the clip path's own `clip-path` ([RG 4.3][RG]) | Builds the coverage (clipping does not isolate, GR 6.16) |
| Masks | Mask region, `maskUnits` and `maskContentUnits` folded, `mask-type`, `color-interpolation`, content subtree id | Luminance or alpha mask in the integer pipeline, with the coefficients graphics chooses (Q-SV-5) |
| Filters | Filter region; the primitive graph with each primitive's resolved inputs, subregion, parameters parsed by `hg-svg-syntax`, `color-interpolation-filters` (initial linearRGB); `feImage` as an image reference or a render subtree id | Integer kernels (GR 6.4) under GR-D18 |
| Markers | Per instance: transform, clip, content subtree id, resolved `context-fill`/`context-stroke` | Paints the content in that transform |

**Cross-origin pixels in SVG effects.** GR-D18 and G69 say a cross-origin surface is never an input to a page-controlled effect. SVG has its own ways to put an opaque image under an effect: an `<image>` of an opaque cross-origin image inside a filtered, masked or blended group, `feImage` of such an image, and a CSS image inside `foreignObject`. Each such record carries the taint flag, and `hg-svg-paint` substitutes transparent black for its pixels in every effect input while compositing the image itself without the effect, as GR-D18 does for iframes. This is a web-visible deviation (other engines apply the filter), recorded with a test (amendment R7; SV-16).

**The same resolver serves HTML.** `clip-path: url(#c)`, `mask: url(#m)`, `filter: url(#f)` and `mask-image: url(#m)` on HTML elements name the same SVG resources; `hg-paint` asks the same resolver for the same instances, so an SVG filter applied to a `<div>` and to a `<rect>` is one implementation (amendment R7). `filter: url()` is on 6.19% of page loads ([chromestatus 1014](https://chromestatus.com/metrics/feature/timeline/popularity/1014); [US 2.8][US]).

**Hit testing and paint order.** `hg-paint-order`'s chunk-to-fragment map gains a chunk-to-**SVG-record** map, so `hg-hit` can descend from an outer `<svg>` fragment into SVG records (amendments R6, R7).

**The bounded path format.** GR's bounded path format stores coordinates as 24.8 fixed point in local units relative to the path's origin, at most 65,536 segments ([GR 6.3][GR]). SVG user units can be very small (`viewBox="0 0 1 1"`, GIS exports in degrees) or very large, so 1/256 of a user unit can be most of the picture. This spec asks for one addition: a **per-path binary exponent** e, with |e| ≤ 100†, so coordinates are stored as 24.8 values of the path's coordinates × 2^e. `hg-svg-geom` chooses e from the **combined** extent of the path and the scale of its record's CTM, so that both the coordinates and the matrix with 2^−e folded in keep at least 24† significant bits. When no e achieves both (extreme scale combined with extreme coordinates), the path is transformed to device space in f64 by `hg-svg-geom` and emitted with an identity matrix; when |e| would exceed 100, the element is in error and logged (amendment R7). A path longer than 65,536 segments is split into consecutive paths only where that preserves the winding result (separate subpaths); otherwise it renders up to the limit, logged as a recorded deviation.

### 6.6 Numbers and the geometry APIs (SV-D2)

**The constraint.** SVG geometry is defined in real numbers; SVG 2 requires coordinate conversion in "at least single-precision" and CTM generation in at least double precision ([RG 3.3][RG]); the DOM exposes geometry as `float` (`SVGLength.value`, `getTotalLength`) and `double` (`DOMRect`, `DOMMatrix`, `DOMPoint`) ([RG 3.4][RG]). Izcal's output must be bit-identical across runs, OSes and the three ISAs (DET-1), and pixels come from integer arithmetic (GR-D3). Layout and PDF chose fixed point for geometry (LY-D3, PD G116), and LY-D3's options table rejected floating-point geometry because cross-ISA identity would need a strict discipline everywhere. SVG's case differs: layout's range is known (CSS pixels, ±33.5 million px in 1/64 px), but SVG user units have **no bounded range**: one document can scale by 10⁵ in a `viewBox` and by 10⁻⁵ in a nested one ([RG 3][RG]). Fixed point would put the error where the user cannot see why; a strict binary64 discipline costs a lint, which the JS engine already pays for every JavaScript Number (JS 6.9, JS-D12).

**The choice: IEEE-754 binary64 under a strict discipline (G122).**

- **Only exact operations.** SVG crates may call on `f64` only `+`, `−`, `×`, `÷`, `sqrt`, `abs`, `copysign`, `floor`, `ceil`, `trunc`, `round_ties_even`, comparisons, `total_cmp`, and `hg-math`. IEEE-754 defines the basic operations and `sqrt` bit-exactly, and x86-64 (SSE2), AArch64 and RV64D implement them exactly. `min`, `max`, `clamp`, `%`, `rem_euclid`, `powi`, `powf` and every other method are rejected by `clippy::disallowed_methods`: Rust documents several of them as having non-deterministic precision ([Rust `f64::powi`](https://doc.rust-lang.org/std/primitive.f64.html#method.powi)), and `min`/`max` may return either zero for `+0.0` and `−0.0` ([Rust `f64::min`](https://doc.rust-lang.org/std/primitive.f64.html#method.min)). Bounds use `hg-svg-geom::fmin`/`fmax`, which return `−0.0` for mixed zeros on every ISA.
- **Fixed order.** No reassociation and no fused multiply-add: Rust does not contract floating-point expressions, and the lint rejects `mul_add` and every FMA intrinsic.
- **Transcendentals only through `hg-math`**, the JS engine's one deterministic implementation (JS 6.9): `sin`, `cos`, `tan` for `rotate()` and `skewX()`, `atan2` for marker and `animateMotion` orientation, the arc conversion's angles. This spec asks `hg-math` for **degree-native** `sin_deg`, `cos_deg` and `tan_deg` that are exact at multiples of 90°, because SVG angles are degrees and `rotate(90)` through radians gives cos(π/2) ≈ 6.1 × 10⁻¹⁷, which `getCTM` would expose; the three engines' answers are measured black-box first (Q-SV-17; amendment R3).
- **Special values.** NaN is canonicalized wherever a value becomes observable (JS-D12's rule). Decimal numbers are parsed by a correctly rounded decimal-to-binary conversion, which is unique.
- **The input is binary64 too.** Every geometry property (`cx`, `r`, `x`, `width`, `transform`, `stroke-width`, `stroke-dasharray`) reaches SVG through the cascade, whose computed-length type is "an IEEE binary type fixed at ST-M0" ([ST 6.9][ST]); this spec asks that it be **binary64**, because a binary32 choice would lose SV-D2's precision before SVG sees a value (amendment R5).

**Where floating point stops.** At one named boundary function shared with layout's rule (`hg-svg-geom::to_display`, built on `hg-layout-num`'s rounding): paths into the bounded path format with their exponent (6.5), each record's CTM into GR's i64 Q32 matrix, glyph positions into the text stack's quantum; round half to even; **NaN** anywhere in an element's geometry or CTM puts the element in error (not rendered, logged); **±∞ and out-of-range** values saturate and are logged. Everything rasterized after that is integer (GR-D3).

| Option | Range and precision | Determinism | Verdict |
|---|---|---|---|
| (a) **IEEE-strict f64, integer at the graphics boundary** | ±1.8×10³⁰⁸, 53-bit relative precision; matches the DOM's `double` | By IEEE-754 for the allowed operations; `hg-math` for the rest; lints and a cross-ISA identity test (SV-30) | **Recommended** |
| (b) Fixed point throughout (i64 Q32.32, matrices in i128), as LY and PD | ±2.1×10⁹ with 2⁻³² absolute precision: a scale of 10⁻⁵ keeps about 15 significant bits, and coordinates above 2³¹ saturate | Exact | Rejected for user space: SVG's range is unbounded |
| (c) f32, as SVG's IDL `float` | 24-bit precision | Same discipline as (a) | Rejected: below SVG 2's double-precision CTM requirement; IDL `float` is applied only where a value is returned through a `float` member |

Reversal trigger for (a): a cross-ISA mismatch in SV-30 traced to compiler or platform floating-point behaviour that no lint or compiler flag can pin; then (b) with a block exponent per user space.

**The geometry APIs.** `getBBox(options)` with `fill`, `stroke`, `markers` and `clipped` ([SVG 2 CR §4.4](https://www.w3.org/TR/SVG2/single-page.html#types-__svg__SVGGraphicsElement__getBBox); Firefox alone ships the options, [US 4.10][US]), `getCTM`, `getScreenCTM`, `isPointInFill`, `isPointInStroke`, `getTotalLength`, `getPointAtLength`, `getComputedTextLength`, `getSubStringLength`, `getStartPositionOfChar`, `getEndPositionOfChar`, `getExtentOfChar`, `getRotationOfChar`, `getCharNumAtPosition` and `getPathSegmentAtLength` are answered by `hg-svg-geom` over the render tree after the flush the forcing table requires (LY 6.1, `hg-layout-query`'s table; amendment R6 adds the SVG entries).

- **Arc length** is computed by adaptive Gauss–Legendre quadrature per segment, a standard method of numerical analysis ([Gander and Gautschi, "Adaptive Quadrature—Revisited", BIT 40, 2000](https://doi.org/10.1023/A:1022318402393)), with a tolerance relative to the segment's **control-polygon length** (2⁻³⁰†), so a closed loop whose chord is zero still has a positive tolerance, and a maximum subdivision depth of 24† per segment. Elliptical arcs are converted to centre form by SVG 2's implementation notes (B.2.4, radii scaled up by √Λ when too small, [RG 2.2][RG]). One arc-length function serves `getTotalLength`, `getPointAtLength`, dashing positions handed to graphics, `textPath` and `animateMotion`, so every consumer agrees on where 50% of a path is ([RG 2.3–2.5][RG]: engines differ by about 0.1–0.9% today).
- **`isPointInFill` and `isPointInStroke`** are defined in the element's user space; points on the path count as inside, open subpaths close implicitly for the fill test, and the stroke test honours width, caps, joins, miter limit, dashes and `vector-effect` ([SS 2.9][SS]; [RG 2.4][RG]). The fill test is an exact winding-number test against the curves (root finding on each Bézier in the allowed arithmetic); the stroke test uses `hg-path`'s stroker on the path in user space with a tolerance of max(2⁻¹⁶† × stroke width, 2⁻⁴⁰† × path extent), so a vanishingly thin stroke cannot demand unbounded subdivision. **Hit testing uses these same functions** (6.11).
- **Work from script is bounded.** Geometry work called from script is charged to a per-call canonical work budget; on exhaustion the result is returned at the precision reached and an IIP event logged (6.17).
- **Geometry reaching CSSOM View and ResizeObserver** (an SVG element's `getBoundingClientRect`, `getClientRects`, ResizeObserver on an SVG element) comes through the `SvgGeometry` trait LY 6.5.10 asked for, which returns rectangles mapped through the property trees and converted by `hg-layout-num`'s one rounding function into `LayoutUnit` (LY-D3).

**Interop where engines disagree.** `getBBox` of `<use>`, `<textPath>`, `<tspan>` and empty containers differs between engines (Chrome and Safari fail subtests in six of ten `getBBox` files, [RG 2.8][RG]); Izcal follows SVG 2's definition, records each disagreement against the three engines as an interoperability observation, and files a specification issue where the text is unclear, not where an engine differs (M-1 part 2).

### 6.7 `<use>`, paint-server templates and instance trees (SV-D5)

**What the specification requires.** A `use` element gets a shadow tree of element instances cloned from the referenced element, created even when the `use` is not rendered; scripts inside it are inert; mutations of the referenced subtree must be reflected ([SVG 2 CR §5.5](https://www.w3.org/TR/SVG2/single-page.html#struct-UseShadowTree); [SS 4.2][SS]). Instances inherit from the `use` element; the outer document's rules do not match inside, but the referencing document's style sheets are cloned into the shadow scope, and `:hover` matches per instance ([SS 4.4][SS]). Event listeners on source elements are mirrored on every instance and events are retargeted to the `use` element when they leave the shadow tree; SMIL animations are mirrored as instances, Web Animations through read-only `ShadowAnimation` objects ([SS 4.5][SS]). Paint-server templates without own children clone their template's children into a shadow tree "like `use`" ([SS 5.2][SS]). Referencing an ancestor is an invalid circular reference; an indirect cycle makes the offending element "in error and must not be rendered" ([SS 4.3][SS]). The specification sets **no limit** on depth or instance count ([SS 4.7][SS]); nested `<use>` grows exponentially, the SVG form of "billion laughs" ([librsvg issue #323](https://gitlab.gnome.org/GNOME/librsvg/-/issues/323); [SR 2.1][SR]).

**The shadow root's mode is moving.** The CR of 6 October 2026 says the tree is "open (inspectable by script), but read-only" and that modifying it throws `NoModificationAllowedError` ([SS 4.2][SS]); the Editor's Draft of 13 October 2026 says "The shadow tree is closed (not inspectable by script)", and its changes list records "Made the use-element shadow tree closed instead of open and read-only" (fact-check FC-M1). Izcal's design makes the mode **one flag on the read-only user-agent shadow root kind**, set from the pinned snapshot; which mode ships at SV-M1 is decided by the snapshot pinned then and by black-box observation of the three engines (Q-SV-18).

**The design.** Instances are **materialized nodes**: real elements in the document's store, in a read-only user-agent shadow root, built by **`hg-svg-instances`** (L3, generic over `StoreWriter`) for both hosts. Materializing is what the specification describes, and style, layout, the semantic tree, hit testing and event dispatch all need a node per instance anyway, because each instance can have its own inherited style, its own `:hover` state and its own accessible object ([SS 4.4, 8.5][SS]).

- **The builder is not DOM's clone algorithm.** DOM's clone runs cloning steps and "create an element", which would run custom-element constructors, create navigables for `<iframe>` or start media pipelines inside the style flush, exactly the re-entrancy DM 6.2 and G91 forbid. The builder copies the tag, the attribute-set reference (shared, copy-on-write), text references and the inline-style reference into new slots, sets an "element instance" flag and records the source (slot and generation) and the `use` host. It runs **no** cloning steps, custom-element creation or upgrade, insertion steps or post-connection steps; instance elements never upgrade, never create navigables, media pipelines or canvas contexts; `<script>` instances are inert; instance building produces no `MutationObserver` records and no `slotchange` (amendment R4).
- **Building runs in the flush, never in script.** The builder runs in the core phase of a style flush when the mutation log shows a change to a `use`'s `href` (or its animated value, 6.9), to the referenced subtree or to an `id` the reference resolves through; it is script-free (DM G91). In scripted documents, a read of a `use` shadow root (tree accessors, `querySelector`, `SVGUseElementShadowRoot` getters) runs the pending builds for that root only, as slot assignment is resolved on read (DM 6.6), not a style flush (forcing-table entry, amendment R6).
- **Regions.** Each instance tree is its own store region; a rebuild drops the region and bumps its slots' generations, so memory is bounded by current instance trees in both hosts (6.1). Incremental rebuild re-clones only the changed sub-subtree using the mutation log's old values; a full rebuild remains the from-scratch reference (G127).
- **Identity and events.** `correspondingElement` and `correspondingUseElement` read the recorded source and host. Instances hold **no listener list**: dispatch on an instance computes the event path as DOM does and, at each instance in the path, invokes the **source element's** current listeners with `currentTarget` the instance, re-read per invocation by DM 6.5's rule, so adding or removing a listener on the source takes effect at once. Retargeting to the `use` element outside the shadow tree is DOM's (DM 6.5; [RG 7.3][RG]).
- **Cycles.** The builder walks with an explicit stack and a visited set over (source, host chain); a cycle marks the offending `use` in error, as the specification says.
- **The budget (6.13).** Instance nodes, template clones and nested `use` levels are charged to the document's ledger, a **prefix sum over document order maintained incrementally** (an order-statistics structure keyed by tree position), so after every flush the charged set equals what a from-scratch build would charge (G124, SV-32). A `use` whose expansion would exceed the ledger is **in error and not rendered**, the specification's state for a circular reference. When a mutation elsewhere (say, a `use` inserted earlier in document order) makes a later `use` lose its budget, that `use` is put in error at the next flush, its instance root is emptied in the same core-phase step, and a `svg.budget` event names the mutation that caused it: this is the only way an instance tree changes without a change to its own inputs, and DST checks it against a from-scratch build.

| Option | What | Verdict |
|---|---|---|
| (a) **Materialized instances by a dedicated builder, read-only shadow root, ledger in document order** | Real nodes; no DOM clone; regions; incremental prefix-sum ledger | **Recommended** |
| (b) Virtual instances: (source node, context) pairs that style and layout traverse without nodes | Saves nodes in deep icon sprites | Rejected: every consumer (style's per-node columns, the semantic tree's stable ids, hit testing, `:hover`, event retargeting) would need a second node identity; the saving is real only for the exponential attacks the ledger already stops |
| (c) Clone on demand at paint time | — | Rejected: instances are styled, hit-tested and exposed, so they must exist before paint |

**External `use`.** `use` may reference another document, including a whole document without a fragment ([SS 4.1][SS]; shipped in Chrome 137 and on the open Interop 2027 proposal, [US 4.16, 5.7][US]); SVG 2 "does not allow cross-origin and data URL resource requests in 'use'" ([SS 4.1][SS]), and Chrome 120 and Firefox 122 removed `data:` in `use` after it bypassed Trusted Types and the Sanitizer API ([Chrome blog](https://developer.chrome.com/blog/migrate-way-from-data-urls-in-svg-use/); [US 4.17][US]; [SR 4.5][SR]). Izcal allows **same-origin `use` only**, loads the target as a secure-static resource document in `hg-static-doc` (6.8), and clones from it into the referencing document's shadow tree. The `crossorigin` attribute on `use` that the WG once resolved to add was deferred past SVG 2 ([GO 6.3][GO]); it is not admitted.

### 6.8 SVG as an image, in CSS and as a resource document (SV-D10, SV-D11, SV-D12)

**Images (SV-D12).** An SVG image is a scriptless document (6.1) rendered by `hg-svg-image` into **pixels at its concrete object size in device pixels** (CSS Images' concrete object size × the device scale, rounded to whole device pixels by GR's snapping rule), and then drawn like any raster image. The natural size and ratio come from the root's `width`, `height` and `viewBox` per SVG Integration's sizing rules, including the 300 × 150 default when neither size nor `viewBox` is given ([SS 1.9][SS]); LY sizes the replaced box from them (LY 6.5.8). Rasterizing at the concrete size, rather than embedding the image's vectors in the page's display list, is what makes the two placements identical: a same-origin image is rasterized by the key's content GPU process (or by the renderer's CPU `hg-raster` when a canvas needs the pixels for origin-clean replay, GR 6.15), an opaque cross-site image by the media process's CPU `hg-raster`, and both produce the same bits because the raster is canonical (GR-D3; SV-31). A static document is complete, and rasterizes, only after every `data:` subresource it declares (images, fonts) has decoded or failed, so the first raster never depends on decode timing.

**The raster cache** is keyed by (static-document key, concrete size in device pixels, fragment identifier, animation sample time, the environment-snapshot values the image's style read: colour scheme, reduced motion, contrast, forced colours), so media queries inside an SVG image evaluate against the user's preferences exactly as HTML's do and a dark-mode image re-rasters when the preference changes. Entries are decoded-image bytes under FP-13 and MD's discard policy. A zoom, a DPR change or a resize asks for a new raster; SV-43 sets its latency.

**The opaque-image protocol.** For an opaque cross-site image the renderer knows only what any image tells it. It sends (size, scale, environment snapshot, sample time) for each rendering opportunity while the image is visible and, if the image is animated, at each frame; the media process answers with a raster or "unchanged". The renderer learns the natural size and whether the image is animated, as it does for a GIF; it never learns which environment values the image read or when its next change is due, because both would be cross-origin information. The media process keys its own cache. `svg.imageInfo` for an opaque image is served to DevTools from the media process through the broker, never through the renderer (6.17; amendments R8, R16). An off-screen or hidden animated image is never rasterized (SV-45).

**What an SVG image cannot do.** Load anything but `data:` URLs and same-document references (SVG 2 treats every other reference "as if a network error occurred", [SS 1.4][SS]); run script; react to input; read the embedding page's style, custom properties or `currentColor` (the theming gap developers complain about, [US 6.4][US]; "link parameters" proposals are not admitted until they meet the admission rule); use the page's web fonts ([US 6.3][US]). Fonts in images are the Izcal font set, the OS tier, and `data:` URL fonts declared inside the image. MD-D20 currently limits fonts in opaque images to "the Izcal font set and the OS tier only" ([MD 6.1][MD]); this spec asks the media spec to add `data:` fonts declared in the image, parsed by `hg-fontfmt` in the media process under TX's fuel, because otherwise the same image would render differently depending on whether it was same-origin, which SV-31 forbids (amendment R8; TX G26 does not list the media process and is unaffected, [RN2 Q25][RN2]).

**`foreignObject` in images and canvas taint (SV-D10).** `foreignObject` is on 8.24% of page loads and in 0.59% of SVG images ([chromestatus 325](https://chromestatus.com/metrics/feature/timeline/popularity/325); [US 2.5][US]), and HTML-to-image libraries draw DOM into a canvas through an SVG image with `foreignObject`. Engines taint a canvas when an HTTP-loaded SVG image with `foreignObject` is drawn; for `blob:` URLs Gecko did not taint and Chromium aligned with Gecko ([SR 5.3–5.4][SR]). The reason is that HTML in the image could render user-specific state; the list of such state below is inferred, not taken from a published study ([SR 5.5][SR], Low). The scriptless host has **no source of user state** by construction: no history service (every link is unvisited), no autofill, no spellchecker, no focus, caret or selection, no form state. What remains reaches the raster through the environment snapshot (system colours, `AccentColor`, forced colours, the visible OS-tier fonts under TX's visibility rule, the language list for quotes, hyphenation and `systemLanguage`, the user's font-size preference), and each of these is already exposed to script by D-S1 as amended by OD-6 or it is a new channel. Izcal therefore designs same-origin and `data:` SVG images with `foreignObject` to be **non-tainting**, and stages it for a security reason: until the proof suite SV-S3 enumerates every source of user-dependent rendering from the style, layout, text and graphics specs and shows each is either absent or already script-visible with its D-S1 status, such an image taints, as in today's engines. Cross-origin images taint regardless, as HTML requires. The canvas origin-clean rule is the graphics spec's (GR 6.15), which this changes (amendment R7).

**SVG in CSS.** Every CSS `<image>` that is an SVG file is an SVG image as above: `background-image`, `border-image-source`, `list-style-image`, `content`, `cursor` (secure static), `mask-image` when the URL does not name a `mask` element ([SS 4.9][SS]), `image-set()` and `image()`. A CSS `url()` that names an SVG **element** (`clip-path`, `mask`, `filter`, `marker-*`, `fill`, `stroke`) is a **resource reference**, resolved by the reference graph (6.5) whether the referencing element is SVG or HTML.

**Resource documents (SV-D11).** A resource reference to another document loads that document once per static-document key (6.1) as a secure-static scriptless document, immutable after load, in which a missing `id` is invalid ([SS 4.8][SS]). The fetch modes are the specification's: style properties and presentation attributes fetch in CORS **anonymous** mode, so a cross-origin resource is usable only when the server approves; CSS Masking's references use the same mode, and a network error means `none` ([SVG 2 CR §16.1.7](https://www.w3.org/TR/SVG2/single-page.html#linking-processingURL-fetch); [SS 4.8, 5.3][SS]); `use` is same-origin only (6.7); animation elements' `href` must be same-document ([SS 4.9][SS]). A resource document is in the renderer because it is same-origin or CORS-approved: no opaque bytes reach the renderer (G56). An invalid reference has each property's specified result: no clipping for `clip-path`, a transparent-black mask (the element disappears) for `mask`, the filter chain ignored for `filter`, no marker, the paint fallback or nothing for paint ([RG 4.4][RG]); the render tree records which rule applied (6.17).

### 6.9 SMIL (SV-D4)

**The evidence.** SMIL is not dying. Blink's intent to deprecate it, posted on 29 April 2015, cited 0.0403% of page views ([blink-dev](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/59rZqirUQNwJ); [HP 3.2][HP]); the counter is discontinuous in 2015–2017 and has risen steadily from 0.58% of page loads in January 2019 to **2.90%** in October 2026, with SMIL inside SVG images on **1.67%** ([chromestatus 501](https://chromestatus.com/metrics/feature/timeline/popularity/501); [chromestatus 768](https://chromestatus.com/metrics/feature/timeline/popularity/768); [AN 6.1–6.2][AN]). Google suspended the deprecation in August 2016 because "there are use cases serviced by SMIL that just don't have high-fidelity replacements yet" (forwarded to www-svg on 17 August 2016, [AN 1.10][AN]). Inside an SVG image, SMIL and CSS animations are the only ways to animate, because no script runs there ([AN 3.8][AN]). The standards do not settle it: SVG Animations Level 2 is an Editor's Draft that defers to SMIL Animation, and Web Animations only says it is "intended" that SVG's animation features be defined in its terms ([SVG Animations ED](https://svgwg.org/specs/animations/); [Web Animations 1 §1.3](https://www.w3.org/TR/web-animations-1/#relationship-to-other-specifications); [SS 6.1, 6.5][SS]). SMIL's timing code was where Firefox's exploited CVE-2016-9079 lived, a timing container's list mutated by a callback while it was walked, on CISA's Known Exploited Vulnerabilities list since 22 June 2023 ([CERT/CC VU#791496](https://kb.cert.org/vuls/id/791496); [SR 1.1–1.2][SR]).

**Ownership (SV-D4), aligned with the animation spec's AN-D6.** About two thirds of SMIL's features, and almost all of its use, lower onto the Web Animations effect model; what does not lower is SMIL's timing graph (begin and end lists, syncbase and event begins, `restart`, the fragment timeline, the sandwich priority) ([AN 7][AN]). Draft 0.1 of this spec proposed to own that timing graph in an `hg-smil` crate; the animation spec, written in parallel, decided instead that **it owns SMIL timing** in the scriptless core of `hg-anim`: the interval engine, the time container (`timelinebegin`, `playbackorder`), `restart`, `min`/`max`, fill, repeat, the dependency graph and its cycle rules, and the mapping of each active interval to an effect in its effect stack ([Izcal animation 6.8, AN-D6](Izcal%20animation.md)). Its reason holds: timing is the part that must share one model, one clock and one composite order with CSS and script animations. This spec adopts that split (draft 0.1's reversal trigger for SV-D4 was exactly this) and owns the rest:

- **The element and attribute layer**: the animation elements, `hg-svg-syntax`'s parsing of every timing and value attribute into the shared time type and typed values, which attributes are presentation attributes (CSS properties) and which are XML attributes, the SVG DOM's `baseVal` and `animVal` and `SVGAnimationElement`'s DOM (through the `SmilControl` trait, implemented by `hg-anim`), `animateMotion`'s path geometry, `rotate="auto"` tangents and `keyPoints` from `hg-svg-geom`, paced spacing precomputed from value distances, and the cloning of animation elements into `use` shadow trees (6.7).
- **One interpolation per value type.** Path data, `points`, number-list and transform-list interpolation live in **`hg-svg-interp`** (L2p, binary64 under G122), called by the effect stack through an `InterpolateValue` trait keyed by `hg-css-props`' value types, for both CSS `d` and `transform` and SMIL attribute effects. Compositor-eligible effects (`animateTransform` on an element with its own property-tree node, `opacity`) are converted to `hg-interp`'s integer form by GR's boundary function like CSS ones; `hg-interp`'s integer form never carries SVG user units.
- **Where animated attribute values go.** The animation spec writes effects on CSS properties into the Animation origin, as a class **after CSS animations and before script animations** (its recorded choice, which answers Q-SV-7 for Izcal; the engines' order is still measured), and says effects on XML attributes "write `animVal` only". This spec asks for one precision (amendment R10): attribute effects are written to an **animated-attribute overlay** (a side table keyed by node slot and attribute), and `hg-tree-traits`' attribute accessors return the animated value where one exists, so **every rendering consumer sees one value**: style (selector matching on an animated `class`, presentational hints), the reference graph (an animated `id` or reference), the instance builder (an animated `use` `href`), layout's natural-size rule (an animated root `viewBox`), text language (`lang`); the DOM's `getAttribute`, serialization and, under SV-D14, `animVal` do not. A change in the overlay is an invalidation input of the same kinds as an attribute mutation (amendment R5). No animation writes the DOM, and no SVG crate runs script while values are computed (G120, G126), which removes CVE-2016-9079's class.
- **What this spec asks of `hg-anim`** (amendment R10): attribute effects with interpolation through `hg-svg-interp` or `hg-svg-model`'s attribute types (number, length, list, `points`, path data, transform list, `viewBox`, colour, discrete string), because `viewBox`, `points` and gradient and filter attributes are not properties ([AN 4.1, 4.3][AN]); iteration composite `accumulate`; paced offsets supplied by this spec; the time type in `hg-types`; a scriptless core free of `hg-heap` and the event loop, which AN-D6 already provides ("a scriptless core usable in the media process"); the overlay rule above; the throttling rule below; the image-timeline rule below.

| Option | What | Verdict |
|---|---|---|
| (a) Timing graph in an SVG crate, effect model in the animation spec | Draft 0.1's recommendation | Superseded by AN-D6 |
| (b) **Timing in the animation spec's `hg-anim`; elements, attributes, geometry and value interpolation here** | One model, one clock, one composite order; SVG semantics stay with SVG | **Recommended**, as AN-D6 decided |
| (c) A separate SMIL engine with its own sandwich and interpolation | What engines historically did | Rejected: two interpolation paths for one property, two compositor rules; Blink called SMIL's cost "significant complexity" ([HP 3.2][HP]) |
| (d) No SMIL | — | Rejected: 2.9% of page loads, and with CSS the only animation in SVG images (P2) |

**Timelines** (implemented by `hg-anim`; the SVG facts it needs are these). A fragment timeline begins at the SVG load event (or at load begin with `timelinebegin="loadbegin"`) and is recorded as a time on the parent timeline, so replay never re-measures network timing ([AN 5.2–5.3][AN]). Its pause and seek state is the fragment's; `setCurrentTime` takes a `float` in seconds as the IDL requires and converts once ([AN 5.4][AN]); the media fragment `t=` seeks as if `setCurrentTime` were called ([SS 3.5][SS]). Event-based begins exist only in scripted documents (secure modes have no interaction); each event's time is the time of the task that dispatched it, a recorded input (JS-D15). `wallclock()` values resolve against the substrate clock, recorded. `<discard>` is not implemented: it is in neither specification ([SS 6.2][SS]), Chrome removed it in 81 and Firefox shipped it in 136–138 and removed it in 139, and its use counter is 0.0008% ([US 4.14][US]).

**SMIL in images.** No specification says when an SVG image's timeline starts or whether two `<img>` showing one URL share it ([AN 3.9][AN]). Izcal's rule, proposed to the animation spec: one fragment timeline per (static document, embedding document), starting when the image is first used for rendering in that document, recorded; every use of the image in that document shares it, so forty animated icons cost one raster per frame per size. The engines' behaviour is measured black-box (Q-SV-4).

**Throttling and energy** (proposed to the animation spec, which owns frame demand). Off-screen or hidden fragments are not **rendered**. In scripted documents, their interval resolution still runs at each rendering opportunity when the fragment has SMIL event listeners or event-handler attributes, or when another fragment's syncbase depends on it, so `beginEvent`, `endEvent` and `repeatEvent` fire on time; GR-D15 throttles rendering opportunities, not timing semantics. Fragments with no observer are not sampled at all, and on return their state at the current time is computed. Hidden documents follow FP-D4's freeze. Catch-up after a gap is charged to its own work budget, separate from the animation spec's per-frame interval budget (AN 6.14), and never makes an animation inert. SMIL in an off-screen image is never sampled (SV-45).

**Reduced motion (SV-D16).** No standard ties SMIL to `prefers-reduced-motion`, and CSS cannot pause SMIL ([AN 3.12][AN]). The accessibility spec pauses `marquee` under the user's reduce-motion setting because its motion is UA-driven and authors cannot gate it with a media query; inline SMIL is the same case. So under the setting, **SVG images render their sample at t = 0, paused, with MD-D19's play control** (not secure static mode, which shows base values and, for a spinner whose base opacity is 0, nothing), and **inline SMIL fragments are paused at t = 0 with a page-level play control**, both with MD-D19's per-site setting to turn animation back on. The reversal trigger is measurable: a breakage rate above 1%† on the SMIL corpus with pausing on (amendments R8, R12, R14).

**Which attributes SMIL may animate (G126).** `hg-svg-model`'s animatable-attribute table is generated from the specifications' animatability column, with one security rule on top: **URL-valued attributes whose consumer navigates or runs script are not animatable** (`href` and `xlink:href` on `a`, on `script`, and on any element whose activation navigates), event-handler attributes are not animatable, and an animated value that would produce a `javascript:` URL is discarded in every case. The PortSwigger vector that animates an `<a>`'s `href` to `javascript:` through a `values` list ([PortSwigger, 2020](https://portswigger.net/research/svg-animate-xss-vector); [SR 4.2–4.3][SR]) therefore cannot reach navigation even when a sanitizer misses it. Animating the `href` of `image`, `feImage`, `use` or `textPath` stays allowed in every mode, because their consumers only fetch or reference; in secure modes a `use` whose `href` animates is bounded by its region and the ledger (6.7). Which attributes the three engines refuse is measured black-box at SV-M0 (Q-SV-8); a page that depends on animating a link's `href` is the reversal trigger.

### 6.10 Text in SVG

**Pipeline.** A `<text>` element is laid out by CSS as a block with its `tspan`, `textPath` and `a` children as inlines ([RG 1.1][RG]); `hg-layout`'s inline library call (LY 6.5.10) lays it out on one line and returns, for every DOM character, its addressable flag, its typographic-character grouping (the text stack's cluster map, TX 6.8) and its position. `hg-svg-layout` then runs SVG 2's text layout algorithm ([SVG 2 CR §11.5](https://www.w3.org/TR/SVG2/text.html#TextLayoutAlgorithm); [RG 1.2–1.7][RG]): resolve `x`, `y`, `dx`, `dy` and `rotate` lists against addressable characters (descendants override ancestors; ligature rules so positions do not depend on whether a font forms the ligature), apply `textLength` (`spacing` spreads the difference; `spacingAndGlyphs` scales glyph advances and outlines horizontally by one factor per node, Izcal's written rule since the specification gives no algorithm), apply absolute positions and anchored chunks with `text-anchor` and bidi, then place characters on paths. Glyph outlines come from the text stack at a **font size quantized in the text's local user units** (TX-D4), so text under a `viewBox` scale keeps its size and the same glyphs are drawn in the renderer and the media process.

**The published algorithm has defects** (`new_check_count`, `resolve_dx` read from `dy`, `lrt`, an x/y loop starting at index 1, undefined `i` and `k` in the path loop), present in the CR and in the 13 October 2026 Editor's Draft, so a literal implementation is wrong ([RG 1.5][RG]). Izcal writes a corrected algorithm in its anchors, tests each correction against the three engines black-box, and files the defects with the SVG WG (Q-SV-1).

**`textPath`.** The path comes from `href` to any geometry element or from the new `path` attribute; `side="right"` reverses it; `startOffset` may be negative or above 100%; a character whose mid-point falls off an open path is hidden; a single closed subpath wraps once ([SS 5.7][SS]; [RG 1.7][RG]). Positions along the path use `hg-svg-geom`'s one arc-length function (6.6). `method="stretch"` and `spacing="auto"` have no algorithm in the specification ([RG 1.8][RG]); Izcal implements `align` and `exact` and treats the others as those, which the specification's "free to make any additional adjustments" permits, and records it (SV-D13).

**Wrapping.** `inline-size` (auto-wrapped text) is in SVG 2, but no engine implements it, every WPT `text-inline-size`, `text-multiline` and `text-shape-inside` reftest fails in all three, and an open WG issue (#1087, April 2026) proposes removing it ([RG 1.10–1.11][RG]; [GO 6.4][GO]). It is **held** (SV-D13); LY's inline call already supports wrapping, so admission is a switch, not a redesign. `shape-inside` was removed from SVG 2 ([SS 5.6][SS]).

**Vertical text** uses `writing-mode` and `text-orientation`; `glyph-orientation-vertical` is supported as SVG 2's alias (`auto` → `mixed`, `0deg` → `upright`, `90deg` → `sideways`) ([RG 1.12][RG]). `text-rendering` is a hint and never disables anti-aliasing (GR 6.16).

**Find, selection, copy and translation.** SVG 2 makes search and selection of SVG text a conformance requirement for viewers in browsers ([RG 1.14][RG]), and SVG text that is "basically an image" to tools is a recorded user failure ([GO 2.7][GO]). Izcal's SVG text is in the document's **text model** like HTML text: find-in-page matches it with the text stack's matching rules, selection runs in logical order as SVG 2 specifies, copy yields its characters, and the semantic plane gets per-character rotated quads (6.12; SV-60). The SVG text DOM methods (`getComputedTextLength`, `getSubStringLength`, `get{Start,End}PositionOfChar`, `getExtentOfChar`, `getRotationOfChar`, `getCharNumAtPosition`) read the same per-character results.

### 6.11 Hit testing and `pointer-events` (SV-D15)

**One definition.** Hit testing **is** `isPointInFill` and `isPointInStroke` (6.6) evaluated at the pointer position mapped into the element's user space through the record's CTM, with the same exact-curve fill test and the same stroker; paint's flattened outline is used only as a conservative pre-filter. Hit testing, the geometry APIs and painting therefore agree, and SV-3 checks the first two on 10⁶ generated points.

`hg-hit` walks paint chunks top-down (LY 6.14); when it reaches a chunk belonging to an SVG root, the chunk-to-SVG-record map (6.5) hands it the records in reverse paint order. For each record:

1. **Eligibility** by `pointer-events` and `visibility`, with all eleven values as SVG 2 defines them (`visiblePainted` the effective initial value; `fill`, `stroke` and `all` ignore paint; `bounding-box` hits the box) ([SVG 2 CR §15.6](https://www.w3.org/TR/SVG2/interact.html#PointerEventsProperty); [RG 7.1][RG]).
2. **Geometry**: the stroke test includes dashes, so a dash gap does not hit, matching both painting and `isPointInStroke`, which includes dashes ([RG 2.4, 7.5][RG]). Text hits per character cell (rotated quads from 6.10); raster images hit their rectangle; **markers never hit** ([RG 7.2][RG]). `clip-path` limits hit testing as CSS Masking specifies. Where SVG 2 leaves the rule open (dash gaps; clip with `bounding-box`), Izcal's rule is written down, tested and filed (Q-SV-10).
3. **Instances.** A hit inside a `use` shadow tree returns the instance; DOM's event retargeting presents the `use` element to listeners outside the shadow tree (6.7).

**Scale.** Charts with thousands of marks hit-test on every pointer move. An SVG root with more than 256† records keeps a **spatial index** of record bounds (a uniform grid sized from the root's bounds and record count), built deterministically by `hg-svg-layout` as class (a) data of the render tree and read through `SvgRenderTree` (G127), so `hg-hit` never reads paint internals (LY G38). A hover is a grid lookup plus exact tests on the few candidates (SV-42).

### 6.12 Accessibility

The accessibility spec computes the semantic tree and owns the generated SVG-AAM and Graphics-AAM mappings (AX 6.2, G103). This spec gives it what SVG-AAM's rules need, through `SvgSemanticSource`, which `hg-semantic` reads under G100 (amendment R12):

| SVG-AAM input ([SVG-AAM 1.0, WD 2026-10-12](https://www.w3.org/TR/svg-aam-1.0/)) | Provided by this spec |
|---|---|
| Inclusion triggers: a non-empty child `title` or `desc`, ARIA naming and role attributes, a valid `tabindex`, focusability ([SS 8.3][SS]) | Read by AX from the tree; this spec supplies which element is the **first applicable `title`** and `desc` child (multiple `title` elements for localisation are at risk in the CR and selected only if admitted, [SS 1.2][SS]) |
| Exclusion: `display: none`, conditional-processing losers under `switch`, and elements never directly rendered, by SVG-AAM's general rule and its element mapping tables ([SS 8.2, 8.4][SS]) | The rendered state from the render tree, including `switch` evaluation |
| "Hidden" = invisible **and** not pointer-interactive ([SS 8.4][SS]) | Computed `visibility` with `pointer-events` eligibility from 6.11 |
| `use` shadow content processed as its children ([SS 8.5][SS]) | The instance tree (6.7) |
| Bounds | Device-space quads through `SvgGeometry` |
| Text | Per-character positions and rotations from 6.10, for the text model and caret browsing (AX 6.12) |

Inline SVG is a `graphics-document` and its shapes `graphics-symbol` when included ([SS 8.2][SS]); the commonly recommended `role="img"` with a `title` works in every screen-reader and browser pairing tested in 2019–2021, while SVG inside links and buttons is the weak spot ([GO 2.2–2.4][GO]). An SVG image's content is never in the tree: the `<img>`'s text alternative is, as HTML-AAM says. A top-level document in the scriptless interactive mode (SV-D9) is exposed like any document: its static document's own style scope and change log feed the semantic thread (amendment R12). **Forced colours**: engines differ (Chromium's UA sheet exempts SVG; Firefox forces `currentColor`) ([GO 2.6][GO]); Izcal follows CSS Color Adjust at the pinned snapshot and the accessibility spec records the UA-sheet rule (Q-SV-9). `title` tooltips are shown by the shell like HTML's `title` attribute.

### 6.13 Security and resource budgets (SV-D6, SV-D9)

**Threats and the design that answers each.**

| Threat | Evidence | Answer |
|---|---|---|
| Memory-safety bugs in SVG code (`use` lifetimes, animation callbacks, resource client lists, type confusion, filters) | CVE-2016-9079, exploited and on CISA's KEV list since 22 June 2023; CVE-2012-5112; CVE-2013-0912 ([SR 1.1–1.7][SR]; the class list is an inference, Low) | Rust without `unsafe` (G128); instances and graph edges as generation-checked ids (6.5, 6.7); no script inside any SVG algorithm, and instance building outside DOM's clone (G120, G126, 6.7); SMIL publishes after computing (6.9) |
| Amplification: nested `use`, pattern and template chains, deep nesting, huge filters, giant paths | librsvg publishes caps of 1,000,000 elements and 500,000 resolved references and "no built-in limits on the total amount of memory or CPU time" ([librsvg security](https://gnome.pages.gitlab.gnome.org/librsvg/devel-docs/security.html)); CVE-2019-20446 (nested patterns) and librsvg #323 (nested `use`) show the attack ([SR 2.1–2.4][SR]); SVG 2 sets no limits ([SS 4.7][SS]) | One **weighted ledger** per document and a cap per process (below) |
| Script in image contexts | SVG as "fully functional, one-file web applications" (Heiderich et al., CCS 2011, [SR 4.1][SR]) | Secure modes are the scriptless host, which cannot name script machinery (G120); probed (SV-13) |
| Fetches from images (tracking, probing) | SVG 2's network-error rule ([SS 1.4][SS]; [SR 5.2][SR]) | The scriptless host has **no fetch endpoint**; `data:` is decoded locally; probed (SV-13) |
| Animated `href` to `javascript:` | PortSwigger 2020; sanitizer bypasses into 2026 ([SR 4.2–4.3][SR]) | G126 |
| Namespace-confusion mXSS | DOMPurify CVE-2020-26870 ([SR 4.4][SR]) | ST's one parser and its round-trip differential, with this spec's SVG and MathML corpus (6.3) |
| `data:` in `use` | Trusted Types and Sanitizer bypasses; removed ([SR 4.5][SR]) | Not supported (6.7) |
| URL parser disagreement | librsvg CVE-2023-38633 read local files ([Canva engineering](https://canva.dev/blog/engineering/when-url-parsers-disagree-cve-2023-38633); [SR 7.3][SR]) | Every SVG URL is parsed once by `hg-url` (NW G31); the media process has no file system or network (MD 6.1) |
| Pixel stealing through filter timing | Stone 2013, Kotcher et al. 2013, Andrysco et al. 2015, Kohlbrenner and Shacham 2017, GPU.zip 2023 ([GPU.zip](https://www.hertzbleed.com/gpu.zip/); [SR 3.1–3.8][SR]) | GR-D18 and G69: cross-origin pixels never enter a page-controlled effect, with SVG's per-record taint flags (6.5); GR's integer kernels have no data-dependent fast paths, which Filter Effects' constant-time rule for tainted primitives asks for ([Filter Effects 1 §15.1](https://www.w3.org/TR/filter-effects-1/#tainted-filter-primitives); [SS 5.5][SS]) |
| Clickjacking through filter logic over framed pages | In December 2025, filter primitives were shown to form logic over a framed cross-origin page's pixels and drive what is drawn ([GO 4.4][GO]); not a timing attack | GR-D18 and G69: a framed cross-origin surface is transparent black to page filters |
| SVG phishing attachments opened from disk | Kaspersky counted 2,825 SVG phishing emails in Q1 2025 ([Securelist](https://securelist.com/svg-phishing/116256/)); KnowBe4 put SVG at 6.6% of malicious attachments ([SR 6.1–6.3][SR]) | SV-D9 below |
| Panics on hostile input in the media process | A panic ends every image and video of the key (MD G53) | SVG crates linked into the media process are built with **the L2m profile** (no panic on any input, checked arithmetic on content-derived sizes, allocation through budget handles, fuel per input-driven loop), the ledger being their fuel; panics count as crash bugs (SV-14; amendment R8) |

**The ledger (SV-D6).** Independent per-resource caps let one document stack them all, and a cap high enough for legitimate content (a million elements) is far beyond any time and memory bound. So every document has **one weighted ledger in canonical units**. Each charge (element, instance node, path segment, marker instance, reference resolution, filter primitive × its output area in **CSS pixels at device scale 1**, nested document; SMIL intervals are budgeted by the animation spec, AN 6.14) carries a generated weight in canonical bytes and canonical work units, and the document's totals are capped at **64 MiB† canonical bytes and 2²⁸† canonical work units**, derived from SV-11 on the slowest reference device. Filter work is counted in CSS pixels and evaluated once per filter definition and region, never per frame, so a filter never disappears when the user zooms or moves to a higher-DPR screen. Scriptless documents in one process also share a **per-process cap of 256 MiB†** canonical bytes, charged in creation order; the media process's share lies inside MD-53. The ledger is a prefix sum over document order maintained incrementally (6.7), so it is a pure function of the tree (G124, SV-32), identical on every device (FP-D19), and every limit is checked against the corpus at SV-M0 (Q-SV-3).

**Structural caps** stay as caps, because they bound depth, not volume:

| Structure | Limit† | On exceeding it |
|---|---|---|
| `use` nesting depth | 64 | The `use` is in error and not rendered |
| Reference-chain depth (templates, resources inside resources, `feImage` of elements) | 64 | The reference is invalid (6.8's per-property result) |
| Nested SVG documents (an SVG image inside an SVG image, `foreignObject` and SVG roots nested) | 4 documents; 64 `foreignObject`/SVG-root levels | Deeper ones are broken images or render nothing; OT-SVG glyph documents may not themselves use OT-SVG fonts |
| Filter primitives per filter | 64 (GR 6.3's semantic limit) | The filter is treated as invalid (chain ignored) |
| Pattern nesting | 8 | Deeper patterns paint nothing |
| Elements in an OT-SVG glyph document | 10,000 (TX 6.3) | The glyph falls back to its outline (TX) |
| Path segments | 65,536 per path (GR 6.3, with splitting, 6.5) | Rendered up to the limit |

On ledger exhaustion, the element whose charge crosses the cap is in error with the specification's state for its kind (an element not rendered, a reference invalid, a filter ignored, an animation inert), in document order, with a `svg.budget` event. **Filter regions are exact**: a filter's effective region is its specified region intersected with the area that can reach the visible output, computed from each primitive's maximum reach (offsets, 3σ of blurs, morphology radii, displacement scale, convolution order). Izcal never reduces filter resolution under pressure, although Filter Effects permits it ([RG 4.6][RG]); this is this spec's rule, proposed to graphics (amendment R7).

**Local SVG files (SV-D9).** A top-level SVG document from the web is a full document with script in its origin (dynamic interactive mode), as the specification requires, and an SVG upload served from a site's own origin is that site's XSS, defended by the site's CSP, sandbox domains or `Content-Disposition` ([SR 4.7][SR]). The case a browser can change is the one the 2025 phishing wave uses: a `.svg` attachment the operating system opens in the default browser, where it runs as a page ([SR 6.3][SR]). Secure animated mode would be the wrong answer: it removes hyperlinks, selection and find (which SV-60 and SVG 2 require of a viewer), and the linked raster images that editors such as Inkscape reference by relative path by default ([SR 7.4][SR]). So Izcal opens a top-level SVG from a `file:` URL, or one the user opened from the downloads list, in its own mode, **scriptless interactive**, recorded as a deviation: no script and no event handlers; hyperlinks followable by user activation; selection, find and accessibility on; references allowed to `data:` and to `file:` URLs in the same directory or below, resolved once by `hg-url` and read through a broker grant for that directory, never by the renderer; no network; an opaque origin. A bar says the file contains active content, with one action to reopen it with script in an opaque origin. **This ships as the default from first release**; SV-S5 measures, separately, the share of a corpus of real local SVG files that needs script, network, links or sibling files, and can only reverse it (trigger: more than 1%† need script). It is an owner decision because it changes what a file does when opened (amendments R4, R14, R17).

**Top-level `data:` SVG.** Chrome 60 and Firefox 59 block page-initiated top-level navigation to `data:` URLs, and Firefox's exemption for `data:image/*` excludes SVG ([SR 4.6][SR]). No Izcal spec states this yet; this spec asks the DOM spec's navigation algorithm to block it, with no exemption for `data:image/svg+xml` (amendment R4).

### 6.14 Privacy

- **Partitioning.** Static documents, resource documents and raster caches are keyed by fetch partition and request mode and never shared across a partition (FP §6.4); an SVG image or sprite cached for site A is invisible to site B (SV-21).
- **No user state in images** (6.8): `:visited` is never applied in scriptless documents, which ST-13's `:visited` suite already probes for SVG-as-image documents.
- **Text geometry.** SVG's text-measuring methods expose the same quantities as HTML text geometry, which font-metrics fingerprinting used to make 34% of over 1,000 browsers unique ([GO 4.1][GO]); they are answered from the same integer, device-independent metrics as HTML (TX-D4) under the same font-visibility rules, so they add no bits.
- **Conditional processing.** `systemLanguage` is evaluated against the language list that `navigator.languages` already exposes **to that document** (for an image, the embedding document's), as D-S1 as amended by OD-6 normalizes it, and `requiredExtensions` is false for every value except the namespaces Izcal supports; SVG 2's own privacy note names both ([SS 7.6][SS]). Because `switch` can then make an image's rendered subtree depend on the language list, and a non-tainting image can be read back from a canvas, the language list is on SV-S3's list and PRIV-2 gains a case.
- **Filters over other origins**: GR-D18 (6.13).

### 6.15 Determinism (B1)

Every SVG output a page, a test or an agent can observe is a pure function of recorded inputs: the render tree and its hash, display items, geometry API results, SMIL samples and events, and SVG image rasters. The decision points below join JS 6.9's consolidated list, which is complete per owning spec (amendment R2):

| Decision point | Recorded as |
|---|---|
| Installation of an external resource document into a referencing document | The network input that completed it, joined by request id, and the flush at which it was installed |
| Completion of an SVG image raster in the media process | The renderer's receipt of the raster at a rendering opportunity |
| Fragment timeline begin; image timeline start | Times on the parent timeline (6.9) |
| Event-based SMIL begins | The dispatching task's time (JS-D15) |
| Arrival of SVG image bytes | Network inputs, recorded by the networking spec's recording |

What is not a decision point because it cannot change output: numbers (G122's discipline); budgets (device-independent, a function of the tree); placement in the renderer or the media process and CPU or GPU raster (SV-31, G70); iteration order (no hash-map order reaches output; reference-graph walks are in document order); incremental rebuilds (differential against from-scratch, G127).

### 6.16 Footprint and performance

**Per element.** A graphics element costs, beyond its DOM columns and style (FP-11): one render record (target ≤ 64 B† average, SV-50), its geometry by content hash, bounding boxes in the record, and nothing for the SVG DOM until script touches it (6.2). **Path data is stored once per partition**: the computed `d` value is the content hash of the parsed path in a per-partition path table shared by style and the render tree; display items reference paths by that hash, and the content GPU process deduplicates by hash per key, so a page that inlines one icon 300 times stores its path once, and a large `d` is not stored as source, computed value, render-tree path and display path four times. This spec asks the footprint spec to count SVG render records in FP-11's **layout** share (SVG elements have no boxes or fragments) and to add **path geometry** as an excluded category like text, with its own ledger line and a per-segment bound (amendment R11).

**Per image.** A static document costs no agent heap, no realm, no wrappers and no event loop (6.1); it does pay for its own style scope, which SV-51's 48 KiB† base includes. One parse serves every use of a static-document key; the raster cache follows FP-13.

**Data visualisation.** Published ceilings for SVG charts are a few thousand marks; one study found SVG and canvas about equal, a micro-benchmark found canvas ten times faster for 1,000 objects, and no engine publishes element-count scaling ([GO 3.1–3.3][GO]; [US 7.4][US]). Izcal sets its own targets for a 10,000-element chart (SV-41, SV-42) and measures the three engines black-box in SV-S2, joined to GR's GR-S3 harness. The costs that dominate are per element (style, geometry building, the render record, hit-test indexing) rather than raster throughput ([RG 5.2–5.3][RG], Medium), which is why the render tree shares paths, the incremental pass re-emits only dirty records, and markers and `use` instances share geometry.

**Compositing.** Izcal's rule: transform and opacity animations of SVG elements (CSS, Web Animations or SMIL) are eligible to be compositor-mutable like any other element's, under GR-D13 and GR-D5's cap of 16† compositor-mutable nodes per document. Chrome reports composited SVG animations since version 89, listing opacity, filter and transform ([AN 4.5][AN]).

### 6.17 Observability and user experience

The DevTools research found SVG among the least visible parts of today's tools: neither major animation inspector lists SMIL ([GO 1.1][GO]), and the box model does not fit SVG elements ([GO 1.3][GO]); the researcher also recalls (unverified, Low) that no tool shows resolved paint servers, filter regions or the provenance of `use` instances ([GO 1.5–1.6][GO]). Izcal has the data because the render tree and the reference graph are explicit. The IIP gains an **`svg.*` domain** (with the DevTools track, under DevTools' stability tiers):

| Query or event | What it shows |
|---|---|
| `svg.renderRecord(node)` | The element's record: CTM and screen transform, viewport chain, geometry, fill, stroke and marker boxes, resolved paint and resources, with the spec anchor of each field |
| `svg.references(node)` | Forward and reverse edges of the reference graph, including which invalid-reference rule applied and why a reference did not resolve (missing id, wrong element type, cross-origin without CORS, `data:` in `use`) |
| `svg.instances(use)` | The instance tree with each instance's source node and the mutation that last rebuilt it |
| `svg.filterGraph(node)` | The primitive graph with subregions and, on request, each primitive's intermediate result as a canonical raster (same-origin content only) |
| `svg.timeline(fragment)` | Served by `hg-anim`'s decision records (AN 6.15): SMIL intervals, instance-time lists, why each begin resolved, and the effects; this spec adds the attribute values and the overlay |
| `svg.budget` events | Every ledger exhaustion and structural cap with its element, limit and, for prefix-sum changes, the mutation that caused it |
| `svg.parseErrors` | Every micro-grammar error with its position (path data rendered up to the error; SVG 2's "report all errors", [RG 2.1][RG]) |
| `svg.imageInfo(image)` | Processing mode, placement, why a font or resource did not load, raster cache entries; for opaque images served from the media process through the broker, never through the renderer |

The DevTools frontend (DT) draws these as overlays (bounding boxes of each kind, viewports and `viewBox`es, the current transform), a filter-graph view and a SMIL timeline beside CSS and Web Animations in the animation spec's inspector. **Agents** (only with AI on, B9 and B10) can read, with the user's consent, the **resolved scene** of an SVG (shapes with geometry, paint and text in device coordinates, with their semantic roles), which is more reliable than raw markup for language models reading charts ([GO 5.1–5.2][GO]). **Tests and agents** get identical SVG screenshots on every machine, because the raster is canonical, where Playwright documents that baselines vary by OS, hardware and power source today ([GO 5.3][GO]).

**For people browsing**: crisp SVG at every zoom (re-rastered, never scaled bitmaps); find, select and copy in SVG text; animated SVG that respects reduced motion with a play control; and local SVG files that open as documents without script (SV-D9).

### 6.18 Crate layout and rules

| Layer | Crate | Role | Tier |
|---|---|---|---|
| L2 (`hg-types`) | time type (owned by the animation spec) | An integer count of a fixed time quantum, shared by SMIL, Web Animations and the frame clock | P |
| L2p | `hg-svg-syntax` | Every SVG micro-grammar (G121) | R, L2m profile |
| L2p | `hg-svg-model` (generated by `hg-svggen` from `.hgsvg`) | Element and attribute registry, static name indices, presentation-attribute mapping, reference and animatable tables, categories | R, L2m profile |
| L2p | `hg-svg-geom` | IEEE-strict f64 geometry: transforms, arcs, extrema, arc length, point at length, containment, bounding boxes, `fmin`/`fmax`, the boundary function `to_display` | R, L2m profile |
| L2p | `hg-svg-interp` | Path-data, `points`, number-list and transform-list interpolation behind `InterpolateValue` | R, L2m profile |
| L3 | `hg-node-store` (proposed; owned by the JS and DOM owners) | The column store, attribute-set references, text arena, untraced rare data, `hg-tree-traits` implementation, `StoreWriter<B>` (6.1) | R |
| L3 | `hg-static-doc` | Scriptless documents: `StaticSink`, `ArenaBarrier`, regions, style scope, resource-document registry, processing-mode capability, `ResourceLoader` implementations | R, L2m profile |
| L3 | `hg-svg-instances` | The instance builder for `use` and templates (6.7) | R, L2m profile |
| L3 | `hg-html-render` (proposed; owned by the DOM owner) | Rendering-only HTML element behaviour shared by `hg-html` and the static host (6.1) | R, L2m profile |
| L3 | `hg-svg-dom` | SVG element behaviour and the SVG DOM for scripted documents (generated bindings through DM's codegen); `SvgGeometryQuery` and `SmilControl` traits | R |
| L4 | `hg-svg-layout` | The SVG layout pass, `SvgRenderTree`, the reference graph, markers, text placement, the spatial index, `SvgGeometry` and `SvgSemanticSource` | R, L2m profile |
| L4 | `hg-svg-image` | Scriptless rendering driver, raster requests and cache keys, the opaque-image protocol, glyph profile for OT-SVG | R, L2m profile |
| L4 | `hg-svg-observe` | IIP `svg.*` types and probes, counters | R |
| tools | `hg-svggen` | `.hgsvg` → `hg-svg-model` tables; fails on disagreement with the pinned IDL and indexes | T |
| tools | `hg-svg-reference` | Independent, spec-literal reference implementation of geometry, viewports, text positioning, the reference graph and SMIL intervals, in exact rational and interval arithmetic Izcal writes itself (no bignum or interval crate) (G129) | T |
| tools | `hg-svg-fuzz`, `hg-wpt-svg`, `hg-svg-corpus` | Grammar, structure and amplification fuzzers on x86-64, arm64 and riscv64; the WPT runner with exact-match reporting; the corpus and its black-box comparison harness | T |

"L2m profile" means the media spec's input-hardening profile (MD G53), applied because these crates run on opaque input in the media process. `hg-svg-paint` stays in the graphics spec (GR 6.24). Every crate above is `#![forbid(unsafe_code)]` with no inventory entry. The L2p crates depend only on L2, L2s (`hg-math`) and each other, as ST's G24 requires.

**Rules G120–G129** (this spec's range):

| Rule | Statement | Why |
|---|---|---|
| **G120** | No crate reachable from `hg-static-doc`, `hg-svg-image`, `hg-svg-layout`, `hg-svg-instances`, `hg-html-render` or the L2p SVG crates (and `hg-anim`'s scriptless core, under the animation spec's matching rule) depends on `hg-heap`, `hg-js-host`, any L2e crate, `hg-dom`, `hg-bindings`, `hg-script`, `hg-script-api`, `hg-html`, or any generated IPC endpoint of the partition service or the font service; loaders reach style and text only through `ResourceLoader` traits; `StoreWriter` is constructed only in `hg-dom` and `hg-static-doc`; `hg-graph-lint` proves it over the media process binary and a static-host build, from SV-M0 | Secure modes by construction (6.1, 6.8); MD-D20's process tier; JS-D4's write discipline |
| **G121** | Only `hg-svg-syntax` parses an SVG micro-grammar; DOM setters, presentational hints, the CSS `d` and `transform` grammars where shared, SMIL and the sanitizer call it; every other check has a CI differential against it | ST G22, G23 extended; parser disagreement is a security class ([SR 7.3][SR]) |
| **G122** | SVG geometry and interpolation use only binary64 `+ − × ÷`, `sqrt`, `abs`, `copysign`, `floor`, `ceil`, `trunc`, `round_ties_even`, comparisons, `total_cmp` and `hg-math`, in a fixed order with no FMA; `min`, `max`, `clamp`, `%`, `rem_euclid`, `powi`, `powf` and other methods are rejected by `clippy::disallowed_methods`; NaN is canonicalized where observable; no `f32` except at IDL `float` boundaries; floating point stops at `hg-svg-geom::to_display` | DET-1 across three ISAs (6.6) |
| **G123** | Every walk over SVG references (instances, templates, resources, `textPath`, `mpath`, `feImage`) and over nested SVG and `foreignObject` roots uses an explicit stack or work list, a visited set and the specification's cycle rule, and charges the ledger; no recursion over content depth | DoS and stack exhaustion ([SR 2.1–2.2][SR]); ST G21, DM G98 |
| **G124** | Every resource bound is either a structural cap or a weighted ledger charge, identical on every device, maintained as a prefix sum over document order, with a specified result and an IIP event on exceeding it; script-called geometry work has a per-call budget | 6.13; FP-D19; G127 |
| **G125** | A document's processing mode is fixed at host creation; secure modes and the scriptless interactive mode are only ever `hg-static-doc` hosts; no API upgrades a static document to a scripted one ("run with script" reopens the file as a new document) | SVG 2's modes as a type |
| **G126** | No SVG crate runs script or writes the DOM during layout, instance building or animation sampling; the instance builder never uses DOM's clone algorithm; animated values reach consumers only through the overlay and the animation spec's effects; URL-valued attributes that navigate or run script are not animatable and no animated value is a `javascript:` URL | [SR 1.1–1.2, 4.2][SR]; DM 6.2 |
| **G127** | `hg-svg-paint`, `hg-hit` and `hg-semantic` read SVG only through `SvgRenderTree`, `SvgGeometry` and `SvgSemanticSource`; the render tree, the spatial index and the ledger are a pure function of the tree, computed style, the overlay, text results and the viewport, with a CI differential of incremental against from-scratch | One source of truth for paint, hit testing and accessibility; LY G42 extended |
| **G128** | Every SVG crate is `#![forbid(unsafe_code)]` and carries spec anchors on its algorithms (G9); crates linked into the media process follow the L2m profile | G5, G9; MD G53 |
| **G129** | `hg-svg-reference` shares no project-local code with `hg-svg-geom`, `hg-svg-layout`, or `hg-svg-interp` beyond `hg-tree-traits`, `hg-types`, style's read API and `hg-text` | An independent oracle, as LY G41 and ST G20 |

### 6.19 Interfaces to other specs

| Interface | Counterpart | Contract |
|---|---|---|
| `hg-node-store`, `StoreWriter<B>` | JS engine (JS-D4), DOM (DM 6.1) | The column store moves out of `hg-dom`; writes to a scripted document only through `StoreWriter<TracedBarrier>`; `hg-heap` traces a `Store<Traced>` through the node-space adapter in `hg-dom` |
| Instance builder outside DOM's clone; listener dispatch from the source; shadow-root reads | DOM (DM 6.2, 6.5, 6.6) | 6.7 |
| `hg-html-render` | DOM (DM 6.12, 6.17) | Rendering-only element behaviour, script-free, shared with the static host |
| `StaticSink`; SVG name indices; style scopes; `ResourceLoader`; binary64 computed lengths; overlay through `hg-tree-traits` | Style (ST 6.2, 6.6a, 6.9, 6.11, G24) | 6.1, 6.2, 6.6, 6.9 |
| Presentational hints; "affects" categories svg-geometry and svg-resource | Style (ST 6.8, 6.9) | `hg-svg-model::hints`; two new mask categories |
| `SvgGeometry`; inline-layout call; forcing-table entries; one work list with `foreignObject` roots | Layout (LY 6.1, 6.5.10, 6.12, G43) | Rectangles converted by `hg-layout-num`'s rounding; SVG geometry methods and `use` shadow-root reads in the forcing table; task-local render trees merged under G43 |
| Chunk-to-SVG-record map | Graphics and layout (`hg-paint-order`) | `hg-hit` descends into SVG roots |
| `SvgRenderTree`; resolved resources; per-path exponent with the f64 fallback; taint flags; canvas origin-clean for `foreignObject` images | Graphics (GR 6.3, 6.15, 6.16, 6.28, GR-D18) | 6.4, 6.5, 6.8 |
| SMIL split (AN-D6), attribute effects, the overlay, `InterpolateValue`, time type, image timelines, throttling with observers | Animation spec | 6.9 |
| Opaque SVG images; `data:` fonts; reduce-motion sample at t = 0; L2m profile | Media (MD-D20, MD-D19, G53) | 6.8, 6.9, 6.13; Q-MD-18 answered by 6.1 |
| OT-SVG glyph profile | Fonts and text (TX 6.3) | Secure static, TX's limits and fuel; OT-SVG fonts not usable inside glyph documents |
| `SvgSemanticSource`; reduce-motion for inline SMIL | Accessibility (AX, G100) | 6.12, 6.9 |
| Gzip-magic `image/svg+xml` decoding | Networking (NW-D6) | 6.3 |
| SVG decision points | JS engine (JS 6.9) | 6.15 |
| OT-SVG glyph export; SVG filters in print | PDF (PD-D19, PD-D20) | Through `hg-svg-paint`'s display items, nothing more |
| `svg.*` domain | DevTools (DT) | 6.17 |

## 7. M-1 compliance

**Built new, by Izcal, from the specifications.** Everything in Section 6.18: the SVG micro-grammar parsers, the element and attribute model and its generator, the geometry and interpolation libraries, the scriptless document host, the instance builder, the SVG DOM, the SVG layout pass and render tree, the reference graph, SVG text positioning and `textPath`, the SMIL timing front end, the SVG image driver and its protocol, hit testing of SVG content, the semantic source, the ledger, the IIP domain and the independent reference implementation with its own exact and interval arithmetic. No code, design template or internal API shape is taken from resvg, usvg, librsvg, Batik, Inkscape, any engine's SVG module, svgtypes, roxmltree, kurbo, lyon, tiny-skia or any other project. Other engines and libraries appear only as published evidence and black-box oracles (M-1 part 3).

**M-1 part 3, evidence.** No link in this spec or its research notes points to source code of any project (code files, headers, commits, diffs or code reviews), under the owner's rule of 19 and 20 October 2026 that extends this to every third-party project. Phase 8 found three borderline links in the notes (a Mercurial changeset of the SVG 2 specification repository, a code library's repository root, and a repository root cited for README prose); the first two were removed and the third now points to the README file, which OD-7 counts as prose (Appendix A, FC-R1). Each note records where search results pointed at source and were not opened: Chromium code reviews and commits, a librsvg source file holding its limits, a Firefox header and WebKit changesets, and third-party pages summarizing Blink's SVG text internals ([HP][HP], [SR][SR], [RG][RG] headers). Values that exist only in source (engines' `use` and filter limits, coordinate storage types, image timeline rules, attributes engines refuse to animate) are black-box questions (Section 11). Bug-tracker pages are cited for their prose only. MDN's compatibility data is cited through its web pages. The specification's own pseudo-code (SVG 2's text algorithm, Filter Effects' feTurbulence) is specification text. Adaptive Gauss–Legendre quadrature is taken from the numerical-analysis literature (6.6), not from any library.

**Dependencies used unmodified.** None beyond Rust's standard library and the crates other specs own (`hg-math`, `hg-url`, `hg-compress`, `hg-text`, `hg-path`). `hg-svg-reference`'s rational and interval arithmetic is written by Izcal; no bignum or interval crate is used.

**Data.**

| Data | Use | Class | Licence |
|---|---|---|---|
| SVG 2 element, attribute, property and IDL indexes (CR 2026-10-06); SVG Animations Level 2 IDL (ED); SVG Paths IDL | Restated in Izcal's own `.hgsvg` text; compiled tables in the engine image | Standards data (ships) | The licence stated on each pinned snapshot is recorded at SV-M0 (W3C publications carry the W3C Document License or the W3C Software and Document License); the standards-data exception covers either. The `.hgsvg` text is Izcal's, MIT |
| SVG UA style sheet | Engine image (ST) | Standards data, owner-confirmed (Q-ST-19) | As recorded by ST |
| SVG-AAM and Graphics-AAM mapping tables | `hg-ax-mappings` (AX) | Standards data | As recorded by AX |
| Izcal's own UI icons | Shell, DevTools, prompts | Izcal-authored | MIT |
| WPT `svg/`, `css/css-masking`, `css/filter-effects`, `svg-aam/` | Tests | Test-only | BSD-3-Clause |
| The SVG 1.1 Second Edition test suite (manual tests) | Tests, converted by Izcal | Test-only | W3C test suite licence |
| resvg's published regression suite (about 1,600 SVG-to-PNG cases, [HP 4.4][HP]) | Black-box test inputs only, never shipped: only `.svg` and `.png` assets are fetched, never its harness files; its PNGs are another renderer's outputs, used for comparison, never as canonical-raster oracles | Test-only | To be recorded before use (Q-SV-16) |

**Anchors** (SVG 2 sections are the CR of 6 October 2026, as numbered in the fact-check, [FK][FK]):

| Part | Anchored to | Oracle |
|---|---|---|
| Grammars | SVG 2 §4 (data types), §9 (path data), §8 (`viewBox`, `preserveAspectRatio`); SVG Animations (timing values) | Grammar fuzzers; WPT `svg/types`, `svg/path` |
| Model and presentation attributes | SVG 2 §6.6 and the indexes | WPT `svg/styling` |
| DOM | SVG 2 §4.3.4–4.3.6 and §4.6 (reflection), the IDL appendix; SVG Paths; SVG Animations IDL | WPT `svg/types/scripted`, `svg/path` |
| Viewports and units | SVG 2 §8 | WPT `svg/coordinate-systems` |
| `use`, templates | SVG 2 §5.5, §14.1.1 | WPT `svg/struct`, `svg/pservers` |
| Resources | CSS Masking 1, Filter Effects 1, SVG 2 §13–14, §16.1.7 | WPT `css/css-masking`, `css/filter-effects`, `svg/painting` |
| Text | SVG 2 §11 (corrected, 6.10), CSS Text and Inline | WPT `svg/text`; `hg-svg-reference` |
| SMIL (attribute layer) | SVG Animations Level 2 ED, SMIL Animation; timing anchored by the animation spec | WPT `svg/animations` |
| Processing modes | SVG 2 §2.2–2.3, SVG Integration | SV-13 probes |
| Hit testing | SVG 2 §15.5–15.6, CSS Masking | WPT `svg/interact` |
| Accessibility | SVG-AAM 1.0, Graphics-AAM | WPT `svg-aam/` (AX-1) |

## 8. Targets

### 8.1 Correctness

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **SV-1** | **WPT SVG conformance** | Per `svg/` subdirectory at Stage 1 exit, pass rate ≥ max(**F**, best shipping engine), F = 95%† for `types`, `geometry`, `coordinate-systems`, `shapes`, `pservers`, `struct`, `embedded`, `interact`; 90%† for `animations`, `painting`, `styling`, `linking`, `path`; 85%† for `text`; directories of held features excluded and listed; non-decreasing afterwards (C-1) | `hg-wpt-svg` in T1 | C-1's failure action |
| **SV-2** | **Resources conformance** (with GR) | `css/css-masking` and `css/filter-effects` ≥ best shipping engine at Stage 1 exit | `hg-wpt-svg` | C-1 |
| **SV-3** | **Geometry is exact, and hit testing is the geometry** | Against `hg-svg-reference`'s interval-certified results on a 10⁶†-case generated corpus: |engine − exact| ≤ 2⁻⁴⁰† × (path extent + 1) for lengths and points; bit-exact `getCTM`, `getScreenCTM` and fill containment outside a 2⁻⁴⁰†-wide band around edges; hit-test results equal `isPointInFill`/`isPointInStroke` on 100% of 10⁶† generated points | T2 | Fix the engine or the reference; never widen a bound without a recorded reason |
| **SV-4** | **Text positions are the corrected algorithm's** | Per-character positions equal `hg-svg-reference`'s on the `svg/text` corpus and a generated corpus (lists, ligatures, bidi, `textLength`, `textPath` on open and closed paths) | T2 | Fix |
| **SV-5** | **SMIL attribute values and motion are exact** | Animated attribute values (path data, `points`, lists, `viewBox`) and `animateMotion` positions and tangents equal `hg-svg-reference`'s on a generated corpus, given `hg-anim`'s sampled progress (interval correctness is the animation spec's target) | T2 | Fix |
| **SV-6** | **One parser per grammar** | 0 disagreements between `hg-svg-syntax` and every other caller's check across fuzzing (G121) | T1 differential | Remove the divergent check |
| **SV-7** | **One engine, inline and as an image** | The same SVG, including `foreignObject` HTML, rendered inline with script off and as an `<img>` at the same size produces identical rasters on 100% of the corpus, except the documented mode differences | T2 | Fix the divergent path |
| **SV-8** | **Instances behave as specified** | Event retargeting, listener mirroring (adding and removing listeners on the source during dispatch), `:hover` per instance, read-only shadow root, and no custom-element construction or navigable creation during a flush: 100% of a generated suite beyond WPT | T2; DM-10's planted-bug suite | Fix |

### 8.2 Security

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **SV-10** | **Memory safety and no recursion** | 0 `unsafe` in SVG crates; 0 recursive functions over content depth outside allowlisted bounded recursions | G128, G123 lints | Build fails |
| **SV-11** | **Amplification is bounded** | A generated document at 100% of every ledger limit and structural cap, and every document in the amplification corpus (nested `use`, template chains, pattern recursion, filter chains and regions, marker storms, path bombs, SMIL graphs, nested documents), completes style, layout and one frame within **250 ms†** and **64 MiB†** above baseline on the slowest reference device, and every exceedance produces the specified result and an IIP event | T2; DST | Lower the weights or caps, or fix the algorithm |
| **SV-12** | **Scriptless closure** | 0 crates of G120's forbidden set in the dependency closure of `hg-static-doc`, `hg-svg-image` and the media process binary; `StoreWriter` constructed only in `hg-dom` and `hg-static-doc` (compile-fail test); checked from SV-M0 on a skeleton build | `hg-graph-lint` | Ship blocker |
| **SV-13** | **Secure modes hold** | Probes over every image, resource, glyph and local-file context: 0 script executions, 0 network requests other than the document itself (and, for local files, the granted directory), 0 reads of user state (`:visited`, autofill, focus) | T2 probe | Ship blocker |
| **SV-14** | **Fuzzing** | ≥ 1,000† CPU-hours per week across grammar, structure and amplification fuzzers, on x86-64, arm64 and riscv64 builds; 0 open crash, panic, arithmetic-trap or hang bugs older than 14† days | Fuzz farm | Release held |
| **SV-15** | **Animated-URL XSS closed** | 0 navigations or script executions on a corpus of animated-`href` and `set` vectors, including sanitizer-bypass corpora ([SR 4.2–4.3][SR]) | T2 | Ship blocker |
| **SV-16** | **No cross-origin pixels through SVG effects** (with GR) | 0 bits recovered above noise by GR-14's probes for SVG filters, masks and blends over opaque images (`<image>`, `feImage`, CSS images in `foreignObject`) and framed pages, CPU and GPU, compression on and off | Lab, exclusive rig booking, joint with GR-14 | Ship blocker |
| **SV-17** | **SVGZ cannot bomb** | Gzip-magic and `.svgz` decoding stays within NW's decompression budget (6.3) on a bomb corpus; 0 bodies decoded outside the partition service | T2 | Ship blocker |
| **SV-18** | **Local SVG runs no script until the user acts** | A local `.svg` with `<script>` and event handlers executes 0 script and makes 0 network requests until "run with script" | T2 probe | Ship blocker |

### 8.3 Privacy

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **SV-20** | **Images carry no user state** | 0 raster differences between profiles that differ in history, autofill, focus, spelling dictionaries, OS accent and system colours, installed OS fonts outside TX's visible set, and locale, beyond the values the environment snapshot already exposes to script (each listed with its D-S1 status); joins ST-13 and PRIV-2 | T2 (SV-S3) | Ship blocker; SV-D10 stays tainting |
| **SV-21** | **No cross-partition sharing** | 0 static documents, resource documents or raster-cache entries reachable across fetch partitions; a no-cors and a CORS load of one URL never share a document (FP-36 extended; G56 probe with a server that varies on `Origin`) | T2 probe | Ship blocker |

### 8.4 Determinism

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **SV-30** | **Cross-ISA, cross-OS identity** | Render-tree hashes, geometry API results, interpolated values and SMIL samples bit-identical across x86-64, arm64 and riscv64 and macOS, Linux and Windows on the corpus and on a special-value set (signed zeros, NaN, ±∞, subnormals, exact multiples of 90°); 1,000 of 1,000 replays identical (DET-1) | CI matrix, riscv64 on JS-D22's pool; replay farm | SV-D2's reversal trigger |
| **SV-31** | **Placement does not change pixels** | SVG image rasters made in the renderer and in the media process bit-identical on 100% of the image corpus | T2 | Ship blocker |
| **SV-32** | **Incremental equals from scratch** | Render tree, instance trees, spatial index, reference graph and ledger after every mutation in the corpus equal a from-scratch build, including insert-at-front after ledger exhaustion (G127) | T1 differential, DST | Fix |

### 8.5 Performance, energy and footprint

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **SV-40** | **SVG images start fast** | Parse-to-first-raster of a 20 KB† icon: warm (process and style scope ready) ≤ **2 ms†** p50 and ≤ **5 ms†** p95; cold media process ≤ **15 ms†** p95; slowest reference device, both placements | Lab | Host or pipeline fix |
| **SV-41** | **Large charts stay interactive** | A 10,000†-element D3-style chart: first frame ≤ **150 ms†**; a hover restyle of one element ≤ **1 frame** at 60 Hz; zoom (`viewBox` change) ≤ **2 frames** | Lab, against SV-S2's black-box baselines | Profile and fix the dominant per-element cost |
| **SV-42** | **Hover hit tests are cheap** | ≤ **50 µs†** p95 per hit test in the 10,000-element chart | Lab | Spatial-index fix |
| **SV-43** | **Re-raster is quick** | Re-raster of every visible SVG image after a zoom or DPR change ≤ **2 frames†** p95 (warm), both placements, slowest reference device | Lab | With MD |
| **SV-44** | **SMIL is cheap and on time** | Sampling 1,000† simple SMIL animations ≤ **1 ms†** per frame; SMIL event timing identical with and without the fragment on screen | Lab; T2 | Fix |
| **SV-45** | **Hidden SVG animation costs nothing** | 0 rasters and 0 samples for off-screen or hidden animated SVG images and for unobserved off-screen fragments; a visible animated SVG image's energy within FP-22's regression rule on the energy corpus | Lab; FP-17's wakeup counter | Fix |
| **SV-50** | **Compact records** | Render record ≤ **64 B†** average per graphics element on the corpus (in FP-11's layout share); each path stored once per partition | Canonical bytes in CI | Field audit |
| **SV-51** | **Static documents are small** | A static document ≤ **48 KiB†** base (style scope included) plus FP-11's per-node bytes; **0** agent heaps for SVG images | Canonical bytes; lab | Host fix |
| **SV-52** | **One document per key** | N uses of one static-document key (URL, partition, request mode, credentials mode, response) in one process → 1 static document | T2 | Fix |
| **SV-53** | **The media process stays within its budget** with the SVG stack linked | FP-38 idle; MD-53 while rasterizing; the per-process cap of 6.13 | Lab (SV-S1 with MD-S10) | MD-D20's reversal |

### 8.6 Accessibility and observability

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **SV-60** | **SVG text is text** | 100% of SVG `<text>` characters on the corpus findable with find-in-page and selectable and copyable, including in local files opened in the scriptless interactive mode; per-character bounds in the semantic tree | T2 | Fix |
| **SV-61** | **SVG-AAM** | AX-1's `svg-aam` floor | AX | AX |
| **SV-62** | **Every SVG decision is explainable** | 100% of ledger and cap events, invalid references, instance rebuilds and SMIL interval changes visible through `svg.*` with a cause | T2 | Fix |
| **SV-63** | **Observability is cheap** | ≤ 1%† of frame time with probes on; 0 cost off | Lab | Fix |

## 9. Milestones and staffing

### 9.1 What is staged, and why

| Item | Reason for staging | Kind |
|---|---|---|
| Non-tainting SVG images with `foreignObject` (SV-D10) | Until SV-S3 proves the host carries no user state beyond what script already sees | Security |
| Opaque SVG images in the media process (MD-D20) | Until SV-S1 and MD-S10 measure start-up, memory and re-raster latency | Measurement |
| `inline-size`, `vector-effect` values beyond `non-scaling-stroke`, `textPath` `method="stretch"` and `spacing="auto"`, `<discard>`, link parameters, cross-origin `use`, multiple `title` and `desc` for localisation, nested links | Not admitted (SV-D13): no shipping engine, at risk in the CR, deferred, or proposed for removal by an open WG issue | Compatibility (P2) |
| Budget weights and caps | Set from first principles, checked against the corpus at SV-M0 | Measurement |

The scriptless default for local SVG files (SV-D9) is **not** staged: it ships from first release, and SV-S5 can only reverse it. Nothing else is staged: the SVG DOM, SMIL, text, resources, hit testing and the semantic source are built from SV-M0.

### 9.2 Milestones

| ID | Milestone | Exit criterion (demo plus measurement) | Depends on |
|---|---|---|---|
| **SV-M0** | Foundations (month 9†) | `.hgsvg` and `hg-svggen` with the pinned indexes and IDL (CR snapshot re-checked against the ED's changes list); `hg-svg-syntax` with fuzzers and its differential; `hg-node-store` with `StoreWriter` agreed with the JS and DOM owners and measured; style scopes agreed with ST; **SV-12 on a skeleton build** of `hg-static-doc` + `hg-style` + `hg-layout` + `hg-text`; **SV-S1** (static-host cost, joined with MD-S10); **SV-S4** (cross-ISA identity of G122's discipline on all three ISAs, riscv64 on JS-D22's pool); **SV-S5** (black-box measurements: engines' limits, image timelines, `animVal`, refused animation targets, SMIL and Web Animations order, `use` shadow-root mode, exact trigonometry at 90°, the local-file corpus); `hg-svg-reference` for viewports, shapes and paths; corrected text algorithm filed with the WG; the corpus fixed | JS-M0 (`hg-heap`), DM-M0, ST-M0 |
| **SV-M1** | First SVG (with GR-M1, LY-M1) | Shapes, paths, viewports, units, transforms, `use` with the ledger, paint servers, the render tree into `hg-svg-paint`; inline SVG and same-origin SVG images in the renderer; local files in the scriptless interactive mode; the SVG DOM core; SV-3, SV-6, SV-8, SV-10, SV-30, SV-32 at threshold for this scope | GR-M1, LY-M1, TX-M1 |
| **SV-M2** | SVG complete (with GR-M2, 30–42 months†) | Text and `textPath`; markers; clip, mask and filter resolution for SVG and HTML with taint flags; SMIL with the animation spec's effect model and `hg-svg-interp`; `hg-html-render` with DM; opaque SVG images in the media process if SV-S1 passes; hit testing with the spatial index; `SvgSemanticSource`; IIP `svg.*`; the full ledger; **SV-S2** (element-count scaling against the three engines) and **SV-S3** (no-user-state proof suite) | GR-M2, MD-M2, the animation spec's effect model, AX-M2 |
| **SV-M3** | Stage 1 exit | Every SV target at its Stage 1 threshold; external security review of the SVG crates, the scriptless host and the opaque-image protocol | Stage 1 M8 |
| **SV-M4** | Admissions (continuous) | Each held feature ships when it meets the admission rule | — |

### 9.3 Staffing and duration

No public source gives a team size for a browser's SVG engine. The estimate follows the earlier specs' method: a **component inventory** in thousands of lines of production Rust†, excluding generated code and tests, at **3–4 KLOC† of verified, reviewed code per engineer-year**.

| Component | Production KLOC† | Non-code work |
|---|---|---|
| `hg-svg-syntax` and grammar fuzzers | 4–6 | — |
| `.hgsvg`, `hg-svggen`, `hg-svg-model`, presentational mapping | 5–7 | Restating the indexes |
| `hg-static-doc`, `StaticSink`, regions, `ArenaBarrier`, loaders (the store move itself is counted in JS 9.3's node-space row) | 4–6 | Review by the JS and DOM owners |
| Style scopes and the weighted ledger | 3–5 | With ST and LY |
| `hg-svg-instances` (builder, regions, listener dispatch from the source, cloning animation elements) | 4–6 | With DM |
| `hg-html-render` (shared with DM; counted here because the static host needs it) | 4–6 | With DM |
| `hg-svg-dom` (60 interfaces, lists, reflection; bindings generated) | 12–18 | — |
| `hg-svg-layout`: viewports, geometry, markers, resource resolution, reference graph, render tree, invalidation, work list | 25–35 | — |
| SVG text: corrected algorithm, `textPath`, text DOM, find and selection glue | 10–14 | WG issues |
| `hg-svg-geom` | 6–9 | Numerical review |
| `hg-svg-interp` | 3–5 | With the animation spec |
| SMIL element and attribute layer, SMIL DOM over `SmilControl`, `animateMotion` geometry (timing is the animation spec's, AN-D6) | 3–5 | With the animation spec |
| `hg-svg-image`, raster cache, opaque-image protocol, image timelines, glyph profile | 6–9 | With MD and TX |
| Hit testing, spatial index, `SvgSemanticSource` | 5–7 | With AX |
| `hg-svg-reference` (with its own rational and interval arithmetic) | 17–24 | Independent authorship; numerical specialist |
| Amplification corpus, fuzzing, WPT and black-box harnesses | 5–8 | Corpus handling |
| IIP `svg.*` and overlays | 5–8 | With the DevTools track |
| **Total** | **121–178** | |

That is 30–59 engineer-years† of code. Over the roughly four years from SV-M0 to Stage 1 exit, adding review, security response, black-box measurement and standards work:

| Track | Scope | Engineers† |
|---|---|---|
| Model, DOM and host | Grammars, model, static host, style scopes, instances, `hg-html-render`, SVG DOM | 4–5 |
| Layout and geometry | `hg-svg-layout`, `hg-svg-geom`, hit testing, semantic source | 3–4 |
| Text | 6.10 | 2 |
| SMIL attributes and interpolation | `hg-svg-interp`, the attribute layer, SMIL DOM, with the animation spec | 1 |
| Images and security | `hg-svg-image`, ledger, protocol, fuzzing, corpus, threat model, a share of the response rota | 2 |
| Reference implementation | `hg-svg-reference`, kept independent | 2 |
| Observability | IIP `svg.*`, with the DevTools track | 1 |
| Standards | SVG WG participation, issues for the text algorithm, hit-test, coefficient and `use`-mode gaps, tests contributed to WPT | 1 |
| **Total** | | **16–18†** |

**Overlap with other specs, line by line.** The graphics spec counts `hg-svg-paint` (3–4†) and every raster and filter kernel; the layout spec counts the MathML and SVG boundary (2†, shared with MathML: outer `<svg>`, `foreignObject`, the inline call); the media spec counts the opaque-image host glue inside its image-pipeline track; the accessibility spec counts the SVG-AAM mappings; the animation spec counts the effect model, SMIL timing (AN-D6) and the extensions of 6.9; the DevTools track counts the frontend views; the JS spec counts the node-store move inside its node-space row. `hg-html-render` is counted here, not in DM. No engineer is counted twice. Reconciliation 2 listed SVG as uncounted ([RN2 Section 4][RN2]); this spec adds **16–18 engineers†** to the running total. **Duration**: SV-M2 follows GR-M2 (30–42 months†); SV-M3 is Stage 1 exit (48–60 months†, set by layout). SVG is not on the critical path if SV-M0 starts with the JS, DOM and style M0s. The schedule risk is hiring people who know SVG's text, filter and SMIL semantics, and a numerical specialist for the reference.

### 9.4 Decisions for the owner

A bare "yes" means the recommendation. Build-versus-reuse is not on this list: the standing answer is "build our own" (M-1, D-S10). **The owner most needs to see SV-D1, SV-D2, SV-D4, SV-D6 and SV-D9.**

| ID | Decision | Options | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|---|
| **★ SV-D1** | Scriptless SVG documents (answers Q-MD-18, RN2 Q25) | (a) shared `hg-node-store` with typed `StoreWriter` capabilities, a scriptless arena host with its own style scope, `hg-html-render` shared with DM; (b) full DOM with an idle agent heap per image; (c) a separate SVG-only tree | **(a)** (6.1) | A crate move and a sealed barrier trait in the JS and DOM area; style scopes in ST and LY; `hg-html-render` | The move costs > 2%† on DOM benchmarks: a shared column-type crate with a differential instead; a reversal of JS-D4 reverses this too |
| **★ SV-D2** | Numbers for SVG geometry | (a) IEEE-strict binary64 (G122's operation list), `hg-math` with degree-native trigonometry, binary64 computed lengths in style, integers from one boundary function, per-path exponent with an f64 fallback; (b) Q32.32 fixed point throughout; (c) f32 | **(a)** (6.6) | Lints; ST, GR and `hg-math` amendments | SV-30 finds a cross-ISA mismatch that cannot be pinned: (b) with a block exponent per user space |
| SV-D3 | Where SVG geometry lives | (a) an SVG layout pass producing `SvgRenderTree`, one work list with `foreignObject` roots; (b) SVG elements as CSS boxes | **(a)** (6.4) | — | — |
| **★ SV-D4** | SMIL | (a) timing graph in an SVG crate; (b) timing in the animation spec's `hg-anim` (AN-D6), with elements, attributes, geometry, `hg-svg-interp` and the animated-attribute overlay here; (c) a separate SMIL engine; (d) no SMIL | **(b)** (6.9), as AN-D6 decided | The requests of R10 | — (the two specs agree; R10's open items are settled with the animation spec's owner) |
| SV-D5 | `<use>` and templates | (a) materialized instances by a dedicated builder (not DOM's clone), read-only shadow root with the mode as one flag, regions, listener dispatch from the source; same-origin `use` only; (b) virtual instances; (c) clone at paint | **(a)** (6.7) | Nodes per instance, bounded | — |
| **★ SV-D6** | Resource budgets | (a) one weighted ledger in canonical units (64 MiB†, 2²⁸† work units), structural depth caps, a 256 MiB† per-process cap, maintained as a prefix sum over document order; (b) independent per-resource caps; (c) no limits | **(a)** (6.13) | Weights to calibrate | Corpus documents exceed the ledger: re-weight by measurement, never remove it |
| SV-D7 | SVG fonts | (a) not supported; (b) supported as Safari does | **(a)**: removed from SVG 2, never in Firefox, use ≈ 0 ([US 2.11, 4.25][US]) | Safari-only content falls back to CSS fonts | Measured use above 0.01%† |
| SV-D8 | SVGZ | (a) `Content-Encoding`, plus gzip-magic `image/svg+xml` bodies and local `.svgz`, decoded in the partition service under NW's budget; (b) `Content-Encoding` only | **(a)** (6.3) | One deviation, recorded | A security finding in the magic-number path |
| **★ SV-D9** | Top-level SVG from local files or downloads | (a) open in Izcal's scriptless interactive mode (links, selection, find, same-directory files, no script, no network) with a one-action "run with script"; (b) secure animated mode; (c) open as a page with script, as today | **(a)** (6.13), from first release | A mode, a bar, a broker grant | SV-S5 finds > 1%† of real local SVG files need script |
| SV-D10 | `foreignObject` in images and canvas taint | (a) non-tainting for same-origin and `data:` images once SV-S3 passes; taint until then; (b) always taint | **(a)** (6.8) | The proof suite; a GR 6.15 change | Any SV-20 difference: (b) |
| SV-D11 | External resource references | (a) the specification's fetch modes: CSS references in CORS anonymous mode, `use` same-origin, animation `href` same-document; resource documents secure static; documents keyed by request mode; (b) same-origin only for all | **(a)** (6.8) | — | — |
| SV-D12 | SVG images | (a) raster at the concrete size in device pixels, one static document per key, renderer or media process with identical bits, the opaque-image protocol, `data:` fonts in opaque images through an MD-D20 amendment; (b) vector embedding in the page's display list | **(a)** (6.8) | Re-raster on zoom | MD-D20's reversal trigger |
| SV-D13 | Admission | Extend ST-D18's rule to SVG elements, attributes and DOM members, including CR features that are at risk, unimplemented in all engines, or proposed for removal by an open WG issue: hold `inline-size`, extra `vector-effect` values, `stretch` and `auto` on `textPath`, `<discard>`, link parameters, cross-origin `use`, multiple `title`/`desc`, nested links; admit `getPathData` by exception (one engine shipping, one prototyping, Interop 2027 proposal) | **As stated** | — | Each held feature meets the rule |
| SV-D14 | SVG DOM surface | (a) SVG 2 plus legacy members still reached, `getPathData`, no `SVGPathSeg`, `animVal` as SVG 2's alias; (b) add `SVGPathSeg` as Safari has | **(a)** (6.2) | — | Compatibility failures traced to `SVGPathSeg`; animated `animVal` if SV-S5 finds WPT and corpus dependence |
| SV-D15 | Hit testing | (a) hit testing is `isPointInFill`/`isPointInStroke` in user space, dash gaps do not hit, markers never hit, a spatial index in the render tree above 256† records; (b) separate hit geometry | **(a)** (6.11) | — | — |
| SV-D16 | Reduced motion | (a) under the user's setting, SVG images show the t = 0 sample paused with a play control, and inline SMIL pauses at t = 0 with a page-level play control (as `marquee`); (b) images only | **(a)** (6.9) | — | Breakage above 1%† on the SMIL corpus with pausing on |

### 9.5 Amendments this spec proposes to other documents (not applied)

| ID | Document | Amendment | Owner |
|---|---|---|---|
| R1 | JavaScript engine spec (JS-D4, 6.2 H-2, G6) | Columns, attribute-set references, text arena, untraced rare data and the `hg-tree-traits` implementation move to `hg-node-store` (L3, no `hg-heap`); H-2's "written only by `hg-dom`" becomes "written only through `StoreWriter<TracedBarrier>`, constructed only in `hg-dom`"; the text arena's release stays in `hg-dom`'s release step; a reversal of JS-D4 to "node in the cage" also reverses SV-D1 | JS owner, with DM |
| R2 | JavaScript engine spec 6.9 | An SVG row in the consolidated decision-point list (6.15) | JS owner |
| R3 | JavaScript engine spec (`hg-math`) | Degree-native `sin_deg`, `cos_deg`, `tan_deg` exact at multiples of 90°, after SV-S5's black-box measurement | JS owner |
| R4 | DOM spec (6.1, 6.2, 6.5, 6.6, 6.12, 6.17, 6.18, G91, navigation) | Instance building as a core-phase step outside DOM's clone algorithm; listener dispatch on instances from the source's list; `use` shadow-root reads resolve pending builds for that root; the shadow root's mode as one flag; `hg-html-render` (L3) factored out of `hg-html`; the SVG rows of 6.12 and 6.18 point to `hg-svg-dom` and `hg-svg-instances`, and SMIL to the animation spec's `hg-anim`; the navigation path opens local SVG in the scriptless interactive mode (SV-D9) and blocks top-level `data:` navigation from content with no SVG exemption; a planted-bug case in DM-10's suite | DM owner |
| R5 | Style spec (6.2, 6.5, 6.6a, 6.8, 6.9, 6.11, 6.15, G23, G24) | `StaticSink`; computed-length type fixed as **binary64**; per-agent tables become per **style scope** (agent or static document); `ResourceLoader` traits for sheet and font loads; presentational hints for SVG from `hg-svg-model`; "affects" categories svg-geometry and svg-resource; overlay values visible through `hg-tree-traits`' attribute accessors; `hg-svg-syntax`, `hg-svg-model`, `hg-svg-geom` and `hg-svg-interp` in L2p under G24; G23's one-parser rule covers SVG micro-grammars | ST owner |
| R6 | Layout spec (6.0, 6.1, 6.5.10, 6.12, 6.13, 6.14, G1, G39, G43) | 6.0: the media process links layout crates through the scriptless host, Q-MD-18 answered; forcing-table entries for SVG geometry methods and `use` shadow-root reads; `SvgGeometry` converts through `hg-layout-num`'s rounding; G39 records that SVG crates follow G122 instead; SVG roots and `foreignObject` roots as one work list merged under G43; fragment store per style scope; G1's list includes `hg-svg-layout`; the chunk-to-SVG-record map | LY owner |
| R7 | Graphics spec (6.3, 6.4, 6.7, 6.15, 6.16, 6.28, `hg-paint-order`) | Per-path binary exponent (|e| ≤ 100†) chosen from path and CTM, with the f64-to-device fallback; one f64-composed CTM per SVG record relative to the nearest property-tree node; per-record taint flags and transparent-black substitution in SVG filters, masks and blends, recorded as a deviation (SV-16 joins GR-14); canvas origin-clean for same-origin and `data:` SVG images with `foreignObject` once SV-S3 passes; display items reference paths by content hash; HTML elements' `url()` resources through the same resolver; filter resolution never reduced; a choice of mask luminance coefficients filed with the CSS WG (Q-SV-5); 6.28's record list replaced by 6.4 and 6.5's | GR owner |
| R8 | Images and media spec (MD-D20, MD-D19, MD-S10, Q-MD-18, G53) | Q-MD-18 answered (6.1); MD-S10 joined with SV-S1; `data:` fonts declared in opaque SVG images, parsed in the media process under TX's fuel; the opaque-image raster protocol with no next-change or environment-read signal to the renderer; an SVG re-raster target separate from MD-56 (SV-43); the L2m profile for SVG crates in the media process; static-document keys include request and credentials mode; under reduce-motion, SVG images show the t = 0 sample paused | MD owner |
| R9 | Fonts and text spec (TX 6.3) | No G26 change is needed (RN2 Q25); the OT-SVG glyph mode is the scriptless host's secure static mode with TX's limits, following OpenType 1.9.1's SVG 1.1 subset ([SS 7.7][SS]); OT-SVG glyph documents may not use OT-SVG fonts; `ResourceLoader` for font loads in static hosts; Q-TX-9 answered: the table makes animation optional and TX disables it | TX owner |
| R10 | Animation spec (draft 0.1, AN-D6, 6.8, 6.14) | Agreed split: SMIL timing in `hg-anim` (AN-D6), SVG's elements, attributes and geometry here. Requests: attribute effects with interpolation through `hg-svg-interp` or `hg-svg-model`'s types; "effects on XML attributes write `animVal` only" becomes "write the animated-attribute overlay, read by every rendering consumer through `hg-tree-traits`; `animVal` per SV-D14"; iteration composite `accumulate`; paced offsets supplied by this spec; the `InterpolateValue` trait (one function for CSS `d`/`transform` and SMIL); the time type in `hg-types`; `SmilControl` implemented by `hg-anim`; SMIL event timing unaffected by off-screen throttling when observed; image timelines per (static document, embedding document); inline SMIL under reduce-motion (SV-D16, its motion levels); the SMIL-versus-Web-Animations order checked black-box against its recorded choice | Animation spec author |
| R11 | Resource-footprint spec (FP-11, FP-36, 8.2) | SVG render records in FP-11's layout share; path geometry as an excluded category with its own ledger line; a static-document line (SV-51) including its style scope; a per-process cap for scriptless documents (256 MiB†); SV-21 joins FP-36's probes | FP owner |
| R12 | Accessibility spec (AX 6.2, G100) | `SvgSemanticSource` (6.12); the semantic tree for a top-level scriptless document; inline SMIL under reduce-motion treated like `marquee`; the forced-colours UA rule for SVG recorded (Q-SV-9) | AX owner |
| R13 | Networking spec (NW-D6) | Gzip-magic `image/svg+xml` bodies and local `.svgz` decoded in the partition service under NW's content-coding budget; NW states the budget's ratio and cap (this spec proposes 1:1,024† and 64 MiB†) | NW owner |
| R14 | Shell UI design | The SV-D9 bar and "run with script" action; play controls for SVG images and inline SMIL under reduce-motion; `title` tooltips | Shell owner |
| R15 | Stage 1 design (2.1, 2.2, G9); rule registry | Crates of 6.18 in L2, L2p, L3, L4 and tools (`hg-svg-instances` and `hg-html-render` in L3; the four L2p crates); G9's list gains the SVG crates; rules **G120–G129** registered after the PDF spec's G110–G119 | Stage 1 owner |
| R16 | DevTools spec | The `svg.*` domain and its overlays (6.17); `svg.imageInfo` for opaque images from the media process through the broker | DevTools owner |
| R17 | Security scope | Threat rows: the static host as a shared parse across agents of one partition; the opaque-image protocol's channels; the scriptless interactive mode's `file:` directory grant | Security owner |
| R18 | Phase 3 ledger; reconciliation 2 Section 4 | G-12 closes when R1, R4, R5 and R10 are agreed; A-FUT-1 and A-FUT-5 satisfied by 6.4, 6.5 and 6.10; SVG staffing 16–18† added to the running total; the animation extensions recorded under G-13 | Ledger and reconciliation owners |

## 10. Risks

| Risk | Effect | Mitigation |
|---|---|---|
| The node-store move, `StoreWriter` or style scopes meet resistance or cost the DOM and style | SV-D1 delayed | The reversal design (a shared column-type crate) keeps the scriptless host possible; style scopes are a type change, not a new algorithm |
| The binary64 discipline leaks a cross-ISA difference (a compiler change, a disallowed method slipping in, style choosing binary32) | DET-1 fails for SVG | G122's lint list, SV-S4 at SV-M0, SV-30 on all three ISAs; R5's binary64 request; reversal to fixed point |
| The animation spec's owner declines attribute effects or the scriptless core | SMIL needs its own effect path | Then SV-D4 (c) within this spec, still with `hg-svg-interp` as the one interpolation |
| Ledger weights cut legitimate content | Broken pages | Corpus check at SV-M0, IIP events naming the charge, re-weight by measurement |
| The SVG 2 text moves after the CR (the `use` shadow root went from open to closed in a week) | Rework, interop surprises | Pinned snapshots, re-checked against the ED's changes list at SV-M0; the mode as one flag; WG participation |
| Interop: Izcal follows the specification where engines agree on something else (the SVG effect taint substitution is one) | Pages break | Black-box comparison on every disagreement; deviations recorded where web compatibility requires it (P2 outranks literalism) |
| SV-D9 surprises users who expect scripts in local SVG | Support load | One action restores script; SV-S5 decides |
| `hg-html-render` is not factored | `foreignObject` in images renders differently from inline | SV-7 is a ship-blocking-level test; the factoring is in R4 |
| Hiring | Schedule | Few SVG specialists exist outside the engines; the reference implementation doubles as training |

## 11. Open questions and flagged claims

### 11.1 Open questions

- **Q-SV-1.** The corrected SVG 2 text algorithm: each of the five defects ([RG 1.5][RG]) fixed in Izcal's anchors, checked black-box, and filed with the SVG WG (SV-M0).
- **Q-SV-2.** What limits do the three engines apply to `use` expansion and depth, template chains, pattern recursion, filter regions and costs, and path segments, and what happens at each? (SV-S5.)
- **Q-SV-3.** Do the ledger's weights and caps (6.13) cut any document in the corpus?
- **Q-SV-4.** When does each engine start an SVG image's SMIL timeline, and do two `<img>` of one URL share it? ([AN 3.9][AN].)
- **Q-SV-5.** Which mask luminance coefficients: SVG 1.1's (0.2125, 0.7154, 0.0721) through CSS Masking, or Filter Effects 1's Rec. 709 (0.2126, 0.7152, 0.0722)? Filed with the CSS WG; graphics decides ([RG 4.5][RG]).
- **Q-SV-6.** Do WPT `svg/animations` and the corpus depend on `animVal` returning animated values, against SVG 2's alias? (SV-D14's last clause.)
- **Q-SV-7.** On one property, do SMIL effects sort below or above Web Animations and CSS Animations in the three engines? The animation spec records Izcal's choice (after CSS animations, before script animations, AN 6.8); this checks it black-box.
- **Q-SV-8.** Which `attributeName` targets do the engines refuse to animate? ([AN open questions][AN].)
- **Q-SV-9.** The forced-colours rule for SVG: Chromium's UA-sheet exemption or Firefox's forcing of `currentColor`, against CSS Color Adjust's text ([GO 2.6][GO]).
- **Q-SV-10.** The hit-test rules SVG 2 leaves open (dash gaps, clip with `bounding-box`), written in 6.11, filed with the WG ([RG 7.5][RG]).
- **Q-SV-11.** Do the engines support SVG 2's media fragments (`xywh=`, `t=`) and `svgView()`? Admission of each depends on it ([SS 3.5][SS]).
- **Q-SV-12.** Why did `foreignObject` use rise from 0.32% to 8.24% of page loads in 2025–2026 ([US 2.5][US])? It decides how much weight SV-D10 carries.
- **Q-SV-13.** What share of real local `.svg` files need script, network, links or sibling files? (SV-S5, SV-D9.)
- **Q-SV-14.** The composition order of `animateMotion`'s supplemental transform with `transform` and CSS Motion Path, measured on WPT's `animateMotion` cases ([AN open questions][AN]).
- **Q-SV-15.** OpenType 1.9.1 discourages `context-fill` in SVG glyphs while SVG 2's font UA sheet uses it ([SS 7.7][SS]): which governs OT-SVG glyphs in Izcal? (With TX.)
- **Q-SV-16.** Which licence covers resvg's regression suite, before it is used as test input (Section 7)?
- **Q-SV-17.** Do the three engines return exact 0 from `getCTM` after `rotate(90)`? (Decides R3.)
- **Q-SV-18.** Which `use` shadow-root mode ships: the CR's open and read-only, or the ED's closed? Measured against the three engines and the snapshot pinned at SV-M1 (6.7).

### 11.2 Claims that are single-source, unverified or conflicting

- **Conflicting**: SVG 2 CR (6 October 2026: `use` shadow tree open and read-only) against the Editor's Draft (13 October 2026: closed) ([FK][FK], FC-M1; Q-SV-18).
- **Conflicting**: SVG 2 (secure static) and SVG Integration (secure animated) on resource documents' processing mode ([SS 1.6, 1.8][SS]); Izcal follows SVG 2.
- **Conflicting**: mask luminance coefficients between CSS Masking (via SVG 1.1) and Filter Effects 1 ([RG 4.5][RG]).
- **Single-source**: the SMIL deprecation's suspension rests on a post forwarded to www-svg on 17 August 2016; the original was not read ([AN 1.10][AN]).
- **Discontinuous data**: Chrome's SMIL counter jumps in 2015–2017; the growth claim uses January 2019 onward ([AN 6.1][AN]).
- **Inferred, Low**: the bug classes behind `use`-instance and resource-client failures ([SR 1.7][SR]); the list of user-state sources behind `foreignObject` taint ([SR 5.5][SR]); that no inspector shows `use` provenance, resolved paint servers or filter regions ([GO 1.5–1.6][GO]).
- **Medium**: arc-length differences between engines of about 0.1% to 0.9% rest on a bug report and a WG thread read as snippets ([RG 2.5][RG]); per-element costs dominating large SVG ([RG 5.2–5.3][RG]).
- **Unknown cause**: the 2025–2026 rise of `foreignObject` use ([US 2.5][US]); the exact trigger of Chrome's `ExternalSVGDocumentResources` counter ([US 2.7][US]).
- **Not used**: claims that Blink moved SVG text to LayoutNG in 2021 rest only on secondary summaries derived from source ([HP 3.10][HP]; [RG 1.16][RG]).
- **Unverified**: whether Chrome's CSS property counters include SVG presentation attributes ([US][US], Claims not verified).
- **Medium**: the Kotcher et al. 2013 paper body was not read; the CCS 2011 "Crouching Tiger" paper could not be fetched ([SR][SR], Claims not verified).
- **Method**: spec text was checked against the 13 October 2026 Editor's Draft where w3.org could not be downloaded; one difference was found (the `use` mode) ([FK][FK]).

## Appendix A. Review disposition (revision 0.2)

Phase 8 was done by an independent [fact-checker][FK] and an independent [design critic][CR], neither of whom wrote the spec or its notes. Every item was applied; several in a different form than proposed (marked).

**Fact-check, must-fix**

| ID | Item | Disposition | Where |
|---|---|---|---|
| FC-M1 | `use` shadow tree: CR open and read-only, ED closed | Applied: mode as one flag, Q-SV-18, conflict in 11.2, R4 | 6.7, 11 |
| FC-M2 | "Sixty-fold" SMIL growth used the wrong baseline | Applied: 0.0403% (2015 intent), discontinuity stated, growth from 0.58% (2019) | Summary, 2.1, 2.2, 6.9 |
| FC-M3 | TX G26 does not cover the media process | Applied: request moved to MD-D20 (R8); R9 withdraws the G26 change | 6.8, 9.5 |
| FC-M4 | `hg-url`'s rule is NW G31 | Applied | 5, 6.13 |
| FC-M5 | GR-D17 is not filter resolution | Applied: stated as this spec's rule, proposed to GR (R7) | 6.13 |
| FC-M6 | No Izcal spec blocks top-level `data:` navigation | Applied: R4 asks DM | 6.13, 9.5 |
| FC-M7 | SVG 2 section numbers in the anchors table | Applied | 7 |
| FC-M8 | 2025 SVG clickjacking is not a timing attack | Applied: separate threat row under GR-D18 | Summary, 2.2, 5, 6.13 |
| FC-M9 | librsvg caps not tied to CVE-2019-20446, which is nested patterns | Applied | 5, 6.13 |
| FC-M10 | Too few inline primary links | Applied: about 35 inline links to specifications, counters, advisories and posts | Throughout |
| FC-M11 | SVGZ limits presented as NW's | Applied: values set here (†), R13 asks NW | 6.3 |

**Fact-check, recommended**

| ID | Disposition |
|---|---|
| FC-R1 | Applied: changeset and library links removed from the notes; resvg cited by its README file |
| FC-R2 | Applied: dead links replaced by CVE records or cited by title |
| FC-R3 | Applied: marked Low in 6.17 and 11.2 |
| FC-R4 | Applied (KEV date) |
| FC-R5 | Applied: "up to about 37.5 MiB"; FP 6.3 and 6.4; GR-D5; LY 6.1's forcing table (the table sits in LY 6.1, so the critic's reading is used); "shared with MathML"; SV-43 has its own number; ST-D20 and ST's 5,000† cap |
| FC-R6 | Applied: SV-D13 asks the owner to extend the admission rule to SVG |
| FC-R7 | Applied: those two flags dropped |
| FC-R8 | Applied in a different form: the CR is pinned by date in the text and 7; anchors link the undated single-page copy whose fragments the notes verified |
| FC-R9, FC-R10, FC-R11 | Applied |
| FC-R12 | Applied (6.3) |
| FC-R13 | Applied (Low markings in 6.5, 6.8, 6.13) |
| FC-R14 | Applied: unused references removed; the 1,405/1,411 counts stated in 3 |
| FC-R15 | Left to the note's owner (dates in RG); the spec cites the 2026 CR |
| FC-R16 | Applied (6.6) |
| FC-R17 | Applied (6.1 option (b)) |
| FC-R18 | Applied (6.12) |
| FC-R19, FC-R20, FC-R21, FC-R22, FC-R23, FC-R24 | Applied (6.9, 2.1, 6.8, 6.16, SV-M0, 11.2) |

**Design critique**

| ID | Severity | Disposition | Where |
|---|---|---|---|
| C1 | High | Applied: one weighted ledger in canonical units, filter work in CSS pixels per definition, structural caps, a per-process cap; SV-11 restated | 6.13, SV-11, SV-D6 |
| C2 | High | Applied: prefix sum over document order maintained incrementally; budget-loss rule; DST case | 6.7, 6.13, G124, SV-32 |
| C3 | High | Applied: `StoreWriter<B: Barrier>`, `Store<Traced>`/`Store<Arena>`, arena release rules, compile-fail test | 6.1, G120, SV-12, R1 |
| C4 | High | Applied: `hg-svg-instances` (L3), time type in `hg-types`, `SvgGeometryQuery`/`SmilControl` traits, `hg-svg-geom` in L2p | 6.0, 6.2, 6.18, R5, R15 |
| C5 | High | Applied: static documents as style scopes; SV-51 includes them | 6.1, R5, R6, R11 |
| C6 | High | Applied: per-instance regions; overlay through `hg-tree-traits`' accessors for every consumer; secure-mode animatability stated | 6.1, 6.7, 6.9, G126 |
| C7 | High | Applied: `hg-html-render` factored from `hg-html` (R4); fallback rule for browsing-context elements; SV-7 | 6.1, R4 |
| C8 | High | Applied: the builder is not DOM's clone; inert instances; DM-10 case | 6.7, G126, R4 |
| C9 | High | Applied: G122's allowed-operation list, `fmin`/`fmax`, boundary function with NaN and ∞ rules, binary64 computed lengths (R5), special-value cases in SV-30 | 6.6, G122, SV-30 |
| C10 | Medium | Applied: exponent from path and CTM, |e| ≤ 100†, f64 fallback; one f64 CTM per record; property-tree node rule | 6.4, 6.5, R7 |
| C11 | Medium | Applied: per-record taint flags with transparent-black substitution, a recorded deviation; SV-16 replaced by a GR-14 joint target | 6.4, 6.5, 6.13, SV-16 |
| C12 | Medium | Applied: premise corrected (`blob:`), proof list extended with D-S1 status, GR 6.15 amendment | 6.8, SV-20, R7 |
| C13 | Medium | Applied: scriptless interactive mode, a recorded deviation, ships from first release; SV-S5 widened; SV-18 | 6.1, 6.13, 9.1, SV-D9 |
| C14 | Medium | Applied: hit testing is the user-space geometry; spatial index moved into the render tree; SV-3 case | 6.4, 6.11, SV-3 |
| C15 | Medium | Applied: control-polygon tolerance with depth cap; stroker tolerance floor; per-call budget; SV-3's bound stated | 6.6, SV-3 |
| C16 | Medium | Applied: observed fragments keep resolving intervals; catch-up budget; SV-44 case; proposed to the animation spec, which owns frame demand | 6.9, SV-44, R10 |
| C17 | Medium | Applied: opaque-image protocol; DevTools via the broker; SV-43's own number; SV-40 cold and warm; SV-45 | 6.8, 6.17, 8.5 |
| C18 | Medium | Applied: `hg-svg-interp` behind `InterpolateValue`; compositor conversion rule | 6.9, R10 |
| — | — | Synthesis author's own change after phase 8: the animation spec's report appeared with AN-D6 owning SMIL timing; SV-D4 now recommends that split, `hg-smil` is removed, staffing drops to 16–18† | 6.9, 9.3, 9.4 |
| C19 | Medium | Applied: listener dispatch from the source; shadow-root reads resolve that root's builds; forcing table in LY (R6) | 6.7 |
| C20 | Medium | Applied: G120 restated with `ResourceLoader`; SV-12 at SV-M0 | 6.1, G120, 9.2 |
| C21 | Medium | Applied: path stored once per partition by hash; FP layout share and a path-geometry category | 6.16, R11 |
| C22 | Medium | Applied: L2m profile for media-linked crates; panics are crash bugs | 6.13, 6.18, SV-14 |
| C23 | Medium | Applied: the critic's recommendation adopted (t = 0 sample paused; inline SMIL paused like `marquee`); measurable trigger | 6.9, SV-D16 |
| C24 | Medium | Applied: keys include request mode, credentials mode and response; G56 probe | 6.1, SV-21, SV-52 |
| C25 | Medium | Applied: one work list of SVG and `foreignObject` roots merged under LY G43; nesting cap | 6.4, 6.13 |
| C26 | Low | Applied with FC-M3; completeness rule for `data:` subresources | 6.8 |
| C27 | Low | Applied (R2) | 6.15 |
| C28 | Low | Applied: SV-32; Q-SV ids; forcing table in LY; SV-43 | Throughout |
| C29 | Low | Applied: at-risk features named; SV-D13 extension stated | 3, 9.1, SV-D13 |
| C30 | Low | Applied: inventory rows added (style scopes and ledger, instances, `hg-html-render`, `hg-svg-interp`); node-store move counted in JS; total 17–19†, then 16–18† after the SMIL realignment | 9.3 |
| C31 | Low | Applied: SV-7, SV-8, SV-17, SV-18 | 8 |
| C32 | Low | Applied: decoding in the partition service under NW's budget (R13) | 6.3 |
| C33 | Low | Applied: riscv64 in SV-14 and SV-S4 | 8.2, 9.2 |
| C34 | Low | Applied: reference resolutions and nested documents in the ledger and caps | 6.13 |
| C35 | Low | Applied: degree-native trigonometry requested after measurement (R3, Q-SV-17) | 6.6 |
| C36 | Low | Applied: the embedding document's list; on SV-S3's list; PRIV-2 case | 6.14 |
| E1–E3, E7, E8 | — | Compliant; kept | 7 |
| E4 | — | Applied: assets only, comparison only, licence before use | 7 |
| E5 | — | Applied: licence per pinned snapshot recorded at SV-M0 | 7 |
| E6 | — | Applied: reference arithmetic written by Izcal | 7, 6.18 |
| E9 | — | Applied: UI icons Izcal-authored, MIT | 6.1, 7 |
| R1–R18 | — | Applied as amendments proposed in 9.5, not to the other documents | 9.5 |

[H2]: HGBA%202%20browser%20architecture%20spec.md
[H3]: Future%20software%20browser%20engine.md
[S1]: HGBA%20Stage%201%20architecture%20design.md
[JS]: Izcal%20JavaScript%20engine.md
[FP]: Izcal%20resource%20footprint.md
[ST]: Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[TX]: Izcal%20fonts%20and%20text.md
[LY]: Izcal%20layout.md
[MD]: Izcal%20images%20and%20media.md
[GR]: Izcal%20graphics%20and%20rendering.md
[DM]: Izcal%20DOM%2C%20bindings%20and%20event%20loop.md
[AX]: Izcal%20accessibility.md
[PD]: Izcal%20PDF.md
[DT]: Izcal%20DevTools%20spec.md
[RN2]: ../research_notes/Phase%202%20reconciliation/reconciliation_2.md
[LG]: ../research_notes/Phase%203%20amendments/ledger.md
[FR]: ../research_notes/Izcal%20SVG/frame.md
[HP]: ../research_notes/Izcal%20SVG/history_postmortems.md
[SS]: ../research_notes/Izcal%20SVG/specs_standards.md
[RG]: ../research_notes/Izcal%20SVG/rendering_text_geometry.md
[AN]: ../research_notes/Izcal%20SVG/animation_smil.md
[SR]: ../research_notes/Izcal%20SVG/security_robustness.md
[US]: ../research_notes/Izcal%20SVG/usage_interop.md
[GO]: ../research_notes/Izcal%20SVG/gaps_opportunities.md
[FK]: ../research_notes/Izcal%20SVG/factcheck.md
[CR]: ../research_notes/Izcal%20SVG/critique.md
