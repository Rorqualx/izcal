# Izcal feature inventory audit: web platform APIs

Compiled 24 October 2026. Primary sources were preferred (specifications, standards indexes, vendor release notes).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

## 0. Scope, method and notation

This strand covers script-facing APIs and HTML features. CSS, graphics, media and real-time APIs (Canvas, WebGL, WebGPU, Web Audio, WebCodecs, WebRTC, Media Capture, MSE/EME, WebXR, Web Animations, images and fonts) belong to the other researcher. Rows that touch that boundary say so.

Sources read for this note, all prose or rendered indexes:

- The WHATWG standards list, [spec.whatwg.org](https://spec.whatwg.org/) (27 standards, read 24 October 2026).
- MDN's Web API index, [developer.mozilla.org/en-US/docs/Web/API](https://developer.mozilla.org/en-US/docs/Web/API) (the "Specifications" list).
- The Interop 2026 announcement, [web.dev/blog/interop-2026](https://web.dev/blog/interop-2026) (12 February 2026; 20 focus areas, 4 investigation areas).
- Chrome release notes for Chrome 141 to 154, [developer.chrome.com/release-notes/141](https://developer.chrome.com/release-notes/141) and the following numbers (Chrome 141 shipped 30 September 2025; Chrome 154 shipped 22 September 2026).
- MDN's Firefox release notes for Firefox 141 to 158, [developer.mozilla.org/en-US/docs/Mozilla/Firefox/Releases/158](https://developer.mozilla.org/en-US/docs/Mozilla/Firefox/Releases/158) and the earlier numbers (Firefox 141 shipped 22 July 2025; Firefox 158 shipped 13 October 2026; Firefox moved to a two-week cadence from Firefox 154 on).
- WebKit's "WebKit Features for Safari" posts for 26.0, 26.2, 26.4, 26.5, 26.6, 27.0 and 27.1, for example [webkit.org/blog/18325](https://webkit.org/blog/18325/webkit-features-for-safari-27-0/) (Safari 27.1 published 23 October 2026).
- Google's Privacy Sandbox update, [privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies) (17 October 2025).
- The WPT repository's top-level directory names only (listed through the GitHub contents API on 24 October 2026; no test files were opened).

No engine source, code review, commit, or data file (such as web-features or browser-compat-data) was opened.

Notation for "Ships in": three fields for Chromium / Gecko / WebKit. "Y" means shipped on by default in stable. A version number means the first stable release with the feature. "N" means not shipped in stable. "flag" means behind a preference or feature flag. "OT" means Chromium origin trial. "Baseline" labels follow the webstatus.dev definitions (newly available = in all three engines; widely available = 30 months later). Baseline labels and versions that come from the release notes above are rated High. Baseline labels recalled from earlier published status, and not re-read in this session, are rated Medium. webstatus.dev itself could not be loaded in this session (its pages are script-rendered and the reader returned errors), so no row cites it directly.

## 1. HTML: documents, browsing contexts, navigation and history

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-1.1 | Document and browsing-context model | Window, Document, WindowProxy, navigables, browsing context groups, `window.open`, `opener`, `noopener`/`noreferrer`, named targets | [HTML §7](https://html.spec.whatwg.org/multipage/document-sequences.html) | Y / Y / Y (Baseline widely available). Chrome 153 removed non-standard `_current` targets. | Medium |
| WA-1.2 | Navigation and session history | `location`, `history.pushState`/`replaceState`, `popstate`, `hashchange`, fragment navigation, session history traversal | [HTML §7.4](https://html.spec.whatwg.org/multipage/browsing-the-web.html) | Y / Y / Y (Baseline widely available) | Medium |
| WA-1.3 | Navigation API | `navigation` object, `navigate` event, `intercept()`, entries, `transition`, precommit handlers | [HTML: Navigation API](https://html.spec.whatwg.org/multipage/nav-history-apis.html#navigation-api) | 102 / 147 / 26.2 (Baseline newly available, 13 January 2026). Precommit handlers: Chrome 141, Firefox 148. Interop 2026 focus area. | High |
| WA-1.4 | Back/forward cache (bfcache) behaviour | Page freeze and restore, `pageshow`/`pagehide` `persisted`, unload deprecation, `notRestoredReasons` | [HTML](https://html.spec.whatwg.org/multipage/nav-history-apis.html#persisted-history-entry-state); [NotRestoredReasons](https://github.com/WICG/bfcache-not-restored-reason) | bfcache Y / Y / Y. `notRestoredReasons` Chromium only. Chrome 149 lets pages with open WebSockets enter bfcache. | Medium |
| WA-1.5 | Page Visibility and Page Lifecycle | `document.visibilityState`, `visibilitychange`; Chromium's `freeze`/`resume` events and `document.wasDiscarded` | [HTML: page visibility](https://html.spec.whatwg.org/multipage/interaction.html#page-visibility); [Page Lifecycle](https://wicg.github.io/page-lifecycle/) | Visibility Y / Y / Y (Baseline widely available). Lifecycle events Chromium only. | Medium |
| WA-1.6 | `beforeunload` / `unload` | Leave-page prompt and unload handlers; `unload` is being deprecated in Chromium | [HTML](https://html.spec.whatwg.org/multipage/nav-history-apis.html#unloading-documents) | Y / Y / Y. Chromium is phasing out `unload`. | Medium |
| WA-1.7 | iframes and embedding | `<iframe>` with `srcdoc`, `sandbox`, `allow`, `loading=lazy`, `referrerpolicy`; `<object>`, `<embed>` | [HTML §4.8](https://html.spec.whatwg.org/multipage/iframe-embed-object.html) | Y / Y / Y. Firefox 142 removed `<object codebase>`. | High |
| WA-1.8 | Credentialless iframes | `<iframe credentialless>` loads with ephemeral storage, for COEP pages | [Anonymous iframe](https://wicg.github.io/anonymous-iframe/) | Chromium only | Medium |
| WA-1.9 | Fenced frames | `<fencedframe>` for ad rendering without cross-boundary communication | [Fenced Frame](https://wicg.github.io/fenced-frame/) | Chromium only; tied to Protected Audience, which Chrome is removing (see WA-14) | Medium |
| WA-1.10 | `Location.ancestorOrigins` | Origins of ancestor frames | [HTML](https://html.spec.whatwg.org/multipage/nav-history-apis.html#dom-location-ancestororigins) | Y / 148 / Y (Baseline newly available, 24 February 2026) | High |
| WA-1.11 | Origin API | `Origin` object; `Origin.from()`, `isSameSite()`, comparing opaque origins | [HTML: Origin](https://html.spec.whatwg.org/multipage/browsers.html#the-origin-interface) | 145 / N / 26.5 (two engines) | High |
| WA-1.12 | Text fragments | `#:~:text=` scroll-to-text directives; `::target-text` | [URL Fragment Text Directives](https://wicg.github.io/scroll-to-text-fragment/) | Y / 131 / 16.1 (Baseline newly available, December 2024) | Medium |
| WA-1.13 | Top-navigation restrictions | Cross-origin iframes need sticky activation to navigate `window.top` | [HTML](https://html.spec.whatwg.org/multipage/browsing-the-web.html) | Y / 144 / Y | High |
| WA-1.14 | Initial `about:blank` behaviour | Synchronous initial `about:blank` load and `load` event, per spec | [HTML](https://html.spec.whatwg.org/multipage/document-sequences.html) | Chromium Y / 148 / Y | High |
| WA-1.15 | Printing | `window.print()`, `beforeprint`/`afterprint`, print layout hooks | [HTML](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#printing) | Y / Y / Y | Medium |
| WA-1.16 | Simple dialogs | `alert`, `confirm`, `prompt`; cross-origin iframe suppression in Chromium | [HTML](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#simple-dialogs) | Y / Y / Y | Medium |
| WA-1.17 | Window and screen properties | `screenX`, `outerWidth`, `devicePixelRatio`, `screen`, bar props (`toolbar.visible`) | [CSSOM View](https://drafts.csswg.org/cssom-view/); [HTML](https://html.spec.whatwg.org/multipage/nav-history-apis.html#barprop) | Y / Y / Y. Safari 27.0 returns static bar-prop values. Chrome 145 fixed values on Android. | High |
| WA-1.18 | Out-of-order HTML streaming | `<template for>` and processing-instruction ranges let a server patch parts of a document without script | [WHATWG proposal](https://github.com/whatwg/html/issues/2142) | 150 / N / N (Chromium only) | High |
| WA-1.19 | XSLT and `XSLTProcessor` | Client-side XSLT 1.0 transforms; under removal | [WHATWG DOM / HTML](https://html.spec.whatwg.org/multipage/) | Deprecated in Chromium (announced Chrome 143; deprecation trial Chrome 152). Gecko and WebKit still ship it. | High |

**Implications:** Navigation and history are the spine of every page load. The Navigation API is now Baseline, so a new engine must build history traversal, precommit handlers and the `navigate` event as one model rather than retrofit it onto the History API.

## 2. HTML: elements, attributes and document metadata

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-2.1 | HTML parser and serializer | Tokenizer, tree builder, error recovery, `innerHTML`/`outerHTML` serialization, `document.write` | [HTML §13](https://html.spec.whatwg.org/multipage/parsing.html) | Y / Y / Y (Baseline widely available) | Medium |
| WA-2.2 | Sectioning, grouping and text-level elements | `article`, `section`, `nav`, `main`, `hgroup`, `search`, `ruby`, `mark`, `time`, `data`, `wbr`, etc. | [HTML §4.3–4.5](https://html.spec.whatwg.org/multipage/sections.html) | Y / Y / Y. `<search>` Baseline 2023. | Medium |
| WA-2.3 | Heading level adjustment | `headingoffset` and `headingreset` global attributes | [HTML proposal](https://github.com/whatwg/html/pull/11086) | N / flag (Firefox 153) / N | High |
| WA-2.4 | Links and `<a>` behaviours | `href`, `download`, `ping`, `rel`, `hreflang`, `referrerpolicy`, `target` | [HTML §4.6](https://html.spec.whatwg.org/multipage/links.html) | Y / Y / Y. SVG `<a>` gained the same IDL in Chrome 144, Safari 26.2, Firefox 155. | High |
| WA-2.5 | `<link>` relation types | `stylesheet`, `icon`, `manifest`, `preload`, `modulepreload`, `prefetch`, `dns-prefetch`, `preconnect`, `alternate`, `expect` | [HTML §4.6.7](https://html.spec.whatwg.org/multipage/links.html#linkTypes) | Mostly Y / Y / Y. `rel=prefetch` Chromium and Gecko. `rel=expect` Chromium only. Safari 26.0 added `dns-prefetch` on iOS. | Medium |
| WA-2.6 | `<meta>` names and pragmas | `viewport`, `theme-color`, `color-scheme`, `referrer`, `http-equiv` refresh and CSP, `rating` | [HTML §4.2.5](https://html.spec.whatwg.org/multipage/semantics.html#the-meta-element) | Y / Y / Y. `<meta name=rating>` Firefox Nightly only (Firefox 146). | High |
| WA-2.7 | `<template>` and `<slot>` | Inert template content; shadow-tree slots | [HTML §4.12](https://html.spec.whatwg.org/multipage/scripting.html#the-template-element) | Y / Y / Y (Baseline widely available) | Medium |
| WA-2.8 | Responsive images markup | `<picture>`, `srcset`, `sizes`, `sizes="auto"`, `loading`, `decoding`, `fetchpriority` (decoding is the media strand's) | [HTML §4.8.4](https://html.spec.whatwg.org/multipage/images.html) | Y / Y / Y. `sizes="auto"`: Chromium Y / 150 / 27.0 (Baseline newly available, September 2026). | High |
| WA-2.9 | Lazy loading | `loading=lazy` on `img` and `iframe`; Chrome 148 added it on `video` and `audio` | [HTML](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#lazy-loading-attributes) | img/iframe Y / Y / Y (Baseline). Media elements Chromium only (Chrome 148). | High |
| WA-2.10 | `<details>` and `<summary>` | Disclosure widget; `name` for exclusive accordions; auto-expand for find-in-page | [HTML §4.11.1](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element) | Y / Y / Y. `name` Baseline 2024. Find-in-page auto-expand Safari 26.2. | High |
| WA-2.11 | `hidden="until-found"` | Content hidden but searchable; `beforematch` event | [HTML](https://html.spec.whatwg.org/multipage/interaction.html#the-hidden-attribute) | 102 / Y / 26.2 | Medium |
| WA-2.12 | Global attributes | `id`, `class`, `lang`, `dir`, `title`, `tabindex`, `contenteditable`, `spellcheck`, `translate`, `inputmode`, `enterkeyhint`, `autocapitalize`, `draggable`, `nonce`, `popover`, `inert` | [HTML §3.2.6](https://html.spec.whatwg.org/multipage/dom.html#global-attributes) | Y / Y / Y | Medium |
| WA-2.13 | `autocorrect` attribute | Global attribute controlling OS autocorrection | [HTML](https://html.spec.whatwg.org/multipage/interaction.html#autocorrection) | 152 / N / Y | High |
| WA-2.14 | Processing instructions in HTML | The HTML parser keeps `<?target ...?>` nodes and gives them an attribute API | [WHATWG proposal](https://github.com/whatwg/html/issues/10997) | 148 (attribute API 150) / N / N | High |
| WA-2.15 | Permission and capability elements | `<geolocation>` (Chrome 144), `<usermedia>` (Chrome 151), `<camera>` and `<microphone>` (Chrome 153); replace the earlier `<permission>` trial | [WICG Page-Embedded Permission Control](https://wicg.github.io/PEPC/permission-elements.html) | Chromium only | High |
| WA-2.16 | `<model>` element | Embedded 3D model with an immersive mode | [Model element](https://immersive-web.github.io/model-element/) | N / N / 26.0 visionOS, 27.0 iOS and macOS (WebKit only). Overlaps the graphics strand. | High |
| WA-2.17 | MathML Core | `<math>` and its elements in HTML; Safari 27.0 added `tabindex`/`focus()` on MathML | [MathML Core](https://www.w3.org/TR/mathml-core/) | Y / Y / Y. Rendering belongs to the text/graphics strand. | High |
| WA-2.18 | Obsolete and removed features | `<applet>`, `<frameset>` (still parsed), `<marquee>`, `longdesc`, plugins (`navigator.plugins` returns a fixed PDF list) | [HTML §16](https://html.spec.whatwg.org/multipage/obsolete.html) | Varied. Still parsed for compatibility in all three. | Medium |

**Implications:** Most of HTML is old and stable, but 2025–2026 added several Chromium-only markup experiments (capability elements, processing instructions, out-of-order streaming). Izcal should decide per item whether to follow, since some may never reach other engines.

## 3. HTML: forms and form controls

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-3.1 | Form submission | `<form>`, `method`, `enctype`, `action`, `requestSubmit()`, `formdata` event, `FormData` | [HTML §4.10](https://html.spec.whatwg.org/multipage/forms.html) | Y / Y / Y (Baseline widely available) | Medium |
| WA-3.2 | Input types | text, search, email, url, tel, password, number, range, date, time, datetime-local, month, week, color, file, checkbox, radio, hidden, image, button | [HTML §4.10.5](https://html.spec.whatwg.org/multipage/input.html) | Y / Y / Y. month/week not in Firefox desktop. Firefox 143 accepts any CSS color in `type=color`. | High |
| WA-3.3 | Constraint validation | `required`, `pattern`, `min`/`max`, `checkValidity`, `reportValidity`, `setCustomValidity`, `:invalid` | [HTML §4.10.20](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#constraints) | Y / Y / Y | Medium |
| WA-3.4 | `<select>`, `<option>`, `<datalist>`, `<textarea>`, `<output>`, `<progress>`, `<meter>`, `<fieldset>` | Standard controls | [HTML §4.10](https://html.spec.whatwg.org/multipage/form-elements.html) | Y / Y / Y | Medium |
| WA-3.5 | Customizable `<select>` | `appearance: base-select`, `<selectedcontent>`, rich content in `<option>` | [HTML proposal](https://github.com/whatwg/html/issues/9799) | 135 / partial (Firefox 153 parses nested content) / 27.0 | High |
| WA-3.6 | `showPicker()` | Open a native picker programmatically | [HTML](https://html.spec.whatwg.org/multipage/input.html#dom-input-showpicker) | Y / Y / Y. Firefox 149 extended it to datalist inputs. | High |
| WA-3.7 | Autofill and `autocomplete` tokens | Address, payment and credential autofill hints; Chromium's autofill event trial | [HTML §4.10.18.7](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#autofill) | Y / Y / Y. Autofill event OT in Chrome 147. | High |
| WA-3.8 | Directory upload | `webkitdirectory`, `File.webkitRelativePath` | [File and Directory Entries](https://wicg.github.io/entries-api/) | Y / Y (Android since 142) / Y | High |
| WA-3.9 | HTML Media Capture | `capture` attribute on file inputs | [HTML Media Capture](https://www.w3.org/TR/html-media-capture/) | Mobile Chromium, Firefox Android, iOS Safari | Medium |
| WA-3.10 | `<input type=checkbox switch>` | Native switch control | [HTML proposal](https://github.com/whatwg/html/pull/9546) | N / N / 17.4 (WebKit only) | Medium |
| WA-3.11 | Color input `alpha` and `colorspace` | Wider-gamut, alpha-capable colour picker | [HTML](https://html.spec.whatwg.org/multipage/input.html#color-state-(type=color)) | N / flag (Firefox 149 Nightly) / Y | Medium |
| WA-3.12 | Form-associated custom elements | `ElementInternals`, `formAssociated`, validity and form state for custom elements | [HTML §4.13.7](https://html.spec.whatwg.org/multipage/custom-elements.html#form-associated-custom-elements) | Y / Y / 16.4 (Baseline newly available, March 2023) | Medium |
| WA-3.13 | OpaqueRange | Live text ranges inside `<input>`/`<textarea>` values | [WICG OpaqueRange](https://github.com/WICG/opaque-range) | 152 / N / N | High |

**Implications:** Forms carry most of a page's user input and autofill integration, and every control needs native pickers, accessibility and IME support per platform. Customizable select is now in two engines and changes how `<select>` is parsed.

## 4. HTML: interactive UI, focus, dialogs and drag and drop

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-4.1 | `<dialog>` | Modal and non-modal dialogs, `showModal()`, `close`, `cancel`, `toggle` event | [HTML §4.11.4](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element) | Y / Y / Y (Baseline widely available). Safari 26.0 added the `toggle` event. Interop 2026 focus area. | High |
| WA-4.2 | Dialog light dismiss (`closedby`) and `requestClose()` | Declarative dismissal policy for dialogs | [HTML](https://html.spec.whatwg.org/multipage/interactive-elements.html#attr-dialog-closedby) | 134 / 141 / N | High |
| WA-4.3 | Popover API | `popover` attribute, `showPopover`/`hidePopover`/`togglePopover`, `beforetoggle`/`toggle`, top layer | [HTML §6.12](https://html.spec.whatwg.org/multipage/popover.html) | Y / Y / Y (Baseline newly available, April 2024). `ToggleEvent.source`: Firefox 145, Safari 26.5. | High |
| WA-4.4 | `popover="hint"` | Hint popovers that do not close auto popovers | [HTML](https://html.spec.whatwg.org/multipage/popover.html#attr-popover-hint) | 133 / 149 / N | High |
| WA-4.5 | Invoker commands | `command` and `commandfor` on buttons, custom `--commands`, `CommandEvent` | [HTML](https://html.spec.whatwg.org/multipage/form-elements.html#attr-button-command) | 135 / 144 / 26.2 (Baseline newly available, December 2025) | High |
| WA-4.6 | Interest invokers | `interestfor` on buttons and links, hover/focus-triggered popovers | [Open UI proposal](https://open-ui.org/components/interest-invokers.explainer/) | 142 / N / N (Chromium only) | High |
| WA-4.7 | `CloseWatcher` | Unified close requests (Esc, Android Back) | [HTML](https://html.spec.whatwg.org/multipage/interaction.html#close-requests-and-close-watchers) | 126 / 149 / N | High |
| WA-4.8 | Light-dismiss input timing change | Popover and dialog light dismiss on `click`, not pointer down/up | [HTML](https://html.spec.whatwg.org/multipage/popover.html) | 154 / ? / ? | High |
| WA-4.9 | Focus management | `focus()` options (`preventScroll`, `focusVisible`), sequential navigation, `autofocus`, `:focus-visible` | [HTML §6.6](https://html.spec.whatwg.org/multipage/interaction.html#focus) | Y / Y / Y | Medium |
| WA-4.10 | `inert` | Subtrees removed from focus, events and accessibility | [HTML](https://html.spec.whatwg.org/multipage/interaction.html#the-inert-attribute) | Y / Y / Y (Baseline newly available, April 2023) | Medium |
| WA-4.11 | Focusgroup | Declarative arrow-key navigation within composite widgets | [Open UI focusgroup](https://open-ui.org/components/scoped-focusgroup.explainer/) | 150 / N / N | High |
| WA-4.12 | User activation | Transient and sticky activation; `navigator.userActivation` | [HTML §6.4](https://html.spec.whatwg.org/multipage/interaction.html#tracking-user-activation) | Y / Y / Y. Chrome 142 keeps sticky activation across same-origin navigations. | Medium |
| WA-4.13 | HTML drag and drop | `draggable`, `DataTransfer`, drag events, file drop | [HTML §6.11](https://html.spec.whatwg.org/multipage/dnd.html) | Y / Y / Y. Chrome 146 and 148 corrected `dropEffect` handling. | High |
| WA-4.14 | Fullscreen API | `requestFullscreen`, `fullscreenchange`, `:fullscreen` | [Fullscreen](https://fullscreen.spec.whatwg.org/) | Y / Y / Y (iPhone Safari support for element fullscreen is limited; not re-checked). Keyboard-lock option: Safari 26.4, Firefox 151. | High |
| WA-4.15 | Pointer Lock | `requestPointerLock`, `unadjustedMovement` | [Pointer Lock 2](https://www.w3.org/TR/pointerlock-2/) | Y / Y (`unadjustedMovement` 152) / Y. Chrome 144 added Android. | High |
| WA-4.16 | Keyboard Lock and Keyboard Map | `navigator.keyboard.lock()`, `getLayoutMap()` | [Keyboard Lock](https://wicg.github.io/keyboard-lock/); [Keyboard Map](https://wicg.github.io/keyboard-map/) | Chromium only (Safari and Firefox expose only the fullscreen `keyboardLock` option) | High |
| WA-4.17 | `ariaNotify()` | Script-driven screen-reader announcements | [ARIA Notify explainer](https://github.com/w3c/aria/pull/2577) | 141 / 150 / N | High |
| WA-4.18 | Editing hooks | `contenteditable`, `execCommand` (paste works in Firefox 148 with activation), `beforeinput`/`input` with `inputType` and `dataTransfer`, `EditContext` | [Input Events 2](https://www.w3.org/TR/input-events-2/); [EditContext](https://w3c.github.io/edit-context/); [execCommand](https://w3c.github.io/editing/docs/execCommand/) | Y / Y / Y. EditContext Chromium only. Izcal has a separate editing spec. | High |

**Implications:** Top-layer UI (dialog, popover, commands, close watchers) moved fast in 2025–2026, and engine support still differs item by item. These features also need consistent focus and accessibility behaviour, so Izcal should build them on one top-layer and close-request model.

## 5. HTML: scripting, modules and resource loading

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-5.1 | Classic scripts | `<script>`, `async`, `defer`, `nomodule`, `crossorigin`, `integrity`, `nonce`, `fetchpriority` | [HTML §4.12.1](https://html.spec.whatwg.org/multipage/scripting.html#the-script-element) | Y / Y / Y | Medium |
| WA-5.2 | Module scripts | `type=module`, dynamic `import()`, `import.meta`, top-level await, module loading algorithm | [HTML §8.1.5](https://html.spec.whatwg.org/multipage/webappapis.html#integration-with-the-javascript-module-system) | Y / Y / Y. Safari 27.0 rewrote its module loader to fix TLA and evaluation order. Firefox 155 stopped caching failed module loads. | High |
| WA-5.3 | JSON and CSS module scripts | `import x from "./a.json" with {type:"json"}`; `type: "css"` returns a `CSSStyleSheet` | [HTML](https://html.spec.whatwg.org/multipage/webappapis.html#module-type-allowed) | JSON Y / Y / Y (Baseline 2025). CSS modules Y / 147 / N. Declarative CSS modules OT Chrome 148. | High |
| WA-5.4 | Text module imports | `with { type: "text" }` | [TC39 proposal](https://github.com/tc39/proposal-import-text) | N / 153 / N | High |
| WA-5.5 | Import maps | `<script type=importmap>`, scopes, `integrity`; multiple import maps | [HTML](https://html.spec.whatwg.org/multipage/webappapis.html#import-maps) | Y / Y / Y (Baseline 2023). Multiple maps: Chromium Y, Firefox 150 flag. | High |
| WA-5.6 | `modulepreload` | Preload module graphs; Chrome 147 added JSON and style modules | [HTML](https://html.spec.whatwg.org/multipage/links.html#link-type-modulepreload) | Y / Y / Y. Firefox 155 changed `load` events for already-fetched modules. | High |
| WA-5.7 | Preload and resource hints | `rel=preload`, `preconnect`, `dns-prefetch`, `fetchpriority`, `103 Early Hints` | [HTML](https://html.spec.whatwg.org/multipage/links.html#link-type-preload) | Y / Y / Y. Early Hints Chromium and Gecko; WebKit partial. | Medium |
| WA-5.8 | Speculation Rules | `<script type=speculationrules>` for prefetch and prerender, eagerness, `document.prerendering`, `prerenderingchange` | [Speculation Rules](https://wicg.github.io/nav-speculation/speculation-rules.html) | Chromium Y (prerender, prefetch; `form_submission` Chrome 151) / flag (Firefox 155 prefetch, 158 moderate eagerness) / N | High |
| WA-5.9 | No-Vary-Search | Response header that ignores chosen query params for cache and prefetch matching | [No-Vary-Search](https://wicg.github.io/nav-speculation/no-vary-search.html) | 141 / 154 / N | High |
| WA-5.10 | Render-blocking | `blocking="render"` on scripts, styles and links | [HTML](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#blocking-attributes) | Y / ? / ? | Low |
| WA-5.11 | Web bundles and signed exchanges | `<script type=webbundle>`, SXG | [Web Bundles](https://wicg.github.io/webpackage/); [SXG](https://wicg.github.io/webpackage/draft-yasskin-http-origin-signed-responses.html) | Chromium only; SXG in decline | Medium |
| WA-5.12 | Integrity Policy | `Integrity-Policy` header requiring SRI on scripts and styles | [SRI Level 2](https://w3c.github.io/webappsec-subresource-integrity/) | Y (Chrome 138) / 145 / 26.0 (Baseline newly available, November 2025) | High |
| WA-5.13 | Signature-based integrity | Ed25519 signatures checked on subresources | [Signature-based SRI](https://wicg.github.io/signature-based-sri/) | 141 / N / N | High |

**Implications:** Module loading is now a large, fast-changing area (new module types, source phase and defer imports in ES). Izcal's loader should be one spec-shaped pipeline that HTML, workers and Wasm share.

## 6. DOM, Web Components and DOM-adjacent APIs

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-6.1 | DOM core | Nodes, elements, attributes, documents, `DocumentFragment`, tree mutation methods | [DOM](https://dom.spec.whatwg.org/) | Y / Y / Y. Name validation relaxed to match the parser: Chrome 143, Firefox 149. | High |
| WA-6.2 | Events | `EventTarget`, dispatch, capture/bubble, `CustomEvent`, passive listeners, `once`, `signal` | [DOM §2](https://dom.spec.whatwg.org/#events) | Y / Y / Y | Medium |
| WA-6.3 | Aborting | `AbortController`, `AbortSignal.timeout()`, `AbortSignal.any()` | [DOM §3](https://dom.spec.whatwg.org/#aborting-ongoing-activities) | Y / Y / Y (Baseline 2024 for `any()`) | Medium |
| WA-6.4 | MutationObserver | Asynchronous DOM change records; mutation events removed | [DOM §4.3](https://dom.spec.whatwg.org/#mutation-observers) | Y / Y / Y. Legacy mutation events removed in Chromium (2024); Safari 27.0 rejects `createEvent("MutationEvent")`. | High |
| WA-6.5 | Ranges, traversal and `StaticRange` | `Range`, `TreeWalker`, `NodeIterator` | [DOM](https://dom.spec.whatwg.org/#ranges) | Y / Y / Y. Firefox 156 made range deletion follow the DOM tree across shadow roots. | High |
| WA-6.6 | `moveBefore()` | Atomic move that keeps element state (iframes, focus, animations) | [DOM](https://dom.spec.whatwg.org/#dom-parentnode-movebefore) | 133 / 144 / N | High |
| WA-6.7 | Shadow DOM | `attachShadow`, slots, `slotchange`, manual slot assignment, `delegatesFocus`, `getComposedRanges` | [DOM §4.2.2](https://dom.spec.whatwg.org/#shadow-trees) | Y / Y / Y. `Selection.getComposedRanges()` Firefox 142. | High |
| WA-6.8 | Declarative shadow DOM | `<template shadowrootmode>`, `shadowrootslotassignment`, `setHTMLUnsafe`, `getHTML` | [HTML](https://html.spec.whatwg.org/multipage/scripting.html#attr-template-shadowrootmode) | Y / Y / Y (Baseline 2024). `shadowrootslotassignment`: 151 / 151 / 27.0 (Baseline newly available, September 2026). | High |
| WA-6.9 | Custom elements | `customElements.define`, lifecycle callbacks, upgrades, `:defined`, `CustomStateSet` (`:state()`) | [HTML §4.13](https://html.spec.whatwg.org/multipage/custom-elements.html) | Y / Y / Y. Customized built-ins (`is=`) Chromium and Gecko only; WebKit declines. | Medium |
| WA-6.10 | Scoped custom element registries | Per-shadow-root `CustomElementRegistry`, `customelementregistry` attribute | [HTML](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-registry-scoped) | 146 / flag (on in Nightly, Firefox 156) / 26.0 (two engines). Interop 2026 focus area. | High |
| WA-6.11 | Reference Target | Cross-shadow-root ARIA and ID references | [WICG Reference Target](https://github.com/WICG/webcomponents/blob/gh-pages/proposals/reference-target-explainer.md) | 152 / N / N | High |
| WA-6.12 | ARIA reflection | `role`, `aria*` properties and element-reference properties (`ariaLabelledByElements`) | [ARIA 1.3 §10](https://w3c.github.io/aria/#idl-interface) | Y / Y / Y. Chrome 152 added `aria-actions`. Accessibility strand covers the tree. | High |
| WA-6.13 | DOM Parsing and Serialization | `DOMParser`, `XMLSerializer`, `insertAdjacentHTML`, `createContextualFragment` | [HTML: DOM parsing](https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html) | Y / Y / Y. Chromium moved non-XSLT XML parsing to a Rust parser (Chrome 147–153). | High |
| WA-6.14 | HTML Sanitizer API | `Sanitizer`, `setHTML()`, safe defaults, `javascriptURLs` option | [HTML Sanitizer](https://wicg.github.io/sanitizer-api/) | 146 / 148 / N (two engines) | High |
| WA-6.15 | XPath and XML documents | `document.evaluate`, `XPathResult`, XML namespaces, `createElementNS` | [DOM: XPath](https://dom.spec.whatwg.org/#xpath) | Y / Y / Y | Medium |
| WA-6.16 | Selection API | `getSelection()`, `Selection`, `selectionchange`, shadow-aware selection | [Selection API](https://w3c.github.io/selection-api/) | Y / Y / Y | Medium |
| WA-6.17 | UI Events | Mouse, keyboard, wheel, focus, composition events; `KeyboardEvent.key/code`; wheel `momentum` (Chrome 151) | [UI Events](https://w3c.github.io/uievents/) | Y / Y / Y. Firefox 143 removed `CompositionEvent.locale`. | High |
| WA-6.18 | Pointer Events | Unified pointer input, `pointerrawupdate`, coalesced and predicted events, `persistentDeviceId` | [Pointer Events 3](https://w3c.github.io/pointerevents/) | Y / Y / Y. `persistentDeviceId` Firefox 141. Fractional coordinates Safari 26.2. | High |
| WA-6.19 | Touch Events | `touchstart` etc. for touch devices | [Touch Events](https://w3c.github.io/touch-events/) | Y / Y / Y (mobile; desktop Firefox off by default) | Medium |
| WA-6.20 | Ink API | `navigator.ink` delegated low-latency ink trails | [Ink API](https://wicg.github.io/ink-enhancement/) | Chromium only | Medium |
| WA-6.21 | Observers | `IntersectionObserver` (with `scrollMargin`), `ResizeObserver` | [Intersection Observer](https://w3c.github.io/IntersectionObserver/); [Resize Observer](https://drafts.csswg.org/resize-observer/) | Y / Y / Y. `scrollMargin`: Firefox 141, Safari 26.0. | High |
| WA-6.22 | Scrolling and caret APIs | `scrollIntoView`, `scrollend`, `caretPositionFromPoint`, `checkVisibility`, scroll promises | [CSSOM View](https://drafts.csswg.org/cssom-view/) | `scrollend` and `caretPositionFromPoint` Baseline newly available December 2025 (Safari 26.2). Scroll promises Chrome 150 only. | High |
| WA-6.23 | Observable API | `Observable`, `EventTarget.when()` | [Observable](https://wicg.github.io/observable/) | 135 / N / N | Medium |
| WA-6.24 | View Transitions, script side | `document.startViewTransition`, types, `activeViewTransition`; cross-document `pageswap`/`pagereveal` | [CSS View Transitions 2](https://drafts.csswg.org/css-view-transitions-2/) | Same-document: 111 / 144 / 18 (Baseline newly available, October 2025). Types and `activeViewTransition`: Firefox 147, Safari 26.2. Cross-document: 126 / N / 18.2. Interop 2026 focus area. Rendering belongs to the CSS strand. | High |
| WA-6.25 | Geometry interfaces | `DOMMatrix`, `DOMPoint`, `DOMRect`, `DOMQuad` | [Geometry Interfaces](https://drafts.fxtf.org/geometry/) | Y / Y / Y | Medium |
| WA-6.26 | Visual Viewport | `window.visualViewport`, resize/scroll/scrollend events | [CSSOM View](https://drafts.csswg.org/cssom-view/#visualViewport) | Y / Y / Y. `scrollend` on it in Firefox 150. | High |

**Implications:** DOM is the largest single API surface and every other spec hooks into it. Scoped registries, `moveBefore` and the Sanitizer change core tree invariants, so Izcal's DOM spec should cover them from the start.

## 7. Event loop, timers, scheduling, workers and messaging

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-7.1 | Event loop, tasks and microtasks | Task queues, microtask checkpoint, rendering update steps, `queueMicrotask` | [HTML §8.1.7](https://html.spec.whatwg.org/multipage/webappapis.html#event-loops) | Y / Y / Y | Medium |
| WA-7.2 | Timers and animation frames | `setTimeout`, `setInterval`, `requestAnimationFrame`, background throttling | [HTML §8.6](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timers) | Y / Y / Y | Medium |
| WA-7.3 | `requestIdleCallback` | Run work in idle periods | [requestIdleCallback](https://w3c.github.io/requestidlecallback/) | Y / Y / N (not in Safari stable as far as found) | Medium |
| WA-7.4 | Prioritized Task Scheduling | `scheduler.postTask`, `TaskController`, `scheduler.yield()` | [Scheduling APIs](https://wicg.github.io/scheduling-apis/) | Y / 142 / N. Firefox 156 fixed priority inheritance for `yield()`. | High |
| WA-7.5 | `isInputPending` | Check for pending input | [isInputPending](https://wicg.github.io/is-input-pending/) | Chromium only; superseded by `scheduler.yield()` | Medium |
| WA-7.6 | Error reporting | `onerror`, `unhandledrejection`, `reportError()` | [HTML §8.1.4](https://html.spec.whatwg.org/multipage/webappapis.html#runtime-script-errors) | Y / Y / Y | Medium |
| WA-7.7 | Dedicated workers | `Worker`, module workers, `importScripts` | [HTML §10](https://html.spec.whatwg.org/multipage/workers.html) | Y / Y / Y (module workers Baseline 2023). Chrome 150 gives `data:` workers opaque origins. | High |
| WA-7.8 | Shared workers | `SharedWorker`, `extendedLifetime` option | [HTML §10.2.6](https://html.spec.whatwg.org/multipage/workers.html#shared-workers-and-the-sharedworker-interface) | Y (Android re-enabled 148; `extendedLifetime` 148) / Y / Y | High |
| WA-7.9 | Worklets | `Worklet` base for audio, paint, layout and animation worklets | [HTML: Worklets](https://html.spec.whatwg.org/multipage/worklets.html) | Y / Y / Y (base); worklet types belong to the CSS and media strands | Medium |
| WA-7.10 | Cross-document messaging | `postMessage`, `MessageEvent`, origin checks | [HTML §9.3](https://html.spec.whatwg.org/multipage/web-messaging.html) | Y / Y / Y | Medium |
| WA-7.11 | Channel messaging | `MessageChannel`, `MessagePort` | [HTML §9.4](https://html.spec.whatwg.org/multipage/web-messaging.html#channel-messaging) | Y / Y / Y | Medium |
| WA-7.12 | Broadcast Channel | Same-origin pub/sub across contexts | [HTML §9.5](https://html.spec.whatwg.org/multipage/web-messaging.html#broadcasting-to-other-browsing-contexts) | Y / Y / 15.4 (Baseline widely available) | Medium |
| WA-7.13 | Structured clone and transfer | `structuredClone()`, transferable objects (ArrayBuffer, MessagePort, ImageBitmap, streams, RTCDataChannel) | [HTML §2.7](https://html.spec.whatwg.org/multipage/structured-data.html) | Y / Y / Y. Transferable streams: 87 / 103 / 27.0 (Baseline newly available, September 2026). | High |
| WA-7.14 | SharedArrayBuffer and Atomics in browsers | Shared memory, gated on cross-origin isolation | [HTML: cross-origin isolation](https://html.spec.whatwg.org/multipage/webappapis.html#concept-settings-object-cross-origin-isolated-capability) | Y / Y / Y (requires COOP+COEP) | Medium |
| WA-7.15 | Web Locks | `navigator.locks.request()` across tabs and workers | [Web Locks](https://w3c.github.io/web-locks/) | Y / Y / 15.4 (Baseline widely available) | Medium |

**Implications:** The event loop governs ordering for every API; the 2025–2026 scheduler work (`postTask`, `yield`) is in two engines. Izcal's DOM, bindings and event loop spec should state its task-priority model explicitly.

## 8. Fetch, networking and data formats

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-8.1 | Fetch API | `fetch()`, `Request`, `Response`, `Headers`, CORS, credentials, redirect modes, `keepalive`, `priority` | [Fetch](https://fetch.spec.whatwg.org/) | Y / Y / Y (Baseline widely available). Chrome 147 added `Request.isReloadNavigation`; Chrome 154 forwards abort reasons to bodies. | High |
| WA-8.2 | Fetch upload streaming and ranges | `ReadableStream` request bodies, `duplex: "half"`, range requests | [Fetch](https://fetch.spec.whatwg.org/#dom-requestinit-duplex) | Chromium Y (HTTP/2+) / flag (Firefox 158 Nightly) / N. Interop 2026 focus area ("Fetch uploads and ranges"). | High |
| WA-8.3 | `fetchLater()` | Deferred fetch sent on page unload | [Fetch: deferred fetching](https://fetch.spec.whatwg.org/#deferred-fetching) | Chromium only (Chrome 135) | Medium |
| WA-8.4 | Body helpers | `json()`, `text()`, `blob()`, `formData()`, `bytes()`; `textStream()` | [Fetch §6.4](https://fetch.spec.whatwg.org/#body-mixin) | Y / Y / Y. `textStream()` Chrome 151 only. | High |
| WA-8.5 | XMLHttpRequest | Legacy request API with progress events and `responseXML` | [XHR](https://xhr.spec.whatwg.org/) | Y / Y / Y | Medium |
| WA-8.6 | Server-sent events | `EventSource` | [HTML §9.2](https://html.spec.whatwg.org/multipage/server-sent-events.html) | Y / Y / Y | Medium |
| WA-8.7 | Beacon | `navigator.sendBeacon()` | [Beacon](https://w3c.github.io/beacon/) | Y / Y / Y | Medium |
| WA-8.8 | WebSockets | `WebSocket`, over HTTP/1.1, HTTP/2 and HTTP/3 (Safari 26.0); options-object constructor (Chrome 154) | [WebSockets](https://websockets.spec.whatwg.org/) | Y / Y / Y (Baseline widely available) | High |
| WA-8.9 | WebSocketStream | Promise and stream-based WebSocket with backpressure | [WebSockets](https://websockets.spec.whatwg.org/#the-websocketstream-interface) | Chromium only | Medium |
| WA-8.10 | WebTransport | HTTP/3 (and HTTP/2) streams and datagrams; stats, send groups, `exportKeyingMaterial` | [WebTransport](https://w3c.github.io/webtransport/) | Y / Y / 26.4 (Baseline newly available, March 2026). Firefox 155 and 158 added send groups and stats. Interop 2026 focus area. | High |
| WA-8.11 | Direct Sockets | Raw TCP/UDP sockets | [Direct Sockets](https://wicg.github.io/direct-sockets/) | Chromium Isolated Web Apps only (multicast Chrome 144) | High |
| WA-8.12 | Streams | `ReadableStream`, `WritableStream`, `TransformStream`, byte streams and BYOB readers, async iteration, `ReadableStream.from()`, pipes | [Streams](https://streams.spec.whatwg.org/) | Y / Y / Y. Byte streams and async iteration Safari 26.4; `from()` Safari 27.0. | High |
| WA-8.13 | Compression Streams | `CompressionStream`/`DecompressionStream`: gzip, deflate, deflate-raw, brotli | [Compression](https://compression.spec.whatwg.org/) | Y / Y / Y (Baseline 2023). Brotli Firefox 147. | High |
| WA-8.14 | Encoding API | `TextEncoder`, `TextDecoder`, stream variants, legacy encodings | [Encoding](https://encoding.spec.whatwg.org/) | Y / Y / Y | Medium |
| WA-8.15 | URL and URLSearchParams | URL parser, `URL.parse()`, `URL.canParse()`, IDNA (ContextJ in Chrome 148) | [URL](https://url.spec.whatwg.org/) | Y / Y / Y | High |
| WA-8.16 | URL Pattern | `URLPattern` matching | [URL Pattern](https://urlpattern.spec.whatwg.org/) | Y / 142 / 26.0 (Baseline newly available, September 2025) | High |
| WA-8.17 | MIME Sniffing | Content-type sniffing rules, `data:` URL MIME parameters (Chrome 146) | [MIME Sniffing](https://mimesniff.spec.whatwg.org/) | Y / Y / Y | High |
| WA-8.18 | Cookies as page-visible state | `document.cookie`, SameSite defaults, prefixes (`__Host-`), CHIPS `Partitioned` | [RFC 6265bis](https://datatracker.ietf.org/doc/draft-ietf-httpbis-rfc6265bis/); [CHIPS](https://privacycg.github.io/CHIPS/) | Y / Y / Y. CHIPS Safari 26.2. Safari 27.0 allows `Secure` cookies on loopback HTTP. | High |
| WA-8.19 | Client Hints | `Accept-CH`, `Sec-CH-UA-*`, `navigator.userAgentData` | [Client Hints](https://wicg.github.io/client-hints-infrastructure/); [UA-CH](https://wicg.github.io/ua-client-hints/) | Chromium only | Medium |
| WA-8.20 | Network Information and Save-Data | `navigator.connection`, `Save-Data` header | [Netinfo](https://wicg.github.io/netinfo/); [Save-Data](https://wicg.github.io/savedata/) | Chromium only | Medium |

**Implications:** Fetch is the funnel for nearly every load and every API that touches the network. Upload streaming and WebTransport are Interop 2026 work, so Izcal's networking spec needs request-body streams and HTTP/3 datagrams at a level other engines are only now matching.

## 9. Storage, files and quota

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-9.1 | Web Storage | `localStorage`, `sessionStorage`, `storage` event | [HTML §12](https://html.spec.whatwg.org/multipage/webstorage.html) | Y / Y / Y | Medium |
| WA-9.2 | IndexedDB 3 | Transactional object store; `getAllRecords()`, `getAll` direction | [IndexedDB 3](https://w3c.github.io/IndexedDB/) | Y / Y / Y (Baseline widely available). `getAllRecords()` Chrome 141, Firefox 153. Interop 2026 focus area. | High |
| WA-9.3 | Cache API | `caches`, `Cache` | [Service Workers](https://w3c.github.io/ServiceWorker/#cache-objects) | Y / Y / Y | Medium |
| WA-9.4 | Storage Standard | `navigator.storage.estimate()`, `persist()`, buckets model, eviction | [Storage](https://storage.spec.whatwg.org/) | Y / Y / Y | Medium |
| WA-9.5 | Storage Buckets | Named storage buckets with separate durability and quota | [Storage Buckets](https://wicg.github.io/storage-buckets/) | Chromium only | Medium |
| WA-9.6 | Cookie Store API | Async `cookieStore.get/set/delete`, `change` events, `maxAge` | [Cookie Store](https://cookiestore.spec.whatwg.org/) | Y / 140 / 18.4 (Baseline newly available, June 2025). `maxAge`: Chrome 145, Safari 27.0. | High |
| WA-9.7 | File API | `Blob`, `File`, `FileReader`, blob URLs, `Blob.stream()`/`text()`/`bytes()` | [File API](https://w3c.github.io/FileAPI/) | Y / Y / Y | Medium |
| WA-9.8 | Origin private file system (OPFS) | `navigator.storage.getDirectory()`, `FileSystemHandle`, sync access handles in workers, writable streams | [File System](https://fs.spec.whatwg.org/) | Y / Y / Y (Baseline 2023). `createWritable` in Safari 26.0. | High |
| WA-9.9 | File System Access | `showOpenFilePicker`, `showSaveFilePicker`, `showDirectoryPicker`, persistent handles, drag-and-drop handles | [File System Access](https://wicg.github.io/file-system-access/) | Chromium only (desktop) | Medium |
| WA-9.10 | File System Observer | Change notifications on file handles | [File System Observer](https://github.com/whatwg/fs/blob/main/proposals/FileSystemObserver.md) | Chromium only | Medium |
| WA-9.11 | File and Directory Entries API | `webkitGetAsEntry`, `FileSystemDirectoryReader` | [Entries API](https://wicg.github.io/entries-api/) | Y / Y / Y | Medium |
| WA-9.12 | Removed storage | Web SQL (removed from Chromium, never in Gecko), Application Cache (removed everywhere) | [Web SQL (note)](https://www.w3.org/TR/webdatabase/) | Removed / N / removed | Medium |

**Implications:** Storage is where partitioning, quota and eviction meet privacy rules. Izcal's storage spec must cover OPFS and Cookie Store, which are Baseline, and decide on the Chromium-only File System Access pickers.

## 10. Service workers, background work, push and notifications

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-10.1 | Service Workers | Registration, lifecycle, `fetch` interception, `Clients`, update checks | [Service Workers](https://w3c.github.io/ServiceWorker/) | Y / Y / Y (Baseline widely available) | Medium |
| WA-10.2 | Module service workers | `register(url, { type: "module" })` | [Service Workers](https://w3c.github.io/ServiceWorker/#dom-registrationoptions-type) | Y / 147 / Y (Baseline newly available, January 2026) | High |
| WA-10.3 | Navigation preload | Parallel navigation request while SW boots | [Service Workers](https://w3c.github.io/ServiceWorker/#navigationpreloadmanager) | Y / Y / 15.4 | Medium |
| WA-10.4 | Static routing API | `InstallEvent.addRoutes()` to bypass the SW for chosen requests | [Service Worker static routing](https://w3c.github.io/ServiceWorker/#dom-installevent-addroutes) | 123 / N / 27.0 (two engines) | High |
| WA-10.5 | Push API | `PushManager.subscribe`, `push` event, VAPID | [Push API](https://w3c.github.io/push-api/) | Y / Y / 16 (iOS 16.4 for Home Screen apps) | Medium |
| WA-10.6 | Declarative Web Push | Push messages shown without a service worker | [Declarative Web Push](https://github.com/w3c/push-api/pull/385) | N / N / 18.4 (WebKit only) | Low |
| WA-10.7 | Notifications API | `Notification`, `showNotification`, permission, `actions` (Firefox 152), `navigate` option (Firefox 158) | [Notifications](https://notifications.spec.whatwg.org/) | Y / Y / Y (Safari for web apps on iOS). `actions` Chromium and Gecko. | High |
| WA-10.8 | Background Sync | One-off `sync` event when connectivity returns | [Background Sync](https://wicg.github.io/background-sync/spec/) | Chromium only | Medium |
| WA-10.9 | Periodic Background Sync | Periodic `periodicsync` for installed apps | [Periodic Background Sync](https://wicg.github.io/periodic-background-sync/) | Chromium only | Medium |
| WA-10.10 | Background Fetch | Large downloads that outlive the page | [Background Fetch](https://wicg.github.io/background-fetch/) | Chromium only (CORS enforced from Chrome 154) | High |
| WA-10.11 | Content Index | Register offline content for the OS | [Content Index](https://wicg.github.io/content-index/spec/) | Chromium Android only | Medium |

**Implications:** Service workers are Baseline and power offline, push and installability. The background family (sync, periodic sync, fetch) is Chromium-only, so Izcal can stage it, but push and notifications are required for PWA parity.

## 11. Installable web apps and OS integration

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-11.1 | Web App Manifest | `manifest.json`: name, icons, display, scope, `start_url`, `id`, shortcuts; localization (Chrome 148) | [Web App Manifest](https://www.w3.org/TR/appmanifest/) | Chromium Y / partial / Y (Safari 26.0 opens every Home Screen site as a web app) | High |
| WA-11.2 | Install prompts | `beforeinstallprompt`, Web Install API (`navigator.install`), `<install>` element | [Web Install](https://github.com/MicrosoftEdge/MSEdgeExplainers/blob/main/WebInstall/explainer.md) | Chromium only (Web Install OT Chrome 143; element OT Chrome 148) | High |
| WA-11.3 | Launch Handler and File Handling | `launch_handler`, `launchQueue`, `file_handlers` | [Launch Handler](https://wicg.github.io/web-app-launch/) | Chromium only (Chrome 146 refined file launches) | High |
| WA-11.4 | Protocol handlers | `registerProtocolHandler`, manifest `protocol_handlers` | [HTML §8.9.1](https://html.spec.whatwg.org/multipage/system-state.html#custom-handlers) | Y / Y / N | Medium |
| WA-11.5 | Web Share Target | Manifest `share_target` | [Web Share Target](https://w3c.github.io/web-share-target/) | Chromium Android/ChromeOS only | Medium |
| WA-11.6 | Badging API | `navigator.setAppBadge()` | [Badging](https://w3c.github.io/badging/) | Chromium Y / N / Y (Home Screen apps) | Medium |
| WA-11.7 | Window Controls Overlay | Title-bar area for installed desktop apps | [WCO](https://wicg.github.io/window-controls-overlay/) | Chromium only | Medium |
| WA-11.8 | Window Management | `getScreenDetails()`, multi-screen placement | [Window Management](https://w3c.github.io/window-management/) | Chromium only | Medium |
| WA-11.9 | Document Picture-in-Picture | Always-on-top window with arbitrary HTML | [Document PiP](https://wicg.github.io/document-picture-in-picture/) | 116 / 151 (desktop) / N | High |
| WA-11.10 | Isolated Web Apps and their APIs | Signed bundle apps with Direct Sockets, Controlled Frame, Smart Card, Sub Apps, Web Printing, Window Shape | [IWA](https://github.com/WICG/isolated-web-apps) | Chromium (mostly ChromeOS) only | High |
| WA-11.11 | Get Installed Related Apps | `navigator.getInstalledRelatedApps()` | [WICG](https://wicg.github.io/get-installed-related-apps/spec/) | Chromium only | Medium |
| WA-11.12 | PWA origin migration | Move an installed app to a new same-site origin | [Manifest migration explainer](https://github.com/WICG/manifest-incubations) | 150 / N / N | High |

**Implications:** Installability differs most between engines. Izcal should define its install model (manifest support, Home Screen-style install, OS integration) as a shell feature with a clear subset of manifest members.

## 12. Permissions, clipboard, sharing and other user-mediated APIs

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-12.1 | Permissions API | `navigator.permissions.query()`, `PermissionStatus` change events | [Permissions](https://w3c.github.io/permissions/) | Y / Y / Y (Baseline 2022) | Medium |
| WA-12.2 | Permissions Policy | `Permissions-Policy` header, iframe `allow`, `document.featurePolicy` | [Permissions Policy](https://w3c.github.io/webappsec-permissions-policy/) | Header Chromium only; `allow` attribute Y / Y / Y | Medium |
| WA-12.3 | Document Policy | `Document-Policy` header | [Document Policy](https://wicg.github.io/document-policy/) | Chromium only | Medium |
| WA-12.4 | Async Clipboard | `navigator.clipboard.read/write`, `ClipboardItem`, paste prompts (Firefox 147) | [Clipboard API](https://w3c.github.io/clipboard-apis/) | Y / Y / Y (Baseline 2024). Chrome 149 defers data fetch until `getType()`. | High |
| WA-12.5 | Clipboard events and custom formats | `copy`/`cut`/`paste` events; web custom formats; `clipboardchange` (Chrome 144) | [Clipboard API](https://w3c.github.io/clipboard-apis/) | Events Y / Y / Y. Custom formats Chromium (Firefox Android flag). `clipboardchange` Chromium only. | High |
| WA-12.6 | Web Share | `navigator.share()`, `canShare()` | [Web Share](https://w3c.github.io/web-share/) | Chromium (Android, ChromeOS, Windows, macOS) / Android only / Y | Medium |
| WA-12.7 | Contact Picker | `navigator.contacts.select()` | [Contact Picker](https://w3c.github.io/contact-picker/) | Chromium Android only | Medium |
| WA-12.8 | EyeDropper | Pick a colour from the screen | [EyeDropper](https://wicg.github.io/eyedropper-api/) | Chromium desktop only | Medium |
| WA-12.9 | Local Font Access | `queryLocalFonts()` | [Local Font Access](https://wicg.github.io/local-font-access/) | Chromium desktop only | Medium |
| WA-12.10 | Web Speech API | `speechSynthesis`; `SpeechRecognition` with on-device options, phrase lists | [Web Speech](https://webaudio.github.io/web-speech-api/) | Synthesis Y / Y / Y. Recognition Chromium and WebKit; Firefox 158 Nightly on-device. May overlap the media strand. | High |
| WA-12.11 | Virtual Keyboard API | `navigator.virtualKeyboard`, overlay control | [VirtualKeyboard](https://w3c.github.io/virtual-keyboard/) | Chromium only | Medium |
| WA-12.12 | Selective permissions intervention | Blocks ad scripts from using granted permissions | [Chrome 146 notes](https://developer.chrome.com/release-notes/146) | Chromium only (browser policy) | High |

**Implications:** Permission prompts are a shell concern and an API concern. Izcal needs one permission model that covers the Permissions API, Permissions Policy, capability elements and user activation.

## 13. Security and isolation (page-visible)

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-13.1 | Same-origin policy and `document.domain` | Origin checks; `document.domain` setter deprecated behind Origin-Agent-Cluster | [HTML §7.1](https://html.spec.whatwg.org/multipage/browsers.html#origin) | Y / Y / Y. Origin-keyed agent clusters by default in Chromium. | Medium |
| WA-13.2 | Content Security Policy Level 3 | `script-src`, nonces, hashes, `strict-dynamic`, `frame-ancestors`, reporting | [CSP 3](https://w3c.github.io/webappsec-csp/) | Y / Y / Y (Baseline widely available). Chrome 141 OT for script-src-v2. | High |
| WA-13.3 | Trusted Types | `trustedTypes.createPolicy`, `TrustedHTML`, `require-trusted-types-for` | [Trusted Types](https://w3c.github.io/trusted-types/dist/spec/) | 83 / 148 / 26.0 (Baseline newly available, 24 February 2026) | High |
| WA-13.4 | Cross-origin isolation | COOP, COEP (`require-corp`, `credentialless`), CORP, `crossOriginIsolated` | [HTML](https://html.spec.whatwg.org/multipage/browsers.html#cross-origin-opener-policies); [Fetch](https://fetch.spec.whatwg.org/#cross-origin-resource-policy-header) | Y / Y / Y. COEP `credentialless` Chromium and Gecko. | Medium |
| WA-13.5 | Document-Isolation-Policy | Per-document isolation without COOP | [WICG](https://github.com/WICG/document-isolation-policy) | Chromium only | Low |
| WA-13.6 | Fetch Metadata | `Sec-Fetch-Site`, `-Mode`, `-Dest`, `-User` request headers | [Fetch Metadata](https://w3c.github.io/webappsec-fetch-metadata/) | Y / Y / 16.4 (Baseline 2023) | Medium |
| WA-13.7 | Referrer Policy | `Referrer-Policy`, `referrerpolicy` | [Referrer Policy](https://w3c.github.io/webappsec-referrer-policy/) | Y / Y / Y | Medium |
| WA-13.8 | Mixed content and HTTPS upgrades | Block or upgrade mixed content, `upgrade-insecure-requests`, HSTS, HTTPS-first | [Mixed Content](https://w3c.github.io/webappsec-mixed-content/) | Y / Y / Y. Chrome 154 warns before HTTP sites by default. | High |
| WA-13.9 | Secure contexts | `isSecureContext`, gating of powerful features | [Secure Contexts](https://w3c.github.io/webappsec-secure-contexts/) | Y / Y / Y. Safari 26.4 restricted device-motion events to secure contexts. | High |
| WA-13.10 | iframe sandbox and X-Frame-Options | `sandbox` flags, `X-Frame-Options` | [HTML](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#attr-iframe-sandbox) | Y / Y / Y | Medium |
| WA-13.11 | Clear-Site-Data | Clear cookies, storage, cache (and bfcache) from a response | [Clear Site Data](https://w3c.github.io/webappsec-clear-site-data/) | Y / Y (cache clears bfcache from 141) / partial | High |
| WA-13.12 | Local Network Access | Permission for requests to local and loopback addresses | [Local Network Access](https://wicg.github.io/local-network-access/) | 142 (split permissions 145; WebSockets/WebTransport 147) / N / N | High |
| WA-13.13 | Connection Allowlists | Declare allowed network destinations for a document | [WICG Connection Allowlists](https://github.com/WICG/connection-allowlists) | 152 / N / N | High |
| WA-13.14 | Web Cryptography | `crypto.getRandomValues`, `randomUUID`, `SubtleCrypto`, Ed25519/X25519; ML-KEM experimental | [Web Crypto](https://w3c.github.io/webcrypto/) | Y / Y / Y. ML-KEM Firefox 157 flag; algorithm OT Chrome 151. Izcal has a WebCrypto spec. | High |

**Implications:** These features decide which content can run and with which privileges. Trusted Types and Integrity-Policy became Baseline in the last year, so they are now part of the minimum security surface.

## 14. Privacy, tracking protection and advertising APIs

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-14.1 | Storage Access API | `document.requestStorageAccess()`, `hasStorageAccess()`; Storage Access headers | [Storage Access](https://privacycg.github.io/storage-access/) | Y / Y / Y (Baseline 2023). Headers Chrome 133, Firefox Nightly flag. Safari 26.2 changed same-site behaviour. | High |
| WA-14.2 | `requestStorageAccessFor` and Related Website Sets | First-party sets and top-level storage access grants | [RWS](https://wicg.github.io/first-party-sets/) | Chromium only; retirement announced 17 October 2025; deprecation and removal planned in Chrome 153 | High |
| WA-14.3 | Storage and cookie partitioning | Third-party storage keyed by top-level site; CHIPS opt-in | [Storage partitioning](https://privacycg.github.io/storage-partitioning/); [CHIPS](https://privacycg.github.io/CHIPS/) | Partitioning: Gecko and WebKit by default; Chromium for storage, cookies unpartitioned by default. CHIPS kept by Google. | Medium |
| WA-14.4 | Bounce tracking mitigations | Clear state of redirect-only trackers | [Nav-Tracking Mitigations](https://privacycg.github.io/nav-tracking-mitigations/) | Y / Y (stateless mode, Firefox 145) / Y | High |
| WA-14.5 | Topics API | Interest topics for ads | [Topics](https://patcg-individual-drafts.github.io/topics/) | Chromium only; retirement announced 17 October 2025 | High |
| WA-14.6 | Protected Audience | On-device ad auctions (interest groups) | [Protected Audience](https://wicg.github.io/turtledove/) | Chromium only; deprecated Chrome 144; removal planned (Chrome 153) | High |
| WA-14.7 | Shared Storage and selectURL | Cross-site storage readable only in worklets | [Shared Storage](https://wicg.github.io/shared-storage/) | Chromium only; deprecated Chrome 144; removal planned | High |
| WA-14.8 | Private Aggregation | Aggregatable reports from worklets | [Private Aggregation](https://patcg-individual-drafts.github.io/private-aggregation-api/) | Chromium only; removed in Chrome 152 | High |
| WA-14.9 | Attribution Reporting | Ad conversion measurement | [Attribution Reporting](https://wicg.github.io/attribution-reporting-api/) | Chromium only; retirement announced; deprecation and removal planned in Chrome 153. A new interoperable standard is in progress at the W3C PATWG. | High |
| WA-14.10 | Private Click Measurement | WebKit's click-through attribution | [PCM](https://privacycg.github.io/private-click-measurement/) | N / N / Y (WebKit only) | Medium |
| WA-14.11 | Private State Tokens | Anti-fraud tokens issued and redeemed across sites | [Private State Token](https://wicg.github.io/trust-token-api/) | Chromium only; Google says it will maintain it | High |
| WA-14.12 | Private Verification Tokens | Successor anti-fraud token proposal | [Chrome 154 notes](https://developer.chrome.com/release-notes/154) | OT in Chrome 154 | High |
| WA-14.13 | Global Privacy Control | `navigator.globalPrivacyControl` and `Sec-GPC` | [GPC](https://w3c.github.io/gpc/) | N / Y / WKWebView option (Safari 27.0); Chromium no | Medium |
| WA-14.14 | User-Agent reduction and string freezes | Frozen UA strings; Safari 26.0 froze the iOS version | [UA reduction](https://www.chromium.org/updates/ua-reduction/) | Chromium (policy removed Chrome 145) / Y / Y | High |
| WA-14.15 | Fingerprinting protections | Script-level blocks of fingerprinting signals (WebKit 26.0), resistance modes (Gecko) | Vendor policy, no single spec | Browser-specific | High |
| WA-14.16 | IP Protection | Proxying third-party traffic | [Google update](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies) | Chromium retired it (17 October 2025) | High |

**Implications:** Google announced on 17 October 2025 that it would retire most Privacy Sandbox APIs, and removals are under way in Chrome 144–153. Izcal can skip the retired ad APIs. It still needs Storage Access, partitioning and CHIPS, which all three engines ship or keep.

## 15. Identity, credentials and payments

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-15.1 | Credential Management | `navigator.credentials.get/create/store`, `PasswordCredential`, `FederatedCredential` | [Credential Management](https://w3c.github.io/webappsec-credential-management/) | Container Y / Y / Y; Password/Federated credentials Chromium only | Medium |
| WA-15.2 | Web Authentication (passkeys) | `PublicKeyCredential`, conditional mediation, related origins, Signal API, PRF, hybrid transport, immediate mediation | [WebAuthn L3](https://w3c.github.io/webauthn/) | Y / Y / Y (Baseline). Signal API Safari 26.0; PRF Safari 26.4; immediate mode Chrome 148. | High |
| WA-15.3 | FedCM | Browser-mediated federated sign-in, Login Status API, IdP registration | [FedCM](https://w3c-fedid.github.io/FedCM/) | Chromium only; Google keeps supporting it | High |
| WA-15.4 | Digital Credentials API | `navigator.credentials.get({digital})` for wallet credentials (mdoc, OpenID4VP); issuance | [Digital Credentials](https://w3c-fedid.github.io/digital-credentials/) | 141 / N / 26.0 (two engines). Issuance OT Chrome 143. | High |
| WA-15.5 | WebOTP | SMS one-time-code retrieval | [WebOTP](https://wicg.github.io/web-otp/) | Chromium Android only | Medium |
| WA-15.6 | Device Bound Session Credentials | Session cookies bound to a device key | [DBSC](https://w3c.github.io/webappsec-dbsc/) | 145 / N / N | High |
| WA-15.7 | Email Verification Protocol | Browser-mediated email ownership proof | [Chrome 150 notes](https://developer.chrome.com/release-notes/150) | OT Chrome 150 | High |
| WA-15.8 | Payment Request API | `PaymentRequest`, `show()`, payment methods | [Payment Request](https://w3c.github.io/payment-request/) | Y / N / Y (Apple Pay) | Medium |
| WA-15.9 | Payment Handler | Web-based payment apps | [Payment Handler](https://w3c.github.io/web-based-payment-handler/) | Chromium only | Medium |
| WA-15.10 | Secure Payment Confirmation | WebAuthn-backed payment confirmation | [SPC](https://w3c.github.io/secure-payment-confirmation/) | Chromium only (browser-bound keys Chrome 145; capability query Chrome 148) | High |
| WA-15.11 | Apple Pay JS and merchant validation | `ApplePaySession` | [Apple Pay on the Web](https://developer.apple.com/documentation/apple_pay_on_the_web) | WebKit only (vendor API) | Medium |

**Implications:** Passkeys are Baseline and expected by every major site. Digital Credentials is in two engines and ties the browser to platform wallets, so Izcal must decide early which OS credential stores it will bridge.

## 16. Device and hardware APIs

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-16.1 | Geolocation | `navigator.geolocation` | [Geolocation](https://w3c.github.io/geolocation/) | Y / Y / Y (Baseline widely available). `<geolocation>` element Chrome 144. | High |
| WA-16.2 | Device orientation and motion events | `deviceorientation`, `devicemotion`, `requestPermission()` | [Device Orientation](https://w3c.github.io/deviceorientation/) | Y (`requestPermission` 151) / Y / Y (secure contexts only since 26.4) | High |
| WA-16.3 | Generic Sensor family | Accelerometer, Gyroscope, Magnetometer, orientation sensors, Ambient Light | [Generic Sensor](https://w3c.github.io/sensors/) | Chromium only (Magnetometer and Ambient Light behind flags) | Medium |
| WA-16.4 | Battery Status | `navigator.getBattery()` | [Battery Status](https://w3c.github.io/battery/) | Chromium only (Firefox removed it) | Medium |
| WA-16.5 | Device Memory | `navigator.deviceMemory`, `Device-Memory` hint | [Device Memory](https://w3c.github.io/device-memory/) | Chromium only (values updated Chrome 147) | High |
| WA-16.6 | Compute Pressure | `PressureObserver` for CPU pressure | [Compute Pressure](https://w3c.github.io/compute-pressure/) | Chromium only | Medium |
| WA-16.7 | CPU Performance API | Coarse CPU performance tier | [Chrome 152 notes](https://developer.chrome.com/release-notes/152) | 152 / N / N | High |
| WA-16.8 | Screen Wake Lock | `navigator.wakeLock.request("screen")` | [Screen Wake Lock](https://w3c.github.io/screen-wake-lock/) | Y / 126 / 16.4 (Baseline newly available, May 2024) | Medium |
| WA-16.9 | Vibration | `navigator.vibrate()` | [Vibration](https://w3c.github.io/vibration/) | Y / Y / N | Medium |
| WA-16.10 | Screen Orientation | `screen.orientation`, `lock()` | [Screen Orientation](https://w3c.github.io/screen-orientation/) | Y / Y (`lock` on Android and tablets, 144) / Y (no `lock`) | High |
| WA-16.11 | Idle Detection | `IdleDetector` | [Idle Detection](https://wicg.github.io/idle-detection/) | Chromium only | Medium |
| WA-16.12 | Gamepad | `navigator.getGamepads()`, connect events, haptics; event-driven input OT (Chrome 149) | [Gamepad](https://w3c.github.io/gamepad/) | Y / Y / Y (Baseline widely available) | High |
| WA-16.13 | Web MIDI | MIDI input and output, SysEx permission | [Web MIDI](https://webaudio.github.io/web-midi-api/) | Y / Y (108) / N | Medium |
| WA-16.14 | Web Serial | Serial ports | [Web Serial](https://wicg.github.io/serial/) | Y (desktop; Android 148) / 151 (desktop, needs site-permission add-on) / N | High |
| WA-16.15 | WebUSB | USB devices | [WebUSB](https://wicg.github.io/webusb/) | Chromium only | Medium |
| WA-16.16 | WebHID | HID devices | [WebHID](https://wicg.github.io/webhid/) | Chromium desktop only | Medium |
| WA-16.17 | Web Bluetooth | BLE GATT devices | [Web Bluetooth](https://webbluetoothcg.github.io/web-bluetooth/) | Chromium only | Medium |
| WA-16.18 | Web NFC | NDEF read/write | [Web NFC](https://w3c.github.io/web-nfc/) | Chromium Android only | Medium |
| WA-16.19 | Device Posture | `navigator.devicePosture`, foldable posture | [Device Posture](https://w3c.github.io/device-posture/) | Y (Chrome 132) / N / flag (Safari 27.1) | High |
| WA-16.20 | Viewport Segments | `window.viewport.segments` for dual-screen devices | [Viewport Segments](https://w3c.github.io/viewport-segments/) | Y / N / flag (Safari 27.1) | High |
| WA-16.21 | Shape Detection | `BarcodeDetector`, `FaceDetector`, `TextDetector` | [Shape Detection](https://wicg.github.io/shape-detection-api/) | Chromium (barcode on some platforms) only | Medium |
| WA-16.22 | Presentation API | Cast to second screens | [Presentation API](https://w3c.github.io/presentation-api/) | Chromium only (media strand overlap) | Medium |
| WA-16.23 | Input Device Capabilities | `sourceCapabilities.firesTouchEvents` | [InputDeviceCapabilities](https://wicg.github.io/input-device-capabilities/) | Chromium only | Medium |
| WA-16.24 | Managed device attributes | Device Attributes API for managed ChromeOS | [Device Attributes](https://wicg.github.io/WebApiDevice/device_attributes/) | ChromeOS managed only | High |

**Implications:** Most hardware APIs are Chromium-only and are a policy choice, not a compatibility requirement. Geolocation, orientation, gamepad and wake lock are Baseline and needed for parity.

## 17. Performance, measurement and reporting

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-17.1 | High Resolution Time | `performance.now()`, `timeOrigin`, coarsening | [HR Time](https://w3c.github.io/hr-time/) | Y / Y / Y | Medium |
| WA-17.2 | Performance Timeline | `PerformanceObserver`, `getEntries*`, entry type registry | [Performance Timeline](https://w3c.github.io/performance-timeline/) | Y / Y / Y. Declarative observer OT Chrome 151. | High |
| WA-17.3 | Navigation Timing 2 | `PerformanceNavigationTiming`, `confidence` (Chrome 145) | [Navigation Timing](https://w3c.github.io/navigation-timing/) | Y / Y / Y | High |
| WA-17.4 | Resource Timing 3 | `PerformanceResourceTiming`, interim response times, `contentType`, `deliveryType` | [Resource Timing](https://w3c.github.io/resource-timing/) | Y / Y / Y. RT3 fields Safari 26.4, Firefox 152; `contentType` Chrome 148. | High |
| WA-17.5 | User Timing 3 | `performance.mark()`/`measure()` with details | [User Timing](https://w3c.github.io/user-timing/) | Y / Y / Y | Medium |
| WA-17.6 | Server Timing | `Server-Timing` header to `serverTiming` | [Server Timing](https://w3c.github.io/server-timing/) | Y / Y / Y (Baseline 2023) | Medium |
| WA-17.7 | Paint Timing | First Paint, First Contentful Paint; `paintTime`/`presentationTime` (Chrome 145) | [Paint Timing](https://w3c.github.io/paint-timing/) | Y / Y / Y | High |
| WA-17.8 | Largest Contentful Paint | LCP entries | [LCP](https://w3c.github.io/largest-contentful-paint/) | 77 / 122 / 26.2 (Baseline newly available, December 2025) | High |
| WA-17.9 | Event Timing and INP | `PerformanceEventTiming`, `interactionId`, `interactionCount` | [Event Timing](https://w3c.github.io/event-timing/) | Y / Y (`interactionId` 144) / 26.2 (Baseline newly available, December 2025). `interactionCount` Chrome 144. | High |
| WA-17.10 | Layout Instability | `layout-shift` entries (CLS) | [Layout Instability](https://wicg.github.io/layout-instability/) | Chromium only | Medium |
| WA-17.11 | Long Tasks and Long Animation Frames | `longtask` and `long-animation-frame` entries with script attribution | [Long Tasks](https://w3c.github.io/longtasks/); [LoAF](https://w3c.github.io/long-animation-frames/) | Chromium only | Medium |
| WA-17.12 | Element Timing and Container Timing | Paint times for marked elements and containers | [Element Timing](https://wicg.github.io/element-timing/); [Container Timing](https://github.com/bloomberg/container-timing) | Element Timing Chromium only. Container Timing OT Chrome 148, Firefox 156 flag. | High |
| WA-17.13 | Soft navigations | `soft-navigation` and `interaction-contentful-paint` entries for SPA routes | [Soft Navigations](https://wicg.github.io/soft-navigations/) | 151 / N / N | High |
| WA-17.14 | Memory and profiling | `measureUserAgentSpecificMemory()`, JS Self-Profiling (`Profiler`), profiling markers OT (Chrome 153) | [Measure Memory](https://wicg.github.io/performance-measure-memory/); [JS Self-Profiling](https://wicg.github.io/js-self-profiling/) | Chromium only | High |
| WA-17.15 | Reporting API | `ReportingObserver`, `Reporting-Endpoints`; CSP, integrity, deprecation, intervention and crash reports | [Reporting](https://w3c.github.io/reporting/) | Y / Y (CSP/integrity reports 149) / Y | High |
| WA-17.16 | Network Error Logging | `NEL` header for network failure reports | [NEL](https://w3c.github.io/network-error-logging/) | Chromium only | Medium |
| WA-17.17 | Crash Report API | `window.crashReport` key-value context for crash reports | [Chrome 145 notes](https://developer.chrome.com/release-notes/145) | 145 / flag (Firefox 152 Nightly reports) / N | High |
| WA-17.18 | Scroll performance timing | Scroll smoothness entries (WPT has a directory) | Proposal; status not confirmed | Unknown | Low |

**Implications:** Core Web Vitals (LCP, INP, CLS) now have two of three metrics Baseline. Izcal must expose timing entries that match other engines, or real-user monitoring scripts will misreport it.

## 18. ECMAScript language (coarse grain)

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-18.1 | ES5 core and Annex B | Objects, functions, strict mode, JSON, legacy web features (`__proto__`, HTML comments, legacy RegExp statics) | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y. Firefox 148 shipped the Legacy RegExp features proposal. | High |
| WA-18.2 | ES2015 syntax | `let`/`const`, arrows, classes, destructuring, spread, template literals, default params | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y | Medium |
| WA-18.3 | ES2015 library | Promise, Map/Set/WeakMap/WeakSet, Symbol, Proxy, Reflect, typed arrays, iterators, generators | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y | Medium |
| WA-18.4 | Proper tail calls | Guaranteed tail-call elimination in strict mode | [ECMA-262](https://tc39.es/ecma262/#sec-tail-position-calls) | N / N / Y (WebKit only) | Medium |
| WA-18.5 | Modules | `import`/`export`, dynamic import, `import.meta`, top-level await | [ECMA-262](https://tc39.es/ecma262/#sec-modules) | Y / Y / Y | Medium |
| WA-18.6 | Async functions and async iteration | `async`/`await`, `for await`, async generators | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y | Medium |
| WA-18.7 | ES2018–2021 additions | Object rest/spread, RegExp named groups, lookbehind, `\p{}`, BigInt, optional chaining, `??`, `globalThis`, logical assignment, numeric separators, `Promise.any`/`allSettled` | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y | Medium |
| WA-18.8 | Weak references | `WeakRef`, `FinalizationRegistry`; Symbols as WeakMap keys | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y. Symbol keys: Firefox 146 (Baseline newly available, December 2025). | High |
| WA-18.9 | ES2022 classes | Public and private fields, private methods, `#x in obj`, static blocks | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y | Medium |
| WA-18.10 | ES2022–2024 library | `.at()`, `Object.hasOwn`, `Error` cause, RegExp `/d` and `/v`, `findLast`, change-array-by-copy, `Object.groupBy`, `Promise.withResolvers`, well-formed strings | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y | Medium |
| WA-18.11 | Shared memory | `SharedArrayBuffer`, `Atomics`, `Atomics.waitAsync`, `Atomics.pause` | [ECMA-262](https://tc39.es/ecma262/#sec-atomics-object) | Y / Y / Y. `waitAsync` Firefox 145 (Baseline newly available, November 2025). `pause` not confirmed in WebKit. | High |
| WA-18.12 | Resizable and transferable ArrayBuffers | `resize()`, `transfer()`, growable SAB | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y | Medium |
| WA-18.13 | Iterator helpers | `Iterator.prototype.map/filter/take/drop/flatMap/reduce/toArray`, `Iterator.from` | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / 18.4 (Baseline 2025) | Medium |
| WA-18.14 | Iterator sequencing, joint iteration and new methods | `Iterator.concat`; `Iterator.zip`/`zipKeyed`; `includes`; `join`; `chunks`/`windows` | [TC39 proposals](https://github.com/tc39/proposals) | `concat`: 146 / 147 / 26.4 (Baseline newly available, March 2026). `zip`: 153 / 148 / N. `join`: 153 / 154 / N. `includes`: 154 / 154 / N. `chunks`/`windows`: Firefox 154 only. | High |
| WA-18.15 | Set methods | `union`, `intersection`, `difference`, `isSubsetOf`, etc. | [ECMA-262](https://tc39.es/ecma262/) | Y / Y / Y (Baseline 2024) | Medium |
| WA-18.16 | ES2025 batch | Import attributes and JSON modules, `RegExp.escape`, RegExp modifiers, duplicate named groups, `Promise.try`, `Float16Array` | [ECMA-262 2025](https://tc39.es/ecma262/2025/) | Y / Y / Y. RegExp modifiers Safari 26.0. | High |
| WA-18.17 | Explicit resource management | `using`, `await using`, `DisposableStack`, `Symbol.dispose` | [TC39 proposal](https://tc39.es/proposal-explicit-resource-management/) | 134 / 141 / N | High |
| WA-18.18 | Upsert | `Map/WeakMap.prototype.getOrInsert(Computed)` | [TC39 proposal](https://tc39.es/proposal-upsert/) | 145 / 144 / 26.2 (Baseline newly available, February 2026) | High |
| WA-18.19 | `Math.sumPrecise` | Exact floating-point sum | [TC39 proposal](https://tc39.es/proposal-math-sum/) | 147 / Y / 26.2 (Baseline newly available, April 2026) | High |
| WA-18.20 | Uint8Array base64/hex | `toBase64`, `fromBase64`, `toHex`, `fromHex` | [TC39 proposal](https://tc39.es/proposal-arraybuffer-base64/) | Y / Y / Y | Low |
| WA-18.21 | `Error.isError`, stack APIs | `Error.isError`; non-standard `Error.captureStackTrace`, `Error.stackTraceLimit` (Firefox 153) | [TC39 proposals](https://github.com/tc39/proposals) | `isError` widely shipped; stack APIs vary | Medium |
| WA-18.22 | `Array.fromAsync` | Build arrays from async iterables | [TC39 proposal](https://tc39.es/proposal-array-from-async/) | Y / Y / Y (Baseline 2024) | Medium |
| WA-18.23 | JSON.parse source text access | `JSON.rawJSON`, reviver `context.source` | [TC39 proposal](https://tc39.es/proposal-json-parse-with-source/) | Y / Y / ? | Low |
| WA-18.24 | `Promise.allKeyed` / `allSettledKeyed` | Await an object of promises | [TC39 proposal](https://github.com/tc39/proposal-await-dictionary) | N / 155 / N | High |
| WA-18.25 | Source phase imports and `import defer` | `import source x from`, `import defer * as ns` | [Source phase](https://tc39.es/proposal-source-phase-imports/); [Defer](https://tc39.es/proposal-defer-import-eval/) | Firefox parses `import source` (153); `import defer` Firefox 158 Nightly flag. Others not confirmed. | Medium |
| WA-18.26 | Not yet shipped | Decorators, ShadowRealm, AsyncContext, RegExp buffer boundaries (Firefox Nightly flag 155), `export * default` (Firefox 157 flag) | [TC39 proposals](https://github.com/tc39/proposals) | None ship by default as far as found | Medium |

**Implications:** ECMAScript gains about ten library features a year, and the 2026 additions (upsert, `sumPrecise`, iterator sequencing) reached Baseline within months. Izcal's JS engine needs a fast path from Stage 4 to shipping, not a once-a-year catch-up.

## 19. Intl and Temporal

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-19.1 | Intl core | `Collator`, `DateTimeFormat`, `NumberFormat` v3, `PluralRules`, `RelativeTimeFormat`, `ListFormat`, `DisplayNames`, `Locale`, `supportedValuesOf` | [ECMA-402](https://tc39.es/ecma402/) | Y / Y / Y. Chrome 143 moved to ICU 77 (Unicode 16). Firefox 149 added `islamic-umalqura`. | High |
| WA-19.2 | `Intl.Segmenter` | Grapheme, word and sentence segmentation | [ECMA-402](https://tc39.es/ecma402/#segmenter-objects) | Y / Y / Y (Baseline 2024) | Medium |
| WA-19.3 | `Intl.DurationFormat` | Locale-aware duration formatting | [ECMA-402](https://tc39.es/ecma402/) | Y / Y / Y (Baseline 2025) | Medium |
| WA-19.4 | `Intl.Locale` info and `variants` | `getCalendars()`, `getWeekInfo()` etc.; `variants` | [Intl Locale Info](https://tc39.es/proposal-intl-locale-info/) | Methods: Chromium Y (getters deprecated Chrome 143) / 153 / partial. `variants`: 149 / 141 / 26.0. | High |
| WA-19.5 | `Intl.PluralRules` notation | `notation` option | [ECMA-402](https://tc39.es/ecma402/) | ? / ? / 26.0 | Medium |
| WA-19.6 | Temporal | `Temporal.Instant`, `PlainDate`, `ZonedDateTime`, `Duration`, calendars and time zones | [Temporal](https://tc39.es/proposal-temporal/) | 144 / 139 / N (Safari Technology Preview behind a flag; not in Safari 26.x or 27.x notes) | Medium |

**Implications:** Intl needs a full CLDR/ICU-class data set, and Izcal's MIT-only rule means building or sourcing that data under a compatible licence. Temporal adds time-zone database needs on top.

## 20. WebAssembly

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-20.1 | Wasm MVP and JS API | `WebAssembly.Module/Instance/Memory/Table/Global`, `compile`, `instantiate` | [Wasm JS API](https://webassembly.github.io/spec/js-api/) | Y / Y / Y | Medium |
| WA-20.2 | Wasm Web API | `compileStreaming`, `instantiateStreaming` (options object Safari 26.6) | [Wasm Web API](https://webassembly.github.io/spec/web-api/) | Y / Y / Y | High |
| WA-20.3 | Post-MVP core features | Multi-value, bulk memory, reference types, sign extension, non-trapping conversions, SIMD | [Wasm 2.0](https://webassembly.github.io/spec/core/) | Y / Y / Y | Medium |
| WA-20.4 | Threads | Shared memory and atomics (needs cross-origin isolation) | [Wasm threads](https://github.com/WebAssembly/threads) | Y / Y / Y | Medium |
| WA-20.5 | Exception handling | `try_table`, `exnref`, `WebAssembly.Tag`/`Exception` | [Wasm 3.0](https://webassembly.github.io/spec/core/) | Y / Y / Y | Medium |
| WA-20.6 | GC and tail calls | Struct/array GC types, typed function references, `return_call` | [Wasm 3.0](https://webassembly.github.io/spec/core/) | Y / Y / Y (Baseline December 2024) | Medium |
| WA-20.7 | Memory64 and multi-memory | 64-bit memories, multiple memories | [Wasm 3.0](https://webassembly.github.io/spec/core/) | Y / Y / N | Medium |
| WA-20.8 | Relaxed SIMD | Hardware-dependent SIMD | [Relaxed SIMD](https://github.com/WebAssembly/relaxed-simd) | Y / ? / ? | Low |
| WA-20.9 | JS String Builtins | Fast imported string operations | [JS String Builtins](https://github.com/WebAssembly/js-string-builtins) | 130 / 134 / 26.2 (Baseline newly available, December 2025) | High |
| WA-20.10 | JavaScript Promise Integration (JSPI) | `WebAssembly.Suspending`, `promising` | [JSPI](https://github.com/WebAssembly/js-promise-integration) | 137 / 153 / 27.0 (Baseline newly available, September 2026). Interop 2026 focus area. | High |
| WA-20.11 | Resizable memory buffers | `toResizableBuffer()`, `toFixedLengthBuffer()` | [Wasm JS API](https://webassembly.github.io/spec/js-api/) | ? / ? / 26.2 | Medium |
| WA-20.12 | Newer proposals | Wide arithmetic and compact imports (Firefox 155), custom descriptors (OT Chrome 141 and 149), branch hinting, ESM integration | [Wasm proposals](https://github.com/WebAssembly/proposals) | Mixed; mostly single-engine | High |

**Implications:** Wasm is now a second execution tier that every large app (Figma, Photoshop, game engines) relies on. JSPI reached Baseline in September 2026, so Izcal's Wasm design must support stack switching.

## 21. On-device AI and agent-facing APIs

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-21.1 | Translator and Language Detector | On-device translation and language detection | [Translation API](https://webmachinelearning.github.io/translation-api/) | Chromium only (Chrome 138; zh-Hant/zh-Hans in Chrome 151) | High |
| WA-21.2 | Summarizer, Writer, Rewriter, Proofreader | Task-specific on-device writing APIs | [Writing Assistance APIs](https://webmachinelearning.github.io/writing-assistance-apis/) | Summarizer Chromium Y; others OT (Proofreader OT Chrome 141) | High |
| WA-21.3 | Prompt API | General on-device language model | [Prompt API](https://webmachinelearning.github.io/prompt-api/) | 148 / N / N (sampling parameters OT) | High |
| WA-21.4 | WebNN | Neural network graph API | [WebNN](https://www.w3.org/TR/webnn/) | OT Chrome 146–147 / N / N. Overlaps the graphics strand. | High |
| WA-21.5 | WebMCP | Pages expose tools to browser agents | [WebMCP](https://webmachinelearning.github.io/webmcp/) | Not shipped by default (WPT directory exists) | Low |

**Implications:** These APIs are Chromium-only and tied to a bundled model. Izcal has an ML service spec, which must decide whether to expose any of them to pages.

## 22. Browser-shipped protocols and infrastructure standards

| # | Feature | What it is | Defined by | Ships in (Chromium / Gecko / WebKit), as of Oct 2026 | Conf. |
|---|---|---|---|---|---|
| WA-22.1 | WebDriver (classic) | HTTP automation protocol; `navigator.webdriver` | [WebDriver 2](https://w3c.github.io/webdriver/) | Y / Y / Y (safaridriver) | Medium |
| WA-22.2 | WebDriver BiDi | Bidirectional automation over WebSocket: events, network interception, emulation, script realms | [WebDriver BiDi](https://w3c.github.io/webdriver-bidi/) | Y / Y (active monthly work, e.g. Firefox 141–158) / partial | High |
| WA-22.3 | WebDriver extension commands | Permissions, Storage Access, FedCM, WebAuthn virtual authenticators, Digital Credentials, sensors | Per-spec "Automation" sections | Varies; Safari 27.0 added Digital Credentials commands | High |
| WA-22.4 | Chrome DevTools Protocol | Vendor debugging protocol, used by Puppeteer and many tools | [CDP](https://chromedevtools.github.io/devtools-protocol/) | Chromium only; Firefox removed its CDP subset in Firefox 141 | High |
| WA-22.5 | Browser MCP servers | Agent control of the browser for developers (Safari MCP server, Safari 27.0) | [WebKit blog](https://webkit.org/blog/18136/introducing-the-safari-mcp-server-for-web-developers/) | WebKit (Safari 27.0); Chromium tooling ships separately | High |
| WA-22.6 | Web IDL | Binding language and its semantics (exceptions, promises, observable arrays) | [Web IDL](https://webidl.spec.whatwg.org/) | Y / Y / Y. Chrome 151 removed `[LegacyNoInterfaceObject]` from `FontFaceSet`. | High |
| WA-22.7 | Infra | Shared data types and algorithms for specs | [Infra](https://infra.spec.whatwg.org/) | n/a (spec infrastructure) | High |
| WA-22.8 | Console | `console.log` family, `table`, `time`, `count`, `group` | [Console](https://console.spec.whatwg.org/) | Y / Y / Y | Medium |
| WA-22.9 | Compatibility and Quirks Mode | WebKit-prefixed aliases, `window.orientation`, quirks-mode behaviours | [Compat](https://compat.spec.whatwg.org/); [Quirks](https://quirks.spec.whatwg.org/) | Y / Y / Y. Interop 2026 has a "Web compat" focus area. | High |
| WA-22.10 | Test Utils | `TestUtils.gc()` for test harnesses | [Test Utils](https://testutils.spec.whatwg.org/) | Test builds only | Medium |
| WA-22.11 | WebExtensions | Cross-browser extension APIs (`browser.*`), MV3 | [W3C WebExtensions CG](https://w3c.github.io/webextensions/specification/) | Y / Y / Y (each with differences). Izcal has an extensions spec. | Medium |
| WA-22.12 | Origin trials | Time-limited feature opt-in by token | [Origin trials explainer](https://github.com/GoogleChrome/OriginTrials) | Chromium, Gecko, Edge; not WebKit | Medium |

**Implications:** WebDriver BiDi is how test suites and agents drive browsers, and Interop measures engines through WPT, which runs on WebDriver. Izcal needs WebDriver classic and BiDi early, or it cannot run WPT at scale.

## Coverage check

The WPT repository had 283 top-level directories on 24 October 2026 (including `.github` and `.well-known`). Every directory in this strand maps to a row above. The remaining directories fall into three groups.

**Out of this strand (graphics, media, real-time, CSS, accessibility, text or images), left to the other researcher:** `accessibility`, `accname`, `apng`, `aria`, `audio-output`, `audio-session`, `autoplay-policy-detection`, `avif`, `animation-worklet`, `captured-mouse-events`, `content-dpr`, `core-aam`, `css`, `cssom`, `density-size-correction`, `dpub-aam`, `dpub-aria`, `encrypted-media`, `fonts`, `forced-colors-mode`, `gif`, `graphics-aam`, `graphics-aria`, `html-aam`, `html-ruby-extensions`, `imagebitmap-renderingcontext`, `images`, `jpegxl`, `mathml-aam`, `media`, `media-capabilities`, `media-playback-quality`, `media-source`, `mediacapture-extensions`, `mediacapture-fromelement`, `mediacapture-handle`, `mediacapture-image`, `mediacapture-insertable-streams`, `mediacapture-record`, `mediacapture-region`, `mediacapture-streams`, `mediasession`, `mst-content-hint`, `picture-in-picture`, `png`, `remote-playback`, `screen-capture`, `scroll-animations`, `svg`, `svg-aam`, `video-rvfc`, `wai-aria`, `web-animations`, `webaudio`, `webcodecs`, `webgl`, `webgpu`, `webrtc`, `webrtc-encoded-transform`, `webrtc-extensions`, `webrtc-ice`, `webrtc-identity`, `webrtc-priority`, `webrtc-stats`, `webrtc-svc`, `webvtt`, `webxr`.

**Test infrastructure, not features:** `.github`, `.well-known`, `common`, `conformance-checkers`, `docs`, `infrastructure`, `interfaces`, `old-tests`, `resources`, `third_party`, `tools`, `acid` (legacy Acid tests).

**Could not place in a row, or the feature's status is unclear:**
- `annotation-model`, `annotation-protocol`, `annotation-vocab`: W3C Web Annotation data model and protocol. No browser exposes these to script, as far as found.
- `html-longdesc`: the obsolete `longdesc` attribute, mentioned only inside WA-2.18.
- `payment-method-basic-card`: basic-card was removed from Chromium; it has no row.
- `proximity`, `geolocation-sensor`: sensor specs that never shipped by default; covered only loosely by WA-16.3.
- `scroll-performance-timing`: a new directory with no confirmed spec or shipping status (WA-17.18 is a placeholder).
- `permissions-request`, `permissions-revoke`: withdrawn Permissions API methods; no row.
- `cpu-performance`, `connection-allowlist`, `subapps`, `web-install`, `webmcp`, `managed`, `trust-tokens`, `private-click-measurement`, `merchant-validation`, `web-bundle`, `signed-exchange`, `gpc`, `https-upgrades`, `hsts`, `device-bound-session-credentials`, `digital-credentials`, `direct-sockets`, `fenced-frame`: all placed (WA-16.7, 13.13, 11.10, 11.2, 21.5, 16.24, 14.11, 14.10, 15.11, 5.11, 5.11, 14.13, 13.8, 13.8, 15.6, 15.4, 8.11, 1.9). They are listed here only because the directory name differs from the spec name.

## Gaps in this inventory

- webstatus.dev could not be read in this session (its pages are script-rendered and the reader failed). Baseline labels for features older than mid-2025 come from earlier published status and are marked Medium, not checked against webstatus.dev today.
- Safari's status was read from the 26.0, 26.2, 26.4, 26.5, 26.6, 27.0 and 27.1 posts. The 26.1 and 26.3 posts were not read, so a WebKit feature that shipped only in those releases could be marked N by mistake.
- Chrome release notes before Chrome 141 and Firefox release notes before Firefox 141 were not read. Version numbers older than those (for example Chrome 135 for invoker commands, or Firefox 139 for Temporal) are from memory or secondary sources and are marked Medium unless a later release note confirmed them.
- The Chrome 155 release-notes page returned 404 on 24 October 2026, so Chrome changes after Chrome 154 (22 September 2026) are not covered.
- Temporal in Safari: no WebKit release post through Safari 27.1 lists it. The only source for its Technology Preview flag is a secondary page dated March 2026.
- Low-confidence rows that need a primary check: WA-5.10 (`blocking=render` outside Chromium), WA-10.6 (Declarative Web Push), WA-13.5 (Document-Isolation-Policy), WA-17.18, WA-18.20 (Uint8Array base64 in Chromium), WA-18.23 (JSON.parse source in WebKit), WA-20.8 (relaxed SIMD outside Chromium), WA-21.5 (WebMCP).
- The Topics API's exact removal milestone in Chrome was not found in the Chrome 141–154 notes; only the 17 October 2025 retirement announcement is confirmed.
- The grain is per spec or feature group, so sub-features with split support (for example single Notification options or single WebAuthn extensions) appear only as notes inside rows.
- Engine-specific non-standard APIs (for example WebKit's `webkit*` touch-force events, Gecko's `moz*` leftovers) are not inventoried beyond the Compat row (WA-22.9).
