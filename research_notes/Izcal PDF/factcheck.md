# Izcal PDF: fact-check (phase 8)

Checked 19 October 2026 against `reports/Izcal PDF.md` (draft 0.1) and its seven research-note files, by a checker who wrote neither the spec nor the notes. Locations are given by **section** (and by PD-, G- or Q- ID where there is one), not by line.

## Method

The claims that carry a decision (PD-D1 to PD-D26), a target (PD-1 to PD-61), a rule (G110 to G119) or the history were checked against primary sources, read directly where they could be reached:
- **Papers, read in full text** (downloaded and converted locally, Poppler `pdftotext` used only as a program): Müller et al., "Processing Dangerous Paths" (NDSS 2021); Mainka, Mladenov and Rohlmann, "Shadow Attacks" (NDSS 2021); Kuchta et al. (EMSE 2018).
- **NVD 2.0 API**, queried 19 October 2026: the "pdfium" keyword set (135 records, re-counted by year, CWE and description keyword), the "PDF in Google Chrome" exact-match set (31 records), and CVE-2026-110136, CVE-2024-4367, CVE-2018-5158, CVE-2015-4495 and CVE-2021-30860.
- **Standards and errata**: the PDF Association's ISO 32000-2:2020 errata pages for clauses 7, 8, 9, 10, 11, 12 and 14 (every heading extracted and compared with the clause numbers the spec cites); the HTML Standard (system-state and timers-and-user-prompts pages); MIME Sniffing; the PDF-AAM Editor's Draft (w3c.github.io).
- **Registries and release data**: the crates.io API (licence and version of every crate the spec names as shipped or excluded); chromiumdash's milestone schedule for Chrome 131; the Firefox 88 and 93 release notes; pdf-insecurity.org.
- **Izcal's own documents**: every cross-reference in the spec was checked against the accepted specs, the reconciliation notes, the licence audit and the Phase 3 ledger in this worktree, and against the security scope and coding standard in the main repository (a helper did this sweep and reported each ID with its source text; the checker reviewed the findings).

