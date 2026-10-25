# Izcal feature inventory audit: CSS, graphics and media

Compiled 24 October 2026. Primary sources were preferred (specifications, standards indexes, vendor release notes).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data, or a well-known fact not re-checked this session; **Low** = secondary source only, or the researcher's recollection not checked against a source this session.

M-1 note: no third-party source code was opened for this note. No engine, codec-library or tool source, headers, commits, diffs, reviews or code-repository data files (web-features, browser-compat-data) were read. Sources read this session: the CSSWG current-work page (last updated 11 September 2026; read in full), the WPT top-level directory listing and the `css/` directory listing on wpt.live (directory names only, no test files), the web.dev "Interop 2026" announcement (12 February 2026; focus and investigation areas read in full), and search-result snippets for Firefox 147 and 150–151 release notes, WebKit's Safari 26.4 notes, Chrome's "Shipping JPEG XL in Chrome" post (6 October 2026) and Grid Lanes coverage.

**Method notes**
- Spec existence and maturity for every CSS row were checked against the CSSWG current-work table. That part of each CSS row is High.
- The "Ships in" column comes mostly from the compiler's knowledge up to mid-2026 plus the spot checks above. webstatus.dev renders client-side and returned no data to the fetch tool, so Baseline status was **not** read from it. A ship cell checked this session is noted in the row; otherwise the row is capped at Medium. Where the compiler is unsure, the row says Low.
- Ship-cell notation: **Y** = on by default in stable; **partial** = a subset, some platforms only, or prefixed; **flag** = behind a flag or preview only; **N** = not shipped; **dep.** = shipped then deprecated or removed. "Single-engine" marks features in only one engine's stable release.
- Licence and patent notes are what the format owners or licensing bodies say in public prose, as recalled. They were not re-read this session (search budget), so they are Medium at best. They need a High re-check before any Izcal decision depends on them.
- Web searches used: 8.

