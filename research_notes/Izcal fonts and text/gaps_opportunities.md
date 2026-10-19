# Izcal fonts and text: landscape, gaps, system font access and cost

Compiled 19 October 2026. Primary sources were preferred (HTTP Archive data, specifications, vendor documentation, maintainers' posts, issue trackers, and Izcal's own measurements).
Confidence: **High** = primary source read and the claim checked in its text, or measured by this researcher; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only, or not found in the page that was read.

**Scope.** Phase 3 of the first-principles-spec method for the text stack: how the web uses fonts today, what developers struggle with, what tools show and hide, how each OS exposes its fonts, what cross-OS consistency costs, and what text stacks have cost the people who built them. It is evidence only (M-1). No other engine's or library's source code was read; libraries appear only through their maintainers' public statements and repository metadata.

**Method notes.**
- The shared web-search budget ran out after 3 searches. Everything else came from direct fetches of known primary pages (about 30) and from measurements.
- Four rows are **measurements** made for this note on 19 October 2026: Google Fonts CSS slices (fetched with a desktop Chrome user agent and counted), the sizes of the 124 Noto Sans JP slices (HTTP `Content-Length`), the macOS font directories and downloadable-font catalog on this Mac (macOS 27.0.1), and Android's `fonts.xml` (AOSP main branch). GitHub repository metadata (creation dates, contributor counts, Noto CJK release sizes) came from the GitHub REST API on the same day.
- The Web Almanac pages are longer than the fetch window, so their final 12–15% (conclusions) was not read.
- Questions left unverified for lack of budget are listed in section 11, not used as findings.

