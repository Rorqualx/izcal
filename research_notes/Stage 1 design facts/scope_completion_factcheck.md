# Scope-completion fact-check (Stage 1 design facts)

Checked 14 October 2026. Verdicts: **correct / partly / wrong / unverifiable**. The "Date" column gives the date of the source.

## DRM

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 1a | Widevine is free to license, but production use needs Google approval / VMP | correct | Access requires a corporate account and signing the Widevine Master License Agreement. Google publishes no fee schedule. Desktop browsers also need VMP signing. | https://developers.google.com/widevine/access (current); Electron Widevine doc https://github.com/electron/electron/blob/v10.0.0-beta.24/docs/tutorial/testing-widevine-cdm.md |
| 1b | Third-party labs such as castlabs can certify | partly | castLabs is a Google-authorised 3PL, but the documented 3PL scope is AOSP devices. Its browser offering (EVS signing, commercial VMP certification) covers **Electron and custom Chromium builds only**. I found no castlabs offering for non-Chromium engines. | https://jotup.co/node/1510752 (2021); https://pypi.org/project/castlabs-evs |
| 1c | A developer waited 4 months and was rejected | correct | Samuel Maddock (Metastream, an Electron app) waited 4 months. Widevine replied: "we're not supporting an open source solution like this." | https://blog.samuelmaddock.com/posts/google-widevine-blocked-my-browser/ (2 Apr 2019) |
| 1d | Brave's founder said Google's review has "serious delays" | wrong (misattributed) / unverifiable | The phrase was not found anywhere. The documented quote is from Brave co-founder/CTO **Brian Bondy** (not Eich): "Small share browsers are at the mercy of Google, and Google is stalling us for no communicated-to-us reason" (2017). | Same Maddock post; https://www.theregister.com/2019/04/03/googles_widevine_drm/ |
| 1e | Vivaldi on Windows-on-ARM was blocked until Google shipped Widevine for WoA (XDA 2022) | correct | XDA, 14 Oct 2022: Vivaldi said a native build was a "flat no" without Widevine, and PlayReady "would take some work". Google began shipping a native arm64 Windows Widevine around Apr 2023. Vivaldi shipped a native WoA build in 6.9 (2024). | https://www.xda-developers.com/google-holding-back-native-browsers-windows-on-arm/ ; https://winfuture.de/news,135463.html (1 Apr 2023); https://www.thurrott.com/?p=307751 |
| 2 | Firefox Widevine "supports only x64 binaries because Google only publishes those" | wrong | **Windows ARM64:** Firefox 67+ used the **x86** CDM under emulation. Google has published native arm64 Windows CDMs since around 2023. **macOS arm64:** Firefox originally ran the x64 CDM under Rosetta 2. Chrome's native arm64 CDM exists, but I could not confirm Firefox's current path. **Linux aarch64:** bug 1612995 is still NEW and not shipped. Google's native Linux arm64 CDM arrived with Chrome 150 for Linux arm64 (Jul 2026). Mozilla is working on it (dependency bug 2076804 ASSIGNED; a user confirmed it works in Fx156 by manual config). Google also publishes x86 (32-bit) CDMs. | https://bugzilla.mozilla.org/show_bug.cgi?id=1534157 (2019); https://hacks.mozilla.org/2021/01/porting-firefox-to-apple-silicon ; https://bugzilla.mozilla.org/show_bug.cgi?id=1612995 (comments Sep 2026); https://www.omgubuntu.co.uk/2026/07/chrome-arm64-linux-available |
| 3 | Firefox uses the Windows Media Foundation PlayReady CDM | correct (with limits) | Shipped in **Firefox 132** (Oct 2024, bug 1919627). It is enabled only for allow-listed sites: initially Netflix, rolled out by gradual Netflix A/B test. Hulu was fixed later. Canal+ and RTL are tracked. Netflix 4K works for "some Firefox users" with SL3000 hardware, not all. A Jul 2026 secondary source still says Firefox is 1080p-capped, so treat 4K as partial. | https://bugzilla.mozilla.org/show_bug.cgi?id=1919627 ; https://bugzilla.mozilla.org/show_bug.cgi?id=1806566 |
| 3b | A third-party browser can use PlayReady via MF without a separate Microsoft agreement | unverifiable | The CDM ships with Windows and is reachable via MF APIs: Chromium's MediaFoundationCdm, Firefox, and reportedly Chrome for Netflix 4K (Jul 2026). I found no Microsoft statement on licensing terms. In practice, **services gate by browser** (Netflix allow-lists). Get legal confirmation. | https://blog.son-video.com/en/2026/07/watching-netflix-in-4k-on-chrome-is-finally-possible/ (secondary) |
| 4 | Ladybird Oct 2025 newsletter: "very likely not have access to Widevine…" | unverifiable (likely wrong citation) | The Oct 2025 newsletter has no Widevine/DRM mention. I scanned the Aug 2025 to Sep 2026 newsletters, ladybird.org, the FAQ and GitHub issues with no hit. Do not cite it as a newsletter quote. | https://ladybird.org/newsletter/2025-10-31/ |

