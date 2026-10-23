# Izcal SVG: usage and interoperability

Compiled 20 October 2026. Primary sources were preferred (HTTP Archive Web Almanac, Chrome use counters on chromestatus.com, wpt.fyi result summaries, MDN browser-compat-data, caniuse data, Interop project issues, benchmark "about" pages).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

M-1 note: no source code of any browser, engine, library, tool or other third-party project was opened or cited. Nothing here rests on code files, headers, commits, diffs or code reviews. The data comes from published datasets (chromestatus JSON, wpt.fyi summary JSON, MDN BCD JSON, caniuse and caniemail JSON), prose chapters, issue discussions and benchmark descriptions. One Interop proposal (#1469) mentions a path-data polyfill repository; it is quoted only as the proposal's own prose and was not opened. Under the owner rule of 20 October 2026, no code-file link appears in this note.

**Method notes**
- About 20 web searches and fetches, plus direct dataset downloads (chromestatus, wpt.fyi, BCD, caniuse, caniemail, Stack Exchange API).
- **Chrome use counters** (`day_percentage`) give the share of Chrome *page loads* that hit a counter on the given day. They are not the share of sites. All values are for **2026-10-18** unless stated. Feature counters are at `https://chromestatus.com/metrics/feature/timeline/popularity/<id>`, CSS counters at `https://chromestatus.com/metrics/css/timeline/popularity/<id>`. The id is given in each row.
- **wpt.fyi** figures were computed by this researcher from the official summary files of aligned stable runs at WPT revision `0f26972422` (Chrome 155.0.8059.84, Firefox 158.0, Safari 27.0; 2026-10-19). Ladybird (1.0-38531538077, 2026-10-20) ran at a different revision (`a26422aab3`), and Servo (0.7.0) at `0f26972422`. Ladybird and Servo are experimental runs, not stable channels. "Norm %" scores each test file as 1 and adds the fraction of its subtests that pass, which is close to the wpt.fyi default view. "Subtest %" is raw subtests passed over subtests run. Safari ran on macOS; the others ran on Linux.
- Rows resting on the researcher's recollection are graded Low and repeated under "Claims not verified".

## 1. How much of the web uses SVG

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | Web Almanac 2021: 46.4% of pages had some SVG. 37.2% had an inline `<svg>` element. 20.0% (desktop) and 18.4% (mobile) used SVG images. Embeds, objects and iframes with SVG were "negligible". | 46.4% / 37.2% / 20.0% / 18.4% | [Web Almanac 2021 Markup, "SVG use"](https://almanac.httparchive.org/en/2021/markup#svg-use) | 2021 | High |
| 1.2 | Web Almanac 2024: inline `<svg>` adoption on mobile grew from **45.5% (2022) to 51.6% (2024)**. `<path>` passed `<meta>` in element frequency. | 45.5% → 51.6% mobile | [Web Almanac 2024 Markup](https://almanac.httparchive.org/en/2024/markup) | 2024 | High |
| 1.3 | Chrome counter `SVGSVGElementInDocument` (id 250): 68.5% of page loads have an `<svg>` in an HTML document. `SVGSVGElement` (id 138) is at 70.3%. The 1 January value was 10.1% in 2015, 27.8% in 2018, 52.8% in 2020, 61.6% in 2023 and 64.8% in 2026. | 68.5% | [chromestatus 250](https://chromestatus.com/metrics/feature/timeline/popularity/250) | 2026-10-18 | High |
| 1.4 | Chrome counter `SVGImage` (id 5492): 57.5% of page loads decode at least one SVG image (`<img>`, CSS image and so on). `SVGSVGElementInXMLDocument` (id 329) is at 2.9%. | 57.5% / 2.9% | [chromestatus 5492](https://chromestatus.com/metrics/feature/timeline/popularity/5492) | 2026-10-18 | High |
| 1.5 | Web Almanac 2024 Media: **SVG is 6.4% of image requests on mobile**, a 36% relative rise since 2022 (a little under 2 percentage points). The chapter leaves vectors out of its pixel and byte analyses, so it gives no byte share for SVG. | 6.4% of images | [Web Almanac 2024 Media](https://almanac.httparchive.org/en/2024/media) | 2024 | High |
| 1.6 | Web Almanac 2025 CDN chapter: SVG requests grew 2.5% from 2022 to 2025 but fell 1.1% from 2024 to 2025. The chapter does not say whether these are percentage points or relative changes. | +2.5% / −1.1% | [Web Almanac 2025 (PDF), ch. 15](https://cdn.httparchive.org/v1/static/almanac/ebooks/web_almanac_2025_en.pdf) | 2025 | Medium |
| 1.7 | **Icon systems:** Font Awesome fell from about 4.4% of sites (2022) to 4.0% (2024) and roughly 3-4% (2025). The Almanac links this to moves to "inline SVGs, icon sprite sheets, or self-hosted icon sets". No published Almanac figure counts `<use href="#id">` sprites. | 4.4% → 3-4% | [Web Almanac 2025 Fonts (PDF)](https://cdn.httparchive.org/v1/static/almanac/ebooks/web_almanac_2025_en.pdf) | 2025 | Medium |
| 1.8 | **SVG favicons:** 0.4% of pages (2021) and 1% (2022, per Meiert's reading of the Almanac data). The 2024 Almanac gives 1% on mobile, against png 42% and ico 27%. | 1% mobile | [Web Almanac 2024 Markup](https://almanac.httparchive.org/en/2024/markup); [Meiert, HTML 2022](https://meiert.com/blog/html-2022/) | 2022-2024 | High |
| 1.9 | W3Techs reports SVG on 63.5% of websites (September 2025). It uses its own crawl and method, which are not comparable with HTTP Archive. | 63.5% | [W3Techs SVG](https://w3techs.com/technologies/breakdown/im-svg/ranking) (search snippet) | 2025-09 | Medium |

**Implications:** SVG is a majority feature. About two thirds of page loads have inline SVG and over half decode an SVG image. The inline path (HTML parser, DOM, style, layout and paint in the renderer) and the image path (MD-D20 media process) both sit on the critical path for most pages, so neither can be a slow or deferred subsystem.

## 2. Feature-level use counters (Chrome, 2026-10-18)

| # | Feature | Counter (id) | Share of page loads | Notes | Conf. |
|---|---|---|---|---|---|
| 2.1 | Any SMIL animation element in a document | `SVGSMILElementInDocument` (501) | **2.90%** | 1 Jan values: 0.047% (2015), 0.46% (2016), 0.58% (2019), 1.28% (2021), 1.86% (2023), 2.70% (2026). Rising every year. | High |
| 2.2 | SMIL animation that applied an effect | `SVGSMILAnimationAppliedEffect` (1477) | 2.33% | | High |
| 2.3 | SMIL in an SVG image (regardless of cache) | `SVGSMILAnimationInImageRegardlessOfCache` (768) | **1.67%** | Rose from 0.05% (2016) to about 2.0% (2023-2025). The older `SVGSMILAnimationInImage` (626) reads 0. | High |
| 2.4 | SMIL inserted after load; `calcMode="spline"`; syncbase timing; `<discard>` parsed | 1457; 1290; 1456; 2968 | 1.36%; 0.67%; 0.37%; 0.0008% | Event-based begin/end (1455) 0.019%; `currentTime` API (1507) 0.016%; `animateColor` 0. | High |
| 2.5 | `<foreignObject>` element | `SVGForeignObjectElement` (325) | **8.24%** | 1 Jan values: 0.32% (2025), 1.78% (2026). The sudden rise has no known cause; it could be a single widely deployed library. In images: `SVGForeignObjectInImage` (5844) 0.59%; drawn into canvas (5845) 0.022%. | High |
| 2.6 | SVG text laid out | `SVGText` (3912) | 2.02% | `textPath` has no public counter. `SVGTextHangingFromPath` (3914) is at 0.0005%. | High |
| 2.7 | External SVG document resources (e.g. `<use href="file.svg#id">`, external filter or clip references) | `ExternalSVGDocumentResources` (5668) | **4.49%** | Counter started 2025-09. Its exact trigger is not documented in prose. `<use>` with `data:` URL (4356) 0.0002%; with `Content-Disposition` (4451) 0.006%. | Medium (semantics) |
| 2.8 | CSS `filter: url(...)` reference | `CSSFilterReference` (1014) | **6.19%** | Rose from 1.2% (2019) to 3.0% (2023) and 6.0% (2026). `blur()` 27.6%, `drop-shadow()` 17.1%. | High |
| 2.9 | SVG filter primitives | `SVGFEGaussianBlurElement` (5756) etc. | feGaussianBlur 4.74%, feOffset 2.53%, feFlood 2.02%, feColorMatrix 2.01%, feBlend 1.93%, feComposite 1.51%, feMorphology 0.26%, feDropShadow 0.18%, feTurbulence 0.097%, feDisplacementMap 0.023%, feImage 0.0077%, feSpecularLighting 0.0025%, feConvolveMatrix 0.0019%, feDiffuseLighting 0.00035%, feTile 0.00019% | The lighting, convolve and tile primitives are below 1 in 40,000 page loads. | High |
| 2.10 | `SVGPathSeg` DOM | `SVGPathSegDOM` (568) | ~0 (last data 2023-02-16) | The API was removed from Chrome and Firefox. `getPathData`/`setPathData` counters (5927, 5935) read 0.000000% (Chrome has not shipped them). | High |
| 2.11 | SVG fonts | `SVGFontElement` (76), `SVGFontInCSS` (156) | 0 | Unsupported in Chrome, so these measure only failed attempts. | High |
| 2.12 | `xml:base` | `SVGElementXmlbase` (681), `ElementBaseURIFromXMLBase` (580) | ~0 (last data 2023-02-16) | | High |
| 2.13 | `context-fill`/`context-stroke` | `SvgContextFillOrStroke` (5156) | 0.0052% | Shipped in Chrome 124 (see 4.1). | High |
| 2.14 | SVG geometry and text DOM methods | `getTotalLength` (2667) 0.32%; `getComputedTextLength` (2674) 0.13%; `getPointAtLength` (2668) 0.084%; `isPointInFill` (2665) 0.0062%; `isPointInStroke` (2666) 0.00005% | | `getBBox` has no public counter. | High |
| 2.15 | SVG in canvas 2D / WebGL | `SVGInCanvas2D` (1835) 0.51%; `SVGInWebGL` (1836) 0.033% | | | High |
| 2.16 | `enable-background`, `BackgroundImage` input, SVGZ | none found | n/a | No public counter was found for these. `enable-background` is not a CSS property in Chrome's list. SVGZ needs HTTP-level data, not a DOM counter. | Medium |
| 2.17 | Legacy SVG 1.1 DOM (`SVG1DOM*`) | `SVG1DOMUriReference` (1101) 1.91%; `ObsoleteSVG1DOMFilter` (1030) 1.26% | | These counters track 1.1-era DOM interfaces that are still reached. | High |

**CSS property counters (presentation attributes may or may not be counted; this is not stated in prose):**

| # | Property (id) | Share | Conf. |
|---|---|---|---|
| 2.18 | `fill` (369) 66.9%; `stroke-width` 47.0%; `stroke` 43.4%; `stroke-dashoffset` 31.3%; `stroke-dasharray` (379) 28.4%; `fill-opacity` 19.8%; `stroke-linecap` 16.8% | majority | High |
| 2.19 | `clip-path` (355) 46.7%; `mask-image` (772) 31.6%; alias `-webkit-mask-image` (289) 31.2%; `mask` (357) 23.3%; alias `-webkit-mask` (280) 18.3%; `mask-type` (376) 3.5% | prefixed and unprefixed masks are equally common | High |
| 2.20 | `transform-box` (559) 3.62%; `paint-order` (434) 1.74%; `vector-effect` (393) 1.53%; `d` (518) 0.69%; `cx` (466) 7.66%; `r` (468) 8.56%; `text-anchor` (392) 2.18%; `dominant-baseline` (388) 0.86%; `marker-end` (373) 0.27% | | High |

**Implications:** SMIL is **rising**, not dying: 2.9% of page loads, about 60 times its level when Chrome proposed deprecation in 2015 (2.1; see "Claims not verified"). It also runs inside SVG images (1.7%), so the scriptless media-process path needs a SMIL timing engine. `foreignObject` (8.2%) and `filter: url()` (6.2%) are large enough to need first-class, deterministic support. Lighting, convolution and tile filters, `xml:base`, SVG fonts and `SVGPathSeg` are at or near zero.

## 3. Interoperability on wpt.fyi (`svg/`, 2026-10-19)

| # | Directory | Tests | Chrome norm/sub % | Firefox | Safari | Ladybird* | Servo* |
|---|---|---|---|---|---|---|---|
| 3.1 | **all `svg/`** | **1,411 files** (15,830 subtests in Chrome) | **87.2 / 96.3** | **88.4 / 97.2** | **73.3 / 91.8** | 49.5 / 65.2 | 24.4 / 37.8 |
| 3.2 | animations | 321 | 96.4 / 96.2 | 92.6 / 92.4 | 77.7 / 77.9 | 21.7 / 25.9 | 4.2 / 4.7 |
| 3.3 | painting | 165 | 91.7 / 97.5 | 84.9 / 94.9 | 81.0 / 94.9 | 69.0 / 81.7 | 28.1 / 41.0 |
| 3.4 | text | 134 | 77.7 / 83.6 | 79.1 / 84.3 | 57.5 / 66.8 | 40.5 / 41.4 | 45.8 / 29.2 |
| 3.5 | styling | 104 | 72.4 / 91.4 | 72.1 / 79.1 | 55.8 / 86.1 | 51.5 / 71.7 | 49.0 / 39.8 |
| 3.6 | types | 100 | 89.6 / 86.0 | 97.6 / 99.0 | 86.4 / 72.6 | 46.6 / 60.1 | 2.1 / 6.7 |
| 3.7 | geometry | 86 | 89.6 / 92.6 | 96.1 / 94.0 | 94.0 / 92.3 | 93.0 / 90.5 | 14.2 / 39.2 |
| 3.8 | linking | 80 | 74.3 / 81.2 | 85.4 / 89.8 | 58.6 / 51.6 | 57.7 / 43.6 | 27.5 / 18.2 |
| 3.9 | struct | 80 | 92.9 / 94.7 | 78.9 / 84.1 | 70.0 / 78.1 | 51.3 / 47.7 | 38.7 / 30.9 |
| 3.10 | path | 72 | **54.7** / 69.8 | 82.9 / 87.8 | **38.3** / 28.7 | 32.8 / 39.5 | 8.0 / 3.3 |
| 3.11 | pservers | 47 | 93.6 / 96.9 | 93.6 / 96.9 | 74.5 / 87.5 | 63.6 / 81.2 | 29.8 / 25.5 |
| 3.12 | interact | 37 (9,254 subtests) | 94.2 / 99.9 | 97.4 / 99.7 | 74.2 / 98.9 | 47.7 / 65.7 | 12.1 / 47.7 |
| 3.13 | shapes | 36 | 83.7 / 76.6 | 97.2 / 97.9 | 94.4 / 95.7 | 82.9 / 80.9 | 38.9 / 38.9 |
| 3.14 | embedded | 27 | 96.3 / 93.5 | 92.6 / 93.5 | 88.9 / 87.1 | 53.7 / 51.6 | 37.0 / 32.3 |
| 3.15 | coordinate-systems | 27 | 92.6 / 97.3 | 100 / 100 | 96.3 / 98.6 | 65.0 / 53.4 | 25.9 / 35.6 |
| 3.16 | extensibility | 25 | 100 / 100 | 95.0 / 92.9 | 60.0 / 64.3 | 76.0 / 78.6 | 32.0 / 32.0 |
| 3.17 | render, layout, scripted, fonts, as-image, crashtests | 12, 12, 10, 4, 3, 23 | mostly 75-100 | mostly 67-100 | layout 33.3 | | |

\* Experimental runs. Ladybird ran at a different WPT revision. Source: computed from the [wpt.fyi summary files](https://wpt.fyi/results/svg?label=master&label=stable&aligned) of the runs named in the method notes. Conf. High for the data; Medium for how closely it matches wpt.fyi's own scoring.

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.18 | Of 1,405 `svg/` test files in the Chrome run, **891 (63.4%) pass fully in all three** major engines. 203 pass in Chrome and Firefox but fail in Safari. 83 pass only in Firefox, 50 only in Chrome, 10 only in Safari. 77 fail in all three. | 891/1405 | computed from wpt.fyi summaries | 2026-10-19 | High |
| 3.19 | There is **no `svg/import` directory** on wpt.fyi. The SVG 1.1 Second Edition suite sits in WPT as unconverted `-manual` tests that are not shown on the site. The SVG 1.1 suite is therefore not an automated oracle. | n/a | [Interop #214](https://github.com/web-platform-tests/interop/issues/214) | 2022-10-13 | High |
| 3.20 | Related directories: `css/css-masking/` has 566 files (norm %: Chrome 92.4, Firefox 93.2, Safari 86.2). `css/filter-effects/` has 395 files (Chrome 94.4, Firefox 89.4, Safari 76.4, Ladybird 65.8). | | computed from wpt.fyi | 2026-10-19 | High |

**Implications:** SVG is one of the least interoperable mature areas. Only 63% of SVG tests pass in all three engines, and the path, text, linking and styling directories sit around 40-80%. Where engines disagree, WPT alone cannot settle the right answer; Izcal must read the spec, and expect to file spec issues and contribute tests. The **1,411-file `svg/` directory**, together with css-masking and filter-effects, is the conformance baseline Izcal can measure. The SVG 1.1 suite would need conversion before it is useful.

## 4. Feature status across engines (MDN BCD 8.1.6, 2026-10-15; caniuse, 2026-10)

| # | Feature | Chrome | Firefox | Safari | Source | Conf. |
|---|---|---|---|---|---|---|
| 4.1 | `context-fill`/`context-stroke` in SVG content | 124 | 111 | **no** | [BCD svg.elements.path.fill](https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/fill#browser_compatibility) | High |
| 4.2 | CSS `d` property | 52 | 97 | **no** | [BCD css.properties.d](https://developer.mozilla.org/en-US/docs/Web/CSS/d#browser_compatibility) | High |
| 4.3 | `paint-order` | 123 | 60 | 11 | BCD | High |
| 4.4 | `href` without `xlink:` (on `use`) | 50 | 1.5* | 12.1 | BCD (*BCD records the attribute itself; plain `href` came to Firefox later) | Medium |
| 4.5 | Geometry properties as CSS (`cx`, `r`, `x`) / `rx` | 42-43 / 43 | 69 / 69 | 9 / **17.4** | BCD | High |
| 4.6 | `transform-box` | 64 | 55 | 11 | BCD | High |
| 4.7 | `<radialGradient fr>` | 24 | 55 | 7 | BCD | High |
| 4.8 | `pathLength` on all geometry elements (`SVGGeometryElement.pathLength`) | 57 | 61 | 12 | BCD | High |
| 4.9 | `isPointInFill` | 33 | 69 | 12 | BCD | High |
| 4.10 | `getBBox(options)` | **no** | 155 | **no** | BCD (experimental) | High |
| 4.11 | `getPathData`/`setPathData`/`getPathSegmentAtLength` | **no** | 137 | **no** | BCD | High |
| 4.12 | `SVGPathSeg` API | removed (`getPathSegAtLength` removed in 62) | removed (97) | still present (`getPathSegAtLength` since 3) | BCD | High |
| 4.13 | `vector-effect` | `non-scaling-stroke` only (6) | only (15) | only (5.1) | BCD (other SVG 2 values not in BCD) | High |
| 4.14 | `<discard>` / `SVGDiscardElement` | removed in 81 | 136-138, removed in 139 | no | BCD | High |
| 4.15 | `textPath` `path` and `side` attributes | no | 61 | no | BCD | High |
| 4.16 | `<use>` without a fragment (whole external file) | 137 | no | no | BCD `svg.elements.use.omit_external_fragment` | High |
| 4.17 | `<use>` with `data:` URL | removed in 120 | removed in 122 | never | BCD | High |
| 4.18 | `transform` on the root `<svg>` | 137 | 23 | no | BCD | High |
| 4.19 | `<script type=module>`/`async`/`defer` in SVG | no | 117 | no | BCD | High |
| 4.20 | `alignment-baseline`, `baseline-shift` as SVG attributes | yes | **no** | yes | BCD | High |
| 4.21 | `tspan` `textLength`/`lengthAdjust` | yes | **no** | yes | BCD | High |
| 4.22 | `feGaussianBlur edgeMode` | no | no | 8 | BCD | High |
| 4.23 | `mesh`, `hatch` (SVG 2 drafts) | no | no | no | not in BCD; never shipped | Medium |
| 4.24 | SMIL animation | yes | yes | yes | [caniuse svg-smil](https://caniuse.com/svg-smil) (97.3% global) | High |
| 4.25 | **SVG fonts** (`<font>`) | no | no | **yes** (Safari, iOS) | [caniuse svg-fonts](https://caniuse.com/svg-fonts) (17.9% global) | High |
| 4.26 | OpenType-SVG color fonts | **no** | 31 | 12.1 | BCD `css.at-rules.font-face.OpenType_SVG` | High |
| 4.27 | COLRv1 (the competing color-font format) | 98 | 107 | no | BCD | High |
| 4.28 | SVG favicons | 80 | 41 | **26.0** | [caniuse link-icon-svg](https://caniuse.com/link-icon-svg) (92.1% global) | High |

**Implications:** No engine implements all of SVG 2. Each ships a different subset, and Safari lacks `d` and `context-fill`. Izcal can target the SVG 2 core at once (`d`, `context-*`, `paint-order`, CSS geometry properties, plain `href`, `getBBox` options, `getPathData`). It should also decide explicitly about SVG fonts (Safari-only, use near 0), `<discard>` (withdrawn by both engines that shipped it) and OpenType-SVG (Chrome refuses it, yet it is 58-60% of color-font requests per 7.3).

## 5. Interop project proposals touching SVG

| # | Proposal | Year | Outcome and stated reason | Source | Conf. |
|---|---|---|---|---|---|
| 5.1 | "Better Support for CSS masks" | 2023 | **Accepted** (Masking focus area) | [Interop #148](https://github.com/web-platform-tests/interop/issues/148) | High |
| 5.2 | "SVG 1.1 and 2.0" investigation | 2023 | Rejected 2023-02-01: "too broad, and … Interop 2023 is not the right venue"; told to work with the working groups | [#214](https://github.com/web-platform-tests/interop/issues/214) | High |
| 5.3 | "css fill/stroke"; "Standardize SVG properties in CSS to support unitless geometry" | 2024 | Not selected | [#440](https://github.com/web-platform-tests/interop/issues/440), [#554](https://github.com/web-platform-tests/interop/issues/554) | High |
| 5.4 | "SVG" (general); "SVG favicons"; "SVG filters" (52 upvotes per #1469); "CSS selectors should not pierce the shadow root created by `<use>`"; "SVG stroke-alignment" | 2025 | All not selected, with the form reply "many more proposals than we could include". #821 cites ten Stack Overflow threads on Safari/Firefox SVG differences (`<image>` rendering, `transform-origin` on `<use>`, `<use>` in Safari, filters on iOS 18). | [#821](https://github.com/web-platform-tests/interop/issues/821), [#745](https://github.com/web-platform-tests/interop/issues/745), [#756](https://github.com/web-platform-tests/interop/issues/756), [#852](https://github.com/web-platform-tests/interop/issues/852), [#750](https://github.com/web-platform-tests/interop/issues/750) | High |
| 5.5 | Interop 2025 backdrop-filter: tests that use SVG filters with `backdrop-filter` were proposed for removal | 2025-03 | test-change proposals | [#946](https://github.com/web-platform-tests/interop/issues/946) | Medium |
| 5.6 | "Allow omission of fragment when using SVG `<use>`" (23 upvotes); "Link parameters for styling non-inline SVG"; "Dark mode for SVG"; "Standardize CSS filters on inline SVG elements"; "SVG stroke-alignment" | 2026 | Closed, not selected. "CSS `shape()`" **accepted** for 2026; a test for the `d`-attribute form of `path()` was added. | [#1045](https://github.com/web-platform-tests/interop/issues/1045), [#996](https://github.com/web-platform-tests/interop/issues/996), [#1058](https://github.com/web-platform-tests/interop/issues/1058), [#1104](https://github.com/web-platform-tests/interop/issues/1104), [#1046](https://github.com/web-platform-tests/interop/issues/1046) | High |
| 5.7 | "Targeted SVG interop improvements" for 2027 (open): `<use>` without fragment, `context-fill/stroke`, SVG filters, `getPathData`/`setPathData`/`getPathSegmentAtLength`. Written because #821 was judged "too big and unfocused". Also open: "CSS Linked Parameters for SVG" (#1346). | 2027 cycle, opened 2026-09-23 | pending | [#1469](https://github.com/web-platform-tests/interop/issues/1469), [#1346](https://github.com/web-platform-tests/interop/issues/1346) | High |

**Implications:** Vendors have turned down whole-SVG interop four years running. The pain is real but unfunded, so Izcal cannot expect the incumbents to converge soon. The 2027 short list (`<use>` without fragment, `context-*`, filters, path-data API, linked parameters) is the best public signal of what developers want next; Izcal should implement these early.

## 6. Developer pain

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | State of HTML 2024: **"SVG issues" is the top content pain point**, with 73 mentions among 398 respondents to the question (about 18%). Image management is next with 68. The text names "styling SVGs with CSS" as the issue. | 73 / 398 | [State of HTML 2024, Content](https://2024.stateofhtml.com/en-US/features/content/) | 2024 | High |
| 6.2 | Stack Overflow `[svg]` tag: **44,229 questions** in all. New questions per year: 2,180 (2012), 4,163 (2014), 4,919 (2016), 3,678 (2018), 3,392 (2020), 2,462 (2022), 1,522 (2023), 830 (2024), 255 (2025). The decline tracks Stack Overflow's overall decline, not just SVG. Co-tags: `css` 12,213, `d3.js` 6,215, `firefox` 785, `internet-explorer` 507, `safari` 490, `fonts` 311. `svg-animate` 727, `svg-filters` 660, `smil` 230. | as stated | [Stack Exchange API](https://api.stackexchange.com/2.3/tags/svg/info?site=stackoverflow) | 2026-10-20 | High |
| 6.3 | **SVG used as an image** has JavaScript disabled, cannot load external resources (images, stylesheets, and so fonts) except through `data:` URLs, and does not render `:visited`. This is MDN's list of image contexts: `<img>`, CSS backgrounds, `list-style-image`, `content`, `<image>`, `<feImage>` and canvas `drawImage`. It explains the recurring "my font or CSS doesn't load in `<img>` SVG" complaint. | n/a | [MDN: SVG as an image](https://developer.mozilla.org/en-US/docs/Web/SVG/Guides/SVG_as_an_image) | 2025-05-13 | High |
| 6.4 | **`currentColor` and theming in images:** an SVG image cannot inherit the host page's color or custom properties. Proposals for "link parameters" (#996, #1346) and "Dark mode for SVG" (#1058) exist for this reason. Prefixed `-webkit-mask-image` (31.2% of page loads) is widely used to recolor icons through a mask. | 31.2% | Interop issues in 5.6-5.7; chromestatus CSS 289 | 2025-2026 | Medium |
| 6.5 | **SVG filters:** in the 2025 SVG filters proposal (#756), a frequent author describes bugs "over a decade and a half old", Firefox displacement-map failures and Safari tiling failures, and calls the area "long neglected". | n/a | [Interop #756](https://github.com/web-platform-tests/interop/issues/756) | 2024-09-26 | High (as a developer account) |
| 6.6 | Known interop complaints (per #821's links and general knowledge): `getBBox` differences (no stroke or markers option outside Firefox), `transform-origin`/`transform-box` defaults on SVG elements, `<use>` cross-origin (external `<use>` must be same-origin), markers, inline `<svg>` sizing without `viewBox`/width (300×150 default), and `vector-effect`. | n/a | [#821](https://github.com/web-platform-tests/interop/issues/821); recollection for the list | 2024 | Low |

**Implications:** The top developer pain is styling and theming SVG that is used as an image, and SVG filters. A deterministic, spec-exact implementation of filters, `getBBox` and `<use>` would be a visible advantage. Link parameters or a similar theming feature are worth tracking for the SVG-in-image path, because MD-D20 forbids page CSS from leaking into the media process.

## 7. Performance and footprint evidence

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **MotionMark 1.3.1**: one of 8 core tests uses SVG. "Suits" covers "SVG clip paths, gradients and transforms". The others are CSS (Multiply, Leaves), canvas (Arcs, Paths, Lines, Images) and HTML text (Design). | 1/8 | [MotionMark about](https://browserbench.org/MotionMark1.3.1/about.html) | 2024 | High |
| 7.2 | **Speedometer 3.1** "Rendering charts" journey: "Loading and interacting with SVG and canvas charts" (Charts-observable-plot, Charts-chartjs, React-Stockcharts-SVG, Perf-Dashboard). SVG DOM creation and style/layout of SVG are part of the leading browser benchmark. | 4 workloads | [Speedometer 3.1 about](https://browserbench.org/Speedometer3.1/about.html) | 2024 | High |
| 7.3 | Web Almanac 2025 color fonts: by requests, roughly 58-60% are SVG-in-OpenType, 25% COLRv0, 16% COLRv1 and a few percent bitmap. The SVG share comes mostly from two Japanese decorative fonts on high-traffic pages. Noto Color Emoji is on about 73% of desktop color-font pages. | 58-60% SVG | [Web Almanac 2025 Fonts (PDF)](https://cdn.httparchive.org/v1/static/almanac/ebooks/web_almanac_2025_en.pdf) | 2025 | High |
| 7.4 | SVG vs canvas: a 2018 TU Dresden poster found D3 SVG and Canvas performed "almost equally". Frame rates fell for all technologies above about 400 nodes (~8,000 graphical elements). A LogRocket benchmark on Chrome 73 put 1,000 objects at about 10 ms in SVG against about 1 ms in canvas. | ~8,000 elements; 10 ms vs 1 ms | [Horak 2018 poster](https://mt.inf.tu-dresden.de/cnt/uploads/Horak-2018-Graph-Performance-Poster.pdf); [LogRocket](https://blog.logrocket.com/when-to-use-html5s-canvas-ce992b100ee8) (snippets) | 2018-2019 | Low |
| 7.5 | Chrome counts `CompositedSVG` (3710) at 3.61% and SVG animation types `AnimationSvgTypes` (5713) at 14.8%. Most of those are non-composited (5716, 13.2%), not composited (5717, 4.33%). SVG animation is mostly main-thread paint work. | 14.8% / 13.2% / 4.3% | chromestatus | 2026-10-18 | High (data); Medium (meaning) |
| 7.6 | No published measurement was found for the memory of icon sprites, re-rasterization of SVG images on zoom, or the cost of large inline SVG. | n/a | n/a | n/a | n/a |

**Implications:** The main benchmarks touch SVG lightly (Suits; Speedometer chart workloads), so performance there does not show SVG quality. Izcal should set its own budgets: about 10,000 elements of D3-style SVG at 60 fps, filter cost per primitive, and the raster-cache policy for SVG images under zoom. These should be measured black-box against incumbents, since no good published numbers exist.

## 8. SVG outside browsers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | SVG favicons: Chrome 80+, Firefox 41+ and **Safari only from 26.0 (2025)**. Served as `image/svg+xml`. Chromium renders them in secure static mode, but SMIL is applied at document time 0. An Interop 2025 proposal for SVG favicons was not selected. | 92.1% global | [caniuse link-icon-svg](https://caniuse.com/link-icon-svg); [#745](https://github.com/web-platform-tests/interop/issues/745) | 2026-10 | High |
| 8.2 | Email: SVG as an image format works in Apple Mail, Outlook (all), Yahoo, AOL, Thunderbird, Fastmail and others. **Gmail rasterizes it to PNG**, and Mail.ru does not support it (tested 2026-09-16). Inline `<svg>` in email is unsupported in Gmail, Outlook, Yahoo, AOL, HEY and Fastmail, where the element is stripped (tested 2020). | as stated | [caniemail image-svg](https://www.caniemail.com/features/image-svg/); [caniemail html-svg](https://www.caniemail.com/features/html-svg/) | 2026-09-16 / 2020-02-06 | High |
| 8.3 | Social card images: X accepts JPG, PNG, WEBP and GIF (first frame) up to 5 MB, and **SVG is not supported**. No Meta/Facebook primary text was found this session. | n/a | [X Cards markup](https://developer.x.com/cards/markup) (snippet) | n/a | Medium |
| 8.4 | Clipboard: Chrome counters `ClipboardSvgRead` (5229) 0.00006% and `ClipboardSvgWrite` (5230) 0.000009%. An Interop 2023 proposal "Clipboard API: SVG and custom types" (#241) was not selected. | ~0 | chromestatus; [#241](https://github.com/web-platform-tests/interop/issues/241) | 2026-10-18 | High |

**Implications:** Outside browsers, SVG is mostly rasterized or refused (Gmail, social cards). Izcal's static, scriptless SVG-image renderer, which is deterministic, can also serve these export uses: favicons, thumbnails, "save as PNG" and print. Favicons should be rendered in the same static mode as other SVG images.

## Open questions for the spec

1. **SMIL owner:** SMIL use is 2.9% of page loads and rising, and 1.7% is inside images (no JS). Does the SVG spec or the animation spec own the SMIL timing model, and how does it run in the scriptless media process?
2. **`foreignObject`** jumped from 0.3% to 8.2% in under two years. Which pattern drives it (HTML-in-SVG screenshots, `foreignObject` in images or canvas)? A black-box crawl of top pages should decide the priority for HTML layout inside SVG, including in the media process.
3. What does `ExternalSVGDocumentResources` (4.49%) count? Measure on a corpus how many pages use external `<use>`, filter and clip references, and from which origins.
4. Should Izcal implement **SVG fonts** (Safari-only, near zero use) and **OpenType-SVG** (Chrome refuses it; 58-60% of color-font requests)? This is a joint decision with `hg-text`.
5. Should Izcal ship **`<discard>`**, given that two engines shipped it and then withdrew it?
6. Should Izcal expose **`SVGPathSeg`** (Safari still has it; Chrome and Firefox removed it) or only `getPathData`?
7. A conformance target: the `svg/` WPT pass rate Izcal must reach before shipping (for example, at least Firefox's 88% norm), and whether to convert the SVG 1.1 manual suite into reftests.
8. Performance budgets with no published baseline: large inline SVG, sprites, raster cache on zoom, and cost per filter primitive. Measure them black-box.
9. Theming SVG images (link parameters, `color-scheme` in SVG images) under MD-D20 isolation.

## Claims not verified

- 1.6: whether the Almanac 2025 CDN deltas are percentage points or relative changes.
- 1.9: the W3Techs figure (search snippet only).
- 2.7: the exact trigger of `ExternalSVGDocumentResources`, which is not documented in prose.
- 2.18-2.20: whether Chrome's CSS property counters include SVG presentation attributes.
- 4.4: the Firefox version for plain `href` (BCD's record may refer to `xlink:href`).
- 4.23: that `mesh` and `hatch` never shipped in any engine (based on their absence from BCD and on recollection).
- 2.1 (context): SMIL deprecation history. The blink-dev "Intent to deprecate: SMIL" thread is dated 2015-04-29 per search snippets ([thread](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/YGEJBsjUAwAJ)); its suspension in 2016 rests on a secondary source ([ZKM](https://extinction.zkm.de/events/smil-browser-support-collapse)). Low.
- 6.6: the list of common complaints, beyond #821's links (recollection). Low.
- 7.4: SVG vs canvas benchmark numbers (search snippets of secondary sources). Low.
- 8.3: X's card formats (snippet); Facebook `og:image` SVG support not checked.
