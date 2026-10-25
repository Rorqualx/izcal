# Izcal feature inventory audit: re-grade of Covered rows (WA and CG)

Checked 24 October 2026 against `reports/` on the `feature-audit` worktree. Inputs: every row marked **Covered** in `map_css_graphics_media.md` (185 rows) and `map_web_platform_apis.md` (199 rows), 384 rows in all. For each row, the cited spec section was opened. Where the citation was thin (a scoping statement, one mention, a claim by reference, an assumed TC39 stage), the feature's names, interface or property names, WPT directory and module short name were also searched. Floors and milestones were checked in each owning spec's §8 (Targets) and §9 (Milestones).

**Grades.**
- **Designed**: a spec specifies Izcal's behaviour, architecture or choices for the feature.
- **Owned**: a spec names the feature, or explicitly names a standard that contains it, as in scope. It has an owner crate or track and a milestone, WPT/test262 floor or target that touches it. It is built from the standard as written. Owned counts as covered.
- **Mentioned**: the feature is named or implied, but has no owner, or no milestone, floor or target touches it. This includes features reached only through a broad claim. Mentioned is a gap.
- **Held**: a single-engine feature held by the admission rule (DM 6.3, DM 9.1, AH 6.15; LY-D10, ST-D18). See `verify_missing_wa_cg.md`.
- **Not there**: the citation does not support the feature.

**Rules applied.**
- **WPT floors.** A per-directory floor counts for a feature only when the feature's tests live in that directory (for example DM-1's `html/browsers/` 95%, ST-3's `css-conditional` 95% or LY-1's `css-ruby` 92%).
- **HTML and DOM claims.** A claim counts as Owned only when the feature is in the WHATWG standard, is not a recent addition whose presence at the (unrecorded) pin is in doubt, and sits under a DM-1 floor or a named milestone area.
- **ECMAScript claims.** A claim counts as Owned only for features in ES2026 or earlier (JS-M2, JS-1).
- **Spot-check verdicts are kept.**
  - Rows it confirmed are Designed or Owned.
  - Rows it graded Only partial are never Designed. They are Owned only where the standard is named with an owner and a floor or milestone, and Mentioned otherwise.
  - WA-22.7 stays Not there.
  - WA-18.18 and WA-18.23 stay Mentioned, because their ES2026 membership rests on secondary sources only.

## Summary

| Map | Rows | Designed | Owned | Mentioned | Held | Not there | Covered (Designed + Owned) | Gap (Mentioned + Not there) |
|---|---|---|---|---|---|---|---|---|
| CSS, graphics and media (CG) | 185 | 151 (82%) | 30 (16%) | 4 (2%) | 0 (0%) | 0 (0%) | 181 | 4 |
| Web platform APIs (WA) | 199 | 120 (60%) | 60 (30%) | 17 (9%) | 1 (1%) | 1 (1%) | 180 | 18 |
| **Both** | **384** | **271** | **90** | **21** | **1** | **1** | **361** | **22** |

**Reading.** The spot-check's 36–48% "only partial" rate came from a strict standard: the spec must state the behaviour. On this scale, most of those rows are **Owned**: a named standard, an owner, and a floor or milestone, with no Izcal-specific design needed. Rows with no owner and no floor or milestone are the real gaps: 4 CG rows (2%) and 18 WA rows (9%). The WA gaps cluster in three places:
- **APIs with no shell owner:** simple dialogs, `showPicker()` and the colour input.
- **Script-facing APIs in DM 6.18 with no floor or milestone:** Streams, SSE, Compression Streams, Performance Timeline, Navigation Timing, User Timing and Paint Timing. Adding the WPT directories `streams/`, `eventsource/`, `compression/`, `performance-timeline/`, `navigation-timing/`, `user-timing/` and `paint-timing/` to a DM or NW floor, plus a milestone, would make them Owned.
- **Rows whose inclusion depends on an unrecorded pin or stage:** Origin API, Upsert, JSON.parse source text and resizable Wasm buffers.

**The weakest Owned rows.** These rest only on a named standard and a floor or milestone, and the feature is never named:
- CG-1.14 media pseudo-classes
- CG-2.11 `@supports selector()`
- CG-2.12 `font-tech()`
- CG-3.4 `contrast-color()`
- CG-4.11 Box Alignment 3
- CG-11.2 Path2D
- WA-1.10 `ancestorOrigins`
- WA-4.7 `CloseWatcher`
- WA-18.19 `Math.sumPrecise`
- WA-18.22 `Array.fromAsync`

If a pin or a floor moves, these are the first to re-check.

