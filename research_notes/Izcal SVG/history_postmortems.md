# Izcal SVG: history and postmortems

Compiled 20 October 2026. Primary sources were preferred (specifications, W3C records and minutes, mailing lists, maintainers' posts, compatibility data).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only, or the researcher's recollection.

M-1 note: this file records ideas, dates, usage numbers and stated reasons. **No source code of any browser, engine or library was opened or cited** (Chromium/Blink, WebKit, Gecko, Presto, librsvg, resvg, Batik, Inkscape, Skia, Cairo and others). Several search results surfaced Chromium code reviews and a third-party page describing Blink's SVG-text internals; none was opened, and nothing here rests on them. Prose only: W3C TR pages, W3C charter pages and minutes, blink-dev threads, maintainers' blogs, project READMEs, MDN compatibility tables and caniuse. Per the owner's later rule (no source code of any third-party project, including libraries and tools), every link was re-checked: none points to a code file, header, commit, diff or code review. The resvg link is its README (prose); an earlier draft linked the raw MDN browser-compat-data JSON, which was replaced with the MDN page.

**Method notes**
- 24 searches and fetches were used. chromestatus.com renders client-side and returned nothing; Chrome feature entries are therefore taken from blink-dev threads or MDN compatibility data (MDN pages, not code).
- Read in full or near full: W3C SVG 2 status section and its Changes appendix (CR Snapshot of 2026-10-06); the SVG 2 publication history; the SVG WG charter history; the 2015-04-30 SVG WG minutes; the blink-dev "Intent to deprecate: SMIL" opening messages; three librsvg maintainer posts; the resvg README; the SVG Integration FPWD; caniuse svg-fonts and svg-html5; the MDN compatibility data for `SVGPathElement` (read as data, not code; cited by its MDN page).
- Rows resting on recollection are graded Low and repeated under "Claims not verified".

## 0. Chronology at a glance

| Year | Event | Section |
|---|---|---|
| 1998 | Six vector-format submissions to W3C, including **PGML** (Adobe, IBM, Netscape, Sun) and **VML** (Autodesk, HP, Macromedia, Microsoft, Vision); SVG WG chartered 1998-07-24 | 1 |
| 1999-2009 | Adobe SVG Viewer plugin era (IE/Netscape) | 2 |
| 2001-09-04 | SVG 1.0 Recommendation | 1 |
| 2003-01-14 | SVG 1.1 plus Mobile profiles (Tiny, Basic) Recommendations; 3GPP mandates SVG Tiny | 1 |
| 2004-2007 | Konqueror/KSVG (2004), Opera 8 (Tiny, 2005), Firefox 1.5 (2005), WebKit/Safari | 2 |
| 2008-12-22 | SVG Tiny 1.2 Recommendation | 1 |
| 2009-01-01 | Adobe ends SVG Viewer support | 2 |
| 2010-2011 | HTML5 parser brings inline `<svg>`: Chrome 7, Firefox 4, IE9 (2011-03-14), Safari 5.1 | 2, 5 |
| 2011-08-16 | SVG 1.1 Second Edition | 1 |
| 2012-08-28 | First SVG 2 Working Draft | 1 |
| 2014 | SVG fonts gone from Chrome 38 / Opera 25; SVG Integration FPWD (2014-04-17) | 3, 1 |
| 2015-04-29 | Blink "Intent to deprecate: SMIL"; SVG WG resolves to move animation out of SVG 2 (2015-04-30) | 3 |
| 2016 | SVGPathSeg API removed in Chrome 48 (early 2016); SVG 2 CR (2016-09-15) | 3, 1 |
| 2018 | SVG 2 re-CR (2018-08-07, 2018-10-04) with mesh, hatch, `solidcolor`, cursor cut | 1 |
| 2018 | Firefox 59 removes SVGPathSeg | 3 |
| 2019-01 | librsvg "almost 100% Rust" | 4 |
| 2025 | Firefox 137 ships `getPathData`/`setPathData` | 3 |
| 2026-06 | Blink "Intent to Prototype" for the SVG Path Data API | 3 |
| 2026-10-06 | New SVG 2 CR Snapshot; Recommendation "no earlier than 2026-12-01" | 1 |

## 1. Specifications and the Working Group

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | 1998 saw six competing submissions; the two that mattered were **PGML** (Adobe, IBM, Netscape, Sun; PostScript/PDF imaging model) and **VML** (Microsoft, Macromedia, HP, Autodesk, Vision). SVG was a new W3C design drawing on both. VML shipped in IE5 and stayed Microsoft's only vector format until IE9. | 6 submissions | [Wikipedia: SVG](https://en.wikipedia.org/wiki/SVG) | 1998 | Low |
| 1.2 | SVG WG charter periods: 1998-07-24 first; then 2001, 2003, 2004, 2008, 2012, 2014, 2017, 2019, 2022, 2024. Seven gaps of 1 to 8.5 months between charters. Current charter 2024-06-27 to 2026-06-30, extended to 2026-11-30; a successor charter entered review on 2026-06-26 with the mission "maintain SVG specifications". **The group was never merged or closed**; it was repeatedly rechartered with a shrinking maintenance scope. | 11 charters | [W3C SVG WG charters](https://www.w3.org/groups/wg/svg/charters/); [charter renewal notice](https://lists.w3.org/Archives/Public/public-new-work/2026Jun/thread.html) | 1998-2026 | High (periods) / Medium (2026 renewal) |
| 1.3 | SVG 1.0 Recommendation 2001-09-04; SVG 1.1 and SVG Mobile (Tiny, Basic) 2003-01-14; SVG Tiny 1.2 2008-12-22; SVG 1.1 Second Edition 2011-08-16 (errata only, no new features). | | [Wikipedia: SVG](https://en.wikipedia.org/wiki/SVG) | 2001-2011 | Medium |
| 1.4 | **Mobile profiles faded.** SVG Tiny was designed for phones without styling or scripting; 3GPP mandated it in 2003. Tiny 1.2 diverged from 1.1 (own DOM "uDOM", `textArea` wrapping, `<video>`/`<audio>`), and browsers never implemented Tiny 1.2 as a whole. Once phones ran full desktop engines (iPhone 2007, Android 2008) the profile's reason disappeared. resvg explicitly does not and will not support Tiny 1.2. | | [Wikipedia](https://en.wikipedia.org/wiki/SVG); [resvg README](https://github.com/linebender/resvg/blob/main/README.md) (README prose only) | 2003-2008 | Low (cause) / High (resvg) |
| 1.5 | SVG 2 publication history: 7 WDs (2012-08-28 to 2015-09-15); CR Snapshots 2016-09-15, 2018-08-07, 2018-10-04; then **nothing for eight years** until a CR Snapshot on 2026-10-06. That snapshot says advancement to Recommendation is expected no earlier than 2026-12-01. | 11 publications | [SVG 2 history](https://www.w3.org/standards/history/SVG2/); [SVG 2 CR 2026-10-06](https://www.w3.org/TR/SVG2/) | 2012-2026 | High |
| 1.6 | At-risk features in the 2026 CR: multiple `title`/`desc` for localisation; nested links; **`vector-effect` values other than `non-scaling-stroke`**. | 3 features | [SVG 2 status](https://www.w3.org/TR/SVG2/) | 2026-10-06 | High |
| 1.7 | **Cut or deferred from SVG 2** (Changes appendix): `mesh`/`meshrow`/`meshpatch` and `hatch`/`hatchpath` deferred to a future version and `solidcolor` removed, all under issue #449 (the 2018 re-CR cut of unimplemented features); `shape-inside`/`shape-subtract` "postponed to a future CSS Shapes level"; `vector-effect` `screen`/`viewport` removed; `z-index` added then deferred; multiple fill/stroke paints rolled back; `child` paint values removed; `stroke-linejoin` `miter-clip`/`arcs` removed (March 2026, issue #1065); `SVGUnknownElement` removed "due to lack of implementations"; `buffered-rendering` replaced by a note on `will-change`; `kerning` removed. | ~20 items | [SVG 2 Changes](https://www.w3.org/TR/SVG2/changes.html) | 2016-2026 | High |
| 1.8 | **Removed SVG 1.1 features**: SVG fonts chapter (`font`, `glyph`, `hkern` …); `tref`, `altGlyph*`, `glyphRef`; `xml:base`; `externalResourcesRequired`, `requiredFeatures`, `baseProfile`, `version`; `SVGPathSeg*` and `SVGAnimatedPathData`; `SVGPaint`/`SVGColor`/`SVGICCColor`; `SVGElementInstance`, `instanceRoot`/`animatedInstanceRoot` ("removed from all browsers"); `getTransformToElement`; `zoomAndPan`, `SVGZoomEvent`; `cursor` element; `xlink:type/role/arcrole/show/actuate`; the DTD ("SVG 2 is not defined in terms of a DTD"); Java and ECMAScript bindings (replaced by Web IDL). Filters, colour and masking moved to Filter Effects, CSS Color and CSS Masking; animation moved to a separate module. | | [SVG 2 Changes](https://www.w3.org/TR/SVG2/changes.html) | 2016-2026 | High |
| 1.9 | **SVG Integration** (FPWD 2014-04-17, never advanced) defines five processing modes: dynamic interactive (script, external refs, animation, interaction); animated; **secure animated** (animation only, no script, no external refs); static; **secure static**. `img` and CSS images use secure animated (or secure static when animation cannot run); resource documents (`use`, `fill`, `filter`, `mask`, `pattern` …) and OpenType SVG glyph documents use secure animated; `iframe`/`object`/`embed` and top-level use dynamic interactive. "Animated" and "static" are used by no referencing mode and flagged for removal. | 5 modes, 6 referencing modes | [SVG Integration](https://www.w3.org/TR/svg-integration/) | 2014-04-17 | High |
| 1.10 | **SVG Native**: a strict profile of SVG 1.1 for native apps and system libraries, aligned with the OpenType `SVG ` table subset; editors Apple and Adobe; still an Editor's Draft (3 July 2026 version) and listed in the charter as unpublished with completion undetermined. | | [SVG Native ED](https://w3c.github.io/svgwg/specs/svg-native/); [2019-04-15 minutes](https://www.w3.org/2019/04/15-svg-minutes.html) | 2019-2026 | Medium |

**Implications:** Izcal targets SVG 2 as of the 2026-10-06 CR plus Filter Effects, CSS Masking and CSS Color, and treats the Changes appendix as a checklist of things **not** to build (fonts, `tref`, PathSeg, DTD) and things whose status is volatile (at-risk `vector-effect` values, deferred paints). The secure-animated / secure-static modes map directly onto MD-D20's scriptless media process.

## 2. Browser support history

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **Adobe SVG Viewer (ASV)** was the de-facto SVG renderer for IE and Netscape from about 1999. In 2006 Adobe announced EOL; after public complaint it moved support end to 2008-01-01 and download end to 2009-01-01, then (December 2006) kept the download available. Support finally ended 2009-01-01. Adobe's FAQ said ASV was not supported on Vista and its source would not be released. No official reason was published; commentators linked it to Adobe's acquisition of Macromedia (Flash) in December 2005. | EOL 2009-01-01 | [codedread 2006-10-11](https://www.codedread.com/blog/archives/2006/10/11/adobe-extends-eol-for-svg-viewer-whew/); [Netgate forum](https://forum.netgate.com/post/22444) | 2006-2009 | Medium (dates) / Low (Flash motive) |
| 2.2 | Native support: Konqueror 3.2 (KSVG) February 2004; Opera 8 (SVG Tiny) 2005, Opera 9 (Basic), 9.5 (partial Tiny 1.2); Gecko incomplete SVG 1.1 Full from Firefox 1.5 (2005), filters with Firefox 3 (Gecko 1.9); WebKit from 2006 (KSVG2 code was merged into WebKit). IE8 and older had none; **IE9 (2011-03-14)** added basic SVG, IE10 filters. By 2011 every major desktop browser had some SVG. | | [Wikipedia: SVG](https://en.wikipedia.org/wiki/SVG) | 2004-2011 | Low |
| 2.3 | Inline `<svg>` in `text/html` arrived with the HTML5 parser: Chrome 7, Firefox 4, IE9, Safari 5.1, Opera 11.6, Edge 12. Before that, SVG in HTML needed `<object>`, `<embed>` or `<iframe>`, or XHTML served as XML. | 6 engines, 2010-2011 | [caniuse svg-html5](https://caniuse.com/svg-html5) | 2010-2011 | High |
| 2.4 | **Why a decade:** (a) the market-leading browser (IE, ~90% share in the mid-2000s) shipped VML, not SVG, until 2011; (b) the plugin route depended on one vendor (Adobe) that came to own the competing Flash; (c) SVG was XML-only and could not be pasted into the HTML most authors wrote (2.3); (d) the specification was large ("SVG 1.1 is almost 900 pages", resvg README) and early engines were partial. | | [resvg README](https://github.com/linebender/resvg/blob/main/README.md) (README prose only); rows 2.1-2.3 | | Low (synthesis) |

**Implications:** SVG became usable when it became native, parsed by the HTML parser and present in every engine. Izcal ships no plugin path and must parse inline SVG through its HTML parser's foreign-content rules from the start.

## 3. Postmortems: removed and deprecated features

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **SVG fonts.** Supported in Chrome 4-37, Opera 9-24, Safari 3.2 to today; **never in Firefox, IE or Edge**. Removed from Chrome 38 (2014) and from SVG 2. Firefox instead pushed the OpenType `SVG ` table (SVG-in-OpenType colour glyphs). The SVG WG's recollection (2015) is that the deprecation debate started when Microsoft said it would implement neither SVG fonts nor SMIL. Mozilla's 2010 objections cited low usage and incomplete implementations. | Chrome 4-37 | [caniuse svg-fonts](https://caniuse.com/svg-fonts); [SVG WG minutes 2015-04-30](https://www.w3.org/2015/04/30-svg-minutes.html); [www-svg 2010-06](https://lists.w3.org/Archives/Public/www-svg/2010Jun/0031.html) | 2014 | High (support) / Medium (reasons) |
| 3.2 | **SMIL deprecation intent (Blink)**, posted 2015-04-29 by Philip Rogers. Reasons: never implemented in all major browsers; spec stalled since 2008; "significant complexity to Blink" including a tear-off rewrite and a hard Oilpan integration; CSS animations and Web Animations as replacements. Usage: **0.0403% of page views**, "low but stable"; **0.006%** inside `<img>` SVGs (where script polyfills cannot run). Approved (one reviewer suggested removal in M45); objections that embedded animated SVG had no replacement because script and Web Animations do not run in SVG images. | 0.0403%, 0.006% | [blink-dev thread](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/YGEJBsjUAwAJ) | 2015-04-29 | High |
| 3.3 | **The suspension.** Chrome suspended the deprecation (reported as 2016) after developer pushback that some SVG animation capabilities (path morphing, animation inside `<img>`) had no CSS or Web Animations equivalent. SMIL still ships in Chrome, Firefox and Safari. The suspension message itself was not found. | | [ZKM Format Obsolescence](https://extinction.zkm.de/events/smil-browser-support-collapse) (secondary) | 2015-2016 | Low |
| 3.4 | **SVG WG reaction (2015-04-30):** resolved to move SVG animation into a separate spec. Microsoft: "IE will never support it", support is "ultimately a business decision". Mozilla's Brian Birtles: Web Animations has no declarative syntax, so path-point animation and SVG-in-OpenType glyph animation would be lost. A WebKit participant wanted to remove the **animVal/baseVal tear-off support, "costly to maintain"**. | | [SVG WG minutes 2015-04-30](https://www.w3.org/2015/04/30-svg-minutes.html) | 2015-04-30 | High |
| 3.5 | **SVGPathSeg API.** `createSVGPathSeg*` and `pathSegList` removed in **Chrome 48** and **Firefox 59**; Safari still ships them. `getPathSegAtLength` lasted to Chrome 62 and Firefox 97. The replacement, the SVG Paths spec's `getPathData()`/`setPathData()`/`getPathSegmentAtLength()` (plain `{type, values}` dictionaries, optional `normalize` to absolute M/L/C/Z), shipped first in **Firefox 137** (2025). Chrome and Safari had not shipped it; Blink posted an Intent to Prototype on 2026-06-05. So for about nine years (2016-2025) no browser except Safari had any structured path API. | Chrome 48, Firefox 59, Firefox 137 | [MDN SVGPathElement compatibility](https://developer.mozilla.org/en-US/docs/Web/API/SVGPathElement#browser_compatibility); [MDN getPathData](https://developer.mozilla.org/en-US/docs/Web/API/SVGPathElement/getPathData); [Blink Intent to Prototype](https://groups.google.com/a/chromium.org/g/blink-dev/c/TVIVah_HHcg/m/J0o47z9lAgAJ) | 2016-2026 | High (versions) / Medium (I2P date) |
| 3.6 | `instanceRoot`/`SVGElementInstance` removed from SVG 2 "as they have been removed from all browsers"; the `use` shadow tree is now a real shadow tree. `getTransformToElement` removed. | | [SVG 2 Changes](https://www.w3.org/TR/SVG2/changes.html) | | High |
| 3.7 | `tref` (text by reference), `altGlyph*`, `glyphRef` removed from SVG 2; `tref` was never implemented in Firefox and was dropped from WebKit/Blink. | | [SVG 2 Changes](https://www.w3.org/TR/SVG2/changes.html) | | High (spec) / Low (engines) |
| 3.8 | `enable-background` and `BackgroundImage`/`BackgroundAlpha` filter inputs were never implemented by Gecko/WebKit/Blink (needed an off-screen copy of everything painted behind). Filter Effects deprecates them; CSS `isolation` and `mix-blend-mode`/`backdrop-filter` took their role. Presto, Batik and Inkscape-era tools implemented them. | | | | Low |
| 3.9 | `glyph-orientation-vertical` was replaced by CSS Writing Modes `text-orientation`; `glyph-orientation-horizontal` dropped. `baseProfile`, `requiredFeatures` (feature strings that never told the truth) and `externalResourcesRequired` (load-gating no engine honoured consistently) were removed. `xml:base` removed (Firefox had supported it and later removed it). | | [SVG 2 Changes](https://www.w3.org/TR/SVG2/changes.html) | | High (spec) / Low (engine detail) |
| 3.10 | **Text rewrite in Blink.** Blink moved SVG text layout onto its LayoutNG inline layout engine (about Chrome 92, 2021), reportedly to share shaping, bidi and line machinery with HTML and to fix long-standing per-character positioning bugs. No primary prose was found this session. | | | ~2021 | Low |

**Implications:** Every removal above had one of two causes: **no second implementation** (fonts, `tref`, mesh, hatch, `enable-background`) or **API shape cost** (PathSeg live lists, tear-offs, `SVGElementInstance`). Izcal should not build features with a single implementation, and should implement SMIL only if the animation spec can host it on one timing model, because SMIL is the only animation that runs inside `<img>` SVG (3.2).

## 4. Non-browser renderers (prose only)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **librsvg** (GNOME): port from C to Rust ran from about 2016 to 2020 (maintainer's own dating); "almost 100% Rust" announced 2019-01-10, remaining C being thin API stubs and GObject boilerplate. CSS parsing moved to Rust (2.47.1) using Servo's CSS crates, dropping libcroco. | ~4 years | [Librsvg is almost rustified now](https://viruta.org/librsvg-is-almost-rustified-now.html); [CSS in librsvg is now in Rust](https://viruta.org/css-in-librsvg-is-now-in-rust-courtesy-of-mozilla-servo.html) | 2019-01-10 | High |
| 4.2 | What the port exposed: "librsvg never had a representation of property values that actually matched the SVG/CSS specs" (no `inherit` vs unspecified distinction; non-inherited properties handled by workarounds). **Filters** called "a huge barrier". | | [Three big things happening in librsvg](https://viruta.org/three-big-things-happening-in-librsvg.html) | 2018-05-21 | High |
| 4.3 | The last C line (2.40.x, deprecated 2017) had a **memory-unsafe path parser** and misread compact arc flags (`a1 1 0 00 1 1`), breaking current Adwaita icons. "Do not use librsvg 2.40.x." | | [Do not use librsvg 2.40.x](https://viruta.org/do-not-use-librsvg-2.40.x.html) | 2020-11-27 | High |
| 4.4 | **resvg** (now under linebender): aims at "the whole SVG spec" but only the **static subset**: no `a`, `script`, `view`, `cursor`, no events, **no animation and no plans for it**; no system text rendering (bundled shaping); Unicode only; no Tiny 1.2. About **1,600 SVG-to-PNG regression tests**, published and engine-neutral. Almost no `unsafe`; guards against endless loops and recursion stack overflow. Renderers below 25% of its suite (wxSvg, LunaSVG, nanosvg) are excluded from its comparison. | ~1,600 tests | [resvg README](https://github.com/linebender/resvg/blob/main/README.md) (README prose only) | 2026 | High |
| 4.5 | **Batik** (Apache, Java) was the most complete SVG 1.1 implementation of the 2000s, including SMIL, scripting and SVG fonts; Schepers listed it in 2015 as a SMIL implementation. **Inkscape** is the main producer of SVG and emits its own namespaced extensions (`sodipodi:`, `inkscape:`) and adopted mesh gradients and `shape-inside` text from SVG 2 drafts that browsers never shipped. | | [SVG WG minutes 2015-04-30](https://www.w3.org/2015/04/30-svg-minutes.html) | | Low (Inkscape detail) |

**Implications:** Rust rewrite experience says the hard parts are the **CSS property model**, **filters** and **robust path parsing**; resvg's suite is a ready black-box oracle for static rendering (license to be checked before any test data is vendored).

## 5. Era constraints and their lessons

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **XML-first.** SVG 1.x was an XML vocabulary with a DTD, `xml:base`, `xml:space`, XLink, and draconian XML error handling. Authors writing HTML could not use it inline until the HTML5 parser added foreign-content rules (2.3). SVG 2 drops the DTD and XLink's extra attributes and deprecates `xlink:href` in favour of `href`. | | [SVG 2 Changes](https://www.w3.org/TR/SVG2/changes.html) | | High |
| 5.2 | **Namespaces** remain: the HTML parser must switch into the SVG namespace, case-fix camelCase element and attribute names, and handle `foreignObject` integration points. | | (HTML Standard, not re-read this session) | | Medium |
| 5.3 | **SVGAnimated\*** splits each attribute into `baseVal` and `animVal` live objects ("tear-offs") so the DOM could expose SMIL's animated value. Blink cited the tear-off rewrite as a SMIL cost; WebKit called the support "costly to maintain"; the WG asked for a gap analysis rather than removal because the interfaces are web-exposed. In practice `animVal` usually returns base values or is little used. | | [blink-dev](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/YGEJBsjUAwAJ); [minutes](https://www.w3.org/2015/04/30-svg-minutes.html) | 2015 | High (stated costs) / Low (usage) |
| 5.4 | Presentation attributes versus CSS properties: SVG 2 makes most geometry (`x`, `r`, `d` …) CSS properties, so the same value can come from attribute, stylesheet, SMIL and Web Animations; librsvg's port (4.2) shows how easily a home-grown property model diverges. | | [SVG 2 Changes](https://www.w3.org/TR/SVG2/changes.html) | | Medium |

**Implications:** Izcal's DOM should hold one value per property in the style system and expose `SVGAnimated*` as thin views over it, never as separate stored copies; the HTML parser owns SVG namespace entry.

## Open questions for the spec

1. SMIL: in the animation spec or in SVG? It is the only animation that runs in `<img>`/secure-animated mode (3.2), which is MD-D20's media process with no JS heap.
2. Which at-risk SVG 2 features (1.6) to implement before the Recommendation lands (expected no earlier than 2026-12-01)?
3. Ship `getPathData`/`setPathData` (Firefox 137 only) and skip SVGPathSeg (Safari only)?
4. SVG fonts: none (Chrome, Firefox) or Safari-compatible? Recommend none.
5. Is the resvg test suite's licence compatible with Izcal's MIT-only rule for vendored test data, or should it be used only as an external oracle?
6. Black-box measurement: what does each engine return for `animVal` during a SMIL animation, and how many sites read it?
7. Blink's SVG-text LayoutNG reasons: find the prose design document or launch note.

## Claims not verified

- 1.1: PGML/VML submission details and VML in IE5 (secondary).
- 1.3, 1.4: Recommendation dates via Wikipedia; Tiny 1.2 uDOM/`textArea` divergence and the cause of mobile profiles fading are recollection.
- 2.1: the Flash/Macromedia motive for ASV's EOL.
- 2.2: per-browser first-support versions and KSVG2 into WebKit.
- 2.4: the "why a decade" synthesis and IE market share.
- 3.3: the date and reasons of the SMIL suspension (only secondary source).
- 3.7, 3.8, 3.9: engine-level history of `tref`, `enable-background`, `xml:base` and `glyph-orientation-*`.
- 3.10: Blink's SVG text move to LayoutNG, its milestone and reasons.
- 4.5: Batik's completeness and Inkscape's adoption of SVG 2 draft features.
- 5.3: how much `animVal` is used.
