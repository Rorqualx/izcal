# Izcal extensions: surface and manifest

Compiled 20 October 2026. Primary sources were preferred (MDN, developer.chrome.com, Firefox extension docs, Apple docs, WECG pages).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = snippet/vendor claim without data; **Low** = secondary only.

M-1 note: no source code of any project was read or cited; prose documentation only.

Method notes: fetched and read in full during compilation: MDN `manifest.json` overview, MDN `manifest.json/background`, MDN `API` (namespace index), MDN `Match_patterns`, MDN `externally_connectable`, MDN `userScripts`, MDN `Background_scripts`; developer.chrome.com manifest reference, MV3 API reference list, service-worker lifecycle page, offscreen page, content-scripts page; Chrome Web Store Developer Program Policies (2025-05-22 revision); Mozilla Add-ons blog "Manifest v3 in Firefox: Recap & Next Steps" (2022-05-18, Rob Wu). Apple's Safari Web Extensions pages (`supported_apis`, `assessing_your_safari_web_extension_s_browser_compatibility`) are JavaScript-rendered and returned no body text to any fetcher (built-in fetch, web-reader); every Safari column below is therefore **Medium** unless it appears in MDN text. MDN's `Browser_support_for_JavaScript_APIs` page delivered only its section list (no compatibility tables) — per-browser rows marked from that list are Medium. URLs that 404'd: `developer.chrome.com/docs/extensions/develop/concepts/mv3`, `.../mv3-overview`, `.../mv3-security`, `.../mv3/remote-hosted-code` (site restructure; the store-policies page was used instead).

---

## 1. Manifest keys (MV3, with MV2 names and per-browser notes)

**Required by the platform (Chrome):** only `manifest_version`, `name`, `version`. MDN states the same three are the only mandatory keys; `default_locale` is required iff a `_locales/` directory exists (and must be absent otherwise). Chrome caps `name` at 75 chars, `description` at 132, `short_name` at 12 (Chrome manifest reference; Medium on the exact caps — read on the page but not re-verified against a store validator).