Five web searches were used (two for claims below, the rest by the helper's sweep through local files only). No browser or PDF-engine source code was opened (M-1 part 3). One search result for ISO 21757-1 was a PDFium source file; it was **not opened** and nothing here rests on it. No personal data was sent in any request.

Verdicts:
- **Correct.**
- **Partly**: the substance holds, but a number, scope, date, attribution or wording is off.
- **Wrong.**
- **Unverifiable**: the cited source does not contain the claim, or could not be reached, and no other source was found.

**Must-fix (FC-M#)** marks a wrong or unsupported fact that a decision, target or rule depends on, or a plain error in a load-bearing sentence. **Recommended (FC-R#)** marks every other fix.

## Summary

| Verdict | Count |
|---|---|
| Correct | 90 |
| Partly | 27 |
| Wrong | 10 |
| Unverifiable | 5 |
| **Total claims** | **132** |

The total is the 114 numbered claims plus the 18 cross-reference rows graded wrong or partly in the cross-reference table (5 wrong, 13 partly). The roughly 70 cross-references found correct are listed in one paragraph and not counted.

Must-fix: 10. Recommended: 26.

The research notes are careful and mostly faithful; the spec mostly reflects them. The serious problems are: a keyword-matching artefact in the security note that the spec turned into "forms (24)" PDFium bugs (FC-M1); a staffing total that does not add up (FC-M2); the PDF 2.0 accessibility-permission bit, which the standard itself tells readers to ignore, so Q-PD-7 is not a deviation (FC-M3); five Izcal cross-references that point at the wrong ID or section (FC-M4 to FC-M7); the G110–G119 range presented as assigned when no registry assigns it (FC-M8); and two attack and cookie statements that overstate (FC-M9, FC-M10).

## Claims

Abbreviations: Sum = summary before Section 1; S1 to S11 = spec sections.

### History, landscape and attack evidence

| # | Claim (spec) | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 1 | Reader files in almost half of 663 targeted attacks (F-Secure, 2009) | Sum; 2.1 | Correct | HP 5.1 (The Register); page not reachable from this host | As the note says (49%), Medium |
| 2 | Acrobat JavaScript API carried many of the worst bugs (`util.printf`, `Collab.getIcon`) | Sum; 2.1 | Correct | NVD CVE-2008-2992, CVE-2009-0927 via SE 2.1–2.2 | — |
| 3 | Reader X Protected Mode (November 2010), "not a panacea" | 2.1; 2.2 | Correct | HP 5.4 (Threatpost) | Medium in the note |
| 4 | FORCEDENTRY: JBIG2 used as a weird machine; decoder ran outside BlastDoor | Sum; 2.1; 5 | Correct | SE 2.9–2.11 (Project Zero); NVD CVE-2021-30860 | 5's "outside the sandbox" should say "outside BlastDoor" (FC-R6) |
| 5 | pdf.js shipped arbitrary script execution through its font path in 2024 (CVE-2024-4367), releases April 2014 to April 2024 | Sum; 3 | Correct | NVD CVE-2024-4367; SE 2.14 (Codean Labs) | — |
| 6 | Google took Foxit's code; Apple uses its OS library; Edge moved to Adobe's engine in 2023; Ladybird bundled pdf.js in 2026 | Sum; 2.1 | Correct | HP 6.2–6.3, 8.2, 8.4, 8.6 | — |
| 7 | Microsoft's five reasons: fidelity, performance, security, text selection, read-aloud | Sum; 1; 5 | Correct | HP 8.2 (MC512918 text, High) | — |
| 8 | Acrobat 1.0 / PDF 1.0 on 15 June 1993; unprofitable about four years; "nobody got it" | 2.1 | Correct | HP 1.3–1.4 | Date Medium |
| 9 | Free Reader with Acrobat 2.0 (1994) | 2.1 | Correct | HP 1.5 (founders' account) | Wikipedia says 1996; note prefers founders |
| 10 | PDF 1.1–1.7 feature table (encryption 1.1 … AES and OpenType 1.6) | 2.1 | Correct | HP 2.1–2.6 | Wikipedia only (Low) |
| 11 | XFA from JetForm, in PDF 1.5, never ISO, deprecated in PDF 2.0, Firefox's XFA still partial in 2026 | 2.1; 6.18 | Partly | HP 9.1–9.5; FS 8.5 | Deprecation rests on vendor sources (FS 8.5 Low), no ISO clause read. Add to 11.2 (FC-R13) |
| 12 | NPAPI removed from Chrome (2015) and Firefox (2017) | 2.1 | Correct | HP 5.5 | Firefox 52 kept Flash; fine at this level |
| 13 | Adobe's patent licence covers ISO 32000-1 only, excludes "any updated version" | 2.1; 10 | Correct | FS 1.7 (licence read in full) | — |
| 14 | Chrome 8 viewer: licensed Foxit engine, sandboxed plugin | 2.1 | Correct | HP 6.1–6.2 | — |
| 15 | Chrome Print Preview design: Windows had no and Linux poor PDF generation; "consistent and size-efficient PDFs" | 2.1 | Correct | HP 11.1 (Chromium design doc, prose) | — |
| 16 | Chrome's writer is "Skia/PDF m155"; Firefox moving to the same backend in 2026 | 2.1 | Correct | WP 1.1, 2.9–2.10 | Linux and macOS only; Windows still Cairo (WP 2.10). FC-R7 |
| 17 | pdf.js: "enlarges the trusted code base"; JS "immune to this class of problems"; Firefox 19 default | 2.1 | Correct | HP 7.2–7.3 | Firefox 19 Medium |
| 18 | pdf.js memory 2014: peak cut 1.2× to 7.8×; Retina page canvas > 20 MiB | 2.1; 3; 5 | Correct | GO 3.1 (Nethercote) | — |
| 19 | ISO 32000-2 deprecates XFA, Movies and Sounds, security handler R1–R5 | 2.1 | Partly | Errata clause 7 ("revisions 1, 2, 3, 4 and 5 is deprecated"); FS 10.1 (13.3–13.4 deprecated) | R1–R5 and sounds/movies confirmed; XFA as above (#11) |
| 20 | Free of charge since 5 April 2023 (Adobe, Apryse, Foxit) with Errata Collection 3 | 2.1; 4.2 | Correct | FS 1.1–1.2 | pdfa.org 403 to this host |
| 21 | Almost no web PDF is 2.0 | 2.1 | Partly | FS 2.7 | Evidence is a 1,045-row sample of the 2021 crawl. Write "none of a 1,045-file sample of the 2021 crawl was PDF 2.0" (FC-R8) |
| 22 | SafeDocs: 20 parsers over 1.04 million files "disagreed widely" | 2.1; 5 | Partly | FS 2.5 (LangSec 2023) | The paper reports features and rules, not a disagreement rate. Write "20 parsers over 1.04 million files showed many behaviour differences" |
| 23 | Chrome 85 (2020) writes tags from "Save as PDF" | 2.1; 5 | Correct | HP 11.4; WP 1.14 | Blog body not read (Medium) |
| 24 | Firefox forms 84, JS 88, tags 89, XFA 93 | 2.1; 3 | Correct | Firefox 88 and 93 release notes; Mozilla Hacks | 93 notes: "XFA-based forms, used by multiple governments and banks" |
| 25 | WTPDF and PDF/UA-2 in 2024 | 2.1 | Correct | HP 13.3; FS 11.3 | — |
| 26 | Typst 0.14 (2025): tagged by default, PDF/UA-1 and every PDF/A part, refuses PDF/UA when alt text missing | 2.1; 5 | Correct | WP 3.1–3.3, 4.2 | — |
| 27 | Typst and Prince show tagged-by-default output is routine | 3 | Partly | WP 3.4 | Prince tags only for PDF/A-a and PDF/UA profiles or with `--tagged-pdf`. FC-R9 |
| 28 | Ladybird bundled pdf.js (April 2026) | 2.1 | Correct | HP 8.6 | — |
| 29 | NVD: 135 "pdfium" records (6 in 2014, 23 in 2019, 1 in 2022, 23 in 2026) | 3 | Correct | NVD API re-count | Exact |
| 30 | CWE tags: 43 UAF, 29 OOB write, 13 integer overflow, 11 heap overflow | 3 | Correct | NVD API re-count | Exact |
| 31 | Descriptions point at JPEG 2000 (18), JPEG (19) and forms (24) | 3; SE 1.3 | **Wrong** | NVD API re-count | No description contains the word "form" or "forms"; the 24–25 hits are substrings ("platform", "information", "perform"). Every "JPEG" hit is JPEG 2000 (OpenJPEG, `opj_*`, JPX). FC-M1 |
| 32 | Chrome "PDF" component: 31 records, 18 UAF; one 2026 advisory "outside the sandbox" | 3 | Correct | NVD API; CVE-2026-110136 (13 Oct 2026, Critical) | — |
| 33 | pdf.js CVE-2015-4495 exploited in the wild to upload local files | 3 | Correct | NVD; SE 2.12 (Mozilla blog) | — |
| 34 | CVE-2018-5158: script injection through Type 4 functions compiled to JS | 3 | Correct | NVD | — |
| 35 | Edge's Adobe engine: stated hardening, no published sandbox design | 3 | Correct | SE 9.5 | — |
| 36 | Three recurring patterns over 18 years (JBIG2 2009 and 2021; SING 2010, BLEND 2015, ATM 2020; generated code) | 3 | Correct | SE 2.4–2.14, 3.1–3.3 (NVD, Project Zero) | — |
| 37 | Müller et al.: 28 applications, 209 files | 3 | Correct | NDSS 2021 paper | — |
| 38 | 17 contacted an attacker's server on open | 3 | Correct | Paper §VII-C ("Altogether, 17 PDF applications") | — |
| 39 | 11 leaked form data silently on print or close | 3 | Correct | SE 4.6 from the paper | — |
| 40 | 12 of 18 Windows viewers leaked NTLM hashes | 3 | Correct | SE 4.5 (paper read in full by the note) | Table not re-extracted cleanly here |
| 41 | 6 launched an executable with no dialog; 532 of 294,586 (0.18%) use Launch | 3; 6.13 | Correct | Paper §VII-E, Table V | — |
| 42 | 578-byte chained Flate stream → 10 GB (18,470,265:1); Chrome and Firefox "fully affected" | 3; 6.3 | Correct | Paper §VII-A | — |
| 43 | Per-tab processes stopped infinite loops, not memory exhaustion | 3 | Correct | Paper: loops "limited" in Chrome, Firefox, Opera; bomb fully affected Chrome and Firefox | — |
| 44 | Safari and Edge (2020) blocked every dangerous path except a clicked link | 3; 6.13 | Correct | Paper §IX-C ("Two positive examples … Safari and Edge") | — |
| 45 | 26 of 28 applications fell to standard-feature attacks | 1; 3.1 | Correct | Paper Table I | — |
| 46 | 63 of 94 edge cases rendered differently | 3; 3.1; 6.23 | Correct | Paper §VII-D | — |
| 47 | Acrobat exposes 6,742 JS properties, PDF-XChange 114 | 3; 6.17 | Correct | Paper Table (6742; 114) | — |
| 48 | "Sandbox it, similar to a tab" (NDSS authors) | 6.1 | Partly | Paper §IX-B | Not verbatim. Paper: "limiting the resources to be consumed by a single document, by sandboxing it – similar to a tab in a modern web browser". FC-R10 |
| 49 | Shadow attacks: 16 of 29 (paper), 15 of 28 (site) | 1; 3; 3.1; 4.4 | Correct | Shadow paper Table II ("16 out of 29"); pdf-insecurity.org ("15 out of 28 desktop") | — |
| 50 | "incremental-update attacks 21 of 22 desktop viewers" | 1 (row 4) | Partly | pdf-insecurity.org ("21 out of 22 desktop viewer applications") | 21/22 is the 2019 study across universal forgery, incremental saving and wrapping, not incremental saving alone (ISA alone: 12 Windows, 5 Linux, 7 macOS, SE 5.2). FC-M9 |
| 51 | Signature forgery 21 of 22; certification 24 of 26; PDFex 23 of 27 | 3 | Correct | pdf-insecurity.org; SE 5.1–5.5 | 24/26 and 23/27 not re-matched on the page text here (Medium) |
| 52 | Kuchta: 13.5% of 2,313 US government PDFs differ in at least one of eight readers | 1; 3; 5 | Correct | Paper ("314 out of 2,313 (13.5%)"; "eight graphical PDF readers") | First page only |
| 53 | Missing-font substitution "has no standardized correct answer" (quoted) | 1 | Partly | Paper §4.1 | Not verbatim: "there is no standardized behaviour on the correct rendering of a missing font". FC-R11 |
| 54 | Kuchta: "Acrobat Reader also has bugs" | 1; 5 | Correct | Paper | — |
| 55 | Bast and Korzen: 14 tools; Type 3 and mandatory hyphens defeat all | 5; 6.9 | Correct | RT 5.2–5.4 (paper read in full by the note) | — |
| 56 | No browser has a published result on the Ghent Output Suite's 48 patches | 3; PD-3 | Partly | RT 1.6 (trade press, Medium) | "48 patches" is from trade press; confirm on the GWG download before PD-3 is fixed (FC-R12) |
| 57 | Chrome 101 and Edge 101 ignored tags (May 2022); Safari, Preview, Acrobat best; Firefox partly | 1; 3 | Correct | RT 7.1–7.4 (IAG) | — |
| 58 | Chrome OCRs image-only PDFs on device, 77 languages; Google's 12% image-only estimate | 1; 3; 6.10 | Correct | RT 8.4; GO 1.13, 4.7 | Google claim, no method (11.2 says so) |
| 59 | PDF-AAM is an Editor's Draft dated 12 October 2026; maps headings, links, sections, articles (and asides, divisions); P, Table, Figure, L, Artifact, ActualText, Alt, Lang open | 2.1; 3; 4.2; 6.10 | Correct | PDF-AAM ED read: role tables cover Art, Aside, Div, Document, Hn, Lbl, Link, NonStruct, Part, Sect, Sub only | — |
| 60 | WCAG 2.2 PDF techniques PDF1–PDF23 | 4.2 | Correct | RT 6.5 (W3C) | — |
| 61 | Forms 9.26% of 20.6 million Common Crawl PDFs; signatures 2%, optional content 4%, encryption 2.5%; links 265 M, widgets 20 M | 1; 3; 11.2 | Correct | GO 1.5–1.10 (Dual Lab, vendor) | Flagged single-source in 11.2 |
| 62 | 0.4% forbid accessibility extraction | 6.5 | Correct | GO 1.9 | — |
| 63 | Tagged share passed 50% July 2025, +1.5 points a year; 2% empty; 0.9% claim PDF/UA | 3; 2.1 | Correct | GO 4.1–4.4; RT 8.1–8.2 | Different Dual Lab wave (20.5 M, 2023–2026) from the feature counts (20.6 M); say so in 11.2 (FC-R13) |
| 64 | "P directly under the root in about 1.6 million documents" | 1 | Correct | GO 4.3 | — |
| 65 | "Half of tagged files break ISO/TS 32005's nesting rules somewhere" | 6.10 | **Wrong** | GO 4.3 | No source gives a share. GO 4.3 gives counts (P under root ~1.6 M, Figure ~1.1 M, Table ~0.95 M, Span in TR ~0.86 M). FC-R1 |
| 66 | Mozilla telemetry: "many" forms use JavaScript, mostly validation and calculation; XFA "fairly common" | 1; 6.17 | Correct | HP 7.6, 10.4 | — |
| 67 | Firefox's PDF JS subset left out network and disk APIs; Edge runs none | 6.17 | Correct | HP 10.3; RT 11.3 (Microsoft Learn: "JavaScript forms aren't supported") | — |
| 68 | XFA "Please wait..." in Chrome and Edge | 1; 6.18 | Correct | GO 2.7–2.8 | Forum and agency evidence (Medium) |
| 69 | Chrome got `@page` margin boxes in Chrome 131 (October 2024) | 1; GO 0, 7.1 | Partly | chromiumdash: early stable 6 Nov 2024; the blog post is 30 Oct 2024 | Chrome 131 shipped in **November** 2024. FC-R2 |
| 70 | Chrome lacks running headers and `target-counter()` | 1 | Correct | WP 8.2; GO 7.2 (Chrome blog) | — |
| 71 | Android Chrome added inline viewing only in 2025 | 4.4 | Correct | GO 5.1–5.2 (press, Medium) | — |
| 72 | Liquid Mode reflows in Adobe's cloud, with limits | 3.1 | Correct | GO 4.10–4.11 | — |
| 73 | Mozilla's PDF reftests suffer anti-aliasing noise | 1; 2.2; 3.1 | Correct | GO 3.10 (Mozilla Hacks) | — |
| 74 | pdf.js text layer needs `aria-owns`, which VoiceOver does not support | 4.4; 6.1 | Partly | HP 7.7 | The VoiceOver statement is from a commenter on the Hacks post, not Mozilla. Write "commenters note" |
| 75 | Linearization: no measured benefit found | 4.4; 5; 6.2 | Correct | HP 12.1–12.4 | An absence (Low); the spec says so |

### Print-to-PDF measurements (WP black-box rows)

| # | Claim | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 76 | Chrome 155: tagged PDF 1.4, links, JPEG passed through, no XMP, no PDF/A or PDF/UA claim, outline off by default | 1; 3; 6.19 | Correct | WP 1.1–1.6 | Measured by the note on `local`; not re-measured here (the evidence rows are specific: byte search, `pdfinfo` output, sizes) |
| 77 | Chrome: Type 3 for variable fonts, collection fonts and colour emoji; emoji text lost; sequential subset tags | 1; 3 | Correct | WP 1.9–1.11 | — |
| 78 | Chrome runs differ only in dates; no `/ID` | 1; 3 | Correct | WP 6.4 | — |
| 79 | WebKit: untagged, no links, JPEG re-encoded 3.4× larger, emoji as 32-px images, one wrong code point, random `/ID` | 1; 3 | Correct | WP 2.2–2.7 | WebKit API path, not Safari's menu (Medium for Safari) |
| 80 | Chrome and WebKit use neither object streams nor xref streams | 6.19 | Correct | WP 1.13, 7.4 | — |
| 81 | PDF 2.0 has no variable-font form | 6.19 | Unverifiable | WP 5.5 (presentation abstract) | Already in 11.2 |
| 82 | OS/2 fsType: version-dependent precedence; 0x0002 restricted; 0x0100 no subsetting; 0x0200 bitmap only | 6.19; Q-PD-5 | Correct | OpenType 1.9.1 OS/2 (`#fstype` fragment present) | — |
| 83 | SOURCE_DATE_EPOCH as pdfTeX and Typst do | 6.19 | Correct | WP 6.1–6.3 | pdfTeX Medium |
| 84 | Firefox prints a viewed PDF as a 150 dpi raster | 6.16 | Unverifiable | RT 10.1 (recollection) | Already Low in the text and 11.2 |
| 85 | Acrobat's overprint preview defaults to PDF/X only | 6.7 | Unverifiable | RT 2.1 (recollection) | Already Low |
| 86 | "a claim on unverified judgement is what the PDF Association criticised in other producers" | 6.19 PD-D18 (c) | Unverifiable | No note row says this | Source it or delete (FC-R14) |

### Standards, clause numbers and the web platform

| # | Claim | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 87 | HTML "PDF viewing support" (`pdfViewerEnabled`, fixed `navigator.plugins` list) at `#pdf-viewing-support` | 4.2; 6.1 | Correct | HTML Standard: `<h5 id=pdf-viewing-support>` 8.10.1.6, "PDF viewer plugin objects" | Fragment exists |
| 88 | MIME Sniffing's `%PDF-` signature at `#identifying-a-resource-with-an-unknown-mime-type` | 4.2 | Correct | MIME Sniffing: id present; pattern `25 50 44 46 2D` in that table | — |
| 89 | A PDF served as `text/plain` is not sniffed into the viewer | 6.15 | Correct | MIME Sniffing: the text-or-binary rules yield only `text/plain` or `application/octet-stream` | — |
| 90 | `window.print()`: `beforeprint`/`afterprint`, printing steps | 4.2 | Correct | HTML `#printing` (id present); WP 8.7 | — |
| 91 | Clause numbers confirmed on errata pages: 7.2, 7.3, 7.3.8.2, 7.3.10, 7.4.4.3, 7.5.4–7.5.8.4, 7.6.5, 7.6.6, 7.11, 8.4.2, 8.5.3, 8.6.5.8, 8.7.3, 8.7.4.5.8, 8.11, 9.6.2.2, 9.6.4, 9.7.5.2, 9.10.3, 10.4.2, 10.6, 11.3.5, 11.6.5, 11.6.6, 12.2, 12.3.2, 12.3.3, 12.4.2, 12.5, 12.6.4.8, 12.7, 12.7.4.3, 12.8, 14.7, 14.7.4, 14.8, 14.8.6, 14.9.2, 14.9.4, 14.11.5, 14.13, Annexes L and Q | 6.3–6.14, 6.25 | Correct | Errata pages, headings extracted | — |
| 92 | Clause numbers **not** on any errata page: 8.4.3.2 (line width), 8.6.7 (overprint), 8.7.4.5.2–8.7.4.5.7, 9.6.5.4 (TrueType cmap selection), 10.7.5 (stroke adjustment), 11.3.4, 11.4.5, 11.4.6, 11.7.4, 14.8.2.3, 7.4.2/7.4.5–7.4.8/7.4.10 | 6.3–6.10 | Partly | Errata pages | Consistent with ISO 32000-1's numbering and with neighbouring confirmed numbers, so probably right; but 11.2's Medium list omits most of them. FC-R3 |
| 93 | A dangling reference is `null` (7.3.10) | 6.3 R6 | Correct | 7.3.10 confirmed; rule is ISO 32000's long-standing text | — |
| 94 | 16 blend modes, the same set as Compositing 1 | 6.6 | Correct | Compositing 1 lists 16 (12 separable + 4 non-separable) | — |
| 95 | Annex Q is the normative test for transparency on a page | 6.6 | Correct | Errata index (Annex Q, normative) | — |
| 96 | A processor "shall support" Adobe-CNS1-7, GB1-5, Japan1-7, KR-9 (9.7.5.2) | 6.8 | Correct | FS 7.3 (errata clause 9) | — |
| 97 | Adobe `cmap-resources` and the Adobe Glyph List are BSD-3-Clause | Sum; 6.8; 7 | Correct | RT 4.4–4.5 (GitHub licence metadata) | — |
| 98 | No MIT metric-compatible substitute for the standard 14 exists | Sum | Partly | RT 4.10 | "None was found" (TeX Gyre not checked). FC-R15 |
| 99 | ISO 32000-2's accessibility-extraction permission bit is "honoured only as a label" by Izcal as a harm-avoidance choice (Q-PD-7) | 6.5; Q-PD-7 | **Wrong** | Secondary sources (GrapeCity API docs, iText forum) quoting ISO 32000-2 Table 22 | PDF 2.0 deprecates bit 10 and tells readers to ignore it; ignoring it **conforms**, it is not a deviation. Medium (primary text not read). FC-M3 |
| 100 | R1–R5 deprecated; R6 AES-256; AES-GCM in ISO/TS 32003 | 6.5 | Correct | Errata clause 7; FS 4.6 (snippet) | — |
| 101 | PDF/A forbids encryption | 6.5 | Correct | WP 4.1 (Prince) | — |
| 102 | ISO 21757-1 defines the AF functions and field semantics the form-logic profile anchors to | 4.2; 6.17; 7 | Unverifiable | Search found only the standard's catalogue pages | The AF functions (`AFNumber_Format` etc.) are historically from Adobe's Acrobat forms documentation; whether ISO 21757-1 includes them is not shown. FC-R16 |
| 103 | ETSI EN 319 142-1 V1.2.1; ISO/TS 32001/32002 add SHA-3 and EdDSA | 6.14; 7 | Correct | FS 5.3–5.4 | Medium (snippets) |
| 104 | ByteRange covers the whole file except `Contents`, to that revision's `%%EOF` | 6.14 | Correct | FS 5.2 (errata clause 12) | — |
| 105 | SHA-256/384/512 and AES are already in `hg-crypto` | 6.5 | Partly | NW spec Section 7 | NW pins `sha2` (SHA-256/384) and `aes` via `aes-gcm`; SHA-512 is new. FC-R17 |

### Licences and crates

| # | Claim | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 106 | `md-5` 0.11, `rc4` 0.2, `cbc` 0.2, MIT OR Apache-2.0 | 6.5; 7 | Correct | crates.io API: md-5 0.11.0, rc4 0.2.0, cbc 0.2.1, all "MIT OR Apache-2.0" | — |
| 107 | `sha2`, `aes` MIT OR Apache-2.0 | 7 | Correct | crates.io: sha2 0.11.1, aes 0.9.3 | — |
| 108 | Excluded licences: `allsorts` Apache-2.0 only; `zlib-rs` Zlib; `jbig2dec` GPL-3.0; DaeDaLus BSD-3; PolyFile and Arlington Apache-2.0 | 7 | Correct | crates.io (allsorts 0.17.0, zlib-rs 0.6.8, jbig2dec 0.3.1); SE 6.5, FS 12.2 | — |
| 109 | `lopdf`, `pdf`, `hayro`, `fax`, `printpdf`, `pdf-writer`, `krilla`, `subsetter` are PDF-engine work, excluded whatever their licence | 7 | Correct | crates.io (lopdf MIT, hayro Apache-2.0 OR MIT, fax MIT, pdf-writer 0.15.0, krilla 0.8.2, subsetter 0.2.6, MIT OR Apache-2.0) | — |
| 110 | veraPDF corpus CC BY 4.0; veraPDF MPL-2.0+/GPL-3.0+ (test-only) | 7 | Correct | FS 13.4 | — |

### Arithmetic and internal consistency

| # | Claim | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 111 | KLOC inventory total 194–307 | 9.3 | **Wrong** | Sum of the 18 rows | Rows sum to **194–277**. FC-M2 |
| 112 | 49–102 engineer-years of code at 3–4 KLOC/engineer-year | 9.3 | **Wrong** | 194/4 = 48.5; 277/3 = 92.3 | **49–92**. FC-M2 |
| 113 | Tracks total 23–33 engineers | 9.3 | Correct | Sum of nine track rows | — |
| 114 | Limits table fits the heap limit (2²⁴ objects × 48 B ≈ 805 MB < 1 GiB) | 6.3; 6.21 | Correct | Arithmetic | Leaves little room for caches (64 MiB) and decoded data (1 GiB "live at once" equals the whole heap). FC-R18 |

### Cross-references to accepted Izcal documents (helper sweep, reviewed)

Correct as cited: TX-D7 and TX 6.18 (quoted word for word); TX-D3, TX-D10, TX-D19, TX-D23, TX-D24, TX-D25, TX-W1; G25, G26, G28, G29; MD 6.18 and R18; G50, G51, G53, G56; MD-D8, MD-D28; `hg-entropy` LZW; MD 9.3's 15–22 KLOC; GR-D3, GR-D6, G61, G62, G70; GR 6.27 (both quotes); GR 6.5 (S8, E16); the graphics print track (3–4); LY-D8, LY 6.7; JS-D15, G18, JS-16; FP-38, FP-D19, FP-S30; G36, `hg-compress` (RN P5); `sha2` and `aes` pins; G21, G23; HGBA-2 M-1, I-1, I-2, S-4a, S-8, C-1; Stage 1 utility row, M7, 13.2, 7.5, D-S2, D-S10, D-S11; OD-L2, OD-L6, licence class 4; HGBA-3 B1, B2, B8, B9, B10; RN2 Q24 and Section 4; ledger G-14, A-FUT-2, A-FUT-6, A-S1-75; shell R18; SEC-12 (main repository); the four accepted specs that call into a PDF spec (layout names a "PDF writer", not a "PDF spec": fine in substance).

Wrong or partly right (fixes in FC-M4 to FC-M8 and FC-R19 to FC-R23):

| ID as cited | Where | What the source says | Verdict |
|---|---|---|---|
| JS-D12 "no FMA contraction, fixed evaluation order" | 4.3; 6.6; G116 | JS-D12 is NaN canonicalisation and Wasm's deterministic profile; the no-FMA, one-libm rule is JS 6.9 (and MD G55) | Wrong |
| GR-D28 "the broker's hit test … rules" | 6.1 | GR-D28 is security prompts on their own OS layer; hit-test routing is GR 6.0 ("hit-test regions, clamped by the broker"), RN2 Q4, LY-D14/LY-D22 | Wrong |
| MD 6.13 (`hg-icc` v2 and v4, never iccMAX) | 6.7 | MD 6.13 is Determinism; the ICC text is MD 6.5 | Wrong |
| MD 6.17 (read-only font endpoint) | 6.1 | MD 6.17 is Observability; the endpoint is MD 6.1 ("SVG as an image") and MD-D20, R15 | Wrong |
| FP-D16 class (a) for in-memory decoded caches | 6.21 | FP-D16 (a) is on-disk regenerable persistence; in-memory discardable stores are FP 6.4 "class (a)" | Wrong |
| G110–G119 "Rule range for this spec" | 1 (What Izcal's specs already decide) | No accepted registry assigns it; the ledger says "G88 onward the next spec"; only this spec's `frame.md` assigns it; the accessibility frame takes G100–G109; nothing else uses G110–G119 | Partly (free, unregistered) |
| IPC-R7 "sealed read-only region" | 6.1 | IPC-R7 is "copy, then validate, what the receiver parses"; read-only handles are IPC-R1 | Partly |
| UPD-R5 emergency switches for form logic, decoders, signatures, PDF viewing | 6.22 | UPD-R5's list is closed (jitless, Wasm off, WebGPU off, one named codec off, DrumBrake off) | Partly: needs an amendment |
| GR 6.4 conflation fallback "where winding numbers disagree" | 6.6 | "where a pixel's minimum and maximum winding numbers differ by more than one, 16† per-pixel samples" | Partly |
| GR 6.1 compositor as I-1 exception | 6.19 | The label is in GR 6.0's diagram and GR 9.5 (OD-8, RN2 Q10) | Partly |
| FP-14 content GPU tile budget | 6.21 | FP-14 is GPU memory under the GPU-service ledger; the content GPU budgets are GR 6.22 | Partly |
| DET-1 scope for cross-platform raster | PD-4 | DET-1 excludes GPU-dependent output; GR-51 extends the pixel scope | Partly |
| G5 "no `unsafe`" | 6.22; PD-21 | G5 is `forbid(unsafe_code)` unless inventoried | Partly |
| RN P1 for L4 joining crates and L2m/L2g | 6.25 | P1 says L3 and above join layers; L2m and L2g come from RN2 Q18 | Partly |
| "Strictest tier the security scope defines for the media process" | 6.1 | The security scope has no media-process row; its strictest tier is the utility row ("compute plus IPC") | Partly |
| G-15 owns the OCR model | 6.10 | G-15 is the ML service for MD-D25's caption and image-description models | Partly (an extension) |
| Stage 1 exit 48–60 months, layout's critical path | 9.3 | The figure is from RN2 Section 4, not Stage 1 | Partly (cite RN2) |
| `hg-font-subset` as "the fonts spec's subsetter" | 1; 6.19; 6.26 | No such crate in TX; it is this spec's proposed amendment | Partly |

## Links

**External links.** 286 distinct URLs: 3 in the spec and 284 in the notes (2 of the spec's 3 appear only in the spec). All were requested with `curl -L` and a browser user agent; failures were retried with a second user agent.
- **Working: 219.** 218 returned HTTP 200; the Hacker News mirror (HP 7.5) returned 419, then 200.
- **crates.io pages: 27 returned 404** to `curl`, because crates.io serves HTML only to browsers. All 27 crates exist; the licences and versions were confirmed through the crates.io API. Not dead.
- **Bot-walled: 34**, all 403 to both user agents: 22 pdfa.org pages, 3 adobe.com pages, Wharton, CreativePro, connect.mozilla.org, alternativeto.net, Windows Report, Android Authority, Android Headlines, Business Wire, Neowin, Labels & Labeling. Not counted as dead.
- **Dead or unreachable: 6.** None is in the spec.
  - `apryse.com/blog/company/apryse-sponsors-nocost-access-latest-pdf-specs` (HP 3.5): 404.
  - `blogs.nottingham.ac.uk/learningtechnology/?p=3111` (RT 8.5, GO 4.8): redirects to a storage host that returns 404.
  - `datalogics.com/xfa-form-deprecation` (FS 8.5): 404. The live page is `https://www.datalogics.com/xfa-form-deprecation-what-it-means-and-what-to-do` (already used in HP 9.6).
  - `print-css.rocks` (WP 8.6): no response from this host.
  - `www2.computerworld.com.au/article/364660/...` (HP 5.4): no response; the host appears retired.
  - `m.wiki.mozilla.org/Firefox/Printing_Refresh/Archives` (HP 11.6): no response; `https://wiki.mozilla.org/Firefox/Printing_Refresh/Archives` works.
- **Odd host: 1.** HP 7.5 cites Hacker News through `home5.southeastasia.cloudapp.azure.com`, a third-party mirror. The page's canonical URL is `https://news.ycombinator.com/item?id=7716022` ("How fast is PDF.js?"). Replace it.

**Fragments.** All 8 `#fragment` targets exist: the spec's `system-state.html#pdf-viewing-support` (an `h5`, section 8.10.1.6) and `mimesniff#identifying-a-resource-with-an-unknown-mime-type`; the notes' `timers-and-user-prompts.html#printing`, three pdf.js FAQ anchors (`#allthepages`, `#optimize`, `#range`), WeasyPrint `#supported-features` and OpenType `os2#fstype`.

**Internal links.**
- All 23 reference-style definitions in the spec resolve to files in this worktree. Every label used is defined. Two are defined but never used: `[FM]` (frame.md) and `[LA]` (licence audit). The body cites the licence audit's class 4, OD-L2 and OD-L6 without the link (FC-R24).
- Documents cited with no link at all: the security scope (IPC-R7, UPD-R5, SEC-12, the strictest tier), the coding standard (CS-PF-8) and the shell UI design (R18). The security scope and coding standard live only in the main repository's `reports/`; link them as siblings, as the other Phase 2 specs do (FC-R24).
- The notes' two relative links (FS 3.4 to `../Izcal images and media/formats_codecs.md`; FS 3.8 to `../../reports/Izcal images and media.md`) resolve.

## M-1 part 3 (engine source as evidence)

No link in the spec or the notes points at browser or PDF-engine source code: no code files, no `/blob/` or `/+/` paths to code, no commits, diffs, pull requests or code reviews, no searchfox, cs.chromium or googlesource links. Each note records the code results its searches surfaced and did not open (Chromium build file and code reviews in HP; code reviews in SE; a Chromium commit in RT). This check's own search surfaced a PDFium source file (for ISO 21757-1's AF functions); it was not opened and nothing rests on it.

Grey-zone links (prose or data inside repositories, acceptable under OD-7), and whether a spec value depends on each:

| Link | Cited at | Value in the spec depends on it? |
|---|---|---|
| Chromium design docs: OOP iframes; Print Preview | SE 9.2; HP 11.1 | No values; history and reasons only |
| `firefox-source-docs.mozilla.org/accessible/TaggedPdfOutput.html` (prose docs site) | RT 10.3 | No |
| GitHub repository front pages of `krilla`, `pdf-writer`, `subsetter` (README prose; the page lists code files) | WP 4.4, 5.8, 9.1–9.3 | No: only licences and capabilities for the exclusion list |
| `arlington-pdf-model`, `daedalus`, `polyfile` front pages (README, licence) | FS 12; SE 6.5 | Licences only |
| `cmap-resources`, `agl-aglfn`, `urw-base35-fonts`, Liberation `LICENSE`, google/fonts `ofl/arimo` (data and licence files) | RT 4.4–4.9 | Yes: the BSD-3-Clause and OFL findings behind PD-D8. Licence files are not code |
| pdf.js wiki FAQ (Markdown prose) | RT 3.1–3.3 | Only the background figure "3.4 MB per page at 1×" |
| WebKit Bugzilla 261790 and 284479 | HP 8.4 | Only dates of the Unified PDF flag. WebKit bugs often carry patches; the note says only bug prose was read. Prefer a WebKit blog or Safari release note if one exists (FC-R25) |

No value in the spec is derived from engine source. Values only engine source would state (other viewers' limits, rasterisation resolution for effects, print paths) are recorded as black-box questions (SE 7.4, WP 11, RT 13, GO 12) and the spec marks its limits †. Poppler's `pdfinfo`, `pdffonts` and `pdftotext` were run as programs only.

**Personal data.** No personal email address appears in the spec or the notes.

## Must fix

1. **FC-M1. "Forms (24)" and "JPEG (19)" PDFium bugs are a keyword artefact** (Section 3, first paragraph; SE 1.3 and SE 12.2). Re-running the note's own NVD query: no PDFium description contains the word "form" or "forms"; the 24–25 hits are substrings of "platform", "information" and "perform". Every "JPEG" hit is JPEG 2000 (OpenJPEG, `opj_*`, JPX), so "JPEG (19)" double-counts "JPEG 2000 (18)". The spec uses forms as evidence that forms are a bug site (Section 3; SE 12.2 feeds the sandbox contents).
   - Section 3, replace "and the descriptions point at JPEG 2000 (18), JPEG (19) and forms (24)" with: **"and 18 to 21 descriptions name the JPEG 2000 decoder (OpenJPEG); none names forms, fonts appear in 2 and JBIG2 in 1"**.
   - SE 1.3: same correction, and remove "forms" from SE 12.2's list of bug sites (keep "forms and the JS API" only where it rests on the Reader-era CVEs of SE 2.1–2.3).
2. **FC-M2. The KLOC total and engineer-years are wrong** (9.3). The 18 rows sum to 194–277, not 194–307. At 3–4 KLOC per engineer-year that is 49–92 engineer-years, not 49–102. Replace "**194–307**" with "**194–277**" and "49–102 engineer-years†" with "49–92 engineer-years†". The track totals (23–33) are correct and still cover this.
3. **FC-M3. The accessibility permission bit is not a deviation** (6.5; Q-PD-7). ISO 32000-2 deprecates bit 10 ("extract … for accessibility") and says readers shall ignore it (secondary sources quoting Table 22; confirm against the no-cost copy at PD-M0). Izcal's behaviour is therefore conformance, not a harm-avoidance choice. Replace "ISO 32000-2's permission bit for accessibility extraction is honoured only as a label, because 0.4% of web PDFs forbid it ([GO 1.9][GO]) and refusing a screen reader is a harm Izcal will not do (an owner-visible choice, Q-PD-7)" with: **"PDF 2.0 deprecates the accessibility-extraction permission bit and tells readers to ignore it (ISO 32000-2, 7.6.4.2, Table 22; to be confirmed against the no-cost copy at PD-M0), so Izcal ignores it, as 0.4% of web PDFs still set it ([GO 1.9][GO])"**. Change Q-PD-7 to: **"Q-PD-7. Confirm against ISO 32000-2's Table 22 that readers shall ignore the accessibility-extraction bit; if the text differs, the owner decides whether Izcal ignores it anyway."**
4. **FC-M4. JS-D12 is the wrong ID for the floating-point discipline** (4.3 is fine; 6.6 "Numbers"; G116). JS-D12 is NaN canonicalisation and Wasm's deterministic profile. Replace "with JS-D12's discipline (no FMA contraction, fixed evaluation order)" with "**with JS 6.9's discipline (no FMA contraction, one `hg-math`, fixed evaluation order; MD G55)**", and in G116 replace "under JS-D12's discipline" with "**under JS 6.9's discipline**". Remove JS-D12 from the header if nothing else uses it.
5. **FC-M5. GR-D28 is the wrong ID for input routing** (6.1, "The viewer (PD-D2)"). GR-D28 places security prompts on their own OS layer. Replace "(GR-D28's rules)" with "**(GR 6.0's hit-test regions clamped by the broker; RN2 Q4)**".
6. **FC-M6. Two media-spec section numbers are wrong** (6.1, 6.7). Replace "[MD 6.17]" (font endpoint) with "**[MD 6.1, MD-D20]**", and "[MD 6.13]" (`hg-icc` v2 and v4, never iccMAX) with "**[MD 6.5]**".
7. **FC-M7. FP-D16 class (a) is the wrong class** (6.21, decoded-data caches row). FP-D16 (a) is on-disk regenerable persistence. Replace "(FP-D16 class (a))" with "**(FP 6.4 class (a), discardable and regenerable)**", and remove FP-D16 from the header unless it is cited elsewhere.
8. **FC-M8. G110–G119 is not yet assigned** (Section 1, "Rule range for this spec: G110–G119"). No accepted registry assigns the range; only this spec's frame does (the accessibility frame takes G100–G109, the ledger says "G88 onward the next spec"). Replace with: **"Rule range proposed for this spec: G110–G119 (free in every registry; registered through Section 9.5)"**, and add a 9.5 row "Phase 3 ledger / reconciliation registry: register G110–G119 for PDF".
9. **FC-M9. "Incremental-update attacks 21 of 22"** (Section 1, row 4). The 21 of 22 is the 2019 study across universal forgery, incremental saving and signature wrapping. Replace "incremental-update attacks 21 of 22 desktop viewers" with "**the 2019 signature attacks (universal forgery, incremental saving, wrapping) 21 of 22 desktop viewers**".
10. **FC-M10. PDF-initiated navigation and cookies** (6.13 URI row; 6.23). An opaque initiator makes the navigation cross-site, but `SameSite=Lax` cookies are still sent on top-level GET navigations (and `None` cookies always); only `Strict` cookies are withheld. The rule behind PD-D15 ("cannot forge same-site requests") holds; "carries no same-site cookies" does not. Replace "so it carries no same-site cookies and cannot forge same-site requests" with: **"so the request is cross-site: `SameSite=Strict` cookies are withheld, `Lax` cookies go only with a top-level GET as for any cross-site link, and the PDF cannot make a same-site request ([SE 4.4][SE])"**.

## Recommended

**Evidence wording**
1. **FC-R1** (6.10, "Wrong tags"). Replace "Half of tagged files break ISO/TS 32005's nesting rules somewhere ([GO 4.3][GO])" with "ISO/TS 32005 nesting violations are common: P directly under the root in about 1.6 million documents, Figure 1.1 million, Table 0.95 million ([GO 4.3][GO])".
2. **FC-R2** (Section 1, row 5; GO 0 and 7.1). "Chrome 131 (October 2024)" → "**Chrome 131 (November 2024)**". The Chrome blog post is dated 30 October 2024; Chrome 131 reached stable in November.
3. **FC-R3** (11.2, clause numbers). Extend the Medium list: "Clause numbers not on any errata page and inferred from ISO 32000-1's numbering: 7.4.2 and 7.4.5–7.4.10; 8.4.3.2; 8.6.7; 8.7.4.5.2–8.7.4.5.7; 9.6.5.4; 10.7.5; 11.3.4; 11.4.5; 11.4.6; 11.7.4; 14.8.2.3".
4. **FC-R4** (4.4 and 6.1, `aria-owns`). Write "commenters on Mozilla's post note that VoiceOver does not support `aria-owns`".
5. **FC-R5** (2.1, SafeDocs row). Replace "disagreed widely" with "showed many behaviour differences ([FS 2.5][FS])".
6. **FC-R6** (Section 5, FORCEDENTRY row). "A JBIG2 decoder outside the sandbox" → "A JBIG2 decoder outside BlastDoor".
7. **FC-R7** (2.1, 2010 Skia row). "Firefox is moving to the same backend in 2026" → "Firefox is moving to the same backend on Linux and macOS in 2026 (Windows still prints through Cairo)".
8. **FC-R8** (2.1, 2017/2020 row). "almost no web PDF is 2.0 yet" → "none of a 1,045-file sample of the 2021 crawl was PDF 2.0 ([FS 2.7][FS])".
9. **FC-R9** (Section 3, last sentence of "Print-to-PDF"). "Typst and Prince show that tagged-by-default output with PDF/UA and PDF/A on request is routine" → "Typst tags by default and Prince tags whenever a PDF/UA or PDF/A level-a profile is requested; both offer PDF/UA and PDF/A on request ([WP 3.1–3.4, 4.4][WP])".
10. **FC-R10** (6.1, "Why per document"). Quote exactly: "by sandboxing it – similar to a tab in a modern web browser" ([SE 4.9][SE]).
11. **FC-R11** (Section 1, row 1). Quote exactly: "there is no standardized behaviour on the correct rendering of a missing font".
12. **FC-R12** (Section 3; PD-3). The "48 patches" count is from trade press (RT 1.6, Medium). Confirm it from the GWG suite itself when the corpus is fixed at PD-M0, and say so in PD-3.
13. **FC-R13** (11.2). Add: "XFA's deprecation in PDF 2.0 rests on vendor sources; the clause was not read ([FS 8.5][FS])", and "the tagged-share figures come from a different Dual Lab wave (20.5 million PDFs, 2023–2026) than the feature counts (20.6 million, CC-MAIN-2026-25)".
14. **FC-R14** (6.19, PD-D18 option (c)). Source "what the PDF Association criticised in other producers" or replace it with "a claim resting on unverified judgement would be a false conformance claim".
15. **FC-R15** (Summary, PD-D8). "no MIT metric-compatible substitute for the standard 14 fonts exists" → "none was found (TeX Gyre's licence was not checked, [RT 4.10][RT])".
16. **FC-R16** (4.2; 6.17; Section 7). Whether ISO 21757-1 specifies the AF functions is unverified. Write "the Acrobat form functions (`AFNumber_*` and their siblings), anchored to ISO 21757-1 where it defines them and otherwise to Adobe's published forms documentation", and add a Q-PD: "Does ISO 21757-1 define the AF functions?".
17. **FC-R17** (6.5). "SHA-256/384/512 and AES are already there" → "SHA-256, SHA-384 and AES are already there; SHA-512 is added for R6" (and list SHA-512 in the NW amendment).
18. **FC-R18** (6.3 and 6.21, limits). "Decoded bytes per document, live at once: 1 GiB" equals the whole canonical heap, which must also hold the resolved view (up to 2²⁴ × 48 B ≈ 805 MB) and the 64 MiB caches. Set the live-decoded limit below the heap (for example 512 MiB†) or state that the heap limit is enforced first.

**Cross-references**
19. **FC-R19** (6.1). IPC-R7 → "IPC-R1 (read-only handle) and IPC-R7 (copy, then validate)"; "the strictest tier the security scope defines for the media process" → "the security scope's strictest (utility) tier, where MD-D5 places the media process".
20. **FC-R20** (6.22; 9.5). UPD-R5's switch list is closed. Write "new emergency switches (form logic, single decoders, signature parsing, PDF viewing), added to UPD-R5 by amendment (9.5)". The 9.5 security-scope row already mentions the switches; name UPD-R5 there.
21. **FC-R21** (6.6). Conflation fallback: "16† samples per pixel where a pixel's minimum and maximum winding numbers differ by more than one ([GR 6.4][GR])".
22. **FC-R22** (6.19, 6.21, PD-4, PD-21). GR 6.1 → "GR 6.0 and 9.5 (OD-8, RN2 Q10)"; FP-14 → "FP-14 and GR 6.22's content GPU budgets"; DET-1 → "DET-1 as extended by GR-51"; "no `unsafe` … G5" → "G5, with no inventory entry".
23. **FC-R23** (6.25; 9.3; 6.10; Section 1). RN P1 → "RN P1 and RN2 Q18"; "Stage 1 exit (48–60 months†, layout's critical path)" → cite "[RN2 Section 4][RN2]"; G-15 → "G-15 (the ML service, extended here to OCR)"; `hg-font-subset` "the fonts spec's subsetter" → "a fonts-spec crate proposed by this spec (9.5)".
24. **FC-R24** (header and link definitions). Trim the header to IDs the body uses: drop HGBA-2 4.6 and I-3, HGBA-3 B5, JS-D13 and G13–G17, FP-1, FP-D11 and FP 6.12, TX-D2 and G27, NW-D6, NW-D23 and G30–G35, MD-D5, G54, G55 and G57, GR-D5, GR-D16 and the unused G60–G72 rules, ST G19, G20, G22 and G24. Add the IDs the body uses: RN P1 and P5; LA class 4, OD-L2 and OD-L6; G-15 and A-S1-75; S-4a, S-8, C-1, D-S2, D-S10, D-S11, M7; FP-14 and FP-S30; TX-W1 and TX-8; G56 and R15; GR 6.0, 6.4, 6.5, G61, G62, G70; RN2 Section 4. Use the unused `[LA]` and `[FM]` definitions, and add sibling links for the security scope (`../../browser/reports/` or as the other specs link it), the coding standard and the shell UI design.

**Notes and links**
25. **FC-R25** (notes). Fix the dead or odd links: HP 7.5's Hacker News mirror → `https://news.ycombinator.com/item?id=7716022`; HP 11.6 → `https://wiki.mozilla.org/Firefox/Printing_Refresh/Archives`; FS 8.5 → `https://www.datalogics.com/xfa-form-deprecation-what-it-means-and-what-to-do`; mark HP 3.5's Apryse post, HP 5.4's Computerworld AU article, RT 8.5 / GO 4.8's Nottingham post and WP 8.6's print-css.rocks as unreachable on 19 October 2026 (archive copies if needed). For HP 8.4, prefer a WebKit blog or Safari release note to the two WebKit Bugzilla entries.
26. **FC-R26** (SE 1.3, GO 6.3). Fix the notes as well as the spec: SE 1.3 as FC-M1; GO 6.3 says "signature-wrapping attacks broke 26 of 29 validators", but 26 of 29 (21 of 22 desktop plus 5 of 7 online) is the 2019 study across all three attack classes.
