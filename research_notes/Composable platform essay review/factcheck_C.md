# Fact-check: Group C (C1–C21)

Checked 14 October 2026 against primary sources wherever possible. Verdicts: correct / partly correct / wrong / unverifiable / likely fabricated.

## Summary table

| # | Topic | Verdict |
|---|---|---|
| C1 | bfcache / Yahoo! JAPAN News | Partly correct (the 13% figure is not in the case study) |
| C2 | Speculation Rules / WordPress | Correct |
| C3 | Soft Navigations | Correct for Chrome 151 shipping; the CrUX part is unverifiable |
| C4 | Firefox performance roadmap | Correct |
| C5 | Chrome Origin Isolation at 4 GB or more | Correct |
| C6 | 2026 IEEE paper on process isolation | Correct |
| C7 | PQC for DTLS in WebRTC | Correct |
| C8 | Edge 151 security baseline | Correct (minor wording issue) |
| C9 | Firefox 151 fingerprinting | Partly correct (the widget shipped in Firefox 152) |
| C10 | Brave 1.93 GPU masking and other items | Correct, but the items come from different versions |
| C11 | Safari anti-fingerprinting by default | Partly correct (overstated) |
| C12 | Firefox roadmap items | Partly correct (mixes shipped and planned items) |
| C13 | Chrome vertical tabs and split view; Firefox tab groups; PDF translation | Mostly correct (dates and status need tightening) |
| C14 | Baseline additions, March 2026 | Correct |
| C15 | Interop 2026 focus areas | Partly correct (list incomplete, one item misnamed) |
| C16 | View Transitions | Partly correct (Baseline date wrong; Willison quote unverifiable) |
| C17 | Trusted Types is Baseline | Correct |
| C18 | WebGPU details | Partly correct (TRANSIENT_ATTACHMENT version wrong) |
| C19 | Wasm 3.0, WASI, string builtins | Partly correct to wrong (Wasm 3.0 already shipped; WASI 1.0 date not official; "26.2" is a Safari version) |
| C20 | Offpunk 3.0 and Kiwix | Correct |
| C21 | "No mainstream browser serves offline-first browsing" | Partly correct (overstated) |

---

