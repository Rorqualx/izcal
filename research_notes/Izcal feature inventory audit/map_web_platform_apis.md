# Izcal feature inventory audit: mapping of web platform APIs

Compiled 24 October 2026 against the reports on main at 217e02d.

Input: `web_platform_apis.md` (333 rows, WA-1.1 to WA-22.12). Every row was searched in `reports/` with several synonyms (a regex over all 33 report files), and the surrounding section was read before a status was given. Missing rows list the search terms in the evidence cell.

## How to read this

**Statuses.** **Covered**: a spec specifies the behaviour. **Partial**: mentioned or partly specified, with what is missing stated. **Excluded**: refused, held by a recorded rule, or deferred with an owner ("Deferred (Stage 2)" and so on). **Missing**: no spec mentions it after at least three searches. Rows marked **[1E]** ship in a single engine, or are retired. For those, the admission rule (LY-D10 and ST-D18: "a WG resolution, WPT tests, and two shipping engines or an Interop focus area", adopted for web APIs by DM 6.3 and DM 9.1) would hold the feature. But no per-feature decision is recorded, so they still count as Missing or Partial.

**Claims by reference** (rows marked "claim" rest on these):

- **DOM and HTML in full.** DM 6.3: "The corpus Izcal ships at Stage 1 is DOM, HTML, Web IDL's own interfaces, and every specification this spec or another accepted spec admits". DM-D15: "Izcal implements what the pinned specifications require, including the legacy surface the web still depends on, and nothing the specifications have removed". So a feature in the WHATWG DOM or HTML Standard at the pin counts as Covered unless it is separately excluded.
- **ECMAScript.** JS-M2: "ES2026 plus Temporal". JS-1 covers "every feature at Stage 4 at the pinned ECMA-262 commit". JS-70: "features reaching Stage 4 implemented by the following June edition". JS-2 covers ECMA-402, and JS-3 covers "Wasm 3.0 plus threads". The reports never record a TC39 proposal's stage except Temporal, decorators, ShadowRealm, shared structs and the module phases. So for proposal rows the stage is this audit's assumption (Stage 4 if Baseline in three engines), and it is flagged in each row.
- **Fetch.** NW 6.7 anchors "each function" to the pinned `fetch.bs`. Fetch features are Covered where NW designs the step.
- **Gap rule.** DM 6.18: "Any web API not in this table or another spec's is recorded in the ledger as a gap before it is admitted." This is why script APIs that no table lists are Partial or Missing even when a transport exists.

**Abbreviations.** DM DOM, bindings and event loop; JS JavaScript engine; NW networking; SG storage; WC WebCrypto; DV devices; ED editing; AX accessibility; EX extensions; DT DevTools; ML ML service; AH app host; SH shell; SHUI HGBA shell UI design; PH phishing; CB content blocking; ST HTML and CSS parsing and style; LY layout; GR graphics; MD images and media; AN animation; TX fonts and text; FP resource footprint; SV SVG; SS security scope; H2 HGBA-2; H3 HGBA-3 (Future software browser engine); S1 HGBA Stage 1 design. "S1 M5" is a Stage 1 milestone; DM-M2 and the like are the specs' own milestones. A † marks a threshold the spec itself sets.

## Mapping

### 1. HTML: documents, browsing contexts, navigation and history

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-1.1 | Document and browsing-context model | **Covered** | DM 6.4, 6.8, 6.9; HTML claim | DM-M2 (S1 M5–M6) | DM 6.8: `hg-agents` "creates groups, applies COOP's browsing-context-group switches"; remote-shaped `WindowProxy` (DM-D7); `opener` in the bfcache table (6.9) |
| WA-1.2 | Navigation and session history | **Covered** | DM 6.9 (`hg-navigation`, DM-D20) | DM-M2 | "Navigables and traversable navigables, session history entries and steps, the navigate algorithm, 'apply the history step'" |
| WA-1.3 | Navigation API | **Covered** | DM 6.9; DM-4 (Interop gate) | DM-M2 | "the Navigation API's `navigate` event with `intercept()` and `NavigationPrecommitController`" |
| WA-1.4 | Back/forward cache behaviour | **Covered** | DM 6.9 (DM-D9, DM-D14); DM-D15 exception for `notRestoredReasons`; S1 §6 | DM-M2 | bfcache policy table; reasons "published in `notRestoredReasons` with cross-origin masking" |
| WA-1.5 | Page Visibility and Page Lifecycle | **Covered** | HTML claim; DM 6.14 (freeze/resume), DM-D15 owner exception | DM-M2/M3 | DM-D15 admits Page Lifecycle `freeze`/`resume`; "`document.wasDiscarded` is set after a discard-and-reload" |
| WA-1.6 | `beforeunload` / `unload` | **Covered** | DM 6.9, DM-D9 | DM-M2 | `unload` fired "only if the document is not salvageable"; beforeunload fan-out with a 1 s† deadline |
| WA-1.7 | iframes and embedding | **Covered** | DM 6.12 (`iframe`, `object` as navigables; `embed` fallback), DM 6.9 (sandbox flags in the broker); HTML claim | DM-M2/M3 | `srcdoc`, `allow`, `referrerpolicy` not named individually; lazy loading in NW 6.3 |
| WA-1.8 | Credentialless iframes | **Partial** [1E] | NW 6.8 (nonce n), NW 6.7 (COEP `credentialless`), ST 6.7 | — | Partition key has "per-navigation nonce (credentialless iframes, fenced frames, recordings)". Missing: the `credentialless` attribute's DOM and navigation design and an admission decision (Chromium-only; LY-D10 would hold it) |
| WA-1.9 | Fenced frames | **Partial** [1E] | NW 6.8 (nonce n) only | — | Named only in the partition-key notation. Missing: any element or navigation design, or a refusal; tied to Protected Audience (WA-14.6) |
| WA-1.10 | `Location.ancestorOrigins` | **Covered** | HTML claim (DM 6.3 corpus, DM-D15); `Location` in `hg-window` (DM 6.17) | DM-M2 | Not named (searched `ancestorOrigins`); in HTML, two engines |
| WA-1.11 | Origin API | **Covered** | HTML claim | — | Not named (searched `Origin.from`, "Origin API", `isSameSite`); in HTML, two engines |
| WA-1.12 | Text fragments | **Covered** | ED 6.12 (ED-D17); DM 6.9 (navigation side) | ED-M2 (S1 M6–M7) | "Text directives (`#:~:text=`, three engines) use the same matcher with the constant-shape rule"; `::target-text` through overlays |
| WA-1.13 | Top-navigation restrictions | **Covered** | DM 6.9 (broker history-action and sandbox checks), DM-D20; HTML claim | DM-M2 | Activation minted only by the broker from observed input |
| WA-1.14 | Initial `about:blank` behaviour | **Covered** | DM 6.9 ("Document creation and initialization"); HTML claim | DM-M2 | Synchronous initial load not named; covered by HTML at the pin |
| WA-1.15 | Printing | **Covered** | LY 6.7 (LY-D8); S1 §6 Printing | Layout and `window.print()` Stage 1; print UI Stage 2 | "Paged layout, `@page` and the `window.print()` entry point are Stage 1 layout and binding work"; `beforeprint` snapshot (LY 6.7) |
| WA-1.16 | Simple dialogs | **Covered** | DM 6.7 (pause state), DM-D15 | DM-M2 | "`alert`/`confirm`/`prompt` including in cross-origin iframes — Implemented as specified"; drawn by the broker's prompt surface |
| WA-1.17 | Window and screen properties | **Covered** | DM 6.16 (environment-values record; D-S1, OD-6); LY (CSSOM View) | DM-M2 | "`screen` and `devicePixelRatio` come from one broker-supplied, normalized environment-values record" |
| WA-1.18 | Out-of-order HTML streaming | **Missing** [1E] | — | — | Searched "template for", "out-of-order", "patching", "processing instruction": no hits. Chromium-only proposal |
| WA-1.19 | XSLT and `XSLTProcessor` | **Excluded** | ST-D4; DM-D15 | — | ST-D4: "do not implement synchronous mutation events or XSLT" |