## Codecs

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 5a | MPEG LA 2010: free internet video royalty-free forever | partly | The waiver covers only **Internet Broadcast AVC Video free to end users** (content distribution). **Decoders and encoders in products, browsers included, remain royalty-bearing.** | https://us.design-reuse.com/news/24238/internet-broadcast-avc-video.html (26 Aug 2010) |
| 5b | Via LA AVC decoder terms | — | 0–100,000 units/yr free (per enterprise). $0.20/unit up to 5M, then $0.10. Enterprise cap **$9.75M/yr**. Streaming tiers were raised for new licensees in 2026 (up to $4.5M). | https://www.via-la.com/licensing-programs/avc-h-264/ (Sep 2026); https://www.streamingmedia.com/Articles/News/Online-Video-News/H.264-Streaming-Fees-What-Changed-Whos-Affected-and-What-It-Means-173935.aspx (17 Mar 2026) |
| 5c | Firefox / Chromium handling of H.264/AAC | correct (summary) | Firefox prefers OS decoders (WMF, VideoToolbox, Android MediaCodec, system FFmpeg/VA-API on Linux). It uses Cisco's OpenH264 (Cisco pays the royalties, binary downloaded from Cisco) mainly for WebRTC. Open-source Chromium disables H.264/AAC by default. They need `proprietary_codecs=true ffmpeg_branding="Chrome"` plus your own patent licence. | https://blog.mozilla.org/en/mozilla/video-interoperability-on-the-web-gets-a-boost-from-ciscos-h-264-codec/ (2013); https://github.com/chromiumembedded/cef/issues/3559 |
| 6a | Access Advance "exploratory AV1/AV2 pool", Aug 2026 | correct | Announced **25 Aug 2026**, covering devices and software. The quoted sentence is a **paraphrase**. The release actually says the royalty-free label "reflects only the royalty-free commitments made by AOM members" and "Many companies holding patents essential to AV1 or AV2 are not AOM members…" | https://accessadvance.com/2026/08/25/access-advance-launches-exploration-phase-for-an-av1-av2-device-and-software-patent-pool-invites-participation-of-potential-licensors/ |
| 6b | VDP pool covers VP9, AV1, HEVC, VVC | correct | AV2 is expected to be added in 2027. | same |
| 6c | Sisvel AV1/VP9 pools since 2019/2020 | correct | Announced Mar 2019. Patent lists Mar 2020. First licensees 27 May 2020. Device-only royalties (AV1 €0.32 display / €0.11 non-display), no content royalties. | https://www.sisvel.com/insights/sisvel-announces-first-licensees-of-av1-vp9-patent-pools/ |
| 6d | Dolby lawsuits against Snap over HEVC and AV1 | partly | **D. Del. No. 1:26-cv-00317-JLH**, filed 23 Mar 2026: **HEVC** patents (4, amended 8 Jun 2026 to add 5 more). **AV1** claims are in **Brazil** (Rio de Janeiro, Mar 2026). There is a third Brazil HEVC case (8 Jun 2026). Snap reportedly counter-sued Dolby and Access Advance in the UK (unconfirmed). All cases are pending. | https://ipfray.com/dolby-doubles-down-on-snap-third-brazil-complaint-and-amended-u-s-pleadings-add-more-hevc-patents/ |
| 6e | Chrome, Firefox, Safari and Edge all decode AV1 | partly | Chrome, Firefox and Edge decode AV1 in software (dav1d). **Safari decodes AV1 only on hardware with an AV1 decoder** (A17 Pro/M3 and later). | https://bitmovin.com/blog/apple-av1-support |

