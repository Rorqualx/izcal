# Izcal images and media: gaps_opportunities

Compiled 19 October 2026. Primary sources were preferred (papers, specifications, maintainers' posts, release notes, issue trackers).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

**Method notes.**
- The shared web-search budget ran out after about 25 searches. After that, only known primary pages were fetched directly. Items not re-fetched are graded Medium or Low and listed in "Claims not verified".
- No source code of any browser or codec library was opened or cited (M-1 part 3). One search result pointed to a Chromium header and a `.cc` file stating how headless virtual time treats audio; it was **not** used, and the behaviour is listed as a black-box measurement (Q-GO-6). Searchfox links returned by one search were not opened.
- "Derived" in the Numbers column means this note's own arithmetic from the cited figures.

---

## 1. What developers get wrong with images (HTTP Archive evidence)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **The LCP element is an image on most pages.** | Image 76.0% mobile, 85.3% desktop; text 23.7% / 14.4% | [Web Almanac 2025, Performance](https://almanac.httparchive.org/en/2025/performance) | 2025 | High |
| 1.2 | **Good LCP is still the weakest Core Web Vital on mobile.** | Good LCP 62% mobile, 74% desktop; poor 13% mobile, 7% desktop | same | 2025 | High |
| 1.3 | **LCP images are still lazy-loaded, and the misuse is growing.** Native `loading=lazy` on the LCP image rose year on year; custom `data-src` lazy-loading fell. | Native lazy LCP 9.5% → 10.4% mobile, 10.2% → 11.5% desktop (2024 → 2025); `data-src` 6.7% → 5.9% mobile; about 16–17% of pages lazy-load the LCP image in total | same | 2025 | High |
| 1.4 | **Few pages tell the browser the LCP image matters.** | `fetchpriority=high` on LCP 17.3% mobile, 16.3% desktop; `<link rel=preload>` 2.1% / 2.2%; `fetchpriority=low` 0.3% | same | 2025 | High |
| 1.5 | LCP images are mostly legacy formats. | JPEG 57%, PNG 26%, WebP 11%, AVIF 0.7% | same | 2025 | High |
| 1.6 | **Format mix of all images (mobile, 2024).** GIF is still a sixth of requests. | JPEG 32.4%, PNG 28.4%, GIF 16.8%, WebP 12%, SVG 6.4%, ICO 1.3%, AVIF 1.0% | [Web Almanac 2024, Media](https://almanac.httparchive.org/en/2024/media) | 2024 | High |
| 1.7 | Bits per pixel by format show where bytes are wasted. | Median 2.1 bpp; GIF 6.7, PNG 3.8, JPEG 2.0, AVIF 1.4, WebP 1.3 | same | 2024 | High |
| 1.8 | **About a third of GIFs are animated**, the case reduced-motion settings target (Section 5). | 32% of GIFs animated (mobile) | same | 2024 | High |
| 1.9 | **Oversized images are the norm.** About half of pages send a largest image above 0.54 megapixels, which the chapter says is more pixels than a mobile viewport can display. | ~50% of pages; median largest image 135 KB | same | 2024 | High |
| 1.10 | **`sizes` is often wrong enough to change which candidate is chosen.** | `srcset` on 42% of pages (mobile); `w` descriptors 62–64%; inaccurate `sizes` 14% mobile, 20% desktop; `<picture>` 9.3% | same | 2024 | High |
| 1.11 | **Most images do not reserve layout space.** | Both `width` and `height` on 32% of `<img>` (mobile) | same | 2024 | High |
| 1.12 | **Colour management is mostly invisible to authors.** Most images carry no ICC profile; wide-gamut profiles are rare. | 87.7% no ICC profile; ~1 in 80 wide-gamut | same | 2024 | High |
| 1.13 | **Alt text is missing on almost half of images.** | Non-blank `alt` 55%; blank or missing 45% | same | 2024 | High |
| 1.14 | Most images cross origins, which matters for what a tool may reveal about them (Section 9). | 62% of image requests cross-domain (both devices) | same | 2024 | High |
| 1.15 | No Web Almanac 2025 Media chapter was found (the 2025 Media URL returned 404 on 19 October 2026). The 2024 chapter is the latest media data. | – | [404 at 2025/media](https://almanac.httparchive.org/en/2025/media) | 2026-10-19 | High |

**Implications:** The biggest real-world image problems are author decisions the engine can detect: a lazy LCP image, a `sizes` value that picks the wrong candidate, an image several times larger than its box, missing dimensions and missing alt. Izcal's tools should name each of these per element, with the bytes and milliseconds it cost, rather than leave them to a separate audit.

## 2. What developers report (surveys, Interop)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **Image management is the second-largest content pain point** in State of HTML 2024, after SVG. | 398 respondents; SVG 73, image management 68, i18n 56, `<iframe>` 42, browser support 38, performance 26 (free-text counts) | [State of HTML 2024: content](https://2024.stateofhtml.com/en-US/features/content/) | 2024 | High |
| 2.2 | **`srcset`/`sizes` is widely used and disliked.** | Used 62.1% (n = 4,297); 81 comments, 58% negative | same | 2024 | High |
| 2.3 | `loading=lazy` is widely used; **`fetchpriority` is unknown to half of respondents**, which matches 1.4. | lazy used 64.4%; `fetchpriority` used 17.6%, never heard of 53.5% | same | 2024 | High |
| 2.4 | **Interop 2026 has no image or `<video>` focus area.** Media items are WebRTC (reliability), the `:playing` media pseudo-classes, and two investigations: **JPEG XL** ("the current tests for this are sparse") and **WebVTT** (review and amend the test suite). | 20 focus areas, 4 investigations | [Mozilla Hacks: Launching Interop 2026](https://hacks.mozilla.org/2026/02/launching-interop-2026/) | 2026-02-12 | High |
| 2.5 | Interop 2025 named WebRTC among reliability work; no other media area is named in Mozilla's 2026 post. | – | same | 2026-02-12 | High |
| 2.6 | The Interop 2027 proposal window ran 3–23 September 2026; results are not yet public. | – | [9to5Mac](https://9to5mac.com/2026/09/07/apple-invites-developers-to-submit-browser-interoperability-ideas-for-interop-2027/) | 2026-09-07 | Low |

**Implications:** Developers struggle most with responsive-image selection and loading priority, both of which the engine decides and could explain. WebVTT and JPEG XL test suites are thin, so Izcal's spec-anchored oracles and conformance streams will matter more than WPT pass rates in those areas.

## 3. What browser DevTools show for media today

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Chrome DevTools Media panel**: per player, Properties, Events, Messages (log, filterable) and a Timeline of playback and buffer status; export as JSON. The page does not mention decoder name, hardware decode or dropped frames. | 4 tabs | [Chrome DevTools: Media panel](https://developer.chrome.com/docs/devtools/media-panel) | 2024-08-20 | High |
| 3.2 | **Firefox has no media panel.** Mozilla's debugging guide sends people to `about:support` (Codec Support Information, hardware decode), the Firefox Profiler "Media" preset, `about:logging` "Media Playback" preset, and module-level logs such as `EME:5`. | – | [Mozilla wiki: Media Debugging Tips](https://wiki.mozilla.org/Media_Debugging_Tips) | undated (retrieved 2026-10-19) | High |
| 3.3 | Firefox's `about:media` was added for Media Source Extensions debug data (bug 1112424, Firefox 38). Its current contents were not checked. | – | [Bugzilla 1112424](https://bugzilla.mozilla.org/show_bug.cgi?id=1112424); [bluishcoder: Firefox MSE update](https://bluishcoder.co.nz/2015/03/03/firefox-media-source-extensions-update/) | 2015-03-03 | Medium |
| 3.4 | **Safari Web Inspector's Media & Animations timeline** lists media elements and plots the DOM events each dispatches. It is off by default because it slows the page. It does not mention dropped frames or power-efficient playback. | – | [WebKit: Timelines tab](https://webkit.org/web-inspector/timelines-tab/) | 2020-09-08 (updated 2022-06-02) | High |
| 3.5 | **Lighthouse "Properly size images"** flags an image when its rendered size (adjusted for DPR) is at least 4 KiB smaller than its actual size; it moved into the "Improve image delivery" insight in Lighthouse 13. It is an audit after the fact, not an explanation of which `srcset` candidate was chosen or why. | 4 KiB threshold | [Lighthouse: Properly size images](https://developer.chrome.com/docs/lighthouse/performance/uses-responsive-images) | 2019-05-02 (page) | High |
| 3.6 | **Autoplay decisions are opaque.** Chrome's policy uses a per-origin Media Engagement Index (playback > 7 s, unmuted audio, active tab, video > 200×140 px; desktop only). The only diagnostic is the `play()` promise rejecting with `NotAllowedError`, plus an internal `about://media-engagement` page. | 4 MEI conditions; Chrome 66 (media), 71 (Web Audio) | [Chrome: Autoplay policy](https://developer.chrome.com/blog/autoplay) | 2018 (policy) | High |
| 3.7 | The W3C **Autoplay Policy Detection** draft adds `navigator.getAutoplayPolicy()` (`allowed`, `allowed-muted`, `disallowed`, by type or per element or `AudioContext`). It says a reason, not why. Implementations are not named in the draft. | 3 values | [Autoplay Policy Detection ED](https://w3c.github.io/autoplay/) | 2025-09-04 | High |
| 3.8 | **The only standard playback-health metric is a pair of counters.** `getVideoPlaybackQuality()` exposes `droppedVideoFrames` ("dropped predecode or dropped because the frame missed its display deadline"), `totalVideoFrames`, `creationTime`; `corruptedVideoFrames` is deprecated. No cause is exposed. | 3 live fields | [Media Playback Quality ED](https://w3c.github.io/media-playback-quality/) | retrieved 2026-10-19 | High |
| 3.9 | Media Capabilities exposes `supported`, `smooth` and `powerEfficient` per configuration: a yes/no hint, not which decoder or why. | 3 booleans | [Media Capabilities ED](https://w3c.github.io/media-capabilities/) | 2026-10-18 | High |
| 3.10 | Mozilla's media guide points users to **YouTube's "Stats for nerds"** to learn which format and resolution is playing: a site-provided overlay is the practical tool for codec and resolution. | – | [Mozilla wiki: Media Debugging Tips](https://wiki.mozilla.org/Media_Debugging_Tips) | retrieved 2026-10-19 | High |

**What no browser tool shows today** (this note's synthesis of 3.1–3.10; each item is absent from every page above):

| Invisible today | Who needs it | Why it is hidden |
|---|---|---|
| Decode time per image, and which thread or process did it | LCP debugging (1.1–1.3) | Decode happens off the main thread in the raster or GPU path; tools show "Decode Image" trace slices at best (not checked this session) |
| Decoded bytes per image, and whether it was downscaled on decode | Memory, FP-13 | Decoded memory lives in caches shared across elements |
| **Why an image looks blurry**: chosen `srcset` candidate, DPR, `sizes` result, downscale factor, upscaled because the candidate was too small | 1.9, 1.10, 2.2 | Selection is an internal algorithm step; Lighthouse only flags bytes (3.5) |
| **Why an image's colour looks the way it does**: embedded ICC or CICP, assumed sRGB, conversion target, HDR tone mapping | 1.12 | Colour management is split between decoder, compositor and OS |
| Why a frame was dropped (decode late, compositor late, GPU busy, background throttling) | 3.8 | Only a counter is exposed |
| A/V sync drift over time and the clock that drives it | Playback quality | Not exposed anywhere found |
| MSE buffer state per SourceBuffer, eviction and append errors in one view | Streaming players | Chrome shows buffer status (3.1); Firefox's `about:media` was MSE-specific (3.3) |
| Autoplay decision and the rule that made it | 3.6, 3.7 | Policy inputs are browser-private |
| Codec chosen, hardware vs software decode, and why the fallback happened | Energy, smoothness | `about:support` shows capability, not per-playback choice (3.2) |
| Energy per playback | Battery (Section 7) | No browser attributes energy to an element |

**Implications:** Every browser exposes media as logs and counters, not causes. Izcal can make "why" the first-class answer: why this candidate, why this colour, why this frame was dropped, why autoplay was refused, each with the spec step (ANC-1) that decided it.

## 4. Captions, subtitles and audio description

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **Firefox does not apply OS caption style preferences** to unstyled WebVTT captions; the reporter says Edge, Safari and Chrome do. Open, unassigned, reproduced on macOS 10.15 in Firefox 90–92. | Opened ~2021, still NEW | [Bugzilla 1721736](https://bugzilla.mozilla.org/show_bug.cgi?id=1721736) | 2021 (retrieved 2026-10-19) | High |
| 4.2 | Chrome's **Live Caption** generates captions for any page audio **on device**: "All audio and captions are processed locally, are never stored, and never leave your device." Caption fonts come from the OS caption preferences. **Live Translate sends captions to Google.** | Windows, Mac, Linux, ChromeOS; 100+ translation targets | [Chrome Help: Live Caption](https://support.google.com/chrome/answer/10538231) | retrieved 2026-10-19 | High |
| 4.3 | In 2016 a W3C text-tracks thread said no browser had implemented the WebVTT `STYLE` block; in 2015 cue-ID `::cue(#id)` selectors worked in no browser. Current support was not re-checked. | – | [public-texttracks 2016-05](https://lists.w3.org/Archives/Public/public-texttracks/2016May/0004.html); [public-tt 2015-03](https://lists.w3.org/Archives/Public/public-tt/2015Mar/0014.html) | 2015–2016 | Low |
| 4.4 | Interop 2026 opened a **WebVTT investigation** to fix the test suite, an admission that caption interoperability cannot yet be measured well. | 1 of 4 investigations | [Mozilla Hacks: Interop 2026](https://hacks.mozilla.org/2026/02/launching-interop-2026/) | 2026-02-12 | High |
| 4.5 | WebVTT reftests from WPT were reported as highly intermittent on Windows in Firefox (timeouts or about half failing). Which bug says this was not checked. | – | [Bugzilla 1018936](https://bugzilla.mozilla.org/show_bug.cgi?id=1018936) | 2014 | Low |
| 4.6 | `<track>` usage was not reported in the 2024 Media chapter; only 3% of `<video>` have `poster` and 9% `controls`, so caption UI is mostly custom player code. | controls 9% | [Web Almanac 2024, Media](https://almanac.httparchive.org/en/2024/media) | 2024 | High |
| 4.7 | Audio description: the HTML `<track kind=descriptions>` is defined for text descriptions, but this session found no evidence on how browsers voice them, and none on browser support for alternative audio tracks (`audioTracks`). | – | [HTML: the track element](https://html.spec.whatwg.org/multipage/media.html#the-track-element) | living standard | Medium |

**Implications:** Izcal should treat captions as a user-agent responsibility: apply OS caption preferences by default (with `::cue` styling layered as the spec allows), offer on-device captions with no network path (B10 off by default), and voice `kind=descriptions` cues through the semantic plane. WebVTT interop is weak enough that Izcal's own spec-anchored WebVTT tests are needed.

## 5. Animated images and reduced motion

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **Safari honours a system "Auto-Play Animated Images" setting** (iOS 17 Accessibility > Motion; macOS 14 Accessibility > Display). Paused GIFs show a play control. | Since iOS 17 / macOS 14 (2023) | [AbilityNet: macOS 14](https://mcmw.abilitynet.org.uk/how-to-stop-websites-and-apps-automatically-playing-animated-images-in-macos-14-sonoma); [Michael Tsai](https://mjtsai.com/blog/2023/09/15/automatically-pausing-animated-images) | 2023-09-15 | Medium |
| 5.2 | **Firefox does not honour that macOS setting** (enhancement, open). It links bug 1725474, which asks for a mode that behaves as if every site saw `prefers-reduced-motion: reduce`. | Open ~3 years | [Bugzilla 1853035](https://bugzilla.mozilla.org/show_bug.cgi?id=1853035) | 2023 (retrieved 2026-10-19) | High |
| 5.3 | Firefox's only control is the hidden pref `image.animation_mode` (`normal`, `once`, `none`). | 3 values | [Snook: stopping animated GIFs in Firefox](https://www.snook.ca/archives/browsers/stopping_animat) | old | Low |
| 5.4 | **There is no author control to pause an animated image.** CSSWG issue 1615 "Disabling image animation" is open; Florian Rivoal (May 2025) notes that when Safari and Firefox stop animations users cannot turn individual ones back on, and proposes `image-animation: synchronized \| independent \| paused <time>?` with a UA rule that pauses images under reduced motion. | Issue open since 2017 | [public-css-archive 2025-05](https://lists.w3.org/Archives/Public/public-css-archive/2025May/0015.html); [csswg-drafts #1615](https://github.com/w3c/csswg-drafts/issues/1615) | 2025-05-01 | High |
| 5.5 | Autoplay Policy Detection explicitly excludes animated GIFs, so pages cannot ask whether animation is allowed. | – | [Autoplay Policy Detection ED](https://w3c.github.io/autoplay/) | 2025-09-04 | High |
| 5.6 | A WebKit bug reported a video used as a poster image playing despite Safari's "never auto-play" setting: video-as-image (MP4 in `<img>` or `<video muted autoplay loop>`) slips past image-animation controls. | – | [WebKit bug 239961](https://bugs.webkit.org/show_bug.cgi?id=239961) | 2022 | Low |

**Implications:** Izcal should pause animated images (GIF, APNG, animated WebP, AVIF sequences, JPEG XL animation, and muted looping video) when the OS reduced-motion or animated-image setting is on, with a per-image play control, and track CSSWG #1615 so authors can opt specific images back in.

## 6. Alt text generated by browsers, and privacy

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **Chrome's "Get image descriptions from Google" sends unlabeled images to a Google server.** Opt-in per page or for all pages; follows sync to all signed-in devices. | Server-side ML | [Chrome Help: image descriptions](https://support.google.com/chrome/answer/9311597) | retrieved 2026-10-19 | High |
| 6.2 | At launch Google reported more than 10 million descriptions generated in testing, hundreds of thousands per day, English only; a description is shown only when confidence is "reasonable". | >10 M descriptions | [Google blog](https://blog.google/outreach-initiatives/accessibility/get-image-descriptions/) | 2019-10-11 | High |
| 6.3 | **Firefox generates alt text on device** for its PDF editor: a ViT encoder plus distilled GPT-2 decoder, about 182 M parameters, under 200 MB quantized, run by an embedded ONNX runtime with Transformers.js in a separate process; "All operations are contained within the device". Fine-tuned on COCO, then retrained to reduce bias. | ~182 M params; a few seconds per image on a laptop; first download minutes | [Mozilla Hacks: local alt-text generation](https://hacks.mozilla.org/2024/05/experimenting-with-local-alt-text-generation-in-firefox-nightly/) | 2024-05-31 | High |
| 6.4 | Firefox shipped it in Firefox 130's PDF editor with user validation; the stated end goal is alt text for screen-reader users in general browsing. | Firefox 130 | same | 2024-05-31 | High |
| 6.5 | 45% of web images lack alt text (1.13), so a generated description would apply to roughly half of images. | 45% (derived) | [Web Almanac 2024, Media](https://almanac.httparchive.org/en/2024/media) | 2024 | High |

**Implications:** The design space is clear: server-side (Chrome) exposes every unlabeled image a user sees to a third party; on-device (Firefox) does not. Izcal, with AI off by default (B10), can offer on-device descriptions as an opt-in, always marked as generated in the semantic plane's repair overlay (B8), never sent off the device, and never fed back to the page.

## 7. Determinism and testing of media

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **Fake capture devices are the standard way to test `getUserMedia` deterministically.** Chrome: `--use-fake-ui-for-media-stream`, `--use-file-for-fake-video-capture=<file>`, `--use-file-for-fake-audio-capture=<file>`; Firefox: pref `media.navigator.streams.fake`. | – | [webrtc.org: Testing](https://webrtc.org/getting-started/testing) | 2019-05-28 | High |
| 7.2 | Chrome headless `--virtual-time-budget` fast-forwards timers, and virtual time does not advance while fetches are pending. Its interaction with audio and video clocks is not documented in prose (Q-GO-6). | – | [Chrome headless CLI](https://developer.chrome.com/docs/automation-and-testing/headless-cli) | retrieved 2026-10-19 | Medium |
| 7.3 | **A production HTML-to-video renderer gave up on `<video>` in headless Chrome.** With BeginFrame control, decoders "skip frames, fail to decode, or stay at `readyState: 0`"; different machines and codec paths gave different output. Its fix: pre-extract every frame with FFmpeg to JPEGs, swap in an `<img>` per frame, and use one worker because parallel seeks ran out of decoders. | 5 s at 30 fps = 150 frames | [HeyGen research: HTML to video](https://www.heygen.com/research/html-to-video) | undated | Medium |
| 7.4 | **Media tests are a known flake source.** A WebKit bug argues that expecting a video to load within one second is unreasonable for a regression test under CPU load. Mozilla bugs record intermittent MSE and WebVTT WPT tests and decoder-dependent timeouts (bug-to-claim mapping not checked). | – | [WebKit 145257](https://bugs.webkit.org/show_bug.cgi?id=145257); [Bugzilla 1066467](https://bugzilla.mozilla.org/show_bug.cgi?id=1066467); [Bugzilla 1096748](https://bugzilla.mozilla.org/show_bug.cgi?id=1096748) | 2014–2015 | Low |
| 7.5 | **Replay.io's limitations list recording length (about 2 minutes recommended), no WebGL, and WebRTC as a platform limitation**; this session found no statement on how it records `<video>` or `<audio>`. The Notion page did not render on re-fetch. | ~2 min | [Replay.io limitations](https://replayio.notion.site/Limitations-5b33bb0e5bd1459cbd7daf3234219c27) | retrieved 2026-10-19 (via search snippet) | Low |
| 7.6 | Replay Chrome was built on Chromium 108 per its docs; a record/replay browser on an old engine lags media features. | v108 | [Replay docs: Replay Chrome](https://docs.replay.io/reference/replay-runtimes/replay-chrome) | retrieved 2026-10-19 (snippet) | Low |
| 7.7 | rr-style user-space record/replay records syscalls and signals; GPU and hardware-decoder state is outside what it can record, so hardware decode is not replayable by construction (this note's inference from the HAL device-class model; no rr media statement was found). | – | [rr project](https://rr-project.org/) | – | Low |
| 7.8 | The `dav1d` tests ran on every commit during the rav1d port and were the oracle for the rewrite: the existing conformance and regression suite, not WPT, is what made a full decoder rewrite safe. | – | [Prossimo: porting C to Rust for AV1](https://www.memorysafety.org/blog/porting-c-to-rust-for-av1/) | 2024-09-09 | High |

**Implications:** Determinism for media splits cleanly. Software decode of a given byte stream can be a pure function (bit-exact for AV1, VP9, VP8, FLAC, PNG), so it replays for free under B1. The timing around it (when a frame was presented, network arrival, audio device clock, hardware decode) must be recorded inputs. Other tools work around non-determinism by abandoning `<video>` (7.3); Izcal can make it replayable instead.

## 8. Cost and effort of building decoders and media stacks

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **dav1d is about 50k lines of C and 250k lines of assembly.** | 50k C, 250k asm | [Prossimo: porting C to Rust for AV1](https://www.memorysafety.org/blog/porting-c-to-rust-for-av1/) | 2024-09-09 | High |
| 8.2 | **Porting dav1d's C to Rust (keeping its assembly) took more than 20 person-months against an estimate of about 7**, with 3 developers plus codec expert Frank Bossen. Causes: preserving performance, transpiler and codebase quirks (duplicated 8- and 16-bit code), Rust's thread-safety model being "overly strict" for dav1d's implicit sharing. | ~7 → >20 person-months; 3 devs | same | 2024-09-09 | High |
| 8.3 | **rav1d performance vs C**: transpiled 3.8% slower; peak 11% slower while making it safe; under 6% at September 2024 on x86-64 (Ryzen 7700X, 8 threads, Chimera 1080p 8-bit); similar on other x86 and ARM. Bounds checks were under 1% of the transpiled overhead; 7% more instructions, ~2% of them bounds checks. | 3.8% / 11% / <6% | [Prossimo: optimizing rav1d](https://www.memorysafety.org/blog/rav1d-performance-optimization/) | 2024-09-10 | High |
| 8.4 | In May 2025 rav1d was about **5% slower** than dav1d and Prossimo offered a **$20,000 bounty** for parity, saying its own team and contractors could not close the gap. Whether it was won was not checked. | ~5%; $20k | [Prossimo: rav1d performance bounty](https://www.memorysafety.org/blog/rav1d-perf-bounty/) | 2025-05-14 | Medium |
| 8.5 | rav1d work began 1 March 2023 (Immunant, contractor). AWS gave Prossimo $1 M in May 2023 across four projects including rav1d; the contract value for rav1d is not public. | $1 M / 4 projects | [Prossimo AV1 initiative](https://memorysafety.org/initiative/av1); [AWS funding](https://www.memorysafety.org/blog/aws-funding/) | 2023 | Medium |
| 8.6 | **Chrome 155 ships JPEG XL decoding with jxl-rs, a pure Rust decoder** by Google Research's JPEG XL team, "approximately as fast as the best non-memory-safe alternative"; fuzzing and AI code review found no memory-safety bugs "throughout the entire implementation history". No effort, team size or speed figures are given. | 5 authors + 5 acknowledged | [Chrome: Shipping JPEG XL in Chrome](https://developer.chrome.com/blog/jpeg-xl-in-chrome) | 2026-10-06 | High |
| 8.7 | Secondary reports say jxl-rs was merged into Chromium in January 2026 and was behind a flag from Chrome 145. | – | [zeli.app summary](https://zeli.app/story/49991227) | 2026-10 | Low |
| 8.8 | **Wuffs** proves bounds and overflow safety at compile time (no runtime checks), cannot allocate or make syscalls, and claims PNG 1.2–2.7× and GIF 2–6× faster than libpng/giflib and the Go and Rust equivalents (project's own benchmarks). Its GIF decoder has shipped in Chrome since M93 (June 2021). | 1.2–2.7× PNG; 2–6× GIF | [Wuffs README](https://github.com/google/wuffs) | 2023-11 | Medium |
| 8.9 | **libwebp CVE-2023-4863** (lossless Huffman table heap overflow) was exploited in the wild. Ben Hawkes says OSS-Fuzz covered lossless WebP extensively, but "the preconditions to trigger this issue are immense" so coverage fuzzing would likely not reach it; he recommends source review, sandboxing, and less reliance on fuzzing. | 1 ITW 0-day | [Isosceles: The WebP 0day](https://blog.isosceles.com/the-webp-0day/) | 2023-09-21 | High |
| 8.10 | **Ladybird moved from its own media code to FFmpeg**: video decoding via FFmpeg in 2024 ("access to a lot more formats than just VP9"), audio decoding in September 2024, and its own Matroska demuxer replaced by `libavformat` in March 2025 because it "could only play H.264 videos if they were in an MKV container". | 3 replacements in ~9 months | [Ladybird March 2025](https://ladybird.org/newsletter/2025-03-31/); [Ladybird September 2024](https://buttondown.com/ladybird/archive/this-month-in-ladybird-september-2024) | 2024-09 – 2025-03 | High (March 2025); Medium (2024) |
| 8.11 | Ladybird media in 2026: MSE with VP9/Opus in WebM (March, "performance and memory use still needed work"); playback rate with pitch-preserving time-stretch, and muted autoplay by default (June); frame selection moved into the compositor (July); MSE fragmented MP4 enabling H.264, HEVC and AV1, Twitch and Plex (August). About 18 months from FFmpeg adoption to mainstream streaming sites. | ~18 months (derived) | [Ladybird March 2026](https://ladybird.org/newsletter/2026-03-31); [June 2026](https://ladybird.org/newsletter/2026-06-30); [July 2026](https://ladybird.org/newsletter/2026-07-31/); [August 2026](https://ladybird.org/newsletter/2026-08-31) | 2026 | Medium |
| 8.12 | **Servo's media crate used GStreamer as its only functional backend** for HTMLMediaElement, Web Audio, WebRTC and media capture; the repository was archived on 23 April 2026. | 1 backend | [servo/media README](https://github.com/servo/media) | 2026-04-23 | High |
| 8.13 | dav1d is developed by the VideoLAN and FFmpeg communities and sponsored by AOM; VideoLAN gives no contributor counts or years. | – | [VideoLAN: dav1d](https://www.videolan.org/projects/dav1d.html) | retrieved 2026-10-19 | High |
| 8.14 | No public team sizes were found for Firefox media, Chrome media, Apple or Google video teams (searches ran out). | – | – | – | – |

**Implications:** The best public cost datum is rav1d: >20 person-months to port only the C part of one decoder (with its 250k lines of assembly kept and an expert advisor), and still ~5% slower. A from-scratch Izcal AV1 decoder must also own the SIMD work (the 250k lines dav1d keeps in assembly) under M-1, so its cost is a multiple of rav1d's, not a fraction. Ladybird and Servo are the counter-evidence: both new engines that tried their own media stacks moved to FFmpeg or GStreamer to reach real sites. Izcal's no-library rule (HGBA-2 2.3.2) has no precedent in a shipping engine, and the spec should budget for it explicitly.

## 9. Energy for video playback

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | **Apple claims Safari gives "up to 5 hours more streaming videos compared with Chrome"** on a 15-inch MacBook Air M5 with 1080p content (Safari 27.0 vs Chrome 151.0.7922.76, August 2026), and "up to 18 hours of video streaming". | +5 h vs Chrome | [Apple: Safari](https://www.apple.com/safari/) | 2026-08 (testing) | Medium (vendor, no data) |
| 9.2 | If the 18-hour figure is Safari's and the battery is about 66.5 Wh (the capacity of earlier 15-inch MacBook Airs; M5 not checked), Safari's whole-system draw is about 3.7 W and Chrome's (13 h) about 5.1 W: a ~38% difference between browsers on the same hardware for the same video. | ~3.7 W vs ~5.1 W (derived) | same | – | Low |
| 9.3 | **Microsoft's 2017 battery tests**: identical Surface Books streaming video full-screen until empty; Edge 12 h 31 m, Chrome 9 h 17 m, Firefox 7 h 4 m. Methodology published (brightness, Quiet Hours, radios fixed). Old versions (Chrome 57, Firefox 52). | Edge +35% vs Chrome, +77% vs Firefox | [TechRepublic on Microsoft's test](https://www.techrepublic.com/article/windows-10-chrome-vs-firefox-vs-edge-in-microsofts-new-battery-life-showdown/) | 2017-04 | Low |
| 9.4 | **Hardware vs software decode on a Linux laptop** (Framework 13, i5-1240P, community measurements with PowerTOP): idle ~5 W; Firefox 112 with VA-API, YouTube 1080p ~10 W; Chromium 112 without hardware decode ~11 W; mpv with hardware decode ~9 W. Another user: Firefox HW decode ~9–10 W vs HW off ~10–15 W. | Browser ~1–2 W over mpv; video adds ~4–6 W over idle | [Framework forum: Linux HW decode power](https://community.frame.work/t/tracking-linux-hardware-video-decoding-power-consumption/29592) | 2023-04 – 2023-05 | Low |
| 9.5 | **Software VP9 on a 6 W Celeron mini PC**: Chrome VP9 ~11 W with many dropped frames; forcing H.264 (hardware) ~8 W; Edge H.264 7–8 W with no drops. | +3 W for software decode | [CNX Software](https://cnx-software.com/2016/06/29/improve-youtube-video-playback-on-low-power-intel-mini-pcs-by-disabling-vp9-support-in-chrome-or-firefox) | 2016-06-29 | Low |
| 9.6 | A Firefox user reported 25% GPU use vs 13% in Edge for the same 1080p60 full-screen video on Windows (closed WORKSFORME): composition path differences show up as GPU load. | 25% vs 13% | [Bugzilla 1746496](https://bugzilla.mozilla.org/show_bug.cgi?id=1746496) | 2021 | Low |
| 9.7 | Mozilla's power-profiling guide recommends joules and watts from RAPL-class counters (accurate to ~20 Hz), notes most are whole-package not per-process, and advises against Activity Monitor's "Energy Impact". No browser attributes energy to a page element or a playback. | ~20 Hz useful rate | [Firefox source docs: power profiling overview](https://firefox-source-docs.mozilla.org/performance/power_profiling_overview.html) | retrieved 2026-10-19 | High |
| 9.8 | Media Capabilities `powerEfficient` is the only web-exposed energy signal, and it is a boolean. | 1 bit | [Media Capabilities ED](https://w3c.github.io/media-capabilities/) | 2026-10-18 | High |
| 9.9 | No peer-reviewed, cross-browser video-playback energy study with published methodology was found this session (search budget ran out before academic sources). | – | – | – | – |

**A first-principles budget metric** (this note's proposal, for the synthesis author):
- **Metric:** whole-system energy per minute of playback (J/min, or average W) for a fixed clip, codec, resolution and frame rate, minus the idle baseline of the same machine with the same display state, reported as a ratio to the OS's own player (QuickTime/Movies on macOS, Films & TV/Media Player on Windows, mpv with hardware decode on Linux) playing the same file.
- **Why a ratio:** absolute watts vary 2–3× across hardware (9.2, 9.4, 9.5); the OS player shows what that hardware can do with hardware decode and direct scanout. 9.4 suggests a browser can be within ~1–2 W of mpv on the same machine, and 9.2 shows browsers differ by ~38% on one machine.
- **Variants needed:** windowed vs full-screen (direct scanout or overlay applies only in some cases), hardware vs software decode (AV1 on hardware without AV1 decode), with captions, with a page busy in the background.
- **Threshold:** to be set from M-phase measurement (†); 9.2–9.5 argue it is achievable to be within a small margin of the OS player when hardware decode and overlays are used.

**Implications:** Energy is the clearest place where browsers differ by tens of percent for identical work, and no browser shows it per playback. Izcal's FP-17 to FP-23 attribution plus a new playback budget (frame.md notes FP has none) can make "this video cost X J/min, Y% over the OS player, because software decode / no overlay" a visible fact.

## 10. Fingerprinting surface of media APIs

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | **AudioContext fingerprinting was found in the wild in 2016**: scripts process an `OscillatorNode` signal (through a compressor or an `AnalyserNode` FFT) and hash the output. On a demo page, 18,500 devices gave 713 distinct fingerprints, estimated at **5.4 bits**. The most popular script was on 512 sites. | 5.4 bits; 713 / 18,500; 512 sites | [Englehardt and Narayanan, 1-million-site measurement (ACM CCS 2016)](https://www.cs.princeton.edu/~arvindn/publications/OpenWPM_1_million_site_tracking_measurement.pdf) | 2016 | High |
| 10.2 | Same study: **WebRTC was used to discover local IP addresses without user interaction on 715 of the top 1 M sites**, 659 by third-party scripts, 625 for tracking. Canvas fingerprinting was on 14,371 sites (1.6%). | 715; 14,371 | same | 2016 | High |
| 10.3 | **Media Capabilities** says it "provides easier access to some information that can be used to fingerprint users"; HDR fields (`colorGamut`, `transferFunction`, `hdrMetadataType`) can add "significant entropy", but none for a software-decoding UA. Suggested mitigations: fake a fixed capability set, top-level only, a privacy budget, and stricter controls for sites that probe without using. | 3 HDR fields | [Media Capabilities ED, Section 3](https://w3c.github.io/media-capabilities/) | 2026-10-18 | High |
| 10.4 | **Brave farbles** (per-session, per-eTLD+1 seeded noise) Web Audio `AnalyserNode` and `AudioBuffer` reads, canvas, WebGL and `enumerateDevices`; "Maximum" returns randomized values with no true input. | – | [Brave: Fingerprinting defenses 2.0](https://brave.com/privacy-updates/4-fingerprinting-defenses-2.0/) | 2020-05-18 | High |
| 10.5 | **Tor Browser** disables WebRTC at compile time, disables MediaDevices "as a defense-in-depth", isolates media streams to the URL-bar domain, disables the media disk cache, and disables Gecko Media Plugins (EME, OpenH264) because they "can't be built reproducibly or are binary blobs". | – | [Tor Browser design](https://2019.www.torproject.org/projects/torbrowser/design/) | 2019 (archived) | High |
| 10.6 | Firefox `resistFingerprinting` also spoofs media statistics (`getVideoPlaybackQuality`, decoded frame counts) and limits codec exposure; the SUMO page did not render, so this is from memory. | – | [Mozilla SUMO: resist fingerprinting](https://support.mozilla.org/en-US/kb/resist-fingerprinting) | – | Low |
| 10.7 | `droppedVideoFrames` (3.8) and `powerEfficient`/`smooth` (3.9) reveal machine load and decoder hardware: the playback-health signals developers want are also fingerprinting signals. | – | [Media Playback Quality ED](https://w3c.github.io/media-playback-quality/) | – | Medium |

**Implications:** Izcal's FP-D13 (every codec in every build, identical everywhere) removes `canPlayType` entropy for software codecs; the remaining entropy is hardware decode (`powerEfficient`, HDR fields) and Web Audio output. Since Izcal owns its audio stack and `hg-math` (JS-D12 cross-ISA identity), Web Audio rendering can be made bit-identical across machines, which would remove the 5.4-bit AudioContext fingerprint at the source instead of adding noise. Rich media diagnostics must be developer-tool-only and never page-exposed.

## 11. Opportunities unique to Izcal

Each row joins a gap above to a capability in `izcal-capabilities.md`. These are proposals for the synthesis, not findings.

| # | Opportunity | Gap it closes | Izcal capability | Evidence |
|---|---|---|---|---|
| 11.1 | **Deterministic decode replay**: software decoder output is a pure function of bytes; a recording stores bytes and presentation decisions, and replays bit-exactly on any OS | Media flakes (7.4), abandoned `<video>` in headless tools (7.3) | B1, DET-1; JS-D15 recorded decision points | 7.3, 7.4, 7.8 |
| 11.2 | **Spec-anchored decoders**: each decode step carries `// spec:` anchors to the bitstream specification (AV1 spec section, PNG chunk, JPEG T.81 clause); a corrupt-stream error names the clause violated | Opaque decode errors (3.1–3.3); thin JPEG XL and WebVTT tests (2.4) | ANC-1, `hg-anchor` | 2.4, 8.9 |
| 11.3 | **"Why does this image look like this"**: one view of the chosen `srcset` candidate and why, DPR, decode downscale, ICC/CICP source space, conversion target, HDR tone map, and the resulting on-screen size vs pixels | 1.9, 1.10, 1.12, 2.2 | Own decoders, own colour pipeline, ANC-1 | 1.9–1.12 |
| 11.4 | **LCP image explainer**: lazy-loaded LCP, missing `fetchpriority`, late discovery, decode time and its process, each with a fix | 1.1–1.4, 2.3 | Typed IPC (B2) shows fetch → decoder process → GPU service timing | 1.3, 1.4 |
| 11.5 | **Cause-of-drop timeline**: each dropped frame tagged with its cause (late decode, late composite, throttled, GPU busy) and A/V drift plotted against the audio clock | 3.8 | Own pipeline; incremental core with reference path (B5) | 3.8 |
| 11.6 | **Per-site, per-element media energy**: J/min per playback and the ratio to the OS player, with the reason (software decode, no overlay) | 9.1–9.9 | FP-17 to FP-23 attribution, per-(site, container) decoder process (FP-D11) | 9.2, 9.4 |
| 11.7 | **Autoplay explanation**: which rule allowed or refused autoplay, and which input (gesture, engagement, permission policy) | 3.6, 3.7 | Recorded decision points (JS-D15) | 3.6 |
| 11.8 | **Agent-accessible media inspection**: an agent can query decode state, candidates, captions, dropped-frame causes through the tool registry, under the agent principal with origin capabilities | All of Section 3 | B9, AI off by default (B10) | – |
| 11.9 | **Private alt text**: on-device descriptions, opt-in, marked as generated in the repair overlay, never sent off device | 1.13, 6.1–6.4 | B8 semantic plane, B10 | 6.1, 6.3 |
| 11.10 | **Captions as UA duty**: OS caption preferences applied by default, `kind=descriptions` voiced through the semantic plane, on-device captions for uncaptioned media | 4.1–4.7 | B8, own WebVTT implementation | 4.1, 4.2 |
| 11.11 | **Reduced motion covers all animated images and muted looping video**, with per-image play | 5.1–5.6 | Own image and media pipeline | 5.4 |
| 11.12 | **Bit-identical Web Audio** across machines removes AudioContext fingerprinting at source | 10.1 | Own audio stack, `hg-math`, JS-D12 | 10.1 |

## 12. Open questions and black-box measurements needed

| ID | Question | How to answer |
|---|---|---|
| Q-GO-1 | Does Chrome (current stable) honour the macOS/iOS "Auto-Play Animated Images" setting; does any browser honour Windows "Show animations"? | Black-box: toggle the OS setting, load an animated GIF, APNG, animated WebP and AVIF |
| Q-GO-2 | Which browsers apply OS caption preferences to WebVTT today, and which honour `::cue(#id)` and the WebVTT `STYLE` block? | Black-box test page per browser and OS |
| Q-GO-3 | How do current browsers expose `kind=descriptions` tracks to screen readers, if at all? | Black-box with VoiceOver, NVDA, Orca |
| Q-GO-4 | Whole-system energy per minute for 1080p and 4K AV1, VP9, H.264 in Safari, Chrome, Firefox, Edge vs the OS player, windowed and full-screen, on one Apple Silicon Mac, one Windows laptop and one Linux laptop | Book an exclusive rig window; powermetrics / RAPL / external meter |
| Q-GO-5 | Energy of AV1 software decode vs hardware decode on a machine with both | Same harness, hardware decode forced off |
| Q-GO-6 | Under Chrome headless virtual time, does audio advance in real time and drag video with it? | Black-box: audible clip under `--virtual-time-budget`, compare `currentTime` to virtual clock |
| Q-GO-7 | What does Firefox `about:media` show today, and does Safari show hardware vs software decode anywhere? | Inspect current builds |
| Q-GO-8 | Entropy of Media Capabilities answers (HDR fields, `powerEfficient`) across a device sample | Published study search when budget refills; else a small panel |
| Q-GO-9 | Was the rav1d parity bounty won, and what is rav1d's current gap? | Re-read Prossimo blog when search budget refills |
| Q-GO-10 | jxl-rs speed and peak memory vs libjxl (Chrome links a performance dashboard) | Read the dashboard; black-box measure with Chrome 155 |

## 13. Claims not verified

- **Firefox Profiler power tracks** (Windows 11 and Apple Silicon energy counters in the profiler) are known from prior knowledge only; the documentation page did not render. Not used as a row.
- **Firefox `resistFingerprinting` media behaviour** (10.6): SUMO did not render; graded Low.
- **The 2017 Edge battery numbers** (9.3) come from press coverage of Microsoft's test, not Microsoft's post.
- **The 66.5 Wh battery capacity** behind 9.2 is from earlier 15-inch MacBook Air models; the M5 capacity was not checked.
- **Which Mozilla bug says what** in 4.5 and 7.4 was not checked bug by bug.
- **Replay.io's media handling**: no statement found; the limitations page was seen only as a search snippet.
- **Interop 2024 media areas** were not confirmed this session.
- **Team sizes** for Firefox, Chrome, Apple and Google media teams: nothing public found before the budget ran out.
- **Academic browser video-energy studies** and **AV1 decoder hardware papers**: not reached (budget).
- **Web Almanac 2025 Media chapter**: the URL returned 404; the chapter may exist under another path.