## C1. bfcache. Verdict: partly correct
- **9% mobile revenue: correct.** The web.dev case study reports a 2-week A/B test with ad revenue up 9.0% on mobile and 0.6% on desktop. https://web.dev/case-studies/yahoo-japan-news-bfcache (last updated 2023-07-25)
- **"+13% page views per session": not in the primary source.** The case study reports page views up 2.26% on mobile and 0.65% on desktop, and gives no per-session figure. The 13% figure comes from a Google AdSense publisher email (24 Mar 2026), as reported by PPC Land: https://ppc.land/google-adsense-pushes-bfcache-speculation-rules-and-ai-debugging-to-publishers/. So AdSense is pushing bfcache (correct), but the 13% number is Google marketing copy that disagrees with the published study. The essay should cite +2.26% page views, or attribute the 13% to the AdSense email.
- **Blockers:** Yahoo fixed `unload` handlers (moved to `pagehide`) and `Cache-Control: no-store` (changed to `no-cache`). Open connections (WebSocket, WebRTC, IndexedDB) are also documented blockers. Caveat for "the fix is mechanical": since 2024–25 Chrome allows some `no-store` pages into bfcache (https://developer.chrome.com/docs/web-platform/bfcache-ccns, updated 9 Sep 2025), so `no-store` is no longer an absolute blocker in Chrome. Calling the fix "mechanical" is opinion. Removing `no-store` is a security decision for authenticated pages, not just a mechanical change.

## C2. Speculation Rules. Verdict: correct
- An origin trial for `prerender_until_script` started in Chrome 144. https://developer.chrome.com/blog/prerender-until-script-origin-trial (23 Jan 2026)
- The Performance Team's "Speculative Loading" plugin (https://wordpress.org/plugins/speculation-rules/) was merged into WordPress core in **6.8 (April 2025)**. Core defaults to prefetch with conservative eagerness. The plugin is still available and defaults to prerender with moderate eagerness. The essay should say it is now in core.

## C3. Soft navigations. Verdict: correct (Chrome part); CrUX part unverifiable
- Chrome 151 (stable 28 Jul 2026) ships the `soft-navigation` and `interaction-contentful-paint` PerformanceTimeline entries. They are listed under "Performance and networking", not under origin trials. https://developer.chrome.com/release-notes/151; chromestatus feature 5144837209194496 (Intent to Ship M151). Earlier origin trials ran M139–M144 and M147–M149 (https://developer.chrome.com/blog/final-soft-navigations-origin-trial, 20 Apr 2026). The API is Chromium-only: Firefox has no signal, and Safari has no position.
- I found no CrUX release note announcing "experimental" soft-navigation data (https://developer.chrome.com/docs/crux/release-notes). Treat that part as unverified.

## C4. Firefox performance roadmap. Verdict: correct
https://www.firefox.com/en-US/whatsnext/ (updated 7 Oct 2026):
- "Speed improvements (Desktop)" includes "prioritizing QUIC and IPv6 where appropriate".
- "Resource monitor" (Desktop) shows which tabs use the most resources.
- "Power saving mode" (Android). The 16 Jun 2026 roadmap post describes it as finding the most resource-hungry tabs on phones (https://blog.mozilla.org/en/firefox/firefox-roadmap-152/).

## C5. Chrome Origin Isolation. Verdict: correct
The Chrome 142 Enterprise release notes say: "Origin Isolation will be enabled by default only on devices with at least 4GB of RAM", controlled by the `OriginKeyedProcessesEnabled` policy, for Chrome 142 on ChromeOS, Linux and Windows. The policy was added in Chrome 140. Source: Chrome 142 Enterprise & Education release notes (PDF via support.google.com/chrome/a/answer/7679408); https://chromeenterprise.google/policies/origin-keyed-processes-enabled/. Note that the notes do not list macOS for the 142 rollout.

## C6. 2026 IEEE paper. Verdict: correct
Kilger, Fischer, Staeves, Marchart, Schönberger and Franzen, "State of Browser Process-Isolation: The Same-Site Weakness", IEEE S&P 2026, pp. 1464–1482, DOI 10.1109/SP63933.2026.00096. The abstract says "attackers can exfiltrate sensitive data across all major browsers". The mechanism is that same-site subdomains share processes (a site-isolation design limit, not a bypass), with demonstrations on Blink, Gecko and WebKit. https://www.sec.in.tum.de/i20/publications/state-of-browser-process-isolation-the-same-site-weakness. The essay should say this exploits same-site process sharing, not that site isolation is "broken".

## C7. PQC with DTLS in WebRTC. Verdict: correct
The Chrome 142 Enterprise release notes cover "PostQuantum cryptography for DTLS in WebRTC". The `WebRtcPostQuantumKeyAgreement` policy "is planned to be removed by Chrome 152". Applies to Chrome 142 on all platforms. Same PDF as C5.

## C8. Edge 151 security baseline. Verdict: correct (minor wording issue)
Microsoft's "Security baseline for Microsoft Edge version 151" (Microsoft Security Baselines blog, late Aug 2026; RSS date 31 Aug 2026) contains "six new recommendations". They are: Enable Process Isolation, Enable renderer in app container, Enable the network service sandbox, browser process code integrity guard, Application Bound Encryption, and Enhance security state (Balanced). Automatic HTTPS is listed as an extra setting to consider. https://techcommunity.microsoft.com/t5/microsoft-security-baselines/security-baseline-for-microsoft-edge-version-151/ba-p/4549607. Wording issue: these are six new baseline *recommendations* covering existing policies, not "six new policies". Edge 151 itself added 4 new policy settings.

## C9. Firefox 151 fingerprinting. Verdict: partly correct
- Firefox 151 (19 May 2026) strengthened fingerprinting protection in Standard ETP. It reduces uniquely identifiable users "by about 14% on average, and about 49% on macOS". https://www.firefox.com/en-US/firefox/151.0/releasenotes/
- **The Blocked Tracker Widget shipped in Firefox 152, not 151.** https://blog.mozilla.org/en/firefox/firefox-roadmap-152/ (16 Jun 2026)

## C10. Brave. Verdict: correct (the items come from different releases)
- Brave 1.93 (rollout from 13 Aug 2026) does three things, on by default on desktop and Android but not iOS: genericizes the WebGL vendor and renderer strings, empties the WebGPU adapter info, and randomizes the WebGL extension list. https://brave.com/privacy-updates/38-webgl-webgpu-fingerprinting-protections
- Email Aliases launched on desktop in 1.94 (27 Aug 2026), not in 1.93. https://brave.com/privacy-updates/39-email-aliases/
- The Shred button reached Android in 1.89. https://brave.com/privacy-updates/37-shred-button-android/
- The adblock engine rework cut "memory consumption by 75%" for the adblock engine (about 45 MB), not for the whole browser. It shipped in v1.85, announced 5 Jan 2026. https://brave.com/privacy-updates/36-adblock-memory-reduction. The Register measured the whole browser at about 36% lower (https://www.theregister.com/2026/01/06/brave_refurbishes_rust_adblocking_engine/).

## C11. Safari anti-fingerprinting. Verdict: partly correct (overstated)
Since Safari 26 (Sept 2025), the script-level Advanced Fingerprinting Protection (noise and API restrictions for known fingerprinting scripts) is on in regular browsing as well as Private Browsing. The separate user setting "Advanced Tracking and Fingerprinting Protection" (network blocking and full protections) still defaults to Private Browsing only. So "on by default across all modes" holds for the base layer only. Sources: https://webkit.org/blog/17333/webkit-features-in-safari-26-0/; https://ppc.land/safari-26-tracking-changes-to-impact-marketing-measurement/. I did not find Apple's own wording saying "all modes".

## C12. Firefox roadmap. Verdict: partly correct (out of date and mixes shipped with planned)
- The 16 Jun 2026 roadmap listed as upcoming: a free built-in VPN on mobile, and native Multi-Account Containers. https://blog.mozilla.org/en/firefox/firefox-roadmap-152/
- The Blocked Tracker Widget had already **shipped** in Firefox 152. It was not a roadmap item.
- Ad blocking on iOS (EasyList with WebKit content blocker, opt-in) **shipped** around late Aug / 1 Sep 2026. https://gigazine.net/gsc_news/en/20260902-firefox-for-ios-ad-blocker
- The current roadmap (https://www.firefox.com/en-US/whatsnext/, 7 Oct 2026) lists Breach alerts (desktop), VPN on iOS (the VPN is already on desktop and Android), and a Privacy dashboard (iOS). Containers no longer appear, which fits with A1's claim that they shipped in Firefox 153.

## C13. Tabs and PDF translation. Verdict: mostly correct
- Native vertical tabs in Chrome reached stable around 7–8 Apr 2026. The menu item is "Show Tabs Vertically", found by right-clicking the tab strip or window. https://chromeunboxed.com/google-is-officially-rolling-out-native-vertical-tabs-to-the-stable-version-of-chrome/. Split view is older: it rolled out in late 2025 (Chrome 142–143), via the tab context menu item "Add tab to new split view". https://gadgetbond.com/google-chrome-split-tabs-feature-v143-stable-update/. So "split view in 2026" is slightly off.
- Firefox tab groups on Android shipped in Firefox 152 (June 2026), with iOS "later this year". This is correct (Mozilla blog, 16 Jun 2026).
- PDF translation through Reading Mode is in Chrome Canary behind the `enable-translate-pdf` flag (Canary 155, Sept 2026). Reports say it is unreliable. https://windowsreport.com/chrome-translate-pdf-reading-mode/. The claim is correct as a Canary experiment only.

## C14. Baseline March 2026. Verdict: correct
The newly available features listed in https://web.dev/blog/baseline-digest-mar-2026 match the claim exactly: math font family, Iterator.concat(), readable byte streams, Reporting API, WebTransport, `text-indent: each-line`, and `text-indent: hanging`.

## C15. Interop 2026. Verdict: partly correct
The official list has 20 focus areas (WebKit announcement, 12 Feb 2026: https://webkit.org/blog/17818/announcing-interop-2026/): Anchor Positioning, Advanced attr(), Container Style Queries, contrast-color(), CSS Zoom, Custom Highlights, Dialog and popover additions, **Fetch Uploads and Ranges**, getAllRecords() for IndexedDB, JSPI for Wasm, Media pseudo-classes, Navigation API, Scoped Custom Element Registries, Scroll-driven Animations, Scroll Snap, shape(), View Transitions, Web Compat, WebRTC, and WebTransport.
- The essay **omits Anchor Positioning and CSS Zoom**.
- It calls the Fetch area "Fetch streaming request bodies". The official name is "Fetch Uploads and Ranges", which covers streaming uploads and Range requests.
- The 4 investigation areas are accessibility testing, JPEG XL, mobile testing, and WebVTT.

## C16. View Transitions. Verdict: partly correct
- **Baseline date wrong.** Same-document View Transitions became Baseline Newly available on **14 Oct 2025**, when Firefox 144 shipped. That is not January 2026. https://api.webstatus.dev/v1/features/view-transitions. Cross-document view transitions are still not Baseline, because Firefox lacks them.
- **Chrome 147 element-scoped view transitions: correct.** `element.startViewTransition()` is available in Chrome 147 stable (7 Apr 2026). https://developer.chrome.com/blog/element-scoped-view-transitions
- **The Simon Willison quote is unverifiable.** I found no 2026 post where he says he is "most excited about" cross-document View Transitions. The closest is a link post from 10 Mar 2025 (https://simonwillison.net/2025/Mar/10/building-websites-with-llms/). Treat the quote as likely misattributed until a source is given.

## C17. Trusted Types. Verdict: correct
Baseline Newly available since **24 Feb 2026**, when Firefox 148 shipped. Support: Chrome and Edge 83 (May 2020), Safari 26 (15 Sep 2025), Firefox 148 (24 Feb 2026). It is expected to become Widely available around Aug 2028. https://api.webstatus.dev/v1/features/trusted-types

## C18. WebGPU. Verdict: partly correct
- **Linux, Intel Gen12+ in Chrome 144: correct.** https://developer.chrome.com/blog/new-in-webgpu-144 (7 Jan 2026)
- **Compatibility Mode on Android OpenGL ES 3.1: correct.** It shipped in Chrome 146. https://developer.chrome.com/blog/new-in-webgpu-146 (25 Feb 2026)
- **WGSL `subgroup_id` extension in Chrome 144: correct.**
- **`TRANSIENT_ATTACHMENT`: wrong version.** The usage flag was introduced in **Chrome 146**. Chrome 149–150 only added stricter validation for it. https://developer.chrome.com/blog/new-in-webgpu-149-150 (17 Jun 2026)
- Related check on A5: Chrome 144's writeBuffer and writeTexture gains are "up to 2X". Subgroup uniformity and experimental synchronous buffer mapping in workers are listed under Chrome 145. Both are consistent with the claim.

## C19. WebAssembly and WASI. Verdict: partly correct to wrong
- **"Wasm 3.0 is the next milestone": wrong.** Wasm 3.0 was completed and published on 17 Sep 2025. It includes GC, Memory64, multiple memories, exception handling, tail calls, typed references, relaxed SIMD, and JS string builtins, and most browsers already ship it. https://webassembly.org/news/2025-09-17-wasm-3.0/
- **"WASI 1.0 expected late 2026 or early 2027": not official.** The official roadmap gives no 1.0 date (https://wasi.dev/roadmap). The window appears only in secondary blogs (byteiota).
- **WASI P3 async: correct, and it has now shipped.** WASI 0.3.0 was released on 11 Jun 2026 and 0.3.1 on 11 Aug 2026. It adds native async (async functions, streams, futures) to the Component Model.
- **"JS String Builtins arrived in Wasm in version 26.2": misleading.** 26.2 is a *Safari* version. Chrome 130 (Oct 2024) and Firefox 134 (Jan 2025) shipped the feature first. Safari 26.6 added the streaming-compile options. https://webkit.org/blog/18178/webkit-features-for-safari-26-6/; https://api.webstatus.dev/v1/features/wasm-string-builtins
- JSPI is an Interop 2026 focus area (see C15). It is shipped in Chromium.

## C20. Offpunk and Kiwix. Verdict: correct
- Offpunk 3.0 was released in February 2026. It integrates "unmerdify" (by Vincent Jousse) for decluttering web pages, and supports Web, Gemini, Gopher, and RSS/Atom. https://noise.getoto.net/2026/02/09/offpunk-3-0-released/ (syndicating LWN)
- Kiwix JS ships as browser extensions, a PWA, Electron and NWJS apps, and a UWP app. https://github.com/kiwix/kiwix-js

## C21. "No mainstream browser serves offline-first browsing." Verdict: partly correct (overstated)
Mainstream browsers do offer save-for-offline features:
- **Safari:** Reading List has "Save Offline" and "Automatically Save Offline" (iOS: Settings > Safari; macOS: Advanced settings).
- **Chrome on Android:** Download page, "Download Page Later" from the offline screen, and offline badges (since 2016–17). https://blog.google/products/chrome/read-web-pages-offline-chrome-android/
- **Chrome desktop:** "Save page as > Webpage, Single File" saves MHTML.
- **Firefox:** Pocket shut down on 8 Jul 2025 (announced 22 May 2025, exports until 8 Oct 2025). The current roadmap lists "Offline mode" for Android (https://www.firefox.com/en-US/whatsnext/).
- **Web Bundles:** Never shipped as a user-facing save format.

The essay's point holds only if it means offline-*first* browsing (syncing, a searchable offline corpus, multiple protocols, as in Offpunk or Kiwix). Saving pages for offline reading does exist in mainstream browsers, so the wording needs to make that distinction.