## UA / compat

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 7a | Ladybird Jan 2026 newsletter UA quote | correct | Verbatim. Ladybird added `Chrome/140.0.0.0` (#7493) and `AppleWebKit/537.36 Safari/537.36` (#7626) and kept "Ladybird". This unlocked modern Google Search, Gmail and Instagram. | https://ladybird.org/newsletter/2026-01-31 |
| 7b | Servo's UA | — | Firefox-shaped: `Mozilla/5.0 (X11; Linux {ARCH}; rv:153.0) Servo/{ver} Firefox/153.0` (with per-OS variants). | https://github.com/servo/servo/blob/main/components/config/prefs.rs |
| 7c | WHATWG HTML issue on UA strings | unverifiable | I did not locate a specific issue. The relevant prior art is Chromium's UA freeze / UA-CH (TAG review #467, 2020). | https://tag-github-bot.w3.org/review/467 |

## iOS

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 8a | "After more than two years, no major browser manufacturer has shipped" | correct (secondary) | This is press wording (pbxscience/SquaredTech). OWA says Apple "has had more than two years". I found no shipped non-WebKit browser. | https://open-web-advocacy.org/blog/28-percent-faster--the-blink-prototype-that-shows-why-apples-ios-browser-engine-ban-must-end/ (18 Jun 2026) |
| 8b | Edge Blink iOS prototype "29% faster in Speedometer 3.1" | partly | **28.6%** (49.27 vs 38.3). iPhone 17 Pro Max, iOS 26.5.1. Single device, research prototype, by Kyle Pflug (Microsoft). | https://www.macrumors.com/2026/06/17/webkit-rule-costs-ios-users-browser-performance/ |
| 8c | Apple requires a separate app | correct | The app must be "distributed solely on iOS and/or iPadOS in the European Union" (so in practice a separate EU-only app). It must also meet the 90% WPT / 80% Test262 bar, among other conditions. | https://developer.apple.com/support/alternative-browser-engines/ |

## Text

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 9a | Parley includes fallback, bidi, shaping, line breaking | correct | Parley 0.12 (Oct 2026) uses **Fontique** for enumeration and fallback, **HarfRust** for shaping (swash was the earlier backend and is now only used in a render example), and **ICU4X** (`icu_segmenter`, `icu_properties`) for segmentation, line breaking and bidi. | https://github.com/linebender/parley ; https://docs.rs/parley/latest/parley/ |
| 9b | IME support | — | Parley's `PlainEditor` has `set_compose` (preedit). Blitz handles winit IME events (`blitz-dom/src/events/ime.rs`). Servo has `ImeEvent::Composition` wired in servoshell. | GitHub code (Oct 2026) |

## Network

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 10a | Let's Encrypt ended OCSP | correct | Announced 5 Dec 2024. OCSP URLs were dropped from certificates 7 May 2025. Responders were shut down **6 Aug 2025**. | https://letsencrypt.org/2024/12/05/ending-ocsp/ |
| 10b | CA/B Forum made OCSP optional | correct | Ballot **SC-063** (Jul 2023). CRLs are mandatory and OCSP optional, effective 15 Mar 2024. | https://cabforum.org/2023/07/14/ballot-sc063v4-make-ocsp-optional-require-crls-and-incentivize-automation |
| 10c | Firefox CRLite | correct | Enabled for all desktop users in **Firefox 137**. | https://hacks.mozilla.org/2025/08/crlite-fast-private-and-comprehensive-certificate-revocation-checking-in-firefox/ |
| 10d | Firefox CT enforcement | correct | Enforced on **desktop from Firefox 135** (Feb 2025), not on Android. | https://groups.google.com/a/mozilla.org/g/dev-platform/c/ueq_33tA_iQ |
| 10e | Chrome CRLSets; Chrome and Safari CT | correct | Chrome uses CRLSets with no online OCSP. Chrome has required CT since Chrome 68 (2018) for certificates issued after 30 Apr 2018. Apple has required CT for certificates issued after 15 Oct 2018, platform-wide. | https://www.digicert.com/blog/apple-announces-certificate-transparency-requirement |
| 11 | Rust networking options | correct | **rustls-platform-verifier** uses OS verifiers and is recommended for shipped apps. **webpki-roots** bundles Mozilla's root list via CCADB (CDLA-Permissive-2.0). **quinn + h3**: h3 is self-described as "still very experimental". **neqo** is Firefox's Rust QUIC/HTTP3 stack and uses NSS for TLS. **hickory-dns** supports DoH, DoT, DoQ and DoH3. | https://github.com/rustls/webpki-roots ; https://github.com/hyperium/h3 ; https://github.com/mozilla/neqo |

## DevTools, Push, Permissions

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 12 | V8 inspector domains vs embedder CDP | correct | V8's `js_protocol.pdl` defines **Runtime, Debugger, Profiler, HeapProfiler (experimental), Console (deprecated), Schema (deprecated)**. The embedder must implement DOM, CSS, Network, Page, Overlay and the rest. The DevTools frontend works JS-only against Node and Deno via `chrome://inspect`. **devtools-frontend is BSD-3-Clause.** | https://chromium.googlesource.com/v8/v8/+/refs/heads/main/include/js_protocol.pdl ; https://github.com/ChromeDevTools/devtools-frontend/blob/main/LICENSE |
| 13 | autopush is self-hostable; Mozilla's hosted service is usable by others?; Chrome uses FCM | partly | autopush (now autopush-rs) is open source under MPL-2.0. The docs frame it around Firefox. **I found no public permission for third-party browsers to use Mozilla's hosted service**, so assume you must self-host. Chrome uses FCM endpoints. | https://mozilla-push-service.readthedocs.io/ ; https://developer.chrome.com/blog/web-push-interop-wins |
| 14 | `<permission>` / `<geolocation>` status | — | The generic `<permission>` (PEPC) ran as an origin trial in Chrome 126–143. **`<geolocation>` shipped in Chrome 144** (blog of 13 Jan 2026). | https://developer.chrome.com/blog/geolocation-html-element |

## Legal / packaging

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 15 | A browser binary is "not subject to export controls" | partly | Publicly available 5D002 source code, and object code compiled from it, is **not subject to the EAR** (§734.3(b)(3), §742.15(b)). The 29 Mar 2021 rule dropped the email notice except for **non-standard cryptography**. A binary containing proprietary or closed components (e.g. a bundled CDM) is not "publicly available". It is typically mass-market 5D992 under §740.17(b)(1), where self-classification reporting was mostly removed in 2021. | https://www.bis.gov/learn-support/encryption-controls/encryption-items-not-subject-to-ear ; https://www.thompsonhine.com/publications/bis-eliminates-most-mass-market-encryption-reporting-obligations |
| 16 | Flatpak vs Chromium-style sandboxes | partly solvable | Firefox has an official Mozilla Flatpak. Its sandbox doesn't depend on user namespaces, so it relies on seccomp and defence-in-depth. Chromium-based Flatpaks use **zypak** (LD_PRELOAD redirection to `flatpak-spawn --sandbox`) or Flatpak's nested-sandbox API. This works, but sub-sandbox strength is disputed and zypak has a single maintainer. Google has no official Chrome Flatpak. | https://github.com/refi64/zypak ; https://discussion.fedoraproject.org/t/secure-firefox-using-native-sandbox-flatpak-flathub-fedora-discussion-clarification/91740/2 |

## Performance

| # | Claim | Verdict | Corrected fact | Source (date) |
|---|---|---|---|---|
| 17a | Cold start 1.4–1.8 s, warm 0.4–0.5 s | unverifiable | One source (single run, M4 MacBook Pro) has cold start to first paint at Chrome 147 = 1.42 s and Firefox 150 = 1.11 s. It gives no warm figures. | https://tech-insider.org/chrome-vs-firefox-2026/ (22 Apr 2026, commercial blog) |
| 17b | Chrome single-tab baseline ~780 MB | unverifiable (likely wrong) | The same source measured **412 MB** (Chrome, Gmail tab) and 338 MB (Firefox). I found no source for 780 MB. | same |
| 17c | Firefox ~18% less RAM per tab | partly | This comes from that single commercial blog (17.9–19.5%). Its methodology is weak, so don't present it as established. | same |