| Key | What it does (one line) | MV2 name/form | Chrome | Firefox | Safari | Conf. |
|---|---|---|---|---|---|---|
| `manifest_version` | Manifest format version; `"3"` is the only value Chrome MV3 accepts | was `2` | Yes (3 only) | 2 or 3 | 2 or 3 | High |
| `name` | Extension display name | same | Yes | Yes | Yes | High |
| `version` | Machine version string | same | Yes | Yes | Yes | High |
| `version_name` | Human-readable version, e.g. `"1.0 beta"` | same | Yes | Yes | Yes | High |
| `short_name` | Compact display name | same | Yes (≤12 chars) | Yes | Yes | High |
| `description` | Store / management-page description | same | Yes | Yes | Yes | High |
| `default_locale` | Default language when `_locales/` present | same | Yes | Yes | Yes | High |
| `icons` | Toolbar, store, management icons | same | Yes | Yes | Yes | High |
| `homepage_url`, `author` | Metadata | same | Yes | Yes (`developer` key is the Firefox variant for author) | Yes | High |
| `action` | Toolbar button (icon, badge, popup), unified successor to both MV2 button types | MV2: `browser_action` + `page_action` (Chrome) | MV3 only | Yes (also keeps `browser_action`/`pageAction` for MV2) | Yes | High |
| `background` | Background context: `service_worker` (MV3), `scripts`/`page` (MV2 + Firefox MV3 event pages), `persistent` (MV2), `type: "module"`, `preferred_environment` | renamed/reshaped in MV3 | `service_worker` only in MV3; MV3 `scripts`/`page` ignored since Chrome 121, load error before | No `service_worker` (bug 1573659); uses `scripts`/`page` event pages | Both; prefers `scripts`/`page` unless `preferred_environment: ["service_worker"]`; synthesizes a service worker from `scripts` if so instructed | High |
| `chrome_settings_overrides` | Override browser settings (homepage, search engine) | same | Yes | Partial (search engine only; Gecko) | — | Medium |
| `chrome_url_overrides` | Replace built-in pages (new tab, bookmarks, history) | same | Yes | Yes | Yes | High |
| `commands` | Keyboard shortcuts bound to extension actions | same | Yes | Yes | Yes | High |
| `content_scripts` | Declarative injection of JS/CSS into matching pages; sub-keys `matches` (required), `js`, `css`, `run_at`, `world` (ISOLATED/MAIN, default ISOLATED), `all_frames`, `match_about_blank`, `match_origin_as_fallback` (Chrome), `exclude_matches`, `include_globs`/`exclude_globs` | same | Yes | Yes (globs too); `world: MAIN` since Firefox 128 | Yes | High |
| `content_security_policy` | CSP for extension pages | MV2: string. MV3: object `{ extension_pages, sandbox }` | Yes (object form; disallows remote script, `unsafe-eval` in `extension_pages`) | Yes; Firefox supports the MV2 string in MV2, object in MV3 | Yes | High |
| `cross_origin_embedder_policy` | COEP header for extension pages that embed cross-origin resources | MV3 | Yes | — (not documented on MDN) | — | Medium |
| `cross_origin_opener_policy` | COOP header to isolate extension pages' browsing contexts | MV3 | Yes | — | — | Medium |
| `declarative_net_request` | Static rule resources + `rule_resources` enabling request blocking/modification without a blocking listener | MV3 | Yes (Chrome 84+ API) | Yes (Firefox 113+) | Yes (15.4+) | High/Medium |
| `devtools_page` | Page that extends DevTools (gets `devtools.*` APIs) | same | Yes | Yes | Yes | High |
| `differential_fingerprint` | Enterprise-managed update fingerprinting (Chrome-only, undocumented on MDN/Chrome reference prose) | same | Yes (enterprise) | No | No | Low |
| `event_rules` | Declarative event registration (paired with deprecated `declarativeContent`); MV2-era | MV2 | Yes (legacy) | No | No | Low |
| `export` | Declare resources this extension shares with others (paired with `import`) | MV3 | Yes | No | No | Medium |
| `import` | Consume another extension's exported shared modules | MV3 | Yes | No | No | Medium |
| `externally_connectable` | Which web pages (`matches`) and which extension IDs (`ids`) may message this extension; default is all extensions, no web pages | same | Yes | **Not supported** for web pages (bug 1319168) | Yes (browser.* form) | High |
| `host_permissions` | Install-time host access declared separately from API permissions | MV3 (split out of MV2 `permissions`) | Yes | Yes (Firefox makes them opt-in at runtime by default) | Yes | High |
| `incognito` | Private-browsing mode: `"spanning"` (default, shared process), `"split"` (separate contexts), `"not_allowed"` | same | Yes | Partial (spanning/not_allowed) | Yes | Medium |
| `key` | Pins the extension ID during development (derived from the public key) | same | Yes | No | No | High |
| `minimum_chrome_version` | Oldest installable browser version | same | Yes | Firefox uses `browser_specific_settings.gecko.strict_min_version` instead | — | High |
| `nacl_modules` | Registered a Native Client module for a MIME type; obsolete (NaCl removed) | MV2 | Removed | No | No | Low |
| `oauth2` | OAuth2 client config (`client_id`, `scopes`) used by `identity.getAuthToken` | same | Yes | Firefox supports `identity.launchWebAuthFlow` but not the `oauth2` key's implicit-Google flow | Yes | Medium |
| `offline_enabled` | Declares offline function (Chrome installer metadata) | same | Yes | No | No | Medium |
| `optional_permissions` | Permissions requestable at runtime via `permissions.request()` | same | Yes | Yes | Yes | High |
| `optional_host_permissions` | Runtime-requestable host permissions | MV3 | Yes | Yes | Yes | High |
| `options_page` | Full-page options UI | same | Yes | Yes | Yes | High |
| `options_ui` | Options page embedded in the management UI (`open_in_tab` flag) | same | Yes (preferred) | Yes | Yes | High |
| `permissions` | Enables extension APIs (non-host) | same | Yes | Yes | Yes | High |
| `requirements` | Declares required technologies, e.g. `{"3D": {"webkit": true}}` (Chrome, largely vestigial) | same | Yes (ignored) | No | No | Low |
| `sandbox` | Declares pages with no extension API access and their own CSP; the sanctioned place for `eval`/remote code inside the package | MV3 object | Yes | Yes | Yes | High |
| `storage` | `managed_schema` — enterprise-policy-managed storage schema for `storage.managed` | same | Yes | Not supported per MDN manifest page (Firefox reads managed storage via `browser_specific_settings.gecko` storage schema since Fx 128; Medium) | — | Medium |
| `tts_engine` | Registers the extension as a text-to-speech engine (receives utterances) | same | Yes | No | No | Medium |
| `update_url` | Self-hosted update manifest URL | same | Yes (not allowed in CWS-packaged extensions) | Firefox uses AMO or `browser_specific_settings.gecko.update_url` | — | Medium |
| `web_accessible_resources` | Extension files reachable from web pages. MV2: string array. MV3: array of `{ resources, matches, extension_ids, use_dynamic_url }` | reshaped in MV3 | Yes (`use_dynamic_url` Chrome-only) | Yes (array form only) | Yes | High |
| `browser_specific_settings` | Firefox-only subtree: `gecko.id`, `strict_min_version`, data-collection disclosure | — | No (fails validation) | Yes | No | High |
| `user_scripts` | Gates the legacy MV2 `userScripts` API | MV2 (Firefox) | No | MV2 only; MV3 needs no key | No | High |
| `side_panel` | Default HTML for `sidePanel` | MV3 (Chrome) | Yes (114+) | Firefox uses `sidebar_action` (different object) | No | Medium |
| `protocol_handlers` | Register web protocol handlers | — | No (uses Web App Manifest) | Yes | — | Medium |
| `omnibox` | Address-bar keyword (Chrome name); Firefox calls the key `omnibox` too, API `omnibox` | same | Yes | Yes | Yes | High |
| `file_browser_handlers`, `file_handlers`, `file_system_provider_capabilities`, `input_components`, `mime_types_handler`, `differential_fingerprint` | ChromeOS-only surface extensions | — | ChromeOS | No | No | Medium |