**Single-engine rows designed with no admission record:**
- CG-1.7 `revert-rule`
- CG-2.5 `sibling-index()`
- CG-3.9 `corner-shape`
- CG-14.15 Document PiP
- CG-15.7 Region Capture
- CG-15.8 Element Capture
- Element Timing (in WA-17.12's family, listed by DM 6.18)

WA-16.5 Device Memory is graded **Held**: it appears only in a fingerprinting row, and DM-D15 records no exception for it.

### Counts per group: CSS, graphics and media

| Group | Rows | Designed | Owned | Mentioned | Held | Not there |
|---|---|---|---|---|---|---|
| 1. Syntax, cascade, selectors | 16 | 13 | 3 | 0 | 0 | 0 |
| 2. Values, units, conditionals | 11 | 8 | 3 | 0 | 0 | 0 |
| 3. Colour, images, backgrounds, borders | 10 | 8 | 2 | 0 | 0 | 0 |
| 4. Box, display, layout modes | 21 | 18 | 3 | 0 | 0 | 0 |
| 5. Scrolling, fragmentation, paged media | 5 | 5 | 0 | 0 | 0 | 0 |
| 6. Text, fonts, lists, content | 16 | 12 | 3 | 1 | 0 | 0 |
| 7. Transforms, animation, effects | 15 | 15 | 0 | 0 | 0 | 0 |
| 8. Object models, observers | 6 | 6 | 0 | 0 | 0 | 0 |
| 9. Forms, UI | 4 | 3 | 1 | 0 | 0 | 0 |
| 10. SVG, MathML | 6 | 6 | 0 | 0 | 0 | 0 |
| 11. Graphics APIs | 11 | 9 | 2 | 0 | 0 | 0 |
| 12. Image formats, colour | 14 | 12 | 1 | 1 | 0 | 0 |
| 13. Video and audio formats | 16 | 8 | 8 | 0 | 0 | 0 |
| 14. Media elements, streaming | 20 | 18 | 2 | 0 | 0 | 0 |
| 15. Capture, RTC | 9 | 5 | 2 | 2 | 0 | 0 |
| 16. XR, speech, print, other | 5 | 5 | 0 | 0 | 0 | 0 |
| **Total** | **185** | **151** | **30** | **4** | **0** | **0** |

### Counts per group: web platform APIs

| Group | Rows | Designed | Owned | Mentioned | Held | Not there |
|---|---|---|---|---|---|---|
| 1. HTML documents, navigation | 15 | 10 | 3 | 2 | 0 | 0 |
| 2. HTML elements, metadata | 14 | 7 | 7 | 0 | 0 | 0 |
| 3. Forms | 8 | 2 | 4 | 2 | 0 | 0 |
| 4. Interactive UI, dialogs | 14 | 6 | 8 | 0 | 0 | 0 |
| 5. Scripting, loading | 8 | 3 | 5 | 0 | 0 | 0 |
| 6. DOM, components | 22 | 19 | 1 | 2 | 0 | 0 |
| 7. Event loop, workers, messaging | 14 | 13 | 1 | 0 | 0 | 0 |
| 8. Fetch, networking, formats | 13 | 7 | 3 | 3 | 0 | 0 |
| 9. Storage, files | 7 | 7 | 0 | 0 | 0 | 0 |
| 10. Service workers, push | 4 | 2 | 2 | 0 | 0 | 0 |
| 12. Permissions, clipboard | 3 | 2 | 1 | 0 | 0 | 0 |
| 13. Security, isolation | 11 | 7 | 3 | 1 | 0 | 0 |
| 14. Privacy | 6 | 6 | 0 | 0 | 0 | 0 |
| 16. Devices | 7 | 6 | 0 | 0 | 1 | 0 |
| 17. Performance, reporting | 10 | 3 | 3 | 4 | 0 | 0 |
| 18. ECMAScript | 19 | 4 | 13 | 2 | 0 | 0 |
| 19. Intl, Temporal | 5 | 3 | 2 | 0 | 0 | 0 |
| 20. WebAssembly | 11 | 7 | 3 | 1 | 0 | 0 |
| 22. Protocols, infrastructure | 8 | 6 | 1 | 0 | 0 | 1 |
| **Total** | **199** | **120** | **60** | **17** | **1** | **1** |

## All Mentioned and Not-there rows, and what each lacks

### CSS, graphics and media (4)

- CG-6.19 Lists and counters 3 (Mentioned): anchor table and staffing line only; no LY-1 `css-lists` floor, no milestone naming lists, counter scoping and marker generation undesigned.
- CG-12.16 Image density correction (Mentioned): one clause in MD 6.6; no rules (which Exif fields, how applied), no milestone naming it, no `density-size-correction/` WPT floor.
- CG-15.10 MediaStreamTrack insertable streams (Mentioned): `MediaStreamTrackProcessor` is listed only in the 6.20 table and in a noise rule. No design of the processor or generator, `VideoTrackGenerator` is absent, no milestone names it, and there is no `mediacapture-insertable-streams/` floor.
- CG-15.17 WebRTC Encoded Transform (Mentioned): one routing sentence. No anchor for the Encoded Transform spec, no milestone names it, no `webrtc-encoded-transform/` floor, and it depends on the unowned RTCRtpSender/Receiver API (CG-15.14).

Also noted: one sub-item inside an Owned row is Not there. G.722 in CG-13.18 is named in no report, and no decision says whether WebRTC supports it.

### Web platform APIs (18)

- WA-1.11 Origin API (Mentioned): named nowhere; no owner, milestone or floor; HTML pin date unrecorded, so the claim's reach is unverifiable.
- WA-1.16 Simple dialogs (Mentioned): DOM pause is designed, but the dialog surface DM assigns to the shell has no class in shell 6.4 or SHUI; no surface design, focus or origin labelling.
- WA-3.6 `showPicker()` (Mentioned): not named; pickers are "browser-process UI" (LY-D11) with no owning shell/UI design; no milestone.
- WA-3.11 Colour input `alpha`/`colorspace` (Mentioned): not named; colour picker unowned; inclusion at the HTML pin unchecked; no milestone or floor.
- WA-6.25 Geometry interfaces (Mentioned): no owner crate, no milestone, no `css/geometry/` floor; only pinned as an SVG input and typed `double`.
- WA-6.26 Visual Viewport (Mentioned): `window.visualViewport` interface and events unnamed; no milestone; `visual-viewport/` WPT under no floor (mechanics designed in GR 6.28/LY-D13).
- WA-8.6 Server-sent events (Mentioned): owner `hg-webapi` named, but no design, no milestone, no `eventsource/` floor.
- WA-8.12 Streams (Mentioned): owner `hg-webapi` named, but no design, no milestone, no `streams/` floor.
- WA-8.13 Compression Streams (Mentioned): owner named; no milestone for the API or encoders (NW-M1 ships decoders only); no `compression/` floor.
- WA-13.10 iframe sandbox and X-Frame-Options (Mentioned): the sandbox is designed, but XFO has no named owner, no design and no floor (`x-frame-options/` is not floored); it rests on the HTML claim alone.
- WA-17.2 Performance Timeline (Mentioned): has an owner (`hg-webapi`), but no milestone, no WPT floor (`performance-timeline/`) and no design for buffering or `PerformanceObserver`.
- WA-17.3 Navigation Timing 2 (Mentioned): has an owner, but no milestone, no floor (`navigation-timing/`) and no design; `confidence` is not named.
- WA-17.5 User Timing 3 (Mentioned): has an owner, but no milestone, no floor (`user-timing/`) and no design.
- WA-17.7 Paint Timing (Mentioned): the event-loop step has an owner, but there is no milestone, no floor (`paint-timing/`) and no FP/FCP entry design.
- WA-18.18 Upsert (Mentioned): not named in any report, and no report records its TC39 stage. Whether it is in ES2026 (JS-M2) rests on secondary sources only. The spot-check's Only-partial verdict is kept.
- WA-18.23 JSON.parse source text access (Mentioned): same as WA-18.18. Not named, stage unrecorded, and ES2026 membership not shown from a primary source.
- WA-20.11 Resizable memory buffers (Mentioned): not named in any report; no owner, design or milestone; covered only if the proposal is merged into the Wasm JS API at the pin (`wasm/jsapi/` floor).
- WA-22.7 Infra (Not there): named in no report; DM-D21 is the IDL-inputs decision (fixed by the spot-check).

## CSS, graphics and media: every Covered row

| # | Feature | Grade | Owner (spec §) | Milestone/target | Evidence (≤ 15 words) |
|---|---|---|---|---|---|
| CG-1.1 | CSS 2.1 core | Designed | LY 6.5.1, 6.5.7 (`hg-layout`); GR 6.3 (`hg-paint`) | LY-M1, GR-M1; LY-1 `CSS2` 97% | Adjoining-margin summary, float area, nine float rules; Appendix E paint chunks |
| CG-1.3 | CSS Syntax 3 | Designed | ST 6.6 (`hg-css-syntax`) | ST-M1; ST-3 `css-syntax` 97% | Hand-written tokenizer, 25 types, source offsets, ten entry points (Confirmed) |
| CG-1.4 | Cascade and inheritance 3/4 | Designed | ST 6.9 (`hg-style`) | ST-M1; ST-3 `css-cascade` 97% | Fixed-width cascade key tuple, eight origin levels, decided-bitset walk |
| CG-1.5 | Cascade layers | Designed | ST 6.9, 6.6a (`hg-style`) | ST-M1 exit names layers; ST-3 `css-cascade` 97% | Layer ranks fixed per scope with `!important` reversal; layered import keeps position |
| CG-1.6 | `@scope` | Owned | ST 6.9 (`hg-style`) | ST-M1 exit names `@scope`; ST-3 `css-cascade` 97% | Only "scope proximity computed per match" in cascade key; scoping-root matching undesigned |
| CG-1.7 | `revert-rule` | Owned | ST 6.9 (ST-D8 match keys) | ST-M1; ST-3 `css-cascade` 97% | Match key keeps full list so revert-rule rolls back; single-engine, no admission record |
| CG-1.8 | CSS Nesting | Designed | ST 6.6, 6.7 | ST-M1; ST-3 `css-nesting` 97% | Declaration-then-rule back-track; `&` as `:is()` with parent max specificity |
| CG-1.10 | Style attributes | Designed | ST 6.11, 6.6 (`hg-dom`, `hg-style`) | ST-M1 | Inline-style block shared by source text, copy-on-write, primary data; element-attached key |
| CG-1.11 | Selectors 3 | Designed | ST 6.7 (ST-D9) | ST-M0 reference, ST-M1; ST-3 `selectors` 95% | Compiled right-to-left programs, rule index buckets, ancestor filter |
| CG-1.12 | Selectors 4 | Designed | ST 6.7, 6.7.1, 6.7.2 (ST-D11) | ST-M1, `:has()` ST-M2; ST-3 `selectors` 95% | `:has()` memo, direction by cost, subtree summaries, work budget |
| CG-1.13 | UI state pseudo-classes | Designed | ST 6.7.1; DM 6.12 (`hg-html`) | ST-M1, DM-M2; DM-1 Interop release gates | Named state sources (fullscreen, top layer, `:autofill` from shell, `:state()`) as recorded tasks |
| CG-1.14 | Media pseudo-classes | Owned | ST 6.7.1 (Selectors 4 anchor) | ST-3 `selectors` 95% (`css/selectors/media/`) | Generic state source only; no pseudo-class named, no media-side writer (Only partial) |
| CG-1.16 | Pseudo-elements 4 | Designed | LY 6.3 (`hg-box-tree`); ED 6.7 highlights | LY-M1; ED-1 `css-highlight-api` 93% | Side tables for first-letter/first-line split points; highlight overlays in Pseudo 4 order |
| CG-1.17 | Shadow DOM styling | Designed | ST 6.9, 6.7 | ST-M1; ST-3 `css-shadow` 87% | Shadow context in cascade key; per-scope rule index (Confirmed) |
| CG-1.18 | Custom properties | Designed | ST 6.9 (ST-D14) | ST-M2 by reference; ST-3 `css-variables` 87% | Change sets over parent by reference; reader sets; 65,536-token limit |
| CG-1.19 | Registered custom properties | Designed | ST 6.9 | ST-M2; ST-3 `css-properties-values-api` 95% | Typed computed values; `inherits: false` is an invalidation boundary (Confirmed) |
| CG-2.1 | Values and Units 3 | Designed | ST 6.9 | ST-M1; ST-3 `css-values` 87% | Superseded by Values 4 design: typed `calc()`, `hg-math` (Confirmed) |
| CG-2.2 | Values and Units 4 | Designed | ST 6.9, 6.10 | ST-M1; ST-3 `css-values` 87% | `calc()` 32 terms/levels, shared `hg-math` trig, font-relative units via TX metric ratio |
| CG-2.3 | Typed `attr()` | Designed | ST 6.9 (substitution interpreter) | ST-M2 exit names typed `attr()` | `attr()` taint as a type; URL parsers reject tainted streams |
| CG-2.5 | `sibling-index()`, `sibling-count()` | Owned | ST 6.9 (substitution interpreter) | ST-M2; ST-3 `css-values` 87% (`tree-counting`) | "Tree-position functions"; memo keyed on reads; single-engine, no admission record |
| CG-2.10 | Conditional 3 | Designed | ST 6.10 | ST-M1; ST-3 `css-conditional` 95% | One environment snapshot per document per frame drives `@media`; mutation-log invalidation |
| CG-2.11 | Conditional 4 (`@supports selector()`) | Owned | ST 7 anchor (Conditional 5) | ST-3 `css-conditional` 95% | Standard anchored, floor covers it; `selector()` evaluation not designed (Only partial) |
| CG-2.12 | Conditional 5 (`font-tech()`, `@when`) | Owned | ST 7 anchor (Conditional 5) | ST-3 `css-conditional` 95% | `font-tech` zero hits; standard + floor only; `@when` held by ST-D18 (Only partial) |
| CG-2.13 | Container size queries | Designed | ST 6.10; LY 6.9 (LY-D12) | ST-M2, LY-M2; LY-6 | Re-entry at scheduler barrier, per-size style versions, structural bound |
| CG-2.14 | Container style queries | Designed | ST 6.10 | ST-M2 container queries; ST-3 `css-conditional` 95% | Evaluated in normal pass against container's computed values, no layout |
| CG-2.16 | Media Queries 3/4 | Designed | ST 6.10; MD 6.5; GR 6.20 | ST-M1 | Snapshot with D-S1 quantization; `color-gamut`/`dynamic-range` buckets only |
| CG-2.20 | Easing functions | Designed | AN 6.2 (AN-D2, `hg-anim-time`) | AN-M1; AN-1 `css-easing` 97% | Exact easing, integer cubic Bézier, `linear()` |
| CG-3.1 | Colour 3 | Designed | ST 6.9; MD 6.5 (`hg-colour`) | ST-M1, GR-M1; GR-1 `css-color` C-1 floor | f64 computed colours, one MD-D28 hand-off to the pixel path |
| CG-3.2 | Colour 4 | Designed | ST 6.9; MD 6.5; GR 6.4 | ST-M1, GR-M1; GR-1 `css-color` | Oklab gradient interpolation, `hg-math` conversions; gamut-mapping choice still open |
| CG-3.3 | Colour 5 | Designed | ST 6.9 | ST-M1; GR-1 `css-color` | `color-mix()`, relative colours via `hg-math`; `@color-profile` to generated ICC parser |
| CG-3.4 | `contrast-color()` | Owned | GR 7 anchor (Color 4 and 5) | GR-1 `css-color` C-1 floor | Zero hits; reachable only via Color 5 anchor and floor (Only partial) |
| CG-3.5 | Colour HDR | Designed | MD 6.5 (MD-D9); ST 6.9 | MD-M2, GR-M2 | 203 cd/m² reference white, BT.2390 EETF; `dynamic-range-limit` to compositor |
| CG-3.6 | Colour adjustment | Designed | AX 6.9; GR 6.23 (GR-D25, `hg-paint`); ST 6.6 | AX-M2, GR-M2 | Forced values in style, backplates in paint; `color-scheme`, `light-dark()` unnamed |
| CG-3.7 | Backgrounds and Borders 3 | Designed | GR 6.4 (GR-D17) | GR-M1 exit; GR-1 `css-backgrounds`, `css-borders` | Corner joint rule, dotted/dashed spacing, double thirds, box-shadow 5% bound |
| CG-3.9 | Borders 4 (`corner-shape`) | Owned | GR 7 anchor (Borders 4); GR-D17; LY 6.14 | GR-M1; GR-1 `css-borders` | Outline and hit test only; shaped border paint undesigned; single-engine (Only partial) |
| CG-3.11 | Images 3 | Designed | LY 6.5.8; GR 6.4; MD 6.6 | LY-M1, MD-M1; GR-1 `css-images` | Default sizing 300×150, `object-fit`; integer sampling, `pixelated`/`crisp-edges` |
| CG-3.12 | Images 4 | Designed | GR 6.4; SV 6.8 | GR-M1; GR-1 `css-images` | 1,024-entry gradient ramp, Oklab, ordered 8×8 dithering (Confirmed) |
| CG-4.1 | Box model 3 | Designed | LY 6.5.1 (`hg-layout`) | LY-M1; LY-1 `css-box` 97% | Block layout per CSS 2.1 and Box 4; margin collapsing summary |
| CG-4.3 | Sizing 3 | Designed | LY 6.6 | LY-M1/M2; LY-1 `css-sizing` 97% | Intrinsic memo by input, cyclic-percentage modes, symbolic percentages |
| CG-4.4 | Sizing 4 | Designed | LY 6.6, 6.9 | LY-M2; LY-1 `css-sizing` 97% | One shared sizing module; last remembered size as frame-boundary state |
| CG-4.5 | Display 3 | Designed | LY 6.3 (LY-D2, `hg-box-tree`) | LY-M1; LY-1 `css-display` 97% | 32 B box columns, anonymous boxes, affects-mask rebuild |
| CG-4.7 | Flexbox | Designed | LY 6.5.3 | LY-M2; LY-1 `css-flexbox` 97% | Web-compatible intrinsic main size, 50% pull-up fragmentation (Confirmed) |
| CG-4.8 | Grid 1 | Designed | LY 6.5.4 | LY-M2; LY-1 `css-grid` 97% | Shared track sizing, at most two layouts per round, ±10,000 lines |
| CG-4.9 | Subgrid | Designed | LY 6.5.4 | LY-M2 exit names subgrid; LY-1 `css-grid` 97% | Per-subgrid contribution record into parent's track sizing |
| CG-4.11 | Box Alignment 3 | Owned | LY 7 anchor (`hg-layout`) | LY-1 `css-align` 97% | Anchor and floor only; no alignment design beyond abspos (Only partial) |
| CG-4.12 | Multicol 1 | Designed | LY 6.5.6 | LY-M2; LY-1 `css-multicol` 92%; LY-24 | Recorded-break balancing, at most 3 layouts per column row, linear nesting |
| CG-4.14 | Tables 3 | Designed | LY 6.5.5 (LY-D17) | LY-M2; LY-1 `css-tables` 92% | Sparse cell grid, collapse pass, black-box choices where Tables 3 silent |
| CG-4.15 | Inline 3 | Designed | LY 6.5.2; TX 6.8 | LY-M1; LY-1 `css-inline` 92% | Flat item list, layout-owned line breaker, `text-box-trim` from TX metrics |
| CG-4.16 | Positioned layout 3 | Designed | LY 6.5.7, 6.10 | LY-M1; LY-1 `css-position` 97% | Inset-modified containing block; sticky as constraints for scroll machinery |
| CG-4.17 | Top layer, `::backdrop`, `overlay` | Designed | LY 6.5.7, 6.9; DM 6.12; AN 6.9 (AN-D9) | LY-M1, DM-M2 | Top-layer boxes use ICB, laid out top-layer level first; `overlay` transitions admitted |
| CG-4.18 | Anchor positioning 1 | Designed | LY 6.9 rule 5 | LY-M2; LY-1 `css-anchor-position` 92% | Last successful option first; remembered scroll offsets; re-entry per fallback |
| CG-4.20 | Overflow 3 | Designed | LY 6.10 (LY-D13) | LY-M1; LY-1 `css-overflow` 92% | Scrollable overflow rectangle as fragment field; delta protocol for offsets |
| CG-4.21 | Overflow 4 (`line-clamp`, `continue`) | Owned | LY 6.7, 7 anchor (Overflow 4) | LY-M2 fragmentation; LY-1 `css-overflow` 92% | One sentence: clamp as fragmentation container; `block-ellipsis`, `max-lines` unnamed |
| CG-4.24 | Writing modes 3/4 | Designed | LY 6.8; TX 6.6 | LY-M2; LY-1 `css-writing-modes` 92% | Logical coordinates throughout, physical conversion once at store |
| CG-4.25 | Containment, `content-visibility` | Designed | LY 6.9 | LY-M2; LY-16 | Skipped subtrees, proximity state, semantic tree unchanged with AT on/off |
| CG-4.26 | `will-change` | Designed | GR 6.3 (GR-D5) | GR-M1 | Hint for 300 BeginFrame ticks; never forces a layer |
| CG-4.27 | Shapes 1 | Owned | LY 6.5.1 (float area) | LY-M1 exit names shapes; LY-1 `css-shapes` 92% | Float area holds polygon/raster shapes; shape geometry, `shape-margin` undesigned |
| CG-4.33 | Quirks mode | Designed | ST 6.6, 6.6a; LY 6.3 | ST-M1, LY-M1 | Quirks UA sheet in image; parsing mode in sheet cache key; recorded mode input |
| CG-5.1 | Scroll snap 1 | Designed | LY 6.10; GR 6.8 | LY-M2; LY-1 `css-scroll-snap` 92% | Compositor snaps during gesture; re-snap after layout with focus/target priority |
| CG-5.3 | Scroll anchoring | Designed | LY 6.10 | LY-M2; LY-1 `css-scroll-anchoring` 92% | Anchor selection, suppression window boundaries, decision records |
| CG-5.6 | Smooth scrolling, scroll APIs, visual viewport | Designed | LY 6.15, 6.10; GR 6.8, 6.28 | LY-M2; LY-1 `cssom-view` 95% | Delta protocol, `scrollend` timing, compositor visual viewport; smooth curve unspecified |
| CG-5.8 | Fragmentation 3 | Designed | LY 6.7 (LY-D8) | LY-M2; LY-1 `css-break` 92% | Appeal-ordered break scoring, best-break record, one rerun per fragmentainer |
| CG-5.10 | Paged Media 3 | Designed | LY 6.7 (LY-D9) | LY-M2; LY-1 `css-page` 90% | 16 margin boxes, print environment as one recorded input |
| CG-6.1 | Text 3 | Designed | TX 6.6 (`hg-unicode`, `hg-ucd`); LY 6.5.2 (`hg-layout`) | TX-M1, LY-M1; TX-2 `css-text`; LY-1 `css-text` 92% | UAX 14 CSS tailorings, language-sensitive case mapping; `hanging-punctuation` named only in TX 6.18 |
| CG-6.2 | Text 4 | Designed | LY 6.5.2, LY-D16; TX 6.6, 6.18 | LY-M1 (`balance`), LY-M3 (`pretty`), TX-M2 | LY-D16 `balance`/`pretty` objectives; `auto-phrase` dictionary method (Chrome-only, no admission record) |
| CG-6.6 | Fonts 3 | Designed | TX 6.4 (`hg-font-match`) | TX-M1; TX-2 `css-fonts` | Matching implements CSS Fonts 4 §5 step by step; one vertical-metrics rule |
| CG-6.7 | Fonts 4 | Designed | TX 6.4, 6.5 | TX-M1/M2; TX-2 `css-fonts` | Synthesis, palettes, `font-variant-emoji`, generics, `local()` restricted to visible set |
| CG-6.8 | Fonts 5 | Owned | TX 4.2, 6.4, 6.8 (`hg-font-match`, metrics query) | TX-M2; TX-2 `css-fonts` | Metric overrides, `size-adjust`, `font-size-adjust` named in matching and metrics query (spot-check confirmed) |
| CG-6.9 | Font Loading API | Designed | TX 6.5 (`hg-font-load`) | TX-M2; TX-2 `css-font-loading` | Faces installed at task-boundary decision point; `font-display` on substrate clock |
| CG-6.10 | WOFF 1/2, OpenType | Designed | TX 6.2 (`.hgfont`, `hg-fontgen`) | TX-M1; TX-5 100% WOFF2 suites | Generated Kani-proved reader; eager/lazy validation; size caps (spot-check confirmed) |
| CG-6.11 | Variable fonts | Designed | TX 6.2 | TX-M2; avar2 TX-M4 | F2Dot14 coordinates, 16.16 deltas rounded once, per-partition instance keying |
| CG-6.12 | Colour font formats | Designed | TX 6.2, 6.3; GR 6.6 | TX-M2; TX-5 COLRv1 reftests 100% | COLRv1 paint-graph fuel and fallback; OT-SVG limits; sbix/CBDT in TX-M2 |
| CG-6.13 | Shaping, AAT, Graphite | Designed | TX 6.7, TX-D8 (`hg-shape`) | TX-M1/M2; TX-3, TX-4 | Izcal shaping specification, generated engines, AAT under fuel; Graphite excluded (TX-D7) |
| CG-6.14 | Unicode algorithms | Designed | TX 6.6, TX-D11 (`hg-unicode`) | TX-M0/M1; TX-1 100% | Per-algorithm table: UAX 9/14/29/15/50/53, UTS 51/46, generated tables |
| CG-6.15 | Hyphenation dictionaries | Designed | TX 6.6, TX-D14 (`hg-hyphen`) | TX-M2; TX-3 review gate | Izcal-authored lexicons, own Liang generator; ships when lexicon reproduced |
| CG-6.16 | Incremental Font Transfer | Owned | TX 6.2, 9.1 (`hg-font-load`) | TX-M4 "IFT shipped if its gate passed" | Patches applied with the spec's caps; gated on WPT and privacy review |
| CG-6.18 | Ruby layout | Owned | LY 6.5.2, LY 7 anchor "Ruby 1" (`hg-layout`) | LY-1 `css-ruby` 92% | Spot-check Only partial: one list item; placement, `ruby-position`, overhang undesigned |
| CG-6.19 | Lists and counters 3 | Mentioned | LY 7 anchor "CSS Lists 3"; LY 9.3 box-tree track | — | Anchor oracle `css-lists` but no LY-1 floor; counters and markers undesigned |
| CG-6.22 | Custom Highlight API | Designed | ED 6.7 (`hg-caret`; map cites 6.10, which is clipboard) | ED-M2; ED-1 `css/css-highlight-api/` 93% | One overlay per highlight type in Pseudo 4 order, keyed chunks |
| CG-7.1 | Transforms 1 | Designed | GR 6.4 (`hg-raster`); AN 6.2 (`hg-interp`) | GR-M1; GR-1 `css-transforms`; AN-1 subtests 95% | Fixed-point device transform; degree trigonometry shared by static and animated |
| CG-7.2 | Transforms 2 (3D) | Designed | GR 6.3 (`hg-paint`) | GR-M2 "3D transforms"; GR-1 `css-transforms` | Newell's algorithm in integers, 1/256 px split grid, 4,096 splits |
| CG-7.3 | Transitions 1 | Designed | AN 6.4 (`hg-anim::css`); ST 6.9 | AN-M1; AN-1 `css-transitions` 97% | Style change event per flush; sequential post-pass creates transitions |
| CG-7.4 | Transitions 2 | Designed | AN 6.4, AN-D9; ST 6.10 | AN-M1; AN-1 `css-transitions` 97% | `@starting-style`, `allow-discrete` flip at 50%, `display`/`overlay` rules; admitted |
| CG-7.5 | Animations 1 | Designed | AN 6.4 (`hg-anim::css`) | AN-M1; AN-1 `css-animations` 97% | Keyframe cascade, name matching, API overrides, events via change list |
| CG-7.6 | Animations 2 | Designed | AN 6.4, AN-D9 | AN-M1; AN-1 `css-animations` 97% | `animation-composition`, `animation-timeline` admitted; `display` ignoring animation origins primitive |
| CG-7.7 | Web Animations 1 | Designed | AN 6.1, 6.2 (`hg-anim`, `hg-anim-time`) | AN-M1; AN-1 `web-animations` 97% | i128 ns `AnimTime`, exact Q32 progress, integer-bisection easing |
| CG-7.9 | Scroll-driven animations | Designed | AN 6.6, AN-D8; LY 6.10; GR 6.8 | AN-M2; AN-1 `scroll-animations` 92%; AN-2 Interop | Q32 timelines, stale-timeline rule, 2026 zero-range resolutions |
| CG-7.11 | Motion path | Designed | AN 6.2, 6.5 (class M), AN-D4 | AN-M1/M2; AN-1 `css/motion` subtests 95%; AN-7 | `offset-*` sampled via SVG path functions on main thread; static placement not designed |
| CG-7.12 | View Transitions 1 | Designed | AN 6.7, AN-D7 (`hg-view-transition`); GR 6.8 | AN-M2, GR-M3; AN-1 `css-view-transitions` 92% | Lifecycle, compositor-held snapshots, 1,024 named elements limit |
| CG-7.13 | View Transitions 2 | Designed | AN 6.7 | AN-M3; AN-53 | Broker decides cross-document eligibility; snapshots unreadable (G137); scoped/nested held |
| CG-7.14 | Filter Effects 1 | Designed | GR 6.4, GR-D18 | GR-M2; GR-1 `filter-effects`; SV-2 | Integer kernels, three-box blur within 3%, feTurbulence fixed-order double |
| CG-7.15 | `backdrop-filter` | Designed | GR 6.3, 6.7 (`hg-layerize`) | GR-M2 "Filter Effects 1 and 2"; GR-1 `filter-effects` | Backdrop-reading chunk placed in same layer as what it reads (spot-check confirmed) |
| CG-7.16 | Compositing and blending | Designed | GR 6.3, 6.4 (`hg-raster`) | GR-M2; GR-1 `compositing` | All 16 blend modes in premultiplied integer alpha; stacking contexts as isolation groups |
| CG-7.17 | Masking | Designed | GR 6.4, 6.16; SV 6.5 | GR-M2 "CSS Masking"; GR-1 `css-masking`; SV-2 | Integer coverage masks, fixed-point luminance; one `url()` resolver; `mask-border` unnamed |
| CG-8.1 | CSSOM | Designed | ST 6.6, 6.11 (`hg-cssom`) | ST-M2 "CSSOM complete"; ST-3 `cssom` 97% | Copy-on-write sheet contents, origin-clean per document, constructed sheets as primary data |
| CG-8.2 | CSSOM View | Designed | LY 6.15, 6.1 (`hg-layout-query`) | LY-M1 (getters, forcing table); LY-1 `cssom-view` 95% | Forcing table incl. SVG geometry methods; `elementsFromPoint` equals paint order (LY-7) |
| CG-8.5 | Worklets | Designed | DM 6.7, 6.8 (`hg-workers`, `hg-agents`) | — (no milestone names worklets; `worklets/` not a DM-1 floor) | Two global scopes chosen by seeded random; worklet agents in owner's cluster |
| CG-8.7 | Resize Observer | Designed | LY 6.15; GR 6.3 | LY-M2; LY-1 `resize-observer` 95% | Depth-ordered loop; `device-pixel-content-box` from `hg-paint-order` snapping function |
| CG-8.8 | Intersection Observer (v1, v2) | Designed | LY 6.15, LY-D10 owner exception for v2 | LY-M2; LY-1 `intersection-observer` 95% | v2 visibility proof, cross-process visibility bit; v2 single-engine via LY-D10 exception |
| CG-8.9 | Update-the-rendering steps | Designed | DM 6.7 (`hg-script::rendering`); GR 6.9 | DM-M1/M2; DM-1 `html/webappapis/` 97% | 17-step table with an owner per step, BeginFrame-queued rendering task |
| CG-9.4 | Native form-control rendering | Designed | LY 6.3, LY-D11 (`hg-box-tree`); DM 6.12 (`hg-forms`) | LY-M1 "form controls", DM-M2 | UA shadow trees with Izcal-defined metrics; pickers only placed, not designed |
| CG-9.5 | HTML rendering section, presentational hints | Designed | LY 6.3; ST 6.6, 6.11 | LY-M1 "Box tree complete for HTML Rendering", ST-M1 | Element generation rules (`button`, `fieldset`, `details`); presentational-hint block in memo key |
| CG-9.6 | `inert` | Owned | DM 6.1, 6.5 (`hg-html::focus`); LY 6.14 (`hg-hit`); AX 6.x | DM-M2; DM-1 `html/semantics/` 97% (`interactivity/inert`) | Node flag, focus and hit-test skip named; no Izcal-specific design |
| CG-9.8 | Forced colours, high contrast | Designed | GR 6.23, GR-D25 (`hg-paint`); AX 6.11, AX-D18; ST 6.10 | GR-M2, AX-M2; AX-42 | Paint-time backplates; real Windows palette, fixed palettes elsewhere |
| CG-10.1 | SVG 1.1 | Designed | SV 6.2–6.11; GR 6.16 (`hg-svg-paint`) | SV-M1/M2; SV-1 per-`svg/` floors | Scriptless host, render tree, exact geometry, reference graph, `use` instances |
| CG-10.2 | SVG 2 | Designed | SV (SVG 2 CR pin), SV-D14, SV-D13 | SV-M1/M2; SV-1 | `animVal` alias, legacy members by counters; held items listed in SV 9.1 |
| CG-10.3 | SMIL | Designed | AN 6.8, AN-D6; SV 6.9, SV-D4 | AN-M2, SV-M2; AN-1 `svg/animations` 90%; SV-5 | Timing with animation model, attribute overlay (spot-check confirmed) |
| CG-10.4 | SVG integration and image modes | Designed | SV 6.8, SV-D10–D12; MD-D20 | SV-M1/M2; SV-7 | Image-mode differences, opaque images in media process, `foreignObject` tainting gate |
| CG-10.6 | SVG markers and strokes | Designed | GR 6.16, 6.4 (`hg-path`, `hg-svg-paint`) | GR-M2, SV-M2 "markers"; SV-1 `painting` 90%; GR-7 | Nehab stroker with evolutes, SVG 2 dash algorithm, markers with `orient` |
| CG-10.7 | MathML Core | Designed | LY 6.5.9, LY-D21; TX 6.8 (`MATH`), TX-W2 | LY-M2; LY-1 `mathml` 90% | Dedicated MathML Core layout section; Izcal math face with `MATH` table |
| CG-11.1 | Canvas 2D | Designed | GR 6.15 (GR-D12), `hg-canvas` | GR-M1 "Canvas 2D"; GR-1 `html/canvas/`; GR-8, GR-9 | Renderer-authoritative CPU replay, canonical readback, no noise; readback bit-exact target GR-8 |
| CG-11.2 | Path2D | Owned | GR 7 anchor "HTML Standard canvas chapter", `hg-canvas` | GR-M1; GR-1 `html/canvas/` (path-objects tests live there) | Zero hits for Path2D; covered by canvas-chapter anchor plus `html/canvas/` floor only |
| CG-11.3 | OffscreenCanvas | Designed | GR 6.15, `hg-canvas` | GR-M3 "OffscreenCanvas"; GR-1 `html/canvas/` | Worker frames as own compositor surface; placeholder updates next window rendering step |
| CG-11.4 | ImageBitmap | Designed | GR 6.15 surface table | GR-M1; GR-1 `html/canvas/`; GR-8 | Per-constructor table: where pixels live, copy path, canonical or noised |
| CG-11.5 | Canvas float16 and HDR | Owned | GR 6.15, 6.5 (E16) | GR-M1/M2; GR-1 `html/canvas/` | One sentence: float16 "follow the HTML standard with the encodings of 6.5"; HDR canvas display output undesigned |
| CG-11.7 | WebGL 1 | Designed | GR 6.14 (GR-D9), `hg-webgl`, `hg-glsl-es` | GR-M2; GR-4 Khronos ≥ 99.5% | No ANGLE; GL state machine on HAL; listed GL-convention emulations each tested |
| CG-11.8 | WebGL 2 | Designed | GR 6.14 (GR-D9), `hg-webgl` | GR-M2; GR-4 Khronos WebGL 2 ≥ 99.5% | Same design; transform feedback emulation flagged (no Metal equivalent) |
| CG-11.10 | WebGPU | Designed | GR 6.13 (GR-D11), `hg-webgpu` | GR-M3; GR-3 CTS ≥ 99%; GR-S6 | Cross-origin-isolated by default, permission elsewhere; buffered command stream, total validation |
| CG-11.11 | WGSL | Designed | GR 6.12 (GR-D7), `hg-wgsl`, `hg-shader-ir` | GR-M1 (internal), GR-M3; GR-3 | LALR(1) parser, uniformity analysis, direct MSL/SPIR-V/DXIL emission, fixed builtin library |
| CG-11.12 | WebGPU optional features | Designed | GR 6.13 (GR-D11) | GR-M3; GR-3 | ≤ 4 buckets fix limits/features/subgroups; timestamps ≥ 100 µs; compat as feature level. Workers unnamed |
| CG-11.14 | `img.decode()`, `decoding`, lazy loading | Designed | MD 6.6 `hg-imgpipe`; LY 6.15 lazy-load observer `hg-layout-query` | MD-M1, LY-M2; LY anchor "HTML lazy loading" WPT | `decoding=sync` holds ≤ 1 frame; decode() resolves at decision point; priority order |
| CG-12.1 | JPEG | Designed | MD 6.2 (MD-D1) | MD-M1; MD-2 (T.83 tolerance) | Pins one integer IDCT and upsampler; web subset only, 12-bit/arithmetic not exposed |
| CG-12.2 | PNG 3rd edition | Designed | MD 6.2, 6.5 | MD-M1; MD-1 PngSuite | Pinned 3rd Edition; descriptor precedence `cICP` > `iCCP` > `sRGB` > `cHRM`+`gAMA` |
| CG-12.3 | APNG | Designed | MD 6.2, 6.6 (MD-D19) | MD-M1; MD-1 PngSuite | Frame-by-frame, ≤ 2 decoded frames, timeline clock, reduced-motion first-frame control |
| CG-12.4 | GIF | Designed | MD 6.2, 6.6 (MD-D19) | MD-M1 | Delay clamp measured black-box at MD-M0; NETSCAPE2.0 looping as observation |
| CG-12.5 | WebP | Owned | MD 6.2 (MD-D1) | MD-M1; MD-1 RFC 9649 lossless cases | Table row: RFC 9649, exact; built from the RFC (spot-check confirmed) |
| CG-12.6 | AVIF | Designed | MD 6.2, MD-D2 gate | MD-M1 (still), MD-M2; MD-1 Argon/AOM | Still, sequences, `tmap` gain maps; decoder gated by FP 6.5 peak-memory measurement |
| CG-12.7 | JPEG XL | Designed | MD-D2 | MD-M3; 18181-3, MD-50 | Ships when 18181-3, MD-50 speed and peak memory pass; patent review first |
| CG-12.11 | BMP, ICO, CUR | Designed | MD 6.2 (MD-D1) | MD-M1 | PNG-in-ICO kept; JPEG and PNG inside BMP dropped, as Microsoft did |
| CG-12.12 | SVG as image | Designed | SV 6.8 (SV-D10–D12); MD 6.1 (MD-D20) | SV-M1 (renderer), SV-M2 (media process); SV-53 | Secure static/animated modes; raster at concrete size; opaque images in media process |
| CG-12.13 | HDR gain-map images | Designed | MD 6.5 (MD-D9) | MD-M2; MD-6 | ISO 21496-1 canonical, Ultra HDR fallback; weight from live headroom and `dynamic-range-limit` |
| CG-12.14 | ICC colour management | Designed | MD 6.5 (MD-D8), `hg-colour` | MD-M1; MD-5 ΔE00 ≤ 0.5 | v2/v4.4, generated parser, tetrahedral CLUT f32 contract; no ICC file ships |
| CG-12.15 | Wide-gamut and HDR output | Designed | MD 6.5 (MD-D8, MD-D9); GR 6.5 (GR-D16) | MD-M2, GR-M2; MD-6 | Class W linear FP16; BT.2390 EETF; E16 group encoding; display conversion only at end |
| CG-12.16 | Image density correction | Mentioned | MD 6.6 (one clause) | — | "density-corrected size where the format carries it"; no rules, no `density-size-correction/` floor |
| CG-12.17 | Image MIME sniffing | Designed | NW 6.2 `hg-net-wire` (MIME Sniffing); MD 6.14 | NW-M1, MD-M1; MD-20 | "One decoder per format chosen by the MIME Sniffing standard's rules; no fall-through" |
| CG-13.1 | MP4 / ISO BMFF, CMAF | Owned | MD 6.2 container table | MD-M2; MD-3; WPT `media-source/` | ISO/IEC 14496-12, -15, 23000-19 pinned; generated parser; MD-3 corpus differential |
| CG-13.2 | WebM / Matroska | Owned | MD 6.2 container table | MD-M2; MD-3 | RFC 9559 plus WebM guidelines; Matroska test files as oracle |
| CG-13.3 | Ogg | Owned | MD 6.2 container table | MD-M2; MD-3 | RFC 3533, 7845, 5334 pinned |
| CG-13.4 | Native HLS | Designed | MD 6.7 (MD-D21) | MD-M3 | Playlist parser in media process; ABR on partition throughput; 10 s/30 s buffering policy |
| CG-13.6 | H.264 / AVC | Designed | MD 6.2 (MD-D3), `hg-h264` | MD-M2 (OS), MD-M3 (Linux per licence); MD-1 H.264.1 | Option (b): OS decoders macOS/Windows; own decoder built, Linux after legal review |
| CG-13.7 | H.265 / HEVC | Designed | MD 6.2 (MD-D3), G57 | MD-M2 | OS full decoders only, never Izcal code; never low-level hardware APIs |
| CG-13.9 | VP8 | Designed | MD 6.2, MD-D4 | MD-M2; MD-1 RFC 6386 vectors | From RFC narrative plus vectors and black-box `vpxdec`, not RFC code |
| CG-13.10 | VP9 | Owned | MD 6.2 | MD-M2; MD-1 VP9 test vectors | Profiles 0–3, VP9 spec v0.7 pinned, exact |
| CG-13.11 | AV1 | Designed | MD 6.2; S1 13.2 | MD-M1 (intra), MD-M2; MD-1 Argon | Film-grain synthesis exact; first conformance option; SIMD spike MD-S7 |
| CG-13.13 | AAC (LC, HE-AAC) | Designed | MD 6.2 (MD-D3) | MD-M2 (OS), MD-M3; MD-2 | AAC-LC own decoder where patents confirmed expired; HE-AAC OS decoders only |
| CG-13.14 | MP3 | Owned | MD 6.2 | MD-M2; MD-2 ISO tolerance | Table row, Izcal pins arithmetic; spot-check confirmed |
| CG-13.15 | Opus | Designed | MD 6.2, MD-D4 | MD-M2; MD-2 RFC 8251 vectors | From prose; conformance by black-box reference plus final range-coder state |
| CG-13.16 | Vorbis | Owned | MD 6.2 | MD-M2; MD-2 (−90 dBFS of float64 reference) | Vorbis I pinned; tolerance, Izcal pins arithmetic |
| CG-13.17 | FLAC | Owned | MD 6.2 | MD-M2; MD-1 FLAC files | RFC 9639, exact |
| CG-13.18 | PCM/WAV, G.711, G.722 | Owned | MD 6.2; MD 6.10 (MD-D30) | MD-M2 ("PCM"; WebRTC "G.711") | PCM, A-law, µ-law named; G.722 zero hits in any report |
| CG-13.21 | MIME and codec strings | Designed | MD 6.15 (MD-D11) | MD-M2; MD-30 | One per-build policy table answers `canPlayType`, `isTypeSupported`, Media Capabilities; RFC 6381 unnamed |
| CG-14.1 | `<video>`, `<audio>` | Designed | MD 6.7 (`hg-media`, MD-D12) | MD-M2; MD-4 `html/semantics/embedded-content/media-elements/`; DM-1 `html/semantics/` | readyState machines, fixed timeupdate 250 ms, progress 350 ms, stalled 3 s |
| CG-14.2 | Default media controls | Designed | MD 6.12 (`hg-media` UA shadow tree); shell UI design (exit affordance) | MD-M2 "native controls (MD-12)"; MD-12 | Control set listed, semantic-plane roles, keyboard, forced colours; `controlslist` not named |
| CG-14.3 | Text tracks | Designed | MD 6.12 (`hg-media`, generated WebVTT parser) | MD-M2; MD-4 media-elements/ (track tests); DM-1 `html/semantics/` | Typed cues to renderer; enter/exit/cuechange on media clock, "time marches on" |
| CG-14.4 | WebVTT | Designed | MD 6.12; ST 6.9 (`::cue`) | MD-M2 "WebVTT with OS caption preferences"; MD-4 `webvtt/` | Generated parser in media process; OS caption prefs applied, not observable |
| CG-14.6 | Media Source Extensions | Designed | MD 6.7 (MD-D22, `hg-media`) | MD-M2; MD-4 `media-source/` | Canonical quotas 128 MiB video / 16 MiB audio; eviction independent of device |
| CG-14.7 | MSE in workers, ManagedMediaSource | Owned | MD 6.7, 6.20 (MD-D21) | MD-M2 "MSE, ManagedMediaSource"; MD-4 `media-source/` (incl. dedicated-worker tests) | Spot-check Only partial: MMS designed; MSE-in-workers, `MediaSourceHandle` only listed in 6.20 |
| CG-14.8 | Encrypted Media Extensions | Designed | MD 6.11; S1 13.1a (D-S7) | MD-M2 (EME Clear Key, PlayReady); S1 M7; MD-4 `encrypted-media/` | Clear Key sessions in renderer, keys stay in media process via `hg-crypto` |
| CG-14.9 | Content Decryption Modules | Designed | S1 13.1a (D-S7); MD 6.11 (MD-D32) | S1 M7; MD-M2 PlayReady exit (needs MD-S5) | Layered licensed CDMs; protected media mode; truthful key-system answers; Linux Clear Key only |
| CG-14.10 | Media Capabilities | Designed | MD 6.7, 6.15 (MD-D11 policy table) | MD-M2 (MD-30 measured); MD-30 | Per-build per-OS-family table; `encodingInfo` from same table as `decodingInfo` |
| CG-14.12 | `requestVideoFrameCallback` | Designed | GR 6.11 (GR-D26); MD 6.7, 6.15 | GR-M2 "video layers"; — no rVFC floor | Metadata from compositor frame record; frame at official position; `processingDuration` coarsened |
| CG-14.13 | Media Session | Designed | MD 6.12 (broker to OS controls) | MD-M2 "Media Session"; MD-4 `mediasession/` | Metadata/actions via broker; artwork decoded in page's media process (thin) |
| CG-14.14 | Picture-in-Picture (video) | Designed | MD 6.12 (shell-owned OS window) | MD-M2 "Picture-in-Picture"; MD-4 `picture-in-picture/` | GPU-service surface in shell window; conflicts with S1 13.2 listing PiP Stage 2 |
| CG-14.15 | Document Picture-in-Picture | Designed | MD 6.12 | MD-M2 "Picture-in-Picture" (implicit); anchor "Document PiP" | New top-level context in opener's renderer; single-engine, no admission record |
| CG-14.18 | Audio Output Devices | Designed | MD 6.8 (audio service, broker chooser) | MD-M2 "audio service"; — no `audio-output/` floor | Spot-check Confirmed. Per-(site, container) IDs, `speaker-selection` policy, recorded `sinkchange` |
| CG-14.20 | Autoplay policy | Designed | MD 6.12 (MD-D13) | MD-M2 "autoplay rule"; MD-4 `autoplay-policy-detection/` | Stateless rule: activation or site setting; muted may autoplay; `getAutoplayPolicy()` |
| CG-14.21 | WebCodecs | Designed | MD 6.7 | MD-M2 decode, MD-M3 encode; MD-4 `webcodecs/` | Canonical saturation 4 requests; prefer-software exact; `decoderImplementation` constant per OS |
| CG-14.22 | ImageDecoder | Owned | MD 6.20 (WebCodecs row, image decoders in media process) | MD-M2 "WebCodecs decode"; MD-4 `webcodecs/` (image-decoder tests) | Spot-check Only partial: one table cell; frame/track access, streaming input undesigned |
| CG-14.23 | Web Audio | Designed | MD 6.8 (`hg-webaudio`, MD-D14) | MD-M2; MD-4 `webaudio/`; MD-30, MD-40 | Bit-identical f32 rendering, one FFT, 48 kHz default everywhere, ownership rule |
| CG-14.24 | AudioWorklet | Designed | MD 6.8 (MD-D31) | MD-M2 "Web Audio with AudioWorklet"; MD-9, MD-10; gated by MD-S8 | Real-time agent requirements stated; option (b) ships until MD-S8 passes |
| CG-14.25 | Web MIDI | Designed | DV 6.6, 6.10 (device-access service, DV-D9) | DV-M1, DV-M2 (behind Experimental setting); DV-M3 exposure decision DV-E5 | SysEx separate tier, prompt per grant; general exposure held by DV-D9 |
| CG-15.1 | Media Capture and Streams | Designed | MD 6.10 (capture service, broker) | MD-M2 "capture service"; MD-4 `mediacapture-streams/` | Pre-grant `enumerateDevices` one device per kind, empty IDs; frames hashed in recordings |
| CG-15.5 | Screen Capture | Designed | MD 6.10; GR 6.27 (capture split) | MD-M2; GR-M3 "capture"; GR-24 | Tab/element via compositor, screen/window via capture service; broker consent prompt (GR-D28) |
| CG-15.7 | Region Capture | Owned | GR 6.27 (compositor table row, `cropTo`) | GR-M3; GR-24 | Named in capture row with process and rule; single-engine, no admission record |
| CG-15.8 | Element Capture | Designed | GR 6.27 (`restrictTo`) | GR-M3; GR-24 | Only restricted element's subtree composed, stacking context required; single-engine, no admission record |
| CG-15.10 | MediaStreamTrack insertable streams | Mentioned | MD 6.20 (`MediaStreamTrackProcessor` listed); MD 6.7 | — (no milestone names it; `mediacapture-insertable-streams/` not floored) | MSTP named in coverage table and noise rule only; `VideoTrackGenerator` absent |
| CG-15.11 | MediaRecorder | Designed | MD 6.7 | MD-M3 "MediaRecorder"; MD-4 `mediacapture-record/` | WebM everywhere, MP4 where MD-D3 allows; `timeslice` chunks in media time |
| CG-15.15 | RTCDataChannel | Owned | NW 6.12 (`hg-sctp`, RFC 8831) | NW-M4; NW-42 "data channels complete", WPT `webrtc/` floors at NW-M4 | Transport designed; JS `RTCDataChannel` object unowned (CG-15.14 gap) |
| CG-15.16 | WebRTC statistics | Designed | MD 6.10 | MD-M2 "WebRTC media"; MD-4 `webrtc/`; anchor WebRTC Statistics | Stats computed from recorded packet arrival times; `getStats` entry point unowned (CG-15.14) |
| CG-15.17 | WebRTC Encoded Transform | Mentioned | MD 6.10, 6.20 (one sentence) | — (MD-M2 says "WebRTC media" only; `webrtc-encoded-transform/` not floored) | "route encoded frames to the page's worker and back"; no anchor, no floor |
| CG-16.5 | Printing | Designed | LY 6.7 (LY-D8); GR 6.27; S1 13.2 (print UI Stage 2) | LY-M2 "paged media"; GR-M3 print raster; GR-28 | Break scoring with appeals, paged layout Stage 1; print raster in content GPU process |
| CG-16.7 | Built-in PDF viewer | Designed | PD (PD-D1 to PD-D26; 6.15 viewer) | PD-M1, PD-M2 (Stage 1 M7) | PDF process per document; HTML viewer app; editing Stage 2 |
| CG-16.8 | Fullscreen rendering | Designed | DM 6.9 (element side), 6.18 `hg-webapi`; MD 6.12; ST 6.7.1 | DM-M2 "top layer"; MD-M2 | Broker-minted activation (DM-D20); video direct scanout; `fullscreen/` WPT not floored |
| CG-16.9 | Scroll-to-text-fragment rendering | Designed | ED 6.12 (`hg-find`); DM 6.9; LY 6.17 | ED-M2; ED anchor WPT `scroll-to-text-fragment/` | Constant-shape matching, single-use top-level activation; `::target-text` via overlays |
| CG-16.10 | Find-in-page highlighting, `until-found` | Designed | ED 6.12 (ED-D17, `hg-find`); LY 6.9 | ED-M2 "find in page across frames"; WPT `html/editing/the-hidden-attribute/` | Reveal (`beforematch`) on Enter or 500 ms pause; `::search-text` held by LY-D10 |

## Web platform APIs: every Covered row

| # | Feature | Grade | Owner (spec §) | Milestone/target | Evidence (≤ 15 words) |
|---|---|---|---|---|---|
| WA-1.1 | Document and browsing-context model | Designed | DM 6.8, 6.4 (`hg-agents`, DM-D6, DM-D7) | DM-M2; DM-1 `html/browsers/` 95% | Origin-keyed clusters, whole clusters per process, remote `WindowProxy`, COOP group switches |
| WA-1.2 | Navigation and session history | Designed | DM 6.9 (`hg-navigation`, DM-D20) | DM-M2; DM-1 `html/browsers/` 95% | Broker/renderer split table, 1 s† beforeunload deadline, child-before-parent unload order |
| WA-1.3 | Navigation API | Designed | DM 6.9 (`hg-window` renderer, broker commits) | DM-M2 names it; DM-4; DM-1 `navigation-api/` 95% | `navigate`/`intercept()`/`NavigationPrecommitController`; renderer proposes entries, broker commits (spot-check Confirmed) |
| WA-1.4 | Back/forward cache | Designed | DM 6.9 (DM-D9, DM-D14), DM 6.12 exception | DM-M2; DM-28, DM-31 | Policy table: no-store 3 min†, opener, locks; masked `notRestoredReasons` |
| WA-1.5 | Page Visibility and Page Lifecycle | Designed | DM 6.14 (FP-D4), DM 6.9; DM-D15 exception | DM-M2/M3 | Broker freezes whole groups after 30 s†; held inputs; `wasDiscarded` set |
| WA-1.6 | `beforeunload` / `unload` | Designed | DM 6.9 (DM-D9) | DM-M2; DM-1 `html/browsers/` 95% | `unload` only when unsalvageable; 1 s† fan-out; prompt surface has no shell 6.4 class |
| WA-1.7 | iframes and embedding | Designed | DM 6.12 (`hg-html`), DM 6.9; LY 6.15 | DM-M2/M3; DM-1 `html/semantics/` 97% | Out-of-process frame geometry over typed IPC; sandbox flags in broker |
| WA-1.10 | `Location.ancestorOrigins` | Owned | DM 6.3 corpus, DM-D15; `hg-window` (DM 6.17) | DM-M2 (`Location`); DM-1 `html/browsers/` 95% | In WHATWG HTML; spot-check Confirmed; not named |
| WA-1.11 | Origin API | Mentioned | — (HTML claim only) | — | Named nowhere; recent HTML addition, pin date unrecorded (spot-check Only partial) |
| WA-1.12 | Text fragments | Designed | ED 6.12 (`hg-find`, ED-D17) | ED-M2 | Text directives share matcher, constant-shape rule; top-level-only scrolling (LY 6.17) |
| WA-1.13 | Top-navigation restrictions | Owned | DM 6.9 (broker sandbox and history-action checks, DM-D20) | DM-M2; DM-1 `html/browsers/` 95% (sandboxing) | HTML sandbox flags decided in broker; top-navigation rule itself not named |
| WA-1.14 | Initial `about:blank` | Owned | DM 6.9 (renderer `hg-window`, "Document creation and initialization") | DM-M2; DM-1 `html/browsers/` 95% | In HTML at pin; spot-check Confirmed; sync initial load not named |
| WA-1.15 | Printing | Designed | LY 6.7 (LY-D8); GR 6.27 | LY-M2 paged media; LY-1 `css-page` 90%; print UI Stage 2 | Paged layout, `@page`, `window.print()` Stage 1; `beforeprint` snapshot |
| WA-1.16 | Simple dialogs | Mentioned | DM 6.7 (`hg-script` pause); rendering assigned to shell, unclaimed | DM-1 `html/webappapis/` 97% (DOM side only) | Pause designed; shell 6.4 has no prompt class for alert/confirm/prompt (spot-check partial) |
| WA-1.17 | Window and screen properties | Designed | DM 6.16, 6.18 (broker environment-values record, D-S1, OD-6) | DM-M2 | `screen`, `devicePixelRatio` from one normalized broker record, PRIV-2 budget |
| WA-2.1 | HTML parser and serializer | Designed | ST 6.2–6.5 (`hg-html-syntax`, `hg-html-driver`, ST-D1–D3) | ST-M1; ST-1 100% html5lib/tree-construction; ST-2 | One parser core, speculative parser on mock nodes, 512 depth cap |
| WA-2.2 | Sectioning, grouping, text-level elements | Owned | DM 6.12 (`hg-html`) | DM-M3 full element set; DM-1 `html/semantics/` 97% | Element table row "Document structure, sections, grouping, text-level semantics… `search`" |
| WA-2.4 | Links and `<a>` | Owned | DM 6.12 (`hg-html`); CB 6.2; NW 6.14 | DM-M2/M3; DM-1 `html/semantics/` 97% (links) | `a`, `area` owned; ping and downloads designed elsewhere; attributes not enumerated |
| WA-2.5 | `<link>` relation types | Owned | DM 6.12 (`hg-html`); NW 6.3, 6.11; ST 6.6a | DM-M3; NW-M2 | Rel processing, preload, modulepreload, Early Hints "as HTML specifies"; manifest unconsumed |
| WA-2.6 | `<meta>` names and pragmas | Owned | DM 6.12 (`hg-html`); NW 6.7; DM 6.9 | DM-M3; DM-1 `html/semantics/` 97% (document-metadata) | `meta` owned in element table; CSP meta in NW; refresh in navigation |
| WA-2.7 | `<template>` and `<slot>` | Designed | DM 6.6, 6.12 (`hg-dom`) | DM-M1; DM-1 `shadow-dom/` 97% | Incremental slot assignment with needs-assignment flags in core phase |
| WA-2.8 | Responsive images markup | Owned | DM 6.12 (selection in `hg-dom`); MD 6.20 | DM-M3; DM-1 `html/semantics/` 97% (embedded-content) | Selection algorithm "HTML's, in `hg-dom`"; `sizes=auto` by claim |
| WA-2.9 | Lazy loading | Designed | NW 6.3; LY 6.15 (`hg-layout-query` lazy-load observer) | NW-M2; LY-M2; DM-1 `html/semantics/` 97% | Lazy-load starts are recorded decision points; per-frame intersection classes |
| WA-2.10 | `<details>` and `<summary>` | Owned | DM 6.12 (`hg-html`); ED 6.12 | DM-M2; DM-4 `details` area 99% | Named element; Interop-labelled tests are release gates |
| WA-2.11 | `hidden="until-found"` | Designed | ED 6.12 (`hg-find`); AX-O11 | ED-M2 | `beforematch`; reveal on Enter, next or 500 ms† idle (spot-check Confirmed) |
| WA-2.12 | Global attributes | Owned | DM 6.5, 6.12; ED 6.8, 6.13 (ED-D18) | DM-M2; ED-M1/M2; DM-1 `html/dom/` 97% | tabindex, popover, inert, inputmode, spellcheck owned; set not enumerated |
| WA-2.13 | `autocorrect` attribute | Designed | ED 6.8 (ED-D19), 6.13 | ED-M3 | Element autocorrect state per HTML defaults drives Izcal autocorrect |
| WA-2.17 | MathML Core | Designed | LY 6.5.9 (`hg-layout-math`, LY-D21); DM 6.12 | LY-M2; LY-1 `mathml` 90% | Math box, baselines, italic correction over text stack's MATH data |
| WA-2.18 | Obsolete and removed features | Designed | DM 6.12 (DM-D15); ST-D4 | — | Legacy table: `document.all`, cross-origin dialogs kept; mutation events, XSLT refused |
| WA-3.1 | Form submission | Owned | DM 6.12 (`hg-forms`) | DM-M2 forms; DM-1 `html/semantics/` 97% (forms) | Entry list with `formdata`, submission "table-driven", each step anchored |
| WA-3.2 | Input types | Owned | DM 6.12 (`hg-forms`); LY-D11 | DM-M2; DM-1 `html/semantics/` 97% | 22 types owned; date/colour/select pickers have no owning shell design |
| WA-3.3 | Constraint validation | Owned | DM 6.12 (`hg-forms`) | DM-M2 "forms and validation"; DM-1 `html/semantics/` 97% | `ValidityState` ten flags, static and interactive validation; message UI unowned |
| WA-3.4 | Standard form controls | Designed | DM 6.12 (`hg-forms`); LY 6.3 (LY-D11) | DM-M2; LY-M2 | UA shadow trees with Izcal-defined intrinsic metrics, no OS widget metrics |
| WA-3.6 | `showPicker()` | Mentioned | — (HTML claim; LY-D11 says pickers are browser-process UI) | — | Not named; no shell or UI spec owns the pickers (spot-check partial) |
| WA-3.7 | Autofill and `autocomplete` | Designed | DM 6.12; ED 6.6 (autofill door); SS CON-R4 | Stage 1 passwords; Stage 2 beyond (S1 13.2) | Fills are edit intents via broker; shell never holds passwords |
| WA-3.11 | Colour input `alpha`/`colorspace` | Mentioned | — (HTML claim; LY-D11 picker) | — | Not named; picker unowned; inclusion at HTML pin unchecked (spot-check partial) |
| WA-3.12 | Form-associated custom elements | Owned | DM 6.6, 6.12 (`hg-forms`, `ElementInternals`) | DM-M2; DM-1 `custom-elements/` 97% | Four form callbacks, `ElementInternals` form association named |
| WA-4.1 | `<dialog>` | Owned | DM 6.12 (`hg-html`, top layer) | DM-M2 dialog; DM-4 dialogs area 99% | One top-layer list, `showModal`, focus restoration "follow HTML at the pin" |
| WA-4.2 | `closedby` and `requestClose()` | Owned | DM 6.12 (`hg-html`) | DM-M2; DM-4 dialogs area 99% | `closedby` named; `requestClose()` by HTML claim within dialog area |
| WA-4.3 | Popover API | Owned | DM 6.12 (`hg-html`) | DM-M2 popover; DM-4 popovers area 99% | `popover`, light dismiss, `:open` follow HTML; Interop tests release gates |
| WA-4.4 | `popover="hint"` | Owned | DM 6.12 (`hg-html`) | DM-M2; DM-4 | "`popover` (`auto`, `manual`, `hint`)" |
| WA-4.5 | Invoker commands | Owned | DM 6.12 (`hg-html`) | DM-M2; DM-4 | "invokers (`command`, `commandfor`)" follow HTML at the pin |
| WA-4.7 | `CloseWatcher` | Owned | DM 6.3 corpus; DM 6.12 light dismiss | DM-M2 (top layer) | Not named; in HTML; spot-check Confirmed |
| WA-4.8 | Light-dismiss input timing | Owned | DM 6.12 (`hg-html`) | DM-M2; DM-4 popovers area | "light dismiss … follow HTML at the pin" |
| WA-4.9 | Focus management | Designed | DM 6.5 (`hg-html::focus`); AX 6.6 (`hg-ax-focus`) | DM-M2 "input, focus and activation" | Surface focus broker-granted; `:focus-visible` modality a recorded input |
| WA-4.10 | `inert` | Owned | DM 6.5 (`hg-html::focus`) | DM-M2 focus | Named in HTML focus list; no Izcal-specific design |
| WA-4.12 | User activation | Designed | DM 6.5 (DM-D20) | DM-M2 | Broker-minted activation, 5 s† transient, renderer mirror (spot-check Confirmed) |
| WA-4.13 | HTML drag and drop | Designed | ED 6.11 (`hg-dnd`, ED-D13) | ED-M2 | Drag store in broker; `DownloadURL` held |
| WA-4.14 | Fullscreen API | Designed | DM 6.7 (element side; broker window side), 6.18 (`hg-webapi`); MD 6.12; shell 6.8 | SH-M1 fullscreen; DM-M2 | Broker-activation gate; shell exit control, Esc/F11; `keyboardLock` not named |
| WA-4.17 | `ariaNotify()` | Designed | AX 6.7; DM 6.18 | AX milestones | Constant-time record, no return value, rate and length limits |
| WA-4.18 | Editing hooks | Designed | ED 6.2 (ED-D1), ED-D2 | ED-M2 | Edit intents, `beforeinput` default actions; EditContext behind developer setting |
| WA-5.1 | Classic scripts | Designed | DM 6.12 (`hg-script`); ST 6.4 (ST-D2); JS 6.14 | DM-M2; ST-M1; DM-1 `html/semantics/` 97% | Parser scheduling, recorded yields, speculative fetches as recorded decision points |
| WA-5.2 | Module scripts | Designed | JS 6.14; DM 6.18 (`hg-script`) | JS-M1 ES modules | Phase-aware module state machine: fetch, source phase, instantiate, evaluate |
| WA-5.3 | JSON and CSS module scripts | Owned | JS 6.14; DM 6.3 corpus | JS-M2; DM-1 `html/semantics/` 97% (scripting-1) | JSON modules named; CSS modules by HTML claim (spot-check Confirmed) |
| WA-5.5 | Import maps | Owned | DM 6.18 (`hg-script`); JS 6.14 | DM-1 `html/semantics/` 97% (import-maps) | "module fetching with import maps (HTML)"; multiple maps by claim |
| WA-5.6 | `modulepreload` | Owned | DM 6.12 (`hg-html` link) | DM-M3 | `link` rel processing names `modulepreload` |
| WA-5.7 | Preload and resource hints | Owned | DM 6.12; NW 6.3, 6.11; ST 6.4 | NW-M2; DM-M3 | Early Hints "preload and preconnect handled as HTML specifies"; speculative keys set |
| WA-5.8 | Speculation Rules | Designed | DM 6.9 (prerendering), DM 6.7; NW 6.11 | DM-M3 (S1 M7) vs S1 §9 Stage 2 conflict | Prerender as ordinary agent, recorded activation; off under Hardened, memory pressure |
| WA-5.10 | Render-blocking | Owned | ST 6.6a; DM 6.7 (render-blocked filter) | ST-M1; DM-1/ST-2 `html/dom/` 97% | `blocking=render` delays first rendering opportunity, "HTML's rules, anchored" |
| WA-6.1 | DOM core | Designed | DM 6.1 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | Node columns, two-phase tree-edit transactions, mutation log, live collections, id maps |
| WA-6.2 | Events | Designed | DM 6.5 (`hg-dom` events core) | DM-M1; DM-1 `dom/` 97% | Dispatch path, retargeting, default-passive rule, non-passive listener column flag for compositor |
| WA-6.3 | Aborting | Owned | DOM claim (DM 6.3, DM 4.2); `hg-dom` | DM-M1; DM-1 `dom/` 97% (`dom/abort/`) | Spot-check confirmed; `AbortSignal` only listed in DM 4.2, no design |
| WA-6.4 | MutationObserver | Designed | DM 6.6, 6.1 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | Records from mutation log, compound microtask, transient observers; mutation events not implemented |
| WA-6.5 | Ranges, traversal and `StaticRange` | Designed | DM 6.1, 6.6 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | Live-range registry updated in core phase; NodeIterator/TreeWalker only via DOM claim |
| WA-6.6 | `moveBefore()` | Designed | DM 6.1 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | "Third primitive": one link rewrite, one `Move` log record, state preserved |
| WA-6.7 | Shadow DOM | Designed | DM 6.6 (`hg-dom`) | DM-M1; DM-1 `shadow-dom/` 97% | Incremental slot assignment in core phase; `slotchange` via signal slots; flat tree traits |
| WA-6.8 | Declarative shadow DOM | Designed | ST 6.4–6.5 (`hg-html-syntax`); DM 6.6 | ST-M1; ST-2 `shadow-dom/declarative` 99% | Parser flag: legacy sinks never attach shadow roots; newer methods do |
| WA-6.9 | Custom elements | Designed | DM 6.6 (`hg-dom`) | DM-M1; DM-1 `custom-elements/` 97% | Registries, ten reactions, reactions stack; customized built-ins implemented; `[CEReactions]` DM-21 |
| WA-6.10 | Scoped custom element registries | Designed | DM 6.6 (`hg-dom`) | DM-M1; DM-4 Interop; DM-1 `custom-elements/` | Registries owned by document, shadow root or element; Interop 2026 gate |
| WA-6.12 | ARIA reflection | Designed | DM 6.18, 6.3 (reflection codegen); AX 6.2 (IDREF resolver in `hg-dom`) | DM-M2; DM-1 `html/dom/` 97% (aria reflection tests) | Element reflection: weak, outward-only explicit targets; Reference Target redirect rules |
| WA-6.13 | DOM Parsing and Serialization | Designed | ST 6.4–6.5 (`FragmentSink`, serializer) | ST-M1; ST-2 `domparsing` ≥ best engine; ST-8 | One parser for `innerHTML`, `DOMParser`, `*HTML`; round-trip fixed-point suite |
| WA-6.14 | HTML Sanitizer API | Designed | ST 6.5 (`SanitizingSink`); DM 6.16 | ST-M1; ST-2 `sanitizer-api` ≥ best engine; ST-8(c) | Sanitizing inside the parser; allow/remove lists compiled to atom sets |
| WA-6.16 | Selection API | Designed | DM 6.6 (data model); ED 6.4 (`hg-caret`) | DM-M2; ED-M1; DM-1 `selection/` 95% | Selection in range registry; per-target `selectionchange` bit; editing boundary stated |
| WA-6.17 | UI Events | Designed | DM 6.5 (`hg-input`) | DM-M2; DM-1 `uievents/` 95% | Event order, user-interaction task source, recorded inputs, coarsened key timestamps |
| WA-6.18 | Pointer Events | Designed | DM 6.5 (`hg-input`) | DM-M2 ("input"); no `pointerevents/` floor | Pointer then compatibility mouse order; coalescing as function of recorded inputs |
| WA-6.19 | Touch Events | Designed | DM 6.5 (`hg-input`, `hg-dom`) | DM-M2 ("input"); no `touch-events/` floor | Default-passive rule; non-passive flag lets compositor scroll without renderer |
| WA-6.21 | Observers | Designed | LY 6.15 (`hg-layout-query`) | LY-M2; LY-1 `resize-observer/`, `intersection-observer/` 95% | Depth-ordered RO loop; IO v2 provable-visibility rule, cross-process visibility bit |
| WA-6.22 | Scrolling and caret APIs | Designed | LY 6.15 (`hg-layout-query`, `hg-scroll`); DM 6.7 | LY-M1; LY-1 `cssom-view` 95%; DM-4 (`scrollend`) | CSSOM View, `caretPositionFromPoint`, `checkVisibility`, forcing table derived from specs |
| WA-6.24 | View Transitions, script side | Designed | AN 6.x (`hg-view-transition`, `hg-anim-api`); DM 6.7 | AN-M2, AN-M3; AN-1 `css/css-view-transitions` 92%; AN-2 | Lifecycle as specified; cross-document eligibility in broker (AN-53) |
| WA-6.25 | Geometry interfaces | Mentioned | SV 6.x (Geometry Interfaces 1 pinned as input); DM 6.18 "geometry APIs" to layout | — | `DOMMatrix`/`DOMPoint` named only as `double`; no owner crate; `css/geometry/` no floor |
| WA-6.26 | Visual Viewport | Mentioned | LY 6.1, LY-D13; GR 6.28 (compositor mechanics) | — | Pinch-zoom mechanics designed; `visualViewport` API never named; `visual-viewport/` under no floor |
| WA-7.1 | Event loop, tasks and microtasks | Designed | DM 6.7 (`hg-script`, DM-D1) | DM-M1; DM-1 `html/webappapis/` 97%; DM-22, DM-24 | Five-class selection policy, anti-starvation, decision-point register, no nested loops |
| WA-7.2 | Timers and animation frames | Designed | DM 6.7 (DM-D10, AN-D3) | DM-M1; DM-14; DM-1 `html/webappapis/` | No 1 ms clamp; hidden-page 1 s alignment; one recorded tick per frame |
| WA-7.3 | `requestIdleCallback` | Designed | DM 6.7 (`hg-script`) | DM-M1 (rIC named) | Idle class: 50 ms cap lowered to next timer/frame, then GC work |
| WA-7.4 | Prioritized Task Scheduling | Designed | DM 6.7 (`hg-script`) | DM-M1 (`postTask` named) | Effective priorities 0–5 mapped to policy classes; `yield()` inherits priority via job callbacks |
| WA-7.6 | Error reporting | Owned | HTML claim (DM 6.3, DM-D15); DM 6.7 (`hg-script`) | DM-M1; DM-1 `html/webappapis/` 97% | Spot-check confirmed; checkpoint "notifies rejected promises"; `reportError` not named |
| WA-7.7 | Dedicated workers | Designed | DM 6.10 (`hg-workers`, `hg-agents`) | DM-M2; DM-1 `workers/` 95%; DM-26 | Broker-computed lifetime predicates; ≤ 768 KiB charged, 256 KiB stack |
| WA-7.8 | Shared workers | Designed | DM 6.10, DM-D8 (`hg-agents`) | DM-M3; DM-1 `workers/` 95% | Spot-check confirmed; 5 s between-loads, 30 s extended-lifetime timeouts |
| WA-7.9 | Worklets | Designed | DM 6.7, 6.8 (`hg-workers`); JS 6.12 | — (no milestone names worklets; `worklets/` no floor) | Two global scopes chosen by seeded random assignment; worklet agents in cluster |
| WA-7.10 | Cross-document messaging | Designed | DM 6.11, DM-D13 (`hg-messaging`, broker) | DM-M2; DM-1 `webmessaging/` 95%; DM-25 | Per-document-pair endpoints; broker stamps origin; `targetOrigin` checked in broker |
| WA-7.11 | Channel messaging | Designed | DM 6.11, DM-D13 (`hg-messaging`) | DM-M2; DM-1 `webmessaging/` 95% | Ports as B2 endpoint pairs, re-minted on transfer, one IPC hop |
| WA-7.12 | Broadcast Channel | Designed | DM 6.11 (`hg-agents` routing) | DM-M2; DM-1 `webmessaging/` 95% (`broadcastchannel/`) | Broker delivers by name and storage key; partitioned; creation order kept |
| WA-7.13 | Structured clone and transfer | Designed | DM 6.11, DM-D12 (`hg-clone-wire`) | DM-M2; DM-1 `html/webappapis/` 97%; DM-12 | One versioned wire format; deserialization runs no author code; no writable mapping |
| WA-7.14 | SharedArrayBuffer and Atomics | Designed | JS 6.12 (`hg-js-runtime`); DM 6.8 | JS-M2; JS-1 test262 `Atomics` | Buffer region shared within one-process agent cluster; gated on cross-origin isolation |
| WA-7.15 | Web Locks | Designed | DM 6.17 (`hg-webapi`, `hg-agents`); SG 6.3, SG-D12 | — (no milestone; `web-locks/` no floor) | Broker routing, stateless, broker-minted partition key (G99), bfcache eviction rule |
| WA-8.1 | Fetch API | Designed | NW 6.7 (partition service); DM 6.18 (`hg-webapi`) | NW-M1; NW-1 `fetch/` ≥ 95% | Renderer/service split of Fetch steps; anchors to `fetch.bs` (NW-10) |
| WA-8.3 | `fetchLater()` | Owned | NW 6.11 (partition service) | NW-M4 (not itemized); NW-1 `fetch/` (`fetch/fetch-later/`) | One sentence: owned by partition service, survives document; single-engine, no admission record |
| WA-8.4 | Body helpers | Owned | DM 6.18 (`hg-webapi`, body mixin) | NW-M1 Fetch core; NW-1 `fetch/` (`fetch/api/`) | Body mixin listed in 6.18 table; no Izcal design; `textStream()` unnamed |
| WA-8.5 | XMLHttpRequest | Designed | DM 6.18, 6.7 (`hg-webapi`); NW 6.7 | DM-M2; NW-1 `xhr/` ≥ 95% | Sync XHR as "blocked on fetch" scheduler state; disallowed during dismissal |
| WA-8.6 | Server-sent events | Mentioned | DM 6.18 (`hg-webapi`, "`EventSource` objects") | — | Listed only; no NW design; `eventsource/` under no floor; no milestone names it |
| WA-8.8 | WebSockets | Designed | NW 6.11 (partition service) | NW-M2 (H1, H2), NW-M3 (H3); NW-7 | Handshake through Fetch, LNA, frozen pages keep sessions with 1 MiB buffer |
| WA-8.10 | WebTransport | Designed | NW 6.11, NW-D13 | NW-M4, gate rule (WPT `webtransport/` + two servers) | Pinned drafts, enable gate stated, frozen-session rule, LNA |
| WA-8.12 | Streams | Mentioned | DM 6.18, 6.17 (`hg-webapi`, "bindings and algorithms") | — | Named in corpus and table only; `streams/` under no floor; no milestone |
| WA-8.13 | Compression Streams | Mentioned | DM 6.18 (`hg-webapi` over `hg-compress`) | — (NW-M1 ships `hg-compress` decoders only) | API listed only; encoders unscheduled; `compression/` under no floor |
| WA-8.14 | Encoding API | Designed | ST 6.x, ST-D16 (`hg-encoding`); DM 6.18 | ST-M0; ST-15 `encoding/` ≥ 97% | Own decoders/encoders, tables generated from standard indexes; one crate for all callers |
| WA-8.15 | URL and URLSearchParams | Designed | ST (`hg-url`, G31); DM 6.18; JS 6.13 (UTS 46) | ST-M0; ST-15 `url/` ≥ 97% | The one URL parser, graph-checked; NW-M1 depends on it |
| WA-8.17 | MIME Sniffing | Owned | NW 6.2 (`hg-net-wire`) | NW-M2 (ORB); NW-1 `fetch/` (`fetch/orb/`) | WHATWG MIME Sniffing named; one parser in `hg-net-wire`, used by ORB |
| WA-8.18 | Cookies as page-visible state | Designed | NW 6.9, NW-D10, NW-D16 (`hg-cookies`) | NW-M2; NW-2 `cookies/` ≥ 95% | Cookie view region per (host, partition); no sync IPC; CHIPS-style default |
| WA-9.1 | Web Storage | Designed | DM 6.13, DM-D11 (`hg-storage-dom`); SG 6.3, 6.7 | SG-M2; SG-3 | Read-only snapshot window, local overlay, last-writer-wins; sessionStorage in broker |
| WA-9.2 | IndexedDB 3 | Designed | SG 6.2–6.6 (`hg-store-engine`); DM 6.13 | SG-M2 ("Full IndexedDB surface"); SG-2 `IndexedDB/` recorded floor | Own engine, durability hints mapped to commit policy; `getAllRecords()` unnamed |
| WA-9.3 | Cache API | Designed | SG 6.3, 6.6 (storage service) | SG-M3; SG-2 cache subset recorded floor | Isolated from HTTP cache; strict default durability; opaque bodies as handles |
| WA-9.4 | Storage Standard | Designed | SG 6.4, 6.5 (SG-D4, SG-D5) | SG-M2; SG-2 `storage/` ≥ 90%; SG-10 | Shed/shelf/bucket/bottle native; 60% pool; `estimate()` padding; whole-bucket eviction |
| WA-9.6 | Cookie Store API | Designed | NW 6.9, NW-D10 | NW-M2; NW-2 (Cookie Store event cases 100%) | Change events fire from service verdict, never the overlay |
| WA-9.7 | File API | Designed | DM 6.18 (`hg-webapi`); SG-D10 | SG-M3 (blob value-log backing) | Blob URL store a partitioned broker registry; blobs ≥ 256 KiB in storage service |
| WA-9.8 | Origin private file system | Designed | SG-D9, SG 6.7; DM 6.18 | SG-M3 (OPFS with sync handles) | Spot-check confirmed; sync handles map committed extents read-only; write-on-close commit |
| WA-10.1 | Service Workers | Designed | DM 6.10, 6.8 (DM-D8), `hg-agents`/`hg-workers`; NW 6.7 routing | DM-M3; DM-1 `service-workers/` 95%; DM-27 | Registration keying, broker job queue, 30 s / 5 min timeouts, process placement DM-D8 |
| WA-10.2 | Module service workers | Owned | DM 6.3 corpus; DM 7 anchors (Service Workers); `hg-workers` | DM-M3; DM-1 `service-workers/` 95% | Not named; Service Workers anchored and floored; spot-check Confirmed |
| WA-10.3 | Navigation preload | Owned | DM 6.10, 6.18 (allocated to NW partition service) | DM-M3; DM-1 `service-workers/` 95% (`navigation-preload/` tests) | Named and allocated only; NW 6.7 silent, DM 9.5 amendment not applied |
| WA-10.4 | Static routing API | Designed | DM 6.10; NW partition service | DM-M3; DM-27 "static routes never start the worker"; `service-workers/` 95% | `addRoutes`, ≤ 1,024 conditions, depth 10, evaluated without starting worker; URLPattern unowned (WA-8.16) |
| WA-12.1 | Permissions API | Owned | DM 6.18 (`hg-webapi`); S1 13.2 store; DV 6.10; SH 6.4 | S1 "detailed design is written before M6" | `query()` states designed only for device descriptors (DV 6.10); store design deferred |
| WA-12.4 | Async Clipboard | Designed | ED 6.10 (`hg-clipboard`, ED-D10, ED-D11); DM 6.18 | ED-M1/ED-M2; ED-1 `clipboard-apis/` 95%; ED-11 | Paste button by prompt renderer, no persistent grant, ClickFix command-text warning |
| WA-12.5 | Clipboard events and custom formats | Designed | ED 6.10 (ED-D12) | ED-M2; ED-1 `clipboard-apis/` 95% | `web ` formats capped 100/session, sanitizer utility; `clipboardchange` held (ED 9.1) |
| WA-13.1 | Same-origin policy and `document.domain` | Designed | DM 6.8 (DM-D6), `hg-agents` | DM-M2; DM-11; DM-1 with named exclusions | Always origin-keyed clusters; `document.domain` no-op; `Origin-Agent-Cluster: ?0` ignored |
| WA-13.3 | Trusted Types | Designed | DM 6.16, DM 7 (`hg-dom` sinks); JS 6.14 host hook; ED 9.5 | Trusted Types WPT (DM 6.16 "Checked by"); `trusted-types/` test input | "Enforced at every sink the bindings generate"; `insertHTML` and HostEnsureCanCompileStrings covered |
| WA-13.4 | Cross-origin isolation | Designed | NW 6.7 (CORP, COEP); DM 6.8 (COOP, isolation modes) | NW-M2 exit (CORP, COEP); NW-1; DM-M2 | Isolation modes none/logical/concrete; only concrete grants capability; credential stripping in partition service |
| WA-13.6 | Fetch Metadata | Owned | NW 6.7 (partition service) | NW-M2; NW-1 `fetch/` ≥ 95% (`fetch/metadata/`) | `Sec-Fetch-*` listed among partition-service steps; no Izcal-specific design |
| WA-13.7 | Referrer Policy | Owned | NW 6.7 (partition service) | NW-M1/M2; NW-1 `referrer-policy/` ≥ 95% | Listed among partition-service steps; anchored to pinned `fetch.bs` |
| WA-13.8 | Mixed content and HTTPS upgrades | Designed | NW 6.7 (HTTPS-First), 6.5.3 (NW-D25) | NW-M2; NW-1 `mixed-content/` | HTTPS-First interstitial, private hosts excluded, own HSTS preload list, partitioned dynamic HSTS |
| WA-13.9 | Secure contexts | Owned | DM 6.3 (`[SecureContext]` descriptors); DV 6.10 | DM-M1; DM-3 idlharness | `isSecureContext` not named; spot-check Confirmed via HTML claim |
| WA-13.10 | iframe sandbox and X-Frame-Options | Mentioned | DM 6.9, 6.16 (broker sandbox flags); XFO by HTML claim | DM-11 (sandbox); none for XFO | Sandbox Designed (broker-decided); XFO named nowhere, `x-frame-options/` under no floor |
| WA-13.11 | Clear-Site-Data | Designed | NW 6.8 (NW-D18); SG 6.9 clearing path | NW-M2; NW-19 cross-container case; SG-M3 | Clears within current container only; never touches another container |
| WA-13.12 | Local Network Access | Designed | NW 6.1, 6.7 (NW-D4 tickets), 6.11 | NW-M2; NW-16; NW-43 | Broker-checked tickets; applies to WebSockets and WebTransport; spot-check Confirmed |
| WA-13.14 | Web Cryptography | Designed | WC spec (`hg-webcrypto`, WC-D1, WC-D2) | WC milestones (WC-M2); NW-49/50/52 | Own spec; algorithm additions, constant-time and audit rules |
| WA-14.1 | Storage Access API | Designed | NW 6.8, 6.9 (NW-D16); DM 6.5 activation | NW-M2; NW-2; NW-19 | Prompt only, no heuristic grants; grants keyed (T, C, embedded site); headers not named |
| WA-14.3 | Storage and cookie partitioning | Designed | NW 6.8 (NW-D7), 6.9; S1 §6 | NW-M2 (partition table complete); NW-19, NW-20, NW-46 | Full partition-key table; third-party cookies keyed (T, C, X) by default |
| WA-14.4 | Bounce tracking mitigations | Designed | NW 6.9; CB 6.5 (`hg-nav-protect`, CB-D8) | NW-M4; CB-10 | 45-day activation rule, 1 h grace, 7-day backstop; grant exemption |
| WA-14.13 | Global Privacy Control | Designed | S1 §6 (GPC row); NW 6.7 (`Sec-GPC: 1`); SH settings | Stage 1 | Sent by default by network service; `navigator.globalPrivacyControl` not named |
| WA-14.14 | User-Agent reduction and string freezes | Designed | S1 13.2 (UA); NW 6.20 (NW-D12) | NW-M1; NW-23; UA-1 | Chrome-compatible UA plus own token; own brands; honest-UA breakage measured |
| WA-14.15 | Fingerprinting protections | Designed | S1 §6 (D-S1, OD-6); DM 6.16; SS 6.15 | Stage 1 baseline, Stage 2 full; PRIV-2 | One normalized environment-values record; noise as one output transform |
| WA-16.5 | Device Memory | Held | DM 6.16 lists `deviceMemory` only in fingerprinting row | — | Chromium-only; no DM-D15 exception; DM 6.3/9.1 rule holds it; spot-check Only partial |
| WA-16.13 | Web MIDI | Designed | DV 6.6 (DV-D13); DM 6.18 bindings | DV-M2 (experimental setting); DV-E5 at DV-M3 | All-ports consent, SysEx high-risk; general exposure held by DV-D9, DV-E5 proposed |
| WA-16.14 | Web Serial | Designed | DV 6.5; DM 6.18 | DV-M2; DV-E6 at DV-M3 | Port-identity check, reset warning; single default-on engine; DV-D9 holds general exposure |
| WA-16.15 | WebUSB | Designed | DV 6.3; DM 6.18 | DV-M2; DV-E1 at DV-M4 | Arbiter proxies transfers, protected classes refused; single-engine, DV-D9 admission record |
| WA-16.16 | WebHID | Designed | DV 6.4; DM 6.18 | DV-M2; DV-E2 at DV-M4 | Input collections refused; single-engine, experimental setting only (DV-D9) |
| WA-16.17 | Web Bluetooth | Designed | DV 6.7; DM 6.18 | DV-M2 (GATT only); DV-E3 at DV-M4 | GATT only, no `acceptAllDevices`, HID/FIDO refused; single-engine, DV-D9 |
| WA-16.18 | Web NFC | Designed | DV 6.8 (DV-D9); DM 6.18 | Built, not shipped; DV-E4 at DV-M5 | NDEF only, foreground, per-tap write consent; Android-only, not shipped |
| WA-17.1 | High Resolution Time | Designed | DM 6.7 (DM-D10), `hg-script` | DM-M1; DM-14 (WPT `hr-time/`) | One coarsening function, 100 µs / 5 µs, seeded per-realm jitter |
| WA-17.2 | Performance Timeline | Mentioned | DM 6.18 (`hg-webapi`) | — | Listed in script-facing table only; `performance-timeline/` under no floor or milestone |
| WA-17.3 | Navigation Timing 2 | Mentioned | DM 6.18 (`hg-webapi`, "Navigation") | — | One word in a list; no floor, milestone or design; `confidence` not named |
| WA-17.4 | Resource Timing 3 | Owned | DM 6.18 (`hg-webapi`); NW 6.8 table row | NW-19 (every 6.8 row) | "Coarsened per Fetch's clamp and coarsen"; cross-origin details opaque; no RT3 fields named |
| WA-17.5 | User Timing 3 | Mentioned | DM 6.18 (`hg-webapi`) | — | Named in a list only; `user-timing/` under no floor or milestone |
| WA-17.7 | Paint Timing | Mentioned | DM 6.7 (`hg-script` "mark paint timing"); DM 6.18; GR 6.x | — | Event-loop step owner only; GR "at specified coarsening"; no floor or milestone |
| WA-17.9 | Event Timing and INP | Owned | DM 6.18 (`hg-webapi`); DM 6.7 input-first policy | GR-44 (cross-site Event Timing channel ≤ 0.1 bit/s) | Only a channel target touches it; `interactionId`/`interactionCount` not named |
| WA-17.10 | Layout Instability | Designed | LY 6.15 (`hg-layout-query`, LY-D10 exception) | LY milestones; WPT `layout-instability/` (LY test inputs) | Shifts attributed to mutation-log causes; single-engine, owner exception recorded in LY-D10 |
| WA-17.11 | Long Tasks and Long Animation Frames | Owned | DM 6.7, 6.12 (DM-D15 exception), `hg-script` | DM-1 per-directory rule, `long-animation-frame/` test input | LoAF from task journal, cross-origin attribution masked; `longtask` (Chromium-only) unnamed, held |
| WA-17.15 | Reporting API | Designed | NW 6.8, 6.11; DM 6.18; FP 6.13 | NW-19 (Reporting row of 6.8) | Endpoints and queued reports keyed (T, C, F, X), same-key delivery; NEL refused |
| WA-18.1 | ES5 core and Annex B | Designed | JS 6.1–6.4, 6.13 (`hg-regex`); JS-5 | JS-M1/M2; JS-1 ≥99% | Engine object model designed; `hg-regex` parses "Annex B included"; JS-5 excludes unshipped Annex B |
| WA-18.2 | ES2015 syntax | Owned | JS (ES2026 claim) | JS-M2 "ES2026 plus Temporal"; JS-1 | ES2015 within ES2026; not named individually; test262 floor ≥99% |
| WA-18.3 | ES2015 library | Owned | JS (ES2026 claim); JS 6.5 ephemerons | JS-M2; JS-1 | In ES2026; WeakMap/WeakSet ephemerons in 6.5; rest not named |
| WA-18.5 | Modules | Designed | JS 6.14 (`hg-js-host`) | JS-M1 "ES modules" | Phase-aware module state machine; ECMA-262 graph, HTML fetching/import maps split |
| WA-18.6 | Async functions and async iteration | Owned | JS (ES2026 claim) | JS-M2; JS-1 | ES2017/ES2018 within ES2026; not named |
| WA-18.7 | ES2018–2021 additions | Owned | JS (ES2026 claim) | JS-M2; JS-1 | Within ES2026; not named |
| WA-18.8 | Weak references | Designed | JS 6.5 (`hg-heap`) | JS-M2; JS-50 (WeakRef order) | `[[KeptAlive]]`, canonical clearing, `HostEnqueueFinalizationRegistryCleanupJob` on seeded scheduler |
| WA-18.9 | ES2022 classes | Owned | JS (ES2026 claim) | JS-M2; JS-1 | Fields, private methods, static blocks in ES2022; not named |
| WA-18.10 | ES2022–2024 library | Owned | JS (ES2026 claim) | JS-M2; JS-1 | Within ES2026; not named |
| WA-18.11 | Shared memory | Designed | JS 6.12 | JS-M2; JS-1 | SAB in buffer region, one process per cluster; Atomics checked against Watt memory model |
| WA-18.12 | Resizable and transferable ArrayBuffers | Owned | JS (ES2026 claim); DM 6.11 clone | JS-M2; JS-1 | ES2024; DM clone lists "`ArrayBuffer` including resizable"; growable SAB in JS 6.12 |
| WA-18.13 | Iterator helpers | Owned | JS (ES2026 claim) | JS-M2; JS-1 | ES2025; reports never name it |
| WA-18.15 | Set methods | Owned | JS (ES2026 claim) | JS-M2; JS-1 | ES2025; reports never name it |
| WA-18.16 | ES2025 batch | Owned | JS (ES2026 claim); JS 6.14 | JS-M2; JS-1 | ES2025; import attributes, JSON modules in 6.14 module surface |
| WA-18.18 | Upsert | Mentioned | — (ES claim only) | — (JS-M2 only if in ES2026) | Not named; stage unrecorded; edition rests on secondary sources (spot-check partial) |
| WA-18.19 | `Math.sumPrecise` | Owned | JS (ES2026 claim) | JS-M2; JS-1 | Stage 4 July 2025, ES2026; reports never name it |
| WA-18.20 | Uint8Array base64/hex | Owned | JS (ES2026 claim) | JS-M2; JS-1 | ES2026 (spot-check confirmed); reports never name it |
| WA-18.22 | `Array.fromAsync` | Owned | JS (ES2026 claim) | JS-M2; JS-1 | In ES2026; reports never name it |
| WA-18.23 | JSON.parse source text access | Mentioned | — (ES claim only) | — (JS-M2 only if in ES2026) | Not named; stage unrecorded; edition rests on secondary sources (spot-check partial) |
| WA-19.1 | Intl core | Designed | JS 6.13 (`hg-intl`, `hg-cldr-build`) | JS-M3; JS-2 ≥95%/99% | Own CLDR/UTS 35 code; compiled shared data; locale set follows build (FP-D13) |
| WA-19.2 | `Intl.Segmenter` | Designed | JS 6.13; TX `hg-unicode` | JS-M3; JS-2 | "`Intl.Segmenter` … call `hg-unicode`"; one pinned Unicode version per build |
| WA-19.3 | `Intl.DurationFormat` | Owned | JS 6.13 (ECMA-402 claim) | JS-M3 "all of ECMA-402"; JS-2 | In ECMA-402 2025; not named |
| WA-19.5 | `Intl.PluralRules` notation | Owned | JS 6.13 (ECMA-402 claim) | JS-M3; JS-2 | Merged into ECMA-402 directly (ecma402#989, April 2025); not named |
| WA-19.6 | Temporal | Designed | JS 6.14, 6.13 (`hg-tz`) | JS-M2 "plus Temporal" | "Temporal is in, from its Stage 4 text"; 17th host hook; own `hg-tz` |
| WA-20.1 | Wasm MVP and JS API | Designed | JS 6.6 (`hg-wasm`) | JS-M2 (W0, JS API); JS-3 100% .wast | Single-pass validator, W0/W1/W2 tiers, guard-region bounds elision designed |
| WA-20.2 | Wasm Web API | Owned | JS 6.6; JS-3 | JS-M2; JS-3 `wasm/webapi/` C-1 floor | W1 "streaming compilation during download"; `wasm/webapi/` held to per-directory floor |
| WA-20.3 | Post-MVP core features | Owned | JS 6.6; JS-3 | JS-M2; JS-3 100% Wasm 3.0 .wast | Wasm 3.0 named and anchored; most post-MVP features not individually designed |
| WA-20.4 | Threads | Owned | JS 6.12; JS-3 | JS-M2; JS-3 "Wasm 3.0 plus threads" | Threads named in JS-3 floor; shared buffers designed for JS side |
| WA-20.5 | Exception handling | Designed | JS 6.6 | JS-M2; JS-3 | "`exnref` and JS exceptions share one unwinding mechanism" |
| WA-20.6 | GC and tail calls | Designed | JS 6.5, 6.6 | JS-M2; JS-3 | One GC for JS, Wasm GC, DOM; tail calls only via Wasm 3.0 |
| WA-20.7 | Memory64 and multi-memory | Designed | JS 6.6 | JS-M2; JS-3 | "memory64 uses explicit checks", validator checks; multi-memory only via Wasm 3.0 |
| WA-20.8 | Relaxed SIMD | Designed | JS 6.9, 6.18; JS-D12 | JS-M2; JS-3 | One relaxed-SIMD lowering per ISA; deterministic profile; RVV fixed 128-bit |
| WA-20.9 | JS String Builtins | Designed | JS 6.6 | JS-M2; JS-3 | "JS String Builtins operate on Izcal's strings directly" |
| WA-20.10 | JSPI | Designed | JS 6.6 (JS-D19); JS 9.1 | JS-M2 "JSPI" | Suspension is a stack-segment switch; ships with W0 and W1 |
| WA-20.11 | Resizable memory buffers | Mentioned | — | — | Not named; only reachable via `wasm/jsapi/` floor if merged at pin |
| WA-22.1 | WebDriver (classic) | Designed | DT 6.3, 6.5 (`hg-bidi`, `izcaldriver`) | S1 M2 slice; M7; DT-5 `webdriver/tests/classic/` | Loopback HTTP server after one-time enablement; `navigator.webdriver` true |
| WA-22.2 | WebDriver BiDi | Designed | DT 6.3 (`hg-bidi` over IIP); NW 6.18 | M7; DT-5 `webdriver/tests/bidi/` | All BiDi modules; user contexts map to containers; a11y `locateNodes` |
| WA-22.4 | Chrome DevTools Protocol | Designed | DT 6.3 (`hg-cdp`), DT-D12 | M7; DT-6; JS-63 | Izcal-authored description; six-domain published profile; defined "not in profile" error |
| WA-22.5 | Browser MCP servers | Designed | DT 6.3 (`hg-agent-mcp`, `izcal-mcp`) | Stage 2 (with B9); DT-13, DT-14 | Scoped grant tokens, compact results ≤2,000 tokens; absent with AI off. Stage 2 |
| WA-22.6 | Web IDL | Designed | DM 6.3 (`hg-webidl`, DM-D2, DM-D21) | DM-M0; DM-1 `webidl/` ≥97% | Own WebIDL front end; overload resolution compiled to type dispatch |
| WA-22.7 | Infra | Not there | — | — | Spot-check fixed: DM-D21 is IDL inputs; Infra named in no report |
| WA-22.8 | Console | Owned | JS 6.10 (`hg-js-observe`), JS-63; DT 9.1 | JS-M1; DT M6 | "evaluation, console and exceptions" at JS-M1; Console Standard API not named |
| WA-22.11 | WebExtensions | Designed | EX 6.0–6.19 (EX-D1–D16) | Stage 2; EX-M0 seams in Stage 1 | Three-ring compat surface, MV3 only, capability permissions. Stage 2 |
