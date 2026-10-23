# Izcal fonts and text: history and postmortems

Compiled 19 October 2026. Primary sources were preferred (specifications, maintainers' posts, project READMEs, release notes, commit messages, W3C reports and issue trackers). GitHub metadata (creation dates, archive dates, contributor counts) was read through the GitHub API on 19 October 2026.
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, secondary coverage that quotes the primary, an unofficial statement by a former insider, or a vendor claim without data; **Low** = secondary source only, or the researcher's recollection not checked against a source this session.

M-1 note: this file records ideas, measurements and reasons. No source code of HarfBuzz, FreeType, Skia, Fontations or any browser was read. Where it says what a project "does", that is evidence to reason from, not a shape to copy.

**Method notes**
- The web search budget for the session ran out after about 20 searches. Later rows were checked by fetching known primary pages directly or through the GitHub API. Rows that rest on recollection are graded Low and listed under "Open questions".
- Behdad Esfahbod's "State of Text Rendering 2024" was the single richest primary source. It is a maintainer's own account, so dates and adoption claims in it are High; its value judgements are labelled as his.
- The Adobe Type 1 end-of-support page and the Chromium OT-SVG bug both refused automated access. Their claims are Medium.
- Numbers about performance are the projects' own claims unless stated. None were reproduced.

## 0. Chronology at a glance

| Year | Event | Section |
|---|---|---|
| 1984–85 | Adobe Type 1 (PostScript outlines, hints as declarative data) | 1 |
| 1991 | Apple TrueType (quadratic outlines, hinting bytecode VM); Microsoft licenses it | 1 |
| 1996–97 | Microsoft and Adobe end the "font wars": OpenType 1.0 (1997) | 1 |
| 1998 | CSS2 `@font-face`; IE ships EOT, Netscape ships TrueDoc; web fonts then fade | 6 |
| 1998–2001 | ClearType announced and shipped (Windows XP) | 4 |
| 2000s | Uniscribe (Windows), ATSUI → CoreText (macOS 10.5, 2007), Pango, ICU LayoutEngine | 2 |
| 2005 | HarfBuzz started (Behdad Esfahbod) | 2 |
| 2007 | ISO/IEC 14496-22 Open Font Format from OpenType 1.4 | 1 |
| 2009 | DirectWrite (Windows 7); OTS created for Chromium | 2, 5 |
| 2010 | Firefox adopts OTS (MFSA 2010-78); Behdad joins Chrome to win HarfBuzz market share | 2, 5 |
| 2011 | Duqu: TrueType zero-day in the Windows kernel | 5 |
| 2012 | "harfbuzz-ng" rewrite ready for public use; WOFF 1.0 Recommendation | 2, 6 |
| 2012–13 | Colour font split: Apple sbix, Google CBDT, Microsoft COLR, Adobe/Mozilla SVG | 7 |
| 2013 | Adobe contributes its CFF rasterizer to FreeType (stem darkening) | 4 |
| 2014 | Universal Shaping Engine presented; ICU 54 deprecates LayoutEngine; Chrome 37 uses DirectWrite | 2 |
| 2015 | Project Zero finds 16 Windows kernel font bugs; Windows 10 moves font rasterization to user mode | 5 |
| 2016 | OpenType 1.8 Font Variations; FreeType 2.7 v40 interpreter; ICU 58 removes LayoutEngine | 1, 4 |
| 2018 | WOFF2 Recommendation (1 Mar); macOS Mojave drops subpixel AA; IFT work starts | 4, 6 |
| 2020 | rustybuzz repository created | 3 |
| 2021 | Firefox 95 sandboxes Graphite with RLBox; Chromium closes OT-SVG request as WontFix | 5, 7 |
| 2022 | COLRv1 ships in Chrome 98 | 7 |
| 2023 | Adobe ends authoring support for Type 1 fonts | 1 |
| 2024 | Chrome 128 uses Fontations for colour fonts and CFF2; Ladybird switches to HarfBuzz and Skia | 3, 5 |
| 2025 | Chrome 133 uses Fontations for all web fonts (Linux, Android, ChromeOS); HarfRust 0.1.0 | 3, 5 |
| 2026 | FreeType removed from Blink (Chrome 145); rustybuzz archived (26 Jul); IFT Candidate Recommendation Draft (13 Oct) | 3, 5, 6 |

## 1. Outline formats and their era

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **Type 1** (Adobe, mid-1980s) stored cubic outlines plus declarative hints (stems, zones). The rasterizer, not the font, decided how to fit them to pixels. Adobe kept the format and the rasterizer proprietary for years, which is the root of the "font wars". | — | [Microsoft OpenType overview and FAQ](https://learn.microsoft.com/en-us/typography/opentype/) | 1985–1997 | Medium (origin dates from recollection) |
| 1.2 | **TrueType** (Apple, 1991) chose quadratic outlines and put hinting in the font as **bytecode for a stack VM** (fpgm, prep, glyph programs). The reason was the era's constraint: 1-bit output on 72–96 dpi CRTs, where only designer-authored per-size instructions gave legible stems. FreeType's maintainers describe these programs as "designed for black-and-white bitmaps on CRT monitors". | — | [FreeType, "The new v40 TrueType interpreter mode"](https://freetype.org/freetype2/docs/hinting/subpixel-hinting.html) | 1991 (article 2016-09-08) | High (rationale); Medium (1991 date) |
| 1.3 | **OpenType** (Microsoft and Adobe, announced 1996, 1.0 in 1997) is a superset container: TrueType or CFF (Type 1-derived) outlines, plus layout tables. The FAQ states the deal: both companies cross-licensed "font rasterizers, production tools, and conversion software", and Adobe converted its Type 1 library. OpenType 1.4 became ISO/IEC 14496-22 (2007). | 64k glyph limit | [Microsoft OpenType overview](https://learn.microsoft.com/en-us/typography/opentype/) | 1997, 2007 | High |
| 1.4 | **Apple GX / AAT** took the other path: state-machine layout tables (`morx`, `kerx`) where the font carries the shaping logic, and run-time interpolation (GX variations). Behdad's account: GX and Multiple Master interpolation "did not survive" into OpenType until Font Variations in 2016, which reused the GX model. | — | [Esfahbod, "State of Text Rendering 2024"](https://behdad.org/text2024/) | 1994–2016 | High (account); Medium (GX dates) |
| 1.5 | **SIL Graphite** puts programmable rules in the font for scripts OpenType shaped badly (e.g. Nastaliq). Firefox ships it; Chrome and Android do not ship HarfBuzz's Graphite backend, so "adoption is small". Firefox 95 runs Graphite inside an RLBox Wasm sandbox. | — | [text2024](https://behdad.org/text2024/); [Mozilla Hacks, RLBox in Firefox 95](https://hacks.mozilla.org/2021/12/webassembly-and-back-again-fine-grained-sandboxing-in-firefox-95/) | 2021-12-06 | High |
| 1.6 | **End of Type 1**: Adobe ended authoring support for Type 1 fonts in its applications in January 2023. Its stated reasons are that the format is legacy, lacks Unicode and modern features, and that OpenType replaced it. After the March 2020 in-the-wild Type 1 (ATMFD) exploits, Microsoft advised disabling the Type 1 parser. | — | [Adobe Help, Type 1 end of support](https://helpx.adobe.com/fonts/kb/postscript-type-1-fonts-end-of-support.html) (403 to automated fetch); [Microsoft ADV200006](https://msrc.microsoft.com/update-guide/vulnerability/ADV200006) | 2020–2023 | Medium |

**Implications:** Two philosophies of hinting and layout competed: logic in the rasterizer (Type 1 hints, OpenType shaping engines) and logic in the font (TrueType bytecode, AAT state machines, Graphite rules). Fonts that carry programs are why font parsing is a code-execution surface. Izcal must still accept TrueType bytecode and AAT state machines from the web, so both need canonical fuel bounds. The Type 1 exit shows a format can be dropped once its last users have a clear successor; the web never supported Type 1, so Izcal need not either.

## 2. Platform and portable shaping engines

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **Uniscribe** (Windows 2000 era) was Windows' complex-script shaper with per-script engines (Indic, Arabic, etc.). Microsoft's script-development guides still describe the 2005 revision of the Indic model and keep old fonts working, a compatibility promise that shaped every later engine. | — | [Microsoft, Developing OpenType fonts for Devanagari](https://learn.microsoft.com/en-us/typography/script-development/devanagari) | 1999–2005 | Medium |
| 2.2 | **DirectWrite** (Windows 7, 2009) replaced GDI text and Uniscribe for new code. Microsoft's stated reasons: independence from the rendering system (GDI, Direct2D, or a custom renderer), OpenType typography that "they couldn't use in WinForms or GDI", layered APIs (layout, script processor, glyph rendering), and **sub-pixel positioning measured "in ideal resolution space"** so layout is resolution-independent. GDI-compatible measuring modes remain for consistency with old UI. | 5 rendering modes | [Microsoft, Introducing DirectWrite](https://learn.microsoft.com/en-us/windows/win32/directwrite/introducing-directwrite) | 2009 (doc 2018) | High |
| 2.3 | Chrome moved its Windows text from GDI to **DirectWrite in Chrome 37**. | — | [Chromium Blog, Chrome 37 Beta](https://blog.chromium.org/2014/07/chrome-37-beta-directwrite-on-windows.html) | 2014-07-17 | High (title); Medium (GDI framing) |
| 2.4 | Apple's **ATSUI** was succeeded by **CoreText**. Safari is "the only major browser that doesn't use HarfBuzz". | — | [text2024](https://behdad.org/text2024/) | 2007–2024 | High |
| 2.5 | **HarfBuzz** began in 2005 as shaping code split out of FreeType. The rewrite "harfbuzz-ng" was ready for public use around 2012, done with Mozilla's Jonathan Kew. In 2010 Behdad moved from Red Hat to Google's Chrome team "partly to win market share for HarfBuzz", because the Linux desktop alone could not justify investment. By 2013 it had caught up with Microsoft's shaping, including Indic and Myanmar. | 9 CVE records, 8 denial-of-service | [text2024](https://behdad.org/text2024/); [HarfBuzz manual](https://github.com/harfbuzz/harfbuzz/blob/main/docs/usermanual-what-is-harfbuzz.xml) | 2005–2013 | High |
| 2.6 | HarfBuzz adoption: Android, Chrome and Firefox on all platforms, LibreOffice, Pango/GTK, OpenJDK, XeTeX (2013), Emacs 27.1 default, Edge via Chromium (2020), and Adobe Photoshop, Illustrator and InDesign (2019–2023). | — | [text2024](https://behdad.org/text2024/) | to 2024 | High |
| 2.7 | **ICU LayoutEngine** was deprecated in ICU 54 (released 2014-10-01) "in favor of HarfBuzz" and removed in ICU 58 (2016). OpenJDK's JEP 258 gives the reason: it "is no longer actively developed", IBM planned to switch ICU to HarfBuzz, and the JDK "must follow suit, or else be left using an unsupported and out-dated library". A compatibility shim (icu-le-hb) kept the old API on HarfBuzz. | — | [ICU 54 download page](https://icu.unicode.org/download/54); [JEP 258](https://openjdk.org/jeps/258) | 2014–2016 | High (deprecation, JEP); Medium (ICU 58 removal) |
| 2.8 | **Universal Shaping Engine**: per-script engines did not scale to every Brahmi-derived and South-East Asian script. Microsoft's Andrew Glass presented a single table-driven engine at the 2014 Unicode Conference. HarfBuzz implemented it and removed its own SEA shaper. Microsoft's guide covers Unicode 16.0 scripts "that have no dedicated engine". | — | [text2024](https://behdad.org/text2024/); [Microsoft, USE guide](https://learn.microsoft.com/en-us/typography/script-development/use) | 2014–2024 | High |
| 2.9 | **Pango** underpins GTK and uses HarfBuzz on all platforms. It became unmaintained and was picked up by Matthias Clasen. | — | [text2024](https://behdad.org/text2024/) | — | High |

**Implications:** The industry converged on **one portable shaper** because shaping is specification-by-implementation: correctness meant matching Microsoft's engines script by script, and only a single shared implementation could amortise that. Two engines died from lack of maintainers (ICU LE, nearly Pango), not from design flaws. USE shows the right move when per-script code stops scaling: generate behaviour from Unicode property data. For Izcal, the missing artefact is a specification of shaping itself (see 3.6); without one, Izcal's own shaper must be checked against black-box outputs, which M-1 part 4 allows only as a differential signal.

## 3. Rust text stacks and ports

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **rustybuzz** ported HarfBuzz's shaping to Rust, "not a faithful line-by-line port": parsing went to ttf-parser, Unicode to external crates. It lacks subsetting, `mort`, Arabic fallback shaping, avar2, and errors on malformed fonts instead of falling back. Archived 2026-07-26; README points users to HarfRust. | ~23 KLOC (~17 KLOC without generated tables); passes 2221 of 2252 HarfBuzz shaping tests; 1.5–2× slower; matches HarfBuzz 10.1.0 | [rustybuzz README](https://github.com/harfbuzz/rustybuzz) | 2020-01 → 2026-07-26 | High |
| 3.2 | **HarfRust** started (repo 2024-07-15) as a rustybuzz fork moving parsing to Fontations' read-fonts, so skrifa users avoid "shipping (and maintaining) multiple implementations of core font parsing". First release 0.1.0 on 2025-06-10, announced as 2–4× slower than HarfBuzz; 0.2.0 was under 25% slower. Now "completely safe" except bytemuck, matches HarfBuzz 14.5.1, and "for the most part passes the HarfBuzz test and fuzzing suites". | 1,639 commits; <25% slower on common fonts | [HarfRust README](https://github.com/harfbuzz/harfrust); [Esfahbod posts (via hackers.pub)](https://hackers.pub/tags/harfbuzz) | 2024–2026 | High (README); Medium (2–4× figure) |
| 3.3 | **Allsorts** (YesLogic) is a parser, shaper and subsetter for OpenType, WOFF and WOFF2, extracted from the Prince HTML-to-PDF typesetter. Its shaper "was developed in conjunction with a specification for OpenType shaping" (the opentype-shaping-documents project). | repo since 2017-09 | [Allsorts README](https://github.com/yeslogic/allsorts) | 2017–2026 | High |
| 3.4 | **swash** (Chad Brokaw) is a pure-Rust introspection, shaping and rasterization crate that leaves layout and composition to callers. Behdad calls it "largely dormant". Its author now co-maintains Fontations (second-largest contributor) and co-wrote Chrome's Skrifa post. | dfrg: 756 Fontations commits | [swash README](https://github.com/dfrg/swash); [text2024](https://behdad.org/text2024/); [GitHub contributors API](https://github.com/googlefonts/fontations/graphs/contributors) | 2021–2026 | High (README); Medium (identity link) |
| 3.5 | **cosmic-text** (System76, 2022) and **Parley** (Linebender, with Fontique for font selection) are full layout stacks. Behdad credits Rust with "respectable text layout systems". | — | [text2024](https://behdad.org/text2024/); [Parley](https://github.com/linebender/parley); [cosmic-text](https://github.com/pop-os/cosmic-text) | 2021–2026 | High |
| 3.6 | **Ladybird** first used a basic shaping algorithm "inherited from SerenityOS" that "worked for simple scripts like Latin and Cyrillic". In August 2024 it adopted HarfBuzz and added support for the `direction` attribute, so it "can now render more complex scripts like Hebrew and Arabic". In September 2024 it let Skia handle font "decoding, extracting all required metrics, and rasterization". In September 2025 it began caching shaping results, which "drastically improves performance on text-heavy pages like Wikipedia and news sites" (no figure published). | — | [Ladybird newsletter, August 2024](https://ladybird.org/newsletter/2024-08-31/); [Ladybird newsletter, September 2024](https://ladybird.org/newsletter/2024-09-30/); [Ladybird newsletter, September 2025](https://ladybird.org/newsletter/2025-09-30/) | 2024–2025 | High |

**Implications:** Every independent attempt so far either ported HarfBuzz (rustybuzz, HarfRust) or gave up its own shaper for it (Ladybird). The single exception, Allsorts, paired its shaper with a written shaping specification. M-1 forbids the port route, so Izcal is in Allsorts' position and needs the specification route. The rustybuzz numbers set a size floor: about 17 KLOC of hand code for shaping alone, and 31 of 2,252 reference tests still failing after six years. A shaping cache keyed by (font, features, text) is a measured large win even in a young engine.

## 4. Rasterizers and hinting

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **ClearType and DirectWrite**: GDI ClearType used LCD stripes for horizontal resolution but kept whole-pixel advances. DirectWrite (from WPF and Vista) added **sub-pixel positioning** and y-direction AA for large text. "Starting with Windows 8, you should use greyscale text antialiasing in most cases"; grayscale is recommended for Store apps because "it is faster and is compatible with standard antialiasing". | — | [Introducing DirectWrite](https://learn.microsoft.com/en-us/windows/win32/directwrite/introducing-directwrite) | 2009–2012 | High |
| 4.2 | **FreeType v40** (default in FreeType 2.7, 2016): v35 implements the 1990s spec; v38 ("Infinality") copied Microsoft's ClearType hacks with per-font rules and was "painfully slow"; v40 **ignores all horizontal hinting instructions**. Reasons: FreeType lacked Microsoft's compatibility rigs, old fonts snap on both axes and new ones only vertically, and ignoring x-hints keeps advance widths unchanged. Stem darkening is incompatible with x-hinting. | v35 "just as fast" as v40 | [FreeType v40 article](https://freetype.org/freetype2/docs/hinting/subpixel-hinting.html) | 2016-09-08 | High |
| 4.3 | **Stem darkening and gamma**: Adobe contributed its CFF engine to FreeType in 2013, with stem darkening. Coverage must be blended in linear space ("about 186 is needed for true 50% brightness", not 128); linear blending thins text, darkening compensates. FreeType 2.6.2 turned darkening off by default; "slight" (vertical-only) hinting became the recommended mode. | 128 vs 186 | [FreeType, text rendering in 2.6.2](https://freetype.org/freetype2/docs/hinting/text-rendering-general.html) | 2015-11-30 | High |
| 4.4 | **macOS dropped subpixel AA (Mojave, 2018).** A former macOS engineer explained the cost: it needs physical pixel geometry threaded through graphics layers (screen-dependent, multi-monitor), "multiplies your glyph caches: glyph * subpixel offset", and "requires knowing your foreground and background colors at render time, which is an unnatural requirement when you want to do GPU-accelerated compositing". He conceded it looks better on 1× displays. Apple made no public statement. | — | [HN comment by ridiculous_fish](https://news.ycombinator.com/item?id=17477526) | 2018-07-07 | Medium (insider, unofficial) |
| 4.5 | **font-rs** (Raph Levien, 2016) showed a simple dense accumulation-buffer rasterizer (exact signed area, then a prefix-sum pass) beats FreeType: vectors are drawn immediately with no sorting, to "avoid branch misprediction and to better exploit data parallelism". TrueType outlines only, no hinting; code "not ready for prime time"; repo idle since 2020. | — | [font-rs README](https://github.com/raphlinus/font-rs) | 2016 | High |
| 4.6 | **Pathfinder** (Patrick Walton, Servo) was a GPU rasterizer for fonts and vectors; Behdad calls it "the most advanced rasterizer" and "no longer under active development". Commits since 2025 are dependency bumps. It never became Firefox's glyph rasterizer; WebRender rasterizes glyphs on the CPU through platform or FreeType backends into a GPU atlas. | 3.9k stars | [text2024](https://behdad.org/text2024/); [servo/pathfinder](https://github.com/servo/pathfinder) | 2017–2026 | High (status); Low (Firefox path, recollection) |
| 4.7 | **Slug** (Eric Lengyel) renders glyphs on the GPU directly from outlines with an analytic per-pixel coverage solution; it is proprietary. **Vello** (Levien) evolved from a compute-shader renderer to the **Sparse Strips** architecture shared by Vello CPU and Vello GPU, with the original compute renderer moved to `research/`. | — | [text2024](https://behdad.org/text2024/); [Lengyel, JCGT 2017](https://jcgt.org/published/0006/02/02/); [Vello README](https://github.com/linebender/vello) | 2017–2026 | High (Vello); Medium (Slug detail) |

**Implications:** Hinting has moved steadily **out of the font and towards the renderer**: v40 drops x-hints, slight hinting drops everything but vertical snapping, macOS drops hinting and subpixel AA, and Windows recommends grayscale. The drivers were sub-pixel positioning (layout independent of device), GPU compositing (unknown background) and cache size. All three match Izcal's B1 determinism and FP-D14 atlas budgets. Grayscale coverage with linear blending and optional vertical-only snapping is the defensible baseline; running TrueType bytecode is then needed only where a font's outlines depend on it. GPU-only glyph rasterizers (Pathfinder) did not survive in a browser; CPU rasterization into a cached atlas did.

## 5. Font security: from kernel parsing to memory-safe stacks

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | Windows parsed TrueType (win32k.sys) and Type 1/CFF (atmfd.dll) **in the kernel**. Duqu used a TrueType kernel zero-day (2011). Project Zero's 2015 fuzzing found 16 kernel font bugs (7 TrueType: glyf, hmtx, maxp, fpgm, OS/2, EBLC; 9 OTF, mostly CFF), two of which collided with pwn2own 2015 and the Hacking Team leak. Windows 10 moved font rasterization to a restricted user-mode process. | 16 bugs; 5 patch rounds | [Project Zero, "A year of Windows kernel font fuzzing"](https://projectzero.google/2016/06/a-year-of-windows-kernel-font-fuzzing-1_27.html) | 2016-06-27 | High |
| 5.2 | **OTS** was created for Chromium (2009) because system rasterizers "have never been part of the attack surface before", "especially on platforms like Windows, where it's a closed-source blob running with high privilege". It parses and **re-serializes** fonts, dropping what it does not validate. | — | [OTS README](https://github.com/khaledhosny/ots); [WebKit bug 31106](https://bugs.webkit.org/show_bug.cgi?id=31106) | 2009-11 | High |
| 5.3 | Firefox adopted OTS in Firefox 3.5.16 / 3.6.13 to keep downloadable fonts away from OS font code. Its font lead, John Daggett, defended stripping tables because "security concerns will always trump other concerns". This trade (new tables rejected until OTS learns them) is OTS's structural cost: every new format (Graphite, colour tables, variations) needed an OTS update before it worked on the web. | — | [MFSA 2010-78](https://www.mozilla.org/security/advisories/mfsa2010-78); [Daggett, www-font](https://lists.w3.org/Archives/Public/www-font/2010OctDec/0112.html) | 2010-12-09 | High (adoption); Medium (structural cost) |
| 5.4 | **Fontations in Chrome**: FreeType kept producing bugs despite sandboxing, OTS and fuzzing. "Merely keeping up with the stream of issues found by fuzzing costs Google at least 0.25 full time software engineers." New FreeType code for COLRv1 and OT-SVG introduced bugs, fixes often lacked tests, and a high-severity COLRv1 bug took more than 10 months to find. Cited: CVE-2020-15999 (exploited heap overflow). | ≥0.25 FTE; ~700 unit tests; 39 fuzz bugs, none security-critical | [Chrome, "Memory safety for web fonts"](https://developer.chrome.com/blog/memory-safety-fonts) | 2025-03-19 | High |
| 5.5 | Rollout: Chrome 128 (Aug 2024) colour fonts and CFF2; Chrome 133 (Feb 2025) all web fonts on Linux, Android, ChromeOS and as fallback on Windows and macOS; Chrome 136 system fonts on Linux and Android; **FreeType removed from Blink in Chrome 145** (editor's note 2026-04-24). Validation: fauntlet comparisons with FreeType and Skia pixel comparisons across AA and hinting modes; continuous fuzzing since June 2024. The only `unsafe` is byte reinterpretation, delegated to bytemuck. | 4 milestones over ~20 months | [Chrome, "Memory safety for web fonts"](https://developer.chrome.com/blog/memory-safety-fonts) | 2024-08 → 2026 | High |
| 5.6 | Chrome's post notes a limit of fuzzing for fonts: "random mutations rarely produce partially valid structures". | — | [same](https://developer.chrome.com/blog/memory-safety-fonts) | 2025-03-19 | High |

**Implications:** The history is a sequence of moving the parser further from privilege: kernel → user mode (Windows 10), OS rasterizer → sanitizer in front (OTS) → memory-safe parser (Fontations). Each step was forced by exploited bugs, not by design. OTS's sanitize-and-reserialize approach bought safety at the price of rejecting new tables until each was taught; a parser generated from format descriptions with bounded, total functions (frame: VER-1 Tier 1) removes the need for a second, lagging validator. Structure-aware fuzzing is required because random byte fuzzing rarely gets past table headers.

## 6. Web fonts

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | CSS2 (1998) linked fonts from style sheets. Microsoft and Netscape supported only EOT and TrueDoc, not TrueType, "the most widely used font format", so "web fonts disappeared from the designer's toolbox". Håkon Wium Lie argued in 2007 for linking TrueType files directly; objections were "mostly legal". `@font-face` was dropped from CSS 2.1 and returned in CSS Fonts 3. | — | [Lie, "CSS @ Ten", A List Apart](https://alistapart.com/article/cssatten/) | 2007-08-28 | High (article); Medium (2.1 removal) |
| 6.2 | **WOFF2** goals: royalty-free, a spec "that doesn't depend on opaque reference code", better median compression, no large cost in RAM or time. Brotli won over LZMA because LZMA's only documentation was reference code without royalty-free terms, and it decompressed more slowly. | Median gain over WOFF 1.0: 23.9% (Google TTF), 26.8% (Adobe TTF), 13.5% (CFF). Decompression 194.3 MB/s (WOFF 1) vs 61.1 (Brotli); memory median ×1.0, worst ×1.7 | [W3C WOFF 2.0 Evaluation Report](https://www.w3.org/TR/WOFF20ER/) | 2016-03-15 | High |
| 6.3 | WOFF2 became a Recommendation on 2018-03-01 (FPWD 2014-05-08; updated 2021, 2022, 2024-08-08). Its security appendix says decompressed fonts "contain no active code" — untrue for TrueType bytecode and AAT state machines, which is a stale assumption from the format's era. | — | [W3C WOFF2](https://www.w3.org/TR/WOFF2/) | 2014–2024 | High |
| 6.4 | **Incremental Font Transfer** has been in development since 2018 (Adobe's IFTB proposal later merged). It became a Candidate Recommendation Draft on 2026-10-13, with "no implementation report" yet. Reason: a browser must download the whole font before rendering, which hurts CJK most, and `unicode-range` splitting breaks kerning and ligatures across subsets. Patches are Brotli diffs keyed by table or by glyph. | ≤100 invalidating and ≤2,000 total patches per extension | [W3C IFT CRD](https://www.w3.org/TR/IFT/); [text2024](https://behdad.org/text2024/) | 2018 → 2026-10-13 | High |

**Implications:** Web fonts were blocked for a decade by licensing and by inventing web-only formats nobody tooled for; they succeeded once browsers took ordinary OpenType wrapped in a thin container. Every container adds a decoder that is part of the attack surface (Brotli in WOFF2, patch application in IFT). IFT's patch model meets FP-D13 and PRIV-2 head-on: which glyphs a page fetches reveals its text to the font server, and the patch caps are the kind of canonical resource bounds Izcal needs everywhere.

## 7. Colour and variable fonts

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | The colour-font split: Apple sbix (2012, PNG, unstandardised until 2016), Google CBDT/CBLC (2013, PNG for Android emoji), Microsoft COLR (2013, solid layers), Adobe and Mozilla SVG-in-OpenType (2013). | 4 formats in ~2 years | [text2024](https://behdad.org/text2024/) | 2012–2016 | High |
| 7.2 | Chrome and Android "never shipped the SVG solution, for architectural reasons" (it embeds a full SVG renderer in the glyph path). The Chromium feature request was closed WontFix in 2021. Patches enabling it exist outside Google. | — | [text2024](https://behdad.org/text2024/); [mpeg-otspec list, 2023-07](https://lists.aau.at/pipermail/mpeg-otspec/2023-July/003014.html) | 2021 | High (stance); Medium (bug status) |
| 7.3 | **COLRv1** (proposed 2019, in OpenType around 2022) adds gradients, compositing and shape reuse. Shipped in Chrome 98; not in Safari, Adobe flagship apps or Figma. | Noto Emoji: 9 MB bitmap → 1.85 MB COLRv1+WOFF2 | [Chrome, COLRv1 fonts](https://developer.chrome.com/blog/colrv1-fonts); [text2024](https://behdad.org/text2024/) | 2022-01 | High |
| 7.4 | **Variable fonts**: revived 2014, blueprint February 2016, six months of joint work, released as OpenType 1.8 Font Variations (2016). avar2 extends the variation model to designspaces 1.8 cannot express; rustybuzz never implemented it. | — | [text2024](https://behdad.org/text2024/); [rustybuzz README](https://github.com/harfbuzz/rustybuzz) | 2016 → | High |

**Implications:** The colour war ended not by agreement but by each platform shipping its own format and the vector one (COLRv1) winning on size. Features that require embedding another whole engine in the glyph path (OT-SVG needs SVG plus CSS) were refused by the largest engine on architecture grounds. Izcal must decide OT-SVG explicitly; COLRv0/v1, sbix and CBDT are all live on the web.

## 8. Unicode text algorithms in browsers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | Shaping engines do not do bidi or line breaking; when Ladybird adopted HarfBuzz it added `direction` support as separate work in the same month. Browsers run UAX 9 and UAX 14 before shaping, and CSS Text layers its own rules (word-break, line-break strictness) on UAX 14. | — | [Ladybird newsletter, August 2024](https://ladybird.org/newsletter/2024-08-31/); [CSS Text 3](https://www.w3.org/TR/css-text-3/) | 2024 | High (newsletter); Medium (general claim) |
| 8.2 | Thai, Lao, Khmer and Myanmar need dictionary segmentation. ICU4X offers two segmenters: an LSTM model "that needs less data but more CPU" and a dictionary that "needs more data but runs faster". ICU4X exists because ICU4C was too large for constrained targets. | — | [Unicode blog, ICU4X 1.0](https://blog.unicode.org/2022/09/announcing-icu4x-10.html) | 2022-09-29 | High |
| 8.3 | **BudouX** (Google, 2021) is a ~15 KB machine-learned phrase breaker for Japanese, Chinese and Thai, behind Chrome's `word-break: auto-phrase` (Japanese first, Chrome 119). For Korean it recommends `word-break: keep-all`. | ~15 KB model | [BudouX README](https://github.com/google/budoux) | 2021–2023 | High (README); Medium (Chrome 119) |
| 8.4 | Hyphenation: Firefox ships pattern dictionaries in the build; Chrome on Android fetches them as downloadable components; Chrome on macOS uses the OS. The result is platform-dependent `hyphens: auto` output. | — | none checked this session | — | Low |

**Implications:** Segmentation splits into rule-based (UAX 14, 29), dictionary and learned models. Learned models (LSTM, BudouX) trade data size for CPU and are harder to make bit-identical across ISAs (JS-D12 rules on floating point apply). FP-D13 already requires dictionaries in every build, which removes the platform split in 8.4.

## 9. Team sizes and costs

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | FontTools, FreeType and HarfBuzz each have "a lead maintainer, plus one or two active developers and a long tail of contributors" (Cosimo Lupo, Werner Lemberg, Khaled Hosny). FontForge funding "dried up". | 1 lead + 1–2 per project | [text2024](https://behdad.org/text2024/) | 2024 | High |
| 9.2 | HarfBuzz commit counts: Esfahbod 15,677; next three 1,436, 1,103, 912. | ~80% of top-8 commits by one person | [GitHub contributors API](https://github.com/harfbuzz/harfbuzz/graphs/contributors) | 2026-10-19 | High |
| 9.3 | Fontations (repo 2022-02-04): top contributors 963, 756, 412, 392, 346 and 279 commits; about six core engineers, Google Fonts and Chrome. | ~6 core | [GitHub contributors API](https://github.com/googlefonts/fontations/graphs/contributors) | 2026-10-19 | High (counts); Medium (headcount) |
| 9.4 | Effort for a Rust shaper port: rustybuzz took ~6 years to 2,221/2,252 tests and ~17 KLOC; HarfRust took a year from fork to <25% slowdown with Fontations engineers. FreeType fuzz upkeep alone costs ≥0.25 FTE. | see 3.1, 3.2, 5.4 | as cited | 2020–2026 | High |

**Implications:** The world's font stack rests on about a dozen people. A team that owns parsing, shaping, rasterization and hinting is at least the size of Fontations plus HarfBuzz's active core (roughly 6–8 engineers), sustained, before Izcal's extra verification work. This is a planning fact, not a reason to stage down.

## 10. What survived, and why

| Idea | Origin | Survived because | Evidence |
|---|---|---|---|
| OpenType container (sfnt tables) | TrueType 1991, OpenType 1997 | Superset that ended the format war; every tool reads it | 1.3 |
| One portable shaper, matched to Microsoft's behaviour | HarfBuzz 2012 | Shaping is defined by implementation; sharing amortises it | 2.5–2.7 |
| Table-driven shaping from Unicode properties | USE 2014 | Per-script engines did not scale | 2.8 |
| Sub-pixel positioning, grayscale AA, vertical-only snapping | DirectWrite 2009, FreeType 2015–16, macOS 2018 | Device-independent layout, GPU compositing, cache size | 4.1–4.4 |
| CPU rasterization into a GPU atlas | All shipping browsers | Simple, cacheable, portable | 4.6 |
| Memory-safe parsing | Fontations 2024–26 | Exploited bugs despite sandbox, OTS and fuzzing | 5.4–5.5 |
| Brotli-based font containers | WOFF2 2018, IFT 2026 | Royalty-free spec, adequate speed | 6.2, 6.4 |
| Gone: kernel font parsing | Windows win32k, ATMFD | Repeated in-the-wild exploits | 5.1, 1.6 |
| Gone: web-only font formats (EOT, TrueDoc) | 1997–98 | No tooling, licensing deadlock | 6.1 |
| Gone: ICU LayoutEngine, old HarfBuzz | 2000s | No maintainers; a better shared engine | 2.7, 2.5 |
| Gone: subpixel (LCD) AA on macOS | 1998–2018 | Needs background colour and pixel geometry at raster time | 4.4 |
| Gone: GPU-only glyph rasterizer in a browser | Pathfinder | Never displaced the CPU path; development stopped | 4.6 |

## Lessons for Izcal

Ideas only; no design is copied.
- **Write the shaping specification first.** The only independent shaper that did not end as a HarfBuzz port (Allsorts) was built with a written shaping specification. Izcal's shaper should be anchored to an explicit, testable specification of OpenType, AAT and USE behaviour, checked against other engines only as a black-box signal.
- **One parser, generated and total.** OTS (a second validator in front of a fragile parser) lagged every new table. Fontations replaced the parser instead. Izcal should have one font reader, generated from table descriptions, used by shaping, rasterization and validation alike, so nothing needs re-serializing.
- **Fonts carry programs.** TrueType bytecode, AAT state machines, Graphite rules and OT-SVG all run font-supplied logic. Each needs canonical fuel and memory bounds, as IFT's patch caps do. WOFF2's "no active code" claim must not be inherited.
- **Make hinting optional, not central.** The field moved to grayscale, sub-pixel positioning and vertical-only snapping; this also fits B1 and the atlas budgets. Subpixel LCD AA conflicts with compositing and multiplies caches, so it should be excluded unless a first-principles case appears.
- **Cache shaping results** keyed by (font, features, text); it is a measured large win.
- **Plan for a team, not a person.** Every surviving component is held up by one lead and one or two others. Izcal owning all of it implies a sustained team of at least 6–8.
- **Decide formats explicitly**: Type 1 (no), OT-SVG (needs a decision), COLRv0/v1, sbix, CBDT, variations including avar2, IFT.

## Open questions and flagged claims

- **Type 1 end-of-support details** (Adobe date and wording; any Windows removal date) were not read; the Adobe page refused automated access (1.6).
- **Chromium OT-SVG WontFix** rests on a mailing-list report; the bug's own closing comment was not read (7.2).
- **WebRender glyph path** and Pathfinder's non-adoption in Firefox are from recollection (4.6).
- **Hyphenation per browser** (8.4) and **Chrome 119 auto-phrase** (8.3) are unverified.
- **ICU 58 removal** of LayoutEngine is from an OpenJDK list post, not ICU release notes (2.7).
- **TrueType 1991, Type 1 1985, ClearType 1998–2001, CoreText 2007** dates are from recollection.
- **Not covered for lack of budget:** UAX 9 browser bug history, old HarfBuzz (2006–2012) internals, Apple's AAT support in HarfBuzz, Firefox's hyphenation dictionary format, Slug's patent status.
