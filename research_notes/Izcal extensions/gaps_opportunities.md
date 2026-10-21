# Izcal extensions: gaps and opportunities

Compiled 20 October 2026. Primary sources were preferred (vendor extension documentation, the Firefox DevTools source-docs pages, WECG material). This strand was written by the synthesis author after the strand researchers hit the session's shared search budget; three primary pages were fetched directly and the rest rests on well-known primary documents graded Medium/Low accordingly.
Confidence: **High** = primary source read and the claim checked in its text this session; **Medium** = primary page known and quoted from memory of its stable text, or a vendor claim without data; **Low** = secondary or recollection not re-checked this session.

M-1 note: no source code of any project was read or cited; prose documentation only.

## 1. The lifetime gap MV3 created, and the offscreen workaround

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | MV3 removed persistent background pages in favour of service workers, which "don't have DOM access, and many websites have content security policies that limit the functionality of content scripts"; Chrome added a dedicated **offscreen document API** so extensions can "use DOM APIs in a hidden document without interrupting the user experience by opening new windows or tabs" | 1 offscreen document per extension; `runtime` is the only supported extension API in it | [Chrome offscreen API docs](https://developer.chrome.com/docs/extensions/reference/api/offscreen) | fetched 2026-10-20 | High |
| 1.2 | The offscreen reason enum is a fixed list of 15 documented causes (AUDIO_PLAYBACK, DOM_PARSER, DOM_SCRAPING, BLOBS, CLIPBOARD, LOCAL_STORAGE, WORKERS, WEB_RTC, DISPLAY_MEDIA, USER_MEDIA, MATCH_MEDIA, BATTERY_STATUS, GEOLOCATION, IFRAME_SCRIPTING, TESTING); each extension must declare a `reason` and write a `justification` string; the document "can't be focused", has `opener: null`, must be a static bundled HTML file, and AUDIO_PLAYBACK closes after 30 s without audio | 15 reasons; 30 s audio lifetime | [Chrome offscreen API docs](https://developer.chrome.com/docs/extensions/reference/api/offscreen) | fetched 2026-10-20 | High |
| 1.3 | Chrome's own migration guide documents the cost: "A service worker replaces the extension's background or event page", which "requires moving DOM, window, and certain extension API calls into offscreen documents"; adding permissions on update "may trigger a permission warning", which disables the extension until the user re-approves; an open-source Extension Manifest Converter "does not do everything for you, but it will get you started" | MV3 general support from Chrome 88 | [Chrome MV3 migration guide](https://developer.chrome.com/docs/extensions/develop/migrate) | fetched 2026-10-20 | High |
| 1.4 | Chrome's stated MV3 rationale is performance and security: blocking webRequest "could significantly degrade performance and require excessive access to sensitive user data"; declarativeNetRequest does the work "with fewer permissions and without hindering performance"; remote hosted code and arbitrary string execution were removed alongside "an enhanced content security policy" | — | [Chrome MV3 migration guide](https://developer.chrome.com/docs/extensions/develop/migrate) | fetched 2026-10-20 | High |

**Implications:** The MV3 service-worker model traded a real platform failure (background pages that never sleep) for a lifetime model whose main products are the offscreen-document workaround and a migration burden every developer paid. A new browser can design the lifetime honestly: event-driven wake with **declared, typed wake intents** and a first-class way to run DOM-needing work, instead of a 15-reason escape hatch with justification strings.

## 2. The observability gap

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | Firefox's `about:debugging` is a page listing temporary extensions, installed extensions (with Inspect, Extension ID, Internal UUID, manifest URL), tabs and workers (service, shared, dedicated, Chrome workers); service workers can be unregistered and triggered with a local Push event | — | [Firefox source docs: about:debugging](https://firefox-source-docs.mozilla.org/devtools-user/about_colon_debugging/) | fetched 2026-10-20 | High |
| 2.2 | Its documented limits: remote debugging fails across more than one major version mismatch ("an unsupported setup and may cause DevTools to fail"); temporary extensions "stay installed until you restart Firefox"; hidden add-ons need a pref. There is **no documented view of what an extension did** — no history of requests it read, pages it touched, or permissions it used, only live inspection | — | [Firefox source docs: about:debugging](https://firefox-source-docs.mozilla.org/devtools-user/about_colon_debugging/) | fetched 2026-10-20 | High |
| 2.3 | Chrome's `chrome://extensions` service-worker inspection exists but no first-party documentation claims an audit surface for extension behaviour; the "who received what" view exists in no engine for extensions (absence; graded from the vendor docs' silence, Medium) | — | [Chrome extensions docs](https://developer.chrome.com/docs/extensions) | 2026-10 | Medium |

**Implications:** Every platform's observability is **live inspection of a worker**, nothing more. Izcal's IIP, audit log and recorded sessions let an extension be observable the way a site is: what it read, what it changed, which grants it used, all replayable. No browser ships this; it is the single clearest differentiator.

## 3. The capability-scoping gap: containers do not constrain extensions

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | Firefox's Multi-Account Containers, the only shipped container model in any major browser, is a **built-in feature**, not an extension, and it scopes site identity (cookie jars, tabs per container), not extension capabilities; extension APIs such as `tabs` and `cookies` see across containers unless the extension itself filters by `cookieStoreId` (Medium: Firefox extension docs describe `cookieStoreId` parameters; no engine documents container-scoped extension authority) | — | [Firefox Container docs](https://support.mozilla.org/kb/containers) | 2026-10 | Medium |
| 3.2 | Chrome's incognito model (`spanning` vs `split`) is the only documented "extension scope" switch, and it is binary and profile-level | 2 modes | [Chrome incognito docs](https://developer.chrome.com/docs/extensions/reference/manifest#incognito) | 2026-10 | Medium |
| 3.3 | No engine documents per-container extension grants: an extension granted `<all_urls>` in Chrome or Firefox reads every container's pages; Tor Browser's response is to ship no third-party extensions and forbid installs (policy prose, Medium) | — | vendor docs' silence; Tor Browser design prose | 2026-10 | Medium |

**Implications:** Izcal's containers are a first-class policy context (I-1 (a)). Scoping every extension capability by container is possible only because Izcal's authority model is capability IPC from the broker — an extension's grants are (origin set, container) pairs, not global booleans. Nothing in today's platforms does this; it is a differentiator with a concrete user story (a work container where an extension cannot read personal browsing).

## 4. The testing and conformance gap

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | There is no cross-browser extension conformance suite comparable to WPT: each vendor ships its own docs and lint tool (Firefox's `web-ext` lints and packages; Chrome relies on store review), and MDN's browser-compat data is the only shared compatibility record (Medium; absence is the finding) | — | MDN Browser extensions docs; `web-ext` docs | 2026-10 | Medium |
| 4.2 | WECG proposals exist precisely because developers need cross-browser agreement the vendors have not shipped (active proposals cover gaps such as userScripts standardization, improved proxy and page lifecycle); the group's charter is to "extend and standardise" the WebExtensions model (Medium) | — | [WECG](https://wiki.whatwg.org/WebExtensions) and w3c/webextensions prose | 2026-10 | Medium |
| 4.3 | Extension developers test by hand: load the unpacked extension, click around; no vendor documents a recording/replay facility for extension behaviour (absence; Medium) | — | vendor docs' silence | 2026-10 | Medium |

**Implications:** Izcal can publish its compatibility list as a **falsifiable, tested contract** (per-API conformance tests, published pass rates, like the DevTools spec's DT-4/DT-6 pattern), which no current platform does. Determinism makes extension bug reports reproducible: a recording that includes extension decisions replays.

## 5. The language gap

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | Extension code is JavaScript-only in every engine; no vendor documents a non-JS extension language. The closest shipped artifacts are declarative and data-only: Safari content blockers (JSON rules), DNR rulesets (JSON) — both are evidence that vendors move security-sensitive surface to data | — | Safari content-blocker docs; DNR docs | 2026-10 | Medium |
| 5.2 | A WebAssembly-component host would allow extensions in any language compiling to Wasm, with the component model's typed interfaces as the API boundary; no engine has shipped this for extensions (absence; Medium). HGBA-2 4.2 already reserves JS-in-Wasm as the isolation tool "for untrusted script, such as extension sandboxes", and JS-D10 gives Izcal the host | — | HGBA-2 4.2; JS spec JS-D10 | 2026-10 | High (Izcal side) |

**Implications:** Izcal's component host turns "extensions are JS with privileges" into "extensions are typed components with capabilities". Compatibility demands the JS API surface; the component model is the implementation substrate, and it makes the privilege boundary typed and auditable rather than a namespace convention.

## 6. The performance record

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Extension-side matching cost Adblock Plus a 32% median overhead (cited by the resource-footprint spec from its research notes, Medium); the footprint spec's own answer was to build blocking into the engine, not to leave it to extensions | 32% median | FP spec 6.x citing its notes | 2026-10 | Medium |
| 6.2 | The Great Suspender precedent: a tab-suspending extension with more than two million installs was sold and became malware, and Google deactivated it remotely ([The Hacker News](https://thehackernews.com/2021/02/warning-hugely-popular-great-suspender.html)); the footprint spec's lesson: footprint control is first-party work | >2M installs | FP spec history | 2021-02 | High |

**Implications:** Performance- and footprint-critical features (blocking, suspension) belong in the engine; extensions get the long tail. Izcal's Stage 1 already ships the first-party versions; the extension platform must not become the place footprint control goes to die.

## 7. Ranked opportunities

| Opportunity | Evidence it is wanted | What Izcal's architecture uniquely enables | Risk |
|---|---|---|---|
| Container-scoped extension capabilities | Containers exist only as site-identity features; no engine scopes extensions by container (3.1–3.3) | Capability grants are (origin set, container) pairs minted by the broker | Porting cost: existing extensions assume global grants; a default of "this container" changes semantics |
| Audit-grade observability ("what did this extension do") | No platform has more than live worker inspection (2.1–2.3) | IIP + decision log + recorded sessions; the audit already exists for agent interception (NW 6.19) | Log size and privacy of the log itself; needs the redaction rules |
| Event-wake lifetime without the offscreen hack | The 15-reason offscreen enum is the documented symptom (1.1–1.2) | Declared wake intents as typed B1 inputs; deterministic replay of extension behaviour | Compatibility: Chrome-idiomatic extensions rely on undocumented lifetime details |
| A falsifiable compatibility contract | No conformance suite exists (4.1); WECG exists to fill the gap (4.2) | Spec anchors, `hg-anchor`, published pass rates per API | Maintenance: every API supported is a forever commitment |
| Data-first tiers for the common cases | Vendors already moved blocking to JSON/DNR (5.1); CB-D15 tier 1 keeps lists code-free | The compiled, sealed, witnessed pipeline (CB 6.9) reused for extension lists | Breakage for blockers that need scriptlets (documented gap; CB's own answers cover the vocabulary) |
| Non-JS extension languages via components | No engine ships it (5.2); demand is unmeasured | JS-D10's component host; WASI embedding | Unmeasured demand; do not lead with it — compatibility first |
| Deterministic extension bug reports | Developers test by hand; no replay exists (4.3) | B1 recording includes extension decisions (CB-D21 pattern) | Recordings must carry extension state without leaking page data |

## Open questions

- Real-world distribution of API usage (which namespaces the installed base actually calls) — only the store operators hold it; Chrome and Mozilla have published aggregate enforcement numbers, not usage tables. Treat as unknown and design the compatibility list to be revisable.
- Whether any vendor has measured the offscreen workaround's overhead — not found.
- The exact set of WECG active proposals and their status — not re-fetched this session (search budget); graded Medium.