### 2. HTML: elements, attributes and document metadata

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-2.1 | HTML parser and serializer | **Covered** | ST (tree builder, fragment parsing, `document.write`) | ST milestones | DM 6.18: parsing and `document.write` are ST's |
| WA-2.2 | Sectioning, grouping and text-level elements | **Covered** | DM 6.12; LY (ruby) | DM-M3 (full element set) | "Document structure, sections, grouping, text-level semantics, … `search`" |
| WA-2.3 | Heading level adjustment | **Missing** [1E] | — | — | Searched `headingoffset`, `headingreset`, "heading level" (only AX's computed level). Not in HTML yet; Firefox flag |
| WA-2.4 | Links and `<a>` behaviours | **Covered** | DM 6.12 (`a`, `area`); HTML claim; CB 6.2 (ping channel); NW 6.14 (downloads) | DM-M2/M3 | Attributes not enumerated; HTML at the pin |
| WA-2.5 | `<link>` relation types | **Covered** | DM 6.12 (rel processing, `preload`, `modulepreload`); NW 6.3 (Early Hints, preconnect), NW 6.11 (prefetch); ST 6.6 (render-blocking) | DM-M3; NW-M2 | `rel=manifest` has no consumer (see WA-11.1) |
| WA-2.6 | `<meta>` names and pragmas | **Covered** | DM 6.12 (`meta`); ST 6.4 (viewport, referrer); NW 6.7 (`<meta>` CSP); DM 6.9 (meta refresh) | DM-M3 | `theme-color`, `rating` not named |
| WA-2.7 | `<template>` and `<slot>` | **Covered** | DM 6.6, 6.12 | DM-M1 | Named and manual slot assignment |
| WA-2.8 | Responsive images markup | **Covered** | DM 6.12 (selection algorithm), MD 6.20; NW 6.3 (`fetchpriority`) | DM-M3 | `sizes=auto` by HTML claim |
| WA-2.9 | Lazy loading | **Covered** | NW 6.3 (lazy-load starts are recorded decision points); LY 6.15 (out-of-process frames) | NW-M2; LY | Chrome 148's lazy `video`/`audio` is not in HTML and not covered |
| WA-2.10 | `<details>` and `<summary>` | **Covered** | DM 6.12; DM-4; ED 6.12 (find auto-expand) | DM-M2 | Interop 2026 tests are release gates |
| WA-2.11 | `hidden="until-found"` | **Covered** | ED 6.12 (`beforematch`, reveal rules); AX-O11 | ED-M2 | Reveal only on Enter, next or 500 ms† idle |
| WA-2.12 | Global attributes | **Covered** | DM 6.5, 6.12 (tabindex, popover, inert); ED 6.13 (`inputmode`, `enterkeyhint`, `autocapitalize`); ED 6.8 (`spellcheck`) | DM-M2; ED-M1/M2 | — |
| WA-2.13 | `autocorrect` attribute | **Covered** | ED 6.8 (ED-D19), ED 6.13 | ED-M3 | "the element's `autocorrect` state is On (HTML's states and defaults)"; shell setting is Stage 2 |
| WA-2.14 | Processing instructions in HTML | **Missing** [1E] | — | — | Searched "processing instruction", `<?target`, "PI node": no hits. Chromium-only |
| WA-2.15 | Permission and capability elements | **Excluded** | Deferred until standardized: S1 13.2 Permissions; SHUI | — | "The `<geolocation>` element (Chrome 144) is adopted once it is standardized"; `<usermedia>`, `<camera>`, `<microphone>` not named |
| WA-2.16 | `<model>` element | **Missing** [1E] | — | — | Searched `<model>`, "model element", "3D model": no hits. WebKit-only |
| WA-2.17 | MathML Core | **Covered** | DM 6.12 (element machinery); LY (MathML Core layout) | LY milestones | — |
| WA-2.18 | Obsolete and removed features | **Covered** | DM 6.12 (`embed`, "No plugins exist"), DM-D15; ST (legacy parsing); AX (`marquee`) | — | Fixed `navigator.plugins` PDF list not named |

### 3. HTML: forms and form controls

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-3.1 | Form submission | **Covered** | DM 6.12 (`hg-forms`) | DM-M2 | "'construct the entry list' with `formdata`, and form submission are a table-driven implementation" |
| WA-3.2 | Input types | **Covered** | DM 6.12 (22 input types); LY-D11 (pickers are browser-process UI) | DM-M2 | — |
| WA-3.3 | Constraint validation | **Covered** | DM 6.12 | DM-M2 | "`ValidityState`'s ten flags, static and interactive validation" |
| WA-3.4 | Standard form controls | **Covered** | DM 6.12 | DM-M2 | — |
| WA-3.5 | Customizable `<select>` | **Partial** | DM 6.12 (`selectedcontent`; forms track "select and customizable select"); AX-O12; LY-D10 holds `appearance: base-select` | — | Element and AX mapping specified; rendering held by LY-D10. The inventory shows two engines now (Chrome 135, Safari 27.0), so the hold's trigger looks met and the decision needs revisiting |
| WA-3.6 | `showPicker()` | **Covered** | HTML claim; LY-D11 pickers | DM-M2 | Not named (searched `showPicker`) |
| WA-3.7 | Autofill and `autocomplete` tokens | **Covered** | DM 6.12 (autofill DOM side); ED 6.6 (autofill door); SS SH-R1, SH-R2; S1 13.2 | Stage 1 passwords; Stage 2 beyond passwords (S1 13.2) | Token set not enumerated; Chromium's autofill event trial not covered |
| WA-3.8 | Directory upload | **Missing** | — | — | Searched `webkitdirectory`, `webkitRelativePath`, "directory upload", "Entries API": no hits. Three engines |
| WA-3.9 | HTML Media Capture | **Missing** | — | — | Searched "capture attribute", "HTML Media Capture", `capture=`: no hits. Mobile engines only (Android is Stage 2, S1 §9) |
| WA-3.10 | `<input type=checkbox switch>` | **Missing** [1E] | — | — | Searched "checkbox switch", "switch control" (only AT switch access): no hits. WebKit-only |
| WA-3.11 | Color input `alpha` and `colorspace` | **Covered** | HTML claim; LY-D11 picker | — | Not named (`colorSpace` hits are canvas only) |
| WA-3.12 | Form-associated custom elements | **Covered** | DM 6.6 (`ElementInternals`, four form callbacks), DM 6.12 | DM-M2 | — |
| WA-3.13 | OpaqueRange | **Missing** [1E] | — | — | Searched `OpaqueRange`, "opaque range", "live range inside input": no hits. Chromium-only |

### 4. HTML: interactive UI, focus, dialogs and drag and drop

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-4.1 | `<dialog>` | **Covered** | DM 6.12 (top layer); DM-4 | DM-M2 | — |
| WA-4.2 | Dialog `closedby` and `requestClose()` | **Covered** | DM 6.12 | DM-M2 | `closedby` named; `requestClose()` by HTML claim |
| WA-4.3 | Popover API | **Covered** | DM 6.12 | DM-M2 | — |
| WA-4.4 | `popover="hint"` | **Covered** | DM 6.12 | DM-M2 | "`popover` (`auto`, `manual`, `hint`)" |
| WA-4.5 | Invoker commands | **Covered** | DM 6.12 | DM-M2 | "invokers (`command`, `commandfor`)" |
| WA-4.6 | Interest invokers | **Excluded** | Held by admission rule: AX-O13 (ST-D18) | When admitted | "Follow the explainer's mapping while admitted (ST-D18's rule)" |
| WA-4.7 | `CloseWatcher` | **Covered** | HTML claim; DM 6.12 (light dismiss) | DM-M2 | Not named (searched `CloseWatcher`, "close request"); in HTML, two engines |
| WA-4.8 | Light-dismiss input timing change | **Covered** | DM 6.12 ("light dismiss … follow HTML at the pin") | DM-M2 | — |
| WA-4.9 | Focus management | **Covered** | DM 6.5 (`hg-html::focus`) | DM-M2 | — |
| WA-4.10 | `inert` | **Covered** | DM 6.5 | DM-M2 | — |
| WA-4.11 | Focusgroup | **Missing** [1E] | — | — | Searched `focusgroup`, "arrow-key navigation", "roving": no hits. Chromium-only |
| WA-4.12 | User activation | **Covered** | DM 6.5 (DM-D20) | DM-M2 | Broker-minted activation, 5 s† transient window, `navigator.userActivation` mirror |
| WA-4.13 | HTML drag and drop | **Covered** | ED 6.11 (ED-D13) | ED-M2 | Drag store in the broker; `DownloadURL` type held |
| WA-4.14 | Fullscreen API | **Covered** | DM 6.18 (Fullscreen row), DM 6.7 (fullscreen steps); MD 6.12 | DM-M2 | Fullscreen `keyboardLock` option not named |
| WA-4.15 | Pointer Lock | **Missing** | — | — | Searched "pointer lock", `requestPointerLock`, `unadjustedMovement`, "mouse lock": no hits. Three engines |
| WA-4.16 | Keyboard Lock and Keyboard Map | **Missing** [1E] | — | — | Searched "keyboard lock", `getLayoutMap`, "keyboard map": no hits. Chromium-only |
| WA-4.17 | `ariaNotify()` | **Covered** | AX 6.7; DM 6.18 | AX milestones | Constant-time record delivered with the next update |
| WA-4.18 | Editing hooks | **Covered** | ED 6.2 (ED-D1), ED-D2 (EditContext behind a developer setting) | ED-M2 | — |

### 5. HTML: scripting, modules and resource loading

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-5.1 | Classic scripts | **Covered** | DM 6.12 (element side); ST 6.4, 6.6 (parser-inserted and speculative); JS 6.14 | DM-M2 | The `integrity` check itself: see WA-5.12 |
| WA-5.2 | Module scripts | **Covered** | JS 6.14; DM 6.18 (module fetching with import maps) | JS-M1 (ES modules) | — |
| WA-5.3 | JSON and CSS module scripts | **Covered** | JS 6.14 (import attributes, JSON modules); CSS modules by HTML claim | JS-M2 | CSS module scripts and declarative CSS modules not named |
| WA-5.4 | Text module imports | **Excluded** | Deferred until TC39 Stage 4: JS 6.14 (state machine names text modules); JS-1, JS-70 | When Stage 4 | Firefox-only; the reports state no shipping rule for Stage 3 proposals beyond JS-1's Stage 4 scope |
| WA-5.5 | Import maps | **Covered** | DM 6.18; JS 6.14 | DM-M2 | Multiple import maps by HTML claim |
| WA-5.6 | `modulepreload` | **Covered** | DM 6.12 | DM-M3 | — |
| WA-5.7 | Preload and resource hints | **Covered** | DM 6.12; NW 6.3 (103 Early Hints); ST 6.4 | NW-M2 | "preload and preconnect handled as HTML specifies for navigations" |
| WA-5.8 | Speculation Rules | **Covered** | DM 6.9 (prerendering), DM 6.7 (speculation rules task source); NW 6.11 | Conflict: DM-M3 (S1 M7) vs S1 §9 (Stage 2) | Prerender off under Hardened and memory pressure. S1 §9 lists "Speculation Rules and prerender" as Stage 2 while DM-M3 lists prerendering in Stage 1 |
| WA-5.9 | No-Vary-Search | **Missing** | — | — | Searched `No-Vary-Search`, "Vary-Search", "query param" (only CB link decoration): no hits. Two engines |
| WA-5.10 | Render-blocking | **Covered** | ST 6.6; DM 6.7 (render-blocked filter) | ST milestones | "`blocking=render` and render-blocking `<link>` elements delay the first rendering opportunity" |
| WA-5.11 | Web bundles and signed exchanges | **Missing** [1E] | — | — | Searched "web bundle", `webbundle`, "signed exchange", SXG: no hits. Chromium-only, in decline |
| WA-5.12 | Integrity Policy | **Missing** | — | — | Searched `Integrity-Policy`, "subresource integrity", SRI: no hits; NW 6.2 carries `integrity` only as a request field. Baseline (three engines) |
| WA-5.13 | Signature-based integrity | **Missing** [1E] | — | — | Searched "signature-based", "Ed25519 signature" (hits are extension packages and devices only). Chromium-only |

### 6. DOM, Web Components and DOM-adjacent APIs

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-6.1 | DOM core | **Covered** | DM 6.1 | DM-M1 | — |
| WA-6.2 | Events | **Covered** | DM 6.5 | DM-M1 | Retargeting, passive defaults |
| WA-6.3 | Aborting | **Covered** | DOM claim; DM 4.2 lists `AbortSignal` | DM-M1 | — |
| WA-6.4 | MutationObserver | **Covered** | DM 6.6 | DM-M1 | Mutation events not implemented (ST-D4) |
| WA-6.5 | Ranges, traversal and `StaticRange` | **Covered** | DM 6.6 | DM-M1 | — |
| WA-6.6 | `moveBefore()` | **Covered** | DM 6.1 | DM-M1 | "`moveBefore` is a third primitive" |
| WA-6.7 | Shadow DOM | **Covered** | DM 6.6 | DM-M1 | — |
| WA-6.8 | Declarative shadow DOM | **Covered** | DM 6.6; ST 6.5 | DM-M1 | — |
| WA-6.9 | Custom elements | **Covered** | DM 6.6 | DM-M1 | "Customized built-ins (`is=`) are implemented" |
| WA-6.10 | Scoped custom element registries | **Covered** | DM 6.6; DM-4 | DM-M1 | — |
| WA-6.11 | Reference Target | **Partial** [1E] | AX 6.2 (IDREF resolution); DM owns the resolver | — | AX: "Reference Target redirects inward through hosts". Missing: DM binding and an admission decision (Chromium-only) |
| WA-6.12 | ARIA reflection | **Covered** | DM 6.18 (ARIA reflection); AX | DM-M2 | `aria-actions` not implemented (AX-O17) |
| WA-6.13 | DOM Parsing and Serialization | **Covered** | ST (fragment parsing, `DOMParser`); DM 6.18 | ST milestones | — |
| WA-6.14 | HTML Sanitizer API | **Covered** | ST (one parser and `setHTML`); DM 6.16 | ST milestones | — |
| WA-6.15 | XPath and XML documents | **Partial** | DOM claim (DM 6.3 corpus); ST (XML parsing) | — | XPath interfaces are in scope by the DOM claim, but no XPath evaluator has a design or owner (searched XPath, `document.evaluate`: hits are only CB/ST "not supported" for filters) |
| WA-6.16 | Selection API | **Covered** | DM 6.6; ED 6.4 | DM-M2; ED-M1 | — |
| WA-6.17 | UI Events | **Covered** | DM 6.5 (`hg-input`) | DM-M2 | — |
| WA-6.18 | Pointer Events | **Covered** | DM 6.5; GR (predicted events an admission question) | DM-M2 | `getPredictedEvents()` held |
| WA-6.19 | Touch Events | **Covered** | DM 6.5 | DM-M2 | Default-passive rule |
| WA-6.20 | Ink API | **Excluded** | Held by admission rule: GR (LY-D10's process) | — | "OS-delegated ink (the WICG Ink API) are admission questions … not designed" |
| WA-6.21 | Observers | **Covered** | LY 6.15 (`hg-layout-query`: ResizeObserver, IntersectionObserver v1 and v2) | LY milestones | `scrollMargin` by claim |
| WA-6.22 | Scrolling and caret APIs | **Covered** | LY 6.15 (CSSOM View, `caretPositionFromPoint`, `checkVisibility`); DM 6.7 (`scrollend`) | LY milestones | "scrolling with promises where admitted" |
| WA-6.23 | Observable API | **Missing** [1E] | — | — | Searched "Observable API", `EventTarget.when`, "WICG Observable": no hits. Chromium-only |
| WA-6.24 | View Transitions, script side | **Covered** | AN spec (with GR); DM 6.7, 6.9 | AN milestones | Cross-document eligibility decided in the broker |
| WA-6.25 | Geometry interfaces | **Covered** | DM 6.18 (layout owns "geometry APIs"); SV (Geometry Interfaces 1 pinned) | LY milestones | — |
| WA-6.26 | Visual Viewport | **Covered** | LY 6.1 (recorded input); GR 6.28 | LY milestones | — |

### 7. Event loop, timers, scheduling, workers and messaging

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-7.1 | Event loop, tasks and microtasks | **Covered** | DM 6.7 (DM-D1) | DM-M1 | — |
| WA-7.2 | Timers and animation frames | **Covered** | DM 6.7 (DM-D10, AN-D3) | DM-M1 | No 1 ms clamp; hidden-page alignment |
| WA-7.3 | `requestIdleCallback` | **Covered** | DM 6.7 | DM-M1 | — |
| WA-7.4 | Prioritized Task Scheduling | **Covered** | DM 6.7 | DM-M1 | "ship from the first milestone" |
| WA-7.5 | `isInputPending` | **Excluded** | DM 6.7; DM-D15 | — | "`isInputPending` is not implemented" |
| WA-7.6 | Error reporting | **Covered** | HTML claim; DM 6.7 (checkpoint "notifies rejected promises") | DM-M1 | `reportError` not named |
| WA-7.7 | Dedicated workers | **Covered** | DM 6.10 | DM-M2 | — |
| WA-7.8 | Shared workers | **Covered** | DM 6.10 (extended-lifetime timeout 30 s†), DM-D8 | DM-M3 | — |
| WA-7.9 | Worklets | **Covered** | DM 6.7, 6.8; JS 6.12 (AudioWorklet) | DM-M2/M3 | — |
| WA-7.10 | Cross-document messaging | **Covered** | DM 6.11 (DM-D13) | DM-M2 | `targetOrigin` checked in the broker |
| WA-7.11 | Channel messaging | **Covered** | DM 6.11 | DM-M2 | — |
| WA-7.12 | Broadcast Channel | **Covered** | DM 6.11 | DM-M2 | Partitioned |
| WA-7.13 | Structured clone and transfer | **Covered** | DM 6.11 (DM-D12) | DM-M2 | 32 serializable and 16 transferable interfaces |
| WA-7.14 | SharedArrayBuffer and Atomics | **Covered** | JS 6.12; DM 6.8; HGBA-2 I-6 | JS-M2 | Gated on cross-origin isolation |
| WA-7.15 | Web Locks | **Covered** | DM 6.17 (`hg-agents`); SG-D12 | DM-M3 | — |

### 8. Fetch, networking and data formats

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-8.1 | Fetch API | **Covered** | NW 6.7; DM 6.18 | NW-M1 (Fetch core) | — |
| WA-8.2 | Fetch upload streaming and ranges | **Partial** | NW 6.2 (bodies as data pipes); NW 6.10 (ranges cached) | — | Missing: a stated design for `duplex: "half"` streaming uploads and the HTTP-version rule; an Interop 2026 area |
| WA-8.3 | `fetchLater()` | **Covered** | NW 6.11 | NW-M4 (not itemized) | Owned by the partition service, survives the document |
| WA-8.4 | Body helpers | **Covered** | DM 6.18 (body mixin) | — | `textStream()` (Chrome-only) not named |
| WA-8.5 | XMLHttpRequest | **Covered** | DM 6.18; DM 6.7 (sync XHR as a blocked state); DM-D15 | DM-M2 | — |
| WA-8.6 | Server-sent events | **Covered** | DM 6.18 (`EventSource` objects); NW 6.7 | — | — |
| WA-8.7 | Beacon | **Partial** | NW 6.11 (`keepalive` owned by the partition service); CB 6.2 (`sendBeacon` as a channel) | — | Missing: `navigator.sendBeacon` is not in DM 6.18's table, which says any API not in it "is recorded in the ledger as a gap" |
| WA-8.8 | WebSockets | **Covered** | NW 6.11 | NW-M2 (H1, H2), NW-M3 (H3) | Options-object constructor not named |
| WA-8.9 | WebSocketStream | **Missing** [1E] | — | — | Searched `WebSocketStream`, "stream-based WebSocket": no hits. Chromium-only |
| WA-8.10 | WebTransport | **Covered** | NW 6.11 (NW-D13); DM 6.18 | NW-M4, through the gate rule | — |
| WA-8.11 | Direct Sockets | **Excluded** | S1 §6 out-of-scope list; NW §1 | — | "raw TCP or UDP sockets for web content" is out of scope for Stages 1 and 2 |
| WA-8.12 | Streams | **Covered** | DM 6.18 | — | — |
| WA-8.13 | Compression Streams | **Covered** | DM 6.18; NW (`hg-compress` encoders) | NW-M1 | — |
| WA-8.14 | Encoding API | **Covered** | ST (ST-D16, `hg-encoding`); DM 6.18 | ST-M0 | — |
| WA-8.15 | URL and URLSearchParams | **Covered** | ST (`hg-url`); DM 6.18; JS 6.13 (UTS 46) | ST-M0 | — |
| WA-8.16 | URL Pattern | **Missing** | — | — | Searched `URLPattern`, "URL Pattern", "urlpattern": no hits (CB's "URL patterns" are filter syntax). Baseline (three engines); also used by service-worker static-route conditions |
| WA-8.17 | MIME Sniffing | **Covered** | NW 6.2 (`hg-net-wire`) | NW-M1 | — |
| WA-8.18 | Cookies as page-visible state | **Covered** | NW 6.9 (NW-D10, NW-D16); S1 §6 | NW-M2 | CHIPS-style partitioning by default |
| WA-8.19 | Client Hints | **Partial** [1E] | S1 13.2 (client-hint brands), S1 §6 (normalized), DM 6.16 ("client-hint mirrors"), NW-D12 | — | Missing: `Accept-CH` negotiation, which `Sec-CH-UA-*` headers are sent, and `navigator.userAgentData`. Chromium-only |
| WA-8.20 | Network Information and Save-Data | **Missing** [1E] | — | — | Searched `navigator.connection`, `Save-Data`, "Network Information", netinfo: no hits. Chromium-only |

### 9. Storage, files and quota

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-9.1 | Web Storage | **Covered** | DM 6.13 (DM-D11); SG 6.3, 6.7 | SG-M2 | — |
| WA-9.2 | IndexedDB 3 | **Covered** | SG 6.3; DM 6.13 | SG-M2 ("Full IndexedDB surface") | `getAllRecords()` not named |
| WA-9.3 | Cache API | **Covered** | SG 6.3 | SG-M3 | — |
| WA-9.4 | Storage Standard | **Covered** | SG 6.4, 6.5 (SG-D4, SG-D5) | SG-M2 | `estimate()` and `persist()` |
| WA-9.5 | Storage Buckets | **Excluded** | Deferred: SG 6.4 (SG-D4) | When the draft stabilizes | "Izcal ships the `default` bucket only until" the draft stabilizes |
| WA-9.6 | Cookie Store API | **Covered** | NW 6.9 (NW-D10) | NW-M2 | — |
| WA-9.7 | File API | **Covered** | DM 6.18; SG-D10 | SG-M3 | — |
| WA-9.8 | Origin private file system | **Covered** | SG-D9, SG 6.7; DM 6.18 | SG-M3 | Sync handles map committed extents read-only |
| WA-9.9 | File System Access | **Partial** | SG 6.14 (byte transport); SS 6.14a (grant rules); S1 13.2 (Stage 1 permission) | Stage 1 | Missing: the picker and handle API (`showOpenFilePicker` and the rest) is not in DM 6.18's table; S1 13.2 says the permission design is "written before M6". Chromium-only, adopted by S1 13.2 |
| WA-9.10 | File System Observer | **Missing** [1E] | — | — | Searched `FileSystemObserver`, "File System Observer": no hits. Chromium-only |
| WA-9.11 | File and Directory Entries API | **Missing** | — | — | Searched "Entries API", `webkitGetAsEntry`, `FileSystemDirectoryReader`: no hits (ED 6.11 drops yield `File` objects only). Three engines |
| WA-9.12 | Removed storage | **Excluded** | DM-D15; SG §2 | — | "AppCache, WebSQL … Not implemented" |

### 10. Service workers, background work, push and notifications

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-10.1 | Service Workers | **Covered** | DM 6.10; NW 6.7; S1 13.2 (Stage 1) | DM-M3 | — |
| WA-10.2 | Module service workers | **Covered** | Service Workers in the IDL corpus (DM 6.3); JS 6.14 | DM-M3 | Not named |
| WA-10.3 | Navigation preload | **Covered** | DM 6.10; NW 6.7 | DM-M3 | Allocated to the partition service |
| WA-10.4 | Static routing API | **Covered** | DM 6.10 | DM-M3 | `InstallEvent.addRoutes`, at most 1,024 conditions |
| WA-10.5 | Push API | **Excluded** | Deferred (Stage 2): S1 13.2; NW 6.14 (push client), NW-M6 | Stage 2 | "push and background sync are Stage 2". `PushManager`'s DOM side is not in DM 6.18 |
| WA-10.6 | Declarative Web Push | **Missing** [1E] | — | — | Searched "declarative web push", "declarative push": no hits. WebKit-only |
| WA-10.7 | Notifications API | **Excluded** | Deferred (Stage 2): SH 6.13, SH-D17 (accepted); DM 6.18 (DOM side) | Stage 2 | "Notifications are Stage 2; Stage 1 denies the permission honestly" |
| WA-10.8 | Background Sync | **Excluded** | Deferred (Stage 2): S1 13.2 | Stage 2 | Named only; no design. Chromium-only |
| WA-10.9 | Periodic Background Sync | **Missing** [1E] | — | — | Searched `periodicsync`, "periodic background sync", "periodic sync": no hits; S1 13.2's "background sync" does not name it. Chromium-only |
| WA-10.10 | Background Fetch | **Excluded** | NW 6.11 | — | "Background Fetch is not implemented (single engine for seven years)" |
| WA-10.11 | Content Index | **Missing** [1E] | — | — | Searched "Content Index", "offline content" (only S1 save-for-offline): no hits. Chromium Android only |

### 11. Installable web apps and OS integration

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-11.1 | Web App Manifest | **Missing** | — | — | Searched "web app manifest", `manifest.json`, `rel=manifest`, PWA, standalone, "home screen", "install": no spec owns it. AH 6.1 lists "Installed web app only" as a rejected alternative for the app host, which is not a decision on PWAs. Multi-engine |
| WA-11.2 | Install prompts | **Missing** [1E] | — | — | Searched `beforeinstallprompt`, `navigator.install`, "Web Install", `<install>`: no hits. Chromium-only |
| WA-11.3 | Launch Handler and File Handling | **Missing** [1E] | — | — | Searched `launch_handler`, `launchQueue`, `file_handlers` (only extension manifest keys, EX Stage 3): no web design. Chromium-only |
| WA-11.4 | Protocol handlers | **Partial** | HTML claim (custom scheme handlers); SH 6.4 (external-protocol prompt, outgoing); EX (`protocol_handlers` key, Stage 3) | — | Missing: design for web-registered handlers (`registerProtocolHandler`), their prompt class and OS registration. Two engines |
| WA-11.5 | Web Share Target | **Missing** [1E] | — | — | Searched `share_target`, "Share Target": no hits. Chromium Android/ChromeOS |
| WA-11.6 | Badging API | **Missing** | — | — | Searched `setAppBadge`, Badging, "app badge": no hits. Two engines |
| WA-11.7 | Window Controls Overlay | **Missing** [1E] | — | — | Searched "Window Controls Overlay", "title bar area": no hits. Chromium-only |
| WA-11.8 | Window Management | **Missing** [1E] | — | — | Searched `getScreenDetails`, "Window Management", "multi-screen"; MD refuses only the API's HDR-headroom extension. Chromium-only |
| WA-11.9 | Document Picture-in-Picture | **Excluded** | Deferred (Stage 2): S1 13.2 ("picture-in-picture"); SHUI | Stage 2 | SHUI: "Document PiP are shell-owned windows"; no API design. Two engines |
| WA-11.10 | Isolated Web Apps and their APIs | **Excluded** | DV 6.3, 6.10; S1 §6 | — | "Isolated Web Apps are not a Stage 1 or Stage 2 principal" |
| WA-11.11 | Get Installed Related Apps | **Missing** [1E] | — | — | Searched `getInstalledRelatedApps`, "related apps": no hits. Chromium-only |
| WA-11.12 | PWA origin migration | **Missing** [1E] | — | — | Searched "origin migration", "manifest migration": no hits. Chromium-only; depends on WA-11.1 |

### 12. Permissions, clipboard, sharing and other user-mediated APIs

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-12.1 | Permissions API | **Covered** | DM 6.18 (Permissions row); S1 13.2 (permission store); SH 6.4 (prompts); DV 6.10 | S1: design before M6 | — |
| WA-12.2 | Permissions Policy | **Partial** | Consumed by DV 6.10, MD-D13, MD (`speaker-selection`), AX 6.7 (`aria-notify`); CB cites the spec | — | Missing: an owner for `Permissions-Policy` header parsing, `allow` inheritance and the feature registry; `document.featurePolicy` not named |
| WA-12.3 | Document Policy | **Missing** [1E] | — | — | Searched `Document-Policy`, "Document Policy": no hits. Chromium-only |
| WA-12.4 | Async Clipboard | **Covered** | ED 6.10 (ED-D10, ED-D11); DM 6.18 | ED-M1/M2 | Paste button drawn by the prompt renderer |
| WA-12.5 | Clipboard events and custom formats | **Covered** | ED 6.10 (ED-D12); `clipboardchange` held (ED 9.1) | ED-M2 | — |
| WA-12.6 | Web Share | **Missing** | — | — | Searched `navigator.share`, "Web Share", `canShare`: no hits. Chromium, WebKit and Firefox Android ship it, so it meets the admission rule |
| WA-12.7 | Contact Picker | **Missing** [1E] | — | — | Searched "Contact Picker", `contacts.select`: no hits. Chromium Android only |
| WA-12.8 | EyeDropper | **Missing** [1E] | — | — | Searched EyeDropper, "eye dropper", "colour from the screen": no hits. Chromium-only |
| WA-12.9 | Local Font Access | **Excluded** | TX-D17 | — | "Local Font Access is not implemented" |
| WA-12.10 | Web Speech API | **Missing** | — | — | Searched `speechSynthesis`, `SpeechRecognition`, "Web Speech", "text-to-speech": no API hits; ML §1 names speech recognition only as an ML captions feature (ML-ASR). Synthesis ships in three engines |
| WA-12.11 | Virtual Keyboard API | **Excluded** | Held by admission rule: ED-D18 | When admitted | "VirtualKeyboard API and `interactive-widget` held for the admission rule" |
| WA-12.12 | Selective permissions intervention | **Missing** [1E] | — | — | Searched "selective permissions", "ad script permission": no hits; CB's interventions do not include it. Chromium browser policy |

### 13. Security and isolation (page-visible)

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-13.1 | Same-origin policy and `document.domain` | **Covered** | DM 6.8 (DM-D6) | DM-M2 | `document.domain` a no-op; `Origin-Agent-Cluster: ?0` ignored |
| WA-13.2 | Content Security Policy Level 3 | **Partial** | NW 6.7 (renderer pre-request checks including `<meta>`; network-side fetch directives); ST (CSP for style requests, nonce hidden); JS 6.14 (`HostEnsureCanCompileStrings`) | NW-M2 | Missing: one owner for CSP's document-side directives (inline script and style checks, `unsafe-eval` beyond Trusted Types, `frame-ancestors`, `sandbox`, reporting); NW pins only "CSP (fetch directives)" |
| WA-13.3 | Trusted Types | **Covered** | DM 6.16; JS 6.14; ED (`insertHTML` sink) | DM-M2 | "Trusted Types enforced at every sink the bindings generate" |
| WA-13.4 | Cross-origin isolation | **Covered** | NW 6.7 (CORP, COEP including `credentialless`); DM 6.8 (COOP, isolation modes) | NW-M2; DM-M2 | — |
| WA-13.5 | Document-Isolation-Policy | **Missing** [1E] | — | — | Searched `Document-Isolation-Policy`, "document isolation": no hits. Chromium-only |
| WA-13.6 | Fetch Metadata | **Covered** | NW 6.7 | NW-M1/M2 | — |
| WA-13.7 | Referrer Policy | **Covered** | NW 6.7 | NW-M1/M2 | — |
| WA-13.8 | Mixed content and HTTPS upgrades | **Covered** | NW 6.7 (HTTPS-First, mixed-content autoupgrade), NW 6.5.3 (HSTS preload) | NW-M2 | — |
| WA-13.9 | Secure contexts | **Covered** | DM 6.3 (`[SecureContext]` in descriptors); DV 6.10; HTML claim | DM-M1 | `isSecureContext` not named |
| WA-13.10 | iframe sandbox and X-Frame-Options | **Covered** | DM 6.9 (sandbox flags decided in the broker); XFO by HTML claim | DM-M2 | `X-Frame-Options` not named |
| WA-13.11 | Clear-Site-Data | **Covered** | NW 6.8 (NW-D18); SG 6.9 | NW-M2; SG-M3 | Container-scoped |
| WA-13.12 | Local Network Access | **Covered** | NW 6.1, 6.7 (NW-D4 tickets); NW 6.11 | NW-M2 | Applies to WebSockets and WebTransport from the start |
| WA-13.13 | Connection Allowlists | **Missing** [1E] | — | — | Searched "Connection Allowlist", "connection-allowlist", "allowed network destinations": no hits. Chromium-only |
| WA-13.14 | Web Cryptography | **Covered** | WC spec (WC-D1, WC-D2) | WC milestones | — |

### 14. Privacy, tracking protection and advertising APIs

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-14.1 | Storage Access API | **Covered** | NW 6.8, 6.9 (NW-D16); DM 6.5 (activation) | NW-M2 | Prompt only, no heuristics; Storage Access headers not named |
| WA-14.2 | `requestStorageAccessFor` and Related Website Sets | **Excluded** | NW 6.9 | — | "Related Website Sets is dead and not implemented" |
| WA-14.3 | Storage and cookie partitioning | **Covered** | NW 6.8, 6.9; S1 §6 | NW-M2 | Third-party cookies partitioned by (T, C, X) |
| WA-14.4 | Bounce tracking mitigations | **Covered** | NW 6.9; CB-D8 | NW-M4 (CB) | — |
| WA-14.5 | Topics API | **Missing** [1E] | No decision recorded | — | Searched "Topics API", "Privacy Sandbox": NW §5 has a lesson row only; SS 6.15 defers "the retired Privacy Sandbox APIs" to "a privacy-spec question, out of scope here", and no privacy spec exists. Retired |
| WA-14.6 | Protected Audience | **Missing** [1E] | No decision recorded | — | Searched "Protected Audience", FLEDGE, "interest group": no hits (SS 6.15 deferral as WA-14.5) |
| WA-14.7 | Shared Storage and selectURL | **Missing** [1E] | No decision recorded | — | Searched "Shared Storage", `selectURL`: no hits |
| WA-14.8 | Private Aggregation | **Missing** [1E] | No decision recorded | — | Searched "Private Aggregation": no hits. Removed in Chrome 152 |
| WA-14.9 | Attribution Reporting | **Missing** [1E] | No decision recorded | — | Searched "Attribution Reporting", PATWG, "conversion measurement": no hits. An interoperable W3C successor is in progress |
| WA-14.10 | Private Click Measurement | **Missing** [1E] | No decision recorded | — | Searched "Private Click Measurement": no hits. WebKit-only |
| WA-14.11 | Private State Tokens | **Missing** [1E] | No decision recorded | — | CB §2 history row only ("CHIPS, FedCM and Private State Tokens survived"). Chromium-only |
| WA-14.12 | Private Verification Tokens | **Missing** [1E] | No decision recorded | — | Searched "Private Verification", "trust token": no hits. Origin trial |
| WA-14.13 | Global Privacy Control | **Covered** | S1 §6 (GPC, Stage 1, sent by default); NW 6.7 (`Sec-GPC: 1`) | Stage 1 | `navigator.globalPrivacyControl` not named |
| WA-14.14 | User-Agent reduction and string freezes | **Covered** | S1 13.2 (UA, UA-1); NW-D12; DM 6.16 | NW-M1 (UA by measurement) | — |
| WA-14.15 | Fingerprinting protections | **Covered** | S1 §6 (D-S1, OD-6); DM 6.16; SS 6.15; PRIV-2 | Stage 1 baseline, Stage 2 full | — |
| WA-14.16 | IP Protection | **Excluded** | S1 §6 out-of-scope list; NW 6.13 (routes are user-supplied) | — | Out of scope: "a browser-operated VPN service"; the refusal names the class, not this feature |

### 15. Identity, credentials and payments

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-15.1 | Credential Management | **Missing** | — | — | Searched "Credential Management", `navigator.credentials`, `PasswordCredential`, `FederatedCredential`: no hits. WebAuthn (WA-15.2) and Digital Credentials need this container |
| WA-15.2 | Web Authentication (passkeys) | **Partial** | S1 §6 ("WebAuthn through platform authenticators", Stage 1); SS SH-R1, SEC-23; SH 6.4 (credential prompt class); DV 6.14 | Stage 1 (no milestone) | Missing: the API and per-OS authenticator-bridge design (conditional mediation, hybrid transport, PRF and other extensions, related origins, Signal API) |
| WA-15.3 | FedCM | **Missing** [1E] | No decision recorded | — | SS 6.15 and SS 11.4: "Out of scope: a privacy-spec question"; otherwise CB history, CB login-flow exceptions and DT redaction mention it. Chromium-only, kept by Google |
| WA-15.4 | Digital Credentials API | **Missing** | — | — | Searched "Digital Credentials", mdoc, OpenID4VP, wallet (hits are hardware wallets): no hits. Two engines |
| WA-15.5 | WebOTP | **Missing** [1E] | — | — | Searched WebOTP, "one-time code" (only ED/DT credential-mode redaction): no API design. Chromium Android |
| WA-15.6 | Device Bound Session Credentials | **Missing** [1E] | — | — | Searched "Device Bound Session", DBSC: no hits. Chromium-only |
| WA-15.7 | Email Verification Protocol | **Missing** [1E] | — | — | Searched "Email Verification": no hits. Origin trial |
| WA-15.8 | Payment Request API | **Partial** | DV 6.14 ("Payments stay in Payment Request"); DM 6.5 (payment among activation-gated capabilities); SH 6.9 (payment autofill Stage 2) | — | Mentioned only: no design, owner or decision. Two engines |
| WA-15.9 | Payment Handler | **Partial** [1E] | DT 6.5 (redacts "payment-handler results") | — | One mention in a redaction list; no design or decision. Chromium-only |
| WA-15.10 | Secure Payment Confirmation | **Missing** [1E] | — | — | Searched "Secure Payment Confirmation", SPC: no hits. Chromium-only |
| WA-15.11 | Apple Pay JS and merchant validation | **Missing** [1E] | — | — | Searched "Apple Pay", `ApplePaySession`, "merchant validation": no hits. WebKit vendor API |

### 16. Device and hardware APIs

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-16.1 | Geolocation | **Partial** | DM 6.18 (DOM side, activation checks); S1 13.2 (location permission); SH 6.4 (capability class) | — | Engine side named only as "Broker stores, prompts and OS integration (shell, security scope)". Missing: the location-provider design (OS APIs, accuracy and coarsening, recording) |
| WA-16.2 | Device orientation and motion events | **Missing** | — | — | Searched `deviceorientation`, `devicemotion`, "device motion": no hits. Three engines |
| WA-16.3 | Generic Sensor family | **Missing** [1E] | — | — | Searched "Generic Sensor", Accelerometer, Gyroscope, Magnetometer, "Ambient Light": no hits. Chromium-only |
| WA-16.4 | Battery Status | **Missing** [1E] | — | — | FP §2 mentions its removal from Firefox as history only; no decision. Chromium-only |
| WA-16.5 | Device Memory | **Covered** | DM 6.16 (`deviceMemory` in the normalized environment-values record) | DM-M2 | Chromium-only value exposed with no recorded admission exception; the `Device-Memory` hint is not covered |
| WA-16.6 | Compute Pressure | **Missing** [1E] | — | — | Searched "Compute Pressure", `PressureObserver`: no hits. Chromium-only |
| WA-16.7 | CPU Performance API | **Missing** [1E] | — | — | Searched "CPU Performance", `cpu-performance`: no hits. Chromium-only |
| WA-16.8 | Screen Wake Lock | **Partial** | DM 6.18 (DOM side) | — | Listed in the DOM-side row only. Missing: OS integration, permission and footprint design. Baseline |
| WA-16.9 | Vibration | **Missing** | — | — | Searched `vibrate`, Vibration, haptic: no web-API hits. Two engines (mobile; Android is Stage 2) |
| WA-16.10 | Screen Orientation | **Missing** | — | — | Searched "Screen Orientation", `screen.orientation`, "orientation lock": no hits. Three engines |
| WA-16.11 | Idle Detection | **Missing** [1E] | — | — | AH 6.8 lists "idle detection" only in the app host's permission registry. Chromium-only |
| WA-16.12 | Gamepad | **Partial** | DV 6.14 ("Gamepads stay with the Gamepad API") | — | Mentioned only. Missing: OS input backends, permission and fingerprinting design. Baseline widely available |
| WA-16.13 | Web MIDI | **Covered** | DV 6.6; DM 6.18 | DV-M2 behind the Experimental setting; DV-E5 at DV-M3 | — |
| WA-16.14 | Web Serial | **Covered** | DV 6.5; DM 6.18 | DV-M2; DV-E6 at DV-M3 | — |
| WA-16.15 | WebUSB | **Covered** | DV 6.3; DM 6.18 | DV-M2; DV-E1 at DV-M4 | — |
| WA-16.16 | WebHID | **Covered** | DV 6.4; DM 6.18 | DV-M2; DV-E2 at DV-M4 | — |
| WA-16.17 | Web Bluetooth | **Covered** | DV 6.7; DM 6.18 | DV-M2 (GATT only); DV-E3 at DV-M4 | — |
| WA-16.18 | Web NFC | **Covered** | DV 6.8 (DV-D9); DM 6.18 | Built, not shipped; Stage 3 if DV-E4 | Android-only |
| WA-16.19 | Device Posture | **Missing** [1E] | — | — | Searched `devicePosture`, "Device Posture", foldable: no hits. Chromium, Safari flag |
| WA-16.20 | Viewport Segments | **Missing** [1E] | — | — | Searched "Viewport Segments", "dual-screen": no hits. Chromium, Safari flag |
| WA-16.21 | Shape Detection | **Missing** [1E] | — | — | Searched `BarcodeDetector`, `FaceDetector`, "Shape Detection": no hits. Chromium-only |
| WA-16.22 | Presentation API | **Missing** [1E] | — | — | Searched "Presentation API", "second screen": no hits. Chromium-only |
| WA-16.23 | Input Device Capabilities | **Missing** [1E] | — | — | Searched `InputDeviceCapabilities`, `sourceCapabilities`, `firesTouchEvents`: no hits. Chromium-only |
| WA-16.24 | Managed device attributes | **Missing** [1E] | — | — | Searched "Device Attributes", "managed device" (unrelated hits only). ChromeOS managed only |

### 17. Performance, measurement and reporting

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-17.1 | High Resolution Time | **Covered** | DM 6.7 (DM-D10) | DM-M1 | 100 µs / 5 µs coarsening with recorded jitter |
| WA-17.2 | Performance Timeline | **Covered** | DM 6.18 | — | — |
| WA-17.3 | Navigation Timing 2 | **Covered** | DM 6.18 ("Navigation") | — | `confidence` not named |
| WA-17.4 | Resource Timing 3 | **Covered** | DM 6.18; NW 6.8 (coarsened timing) | — | — |
| WA-17.5 | User Timing 3 | **Covered** | DM 6.18 | — | — |
| WA-17.6 | Server Timing | **Missing** | — | — | Searched `Server-Timing`, "Server Timing", `serverTiming`: no hits. Baseline |
| WA-17.7 | Paint Timing | **Covered** | DM 6.7 ("mark paint timing"); DM 6.18 | — | — |
| WA-17.8 | Largest Contentful Paint | **Partial** | LY 6.1 (LCP among clock reads); MD (LCP render times as a timing channel); DM 6.18 (Performance Timeline family) | — | DM 6.18's row names User, Navigation, Resource, Event, Paint, Element Timing and LoAF, not LCP. Missing: an owner for candidate selection and the entry. Baseline (December 2025) |
| WA-17.9 | Event Timing and INP | **Covered** | DM 6.18 ("Event"); DM 6.7 | — | — |
| WA-17.10 | Layout Instability | **Covered** | LY 6.15 (owner exception, LY-D10) | LY milestones | — |
| WA-17.11 | Long Tasks and Long Animation Frames | **Covered** | DM-D15 (LoAF exception); DM 6.7 | — | `longtask` entries not named |
| WA-17.12 | Element Timing and Container Timing | **Partial** [1E] | DM 6.18 lists Element Timing | — | Element Timing is Chromium-only and DM-D15 records no exception for it (an inconsistency); Container Timing not covered |
| WA-17.13 | Soft navigations | **Missing** [1E] | — | — | Only ESS1's fact-check row mentions the API. Chromium-only |
| WA-17.14 | Memory and profiling | **Missing** [1E] | — | — | Searched `measureUserAgentSpecificMemory`, "JS Self-Profiling", `Profiler` (hits are the DevTools profiler, not a page API). Chromium-only |
| WA-17.15 | Reporting API | **Covered** | NW 6.11 ("The Reporting API's other report types are implemented"); DM 6.18; FP 6.13 (intervention reports) | — | — |
| WA-17.16 | Network Error Logging | **Excluded** | NW-D15 | — | "Network Error Logging (NW-D15) is not implemented" |
| WA-17.17 | Crash Report API | **Partial** [1E] | FP 6.7 (WICG crash reporting: `oom` reports only when the site's own process crossed the threshold) | — | `window.crashReport` key-value context not covered |
| WA-17.18 | Scroll performance timing | **Missing** [1E] | — | — | Searched "scroll performance", "scroll smoothness": no hits. No spec exists yet |

### 18. ECMAScript language

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-18.1 | ES5 core and Annex B | **Covered** | JS-1 (test262 for Stage 4 features); JS 6.13 (Annex B regex); JS-5 | JS-M1/M2 | — |
| WA-18.2 | ES2015 syntax | **Covered** | ES2026 claim (JS-M2, JS-1) | JS-M2 | — |
| WA-18.3 | ES2015 library | **Covered** | ES2026 claim | JS-M2 | — |
| WA-18.4 | Proper tail calls | **Partial** | ES2026 claim (JS-1) | — | No decision on PTC, which two engines refuse; JS-5's exclusion list names only Stage 2.7 features and unshipped Annex B items. Interacts with the debugger and JS-D19's stack segments |
| WA-18.5 | Modules | **Covered** | JS 6.14 | JS-M1 | — |
| WA-18.6 | Async functions and async iteration | **Covered** | ES2026 claim | JS-M2 | — |
| WA-18.7 | ES2018–2021 additions | **Covered** | ES2026 claim | JS-M2 | — |
| WA-18.8 | Weak references | **Covered** | JS 6.5 (WeakRef processing model) | JS-M2 | Cleanup scheduled by the host |
| WA-18.9 | ES2022 classes | **Covered** | ES2026 claim | JS-M2 | — |
| WA-18.10 | ES2022–2024 library | **Covered** | ES2026 claim | JS-M2 | — |
| WA-18.11 | Shared memory | **Covered** | JS 6.12; DM 6.7 (`Atomics.waitAsync` decision point) | JS-M2 | — |
| WA-18.12 | Resizable and transferable ArrayBuffers | **Covered** | ES2026 claim; DM 6.11 (resizable buffers in clone) | JS-M2 | — |
| WA-18.13 | Iterator helpers | **Covered** | ES2026 claim | JS-M2 | — |
| WA-18.14 | Iterator sequencing, joint iteration and new methods | **Partial** | ES2026 claim for `Iterator.concat`; others under JS-1's Stage 4 scope | JS-M2 (concat) | None named; the stages of `zip`, `join`, `includes` and `chunks`/`windows` are not recorded, and the reports give no rule for Stage 3 proposals that ship in two engines |
| WA-18.15 | Set methods | **Covered** | ES2026 claim | JS-M2 | — |
| WA-18.16 | ES2025 batch | **Covered** | ES2026 claim; JS 6.14 (import attributes) | JS-M2 | — |
| WA-18.17 | Explicit resource management | **Excluded** | Deferred until TC39 Stage 4: JS-1, JS-70 | When Stage 4 | Not named; ships in two engines; stage not recorded in the reports |
| WA-18.18 | Upsert | **Covered** | JS-1/JS-70 Stage 4 scope | JS-70: the June edition after Stage 4 | Not named; Stage 4 assumed from three-engine Baseline |
| WA-18.19 | `Math.sumPrecise` | **Covered** | JS-1/JS-70 Stage 4 scope | JS-70 | Not named; Stage 4 assumed |
| WA-18.20 | Uint8Array base64/hex | **Covered** | JS-1/JS-70 Stage 4 scope | JS-70 | Not named; Stage 4 assumed |
| WA-18.21 | `Error.isError`, stack APIs | **Partial** | JS-1 Stage 4 scope (`Error.isError`) | — | Non-standard `Error.captureStackTrace` and `Error.stackTraceLimit` (web-compat surface) not covered |
| WA-18.22 | `Array.fromAsync` | **Covered** | JS-1/JS-70 Stage 4 scope | JS-70 | Not named; Stage 4 assumed |
| WA-18.23 | JSON.parse source text access | **Covered** | JS-1/JS-70 Stage 4 scope | JS-70 | Not named; Stage 4 assumed, unconfirmed (WebKit status unknown) |
| WA-18.24 | `Promise.allKeyed` / `allSettledKeyed` | **Excluded** | Deferred until TC39 Stage 4: JS-1 | When Stage 4 | Firefox-only; not named |
| WA-18.25 | Source phase imports and `import defer` | **Excluded** | Deferred until TC39 Stage 4: JS 6.14 (module state machine designed for them); Q-JS-12 | When Stage 4 | "When do `import defer` and source phase imports reach Stage 4" (Q-JS-12) |
| WA-18.26 | Not yet shipped proposals | **Excluded** | JS 6.14; JS 9.1 | When Stage 4 | "Not until Stage 4: decorators … and ShadowRealm"; AsyncContext, RegExp buffer boundaries and `export * default` not named |

### 19. Intl and Temporal

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-19.1 | Intl core | **Covered** | JS 6.13 (`hg-intl`, `hg-cldr-build`); JS-2 | JS-M3 (CLDR pipeline) | — |
| WA-19.2 | `Intl.Segmenter` | **Covered** | JS 6.13 (over `hg-unicode`) | JS-M3 | — |
| WA-19.3 | `Intl.DurationFormat` | **Covered** | ECMA-402 claim (JS-2) | JS-M3 | — |
| WA-19.4 | `Intl.Locale` info and `variants` | **Partial** | ECMA-402 claim (JS-2) | — | The Intl Locale Info proposal is not named and is outside ECMA-402 until merged; no stated rule for it |
| WA-19.5 | `Intl.PluralRules` notation | **Covered** | ECMA-402 claim (JS-2) | JS-M3 | — |
| WA-19.6 | Temporal | **Covered** | JS 6.14 ("Temporal is in, from its Stage 4 text"); JS 6.13 (`hg-tz`) | JS-M2 | — |

### 20. WebAssembly

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-20.1 | Wasm MVP and JS API | **Covered** | JS 6.6; JS-3 | JS-M2 (W0, JS API) | — |
| WA-20.2 | Wasm Web API | **Covered** | JS 6.6 (streaming compilation); JS-3 (`wasm/webapi/`) | JS-M2 | — |
| WA-20.3 | Post-MVP core features | **Covered** | JS-3 (Wasm 3.0) | JS-M2 | — |
| WA-20.4 | Threads | **Covered** | JS-3 ("Wasm 3.0 plus threads"); HGBA-2 4.2 | JS-M2 | — |
| WA-20.5 | Exception handling | **Covered** | JS 6.6 (`exnref`) | JS-M2 | — |
| WA-20.6 | GC and tail calls | **Covered** | JS 6.5, 6.6 | JS-M2 | — |
| WA-20.7 | Memory64 and multi-memory | **Covered** | JS 6.6 (memory64 explicit checks); JS-3 (Wasm 3.0) | JS-M2 | — |
| WA-20.8 | Relaxed SIMD | **Covered** | JS 4.2 (Wasm 3.0 includes relaxed SIMD); JS-3 | JS-M2 | — |
| WA-20.9 | JS String Builtins | **Covered** | JS 6.6; HGBA-2 4.2 | JS-M2 | "JS String Builtins operate on Izcal's strings directly" |
| WA-20.10 | JSPI | **Covered** | JS 6.6 (JS-D19); JS 9.1 | JS-M2 | "JSPI (phase 5) ships with W0 and W1" |
| WA-20.11 | Resizable memory buffers | **Covered** | JS-3 (WPT `wasm/jsapi/`) | JS-M2 | Not named; covered only if merged into the JS API at the pin |
| WA-20.12 | Newer proposals | **Excluded** | Held by the phase rule: JS 9.1; Q-JS-12 (ESM integration) | At Wasm phase 4 | "Stack Switching waits for Wasm phase 4 … Same rule as TC39 proposals"; wide arithmetic, compact imports, custom descriptors and branch hinting not named |

### 21. On-device AI and agent-facing APIs

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-21.1 | Translator and Language Detector | **Excluded** | Deferred (Stage 2): ML 6.5 (ML-D2); HGBA-2 4.5 | Stage 2 under the AI grant | "Translate, summarise, prompt … To be specified with the task APIs" |
| WA-21.2 | Summarizer, Writer, Rewriter, Proofreader | **Excluded** | Deferred (Stage 2): ML-D2; HGBA-2 4.5 | Stage 2 | As WA-21.1 |
| WA-21.3 | Prompt API | **Excluded** | Deferred (Stage 2): ML-D2; HGBA-3 B10 | Stage 2 | "the Prompt API does not exist with AI off" |
| WA-21.4 | WebNN | **Excluded** | HGBA-2 4.5; ML-D2 | Behind a flag | "WebNN stays behind a flag and is not shipped in Stage 1" |
| WA-21.5 | WebMCP | **Excluded** | Deferred (Stage 3): S1 §9; HGBA-3 B9, B10; DM 6.15 (inert surface) | Stage 3 | "WebMCP tool registry (if the standard survives)"; `registerTool` stays inert with AI off |

### 22. Browser-shipped protocols and infrastructure standards

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| WA-22.1 | WebDriver (classic) | **Covered** | DT 6.1 (`hg-bidi`: "WebDriver Classic and BiDi"); DT 6.5 (`navigator.webdriver` true) | S1 M2 slice; M7 | — |
| WA-22.2 | WebDriver BiDi | **Covered** | DT 6.1; NW 6.18 (interception) | M7 (DT-5) | — |
| WA-22.3 | WebDriver extension commands | **Partial** | DV 6.13 (virtual devices), ED 6.16 (virtual IME) as DevTools oracles | — | Missing: a plan for the spec-defined extension commands (set permission, WebAuthn virtual authenticators, Storage Access, FedCM, sensors) that WPT's testdriver uses |
| WA-22.4 | Chrome DevTools Protocol | **Covered** | DT (CDP profile); S1 D-S11, DT-D12 | M7 | Protocol description authored by Izcal |
| WA-22.5 | Browser MCP servers | **Covered** | DT (MCP adapter when AI is on); S1 D-S11 | Stage 2 (DT 9.1, with B9) | — |
| WA-22.6 | Web IDL | **Covered** | DM 6.3 (DM-D2, DM-D21) | DM-M0 | — |
| WA-22.7 | Infra | **Covered** | Implied by DM-D21's pinned specifications | n/a | No runtime surface; not named |
| WA-22.8 | Console | **Covered** | JS 6.10 (evaluation and console, JS-63); DT 9.1 (Console panel) | JS-M1; DT M6 | Console Standard not cited by name |
| WA-22.9 | Compatibility and Quirks Mode | **Partial** | ST (quirks, limited-quirks and no-quirks modes; quirks UA sheet) | ST milestones | Missing: the Compat Standard surface (`-webkit-` aliases, `window.orientation`, webkit event aliases); the Interop 2026 web-compat area is cited only in HGBA-3 §3 |
| WA-22.10 | Test Utils | **Missing** | — | — | Searched `TestUtils`, testutils, `gc()`: no hits. Test builds only |
| WA-22.11 | WebExtensions | **Covered** | EX spec | Stage 2 (S1 §6, D-S3) | — |
| WA-22.12 | Origin trials | **Missing** | — | — | Searched "origin trial" (only WebNN and WebMCP status notes), "trial token", "per-origin token": no decision on running or honouring trial tokens; the admission rule has no trial mechanism |

## Summary

### Counts per status per group

| Group | Rows | Covered | Partial | Excluded | Missing | of which [1E] Missing |
|---|---|---|---|---|---|---|
| 1. HTML: documents, browsing contexts, navigation and history | 19 | 15 | 2 | 1 | 1 | 1 |
| 2. HTML: elements, attributes and document metadata | 18 | 14 | 0 | 1 | 3 | 3 |
| 3. HTML: forms and form controls | 13 | 8 | 1 | 0 | 4 | 2 |
| 4. HTML: interactive UI, focus, dialogs and drag and drop | 18 | 14 | 0 | 1 | 3 | 2 |
| 5. HTML: scripting, modules and resource loading | 13 | 8 | 0 | 1 | 4 | 2 |
| 6. DOM, Web Components and DOM-adjacent APIs | 26 | 22 | 2 | 1 | 1 | 1 |
| 7. Event loop, timers, scheduling, workers and messaging | 15 | 14 | 0 | 1 | 0 | 0 |
| 8. Fetch, networking and data formats | 20 | 13 | 3 | 1 | 3 | 2 |
| 9. Storage, files and quota | 12 | 7 | 1 | 2 | 2 | 1 |
| 10. Service workers, background work, push and notifications | 11 | 4 | 0 | 4 | 3 | 3 |
| 11. Installable web apps and OS integration | 12 | 0 | 1 | 2 | 9 | 7 |
| 12. Permissions, clipboard, sharing and other user-mediated APIs | 12 | 3 | 1 | 2 | 6 | 4 |
| 13. Security and isolation (page-visible) | 14 | 11 | 1 | 0 | 2 | 2 |
| 14. Privacy, tracking protection and advertising APIs | 16 | 6 | 0 | 2 | 8 | 8 |
| 15. Identity, credentials and payments | 11 | 0 | 3 | 0 | 8 | 6 |
| 16. Device and hardware APIs | 24 | 7 | 3 | 0 | 14 | 11 |
| 17. Performance, measurement and reporting | 18 | 10 | 3 | 1 | 4 | 3 |
| 18. ECMAScript language | 26 | 19 | 3 | 4 | 0 | 0 |
| 19. Intl and Temporal | 6 | 5 | 1 | 0 | 0 | 0 |
| 20. WebAssembly | 12 | 11 | 0 | 1 | 0 | 0 |
| 21. On-device AI and agent-facing APIs | 5 | 0 | 0 | 5 | 0 | 0 |
| 22. Browser-shipped protocols and infrastructure standards | 12 | 8 | 2 | 0 | 2 | 0 |
| **Total** | **333** | **199** | **27** | **30** | **77** | **58** |

### Missing rows (77)

**Multi-engine or Baseline: real gaps (19).** Other engines ship these, and the admission rule would admit them. No spec mentions them. WA-22.10 (TestUtils) and WA-22.12 (origin trials) are test and process items, not page features.

- WA-3.8 Directory upload
- WA-3.9 HTML Media Capture
- WA-4.15 Pointer Lock
- WA-5.9 No-Vary-Search
- WA-5.12 Integrity Policy
- WA-8.16 URL Pattern
- WA-9.11 File and Directory Entries API
- WA-11.1 Web App Manifest
- WA-11.6 Badging API
- WA-12.6 Web Share
- WA-12.10 Web Speech API
- WA-15.1 Credential Management
- WA-15.4 Digital Credentials API
- WA-16.2 Device orientation and motion events
- WA-16.9 Vibration
- WA-16.10 Screen Orientation
- WA-17.6 Server Timing
- WA-22.10 Test Utils
- WA-22.12 Origin trials

**Single-engine or retired [1E] (58).** The admission rule would hold these, so each needs only a one-line recorded decision (held, refused or admitted) in the owning spec's admission table, not a design.

WA-1.18 Out-of-order HTML streaming, WA-2.3 Heading level adjustment, WA-2.14 Processing instructions in HTML, WA-2.16 `<model>` element, WA-3.10 `<input type=checkbox switch>`, WA-3.13 OpaqueRange, WA-4.11 Focusgroup, WA-4.16 Keyboard Lock and Keyboard Map, WA-5.11 Web bundles and signed exchanges, WA-5.13 Signature-based integrity, WA-6.23 Observable API, WA-8.9 WebSocketStream, WA-8.20 Network Information and Save-Data, WA-9.10 File System Observer, WA-10.6 Declarative Web Push, WA-10.9 Periodic Background Sync, WA-10.11 Content Index, WA-11.2 Install prompts, WA-11.3 Launch Handler and File Handling, WA-11.5 Web Share Target, WA-11.7 Window Controls Overlay, WA-11.8 Window Management, WA-11.11 Get Installed Related Apps, WA-11.12 PWA origin migration, WA-12.3 Document Policy, WA-12.7 Contact Picker, WA-12.8 EyeDropper, WA-12.12 Selective permissions intervention, WA-13.5 Document-Isolation-Policy, WA-13.13 Connection Allowlists, WA-14.5 Topics API, WA-14.6 Protected Audience, WA-14.7 Shared Storage and selectURL, WA-14.8 Private Aggregation, WA-14.9 Attribution Reporting, WA-14.10 Private Click Measurement, WA-14.11 Private State Tokens, WA-14.12 Private Verification Tokens, WA-15.3 FedCM, WA-15.5 WebOTP, WA-15.6 Device Bound Session Credentials, WA-15.7 Email Verification Protocol, WA-15.10 Secure Payment Confirmation, WA-15.11 Apple Pay JS and merchant validation, WA-16.3 Generic Sensor family, WA-16.4 Battery Status, WA-16.6 Compute Pressure, WA-16.7 CPU Performance API, WA-16.11 Idle Detection, WA-16.19 Device Posture, WA-16.20 Viewport Segments, WA-16.21 Shape Detection, WA-16.22 Presentation API, WA-16.23 Input Device Capabilities, WA-16.24 Managed device attributes, WA-17.13 Soft navigations, WA-17.14 Memory and profiling, WA-17.18 Scroll performance timing.

### Partial rows (27) and what is missing

- **WA-1.8 Credentialless iframes** [1E]: Partition key has "per-navigation nonce (credentialless iframes, fenced frames, recordings)". Missing: the `credentialless` attribute's DOM and navigation design and an admission decision (Chromium-only; LY-D10 would hold it)
- **WA-1.9 Fenced frames** [1E]: Named only in the partition-key notation. Missing: any element or navigation design, or a refusal; tied to Protected Audience (WA-14.6)
- **WA-3.5 Customizable `<select>`**: Element and AX mapping specified; rendering held by LY-D10. The inventory shows two engines now (Chrome 135, Safari 27.0), so the hold's trigger looks met and the decision needs revisiting
- **WA-6.11 Reference Target** [1E]: AX: "Reference Target redirects inward through hosts". Missing: DM binding and an admission decision (Chromium-only)
- **WA-6.15 XPath and XML documents**: XPath interfaces are in scope by the DOM claim, but no XPath evaluator has a design or owner (searched XPath, `document.evaluate`: hits are only CB/ST "not supported" for filters)
- **WA-8.2 Fetch upload streaming and ranges**: Missing: a stated design for `duplex: "half"` streaming uploads and the HTTP-version rule; an Interop 2026 area
- **WA-8.7 Beacon**: Missing: `navigator.sendBeacon` is not in DM 6.18's table, which says any API not in it "is recorded in the ledger as a gap"
- **WA-8.19 Client Hints** [1E]: Missing: `Accept-CH` negotiation, which `Sec-CH-UA-*` headers are sent, and `navigator.userAgentData`. Chromium-only
- **WA-9.9 File System Access**: Missing: the picker and handle API (`showOpenFilePicker` and the rest) is not in DM 6.18's table; S1 13.2 says the permission design is "written before M6". Chromium-only, adopted by S1 13.2
- **WA-11.4 Protocol handlers**: Missing: design for web-registered handlers (`registerProtocolHandler`), their prompt class and OS registration. Two engines
- **WA-12.2 Permissions Policy**: Missing: an owner for `Permissions-Policy` header parsing, `allow` inheritance and the feature registry; `document.featurePolicy` not named
- **WA-13.2 Content Security Policy Level 3**: Missing: one owner for CSP's document-side directives (inline script and style checks, `unsafe-eval` beyond Trusted Types, `frame-ancestors`, `sandbox`, reporting); NW pins only "CSP (fetch directives)"
- **WA-15.2 Web Authentication (passkeys)**: Missing: the API and per-OS authenticator-bridge design (conditional mediation, hybrid transport, PRF and other extensions, related origins, Signal API)
- **WA-15.8 Payment Request API**: Mentioned only: no design, owner or decision. Two engines
- **WA-15.9 Payment Handler** [1E]: One mention in a redaction list; no design or decision. Chromium-only
- **WA-16.1 Geolocation**: Engine side named only as "Broker stores, prompts and OS integration (shell, security scope)". Missing: the location-provider design (OS APIs, accuracy and coarsening, recording)
- **WA-16.8 Screen Wake Lock**: Listed in the DOM-side row only. Missing: OS integration, permission and footprint design. Baseline
- **WA-16.12 Gamepad**: Mentioned only. Missing: OS input backends, permission and fingerprinting design. Baseline widely available
- **WA-17.8 Largest Contentful Paint**: DM 6.18's row names User, Navigation, Resource, Event, Paint, Element Timing and LoAF, not LCP. Missing: an owner for candidate selection and the entry. Baseline (December 2025)
- **WA-17.12 Element Timing and Container Timing** [1E]: Element Timing is Chromium-only and DM-D15 records no exception for it (an inconsistency); Container Timing not covered
- **WA-17.17 Crash Report API** [1E]: `window.crashReport` key-value context not covered
- **WA-18.4 Proper tail calls**: No decision on PTC, which two engines refuse; JS-5's exclusion list names only Stage 2.7 features and unshipped Annex B items. Interacts with the debugger and JS-D19's stack segments
- **WA-18.14 Iterator sequencing, joint iteration and new methods**: None named; the stages of `zip`, `join`, `includes` and `chunks`/`windows` are not recorded, and the reports give no rule for Stage 3 proposals that ship in two engines
- **WA-18.21 `Error.isError`, stack APIs**: Non-standard `Error.captureStackTrace` and `Error.stackTraceLimit` (web-compat surface) not covered
- **WA-19.4 `Intl.Locale` info and `variants`**: The Intl Locale Info proposal is not named and is outside ECMA-402 until merged; no stated rule for it
- **WA-22.3 WebDriver extension commands**: Missing: a plan for the spec-defined extension commands (set permission, WebAuthn virtual authenticators, Storage Access, FedCM, sensors) that WPT's testdriver uses
- **WA-22.9 Compatibility and Quirks Mode**: Missing: the Compat Standard surface (`-webkit-` aliases, `window.orientation`, webkit event aliases); the Interop 2026 web-compat area is cited only in HGBA-3 §3

### Inconsistencies found while mapping

- **Prerender stage conflict (WA-5.8).** DM-M3 lists prerendering in Stage 1 (S1 M7), but S1 §9 lists "Speculation Rules and prerender" as Stage 2.
- **Single-engine APIs admitted without a recorded exception.** DM 6.18 lists Element Timing (WA-17.12), and DM 6.16's environment record exposes `deviceMemory` (WA-16.5). Both ship only in Chromium, but DM-D15's exception list names only Page Lifecycle, LoAF and `notRestoredReasons`.
- **A hold whose trigger looks met (WA-3.5).** LY-D10 holds `appearance: base-select`, but the inventory shows customizable `<select>` in Chrome 135 and Safari 27.0.
- **A deferral to a spec that does not exist.** SS 6.15 and SS 11.4 send "CHIPS and FedCM versus the retired Privacy Sandbox APIs" to "a privacy-spec question, out of scope here". No privacy spec is in `reports/`, so WA-14.5 to WA-14.12 and WA-15.3 have no decision anywhere.
- **Notifications and Push are both Stage 2, through different decisions.** Notifications are deferred by SH-D17, and Push and background sync by S1 13.2. Neither puts `PushManager` or the background-sync APIs in DM 6.18's table.

### Proposed grouping of Missing and Partial rows

| Candidate | Rows | Why |
|---|---|---|
| New spec: installable web apps | WA-11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8, 11.11, 11.12, 10.11 (and the Stage 2 deferrals 10.5, 10.8, 10.9, 10.6, 11.9 when they are designed) | Nothing owns the manifest, install, launch or OS-integration surface. AH 6.1 rejects installed web apps as an app-host design but leaves PWAs undecided, and push and background work belong with installability. |
| New spec: identity, credentials and payments | WA-15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7, 15.8, 15.9, 15.10, 15.11 | WebAuthn is a one-line Stage 1 commitment (S1 §6) with no API or authenticator-bridge design. `navigator.credentials`, FedCM, Digital Credentials and Payment Request share one container, one browser-mediated chooser and the OS credential stores. |
| New spec: privacy (the one SS 6.15 already assumes) | WA-14.5, 14.6, 14.7, 14.8, 14.9, 14.10, 14.11, 14.12, 8.19, 8.20, 16.4, 12.12; FedCM's privacy decision (15.3) with the identity spec | SS 6.15 defers the retired Privacy Sandbox APIs to a privacy spec. One document can refuse the retired ad APIs, decide the anti-fraud tokens and the successor attribution standard, and fix which client hints and fingerprintable signals are exposed. |
| New spec or devices amendment: sensors, platform input and screen | WA-16.1, 16.2, 16.3, 16.6, 16.7, 16.8, 16.9, 16.10, 16.11, 16.12, 16.19, 16.20, 16.21, 16.22, 16.23, 16.24, 4.15, 4.16, 12.7, 12.8 | DV §3.5 limits the devices spec to six APIs and calls a wider device-class API "a new surface with its own spec". Gamepad, sensors, orientation, wake lock, Pointer Lock and Keyboard Lock need the same OS backends, permission grants, recording and fingerprinting rules. |
| DM amendment: admission table and web-API table | WA-1.8, 1.9, 1.18, 2.3, 2.14, 2.16, 3.8, 3.9, 3.10, 3.13, 4.11, 6.11, 6.15, 6.23, 8.7, 8.9, 8.16, 9.9, 9.10, 9.11, 12.6, 13.5, 13.13, 17.8, 17.12, 17.13, 17.14, 17.17, 17.18 | Most of these are small. Add multi-engine APIs to DM 6.18 with owners (URLPattern, Entries API and directory upload, HTML Media Capture, `sendBeacon`, Web Share, LCP, the File System Access handles, an XPath evaluator), and record held or refused for the single-engine HTML and DOM proposals in DM-D15's table. |
| NW amendment: loading and response headers | WA-5.9, 5.11, 5.12, 5.13, 8.2, 17.6 | These are Fetch-layer or response-header features (No-Vary-Search, SRI and Integrity-Policy, signature-based SRI, upload streaming, Server-Timing, web bundles) whose checks belong in the partition service next to CORS and ORB. |
| New spec or SS amendment: web security policy (CSP and Permissions Policy) | WA-13.2, 12.2, 12.3, 2.15 | CSP's document-side directives and Permissions Policy are consumed by NW, ST, DV, MD and AX, but no spec owns their parsing, inheritance and enforcement. S1 13.2 already promises a permission design "before M6", and permission elements belong in it. |
| ML or media amendment: speech | WA-12.10 | Web Speech needs an OS or Izcal TTS path and recognition through the ML service under B10. ML §1 already mentions recognition for captions, so the decision belongs to ML or MD. |
| JS amendment: proposal-stage rule and edge decisions | WA-18.4, 18.14, 18.21, 19.4 (and the Deferred rows 5.4, 18.17, 18.24) | Record each proposal's stage at the pin, a rule for Stage 3 proposals already shipping in two engines, a proper-tail-calls decision, and whether the non-standard `Error.captureStackTrace` compat surface ships. |
| DT amendment: automation | WA-22.3, 22.10, 22.12 | WPT's testdriver needs WebDriver extension commands (permissions, virtual authenticator, sensors) and `TestUtils.gc()`. Whether Izcal runs origin trials is a platform-process decision with no home today; DT or the admission rule's owner should record it. |
| ST/DM amendment: web compat | WA-22.9 | Quirks modes are in ST, but the Compat Standard's `-webkit-` aliases and `window.orientation` are an Interop 2026 area with no owner. |

