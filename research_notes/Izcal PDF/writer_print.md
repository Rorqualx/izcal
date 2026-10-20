# Izcal PDF: writer_print

Compiled 19 October 2026. Primary sources were preferred (specifications, maintainers' posts, release notes, issue trackers, product documentation) and black-box measurement of shipping browsers.
Confidence: **High** = primary source read and the claim checked in its text, or measured in this session; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only, or the researcher's recollection.

M-1 note: this file records ideas, dates, measurements and stated reasons. No source code of any browser, PDF engine or PDF library was opened. Crates are named only with their licence (from the crates.io registry metadata) and README prose. Browser behaviour comes from release notes, blog posts, bug-tracker prose and black-box tests: HTML pages were printed with Chrome 155 headless and with WebKit through the macOS `WKWebView` API, and the output files were inspected as bytes with a small Python script plus Poppler's `pdfinfo`, `pdffonts` and `pdftotext` used only as black-box test oracles (Poppler is excluded from shipping). Where a value is stated only in an engine's source, it is listed in section 11 as a question for black-box measurement.

**Method notes**
- About 30 web searches and fetches were used, then the session's shared search budget ran out. Rows left without a primary read are graded Medium or Low and repeated under "Claims not verified".
- Black-box rig: `local` (this Mac, macOS 27.0.1), light work only. Chrome **155.0.8059.80** headless (`--print-to-pdf`, `--no-pdf-header-footer`). WebKit output came from `WKWebView.createPDF` and `WKWebView.printOperation` (save-to-file disposition); this is the system WebKit Safari uses, but Safari's own "Export as PDF" menu path was not driven, so Safari rows are graded Medium unless they say "WebKit API".
- Test pages: a probe page (headings, external and internal links, Latin/Greek/Cyrillic/Japanese text, Menlo, emoji, `filter: blur`, `box-shadow`, `mix-blend-mode: multiply`, gradient, opacity, an 800×800 JPEG, a 400×400 PNG, inline SVG, a table and a list); a 200-section text document with `@page` margin boxes; a variable-font page (SF Pro variable at weights 150, 420, 880, plus the Skia variable font).
- Firefox was not measured: it has no documented headless print-to-file command line. Firefox rows rest on Bugzilla prose.

## 0. Chronology at a glance

| Year | Event | Section |
|---|---|---|
| 1993-2006 | PDF 1.0 to 1.7; Type 3, TrueType (FontFile2) and CFF (FontFile3) embedding; object streams and xref streams in PDF 1.5 (2003) | 5, 7 |
| 2005 | PDF/A-1 (ISO 19005-1), PDF 1.4 based, no transparency | 4 |
| 2011-2012 | PDF/A-2 (ISO 19005-2, PDF 1.7 based); PDF/A-3 (ISO 19005-3, any embedded file) | 4 |
| 2012, 2014 | PDF/UA-1 (ISO 14289-1), revised 2014 | 3 |
| 2015-09-01 | SOURCE_DATE_EPOCH spec 1.0 (Reproducible Builds); 1.1 on 2017-11-27 | 6 |
| 2016-2017 | pdfTeX adds SOURCE_DATE_EPOCH, `\pdftrailerid`, `\pdfinfoomitdate` | 6 |
| 2017 | PDF 2.0 (ISO 32000-2); revised 2020 | 5, 6 |
| 2020-07-29 | Chrome announces tagged PDF by default from Chrome 85 | 3 |
| 2020 | PDF/A-4 (ISO 19005-4, PDF 2.0 based); LibreOffice 7.0 PDF/UA export (experimental) | 4, 3 |
| 2020 | PDF Days: PDF 2.0 supports neither OpenType Variable nor OpenType SVG fonts | 5 |
| 2024 | PDF/UA-2 (ISO 14289-2); LibreOffice 24.2 PDF/A + PDF/UA together | 3 |
| 2024-10-30 | Chrome 131 ships `@page` margin boxes with `counter(page)` / `counter(pages)` | 8 |
| 2025-10-24 | Typst 0.14: tagged by default, PDF/UA-1, all PDF/A parts, export on krilla | 3, 4 |
| 2026-05-26 | Firefox bug 2034406 (`[skpdf]` missing selectable text) verified fixed: Firefox moving print-to-PDF to a Skia PDF backend on Linux and macOS | 2 |

## 1. What Chromium's print-to-PDF produces (black-box, Chrome 155)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | The producer string is **"Skia/PDF m155"**; the Creator is the full user-agent string. The header says **PDF 1.4**. Default page size in headless mode is US Letter. | 612×792 pt | Black-box: `pdfinfo` on Chrome 155 output | 2026-10-19 | High |
| 1.2 | Output is **tagged by default**: `/MarkInfo`, `/StructTreeRoot`, `/Lang (en)` from the `lang` attribute. Structure types seen: Document, H1, H2, P, L, LI, Lbl, Table, TR, TH, TD, Figure (with `/Alt` from `alt`), Link, NonStruct. | 801 StructElems in a 64-page document | Black-box byte inspection | 2026-10-19 | High |
| 1.3 | Chrome writes **no XMP metadata stream** and therefore no `pdfuaid` or `pdfaid` identification. The output does not claim PDF/UA or PDF/A conformance. | Metadata Stream: no | Black-box: `pdfinfo` | 2026-10-19 | High |
| 1.4 | **Document outline** is off by default and appears with `--generate-pdf-document-outline`: entries came from h1/h2 headings. | 0 outlines default; 3 entries with flag | Black-box | 2026-10-19 | High |
| 1.5 | **Link annotations**: both the external `https://` link (`/URI`) and the in-page `#sec2` link (`/Dest`) became `/Link` annotations. | 2 links, 2 annotations | Black-box | 2026-10-19 | High |
| 1.6 | **JPEG passthrough**: the original JPEG file bytes appear unchanged in the PDF as a `/DCTDecode` image (800×800, `ICCBased` colour space), even though it was displayed at 400 CSS px. | 82,457 B in, identical bytes out | Black-box: byte search for the source file | 2026-10-19 | High |
| 1.7 | **PNG is re-encoded** as a Flate image at source resolution with an ICC colour space; the Flate stream (275,927 B) is larger than the source PNG (203,551 B). | +36% | Black-box | 2026-10-19 | High |
| 1.8 | **Filters and shadows are rasterized**: `filter: blur()` and `box-shadow` became image XObjects (one 729×291 DCT-compressed grey image, one 689×267 RGB image with a soft mask), so blurred regions are lossy raster at roughly device scale. `mix-blend-mode: multiply` stayed **vector** as an ExtGState `/BM /Multiply`; opacity became a transparency group. | 2 raster images for 2 effects | Black-box | 2026-10-19 | High (which image maps to which effect is inferred from sizes, Medium) |
| 1.9 | **Fonts**: system TrueType fonts are embedded as **CIDFontType2 subsets** (`FontFile2`, Identity-H, ToUnicode). The CJK fallback font (PingFang, from a `.ttc` collection) and **variable fonts** (SF Pro at three weights, Skia) were embedded as **Type 3** fonts. No `FontFile3` (CFF) was emitted. | 6 FontFile2, 4 Type 3 on the probe page | Black-box: `pdffonts` | 2026-10-19 | High |
| 1.10 | **Colour emoji** (Apple Color Emoji, `sbix` bitmaps) became **Type 3 glyphs that draw 66×66 RGB images with soft masks**; `pdftotext` recovered no text for them. | Emoji not extractable | Black-box | 2026-10-19 | High |
| 1.11 | **Subset tags are sequential, not hashed**: AAAAAA+, BAAAAA+, CAAAAA+… in order of first use. The same font gets a different tag per use site (three Helvetica-Bold subsets). | 10 font objects on a 2-page probe | Black-box | 2026-10-19 | High |
| 1.12 | **Text extraction order** follows paint order, not logical order: `pdftotext` placed the sentence's final full stop before the Japanese run that came from a fallback Type 3 font. Variable-font text (Type 3 with ToUnicode) extracted correctly. | — | Black-box | 2026-10-19 | High |
| 1.13 | Chrome writes **no object streams and no xref stream** (classic xref table). | 0 ObjStm | Black-box | 2026-10-19 | High |
| 1.14 | The Chromium announcement says tags were enabled by default for "Save as PDF" from **Chrome 85** (stable late August 2020), headless needs `--print-to-pdf` with `--export-tagged-pdf` at the time, and a Chrome 84 flag `chrome://flags/#export-tagged-pdf` preceded it. Tags cover headings, lists, tables, paragraphs and image descriptions. | Chrome 85 | [Chromium blog, "Using Chrome to generate more accessible PDFs"](https://blog.chromium.org/2020/07/using-chrome-to-generate-more.html); [9to5Google](https://9to5google.com/2020/07/29/chrome-tagged-pdf/) | 2020-07-29 | Medium (post body not retrieved; details via press coverage) |
| 1.15 | Press coverage reports that CommonLook advised Google and that tag quality "is highly dependent on the underlying HTML"; iText noted Chrome did not claim PDF/UA. | — | [Search Engine Journal](https://www.searchenginejournal.com/chrome-85-tagged-pdf/376157/); [iText](https://itextpdf.com/es/node/14686) | 2020 | Low |

**Implications:** the reference browser's writer is vector-first but rasterizes every filter and shadow, falls back to Type 3 for anything that is not a plain static TrueType (collections, variable fonts, colour emoji), and loses emoji text. Izcal can beat it measurably on three axes: instanced variable fonts as real CID fonts, emoji with ToUnicode (ActualText), and logical reading order from the structure tree.

## 2. WebKit (Safari) and Firefox print-to-PDF

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | WebKit's PDF is written by the OS: producer **"macOS Version 27.0.1 (Build 26A434) Quartz PDFContext"**, header PDF 1.4. `createPDF` gives one tall page (800×1121 pt); `printOperation` paginates to Letter. | 2 pages paginated | Black-box: WebKit API | 2026-10-19 | High |
| 2.2 | WebKit output is **untagged**: no StructTreeRoot, no MarkInfo, **no link annotations** and no outline on either API path. | 0 links of 2 | Black-box: WebKit API | 2026-10-19 | High (Safari's menu path not driven: Medium for Safari) |
| 2.3 | **JPEG is decoded and re-encoded as Flate at display resolution** (800×800 source became a 400×400 Flate image of 279,861 B, 3.4× the 82,457 B JPEG). PNG was downsampled to display size too (200×200). All images carry `/Interpolate true`. | 3.4× larger | Black-box: WebKit API | 2026-10-19 | High |
| 2.4 | WebKit rasterizes blur and shadow (image + soft mask), keeps `multiply` as vector, and draws **emoji as 32×32 images** (not text). | 32×32 px emoji | Black-box | 2026-10-19 | High |
| 2.5 | Quartz embeds simple **TrueType subsets with MacRoman/WinAnsi encodings**, many **without ToUnicode**; CJK went out as a Type 1 (CFF-derived) font. Extraction mapped 日 (U+65E5) to the Kangxi radical ⽇ (U+2F47): a wrong Unicode mapping. | 1 wrong code point of 3 | Black-box: `pdffonts`, `pdftotext` | 2026-10-19 | High |
| 2.6 | Quartz **instances variable fonts** into static TrueType subsets named with the axis values (e.g. `.SFNS-Regular_wdth_opsz140000_GRAD_wght960000`, `Skia-Regular_wghtC810_wdth`). | 4 instances, 13.6 KB file | Black-box | 2026-10-19 | High |
| 2.7 | Quartz writes a **random `/ID` per run**; two runs of the same page produce files of identical length that differ in bytes. | 420,309 B both runs | Black-box | 2026-10-19 | High |
| 2.8 | Apple asked in 2022 whether Safari/Preview PDFs carry tags or the PDF/UA flag; PDFKit documentation does not mention tagging. Apple's iWork 8.2 apps do emit tagged PDF (cannot be turned off). | — | [Apple Developer Forums 713855](https://developer.apple.com/forums/thread/713855); [PDF Association, "Apple tags PDF"](https://pdfa.org/apple-tags-pdf/) | 2022 | Medium |
| 2.9 | Firefox is moving Save-to-PDF and printing to a **Skia PDF backend ("skpdf")**, behind `print.experimental.skpdf` (default on in Nightly). Bug 2034406 "[skpdf] PDFs generated from pdf.js (via MozPrintCallback) are missing selectable text" was filed 2026-04-23 and verified fixed 2026-05-26 in Printing: Output. | Firefox 153 fix | [Mozilla bug 2034406 (REST)](https://bugzilla.mozilla.org/rest/bug/2034406) | 2026-05-26 | High |
| 2.10 | A Mozilla developer in bug 2050228 says the Cairo win32 backend is "still how Firefox prints" on Windows, including to Microsoft Print to PDF; skpdf is on Linux and macOS. Press reports say Firefox 158 beta ships skpdf for "more accessible PDFs". | — | [Mozilla bug 2050228](https://bugzilla.mozilla.org/show_bug.cgi?id=2050228); [LinuxToday](https://www.linuxtoday.com/blog/mozilla-firefox-158-enters-public-beta-testing-with-better-pdf-handling/) | 2026 | Medium (press article refused fetch; bug text from search snippet) |
| 2.11 | Older Firefox reports describe print-to-PDF pages turned into **low-resolution images**, text not selectable (bug 1906126, Firefox 127 Flatpak; workaround: "Simplified" mode). On Linux, output historically passed through PostScript and Ghostscript in some setups. | — | [Mozilla bug 1906126](https://bugzilla.mozilla.org/show_bug.cgi?id=1906126); [bug 329886](https://bugzilla.mozilla.org/show_bug.cgi?id=329886) | 2006-2024 | Medium |
| 2.12 | A blogger reports a 400+ page Chrome print producing a 100 MB+ PDF through the system dialog while Firefox and Safari stayed under 10 MB; Chrome's own "Save" button was faster and much smaller. | 100 MB vs <10 MB | [flaviocopes, "How to print your HTML with style"](https://flaviocopes.com/css-printing.md) | n.d. | Low (anecdote) |

**Implications:** no shipping browser writes PDF/UA, and two of three engines hand the job to a general 2D library's PDF backend (Skia) or the OS (Quartz). The writer's quality is therefore set by a drawing-API abstraction that loses structure, link targets or text mappings. Izcal's writer should consume the display list plus the accessibility tree directly, not a generic canvas.

## 3. Tagged PDF and PDF/UA output in other producers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Typst 0.14** (2025-10-24) writes **tagged PDF by default**, with tags chosen automatically from built-in elements; `--no-pdf-tags` disables it and blocks accessible PDF/A levels and PDF/UA. | — | [Typst PDF reference](https://typst.app/docs/reference/pdf/); [Typst 0.14 changelog](https://www.typst.app/docs/changelog/0.14.0) | 2025-10-24 | High |
| 3.2 | Typst's PDF/UA-1 mode **refuses to produce a PDF** when it finds critical issues such as missing alt text, and shows what to fix. The post says "printing to PDF will result in an untagged file" (of other tools). | — | [Haug, "Accessible PDF" (Typst blog)](https://typst.app/blog/2025/accessible-pdf) | 2025-10-31 | High |
| 3.3 | Typst supports **PDF/UA-1 only** (PDF 1.7 based). PDF/UA-1 is incompatible with PDF 2.0 and with PDF/A-4; PDF/UA-2 is not supported. Maths needs manual alt text. | — | [Typst PDF reference](https://typst.app/docs/reference/pdf/) | 2026 | High |
| 3.4 | **Prince** tags automatically for PDF/A-1a/2a/3a and PDF/UA-1, or with `--tagged-pdf`; tagged output uses PDF 1.5+ with object streams on by default; `-prince-pdf-tag-type` and `-prince-pdf-role-map` tune tags. Prince says tagging alone does not make a document WCAG-compliant. | — | [Prince, PDF profiles](https://www.princexml.com/doc/pdf-profiles/) | 2026 | High |
| 3.5 | **WeasyPrint** offers `--pdf-tags` (off by default) and variants `pdf/ua-1` and `pdf/ua-2`, but warns that output is not guaranteed valid and must be checked. | — | [WeasyPrint API reference](https://doc.courtbouillon.org/weasyprint/stable/api_reference.html) | 2026 | High |
| 3.6 | **LibreOffice** exposed PDF/UA export in 7.0 (2020; reports conflict on whether it was experimental-only); tagging fixes followed in 7.4.4 and 7.5.0 (tables, background artifacts, figure bounding boxes). LibreOffice 24.2 can write PDF/A and PDF/UA-1 together, but not PDF/A level "a". | — | [LibreOffice bug 135638](https://bugs.libreoffice.org/show_bug.cgi?id=135638); [PDF Association, "PDF/A and PDF/UA together at last"](https://pdfa.org/pdf-a-pdf-ua-together-at-last/) | 2020-2024 | Medium (pdfa.org page refused fetch) |
| 3.7 | Chrome's tags come from the page's semantics (the blog presents it as an accessibility feature; quality depends on HTML). Measured: Chrome emits `/Alt` on figures and `Lang`, but not the PDF/UA identification, so it cannot pass a PDF/UA validator as-is (row 1.3). | — | Rows 1.2-1.3, 1.14 | 2026-10-19 | High (measured part) |
| 3.8 | PDF/UA-1 was published as ISO 14289-1 in 2012 and revised in 2014; PDF/UA-2 (ISO 14289-2, PDF 2.0 based) was published in 2024. | — | Researcher's recollection | 2012-2024 | Low |

**Implications:** the bar set by document tools (Typst, Prince) is "tagged by default, PDF/UA on request, refuse or warn on violations". A browser can derive tags from the accessibility tree it already builds, so Izcal can make PDF/UA-identified output a falsifiable target (veraPDF or the PAC checker as test oracles), which no browser meets today.

## 4. PDF/A: requirements, emitters, cost

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | PDF/A profile matrix as Prince documents it: PDF/A-1a/1b on PDF 1.4; 2a/2b and 3a/3b on PDF 1.7; "a" levels need tagging; all need **embedded fonts**, device-independent colour with an **output intent**, and **no encryption**; 3a/3b allow attachments; audio and video are forbidden in all PDF/A and PDF/X. | — | [Prince, PDF profiles](https://www.princexml.com/doc/pdf-profiles/) | 2026 | High (Prince's summary also lists "no transparency" for 2/3, which conflicts with 4.2) |
| 4.2 | Typst notes **PDF/A-1b forbids transparency**; PDF/A-2a and 3a forbid Unicode Private Use Area code points; attachments in PDF/A-3 and PDF/A-4f; 4e equals 4f when there is no 3D. Typst supports 1b, 1a, 2b, 2u, 2a, 3b, 3u, 3a, 4, 4f, 4e. | 11 profiles | [Typst PDF reference](https://typst.app/docs/reference/pdf/) | 2026 | High |
| 4.3 | **PDF/A-4** (ISO 19005-4:2020) is based on PDF 2.0 and drops the a/b/u levels for profiles 4, 4f, 4e; the output intent is optional when colour is device-independent; XMP must carry `pdfaid:part` (and level where used). | 3 profiles | [PDFlib, technical concepts in PDF/A](https://pdflib.com/pdf-knowledge-base/pdfa/technical-concepts); [Wikipedia, PDF/A](https://en.wikipedia.org/wiki/PDF/A) | 2020 | Low (secondary; sources disagree on PDF/A-4 tagging and Unicode rules) |
| 4.4 | Emitters: **Typst** (all parts), **Prince** (1-3), **WeasyPrint** (`pdf/a-1b` to `pdf/a-4f`, unchecked), **LibreOffice** (b/u levels; with PDF/UA since 24.2), **krilla** (PDF/A-1 to A-4 "validated export modes"). No browser emits PDF/A: Chrome writes no XMP at all (row 1.3). | — | Rows 3.1-3.6; [krilla README](https://github.com/LaurenzV/krilla) | 2026 | High |
| 4.5 | **Cost to support** follows from what Izcal's writer must already do: embed and subset every font (needed anyway for fidelity), ToUnicode for every glyph (needed for search and PDF/UA), an sRGB output intent ICC profile (a single standard data file), XMP serialisation, a trailer `/ID`, no encryption, and a validator in CI. The incremental items are XMP, the output intent, transparency flattening for PDF/A-1 only, and refusing features (JavaScript, audio/video, external references). | — | Derived from 4.1-4.3 | 2026-10-19 | Medium (analysis) |

**Implications:** PDF/A-2b/2u/4 cost little once fonts, ToUnicode and determinism are done right; PDF/A-1 costs transparency flattening, which Izcal should decline (†) unless a user need appears. PDF/A-2a/3a combined with PDF/UA-1 is the profile set the tagged writer naturally reaches.

## 5. Font embedding and subsetting

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | PDF font programs: TrueType outlines go in **FontFile2** (simple TrueType or **CIDFontType2**); bare CFF goes in **FontFile3** with `/Subtype /Type1C` or **`/CIDFontType0C`**; a whole OpenType font can go in FontFile3 `/Subtype /OpenType` (PDF 1.6). | — | ISO 32000-1 §9.9 (researcher's recollection) | 2008 | Medium |
| 5.2 | A **subset font's name** is a tag of exactly six uppercase letters, a plus sign, then the PostScript name; the spec does not say how the letters are chosen. Chrome numbers them sequentially (AAAAAA, BAAAAA); Quartz uses AAAAAB, AAAAAC… (rows 1.11, 2.5). Tag choice is free, so a content hash makes it deterministic and lets identical subsets dedupe. | 6 letters A-Z | ISO 32000-1 §9.6.4 (recollection); black-box | 2008 / 2026-10-19 | Medium (spec) / High (measured) |
| 5.3 | **OS/2 fsType**: bits 0-3 are usage permissions with valid values 0 (installable), 2 (restricted: must not be embedded without permission), 4 (preview & print: document opened read-only), 8 (editable). Bit 8 (0x0100) = **no subsetting**; bit 9 (0x0200) = **bitmap embedding only**, and with no bitmaps the font is unembeddable. Embedders "must not embed fonts which are not licensed to permit embedding". | 0/2/4/8; 0x0100; 0x0200 | [OpenType 1.9.1 OS/2 table, fsType](https://learn.microsoft.com/en-us/typography/opentype/spec/os2#fstype) | 2024-05-29 | High |
| 5.4 | For OS/2 **versions 0-2**, if more than one of bits 0-3 is set the **least-restrictive permission** takes precedence (e.g. bits 2 and 3 both set); from **version 3** (OpenType 1.4) the bits must be mutually exclusive. | — | [OpenType OS/2 version differences](https://learn.microsoft.com/en-us/typography/opentype/spec/os2#fstype) | 2024 | High |
| 5.5 | **Variable fonts and CFF2**: PDF 2.0 (ISO 32000-2) supports neither OpenType Variable nor OpenType SVG fonts; the producer must synthesise a static instance per used design-space location and embed that (CFF or TrueType), as Adobe's former principal scientist describes. | 0 native support | [PDF Association, Isaacs, "OpenType Variable and OpenType SVG Fonts in PDF"](https://www.pdfa.org/presentation/opentype-variable-and-opentype-svg-fonts-in-pdf/) | 2020 | Medium (presentation abstract) |
| 5.6 | Two shipping strategies for variable fonts, measured: Chrome emits **Type 3** glyph procedures per instance (row 1.9); Quartz emits **instanced static TrueType** subsets (row 2.6). Both kept text extractable when ToUnicode was present. | — | Black-box | 2026-10-19 | High |
| 5.7 | **Colour fonts** (COLR/CPAL, sbix, CBDT, SVG) have no native PDF font form; producers use **Type 3** fonts whose glyphs paint vectors or images (Chrome: 66×66 images in Type 3) or plain images outside any font (WebKit: 32×32). Neither recovered emoji text. COLRv0/v1 glyphs can be expressed as vector Type 3 procedures with gradients; sbix/CBDT only as images. | — | Black-box; COLR mapping is analysis | 2026-10-19 | High (measured) / Medium (analysis) |
| 5.8 | The `subsetter` crate (Typst) subsets "OpenType fonts with TrueType or CFF outlines", does **not** instance variable fonts (listed as future work), does not mention CFF2, and warns subsets are "unusable in any other contexts than PDF writing". | MIT OR Apache-2.0 | [subsetter README](https://github.com/typst/subsetter) | 2026 | High |
| 5.9 | **ToUnicode** must map each glyph ID to the Unicode text it represents, including ligatures (one glyph → several code points) and glyphs reached through shaping. Quartz produced a wrong mapping for 日 (row 2.5) and Chrome lost emoji (row 1.10): ToUnicode is a common correctness gap. PDF also offers `/ActualText` in marked content for cases a CMap cannot express. | — | Black-box; ISO 32000-1 §9.10, §14.9.4 (recollection) | 2026-10-19 | High (measured) / Medium (spec) |
| 5.10 | WeasyPrint exposes `--full-fonts` ("Embed unmodified font files when possible") and `--hinting` (keep hinting in embedded fonts), both off by default: subsetting and hint stripping are the default size trade-off. | — | [WeasyPrint API reference](https://doc.courtbouillon.org/weasyprint/stable/api_reference.html) | 2026 | High |

**Implications:** Izcal's writer needs its own subsetter for TrueType and CFF, its own variable-font instancer (glyf deltas and CFF2 → CFF conversion), an fsType gate with the version-dependent precedence rule, hashed subset tags, and ToUnicode plus ActualText from the shaper's cluster map. Colour emoji should go out as Type 3 vector glyphs (COLR) with ToUnicode, a measurable win over both engines.

## 6. Determinism and reproducible output

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **SOURCE_DATE_EPOCH**: seconds since 1970-01-01 00:00:00 UTC, ASCII integer; build tools "MUST use this variable for embedded timestamps in place of the 'current' date and time"; other build-specific timestamps must be clamped to it; malformed values should fail. | spec 1.0 2015-09-01; 1.1 2017-11-27 | [Reproducible Builds, SOURCE_DATE_EPOCH](https://reproducible-builds.org/specs/source-date-epoch/) | 2017-11-27 | High |
| 6.2 | **pdfTeX**: with SOURCE_DATE_EPOCH set, CreationDate, ModDate and the trailer `/ID` seed come from it, which "by itself should suffice to create reproducible PDFs". `\pdftrailerid{}` omits the `/ID`; `\pdfinfoomitdate` omits the dates; `\pdfomitinfodict` (1.40.25) drops `/Info`; the `/ID` seed uses only the base file name since 2017; `FORCE_SOURCE_DATE=1` also drives `\year`, `\time` (discouraged). | — | [pdfTeX NEWS](https://tug.ctan.org/systems/doc/pdftex/NEWS); [pdftex(1)](https://man.archlinux.org/man/pdftex.1) | 2016-2023 | Medium (search snippets of primary pages) |
| 6.3 | **Typst** takes `--creation-timestamp=UNIX_TIMESTAMP` or SOURCE_DATE_EPOCH for the document's creation date. Its PDF documentation says nothing else about determinism. | — | [typst-compile(1)](https://man.archlinux.org/man/extra/typst/typst-compile.1.en); [Typst PDF reference](https://typst.app/docs/reference/pdf/) | 2026 | Medium |
| 6.4 | **Chrome**: two runs on the same page gave identical length; the **only** differing bytes were `/CreationDate` and `/ModDate` (2 bytes differed). Chrome writes **no trailer `/ID`** and has no flag to fix the date. | 2 bytes differ | Black-box | 2026-10-19 | High |
| 6.5 | **Quartz (WebKit)**: a random `/ID` per run (row 2.7); bytes differ even at equal length. | — | Black-box | 2026-10-19 | High |
| 6.6 | **WeasyPrint** takes `--pdf-identifier` for the `/ID`; attachment created/modified dates default to "current date and time", so output varies unless set. | — | [WeasyPrint API reference](https://doc.courtbouillon.org/weasyprint/stable/api_reference.html) | 2026 | High |
| 6.7 | What breaks determinism, from the rows above and general practice: wall-clock dates in `/Info` and XMP, random or path-seeded `/ID`, hash-map iteration order deciding object numbers or resource names, subset tags from counters that depend on traversal order, compressor settings or library versions (zlib level and implementation change Flate bytes), absolute file paths, multithreaded encoding order, and locale-formatted dates. | — | Analysis from 6.1-6.6 | 2026-10-19 | Medium |
| 6.8 | PDF 2.0 requires the trailer `/ID`; PDF/A requires it too. Chrome's omission is legal for PDF 1.4 but not for a PDF/A or PDF 2.0 claim. | — | ISO 32000-2 §14.4, ISO 19005 (recollection) | 2017/2020 | Medium |

**Implications:** Izcal's B1 determinism goal maps cleanly onto the writer: `/ID` = hash of the content with a fixed seed, dates from a caller-supplied epoch (SOURCE_DATE_EPOCH honoured in headless/agent mode, real date for a user's Save), hashed subset tags, deterministic object order and a pinned Flate implementation. A byte-identical-output test across x86-64, arm64 and riscv64 becomes a falsifiable target.

## 7. Size and speed

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | Chrome 155, probe page (2 pages, 82 KB JPEG + 204 KB PNG + effects + emoji): **463,753 B** in **1.3-1.7 s** wall time including headless browser start-up. | 463,753 B | Black-box on `local` | 2026-10-19 | High |
| 7.2 | Chrome 155, 200 sections of Latin text: **64 pages, 174,070 B, 0.9 s** wall including start-up (~2.7 KB per page, tagged). | 64 pp, 174 KB | Black-box | 2026-10-19 | High |
| 7.3 | WebKit API, same probe: **420,309 B** (one tall page) and 418,148 B paginated; JPEG re-encoding to Flate inflated the photo 3.4× (row 2.3). | 420 KB | Black-box | 2026-10-19 | High |
| 7.4 | Neither Chrome nor Quartz uses **object streams or xref streams** (PDF 1.5 features) in these files; Prince turns object streams on by default for tagged output. Structure-tree-heavy tagged PDFs are many small dictionaries, which is where object streams save most. | 0 ObjStm | Black-box; [Prince](https://www.princexml.com/doc/pdf-profiles/) | 2026-10-19 | High |
| 7.5 | WeasyPrint's size levers: `--optimize-images` (lossless), `--jpeg-quality` (0-95), `--dpi` (max image resolution), `--uncompressed-pdf` for debugging. | — | [WeasyPrint API reference](https://doc.courtbouillon.org/weasyprint/stable/api_reference.html) | 2026 | High |
| 7.6 | **JPEG passthrough** (Chrome) keeps the original bytes and quality; **decode and re-encode** (Quartz) costs size and, for a lossy target, quality. Passthrough needs care with CMYK/Adobe-inverted JPEGs, EXIF orientation and the image's colour profile. | 1.0× vs 3.4× | Black-box; analysis | 2026-10-19 | High / Medium |

**Implications:** a size target per page type (text page, image page) and a time target for a 100-page document are measurable now against Chrome as baseline. JPEG passthrough, object streams and xref streams are cheap wins; the costs that matter are raster fallbacks for effects, which Izcal should minimise by expressing blur-free effects as vector and choosing raster resolution per effect (†).

## 8. Paged media, `window.print()` and events

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **Chrome 131** (article 2024-10-30) supports `@page` **margin boxes**: 16 at-rules (`@top-left-corner` … `@bottom-right-corner`), generated content, `counter(page)` and `counter(pages)`. The browser may still add its own header/footer when there is room; zero margins suppress them. | 16 boxes | [Chrome for Developers, "Add content to the margins of web pages when printed using CSS"](https://developer.chrome.com/blog/print-margins) | 2024-10-30 | High |
| 8.2 | Chrome lists **`string-set` / `string()` (running headers) as "future possibilities"**; `element()` running elements are not mentioned. | — | [Chrome print-margins article](https://developer.chrome.com/blog/print-margins) | 2024-10-30 | High |
| 8.3 | Measured: Chrome 155 rendered `@bottom-center{content:"Page " counter(page) " of " counter(pages)}`; the text "Page 3 of 64" extracts from page 3. | — | Black-box | 2026-10-19 | High |
| 8.4 | **WeasyPrint** claims all of CSS Paged Media Level 3 (margin boxes, `:left/:right/:first/:blank`, `:nth()`, named pages, bleed, marks), GCPM named strings, running elements (without `element()`'s start parameter), `target-counter()`/`target-text()`, leaders, footnotes (`float: footnote`), and `bookmark-level/label/state` for outlines. Page breaks work for pages, not columns. | — | [WeasyPrint supported features](https://doc.courtbouillon.org/weasyprint/stable/api_reference.html#supported-features) | 2026 | High |
| 8.5 | GCPM defines **named strings** (text-only, scoped from the setting page onward) and **running elements** (whole elements moved to margin boxes); the CSSWG test suite has 17 running header/footer tests. GCPM has stayed a Working Draft since 2006. | 17 tests | [CSS GCPM drafts](https://drafts.csswg.org/css-gcpm-3/); [GCPM test suite](https://test.csswg.org/suites/css-gcpm-3_dev/nightly-unstable/html/chapter-1.htm) | 2006-2014 | Medium |
| 8.6 | print-css.rocks compares PDFreactor, Prince, Antenna House, WeasyPrint, Typeset.sh, Vivliostyle and BFO; Vivliostyle is a JavaScript paginator running on Chromium. | 7 tools | [print-css.rocks](https://print-css.rocks) | 2022 | Medium |
| 8.7 | **`window.print()`** (WHATWG HTML): returns if the document is not fully active; waits for load ("print when loaded flag"); the UA "may display a message to the user or return (or both)"; sandboxed modals suppress it and **neither beforeprint nor afterprint fires**; otherwise fire **beforeprint** on the window and child navigables, offer physical or PDF output (optionally waiting, which must pause script, using the document state at that step), then fire **afterprint**. A kiosk may silently ignore print. | — | [HTML Standard, Printing](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#printing) | 2026 | High |

**Implications:** Chrome's paged-media support is margin boxes and page counters only; the document-tool engines implement the rest of GCPM. Izcal's layout fragmentation plus a margin-box engine can cover named strings, `target-counter()`, bookmarks and footnotes, closing a real gap for web-to-print without Vivliostyle-style script paginators. The spec's "state as of beforeprint" rule makes print a natural snapshot point for Izcal's deterministic display list.

## 9. Rust PDF-writing crates: landscape and licences

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | **pdf-writer** 0.15.0: "A step-by-step PDF writer", typed builders per object, one buffer; **MIT OR Apache-2.0**; from the Typst project. It is a document-engine library in the sense of the owner direction (PDF engine work). | MIT OR Apache-2.0 | [pdf-writer README](https://github.com/typst/pdf-writer); crates.io | 2026-05-27 | High |
| 9.2 | **krilla** 0.8.2: "high-level, ergonomic Rust library for creating PDF documents" on pdf-writer, with PDF/A-1 to A-4 and PDF/UA-1 export modes, tagged PDF, CFF and TTF subsetting and colour fonts; **MIT OR Apache-2.0**. Typst 0.14 moved its export onto it. PDF engine work: landscape only. | MIT OR Apache-2.0 | [krilla README](https://github.com/LaurenzV/krilla); [Typst 0.14 coverage](https://alternativeto.net/news/2025/10/typst-0-14-brings-pdf-accessibility-by-default-and-native-pdf-images) | 2026-06-04 | High (licence) / Medium (Typst on krilla) |
| 9.3 | **subsetter** 0.2.6: PDF-specific font subsetting (TrueType, CFF), **MIT OR Apache-2.0**. Its output is PDF-only by its own README, so it is part of a PDF engine: landscape only. | MIT OR Apache-2.0 | [subsetter README](https://github.com/typst/subsetter) | 2026-06-04 | High |
| 9.4 | Generic infrastructure that is not engine work: **miniz_oxide** 0.9.1 (MIT OR Zlib OR Apache-2.0), **flate2** 1.1.10 (MIT OR Apache-2.0), RustCrypto **sha2** 0.11.1, **md-5** 0.11.0, **aes** 0.9.3, **cbc** 0.2.1, **rc4** 0.2.0 (all MIT OR Apache-2.0). MD5, RC4 and AES-CBC are what PDF's standard security handler needs if Izcal ever writes encrypted PDF; PDF/A forbids encryption. | all MIT-dual | crates.io registry metadata | 2026-10-19 | High |
| 9.5 | Font parsing crates in the area: **ttf-parser** 0.25.1 (MIT OR Apache-2.0), Google Fonts' **read-fonts / write-fonts / skrifa** (MIT OR Apache-2.0; skrifa is the font scaler Chrome adopted, so it may count as browser-engine work under the owner direction; the fonts and text spec decides), **allsorts** 0.17.0 (**Apache-2.0 only**: excluded). | — | crates.io registry metadata | 2026-10-19 | High (licences) / Medium (engine status) |
| 9.6 | **zune-jpeg** 0.5.15 (MIT OR Apache-2.0 OR Zlib) is a JPEG decoder; with passthrough the writer needs only a JPEG header parser (dimensions, components, Adobe marker), which Izcal's image stack already has. | — | crates.io | 2026-10-19 | High |

**Implications:** usable generic crates exist under MIT-dual for Flate and the hashes Izcal needs for `/ID` and content addressing. Every PDF-specific crate (pdf-writer, krilla, subsetter) is PDF engine work: Izcal builds its own writer, subsetter and instancer per owner direction, and may use these crates' published output only as black-box test oracles.

## 10. Cross-engine comparison (measured 2026-10-19)

| Property | Chrome 155 (Skia/PDF) | WebKit API (Quartz) | Typst 0.14 / Prince (docs) |
|---|---|---|---|
| Tagged | Yes, by default | No | Yes (Typst default; Prince on request) |
| PDF/UA identified | No (no XMP) | No | Yes on request |
| PDF/A | No | No | Yes (several parts) |
| Links | URI and internal | None | Yes |
| Outline | With flag | No | Yes |
| JPEG | Passthrough | Decode, Flate re-encode at display size | n/m |
| Blur / shadow | Raster | Raster | n/a |
| Blend modes | Vector | Vector | n/a |
| Variable fonts | Type 3 | Instanced TrueType | Typst/krilla n/m |
| Colour emoji | Type 3 with 66 px images, no text | 32 px images, no text | n/m |
| ToUnicode | Present (wrong order for fallback runs) | Often missing; one wrong code point | n/m |
| `/ID` | Absent | Random per run | WeasyPrint settable |
| Dates | Wall clock | Absent from `/Info` | SOURCE_DATE_EPOCH (Typst) |
| Object streams | No | No | Prince yes (tagged) |

n/m = not measured; n/a = not applicable.

**Implications:** every cell where a browser falls short is a candidate falsifiable target for Izcal's writer (tagged + PDF/UA-identified, all links, outline by default, passthrough JPEG, vector blend, instanced variable fonts, emoji text, deterministic `/ID` and dates, object streams).

## 11. Questions for black-box measurement (values only an engine's source would state)

| # | Question | How to measure |
|---|---|---|
| 11.1 | At what resolution does Chrome rasterize `filter: blur()` and `box-shadow` (device-pixel ratio, fixed DPI, or effect bounds)? | Print boxes of known size at several blur radii and zoom factors; read image widths. |
| 11.2 | Which effects stay vector in Chrome and Quartz: all 16 blend modes, `backdrop-filter`, `clip-path`, masks, SVG filters, `opacity` on groups? | One element per feature; inspect XObjects and ExtGStates. |
| 11.3 | Does Chrome ever re-encode a JPEG (CMYK, progressive, EXIF orientation, huge images)? | Print each variant and byte-search for the source. |
| 11.4 | Does either browser honour OS/2 fsType 2 (restricted) or 0x0200 (bitmap only)? | Web fonts with each fsType value; inspect embedding. |
| 11.5 | How do Chrome and Quartz handle COLRv1 fonts (vector Type 3 vs raster)? | A COLRv1 test font via `@font-face`. |
| 11.6 | Firefox skpdf output on macOS/Linux: tags, links, fonts, determinism. | Drive Firefox print-to-file through its remote protocol; same probe pages. |
| 11.7 | Safari's own "Export as PDF" menu: does it add links or tags absent from the WebKit API path? | Drive Safari by UI automation; compare files. |
| 11.8 | Chrome print-to-PDF time and memory for 1,000-page documents and image-heavy pages, without start-up time. | DevTools protocol `Page.printToPDF` timing on a warmed instance, with `rig` booking. |

## Claims not verified

- 1.14-1.15: the Chromium blog post body was not retrieved; Chrome 85 details come from press coverage. The `--export-tagged-pdf` headless flag may be obsolete now that tagging is the default (Chrome 155 tagged without it).
- 1.8: mapping of each raster image to blur vs shadow is inferred from image sizes.
- 2.10: Firefox 158 skpdf rollout comes from a press article that refused fetch and a search snippet of bug 2050228.
- 2.2, 2.3: Safari's menu path was not driven; only the WebKit API was measured.
- 3.6: LibreOffice 7.0 PDF/UA status (experimental or not) conflicts between reporters; the PDF Association LibreOffice 24.2 article refused fetch.
- 3.8, 5.1, 5.2 (spec part), 5.9 (spec part), 6.8: ISO 32000-1/-2, ISO 14289 and ISO 19005 details are from the researcher's recollection; the standards were not read this session.
- 4.1: Prince's summary lists "no transparency" for PDF/A-2/3, which conflicts with ISO 19005-2 as commonly described (transparency allowed from PDF/A-2); treat as a summarisation error until the standard is checked.
- 4.3: PDF/A-4 tagging and Unicode requirements are from secondary sources that disagree.
- 5.5: PDF 2.0 lacking variable and SVG font support is from a 2020 presentation abstract; whether ISO 32000-2:2020 or later amendments changed this was not checked.
- 6.2: pdfTeX details are from search snippets of the NEWS file and man page.
- 9.2: "Typst 0.14 export rewritten on krilla" comes from third-party coverage; the official 0.14 blog page fetched did not mention it.
- 9.5: whether skrifa/read-fonts count as browser-engine libraries is a judgement for the fonts and text spec, not a finding.
- 2.12: the 100 MB Chrome anecdote is a single unsourced blog post.
- Unreachable on 19 October 2026 (fact-check FC-R25): print-css.rocks (8.6).
