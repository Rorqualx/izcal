# Browser UI architecture and process model: evidence notes

Compiled 21 October 2026 for the Izcal shell specification (phase 2c).

Scope: how shipped browsers build and place their own UI, what that costs in memory, startup and responsiveness, which process owns OS windows and input, how accessibility and IMEs force code into the window owner, and what can be recorded and replayed. Sources are prose documentation, vendor posts and public measurement write-ups read in text on 21 October 2026; no source code, headers, commits, diffs or code reviews were consulted (M-1 part 3).

Confidence: **High** = primary prose read in full text; **Medium** = vendor claim, abstract/snippet, or a known page not read this session; **Low** = secondary only. "(single source)" marks a number reported by only one source. "[Izcal]" marks an internal anchor, not external evidence. Undated documentation pages are marked n.d.; all were retrieved on the compiled date.

## 1. What shipped browser chrome is made of

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 1 | Chrome's desktop browser chrome is native per-platform UI built with Views; the same document calls WebUI (browser UI implemented in the HTML content area) "another UI toolkit". | 2 toolkits: Views + WebUI | [Chromium: How to write cross-platform UI](https://chromium.googlesource.com/chromium/src/+/main/docs/cross_platform_ui.md) | n.d. | High |
| 2 | Chrome ships Settings, History and Downloads as WebUI: HTML, CSS and JavaScript in a renderer. | 3 example surfaces | [Chromium: WebUI explainer](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/webui_explainer.md) | n.d. | High |
| 3 | The C++ that serves `chrome:` URLs and performs browser actions lives in the browser process; the WebUI renderer is sandboxed and has no file access. | — | [Chromium: WebUI explainer](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/webui_explainer.md) | n.d. | High |
| 4 | WebUI bindings are special powers: load `chrome:` URLs, let the browser execute arbitrary JavaScript in that renderer, use `chrome.send()`, and ignore content settings for images and JavaScript. | 4 powers | [Chromium: WebUI explainer](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/webui_explainer.md) | n.d. | High |
| 5 | WebUI pages cannot embed http/https resources or issue http/https fetches; the default Content Security Policy disallows frames. | — | [Chromium: WebUI explainer, security](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/webui_explainer.md#security-considerations) | n.d. | High |
| 6 | WebUI is privileged enough that taking control of one "can sometimes be sufficient to escape Chrome's sandbox". | — | [Chromium: WebUI explainer, security](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/webui_explainer.md#security-considerations) | n.d. | High |
| 7 | `chrome://` is historically built as "an extension to the browser process"; all `chrome://` pages can use camera and microphone without asking permission. | — | [Chromium: chrome-untrusted:// FAQ](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/chrome_untrusted.md) | n.d. | High |
| 8 | `chrome-untrusted://` serves bundled resources into an ordinary sandboxed renderer with no default dangerous capabilities; a compromised page there is "no worse than an ordinary web page". | — | [Chromium: chrome-untrusted:// FAQ](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/chrome_untrusted.md) | n.d. | High |
| 9 | The stated reason is the rule of 2: a `chrome://` page is an extension of the browser process, the renderer is C++, and the page runs in a privileged context, so untrusted content there combines three unsafe properties. | 3 properties | [Chromium: chrome-untrusted:// FAQ](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/chrome_untrusted.md) | n.d. | High |
| 10 | WebUI accessibility runs through the normal Blink-to-content-to-platform pipeline, with ARIA added largely by JavaScript. | — | [Chromium: Accessibility overview, WebUI](https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md) | n.d. | High |
| 11 | WebKit2 splits web content into a web process and keeps application UI and application logic in the UI process; the web process is isolated, for responsiveness, robustness and security. | 2 processes | [WebKit: WebKit2 wiki](https://trac.webkit.org/wiki/WebKit2) | n.d. | High |
| 12 | Electron has one main process (Node.js) that creates and manages application windows; each window's page runs in its own renderer; renderers have no direct OS access and reach native functions through preload scripts and IPC. | 1 main + N renderers | [Electron: Process model](https://www.electronjs.org/docs/latest/tutorial/process-model) | n.d. | High |
| 13 | Edge moved features to WebUI 2.0, a markup-first architecture that shrinks bundles and cuts JavaScript on the initialization path; Browser Essentials became 42% faster (76% on machines without an SSD or with less than 8 GB RAM) and Favorites 40% faster in Edge 124. | 42%, 76%, 40% (vendor, single source) | [Microsoft Edge blog: An even faster Microsoft Edge](https://blogs.windows.com/msedgedev/2024/05/28/an-even-faster-microsoft-edge/) | 2024-05-28 | Medium |
| 14 | Firefox's browser chrome is XUL, an XML UI language for windows, toolbars and menus; the browser window, popups and panels still require XUL, and Mozilla says new code should use HTML wherever possible. | — | [Firefox Source Docs: XUL and HTML](https://firefox-source-docs.mozilla.org/browser/components/storybook/docs/README.xul-and-html.stories.html) | n.d. | High |
| 15 | Some chrome widgets cannot be plain HTML: XUL elements are supplied to an HTML page "without the page having any control over them", panels can draw outside the window's bounds, and OS-specific styling tends to stay XUL. | — | [Firefox Source Docs: XUL and HTML](https://firefox-source-docs.mozilla.org/browser/components/storybook/docs/README.xul-and-html.stories.html) | n.d. | High |
| 16 | Proton (Firefox 89) redesigned and modernised the core experience; it also introduced non-native form controls for web pages, claiming page-load performance gains. | — | [Firefox 89 release notes](https://www.mozilla.org/en-US/firefox/89.0/releasenotes/) | 2021-06-01 | High |
| 17 | Firefox 57 (Quantum) was claimed to be "over twice as fast as Firefox from 6 months ago". | about 2x (vendor, single source) | [Firefox 57 release notes](https://www.mozilla.org/en-US/firefox/57.0/releasenotes/) | 2017-11-14 | Medium |
| 18 | browser.html was a research project to build an experimental Servo browser in HTML, with the Graphene runtime; it is retired and the repository was archived on 27 July 2018. | retired; archived 2018-07-27 | [browserhtml project README and archive notice (prose; no code read)](https://github.com/browserhtml/browserhtml) | 2018-07-27 | High |

**Implications**

- Every shipped HTML chrome (WebUI, Edge, Electron, WebKit2's UI process) keeps window ownership, privileged services and the OS event loop outside the page's renderer (rows 3, 11-13). Izcal's "shell is a page" (UP4) matches the industry pattern only while the broker keeps that split.
- Chrome treats `chrome://` as part of the browser process and still had to invent `chrome-untrusted://` for untrusted content (rows 7-9). Izcal should not create a trusted page scheme at all: the shell renderer gets explicit capabilities, and anything that renders untrusted bytes is a different, ordinary renderer.
- Edge's 42/76/40% (row 13) shows HTML chrome speed is dominated by how much JavaScript runs before first paint and how large the bundles are. It is an engineering constraint on the shell, not proof that HTML chrome is inherently slow.
- Firefox keeps XUL where the OS demands window-like behaviour (rows 14-15). Izcal's equivalents belong in broker/native services, not in shell CSS.
- browser.html (row 18) is the cautionary precedent: an HTML browser UI research project that was retired, not a shipped success.

## 2. Process placement and privilege

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 19 | Chromium's sandbox is process-granular: anything to be sandboxed must live in its own process; the browser process is the privileged broker that sets policy, spawns targets, performs policy-allowed actions for them and outlives them. | broker + N targets | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 20 | The sandbox threat model assumes sandboxed code is malicious from the first external input, or before the main loop is entered. | — | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 21 | The browser process is unsandboxed on all platforms. | 1 process | [Chromium: Unsandboxed processes by platform (M128)](https://chromium.googlesource.com/chromium/src/+/main/docs/security/process-sandboxes-by-platform.md) | 2024 (M128) | High |
| 22 | The network service is unsandboxed on Android, Windows and Linux; the GPU process is unsandboxed on Android and Linux/X11. | 2 service types | [Chromium: Unsandboxed processes by platform (M128)](https://chromium.googlesource.com/chromium/src/+/main/docs/security/process-sandboxes-by-platform.md) | 2024 (M128) | High |
| 23 | On X11 the GPU process opens its X11 connection before the sandbox engages and is therefore treated as unsandboxed. | — | [Chromium: X11 threat model](https://chromium.googlesource.com/chromium/src/+/main/docs/security/x11-threat-model.md) | n.d. | High |
| 24 | X11 does not isolate clients: any client can inject input via XTest, read input including passwords, capture the screen, spoof UI with override-redirect windows, and change XSETTINGS for every client. | 5 capabilities | [Chromium: X11 threat model](https://chromium.googlesource.com/chromium/src/+/main/docs/security/x11-threat-model.md) | n.d. | High |
| 25 | Wayland compositors isolate clients, so Wayland issues are evaluated as normal security issues. | — | [Chromium: X11 threat model](https://chromium.googlesource.com/chromium/src/+/main/docs/security/x11-threat-model.md) | n.d. | High |
| 26 | The Windows sandbox combines a restricted token, a job object, an alternate desktop, integrity levels and process mitigation policies; renderers additionally run with win32k.sys system-call lockdown. | 5 mechanisms | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 27 | "Disable extension points" blocks AppInit DLLs, Winsock layered service providers, global (non-thread-targeted) window hooks and legacy IMEs. | 4 vectors | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 28 | CIG (`ProcessSignaturePolicy`) blocks unsigned code and third-party DLL injection; it is enabled pre-startup for renderers, eliminating the launch-time injection gap. | — | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 29 | ACG (`ProcessDynamicCodePolicy`) makes code pages immutable; it is on by default for sandboxed no-JIT processes and can be enabled for the browser process. | — | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 30 | The broker re-issues policy-allowed Win32 calls for sandboxed targets; the interception mechanism exists for compatibility, not security, and policy is re-checked in the target only as a speed optimisation. | — | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 31 | Native accessibility APIs are synchronous, while renderers are sandboxed and cannot implement OS APIs; Chromium caches the AX tree in the browser process and implements the APIs from that cache; blocking the browser main thread is "strictly disallowed", and the cache may lag the renderer by a fraction of a second. | lag below 1 s | [Chromium: Accessibility overview](https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md) | n.d. | High |
| 32 | Chromium implements more than 150 accessibility API methods on Windows alone. | over 150 methods | [Chromium: Accessibility overview](https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md) | n.d. | High |
| 33 | Views accessibility and web-content accessibility are merged in the browser process by BrowserAccessibilityManager into one platform tree. | 2 trees into 1 | [Chromium: Accessibility overview, Views](https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md) | n.d. | High |
| 34 | Chrome's accessibility features are off by default and enabled automatically on demand when a client is detected. | — | [Chromium: Accessibility overview](https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md) | n.d. | High |
| 35 | A compromised renderer must not influence the omnibox URL, the secure/not-secure chip, content-settings icons or the origin shown in dialogs; the browser validates origin and URL at commit and kills the process on mismatch. | — | [Chromium: Compromised renderers](https://chromium.googlesource.com/chromium/src/+/main/docs/security/compromised-renderers.md) | n.d. | High |
| 36 | Forging a user gesture that affects trusted browser UI is listed as work in progress and "not protected today". | — | [Chromium: Compromised renderers](https://chromium.googlesource.com/chromium/src/+/main/docs/security/compromised-renderers.md) | n.d. | High |
| 37 | In the browser process cc runs single-threaded because the browser main thread is "cheap and light"; renderers use multi-threaded cc; input is forwarded from the browser process, and the compositor thread handles scroll and pinch without Blink; Chrome runs separate Blink, display-compositor, browser-UI and GPU schedulers. | 1 vs N cc threads; 4 schedulers | [Chromium: How cc works](https://chromium.googlesource.com/chromium/src/+/main/docs/how_cc_works.md) | n.d. | High |

**Implications**

- The broker is the only component that can hold OS authority (rows 19-22). The shell renderer must reach browser state only through capability endpoints; there is no safe "trusted renderer" middle ground.
- On Linux, a window owner that supports X11 holds an X11 connection and is therefore unsandboxed (rows 23-25). The trusted computing base differs by display server; Wayland should be the primary path and X11 a documented weaker mode.
- Windows mitigations already forbid the shell renderer from hosting hooks and legacy IMEs (rows 26-29). Any feature that needs them must move out of the renderer, not weaken the renderer's mitigations.
- Accessibility must be a broker responsibility: the synchronous OS provider sits with the window owner, and the shell renderer's AX tree is asynchronous data (rows 31-34). A separate prompt renderer adds a second AX source that the broker must merge.
- The browser main thread must never block on a renderer (rows 31, 37); shell IPC must be asynchronous, and compositor scrolling and animation must work without broker main-thread involvement.
- A compromised renderer is assumed able to lie about URL, origin and gestures (rows 35-36), so security UI values must be computed and drawn outside the renderer's reach.

## 3. Measured costs: memory, startup, responsiveness

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 38 | Isolating all sites costs about 10-13% more memory on desktop Chrome 67 with many tabs, and 3-5% on Android Chrome 77. | 10-13% desktop; 3-5% Android (vendor measurement, single source) | [Chromium: Site Isolation](https://www.chromium.org/Home/chromium-security/site-isolation/) | n.d. | Medium |
| 39 | Separate processes buy crash and hang containment and parallel responsiveness at a memory cost; desktop applies a soft process limit and reuses same-site processes, and the New Tab Page is process-per-site. | — | [Chromium: Process model and site isolation](https://chromium.googlesource.com/chromium/src/+/main/docs/process_model_and_site_isolation.md) | n.d. | High |
| 40 | Chrome measures startup as `Startup.FirstWebContents.NonEmptyPaint2` and notes that the metric regresses after every update because the share of cold starts rises. | 1 metric | [Chrome Speed Launch Metrics survey](https://docs.google.com/document/d/1Ww487ZskJ-xBmJGwPO-XPz_QcJvw-kSNffm0nPhVpj8/edit) | 2019-04 | High |
| 41 | Omnibox latency is measured as `Omnibox.CharTypedToRepaintLatency`, with no lab coverage and acknowledged timestamp problems. | 1 metric | [Chrome Speed Launch Metrics survey](https://docs.google.com/document/d/1Ww487ZskJ-xBmJGwPO-XPz_QcJvw-kSNffm0nPhVpj8/edit) | 2019-04 | High |
| 42 | Tab switching is measured from the input event (`Browser.Tabs.InputEventToSelectionTime`) and as total switch duration; the start time comes from the browser process and the presentation time from the GPU process, split by whether a saved frame exists. | 2+ metrics | [Chrome Speed Launch Metrics survey](https://docs.google.com/document/d/1Ww487ZskJ-xBmJGwPO-XPz_QcJvw-kSNffm0nPhVpj8/edit) | 2019-04 | High |
| 43 | Session restore is measured to first paint of the foreground tab and separately to all-tabs-loaded. | 2 metrics | [Chrome Speed Launch Metrics survey](https://docs.google.com/document/d/1Ww487ZskJ-xBmJGwPO-XPz_QcJvw-kSNffm0nPhVpj8/edit) | 2019-04 | High |
| 44 | Browser-UI responsiveness is measured as the number of 100 ms intervals with jank per 30 s of user activity, over native events and browser UI and IO thread tasks. | 100 ms bins; 30 s windows | [Chrome Speed Launch Metrics survey](https://docs.google.com/document/d/1Ww487ZskJ-xBmJGwPO-XPz_QcJvw-kSNffm0nPhVpj8/edit) | 2019-04 | High |
| 45 | Chrome reports browser, renderer and GPU private memory footprints separately. | 3 process classes | [Chrome Speed Launch Metrics survey](https://docs.google.com/document/d/1Ww487ZskJ-xBmJGwPO-XPz_QcJvw-kSNffm0nPhVpj8/edit) | 2019-04 | High |
| 46 | Chrome optimises WebUI by bundling JavaScript modules, minifying with terser and gzip-compressing HTML, JS, CSS and SVG resources. | — | [Chromium: Optimizing Web UIs](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/optimizing_web_uis.md) | n.d. | High |
| 47 | Web "good" INP is 200 ms or less and "poor" is more than 500 ms; PerformanceObserver entries below 104 ms are not reported by default, and the configurable minimum is 16 ms. | 200 ms; 500 ms; 104 ms default; 16 ms floor | [web.dev: Interaction to Next Paint](https://web.dev/articles/inp) | 2025-09-02 | High |
| 48 | Chrome's responsiveness target is 60 fps, while cc has a high-latency mode that draws without waiting for a slow main thread, trading latency for throughput. | 16.7 ms per frame | [Chromium: Speed domains](https://chromium.googlesource.com/chromium/src/+/main/docs/speed/speed_domains.md) and [How cc works](https://chromium.googlesource.com/chromium/src/+/main/docs/how_cc_works.md) | n.d. | High |
| 49 | Shared-memory accounting is inherently ambiguous: Chrome attributes each anonymous shared region to exactly one process, and Linux PSS depends on how many processes map a region. | — | [Chromium: Key concepts in Chrome memory](https://chromium.googlesource.com/chromium/src/+/main/docs/memory/key_concepts.md) | n.d. | High |
| 50 | [Izcal] FP-1 budgets the broker at 20 MiB, the shell renderer at 14 MiB, the compositor GPU service at 12 MiB, the shell content GPU at 2 MiB, network at 8, storage at 4, the crash handler at 2 and the Linux spawner at 1, for a total of 64 MiB at idle (60.5); third-party IME modules in the broker are an attributed line outside the sum. | at most 64 MiB total; 60.5 idle | [Izcal resource footprint, FP-1](../../reports/Izcal%20resource%20footprint.md) | 2026-10-21 | Medium |

**Implications**

- Process separation is not free: 10-13% on desktop (row 38) is the measured price of one process per site. Izcal's 64 MiB total (row 50) leaves little room for extra process kinds, so every new process needs its own budget line plus a spawn and exit rule.
- The metrics that matter span processes (rows 40-44): startup ends at a content first paint, tab switching ends at GPU presentation, and an address-bar keystroke may repaint in the shell renderer. Instrumentation must join timestamps across broker, shell renderer, compositor and GPU, or it will miss the user-visible event.
- The 16 ms p95 target (UI-7) is 12.5 times tighter than the 200 ms "good" INP threshold (row 47); the shell cannot rely on web-page-grade responsiveness, and the 16 ms observation floor means lab instrumentation, not field APIs, has to measure it.
- Bundle size and JavaScript-before-first-paint are the levers Edge used (row 13). The shell needs a first-paint JavaScript budget and should ship markup-first.
- Memory accounting must be specified per process and per shared region (rows 45, 49), or the shell renderer's 14 MiB line will absorb or hide tiles that the broker and compositor actually own.

## 4. Security of browser UI

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 51 | Browser UI faces two main attacks: spoofing and clickjacking or keyjacking. | 2 classes | [Chromium: Security considerations for browser UI](https://chromium.googlesource.com/chromium/src/+/main/docs/security/security-considerations-for-browser-ui.md) | n.d. | High |
| 52 | The "line of death" is the traditional separation of trusted chrome from content, but research shows users do not understand it; familiarity and visual cues matter more than position. | — | [Chromium: Security considerations for browser UI](https://chromium.googlesource.com/chromium/src/+/main/docs/security/security-considerations-for-browser-ui.md) | n.d. | High |
| 53 | Do not mix trustworthy with untrustworthy content, and do not show website-provided strings in permission prompts; attackers use them to trick users into grants. | — | [Chromium: Security considerations for browser UI](https://chromium.googlesource.com/chromium/src/+/main/docs/security/security-considerations-for-browser-ui.md) | n.d. | High |
| 54 | Occlusion can be worse than spoofing: covering the origin in a prompt changes which site the user thinks they are granting, and occluding the fullscreen bubble leaves no trustworthy pixels. | — | [Chromium: Security considerations for browser UI](https://chromium.googlesource.com/chromium/src/+/main/docs/security/security-considerations-for-browser-ui.md) | n.d. | High |
| 55 | Defenses: no default-selected accept button, multiple clicks or gestures in different places, and a delay before the call to action activates; Chrome uses about 3 s for large security surfaces and 500 ms for dialogs, and ignores UI events within 500 ms of showing sensitive UI. | 3 s; 500 ms | [Chromium: Security considerations for browser UI](https://chromium.googlesource.com/chromium/src/+/main/docs/security/security-considerations-for-browser-ui.md) | n.d. | High |
| 56 | Prefer negative security indicators and existing UI patterns; route choices to Settings rather than inventing new prompt types. | — | [Chromium: Security considerations for browser UI](https://chromium.googlesource.com/chromium/src/+/main/docs/security/security-considerations-for-browser-ui.md) | n.d. | High |
| 57 | Many browser UI surfaces are clickjackable, and "not all browser UI surfaces are hardened against clickjacking/keyjacking"; new ones should be. | — | [Chromium: Security considerations for browser UI](https://chromium.googlesource.com/chromium/src/+/main/docs/security/security-considerations-for-browser-ui.md) | n.d. | High |
| 58 | Fullscreen gives a site control of all pixels; users cannot reliably tell they are in fullscreen, so Chrome prioritises trusted UI in sensitive fullscreen scenarios. | — | [Chromium: Fullscreen security FAQ](https://chromium.googlesource.com/chromium/src/+/main/docs/security/fullscreen.md) | n.d. | High |
| 59 | A site can frontload deceptive framing around a genuine prompt; the proposed Page Embedded Permission Control embeds a user-agent-controlled element whose text, size, contrast and visibility the site cannot change. | — | [Chromium: You probably don't need a permission prompt](https://chromium.googlesource.com/chromium/src/+/main/docs/security/no-prompts-please.md) | n.d. | High |
| 60 | Permission mechanisms must clearly disclose the origin of the request. | — | [Chromium: Permissions for powerful web platform features](https://chromium.googlesource.com/chromium/src/+/main/docs/security/permissions-for-powerful-web-platform-features.md) | n.d. | High |
| 61 | Window Controls Overlay is cited as a precedent for direct controls placed in trusted browser UI. | — | [Chromium: You probably don't need a permission prompt](https://chromium.googlesource.com/chromium/src/+/main/docs/security/no-prompts-please.md) | n.d. | High |
| 62 | `chrome-untrusted://` guidance: do not grant capabilities a normal renderer would not get; treat everything the page sends as untrusted; never send secrets; re-create interfaces per page load to avoid lifetime bugs. | 4 rules | [Chromium: chrome-untrusted:// FAQ](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/chrome_untrusted.md) | n.d. | High |
| 63 | If a WebUI embeds web content, the frame tree and ancestor origins are visible to it, it shares the WebUI's StoragePartition and profile, and `postMessage` payloads are untrusted. | — | [Chromium: WebUI explainer, security](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/webui_explainer.md#security-considerations) | n.d. | High |

**Implications**

- Prompt UI is a security surface first: the no-default-accept rule and the activation delay (row 55) must be enforced by the prompt renderer itself, not by page conventions.
- Occlusion (row 54) is the argument for drawing prompts above both page and shell content in a layer no renderer can cover. A prompt drawn inside the same renderer as shell content fails by construction.
- Prompts must not display site-controlled text (row 53); the origin string is broker-derived and validated (row 35) and drawn by the trusted prompt renderer (rows 59-60).
- Chrome's own WebUI guidance (rows 62-63) treats even a "trusted" page as a capability holder with untrusted inputs. Izcal should treat the shell renderer the same way and never let it become `chrome://`.
- Fullscreen and Window Controls Overlay (rows 58, 61) are the two places where trusted UI must coexist with content that controls the viewport; both need a broker-owned layer, not CSS in the page.

## 5. OS windows, input and compositing

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 64 | `SetParent` across processes forces a reset of the child window's process DPI awareness on Windows 8.1 through Windows 10 1703 and later. | — | [Microsoft: SetParent](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent) | n.d. | High |
| 65 | Cocoa: `NSWindow` and its descendants are generally thread-unsafe; `NSView` and its descendants must be created, destroyed, resized and moved only on the main thread; the main thread handles events. | — | [Apple: Thread safety summary](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html) | 2014-07-15 | High |
| 66 | Windows accessibility clients obtain an object by sending `WM_GETOBJECT` to the window procedure of the server application, which returns the IAccessible or UI Automation provider; the OS marshals it back to the client. | — | [Microsoft: WM_GETOBJECT](https://learn.microsoft.com/en-us/windows/win32/winauto/wm-getobject) | 2025-07-14 | High |
| 67 | On Wayland the compositor is the display server: it owns the scenegraph, decides which surface receives input, and clients render into buffers they hand to the compositor. | — | [Wayland: Architecture](https://wayland.freedesktop.org/architecture.html) | n.d. | High |
| 68 | Windows app notifications are OS-drawn popups outside the app's window; clicking one can launch the app or trigger a background action. | — | [Windows App SDK: App notifications](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/) | n.d. | High |
| 69 | A Windows desktop is a security boundary: processes with windows on the same desktop are effectively in one security context because window messages are not security-checked; the sandbox puts targets on a separate, non-interactive desktop that costs about 4 MB of RAM. | 3 desktops; about 4 MB (single source) | [Chromium: Sandbox design, alternate desktop](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 70 | [Izcal] Prompt surfaces bypass the page compositor; the broker places shell tiles in its own OS layer (CALayer, DirectComposition, Wayland subsurface, SurfaceControl) and derives the unoccluded rect from its own layer tree. | — | [Izcal graphics and rendering, GR-D28](../../reports/Izcal%20graphics%20and%20rendering.md) | 2026-10-21 | Medium |

**Implications**

- Window ownership forces an OS event loop on the owning process: the macOS main thread must run the application and touch views there (row 65), a visible Windows window must live on the interactive desktop (row 69), and Wayland input arrives at the surface owner (row 67). The broker cannot delegate window ownership to the shell renderer.
- Cross-process embedding is not a shortcut: `SetParent` resets DPI awareness (row 64), and windows on the same desktop share an unchecked message context (row 69). A UI-host process on the user's desktop is reachable by other desktop apps.
- The accessibility provider must be the window owner (row 66), so the broker answers assistive technology synchronously and the shell renderer only supplies data.
- Notifications and similar shell-integrated UI are OS-drawn but activated through the app (row 68); they need broker-held identity and routing, not a shell page.
- Izcal's plan to put prompt surfaces outside the page compositor (row 70) matches the occlusion constraint in section 4; the layer tree must stay broker-owned so page content cannot reorder it.

## 6. Windows: why a UI-host process exists

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 71 | TSF text services are in-process COM servers; a text service never interacts with an application directly, and all communication passes through the TSF manager. | in-process COM | [Microsoft: TSF architecture](https://learn.microsoft.com/en-us/windows/win32/tsf/architecture) | n.d. | High |
| 72 | TSF text-service providers are expected to ship digitally signed binaries. | — | [Microsoft: Text Services Framework](https://learn.microsoft.com/en-us/windows/win32/tsf/text-services-framework) | n.d. | High |
| 73 | Chromium's disable-extension-points mitigation blocks legacy IMEs and global window hooks; CIG blocks unsigned DLLs but not signed ones; ACG blocks new executable code and is only enabled for no-JIT processes. | 3 mitigations | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 74 | Windows integrity levels apply User Interface Privilege Isolation to window messages between processes on the same desktop; a low-integrity process can send messages only through `ChangeWindowMessageFilter`; at M75 Chrome ran the browser and network utility at Medium integrity, the GPU at Low and renderers at Untrusted. | 5 integrity levels; M75 snapshot | [Chromium: Sandbox design, integrity levels](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 75 | [Izcal] Third-party IME modules loaded by the OS into the broker are accounted as "attributed OS code" on their own line, outside FP-1 and outside the broker's 20 MiB budget. | outside budgets | [Izcal resource footprint](../../reports/Izcal%20resource%20footprint.md) | 2026-10-21 | Medium |

**Implications**

- The documented Windows model forces a split: an in-process TSF text service loads into the process that hosts text input, so that process cannot be a CIG plus extension-point-disable plus ACG renderer (rows 71-73). Izcal's Windows UI host is a consequence of the OS model, not a design preference.
- Code integrity raises the bar to signed binaries but does not exclude signed third-party IME and accessibility DLLs (rows 72-73). The UI host must assume those DLLs are in its address space and keep no secrets there.
- That host must own the HWNDs (row 66) and sit on the interactive desktop (row 69), so it is reachable by other desktop processes; UIPI limits but does not eliminate that surface (row 74).
- Keep the UI host minimal: no web-content parsing, no profile database, no network stack, no JIT, and a documented, audited module list (rows 71-75).
- macOS and Linux have no equivalent forced DLL host; the broker can be smaller there. The Windows UI host should be an optional, separately budgeted process rather than the default shape of the broker.

## 7. Crash, recovery, safe mode and session restore

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 76 | Separate processes limit the impact of a renderer crash or hang and let other content keep working. | — | [Chromium: Process model and site isolation](https://chromium.googlesource.com/chromium/src/+/main/docs/process_model_and_site_isolation.md) | n.d. | High |
| 77 | The broker is expected to outlive all targets it spawned; a browser-process crash therefore takes every window with it. | — | [Chromium: Sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md) | n.d. | High |
| 78 | Firefox's session restore persists windows, tabs and tab groups with per-tab history, scroll position and form data, and SessionStartup decides at startup whether to restore the previous session. | — | [Firefox Source Docs: Session restore](https://firefox-source-docs.mozilla.org/browser/components/sessionstore/docs/index.html) | n.d. | High |
| 79 | Chrome measures session restore to first paint of the visible tab and to all tabs loaded, with the foreground metric flagged as simplistic and lacking lab coverage. | 2 metrics | [Chrome Speed Launch Metrics survey](https://docs.google.com/document/d/1Ww487ZskJ-xBmJGwPO-XPz_QcJvw-kSNffm0nPhVpj8/edit) | 2019-04 | High |
| 80 | Firefox's Troubleshoot Mode starts the browser without add-ons and customisations and with hardware acceleration disabled, to isolate a broken profile or add-on. | — | [Mozilla Support: Troubleshoot Mode](https://support.mozilla.org/en-US/kb/troubleshoot-mode-firefox) | n.d. | Medium |
| 81 | [Izcal] A native recovery window opens after two failed HTML-shell starts; it can reset the profile, start without hardware acceleration and send the crash report, nothing else. | 2 failed starts | [Izcal: HGBA shell UI design, R3](../../reports/HGBA%20shell%20UI%20design.md) | 2026-10-21 | Medium |

**Implications**

- Broker crash is total (row 77), so broker code must stay small, avoid parsing untrusted input and be restartable independently of the shell. The recovery path cannot be an HTML page in the same renderer (row 81).
- Renderer containment (row 76) is what makes a shell-renderer crash survivable, but it must not leave the user with a blank window. The broker should own a minimal fallback surface.
- Session state must be written crash-safely and restored without the HTML shell (rows 78-79). Firefox's state set (windows, tabs, history, scroll, form data) is the user expectation to match.
- Safe or troubleshoot mode (row 80) is a separate, native, minimal UI and the natural place to disable the shell, add-ons and third-party IME or accessibility modules when diagnosing startup failure.
- Recovery must not depend on the profile being readable. A corrupt profile is a startup path, not an error dialog shown after the shell loads.

## 8. Deterministic UI, testing and record/replay

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 82 | Chrome WebUI tests are Mocha suites loaded into the real `chrome://` page by a browser-test base, with code coverage collected when enabled. | — | [Chromium: Testing WebUI pages](https://chromium.googlesource.com/chromium/src/+/main/docs/webui/testing_webui.md) | n.d. | High |
| 83 | Firefox browser-chrome mochitests run in the context of the desktop browser window and are the way UI behaviour is tested. | — | [Firefox Source Docs: Browser chrome mochitests](https://firefox-source-docs.mozilla.org/testing/browser-chrome/index.html) | n.d. | High |
| 84 | Playwright's trace viewer explores recorded traces after a scripted run, with before/after snapshots per action and the ability to move back and forward in time. | — | [Playwright: Trace viewer](https://playwright.dev/docs/trace-viewer) | n.d. | High |
| 85 | Chrome DevTools Recorder records, replays and measures user flows; replay waits for elements to be visible or clickable and can be slowed, breakpointed and stepped. | — | [Chrome DevTools: Recorder](https://developer.chrome.com/docs/devtools/recorder) | n.d. | High |
| 86 | rr records an execution and replays it deterministically as many times as needed, with low overhead on mostly-single-threaded workloads. | — | [rr: Record and replay](https://rr-project.org/) | n.d. | High |

**Implications**

- The shipped way to test browser UI deterministically is a scripted browser harness driving the real UI (rows 82-83), plus action-level traces with DOM snapshots (rows 84-85). No shipped browser exposes a production whole-UI deterministic record and replay.
- Action-level replay (rows 84-85) is enough for regression tests but not for pixel- or timing-level debugging; rr (row 86) is whole-process but Linux- and C/C++-centric and not a browser feature.
- Izcal's deterministic substrate should make the shell page itself replayable, and the prompt and recovery layers should be testable through the same scripted harness. Record and replay must not depend on native widget state the harness cannot see.
- The prompt renderer and recovery window need their own scripted tests, or the deterministic story stops at the shell boundary.

## 9. Additional shell-owned surfaces

| # | finding | numbers | source link | date | confidence |
|---|---|---|---|---|---|
| 87 | The Document Picture-in-Picture API opens an always-on-top window that can be populated with arbitrary HTML and that carries minimal browser-generated controls; it is a separate window from the tab. | 1 extra always-on-top window | [MDN: Document Picture-in-Picture API](https://developer.mozilla.org/en-US/docs/Web/API/Document_Picture-in-Picture_API) | n.d. | High |
| 88 | Windows notification-area icons are registered against an application window handle (or a GUID) and the application handles clicks and its own shortcut menus; icons must be high-DPI aware. | — | [Microsoft: Notifications and the notification area](https://learn.microsoft.com/en-us/windows/win32/shell/notification-area) | n.d. | High |
| 89 | [Izcal] Default-browser handling uses OS-sanctioned APIs only and never re-prompts after the user says no. | — | [Izcal: HGBA shell UI design, R19](../../reports/HGBA%20shell%20UI%20design.md) | 2026-10-21 | Medium |

**Implications**

- Picture-in-Picture (row 87) is a second always-on-top OS window whose content is page-controlled; the broker owns the window and its minimal controls, so page content can never become the window chrome.
- Tray icons and notification-area menus are app-owned OS surfaces tied to an application window (row 88); they cannot be a shell page and must survive a shell-renderer crash.
- Default-browser registration is an OS identity operation, not a UI feature; it belongs to the broker and must be idempotent (row 89).

## Design constraints the shell process model must meet

1. Exactly one process owns every OS window, including Picture-in-Picture windows, and runs the OS event loop; on macOS that process's main thread runs the application loop and touches views, and the shell renderer receives no window object (rows 37, 64-67, 87).
2. The shell renderer holds no HWND, X11 connection or win32k access; it draws into surfaces owned and placed by the broker (rows 21-26, 70).
3. Every browser-issued prompt is drawn by a renderer that shares no process with page content or with the shell page, in a broker-owned layer above both, with no default accept and an arming delay of at least 500 ms (rows 54-55, 59-61, 70).
4. The window-owning process answers `WM_GETOBJECT` and every synchronous OS accessibility call from its own cached tree; the shell renderer supplies accessibility data asynchronously (rows 31-34, 66).
5. On Windows, IME and assistive-technology hosting is confined to a dedicated UI-host process with no web parsing, no profile secrets, no network stack and no JIT; only signed modules load, and its loaded-module list is an audited artifact (rows 27-29, 71-75).
6. On Linux, any process holding an X11 connection is treated as unsandboxed; only the broker and, where unavoidable, the compositor GPU service may be X11 clients, no renderer may be, and Wayland is the primary display path (rows 21-25).
7. Shell first paint and address-bar keystroke-to-paint are instrumented across broker, shell renderer, compositor and GPU with p95 budgets, and cold and warm starts are reported separately (rows 40-44, 47).
8. No new process kind ships without its own FP-1 line, spawn policy and exit policy; every shared region is charged to exactly one owner (rows 38, 45, 49, 50).
9. A compromised shell renderer cannot set or alter the URL, origin, security indicators or prompt text; the broker computes those values and the prompt renderer draws them (rows 35-36, 51, 60).
10. Recovery is native and independent of the HTML shell and of profile readability: after two failed starts a recovery window can reset the profile, disable acceleration and send the crash report (rows 77, 80-81).
11. The shell, prompt and recovery UIs are all driven by scripted browser harnesses and the deterministic substrate; no security-relevant UI state lives only in native widgets the harness cannot see (rows 82-86).
12. Page content, shell UI and prompts never share a process, and no renderer that displays untrusted bytes receives capability endpoints beyond a normal renderer's (rows 6-9, 62-63).
