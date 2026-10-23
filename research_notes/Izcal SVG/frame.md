# Izcal SVG: frame

Written 20 October 2026 (phase 1 of the first-principles-spec method), by the synthesis author.

## The problem and who has it

Izcal has no owner for SVG's document model. The Phase 3 ledger records the gap as **G-12** ("SVG (document model, SMIL, SVG DOM, geometry APIs)"); reconciliation 2's **Q20** table lists "SVG document model, SMIL, SVG DOM, geometry APIs" as **unowned**, and **Q25** says the SVG spec's staffing "is counted nowhere". Five accepted or in-progress specs already call into a spec that does not exist:

- **Graphics (GR 6.16, 6.28):** owns SVG painting (`hg-svg-paint`) and consumes "typed records ... element id, geometry in the bounded path format, resolved paint servers, markers with resolved orientation, clip, mask and filter references, isolation flags and `paint-order`" from the SVG spec.
- **Layout (LY 6.5.10):** owns outer `<svg>` sizing as a replaced element, `foreignObject` as a CSS layout root, and an inline-layout library call that returns per-character positions; "SVG geometry that reaches CSSOM View and ResizeObserver ... is answered by the future SVG spec through a trait `hg-layout-query` calls".
- **Images and media (MD-D20, MD-S10, Q-MD-18):** opaque cross-site SVG images are rasterized in the media process "by the scriptless SVG, XML, style, layout and text crates", once spike MD-S10 shows "the DOM can exist without a JS agent heap in that process".
- **Fonts and text (TX-D7, TX 6.3):** OT-SVG colour glyphs are rendered by "Izcal's SVG engine in the renderer, restricted glyph mode" (≤ 1 MiB, ≤ 10,000 elements, no script, external resources, animation or `foreignObject`, under the SVG engine's own fuel).
- **DOM (DM 6.12, 6.18):** owns generic element machinery, bindings and the `HTMLOrSVGElement` members; "SVG DOM, SMIL: Future SVG spec (G-12)".
- **Accessibility (AX):** generates SVG-AAM tables; needs SVG names, inclusion inputs and bounds.
- **Animation (being written in parallel, G-13):** its frame says "SMIL: coordinate with the SVG spec ... and decide which spec owns SMIL timing", and lists "the SVG element and attribute layer and SVG DOM `animVal`" as the SVG spec's.
- **PDF (PD-D19, PD-D20):** OT-SVG glyphs exported as vector Type 3 glyphs "drawn from Izcal's SVG painting"; SVG filters rasterized in print.

Scope (the coordinator's list): the SVG document model on the DOM spec's node storage (SVG 2 elements and attributes, SVG DOM interfaces, presentation attributes into style); SVG layout and geometry (viewBox, preserveAspectRatio, units, the layout spec's boundary, text and `textPath` with the text stack); painting hand-off to graphics (fill, stroke, markers, paint servers, clipping, masking, filters); `<use>` and shadow trees with resource-exhaustion security; SVG as an image (MD-D20; settle Q-MD-18); SVG in CSS; SMIL (decide with the animation spec); SVG fonts (decide); SVGZ; inline SVG in HTML parsing (ST's parser); accessibility (SVG-AAM with AX); hit testing and `pointer-events`; security (`data:` URLs, external references, `foreignObject`); determinism; footprint; observability.

Not in scope: rasterization, filter kernels, compositing (graphics); CSS box layout (layout); parsing HTML and XML (style spec's parsers; this spec supplies SVG tables); the semantic tree and OS adapters (accessibility); Web Animations, CSS animations and transitions and the frame clock (animation spec, graphics); image decoding of rasters referenced by `<image>` (media).

## Who has the problem

- **People browsing**: icons, logos, charts and illustrations on almost every page must render correctly, crisply at any zoom, and never let an image from another site run code or read their data.
- **Web developers and designers**: "why is my `getBBox` different across browsers?", "why doesn't my `<use>` from another file work?", "why does my SVG image not load its font?", "why is this filter slow?", "which SMIL animation is running?".
- **Data-visualisation authors**: thousands to tens of thousands of elements, with interaction.
- **Accessibility users**: SVG graphics that are named, navigable and readable, including text in SVG that can be found, selected and translated.
- **Izcal itself**: the shell and DevTools draw icons with SVG; OT-SVG fonts; the PDF writer.

## What Izcal's specs already decide or constrain

- **HGBA-2 2.3.2**: P1–P6; M-1 parts 1–4 (no engine libraries: resvg, usvg, librsvg, Batik, tiny-skia, lyon, kurbo-as-engine and the rest are excluded; MIT only; spec anchors; no third-party source as evidence, owner rule of 19 October 2026); I-1, I-2; C-1 WPT floors; ANC-1.
- **HGBA-3**: B1, B2, B3, B4 (as restated by JS-D4), B5, B6, B8, B9, B10; DET-1; PRIV-2.
- **Stage 1**: G1 (style, layout, paint, semantic read `hg-tree-traits`, never `hg-dom`), G3, G5, G9.
- **JS**: JS-D4 (node columns outside the cage traced as `hg-heap`'s node space), JS-D17 (one cage per process), JS G6 (only `hg-dom`, `hg-bindings`, `hg-script`, `hg-inspect-renderer` touch the engine), `hg-math` (L2s; deterministic transcendental functions), JS-D15.
- **DOM**: DM 6.1 (columns, rare data, mutation log, two-phase edits), DM 6.6 (shadow DOM), DM 6.12 (element ownership), G90–G99 (G98: no recursion on content depth).
- **Style**: ST-D5 (`hg-xml-syntax`), ST 6.2 (one parser core; foreign content), ST 6.9–6.11 (attribute sets carry the presentational-hint block; memo), ST-D18 admission, ST-D20 (XML entity budget), G19–G24 (G22 differential, G23 one parser per grammar), SVG UA sheet as standards data (Q-ST-19).
- **Text**: TX-D4 (font size quantized in local units, so SVG user units), TX-D7, TX 6.3 fuel, G25–G29 (G28 no floats on observable text paths).
- **Layout**: LY-D3 (1/64 px `LayoutUnit`), LY 6.5.10, LY 6.14 (hit test = reverse paint order), LY-D10 admission, G37–G45 (G39 no floats after the boundary).
- **Graphics**: GR-D3 (integer canonical pixels), GR 6.3 (bounded path format 24.8, 65,536 segments), GR 6.4 (matrices i64 Q32 composed in i128), GR 6.16, GR-D18 (cross-origin pixels never feed page effects), G60–G72 (G61 no floats in canonical crates).
- **Media**: MD-D5, MD-D20, MD-S10, Q-MD-18, G50 (SVG exception), G56, FP-38, MD-53, MD-56.
- **Footprint**: FP-11 (48 B columns; 512 B per node across DOM, style, layout, paint), FP-4, FP-34 (768 KiB per further agent heap), FP-D19 (identical limits on every device).
- **Accessibility**: AX tables generated from SVG-AAM and Graphics-AAM; G100 (semantic reads only through tree traits, style, fragments, text).
- **PDF**: G110–G119; OT-SVG export.
- **Rule range for this spec: G120–G129. ID prefix: SV-, SV-D#, SV-M#.**

## What success looks like (measurable)

- **Correctness**: WPT `svg/` (every subdirectory), `css/css-masking`, `css/filter-effects` SVG parts, `svg-aam/` at C-1 floors ≥ the best shipping engine; geometry APIs agree with an independent reference implementation exactly.
- **Security**: no `unsafe`; no recursion on content depth; every resource budget enforced deterministically; no cross-site SVG source in a renderer that cannot see it (G56); no script and no fetch in secure modes by construction.
- **Determinism**: render trees, display lists, geometry API results and SMIL samples bit-identical across runs, OSes and ISAs; SVG image rasters identical whether produced in the renderer or the media process.
- **Footprint**: SVG image documents cost no JS heap; shared per URL and partition; per-element bytes within FP-11.
- **Performance**: large data visualisations and icon-heavy pages meet frame and interaction targets.
- **Accessibility**: SVG-AAM conformance; SVG text in find, selection and the text model.
- **Observability**: every resolved reference, `<use>` instance, budget event and SMIL interval visible through an IIP `svg.*` domain.
