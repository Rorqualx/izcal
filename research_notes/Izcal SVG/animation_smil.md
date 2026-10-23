# Izcal SVG: animation_smil

Compiled 20 October 2026. Primary sources were preferred (specifications, maintainers' posts, mailing lists, chromestatus use counters, wpt.fyi results, MDN compatibility data).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data, or the researcher's reading of a spec where the spec does not say it in so many words; **Low** = secondary source only, or recollection.

M-1 note: **no source code was opened or cited**. That covers every browser engine and every third-party project (Lottie, SVGator, sanitizers and others). The evidence is prose: specifications, the blink-dev thread, www-svg mails, Brian Birtles' blog, Bugzilla prose, Chrome's developer blog and vendor help pages. It also uses black-box data: chromestatus use-counter JSON (`/data/featurepopularity`, `/data/csspopularity`, timelines), the wpt.fyi results API for the 2026-10-19 stable runs, and MDN's browser-compatibility data, cited through MDN's reference pages. Search results surfaced two Chromium code reviews; they were not opened. Where a behaviour is stated only in engine source, it is listed under "Open questions" as a black-box measurement.

**Method notes**
- 21 web searches and fetches were used. Specs and data were also downloaded with plain HTTP and searched as text: Web Animations 1 and 2, SVG Animations Level 2, SVG 1.1 chapter 19, SVG 2 appendix D and chapter 2, the W3C TR status lines and the HTML images section.
- The blink-dev pages for the SMIL intent were truncated after 2015-04-30. The 2016 suspension text comes from a www-svg forward of Google's post. The thread itself could not be read past 2015-04-30.
- The wpt.fyi runs are Chrome 155.0.8059.84, Firefox 158.0, Safari 27.0 and Edge 155, all dated 2026-10-19. The reported pass counts are subtest sums from the search API. Reftests that report 0/0 are not counted.

## 0. Chronology at a glance

| Date | Event | Row |
|---|---|---|
| 1998-06-15 | SMIL 1.0 Recommendation | 1.1 |
| 2001-09-04 | SMIL Animation Recommendation; SVG 1.0 animation built on it | 1.2 |
| 2006 (Opera 9) | First SVG SMIL in a shipping browser (MDN data: Opera 9) | 1.5 |
| 2008-12 | SMIL 3.0 (2008-12-01) and SVG Tiny 1.2 (2008-12-22) Recommendations | 1.3 |
| ~2009 | WebKit (Safari 3.1 to 4) and Chrome (Chrome 2) ship `<animate>` | 1.5 |
| 2011-03 | Firefox 4 ships SMIL, built on Brian Birtles' work | 1.6 |
| 2011-08-16 | SVG 1.1 Second Edition, chapter 19 Animation | 1.4 |
| 2013-06-25 | First Web Animations Working Draft | 2.1 |
| 2015-04-29 | Blink "Intent to deprecate: SMIL" | 1.8 |
| 2015-05-01 | Birtles, "What do we do with SMIL?" | 2.3 |
| 2016-08 (forwarded 2016-08-17) | Chrome suspends the deprecation | 1.10 |
| 2020-01 | Chromium Edge 79 inherits SMIL; IE and EdgeHTML never had it | 1.7 |
| 2021 (Chromium 89) | Chrome composites SVG animations | 4.5 |
| 2023-06-05 | Latest Web Animations 1 Working Draft | 2.1 |
| 2025-09-14 | SVG Animations Level 2 Editor's Draft (Birtles, editor) | 1.4 |

## 1. History, support and Chrome's deprecation attempt

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | SMIL 1.0 became a W3C Recommendation on 15 June 1998. It was a multimedia presentation language, not an animation language. | 1998-06-15 | [REC-smil-19980615](https://www.w3.org/TR/1998/REC-smil-19980615/) | 1998 | High |
| 1.2 | **SMIL Animation** (editors from Microsoft and Intel) became a Recommendation on 4 September 2001. It was written as a module for host languages to embed. SVG's animation elements are defined "in terms of SMIL Animation". | 2001-09-04 | [SMIL Animation](https://www.w3.org/TR/smil-animation/) | 2001 | High |
| 1.3 | The final SMIL is SMIL 3.0 (Recommendation, 2008-12-01). SVG Tiny 1.2 (Recommendation, 2008-12-22) added `<discard>` and `timelineBegin`. | 2008 | [SMIL 3.0](https://www.w3.org/TR/SMIL3/); [SVG Tiny 1.2](https://www.w3.org/TR/SVGTiny12/) | 2008 | High (dates); Medium (Tiny features) |
| 1.4 | SVG 1.1 SE chapter 19 defines `animate`, `set`, `animateMotion`, `animateColor` and `animateTransform`. **Document begin** is "the exact time at which the 'svg' element's SVGLoad event is triggered". SVG 2 (CR Snapshot 2026-10-06) moves the elements to a separate spec, **SVG Animations Level 2** (Editor's Draft 2025-09-14, editor Brian Birtles). That draft says it exists "to separate out SVG's animation features into a separate document". It keeps the load-event document begin and adds `timelinebegin` (`loadend`/`loadbegin`). | ED dated 2025-09-14 | [SVG 1.1 ch. 19](https://www.w3.org/TR/SVG11/animate.html); [SVG Animations L2](https://svgwg.org/specs/animations/); [SVG 2 App. D](https://www.w3.org/TR/SVG2/animate.html) | 2011, 2025 | High |
| 1.5 | MDN compatibility data lists `<animate>` support from Chrome 2, Safari 3.1, Firefox 4 and Opera 9. `<animateMotion>` arrives in Chrome 19 and Safari 6. Plain `href` on animation elements (not `xlink:href`) arrives in Chrome 50, Firefox 51 and Safari 12.1. | versions | [MDN: animate](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/animate); [MDN: animateMotion](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/animateMotion) | 2026 data | Medium |
| 1.6 | Gecko's SMIL builds on Brian Birtles' work. A pref (`svg.smil.enabled`) could turn it off until Firefox 21; from Firefox 21 it is always on. Firefox 4 shipped in March 2011 (recollection). | Fx 4, Fx 21 | [MozillaWiki: SMIL](https://wiki.mozilla.org/SMIL) | n/a | Medium |
| 1.7 | Internet Explorer and EdgeHTML never implemented SMIL. The intent's own compatibility note says "Internet Explorer does not support SMIL which limited its use for critical functionality". Chromium-based Edge inherits SMIL (MDN marks Edge as "mirror" of Chrome). | n/a | [blink-dev intent](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/59rZqirUQNwJ) | 2015-04-29 | High (IE); Medium (EdgeHTML) |
| 1.8 | **Intent to deprecate**: Philip Rogers posted it on 29 April 2015, with Eric Willigers as co-contact. It reads: "We intend to deprecate SMIL animations in favor of CSS animations and Web animations". The stated motivation is to "double-down on support and tooling" for the integrated model, plus the complexity SMIL adds to Blink. Search snippets name two large tear-off rewrites and a hard Oilpan integration. Usage was "low but stable at **0.0403%** of pageviews", and **0.006%** in SVG used as `<img>`, where polyfills cannot run. The suggested alternative was Willigers' SMIL polyfill built on Web Animations. Three LGTMs came the same day; Chris Harrelson suggested removal in M45. | 0.0403%; 0.006% | [blink-dev intent](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/59rZqirUQNwJ) | 2015-04-29 | High (post); Medium (rewrite detail) |
| 1.9 | Early objections, read in the thread: Elliott Sprehn asked how authors would animate several embedded SVG files without SMIL. Willigers replied that CSS animations work in SVG documents, but script, and so Web Animations, does not run in embedded SVG images. Tobi Reif warned that existing content would break. The SVG WG discussed the intent on 2015-04-30. | n/a | same thread; [SVG WG minutes 2015-04-30](https://www.w3.org/2015/04/30-svg-minutes.html) | 2015-04-29/30 | High (thread); Medium (minutes, snippet) |
| 1.10 | **Suspension.** Google's post, forwarded to www-svg on 17 August 2016, says: "it's clear that there are use cases serviced by SMIL that just don't have high-fidelity replacements yet. As a result, we've decided to suspend our intent to deprecate and take smaller steps toward other options." No specific feature is named in the forwarded excerpt. The commonly cited ones (path morphing via `d`, animation inside `<img>`) are named in rows 1.9 and 2.3, not in the suspension text. | 2016-08 | [www-svg 2016Aug/0037](https://lists.w3.org/Archives/Public/www-svg/2016Aug/0037.html); [0038](https://lists.w3.org/Archives/Public/www-svg/2016Aug/0038.html) | 2016-08-17 | High (quote); Medium (exact post date) |
| 1.11 | Chrome's deprecation console warning was later removed, and SMIL ships unflagged in all three engines in 2026. The removal date of the warning was not found. | n/a | wpt.fyi runs (section 3); recollection | n/a | Low (warning) |

**Implications:** SMIL is a frozen 2001 design maintained by one editor's draft. It survived in Chrome only because nothing else animates attributes such as `d` and runs inside script-free images. Izcal must support it, because content depends on it (section 6). Nothing requires it to be a separate engine.

## 2. SMIL and Web Animations: stated relationships

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | Web Animations 1 (WD 2023-06-05; earlier WDs 2013-06-25 to 2022-09-08) says: "The Web Animations model is intended to provide the features necessary for expressing CSS Transitions, CSS Animations, and SVG [SVG11]". It adds that SVG 1.1 animation is "defined in terms of SMIL Animation… It is intended that by defining SVG's animation features in terms of the Web Animations model, the dependency between SVG and SMIL Animation can be removed." | n/a | [Web Animations 1 §1.1–1.2](https://www.w3.org/TR/web-animations-1/) | 2023-06-05 | High |
| 2.2 | WA1 deliberately carries SMIL hooks. **End delay** "is included here for the purpose of representing the min attribute in SVG". The timing hierarchy is "shallow" in Level 1; group and sequence effects, the `iterationComposite: accumulate` operation and custom effects are in **Web Animations Level 2** (Editor's Draft). | n/a | [WA1 §4.5.3](https://www.w3.org/TR/web-animations-1/); [WA2](https://drafts.csswg.org/web-animations-2/) | 2023 / ED | High |
| 2.3 | Birtles (2015-05-01) says he spent "quite a lot of time working on SMIL in Gecko". He started Web Animations partly because Microsoft wanted a unified model and was open to a script implementation of SMIL. He expected Blink to *reimplement* SMIL in JS rather than deprecate it. He lists gaps that no CSS feature closes: path (`d`) interpolation, animation grouping, additive lists, and animating glyphs in SVG-in-OpenType where script cannot run. He also tried to make `viewBox` a property and hit cycles with media queries. | n/a | [birtles.blog](https://birtles.blog/2015/05/01/what-do-we-do-with-smil/) | 2015-05-01 | High |
| 2.4 | No public prose design document was found for "SMIL on top of Web Animations" in Chrome. The only stated artefact is Willigers' JS polyfill (row 1.8). Whether any engine runs SMIL internally on its Web Animations model is **unknown from prose**. | n/a | searches this session | n/a | Medium (absence) |
| 2.5 | SVG 2 Appendix D lists four ways to animate: SVG animation elements, CSS Animations, CSS Transitions and the Web Animations API. It says SVG "does not mandate support for any of these", but UAs that support them "are expected to support them for SVG documents". | n/a | [SVG 2 App. D](https://www.w3.org/TR/SVG2/animate.html) | 2026 | High |

**Implications:** The specs' own intent is that SMIL is a **front end on the Web Animations model**, not a parallel engine. The gaps are timing (Level 2 and beyond) and non-property attributes. The feature table at the end sets out what Izcal must add.

## 3. Interop, APIs, images, security, accessibility

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **WPT `svg/animations`**: 321 files and 341 subtests. Passing subtests: Chrome **328** (96.2%), Firefox **315** (92.4%), Safari **265** (77.7%). Fully passing files: 310, 299 and 253. | 2026-10-19 runs | [wpt.fyi svg/animations](https://wpt.fyi/results/svg/animations?label=master&label=stable) | 2026-10-19 | High |
| 3.2 | Where they fail: Chrome fails 11 files, mostly `animateMotion` basic shapes and the unitType of `SVGAngle`/`SVGLengthList`. Firefox fails 22, mostly `.tentative` path-interpolation edge cases, `animateMotion` degenerate paths and seeking events. Safari fails 68, mostly path interpolation, **syncbase and event timing** (`cyclic-syncbase-*`, `dependent-begin-on-syncbase`, `seeking-events-1..8`, `beginelement-instance-time-1`) and `getStartTime`. | file lists | same | 2026-10-19 | High |
| 3.3 | **Edge semantics are the interop risk**: syncbase chains, cycles, seeking and event ordering. These are the parts with no Web Animations counterpart (table at end). | n/a | inference from 3.2 | n/a | Medium |
| 3.4 | `SVGAnimationElement` (`beginElement[At]`, `endElement[At]`, `getStartTime`, `getCurrentTime`, `getSimpleDuration`, `targetElement`) has shipped since Chrome 2, Firefox 4 and Safari 4. The `beginEvent`/`endEvent`/`repeatEvent` handler attributes arrived in Chrome 35, Firefox 93 and Safari 26.2–26.5. `SVGSVGElement.pauseAnimations`/`setCurrentTime` are listed even for IE 9; the method existed there without SMIL. WPT `svg/animations/scripted`: Chrome 18/18, Firefox 17/18, Safari 13/18. | versions | [MDN: SVGAnimationElement](https://developer.mozilla.org/en-US/docs/Web/API/SVGAnimationElement); [MDN: SVGSVGElement](https://developer.mozilla.org/en-US/docs/Web/API/SVGSVGElement); wpt.fyi | 2026 | Medium (MDN); High (WPT) |
| 3.5 | SVG Animations L2 defines `pauseAnimations`, `unpauseAnimations`, `animationsPaused`, `getCurrentTime` and `setCurrentTime(float seconds)` on the **outermost svg element's timeline**. `getCurrentTime` returns 0 before the timeline begins. The timeline is per SVG document fragment, separate from the document timeline. | n/a | [SVG Animations L2 §5.8](https://svgwg.org/specs/animations/) | 2025-09-14 | High |
| 3.6 | **`<discard>`** (from SVG Tiny 1.2; listed among SVG 2's animation elements) removes its target when its begin time arrives. Chromestatus counters: `SVGSMILDiscardElementParsed` 0.0008% of page loads; `SVGSMILDiscardElementTriggered` ~0 (last data 2023-02-16). Firefox and Safari support is not confirmed this session; WPT "discard" searches are polluted by unrelated tests. | 0.0008% | [chromestatus metrics](https://chromestatus.com/metrics/feature/popularity) | 2026-10-18 | High (counter); Low (support) |
| 3.7 | **SVG as image.** SVG 2 defines **secure animated mode**: no script, no external references, declarative animation yes, no interactivity. SVG in an `<img>` or a CSS `<image>` uses secure animated mode "if the embedding document supports declarative animation". Documents referenced by `use` or paint servers are processed in secure static mode, so their animations do not run. HTML requires an image that is "non-interactive, optionally animated… neither paged nor scripted". | n/a | [SVG 2 §2.2.4, §2.3](https://www.w3.org/TR/SVG2/conform.html); [HTML images](https://html.spec.whatwg.org/multipage/images.html) | 2026 | High |
| 3.8 | SMIL and CSS animations are the **only** ways to animate an SVG image (row 1.9; SVGator's docs say script-driven exports do not play in image tags). MDN lists SVG favicons from Chrome 80; whether favicons animate is a per-browser UI choice, not stated in prose found. | n/a | [blink-dev](https://groups.google.com/a/chromium.org/g/blink-dev/c/5o0yiO440LM/m/59rZqirUQNwJ); [SVGator help](https://www.svgator.com/help/export-and-file-formats/js-animations-vs-css-animations) | 2015; 2026 | High (img); Low (favicons) |
| 3.9 | **When an image's timeline starts** (at the image document's load, at first paint, or per `<img>` instance) and **whether two `<img>` showing the same URL share a timeline** are not specified anywhere read. Chrome counts `AnimatedImageUsedMoreThanOnce` at 22.1% (all formats). | 22.1% | chromestatus | 2026-10-18 | Medium (gap) |
| 3.10 | **Throttling**: no published prose was found on whether engines throttle or pause off-screen SMIL, or SMIL in hidden tabs. Black-box question. | n/a | none found | n/a | Low |
| 3.11 | **Security**: animating `href` through `<animate values="…;javascript:…">` or `<set>` on an `<a>` is a documented XSS vector against sanitizers (PortSwigger, Gareth Heyes, 2020-01-28). Its lab slug mentions "event handlers and href attributes blocked", which means sanitizer filtering. No browser vendor prose was found that defines which attributes SMIL may not animate. In SVG 1.1, `href` on animation elements is "Animatable: no", and `<time>` values are not animatable. Sanitizer advisories in 2026 (a CVE in sanitize-html; SiYuan) recommend rejecting `attributeName=href` and `xlink:href`. | n/a | [PortSwigger](https://portswigger.net/research/svg-animate-xss-vector); [Corgea advisory](https://corgea.com/advisories/vulnerabilities/CVE-2026-84371) | 2020; 2026 | High (vector); Low (CVE detail) |
| 3.12 | **Accessibility**: no mechanism ties SMIL to `prefers-reduced-motion`. CSS cannot pause SMIL, so authors hide elements or swap images with `<picture>` media sources. A WebKit bug (283894) shows the reduced-motion override not reaching CSS inside an embedded SVG. No svgwg issue was found. | n/a | [Elijah Manor](https://elijahmanor.com/prefers-reduced-motion); [Smashing 2026-08](https://smashingmagazine.com/2026/08/timing-charts-blueprint-smil-animations/); [WebKit 283894](https://bugs.webkit.org/show_bug.cgi?id=283894) | 2026 | Medium |

**Implications:** Most SMIL use is in images, where no script runs. That is exactly the media-process context in MD-D20. Izcal therefore needs SMIL **without JS** in the scriptless media process. Izcal must also define, and spec, three things browsers leave open: when an image's timeline starts, whether `<img>` instances share it, and an attribute deny list (`href` to `javascript:` URLs, `on*` attributes).

## 4. CSS animations and transitions on SVG

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | CSS can animate only **properties**. SVG 2 made many geometry attributes properties (`cx`, `cy`, `r`, `rx`, `ry`, `x`, `y`, `width`, `height`, `d`), so CSS animations can drive them. Non-property attributes (`viewBox`, `points`, gradient and filter-primitive attributes, `href`) stay SMIL-only or script-only. | n/a | SVG 2; [Birtles 2015](https://birtles.blog/2015/05/01/what-do-we-do-with-smil/) | 2015–2026 | Medium |
| 4.2 | **`d` as a CSS property**: Chrome 52+, Firefox 97+, Safari "parses but has no effect" (MDN). WPT `svg/path/property`: Chrome 210/234, Firefox 225/234, Safari **0/234**. | 2026-10-19 | [MDN: d](https://developer.mozilla.org/en-US/docs/Web/CSS/d); [wpt.fyi svg/path/property](https://wpt.fyi/results/svg/path/property) | 2026 | High |
| 4.3 | `viewBox` is not a property. Birtles' attempt to make it one hit "cyclic dependencies with media queries". | n/a | [Birtles blog comments](https://birtles.blog/2015/05/01/what-do-we-do-with-smil/) | 2015-05-01 | High |
| 4.4 | CSS-property use counters (share of page loads, 2026-10-18): `stroke-dashoffset` 31.3%, `r` 8.56%, `cx` 7.66%, `cy` 7.62%, `transform-box` 3.62%, `x` 0.98%, `y` 0.98%, `rx` 0.81%, `d` 0.69%, `ry` 0.47%, `offset-path` 0.45%, `offset-rotate` 0.35%. Whether these count presentation attributes too is not stated. | as listed | [chromestatus CSS popularity](https://chromestatus.com/metrics/css/popularity) | 2026-10-18 | High (numbers); Medium (meaning) |
| 4.5 | **Compositing**: Chrome's developer blog (Una Kravets, last updated 2021-02-22) says: "As of Chromium 89, Chrome will join the likes of Firefox to enable hardware-acceleration by default on SVG animations". Composited properties are `opacity`, `filter` and `transform`. Chrome's counters: `AnimationSvgTypes` 14.8%, `AnimationSvgCompositedTypes` 4.33%, `AnimationSvgNonCompositedTypes` 13.2%. These are probably CSS and WA animations on SVG elements; the counter definitions were not read. Safari's behaviour is not found in prose. | Chromium 89 | [Chrome blog](https://developer.chrome.com/blog/hardware-accelerated-animations); chromestatus | 2021; 2026 | High (Chrome claim); Medium (Firefox via Chrome); Low (Safari) |
| 4.6 | Firefox moved some compositor animation decisions to match Chrome, Edge and Safari (opacity runs on the compositor even when a transform animation on the same element cannot). Bug 1218620 shipped in Firefox 48. The bug is about HTML; SVG is not discussed. | Fx 48 | [Bugzilla 1218620](https://bugzilla.mozilla.org/show_bug.cgi?id=1218620) | ~2016 | High |

**Implications:** Lowering SMIL `animate` of a property onto the same effect stack as CSS lets SMIL share compositing. Izcal's integer rasterizer and GR-D3 determinism must define compositor sampling for SVG transform and opacity exactly as for HTML.

## 5. Determinism of animation time

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | A WA **document timeline** is inactive until the document's time origin exists, then is "monotonically increasing". It is advanced in "update animations and send events" with a frame timestamp `now`. | n/a | [WA1 §4.3](https://www.w3.org/TR/web-animations-1/) | 2023 | High |
| 5.2 | The SVG fragment timeline begins at the **svg load event** (or at load begin with `timelinebegin=loadbegin`). It can be paused and seeked per fragment. Its zero is therefore an offset into the document timeline that depends on network timing. | n/a | [SVG Animations L2](https://svgwg.org/specs/animations/) | 2025 | High |
| 5.3 | Values Izcal must pin for bit-identical output: (a) the frame timestamp, taken from the deterministic clock that also drives rAF and WA; (b) the fragment's begin offset, recorded as a document-timeline time at load; (c) event times for event-based begins, quantised to the frame that dispatched them; (d) seek and pause times; (e) image timeline start per §3.9; (f) the arithmetic for clock values, `keySplines` solving and paced distances, done in fixed precision. | n/a | derived from 5.1–5.2 | n/a | Medium |
| 5.4 | `setCurrentTime` takes a `float` (single precision) in seconds. WA times are doubles in milliseconds. Converting between them loses precision for long-lived pages. | n/a | SVG Animations L2 IDL; WA1 IDL | n/a | High |

**Implications:** Run one clock, the document timeline. Model each SVG fragment timeline as a child timeline with an origin offset and a pause/seek state. Sample SMIL in the same "update animations" step as WA, so a frame's style is a pure function of (DOM, timestamp).

## 6. Usage and the tools landscape

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | SMIL usage has grown about 60-fold since the intent. `SVGSMILElementInDocument` was 0.0425–0.0488% of page loads in April 2015 and is **2.896%** on 2026-10-18. Year-start values: 0.459% (2016), 0.089% (2017), 0.354% (2018), 0.577% (2019), 0.808% (2020), 1.28% (2021), 1.57% (2022), 1.86% (2023), 2.18% (2024), 2.46% (2025), 2.70% (2026). The jump to ~0.6% in mid-August 2015 is unexplained; it may be a counter change. | as listed | [chromestatus timeline, bucket 501](https://chromestatus.com/metrics/feature/timeline/popularity/501) | 2026-10-18 | High |
| 6.2 | SMIL inside images: `SVGSMILAnimationInImageRegardlessOfCache` is **1.673%** (2026-10-18); it was 0.05% at the start of 2016. | 1.67% | [bucket 768](https://chromestatus.com/metrics/feature/timeline/popularity/768) | 2026-10-18 | High |
| 6.3 | Detailed counters (2026-10-18): `SVGAnimationElement` 4.17%; `SVGSMILAnimationAppliedEffect` 2.33%; `SVGSMILElementInsertedAfterLoad` 1.36%; `SVGSMILBeginOrEndSyncbaseValue` **0.372%**; `SVGSMILBeginEndAnimationElement` 0.284%; `SVGSMILAdditiveAnimation` 0.153%; `SVGSMILPausing` 0.079%; `SVGSMILBeginOrEndEventValue` **0.0186%**; `SVGSMILCurrentTime` 0.0164%; `SVGSMILAnimationElementTiming` 0.00012%; `SMILElementHasRepeatNEventListener` ~0; `SVGAnimateColorElement` 0. | as listed | [chromestatus feature popularity](https://chromestatus.com/metrics/feature/popularity) | 2026-10-18 | High (numbers); Medium (exact meaning of each counter) |
| 6.4 | For scale: `V8Element_Animate_Method` (WA `element.animate`) 13.9%; SVG root in document 68.5%. SMIL is about one fifth as common as WA scripting. | as listed | same | 2026-10-18 | High |
| 6.5 | **SVGator** exports CSS or JavaScript, with JS recommended. Morph and path animation need the JS player. JS exports do not play in image tags, so SVGator recommends "CSS Only" there. SMIL is not offered. | n/a | [SVGator help](https://www.svgator.com/help/export-and-file-formats/js-animations-vs-css-animations) | 2026 | High |
| 6.6 | **Lottie** (bodymovin) web players render to SVG, canvas or HTML from JSON, driven by script each frame. They need script, so they do not work as `<img>`. | n/a | recollection; docs fetch failed | n/a | Low |
| 6.7 | Loading spinners and icon sets (SVG spinner collections, animated icon sets) are a common source of SMIL, because it works in `<img>` and CSS backgrounds. No measured breakdown was found. The Web Almanac breakdown of SVG animation was not found. | n/a | none | n/a | Low |

**Implications:** At about 2.9% of page loads and 1.7% in images, and still rising, SMIL is a compatibility requirement. The rare timing features (event begins at 0.019%, syncbase at 0.37%) set how much timing machinery is needed. They do not justify a separate engine.

## 7. Feature table: who owns SMIL

"Expressible" means the Web Animations model (Level 1 unless noted) can represent the feature with static keyframes and timing, possibly with Level 2. Ratings are the researcher's reading of WA1, WA2 and SVG Animations L2 (Medium unless noted).

| SMIL feature | Expressible in the Web Animations model? | Needs SMIL-specific timing machinery? | Usage evidence (Chrome, 2026-10-18) |
|---|---|---|---|
| Single offset `begin="2s"`, `dur` | Yes (start delay, duration) | No | `SVGSMILAnimationAppliedEffect` 2.33% |
| `repeatCount` (fractional, `indefinite`) | Yes (iterations, iteration start) | No | n/a |
| `repeatDur` | Yes once `dur` is resolved: iterations = repeatDur/dur | No | n/a |
| `min` / `max` | `min`: yes, WA1 end delay exists "for the purpose of representing the min attribute" (High). `max`: yes by clamping the active duration | Recompute when the intervals change | n/a |
| `fill="freeze"/"remove"` | Yes (fill forwards / none) | No | n/a |
| `values`/`keyTimes`, `from`/`to`/`by`, to-animation | Yes (keyframes and offsets; missing keyframe = underlying value; by = composite add) | No | n/a |
| `calcMode="linear"/"discrete"/"spline"` + `keySplines` | Yes (linear; step easing; per-keyframe `cubic-bezier`) | No | n/a |
| `calcMode="paced"` | **No** in WA1: offsets must be precomputed from value distances. Old WA drafts had paced spacing; removal not confirmed (Low) | A lowering step computes distances; this is not timing | n/a |
| `additive="sum"` | Yes (composite `add`; for transform lists, append) | No | `SVGSMILAdditiveAnimation` 0.153% |
| `accumulate="sum"` | Only in **WA Level 2** (`iterationComposite: accumulate`) | No | n/a |
| `animateTransform` | Yes for `transform`. Gradient and pattern transforms are attributes (see the attribute rows) | No | n/a |
| `animateMotion` path, `mpath`, `keyPoints`, `rotate=auto/auto-reverse` | Yes through CSS Motion Path (`offset-path`, `offset-distance` keyframes, `offset-rotate: auto/reverse`). Composition order with `transform` must match SMIL's supplemental transform | No | `offset-path` 0.45% (CSS); WPT failures cluster here (Chrome, Safari) |
| `set` | Yes (discrete keyframe) | No | n/a |
| Animating CSS properties (`fill`, `opacity`, `d`, `cx`, …) | Yes | No | n/a |
| Animating **non-property attributes** (`viewBox`, `points`, `stdDeviation`, gradient attributes, `href`, `class`) | **No**: WA targets CSS properties only. WA2 custom effects need script | No, but needs an **attribute-effect** extension of the effect stack (sampled into animVal) | Included in 2.33%; split unknown |
| Begin and end **lists** (`begin="0s;5s"`), `end` attribute | **No**: interval and instance-time lists | **Yes** (interval creation) | `SVGSMILBeginEndAnimationElement` 0.284% |
| **Syncbase** `begin="a.end+1s"`, chains, cycles | Partly: static chains ≈ WA2 sequence effects. Dynamic dependency propagation and cycles: **no** | **Yes** | `SVGSMILBeginOrEndSyncbaseValue` 0.372% |
| **Event-based** `begin="click"`, `repeat(n)`, `accessKey`, `wallclock` | **No** (script would call `play()`; images have no script) | **Yes** | `SVGSMILBeginOrEndEventValue` 0.0186% |
| `restart="always/whenNotActive/never"` | **No** | **Yes** | n/a |
| `beginElement[At]`/`endElement[At]` | Partly (`play()` with a start time) | Yes (instance times) | `SVGSMILAnimationElementTiming` 0.00012% |
| `beginEvent`/`endEvent`/`repeatEvent` | **No** (WA has finish and cancel only) | Yes | `SMILElementHasRepeatNEventListener` ~0 |
| Per-fragment timeline, `pauseAnimations`, `setCurrentTime` | Partly: a child timeline with an offset and pause state. WA1 has no pausable or seekable timeline type | Yes (fragment timeline) | `SVGSMILPausing` 0.079%; `SVGSMILCurrentTime` 0.0164% |
| `timelinebegin`, load-event document begin | **No** | Yes | n/a |
| `<discard>` | **No** (removes DOM) | Yes (a timed DOM mutation) | parsed 0.0008%; triggered ~0 |
| Sandwich priority (begin time, then document order) | Partly (WA composite order is by creation) | Yes (a priority rule) | n/a |

**Implications:** About two thirds of the features, and almost all usage by volume, lower onto the Web Animations effect model (with WA2 `accumulate`, Motion Path and an attribute-effect extension). The remaining third is the **SMIL interval and timing graph**: begin and end lists, syncbase, events, restart, fragment timelines and discard. It is small, self-contained and must run without JS in the media process.

**Recommendation for the ownership decision (Medium):** the animation spec owns the **effect model**, extended with attribute effects, `accumulate` and paced lowering. The SVG spec owns a scriptless **SMIL timing front end**. That front end resolves intervals and the timing graph and emits Web Animations effects on a per-fragment child timeline.

## Open questions for the spec

1. Black-box measurement: when does an SVG image's SMIL timeline start (load, decode, first paint), and do two `<img>` elements showing the same URL share it, in Chrome, Firefox and Safari?
2. Black-box measurement: do engines throttle or pause SMIL off-screen, in background tabs, or inside `<img>` scrolled out of view? Is `getCurrentTime` affected?
3. Black-box measurement: which `attributeName` targets do browsers refuse (`href` to a `javascript:` URL on `<a>`, `on*`, `style`, `xlink:href` on `<script>`)? No vendor prose defines this.
4. Does Firefox or Safari implement `<discard>`? (MDN has no entry; the WPT search was polluted.)
5. Does any engine run SMIL on its WA model internally? No prose found. Not needed for Izcal, but it would show which paths are proven.
6. Should `prefers-reduced-motion: reduce` pause SMIL in images by default? No standard covers it; Izcal could lead with a user setting.
7. Where does the composition order between `animateMotion`'s supplemental transform and CSS Motion Path plus `transform` differ? Measure it on the WPT `animateMotion-*` cases.
8. What exactly do Chrome's `AnimationSvg*Types` counters count? This is needed before using 14.8% as a CSS-on-SVG figure.

## Claims not verified

- Firefox 4 shipping in March 2011 (row 1.6): recollection; the wiki gives no date.
- Opera 9 in 2006 and WebKit or Chrome dates near 2009 (chronology): MDN version numbers were read; the release dates are recollection.
- EdgeHTML never supported SMIL (row 1.7): widely stated; not read in a primary Microsoft source.
- Removal of Chrome's SMIL deprecation warning (row 1.11).
- Earlier Web Animations drafts' paced spacing, and its removal (section 7).
- Lottie's renderers and their reliance on script (row 6.6): the docs fetch failed.
- CVE-2026-84371 details (row 3.11): seen only in advisory snippets.
- Firefox composites SVG transform and opacity animations (row 4.5): stated only by Chrome's blog.
- The mid-August 2015 jump in the `SVGSMILElementInDocument` counter (row 6.1): the cause is not known.
