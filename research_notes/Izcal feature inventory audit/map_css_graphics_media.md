# Izcal feature inventory audit: mapping of CSS, graphics and media

Compiled 24 October 2026 against the reports on main at 217e02d.

Input: `css_graphics_media.md` (277 rows, CG-1.1 to CG-16.11). Each row was searched in `reports/*.md` with several synonyms (a case-insensitive regex sweep across all 33 reports), and the matching section was read before a status was given.

**Statuses.** **Covered**: a spec specifies the behaviour, or explicitly scopes in the module (an anchor table naming the module as pinned input plus its WPT oracle, a WPT floor, or a milestone exit). **Partial**: mentioned, or covered only in part (for example parsed by the style generator's "every property in webref" rule, with no layout or paint design); the cell says what is missing. **Excluded**: refused by a recorded decision. **Deferred (admission)**: held by an admission rule (ST-D18, LY-D10, AN-D9, SV-D13) until it meets that rule. **Deferred (Stage 2/3)**: assigned to a later stage. **Missing**: no report mentions it after at least three distinct searches, which are listed.

**Scoping statements used** (so individual rows can cite them briefly):
- **ST-scope**: style spec 4.2 and 7 anchor table: "Syntax 3, Selectors 4 and CSS Shadow 1, Cascade 5 and 6, Nesting 1, Values 4 and 5 …, Properties and Values API, Conditional 5 (media, supports, container queries), CSSOM 1"; ST-M1 "parsers for every property in webref". The webref rule covers **parsing and computed values only**, not layout or paint.
- **LY-scope**: layout 7 anchor table (Display 3/4, HTML Rendering §15, Lists 3, Ruby 1, Pseudo 4, CSS 2.1 §8–10, Box 3/4, Shapes 1, Inline 3, Text 3/4, Flexbox, Grid 1–3, Align 3, Tables 3, Multicol 1/2, Fragmentation 3/4, Paged Media 3, Overflow 3/4, Position 3, Anchor Positioning 1, Sizing 3/4, Contain 2, Scroll Anchoring 1, Scroll Snap 1/2, CSSOM View, observers, MathML Core) and the LY-1 per-directory WPT floors.
- **GR-scope**: graphics 4.2 and 7 anchor table (Appendix E, Compositing 1, Filter Effects 1/2, Masking 1, Backgrounds 3, Borders 4, Images 3/4, Color 4/5, Transforms 1/2, Overflow 3, Scroll Snap, Overscroll Behavior, Web Animations 1, Scroll-driven Animations 1, View Transitions 1/2, SVG 2 painting, HTML canvas, WebGPU/WGSL, WebGL 1/2).
- **TX-scope**: fonts 4.2 (CSS Fonts 4 and 5, Font Loading 3, Text 3/4, Writing Modes 3/4, Inline 3, canvas text, WOFF 1/2, IFT, MathML Core §5) and the 6.18 coverage table.
- **MD-scope**: media 6.20 coverage table and 7 anchor table.
- **Admission rules**: ST-D18 ("a WG resolution, WPT tests, and either ships in two engines or is an Interop focus area"), extended by LY-D10 (layout), AN-D9 (animation) and SV-D13 (SVG). Where a single-engine or no-engine feature is not named by any spec, the row says "rule applies, not named".

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| **1. CSS syntax, cascade, selectors** | | | | | |
| CG-1.1 | CSS 2.1 core | Covered | LY 6.5.1, 6.5.7 (anchors "CSS 2.1 §8–10"); GR 6.3 (Appendix E paint order) | LY-M1, GR-M1 | LY-1 floor `CSS2` 97%†; tables LY 6.5.5 |
| CG-1.2 | CSS Snapshot 2026 | Missing | — | — | Searched "Snapshot 202", "CSS Snapshot", "css-2026", "must ship". No spec uses the Snapshot as a floor; module coverage is assembled from four anchor tables (ST, LY, GR, TX), which leaves orphans (CG-1.9, 4.23, 6.20, 6.3, 9.1, 2.19). ST-D18 is the only completeness policy and it governs drafts, not the stable set |
| CG-1.3 | CSS Syntax 3 | Covered | ST 6.6 ("hg-css-syntax implements CSS Syntax 3 by hand, in spec order") | ST-M1 | ST-3 `css-syntax` 97%† |
| CG-1.4 | Cascade and inheritance 3/4 | Covered | ST 6.9 (cascade keys, eight origin levels) | ST-M1 | ST-3 `css-cascade` 97%† |
| CG-1.5 | Cascade layers | Covered | ST 6.9 (layer rank, `revert-layer`); 6.6a (layered `@import` keeps its position) | ST-M1 | "cascade with layers, nesting and `@scope`" (ST-M1) |
| CG-1.6 | `@scope` | Covered | ST 6.9 (scope proximity in the cascade key) | ST-M1 | ST-M1 exit names `@scope` |
| CG-1.7 | `revert-rule` | Covered | ST 6.9 ("`revert`, `revert-layer` and `revert-rule` roll back to lower entries") | ST-M1 | Single-engine in the inventory, yet named in the design; ST-D18 not applied to it |
| CG-1.8 | CSS Nesting | Covered | ST 6.6, 6.7 (back-track, `&` as `:is()`) | ST-M1 | ST-3 `css-nesting` 97%† |
| CG-1.9 | CSS Namespaces (`@namespace`) | Missing | — | — | Searched "@namespace", "CSS Namespaces", "css-namespaces", "namespace" in the style spec (hits are XML Namespaces for `hg-xml-syntax` only). A REC in all three engines; `ns|E` selectors in Selectors 4 need it. Probably an oversight in ST-scope |
| CG-1.10 | Style attributes | Covered | ST 6.11 (inline-style block, presentational hints); 6.6 entry points | ST-M1 | — |
| CG-1.11 | Selectors 3 | Covered | ST 6.7 | ST-M0 (reference), ST-M1 | `hg-style-reference` matches Selectors 3 at ST-M0 |
| CG-1.12 | Selectors 4 | Covered | ST 6.7, 6.7.2 (`:has()`, ST-D11), 6.7.1 state sources | ST-M1, `:has()` ST-M2 | ST-3 `selectors` 95%†; `||` and time pseudo-classes not mentioned (not interoperable anyway) |
| CG-1.13 | UI state pseudo-classes (`:open`, `:popover-open`, `:modal`, `:fullscreen`, `:autofill`, `:state()`) | Covered | ST 6.7.1 (named state sources incl. fullscreen and top layer, `:autofill` from the shell, `:state()`); DOM 6.12 (`:open`, popover, dialog) | ST-M1, DM-M2 | — |
| CG-1.14 | Media pseudo-classes (`:playing`, `:paused`, …) | Covered | ST 6.7.1 ("Sources outside script (media playback, …) post a task") | ST-M1 | Covered by the state-source design; no pseudo-class named, and the state-source table is an ST-M0 deliverable. Interop 2026 area, so LY-2-style scoring would apply only if a style target named it (ST has no Interop target) |
| CG-1.15 | Selectors 5 | Excluded | ST-D18 (rule applies, not named) | — | Searched "Selectors 5", "selectors-5", ":local-link", "reference combinator": no hits. No engine ships it |
| CG-1.16 | Pseudo-elements 4 | Covered | LY 6.3 / anchor "Pseudo 4" (box generation, `::first-line`, `::first-letter`, `::marker`, `::details-content`); ED 6.10 (highlight pseudo-elements in CSS Pseudo 4's order, `::target-text`, spelling and grammar) | LY-M1, ED-M2 | `::placeholder` and `::file-selector-button` not named but fall under form-control UA shadow trees (LY-D11) |
| CG-1.17 | Shadow DOM styling (`:host`, `::slotted`, `::part`) | Covered | ST-scope (CSS Shadow 1); ST 6.9 (shadow context in cascade key), 6.7 (per-scope rule index) | ST-M1 | ST-3 `css-shadow` 87%†; `:host-context()` held by ST-D18 (ST 9.1) |
| CG-1.18 | Custom properties | Covered | ST 6.9 (ST-D14, by-reference sets, 65,536† token limit) | ST-M1 | ST-3 `css-variables` 87%† |
| CG-1.19 | Registered custom properties | Covered | ST-scope (Properties and Values API); ST 6.9 | ST-M2 | ST-3 `css-properties-values-api` 95%† |
| CG-1.20 | `@function`, `@mixin` | Deferred (admission) | ST 9.1, ST-D18 ("`if()`, `@function`, `@mixin` … wait for ST-D18's admission rule"); interpreter built general enough (ST 6.9) | When admitted | Designed but held |
| CG-1.21 | CSS Speech | Excluded | ST-D18 (rule applies); AX 6.3 ("`speak-as` if admitted") | — | No engine ships it; `speak` is read by the semantic plane (ST 6.16) |
| **2. CSS values, units, conditionals** | | | | | |
| CG-2.1 | Values and Units 3 | Covered | ST-scope (Values 4 and 5 supersede 3); ST 6.9 | ST-M1 | `toggle()` not mentioned (never shipped) |
| CG-2.2 | Values and Units 4 | Covered | ST 6.9 (`calc()` typing, 32 terms, `hg-math` trig; font-relative units via TX metrics query) | ST-M1 | ST-3 `css-values` 87%†; viewport units come from the environment snapshot (ST 6.10) |
| CG-2.3 | Typed `attr()` | Covered | ST 6.9 (substitution interpreter, `attr()` taint as a type) | ST-M2 | ST-M2 exit names "typed `attr()`" |
| CG-2.4 | `if()` | Deferred (admission) | ST 9.1, ST-D18 | When admitted | Interpreter designed for it (ST 6.9) |
| CG-2.5 | `sibling-index()`, `sibling-count()` | Covered | ST 6.9 ("the tree-position functions"; memo keyed on `sibling-index()` reads) | ST-M2 | Not in ST 9.1's hold list although only Chromium and partial WebKit ship it; ST-D18 status unstated |
| CG-2.6 | `random()` | Excluded | ST-D18 (rule applies, not named) | — | Searched "random()", "randomness", "css-values-5 random": no hits |
| CG-2.7 | `progress()`, `*-mix()`, `calc-size()` | Deferred (admission) | LY 6.15, LY-D10 and AN-D9 ("`interpolate-size`, `calc-size()` … Held") | When admitted | `progress()` and the generic `*-mix()` functions are not named (rule applies); `color-mix()` is Covered under CG-3.3 |
| CG-2.8 | `env()` variables | Partial | ST 6.10 (`env()` from the environment snapshot); AX 6.9 (`env(preferred-text-scale)` bucketed) | ST-M1 | The mechanism is designed; which variables exist is not: `safe-area-inset-*`, `titlebar-area-*`, `keyboard-inset-*` (VirtualKeyboard held, ED-D18) and `viewport-segment-*` are unnamed |
| CG-2.9 | Linked parameters | Deferred (admission) | SV 9.1, SV-D13 ("link parameters" held) | When admitted | — |
| CG-2.10 | Conditional 3 | Covered | ST-scope (Conditional 5 superset); ST 6.10 | ST-M1 | ST-3 `css-conditional` 95%† |
| CG-2.11 | Conditional 4 (`@supports selector()`) | Covered | ST-scope (Conditional 5) | ST-M1 | — |
| CG-2.12 | Conditional 5 (`font-tech()`, `@when`) | Covered | ST-scope ("Conditional 5 (media, supports, container queries)") | ST-M1 | `@when`/`@else` unshipped: ST-D18 applies, not named |
| CG-2.13 | Container size queries | Covered | ST 6.10, LY 6.9 (LY-D12 re-entry protocol) | ST-M2, LY-M2 | LY-6 target |
| CG-2.14 | Container style queries | Covered | ST 6.10 ("Style queries need no layout") | ST-M2 | — |
| CG-2.15 | Scroll-state queries | Deferred (admission) | ST 9.1, ST-D18; LY 9.1, LY-D10; snapshot point designed (LY 6.9) | When admitted | — |
| CG-2.16 | Media Queries 3/4 | Covered | ST 6.10 (one environment snapshot per document per frame, D-S1 quantization) | ST-M1 | `color-gamut`/`dynamic-range` buckets: MD 6.5, GR 6.20 |
| CG-2.17 | Media Queries 5 | Partial | ST 6.10 (`prefers-*`, `forced-colors`); AX 6.9 (`prefers-reduced-transparency`, `inverted-colors`, contrast buckets) | ST-M1, AX-M2 | Not mentioned: `display-mode`, `prefers-reduced-data`, the `scripting` feature, `video-dynamic-range`, `@custom-media` (unshipped) |
| CG-2.18 | `@viewport` and the viewport meta | Partial | ST 6.4 (speculative parser applies `<meta name=viewport>`) | — | `@viewport` is deprecated (fine to omit). The meta viewport's effect on the layout viewport (`width=device-width`, `initial-scale`) is not designed; matters for Android (Stage 2) |
| CG-2.19 | CSS `zoom` property | Partial | LY 6.2 (zoom factors converted once at the boundary) | — | Only a numeric input; the property's semantics (effective zoom applied to used lengths, CSSOM interaction) are not designed and `css-zoom` has no WPT floor. Interop 2025–2026 focus area |
| CG-2.20 | Easing functions | Covered | AN 6.2 (AN-D2 exact easing, integer cubic Bézier, `linear()`) | AN-M1 | AN-1 measures `css-easing` |
| **3. Colour, images, backgrounds, borders** | | | | | |
| CG-3.1 | Colour 3 | Covered | ST 6.9 (f64 computed colours, MD-D28 hand-off); GR-scope | ST-M1, GR-M1 | — |
| CG-3.2 | Colour 4 | Covered | ST 6.9 (`hg-math`, gamut mapping chosen jointly); MD 6.5; GR 6.4 (Oklab interpolation) | ST-M1, GR-M1 | Q: which of CSS Color 4's three gamut-mapping algorithms is still open (ST 6.9) |
| CG-3.3 | Colour 5 | Covered | ST 6.9 (`color-mix()`, relative colours; `@color-profile` bytes to the media ICC parser) | ST-M1 | — |
| CG-3.4 | `contrast-color()` | Covered | GR-scope and MD 6.20 ("CSS Color 4 and 5") | ST-M1 | Covered by scoping only; not named anywhere. Interop 2026 focus area |
| CG-3.5 | Colour HDR | Covered | MD 6.5, MD-D9; ST 6.9 (`dynamic-range-limit` carried to the compositor) | MD-M2, GR-M2 | — |
| CG-3.6 | Colour adjustment | Covered | AX 4.2 (CSS Color Adjust 1); GR-D25 (forced-colours backplates); ST 6.6 (UA variants) | AX-M2, GR-M2 | `print-color-adjust` and `color-scheme`/`light-dark()` not named |
| CG-3.7 | Backgrounds and Borders 3 | Covered | GR 6.4, GR-D17 (corner joins, dash spacing) | GR-M1 | GR-M1 "Paint for CSS 2 and Backgrounds and Borders" |
| CG-3.8 | Backgrounds 4 (`background-clip: text`, `border-area`) | Partial | ST-M1 webref parsing only | — | GR-scope lists Backgrounds **3**; text clipping and `border-area` paint are not designed |
| CG-3.9 | Borders 4 (`corner-shape`) | Covered | GR-scope ("Borders 4"); GR-D17 (outline follows `corner-shape`); LY 6.14 (hit test of `corner-shape` regions) | GR-M1 | Tension: `corner-shape` is single-engine and ST-D18/LY-D10 would hold it, but GR scopes it in |
| CG-3.10 | Gap decorations | Deferred (admission) | LY 9.1, LY-D10 | When admitted | — |
| CG-3.11 | Images 3 | Covered | LY 6.5.8 (`object-fit`, `object-position`); MD 6.6 (`image-orientation`); GR 6.4 (`image-rendering`) | LY-M1, MD-M1 | — |
| CG-3.12 | Images 4 | Covered | GR-scope (Images 3 and 4: gradients); SV 6.8 (`image-set()`) | GR-M1 | `cross-fade()` and `element()` not named (prefixed or single-engine) |
| CG-3.13 | `object-view-box` | Excluded | ST-D18/LY-D10 (rule applies, not named) | — | Searched "object-view-box", "Images 5", "view box crop": no hits |
| CG-3.14 | `image-animation` | Deferred (admission) | MD 6.6, MD-D19 ("Izcal follows CSSWG issue 1615 so authors can opt specific images in when it is specified") | When specified | Reduced-motion first-frame behaviour is designed |
| CG-3.15 | Fill and Stroke 3 | Excluded | ST-D18 (rule applies, not named) | — | Searched "fill-stroke", "Fill and Stroke", "stroke on boxes": no hits; no engine ships it |
| CG-3.16 | Legacy `-webkit-` features, Compat Standard | Partial | TX 6.18 (`-webkit-text-stroke` owned by paint and SVG) | — | The WHATWG Compat Standard is cited nowhere; `-webkit-box-reflect`, `-webkit-text-fill-color`, `-webkit-box` and the prefixed alias list have no owner. `-webkit-line-clamp` maps to LY 6.7's `line-clamp` |
| **4. Box model, display, layout modes** | | | | | |
| CG-4.1 | Box model 3 | Covered | LY 6.5.1 (anchors Box 3 and 4) | LY-M1 | LY-1 `css-box` 97%† |
| CG-4.2 | `margin-trim` | Deferred (admission) | LY 6.5.1, LY-D10 | When admitted | — |
| CG-4.3 | Sizing 3 | Covered | LY 6.6 | LY-M1/M2 | LY-1 `css-sizing` 97%† |
| CG-4.4 | Sizing 4 | Covered | LY 6.6 (`aspect-ratio`, `contain-intrinsic-size`, `stretch`) | LY-M2 | `interpolate-size` held (LY-D10) |
| CG-4.5 | Display 3 | Covered | LY 6.3 | LY-M1 | LY-1 `css-display` 97%† |
| CG-4.6 | Display 4 (`reading-flow`) | Deferred (admission) | LY 9.1, LY-D10; AX 6.3 uses it "when the property is admitted" | When admitted | `display` animation is admitted (AN-D9) |
| CG-4.7 | Flexbox | Covered | LY 6.5.3 | LY-M2 | LY-1 97%† |
| CG-4.8 | Grid 1 | Covered | LY 6.5.4 | LY-M2 | LY-1 97%† |
| CG-4.9 | Subgrid | Covered | LY 6.5.4 | LY-M2 | — |
| CG-4.10 | Grid Lanes (masonry) | Deferred (admission) | LY 6.5.4, LY-D10 ("not admitted"; sizing written lanes-ready) | When admitted | — |
| CG-4.11 | Box Alignment 3 | Covered | LY anchor table | LY-M2 | LY-1 `css-align` 97%† |
| CG-4.12 | Multicol 1 | Covered | LY 6.5.6 (bounded balancing) | LY-M2 | LY-24 |
| CG-4.13 | Multicol 2 (`column-wrap`, `::column`) | Deferred (admission) | LY 6.5.6, LY-D10 | When admitted | — |
| CG-4.14 | Tables 3 | Covered | LY 6.5.5, LY-D17 | LY-M2 | — |
| CG-4.15 | Inline 3 | Covered | LY 6.5.2 (`text-box-trim`, `initial-letter`, `vertical-align`); TX 6.8 | LY-M1 | — |
| CG-4.16 | Positioned layout 3 | Covered | LY 6.5.7, 6.10 (sticky) | LY-M1 | — |
| CG-4.17 | Top layer, `::backdrop`, `overlay` | Covered | LY 6.5.7; DOM 6.12; AN-D9 (`overlay` transitions) | LY-M1, DM-M2 | — |
| CG-4.18 | Anchor positioning 1 | Covered | LY 6.9 rule 5 (`@position-try`, last successful option) | LY-M2 | LY-1 `css-anchor-position` 92%† |
| CG-4.19 | Anchor positioning 2 | Deferred (admission) | LY-D10 ("anchored container queries") | When admitted | — |
| CG-4.20 | Overflow 3 | Covered | LY 6.10 | LY-M1 | — |
| CG-4.21 | Overflow 4 (`line-clamp`, `continue`) | Covered | LY 6.7 ("`line-clamp` and `continue: discard` turn ordinary blocks into fragmentation containers") | LY-M2 | `block-ellipsis`, `max-lines` not named |
| CG-4.22 | Overflow 5 carousels (`::scroll-marker`, `::scroll-button()`) | Excluded | LY-D10 (rule applies, not named) | — | Searched "scroll-marker", "scroll-button", "Overflow 5", "carousel": only "`::column` carousels" in LY 2.2 |
| CG-4.23 | Logical properties | Partial | ST-M1 webref parsing; LY 6.8 (algorithms in logical coordinates) | ST-M1 | No spec designs the logical-to-physical property-group resolution in the cascade, and `css-logical` has no WPT floor in ST-3 or LY-1. Only the shell "uses logical properties" |
| CG-4.24 | Writing modes 3/4 | Covered | LY 6.8; TX 6.6 (UAX 50) | LY-M2 | LY-1 `css-writing-modes` 92%† |
| CG-4.25 | Containment, `content-visibility` | Covered | LY 6.9 | LY-M2 | LY-16 (semantic tree unchanged) |
| CG-4.26 | `will-change` | Covered | GR 6.3 (300† BeginFrame ticks, never forces a layer) | GR-M1 | — |
| CG-4.27 | Shapes 1 | Covered | LY 6.5.1 (float area with shapes) | LY-M1 | LY-1 `css-shapes` 92%† |
| CG-4.28 | `shape()` function | Missing | — | — | Searched "shape()", "Shapes 2", "css-shapes-2", "responsive path shape": no hits. Interop 2026 focus area, so LY-2 ("≥ best shipping engine on every layout-related Interop area") requires it with no design. Webref parsing would accept its grammar |
| CG-4.29 | Exclusions | Excluded | LY-D10 (rule applies, not named) | — | Searched "Exclusions", "wrap-flow", "css-exclusions": no relevant hits |
| CG-4.30 | Regions | Excluded | LY-D10 (rule applies, not named) | — | Searched "CSS Regions", "flow-into", "flow-from": no relevant hits |
| CG-4.31 | Page floats, line grid, rhythm, template, round display | Excluded | LY-D9 (page floats explicitly not admitted); others LY-D10 (rule applies, not named) | — | — |
| CG-4.32 | CSS Layout API | Excluded | LY-D10 (rule applies, not named) | — | DOM 6.9's agent table lists a "layout" worklet agent kind, which no spec defines: a small inconsistency |
| CG-4.33 | Quirks mode | Covered | ST 6.6 (quirks UA sheet); LY 6.3 ("Quirks mode is a recorded document mode input of every algorithm") | ST-M1, LY-M1 | — |
| **5. Scrolling, fragmentation, paged media** | | | | | |
| CG-5.1 | Scroll snap 1 | Covered | LY 6.10; GR 6.8 (compositor snaps during a gesture) | LY-M2 | LY-1 `css-scroll-snap` 92%† |
| CG-5.2 | Scroll snap 2 events | Deferred (admission) | LY 6.10 ("`scrollsnapchange` and `scrollsnapchanging` are admitted per LY-D10") | When admitted | — |
| CG-5.3 | Scroll anchoring | Covered | LY 6.10 | LY-M2 | — |
| CG-5.4 | Overscroll behaviour | Partial | GR 7 anchor table ("Overscroll Behavior") | GR-M1 | Named as a pinned input only; scroll chaining, bounce and pull-to-refresh suppression are not designed in GR 6.8 |
| CG-5.5 | Scrollbar styling | Partial | LY-D11 (`scrollbar-width` thin 8†, none 0); GR 6.28 (scrollbars painted as layers) | LY-M1 | `scrollbar-color` and `::-webkit-scrollbar` not named |
| CG-5.6 | Smooth scrolling, scroll APIs, visual viewport | Covered | LY 6.15 (CSSOM View scrolling), 6.10; DOM 6.9 (`scroll`, `scrollend`); GR 6.28 (visual viewport, pinch) | LY-M2 | `scroll-behavior: smooth`'s animation curve is not specified |
| CG-5.7 | Spatial navigation | Missing | — | — | Searched "spatial navigation", "spatnav", "css-nav", "directional focus": no hits |
| CG-5.8 | Fragmentation 3 | Covered | LY 6.7 (LY-D8, break scoring) | LY-M2 | — |
| CG-5.9 | Fragmentation 4 | Deferred (admission) | LY 6.7 ("`margin-break` … admitted by LY-D10's rule") | When admitted | — |
| CG-5.10 | Paged Media 3 | Covered | LY 6.7 (16 margin boxes, named pages, `size`) | LY-M2 (Stage 1) | LY-1 `css-page` 90%† |
| CG-5.11 | GCPM | Excluded | LY-D9 (not admitted; reserved-area slots from LY-M0) | — | Reversal: Interop area or second engine |
| **6. Text, fonts, lists, generated content** | | | | | |
| CG-6.1 | Text 3 | Covered | TX 6.6 (UAX 14 tailorings, case mapping); LY 6.5.2 | TX-M1, LY-M1 | `hanging-punctuation` owned by layout (TX 6.18) |
| CG-6.2 | Text 4 | Covered | LY 6.5.2 (LY-D16 `balance`, `pretty`); TX 6.18 (`text-spacing-trim`, `text-autospace`), TX 6.6 (`auto-phrase`) | LY-M3, TX-M2 | `hyphenate-character`, `word-space-transform` not named |
| CG-6.3 | Text decoration 3 | Partial | TX 6.18 (owner "Layout and paint"; TX supplies underline metrics, emphasis glyphs) | — | Neither layout nor graphics designs decoration painting, `text-emphasis` placement or `text-shadow`; GR 6.3 mentions decoration only as ink overflow; no `css-text-decor` WPT floor |
| CG-6.4 | Text decoration 4 | Partial | TX 6.18 (glyph outlines for skip-ink) | — | As CG-6.3; skip-ink intersection and thickness or offset painting undesigned |
| CG-6.5 | `text-size-adjust` | Missing | — | — | Searched "text-size-adjust", "text autosizing", "font inflation": no hits. Mobile-relevant (Android Stage 2) |
| CG-6.6 | Fonts 3 | Covered | TX 6.4 (CSS Fonts 4 §5 matching), 6.5 | TX-M1 | — |
| CG-6.7 | Fonts 4 | Covered | TX 6.4 (palettes, synthesis, `size-adjust`, generics), 6.5 (`font-display`) | TX-M1/M2 | — |
| CG-6.8 | Fonts 5 | Covered | TX-scope (CSS Fonts 5 metric overrides, `font-size-adjust`) | TX-M2 | — |
| CG-6.9 | Font Loading API | Covered | TX 6.5 | TX-M2 | — |
| CG-6.10 | WOFF 1/2, OpenType | Covered | TX 6.2 | TX-M1 | — |
| CG-6.11 | Variable fonts | Covered | TX 6.2 (avar 1 and 2, CFF2) | TX-M2; avar2 TX-M4 | — |
| CG-6.12 | Colour font formats | Covered | TX 6.2, 6.3 (COLRv0/v1, sbix, CBDT, OT-SVG); GR 6.6 | TX-M2 | — |
| CG-6.13 | Shaping, AAT, Graphite | Covered | TX 6.7 (Izcal shaping specification); AAT `morx`/`kerx` (TX 6.3) | TX-M1/M2 | Graphite excluded by TX-D6/TX-D7 ("Graphite, LCD AA and Local Font Access are not built") |
| CG-6.14 | Unicode algorithms | Covered | TX 6.6 (UAX 9, 14, 29, 50, 15, UTS 51) | TX-M0/M1 | TX-1 at 100% of the Unicode tests |
| CG-6.15 | Hyphenation dictionaries | Covered | TX 6.6, TX-D14 ("The patterns are Izcal's own … No third-party pattern set (hyph-utf8 or any other) is used") | TX-M2 | Per-language gate: lexicon reproduced exactly plus native review |
| CG-6.16 | Incremental Font Transfer | Covered | TX 6.2, 9.1 (ships when its WPT suite and privacy review pass) | TX-M4 if gated | — |
| CG-6.17 | Local Font Access | Excluded | TX-D17 ("not implemented; per-site local-font grants from site settings") | — | — |
| CG-6.18 | Ruby layout | Covered | LY 6.5.2, anchor "Ruby 1" | LY-M2 | LY-1 `css-ruby` 92%† |
| CG-6.19 | Lists and counters 3 | Covered | LY anchor "CSS Lists 3"; box-tree track ("lists and counters' layout side") | LY-M1 | `marker-side` not named |
| CG-6.20 | Counter Styles 3 | Partial | Implied by Lists 3 (LY) and webref parsing | — | Searched "@counter-style", "counter-style", "symbols()", "predefined counter styles": no hits. The `@counter-style` algorithms and the predefined-styles data (W3C NOTE, a licence question) have no owner |
| CG-6.21 | Generated content 3 | Partial | LY 6.3 (`content` in the box-tree "affects" category; pseudo-element boxes); AX 6.3 (CSS Content 3 alt text) | LY-M1 | CSS Content 3 is not in LY-scope; `quotes`, content images and counters in `content` are not designed |
| CG-6.22 | Custom Highlight API | Covered | ED 6.10 (overlay order, highlight inheritance) | ED-M2 | ED oracle `css/css-highlight-api/` |
| **7. Transforms, animation, effects** | | | | | |
| CG-7.1 | Transforms 1 | Covered | GR 6.3; AN 6.2 | GR-M1 | — |
| CG-7.2 | Transforms 2 (3D) | Covered | GR 6.3 (Newell's algorithm in integers, 4,096† splits) | GR-M2 | — |
| CG-7.3 | Transitions 1 | Covered | AN 6.4; ST 6.9 (post-pass transition creation) | AN-M1 | — |
| CG-7.4 | Transitions 2 | Covered | AN-D9 (admitted), ST 6.10 (`@starting-style`) | AN-M1 | — |
| CG-7.5 | Animations 1 | Covered | AN 6.4 | AN-M1 | — |
| CG-7.6 | Animations 2 | Covered | AN-D9 (`animation-composition`, `animation-timeline` admitted) | AN-M1 | `animation-trigger` held (AN-D9) |
| CG-7.7 | Web Animations 1 | Covered | AN 6.1 | AN-M1 | — |
| CG-7.8 | Web Animations 2 | Deferred (admission) | AN-D9 ("`GroupEffect`, `SequenceEffect` … Held"; iteration composite internal for SMIL) | When admitted | — |
| CG-7.9 | Scroll-driven animations | Covered | AN 6.6 (AN-D8); LY 6.10; GR 6.8 | AN-M2 | — |
| CG-7.10 | Animation Worklet | Excluded | AN-D9 ("Not implemented") | — | — |
| CG-7.11 | Motion path | Covered | AN 6.2 (`offset-*` sampling through SVG path functions) | AN-M1/M2 | Off-main-thread motion path held until AN-S2 |
| CG-7.12 | View Transitions 1 | Covered | AN 6.7 (AN-D7); GR 6.8 | AN-M2 | — |
| CG-7.13 | View Transitions 2 | Covered | AN 6.7 (cross-document admitted; broker decides eligibility) | AN-M3 | Element-scoped transitions and nested groups held (AN-D9) |
| CG-7.14 | Filter Effects 1 | Covered | GR 6.4 (integer kernels), GR-D18 (cross-origin rule) | GR-M2 | — |
| CG-7.15 | `backdrop-filter` | Covered | GR-scope (Filter Effects 2); GR 6.3, 6.7 | GR-M2 | — |
| CG-7.16 | Compositing and blending | Covered | GR 6.4 (all 16 blend modes) | GR-M2 | — |
| CG-7.17 | Masking | Covered | GR 6.4; SV 6.5 (one resolver for `url()` references) | GR-M2 | `mask-border` not named |
| CG-7.18 | Geometry Interfaces | Partial | SV 4.2 (pinned input), SV 6.6 (`DOMRect`, `DOMMatrix` as double) | — | Named as an input for SVG geometry; no spec owns `DOMMatrix`/`DOMPoint`/`DOMQuad` as interfaces (string parsing of transforms, use by canvas and CSSOM) |
| **8. Object models, Houdini, observers** | | | | | |
| CG-8.1 | CSSOM | Covered | ST-scope (CSSOM 1), ST 6.6 (copy-on-write sheets, constructed sheets) | ST-M2 | ST-3 `cssom` 97%† |
| CG-8.2 | CSSOM View | Covered | LY 6.15, 6.1 (forcing table) | LY-M1/M2 | LY-1 `cssom-view` 95%† |
| CG-8.3 | CSS Typed OM | Partial | ST 9.3 ("Typed OM if admitted"); Q-ST-14 open | — | An open question, not a decision |
| CG-8.4 | CSS Painting API | Excluded | ST-D18 (rule applies, not named) | — | Searched "Paint API", "registerPaint", "paint worklet": DOM 6.9 lists a "paint" worklet agent kind, but no spec defines `paint()` images |
| CG-8.5 | Worklets | Covered | DOM 6.9, 6.10 (worklet agents and global scopes) | DM-M3 | — |
| CG-8.6 | CSS parser API | Excluded | ST-D18 (rule applies, not named) | — | Unofficial; no engine |
| CG-8.7 | Resize Observer | Covered | LY 6.15 | LY-M2 | LY-1 95%† |
| CG-8.8 | Intersection Observer (v1, v2) | Covered | LY 6.15; LY-D10 owner exception for v2 | LY-M2 | — |
| CG-8.9 | Update-the-rendering steps | Covered | DOM 6.9 (step table with owners); GR 6.9 | DM-M1/M2 | — |
| **9. Forms, UI, interactivity** | | | | | |
| CG-9.1 | Basic UI 3 | Partial | ED (`caret-color`); LY 6.5.2 and TX 6.18 (`text-overflow`); GR-D17 (outline) | — | `cursor` (beyond SVG cursor images, SV 6.8) and `resize` have no design; no `css-ui` WPT floor in LY or GR |
| CG-9.2 | Basic UI 4 | Partial | LY-D11 (`appearance`, `field-sizing`); ED 6.x (`user-select`, `caret-shape`, `caret-animation`); LY 6.14 (`pointer-events`) | — | `accent-color` and `nav-*` not mentioned |
| CG-9.3 | Customisable form controls (`base-select`) | Deferred (admission) | LY 6.3, LY-D10 ("held by LY-D10 until it meets the admission rule") | When admitted | Conflict: DOM 9.3 staffs "select and customizable select" and AX-O12 maps it, while LY-D10 holds it |
| CG-9.4 | Native form-control rendering | Covered | LY 6.3, LY-D11 (UA shadow trees, Izcal metrics); DOM 6.12 (`hg-forms`, 22 input types); shell UI design (pickers from `hg-layout-query` rects) | LY-M1, DM-M2 | The picker UIs themselves (date, colour, `select` popup) are not designed beyond placement |
| CG-9.5 | HTML rendering section, presentational hints | Covered | ST 6.6 (UA sheet in the engine image), 6.11 (presentational-hint block); LY 6.3 | ST-M1, LY-M1 | — |
| CG-9.6 | `inert` | Covered | DOM 6.11 (is-inert flag); LY 6.14 (skipped in hit testing); AX 4.2 | DM-M2 | — |
| CG-9.7 | `interactivity: inert` | Excluded | ST-D18 (rule applies, not named) | — | Searched "interactivity", "interactivity: inert": no hits |
| CG-9.8 | Forced colours, high contrast | Covered | GR-D25; AX R0; ST 6.6 | GR-M2, AX-M2 | — |
| CG-9.9 | Viewport and device environment | Partial | Visual viewport: GR 6.28, LY-D13; VirtualKeyboard and `interactive-widget` held (ED-D18) | — | Meta viewport behaviour undesigned (see CG-2.18); Viewport Segments (foldables) not mentioned |
| CG-9.10 | EyeDropper API | Missing | — | — | Searched "EyeDropper", "eye dropper", "pick colour from screen": no hits |
| CG-9.11 | Delegated ink | Deferred (admission) | GR 6.9 and 9.1 ("OS-delegated ink … admission questions under … LY-D10's process") | When admitted | — |
| **10. SVG and MathML** | | | | | |
| CG-10.1 | SVG 1.1 | Covered | SV 6.2–6.11; GR 6.16 | SV-M1/M2 | — |
| CG-10.2 | SVG 2 | Covered | SV (pinned to the SVG 2 CR of 6 October 2026), SV-D14 | SV-M1/M2 | `<mesh>`, `<hatch>` not mentioned (unshipped); several SVG 2 items held by SV-D13 |
| CG-10.3 | SMIL | Covered | AN 6.8 (AN-D6), SV 6.9 (SV-D4) | AN-M2, SV-M2 | — |
| CG-10.4 | SVG integration and image modes | Covered | SV 6.8 (SV-D10 to SV-D12), MD-D20 | SV-M1/M2 | — |
| CG-10.5 | SVG fonts | Excluded | SV-D7 ("not supported") | — | Reversal: measured use above 0.01%† |
| CG-10.6 | SVG markers and strokes | Covered | GR 6.16 (markers with `orient`, dashing); GR 6.4 (stroker) | GR-M2 | SVG Strokes module extras (unshipped) not named |
| CG-10.7 | MathML Core | Covered | LY 6.5.9, LY-D21; TX 6.8 (MATH table); TX-W2 math face | LY-M2 | LY-1 `mathml` 90%† |
| CG-10.8 | MathML 4 beyond Core | Excluded | LY-D21 ("MathML outside Core is not implemented") | — | Reversal: corpus or Interop need |
| **11. 2D and 3D graphics APIs** | | | | | |
| CG-11.1 | Canvas 2D | Covered | GR 6.15 (GR-D12, canonical readback, no noise) | GR-M1 | WPT `html/canvas/` |
| CG-11.2 | Path2D | Covered | GR-scope ("HTML Standard canvas chapter") | GR-M1 | Covered by scoping only; not named |
| CG-11.3 | OffscreenCanvas | Covered | GR 6.15 | GR-M3 | — |
| CG-11.4 | ImageBitmap | Covered | GR 6.15 (surface table) | GR-M1 | — |
| CG-11.5 | Canvas float16 and HDR | Covered | GR 6.5, 6.15 (`float16` stored as E16) | GR-M1/M2 | HDR canvas output to the display is not separately stated |
| CG-11.6 | HTML-in-canvas | Deferred (admission) | GR 6.15, 9.1 ("if admitted under the layout spec's admission rule") | When admitted | — |
| CG-11.7 | WebGL 1 | Covered | GR 6.14 (GR-D9, no ANGLE) | GR-M2 | Khronos suite |
| CG-11.8 | WebGL 2 | Covered | GR 6.14 | GR-M2 | — |
| CG-11.9 | WebGL extensions | Missing | — | — | Searched "WebGL extension", "S3TC", "ASTC", "compressed texture", "WEBGL_", "OES_", "EXT_", "multi-draw", "lose_context": no hits. The extension set Izcal exposes per device bucket (and its fingerprinting effect) is undecided; GR-D11's bucket logic covers WebGPU only |
| CG-11.10 | WebGPU | Covered | GR 6.13 (GR-D11: on by default only in cross-origin-isolated contexts, permission elsewhere) | GR-M3 | — |
| CG-11.11 | WGSL | Covered | GR 6.12 (`hg-wgsl`, direct MSL/SPIR-V/DXIL emission) | GR-M1 (internal), GR-M3 | — |
| CG-11.12 | WebGPU optional features | Covered | GR 6.13 (≤ 4† buckets with fixed features, timestamps ≥ 100 µs, compatibility mode as a feature level) | GR-M3 | WebGPU in workers not mentioned |
| CG-11.13 | Shape Detection | Missing | — | — | Searched "Shape Detection", "BarcodeDetector", "FaceDetector", "TextDetector": no hits (the ML service covers OCR for PDFs only) |
| CG-11.14 | `img.decode()`, `decoding`, lazy loading | Covered | MD 6.6; LY 6.15 (lazy-load observer) | MD-M1, LY-M2 | — |
| **12. Image formats and colour management** | | | | | |
| CG-12.1 | JPEG | Covered | MD 6.2 (MD-D1) | MD-M1 | 12-bit, lossless, hierarchical and arithmetic coding not exposed until a second engine ships them |
| CG-12.2 | PNG 3rd edition | Covered | MD 6.2 (incl. `cICP`, `mDCV`, `cLLI`) | MD-M1 | — |
| CG-12.3 | APNG | Covered | MD 6.2, 6.6 | MD-M1 | — |
| CG-12.4 | GIF | Covered | MD 6.2 (delay clamp measured black-box) | MD-M1 | — |
| CG-12.5 | WebP | Covered | MD 6.2 | MD-M1 | — |
| CG-12.6 | AVIF | Covered | MD 6.2 (still at MD-M1, sequences and gain maps) | MD-M1 | Gated by FP 6.5 peak-memory measurement |
| CG-12.7 | JPEG XL | Covered | MD-D2 (ships when 18181-3, MD-50 and peak memory pass; patent legal review first) | MD-M3 | — |
| CG-12.8 | HEIC/HEIF | Excluded | MD-D1 ("Not exposed … HEIC (HEVC patents; Safari only, through the OS)") | — | — |
| CG-12.9 | TIFF | Excluded | MD-D1 (TIFF decoded only for `sbix` glyphs) | — | — |
| CG-12.10 | JPEG 2000 | Excluded | MD-D1 (not exposed; `hg-jpx` for PDF only) | — | — |
| CG-12.11 | BMP, ICO, CUR | Covered | MD 6.2 | MD-M1 | — |
| CG-12.12 | SVG as image | Covered | SV 6.8, MD-D20 | SV-M1/M2 | — |
| CG-12.13 | HDR gain-map images | Covered | MD-D9 (ISO 21496-1 first, Ultra HDR fallback) | MD-M2 | — |
| CG-12.14 | ICC colour management | Covered | MD 6.5 (v2 and v4.4, generated parser, no ICC file ships) | MD-M1 | — |
| CG-12.15 | Wide-gamut and HDR output | Covered | MD-D8, MD-D9; GR 6.5 (E16, compositor tone mapping) | MD-M2, GR-M2 | — |
| CG-12.16 | Image density correction | Covered | MD 6.6 ("density-corrected size where the format carries it") | MD-M1 | One sentence; no WPT floor named |
| CG-12.17 | Image MIME sniffing | Covered | NW 6.2 (`hg-net-wire` sniffing per WHATWG MIME Sniffing); MD threat table ("One decoder per sniffed format") | NW-M1, MD-M1 | — |
| **13. Video and audio formats** | | | | | |
| CG-13.1 | MP4 / ISO BMFF, CMAF | Covered | MD 6.2 | MD-M2 | — |
| CG-13.2 | WebM / Matroska | Covered | MD 6.2 | MD-M2 | — |
| CG-13.3 | Ogg | Covered | MD 6.2 | MD-M2 | — |
| CG-13.4 | Native HLS | Covered | MD 6.7 (MD-D21) | MD-M3 | — |
| CG-13.5 | MPEG-DASH | Excluded | MD 6.20 ("DASH stays in page script (no browser ships native DASH)") | — | Played through MSE |
| CG-13.6 | H.264 / AVC | Covered | MD-D3 (owner decision 19 October 2026: OS decoders on macOS and Windows; Izcal's own built from M0; licence decision before first release; "Until then Linux has no H.264") | MD-M2 (OS), licence-gated on Linux | Licence-encumbered (see below) |
| CG-13.7 | H.265 / HEVC | Covered | MD-D3, G57 (OS full decoders only; never Izcal code) | MD-M2 | Licence-encumbered |
| CG-13.8 | H.266 / VVC | Missing | — | — | Searched "H.266", "VVC", "Versatile Video Coding": no hits; MD-D1's "Not exposed" list names AV2 but not VVC. No engine ships it, so this is a record gap, not work |
| CG-13.9 | VP8 | Covered | MD 6.2, MD-D4 (from the RFC narrative) | MD-M2 | — |
| CG-13.10 | VP9 | Covered | MD 6.2 | MD-M2 | — |
| CG-13.11 | AV1 | Covered | MD 6.2 (film grain exact); Stage 1 13.2 keeps AV1 despite pools | MD-M1 (intra), MD-M2 | — |
| CG-13.12 | Theora | Excluded | MD-D1 (not exposed) | — | — |
| CG-13.13 | AAC (LC, HE-AAC) | Covered | MD-D3 (AAC-LC own decoder "wherever its patents are confirmed expired"; HE-AAC OS decoders only) | MD-M2 (OS), MD-M3 | Licence-encumbered |
| CG-13.14 | MP3 | Covered | MD 6.2 | MD-M2 | — |
| CG-13.15 | Opus | Covered | MD 6.2, MD-D4 | MD-M2 | — |
| CG-13.16 | Vorbis | Covered | MD 6.2 | MD-M2 | — |
| CG-13.17 | FLAC | Covered | MD 6.2 | MD-M2 | — |
| CG-13.18 | PCM/WAV, G.711, G.722 | Covered | MD 6.2 (PCM, A-law, µ-law); MD-D30 (G.711) | MD-M2 | G.722 not mentioned |
| CG-13.19 | AC-3, E-AC-3, Dolby Vision | Missing | — | — | Searched "AC-3", "E-AC-3", "Dolby" (only the Dolby v. Snap lawsuit), "Dolby Vision": no codec decision. Neither the royalty-free policy nor an OS-decoder route is recorded |
| CG-13.20 | ALAC | Missing | — | — | Searched "ALAC", "Apple Lossless", "m4a lossless": no hits |
| CG-13.21 | MIME and codec strings | Covered | MD 6.15 (one policy table answers `canPlayType`, `isTypeSupported`, Media Capabilities) | MD-M2 | RFC 6381 parsing not named |
| **14. Media elements and streaming** | | | | | |
| CG-14.1 | `<video>`, `<audio>` | Covered | MD 6.7 (MD-D12 fixed UA timing) | MD-M2 | — |
| CG-14.2 | Default media controls | Covered | MD 6.12 (UA shadow tree, MD-12); shell UI design (exit affordance) | MD-M2 | `controlslist` not named |
| CG-14.3 | Text tracks | Covered | MD 6.12 (typed cues, "time marches on") | MD-M2 | — |
| CG-14.4 | WebVTT | Covered | MD 6.12; ST 6.9 (`::cue`) | MD-M2 | — |
| CG-14.5 | Media Fragments | Partial | SV Q-SV-11 (SVG `xywh=`, `t=` only) | — | `#t=` on media URLs is not in the media spec |
| CG-14.6 | Media Source Extensions | Covered | MD 6.7 (MD-D22 canonical quotas) | MD-M2 | — |
| CG-14.7 | MSE in workers, ManagedMediaSource | Covered | MD 6.7, 6.20 | MD-M2 | — |
| CG-14.8 | Encrypted Media Extensions | Covered | MD 6.11; Stage 1 13.1a (D-S7) | MD-M2 (Stage 1 M7) | — |
| CG-14.9 | Content Decryption Modules | Covered | D-S7 (13.1a): Clear Key own; native PlayReady on Windows (M7); Widevine and FairPlay only in protected media mode (WebView2, WKWebView); Linux Clear Key only; MD-D32 | M7 | Licence-encumbered. Signed-in playback tests deferred by the owner (MD-S5) |
| CG-14.10 | Media Capabilities | Covered | MD 6.7, 6.15 | MD-M2 | — |
| CG-14.11 | Media Playback Quality | Partial | MD 3 (cited as "the only standard health metric") | — | `getVideoPlaybackQuality()` has no design, anchor or WPT floor; MD-7 to MD-9 measure dropped frames internally |
| CG-14.12 | `requestVideoFrameCallback` | Covered | GR 6.11; MD 6.7 | MD-M2 | — |
| CG-14.13 | Media Session | Covered | MD 6.12 | MD-M2 | — |
| CG-14.14 | Picture-in-Picture (video) | Covered | MD 6.12 | MD-M2 | Conflict: Stage 1 13.2 "Everyday features" lists picture-in-picture under Stage 2 |
| CG-14.15 | Document Picture-in-Picture | Covered | MD 6.12 | MD-M2 | Single-engine, yet no admission check is recorded |
| CG-14.16 | Remote Playback | Missing | — | — | Searched "Remote Playback", "video.remote", "casting", "AirPlay", "Chromecast": no hits |
| CG-14.17 | Presentation API | Missing | — | — | Searched "Presentation API", "second screen", "PresentationRequest": no hits |
| CG-14.18 | Audio Output Devices | Covered | MD 6.8 | MD-M2 | — |
| CG-14.19 | Audio Session | Missing | — | — | Searched "Audio Session", "audioSession", "ducking": no hits |
| CG-14.20 | Autoplay policy | Covered | MD-D13 (stateless rule, `getAutoplayPolicy()`) | MD-M2 | — |
| CG-14.21 | WebCodecs | Covered | MD 6.7 | Decode MD-M2, encode MD-M3 | — |
| CG-14.22 | ImageDecoder | Covered | MD 6.20 | MD-M2 | — |
| CG-14.23 | Web Audio | Covered | MD 6.8 (MD-D14 bit-identical) | MD-M2 | — |
| CG-14.24 | AudioWorklet | Covered | MD-D31 | MD-M2 | Real-time thread gated by MD-S8 |
| CG-14.25 | Web MIDI | Covered | Devices spec (DV); Stage 1 13.2 ("behind the off-by-default per-origin Experimental device APIs setting") | DV-M2; exposure decision Stage 2 | — |
| **15. Capture and real-time communication** | | | | | |
| CG-15.1 | Media Capture and Streams | Covered | MD 6.10, anchor table | MD-M2 | — |
| CG-15.2 | Media Capture extensions (blur, framing, PTZ) | Missing | — | — | Searched "background blur", "face framing", "voice isolation", "PTZ", "pan-tilt", "mediacapture-extensions": no hits |
| CG-15.3 | Image Capture | Missing | — | — | Searched "ImageCapture", "takePhoto", "Image Capture": no hits |
| CG-15.4 | Capture from DOM elements | Partial | GR 6.15, 6.27 (`canvas.captureStream()`) | GR-M3 | `video.captureStream()` / `audio.captureStream()` not mentioned |
| CG-15.5 | Screen Capture | Covered | MD 6.10; GR 6.27 (capture split) | MD-M2, GR-M3 | — |
| CG-15.6 | Capture Handle | Missing | — | — | Searched "Capture Handle", "captureHandle", "setCaptureHandleConfig": no hits |
| CG-15.7 | Region Capture | Covered | GR 6.27 (`cropTo`) | GR-M3 | Single-engine, with no admission check recorded |
| CG-15.8 | Element Capture | Covered | GR 6.27 (`restrictTo`) | GR-M3 | As CG-15.7 |
| CG-15.9 | Captured Surface Control, captured mouse events | Missing | — | — | Searched "Captured Surface Control", "captured mouse", "CapturedMouseEvent": no hits |
| CG-15.10 | MediaStreamTrack insertable streams | Covered | MD 6.7, 6.20 (`MediaStreamTrackProcessor`) | MD-M2 | `VideoTrackGenerator` not named |
| CG-15.11 | MediaRecorder | Covered | MD 6.7 (WebM everywhere, MP4 where MD-D3 allows) | MD-M3 | — |
| CG-15.12 | Content hints | Missing | — | — | Searched "contentHint", "content hint", "mst-content-hint": no hits |
| CG-15.13 | HTML Media Capture (`<input capture>`) | Missing | — | — | Searched "HTML Media Capture", "capture=", "`capture` attribute": no hits. Mobile (Stage 2) |
| CG-15.14 | WebRTC 1.0 API | Partial | NW 6.12 (ICE, DTLS, SRTP, SCTP); MD 6.10 (depacketizing, jitter buffers, encode, FEC); MD anchor table cites "WebRTC 1.0" | NW-M4, MD-M2 | Searched "RTCPeerConnection", "SDP", "JSEP", "transceiver", "simulcast", "DTMF", "setCodecPreferences": no hits. The API layer (peer-connection state machines, JSEP offer and answer, SDP parsing, transceivers, simulcast, DTMF) has no owner |
| CG-15.15 | RTCDataChannel | Covered | NW 6.12 (`hg-sctp`, RFC 8831); NW-42 ("Calls and data channels complete") | NW-M4 | The `RTCDataChannel` API object belongs to the CG-15.14 gap |
| CG-15.16 | WebRTC statistics | Covered | MD 6.10 (computed from recorded arrival times) | MD-M2 | — |
| CG-15.17 | WebRTC Encoded Transform | Covered | MD 6.10 (`RTCRtpScriptTransform`) | MD-M2 | — |
| CG-15.18 | WebRTC extensions, SVC, priority | Missing | — | — | Searched "SVC", "scalable video", "RTP header extension", "DSCP", "webrtc-priority", "RTCIceTransport": no relevant hits |
| CG-15.19 | WebRTC Identity | Missing | — | — | Searched "WebRTC Identity", "peerIdentity", "identity provider" (WebRTC sense): no hits. Barely shipped; low priority |
| CG-15.20 | WebRTC codecs and media stack | Partial | MD-D30 (VP8, Opus, G.711, H.264 per MD-D3); MD-D16 (`hg-voice` echo cancellation); MD 6.10 (jitter buffers, FEC, retransmission) | MD-M2/M3 | RTP congestion control and bandwidth estimation (transport-cc or equivalent), RTCP feedback and rate adaptation are not designed. Searched "congestion control" (QUIC only), "bandwidth estimation", "transport-cc" |
| **16. XR, speech, printing and other rendering** | | | | | |
| CG-16.1 | WebXR Device API | Missing | — | — | Searched "WebXR", "immersive", "OpenXR", "XRWebGLLayer": only an unrelated citation in HGBA-2 9.3 and Appendix A |
| CG-16.2 | WebXR modules | Missing | — | — | As CG-16.1 |
| CG-16.3 | Web Speech: synthesis | Partial | MD 6.20 ("speech synthesis and recognition (a separate spec …)"); extensions `tts` Stage 3 | — | The "separate spec" does not exist; no owner, design or stage |
| CG-16.4 | Web Speech: recognition | Partial | MD 6.20; ML 1 ("speech recognition is 'an ML service feature under B10'") | — | No design in the ML service spec's model set (image descriptions, OCR, captions) |
| CG-16.5 | Printing | Covered | LY 6.7, LY-D8 (paged layout Stage 1); GR 6.27 (print raster and preview Stage 1); Stage 1 13.2 (print UI Stage 2) | LY-M2, GR-M3; UI Stage 2 | — |
| CG-16.6 | Save as PDF | Deferred (Stage 2) | PDF 6.19 (writer core Stage 1 for saving and printing PDFs); PD-M4 "Writer (Stage 2, with the print UI)"; Stage 1 13.2 | PD-M4 (Stage 2) | — |
| CG-16.7 | Built-in PDF viewer | Covered | PDF spec (PD-D1 to PD-D26) | PD-M1/M2 (Stage 1 M7) | Editing Stage 2 |
| CG-16.8 | Fullscreen rendering | Covered | DOM 6.9 (fullscreen steps, broker-minted activation); MD 6.12; ST 6.7.1 | DM-M2, MD-M2 | — |
| CG-16.9 | Scroll-to-text-fragment rendering | Covered | ED (URL Fragment Text Directives, `::target-text`); LY 6.17 (top-level only) | ED-M2 | — |
| CG-16.10 | Find-in-page highlighting, `until-found` | Covered | ED (find across frames, `::search-text`); LY 6.9 (`hidden=until-found`); Stage 1 §6 | ED-M2 | `beforematch` not named |
| CG-16.11 | Reader view | Deferred (Stage 2) | Stage 1 §6 ("Save for offline … Reader rendering on top", Stage 2); PDF 9.1 ("Stage 2 with the reader view") | Stage 2 | No design yet |

## Summary

### Counts per status per group

"Deferred" combines admission holds and later-stage assignments.

| Group | Rows | Covered | Partial | Missing | Excluded | Deferred |
|---|---|---|---|---|---|---|
| 1. Syntax, cascade, selectors | 21 | 16 | 0 | 2 | 2 | 1 |
| 2. Values, units, conditionals | 20 | 11 | 4 | 0 | 1 | 4 |
| 3. Colour, images, backgrounds, borders | 16 | 10 | 2 | 0 | 2 | 2 |
| 4. Box, display, layout modes | 33 | 21 | 1 | 1 | 5 | 5 |
| 5. Scrolling, fragmentation, paged media | 11 | 5 | 2 | 1 | 1 | 2 |
| 6. Text, fonts, lists, content | 22 | 16 | 4 | 1 | 1 | 0 |
| 7. Transforms, animation, effects | 18 | 15 | 1 | 0 | 1 | 1 |
| 8. Object models, observers | 9 | 6 | 1 | 0 | 2 | 0 |
| 9. Forms, UI | 11 | 4 | 3 | 1 | 1 | 2 |
| 10. SVG, MathML | 8 | 6 | 0 | 0 | 2 | 0 |
| 11. Graphics APIs | 14 | 11 | 0 | 2 | 0 | 1 |
| 12. Image formats, colour | 17 | 14 | 0 | 0 | 3 | 0 |
| 13. Video and audio formats | 21 | 16 | 0 | 3 | 2 | 0 |
| 14. Media elements, streaming | 25 | 20 | 2 | 3 | 0 | 0 |
| 15. Capture, RTC | 20 | 9 | 3 | 8 | 0 | 0 |
| 16. XR, speech, print, other | 11 | 5 | 2 | 2 | 0 | 2 |
| **Total** | **277** | **185** | **25** | **24** | **23** | **20** |

Of the 20 Deferred rows, 18 are admission holds (ST-D18, LY-D10, AN-D9, SV-D13, MD-D19's CSSWG-issue wait) and 2 are Stage 2 assignments (CG-16.6 Save as PDF, CG-16.11 reader view). Some Covered rows rest only on a scoping statement, with the feature itself unnamed (CG-3.4 `contrast-color()`, CG-11.2 Path2D, CG-2.11, CG-2.12), or on a one-line mention (CG-12.16). Their evidence cells say so, and they are the first to re-check if a module's scope changes.

### All Missing rows (24)

| # | Feature | Note |
|---|---|---|
| CG-1.2 | CSS Snapshot as a floor | No completeness check of the stable CSS set exists; the cause of several orphans below |
| CG-1.9 | CSS Namespaces / `@namespace` | REC, three engines; likely an oversight |
| CG-4.28 | `shape()` | **Interop 2026 focus area**; LY-2 implicitly requires it |
| CG-5.7 | Spatial navigation | Flag-only in Chromium; low priority |
| CG-6.5 | `text-size-adjust` | Mobile (Android, Stage 2) |
| CG-9.10 | EyeDropper | Single-engine |
| CG-11.9 | WebGL extensions | Three engines expose subsets; a fingerprinting and compatibility decision is needed |
| CG-11.13 | Shape Detection | Single-engine (partial) |
| CG-13.8 | H.266 / VVC | No engine ships it; add to MD-D1's "not exposed" list |
| CG-13.19 | AC-3, E-AC-3, Dolby Vision | Licence-encumbered; no decision |
| CG-13.20 | ALAC | Single-engine |
| CG-14.16 | Remote Playback | Two engines (partial) |
| CG-14.17 | Presentation API | Single-engine |
| CG-14.19 | Audio Session | Single-engine |
| CG-15.2 | Media Capture extensions | Partial in all three engines |
| CG-15.3 | Image Capture | Single-engine |
| CG-15.6 | Capture Handle | Single-engine |
| CG-15.9 | Captured Surface Control, captured mouse events | Single-engine |
| CG-15.12 | Content hints | Two engines |
| CG-15.13 | HTML Media Capture | Mobile (Stage 2) |
| CG-15.18 | WebRTC extensions, SVC, priority | Partial in all engines; SVC matters for video calls |
| CG-15.19 | WebRTC Identity | Effectively dead |
| CG-16.1 | WebXR Device API | Chromium and visionOS |
| CG-16.2 | WebXR modules | As CG-16.1 |

Several of these are single-engine features that an admission rule would hold. They are Missing rather than Excluded because ST-D18 and LY-D10 are CSS and layout rules and do not name them; graphics extends "LY-D10's process" only to delegated ink and HTML-in-canvas. **No admission rule covers non-CSS web APIs in general** (capture, media, devices outside the devices spec).

### All Partial rows (25) and what is missing

| # | Feature | Missing piece |
|---|---|---|
| CG-2.8 | `env()` | Which environment variables exist (safe-area, titlebar-area, keyboard-inset, viewport-segment) |
| CG-2.17 | Media Queries 5 | `display-mode`, `prefers-reduced-data`, `scripting`, `video-dynamic-range`, `@custom-media` |
| CG-2.18 | Viewport meta | Effect of `<meta name=viewport>` on the layout viewport |
| CG-2.19 | CSS `zoom` | Semantics (effective zoom), CSSOM interaction, `css-zoom` WPT floor; Interop 2025–2026 |
| CG-3.8 | Backgrounds 4 | `background-clip: text` / `border-area` painting; GR scopes Backgrounds 3 only |
| CG-3.16 | `-webkit-` legacy, Compat Standard | No owner for the Compat Standard or the prefixed-alias list |
| CG-4.23 | Logical properties | Logical-to-physical group resolution in the cascade; `css-logical` floor |
| CG-5.4 | Overscroll behaviour | Scroll chaining and bounce design |
| CG-5.5 | Scrollbar styling | `scrollbar-color`, `::-webkit-scrollbar` |
| CG-6.3 | Text decoration 3 | Decoration, emphasis-mark and `text-shadow` painting; `css-text-decor` floor |
| CG-6.4 | Text decoration 4 | Skip-ink intersection, thickness and offset painting |
| CG-6.20 | Counter Styles 3 | `@counter-style`, `symbols()`, predefined counter-style data (and its licence) |
| CG-6.21 | Generated content 3 | CSS Content 3 not in LY-scope; `quotes`, content images |
| CG-7.18 | Geometry Interfaces | An owner for `DOMMatrix`, `DOMPoint`, `DOMQuad` |
| CG-8.3 | Typed OM | Q-ST-14 open; no decision |
| CG-9.1 | Basic UI 3 | `cursor` and `resize`; `css-ui` floor |
| CG-9.2 | Basic UI 4 | `accent-color`, `nav-*` |
| CG-9.9 | Viewport and device environment | Viewport segments; meta viewport |
| CG-14.5 | Media Fragments | `#t=` for media URLs |
| CG-14.11 | Media Playback Quality | `getVideoPlaybackQuality()` |
| CG-15.4 | Capture from elements | `video.captureStream()` |
| CG-15.14 | WebRTC 1.0 API | RTCPeerConnection, JSEP, SDP, transceivers, simulcast, DTMF: no owner |
| CG-15.20 | WebRTC media stack | RTP congestion control, bandwidth estimation, RTCP feedback |
| CG-16.3 | Speech synthesis | The "separate spec" MD 6.20 points to does not exist |
| CG-16.4 | Speech recognition | No design in the ML service |

### Proposed grouping into new specs or amendments

| Candidate | Rows | Reasoning |
|---|---|---|
| **New: WebRTC API and real-time media spec** | CG-15.14, 15.20, 15.18, 15.12, 15.19 (record as excluded), 15.15's API half | Transport (NW 6.12) and media plumbing (MD 6.10) are owned, but nobody owns the API between them: RTCPeerConnection, JSEP/SDP, transceivers, simulcast and SVC, congestion control. This is the largest single unowned piece in the strand, and RFC 7742 interop (NW-42) depends on it |
| **New: Speech spec** (synthesis and recognition) | CG-16.3, 16.4, plus CSS Speech (CG-1.21) as context | MD 6.20 already names "a separate spec"; recognition must live under B10 with the ML service and synthesis needs OS-voice versus Izcal-voice and fingerprinting decisions |
| **Decision record: XR** | CG-16.1, 16.2 | Either a WebXR spec (GPU, devices, HAL and permissions) or an explicit HGBA-2 non-goal entry. Today it is silent |
| **Amendment: style spec, CSS completeness** (adopt CSS Snapshot 2026 as the floor, as the audit's scoping note suggests) | CG-1.2, 1.9, 4.23, 6.20, 2.17, 2.8, 2.19, 3.16, 8.3 | One scoping sentence ("every module in CSS Snapshot 2026, plus the Compat Standard") would close most orphans, and it forces each module to an owner. Add `css-namespaces`, `css-logical`, `css-counter-styles`, `css-zoom`, `compat` to ST-3's floors |
| **Amendment: graphics spec, paint coverage** | CG-6.3, 6.4, 3.8, 9.1 (cursor), 5.4, 5.5 | Add Text Decoration 3 and 4, Backgrounds 4, `scrollbar-color` and overscroll effects to GR 4.2 and the GR anchor table, with WPT `css-text-decor` and `css-backgrounds` floors |
| **Amendment: layout spec** | CG-4.28 (`shape()` with Shapes 2), 6.21 (CSS Content 3 in LY-scope), 2.18/9.9 (meta viewport, viewport segments for Stage 2), 6.5 (`text-size-adjust`, Stage 2) | `shape()` is an Interop 2026 area that LY-2 already makes binding; the rest are box-generation and mobile viewport rules that belong with LY 6.3 and the Stage 2 Android work |
| **Amendment: images and media spec** | CG-14.5, 14.11, 14.16, 14.17, 14.19, 15.2, 15.3, 15.4 (video), 15.6, 15.9, 15.13, 13.8, 13.19, 13.20 | Extend the 6.20 coverage table with an explicit line per API: either designed or recorded as not admitted. The codec rows (VVC, Dolby, ALAC) belong in MD-D1's exposed and not-exposed tables with a licence note |
| **Amendment: graphics spec, GPU API surface** | CG-11.9 (WebGL extensions), CG-11.12's WebGPU in workers, CG-11.13 (Shape Detection → or the ML service) | GR-D11's bucket model should extend to the WebGL extension set, which is both a compatibility list and a fingerprint |
| **Amendment: DOM or a small "geometry and UI APIs" section** | CG-7.18, 9.10 (EyeDropper), 9.2 (`accent-color` with forms) | Geometry Interfaces are shared by SVG, canvas and CSSOM; DOM's bindings spec is the natural owner |
| **Amendment: HGBA-2 or Stage 1, admission rule for non-CSS APIs** | All single-engine Missing rows above | ST-D18 and LY-D10 cover CSS and layout; GR 6.9 borrows "LY-D10's process" ad hoc. One platform-wide admission rule would turn about ten Missing rows into recorded holds and settle the inconsistencies found (Document PiP, Region and Element Capture shipped while single-engine; `corner-shape` scoped in GR but held by the rule; customizable `select` staffed in DOM but held in LY) |

### Inconsistencies found between specs

- Customizable `select`: DOM 9.3 staffs it and AX-O12 maps its roles; LY-D10 holds `appearance: base-select`.
- `corner-shape`: GR scopes Borders 4 and designs `corner-shape` outlines; it is single-engine, so ST-D18 and LY-D10 would hold it.
- Picture-in-Picture: MD 6.12 and MD-M2 build it in Stage 1; Stage 1 13.2's "Everyday features" lists it under Stage 2.
- H.264 on Linux: Stage 1 13.2 says OS decoders "(Media Foundation, VideoToolbox or GStreamer)"; MD-D3 (owner decision) says Linux has no H.264 until a licence decision and uses no system codec framework. Media 9.5's amendment to 13.2 is still unapplied.
- DOM 6.9's agent table names "paint" and "layout" worklet agents that no spec defines (CG-4.32, CG-8.4).

### Licence-encumbered items and what the specs decide

| Item | Decision | Where |
|---|---|---|
| H.264 / AVC | Royalty-free by default: OS full decoders and encoders on macOS and Windows; Izcal's own decoder and Constrained Baseline encoder built from M0, shipped on Linux only if the owner takes the AVC licence at a decision point before first public release; "Until then Linux has no H.264" and Linux WebRTC negotiates VP8, VP9 or AV1. Never drive H.264 hardware through low-level APIs (G57). OpenH264 rejected (M-1, MIT-only, FP-D13) | MD-D3 (owner decision 19 October 2026), MD-D30, Stage 1 13.2 |
| AAC | AAC-LC: Izcal's own decoder ships "wherever its patents are confirmed expired" (legal review); HE-AAC v1 and v2 only through OS decoders. OS-decoded audio keeps D-S1 noise until `hg-aac` replaces it | MD-D3, MD 6.2, MD-D14 |
| HEVC | Only OS full decoders (macOS; Windows when the HEVC Video Extensions are installed); never Izcal code; never low-level hardware APIs | MD-D3, G57 |
| HEIC / HEIF | Not exposed (HEVC patents) | MD-D1 |
| DRM CDMs | D-S7: no unlicensed or reimplemented CDM, ever; Clear Key built by Izcal; native PlayReady on Windows (M7); Widevine and FairPlay only through protected media mode (WebView2 or WKWebView, user-visible switch on real encrypted playback, never on a probe); Linux Clear Key only until a licensing track succeeds; truthful key-system answers plus a user-initiated affordance (MD-D32) | Stage 1 13.1 and 13.1a, MD 6.11 |
| Hyphenation data | Izcal authors its own lexicons and patterns per language with native linguists; no hyph-utf8 or any third-party pattern set, whatever its licence | TX-D14 |
| AV1, VP9 (pool claims) | Kept despite Sisvel and Access Advance pools; AOM licence text shipped; Izcal asserts no patents against these formats | MD 6.2, Stage 1 13.2 |
| JPEG XL | Legal review before MD-D2 ships (2022 rANS-related patent question) | MD 6.2 patent table |
| AC-3, E-AC-3, Dolby Vision | **No decision recorded** (CG-13.19 Missing) | — |
| Voice processing (AEC, noise suppression) | Patent search by the legal review before MD-M2 | MD 6.2 patent table |
| HRTF data | Izcal commissions its own measured set (MIT); analytic model until then | MD-D15 |
| ICC profiles | No ICC-authored profile ships; colour spaces built from published parameters | MD-D8 |
| Fonts (Noto, Liberation, STIX, Twemoji) | None ships; Izcal font set (MIT) plus an interim OS tier; COLRv1 emoji from Fluent Emoji (MIT) | TX-D24, TX-D25, TX 6.4 |
| Counter-style predefined data | Not addressed (CG-6.20 Partial); the W3C NOTE's data would need the standards-data check | — |
