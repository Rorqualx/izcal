# Web performance and browser-compatibility pain points: claim verification and engine-level mitigations (as of October 2026)

Notes:
- Compatibility data comes from MDN browser-compat-data (BCD) v8.1.5, timestamp 2026-10-08, downloaded from https://unpkg.com/@mdn/browser-compat-data/data.json. Per-feature MDN pages render this same data.
- Older data is labeled with its date.

## Q1. Is the "53% abandon after 3 seconds" claim accurate, and the "three-quarters encountered a slow site" claim?

### Takeaway
The 53% figure is real, but it is a 2016 Google modeled finding, and the original says visits are "likely to be abandoned". It is not a measured share of users. Verdict: **partly correct** (accurately quoted, but 10 years old and often over-stated). The "roughly three-quarters encountered a slow site" claim is **unverifiable**: it traces to a Kissmetrics/third-party "73%" stat that I could not find on any Google primary page.

### Cited Findings
- Google's "The Need for Mobile Speed" study (September 2016, published via the DoubleClick/Google Ad Manager blog) says 53% of mobile site visits are "likely to be abandoned" if pages take longer than 3 seconds to load. — [Google Ad Manager blog](https://blog.google/products/admanager/the-need-for-mobile-speed/); [DoubleClick Publishers blog, Sept 2016](https://doubleclick-publishers.googleblog.com/2016/09/)
- Methodology: the study drew on more than 10,000 mobile web domains, using Google Analytics and DoubleClick data. The same study reported an average mobile load time of 19 s on 3G. It projected (modeled, not measured) that sites loading in 5 s could earn up to 2x the mobile ad revenue of 19 s sites. — [AMP blog, 2016-09-08](https://blog.amp.dev/2016/09/08/new-study-speedmatters-for-mobile-user-engagement-and-revenue/); [MediaPost, 2016](https://www.mediapost.com/publications/article/284398/)
- Wording drift: Google's post says "likely to be abandoned". Secondary outlets (e.g., the ARF) state it as a firm abandonment rate, and many later citations say "53% of users" rather than "53% of visits". — [Google Ad Manager blog](https://blog.google/products/admanager/the-need-for-mobile-speed/); [thearf.org](https://thearf.org/?p=341315)
- Think with Google still republishes "53% of visits are abandoned if a mobile site takes longer than 3 seconds to load." Its related 2017/2018 benchmark found it took nearly 7 s for above-the-fold content to appear on 70% of pages analyzed. — [Think with Google mobile load-time stats](https://thinkwithgoogle.com/consumer-insights/consumer-trends/mobile-site-load-time-statistics/); [Think with Google benchmarks PDF](https://thinkwithgoogle.com/_qs/documents/57/mobile-page-speed-new-industry-benchmarks.pdf)
- "73% of mobile internet users say they've encountered a website that was too slow to load" appears only on third-party pages that credit Kissmetrics. It does not appear on any Think with Google page I found. — [eJewishPhilanthropy (citing Kissmetrics)](https://ejewishphilanthropy.com/how-loading-time-affects-your-bottom-line/); [SlideShare vendor deck](https://www.slideshare.net/slideshow/zecloudcom/38669492)
- Current contrast (2026): Contentsquare's 2026 benchmark says slow page loads fell 20% YoY and now affect 10.9% of sessions (99B sessions, 6,500 sites). — [Contentsquare 2026 Digital Experience Benchmark – Frustration, 2026-04-01](https://contentsquare.com/guides/digital-experience-benchmark/frustration/)

### Inferences
- The 2016 numbers reflect 3G-era conditions, with a 19 s average load. Field data from 2025 (Q2 below: 62% of mobile origins have good LCP ≤2.5 s) suggests typical loads are far faster now. The 53% figure should be presented as historical and modeled, not as a current behavioral measurement.
- A better current replacement is the CrUX/Web Almanac 2025 pass rates, or the Contentsquare 2026 figure that 10.9% of sessions have slow page loads (vendor data).

### Gaps
- I could not locate the original Kissmetrics survey (date, sample) for the "73%/three-quarters" figure. Treat it as unverifiable.
- Google never published the 2016 study's detailed abandonment methodology (e.g., the definition of "abandon" and the bounce versus timeout split) in the sources found.

## Q2. "Technically fast" pages that pass Core Web Vitals yet feel unresponsive: INP, CrUX pass rates, long tasks/LoAF

### Takeaway
**Correct in substance.** INP replaced FID as a Core Web Vital on 12 March 2024. Mobile field INP pass rates are high (77% good in July 2025). However, lab Total Blocking Time on mobile rose 58% YoY to a median of 1,916 ms, so main-thread congestion is growing even as field metrics improve. Only 48% of mobile origins pass all three CWVs.

### Cited Findings
- INP became a Core Web Vital and replaced FID on 12 March 2024. — [web.dev: INP is officially a Core Web Vital](https://web.dev/blog/inp-cwv-march-12)
- Web Almanac 2025 Performance chapter (CrUX and HTTP Archive data from July 2025):
  - Good CWV overall: **48% mobile, 56% desktop**, up from 44%/55% in 2024 (and 36% mobile in 2023).
  - **LCP** good/NI/poor: mobile 62/25/13; desktop 74/18/7.
  - **INP** good/NI/poor: mobile 77/21/3 (74% in 2024); desktop 97/2/<1.
  - **CLS** good/NI/poor: mobile 81/10/9; desktop 72/17/10.
  - Source: [HTTP Archive Web Almanac 2025 – Performance](https://almanac.httparchive.org/en/2025/performance)
- Lab **TBT** median is 1,916 ms on mobile, up 58% from 1,209 ms in 2024; p90 is 7,555 ms. Desktop median is 92 ms. The chapter notes TBT rose even as field INP improved. — [Web Almanac 2025 – Performance](https://almanac.httparchive.org/en/2025/performance)
- Other 2025 Almanac findings: non-composited animations appear on 40% of mobile pages, and images without explicit dimensions on 62% of mobile pages. Both contribute to jank and CLS. — [Web Almanac 2025 – Performance](https://almanac.httparchive.org/en/2025/performance)
- **Long Animation Frames (LoAF)** API (`PerformanceLongAnimationFrameTiming`): Chrome/Edge 123+. `paintTime`/`presentationTime` were added in Chrome 145. Not supported in Firefox or Safari as of BCD 2026-10-08. — [MDN BCD data.json](https://unpkg.com/@mdn/browser-compat-data/data.json); [MDN PerformanceLongAnimationFrameTiming](https://developer.mozilla.org/en-US/docs/Web/API/PerformanceLongAnimationFrameTiming)
- The Almanac 2025 performance chapter has no LoAF or long-task field statistics. — [Web Almanac 2025 – Performance](https://almanac.httparchive.org/en/2025/performance)
- Secondary sources report a slightly different CrUX mobile pass figure (~49.7%), depending on dataset and month. — [corewebvitals.io](https://corewebvitals.io/core-web-vitals) (secondary; not independently verified)

### Inferences
- The "passes CWV but feels slow" gap is plausible for three reasons. INP is a p75 per-origin metric measured on the worst interactions. CrUX covers only Chrome users. Rising TBT shows that a heavy JS load still exists, likely masked by faster devices or by interactions that don't land during long tasks.
- Because CrUX is Chrome-only and LoAF is Chromium-only, a non-Chromium engine cannot be judged on these metrics directly.

### Gaps
- I did not retrieve CrUX monthly data for 2026 (e.g., an August/September 2026 CrUX dashboard). The latest authoritative per-metric figures found are from July 2025.
- I found no published statistic on the share of CWV-passing pages that still have long tasks or LoAFs over 200 ms.

## Q3. "Travel sites saw +20.6% API error rates and +20% JavaScript error rates"

### Takeaway
**Correct, with caveats.** The source is Contentsquare's 2026 Digital Experience Benchmark (published 1 April 2026). The figures are year-over-year changes in error *rates* (relative increases, not percentage points). The comparison years are not named. Travel was the only industry where overall frustration rose.

### Cited Findings
- Travel and hospitality: API error rate **+20.6% YoY**, JavaScript error rate **+20% YoY**, and overall frustration **+7.6% YoY** (the only industry with rising frustration). — [Contentsquare 2026 Benchmark – Frustration, 2026-04-01](https://contentsquare.com/guides/digital-experience-benchmark/frustration/)
- Cross-industry figures: API errors **+16% YoY**; JS error rates **+0.3%** overall, driven by financial services (+18.1%) and travel (+20%); overall frustration down 4.3%; frustration still affects 35.2% of sessions. — [Contentsquare 2026 Benchmark – Frustration](https://contentsquare.com/guides/digital-experience-benchmark/frustration/); [Contentsquare 2026 Benchmarks landing](https://contentsquare.com/guides/digital-experience-benchmark/)
- Dataset: "over 99 billion sessions on 6,500 websites across 9 key industries." The baseline is described only as "compared to last year", and the page gives no detailed methodology. — [Contentsquare 2026 Benchmark – Frustration](https://contentsquare.com/guides/digital-experience-benchmark/frustration/)
- A secondary source says JS errors affect 14.5% of travel sessions, and that the 2026 report's methodology changed, so figures are not directly comparable with prior editions. — [atlasperk.com (secondary)](https://atlasperk.com/guides/seo-for-travel/technical-seo/)

### Inferences
- These are relative changes from an unstated baseline, so "+20%" could mean, for example, a rate moving from 10% to 12% of sessions. Do not present them as "20% of sessions had errors."
- The source is vendor data from Contentsquare's customer base, not a representative sample of the web.

### Gaps
- No absolute baseline API or JS error rates for travel were found on the primary page.
- No independent source (e.g., Noibu or Quantum Metric) corroborates the travel figures. Uptrends' State of API Reliability 2025 has no travel breakdown. — [Uptrends 2025](https://www.uptrends.com/state-of-api-reliability-2025)

## Q4. Compatibility claims

### Takeaway
- Promise.withResolvers versions: **correct**.
- Module service workers first shipping in Firefox 147: **correct**. Firefox was the last major engine to ship them, in January 2026.
- Edge "deprecated Compatibility View": **partly correct/misattributed**. Compatibility View was an IE feature, deprecated in IE11. Edge never had it; Edge uses IE mode.
- Firefox for Android lacking passkey conditional UI "through 125": **unverifiable/conflicting**, and stale as of October 2026.

### Cited Findings
- **Promise.withResolvers**: Chrome 119, Edge 119, Chrome Android 119, Firefox 121 (desktop and Android), Safari 17.4 (macOS and iOS). Baseline 2024 (since March 2024). — [MDN BCD v8.1.5](https://unpkg.com/@mdn/browser-compat-data/data.json); [MDN Promise.withResolvers](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/withResolvers)
  - W3Schools lists Chrome 117 / Firefox 119, which conflicts with MDN. Trust MDN BCD. — [W3Schools](https://www.w3schools.com/jsref/jsref_promise_withresolvers.asp)
- **Module service workers** (`ServiceWorker` ECMAScript modules):
  - Support: Chrome/Edge 91, Safari 15, Firefox 147 (desktop and Android). — [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
  - The Firefox 147 release notes (13 January 2026) list ES module support for service workers, "bringing Firefox in line with other major browsers." — [Firefox 147 release notes](https://www.firefox.com/firefox/147.0/releasenotes/)
  - For comparison, dedicated and shared module workers shipped in Firefox 114. — [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- **Edge Compatibility View**:
  - Microsoft's IE11 docs say the Compatibility View button was removed in IE11 and the feature is "deprecated". — [Microsoft Learn: Missing the Compatibility View button (IE11)](https://learn.microsoft.com/de-de/previous-versions/windows/internet-explorer/ie-it-pro/internet-explorer-11/ie11-deploy-guide/missing-the-compatibility-view-button)
  - A Microsoft Q&A answer (2022, community) says "there's no compatibility view in Edge"; the path is IE mode. — [Microsoft Q&A](https://learn.microsoft.com/en-us/answers/a/713360)
  - IE mode in Edge is reportedly supported through at least 2029 (Message Center MC1473151), per a secondary report. — [Windows Report](https://windowsreport.com/microsoft-is-keeping-ie-mode-alive-inside-edge-until-at-least-2029/)
- **Firefox for Android passkey conditional UI**:
  - MDN BCD lists `PublicKeyCredential.isConditionalMediationAvailable()` as supported in Firefox **and Firefox for Android from 119**, with no Android caveat note. — [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
  - The newer `Credential.isConditionalMediationAvailable()` (Chrome 148) is not supported in Firefox. — [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
  - A vendor blog (MojoAuth) says Firefox for Android does not support passkey autofill (conditional UI) "through version 125". Another MojoAuth page says conditional UI arrived in Gecko 122 without separating Android. — [MojoAuth: Known broken passkey combinations](https://mojoauth.com/blog/known-broken-passkey-combinations-and-workarounds.md); [MojoAuth: Conditional UI support](https://mojoauth.com/blog/conditional-ui-browser-support-passkey-autofill)
  - A 2023 Mozilla Connect thread suggested that Android would get an MVP passkey implementation in Firefox 120, with conditional UI desktop-only. This is a user's reading, not an official statement. — [Mozilla Connect](https://connect.mozilla.org/t5/ideas/support-webauthn-passkeys/idc-p/39458)
  - A 1Password community thread reports that Firefox for Android did not use the Android CredentialManager provider until Nightly 128 testing in 2024. — [1Password community](https://www.1password.community/1password-at-home-31/firefox-passkeys-1password-android-14-3469)

### Inferences
- The Firefox Android claim may describe real-world UX (no autofill passkey chip) even though the detection API reports availability. That would mean feature detection can return true without a usable UI, which is a classic compat pain point. Firefox 125 dates from April 2024, so the claim says nothing about current Firefox for Android (~v150 in October 2026).
- The accurate wording for Edge: "Internet Explorer's Compatibility View is deprecated; Edge offers IE mode (Enterprise Mode Site List) instead."

### Gaps
- I found no Mozilla release note or Bugzilla resolution that states when conditional-mediation autofill UI shipped on Firefox for Android.
- I did not directly read the Microsoft MC1473151 advisory.

## Q5. Engine/browser-level mitigations and their measured effects

### Takeaway
Engines already ship several responsiveness and navigation mitigations: scheduler.yield/postTask, isInputPending, bfcache, Speculation Rules prerender/prefetch, heavy-ad intervention, and background-timer throttling. Measured gains are substantial but mostly come from Chromium data and case studies. Prerender cut LCP by about 43% at Ray-Ban and up to 17.9% at Monrif. Back/forward makes up 20% of mobile navigations. Intensive throttling cut CPU by up to 5x. Cross-engine support is uneven: Safari lacks scheduler and prerender, and Firefox lacks speculation rules.

### Cited Findings
**Prioritized Task Scheduling / input prioritization**
- Support for `scheduler.postTask`: Chrome/Edge 94, Firefox 142. `scheduler.yield()`: Chrome/Edge 129, Firefox 142. Safari supports neither (as of 2026-10-08). — [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json); [MDN Scheduler.yield](https://developer.mozilla.org/en-US/docs/Web/API/Scheduler/yield)
- `navigator.scheduling.isInputPending()`: Chrome/Edge 87 only. — [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)

**Back/forward cache**
- In Chrome usage data, "1 in 10 navigations on desktop and 1 in 5 on mobile are either back or forward." — [web.dev bfcache (pub. 2023-05-25, updated 2026)](https://web.dev/articles/bfcache)
- Chrome and Firefox make pages ineligible for bfcache on desktop if they add `unload` listeners. Chrome is deprecating `unload`. Work to allow `Cache-Control: no-store` pages into bfcache was described as "underway" with no ship date. — [web.dev bfcache](https://web.dev/articles/bfcache)
- Remaining blockers (July 2025):
  - Unload handlers on 20% of top-1k mobile pages and 10% of all mobile sites.
  - `Cache-Control: no-store` on 23% of sites (up from 21%).
  - Source: [Web Almanac 2025 – Performance](https://almanac.httparchive.org/en/2025/performance)
- The `NotRestoredReasons` API (for debugging bfcache misses) is supported in Chrome 125+ only. — [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)

**Speculation Rules / prerender**
- Support:
  - Prerender: Chrome/Edge 105+ (Android 103+). Not in Firefox (bug 1969838) or Safari.
  - Prefetch: Chrome 110+. Safari 26.2 behind the "SpeculationRules prefetch" preference only. Not in Firefox (bug 1969396).
  - Source: [MDN BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- Adoption (July 2025): speculation rules appear on about 15% of top-1M home pages and 24–25% of top-10M. The Almanac figure description says 35% mobile for all sites, which conflicts with the text. — [Web Almanac 2025 – Performance](https://almanac.httparchive.org/en/2025/performance)
- **Ray-Ban** (web.dev, 2025-01-28), prerendered PDPs, CrUX URL-level data:
  - Mobile LCP fell from 4.69 s to 2.66 s (about 43%); desktop from 3.03 s to 1.74 s (about 43%).
  - Conversion rate: +101% mobile, +156% desktop. Exit rate: about -13%.
  - Prerender rate: 29% mobile, 50% desktop.
  - Source: [web.dev Ray-Ban case study](https://web.dev/case-studies/rayban-speculation-rules)
- **Monrif** (web.dev, December 2025): prerendering top article paths gave up to 17.9% LCP improvement and up to +8.9% engagement in some segments. bfcache support was added later. — [web.dev Monrif case study](https://web.dev/case-studies/monrif-cwv)
- **Google Search** A/B tests (2025-02-12): prefetching the first two results cut LCP for clicked sites by 67 ms on Chrome Android and 58.6 ms on desktop. Hover-triggered prefetch on later results cut desktop LCP by 9.5 ms. There was no mobile gain for later results, so that was not enabled. — [Chrome for Developers: How Google Search uses speculation rules](https://developer.chrome.com/blog/search-speculation-rules)
- **Akamai**: p75 LCP was 537 ms for prerendered pages versus 714 ms for standard pages (-177 ms). — [Akamai blog, Aug 2024](https://www.akamai.com/blog/developers/2024/aug/faster-page-loads-with-the-speculation-rules-api)
- Measurement caveat: raw LCP for prerendered pages must subtract `activationStart`. One write-up saw 6,244 ms raw versus 103.5 ms adjusted. — [jangwook.net (blog)](https://jangwook.net/en/tags/core-web-vitals/)

**Chrome intervention policies**
- **Heavy ad intervention** (announced May 2020, for about Chrome 85):
  - Unloads ads that use more than 4 MB of network, more than 15 s of CPU in any 30 s window, or more than 60 s of total CPU.
  - Only 0.3% of ads exceed these thresholds, but they account for 27% of ad network data and 28% of ad CPU.
  - Sources: [Chromium Blog](https://goo.gle/2Xj8HJD); [Computerworld](https://www.computerworld.com/article/1619215/chrome-85-to-erase-ads-that-hog-network-cpu-resources.html)
- **Intensive background-tab throttling** (Chrome 87, November 2020): JS timer wake-ups in background tabs are limited to once per minute. This reduced CPU use by up to 5x and extended battery life by up to 1.25 h in Google's internal tests (2020 Intel MacBook Air). — [Chromium Blog, Nov 2020](https://blog.chromium.org/2020/11/tab-throttling-and-more-performance.html); [BleepingComputer](https://www.bleepingcomputer.com/news/google/google-chrome-feature-reduces-cpu-use-to-extend-battery-life)

**Interop 2026 web-compat area**
- Interop 2026 (announced 2026-02-12) has 20 focus areas. Among them: Navigation API, View Transitions, Scroll-driven animations, IndexedDB, Fetch uploads and ranges, and Scroll snap. — [web.dev Interop 2026](https://web.dev/blog/interop-2026); [WebKit: Announcing Interop 2026](https://webkit.org/?p=17818)
- The **Web compat** focus area covers three items: ESM module loading, the timing of scroll events relative to animation events, and unprefixing `-webkit-user-select`. — [web.dev Interop 2026](https://web.dev/blog/interop-2026); [Microsoft Edge and Interop 2026](https://blogs.windows.com/msedgedev/2026/02/12/microsoft-edge-and-interop-2026/)
- Igalia counts 19 focus areas, 3 cleanup areas and 4 investigation areas, which differs from the 20 counted by WebKit and web.dev. — [Igalia](https://igalia.com/news/interop-2026.html)

### Inferences
- For a browser engine, the most effective levers in the evidence are:
  - bfcache, because of the 20% of mobile navigations that are back/forward, plus relaxing the `no-store` and `unload` blockers.
  - Same-origin prerender, with roughly 40% LCP gains in best cases. Google Search's 60 ms gains show that conservative prefetch helps less.
  - Shipping `scheduler.yield`. Safari is the remaining gap, which limits developer adoption.
- Case-study conversion gains (e.g., Ray-Ban's +101%) compare prerendered with non-prerendered sessions and are likely affected by selection bias (higher-intent users hover or click). Treat them as upper bounds.
- The Firefox 147 module-service-worker release and the Interop 2026 "ESM module loading" item both point to module loading as an active compat pain point in 2026.

### Gaps
- I found no published, engine-wide measured effect of `scheduler.yield` or input prioritization on field INP. Evidence is per-site.
- No web.dev source gave a current bfcache hit-rate figure for Chrome, and I found no measured CWV gain from bfcache at the population level.
- I did not verify whether Chrome has shipped bfcache for `Cache-Control: no-store` pages by October 2026.
- The CrUX 2026 monthly pass rates were not retrieved.