## 1. How the web uses fonts (HTTP Archive)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | Web fonts are nearly universal. In the 2025 crawl, 88% of sites use them, up from 87% in 2024 | 88% (2025), 87% (2024) | [Web Almanac 2025 Fonts](https://almanac.httparchive.org/en/2025/fonts); [2024 Fonts](https://almanac.httparchive.org/en/2024/fonts) | 2026-01-15; 2024-11-11 | High |
| 1.2 | **Self-hosting is rising and Google Fonts is falling.** 72% of sites self-host at least one font; 31.5% desktop and 36.4% mobile self-host only. Google Fonts appears on 54% desktop and 47% mobile (57% and 48% in 2024) | self-host-only 22%→28%→31.5% desktop (2022→2024→2025) | same | 2026-01-15 | High |
| 1.3 | WOFF2 dominates but older formats persist. 2024: WOFF2 on 81% desktop and 78% mobile sites, WOFF 8–10%, TTF 6% of self-hosted. 2025 reports WOFF2 at 65% and WOFF at 16%, which is probably a different denominator (requests versus sites); 5–6% of sites serve fonts as `application/octet-stream` | 81% vs 65% (unexplained) | same | 2024–2026 | Medium (the two years conflict) |
| 1.4 | Font files are small at the median and large in the tail: median ~35–39 KB, 90th percentile ~112–116 KB. The median Google Fonts WOFF2 is ~16 KB against ~39 KB self-hosted, which suggests subsetting matters | p50 35–39 KB; p90 112–116 KB | same | 2024–2026 | High |
| 1.5 | **Variable fonts are mainstream**: 39.4% desktop and 41.3% mobile sites in 2025 (28–29% in 2022). Over 99% of variable fonts use `glyf` outlines; CFF2 is ~0.6%. Noto CJK families are ~42% of desktop variable-font sites | wght axis in ~99%; opsz/wdth 2–3% | same | 2024–2026 | High |
| 1.6 | **Color fonts are rare on the web** (0.04% of sites in 2024, 0.05–0.06% in 2025), but four formats are live: SVG 53%, COLRv0 28%, sbix 13%, COLRv1 8%, CBDT 3% (2024 desktop). The same chapter notes that excluding fonts with under 5% of code points mapped to colour glyphs cuts SVG to about 5%, leaving COLRv0 25%, sbix 12%, COLRv1 7%, CBDT 3% (fact-check FC-M3). Noto Color Emoji is the top color font | ~6,000 sites | [2024 Fonts](https://almanac.httparchive.org/en/2024/fonts) | 2024-11-11 | High |
| 1.7 | **Icon fonts are a large share of web fonts**: ~18% of web fonts (2024). Font Awesome is on ~8.5–9.3% of pages, and the top 10 `font-display: block` families are all icon fonts | ~17% of pages use FA with `block` (2025) | 2024 and 2025 chapters | 2024–2026 | High |
| 1.8 | `font-display`: `swap` on ~50% of pages (11% in 2020, 30% in 2022), `block` ~25%, `auto` ~9%, `fallback` ~5%, `optional` ~0.5% | swap 50.1% mobile | [2025 Fonts](https://almanac.httparchive.org/en/2025/fonts) | 2026-01-15 | High |
| 1.9 | `system-ui` appears on ~10% desktop and ~9% mobile pages; `text-wrap: balance` ~2.5–2.7%, `pretty` ~1.7%, `hyphens: auto` ~10.5% | — | same | 2026-01-15 | High |
| 1.10 | 55% of fonts carry OpenType features (liga 40%, kern 38%, locl 27%); `font-feature-settings` use is falling (10.3% desktop) while `font-variant` rises (4.5%). 70–77% of sites set non-standard font-smoothing properties | — | [2024 Fonts](https://almanac.httparchive.org/en/2024/fonts) | 2024-11-11 | High |

**Implications:** a conforming engine must handle WOFF2, WOFF, raw TTF/OTF and mislabelled MIME types; `glyf` variable fonts are the common case and CFF2 the rare one, so CFF2 is a correctness requirement but not a performance one. Four color formats are live, so "COLRv1 only" is not an option; by real colour use (excluding fonts with few colour glyphs) COLRv0 leads and SVG is about 5% (fact-check FC-M3). Icon fonts with `font-display: block` are a quarter of `block` use, so block-period behaviour and Private Use Area code points matter more than their typographic weight suggests.

## 2. Scripts on the web and how CJK fonts are delivered

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | Content languages (share of sites with known language): English 49.5%, Japanese 4.9%, Russian 3.4%, Chinese 1.3%, Persian 1.0%, Vietnamese 0.9%, Korean 0.9%, Arabic 0.6%, Hebrew 0.4%, Thai 0.3%, Hindi <0.1%. Site counts, not readers: they under-represent large-population scripts | ~50% non-English | [W3Techs content languages](https://w3techs.com/technologies/overview/content_language) | 2026-10-19 | High |
| 2.2 | Web fonts covering scripts (2025, a font "supports" a script at >5% of its characters): Latin ~54%, Private Use Area ~54%, Cyrillic 12–13%, Greek 7–8%, Hebrew ~1.2%, Katakana/Hiragana ~1.3–1.4%, Hangul ~1%, Han ~0.4%. "Chinese web font use remains near zero, mainly due to file size" (2024) | Han 0.4% | [2025 Fonts](https://almanac.httparchive.org/en/2025/fonts); [2024 Fonts](https://almanac.httparchive.org/en/2024/fonts) | 2024–2026 | High |
| 2.3 | **Most non-Latin text is therefore drawn with OS or fallback fonts, not web fonts.** The engine's system-font and fallback path decides how most CJK, Indic, Arabic and Thai pages look | inference from 2.1–2.2 | — | — | Medium (inference) |
| 2.4 | **Google Fonts slices CJK into ~100–124 `unicode-range` faces.** Measured: Noto Sans JP 124 `@font-face` blocks, Noto Sans SC 101, Noto Sans KR 124, Roboto 9. The 124 JP slices total 2,935,108 bytes (2.8 MiB), 4.4–49 KB each | 124 slices; 2.8 MiB | Measured: [Google Fonts CSS for Noto Sans JP](https://fonts.googleapis.com/css2?family=Noto+Sans+JP&display=swap) | 2026-10-19 | High (measured; varies by user agent) |
| 2.5 | Full CJK fonts are large. Noto CJK release archives: Sans `.ttc` 90.8 MiB (zip), Sans per-language variable 25–48 MiB, Serif `.ttc` 132 MiB, Serif OTF set 662 MiB; one face per region (JP, KR, SC, TC, HK). Licensed **SIL OFL 1.1** | 90–660 MiB per archive | [noto-cjk releases (GitHub API)](https://github.com/notofonts/noto-cjk/releases); [Sans LICENSE](https://github.com/notofonts/noto-cjk/blob/main/Sans/LICENSE) | 2022-01-27 (Sans 2.004); 2024-07-30 (Serif 2.003) | High |
| 2.6 | Noto covers "more than 1,000 languages and over 150 writing systems"; Noto Sans and Noto Serif each have "more than 200,000 glyphs" | 150+ scripts | [notofonts.github.io](https://notofonts.github.io/); [State of Text Rendering 2024](https://behdad.org/text2024/) | 2024–2026 | High |

**Implications:** the frame's "all scripts render" requirement is carried mostly by system and fallback fonts, so FP-D13 (identical data in every build) and the footprint spec's session image (6.3) meet a hard number here: a pan-script, two-style CJK set is hundreds of MiB, against FP-25's 24 MiB placeholder for web-observable data. `unicode-range` faces with 100+ slices per family are the common CJK web-font shape, so font matching must treat a family as a set of many faces and the load path must handle 100+ small requests per family efficiently.

## 3. What developers struggle with

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Font swaps cause layout shifts** because fallback and web font occupy different space. The fix is `size-adjust` plus `ascent-override`, `descent-override` and `line-gap-override`, computed by hand from average character width and font metrics. Mac and Windows read different vertical tables (hhea, typo, win), so ~10% of Google Fonts need OS-specific overrides | ~90% same on Mac and Windows | [Improved font fallbacks (Chrome)](https://developer.chrome.com/blog/font-fallbacks) | updated 2023-02-10 | High |
| 3.2 | Frameworks now compute these overrides at build time (Next.js 13 `next/font`, Nuxt 3 `@nuxtjs/fontaine`, Fontaine, a Google Fonts metrics repository). The browser exposes none of the numbers it uses, so tools reimplement font-metric reading | 4 tools | same | 2023-02-10 | High |
| 3.3 | **Text box height is uncontrollable without new CSS.** `text-box-trim` (Chrome 133, Safari 18.2) exists because "every font produces a different amount of this block directional space", which "has been impossible to control, until now" | — | [CSS text-box-trim (Chrome)](https://developer.chrome.com/blog/css-text-box-trim) | 2025-01-14 | High |
| 3.4 | Interop 2025 text-related focus areas: unprefixed `text-decoration`, and writing modes (`sideways-lr`, `sideways-rl`). Fonts and line height are not a focus area | 2 of ~19 areas | [Interop 2025 README](https://github.com/web-platform-tests/interop/blob/main/2025/README.md) | read 2026-10-19 | High |
| 3.5 | State of CSS 2025's free-text pain points name no typography item in their top lists; "browser support" and "Safari" lead (41 and 14 mentions of 419 answers) | ~5,500 respondents | [State of CSS 2025 usage](https://2025.stateofcss.com/en-US/usage/) | 2025 | Medium (top items only; text answers not read) |
| 3.6 | 70–77% of sites set non-standard `-webkit-font-smoothing`/`-moz-osx-font-smoothing`, which developers use to make macOS text look the same as elsewhere | 70–77% | [2024 Fonts](https://almanac.httparchive.org/en/2024/fonts) | 2024-11-11 | High |

**Implications:** the recurring struggle is not exotic typography but **unknown metrics**: which font is used, what its ascent, descent and average width are, and how much it shifts on swap. These are facts the engine has and does not expose. Surveys do not rank fonts as a top pain, so Izcal's case for text visibility should rest on CLS and metrics evidence (3.1–3.3), not on survey rank.

## 4. What developer tools show about fonts today

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | Firefox's Fonts tab lists fonts used by the selected element (grouped by family) and all fonts on the page, labels each font "System" or with its web-font URL, includes "system fallback fonts used when nothing from the font-family CSS declaration has been applied", and edits variable-font axes and named instances | — | [Firefox: Edit fonts](https://firefox-source-docs.mozilla.org/devtools-user/page_inspector/how_to/edit_fonts/index.html) | read 2026-10-19 | High |
| 4.2 | It does **not** document which characters were drawn by which font, why a fallback was chosen, or any shaping or line-break reason | — | same | read 2026-10-19 | High (absence in the docs) |
| 4.3 | Chrome DevTools: a "Rendered Fonts" list in the Computed pane (family, local or network, glyph count) is known from the product but was not found in the current CSS reference; a Font Editor in the Styles pane was listed as experimental in Chrome 89 | — | [CSS features reference](https://developer.chrome.com/docs/devtools/css/reference); [What's new in DevTools](https://developer.chrome.com/blog/new-in-devtools-85) | read 2026-10-19 | Low (Rendered Fonts) / Medium (Font Editor) |
| 4.4 | No tool documentation read shows the `font-display` period state (block, swap, failure) per face, the fallback chain walked for a character, the shaping features applied, or the UAX 14 rule that produced a break | — | 4.1–4.3 | read 2026-10-19 | Medium (absence in the pages read) |

**Implications:** today's tools answer "which fonts are on this element" at family granularity. They do not answer "which font drew this glyph and why", "why did this line break here", or "when did this face swap". Those are engine facts an owned stack produces anyway.

## 5. System font access per OS

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **Windows installs many script fonts on demand.** Supplemental fonts are packaged by script as Features on Demand; "most are added automatically by Windows Update when the associated languages are enabled", or manually under Optional features. Packages include Arabic, Bangla, Devanagari, Chinese (Simplified, Traditional), Japanese, Korean, Hebrew, Thai, Tamil, Telugu and Pan-European | ~24 FOD font packages | [Windows 10 font list](https://learn.microsoft.com/en-us/typography/fonts/windows_10_font_list) | updated 2024-03-18 | High |
| 5.2 | DirectWrite's system fallback is `IDWriteFontFallback::MapCharacters` ("determines an appropriate font to use to render the beginning range of text"), from `IDWriteFactory2::GetSystemFontFallback` (Windows 8.1+); `IDWriteFactory3::GetSystemFontSet` (Windows 10) lists system fonts | — | [IDWriteFontFallback](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_2/nn-dwrite_2-idwritefontfallback); [GetSystemFontSet](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_3/nf-dwrite_3-idwritefactory3-getsystemfontset) | 2018–2025 | High |
| 5.3 | **macOS has three font tiers.** Measured on this Mac (macOS 27.0.1): `/System/Library/Fonts` 564 MB (85 top-level files plus 289 in `Supplemental`, 129 MB), `/Library/Fonts` 540 MB (156 files, machine-wide installs), and an on-demand catalog (`com_apple_MobileAsset_Font8`) listing **363 downloadable font assets totalling ~1,009 MiB**, of which 1.8 MB were present locally | 363 assets; ~1 GiB | Local measurement (`du`, asset catalog XML) | 2026-10-19 | High (measured, one machine) |
| 5.4 | The system UI font on macOS is San Francisco (`SFNS.ttf` and others). Its license allows use "solely for creating mock-ups of user interfaces" on Apple OSes, forbids embedding, and excludes "website content". The web reaches it only by name (`system-ui`, `-apple-system`) | — | [Apple Fonts](https://developer.apple.com/fonts/) | read 2026-10-19 | High |
| 5.5 | **Linux: fontconfig is a configuration and matching library, not an OS service.** It reads XML (`/etc/fonts/fonts.conf`, `conf.d`, `$XDG_CONFIG_HOME/fontconfig`), applies `<alias>` with `prefer`/`accept`/`default`, filters with `<acceptfont>`/`<rejectfont>`, caches per-architecture binary files, and returns the closest match: "a font will always be returned, but doesn't ensure that it is anything like the requested pattern" | cache format v8 | [fontconfig user docs](https://www.freedesktop.org/software/fontconfig/fontconfig-user.html) | v2.15.0 | High |
| 5.6 | **Android: `fonts.xml` is deprecated** as a source; vendors use `font_fallback.xml`, and apps are told to call `SystemFonts#getAvailableFonts` or the `ASystemFontIterator` NDK API. Its matching rule: full BCP-47 tag with script, then language, then file order (first font containing the glyph); order also breaks weight ties. Measured: 161 families (149 with `lang`), 419 font entries, 24 aliases | 161 families | Measured: [AOSP fonts.xml](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/data/fonts/fonts.xml) | read 2026-10-19 | High |

**Implications:** the set of fonts an OS can offer is not fixed: Windows and macOS both install fonts when the user adds a language or an app asks, so a fallback answer can change while a session runs (the Firefox reports in 6.1–6.2 show the cost of enumerating them). This supports the footprint spec's sealed per-session font image (6.3): snapshot what exists at session start, and treat later OS changes as recorded B1 inputs. San Francisco cannot ship with Izcal, so `system-ui` must resolve to the OS's own font by reading it in place, never by bundling it. On Linux, fontconfig's rules are configuration *data* an Izcal font service can read itself (M-1 permits reading OS data), and its matching is engine work Izcal would do itself.

## 6. Fallback, enumeration, fingerprinting and start-up cost

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Firefox on Windows: a ~2.1 s stall at session restore was traced to `GetSystemFontCollection` taking over a second in a content process (2 ms on the developer's machine), suspected to be a DirectWrite cache failure that repeated on each launch. Closed INCOMPLETE | >1 s vs 2 ms | [Bugzilla 1766908](https://bugzilla.mozilla.org/show_bug.cgi?id=1766908) | 2022 | High |
| 6.2 | Firefox: delayed `InitFontList` ~127–136 ms on Windows, 311–345 ms under a sandboxing tool (bug 1661041); main thread idle ~140 ms waiting for the font-list thread on macOS, fixed in Firefox 120 (bug 1819275); hangs up to 23 s reading face names on Windows 11 (bug 1923941) | 0.13–23 s | [1661041](https://bugzilla.mozilla.org/show_bug.cgi?id=1661041); [1819275](https://bugzilla.mozilla.org/show_bug.cgi?id=1819275); [1923941](https://bugzilla.mozilla.org/show_bug.cgi?id=1923941) | 2020–2024 | Medium (search summaries) |
| 6.3 | A Chromium review warned that DirectWrite waiting on the Windows FontCache service could add significant per-process start-up cost | — | [codereview 209163002](https://codereview.chromium.org/209163002) | 2014 | Medium (search summary) |
| 6.4 | **WebKit hides user-installed fonts**: font availability is limited "to only include web fonts and fonts that come with the operating system" | — | [WebKit tracking prevention](https://webkit.org/tracking-prevention/) | read 2026-10-19 | High (date not on page) |
| 6.5 | Tor Browser: "Font-based fingerprinting MUST be rendered ineffective". It bundles Noto fonts and allows only a whitelist; on Linux it ships its own `fonts.conf`; allowing OS fonts on Windows and macOS "is currently a compromise" | — | [Tor Browser design](https://2019.www.torproject.org/projects/torbrowser/design/) | 2019 snapshot | High |
| 6.6 | Local Font Access (`queryLocalFonts`, Chrome 103 desktop): permission-gated, returns full SFNT data, sorted list to avoid order fingerprinting; the article itself calls the font list "a highly fingerprintable surface". Safari and Firefox do not ship it | — | [Local Font Access](https://developer.chrome.com/docs/capabilities/web-apis/local-fonts) | 2020-08-24 | High |

**Implications:** asking the OS at the moment a page needs a font is slow (a second or more in bad cases), non-deterministic (the answer depends on caches, installs and language packs) and a fingerprinting channel. Three browsers converged on restricting the visible set (WebKit: OS-bundled only; Tor: bundled Noto plus a whitelist; Chrome: a permission). This is independent evidence for D-S1 and the footprint spec's session image (6.3): build the font list once, off the critical path, filtered to an OS-bundled or Izcal-bundled set.

## 7. Cross-OS consistency and deterministic text

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | Playwright states that "browser rendering can vary based on the host OS, version, settings, hardware", power source and headless mode, and that screenshots differ by "rendering, fonts and more". Its only remedy: run tests "in the same environment where the baseline screenshots were generated" | — | [Playwright visual comparisons](https://playwright.dev/docs/test-snapshots) | read 2026-10-19 | High |
| 7.2 | WPT avoids real fonts for layout tests with **Ahem**: square em, ascent plus descent equal to the em, baseline at 0.8 em, most ASCII glyphs a 1×1 em square. It still recommends font sizes that are multiples of 5 px or "baseline alignment may be rendered inconsistently" | — | [WPT: the Ahem font](https://web-platform-tests.org/writing-tests/ahem.html) | read 2026-10-19 | High |
| 7.3 | WPT reftests allow `<meta name=fuzzy content="maxDifference;totalPixels">`, justified by anti-aliasing differences | — | [WPT reftests](https://web-platform-tests.org/writing-tests/reftests.html) | read 2026-10-19 | High |
| 7.4 | Skia's GM tests use a portable typeface so that tests "detect when rendering is different across platforms and configurations" rather than font differences | — | [Skia: fonts and GM tests](https://skia.org/docs/dev/testing/fonts/) | read 2026-10-19 | Medium |
| 7.5 | Chrome checks its new Rust font scaler against the old one with a comparison tool (fauntlet) plus pixel tests: correctness of a font stack is established differentially | ~700 unit tests | [Memory safety for web fonts](https://developer.chrome.com/blog/memory-safety-fonts) | 2025-03-19 | High |
| 7.6 | Mac and Windows choose different vertical metric tables for the same font (3.1), so the same web font produces different line boxes per OS | ~10% of Google Fonts | [Improved font fallbacks](https://developer.chrome.com/blog/font-fallbacks) | 2023-02-10 | High |

**Implications:** the industry's answer to cross-OS text differences is avoidance: one fixed environment, a test-only font, or a pixel tolerance. Nobody ships identical text output across operating systems. With its own parser, shaper, metrics and rasterizer, and fonts read as bytes rather than through OS scalers, Izcal can make identical bytes in give identical glyph positions and identical pixels on every OS and ISA. The remaining OS-dependent input is *which* fonts exist, which the footprint spec's sealed session image (6.3) already makes a recorded input. "Identical screenshots across OSes for the same font set" would be a real advantage for developers.

## 8. Cost evidence: who builds text stacks, and how long they took

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | The open text stack is a handful of people: FreeType, HarfBuzz and FontTools "each have a lead maintainer, plus one or two active developers"; the three are maintained by three people | ~3 core maintainers | [State of Text Rendering 2024](https://behdad.org/text2024/) | 2024-07-08 | High |
| 8.2 | **Years to maturity:** HarfBuzz started 2005; the rewrite was usable ~2012 and "fairly complete" then; the Universal Shaping Engine came in 2014, after which "no new shapers have been needed". FreeType dates from the 1990s and lists 109 CVE records; HarfBuzz 9 (8 denial of service) | ~7–9 years to complete shaping | same | 2024-07-08 | High |
| 8.3 | HarfBuzz repository: 285 contributors; the founder has 15,677 commits, the next four 500–1,436 each. One person wrote most of the code | 285 contributors | GitHub API: [harfbuzz/harfbuzz](https://github.com/harfbuzz/harfbuzz) | 2026-10-19 | High |
| 8.4 | Fontations (Google's Rust font stack): repository created 2022-02-04, 39 contributors, top five 346–963 commits. Skrifa replaced FreeType for web fonts on Linux, Android and ChromeOS in Chrome 133 (February 2025), for system fonts there in Chrome 136, and FreeType left Blink in Chrome 145 | ~3 years from repo to default | [fontations](https://github.com/googlefonts/fontations); [Memory safety for web fonts](https://developer.chrome.com/blog/memory-safety-fonts) | 2025-03-19 (+2026-04-24 note) | High |
| 8.5 | Keeping up with fuzzer findings in FreeType cost Google "at least 0.25 full time software engineers"; a high-severity COLRv1 bug "took more than 10 months to find"; Skrifa's fuzzing since June 2024 found 39 bugs, none security-critical | 0.25 FTE; 39 bugs | same | 2025-03-19 | High |
| 8.6 | Rust shapers: rustybuzz repository created 2020-01-12 (23 contributors), HarfRust 2024-07-15 (38), Allsorts 2017-09-04 (20). ICU4X repository created 2020-02-27 | — | GitHub API: [rustybuzz](https://github.com/harfbuzz/rustybuzz), [harfrust](https://github.com/harfbuzz/harfrust), [allsorts](https://github.com/yeslogic/allsorts), [icu4x](https://github.com/unicode-org/icu4x) | 2026-10-19 | High (dates) / Medium (what they show) |
| 8.7 | Three OpenType implementations matter in practice: DirectWrite, Core Text, and FreeType plus HarfBuzz | 3 | [State of Text Rendering 2024](https://behdad.org/text2024/) | 2024-07-08 | High |

**Implications:** a complete shaper took a gifted small team about 7–9 years; a funded, focused Google team took about 3 years to ship a Rust scaler as the default for web fonts, with an existing implementation to compare against at every step. Izcal will have no reference library inside its build (M-1), but it may run other engines as black-box differential signals (M-1 part 4), which is what made Fontations' pace possible. Plan for a dedicated text team, not a side task. No source gives a team-size number for "a text stack"; this is unknown.

## 9. What Izcal could expose that no browser does

| # | Idea | Evidence it matters | What Izcal has that makes it possible |
|---|---|---|---|
| 9.1 | **Per-glyph provenance**: for any text range, the face that drew each glyph, the fallback step chosen (author list, `unicode-range` slice, language rule, system fallback, last resort) and why | Firefox stops at the family (4.1–4.2); most non-Latin text uses fallback (2.3) | Own matcher; spec anchors to CSS Fonts 4 §5 matching steps |
| 9.2 | **Metrics as data**: ascent, descent, line gap and average width as used, plus computed `size-adjust` and overrides that would remove a swap's layout shift | Developers compute these in build tools (3.1–3.2) | Own parser and metrics; replay can measure a swap's shift exactly |
| 9.3 | **Font load timeline**: per face, request, block and swap periods, swap time, and the layout shift each swap caused | `font-display` on most pages (1.8); CLS from swaps (3.1) | B1 virtual clock; B4 mutation log ties a swap to its relayout |
| 9.4 | **Break and bidi explanations**: the UAX 14 rule or dictionary decision behind a break, and bidi levels and visual order | No tool shows them (4.4) | Own Unicode algorithms with spec anchors to UAX 14 and UAX 9 rules |
| 9.5 | **Shaping trace**: OpenType features and lookups applied per run | No tool documents it (4.2) | Own shaper |
| 9.6 | **Identical cross-OS text output** for the same font set | Playwright, WPT, Skia all work around OS differences (7.1–7.4) | Own rasterizer and metrics; DET-1 |
| 9.7 | **Text extraction with logical order for agents and assistive tech**: logical-order text, language and direction runs, and glyph-to-character mapping from the semantic plane | Agents and screen readers read DOM text, not what was drawn | B8 semantic plane; B9 agent principal |

**Privacy limit.** Ideas 9.1 and 9.2 show which local fonts exist, so in page-visible APIs they must report the per-page font view (footprint 6.3, PRIV-2), and the full view only to the local DevTools user.

## 10. What is invisible today

| Gap | Evidence | What an owned stack can do |
|---|---|---|
| Which font drew which glyph | Firefox lists fonts per element only (4.1–4.2); Chrome lists rendered families (4.3) | Glyph-level provenance with the matching step, anchored to CSS Fonts 4 |
| Why a fallback was chosen | No tool documents it (4.4); OS fallback APIs return a font, not a reason (5.2, 5.5) | Izcal's own fallback order, recorded per decision |
| Metric overrides that would remove CLS | Developers compute them in Next.js, Nuxt and Fontaine (3.2) | Show the measured shift and suggest exact `size-adjust` and overrides |
| When each face blocked, swapped or failed | `font-display: swap` on ~50% of pages and `block` on ~25% (1.8); no timeline in tools (4.4) | Font-load events on B1's virtual clock, joined to the layout shifts they caused |
| Why a line broke here | No tool (4.4) | UAX 14 rule ID, dictionary word or `text-wrap` decision per break |
| Shaping decisions | No tool (4.2) | Per-run feature and lookup trace |
| Why text looks different on another OS | Vendors advise using one environment (7.1) | Same output everywhere; differences come only from the font set, which is visible |
| Which system fonts a page may see | WebKit, Tor and Chrome each restrict differently (6.4–6.6) | Show the page its sealed per-page font view and the policy that produced it |

## 11. Implications for Izcal (ideas only)

- **Ship a deterministic default font set.** Most non-Latin text depends on fallback (2.3), OS sets differ and change at run time (5.1, 5.3), and enumeration is slow and fingerprintable (6.1–6.6). An Izcal-bundled, OFL-licensed pan-script set (Noto-class coverage, 2.5–2.6) would give identical fallback on every OS and close the font-list fingerprint. Its size is the main cost: CJK alone is tens to hundreds of MiB, which tests FP-25's 24 MiB placeholder and FP-D13. This is a trade-off the spec must settle with numbers, not a default to assume.
- **Treat OS fonts as data read once.** Read the OS's font files and configuration (DirectWrite collections, the macOS directories, fontconfig XML, Android's `font_fallback.xml`) in the strictest-tier utility process at session start, never at first paint, and seal the result (footprint 6.3). Font installs mid-session become recorded B1 inputs.
- **Make text facts a product feature.** The ideas in section 9 come for free from owning the stack, and need only stable, versioned event schemas, as the JS engine note concluded for deopts.
- **Use differential testing as the maturity accelerator.** Fontations' pace came from comparing against FreeType at every step (7.5, 8.4). Izcal can do the same against other engines as black boxes (M-1 part 4) and against Unicode's conformance files.
- **Design for `unicode-range` at scale.** 100+ slices per CJK family (2.4) means face sets, request coalescing and per-character slice selection are hot paths.
- **Do not plan to support San Francisco except by name** (5.4).

## 12. Unverified and open items

- **Windows per-user fonts** (installable without admin rights since Windows 10 1809, under `%LOCALAPPDATA%\Microsoft\Windows\Fonts`) and **`IDWriteFactory6::GetSystemFontSet(includeDownloadableFonts)`**: known, but not checked in a primary source this session.
- **Core Text**: `CTFontCreateForString` as the macOS fallback API, font registration scopes, and font-change notifications on every OS were not checked.
- **Chrome's font fallback IPC cost** and Chrome's per-OS fallback path: only the 2014 review in 6.3 was found.
- **WOFF2 share**: 81% (2024) against 65% (2025) is unexplained (1.3).
- **ICU4X segmenter**: the README read gives no data or LSTM model sizes, so none are quoted.
- **Puppeteer and Chromium `--font-render-hinting`** as a screenshot-stability flag: not checked.
- **Text-stack team size**: no source gives a number; 8.1–8.5 are proxies.
- **Linux distributions shipping Noto by default**, and the size of a full Noto install: not measured.
