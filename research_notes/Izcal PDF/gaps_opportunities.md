# Izcal PDF: gaps_opportunities

Compiled 19 October 2026. Primary sources were preferred (papers, specifications, maintainers' posts, release notes, issue trackers).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

M-1 note: this file records ideas, dates, measurements and stated reasons. No source code of any browser or PDF engine or library was opened: no pdf.js, PDFium, Poppler, MuPDF, qpdf, hayro, lopdf, pdf-rs, pdf-writer, krilla or Skia file, header, commit, diff or review. Sources are blog posts, release notes, papers, issue-tracker and forum prose, and registry metadata (crates.io and npm JSON, read for licence and package size only). Where a value would only be found in source code, it is listed as a black-box measurement question in Section 12.

**Method notes**
- About 30 searches and fetches were used before the shared web-search budget ran out. After that, only known primary pages were fetched. Several pages refused automated access (pdfa.org presentation and survey PDF, Neowin, Android Authority, Adobe HelpX); claims from them rest on search snippets and are graded Medium or Low.
- The Dual Lab "PDF Trends 2026" series (Common Crawl, June 2026, 20.6 million PDFs) is the most recent large measurement of PDF features in the wild. It is a vendor study (Dual Lab sells PDF accessibility tooling) posted to a mailing list and a blog, not a peer-reviewed paper; its numbers are graded Medium even where read in full.
- The SafeDocs corpus paper (Allison et al.) was not found by search; the corpus page was read instead. No feature-prevalence table from it was found, so JavaScript and XFA prevalence on the web remain **unknown** (Section 1).
- Crate licences were read from the crates.io API on 19 October 2026 (Section 11). The accepted specs already decide most of the generic-crate questions this strand was asked; Section 11 says which.

## 0. Summary of gaps found

| Area | Gap today | Evidence strength | Section |
|---|---|---|---|
| Forms and XFA | Chrome and Edge show XFA's "Please wait..." page; only Firefox renders XFA (since Firefox 93, 5 October 2021) | High for Firefox, Medium for Chrome and Edge | 2 |
| Prevalence | ~9% of web PDFs have interactive forms, ~2% signatures, ~2.5% are encrypted with an empty password; JavaScript and XFA rates on the web are unmeasured in public | Medium | 1 |
| Footprint | No neutral published benchmark of pdf.js against PDFium; the only careful public memory study is from 2014 | High that none was found; Medium that none exists | 3 |
| Accessibility | Tagged share passed 50% in July 2025; Google estimates 12% of web PDFs are inaccessible to screen readers; structure errors are common | Medium | 4 |
| Mobile | Chrome for Android viewed PDFs inline only from April 2025 | Medium | 5 |
| Signatures | 16 of 29 viewers fell to shadow attacks (NDSS 2021); nothing shows users what a signature covers | High | 6 |
| Print-to-PDF | Margin boxes arrived in Chrome 131 (October 2024); running headers, `target-counter` and footnotes are still missing | High | 7 |
| Invisible state | Font substitution, actions and scripts, signature coverage, untagged regions and render differences are not shown by any browser viewer | Medium (absence of evidence) | 9 |

## 1. How much PDF the web carries, and which features it uses

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | The SafeDocs corpus re-fetched every PDF found in one Common Crawl snapshot without Common Crawl's truncation | 8.3 M URLs with a PDF, 7.9 M unique by SHA-256, out of a crawl of 3.15 billion pages; ~8 TB uncompressed; 7,933 ZIPs of up to 1,000 PDFs | [Digital Corpora, CC-MAIN-2021-31-PDF-UNTRUNCATED](https://digitalcorpora.org/corpora/file-corpora/cc-main-2021-31-pdf-untruncated/) | crawl July–Aug 2021 | High |
| 1.2 | Derived: PDF responses were about **0.26%** of crawled URLs in that snapshot (8.3 M / 3.15 B). This is a crawl share, not a share of what users open | 0.26% | derived from 1.1 | 2021 | Medium |
| 1.3 | Common Crawl truncated PDFs at 1 MB in 2021, which is why the SafeDocs corpus re-fetched them. The 2026 Dual Lab study says 5 MB. Studies built on raw Common Crawl undercount large PDFs | 1 MB (2021); 5 MB (2026 claim) | [Digital Corpora](https://digitalcorpora.org/corpora/file-corpora/cc-main-2021-31-pdf-untruncated/); [Dual Lab 2026Q2](https://dev.to/katash/pdf-trends-2026q2-by-dual-lab-company-4c00) | 2021; 2026-08 | High / Medium |
| 1.4 | The SafeDocs corpus licence is not stated on its page; it defers to Common Crawl's terms and Digital Corpora's terms of use. Usable as **test-only input** (any licence permitted), never shipped | — | [Digital Corpora](https://digitalcorpora.org/corpora/file-corpora/cc-main-2021-31-pdf-untruncated/) | 2021 | High |
| 1.5 | Dual Lab analysed every PDF in Common Crawl CC-MAIN-2026-25 | 20,578,394 PDFs, ~38 TB; ~5% excluded for missing or invalid dates; previous wave ~15 M PDFs from CC-MAIN-2026-04 | [Dual Lab 2026Q2](https://dev.to/katash/pdf-trends-2026q2-by-dual-lab-company-4c00) | 2026-08-03 | Medium |
| 1.6 | Most web PDFs are short | 5.8 M single-page; 4.9 M 2–3 pages; 3.4 M 4–7; 2.7 M 8–15 (of 20.6 M); ~8,000 had malformed page trees | same | 2026-08-03 | Medium |
| 1.7 | **Interactive forms** are present in about one PDF in eleven | 1,905,490 (9.26%) | same | 2026-08-03 | Medium |
| 1.8 | **Digital signatures** and **optional content** are minority features but not rare | signatures 419,069 (2.04%); optional content 888,237 (4.32%) | same | 2026-08-03 | Medium |
| 1.9 | **Encryption with an empty open password** (permissions only) | 513,342 (2.5%); 80,062 with unknown passwords excluded; of the encrypted, 91.8% allow printing, 84.1% high-resolution printing, 83.9% accessibility extraction; ~16% of encrypted (0.4% of all) forbid accessibility extraction | same | 2026-08-03 | Medium |
| 1.10 | Annotation counts are dominated by links; widgets (form fields) are second | Link 265.3 M; Widget 20.4 M; Square 6.4 M | same | 2026-08-03 | Medium |
| 1.11 | PDF 1.7 is the most common header version | > 6 M documents; 1.4, 1.5, 1.6 follow (no counts); 81 invalid versions | same | 2026-08-03 | Medium |
| 1.12 | **JavaScript and XFA prevalence on the web are not published** in any source found. Mozilla's 2021 test corpus held ~1,900 PDFs with JavaScript and ~1,200 XFA PDFs, but the denominator is not given; Mozilla said only that telemetry showed "many" forms use JavaScript and XFA was "fairly common" | ~1,900 JS; ~1,200 XFA; ~32,000 tagged (denominator unknown) | [Mozilla Hacks](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021-10-07 | High (that no rate is given) |
| 1.13 | Google estimates **360+ billion PDFs, 12% of all PDFs on the web**, are inaccessible to screen readers. Implies a total of ~3 trillion PDFs; method not published | 360 B; 12% | [ChromeOS.dev, Bernal et al.](https://chromeos.dev/en/posts/frictionless-pdf-accessibility-with-ml-and-ocr) | 2024-09-24 | Medium |
| 1.14 | No browser publishes how many PDFs its users open (share of navigations or of downloads). Firefox's PDF telemetry exists but its counts are not public in any source found | — | [Mozilla Hacks](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021-10-07 | Medium |

**Implications:** forms (~9%), annotations, encryption (2.5%), optional content and signatures are mainstream features that must work at Stage 1 of the viewer; they are not edge cases. JavaScript and XFA rates are a gap Izcal should measure itself on the SafeDocs corpus (test-only) before deciding PD-D for JavaScript-in-PDF and XFA, rather than guessing.

## 2. Forms, JavaScript and XFA in today's browser viewers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | Firefox's viewer gained form filling in steps: AcroForms in Firefox 84, PDF JavaScript in 88, tagged PDF in 89, XFA in 93 | Firefox 93 released 2021-10-05 | [Mozilla Hacks](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021-10-07 | High |
| 2.2 | Mozilla runs PDF JavaScript in a sandbox and **left out the network and disk APIs** of the Acrobat JavaScript API "for security reasons"; scripts are mostly used to validate input and compute fields | — | same | 2021-10-07 | High |
| 2.3 | Firefox's layout engine "couldn't handle" XFA's layout features, so Mozilla wrote a separate XFA layout engine | — | same | 2021-10-07 | High |
| 2.4 | XFA forms often do not embed fonts; Mozilla ships Liberation fonts and rescales glyphs to approximate fonts such as Myriad Pro | — | same | 2021-10-07 | High |
| 2.5 | Filled form values do not print through the annotation layer, so Mozilla draws them onto the canvas for printing: a printing difference that users see as "my answers are missing" when it fails | — | same | 2021-10-07 | High |
| 2.6 | A commenter on that post, a former XFA developer, urged Mozilla to drop XFA, arguing no other implementation will ever be fully compatible | — | same (comments) | 2021-10 | Medium |
| 2.7 | Dynamic XFA in Chrome or Edge shows Adobe's placeholder page "Please wait... If this message is not eventually replaced..."; Adobe's advice is to download and open in Acrobat | — | [Adobe Community](https://community.adobe.com/t5/acrobat-reader/not-able-to-open-dynamic-xml-format-pdf-in-chrome/td-p/10155711); [Orange County Auditor guide](https://ocauditor.gov/wp-content/uploads/2023/09/Viewing-Fillable-PDF-Forms-Workaround-for-Error-Message.pdf) | 2018–2023 | Medium |
| 2.8 | A Microsoft employee said Edge's viewer omitted XFA by design because XFA "is not part of the PDF standard"; a later answer pointed to an `edge://flags` "Enable support for XFA in PDF" flag, which users said showed only the first page | — | [Microsoft Tech Community](https://techcommunity.microsoft.com/discussions/edgeinsiderdiscussions/opening-xfa-compliant-pdf-documents-does-not-work-with-built-in-pdf-player-of-mi/3410714) | 2022–2024 | Medium |
| 2.9 | Government and banking forms are where XFA lives (Firefox 93 release notes: "XFA-based forms, used by multiple governments and banks") | — | [Firefox 93 release notes](https://www.firefox.com/en-US/firefox/93.0/releasenotes/) (via search snippet) | 2021-10-05 | Medium |
| 2.10 | Edge replaced its own PDF engine with Adobe's Acrobat engine: opt-in from March 2023, legacy removal first planned March 2024, then (per a February 2025 Microsoft update reported by Neowin) managed-device rollout September 2025 and removal "early 2026". Current status not confirmed | — | [Adobe/Microsoft press release](https://www.businesswire.com/news/home/20230208005269/en); [Neowin](https://www.neowin.net/news/microsoft-shares-new-details-on-adobe-for-edge-that-replaces-native-legacy-pdf-reader/) (not fetched) | 2023-02-08; 2025-02 | Medium / Low |

**Implications:** the user-visible failure is not "XFA unsupported" but a blank promise ("Please wait") with no explanation. Whatever Izcal decides on XFA, the viewer must say plainly what the file contains and why it cannot be shown, and offer the safe alternatives (save, open elsewhere). Mozilla's choice to drop network and disk APIs from PDF JavaScript is the precedent for any Izcal JavaScript-in-PDF profile.

## 3. Performance and footprint

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | The only careful public memory study of a browser PDF viewer is Nethercote's 2014 pdf.js work. Canvas caching drove memory; on a Retina Mac each page canvas exceeded 20 MiB | Peak RSS over 1,000 MiB on a Mac vs 400 MiB on Linux for the same document; Firefox 28→33 reduced peak RSS 1.2×, 4.4× and 7.8× on 14-, 122- and 226-page documents; cache cut from 20 to 10 canvases | [Nethercote, "An even slimmer pdf.js"](https://blog.mozilla.org/nnethercote/2014/06/16/an-even-slimmer-pdf-js/) | 2014-06-16 | High |
| 3.2 | Nethercote's own conclusion: pdf.js "still uses substantially more memory than native PDF viewers" (no native number given) | — | same | 2014-06-16 | High |
| 3.3 | **No neutral published benchmark** comparing pdf.js and PDFium for time to first page or memory was found. Vendor tables rate them without data | — | search, 19 October 2026 | 2026 | Medium |
| 3.4 | A PDF Association survey of open-source renderers (June 2021) measured accuracy, processing time and memory, including a 1,310-page document at 150 dpi; the PDF refused automated access, so its numbers were not read | — | [PDF Association survey](https://pdfa.org/wp-content/uploads/2021/06/Survey-of-OpenSource-Solutions.pdf) (not read) | 2021-06 | Low |
| 3.5 | Firefox bug 1722807: rendering one PDF could block the browser UI for minutes with large memory use; fixed in Firefox 96 | minutes | [Bugzilla 1722807](https://bugzilla.mozilla.org/show_bug.cgi?id=1722807) (via search snippet) | 2021 | Medium |
| 3.6 | A 2024 Firefox support thread: one oversized page (17.78 × 84.22 in) took about 19 s to render in Firefox while Edge was quick; a responder suspected bitmap size limits | ~19 s | [SUMO 1445689](https://support.mozilla.org/en-US/questions/1445689) (via search snippet) | 2024 | Medium |
| 3.7 | Users report a 100 MB PDF stalling the whole browser and a 68 MB PDF printing so slowly from the built-in viewer it was unusable | — | Firefox support forum threads (via search snippets) | ~2020 | Low |
| 3.8 | pdf.js as a distributable package (`pdfjs-dist` 6.4.299, Apache-2.0) is ~34.9 MB unpacked across 554 files; this is the package, not what Firefox ships, and it includes maps and legacy builds | 34,913,648 bytes | [npm registry metadata](https://registry.npmjs.org/pdfjs-dist/latest) | 2026-10-19 | High (for the package) |
| 3.9 | No published figure was found for the share of Chrome's binary that PDFium and its viewer occupy, nor for Edge's Acrobat-engine component | — | — | 2026 | — |
| 3.10 | Reftests of PDF rendering suffer anti-aliasing noise that creates false regressions, and large reftests are hard to investigate (Mozilla's own account) | — | [Mozilla Hacks](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021-10-07 | High |

**Implications:** footprint numbers for PDF viewers barely exist in public, so Izcal's budgets (FP-38 style) must be set † from first principles and then checked by its own black-box measurements of Chrome, Firefox and Safari on a fixed corpus (Section 12). The 2014 lesson is general: page-raster caches scale with device pixel ratio and dominate memory, so the cache must be budgeted in bytes, not pages. Canonical integer rasterisation (GR-D3) removes the anti-aliasing noise that Mozilla's reftests suffer.

## 4. Accessibility: tags, untagged files, OCR and reflow

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | The share of web PDFs with a structure tree passed **50% in July 2025**, growing ~1.5 points a year (January 2023 to June 2026); reaching 70% would take 10+ years at that rate | > 50%; ~1.5 pp/yr | [Dual Lab, PDF Trends 2026 (Common Crawl group)](https://groups.google.com/g/common-crawl/c/_ttmwLV7wuk) | 2026-10-05 | Medium |
| 4.2 | A tag tree is not correct structure. ~2% of documents have an empty structure tree; 161,081 have zero structure elements | — | same | 2026-10-05 | Medium |
| 4.3 | ISO/TS 32005 nesting violations are common: P directly under StructTreeRoot in ~1.6 M documents, Figure ~1.1 M, Table ~0.95 M, Span directly in TR ~0.86 M | — | same | 2026-10-05 | Medium |
| 4.4 | Conformance claims are rare: ~0.9% claim PDF/UA only, ~2.25% PDF/A only, ~0.07% both; of PDF/UA claimants ~99.3% are PDF/UA-1, ~0.3% PDF/UA-2 | — | same | 2026-10-05 | Medium |
| 4.5 | The most common custom structure type is an empty name, and unmapped custom types (TextBox, InlineShape, Workbook...) number in the tens of thousands of documents each: role maps are often missing | TextBox 55,733 unmapped; Artifact 44,333 | same | 2026-10-05 | Medium |
| 4.6 | Firefox exposes tagged PDF structure to assistive technology since Firefox 89; a commenter noted screen-reader users must switch viewers depending on how a file is tagged, and that VoiceOver does not support the `aria-owns` the approach relies on | — | [Mozilla Hacks](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021-10-07 | High |
| 4.7 | Chrome runs on-device OCR for image-only PDFs (reading mode; ChromeOS Gallery), 77 languages plus 7 scripts | 77 languages | [ChromeOS.dev](https://chromeos.dev/en/posts/frictionless-pdf-accessibility-with-ml-and-ocr) | 2024-09-24 | High |
| 4.8 | Desktop Chrome's searchable, copyable OCR of scanned PDFs was reported rolling out in April 2025 (Global Accessibility Awareness Day); users report the on-demand Screen AI component draining battery | — | [Nottingham learning-technology blog](https://blogs.nottingham.ac.uk/learningtechnology/?p=3111), [Vivaldi forum](https://forum.vivaldi.net/post/868067) (via snippets) | 2025-05 | Low |
| 4.9 | Firefox 130 generates alt text for images added in its PDF editor on the device: ~180 M-parameter ViT + distilled GPT-2, int8, ~180 MB on disk, downloaded on first use, labelled "Created automatically" | ~180 M params; ~180 MB | [Mozilla blog, Ziadé and Safko](https://blog.mozilla.org/en/?p=77294) | 2024-11-04 | High |
| 4.10 | Adobe's **Liquid Mode** (Acrobat Reader mobile, 23 September 2020) reflows PDFs for phones by sending the file to Adobe's Document Cloud for AI structure inference | cloud processing | [TechCrunch](https://techcrunch.com/2020/09/23/adobes-liquid-mode-uses-ai-to-automatically-redesign-pdfs-for-mobile-devices) | 2020-09-23 | High |
| 4.11 | Liquid Mode did not support forms, slideshows, scans, or files over 10 MB or 200 pages at launch; alt text for images could not be generated | 10 MB; 200 pages | [Adobe blog](https://blog.adobe.com/en/publish/2020/12/10/adobe-delivers-enhanced-pdf-reading-experience-for-accessibility); HelpX (via snippets) | 2020-12-10 | Medium |
| 4.12 | No mainstream browser viewer reflows a PDF from its own structure tree; reflow exists only in Acrobat (cloud inference) and in assistive tools | — | absence across sources above | 2026 | Low |

**Implications:** half of web PDFs now carry tags, but many trees are wrong, so a viewer that trusts tags blindly gives screen-reader users a broken tree. Izcal's semantic plane already separates an author tree from a repaired tree (B8); applying the same author-versus-repaired view to PDF structure, with ISO/TS 32005 rule IDs as anchors, is a direct fit. Local reflow from tags (no cloud) is unclaimed ground; OCR stays an ML-service hook under B10 per the frame.

## 5. Mobile PDF viewing

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | Until 2025, Chrome for Android downloaded PDFs and handed them to another app (typically Google Drive) instead of viewing them | — | [Chrome Unboxed](https://chromeunboxed.com/chrome-for-android-finally-has-a-simple-built-in-pdf-viewer/) (via snippet) | 2025-04-27 | Medium |
| 5.2 | Native inline viewing rolled out in April 2025 after sitting behind flags ("Open PDF Inline on Android", and a "pre-V" variant); first seen in February 2024, functional from December 2024; observed on Android 15 | — | [Android Authority](https://www.androidauthority.com/google-chrome-native-pdf-reader-3547778/), [Android Headlines](https://www.androidheadlines.com/?p=1211364), [nextpit](https://nextpit.com/google-chrome-built-in-pdf-viewer-reader-features-availability) (via snippets) | 2025-04-24 | Medium |
| 5.3 | The Android viewer offers search and ink annotation (pen, highlighter, eraser) | — | Android Authority (via snippet) | 2025-04 | Low |
| 5.4 | Server `Content-Disposition: attachment` still forces a download dialog in Chrome 143 beta on Android 15 (user report; not reproduced by the site's staff) | — | [Pipefy community](https://community.pipefy.com/ask-the-community-43/pdf-files-forced-to-download-on-chrome-android-inline-viewing-fails-4864) (via snippet) | 2025-11 | Low |
| 5.5 | Fixed-layout pages on phones remain the main mobile complaint; Adobe's answer was cloud reflow (4.10), not a browser feature | — | 4.10 | 2020 | Medium |

**Implications:** on Android, the browser PDF viewer is new (2025) and partial, so mobile viewing with reflow and forms is open ground. Izcal should state its `Content-Disposition` policy for PDFs explicitly, since sites and users disagree about it.

## 6. Signatures and trust: what users cannot see

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Shadow attacks hide or replace content in signed PDFs using legitimate features (incremental updates, overlays), without viewer bugs | 16 of 29 viewers vulnerable, including Adobe Acrobat and Foxit Reader | [NDSS 2021, Mainka, Mladenov, Rohlmann](https://www.ndss-symposium.org/ndss-paper/shadow-attacks-hiding-and-replacing-content-in-signed-pdfs) | 2021-02 | High |
| 6.2 | The project site gives slightly different shadow-attack counts (15 of 28 desktop viewers, dated July 2020); the paper's 16 of 29 is the citable figure | 15/28 vs 16/29 | [pdf-insecurity.org](https://pdf-insecurity.org/) | 2020–2021 | High (that they differ) |
| 6.3 | The 2019 signature attacks (universal forgery, incremental saving and signature wrapping together) broke 26 of 29 validators (21 of 22 desktop viewers, 5 of 7 online services) | 26/29 | same | 2019-11 | High |
| 6.4 | Certification attacks (IEEE S&P 2021) broke 24 of 26 applications; 11 of 26 mis-applied certification permissions | 24/26; 11/26 | same | 2021-05 | High |
| 6.5 | PDFex encryption attacks: 23 of 27 viewers allowed direct exfiltration; all 27 were vulnerable to CBC malleability gadgets | 23/27; 27/27 | same | 2019-11 | High |
| 6.6 | "Insecure features" study: 26 of 28 applications vulnerable to at least one attack using standard PDF features (actions, forms, external references) | 26/28 | same | 2021-01 | High |
| 6.7 | Across these studies the root cause is the same: the viewer shows a green "signed" state but does not show **which bytes and which visible content** the signature covers, or what was appended after signing | — | synthesis of 6.1–6.6 | 2019–2021 | Medium |

**Implications:** signature coverage is the clearest "invisible today" item with hard evidence behind it. A viewer that renders the signed revision and the current revision separately and shows the visual difference (Izcal owns its display list, so a page-level diff is a display-list diff) answers the shadow-attack class directly. Security detail belongs to the security strand; this strand records only the visibility gap.

## 7. Print-to-PDF: developer pain

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | Chrome 131 added CSS `@page` margin boxes (16 boxes, `page` and `pages` counters) for print and Save as PDF; before that, authors could not put their own headers and footers in page margins in Chrome or Firefox | Chrome 131 | [Chrome for Developers, Rachel Andrew](https://developer.chrome.com/blog/print-margins) | 2024-10-30 | High |
| 7.2 | Still missing in Chrome: `string-set` (running chapter titles), `target-counter` (page-number cross-references), `float: footnote`; the post says paged media "have never been well supported in web browsers" | — | same | 2024-10-30 | High |
| 7.3 | Browser-generated headers and footers still appear when there is room unless the user turns them off; a Chromium layout default suppresses them on later pages if the first page lacks room | — | same | 2024-10-30 | High |
| 7.4 | Chrome 85 began emitting **tagged PDF** from Save as PDF, built from the accessibility tree; headless needs `--print-to-pdf` with `--export-tagged-pdf` | Chrome 85 | [Chromium blog](https://blog.chromium.org/2020/07/using-chrome-to-generate-more.html) (body via snippets); [9to5Google](https://9to5google.com/2020/07/29/chrome-tagged-pdf/) | 2020-07-29 | Medium |
| 7.5 | Puppeteer users report `break-inside: avoid` on table rows failing, especially with `rowspan`; `avoid` is a preference the engine relaxes when a box exceeds a page; `headerTemplate` is separate from repeated table headers | — | [Puppeteer issue mirror](https://lightrun.com/answers/puppeteer-puppeteer-table-borders-are-uneven-on-page-break) and guides (via snippets) | 2020–2025 | Low |
| 7.6 | Developers cannot see where or why a page break fell, which fonts were embedded or subset, or whether output is byte-reproducible; no browser exposes this for print | — | absence across 7.1–7.5 | 2026 | Low |

**Implications:** Izcal's paged layout (LY-D8) and own writer can expose "why did the break fall here" with the same explainers layout uses on screen, and give byte-deterministic output (B1) for regression tests of generated invoices and reports. Margin boxes, `string-set`, `target-counter` and footnotes are where browser print has lagged Prince-class formatters; the layout spec decides which to support.

## 8. Annotation, editing, comparison and dark mode

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | Browser viewers now add editing: Firefox (text, ink, images, alt text by Firefox 130), Chrome on Android (ink), Edge via Acrobat's engine | — | 4.9, 5.3, 2.10 | 2024–2025 | Medium |
| 8.2 | Whether edits and filled fields are saved back into the file (incremental save) or lost on close differs by viewer and is a recurring user complaint; no systematic source was found in budget | — | — | — | Low |
| 8.3 | No browser viewer compares two PDFs; comparison is a paid desktop feature (Acrobat Pro "Compare Files") | — | researcher's recollection | — | Low |
| 8.4 | Dark mode for PDF content is done by colour inversion in several viewers, which corrupts images and colour-coded content; no source measured user satisfaction | — | researcher's recollection | — | Low |

**Implications:** comparison and faithful dark rendering are cheap for an engine that owns its display list and colour pipeline: diff two display lists, or re-map colours by role (text, fill, image) instead of inverting pixels. These are product choices for the spec, not evidence-backed demands; grade them as opportunities, not requirements.

## 9. What is invisible today

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | **Font substitution.** XFA and many AcroForm files do not embed fonts; viewers substitute (Firefox: Liberation, rescaled) without telling the user which font was replaced or by what | — | 2.4 | 2021 | High (that substitution happens); Low (that no viewer shows it) |
| 9.2 | **Actions and scripts.** A PDF can carry JavaScript, launch actions, submit-form targets and external references; 26 of 28 applications were exploitable through such standard features, yet viewers show none of these before they run | 26/28 | 6.6 | 2021-01 | High |
| 9.3 | **Signature coverage** (Section 6) | — | 6.1–6.7 | 2019–2021 | High |
| 9.4 | **Untagged and mis-tagged regions.** Validators (veraPDF, Acrobat's checker) report rule failures, but no browser viewer overlays which page regions are untagged or which structure elements are wrongly nested | ~1.6 M documents with P under root | 4.3 | 2026 | Medium |
| 9.5 | **Why a page renders differently** in two viewers (missing font, unsupported blend mode, overprint, colour management, repaired xref) is not explained by any viewer; differences surface as user bug reports and reftest noise | — | 3.10 and frame | — | Low |
| 9.6 | **Repair.** Damaged files are silently repaired; users and authors never learn the file was broken | ~8,000 malformed page trees in 20.6 M | 1.6 | 2026 | Medium |
| 9.7 | **Permissions.** Encryption permissions (print, extract) are enforced or ignored silently; 0.4% of web PDFs forbid accessibility extraction | 0.4% | 1.9 | 2026 | Medium |

**Implications:** each of these is a fact the parser and interpreter already compute and then discard. Making them visible costs a typed record per document, not new analysis.

## 10. What Izcal could do that no one does

Each row ties a gap to an Izcal capability from `references/izcal-capabilities.md`. These are design options for phase 6, not evidence; the "Source" column names the capability's defining spec.

| # | Opportunity | Gap it answers | Capability | Source | Conf. |
|---|---|---|---|---|---|
| 10.1 | **PDF inspector** (an IIP `pdf.*` domain in DevTools): object graph, xref and revisions, filters, fonts and their substitutes, colour spaces, optional content, annotations, form fields with their scripts, structure tree with ISO/TS 32005 failures, repair log | 9.1, 9.4, 9.5, 9.6 | Spec anchors (ISO 32000-2 clause per object and operator); engine renders its own UI (D-S11) | HGBA-2 M-1, ANC-1; Stage 1 D-S11 | — |
| 10.2 | **Safe-mode view and action ledger**: before any action or script runs, list every action, its trigger, target URL or field, and what the JavaScript profile would allow; run nothing by default from an untrusted origin | 2.2, 9.2, 6.6 | Typed capability channels: the PDF process has no ambient authority, so every action is an IPC request that can be shown and refused | HGBA-3 B2; I-2 | — |
| 10.3 | **Signature coverage view**: render the signed revision and the current one, diff their display lists, and mark appended or covered content; show byte ranges | 6.1–6.7 | Own display list; deterministic rendering | GR-D5/D6; HGBA-3 B1 | — |
| 10.4 | **Deterministic rendering for regression tests**: identical pixels and identical writer bytes on macOS, Linux, Windows and all three ISAs, so PDF generators can test output with exact hashes, and Izcal's own reftests lose anti-aliasing noise | 3.10, 7.6 | Deterministic substrate; canonical integer raster (GR-D3) | HGBA-3 B1, DET-1 | — |
| 10.5 | **Accessible reflow from tags, local**: reflow a tagged PDF from its (repaired) structure tree into an HTML-like view rendered by Izcal, with author-versus-repaired tree shown; no cloud, unlike Liquid Mode | 4.1–4.6, 4.10–4.12, 5.5 | Semantic plane with repair overlay | HGBA-3 B8 | — |
| 10.6 | **Agent reading of PDFs with consent**: agents receive the structure tree and text through the agent principal's tool registry, origin-scoped, audited, off by default | 4.x; agent use of documents | Agent principal and origin capabilities; AI off (B10) | HGBA-3 B9, B10 | — |
| 10.7 | **Record and replay of PDF sessions**: a form-filling bug report that replays exactly (inputs, script execution, layout of XFA if supported) | 2.x | Deterministic substrate | HGBA-3 B1 | — |
| 10.8 | **Print explainers**: "why did the page break here", "which fonts were embedded and subset", "which elements became which tags" for print-to-PDF output | 7.5, 7.6 | Incremental core with reference path; layout explainers; semantic plane | HGBA-3 B5, B8; LY-D8 | — |
| 10.9 | **PDF diff** of two documents (display-list diff plus text diff) | 8.3 | Own display list | GR-D5/D6 | — |
| 10.10 | **Role-aware dark mode**: remap colours by drawing role (text, path fill, image) through the colour pipeline instead of pixel inversion | 8.4 | Own display list and colour pipeline | GR; MD-D28 | — |

**Implications:** 10.1, 10.2 and 10.3 answer evidence-backed gaps (Sections 6 and 9) and fall out of Izcal's architecture with little extra machinery; they are the strongest candidates. 10.5 and 10.6 depend on the accessibility spec's interfaces. 10.9 and 10.10 are product choices with weak demand evidence.

## 11. Generic Rust crates: licences and whether Izcal may use them

Licences were read from the crates.io API on 19 October 2026 (latest stable version shown). No crate source was opened. "Superseded" means an accepted Izcal spec already decided to build its own.

| # | Crate | Version | Licence | Status for the PDF spec | Source | Conf. |
|---|---|---|---|---|---|---|
| 11.1 | `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 | Licence acceptable; **superseded**: Flate goes through `hg-compress` (NW-D6, shared by network, fonts and images) | [crates.io](https://crates.io/crates/miniz_oxide); Izcal networking 6.3 | High |
| 11.2 | `flate2` | 1.1.10 | MIT OR Apache-2.0 | Licence acceptable; superseded as 11.1 | [crates.io](https://crates.io/crates/flate2) | High |
| 11.3 | `zlib-rs` | 0.6.8 | Zlib only | **Excluded** by MIT-only | [crates.io](https://crates.io/crates/zlib-rs) | High |
| 11.4 | `zune-inflate` | 0.2.54 | MIT OR Apache-2.0 OR Zlib | Licence acceptable; superseded as 11.1 | [crates.io](https://crates.io/crates/zune-inflate) | High |
| 11.5 | `weezl` (LZW) | 0.2.1 | MIT OR Apache-2.0 | Licence acceptable; **superseded**: LZW is in `hg-entropy` (images and media spec, L2m). PDF's LZWDecode `EarlyChange` parameter must be supported there | [crates.io](https://crates.io/crates/weezl); Izcal images and media 6.x | High |
| 11.6 | `zune-jpeg` | 0.5.15 | MIT OR Apache-2.0 OR Zlib | Licence acceptable; **excluded by owner direction** (image decoding is engine work; Izcal builds `hg-jpeg`) | [crates.io](https://crates.io/crates/zune-jpeg); Izcal images and media | High |
| 11.7 | `jpeg-decoder` | 0.3.2 | MIT OR Apache-2.0 | As 11.6 | [crates.io](https://crates.io/crates/jpeg-decoder) | High |
| 11.8 | `png` | 0.18.1 | MIT OR Apache-2.0 | As 11.6 (`hg-png`); PDF's PNG predictors are a filter, written in the PDF parser or `hg-compress` | [crates.io](https://crates.io/crates/png) | High |
| 11.9 | `aes` | 0.9.3 | MIT OR Apache-2.0 | Usable through `hg-crypto` (NW-D23; G36 says only `hg-crypto` holds primitives) for AESV2/AESV3 | [crates.io](https://crates.io/crates/aes); Izcal networking NW-D23 | High |
| 11.10 | `cbc` | 0.2.1 | MIT OR Apache-2.0 | Needed for PDF's AES-CBC; not yet in `hg-crypto`'s pin list: amendment candidate | [crates.io](https://crates.io/crates/cbc) | High |
| 11.11 | `sha2` | 0.11.1 | MIT OR Apache-2.0 | Already pinned in `hg-crypto` (SHA-256/384/512 for R6 key derivation and signatures) | [crates.io](https://crates.io/crates/sha2) | High |
| 11.12 | `md-5` | 0.11.0 | MIT OR Apache-2.0 | Needed for R2–R4 security handlers; not in `hg-crypto`'s pins: amendment candidate, legacy-only | [crates.io](https://crates.io/crates/md-5) | High |
| 11.13 | `rc4` | 0.2.0 | MIT OR Apache-2.0 | Needed for R2–R4 (RC4 40/128); not in `hg-crypto`: amendment candidate, legacy-only; or Izcal writes RC4 itself (trivial size) | [crates.io](https://crates.io/crates/rc4) | High |
| 11.14 | `ttf-parser` | 0.25.1 | MIT OR Apache-2.0 | Licence acceptable; **excluded**: fonts spec makes `hg-fontfmt` the only reader of font bytes (G25) and excludes font libraries | [crates.io](https://crates.io/crates/ttf-parser); Izcal fonts and text | High |
| 11.15 | `read-fonts`, `skrifa` (Fontations) | 0.45.1; 0.48.0 | MIT OR Apache-2.0 | Excluded by name in the fonts spec (Fontations is browser-engine work) | [crates.io](https://crates.io/crates/skrifa); Izcal fonts and text | High |
| 11.16 | `subsetter` (typst) | 0.2.6 | MIT OR Apache-2.0 | Licence acceptable; excluded: font subsetting for print-to-PDF is engine work and G25 applies | [crates.io](https://crates.io/crates/subsetter) | High |
| 11.17 | `lopdf` | 0.45.0 | MIT | **Excluded**: PDF engine library (M-1); landscape or black-box oracle only | [crates.io](https://crates.io/crates/lopdf) | High |
| 11.18 | `pdf` (pdf-rs) | 0.10.0 | MIT | Excluded as 11.17 | [crates.io](https://crates.io/crates/pdf) | High |
| 11.19 | `hayro` | 0.8.0 | Apache-2.0 OR MIT | Excluded as 11.17 (PDF rasteriser) | [crates.io](https://crates.io/crates/hayro) | High |
| 11.20 | `hayro-jpeg2000`, `hayro-jbig2` | 0.4.1; 0.3.1 | Apache-2.0 OR MIT | Excluded: PDF-engine-project codecs; Izcal builds `hg-jpx`, `hg-jbig2` (images and media R18) | [crates.io](https://crates.io/crates/hayro-jbig2) | High |
| 11.21 | `fax` (pdf-rs project, CCITT) | 0.3.0 | MIT | Excluded as 11.20; Izcal builds `hg-ccitt` | [crates.io](https://crates.io/crates/fax) | High |
| 11.22 | `pdf-writer` (typst) | 0.15.0 | MIT OR Apache-2.0 | Excluded: PDF writer is engine work in scope of this spec | [crates.io](https://crates.io/crates/pdf-writer) | High |
| 11.23 | `krilla` | 0.8.2 | MIT OR Apache-2.0 | Excluded as 11.22 (high-level PDF and tagged-PDF writer) | [crates.io](https://crates.io/crates/krilla) | High |
| 11.24 | `printpdf` | 0.12.8 | MIT | Excluded as 11.22 | [crates.io](https://crates.io/crates/printpdf) | High |
| 11.25 | `pdfium-render` | 0.9.4 | MIT OR Apache-2.0 (wrapper); PDFium itself is BSD-3-Clause | Excluded: wraps PDFium (browser engine library, not MIT) | [crates.io](https://crates.io/crates/pdfium-render) | High (wrapper); Medium (PDFium licence, from recollection) |
| 11.26 | `jpeg2k`, `openjpeg-sys` | 0.10.1; 1.0.12 | MIT/Apache-2.0 (wrapper); BSD-2-Clause (bindings) | Excluded: bind OpenJPEG (C, BSD-2) | [crates.io](https://crates.io/crates/openjpeg-sys) | High |
| 11.27 | `jbig2dec` | 0.3.1 | GPL-3.0 | Excluded: GPL | [crates.io](https://crates.io/crates/jbig2dec) | High |
| 11.28 | `pdfjs-dist` (npm, for comparison) | 6.4.299 | Apache-2.0 | Excluded; black-box oracle only | [npm](https://registry.npmjs.org/pdfjs-dist/latest) | High |

**Implications:** after the accepted specs, the PDF spec needs almost no new third-party crates. The only new generic needs are legacy cryptography for the standard security handler (`md-5`, `rc4`, `cbc`, all MIT OR Apache-2.0), which belong in `hg-crypto` by G36 and need an amendment to the networking spec's pin list. Everything else (inflate, LZW, JPEG, JPX, JBIG2, CCITT, fonts, subsetting, writing) is Izcal's own under accepted decisions.

## 12. Questions for black-box measurement

Values that no prose source gave, to be measured by Izcal against shipping browsers (black box only):

| # | Question | Method |
|---|---|---|
| 12.1 | Time to first page and peak memory of Chrome (PDFium), Firefox (pdf.js), Safari (PDFKit) and Edge (Acrobat engine) on a fixed corpus: 1-page, 100-page, 1,000-page, scanned, form-heavy, and an oversized page | Same machine, cold and warm, RSS sampled per process |
| 12.2 | On-disk size of each browser's PDF component | Compare installed sizes with and without the PDF component where builds allow; otherwise file sizes of identifiable resources |
| 12.3 | Share of web PDFs with JavaScript, XFA (static and dynamic), launch or submit actions, embedded files, and non-embedded fonts | Izcal's own parser over the SafeDocs corpus (test-only input) |
| 12.4 | How each viewer handles a shadow-attack and incremental-update corpus in 2026 | pdf-insecurity.org test files |
| 12.5 | Whether filled form values and annotations survive save and reopen in each browser | Scripted round-trip on a form corpus |

## Claims not verified

- 1.5–1.11, 4.1–4.5: Dual Lab figures come from a vendor's blog and mailing-list posts; methodology was not independently checked, and the tagged-share growth is stated both as "percentage points" and "%".
- 1.13: Google's 360 billion and 12% are stated without method.
- 2.7–2.8: Chrome and Edge XFA behaviour rests on forum posts from 2018–2024; current Edge behaviour under the Acrobat engine was not checked and may now render XFA.
- 2.10: the Edge engine-replacement timeline after February 2025 was not confirmed; the Neowin page refused access.
- 3.4: the PDF Association 2021 survey's numbers were not read.
- 3.5–3.7: Firefox performance items are from search snippets of Bugzilla and support pages.
- 4.8: desktop Chrome OCR date (April 2025) rests on a university blog, not Google release notes.
- 4.11: Liquid Mode limits are from search snippets; the HelpX page refused access.
- 5.1–5.4: Chrome for Android dates come from press reports read as snippets; no Chrome release note gives a version.
- 7.4: Chrome 85 tagged-PDF details come from snippets; the Chromium blog body did not load.
- 7.5: Puppeteer pain is from issue mirrors and tutorials, not the Puppeteer tracker itself.
- 8.2–8.4, 11.25 (PDFium's BSD-3-Clause licence): researcher's recollection, not read this session.
- No source was found for the share of browser navigations or downloads that are PDFs, for Chrome or Firefox feature-request vote counts on PDF issues, or for user surveys about browser PDF viewers; these remain unknown.
- Unreachable on 19 October 2026 (fact-check FC-R25): the Nottingham learning-technology post (4.8).
