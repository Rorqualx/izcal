# Izcal PDF: history and postmortems

Compiled 19 October 2026. Primary sources were preferred (papers, specifications, maintainers' posts, release notes, issue trackers).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

M-1 note: this file records ideas, dates, measurements and stated reasons. No source code of any browser or PDF engine or library was opened or cited (pdf.js, PDFium, Poppler, xpdf, MuPDF, Skia, Cairo, Foxit, WebKit and others). One search result surfaced a Chromium build-configuration file and two Chromium code reviews; they were not opened and nothing in this note rests on them. Prose only: blog posts, release notes, Microsoft message-centre text, design documents, interviews, Bugzilla prose, papers and security write-ups. Where a row says what a project "does", that is evidence to reason from, not a shape to copy. Projects named here are history and landscape, or black-box test oracles; none is a dependency.

**Method notes**
- 33 searches and fetches were used. pdfa.org refused automated fetches (HTTP 403), so PDF Association claims come from search snippets and are graded Medium.
- Read in full: Mozilla's 2013 pdf.js announcement; Mozilla Hacks' October 2021 post on forms, scripting, XFA and tagged PDF; Microsoft's message-centre notice MC512918 (through a mirror); Knowledge at Wharton's 2013 interview with Warnock and Wulff; Chromium's Print Preview design document (2010); Ladybird's April 2026 newsletter; Wikipedia's History of PDF (secondary; used for the version table only).
- Rows that rest on the researcher's recollection rather than a source read this session are graded Low and repeated under "Claims not verified".

## 0. Chronology at a glance

| Year | Event | Section |
|---|---|---|
| 1991 | Warnock's "Camelot" paper proposes "Interchange PostScript" | 1 |
| 1993-06-15 | Acrobat 1.0 and PDF 1.0 ship (shown at a conference in January 1993) | 1 |
| 1994 (fall) | Acrobat 2.0; Reader given away free | 1 |
| 1994-2006 | PDF 1.1 to 1.7 (encryption, AcroForm, JavaScript, transparency, tagged PDF, object streams, XFA, AES, 3D) | 2 |
| 1999-05 | JetForm submits XFA to the W3C | 9 |
| 2001 / 2005 / 2008 / 2010 / 2012 | PDF/X, PDF/A-1, PDF/E, PDF/VT, PDF/UA-1 | 13 |
| 2002-02 | Adobe acquires Accelio (formerly JetForm) and with it XFA | 9 |
| 2007-01-29 | Adobe says it will hand PDF 1.7 to ISO through AIIM | 3 |
| 2008-07 | ISO 32000-1:2008 (PDF 1.7) published; Adobe Public Patent License | 3 |
| 2008-2010 | Adobe Reader becomes the most-targeted application in attacks | 5 |
| 2010 | Chrome's sandboxed built-in viewer (dev June, beta November, stable Chrome 8 December); Reader X Protected Mode sandbox (November); Chrome Print Preview design (October) | 5, 6, 11 |
| 2011-07 | pdf.js first release | 7 |
| 2013-01 | pdf.js tested in Firefox Beta; default in Firefox 19 | 7 |
| 2014-06 | Google open-sources PDFium (Foxit code base, BSD licence) | 6 |
| 2015 | Chrome removes NPAPI (September); Edge (EdgeHTML) ships with Windows 10 with a built-in PDF reader | 5, 8 |
| 2017-03 | Firefox 52 removes NPAPI plugins except Flash | 5 |
| 2017-07 | ISO 32000-2:2017 (PDF 2.0), deprecating XFA and other features | 3, 4 |
| 2020 | Chrome 85 writes tagged PDF from "Save as PDF" (August); ISO 23504 PDF/R; ISO 32000-2:2020 (December) | 11, 13, 3 |
| 2020-12 to 2021-10 | Firefox ships AcroForms (84), PDF JavaScript (88), tagged PDF (89), XFA (93) | 7, 9, 10 |
| 2021 | FORCEDENTRY: a PDF's JBIG2 stream used as a "weird machine" (CVE-2021-30860); shadow attacks on signatures (NDSS 2021) | 14 |
| 2023-02-08 | Microsoft announces Edge will use Adobe's Acrobat PDF engine | 8 |
| 2023-04-05 | ISO 32000-2 free of charge, sponsored by Adobe, Apryse and Foxit | 3 |
| 2023-09 | WebKit adds a flag for a new "Unified PDF" plugin | 8 |
| 2024 | WTPDF (February) and ISO 14289-2 PDF/UA-2 (March); Edge's legacy PDF engine scheduled for removal 2024-03-31 | 13, 8 |
| 2026-04 | Ladybird renders PDFs inline with a bundled pdf.js | 8 |

## 1. Origins: from PostScript to PDF (1991-1996)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | Warnock dates the Camelot paper to 1991 in his own words. Adobe's current timeline page places the Camelot project in 1990. The paper proposed a simplified, screen-oriented PostScript ("Interchange PostScript"). | 1991 (Warnock) vs 1990 (Adobe page) | [Knowledge at Wharton interview](https://knowledge.wharton.upenn.edu/article/adobe-acrobat-at-20-successes-second-guesses-and-a-few-miscues/); [Adobe PDF timeline](https://www.adobe.com/uk/acrobat/resources/pdf-timeline.html) | 2013 | High |
| 1.2 | **Why not PostScript itself:** PostScript is a full programming language. A page can only be known by executing the program, which makes random page access, search and safe display hard. PDF fixed the page description into static objects. This rationale is widely stated; this session did not read it in the Camelot text itself. | n/a | [Wikipedia: History of PDF](https://en.wikipedia.org/wiki/History_of_PDF) | n/a | Low |
| 1.3 | Acrobat 1.0 and PDF 1.0 launched 15 June 1993 (shown at a Windows and OS/2 conference in January 1993). PDF 1.0 had text, images, pages, internal links and bookmarks. | 1993-06-15 | [Adobe PDF timeline](https://www.adobe.com/uk/acrobat/resources/pdf-timeline.html); [Wikipedia](https://en.wikipedia.org/wiki/History_of_PDF) | 1993 | Medium |
| 1.4 | **Early adoption failed.** Warnock: "nobody got it". A Gartner analyst called it "worthless". Acrobat was unprofitable for about four years. Reader cost $35-50 per seat. | ~4 years unprofitable; $35-50/seat | [Wharton](https://knowledge.wharton.upenn.edu/article/adobe-acrobat-at-20-successes-second-guesses-and-a-few-miscues/) | 2013 | High |
| 1.5 | **Reader became free with Acrobat 2.0, fall 1994.** Stated reasons: "We had to get people out there using it in order to understand it" (Warnock); paid sales were weak and free browsers plus rival readers (Envoy, Common Ground, Replica) showed the market moving (Wulff). Wikipedia instead says Reader 2.0 in 1996; the founders' account is preferred. | 1994 vs 1996 | [Wharton](https://knowledge.wharton.upenn.edu/article/adobe-acrobat-at-20-successes-second-guesses-and-a-few-miscues/) | 2013 | High |
| 1.6 | Founders' own miscues: promising SGML support in the 1.0 press release (never shipped); weak early compression (fixed only in Acrobat 6); security overlooked. Warnock: "If you have the most popular file format in the world, guess where hackers go?" | n/a | [Wharton](https://knowledge.wharton.upenn.edu/article/adobe-acrobat-at-20-successes-second-guesses-and-a-few-miscues/) | 2013 | High |
| 1.7 | Early limits per secondary history: no external hyperlinks in 1.0, file sizes too big for modems, slow rendering on older machines. | n/a | [Wikipedia](https://en.wikipedia.org/wiki/History_of_PDF) | n/a | Low |

**Implications:** PDF won through a free reader and a fixed, non-programmatic page model. Its founders name security as the thing they missed; Izcal should treat every PDF as hostile input from day one.

## 2. PDF 1.1 to 1.7: what each version added (1994-2006)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | PDF 1.1 (1994): passwords, encryption, a binary form for smaller files. | 1994 | [Wikipedia](https://en.wikipedia.org/wiki/History_of_PDF) | n/a | Low |
| 2.2 | PDF 1.2 (1996): interactive forms (AcroForm), Unicode, Flate (zlib) compression. | 1996 | same | n/a | Low |
| 2.3 | PDF 1.3 (2000): digital signatures, **JavaScript actions**, Web Capture. | 2000 | same | n/a | Low |
| 2.4 | PDF 1.4 (2001): **transparency**, JBIG2, XMP metadata, **tagged PDF**. PDF/A-1 was later built on this version. | 2001 | same; [PDF/A basics (PDF Association)](https://www.pdfa.org/wp-content/until2016_uploads/2011/08/PDFA_The-Basics_A-new-standard-for-long-term-archiving.pdf) | n/a | Low |
| 2.5 | PDF 1.5 (2003): JPEG 2000, **object streams** and cross-reference streams, public-key security, **XFA** (with Acrobat 6). | 2003 | same; [Wikipedia: XFA](https://en.wikipedia.org/wiki/XFA) | n/a | Low |
| 2.6 | PDF 1.6 (2004): 3D artwork, OpenType embedding, AES encryption. PDF 1.7 (2006): portfolios and more attachments. | 2004, 2006 | [Wikipedia](https://en.wikipedia.org/wiki/History_of_PDF) | n/a | Low |
| 2.7 | After ISO 32000-1, Adobe shipped "Extension Levels" (1, 3, 5, 6, 8) instead of a PDF 1.8, and said it would not produce a 1.8 reference. These proprietary extensions are the main source of features outside the ISO text. | 2008-2011 | same | n/a | Low |

**Implications:** Each feature arrived in a known version, so Izcal can measure feature use per version in a corpus. Adobe Extension Levels are a known gap between the ISO text and real files.

## 3. Standardisation: ISO 32000-1, ISO 32000-2 and free access

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | Adobe announced on 29 January 2007 that it would release PDF 1.7 to AIIM for ISO publication. ISO published ISO 32000-1:2008 in July 2008 (sources say 1 or 2 July). | 2007-01-29; 2008-07 | [CreativePro](https://creativepro.com/pdf-format-becomes-iso-standard/) | 2008 | Medium |
| 3.2 | **Why:** Adobe CTO Kevin Lynch cited governments and organisations asking for open formats, and said maintenance by a participatory body would keep innovation going. A commentator noted PDF had been "open in name only": the spec was free but Adobe set its direction. | n/a | [CreativePro](https://creativepro.com/pdf-format-becomes-iso-standard/) | 2008 | Medium |
| 3.3 | Adobe published a **Public Patent License** for ISO 32000-1, royalty-free for patents necessary to implement conforming PDF. | 2008 | same (snippet) | 2008 | Medium |
| 3.4 | ISO 32000-2:2017 (PDF 2.0) was published in July 2017 by ISO TC 171 SC 2 WG 8; it "includes no proprietary technology". The second edition, ISO 32000-2:2020, followed in December 2020 and is the current text. | 2017-07; 2020-12 | [Wikipedia](https://en.wikipedia.org/wiki/History_of_PDF) | n/a | Medium |
| 3.5 | Until 5 April 2023, PDF 2.0 had to be bought from ISO (a forum user cites about CHF 208). Since then it is free through the PDF Association, sponsored by **Adobe, Apryse and Foxit** (the task brief named only two). The package includes the ISO/TS 32001 and 32002 crypto specs and errata. Apryse says sponsors will keep the latest versions free. | 2023-04-05 | [PDF Association announcement](https://pdfa.org/announcing-no-cost-access-to-iso-32000-2-pdf-2-0/) (snippet; 403 on fetch); [Apryse](https://apryse.com/blog/company/apryse-sponsors-nocost-access-latest-pdf-specs) | 2023 | Medium |

**Implications:** Izcal can anchor to ISO 32000-2:2020 with its errata at no cost. The 2008 patent licence covers ISO 32000-1; whether an equivalent covers PDF 2.0 must be checked before the spec relies on it (royalty-free rule, MD-D3).

## 4. PDF 2.0 deprecations and removals

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | PDF 2.0 deprecates **XFA**: writers should not write it and readers are recommended to ignore it. XFA was never an ISO standard. | n/a | [Global Graphics via search](https://blog.globalgraphics.com/?p=212); [Qoppa](https://kbpdfstudio.qoppa.com/tag/pdf-2-0/); [Mozilla Hacks 2021](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) ("removed from the official PDF specification") | 2017-2021 | Medium |
| 4.2 | The catalog key **NeedsRendering** (XFA's "render me from XML" switch) is deprecated. | n/a | [ABCpdf reference](https://www.websupergoo.com/helppdfnet/source/9-abcpdf.elements/07-syntax/1081-catalogelement/2-properties/28-entryneedsrendering.htm) | n/a | Medium |
| 4.3 | **PostScript XObjects** are removed (already discouraged in 32000-1). **Movie and Sound** annotations are deprecated in favour of RichMedia. | n/a | [ONLYOFFICE forum](https://community.onlyoffice.com/t/update-standard-pdf-2-0-iso-32000-2-2020/7911); [PDF Tools overview](https://www.pdf-tools.com/de/pdf-knowledge/pdf-2-0-a-quick-overview/) | n/a | Low |
| 4.4 | Encryption: PDF 2.0 adds security handler revision 6 (AES-256 with a new key derivation) and deprecates older options as insecure, including Adobe's interim AES-256 revision 5. Whether RC4 and AES-128 are deprecated by name was not confirmed in a primary text. | R6 | [Global Graphics](https://blog.globalgraphics.com/?p=212); [iText on R=6](https://itextpdf.com/blog/technical-notes/how-solve-unknown-encryption-type-r-6-errors) | n/a | Medium |
| 4.5 | "Deprecated" means: writers should not emit the feature; readers are recommended to ignore it. It does not forbid reading old files. | n/a | [Global Graphics](https://blog.globalgraphics.com/?p=212) | n/a | Medium |

**Implications:** Deprecation gives Izcal a standards basis for declining XFA and legacy multimedia. It does not make old encrypted files go away: reading RC4 and AES-128 files is still needed, writing them is not.

## 5. The plugin era and why users left it

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | F-Secure: Reader files were used in almost 49% of 663 targeted attacks in 2009, up from about 29% in 2008; Word fell to about 39%. F-Secure blamed more Acrobat/Reader vulnerabilities. | 49% of 663 (2009); 29% (2008) | [The Register](https://www.theregister.com/2010/03/09/adobe_reader_attacks/) | 2010-03-09 | Medium |
| 5.2 | ScanSafe: malicious PDFs were 56% of exploits in Q1 2009 and 80% in Q4 2009. Kaspersky: about 50% of exploits in Q1 2010 targeted Adobe software, of which about 90% targeted Reader. | 56% to 80% (2009); 50% and 90% (Q1 2010) | [Computerworld](https://www.computerworld.com/article/1598190/patch-adobe-s-pdf-bug-pronto-expert-urges.html); search snippets | 2010 | Medium |
| 5.3 | Many headline bugs were in the **JavaScript API**: `util.printf()` stack overflow (Reader 8.1.2, 2008; workaround "disable JavaScript"); `Collab.getIcon()` stack overflow (CVE-2009-0927). | n/a | [CORE advisory CORE-2008-0526](https://dl.packetstormsecurity.net/0811-advisories/CORE-2008-0526.txt); [Trend Micro](https://www.trendmicro.com/vinfo/us/threat-encyclopedia/vulnerability/159/adobe-acrobat-and-reader-collab-geticon-javascript-method-remote-code-execution-vulnerability) | 2008-2009 | Medium |
| 5.4 | **Adobe's response: Reader X Protected Mode (November 2010).** PDF and image parsing, JavaScript, font rendering and 3D run in a sandbox; actions outside go through a broker process. Built with help from Microsoft and Google security teams. Adobe said it was "not a panacea". Windows only at launch. | 2010-11 | [Threatpost](https://threatpost.com/adobe-releases-reader-x-sandbox-111810); [Computerworld AU](https://www2.computerworld.com.au/article/364660/adobe_reader_x_adds_security_productivity_mobility/) | 2010 | Medium |
| 5.5 | **End of the in-browser plugin:** Chrome disabled NPAPI by default in April 2015 and removed it in September 2015. Firefox 52 (March 2017) dropped NPAPI except Flash. Safari 12 and Chromium Edge do not run the Reader plugin. Only Internet Explorer remained supported by Adobe. | 2015-04, 2015-09, Firefox 52, Safari 12 | [Adobe help: change in plug-in support](https://helpx.adobe.com/acrobat/kb/change-in-support-for-acrobat-and-reader-plug-ins-in-modern-web-.html); [Mozilla support](https://support.mozilla.org/en-US/kb/use-adobe-reader-view-pdf-files-firefox) | 2015-2021 | Medium |

**Implications:** The plugin era ended for two reasons: an exploit record concentrated in one native, scriptable viewer, and browsers removing the plugin API itself. A viewer with full process isolation and no ambient authority is the minimum Izcal inherits from this history.

## 6. Chrome's built-in viewer and PDFium

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Chrome's viewer reached developer builds in June 2010 (off by default, enabled in chrome://plugins), beta in November 2010 and stable in **Chrome 8 (December 2010)**. Google's stated reason: a "sandbox" layer around the viewer against malware and attacks targeted at PDF files. | 2010-06; 2010-11; 2010-12 | [Chrome Blog: PDF goodness in Chrome](https://chrome.googleblog.com/2010/11/pdf-goodness-in-chrome.html); [Chrome Releases](https://chromereleases.googleblog.com/2010/12/stable-beta-channel-updates.html) | 2010 | Medium |
| 6.2 | The 2010 viewer was closed source (licensed, from Foxit) and ran as a Pepper plugin. It lacked some Reader features at launch. | n/a | [The Register](https://www.theregister.com/2010/06/18/chrome_dev_builds_get_built_in_pdf_reader/) | 2010-06-18 | Medium |
| 6.3 | **PDFium open-sourced, June 2014**, from Foxit's code base under the BSD licence. Stated reasons: give developers "a robust and reliable PDF library to view, search, print, and form fill" (Beaufort); reduce Chrome's closed components (Kasting called it "almost certainly the highest-quality PDF engine available in the open-source world"). Foxit stays a contributor. | 2014-06-10 | [Foxit press release](https://www.foxit.com/company/press/1614.html); [Computerworld](https://www.computerworld.com/article/1393378/google-open-sources-pdf-software-library.html); [InfoQ](https://infoq.com/news/2014/06/google-chrome-pdf-engine-free) | 2014 | Medium |
| 6.4 | PDFium includes a JavaScript runtime for Acrobat-style form scripts using V8; Chrome enables it. Chrome does not render XFA (users report fallback pages). Both are stated only in configuration or user reports this session saw; they need black-box confirmation. | n/a | [Vivaldi forum](https://forum.vivaldi.net/topic/80931/is-javascript-enabled-by-default-in-the-built-in-pdf-viewer) | n/a | Low |
| 6.5 | PDFium has a steady CVE stream in Chrome releases (e.g. CVE-2020-16029 navigation-restriction bypass via a crafted PDF; CVE-2018-6170 type confusion). | n/a | [SUSE CVE-2020-16029](https://www.suse.com/security/cve/CVE-2020-16029/); [Fortiguard CVE-2018-6170](https://fortiguard.com/encyclopedia/endpoint-vuln/57366) | 2018-2020 | Medium |

**Implications:** Google chose a native C++ engine and contained it with a sandbox; the code remained a source of CVEs. Izcal's choice (memory-safe Rust, its own engine, sandboxed process) addresses the same risk from two sides.

## 7. Firefox and pdf.js

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | Andreas Gal and Chris Jones started pdf.js as a research project; first release 2 July 2011. | 2011-07-02 | [Mozilla Future Releases blog](https://blog.mozilla.org/futurereleases/2013/01/11/mozilla-tests-a-built-in-secure-pdf-viewer-in-firefox-beta-leveraging-the-power-of-html5/); [Computerworld](https://www.computerworld.com/article/1532288/mozilla-to-add-built-in-pdf-viewer-to-firefox.html) | 2013-01-11 | High (founders), Medium (date) |
| 7.2 | **Why JavaScript/HTML5:** plugins contain "proprietary closed source code that could potentially expose users to security vulnerabilities"; plugins duplicate drawing code the browser already has; and to "push the boundaries of the HTML5 platform". Gal (2011): a native plugin "enlarges the trusted code base", Chrome had to sandbox its renderer, and an HTML5 implementation is "immune to this class of problems". | n/a | [Mozilla 2013](https://blog.mozilla.org/futurereleases/2013/01/11/mozilla-tests-a-built-in-secure-pdf-viewer-in-firefox-beta-leveraging-the-power-of-html5/); [Computerworld](https://www.computerworld.com/article/1532288/mozilla-to-add-built-in-pdf-viewer-to-firefox.html) | 2011, 2013 | High |
| 7.3 | Firefox Beta tested it in January 2013; it became the default in Firefox 19 (2013) after being included earlier. The 2013 post claimed most PDFs "load and render quickly" and expected gains as JS engines improved; it said nothing about fidelity. | Firefox 19 | [Mozilla 2013](https://blog.mozilla.org/futurereleases/2013/01/11/mozilla-tests-a-built-in-secure-pdf-viewer-in-firefox-beta-leveraging-the-power-of-html5/) | 2013 | High (claim), Medium (version) |
| 7.4 | **Complaints:** over-rendering images beyond screen resolution (bug 962546), UI freezes and memory growth (bug 1722807, 2021), 457 MB for about 5 MB of PDFs (support thread), "much much slower than chrome's viewer" on a file (bug 1119445), single-page PDFs taking up to 4 s because of huge images (Mozilla Connect). Several were closed WORKSFORME. | 457 MB / ~5 MB; up to 4 s | [Bug 962546](https://bugzilla.mozilla.org/show_bug.cgi?id=962546); [Bug 1722807](https://bugzilla.mozilla.org/show_bug.cgi?id=1722807); [Bug 1119445](https://bugzilla.mozilla.org/show_bug.cgi?id=1119445); [Mozilla Connect](https://connect.mozilla.org/t5/discussions/pdf-viewer-extremly-slow/m-p/58458) | 2014-2021 | Medium |
| 7.5 | A profiling discussion found pdf.js was not bottlenecked by JavaScript speed but by browser-side work. Ladybird's 2026 profiling of pdf.js agrees in kind: fixes in typed-array view caching and `:has()` invalidation saved about 250 ms and 650 ms on one large PDF. | ~250 ms, ~650 ms; `:has()` visits 71k to 1.6k | [HN thread](https://news.ycombinator.com/item?id=7716022); [Ladybird April 2026](https://ladybird.org/newsletter/2026-04-30) | 2014; 2026 | Low (HN), High (Ladybird) |
| 7.6 | Staged feature growth: AcroForms Firefox 84, PDF JavaScript 88, tagged PDF 89, XFA 93 (5 October 2021). Driven by **telemetry**: many forms use JavaScript, mostly for validation and calculation; XFA was "fairly common". Test corpus: about 32,000 tagged PDFs, 1,900 with JavaScript, 1,200 XFA. | 84/88/89/93; 32k/1.9k/1.2k | [Mozilla Hacks 2021](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021-10-07 | High |
| 7.7 | Tagged PDF is exposed by inserting ARIA-role structure elements and linking to text-layer spans with `aria-owns`; commenters note VoiceOver does not support `aria-owns`. | n/a | same | 2021 | High |

**Implications:** pdf.js shows that a viewer in a memory-safe runtime removes a class of bugs, at a measurable cost in speed and memory, much of it outside the PDF code itself. Izcal's native Rust viewer drawing into its own display list can aim for both. Telemetry-led prioritisation (forms, then scripts, tags, XFA) is a lesson worth repeating with Izcal's own corpus.

## 8. Edge, Safari, Opera, Ladybird, Servo

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | Legacy Edge (EdgeHTML, 2015) had a built-in PDF reader praised later for two-page view, smooth zoom and highlighting. Chromium Edge (2020) switched to Chromium's PDFium viewer and lost some of those features at first (users missed the contents list). | 2015; 2020 | [Windows Latest](https://windowslatest.com/2021/04/25/microsoft-edge-is-getting-new-pdf-features-to-rival-google-chrome); [TechRadar](https://www.techradar.com/news/microsoft-edge-is-bringing-back-an-important-feature); [MS Tech Community](https://techcommunity.microsoft.com/t5/discussions/edgehtml-pdf-style/m-p/1321820) | 2020-2021 | Low |
| 8.2 | **Edge to Adobe's engine (announced 8 February 2023).** Stated reasons: "higher fidelity for more accurate colors and graphics", "improved performance", "strong security for PDF handling", "better text selection and read-aloud narration", framed as an Adobe-Microsoft collaboration. Promised "no loss of functionality". Consumers from March 2023; organisations no earlier than September 2023; legacy engine removal and opt-out end 31 March 2024. Advanced editing requires an Acrobat subscription. | 2023-03; 2023-09; 2024-03-31 | [MC512918](https://mc.merill.net/message/MC512918); [PCWorld](https://www.pcworld.com/article/1507272/edge-tosses-its-pdf-reader-overboard-in-favor-of-adobe-acrobat.html) | 2023 | High (notice text), Medium (completion) |
| 8.3 | Edge's switch drew criticism over branding and paid upsell inside the browser. Whether the March 2024 removal happened on schedule was not confirmed. | n/a | [Pureinfotech](https://pureinfotech.com/microsoft-edge-pdf-reader-adobe-acrobat/) | 2023 | Low |
| 8.4 | Safari renders PDFs through Apple's system PDFKit. In September 2023 WebKit added a flag for a new cross-platform "Unified PDF" plugin architecture; it was on by default from macOS 15.2, with the older PDFPlugin kept for older macOS. | 2023-09; macOS 15.2 | [WebKit bug 261790](https://bugs.webkit.org/show_bug.cgi?id=261790); [WebKit bug 284479](https://bugs.webkit.org/show_bug.cgi?id=284479) | 2023-2025 | Medium |
| 8.5 | Opera uses Chromium since 2013 and therefore Chromium's viewer; no Opera-specific PDF history was found. | n/a | none read | n/a | Low |
| 8.6 | **Ladybird (April 2026)** renders PDFs inline through a bundled pdf.js. The newsletter gives no reason and does not mention SerenityOS's own LibPDF. Servo: no published PDF viewer prose was found. | 2026-04 | [Ladybird newsletter](https://ladybird.org/newsletter/2026-04-30) | 2026-04-30 | High |

**Implications:** No browser vendor today writes its own PDF engine from scratch: Google adopted Foxit's, Mozilla wrote one in JS, Apple uses its OS library, Microsoft adopted Adobe's, Ladybird adopted pdf.js. Izcal's own engine is unusual; the fidelity, text selection and read-aloud reasons Microsoft gave are the bar it will be judged against.

## 9. XFA: rise and fate

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | XFA (XML Forms Architecture) came from JetForm, which submitted it to the W3C in May 1999. JetForm renamed itself Accelio (13 September 2001); Adobe bought Accelio in February 2002. | 1999-05; 2001-09-13; 2002-02 | [Wikipedia: XFA](https://en.wikipedia.org/wiki/XFA); [Wikipedia: JetForm](https://en.wikipedia.org/wiki/JetForm) | n/a | Low |
| 9.2 | XFA entered PDF 1.5 with Acrobat 6 and 7 (2003). Forms were authored in Designer (later LiveCycle Designer) and stored in PDF or XDP files; dynamic XFA re-lays out the form from XML at run time. | 2003 | [Wikipedia: XFA](https://en.wikipedia.org/wiki/XFA); [Wikipedia: LiveCycle Designer](https://en.wikipedia.org/wiki/Adobe_LiveCycle_Designer) | n/a | Low |
| 9.3 | XFA was never made an ISO standard and is deprecated in PDF 2.0 (2017). | 2017 | see 4.1 | 2017 | Medium |
| 9.4 | **Why it lingers:** governments and enterprises built large form estates on it. Firefox's telemetry found XFA "fairly common", so Mozilla built a custom layout engine that turns XFA into HTML elements over the canvas, shipping Liberation fonts with glyph rescaling for missing fonts (Firefox 93, October 2021). | Firefox 93 | [Mozilla Hacks 2021](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021 | High |
| 9.5 | Firefox's XFA support remains partial: a 2026 support thread reports dynamic add/remove buttons do not work where Acrobat's do. | Firefox 151 | [Mozilla support](https://support.mozilla.org/hi-IN/questions/1584865) | 2026 | Low |
| 9.6 | Users report most third-party viewers lack XFA; Chrome shows a fallback. Vendors (Datalogics, Apryse) market XFA-to-AcroForm migration, which shows the estate is shrinking but not gone. | n/a | [Datalogics](https://www.datalogics.com/xfa-form-deprecation-what-it-means-and-what-to-do); [Apryse](https://apryse.com/blog/xfa-options-alternatives-by-apryse) | n/a | Low |

**Implications:** XFA is a second document model (an XML layout engine with scripts) inside PDF. Its fate argues for not rendering dynamic XFA in Izcal's first viewer, showing the PDF's own fallback content and an "open externally" path, with corpus measurement deciding later.

## 10. JavaScript in PDF

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | JavaScript actions entered PDF 1.3 (2000) with Acrobat 4-era forms; Adobe's Acrobat JavaScript API grew to include network, collaboration and file APIs. | 2000 | [Wikipedia](https://en.wikipedia.org/wiki/History_of_PDF) | n/a | Low |
| 10.2 | The API was a main exploit path in the 2008-2009 peak (see 5.3); "disable JavaScript" was the standard vendor workaround. | n/a | [CORE-2008-0526](https://dl.packetstormsecurity.net/0811-advisories/CORE-2008-0526.txt) | 2008 | Medium |
| 10.3 | **Mozilla's design (Firefox 88, 2021):** `eval` rejected as "very unsafe"; a Wasm-compiled JS engine rejected for the built-in viewer as more attack surface, size and slowness; chosen: Firefox's privileged sandbox with a null principal, "at least as safe as executing one from a normal web page". The generic web viewer and tests use QuickJS compiled to Wasm. Field data is cloned in and results cloned out. **I/O APIs (network, disk) were not implemented "for security reasons".** | Firefox 88 | [Mozilla Hacks 2021](https://hacks.mozilla.org/2021/10/implementing-form-filling-and-accessibility-in-the-firefox-pdf-viewer/) | 2021-10-07 | High |
| 10.4 | Why support it at all: telemetry showed forms use JS mostly to validate input and compute fields. About 1,900 of Mozilla's corpus PDFs had JavaScript. | ~1,900 files | same | 2021 | High |
| 10.5 | Chrome's PDFium runs form JavaScript in V8 inside the PDF process; there is no separate user toggle (a 2022 forum thread). | n/a | [Vivaldi forum](https://forum.vivaldi.net/topic/80931/is-javascript-enabled-by-default-in-the-built-in-pdf-viewer) | 2022 | Low |

**Implications:** The safe pattern is a narrow API (forms: validate, format, calculate), no I/O, a separate isolated realm and data passed by copy. Izcal already has its own JS engine, so the decision is about API surface and isolation, not about engine choice.

## 11. Print-to-PDF and tagged output

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 11.1 | Chrome's Print Preview design (draft 14 October 2010): preview by generating a PDF "to generate a preview that most closely resembles what will be printed", shown in the bundled PDF plugin. Reason for a Skia PDF backend: Windows had "no" and Linux "poor" PDF generation, Mac had a high-fidelity backend built in; goal "consistent and size-efficient PDFs". | 2010-10-14 | [Chromium Print Preview design doc](https://www.chromium.org/developers/design-documents/print-preview/) | 2010 | High |
| 11.2 | By October 2010 every major browser except Chrome had print preview (per the same doc). | n/a | same | 2010 | High |
| 11.3 | Skia's PDF backend could not represent some effects (image filters) and fell back to raster for them. Known only from code-review titles seen in search results, not opened. | n/a | none citable (M-1) | n/a | Low |
| 11.4 | **Chrome 85 (25 August 2020) writes tagged PDF** from "Save as PDF", with headings, lists, tables, paragraphs and image descriptions. Stated reasons: screen readers and software that extracts data from PDFs. Headless: `--print-to-pdf` plus `--export-tagged-pdf`. | Chrome 85 | [Chromium blog: Using Chrome to generate more accessible PDFs](https://blog.chromium.org/2020/07/using-chrome-to-generate-more.html) | 2020-07 | Medium |
| 11.5 | iText/PDF Association analysis: Chrome does not claim PDF/UA conformance; the result depends on the page's own semantics. | n/a | [PDF Association: Chrome-plated PDFs](https://pdfa.org/chrome-plated-pdfs-exploring-google-chromes-new-pdf-capabilities) | 2020 | Medium |
| 11.6 | Firefox redesigned print preview in Firefox 81 (22 September 2020) with a built-in "Save to PDF" destination; 81 was transitional and some options came in 82. Firefox's PDF output uses Cairo's PDF surface (not confirmed in prose). | 2020-09-22 | [Mozilla wiki: Printing Refresh](https://wiki.mozilla.org/Firefox/Printing_Refresh/Archives); [Winaero](https://winaero.com/firefox-81-receives-a-new-print-preview-dialog/) | 2020 | Medium (date), Low (Cairo) |
| 11.7 | Safari's "Export as PDF" uses the macOS print system (Core Graphics). No prose on tagged output from Safari was found. | n/a | none read | n/a | Low |

**Implications:** Every browser's print-to-PDF began as a by-product of the printing path, and tags came a decade later (2020). Izcal can design the writer from the accessibility tree and display list together from day one, and must not claim PDF/UA unless it validates the output.

## 12. Linearization ("Fast Web View")

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 12.1 | Linearization puts first-page objects and hint tables at the file's start so a viewer can show page 1 before download ends. It requires HTTP byte-range support and on-demand fetching; without them the whole file downloads. | n/a | [Nutrient guide](https://www.nutrient.io/blog/linearized-pdf.md); [pdfRest](https://pdfrest.com/learning/solutions/optimize-pdfs-for-fast-web-viewing/) | n/a | Medium |
| 12.2 | Benefit is small for small or local files; one vendor's viewer fetches a first chunk of about 128 KB, so smaller files arrive in one request anyway. | ~128 KB | [Nutrient](https://www.nutrient.io/guides/web/performance/linearized-downloads.md) | n/a | Medium |
| 12.3 | A Mozilla maintainer: for large PDFs pdf.js uses "incremental loading via range requests", and display speed depends on how the hosting server is set up. Prose does not say whether pdf.js uses the hint tables or just fetches the trailer and objects by range. | n/a | [Bug 1158610](https://bugzilla.mozilla.org/show_bug.cgi?id=1158610) | 2015 | Medium |
| 12.4 | Whether linearization "paid off" is not established by any measured study found. Object streams (PDF 1.5) and incremental updates break linearization, and many producers do not linearize. | n/a | none (reasoned) | n/a | Low |

**Implications:** Range-request loading helps large remote PDFs whether or not they are linearized. Izcal should measure time-to-first-page on a corpus with and without linearization (black-box against other viewers) before spending design on hint tables.

## 13. Subset and profile standards: PDF/X, PDF/A, PDF/UA, WTPDF, PDF/R

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 13.1 | Profiles by year: PDF/X 2001 (print exchange), PDF/A 2005, PDF/E 2008 (engineering), PDF/VT 2010 (variable print), PDF/UA 2012. | n/a | [Wikipedia](https://en.wikipedia.org/wiki/History_of_PDF) | n/a | Low |
| 13.2 | **PDF/A** began in 2002 under AIIM and NPES with the US courts' administrative office and the Library of Congress, to make PDF preservable and consistently renderable for long periods. ISO approved ISO 19005-1 on 28 September 2005 (published about 1 October), a constrained PDF 1.4: device independent, self-contained, self-documenting. | 2002; 2005-09-28 | [PDF Tools history](https://www.pdf-tools.com/en/resources/pdfa-archive-format/history-of-pdfa/); [Library of Congress FDD](https://loc.gov/preservation/digital/formats/fdd/fdd000125.shtml) | 2005 | Medium |
| 13.3 | PDF/UA-1 (ISO 14289-1, 2012) was built on PDF 1.7. PDF 2.0 redesigned tagged PDF, which led to **PDF/UA-2 (ISO 14289-2, 15 March 2024)** and the PDF Association's free **Well-Tagged PDF (WTPDF)** (late February 2024). New: PDF 2.0 structure elements, MathML, structure destinations, Associated Files. Also fixes ambiguities that made validators disagree. | 2012; 2024-02; 2024-03-15 | [iText: PDF/UA-2 is here](https://itextpdf.com/blog/itext-news/pdfua-2-introducing-new-pdf-accessibility-standard); [axes4](https://www.axes4.com/en/blog/post/2024/pdf-ua-2-vs-well-tagged-pdf); [BFO](https://bfo.com/blog/2024/03/12/bfo_pdf_library_2_28_4_with_pdf_ua_2_and_wtpdf/) | 2024 | Medium |
| 13.4 | WTPDF exists because ISO 14289-2 must be bought; a file conforming to WTPDF's accessibility level also conforms to PDF/UA-2. | n/a | [PDF Association WTPDF example](https://pdfa.org/download-area/examples/WTPDF/apryse-itext-wtpdf.pdf) (snippet) | 2024 | Medium |
| 13.5 | **PDF/R (ISO 23504-1:2020)**, from the TWAIN Working Group's TWAIN Direct work: a strict subset for scanned pages (raster only; JPEG, CCITT G4 or uncompressed; no object streams, no actions, no bookmarks), meant to replace TIFF. Every PDF/R file is valid PDF. | 2020 | [PDF Association: ISO 23504](https://pdfa.org/resource/iso-23504-pdfr/); [PDF/raster overview](https://www.pdfa.org/pdfraster-an-overview/) | 2020 | Medium |

**Implications:** Each profile was driven by a buyer with a concrete need (printers, archives and courts, accessibility law, scanners). For the writer, WTPDF is the free text to anchor tagged output to; PDF/A is a choice for the spec, not a default.

## 14. Attempts to make PDF safer or simpler, and attacks that shaped them

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 14.1 | **DARPA SafeDocs** aims at high-assurance parsers and at "reducing these formats to their safe, unambiguous, verification-friendly subsets", using LangSec. The PDF Association is industry partner (Peter Wyatt as PDF principal investigator). No published "safe PDF subset" standard was found as an outcome. | n/a | [DARPA SafeDocs](https://www.darpa.mil/research/programs/safe-documents); [PDF Association update](https://pdfa.org/presentation/a-safedocs-update-to-industry/) | 2019-2023 | Medium |
| 14.2 | SafeDocs' durable outputs: the **Arlington PDF Model** (machine-readable DOM of every PDF 2.0 object, as TSV files with 12 fields, used alongside ISO 32000-2:2020) and the **UNSAFE-DOCS** corpus of over 5 million files from Common Crawl. Arlington's licence was not confirmed this session (believed Apache-2.0); Izcal may use it as test or reference data only unless it is MIT. | 5.0M+ files; 12 fields | [Arlington PDF Model](https://pdfa.org/resource/arlington-pdf-model/); [Digital Corpora UNSAFE-DOCS](https://digitalcorpora.org/?p=3002) | 2021-2023 | Medium |
| 14.3 | Lesson from SafeDocs: the subset idea turned into tooling (formal models, corpora, parser checks) and into tightening the ISO text through errata, not into a new "safe PDF" profile that producers adopted. This is an inference from what was published. | n/a | as 14.1-14.2 | n/a | Low |
| 14.4 | **Signature attacks:** incremental-saving attacks (2019) and **shadow attacks** (NDSS 2021) are spec-compliant: 16 of 29 viewers, including Acrobat and Foxit, were vulnerable to shadow attacks. CERT-EU: the attack "does not break the signature algorithm" but makes victims sign more than they see. | 16/29 viewers | [NDSS 2021 paper](https://www.ndss-symposium.org/wp-content/uploads/ndss2021_1B-4_24117_paper.pdf); [pdf-insecurity.org](https://pdf-insecurity.org/signature-shadow/shadow-attacks.html) | 2019-2021 | Medium |
| 14.5 | **FORCEDENTRY (CVE-2021-30860):** a PDF disguised as a .gif in iMessage hit an integer overflow in Apple's xpdf-derived JBIG2 decoder; NSO used JBIG2 segment operations as a "weird machine" to build the next stage. Project Zero called it among the most technically sophisticated exploits it had seen. Parsing ran outside the BlastDoor sandbox before iOS 15. | CVE-2021-30860 | [Project Zero: FORCEDENTRY sandbox escape](https://projectzero.google/2022/03/forcedentry-sandbox-escape.html) | 2021-12 to 2022-03 | Medium |
| 14.6 | PDF/R (13.5) is the one "simplified PDF" that shipped as a standard; it succeeded by serving a single narrow use (scans), not by replacing general PDF. | n/a | as 13.5 | 2020 | Medium |

**Implications:** The ambiguous parts of PDF (incremental updates, decoders like JBIG2, rich object graphs) are where both memory bugs and spec-level attacks live. Izcal should use Arlington and UNSAFE-DOCS as test inputs (licences permitting), show incremental-update history in signed files, and treat every decoder as an attack surface in the sandbox.

## 15. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Acrobat 1.0 (1993) | Paid reader, slow uptake; free from 1994 | Reach comes from a zero-cost reader | Viewer is built in, no account or upsell (contrast 8.2-8.3) |
| Adobe Reader plugin | Most-attacked app 2009-2010; JS API a main vector | A native, scriptable viewer with ambient authority is a top target | Separate sandboxed PDF process; no I/O APIs to PDF scripts |
| Reader X sandbox | Broker model, "not a panacea" | Sandboxing contains, it does not remove bugs | Memory-safe parser plus process isolation; both, not either |
| Chrome viewer / PDFium | Licensed C++ engine, sandboxed, open-sourced 2014; steady CVEs | Containment of a C++ engine is a permanent tax | Own Rust engine; fuzz each decoder |
| pdf.js | Safety by running in JS; speed/memory complaints | Safe runtime costs speed; much cost was browser-side | Native Rust, render to Izcal's display list; measure time-to-first-page |
| Firefox forms work | Telemetry-led staging: forms, JS, tags, XFA | Let corpus evidence order the features | Build a corpus and measure feature use before committing |
| Edge (2023) | Adopted Adobe for fidelity, perf, security, text selection, read-aloud | Those five are what users notice | Make them falsifiable targets |
| XFA | Proprietary, never ISO, deprecated 2017, still in the wild | A second document model inside PDF is very costly | Decline dynamic XFA initially; show fallback; measure share |
| Chrome tagged PDF (2020) | Tags added ten years after print-to-PDF | Retro-fitting tags is slow | Write tags from the accessibility tree from the start |
| SafeDocs | Models and corpora, not a new profile | Tightening comes from tests and errata | Use Arlington/UNSAFE-DOCS as test data |
| Linearization | Benefit unmeasured; range loading works without it | Do not design for a feature with no evidence | Range loading first; measure before hint tables |

## Claims not verified

- Camelot paper date: Warnock says 1991, Adobe's timeline says 1990 (1.1).
- PostScript-to-PDF rationale (static objects instead of a program) was not read in the Camelot text (1.2).
- The PDF version feature table (section 2) and Adobe Extension Levels rest on Wikipedia only.
- Whether an Adobe patent licence equivalent to the 2008 one covers ISO 32000-2 (3.3).
- Exact PDF 2.0 encryption deprecations, in particular RC4 and AES-128 by name (4.4); PostScript XObjects "removed" vs deprecated (4.3).
- Chrome's PDFium uses V8 for PDF scripts and Chrome does not render XFA: no prose primary source was read; needs black-box confirmation (6.4, 10.5).
- PDFium licence stated as "New BSD" from press only (6.3).
- Firefox 19 as the version that made pdf.js the default (7.3).
- Legacy Edge PDF reader details and whether Edge removed its legacy engine on 31 March 2024 (8.1, 8.3).
- Opera has no separate PDF history (8.5); no Servo PDF prose found (8.6); why Ladybird chose pdf.js (8.6).
- XFA corporate history dates (9.1-9.2) rest on Wikipedia.
- Firefox print-to-PDF via Cairo; Safari export via Core Graphics (11.6, 11.7). Skia PDF backend raster fallback (11.3) is not citable under M-1.
- Linearization's real-world benefit (12.4): no measured study found.
- Arlington PDF Model licence (14.2): believed Apache-2.0, not confirmed; if not MIT it is test/reference data only.
- SafeDocs produced no adopted "safe subset" profile (14.3) is an inference.
- The PDF Association's no-cost announcement page could not be fetched (HTTP 403); its content is from snippets (3.5).
- Unreachable on 19 October 2026 (fact-check FC-R25): the Apryse sponsorship post (3.5) and the Computerworld Australia article (5.4); claims rest on the other sources in those rows. WebKit Bugzilla entries (8.4) were read as bug prose only.