## 1. CSS: syntax, cascade and selectors

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-1.1 | CSS 2.1 core | The base visual formatting model: block/inline formatting, floats, the CSS2 positioning schemes, stacking and z-index, tables, generated content. | [CSS 2.1 (REC)](https://www.w3.org/TR/CSS21/) | Y / Y / Y | High |
| CG-1.2 | CSS Snapshot 2026 | The CSSWG's statement of which modules count as stable CSS in 2026. Useful as a "must ship" floor. | [CSS Snapshot 2026 (NOTE)](https://www.w3.org/TR/css-2026/) | n/a (index) | High |
| CG-1.3 | CSS Syntax 3 | The tokenizer and parser for style sheets, `@` rules, declarations, error recovery, and `An+B`. | [CSS Syntax 3 (CRD)](https://www.w3.org/TR/css-syntax-3/) | Y / Y / Y | Medium |
| CG-1.4 | Cascade and inheritance 3/4 | Origins, importance, specificity, inheritance, `initial`/`inherit`/`unset`/`revert`, `all`, `@import`. | [css-cascade-4 (CR)](https://www.w3.org/TR/css-cascade-4/) | Y / Y / Y | Medium |
| CG-1.5 | Cascade layers | `@layer`, `revert-layer`, layered `@import`. | [css-cascade-5 (CR)](https://www.w3.org/TR/css-cascade-5/) | Y / Y / Y | Medium |
| CG-1.6 | `@scope` | Scoped style rules with upper and lower bounds and scope proximity in the cascade. | [css-cascade-6 (WD)](https://drafts.csswg.org/css-cascade-6/) | Y / Y / Y (Gecko since late 2025, recalled) | Low |
| CG-1.7 | `revert-rule` keyword | Rolls a property back to the previous matching rule. | [css-cascade-6 (WD)](https://drafts.csswg.org/css-cascade-6/) | N / Y (Firefox 150, snippet) / N — single-engine | Medium |
| CG-1.8 | CSS Nesting | Nested style rules and the `&` selector. | [css-nesting-1 (WD)](https://drafts.csswg.org/css-nesting-1/) | Y / Y / Y | Medium |
| CG-1.9 | CSS Namespaces | `@namespace` and namespace prefixes in selectors (XML, SVG, MathML). | [css-namespaces-3 (REC)](https://www.w3.org/TR/css-namespaces-3/) | Y / Y / Y | High |
| CG-1.10 | Style attributes | The grammar of the `style=""` attribute. | [css-style-attr (REC)](https://www.w3.org/TR/css-style-attr/) | Y / Y / Y | High |
| CG-1.11 | Selectors 3 | Type, class, ID, attribute, structural pseudo-classes and combinators. | [selectors-3 (REC)](https://www.w3.org/TR/selectors-3/) | Y / Y / Y | High |
| CG-1.12 | Selectors 4 | `:is()`, `:where()`, `:not()` lists, `:has()`, `:nth-child(of S)`, `:focus-visible`, `:focus-within`, `:user-valid`/`:user-invalid`, `:dir()`, `:lang()` ranges, `:read-write`, `:placeholder-shown`, `:defined`. The column combinator `||` and time pseudo-classes are not interoperable. | [selectors-4 (WD)](https://www.w3.org/TR/selectors-4/) | Y / Y / Y (core); `||` N / N / N | Medium |
| CG-1.13 | State pseudo-classes for UI | `:open`, `:popover-open`, `:modal`, `:fullscreen`, `:picture-in-picture`, `:autofill`, `:state()` (custom states). `:open` is an Interop 2026 focus area. | [selectors-4](https://drafts.csswg.org/selectors-4/), [HTML](https://html.spec.whatwg.org/multipage/semantics-other.html#pseudo-classes) | mostly Y / Y / Y; `:open` partial | Medium |
| CG-1.14 | Media pseudo-classes | `:playing`, `:paused`, `:seeking`, `:buffering`, `:stalled`, `:muted`, `:volume-locked`. Interop 2026 focus area. | [selectors-4](https://drafts.csswg.org/selectors-4/#resource-pseudos) | N / Y (Firefox 150, snippet) / Y | Medium |
| CG-1.15 | Selectors 5 | Exploratory: `:local-link`, reference combinators, more. | [selectors-5 (FPWD)](https://drafts.csswg.org/selectors-5/) | N / N / N | Medium |
| CG-1.16 | Pseudo-elements 4 | `::before`/`::after`, `::marker`, `::placeholder`, `::file-selector-button`, `::selection`, `::target-text`, `::spelling-error`, `::grammar-error`, `::highlight()`, `::details-content`, `::first-line`/`::first-letter`. | [css-pseudo-4 (WD)](https://www.w3.org/TR/css-pseudo-4/) | Y / partial / partial (spelling/grammar pseudo-elements not in Gecko) | Medium |
| CG-1.17 | Shadow DOM styling | `:host`, `:host()`, `:host-context()` (Chromium only), `::slotted()`, `::part()`. | [css-scoping-1](https://drafts.csswg.org/css-scoping-1/), [css-shadow-parts-1](https://www.w3.org/TR/css-shadow-parts-1/) | Y / Y / Y; `:host-context` Chromium only | Medium |
| CG-1.18 | Custom properties | `--*` properties and `var()`. | [css-variables-1 (CR)](https://www.w3.org/TR/css-variables-1/) | Y / Y / Y | High |
| CG-1.19 | Registered custom properties | `@property` and `CSS.registerProperty()`: typed, animatable custom properties. | [css-properties-values-api-1 (WD)](https://www.w3.org/TR/css-properties-values-api-1/) | Y / Y / Y | Medium |
| CG-1.20 | Custom functions and mixins | `@function` custom functions (shipped in Chromium only) and `@mixin`/`@apply` (no engine). | [css-mixins-1 (FPWD)](https://drafts.csswg.org/css-mixins-1/) | partial / N / N — single-engine | Low |
| CG-1.21 | CSS Speech | `speak`, `voice-family`, pauses and cues for aural rendering. | [css-speech-1 (CRD)](https://www.w3.org/TR/css-speech-1/) | N / N / N | Medium |

**Implications:** The cascade and selector surface is fully interoperable and large. `:has()`, layers, `@scope` and nesting are all in three engines, so Izcal's style engine must treat them as baseline, not stretch goals.

## 2. CSS: values, units, functions and conditional rules

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-2.1 | Values and Units 3 | Length, angle, time, resolution units; `calc()`; `attr()` (legacy, `content` only); `toggle()` never shipped. | [css-values-3 (CR)](https://www.w3.org/TR/css-values-3/) | Y / Y / Y | High |
| CG-2.2 | Values and Units 4 | Viewport units (`svh`/`lvh`/`dvh`), `rem`, `ex`/`ch`/`ic`/`cap`/`rlh`/`lh`, math functions (`min`/`max`/`clamp`, trig, `round`, `mod`, `rem`, `abs`, `sign`, `pow`, `sqrt`, `hypot`, `log`, `exp`). | [css-values-4 (WD)](https://www.w3.org/TR/css-values-4/) | Y / Y / Y | Medium |
| CG-2.3 | Typed `attr()` | `attr()` in any property, with a type or unit. Interop 2026 focus area. | [css-values-5 (WD)](https://drafts.csswg.org/css-values-5/#attr-notation) | Y / partial / N | Medium |
| CG-2.4 | `if()` | Inline conditionals on style, media and supports queries. | [css-values-5](https://drafts.csswg.org/css-values-5/#if-notation) | Y / N / N — single-engine | Low |
| CG-2.5 | Tree-counting functions | `sibling-index()`, `sibling-count()`. | [css-values-5](https://drafts.csswg.org/css-values-5/#tree-counting) | Y / N / partial | Low |
| CG-2.6 | `random()` | Per-element random values. | [css-values-5](https://drafts.csswg.org/css-values-5/#randomness) | N / N / partial | Low |
| CG-2.7 | `progress()`, `*-mix()`, `calc-size()` | Interpolation helpers. `calc-size()` and `interpolate-size` let `height: auto` animate. | [css-values-5](https://drafts.csswg.org/css-values-5/), [css-sizing-4](https://drafts.csswg.org/css-sizing-4/) | partial / N / N — mostly single-engine | Low |
| CG-2.8 | Environment variables | `env()`: `safe-area-inset-*`, `titlebar-area-*`, `keyboard-inset-*`, `viewport-segment-*`. | [css-env-1 (FPWD)](https://drafts.csswg.org/css-env-1/) | Y / Y / Y (safe-area); others Chromium only | Medium |
| CG-2.9 | Linked parameters | `link-parameters` to pass custom properties into SVG images. | [css-link-params-1 (FPWD)](https://drafts.csswg.org/css-link-params-1/) | N / N / N | Medium |
| CG-2.10 | Conditional rules 3 | `@media`, `@supports` and `CSS.supports()`. | [css-conditional-3 (CR)](https://www.w3.org/TR/css-conditional-3/) | Y / Y / Y | High |
| CG-2.11 | Conditional rules 4 | `@supports selector()`. | [css-conditional-4 (CR)](https://www.w3.org/TR/css-conditional-4/) | Y / Y / Y | Medium |
| CG-2.12 | Conditional rules 5 | `@supports font-tech()`/`font-format()`, `@when`/`@else` (unshipped), container queries are specified in css-contain-3. | [css-conditional-5 (WD)](https://drafts.csswg.org/css-conditional-5/) | partial / partial / partial; `@when` N/N/N | Low |
| CG-2.13 | Container size queries | `container-type`, `@container`, `cq*` units, name-only queries (Safari 26.4). | [css-contain-3 (WD)](https://drafts.csswg.org/css-contain-3/) | Y / Y / Y | Medium |
| CG-2.14 | Container style queries | `@container style(--x: y)`. Interop 2026 focus area. | [css-contain-3](https://drafts.csswg.org/css-contain-3/#style-container) | Y / Y (Firefox 151, snippet) / Y | Medium |
| CG-2.15 | Container scroll-state queries | `@container scroll-state(stuck/snapped/scrollable)`. | [css-conditional-5](https://drafts.csswg.org/css-conditional-5/#scroll-state-container) | Y / N / N — single-engine | Low |
| CG-2.16 | Media Queries 3/4 | Media types, range syntax, `width`, `resolution`, `orientation`, `hover`/`pointer`, `any-*`, `color-gamut`, `dynamic-range`, `update`, `overflow-*`. | [mediaqueries-4 (CRD)](https://www.w3.org/TR/mediaqueries-4/) | Y / Y / Y | Medium |
| CG-2.17 | Media Queries 5 | User-preference features `prefers-color-scheme`, `prefers-reduced-motion`, `prefers-contrast`, `forced-colors`, `prefers-reduced-transparency`, `prefers-reduced-data`, `inverted-colors`, `scripting`, `video-dynamic-range`, `display-mode`; `@custom-media`. | [mediaqueries-5 (WD)](https://www.w3.org/TR/mediaqueries-5/) | partial / partial / partial (first four Y everywhere; `@custom-media` N/N/N) | Medium |
| CG-2.18 | Viewport rule | `@viewport` (deprecated) and the `viewport` meta interaction, now `css-viewport`'s `zoom` interaction. | [css-viewport-1 (WD)](https://drafts.csswg.org/css-viewport/) | dep. | Medium |
| CG-2.19 | CSS `zoom` | Standardised `zoom` property that scales and affects layout. Interop 2025–2026 focus area. | [css-viewport-1](https://drafts.csswg.org/css-viewport/#zoom-property) | Y / Y / Y | Medium |
| CG-2.20 | Easing functions | `cubic-bezier()`, `steps()`, `linear()` piecewise easing. | [css-easing-1 (CRD)](https://www.w3.org/TR/css-easing-1/), [css-easing-2](https://drafts.csswg.org/css-easing-2/) | Y / Y / Y | Medium |

**Implications:** The value grammar is where single-engine features accumulate fastest (`if()`, `@function`, `random()`). Izcal needs a policy for which Values 5 functions to track, separate from the stable Values 4 floor.

## 3. CSS: colour, images, backgrounds and borders

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-3.1 | Colour 3 | sRGB keywords, `rgb()`/`hsl()`, alpha, `currentColor`. | [css-color-3 (REC)](https://www.w3.org/TR/css-color-3/) | Y / Y / Y | High |
| CG-3.2 | Colour 4 | `lab()`, `lch()`, `oklab()`, `oklch()`, `hwb()`, `color()` with predefined spaces (display-p3, rec2020, a98, prophoto, xyz), gamut mapping. | [css-color-4 (CRD)](https://www.w3.org/TR/css-color-4/) | Y / Y / Y | Medium |
| CG-3.3 | Colour 5 | `color-mix()` (multi-colour mix in Firefox 150), relative colour syntax, `light-dark()` (images in Firefox 150), `@color-profile` and `device-cmyk()` (unshipped). | [css-color-5 (WD)](https://drafts.csswg.org/css-color-5/) | Y / Y / Y (core); `@color-profile` N/N/N | Medium |
| CG-3.4 | `contrast-color()` | Picks black or white for contrast. Interop 2026 focus area. | [css-color-5](https://drafts.csswg.org/css-color-5/#contrast-color) | partial / Y / Y | Low |
| CG-3.5 | Colour HDR | `dynamic-range-limit`, HDR colour spaces and headroom in CSS. | [css-color-hdr-1 (WD)](https://drafts.csswg.org/css-color-hdr/) | partial / N / partial | Low |
| CG-3.6 | Colour adjustment | `color-scheme`, `forced-color-adjust`, `print-color-adjust`, forced colours mode, system colours. | [css-color-adjust-1 (CRD)](https://www.w3.org/TR/css-color-adjust-1/) | Y / Y / partial (forced colours) | Medium |
| CG-3.7 | Backgrounds and borders 3 | Multiple backgrounds, `background-clip`/`origin`/`size`, `border-radius`, `border-image`, `box-shadow`. | [css-backgrounds-3 (CRD)](https://www.w3.org/TR/css-backgrounds-3/) | Y / Y / Y | High |
| CG-3.8 | Backgrounds 4 | `background-clip: text`/`border-area`, `background-position-x/y`, logical background positions. | [css-backgrounds-4 (FPWD)](https://drafts.csswg.org/css-backgrounds-4/) | partial / partial / partial | Low |
| CG-3.9 | Borders and box decorations 4 | `corner-shape` (squircle, bevel, notch), `border-shape`, partial borders, `box-shadow` longhands. | [css-borders-4 (WD)](https://drafts.csswg.org/css-borders-4/) | partial / N / N — `corner-shape` single-engine | Low |
| CG-3.10 | Gap decorations | Rules between grid, flex and masonry tracks (`column-rule`/`row-rule` on any gap). | [css-gaps-1 (WD)](https://drafts.csswg.org/css-gaps-1/) | partial / N / N | Low |
| CG-3.11 | Images 3 | Linear and radial gradients, `object-fit`, `object-position`, `image-orientation`, `image-rendering`. | [css-images-3 (CR)](https://www.w3.org/TR/css-images-3/) | Y / Y / Y | Medium |
| CG-3.12 | Images 4 | `conic-gradient()`, repeating gradients, gradient colour-space interpolation, `image-set()`, `cross-fade()` (prefixed), `element()` (Gecko prefixed only). | [css-images-4 (WD)](https://drafts.csswg.org/css-images-4/) | Y / Y / Y (gradients, image-set); `cross-fade` partial / N / partial; `element()` N / partial / N | Medium |
| CG-3.13 | `object-view-box` | Crops a replaced element to a view box. | [css-images-5](https://drafts.csswg.org/css-images-5/) | Y / N / N — single-engine | Low |
| CG-3.14 | Image animation control | `image-animation: normal/paused/stopped/running` for animated images. | [css-image-animation-1 (FPWD)](https://drafts.csswg.org/css-image-animation-1/) | N / N / N | Medium |
| CG-3.15 | Fill and stroke 3 | `fill`/`stroke` paint properties for CSS boxes and text, beyond SVG. | [fill-stroke-3 (FPWD)](https://drafts.csswg.org/fill-stroke-3/) | N / N / N (SVG subset Y) | Medium |
| CG-3.16 | Legacy `-webkit-` features | `-webkit-text-stroke`, `-webkit-text-fill-color`, `-webkit-box-reflect` (Chromium/WebKit), `-webkit-line-clamp`, `-webkit-box`, `-webkit-mask-*`, prefixed aliases required for web compat. | [Compat Standard (WHATWG)](https://compat.spec.whatwg.org/) | Y / Y / Y (most); box-reflect N in Gecko | Medium |

**Implications:** Colour is a pipeline decision, not a property list: Colour 4/5, HDR and colour management (group 11) all depend on one wide-gamut, float compositing design. The WHATWG Compat standard is a must-ship list that no CSS module covers.

## 4. CSS: box model, display and layout modes

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-4.1 | Box model 3 | Margins, padding, borders. | [css-box-3 (REC)](https://www.w3.org/TR/css-box-3/) | Y / Y / Y | High |
| CG-4.2 | Box model 4 | `margin-trim`. | [css-box-4 (WD)](https://drafts.csswg.org/css-box-4/) | N / N / Y — single-engine | Medium |
| CG-4.3 | Box sizing 3 | Intrinsic sizing, `min-content`/`max-content`/`fit-content`, `box-sizing`. | [css-sizing-3 (WD)](https://www.w3.org/TR/css-sizing-3/) | Y / Y / Y | Medium |
| CG-4.4 | Box sizing 4 | `aspect-ratio`, `contain-intrinsic-size`, `stretch` keyword, `interpolate-size`. | [css-sizing-4 (WD)](https://drafts.csswg.org/css-sizing-4/) | Y / Y / Y (aspect-ratio, contain-intrinsic-size); others partial | Medium |
| CG-4.5 | Display 3 | Two-value `display`, `contents`, `flow-root`, `list-item`, `ruby`, anonymous box rules, `visibility`. | [css-display-3 (CR)](https://www.w3.org/TR/css-display-3/) | Y / Y / Y | Medium |
| CG-4.6 | Display 4 | `reading-flow`/`reading-order` (focus and reading order follow visual order), `display` animation. | [css-display-4 (WD)](https://drafts.csswg.org/css-display-4/) | Y / N / N — single-engine | Low |
| CG-4.7 | Flexbox | Flexible box layout. | [css-flexbox-1 (CR)](https://www.w3.org/TR/css-flexbox-1/) | Y / Y / Y | High |
| CG-4.8 | Grid 1 | Grid layout, named lines and areas, auto-placement. | [css-grid-1 (CRD)](https://www.w3.org/TR/css-grid-1/) | Y / Y / Y | High |
| CG-4.9 | Grid 2 (subgrid) | `subgrid` for rows and columns. | [css-grid-2 (CRD)](https://www.w3.org/TR/css-grid-2/) | Y / Y / Y | Medium |
| CG-4.10 | Grid 3 (Grid Lanes / masonry) | `display: grid-lanes`, `item-tolerance`. WebKit ships it in Safari 26.4; Chromium and Gecko only behind flags (snippets). | [css-grid-3 (WD)](https://drafts.csswg.org/css-grid-3/) | flag / flag / Y — single-engine | Medium |
| CG-4.11 | Box alignment 3 | `justify-*`, `align-*`, `place-*`, `gap` across block, flex, grid, abs-pos; `align-content` in block layout. | [css-align-3 (WD)](https://www.w3.org/TR/css-align-3/) | Y / Y / Y | Medium |
| CG-4.12 | Multi-column 1 | `columns`, `column-span`, `column-fill`, column rules. | [css-multicol-1 (CR)](https://www.w3.org/TR/css-multicol-1/) | Y / Y / Y | High |
| CG-4.13 | Multi-column 2 | `column-wrap`, `column-height`, `::column` (used by CSS carousels). | [css-multicol-2 (FPWD)](https://drafts.csswg.org/css-multicol-2/) | partial / N / N | Low |
| CG-4.14 | Tables 3 | Table layout algorithm (fixed and auto), border collapsing, `visibility: collapse`. | [css-tables-3 (WD)](https://drafts.csswg.org/css-tables-3/) | Y / Y / Y | Medium |
| CG-4.15 | Inline layout 3 | `text-box-trim`/`text-box-edge`, `initial-letter`, `dominant-baseline`, `line-height` model. | [css-inline-3 (WD)](https://drafts.csswg.org/css-inline-3/) | Y / N / Y (text-box); `initial-letter` partial | Medium |
| CG-4.16 | Positioned layout 3 | `relative`, `absolute`, `fixed`, `sticky`, inset properties. | [css-position-3 (WD)](https://www.w3.org/TR/css-position-3/) | Y / Y / Y | High |
| CG-4.17 | Positioned layout 4 (top layer) | Top layer rendering for dialogs, popovers and fullscreen; `overlay` property; `::backdrop`. | [css-position-4 (WD)](https://drafts.csswg.org/css-position-4/) | Y / Y / Y | Medium |
| CG-4.18 | Anchor positioning 1 | `anchor-name`, `position-anchor`, `anchor()`, `anchor-size()`, `position-area`, `position-try`, `position-visibility`. Interop 2025–2026. Firefox 147 shipped it (release notes snippet); Safari 26.0. | [css-anchor-position-1 (WD)](https://www.w3.org/TR/css-anchor-position-1/) | Y / Y / Y | Medium |
| CG-4.19 | Anchor positioning 2 | Anchored container queries and later features. | [css-anchor-position-2 (FPWD)](https://drafts.csswg.org/css-anchor-position-2/) | partial / N / N | Low |
| CG-4.20 | Overflow 3 | `overflow-x/y`, `overflow: clip`, `overflow-clip-margin`, `scrollbar-gutter`, logical overflow (Safari 26). | [css-overflow-3 (WD)](https://www.w3.org/TR/css-overflow-3/) | Y / Y / Y | Medium |
| CG-4.21 | Overflow 4 | `line-clamp`, `block-ellipsis`, `max-lines`, `continue`. Only `-webkit-line-clamp` is interoperable. | [css-overflow-4 (WD)](https://drafts.csswg.org/css-overflow-4/) | partial / partial / partial | Medium |
| CG-4.22 | Overflow 5 (CSS carousels) | `::scroll-marker`, `::scroll-marker-group`, `::scroll-button()`, `scroll-target-group`. | [css-overflow-5 (FPWD)](https://drafts.csswg.org/css-overflow-5/) | Y / N / N — single-engine | Low |
| CG-4.23 | Logical properties | Flow-relative margins, padding, borders, insets, sizes. | [css-logical-1 (WD)](https://www.w3.org/TR/css-logical-1/) | Y / Y / Y | Medium |
| CG-4.24 | Writing modes 3/4 | `writing-mode`, `direction`, `unicode-bidi`, `text-orientation`, `text-combine-upright`, `sideways-*`. | [css-writing-modes-4 (CR)](https://www.w3.org/TR/css-writing-modes-4/) | Y / Y / Y (`sideways-*` partial) | Medium |
| CG-4.25 | Containment 1/2 | `contain` (size, layout, paint, style, inline-size), `content-visibility`. | [css-contain-1 (REC)](https://www.w3.org/TR/css-contain-1/), [css-contain-2](https://www.w3.org/TR/css-contain-2/) | Y / Y / Y | Medium |
| CG-4.26 | `will-change` | Hints that create compositing layers early. | [css-will-change-1 (CRD)](https://www.w3.org/TR/css-will-change-1/) | Y / Y / Y | Medium |
| CG-4.27 | Shapes 1 | `shape-outside`, `shape-margin`, `shape-image-threshold` for floats; basic shapes. | [css-shapes-1 (CRD)](https://www.w3.org/TR/css-shapes-1/) | Y / Y / Y | Medium |
| CG-4.28 | `shape()` function | Responsive path shapes for `clip-path` and `shape-outside`. Interop 2026 focus area. | [css-shapes-2](https://drafts.csswg.org/css-shapes-2/) | Y / partial / Y | Low |
| CG-4.29 | Exclusions | `wrap-flow`, `wrap-through`. Only legacy EdgeHTML/IE shipped it. | [css-exclusions-1 (WD)](https://www.w3.org/TR/css3-exclusions/) | N / N / N (dep. with IE) | Medium |
| CG-4.30 | Regions | `flow-into`/`flow-from`. WebKit removed its implementation. | [css-regions-1 (WD)](https://www.w3.org/TR/css-regions-1/) | N / N / dep. | Medium |
| CG-4.31 | Page floats, line grid, rhythmic sizing, template layout, round display | Exploratory modules with no shipping engine. | [css-page-floats-3](https://drafts.csswg.org/css-page-floats-3/), [css-line-grid-1](https://drafts.csswg.org/css-line-grid/), [css-rhythm-1](https://drafts.csswg.org/css-rhythm/), [css-template-1 (NOTE)](https://www.w3.org/TR/css-template-3/), [css-round-display-1](https://www.w3.org/TR/css-round-display-1/) | N / N / N | Medium |
| CG-4.32 | CSS Layout API | Houdini custom layout worklets. | [css-layout-api-1 (FPWD)](https://drafts.css-houdini.org/css-layout-api-1/) | flag / N / N | Medium |
| CG-4.33 | Quirks mode | Required rendering differences for no-doctype pages (line height, table sizing, body height, unitless lengths). | [Quirks Mode Standard (WHATWG)](https://quirks.spec.whatwg.org/) | Y / Y / Y | Medium |

**Implications:** Layout is three engines' worth of modes (block, inline, flex, grid, table, multicol, abs-pos, anchor, grid-lanes) and every one is shipped. Grid Lanes became shipping reality in WebKit in 2026, so the layout spec should reserve room for it now.

## 5. CSS: scrolling, fragmentation and paged media

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-5.1 | Scroll snap 1 | `scroll-snap-type`, `scroll-snap-align`, `scroll-padding`/`margin`. Interop 2026 focus area. | [css-scroll-snap-1 (CR)](https://www.w3.org/TR/css-scroll-snap-1/) | Y / Y / Y | Medium |
| CG-5.2 | Scroll snap 2 | `scrollsnapchange`/`scrollsnapchanging` events, `scroll-start`. | [css-scroll-snap-2 (FPWD)](https://drafts.csswg.org/css-scroll-snap-2/) | Y / N / N — single-engine | Low |
| CG-5.3 | Scroll anchoring | `overflow-anchor`, keeping content stable when content above changes. | [css-scroll-anchoring-1 (WD)](https://drafts.csswg.org/css-scroll-anchoring/) | Y / Y / N | Medium |
| CG-5.4 | Overscroll behaviour | `overscroll-behavior` (scroll chaining, bounce, pull-to-refresh). | [css-overscroll-1 (FPWD)](https://drafts.csswg.org/css-overscroll-1/) | Y / Y / Y | Medium |
| CG-5.5 | Scrollbar styling | `scrollbar-width`, `scrollbar-color`; legacy `::-webkit-scrollbar` pseudo-elements. | [css-scrollbars-1 (CR)](https://www.w3.org/TR/css-scrollbars-1/) | Y / Y / partial; `::-webkit-scrollbar` Chromium/WebKit | Low |
| CG-5.6 | Smooth scrolling and scroll APIs | `scroll-behavior`, `scrollIntoView` options, `scrollend` event, visual viewport. | [cssom-view-1](https://drafts.csswg.org/cssom-view/), [Visual Viewport](https://wicg.github.io/visual-viewport/) | Y / Y / Y (`scrollend` WebKit partial) | Medium |
| CG-5.7 | Spatial navigation | Arrow-key focus navigation by geometry. | [css-nav-1 (WD)](https://drafts.csswg.org/css-nav-1/) | flag / N / N | Medium |
| CG-5.8 | Fragmentation 3 | `break-before`/`after`/`inside`, `orphans`, `widows`, `box-decoration-break`. | [css-break-3 (CR)](https://www.w3.org/TR/css-break-3/) | Y / Y / partial (`box-decoration-break` prefixed in WebKit) | Medium |
| CG-5.9 | Fragmentation 4 | `margin-break`, region-style fragmentation details. | [css-break-4 (FPWD)](https://drafts.csswg.org/css-break-4/) | N / N / N | Medium |
| CG-5.10 | Paged media 3 | `@page`, page size and margins, page selectors, named pages (`page`), page-margin boxes (`@top-center` etc.). | [css-page-3 (WD)](https://www.w3.org/TR/css-page-3/) | Y / partial / partial; margin boxes Chromium only (recalled) | Low |
| CG-5.11 | Generated content for paged media | Running headers, `string-set`, `target-counter()`, footnotes, leaders. | [css-gcpm-3 (WD)](https://www.w3.org/TR/css-gcpm-3/) | N / N / N (print formatters only) | Medium |

**Implications:** Scrolling is split across CSS and CSSOM View and has single-engine pieces. Paged media is weakly implemented everywhere, which matters because Izcal's print path (group 16) must fragment into pages.

## 6. CSS: text, fonts, lists and generated content

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-6.1 | Text 3 | `white-space`, `text-transform`, `tab-size`, `word-break`, `line-break`, `overflow-wrap`, `text-align`/`-last`, `text-justify`, `text-indent`, `hyphens`, `letter-spacing`, `word-spacing`, `hanging-punctuation` (WebKit only). | [css-text-3 (CRD)](https://www.w3.org/TR/css-text-3/) | Y / Y / Y (core) | Medium |
| CG-6.2 | Text 4 | `text-wrap: balance/pretty/stable`, `white-space-collapse`, `text-wrap-mode`, `text-spacing-trim`, `text-autospace`, `hyphenate-character`, `hyphenate-limit-chars`, `word-space-transform`. | [css-text-4 (WD)](https://drafts.csswg.org/css-text-4/) | Y / partial / Y (`balance` Y everywhere; `pretty` not in Gecko) | Low |
| CG-6.3 | Text decoration 3 | `text-decoration-*`, `text-emphasis`, `text-shadow`, `text-underline-position`. | [css-text-decor-3 (CRD)](https://www.w3.org/TR/css-text-decor-3/) | Y / Y / Y | Medium |
| CG-6.4 | Text decoration 4 | `text-decoration-thickness`, `text-underline-offset`, `text-decoration-skip-ink`, spelling and grammar decorations. | [css-text-decor-4 (WD)](https://drafts.csswg.org/css-text-decor-4/) | Y / Y / Y | Medium |
| CG-6.5 | Text size adjust | `text-size-adjust` on mobile. | [css-size-adjust-1](https://drafts.csswg.org/css-size-adjust/) | partial / N / partial (prefixed) | Medium |
| CG-6.6 | Fonts 3 | `@font-face`, `font-*` properties, font matching, `unicode-range`, `font-feature-settings`. | [css-fonts-3 (REC)](https://www.w3.org/TR/css-fonts-3/) | Y / Y / Y | High |
| CG-6.7 | Fonts 4 | Variable fonts (`font-variation-settings`, `font-optical-sizing`), colour fonts (`font-palette`, `@font-palette-values`), `font-display`, `size-adjust`/`ascent-override` descriptors, `font-size-adjust`, `font-synthesis-*`, generic families (`system-ui`, `ui-*`, `math`, `emoji`, `fangsong`). | [css-fonts-4 (WD)](https://www.w3.org/TR/css-fonts-4/) | Y / Y / Y (most); `size-adjust` descriptors partial in WebKit | Medium |
| CG-6.8 | Fonts 5 | `font-size: math`, `font-size-adjust` extensions, `@font-feature-values` extensions, font tech queries. | [css-fonts-5 (WD)](https://drafts.csswg.org/css-fonts-5/) | partial / partial / partial | Low |
| CG-6.9 | Font loading API | `FontFace`, `document.fonts`, `FontFaceSet` events; also in workers. | [css-font-loading-3 (WD)](https://www.w3.org/TR/css-font-loading-3/) | Y / Y / Y | Medium |
| CG-6.10 | Web font formats | TrueType/OpenType, WOFF 1.0, WOFF 2.0 (Brotli, RFC 7932). | [WOFF2 (REC)](https://www.w3.org/TR/WOFF2/), [OpenType (ISO/IEC 14496-22)](https://learn.microsoft.com/en-us/typography/opentype/spec/) | Y / Y / Y | Medium |
| CG-6.11 | Variable fonts | OpenType font variations (`fvar`, `gvar`, `avar` 2 partial), CFF2 outlines. | [OpenType spec](https://learn.microsoft.com/en-us/typography/opentype/spec/otvaroverview) | Y / Y / Y | Medium |
| CG-6.12 | Colour font formats | COLRv0/CPAL (all), COLRv1 (Chromium, Gecko), OpenType-SVG (Gecko, WebKit), `sbix` (WebKit), `CBDT/CBLC` (Chromium). No single format is in all three engines beyond COLRv0. | [OpenType COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr), [SVG table](https://learn.microsoft.com/en-us/typography/opentype/spec/svg) | partial / partial / partial | Medium |
| CG-6.13 | Text shaping and layout tables | OpenType Layout (`GSUB`, `GPOS`, `GDEF`, `BASE`, `MATH`), AAT `morx`/`kerx` (WebKit relies on it), complex-script shaping, Graphite (Gecko only). | [OpenType Layout](https://learn.microsoft.com/en-us/typography/opentype/spec/ttochap1) | Y / Y / Y | Medium |
| CG-6.14 | Unicode text algorithms | Bidi (UAX #9), line breaking (UAX #14), segmentation (UAX #29), vertical orientation (UAX #50), East Asian width, emoji sequences (UTS #51). | [Unicode Standard Annexes](https://www.unicode.org/reports/) | Y / Y / Y | Medium |
| CG-6.15 | Hyphenation dictionaries | Language-specific hyphenation for `hyphens: auto` across about 30+ languages in Gecko, fewer in Chromium. The commonly used TeX patterns carry mixed licences. | [css-text-3 §hyphens](https://www.w3.org/TR/css-text-3/#hyphenation) | Y / Y / Y (language sets differ) | Low |
| CG-6.16 | Incremental Font Transfer | Patch-based font streaming. | [IFT (WD)](https://www.w3.org/TR/IFT/) | N / N / N | Medium |
| CG-6.17 | Local Font Access | `queryLocalFonts()` to enumerate installed fonts. | [Local Font Access (WICG)](https://wicg.github.io/local-font-access/) | Y / N / N — single-engine | Medium |
| CG-6.18 | Ruby layout | `ruby-position`, `ruby-align`, `ruby-merge`, `ruby-overhang`; HTML ruby extensions (`rtc`). | [css-ruby-1 (WD)](https://www.w3.org/TR/css-ruby-1/), [HTML Ruby Markup Extensions](https://www.w3.org/TR/html-ruby-extensions/) | partial / Y / partial | Low |
| CG-6.19 | Lists and counters 3 | `::marker`, `list-style-*`, `counter-*`, `counter-set`, `list-item` counter, `marker-side` (unshipped). | [css-lists-3 (WD)](https://www.w3.org/TR/css-lists-3/) | Y / Y / Y | Medium |
| CG-6.20 | Counter styles 3 | `@counter-style`, `symbols()`, predefined styles. | [css-counter-styles-3 (CR)](https://www.w3.org/TR/css-counter-styles-3/), [Predefined Counter Styles (NOTE)](https://www.w3.org/TR/predefined-counter-styles/) | Y / Y / Y (`symbols()` Gecko only) | Medium |
| CG-6.21 | Generated content 3 | `content` (strings, images, alt text after `/`, counters, quotes, `attr()`), `quotes`, `content: element()` (unshipped). | [css-content-3 (WD)](https://drafts.csswg.org/css-content-3/) | Y / Y / Y (core) | Medium |
| CG-6.22 | Custom Highlight API | `CSS.highlights`, `Highlight`, `::highlight()`. Interop 2026 focus area. | [css-highlight-api-1 (WD)](https://www.w3.org/TR/css-highlight-api-1/) | Y / Y / Y | Medium |

**Implications:** Fonts and text need large non-CSS infrastructure (OpenType parsing, shaping, Unicode algorithms, colour-font rasterisers, hyphenation data). The hyphenation data licence question blocks "MIT only" unless Izcal generates its own patterns.

## 7. CSS: transforms, animation and visual effects

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-7.1 | Transforms 1 | 2D transforms, `transform-origin`, `transform-box`. | [css-transforms-1 (CR)](https://www.w3.org/TR/css-transforms-1/) | Y / Y / Y | High |
| CG-7.2 | Transforms 2 | 3D transforms, `perspective`, `backface-visibility`, `transform-style: preserve-3d`, individual `translate`/`rotate`/`scale`. | [css-transforms-2 (WD)](https://drafts.csswg.org/css-transforms-2/) | Y / Y / Y | Medium |
| CG-7.3 | Transitions 1 | `transition-*`, `transitionend` events. | [css-transitions-1 (WD)](https://www.w3.org/TR/css-transitions-1/) | Y / Y / Y | Medium |
| CG-7.4 | Transitions 2 | `@starting-style`, `transition-behavior: allow-discrete`, `CSSTransition` objects. | [css-transitions-2 (WD)](https://drafts.csswg.org/css-transitions-2/) | Y / Y / Y | Medium |
| CG-7.5 | Animations 1 | `@keyframes`, `animation-*`. | [css-animations-1 (WD)](https://www.w3.org/TR/css-animations-1/) | Y / Y / Y | High |
| CG-7.6 | Animations 2 | `animation-composition`, `CSSAnimation` objects, `animation-trigger` (scroll-triggered animations, Chrome 146 per snippet). | [css-animations-2 (WD)](https://drafts.csswg.org/css-animations-2/) | Y / Y / Y (composition); triggers Y / N / N | Low |
| CG-7.7 | Web Animations 1 | `Element.animate()`, `Animation`, `KeyframeEffect`, `document.timeline`, `getAnimations()`. | [web-animations-1 (WD)](https://www.w3.org/TR/web-animations-1/) | Y / Y / Y | Medium |
| CG-7.8 | Web Animations 2 | Group and sequence effects, `iterationComposite` refinements. | [web-animations-2 (WD)](https://drafts.csswg.org/web-animations-2/) | partial / partial / partial | Low |
| CG-7.9 | Scroll-driven animations | `scroll-timeline`, `view-timeline`, `animation-timeline`, `animation-range`, `ScrollTimeline`/`ViewTimeline`. Interop 2026 focus area. Safari 26.0 shipped it (snippet). | [scroll-animations-1 (WD)](https://drafts.csswg.org/scroll-animations-1/) | Y / flag / Y | Medium |
| CG-7.10 | Animation Worklet | Houdini off-main-thread scripted animations. | [css-animation-worklet-1 (FPWD)](https://drafts.css-houdini.org/css-animationworklet-1/) | flag / N / N | Medium |
| CG-7.11 | Motion path | `offset-path`, `offset-distance`, `offset-rotate`, `offset-anchor`. | [motion-1 (WD)](https://www.w3.org/TR/motion-1/) | Y / Y / Y | Medium |
| CG-7.12 | View transitions 1 (same-document) | `document.startViewTransition()`, `view-transition-name`, `::view-transition-*` pseudo-elements. Interop 2026 focus area. | [css-view-transitions-1 (CR)](https://www.w3.org/TR/css-view-transitions-1/) | Y / Y / Y | Medium |
| CG-7.13 | View transitions 2 | Cross-document transitions (`@view-transition`), `view-transition-class`, transition types, `:active-view-transition`, `pageswap`/`pagereveal`, `rel="expect"`/`blocking="render"`; element-scoped transitions. Interop 2026 focus area. | [css-view-transitions-2 (WD)](https://drafts.csswg.org/css-view-transitions-2/) | Y / N / Y (cross-document) | Medium |
| CG-7.14 | Filter effects 1 | `filter` functions on HTML, SVG `<filter>` primitives, `filter: url()` on HTML. | [filter-effects-1 (WD)](https://www.w3.org/TR/filter-effects-1/) | Y / Y / Y (`url()` filters on HTML partial in WebKit) | Medium |
| CG-7.15 | Filter effects 2 | `backdrop-filter`. | [filter-effects-2](https://drafts.fxtf.org/filter-effects-2/) | Y / Y / Y | Medium |
| CG-7.16 | Compositing and blending | `mix-blend-mode`, `background-blend-mode`, `isolation`, Porter-Duff operators. | [compositing-1 (CR)](https://www.w3.org/TR/compositing-1/) | Y / Y / Y | High |
| CG-7.17 | Masking | `clip-path` (basic shapes, `path()`, `url()`), `mask-*` (unprefixed in all), `mask-border` (prefixed). | [css-masking-1 (CRD)](https://www.w3.org/TR/css-masking-1/) | Y / Y / Y | Medium |
| CG-7.18 | Geometry interfaces | `DOMMatrix`, `DOMPoint`, `DOMRect`, `DOMQuad`. | [geometry-1 (CR)](https://www.w3.org/TR/geometry-1/) | Y / Y / Y | High |

**Implications:** Animation and effects drive the compositor design: scroll-driven and view transitions both need off-main-thread timing and snapshotting, and they are all three-engine or nearly so in 2026.

## 8. CSS: object models, Houdini and observers

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-8.1 | CSSOM | `CSSStyleSheet`, `CSSRule` tree, `getComputedStyle()`, `element.style`, constructable style sheets and `adoptedStyleSheets`. | [cssom-1 (WD)](https://www.w3.org/TR/cssom-1/) | Y / Y / Y | Medium |
| CG-8.2 | CSSOM View | `getBoundingClientRect`, `getClientRects`, scroll APIs, `elementFromPoint`, `caretPositionFromPoint`, `checkVisibility()`, `matchMedia`, `screen`, `devicePixelRatio`, offset/client/scroll metrics. | [cssom-view-1 (WD)](https://www.w3.org/TR/cssom-view-1/) | Y / Y / Y (`caretPositionFromPoint` now all, recalled) | Medium |
| CG-8.3 | CSS Typed OM | `CSSStyleValue`, `attributeStyleMap`, `computedStyleMap()`, `CSSNumericValue` math. | [css-typed-om-1 (WD)](https://www.w3.org/TR/css-typed-om-1/) | Y / N / Y | Medium |
| CG-8.4 | CSS Painting API | `registerPaint()` paint worklets, `paint()` images. | [css-paint-api-1 (CRD)](https://www.w3.org/TR/css-paint-api-1/) | Y / N / N — single-engine | Medium |
| CG-8.5 | Worklets | The worklet global scope model used by paint, audio, layout and animation worklets. | [HTML Worklets](https://html.spec.whatwg.org/multipage/worklets.html) | Y / Y / Y | Medium |
| CG-8.6 | CSS parser API | Proposed JS access to the CSS parser. | [css-parser-api (unofficial)](https://wicg.github.io/css-parser-api/) | N / N / N | Low |
| CG-8.7 | Resize Observer | Callbacks on element box size changes (content, border, device-pixel boxes). | [resize-observer-1 (WD)](https://www.w3.org/TR/resize-observer-1/) | Y / Y / Y | Medium |
| CG-8.8 | Intersection Observer | Visibility of an element relative to a root, with thresholds and margins; v2 occlusion tracking (`trackVisibility`) is Chromium only. | [IntersectionObserver (WD)](https://www.w3.org/TR/intersection-observer/) | Y / Y / Y; v2 single-engine | Medium |
| CG-8.9 | Rendering timing hooks | `requestAnimationFrame`, the "update the rendering" steps that run style, layout, observers, animations and paint in order. | [HTML event loop](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering) | Y / Y / Y | Medium |

**Implications:** The update-the-rendering step order (CG-8.9) is a cross-cutting contract that observers, animations and view transitions all hang off; it should have one owning spec.

## 9. CSS and HTML: forms, UI and interactivity

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-9.1 | Basic UI 3 | `box-sizing`, `outline`, `cursor`, `caret-color`, `resize`, `text-overflow`. | [css-ui-3 (REC)](https://www.w3.org/TR/css-ui-3/) | Y / Y / Y | High |
| CG-9.2 | Basic UI 4 | `appearance`, `accent-color`, `user-select`, `pointer-events`, `caret-shape`, `caret-animation`, `nav-*`, `field-sizing`. | [css-ui-4 (WD)](https://drafts.csswg.org/css-ui-4/) | Y / Y / Y (appearance, accent-color, user-select); `field-sizing` Y / N / Y (recalled) | Low |
| CG-9.3 | Customisable form controls | `appearance: base-select`, `::picker(select)`, `::picker-icon`, `::checkmark`, `<selectedcontent>`; the CSS Forms draft. | [css-forms-1](https://drafts.csswg.org/css-forms-1/), [HTML select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element) | Y / N / N — single-engine | Low |
| CG-9.4 | Native form control rendering | Platform-themed rendering of every `<input>` type, `<select>`, `<progress>`, `<meter>`, date and colour pickers, `<details>` disclosure marker. A rendering obligation even though HTML defines the elements. | [HTML rendering §15](https://html.spec.whatwg.org/multipage/rendering.html) | Y / Y / Y | Medium |
| CG-9.5 | HTML rendering section | The UA style sheet and presentational-hint mapping (`<font>`, `bgcolor`, `align`, `<center>`, tables, `hidden`). | [HTML rendering §15](https://html.spec.whatwg.org/multipage/rendering.html) | Y / Y / Y | Medium |
| CG-9.6 | `inert` | Inert subtrees: not focusable, not hit-testable, hidden from AT. | [HTML inert](https://html.spec.whatwg.org/multipage/interaction.html#inert-subtrees) | Y / Y / Y | Medium |
| CG-9.7 | `interactivity: inert` | CSS way to make subtrees inert. | [css-ui-4](https://drafts.csswg.org/css-ui-4/#inertness) | Y / N / N — single-engine (recalled) | Low |
| CG-9.8 | Forced colours and high contrast | Forced colours mode rendering, system colour keywords, `forced-color-adjust`. | [css-color-adjust-1](https://www.w3.org/TR/css-color-adjust-1/) | Y / Y / partial | Medium |
| CG-9.9 | Viewport and device environment | `<meta name=viewport>`, `interactive-widget`, Virtual Keyboard API, Visual Viewport API, Viewport Segments (foldables). | [css-viewport-1](https://drafts.csswg.org/css-viewport/), [Virtual Keyboard](https://www.w3.org/TR/virtual-keyboard/), [Viewport Segments](https://www.w3.org/TR/viewport-segments/) | partial / partial / partial (Virtual Keyboard and segments Chromium only) | Low |
| CG-9.10 | EyeDropper API | Pick a colour from the screen. | [EyeDropper (WICG)](https://wicg.github.io/eyedropper-api/) | Y / N / N — single-engine | Medium |
| CG-9.11 | Ink and pen rendering | Delegated ink trail (low-latency pen strokes drawn by the compositor). | [Ink API (WICG)](https://wicg.github.io/ink-enhancement/) | Y / N / N — single-engine | Medium |

**Implications:** Form-control rendering (CG-9.4, CG-9.5) is often missed in CSS inventories because HTML defines it, yet it is a large chunk of rendering work.

## 10. SVG and MathML

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-10.1 | SVG 1.1 | Shapes, paths, text, paint servers, clipping, masking, filters, markers, `<use>`, `<foreignObject>`, `<switch>`, DOM interfaces. | [SVG 1.1 2nd ed. (REC)](https://www.w3.org/TR/SVG11/) | Y / Y / Y | High |
| CG-10.2 | SVG 2 | Geometry properties as CSS, `paint-order`, `vector-effect`, `href` without xlink, `<mesh>`/`<hatch>` (unshipped), new DOM. | [SVG 2 (CR)](https://www.w3.org/TR/SVG2/) | partial / partial / partial | Medium |
| CG-10.3 | SVG animation (SMIL) | `<animate>`, `<animateTransform>`, `<animateMotion>`, `<set>`, timing model. | [SVG Animations](https://svgwg.org/specs/animations/), [SMIL 3.0](https://www.w3.org/TR/SMIL3/) | Y / Y / Y | Medium |
| CG-10.4 | SVG integration and image modes | SVG as `<img>`/CSS image (secure animated mode, no script), as document, in `<iframe>`/`<object>`, inline in HTML. | [SVG Integration](https://svgwg.org/specs/integration/) | Y / Y / Y | Medium |
| CG-10.5 | SVG fonts | `<font>`/`<glyph>` in SVG. Deprecated. | [SVG 1.1 fonts](https://www.w3.org/TR/SVG11/fonts.html) | N / N / dep. | Medium |
| CG-10.6 | SVG markers and strokes | `marker-*`, stroke dashing and joins, SVG Strokes module (`stroke-dash-corner`, etc. unshipped). | [SVG Strokes](https://svgwg.org/specs/strokes/) | partial / partial / partial | Low |
| CG-10.7 | MathML Core | Math layout: `<mi>`, `<mfrac>`, `<msqrt>`, `<mtable>`, `<mmultiscripts>`, operator dictionary, OpenType MATH table use. | [MathML Core (CRD)](https://www.w3.org/TR/mathml-core/) | Y / Y / Y | Medium |
| CG-10.8 | MathML 4 full | Content MathML and presentation attributes outside Core (`mfenced`, `menclose` partial). | [MathML 4 (WD)](https://www.w3.org/TR/mathml4/) | N / partial / partial | Medium |

**Implications:** SVG needs its own rendering tree and animation clock beside CSS; MathML Core is three-engine since 2023 and needs the OpenType MATH table in the font stack.

## 11. 2D and 3D graphics APIs

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-11.1 | Canvas 2D | `CanvasRenderingContext2D`: paths, text, images, compositing, pixel access, `roundRect`, `reset`, `filter`, `letterSpacing`, `willReadFrequently`, `colorSpace: display-p3`. | [HTML canvas](https://html.spec.whatwg.org/multipage/canvas.html) | Y / Y / Y (newer attributes partial) | Medium |
| CG-11.2 | Path2D | Reusable path objects, SVG path strings, `addPath()`. | [HTML canvas](https://html.spec.whatwg.org/multipage/canvas.html#path2d-objects) | Y / Y / Y | Medium |
| CG-11.3 | OffscreenCanvas | Canvas in workers, `transferControlToOffscreen()`, 2D and WebGL contexts. | [HTML OffscreenCanvas](https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface) | Y / Y / Y | Medium |
| CG-11.4 | ImageBitmap | `createImageBitmap()`, transferable decoded images, `ImageBitmapRenderingContext`. | [HTML ImageBitmap](https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html) | Y / Y / Y | Medium |
| CG-11.5 | Canvas HDR and float pixels | `float16` canvas storage, extended-range colour, HDR canvas output. | [WHATWG proposal (prose)](https://github.com/whatwg/html/issues/9461) | flag / N / partial | Low |
| CG-11.6 | HTML-in-canvas | Drawing live DOM elements into a canvas (`drawElement`/`placeElement`). | [WICG explainer (prose)](https://github.com/WICG/html-in-canvas) | flag / N / N | Low |
| CG-11.7 | WebGL 1.0 | OpenGL ES 2.0-based 3D API. | [WebGL 1.0 (Khronos)](https://registry.khronos.org/webgl/specs/latest/1.0/) | Y / Y / Y | High |
| CG-11.8 | WebGL 2.0 | OpenGL ES 3.0-based 3D API (3D textures, transform feedback, UBOs, instancing). | [WebGL 2.0 (Khronos)](https://registry.khronos.org/webgl/specs/latest/2.0/) | Y / Y / Y | High |
| CG-11.9 | WebGL extensions | The ratified and community extensions registry (compressed textures S3TC/ETC/ASTC/BPTC, float textures, `OES_draw_buffers_indexed`, multi-draw, `WEBGL_lose_context`). Engines expose different subsets per GPU. | [WebGL extension registry](https://registry.khronos.org/webgl/extensions/) | partial / partial / partial | Medium |
| CG-11.10 | WebGPU | Modern GPU API (adapters, devices, pipelines, compute, bind groups, canvas context). Firefox 141 on Windows, 147 on Apple Silicon macOS (snippet); Safari 26. | [WebGPU (CRD)](https://www.w3.org/TR/webgpu/) | Y / partial / Y | Medium |
| CG-11.11 | WGSL | The WebGPU shading language. Izcal needs its own compiler to native GPU languages. | [WGSL (CRD)](https://www.w3.org/TR/WGSL/) | Y / partial / Y | Medium |
| CG-11.12 | WebGPU optional features | `shader-f16`, subgroups, timestamp queries, texture compression features, compatibility mode (`featureLevel`), WebGPU in workers. | [WebGPU features](https://www.w3.org/TR/webgpu/#feature-index) | partial / partial / partial | Low |
| CG-11.13 | Shape Detection | `BarcodeDetector`, `FaceDetector`, `TextDetector`. | [Shape Detection (WICG)](https://wicg.github.io/shape-detection-api/) | partial / N / N — single-engine | Medium |
| CG-11.14 | Image decode and lazy decode | `img.decode()`, `decoding` attribute, `loading=lazy`. | [HTML img](https://html.spec.whatwg.org/multipage/embedded-content.html#dom-img-decode) | Y / Y / Y | Medium |

**Implications:** WebGPU and WGSL are now in all three engines on major platforms; Izcal needs both a WGSL compiler and a per-OS GPU backend (Metal, D3D12, Vulkan), which is large standalone work.

## 12. Image formats and colour management

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-12.1 | JPEG (baseline and progressive) | DCT image format, including progressive scans, arithmetic coding (partial), CMYK/YCCK and EXIF orientation. Core patents have expired (Medium). | [ITU-T T.81](https://www.w3.org/Graphics/JPEG/itu-t81.pdf), [JFIF](https://www.w3.org/Graphics/JPEG/jfif3.pdf) | Y / Y / Y | Medium |
| CG-12.2 | PNG (3rd edition) | Lossless raster, interlacing, gamma/`iCCP`/`sRGB`, plus `cICP` HDR signalling and APNG folded into the spec. Royalty-free by design. | [PNG 3rd ed. (W3C)](https://www.w3.org/TR/png-3/) | Y / Y / Y (`cICP` partial) | Medium |
| CG-12.3 | APNG | Animated PNG. | [PNG 3rd ed. (W3C)](https://www.w3.org/TR/png-3/) | Y / Y / Y | Medium |
| CG-12.4 | GIF | GIF89a, animation, transparency. LZW patents expired in 2003–2004 (Medium). | [GIF89a](https://www.w3.org/Graphics/GIF/spec-gif89a.txt) | Y / Y / Y | Medium |
| CG-12.5 | WebP | VP8-based lossy, lossless, alpha, animation. Google states a royalty-free patent grant (Medium). | [RFC 9649](https://www.rfc-editor.org/rfc/rfc9649) | Y / Y / Y | Medium |
| CG-12.6 | AVIF | AV1 still and animated images in HEIF container, HDR, alpha, grid tiling. Covered by the AOMedia royalty-free patent licence; a third-party pool (Sisvel) claims essential patents in AV1 (Medium). | [AVIF (AOM)](https://aomediacodec.github.io/av1-avif/) | Y / Y / Y | Medium |
| CG-12.7 | JPEG XL | ISO/IEC 18181 image format with lossless JPEG recompression, progressive decode, HDR. JPEG states a royalty-free baseline (Medium). Chrome announced default-on decoding from Chrome 155 on 6 October 2026 (snippet); Chrome 155 is not yet stable on 24 October 2026 by the four-week cadence (Low). Interop 2026 investigation area. | [JPEG XL (JPEG)](https://jpeg.org/jpegxl/) | flag (155 pending) / flag / Y | Medium |
| CG-12.8 | HEIC/HEIF | HEVC-coded images in HEIF. HEVC patents are licensed through several pools (Access Advance, Via LA) plus individual holders, so a royalty-free MIT decoder is not possible without licences (Medium). | [ISO/IEC 23008-12](https://www.iso.org/standard/83650.html) | N / N / Y — single-engine | Medium |
| CG-12.9 | TIFF | Tagged image file format in `<img>`. | [TIFF 6.0 (Adobe)](https://www.itu.int/itudoc/itu-t/com16/tiff-fx/docs/tiff6.pdf) | N / N / Y — single-engine | Medium |
| CG-12.10 | JPEG 2000 | Wavelet format. WebKit dropped it (recalled Safari 18). | [ITU-T T.800](https://www.itu.int/rec/T-REC-T.800) | N / N / dep. | Low |
| CG-12.11 | BMP, ICO, CUR | Bitmap, favicon and cursor formats. | [Microsoft BMP/ICO docs](https://learn.microsoft.com/en-us/windows/win32/gdi/bitmap-storage) | Y / Y / Y | Medium |
| CG-12.12 | SVG as image | SVG in `<img>`, CSS images, favicons; static and animated secure modes. | [SVG Integration](https://svgwg.org/specs/integration/) | Y / Y / Y | Medium |
| CG-12.13 | HDR gain-map images | Ultra HDR / ISO 21496-1 gain maps in JPEG (and AVIF), shown brighter on HDR displays. | [ISO 21496-1](https://www.iso.org/standard/86775.html) | Y / N / partial | Low |
| CG-12.14 | ICC colour management | Tagged-image colour management, ICC v2 and v4 profiles, untagged-as-sRGB, display profile conversion. | [ICC.1:2022](https://www.color.org/specification/ICC.1-2022-05.pdf) | Y / Y / Y | Medium |
| CG-12.15 | Wide-gamut and HDR output | Display-P3 and Rec.2020 content, PQ/HLG transfer, `dynamic-range` media queries, HDR video and image tone mapping. | [css-color-hdr-1](https://drafts.csswg.org/css-color-hdr/), [ITU-R BT.2100](https://www.itu.int/rec/R-REC-BT.2100) | Y / partial / Y | Low |
| CG-12.16 | Image density correction | Using EXIF resolution to set intrinsic size. | [HTML density-corrected size](https://html.spec.whatwg.org/multipage/images.html#density-corrected-intrinsic-width-and-height) | Y / N / Y | Low |
| CG-12.17 | Image MIME sniffing | Detecting image types from bytes regardless of `Content-Type`. | [MIME Sniffing (WHATWG)](https://mimesniff.spec.whatwg.org/) | Y / Y / Y | Medium |

**Implications:** Every image format in three engines (JPEG, PNG/APNG, GIF, WebP, AVIF, BMP/ICO, SVG) is royalty-free per its owner, so Izcal can write its own MIT decoders. JPEG XL is becoming three-engine. HEIC cannot be shipped MIT-only without HEVC licences.

## 13. Video and audio formats

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-13.1 | MP4 / ISO BMFF container | `.mp4`, `.m4a`, fragmented MP4 and CMAF for streaming. | [ISO/IEC 14496-12](https://www.iso.org/standard/83102.html) | Y / Y / Y | Medium |
| CG-13.2 | WebM / Matroska container | `.webm` with VP8, VP9, AV1, Vorbis, Opus. Royalty-free (WebM project prose, Medium). | [WebM spec](https://www.webmproject.org/docs/container/) | Y / Y / Y | Medium |
| CG-13.3 | Ogg container | `.ogg`/`.oga`/`.opus` with Vorbis, Opus, FLAC. | [RFC 3533](https://www.rfc-editor.org/rfc/rfc3533) | Y / Y / partial | Medium |
| CG-13.4 | HLS native playback | `.m3u8` playlists with MPEG-TS or fMP4 segments, played by `<video src>` without MSE. Safari and Chrome on Android; Chrome desktop status unclear. | [RFC 8216](https://www.rfc-editor.org/rfc/rfc8216) | partial / N / Y | Low |
| CG-13.5 | MPEG-DASH | Adaptive streaming manifests, played through MSE by script libraries; no engine plays DASH natively. | [ISO/IEC 23009-1](https://www.iso.org/standard/83314.html) | N / N / N (via MSE) | Medium |
| CG-13.6 | H.264 / AVC | Main video codec of the web. Licensed through the Via LA AVC/H.264 pool; Cisco distributes a royalty-paid OpenH264 binary for WebRTC (Medium). | [ITU-T H.264](https://www.itu.int/rec/T-REC-H.264) | Y / Y / Y | Medium |
| CG-13.7 | H.265 / HEVC | Successor to H.264, used for HDR and Apple capture. Pools: Access Advance, Via LA, plus non-pool holders (Medium). Chromium and Gecko use platform/hardware decoders only. | [ITU-T H.265](https://www.itu.int/rec/T-REC-H.265) | partial / partial / Y | Low |
| CG-13.8 | H.266 / VVC | Next MPEG codec. Pools exist; no browser ships it. | [ITU-T H.266](https://www.itu.int/rec/T-REC-H.266) | N / N / N | Medium |
| CG-13.9 | VP8 | Google's codec, mandatory in WebRTC. Royalty-free grant (Medium). | [RFC 6386](https://www.rfc-editor.org/rfc/rfc6386) | Y / Y / Y | Medium |
| CG-13.10 | VP9 | Google's codec; YouTube's main codec. Royalty-free grant (Medium). | [VP9 bitstream (WebM project)](https://www.webmproject.org/vp9/) | Y / Y / Y (WebKit partial by platform) | Medium |
| CG-13.11 | AV1 | AOMedia codec. AOMedia Patent License 1.0 is royalty-free; Sisvel runs a separate pool claiming AV1 patents (Medium). WebKit decodes only where hardware supports it. | [AV1 (AOM)](https://aomediacodec.github.io/av1-spec/) | Y / Y / partial | Medium |
| CG-13.12 | Theora | Xiph video codec. Deprecated and removed from Chromium (2024, recalled). | [Theora spec (Xiph)](https://www.theora.org/doc/Theora.pdf) | dep. / partial / N | Low |
| CG-13.13 | AAC (LC, HE-AAC) | Main lossy audio codec in MP4. Licensed through the Via LA AAC pool (Medium). | [ISO/IEC 14496-3](https://www.iso.org/standard/76383.html) | Y / Y / Y | Medium |
| CG-13.14 | MP3 | MPEG-1/2 Layer III. Fraunhofer and Technicolor ended the licensing programme in April 2017 (Medium). | [ISO/IEC 11172-3](https://www.iso.org/standard/22412.html) | Y / Y / Y | Medium |
| CG-13.15 | Opus | IETF audio codec for voice and music; mandatory in WebRTC. Royalty-free licences per opus-codec.org (Medium). | [RFC 6716](https://www.rfc-editor.org/rfc/rfc6716) | Y / Y / Y | Medium |
| CG-13.16 | Vorbis | Xiph lossy audio codec. Royalty-free. | [Vorbis I spec (Xiph)](https://xiph.org/vorbis/doc/Vorbis_I_spec.html) | Y / Y / partial | Medium |
| CG-13.17 | FLAC | Lossless audio. Royalty-free. | [RFC 9639](https://www.rfc-editor.org/rfc/rfc9639) | Y / Y / Y | Medium |
| CG-13.18 | PCM / WAV, G.711, G.722 | Uncompressed audio files; telephony codecs for WebRTC. | [RFC 2361](https://www.rfc-editor.org/rfc/rfc2361), [ITU-T G.711](https://www.itu.int/rec/T-REC-G.711) | Y / Y / Y | Medium |
| CG-13.19 | AC-3 / E-AC-3 / Dolby | Dolby audio and Dolby Vision video, via platform decoders. AC-3 patents expired; E-AC-3 and Dolby Vision are Dolby-licensed (Medium). | [ETSI TS 102 366](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/) | partial / N / Y | Low |
| CG-13.20 | ALAC | Apple Lossless. | Apple Lossless (no web spec) | N / N / Y — single-engine | Low |
| CG-13.21 | Media MIME and codec strings | `canPlayType()`, RFC 6381 codec parameters, `isTypeSupported()`. | [RFC 6381](https://www.rfc-editor.org/rfc/rfc6381) | Y / Y / Y | Medium |

**Implications:** H.264 and AAC are in all three engines and are pool-licensed, so an MIT-only Izcal cannot ship them without either platform decoders or a licence decision. That is a product decision the media spec must name explicitly, not a coding task.

## 14. HTML media elements and streaming

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-14.1 | `<video>` and `<audio>` | `HTMLMediaElement`: loading, buffering, seeking, ready states, `playbackRate`, `preservesPitch`, media controls UI, `poster`, `<source>` selection, `srcObject`. | [HTML media elements](https://html.spec.whatwg.org/multipage/media.html) | Y / Y / Y | Medium |
| CG-14.2 | Default media controls | The built-in controls UI, fullscreen, captions menu, `controlslist`. | [HTML media](https://html.spec.whatwg.org/multipage/media.html#user-interface) | Y / Y / Y (`controlslist` Chromium only) | Medium |
| CG-14.3 | Text tracks | `<track>`, `TextTrack`, `TextTrackCue`, `VTTCue`, in-band tracks. | [HTML text tracks](https://html.spec.whatwg.org/multipage/media.html#timed-text-tracks) | Y / Y / Y | Medium |
| CG-14.4 | WebVTT | Caption format, cue settings, regions, `::cue` styling. Interop 2026 investigation area. | [WebVTT (CR)](https://www.w3.org/TR/webvtt1/) | Y / Y / Y (regions partial) | Medium |
| CG-14.5 | Media Fragments | `#t=10,20` time fragments in media URLs. | [Media Fragments URI (REC)](https://www.w3.org/TR/media-frags/) | Y / Y / Y | Medium |
| CG-14.6 | Media Source Extensions | `MediaSource`, `SourceBuffer`, `changeType()`. Basis of all adaptive streaming players. | [MSE 2 (WD)](https://www.w3.org/TR/media-source-2/) | Y / Y / Y | Medium |
| CG-14.7 | MSE in workers and ManagedMediaSource | MSE from dedicated workers; `ManagedMediaSource` (power-aware, used on iPhone). | [MSE 2](https://www.w3.org/TR/media-source-2/) | partial / partial / Y | Low |
| CG-14.8 | Encrypted Media Extensions | `requestMediaKeySystemAccess`, `MediaKeys`, sessions, `encryptionScheme`, HDCP policy check. | [EME (REC) / EME 2](https://www.w3.org/TR/encrypted-media-2/) | Y / Y / Y | Medium |
| CG-14.9 | Content Decryption Modules | Clear Key (spec-defined, open) and proprietary CDMs: Widevine (Google), PlayReady (Microsoft), FairPlay (Apple). Proprietary CDMs require commercial agreements with their owners and cannot be written by Izcal (Medium). | [EME Clear Key](https://www.w3.org/TR/encrypted-media-2/#clear-key) | Widevine+Clear Key / Widevine+Clear Key / FairPlay (+Clear Key partial) | Medium |
| CG-14.10 | Media Capabilities | `decodingInfo()`/`encodingInfo()`: smooth, power-efficient, HDR, key-system queries. | [Media Capabilities (WD)](https://www.w3.org/TR/media-capabilities/) | Y / Y / Y | Medium |
| CG-14.11 | Media Playback Quality | `getVideoPlaybackQuality()` dropped-frame counts. | [Media Playback Quality](https://w3c.github.io/media-playback-quality/) | Y / Y / Y | Medium |
| CG-14.12 | `requestVideoFrameCallback` | Per-video-frame callbacks with presentation metadata. | [video-rvfc (WICG)](https://wicg.github.io/video-rvfc/) | Y / Y / Y | Medium |
| CG-14.13 | Media Session | Metadata, artwork and action handlers for OS media controls; `setPositionState`. | [Media Session (WD)](https://www.w3.org/TR/mediasession/) | Y / Y / Y | Medium |
| CG-14.14 | Picture-in-Picture (video) | `requestPictureInPicture()`, PiP window events. Gecko has a PiP UI but no web API. | [Picture-in-Picture (WD)](https://www.w3.org/TR/picture-in-picture/) | Y / N (UI only) / Y | Medium |
| CG-14.15 | Document Picture-in-Picture | Always-on-top window with arbitrary HTML. | [Document PiP (WICG)](https://wicg.github.io/document-picture-in-picture/) | Y / N / N — single-engine | Medium |
| CG-14.16 | Remote Playback | `video.remote` for casting (Cast, AirPlay). | [Remote Playback (CR)](https://www.w3.org/TR/remote-playback/) | partial (Android) / N / Y | Medium |
| CG-14.17 | Presentation API | Showing a page on a second screen or Cast receiver. | [Presentation API (CR)](https://www.w3.org/TR/presentation-api/) | Y / N / N — single-engine | Medium |
| CG-14.18 | Audio Output Devices | `HTMLMediaElement.setSinkId()`, `selectAudioOutput()`, `AudioContext.setSinkId()`. | [Audio Output Devices (CR)](https://www.w3.org/TR/audio-output/) | Y / Y (`selectAudioOutput` Gecko only) / N | Medium |
| CG-14.19 | Audio Session | `navigator.audioSession.type` to set mixing and ducking behaviour. | [Audio Session (WD)](https://www.w3.org/TR/audio-session/) | N / N / Y — single-engine | Medium |
| CG-14.20 | Autoplay policy | User-activation and muted-autoplay rules; `navigator.getAutoplayPolicy()` (Gecko only). | [Autoplay Policy Detection (WD)](https://www.w3.org/TR/autoplay-detection/), [HTML](https://html.spec.whatwg.org/multipage/media.html#eligible-for-autoplay) | Y / Y / Y (policy); detection API Gecko only | Medium |
| CG-14.21 | WebCodecs | `VideoDecoder`, `VideoEncoder`, `AudioDecoder`, `AudioEncoder`, `VideoFrame`, `AudioData`, `EncodedVideoChunk`; codec registry. | [WebCodecs (WD)](https://www.w3.org/TR/webcodecs/) | Y / Y / Y (WebKit audio codecs added in Safari 26, recalled) | Low |
| CG-14.22 | ImageDecoder | WebCodecs image decoding with frame access. | [WebCodecs ImageDecoder](https://www.w3.org/TR/webcodecs/#imagedecoder-interface) | Y / Y / partial | Low |
| CG-14.23 | Web Audio 1.0/1.1 | `AudioContext`, node graph (oscillator, gain, biquad, convolver, panner/HRTF, analyser, dynamics compressor, wave shaper), `OfflineAudioContext`, `decodeAudioData`. | [Web Audio 1.1 (WD)](https://www.w3.org/TR/webaudio-1.1/) | Y / Y / Y | Medium |
| CG-14.24 | AudioWorklet | Script audio processing on the render thread. | [Web Audio AudioWorklet](https://www.w3.org/TR/webaudio-1.1/#AudioWorklet) | Y / Y / Y | Medium |
| CG-14.25 | Web MIDI | MIDI device access, often bundled with audio. | [Web MIDI (WD)](https://www.w3.org/TR/webmidi/) | Y / Y / N | Medium |

**Implications:** Media is three layers: the element (CG-14.1–14.5), the streaming and DRM stack (MSE, EME, CDMs) and the low-level APIs (WebCodecs, Web Audio). EME's proprietary CDMs are an external dependency Izcal cannot build, which has product consequences for streaming sites.

## 15. Media capture and real-time communication

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-15.1 | Media Capture and Streams | `getUserMedia()`, `MediaStream`, `MediaStreamTrack`, constraints, `enumerateDevices()`, `devicechange`. | [mediacapture-streams (CR)](https://www.w3.org/TR/mediacapture-streams/) | Y / Y / Y | Medium |
| CG-15.2 | Media Capture extensions | Background blur, face framing, voice isolation, frame stats, PTZ controls. | [mediacapture-extensions](https://w3c.github.io/mediacapture-extensions/) | partial / partial / partial | Low |
| CG-15.3 | MediaStream Image Capture | `ImageCapture.takePhoto()`, photo settings. | [Image Capture (WD)](https://www.w3.org/TR/image-capture/) | Y / N / N — single-engine | Medium |
| CG-15.4 | Capture from DOM elements | `canvas.captureStream()`, `video.captureStream()`. | [mediacapture-fromelement (WD)](https://www.w3.org/TR/mediacapture-fromelement/) | Y / Y (video prefixed) / partial | Medium |
| CG-15.5 | Screen Capture | `getDisplayMedia()` for screens, windows and tabs; system audio; `displaySurface` hints. Desktop only. | [Screen Capture (WD)](https://www.w3.org/TR/screen-capture/) | Y / Y / Y | Medium |
| CG-15.6 | Capture Handle | Letting a captured page identify itself to the capturer. | [Capture Handle (WD)](https://www.w3.org/TR/capture-handle-identity/) | Y / N / N — single-engine | Medium |
| CG-15.7 | Region Capture | `CropTarget`, `track.cropTo()`. | [Region Capture (WD)](https://www.w3.org/TR/mediacapture-region/) | Y / N / N — single-engine | Medium |
| CG-15.8 | Element Capture | `RestrictionTarget`, `track.restrictTo()`: capture one element only. | [Element Capture](https://screen-share.github.io/element-capture/) | Y / N / N — single-engine | Medium |
| CG-15.9 | Captured Surface Control and captured mouse events | Scrolling and zooming a captured tab; cursor position events. | [Captured Surface Control](https://screen-share.github.io/captured-surface-control/), [Captured Mouse Events](https://screen-share.github.io/captured-mouse-events/) | partial / N / N | Low |
| CG-15.10 | MediaStreamTrack insertable streams | `MediaStreamTrackProcessor`, `VideoTrackGenerator` (breakout box). | [mediacapture-transform (WD)](https://www.w3.org/TR/mediacapture-transform/) | Y / N / partial | Low |
| CG-15.11 | MediaRecorder | Recording streams to WebM/MP4. | [MediaStream Recording (WD)](https://www.w3.org/TR/mediastream-recording/) | Y / Y / Y | Medium |
| CG-15.12 | Content hints | `track.contentHint` (motion, detail, speech, music). | [mst-content-hint (WD)](https://www.w3.org/TR/mst-content-hint/) | Y / N / Y | Low |
| CG-15.13 | HTML Media Capture | `<input capture>` opens camera or microphone on mobile. | [HTML Media Capture (WD)](https://www.w3.org/TR/html-media-capture/) | partial (Android) / partial / Y (iOS) | Medium |
| CG-15.14 | WebRTC 1.0 | `RTCPeerConnection`, ICE/STUN/TURN, DTLS-SRTP, SDP offer/answer, transceivers, simulcast, `setCodecPreferences`, DTMF. | [WebRTC (REC)](https://www.w3.org/TR/webrtc/) | Y / Y / Y | Medium |
| CG-15.15 | RTCDataChannel | SCTP-over-DTLS data channels. | [WebRTC §6](https://www.w3.org/TR/webrtc/#peer-to-peer-data-api) | Y / Y / Y | Medium |
| CG-15.16 | WebRTC statistics | `getStats()` dictionaries. | [webrtc-stats (CR)](https://www.w3.org/TR/webrtc-stats/) | Y / Y / Y (coverage differs) | Medium |
| CG-15.17 | WebRTC Encoded Transform | `RTCRtpScriptTransform` for end-to-end encryption and frame processing; Chromium's earlier `createEncodedStreams`. | [webrtc-encoded-transform (WD)](https://www.w3.org/TR/webrtc-encoded-transform/) | partial / Y / Y | Low |
| CG-15.18 | WebRTC extensions, SVC, priority, ICE | Scalable video coding modes, RTP header extension control, `RTCIceTransport` extensions, DSCP priority. | [webrtc-extensions](https://www.w3.org/TR/webrtc-extensions/), [webrtc-svc](https://www.w3.org/TR/webrtc-svc/), [webrtc-priority](https://www.w3.org/TR/webrtc-priority/) | partial / partial / partial | Low |
| CG-15.19 | WebRTC Identity | Identity providers for peer identity. Never broadly shipped. | [webrtc-identity (CR)](https://www.w3.org/TR/webrtc-identity/) | N / partial / N — single-engine | Medium |
| CG-15.20 | WebRTC mandatory codecs and media stack | VP8 and H.264 (video), Opus and G.711 (audio), RTP/RTCP, congestion control, echo cancellation, noise suppression, jitter buffer. Interop 2025–2026 focus area (WebRTC). | [RFC 7742](https://www.rfc-editor.org/rfc/rfc7742), [RFC 7874](https://www.rfc-editor.org/rfc/rfc7874) | Y / Y / Y | Medium |

**Implications:** WebRTC requires a whole real-time media stack (echo cancellation, jitter buffers, congestion control) that no spec gives as an algorithm; and its mandatory H.264 brings back the licence question from group 13.

## 16. XR, speech, printing and other rendering features

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| CG-16.1 | WebXR Device API | Immersive VR and inline sessions, reference spaces, poses, `XRWebGLLayer`. Shipped in Chromium (Android, desktop with OpenXR, Quest browser) and visionOS Safari; Gecko removed its implementation. | [WebXR (CRD)](https://www.w3.org/TR/webxr/) | Y / N / partial (visionOS) | Medium |
| CG-16.2 | WebXR modules | AR module, hit test, anchors, DOM overlay, depth sensing, lighting estimation, hand input, layers, gamepads, camera access. | [Immersive Web WG specs](https://immersive-web.github.io/) | partial / N / partial | Low |
| CG-16.3 | Web Speech: synthesis | `speechSynthesis`, voices, utterances, boundary events. | [Web Speech API](https://webaudio.github.io/web-speech-api/) | Y / Y / Y | Medium |
| CG-16.4 | Web Speech: recognition | `SpeechRecognition`; server-based in Chromium with on-device options added in 2025 (recalled); `webkit`-prefixed in Safari. | [Web Speech API](https://webaudio.github.io/web-speech-api/) | Y / N / partial | Low |
| CG-16.5 | Printing | `window.print()`, `beforeprint`/`afterprint`, print media type, `@page`, fragmentation into pages, print preview, headers and footers, background printing control. | [HTML printing](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#printing), [css-page-3](https://www.w3.org/TR/css-page-3/) | Y / Y / Y | Medium |
| CG-16.6 | Save as PDF output | Generating PDF from the print path. PDF 2.0 (ISO 32000-2) is available at no cost from the PDF Association (Medium). | [ISO 32000-2](https://pdfa.org/resource/iso-32000-2/) | Y / Y / Y | Medium |
| CG-16.7 | Built-in PDF viewer | Rendering PDF documents in a tab, navigating, searching, form filling, printing. | [ISO 32000-2](https://pdfa.org/resource/iso-32000-2/), [HTML PDF viewer support](https://html.spec.whatwg.org/multipage/system-state.html#pdf-viewing-support) | Y / Y / Y | Medium |
| CG-16.8 | Fullscreen rendering | Element fullscreen, `:fullscreen`, `::backdrop`, top layer. | [Fullscreen API (WHATWG)](https://fullscreen.spec.whatwg.org/) | Y / Y / Y (iPhone video-only) | Medium |
| CG-16.9 | Scroll to text fragment rendering | Highlighting `#:~:text=` matches with `::target-text`. | [URL Fragment Text Directives (WICG)](https://wicg.github.io/scroll-to-text-fragment/) | Y / Y / Y | Medium |
| CG-16.10 | Find in page highlighting | The UA's find-in-page highlight rendering, `hidden=until-found` reveal, `beforematch`. | [HTML hidden=until-found](https://html.spec.whatwg.org/multipage/interaction.html#the-hidden-attribute) | Y / Y / Y (until-found newer in Gecko/WebKit, recalled) | Low |
| CG-16.11 | Reader view and text-only rendering | Browser reader mode. A UA feature, not a spec. | none (UA feature) | Y (side panel) / Y / Y | Low |

**Implications:** Printing, PDF output and the PDF viewer are rendering features without a single owning web spec, so they are easy to lose in a spec-driven inventory. WebXR has dropped to roughly two engines.

## Coverage check

WPT top-level directories in this strand were all placed except those below. `css/` subdirectories were checked against the `css/` listing on wpt.live.

**Placed (for the record):** `css/*` (all `css-*` modules map to groups 1–9; `CSS1`, `CSS2` → CG-1.1; `WOFF2` → CG-6.10; `compositing` → CG-7.16; `filter-effects` → CG-7.14/15; `fill-stroke` → CG-3.15; `geometry` → CG-7.18; `mediaqueries` → CG-2.16/17; `motion` → CG-7.11; `printing` → CG-16.5; `selectors` → CG-1.11–15; `cssom`, `cssom-view` → group 8; `css-device-adapt` → CG-2.18; `css-gaps` → CG-3.10; `css-mixins` → CG-1.20; `css-forms` → CG-9.3; `css-zoom` → CG-2.19; `css-size-adjust` → CG-6.5; `css-shadow` → CG-1.17), `cssom`, `fonts`, `font-access`, `svg`, `mathml`, `webgl`, `webgpu`, `imagebitmap-renderingcontext`, `images`, `apng`, `avif`, `gif`, `jpegxl`, `png`, `density-size-correction`, `media`, `media-source`, `encrypted-media`, `media-capabilities`, `media-playback-quality`, `mediasession`, `video-rvfc`, `webvtt`, `webcodecs`, `webaudio`, `audio-output`, `audio-session`, `autoplay-policy-detection`, `picture-in-picture`, `document-picture-in-picture`, `remote-playback`, `presentation-api`, `mediacapture-*` (all six), `html-media-capture`, `mst-content-hint`, `screen-capture`, `captured-mouse-events`, `webrtc`, `webrtc-encoded-transform`, `webrtc-extensions`, `webrtc-ice`, `webrtc-identity`, `webrtc-priority`, `webrtc-stats`, `webrtc-svc`, `webxr`, `speech-api`, `webmidi`, `web-animations`, `scroll-animations`, `animation-worklet`, `worklets`, `resize-observer`, `intersection-observer`, `inert`, `forced-colors-mode`, `html-ruby-extensions`, `print`, `quirks`, `compat`, `shape-detection`, `eyedropper`, `delegated-ink`, `viewport`, `visual-viewport`, `viewport-segments`, `virtual-keyboard`, `fullscreen`, `scroll-to-text-fragment`, `mimesniff`.

**Could not place confidently:**
- `css/css-navigation` — may be spatial navigation (CG-5.7) or a navigation-related pseudo-class draft; not confirmed.
- `css/css-parser-api` — placed tentatively at CG-8.6; the spec status is unclear.
- `css/fetching` — probably CSS resource-fetching rules (CORS mode for fonts and images); no CSS module of that name on the current-work page.
- `css/css-rhythm` — placed with CG-4.31 but rhythmic sizing has no shipping engine; tests may cover only a fragment.
- `content-dpr` — the `Content-DPR` response header for image density; may be deprecated client-hints work. Not placed.
- `acid` — the Acid2/Acid3 legacy conformance pages; a cross-check rather than a feature.
- `paint-timing`, `largest-contentful-paint`, `layout-instability`, `element-timing`, `container-timing`, `long-animation-frame` — rendering-performance observability. They belong to the performance strand, not this one, but they depend on paint and layout events defined here.
- `webnn` — ML inference on GPU/NPU; graphics-adjacent but belongs to the APIs strand.
- `screen-details`, `window-management` — multi-screen placement; belongs to the windowing strand.

## Gaps in this inventory

- **Ship cells were not checked against Baseline data.** webstatus.dev renders client-side and returned nothing to the fetch tool. Most ship cells are from recollection to mid-2026 plus a few release-note snippets. A follow-up pass should verify every "Low" ship cell, especially for features that moved between June and October 2026 (`if()`, `random()`, `sibling-index()`, `contrast-color()`, `field-sizing`, `shape()`, Grid Lanes in Chromium/Gecko, scroll-driven animations in Gecko, cross-document view transitions in Gecko, HEVC in Gecko, native HLS in Chrome desktop).
- **Licence and patent notes are recalled, not re-read.** H.264, HEVC, AAC, AV1 (Sisvel), E-AC-3, JPEG XL and WebP claims need a High re-check from each licensing body's own pages before any Izcal decision.
- **Interop 2021–2025 focus areas were not read individually.** Only the Interop 2026 list was read in full. Earlier years' areas are believed to be covered by rows above but were not cross-checked one by one.
- **Coarse grain hides big sub-features.** One row per module means Text 4, Fonts 4, Canvas 2D and Web Audio each carry dozens of properties or interfaces with uneven support. The mapping step should not treat a row as "covered" because a spec names the module.
- **Platform and UA features are thin.** Native scrollbar rendering, overlay scrollbars, OS theme integration, system font fallback lists, emoji font choice, sub-pixel text anti-aliasing, and GPU blocklists are real shipping obligations without web specs; only some appear above.
- **Mobile-only rendering** (pinch zoom, text autosizing beyond `text-size-adjust`, tap highlight `-webkit-tap-highlight-color`, `touch-action` rendering effects) is under-covered.
- **Not covered at all:** CSS Template Layout and other abandoned CSSWG notes beyond a single row; the WHATWG Compat standard's full list; legacy Netscape/IE rendering features (`<marquee>`, `<blink>`), which HTML still requires for `<marquee>`.