**Implications:** The mandatory set is trivially small (three keys), so the compatibility surface is entirely in the optional keys — and the two big MV3 reshapes (`background`, `web_accessible_resources`, plus the string→object CSP change and the `permissions`/`host_permissions` split) are exactly where cross-browser manifests fork. A single MV3 manifest that loads in all three browsers is possible today (dual `scripts` + `service_worker` background, array-form WAR is not — Chrome requires the object form, so the manifest must be browser-shaped at build time). Izcal should treat the manifest as a *permissions contract* the user can audit, not a config file: every key is either a capability request (`permissions`, `host_permissions`, `content_scripts`, `externally_connectable`) or UI registration, and we can type-check the contract rather than ignoring unknown keys the way Chrome's `requirements` shows.

## 2. API namespace inventory

Chrome's MV3 reference lists **85 API namespaces** (High, counted from the fetched list). MDN's WebExtensions index documents ~51 namespaces with Firefox/Safari columns. Table: ★ marks the ~15 most privacy/security-significant. "DO" = capability in one phrase.

### 2a. Cross-browser core (Chrome + Firefox, Safari unless noted)

| Namespace | What it can do | MV2/MV3 | Chrome | Firefox | Safari | ★ | Conf. |
|---|---|---|---|---|---|---|---|
| `runtime` | Extension lifecycle, message bus (incl. `onMessageExternal`), manifest, native messaging ports | both | Yes | Yes | Yes | ★ (nativeMessaging: full OS exec via a host binary) | High |
| `tabs` | List all tabs with URLs and titles, open/close/move/reload, inject via `tabs.executeScript` (MV2) | both | Yes | Yes | Yes | ★ (page enumeration: reads URLs of every tab incl. incognito unless excluded) | High |
| `windows` | Create/modify/list browser windows | both | Yes | Yes | Yes | ★ | High |
| `storage` | Key-value + byte stores: `local`, `sync`, `session` (Chrome 102+), `managed` (policy) | both | Yes | Yes | Yes | ★ | High |
| `scripting` | Inject JS/CSS into pages at runtime (`executeScript`, `insertCSS`, `removeCSS`) | MV3 (Chrome 88+) | Yes | Yes (112+) | Yes | ★ (arbitrary code into any granted page) | High |
| `action` / `browserAction` | Toolbar button state, badge, popup | `action`=MV3; `browserAction`=MV2 | Yes | Yes | Yes | — | High |
| `pageAction` | Address-bar button | MV2 mostly | Yes | Yes | Yes | — | High |
| `alarms` | Persist-across-restart timers (min period 1 min; 30 s in Chrome 120+ dev/unpacked? documented: "as short as 30 seconds" matching SW lifecycle) | both | Yes | Yes | Yes | — | High |
| `webNavigation` | Events for every navigation stage, with URLs, transition types, frame IDs | both | Yes | Yes | Yes | ★ (full URL history of all frames, incl. subframes) | High |
| ★ `webRequest` | Observe every HTTP request (headers, bodies with `webRequestBlocking`); MV2 could block/redirect/c modify synchronously — MV3 Chrome: observe + `onAuthRequired`/`onHeadersReceived`-limited blocking only via DNR; **Firefox keeps blocking in MV3** | both | Yes (blocking removed in MV3) | Yes, blocking kept | Observe only; blocking never supported | ★ | High |
| `declarativeNetRequest` | Declarative block/redirect/modify/allow rules; `updateDynamicRules`, `updateSessionRules`, `onRuleMatchedDebug` (unpacked only) | MV3 | Yes (84+) | Yes (113+) | Yes (15.4+) | ★ (block at network layer without listener) | High |
| `cookies` | Read/write/clear any cookie for granted hosts; change notifications | both | Yes | Yes | Yes | ★ (session theft) | High |
| `bookmarks` | Read/create/reorganize all bookmarks | both | Yes | Yes | Yes | ★ | High |
| `history` | Read/delete/search full URL history | both | Yes | Yes | Yes | ★ | High |
| `downloads` | Start/pause/cancel/search/erase all downloads; `onDeterminingFilename` naming control | both | Yes | Yes | Yes | ★ | High |
| `browsingData` | Wipe cache/cookies/history/localStorage etc. | both | Yes | Yes | Yes | — | High |
| `notifications` | OS-level notifications with click handling | both | Yes | Yes | Yes | — | High |
| `contextMenus` / `menus` | Right-click menu items (page, selection, editable, media contexts) | both (Chrome `contextMenus`, Firefox aliases `menus`) | Yes | Yes | Yes | — | High |
| `commands` | Receive keyboard-shortcut events | both | Yes | Yes | Yes | — | High |
| `omnibox` | Address-bar keyword suggestions with XML-ish suggestion objects | both | Yes | Yes | Yes | — | High |
| `i18n` | Locale detection, `getMessage`, accept-languages | both | Yes | Yes | Yes | — | High |
| `permissions` | Request optional permissions at runtime | both | Yes | Yes | Yes | ★ (runtime privilege escalation UX) | High |
| `idle` | System idle/locked state, threshold events | both | Yes | Yes | Yes | — | High |
| `management` | List/install/uninstall/enable other extensions; `getPermissionWarningsById` | both | Yes | Yes | Yes | ★ (enumerate the user's whole extension set) | High |
| `options_ui`-adjacent — n/a | | | | | | | |
| `devtools.*` (`inspectedWindow`, `network`, `panels`, `recorder`, `performance`) | Panels in DevTools; eval in inspected window; HAR | both | Yes | Partial | Partial | ★ (`inspectedWindow.eval` runs in page context) | Medium |
| `proxy` | Firefox: full proxy config incl. `onRequest` PAC-like listener; Chrome: `settings`-based fixed config | both (divergent designs) | Yes (different shape) | Yes | No | ★ (route all traffic) | Medium |
| `privacy` | Toggle autofill, hyperlink auditing, tracking protection (Firefox), network prediction | both (divergent properties) | Yes | Yes | No | ★ | Medium |
| `sessions` | Recently closed tabs/windows, restore; `setTabValue` | both | Yes | Yes | Yes | — | High |
| `search` | Query installed engines, run a search | both | Yes (87+) | Yes | Yes | — | Medium |
| `topSites` | Most-visited page list | both | Yes | Yes | Yes | ★ | High |
| `tts`, `ttsEngine` | Speak text / act as a TTS engine | both (engine: Chrome only per manifest docs) | Yes | Yes (speak) | — | — | Medium |
| `types` | `BrowserSetting` shared type | both | Yes | Yes | — | — | Medium |
| `webAuthenticationProxy` | Let remote-desktop software relay WebAuthn ceremonies | MV3 | Yes (115+) | No | No | ★ | High |
| `nativeMessaging` (accessed via `runtime.connectNative`; manifest needs nothing but the host allowlist) | Launch and talk to a native OS binary | both | Yes | Yes | Yes | ★ (arbitrary code execution channel; the classic sandbox escape hatch) | High |
| `userScripts` | Register third-party user scripts (Greasemonkey-style) into a USER_SCRIPT world | MV3 | Yes (120+; user toggles it on) | Yes (128+; optional permission) | — | ★ | High |
| `tabGroups` | Create/rename/move tab groups | MV3 (89+) | Yes | Yes (partial) | No | — | Medium |
| `readingList` | Read/write the reading list | MV3 | Yes (120+) | No | Yes (17+) | — | Medium |
| `dom` | Extension-only DOM additions (e.g. `openOrClosedShadowRoot`) | both | Yes (88+) | Yes | Yes | — | Medium |
| `extension` | Legacy utilities (`getURL`, `getBackgroundPage`, `isAllowedIncognitoAccess`) | both (deprecated) | Yes | Yes | Yes | — | High |
| `events`, `extensionTypes` | Shared event/enum plumbing | both | Yes | Yes | Yes | — | High |

### 2b. Chrome-only (no Firefox namespace; Safari per Apple list, Medium)

| Namespace | What it can do | MV | ★ | Conf. |
|---|---|---|---|---|
| `offscreen` | Create hidden DOM-capable documents (audio, clipboard, DOM parsing, WebRTC, geolocation...) from a service worker | MV3 (109+) | — | High |
| `sidePanel` | Host persistent UI in Chrome's side panel | MV3 (114+) | — | High |
| `gcm` | Push messages via Firebase Cloud Messaging | both | — | Medium |
| `identity` | OAuth2 tokens (`getAuthToken` Chrome-account flow; `launchWebAuthFlow` generic) | both | ★ (token brokering) | High |
| `debugger` | Full Chrome DevTools Protocol attach — read/write anything in a tab, network, DOM, JS | both | ★★ (most powerful API in the platform) | High |
| `desktopCapture` | Capture screen/window/tab media | both | ★ | High |
| `tabCapture` | Capture tab audio/video streams | both | ★ | High |
| `pageCapture` | Save a tab as MHTML | both | — | High |
| `contentSettings` | Per-site cookie/JS/image/plugin settings | both | ★ | Medium |
| `fontSettings` | Read/change font preferences | both | — | Medium |
| `power` | Block system sleep | both | — | Medium |
| `system.cpu/display/memory/storage` | Hardware metadata; storage attach/detach events | both | — | Medium |
| `accessibilityFeatures` | Read/set a11y settings via ChromeSetting | both | — | Medium |
| `printerProvider`, `printing`, `printingMetrics` | Print ecosystem | both/ChromeOS | — | Medium |
| `vpnProvider` | Implement a VPN client (ChromeOS) | both | ★ | High |
| `certificateProvider`, `platformKeys`, `enterprise.*` | Enterprise TLS keys, device attributes | ChromeOS | ★ | Medium |
| `fileBrowserHandler`, `fileSystemProvider` | ChromeOS file system extensions | ChromeOS | — | Medium |
| `input.ime` | Custom input methods | ChromeOS | — | Medium |
| `loginState`, `instanceID`, `wallpaper`, `audio`, `mimeHandler`, `systemLog`, `documentScan`, `dns` (Chrome Dev-channel `dns` ≠ Firefox `dns`) | OS/device surface | ChromeOS or dev | — | Medium |
| `processes` | Inspect browser processes (Dev channel) | dev | — | Low |
| `declarativeContent` | Act on page state without host permissions (superseded by DNR/action) | MV2 legacy | — | High |

### 2c. Firefox-only

| Namespace | What it can do | ★ | Conf. |
|---|---|---|---|
| `contextualIdentities` | Create/manage container tabs identities | ★ (multi-account isolation) | High |
| `browserSettings` | Change global settings (homepage, cache, New Tab override…) | ★ | High |
| `theme` | Read/update the browser theme | — | High |
| `sidebarAction` | Firefox's sidebar UI (≠ Chrome `sidePanel`) | — | High |
| `find` | Find/highlight text in a page | — | High |
| `dns` | Resolve domain names from the extension | ★ | High |
| `pkcs11` | Enumerate PKCS#11 modules | — | High |
| `captivePortal`, `publicSuffix`, `clipboard` (image write), `menus` (superset), `proxy.onRequest` (the blocking part) | Firefox platform hooks | — | High |
| `urlbar` | Experiment API for address-bar providers (experiment-only, not shippable) | — | Low |

**Implications:** Of the ~85 Chrome namespaces, maybe 30 do real ecosystem work; the rest are ChromeOS/enterprise surface that a new browser can refuse outright. The privacy-critical core is a short list — `tabs`+`webNavigation`+`webRequest`+`cookies`+`history`+`bookmarks`+`downloads` (total-visibility read surface), `scripting`+`debugger`+`userScripts` (write-and-execute), `nativeMessaging`+`proxy`+`vpnProvider` (leave-the-browser escalation), `management`, `identity`, `privacy`, `topSites`. Everything else is UI plumbing. Izcal can collapse the inventory to the load-bearing ~40 namespaces and explicitly version the rest out, at real cost only to niche ChromeOS extensions that can't run anywhere else anyway.

## 3. Background contexts: pages → event pages → service workers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | MV2 default was a persistent background page, kept in memory from load until disable/uninstall; `persistent: false` opted into the "event page" that unloads when idle and restarts on its registered events | default `persistent: true` (MV2) | MDN manifest.json/background | current | High |
| 3.2 | MV3 in Chrome: `persistent: true` is an error; only `service_worker` (or, since 121, an ignored `scripts`) is accepted | — | MDN background key + Chrome manifest ref | current | High |
| 3.3 | Chrome extension service workers terminate after 30 s of inactivity; any incoming event or extension API call resets the timer; a single event/API call taking >5 min forces termination; a `fetch()` response slower than 30 s to arrive kills the worker | 30 s; 5 min; 30 s | developer.chrome.com lifecycle | current | High |
| 3.4 | Keepalive rule history: Chrome 105 `connectNative()` keeps alive; 109 offscreen-document messages reset timers; 110 extension API calls reset timers (previously only running handlers); 114 long-lived messaging keeps alive but merely opening a port does not; 116 active WebSocket activity resets the timer and four user-prompt APIs (`desktopCapture.chooseDesktopMedia`, `identity.launchWebAuthFlow`, `management.uninstall`, `permissions.request`) may exceed the 5-min cap; 118 active debugger sessions keep alive; 120 alarms allowed down to 30 s to match the lifecycle | Chrome 105/109/110/114/116/118/120 | developer.chrome.com lifecycle | current | High |
| 3.5 | Worker globals die with the worker; `Web Storage` is unavailable in service workers — docs direct persistence to `storage`, IndexedDB, CacheStorage | — | developer.chrome.com lifecycle | current | High |
| 3.6 | Wakeup model: dormant workers are revived by any event with a synchronously-registered top-level listener; `install` → `runtime.onInstalled` → `activate` fires in that order at install; `runtime.onStartup` at profile start | — | developer.chrome.com lifecycle | current | High |
| 3.7 | The offscreen document API (Chrome 109+, MV3, `"offscreen"` permission) exists because service workers have no DOM: hidden bundled-HTML pages, one per extension, only `runtime` API inside, can't be focused; `AUDIO_PLAYBACK` documents close after 30 s without audio playing, all other reasons have no lifetime limit; 15 documented reasons (AUDIO_PLAYBACK, DOM_SCRAPING, USER_MEDIA, DISPLAY_MEDIA, WEB_RTC, CLIPBOARD, LOCAL_STORAGE, WORKERS, BATTERY_STATUS, MATCH_MEDIA, GEOLOCATION, DOM_PARSER, BLOBS, IFRAME_SCRIPTING, TESTING); `justification` string is mandatory | 1 document; 30 s audio rule; 15 reasons | developer.chrome.com offscreen | current | High |
| 3.8 | Why audio broke: MV3's no-DOM worker cannot host an `<audio>` element or `getUserMedia` pipeline, so recorders/transcribers either died or had to move to an offscreen document — Chrome's answer was exactly that API (USER_MEDIA/DISPLAY_MEDIA/WEB_RTC/AUDIO_PLAYBACK reasons) plus the Chrome 109 release | Chrome 109 | offscreen page + Medium secondary for the breakage narrative | 2022–23 | High (API facts) / Medium (narrative) |
| 3.9 | Firefox: no extension service workers at all (bug 1573659); MV3 background is an event page; background scripts unload "after a few seconds of inactivity" (MDN does not commit to a number), never until all visible views and message **ports** are closed — but ports are closed *when* the page idles, they cannot hold it open | — | MDN Background_scripts | current | High |
| 3.10 | Firefox's stated reasons (Rob Wu, Mozilla Add-ons blog): service workers "can't fully support various use cases we consider important, especially around DOM-related features and APIs"; blocking `webRequest` "provides a level of power and flexibility that is critical to enabling advanced privacy and content blocking features"; memory pressure makes unload "sometimes inevitable (e.g. on Android)" | — | Mozilla Add-ons blog, 2022-05-18 | 2022 | High |
| 3.11 | Firefox event pages get `runtime.onSuspend` / `onSuspendCanceled`; `getBackgroundPage()` returns `null` when suspended; best practice is synchronous top-level listener registration, `storage.session` for in-memory state, `alarms` instead of timers | — | MDN Background_scripts | current | High |
| 3.12 | Safari: supports both contexts; MV3 service workers since 16.4; historically preferred non-persistent background pages; `preferred_environment` (with Firefox) selects per-browser background shape from one manifest | Safari 16.4 | MDN background key (Safari column) + search snippet | current | Medium |

**Implications:** The 30 s / 5 min numbers are the single most ecosystem-punishing design decision in MV3 — they made "long-running" extensions (recorders, download managers, IDE integrations) architecturally different programs, and drove `alarms` + `storage.session` + offscreen documents into every port. Firefox's countervailing argument (DOM in the background is a feature, not a bug) is well documented and Izcal can adopt it directly: a restartable *document* event page with an honest idle timeout of minutes, not seconds, keeps `getUserMedia`, audio, and long computations first-class without inventing an offscreen bolt-on.

## 4. Content scripts: worlds, timing, matching

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | Content scripts run in an "isolated world": a private JS execution environment; neither page nor other extensions can see their variables, and they can't see page JS variables — but DOM is shared, so both sides read/modify it | — | developer.chrome.com content-scripts | current | High |
| 4.2 | `"world"` in the manifest and in `scripting.executeScript` defaults to `ISOLATED`; `"MAIN"` runs in the page's own context, where the **page's CSP applies** (not the extension's) | — | Chrome content-scripts page | current | High |
| 4.3 | Isolated-world CSP is fixed by the browser: `script-src 'self' 'wasm-unsafe-eval' 'inline-speculation-rules' chrome-extension://<id>/; object-src 'self'` — no `eval`, no remote scripts (unpacked extensions additionally allow localhost) | — | Chrome content-scripts page | current | High |
| 4.4 | A content script's direct API surface is only `dom`, `i18n`, `storage` and a subset of `runtime` (`connect`, `sendMessage`, `getManifest`, `getURL`, `id`, `onConnect`, `onMessage`); everything else goes through message passing to privileged contexts | — | Chrome content-scripts page | current | High |
| 4.5 | `run_at`: `document_start` (after CSS, before any DOM/scripts), `document_end` (DOM complete, before subresource load), `document_idle` (between `document_end` and shortly after `window.onload`) — **default is `document_idle`** | 3 values | Chrome content-scripts page | current | High |
| 4.6 | Frame options: `all_frames` (default false = top frame only), `match_about_blank` (inject into `about:blank` frames whose parent/opener matches), `match_origin_as_fallback` (Chrome; inject into `about:`/`data:`/`blob:`/`filesystem:` frames by initiator origin; path must be `*`; wins over `match_about_blank`) | — | Chrome content-scripts page | current | High |
| 4.7 | Match-pattern grammar: `<scheme>://<host><path>`; scheme wildcard `*` = http+https only (ws/wss in some browsers); host `*`, `*.example.com`, or exact host; wildcards only at host start; ports allowed in Chrome, **not Firefox**; path must start with `/`, `*` anywhere, matched against path+query; fragment ignored; `<all_urls>` covers http/https/ws/wss/ftp/data/file | — | MDN Match_patterns | current | High |
| 4.8 | `include_globs`/`exclude_globs` add Greasemonkey-style `*`/`?` refinement on top of patterns | — | Chrome content-scripts page | current | High |
| 4.9 | Page↔extension communication through a content script is DOM-only (e.g. `window.postMessage` with `event.source === window` checks); `externally_connectable.matches` instead lets *page* scripts call `runtime.sendMessage`/`connect` directly, received on `onMessageExternal`/`onConnectExternal`; since Chrome 106, `chrome.runtime` is undefined on pages with no listening extension (anti-fingerprinting) | Chrome 106 | Chrome content-scripts page + MDN externally_connectable | current | High |
| 4.10 | Firefox `userScripts` (MV3): third-party scripts run in a per-extension `USER_SCRIPT` world — like a content-script environment **but with zero extension APIs**; sub-worlds via `worldId`; per-world CSP and messaging opt-in (`configureWorld({messaging: true})`), received on dedicated `runtime.onUserScriptMessage`; `userScripts` is an *optional-only* permission in Firefox vs install-time + manual user toggle in Chrome; user scripts are cleared on extension update | — | MDN userScripts | current | High |
| 4.11 | MV2 legacy `userScripts` exposed `eval`-style execution for user-script managers; Mozilla's 2022 post: with string eval removed, "user script managers can currently not be ported to MV3" until the new API shipped | — | Mozilla blog 2022-05-18 | 2022 | High |

**Implications:** The isolated-world model is the quiet genius of the platform: DOM is the shared interface, JS is not, and that single decision is why content scripts have never been a top XSS vector the way main-world injection is. Izcal should keep the two-world model, make `world: "MAIN"` an explicit manifest-auditable request, adopt Firefox's stricter userScripts posture (optional permission, no API access in the user-script world, per-world CSP), and treat `externally_connectable` as the fingerprinting/attack surface it is.

## 5. The MV3 remote-code ban

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | Store policy (rev. 2025-05-22): extension "logic of how each extension operates should be self contained"; external resources must contain no logic | — | CWS Program Policies | 2025-05-22 | High |
| 5.2 | Forbidden: `<script src>` pointing outside the package; `eval()` or equivalent on remotely fetched strings; "building an interpreter to run complex commands fetched remotely, even if delivered as data" | — | CWS Program Policies | 2025-05-22 | High |
| 5.3 | Platform enforcement: MV3 CSP for `extension_pages` blocks remote script and `unsafe-eval`; isolated-world content-script CSP likewise; `sandbox` pages are the sanctioned escape — no extension APIs, own CSP, may run arbitrary bundled or remote code | — | Chrome manifest/CSP + content-scripts pages | current | High |
| 5.4 | Allowed: fetching remote *data* and config where all logic is bundled; remote images/fonts; code in isolated contexts (iframes, sandboxed pages) with functionality "discernible"; server-side computation; and — explicitly — remote code execution through the **Debugger API or User Scripts API** "when consistent with the documented API purpose" (the two sanctioned holes in the ban) | — | CWS Program Policies | 2025-05-22 | High |
| 5.5 | WASM: bundling wasm is fine; fetching wasm at runtime is treated as remote hosted code; CSP token `wasm-unsafe-eval` (present in the documented content-script CSP) governs compilation | — | content-scripts page (CSP string) + Medium secondary for the fetch-wasm-is-remote rule | current | High/Medium |
| 5.6 | Firefox: same no-remote-code goal but enforced through CSP; keeps `eval`-free MV3 while **keeping blocking webRequest** and shipping the userScripts API for user-script managers; Mozilla's framing is capability-preserving rather than restriction-first | — | Mozilla blog 2022-05-18 | 2022 | High |
| 5.7 | Safari: MV3 since 16.4 with service workers and DNR; Apple does not document a store-level remote-hosted-code policy in the pages fetched (JS-rendered); WebKit enforces the MV3 CSP, so the platform-level ban applies | — | search snippet + absence | current | Medium |

**Implications:** The ban's two sanctioned holes — debugger and userScripts — mean "no remote code" is really "no remote code *except through APIs we gate hardest*." That is the right shape for Izcal: a structural ban in the default context, plus named, permission-gated, auditable escape hatches, rather than a categorical prohibition that forces every emulator and code-runner into policy-limbo. The "interpreter for remotely fetched commands" clause is worth copying verbatim in spirit: it closes the data-vs-code loophole that `eval` bans otherwise leak through.

## 6. `chrome.*` vs `browser.*`, callbacks vs promises

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Firefox and Safari have "always" used the `browser` namespace with promises; Chromium shipped `chrome` + callbacks; Chrome added promises with MV3 and the `browser` namespace in Chrome 148 (initially excluding extensions with a DevTools page; limitation removed in Chrome 152, tracked as Chrome bug 500769389) | Chrome 148 → 152 | MDN API page | current | High |
| 6.2 | Firefox supports **both** namespaces: `chrome` (callbacks, for Chrome-ported code) and `browser` (promises); the webextension-polyfill is a no-op on Chrome 148+ | — | MDN API page | current | High |
| 6.3 | Bottom line per MDN: "all major browsers now use `browser` namespace with promises for async functions" — the duality is now a legacy-compat matter, not a design fork | — | MDN API page | current | High |
| 6.4 | Safari accepts both `chrome.*` and `browser.*` spellings (per its compatibility-guidance page title/abstract; full table not fetchable) | — | Apple docs (abstract only) | current | Medium |

**Implications:** A new browser in 2026 should ship **only** `browser.*` with promises and treat `chrome.*` as a porting shim — Chrome itself has now converged, so the shim has an end of life. The interesting residue is not namespace but *capability*: `chrome.*` still carries Chrome-only namespaces, so Izcal's `browser.*` will be a strict semantic subset by design.

## 7. Ecosystem load-bearing-ness (documented usage data)

No hard per-API usage statistics were found in the prose sources fetched (Chrome's "extension-platform usage disclosures" and Mozilla's add-on usage stats pages were not located via search within budget). Qualitative anchors only: Mozilla's 2022 post identifies content blockers, user-script managers, and privacy tools as the flagship MV3-affected categories (High); Chrome's offscreen/audio migration guidance and the lifecycle keepalive chronology identify recorders/media extensions as the MV3-broken cohort (High); the store policy's explicit carve-outs (debugger, userScripts) identify emulators, IDE-integrations and script managers as the categories the remote-code ban pressured (High). Everything else in this section: **Low** (inference from API design, not data). One search-priced follow-up should target Chrome's published extension-permission telemetry and addons.mozilla.org's published install counts before the spec freezes any "must support" list.

## Open questions

- Per-API timing/limit behaviours observable only black-box: exact Firefox event-page idle seconds; whether Firefox DNR applies before or after webRequest listeners; Chrome's `use_dynamic_url` semantics across navigation; Safari's actual per-namespace matrix (Apple docs are JS-rendered — scrape the rendered DOM or use the compatibility-checker tool's published output).
- The 30-second alarms floor in Chrome 120: documented on the lifecycle page as a possibility, but the shipped minimum period per channel (packed vs unpacked) needs verification.
- `chrome.declarativeContent` removal status and whether `event_rules` still validates at all in current Chrome.
- Firefox managed-storage schema (`storage.managed`) support: MDN manifest page says the `storage` key is unsupported in Firefox, yet `storage.managed` works via `browser_specific_settings` since ~128 — needs a primary-source read.
- `minimum_chrome_version` semantics in Firefox/Safari manifests (ignored? rejected?).
- Hard usage data: which of the 85 Chrome namespaces actually matter to the published corpus (Chrome Web Store stats, AMO install/usage disclosures, any Chrome extension-platform telemetry blog).
- Whether Chrome's `browser` namespace covers `chrome.` -only namespaces identically (e.g. `browser.offscreen`) — i.e. is it a pure alias or a capability gate.

---

## Summary

- Only three manifest keys are mandatory (`manifest_version`, `name`, `version`); the compatibility burden lives entirely in ~40 optional keys, with MV3's reshapes of `background`, `web_accessible_resources`, `content_security_policy` (string→object) and the `permissions`/`host_permissions` split the real fork points.
- Chrome MV3 lists 85 API namespaces; the load-bearing core is ~30–40. The rest is ChromeOS/enterprise/Dev-channel surface a new browser can refuse without ecosystem damage.
- Most dangerous ~15: `tabs`, `webNavigation`, `webRequest`, `cookies`, `history`, `bookmarks`, `downloads`, `topSites`, `scripting`, `debugger`, `userScripts`, `nativeMessaging`, `proxy`/`vpnProvider`, `management`, `identity` (+ `webAuthenticationProxy` as honorable mention) — read-everything, execute-anything, and leave-the-browser classes.
- Chrome service workers idle at 30 s, hard-cap single events at 5 min, and needed six years of keepalive patches (105→120) plus a whole `offscreen` API (109+) to un-break audio/screen-recording extensions; the offscreen document is a hidden DOM page with 15 enumerated reasons and only `runtime` API access.
- Firefox never shipped extension service workers (bug 1573659): its MV3 background is a DOM-capable event page, with Mozilla's stated reasons being DOM use-cases and memory; it keeps blocking `webRequest` in MV3 as "critical to enabling advanced privacy and content blocking" — the sharpest capability divergence in the ecosystem.
- Second-sharpest divergence: blocking `webRequest` (Chrome MV3: gone; Firefox: kept; Safari: never had it — DNR everywhere, Safari 15.4+).
- Third: `externally_connectable` web-page messaging — Chrome and Safari yes, Firefox no (bug 1319168); Chrome 106 made `chrome.runtime` undefined on pages without listeners to cut fingerprinting.
- Content scripts: isolated world by default (DOM shared, JS not), `world: "MAIN"` where the page's CSP applies, `document_idle` default run-at; direct API surface limited to `storage`, `i18n`, `dom` and a `runtime` subset.
- Match patterns: `<scheme>://<host><path>`, scheme `*` = http+https only, wildcards only leading in host, path matched against path+query, fragment ignored, `<all_urls>` includes data/file; ports in patterns work in Chrome, not Firefox.
- Firefox's MV3 `userScripts` is stricter than Chrome's: optional-only permission, `USER_SCRIPT` world with zero extension APIs, per-world `worldId`/CSP/messaging opt-in via `configureWorld`, dedicated `onUserScriptMessage` receivers; Chrome's is install-time plus a manual user toggle.
- Remote-code ban: policy (rev. 2025-05-22) forbids out-of-package scripts, `eval` of fetched strings, and even "building an interpreter to run complex commands fetched remotely"; sanctions are `sandbox` pages, remote *data*, and explicitly the Debugger and userScripts APIs — the ban is structural with two hard-gated holes, which is the right shape.
- WASM may be bundled (CSP token `wasm-unsafe-eval`); fetching wasm at runtime is treated as remote hosted code.
- `chrome.*` vs `browser.*`: Firefox/Safari promise-native, Chrome converged — `browser` namespace in Chrome 148, full promise coverage by 152; the duality is now legacy-compat, so a new browser ships `browser`-only with a thin `chrome` shim.
- Safari: MV3 + service workers since 16.4, DNR since 15.4, both namespaces accepted; its per-API matrix is not machine-readable from Apple's docs (JS-rendered) and remains the biggest documentation gap in this strand.
- No hard per-API usage statistics found in prose sources; the flagship categories by vendor statements are content blockers, privacy tools, user-script managers (Mozilla) and recorders/media extensions (Chrome's offscreen and lifecycle docs).
