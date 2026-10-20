# Izcal PDF: rendering_text_accessibility

Compiled 19 October 2026. Primary sources were preferred (papers, specifications, maintainers' posts, release notes, issue trackers).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only, or the researcher's recollection.

M-1 note: this file records ideas, dates, measurements and stated reasons. No source code of any browser or PDF engine or library was opened (pdf.js, PDFium, Poppler, xpdf, MuPDF, Ghostscript and the rest). A search result pointed to a Chromium commit about PDF tags; it was not opened and is not cited. The pdf.js wiki FAQ (a Markdown prose page) was read as prose. Where a row says what a viewer "does", it is evidence to reason from, not a shape to copy. Values that only source code would state are listed as black-box measurements at the end.

**Method notes**
- About 30 searches and fetches were used before the shared search budget ran out. After that, only known primary pages were read directly (licence files through the GitHub API, Firefox release notes, the pdf.js FAQ, two papers converted to text locally).
- Firefox release notes 80 to 145 were scanned for every sentence that mentions PDF, so Firefox feature dates are High.
- Kuchta et al. (EMSE 2018) and Bast and Korzen (JCDL 2017) were read in full text.
- adobe.com help pages, pdfa.org and iso.org refused automated access (HTTP 403). PDF/UA-2, Well-Tagged PDF, ISO 32000-2 availability and Adobe's Type 1 end-of-support details are therefore Medium or Low.
- Rows that rest on the researcher's recollection are graded Low and repeated under "Claims not verified".

## 1. How often viewers disagree: published measurements

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | Kuchta, Lutellier, Wong, Tan and Cadar did the first systematic cross-reader study. They examined the first page of a random sample of Govdocs1 (US government) PDFs in 8 readers: Acrobat Reader, Xpdf, Evince, Okular, qpdfview, MuPDF, Firefox and Chromium. | 314 of 2,313 files (13.5%, ±1% at 95%) rendered differently in at least one reader; in 41% of those, 7 of 8 readers agreed | [Kuchta et al., EMSE 2018](https://srg.doc.ic.ac.uk/files/papers/pdf-emse-18.pdf), Table 4 | 2018 | High |
| 1.2 | The study classed a reader as wrong by majority vote (5 or more identical renderings). The authors refused to treat Acrobat as ground truth: "Acrobat Reader also has bugs". | Majority rule, 8 readers | same, §4.1 | 2018 | High |
| 1.3 | Missing-font substitution has **no standardized correct answer**: "Most readers use different default fonts, and because there is no standardized behaviour on the correct rendering of a missing font, we cannot determine which reader behaves correctly." | Reported only as a total count | same, §4.1 | 2018 | High |
| 1.4 | Symptom categories were missing images, font issues, gradient, form, map, colour and performance inconsistencies. A Firefox (pdf.js) example filled a map with black. Chromium and Acrobat highlighted editable form fields; the other readers did not. | 30 unique bugs found automatically over 230K files; 33 reported | same, §1, §4.3, Fig. 1 | 2018 | High (per-category counts: Medium, the table did not extract cleanly) |
| 1.5 | Projected over Govdocs1, 28K to 33K files are inconsistent. Since only first pages were checked, the real share is higher. | 230K files | same, §4.2 | 2018 | High |
| 1.6 | No published, systematic test of browser PDF viewers against the Ghent Workgroup (GWG) Output Suite 5.0 was found. The suite is 48 patches on a six-page PDF/X-4 file covering overprint mode, transparency, soft masks, spot colours and ICC handling. Pass claims are self-reported by prepress vendors (EFI Fiery, Esko, callas). | 48 patches | [Industria Gráfica on GWG 5.0](https://www.industriagraficaonline.com/articulo/15788/); [Esko report](https://www.labelsandlabeling.com/news/workflow-and-inspection/esko-reports-compliance-gwg-pdf-output-suite-version-50) | about 2019 | Medium |
| 1.7 | Practitioners' consensus (forums and vendor FAQs, not studies): browser viewers are "convenience utilities with very limited rendering range"; transparency, blend modes and overprint vary by viewer, and flattening on export is the usual workaround. Apple Preview was reported not to show an InDesign blend effect. | none | [Adobe Community thread](https://community.adobe.com/questions-9/why-pdf-documents-can-appear-differently-across-pdf-viewers-1299966); [Apryse FAQ](https://docs.apryse.com/documentation/web/faq/pdf-color-different/) | 2023-2025 | Low |
| 1.8 | No PDF Association "PDF rendering" comparative study was found in the budget; pdfa.org refused automated access. | none | (search, 2026-10-19) | 2026-10-19 | Low |

**Implications:** About one government PDF in seven already renders differently somewhere, and font substitution has no "right" answer, so Izcal cannot target "match Acrobat". It needs its own reference: the spec's normative rendering model plus a published conformance corpus (GWG 5.0 patches, Govdocs1-style corpora as test-only inputs) run as a differential oracle against other viewers in black-box mode.

## 2. Rendering features where viewers disagree (spec-level)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **Overprint** (OP, op, OPM) only has a defined visible effect in separations or with overprint simulation. Acrobat's "Overprint Preview" setting defaults to PDF/X files only, so ordinary documents show differently between viewers that simulate and those that do not. | default: PDF/X only | Recollection of Acrobat preferences | n/a | Low |
| 2.2 | **Knockout groups**, isolated groups, soft masks and the 16 blend modes (12 separable, 4 non-separable) are defined in ISO 32000-2 §11. Non-separable modes in CMYK and blending in separations (spot colorants under transparency) are the classic divergence points. | 16 blend modes | ISO 32000-2 §11 (not re-read) | 2017/2020 | Medium |
| 2.3 | **Shading types 4 to 7** (free-form and lattice Gouraud triangle meshes, Coons and tensor-product patch meshes) need mesh subdivision. Gradient inconsistencies were one of Kuchta's categories. | 7 shading types | ISO 32000-2 §8.7.4.5; Kuchta Table 5 | 2017/2018 | Medium |
| 2.4 | **Conflation artifacts**: coverage-based anti-aliasing draws faint seams where two shapes share an edge (maps, CAD, tiled images), because per-shape coverage is blended separately. The only exact remedies are supersampling or analytic multi-shape coverage. | none | Recollection (Kilgard, NV_path_rendering papers, 2012) | 2012 | Low |
| 2.5 | **Stroke adjustment** (the SA graphics-state flag) asks the renderer to snap thin strokes to whole device pixels so hairlines look even. Whether and how each viewer applies it is not documented. | none | ISO 32000-2 §10.7.5 (not re-read) | 2017 | Medium |
| 2.6 | **Type 3 fonts** are content streams per glyph (bitmaps or vector); they inherit graphics state and are often used by TeX for bitmap fonts. They fail text extraction (5.4). | none | Bast and Korzen §6 | 2017 | High (extraction); Medium (rendering) |
| 2.7 | **Missing non-embedded fonts** are a top source of inconsistency (1.3, 1.4). | none | Kuchta | 2018 | High |

**Implications:** Overprint simulation, knockout and blend-in-separation semantics, mesh shadings, stroke adjustment and conflation are the exact places to make Izcal's rendering normative and tested. The display list needs a mode for analytic multi-shape coverage or supersampled tiles, chosen per page, so maps and CAD drawings have no seams.

## 3. How browser viewers render, and published performance figures

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | pdf.js renders each page into a canvas and holds canvases only for visible pages. A US Letter page at 96 DPI is 816×1056 px, 3,446,784 bytes as RGBA; at device pixel ratio 2 it is about 14 MB per page. | 3.4 MB (1x), 14 MB (2x) per page | [pdf.js FAQ](https://github.com/mozilla/pdf.js/wiki/Frequently-Asked-Questions#allthepages) | read 2026-10-19 | High |
| 3.2 | pdf.js uses HTTP Range requests to fetch only the parts needed for visible pages, when the server supports ranges. | none | [pdf.js FAQ](https://github.com/mozilla/pdf.js/wiki/Frequently-Asked-Questions#range) | read 2026-10-19 | High |
| 3.3 | pdf.js maintainers state what makes files slow: high-resolution images (150 dpi suffices for screens), masking and transitions (flatten transparency), generators emitting many tiny images, missing linearization, corrupt files. "The amount of pages does not affect the performance"; page size and complexity do. | 150 dpi guidance | [pdf.js FAQ](https://github.com/mozilla/pdf.js/wiki/Frequently-Asked-Questions#optimize) | read 2026-10-19 | High |
| 3.4 | pdf.js lays a text layer of transparent, absolutely positioned DOM spans over the canvas for selection and find; the structure tree becomes a hidden DOM tree for screen readers. | none | [Nutrient tutorial](https://www.nutrient.io/blog/pdfjs-accessibility-structtree-printing/) (vendor) and recollection | 2024 | Low |
| 3.5 | Chrome's viewer uses PDFium in a separate process and paints tiled page images into the browser's compositor; text selection and accessibility come from PDFium's text extraction, not a DOM. | none | Recollection | n/a | Low |
| 3.6 | No published, reproducible benchmark of page render time or memory for large PDFs across Chrome, Firefox, Safari and Edge was found in the budget. | none | (search) | 2026-10-19 | Low |
| 3.7 | Edge's built-in reader has shipped view and print since Edge 79, inking since 80, highlight since 81, read aloud since 84, table of contents since 86, text notes since 88; it is Chromium-based and isolates PDFs per site process. | versions listed | [Microsoft Learn: PDF reader in Edge](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-pdf) | 2024-07-18 (page date) | High |
| 3.8 | Edge announced a move to the Adobe Acrobat PDF engine for its reader (2023). | none | Recollection | 2023 | Low |

**Implications:** Memory is dominated by page rasters, not by the document, so Izcal's viewer should raster only visible tiles into the shared display-list cache under the footprint budget and stream with range requests. Page-render time and peak memory for large PDFs have no public baseline; Izcal must measure other viewers itself (Section 13).

## 4. Fonts in PDF: formats, selection rules and the substitute-font licence problem

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | PDF font programs: Type 1 (FontFile), TrueType (FontFile2), bare CFF and CID-keyed CFF (FontFile3/Type1C, CIDFontType0C), OpenType (FontFile3/OpenType), and Type 3 (content streams). Composite (Type 0) fonts use CIDFonts plus a CMap. | 5 embedding forms | ISO 32000-2 §9.6-9.9 (not re-read) | 2017/2020 | Medium |
| 4.2 | Adobe ended support for authoring with PostScript Type 1 fonts in its applications in January 2023. Embedded Type 1 in existing PDFs still must render; the end of support concerns installed fonts in authoring apps. | January 2023 | [Adobe help page](https://helpx.adobe.com/fonts/kb/postscript-type-1-fonts-end-of-support.html) (HTTP 403 to automated fetch) | 2023-01 | Medium |
| 4.3 | **TrueType symbolic fonts and cmap selection**: the spec gives an order of preference among the font's (3,1) Microsoft Unicode, (1,0) Macintosh Roman and (3,0) Microsoft Symbol subtables, with the (3,0) lookup offset into F000-F0FF, and different rules for symbolic and non-symbolic fonts (the Flags bit 3). Producers often set these flags wrongly. | 3 subtables | ISO 32000-2 §9.6.5.4 (not re-read) | 2017 | Medium |
| 4.4 | **Adobe CMap resources** (Adobe-Japan1-7, Adobe-GB1-6, Adobe-CNS1-7, Adobe-Korea1-2, Adobe-KR-9, Adobe-Manga1-0, Adobe-Identity-0) are licensed **BSD-3-Clause** ("Copyright 1990-2023 Adobe"). That is not MIT. The predefined CMap names are normative in ISO 32000, which may argue for the standards-data exception, but Adobe is not a standards body. | 7 collections | [cmap-resources licence (GitHub API)](https://github.com/adobe-type-tools/cmap-resources) | read 2026-10-19 | High |
| 4.5 | The **Adobe Glyph List** (AGL, AGLFN, ITC Zapf Dingbats list), which maps glyph names to Unicode and is the fallback when a font has no ToUnicode, is also **BSD-3-Clause**. | 3 files | [agl-aglfn licence (GitHub API)](https://github.com/adobe-type-tools/agl-aglfn) | read 2026-10-19 | High |
| 4.6 | **The standard 14 fonts**: Times (4 styles), Helvetica (4), Courier (4), Symbol and ZapfDingbats. Non-embedded use is common in older files; PDF 2.0 deprecates relying on them unembedded. | 14 | ISO 32000-2 §9.6.2.2 (not re-read) | 2017 | Medium (deprecation detail: Low) |
| 4.7 | **Liberation** (metric-compatible with Times New Roman, Arial, Courier New) is SIL OFL 1.1 with reserved font names. | OFL 1.1 | [Liberation LICENSE](https://github.com/liberationfonts/liberation-fonts/blob/main/LICENSE) | read 2026-10-19 | High |
| 4.8 | **Croscore** Arimo, Tinos and Cousine (the Liberation 2.x base) are listed as OFL in Google Fonts' metadata. | OFL | [google/fonts ofl/arimo METADATA](https://github.com/google/fonts/tree/main/ofl/arimo) | read 2026-10-19 | High |
| 4.9 | **URW++ base 35** (the Ghostscript substitutes for all 35 PostScript Level 2 fonts, including Symbol and Dingbats) ship with the AGPL-3.0 text in COPYING. Whether a font-embedding exception applies was not checked. | 35 faces | [urw-base35-fonts](https://github.com/ArtifexSoftware/urw-base35-fonts) | read 2026-10-19 | High (licence file); Low (exception) |
| 4.10 | No MIT-licensed metric-compatible substitute for the standard 14 was found. TeX Gyre (GUST Font License) was not checked this session. | 0 MIT sets found | (search, licence files) | 2026-10-19 | Medium |
| 4.11 | The accepted fonts spec already supports Type 1 and Type 3 "for PDF only, in the PDF process", and Izcal designs its own metric-compatible aliases under a widths contract (TX-D19, TX-8), checked against Adobe's published metrics for the PostScript core families. Symbol and ZapfDingbats are not named among the aliases. | n/a | `reports/Izcal fonts and text.md` TX-D7, TX-D19, TX-D24 | 2026-10-19 | High |

**Implications:** Izcal cannot ship Liberation, Croscore or URW, so the standard 14 substitutes must be Izcal's own faces, and the PDF spec should ask the fonts track to add Symbol and ZapfDingbats (with their built-in encodings) to TX-W1. Adobe's CMaps and AGL are BSD-3: either the owner rules them standards data (they are named normatively by ISO 32000), or Izcal regenerates equivalent tables from Unicode data and the published character-collection specifications; this is a licence decision to raise.

## 5. Text extraction, reading order, ligatures and hyphenation

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | PDF stores glyphs and positions, not words, lines or roles. Word identification is hard because inter-letter spacing varies, words are hyphenated, ligatures appear, and characters with diacritics may be drawn as two glyphs at different positions. | none | [Bast and Korzen, JCDL 2017](https://ad-publications.informatik.uni-freiburg.de/benchmark.pdf), §1.1 | 2017 | High |
| 5.2 | Bast and Korzen built a benchmark from parallel TeX and PDF for 12,098 arXiv papers and evaluated 14 extraction tools. Their own tool, Icecite, was best "but is still not perfect". | 12,098 docs; 14 tools | same, abstract | 2017 | High |
| 5.3 | pdftotext does not merge hyphenated words and does not handle ligatures or diacritics; PdfBox handles ligatures and diacritics but not hyphenation; pdfminer and pdf-extract have large word-error counts because they do not translate ligatures or merge hyphenation; two-column articles broke reading order in pdf2xml and pdf-extract. | per-tool criteria | same, §4-5 | 2017 | High |
| 5.4 | Two problems remained for every tool: glyphs in **Type 3 fonts** "are not identifiable as text", and **mandatory hyphens** at line ends ("sugar-free") are wrongly merged. The authors say Type 3 needs OCR- or learning-based methods. | none | same, §5 | 2017 | High |
| 5.5 | Word and line detection in mainstream tools (pdfminer's layout analysis with char_margin, line_margin, word_margin parameters) is heuristic and parameter-driven. | none | Recollection of pdfminer documentation | n/a | Low |
| 5.6 | ICDAR runs competitions on table detection and page segmentation (for example the 2013 table competition and cTDaR 2019); they show learned layout analysis beating rules on scanned and born-digital pages. | none | Recollection | 2013, 2019 | Low |
| 5.7 | ToUnicode CMaps are optional and often absent or wrong; the fallback chain is ToUnicode, then a predefined CMap's character collection, then glyph names through AGL, then nothing. Copy-paste of "ﬁ" as one code point, Private Use Area output, and gibberish from subset fonts with custom encodings are the visible failures. ActualText on marked content overrides the glyphs for extraction. | none | ISO 32000-2 §9.10, §14.9.4 (not re-read) | 2017 | Medium |
| 5.8 | Firefox's PDF viewer turns all URLs in text into links (Firefox 137), which needs reliable word extraction. | Firefox 137, 2025-04-01 | [Firefox 137 release notes](https://www.firefox.com/en-US/firefox/137.0/releasenotes/) | 2025-04-01 | High |
| 5.9 | Firefox's PDF find bar has options for case and diacritics matching; Chrome's find in PDF is PDFium text search. Neither vendor documents the matching rules. | none | Recollection | n/a | Low |

**Implications:** Izcal's extractor should be one component shared by find, selection, copy, accessibility and "save as text": ToUnicode, then CMap and AGL fallbacks, ActualText honoured, ligature decomposition by Unicode compatibility mappings, a hyphenation join that consults the line-end glyph and the dictionary (TX hyphenation data, not a heuristic merge), and a structure-tree reading order when tagged. Untagged reading order should be a measured component, scored on the Bast-Korzen benchmark (test-only input, licence to check). Type 3 text needs OCR (Section 8).

## 6. Accessibility standards: tagged PDF, PDF/UA, WTPDF, PDF-AAM, WCAG

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **PDF/UA-1** is ISO 14289-1 (2012, revised 2014) on PDF 1.7. **PDF/UA-2** is ISO 14289-2 (2024) on PDF 2.0. | 2 parts | [Dual Lab post](https://groups.google.com/g/common-crawl/c/_ttmwLV7wuk) (pairing only); dates from recollection | 2026-10-05 | Medium |
| 6.2 | **Well-Tagged PDF (WTPDF)** 1.0 is a PDF Association specification for PDF 2.0 tagging with two conformance levels, accessibility and reuse, aligned with PDF/UA-2. | 2 levels | pdfa.org (HTTP 403); recollection | 2024 | Low |
| 6.3 | **PDF-AAM 1.0** is a W3C ARIA Working Group Editor's Draft dated 12 October 2026, not a Recommendation. It maps H1-H6 to `heading` with `aria-level`, Link to `link` (no destination: `generic`), Sect to `region` if named else `generic`, Part and Art to `article`, Aside to `complementary`, Div to `group` or `generic`, Lbl contextually. P, Table, Figure, L, Artifact, ActualText, Alt, Lang and role maps are TBD or absent. | Editor's Draft | [PDF-AAM ED](https://w3c.github.io/aria/pdf-aam/) | 2026-10-12 | High |
| 6.4 | The W3C PDF-AAM Task Force work statement was approved on 12 November 2024; it is facilitated by Duff Johnson (PDF Association) and Valerie Young, building on the PDF Association's PDF/UA Processor work. | 2024-11-12 | [Task force work statement](https://www.w3.org/WAI/about/groups/task-forces/pdf-aam/work-statement) | 2024-11-12 | Medium |
| 6.5 | **WCAG 2.2 PDF Techniques** are PDF1 to PDF23: Alt text (PDF1), bookmarks (PDF2), tab and reading order (PDF3), Artifact for decorative images (PDF4), required form controls (PDF5), table tags (PDF6), OCR of scanned pages (PDF7), list tags (PDF21), input errors (PDF22), interactive form controls (PDF23), among others. | 23 techniques | [WCAG 2.2 Techniques](https://www.w3.org/WAI/WCAG22/Techniques/) | read 2026-10-19 | High |
| 6.6 | The Matterhorn Protocol lists PDF/UA-1 failure conditions (31 checkpoints, 136 failure conditions), most machine-checkable. | 31 / 136 | Recollection | 2014 | Low |

**Implications:** The mapping from PDF structure to an accessibility tree is still being standardized, and the draft does not cover tables, figures, lists, Alt, ActualText or Lang. Izcal must define its own complete mapping now (anchored to PDF-AAM where it exists and to HTML-AAM by analogy where it does not), record each gap as a spec decision, and contribute it back.

## 7. How browsers expose tagged PDF to screen readers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | Information Access Group tested one tagged file (H1-H3, tag order different from visual order, an image with alt text, a table with header cells) with JAWS, NVDA and VoiceOver on 18 May 2022. | 6 viewers | [IAG comparison](https://informationaccessgroup.com/comparing-how-well-pdf-viewer-applications-work-with-screen-readers/) | 2022-05-18 | High |
| 7.2 | **Chrome 101 and Edge 101**: reading order followed visual stacking, not tags; H1, H3 and body text announced as H2; the image skipped (Chrome) or merged with the next heading (Edge); table cells merged, not announced as a table. Neither exposed the tag structure. | Chrome 101, Edge 101 | same | 2022-05-18 | High |
| 7.3 | **Firefox 100**: reading order followed tags and headings matched; the image was skipped; the table was announced but first-column cells repeated. | Firefox 100 | same | 2022-05-18 | High |
| 7.4 | **Safari 15.4 and Preview 11.0** (macOS PDFKit) and **Acrobat Pro** followed tags, announced the image in place with alt text and the table with header labels. These were the best. | 3 viewers | same | 2022-05-18 | High |
| 7.5 | Firefox's viewer has exposed tagged-PDF structure (headings, tables, alt text) to assistive technology since about Firefox 89 (released 2021-06-01; the source says 1 July). | Firefox 89 | [Fondazione LIA](https://www.fondazionelia.org/en/?p=5275) | 2022-01-17 | Medium |
| 7.6 | Whether Chrome's viewer now exposes tags in stable was not confirmed: no Chrome release note was found, and the only evidence is a code change that was not opened (M-1). | unknown | (search) | 2026-10-19 | Low |
| 7.7 | Edge documents keyboard access, caret browsing, high-contrast rendering using OS settings, and "screen reader support", without describing tag handling. | none | [Microsoft Learn](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-pdf) | 2024-07-18 | High |
| 7.8 | Caret browsing works in Firefox's PDF viewer since Firefox 124. | 2024-03-19 | [Firefox 124 release notes](https://www.firefox.com/en-US/firefox/124.0/releasenotes/) | 2024-03-19 | High |

**Implications:** As late as 2022, two of the four major browsers threw tags away, while PDFKit, which builds a native accessibility tree, did best. Izcal should build the PDF accessibility tree in the same tree model as HTML (one accessibility pipeline), driven by the structure tree, and run an IAG-style screen-reader test matrix as a release gate.

## 8. Untagged PDFs: prevalence and OCR

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | Dual Lab (PDF4WCAG) analysed more than 20.5 million PDFs from January 2023 to June 2026. The tagged share first passed 50% in July 2025 and grows about 1.5 points a year. About 2% have an empty structure tree. | >50%; +1.5 pt/yr; 2% empty | [Dual Lab, Common Crawl group](https://groups.google.com/g/common-crawl/c/_ttmwLV7wuk) | 2026-10-05 | High (as reported); not peer-reviewed |
| 8.2 | In the same corpus about 0.9% claim PDF/UA only, 2.25% PDF/A only, 0.07% both. Of PDF/UA files, 99.3% are PDF/UA-1 and 0.3% PDF/UA-2. The commonest tree has 64-127 elements and 4-7 levels; 308 trees exceed 128 levels. | as stated | same | 2026-10-05 | High (as reported) |
| 8.3 | A 2023 Luxembourg government study of PDFs on most-visited sites found 52% untagged; a scholarly-PDF study found 13.4% marked tagged. | 52%; 13.4% | [accessibilite.public.lu](https://accessibilite.public.lu/en/news/2023-04-28-des-pdf-majoritairement-inaccessibles.html); [arXiv 2105.00076](https://arxiv.org/pdf/2105.00076) | 2023; 2021 | Medium |
| 8.4 | Google: 360+ billion PDFs, 12% of PDFs on the web (Google internal study), are inaccessible to screen readers (image-only). Chrome's ML OCR runs **on the device**, offline, on macOS, Windows and ChromeOS; post-processing builds navigation and landmarks; 77 languages and seven more scripts; over 1 million pages OCRed weekly. It is downloaded on demand by hardware, not built in. | 12%; 77 languages; >1M pages/week | [chromeos.dev](https://chromeos.dev/en/posts/frictionless-pdf-accessibility-with-ml-and-ocr) | 2024-09-24 | High (as Google's claim) |
| 8.5 | Desktop Chrome made OCR of scanned PDFs (search, select, copy, screen reader) broadly available around April 2025. The OCR component (Screen AI) is reported to use significant resources and to have broken in one release. | about 2025-04 | [Nottingham blog](https://blogs.nottingham.ac.uk/learningtechnology/?p=3111) (not opened); [Vivaldi forum](https://forum.vivaldi.net/post/831700) | 2025 | Low |
| 8.6 | Tagged-but-wrong is common: empty trees (8.1), and IAG-style heading mis-levels. No study measured the share of tagged PDFs whose tags are correct. | unknown | (gap) | 2026-10-19 | Medium |

**Implications:** About half of web PDFs have no tags and a tenth have no text at all, so Izcal needs both a heuristic structure builder for untagged text PDFs and on-device OCR for image-only pages. Building our own OCR model and runtime is engine-adjacent ML work with no MIT model available known to this strand; it is a large item for the spec to cost, with the model's training data licence checked.

## 9. Colour: ICC, CMYK, output intents, spot colours

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | Colour inconsistencies were a category in Kuchta's study, including Acrobat Reader 9.5.5 on Ubuntu rendering a wrong background colour. | none | Kuchta §4.3 | 2018 | High |
| 9.2 | Vendors note that colour differs between viewers because some disable colour management or overprint by default. | none | [Apryse FAQ](https://docs.apryse.com/documentation/web/faq/pdf-color-different/) | n/a | Low |
| 9.3 | PDF colour spaces: DeviceGray/RGB/CMYK, CalGray, CalRGB, Lab, ICCBased, Indexed, Pattern, Separation, DeviceN. Output intents (PDF/X, PDF/A) supply the ICC profile that DeviceCMYK should be interpreted through. | 11 families | ISO 32000-2 §8.6, §14.11.5 (not re-read) | 2017 | Medium |
| 9.4 | Simple viewers convert DeviceCMYK with a naive formula or a fixed polynomial instead of an ICC transform; this gives over-saturated, wrong-hue CMYK images. Exact viewer defaults are not documented in prose. | none | Recollection | n/a | Low |
| 9.5 | Spot colours (Separation, DeviceN) render on screen through their alternate space and tint transform (a PostScript calculator function or a sampled function); viewers differ on whether overprint of spots is simulated. | none | ISO 32000-2 §8.6.6 (not re-read) | 2017 | Medium |

**Implications:** Izcal's colour pipeline (the accepted graphics spec's colour management) should treat PDF exactly: ICC transforms for ICCBased and output intents, a named default CMYK profile (which must be MIT or Izcal-made; FOGRA and SWOP characterization data licences need checking), and overprint simulation on by default for files that use overprint. The default CMYK profile is an open licence question.

## 10. Printing from the viewer

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | Firefox's PDF viewer prints by rendering pages to canvas images at a fixed resolution (150 DPI by default), so printed text is raster and can look soft; users raise it as a complaint. | 150 DPI default | Recollection | n/a | Low |
| 10.2 | Edge prints local, online and embedded PDFs (since Edge 79). | Edge 79 | [Microsoft Learn](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-pdf) | 2024-07-18 | High |
| 10.3 | Firefox can write tagged PDF when printing to PDF, behind `accessibility.tagged_pdf_output.enabled`, using its accessibility engine and Skia's PDF backend. | pref | [Firefox source docs (prose): Tagged PDF Output](https://firefox-source-docs.mozilla.org/accessible/TaggedPdfOutput.html) | n/a | Medium |
| 10.4 | Chrome's "Save as PDF" writes tagged PDF since about Chrome 85 (2020). | Chrome 85 | [9to5Google](https://9to5google.com/2020/07/29/chrome-tagged-pdf/) | 2020-07-29 | Medium |
| 10.5 | Whether Chrome prints a viewed PDF by passing the original file through to the print system or by re-rendering is not documented in prose. | unknown | (gap) | 2026-10-19 | Low |

**Implications:** Printing a viewed PDF should pass through vector content (the original page content, or Izcal's display list emitted as vector to the print backend), never a fixed-DPI raster; that is measurable (13.6). Tagged output on print-to-PDF already ships in Chrome, so Izcal's writer should emit tags from the start.

## 11. Forms: AcroForm, appearances, saving, JavaScript, XFA

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 11.1 | Firefox runs JavaScript embedded in PDF forms (validation and interactivity) since Firefox 88. | 2021-04-19 | [Firefox 88 release notes](https://www.firefox.com/en-US/firefox/88.0/releasenotes/) | 2021-04-19 | High |
| 11.2 | Firefox supports XFA-based forms ("used by multiple governments and banks") since Firefox 93. | 2021-10-05 | [Firefox 93 release notes](https://www.firefox.com/en-US/firefox/93.0/releasenotes/) | 2021-10-05 | High |
| 11.3 | Edge: "Basic form filling (JavaScript forms aren't supported)" since Edge 79; "XFA … isn't supported in Microsoft Edge." | Edge 79 | [Microsoft Learn](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-pdf) | 2024-07-18 | High |
| 11.4 | Chrome's viewer does not support XFA by default; an experimental "PDF XFA support" flag existed around 2023. Adobe's guidance is to use Acrobat for full XFA. | flag | [Adobe Experience League](https://experienceleague.adobe.com/en/docs/experience-manager-65/content/forms/troubleshooting/xfa-forms-firefox-chrome); [IMS guide](https://knowledge.imstechnologygroup.com/en/knowledge/how-to-configure-google-chrome-to-display-pdf-xfa-documents) | 2023 | Medium |
| 11.5 | XFA is deprecated in PDF 2.0 (ISO 32000-2). | none | Recollection | 2017 | Medium |
| 11.6 | Kuchta found form inconsistencies across readers, including field highlighting. Appearance streams (/AP) and NeedAppearances mean a filled field can look different depending on whether the viewer regenerates appearances. | none | Kuchta §4.3; ISO 32000-2 §12.7 (not re-read) | 2018 | Medium |

**Implications:** Firefox shows that a browser viewer can fill XFA and run form JavaScript; Edge and Chrome show that a viewer can decline both. Izcal's choices (XFA yes or no; form JS through Izcal's own engine under a restricted API, or not at all) are spec decisions; the evidence here is that real users with government and bank forms hit XFA, and that it is deprecated.

## 12. Annotations and editing in browser viewers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 12.1 | Firefox 106 added editing: free text, drawing (ink) and signatures. | 2022-10-18 | [Firefox 106 release notes](https://www.firefox.com/en-US/firefox/106.0/releasenotes/) | 2022-10-18 | High |
| 12.2 | Firefox 116: edit existing text annotations. Firefox 119 (gradual): add images with alt text. Firefox 121: a floating delete button. Firefox 125: text highlighting. Firefox 137: signatures without leaving Firefox. Firefox 145: comments with a comment sidebar. | 2023-08-01 to 2025-11-11 | [Firefox release notes 116](https://www.firefox.com/en-US/firefox/116.0/releasenotes/), [119](https://www.firefox.com/en-US/firefox/119.0/releasenotes/), [121](https://www.firefox.com/en-US/firefox/121.0/releasenotes/), [125](https://www.firefox.com/en-US/firefox/125.0/releasenotes/), [137](https://www.firefox.com/en-US/firefox/137.0/releasenotes/), [145](https://www.firefox.com/en-US/firefox/145.0/releasenotes/) | 2023-2025 | High |
| 12.3 | Firefox's image editor asks for alt text or "decorative" when an image is added, and downloading saves the annotations into the file. | none | [Mozilla support](https://support.mozilla.org/en-US/kb/view-pdf-files-firefox-or-choose-another-viewer) | n/a | Medium |
| 12.4 | Edge: inking (Edge 80), highlights (81), text notes (88). | versions | [Microsoft Learn](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-pdf) | 2024-07-18 | High |
| 12.5 | Chrome: Chrome OS planned pen annotation on the "Ink" library in 2018; a desktop flag "annotate PDFs using a new ink library" followed; a 2026 guide describes Highlight and Note tools with download and Save to Google Drive. Chrome for Android gained a native PDF viewer with annotation in Chrome 135 (2025). | Chrome 135 (Android) | [9to5Google 2018](https://9to5google.com/2018/12/12/chrome-os-draw-pdfs-pixelbook-pen/); [Windows Report](https://windowsreport.com/google-chrome-will-soon-let-you-annotate-pdfs-with-ink-signatures/); [ChromeStory](https://chromestory.com/2026/06/annotate-pdf-chrome-built-in/); [heise](https://heise.de/-10363031) | 2018-2026 | Low |
| 12.6 | No usage statistics on which annotation tools people use in browser viewers were found. The order in which vendors shipped (ink and text first, then highlight, then signatures and comments) is the only proxy. | none | (gap) | 2026-10-19 | Low |

**Implications:** Every major browser now ships ink, highlight and notes, and Firefox adds signatures, images with alt text and comments, all saved into the file. Izcal needs an incremental-update writer that preserves the original bytes (signatures) and generates appearance streams deterministically; usage data should come from Izcal's own opt-in telemetry, since none is published.

## 13. Open questions and black-box measurements needed

| # | Question | How to measure |
|---|---|---|
| 13.1 | Each viewer's missing-font substitution for the standard 14 and for unknown names | Render a corpus of non-embedded-font files in Chrome, Firefox, Safari, Edge, Acrobat; compare glyph advance widths and shapes |
| 13.2 | Overprint, knockout, blend-in-separation and spot behaviour per viewer | Render the GWG Output Suite 5.0 patches (test-only input) in each viewer; score against the suite's own pass marks |
| 13.3 | Seams and stroke adjustment | Synthetic files with abutting shapes and 0-width strokes at fractional zoom; measure seam luminance |
| 13.4 | Page render time and peak memory for large PDFs | 1,000-page text PDF, 100-page scanned PDF, a heavy CAD page; time-to-first-page, time per page, RSS, in each browser |
| 13.5 | Whether Chrome stable now exposes tags | Re-run the IAG test file with NVDA, JAWS and VoiceOver |
| 13.6 | Print path per viewer (raster or vector, DPI) | Print to a PDF printer driver and inspect the output for images versus text operators |
| 13.7 | Default CMYK conversion per viewer | Render DeviceCMYK and ICCBased-CMYK patches; compare to an ICC reference transform |
| 13.8 | Find-in-PDF matching rules (ligatures, diacritics, hyphenation across lines, case) | A test PDF with ﬁ ligatures, combining marks, soft hyphens and line-end hyphens; search each viewer |
| 13.9 | Copy-paste quality per viewer | Copy from test files without ToUnicode, with wrong ToUnicode, with ActualText; diff the clipboard |
| 13.10 | Which annotation types are saved how (incremental update or full rewrite; appearance streams present) | Annotate in each browser, save, inspect the file structure |

## Claims not verified

- 1.6 GWG 5.0 contents and vendor pass claims (trade-press only); 1.7 practitioners' consensus (forums).
- 2.1 Acrobat's overprint-preview default; 2.2, 2.3, 2.5, 4.1, 4.3, 4.6, 5.7, 9.3, 9.5, 11.5, 11.6 ISO 32000-2 section numbers and details (from recollection of the specification, not re-read this session; the spec is the anchor and must be checked); 2.4 conflation-artifact attribution.
- 3.4 pdf.js text layer and structure-tree DOM (vendor tutorial and recollection); 3.5 Chrome's PDFium process and tiling model; 3.8 Edge's move to Adobe's engine.
- 4.2 Adobe's Type 1 end-of-support details (help page refused access); 4.6 PDF 2.0 deprecation of non-embedded standard 14; 4.9 whether URW base35 carries a font exception; 4.10 TeX Gyre licence.
- 5.5 pdfminer parameters; 5.6 ICDAR competitions; 5.9 find-bar behaviour.
- 6.1 PDF/UA dates; 6.2 WTPDF; 6.6 Matterhorn counts.
- 7.5 the Firefox 89 attribution (the source's release date is wrong; the feature version is not confirmed by Firefox release notes); 7.6 Chrome's current tag support.
- 8.3 the Luxembourg and scholarly studies (search snippets only); 8.5 Chrome desktop OCR date and Screen AI resource reports.
- 9.4 viewers' CMYK defaults; 10.1 Firefox print DPI; 10.4 Chrome 85 tagged output (press); 10.5 Chrome print path.
- 11.4 Chrome XFA flag state; 12.5 Chrome annotation history (press and blogs only).
- 1.8, 3.6, 8.6, 12.6 are absences: nothing was found in the budget, which is not proof that nothing exists.
- Unreachable on 19 October 2026 (fact-check FC-R25): the Nottingham learning-technology post (8.5).
