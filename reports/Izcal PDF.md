# Izcal PDF

*First-principles specification of Izcal's own PDF support: the PDF viewer (ISO 32000-2 parsing, filters, encryption, incremental updates and repair, content-stream interpretation, fonts in PDF, images and colour, transparency and overprint, annotations and AcroForm, actions, the XFA and JavaScript decisions, signatures, text extraction, search, selection, tagged-PDF accessibility and printing a PDF), the PDF process and its sandbox, how PDF pages reach the graphics spec's display list and canonical raster, and the PDF writer behind print-to-PDF and `window.print()` (vector output from the display list, structure from fragments and the semantic plane, tagged PDF, the PDF/A decision, font embedding and subsetting, deterministic output), with footprint and observability. Date: 21 October 2026. Status: draft 0.2.5, phase 8 applied; decisions accepted (PD-D8 decided by the owner on 20 October 2026). Research notes: `research_notes/Izcal PDF/` ([`frame.md`][FM], `history_postmortems.md`, `formats_specs.md`, `security_robustness.md`, `rendering_text_accessibility.md`, `writer_print.md`, `gaps_opportunities.md`; phase 8: `factcheck.md`, `critique.md`). Relates to: HGBA-2 2.4.4 [M-1, P1–P6, I-1, I-2, S-4a, S-8, C-1][H2]; HGBA-3 [B1, B2, B8, B9, B10, DET-1][H3]; Stage 1 design [Sections 1, 2 (G1–G9), 6, 7.5, 8.1 M7, 13.2; D-S2, D-S10, D-S11][S1]; JavaScript engine spec [JS 6.9, JS-D15, G18, JS-16][JS]; resource-footprint spec [FP 6.4, FP-14, FP-38, FP-D8, FP-D19, FP-S30][FP]; HTML and CSS parsing and style spec [G21, G22, G23, G24][ST]; fonts and text spec [TX-D3, TX-D7, TX-D10, TX-D19, TX-D23, TX-D24, TX-D25, TX-W1, TX-8, G25, G26, G28, G29, 6.18][TX]; networking spec [`hg-compress`, `hg-crypto`, `hg-pki`, G36][NW]; layout spec [LY-D8, LY 6.7][LY]; images and media spec [MD 6.1, MD 6.5, MD-D8, MD-D20, MD-D28, R18, G50, G51, G53, G56][MD]; graphics and rendering spec [GR 6.0, 6.3, 6.4, 6.5, 6.22, 6.27; GR-D3, GR-D6, GR-51; G61, G62, G70][GR]; DevTools spec [IIP][DT]; security scope [IPC-R1, IPC-R7, UPD-R5, SEC-12][SS]; [reconciliation P1, P5][RN]; [reconciliation 2 Q4, Q18, Q24, Section 4][RN2]; [licence audit class 4, OD-L2, OD-L6][LA]; [Phase 3 ledger G-14, G-15, C-21, A-FUT-2, A-FUT-6, A-S1-75][LG]; [shell UI design R18][UI]; accessibility spec (in progress, AX- prefix) for the semantic interface. Links to the security scope and coding standard resolve once they merge.*

*Revision 0.2.5 (22 October 2026): freshness pass: the status line records the decisions accepted, and Section 1's introduction is restated in the past tense now that this spec owns the PDF renderer (G-14 and C-21 closed 21 October 2026). No decision or threshold changes.*

*Revision 0.2.4 (21 October 2026): phase 3 receiver amendment A-PD-1 applied, from the accepted editing spec (draft 0.3, owner decisions of 20 October 2026). **AcroForm field editing runs through `hg-edit-model`'s text-layout and backing traits**, which `hg-pdf-forms` implements, so it uses the same edit operations, movement rules and IME protocol as web content; the fonts spec's caret and selection rules are the editing crates' (editing spec 6.14, 6.19). **Credential mode** applies `hg-edit-model`'s `CredentialPolicy` to PDF password fields, so spelling, autocorrect, IME hints, snapshots and recording redaction treat a field that is or was a password as web content does (G148). **Copy** from a PDF goes through the viewer renderer to the broker, since the PDF process holds no clipboard authority (G114). The dependency table's form-field row points at `hg-edit-model`. No decision or target changed.*

*Revision 0.2.3 (21 October 2026): batch-4 amendment from the shell spec (draft 0.3, SH-D1–SH-D30 accepted by the owner on 21 October 2026). **PD-D2's document password prompt is the shell's `document-password` class** (shell 6.4, 9.5): broker-owned, high risk, the secret goes straight to the document process and is never recorded. 6.1's diagram note, 6.15 and 9.5's security-scope and shell-UI-design rows are restated; no target, threshold or decision changes.*

*Revision 0.2.2 (21 October 2026): batch-4 amendment from the ML service spec (draft 0.3, its decisions accepted by the owner on 21 October 2026). **6.10's OCR paragraph cites ML-OCR and the ML service's gates**: raster in from the content GPU process, text runs out with confidence and reading order, labelled recognised, only with the accessibility grant on; **OCR starts on a user action in the viewer or an assistive-technology read, never on document open**, so an embedded image-only PDF causes no job (ML-12); the "future ML spec owns it (G-15)" clause becomes "the ML service spec owns it (ML-OCR, extended here to OCR)". The component table and 9.5's future-ML row are restated, and PD-D11's decision text is unchanged because it already names the ML service. No target, threshold or decision changes.*

*Revision 0.2.1 (20 October 2026): HGBA-2 2.3.3 source-evidence pass (M-1 part 3 now covers every third-party project's source code): 45 items checked (45 code-host links, all in the research notes; the unlinked claims were checked too): 45 allowed, 0 replaced, 0 measurement, 0 removed. The links are crates.io pages (licence and version facts), repository landing pages used for README prose, Adobe's `cmap-resources` and `agl-aglfn` registries (standards data, PD-D8), the Arlington model (ISO-derived specification data, test-only), the veraPDF corpus (test-only), pdf.js wiki FAQ pages (prose) and licence files. No claim rests on a PDF library's or viewer's source: Acrobat's bundled form scripts and other viewers' `AF*` implementations stay unread (6.17), and values only their source states remain black-box questions. **Owner decision PD-D8 recorded: (a), as recommended**: Adobe's CMaps, CID-to-Unicode tables and the Adobe Glyph List are standards registry data (HGBA-2 2.3.3) and ship with the BSD-3 notice; 6.8, 7, 9.3, 9.4, the risk table and Q-PD-2 are updated, and PD-D8 leaves the legal review. Section 1's M-1 line and Section 7's evidence statement now cite 2.3.3 and cover every third-party project. No target or threshold changed. Per-item log: [`source_evidence_2.3.3.md`](../research_notes/Phase%203%20amendments/source_evidence_2.3.3.md).*

*Revision 0.2 (19 October 2026): phase 8 applied, by a fact-checker and a design critic who did not write the spec. The [fact-check][FK] checked 132 claims (90 correct, 27 partly correct, 10 wrong, 5 unverifiable) and 286 links (6 dead, all in the notes, now replaced or marked; every fragment present; no link to engine source) and listed 10 must-fix and 26 recommended items; all are applied. The [design critique][CR] raised 36 findings (6 High, 18 Medium, 12 Low), 9 engine-source and licence items and 17 reconciliation items; all are applied, three in a different form than proposed (Appendix A). What changed: **every PDF-related process is chosen by the document's renderer process key** (origin where FP-D8 keys renderers by origin), with at most 8† live PDF processes per key (6.1); **range reads are pinned** to one validated copy and the grant is revoked before the first form edit, closing a byte-level shadow attack and an exfiltration channel (6.2); **the broker never acts as a decryption oracle**: it returns only a derived file key, with consent (6.5); **saving an encrypted form re-encrypts** appended objects with the document's own handler, through a separate `hg-crypto-legacy` crate (6.5, G119); **PDF images decode to a new source-samples class P**, so CMYK and spot channels survive, and colour conversion uses `hg-colour`'s one f32 contract (6.4, 6.7); the **process-colour encoding carries shape and opacity** as ISO 32000-2 clause 11 requires (6.6); **PD-D8 now recommends ruling Adobe's CMaps and glyph list registry data**, because building our own would copy the same Adobe data (6.8); the **print assembler is replaced by a keyless joiner that sees only typed descriptors**, so no I-1 exception is needed (6.19); secrets never enter the viewer; content GPU processes get per-producer namespaces; glyph paths, XML placement, residency, hidden documents with unsaved changes, and costs for GPU and SIMD kernels on three ISAs are designed; four targets added (PD-13 to PD-16); staffing 25–36†.*

*Revision 0.1 (19 October 2026): first draft, written from the six research notes by the synthesis author.*

PDF is the one document format every browser must show that the web platform did not design. Izcal had no owner for it until this spec: the Phase 3 ledger recorded the gap as **G-14** and the process conflict as **C-21**, and four accepted specs had already called into a PDF spec that did not exist (fonts: Type 1 and Type 3 for PDF only; media: `hg-jpx`, `hg-jbig2`, `hg-ccitt` inside "the PDF process"; graphics: PDF viewing through "the future PDF spec's display list"; layout: vector print output with structure from fragments). The history has two lessons. **First, PDF readers have been the most attacked desktop software of their era**: Reader files were used in almost half of 663 targeted attacks F-Secure found in 2009 ([HP 5.1][HP]), the Acrobat JavaScript API carried many of the worst bugs ([HP 5.3][HP]), sandboxing a C++ engine contained bugs without removing them ([HP 5.4, 6.5][HP]), a JBIG2 decoder inside a PDF became a "weird machine" in FORCEDENTRY ([HP 14.5][HP]), and even pdf.js, written in JavaScript to avoid native bugs, shipped arbitrary script execution through its font path in 2024 ([SE 2.14][SE]). **Second, every browser stopped building its own PDF engine**: Google took Foxit's code, Apple uses its OS library, Microsoft moved Edge to Adobe's engine in 2023 citing fidelity, performance, security, text selection and read-aloud, and Ladybird bundled pdf.js in 2026 ([HP 6.3, 8.2, 8.4, 8.6][HP]). Izcal must build its own (M-1), so this spec designs it to be safer than any of them by construction and measurably better on the five things Microsoft named.

The central idea is that **a PDF is hostile, untrusted data that Izcal turns into its own trusted representations as early as possible, in one place**: a generated, non-recursive parser in a per-document PDF process with no authority turns the file into one resolved object view, an interpreter turns pages into the graphics spec's display list (rastered canonically by `hg-raster` in the content GPU process), a text model and a semantic subtree; nothing downstream ever sees PDF bytes, and the same view feeds rendering, search, accessibility, forms, signatures and saving, so the parser differentials behind shadow attacks cannot arise inside Izcal. The writer runs the same idea backwards: print output is a pure function of the display list, the fragment tree and the semantic plane, so its bytes are deterministic and its tags come from the accessibility tree. The decisions that matter most:

- **PD-D1 ★ Placement.** One **PDF process per open document**, chosen and charged by the document's renderer process key ((site, container), or origin where FP-D8 keys renderers by origin), at the security scope's strictest tier, with no network, file, clipboard or device authority. It reconciles Stage 1's utility row with the media spec's "PDF process" (C-21). The viewer UI is an Izcal HTML application in the same key's renderer; PDF pages are embedded surfaces, like out-of-process frames.
- **PD-D4 ★ Rendering.** PDF pages lower to the graphics spec's display list, extended with the PDF imaging model's missing primitives (knockout and non-isolated groups, blending colour spaces including CMYK and spot colorants, overprint, soft masks with backdrop and transfer, mesh shadings, tiling cells), and are rastered by the one canonical rasterizer. No second rasterizer exists.
- **PD-D13 ★ JavaScript in PDF.** No general PDF scripting. Form scripts run only through a **declarative form-logic profile**: Izcal implements the Acrobat form functions (`AFNumber_Format`, `AFSimple_Calculate` and their siblings, ISO 21757-1's form subset) natively in Rust and recognises scripts that only call them; other scripts never run, and the user sees what was skipped. A sandboxed subset on Izcal's own jitless engine is an alternative the owner may choose, measured first (PD-S4).
- **PD-D14 ★ XFA.** Dynamic XFA is not rendered: PDF 2.0 deprecates it and it is a second document model with its own layout engine and scripts. Files with an AcroForm fallback render it; dynamic-only files get a plain explanation and safe ways out, never Adobe's "Please wait..." page.
- **PD-D18 ★ Writer.** Print-to-PDF is PDF 2.0, **tagged from the accessibility tree by default** (Well-Tagged PDF), identified as **PDF/A-4**, with Izcal's own font subsetter and variable-font instancer, JPEG pass-through, and **byte-identical output** for identical recorded inputs on every OS and ISA. It never claims PDF/UA automatically; it reports what a person must still check.
- **PD-D8 ★ Data licences.** Adobe's CMap resources and the Adobe Glyph List are BSD-3-Clause, and no MIT metric-compatible substitute for the standard 14 fonts was found. ISO 32000-2 makes the four CJK collections' CMaps a "shall support", and their CID orderings are arbitrary assignments only Adobe publishes, so building "our own" would copy the same data. **Decided (owner, 20 October 2026; HGBA-2 2.3.3): these tables are standards registry data under OD-L6**, like the OpenType registries, and ship with the BSD-3 notice; the fonts spec's aliases gain Symbol and ZapfDingbats.
- **PD-D16 ★ Writer placement.** Per-key writer jobs write complete objects into file segments the broker concatenates without parsing; a keyless joiner sees only typed descriptors. No process holds two keys' content.

Note sources are cited as **HP** ([history and postmortems][HP]), **FS** ([formats and specifications][FS]), **SE** ([security and robustness][SE]), **RT** ([rendering, text and accessibility][RT]), **WP** ([writer and print][WP]) and **GO** ([gaps and opportunities][GO]), with the row number. Primary sources are linked inline for the claims a decision or target depends on.

## 1. The problem and who has it

| Who | What they need from PDF support | Evidence |
|---|---|---|
| **People reading documents** | Every PDF opens fast, looks as its author intended, scrolls smoothly at any size, and can be searched, selected, copied, read aloud and printed; filled forms are kept | 13.5% of 2,313 US government PDFs rendered differently in at least one of eight readers, and "there is no standardized behaviour on the correct rendering of a missing font" ([Kuchta et al., EMSE 2018](https://srg.doc.ic.ac.uk/files/papers/pdf-emse-18.pdf); [RT 1.1–1.3][RT]); Microsoft moved Edge to Adobe's engine for "higher fidelity", "improved performance", "better text selection and read-aloud narration" ([HP 8.2][HP]) |
| **People filling forms** | Government, tax, bank and medical forms that work, calculate, validate and save | 9.26% of 20.6 million Common Crawl PDFs have interactive forms ([GO 1.7][GO], vendor study); XFA forms "used by multiple governments and banks" show "Please wait..." in Chrome and Edge ([GO 2.7–2.9][GO]); Mozilla's telemetry showed "many" forms use JavaScript, mostly to validate and calculate ([HP 7.6, 10.4][HP]) |
| **People who use assistive technology** | The author's tags exposed faithfully, a usable reading order when tags are absent or wrong, and text from image-only pages | In May 2022 Chrome 101 and Edge 101 ignored the tag structure with JAWS, NVDA and VoiceOver; Safari, Preview and Acrobat followed it ([RT 7.1–7.4][RT]); just over half of web PDFs are tagged, many wrongly (P directly under the root in about 1.6 million documents), and Google estimates 12% are image-only ([GO 4.1–4.3][GO]; [RT 8.4][RT]) |
| **People whose documents are attacked** | Opening a PDF must never compromise the browser, leak data, or let a signed document say something other than what was signed | FORCEDENTRY ([HP 14.5][HP]); shadow attacks fooled 16 of 29 viewers, the 2019 signature attacks (universal forgery, incremental saving, wrapping) 21 of 22 desktop viewers, PDFex 23 of 27 ([GO 6.1–6.5][GO]); 26 of 28 applications fell to attacks through standard features alone ([GO 6.6][GO]) |
| **Web developers producing PDFs** | Print-to-PDF that paginates predictably, keeps text, links, headings and fonts, is accessible, and gives the same bytes for the same input so generated invoices and reports can be tested | Chrome got `@page` margin boxes only in Chrome 131 (November 2024) and still lacks running headers and `target-counter()` ([GO 7.1–7.2][GO]); Chrome's output is tagged but claims neither PDF/UA nor PDF/A, falls back to Type 3 for variable fonts and colour emoji, and loses emoji text; WebKit's output is untagged with no links and inflates a JPEG 3.4×; Chrome's two runs differ in dates and WebKit's in a random `/ID` ([WP 1.2–1.11, 2.2–2.7, 6.4–6.5][WP]) |
| **Developers' tools and agents** | Why a page renders as it does, which fonts were substituted, what actions and scripts a file carries, what a signature covers, what is untagged; structured text and structure for agents with consent | No browser viewer shows font substitution, actions, signature coverage, repair or untagged regions ([GO 9.1–9.7][GO]) |
| **Izcal's engineers** | A PDF stack that is memory-safe with no `unsafe`, deterministic (B1), inside its process's footprint budget, anchored to ISO 32000-2 clause by clause, and testable without matching any other engine | Kuchta et al. refused to treat Acrobat as ground truth ("Acrobat Reader also has bugs", [RT 1.2][RT]); Mozilla's PDF reftests suffer anti-aliasing noise ([GO 3.10][GO]) |

**What Izcal's specs already decide.** HGBA-2 2.3.3: M-1 (no engine libraries: pdf.js, PDFium, Poppler, MuPDF, qpdf, lopdf, pdf-rs, hayro, pdf-writer and krilla are excluded; MIT-only shipped code and data; no third-party project's source is ever evidence; Adobe's CMap and glyph-list registries are standards registry data, PD-D8), I-1 (process per (site, container), origin-keyed on desktop under FP-D8), I-2 (zero ambient authority), I-3. HGBA-3: B1, B2, B8 (semantic plane with an author tree and a repair overlay), B9 and B10 (agents; AI off by default). Stage 1: the process map's utility row ("PDF parsing and rendering"), M7 ("PDF via utility processes"), 13.2 (printing and print-to-PDF are Stage 2). Fonts and text ([TX 6.18, TX-D7][TX]): the PDF process links `hg-fontfmt`, `hg-font-vm` and `hg-glyph-raster`; Type 1 and Type 3 are supported for PDF only, in that process, under fuel; print-to-PDF subsets fonts and honours `OS/2.fsType`; G25 (only `hg-fontfmt` reads font bytes). Images and media ([MD 6.18, R18][MD]): `hg-jpx`, `hg-jbig2`, `hg-ccitt` are built and staffed by the media spec and run inside the PDF process, which follows G50–G53 like a media process. Graphics ([GR 6.27][GR]): PDF viewing uses "the future PDF spec's display list, rastered by these crates"; vector print output takes its appearance from the display list and structure from layout's fragments; canonical integer pixels (GR-D3). Layout ([LY 6.7, LY-D8][LY]): paged layout and `window.print()`'s layout are Stage 1; the print environment is one recorded input. Networking: `hg-compress` holds the deflate encoder and decoders ([RN P5][RN]); `hg-crypto` is the only crate with cryptographic primitives (G36). Footprint: identical limits on every device (FP-D19), decoder-class processes ≤ 2 MiB† idle (FP-38). The accessibility spec (in progress) owns the interface PDF content uses to reach the tree. **Rule range proposed for this spec: G110–G119** (free in every registry; registered through Section 9.5).

**Success, in terms someone can measure later**: on a fixed conformance and web corpus, every page renders within a stated tolerance of an independent float64 reference renderer and bit-identically on every device; zero `unsafe` and zero parser or decoder outside the PDF process; no PDF action leaves the document without a person's action; first page shown and memory per document at or better than the best shipping viewer; tagged PDFs exposed with their structure and untagged ones with a labelled inferred structure, judged by screen-reader users; print-to-PDF output that passes PDF/A-4 and Well-Tagged PDF machine checks and is byte-identical across OSes and ISAs (Section 8).

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1991–93 | **Camelot, PDF 1.0, Acrobat 1.0** (15 June 1993) | A page description fixed into static objects, derived from PostScript ("Interchange PostScript"); text, images, pages, links, bookmarks | PostScript is a program, so a page is known only by running it; a fixed object model gives random access, search and safer display ([HP 1.1–1.3][HP]) | Unprofitable for about four years; Warnock: "nobody got it" ([HP 1.4][HP]) |
| 1994 | **Free Reader** (Acrobat 2.0) | Reader given away | "We had to get people out there using it"; rival free readers ([HP 1.5][HP]) | Made PDF universal; the founders later named security as what they overlooked ([HP 1.6][HP]) |
| 1994–2006 | **PDF 1.1 to 1.7** | Encryption (1.1), AcroForm and Flate (1.2), signatures and **JavaScript** (1.3), **transparency, JBIG2, tagged PDF** (1.4), JPEG 2000, **object streams**, **XFA** (1.5), AES and OpenType (1.6) ([HP 2.1–2.6][HP]) | Each version served a buyer (forms, print, archives, accessibility) on Acrobat's schedule | One file format now carries three programming models (content streams, JavaScript, XFA) and two document models (PDF objects, XFA's XML) |
| 1999–2003 | **XFA** (JetForm, then Adobe) | An XML form language with its own layout and scripts, embedded in PDF 1.5 | Enterprise forms authored in Designer, re-laid out at run time ([HP 9.1–9.2][HP]) | Never an ISO standard; deprecated in PDF 2.0; government and bank forms still use it; Firefox wrote a separate XFA layout engine, still partial in 2026 ([HP 9.3–9.5][HP]) |
| 2008–2010 | **The Reader exploit era** | Reader files in about 49% of 663 targeted attacks (2009); malicious PDFs 56% to 80% of exploits ([HP 5.1–5.2][HP]); `util.printf()` and `Collab.getIcon()` JavaScript bugs ([HP 5.3][HP]) | A native, scriptable plugin with the user's full authority, in every browser | Reader X's Protected Mode sandbox (November 2010): "not a panacea" ([HP 5.4][HP]); NPAPI removed from Chrome (2015) and Firefox (2017) ([HP 5.5][HP]) |
| 2008 | **ISO 32000-1** | PDF 1.7 becomes an ISO standard; Adobe's public patent licence | Governments asked for open formats ([HP 3.1–3.2][HP]) | The patent licence covers 32000-1 only and excludes "any updated version" ([FS 1.7][FS]) |
| 2010 | **Chrome's built-in viewer** (Chrome 8) | A licensed Foxit engine as a sandboxed plugin | "A sandbox" layer against PDF malware ([HP 6.1–6.2][HP]) | Open-sourced as PDFium in 2014; a steady stream of CVEs since ([HP 6.3–6.5][HP]) |
| 2010 | **Chrome Print Preview and Skia's PDF backend** | Print preview by generating a PDF | Windows had no PDF generation and Linux a poor one; "consistent and size-efficient PDFs" ([HP 11.1][HP]) | Still Chrome's writer (producer "Skia/PDF m155"); Firefox is moving to the same backend on Linux and macOS in 2026, while Windows still prints through Cairo ([WP 1.1, 2.9–2.10][WP]) |
| 2011–13 | **pdf.js** (Firefox 19 default) | A PDF engine in JavaScript and HTML5 | A native plugin "enlarges the trusted code base"; JS is "immune to this class of problems" ([HP 7.1–7.3][HP]) | Speed and memory complaints, much of the cost browser-side ([HP 7.4–7.5][HP]); a 2024 arbitrary-JavaScript bug through its font path showed a memory-safe runtime does not remove logic bugs ([SE 2.14][SE]) |
| 2014 | **pdf.js memory work** | Peak memory cut 1.2× to 7.8× by limiting cached page canvases | Each Retina page canvas exceeded 20 MiB ([GO 3.1][GO]) | The lesson is general: page rasters, not the document, dominate memory |
| 2017, 2020 | **ISO 32000-2 (PDF 2.0)** | Deprecates XFA, Movies and Sounds, security handler revisions 1–5; new structure namespace; associated files ([FS 4.1, 10.1, 9.1][FS]) | A vendor-neutral PDF with "no proprietary technology" ([HP 3.4][HP]) | Free of charge since 5 April 2023 (Adobe, Apryse, Foxit) with Errata Collection 3 ([FS 1.1–1.2][FS]); none of a 1,045-file sample of the 2021 crawl was PDF 2.0 ([FS 2.7][FS]) |
| 2019–21 | **Signature and encryption attacks** (Ruhr University Bochum) | Incremental-saving, signature-wrapping, shadow, certification and PDFex attacks | They use legitimate features, not memory bugs ([GO 6.1–6.6][GO]) | 16 of 29 viewers fell to shadow attacks; viewers show "signed" without showing what was signed ([GO 6.7][GO]) |
| 2019–23 | **DARPA SafeDocs** | High-assurance parsers, the Arlington PDF Model, the UNSAFE-DOCS and untruncated Common Crawl corpora | Reduce formats to "safe, unambiguous, verification-friendly subsets" ([HP 14.1–14.2][HP]) | Left tools and corpora, not an adopted safe profile ([HP 14.3][HP]); 20 parsers over 1.04 million files showed many behaviour differences ([FS 2.5][FS]) |
| 2020 | **Chrome 85 tagged PDF** | "Save as PDF" writes tags from the accessibility tree | Screen readers and data extraction ([HP 11.4][HP]) | Ten years after print-to-PDF; no PDF/UA or PDF/A claim, no XMP ([WP 1.2–1.3][WP]) |
| 2020–21 | **Firefox forms, scripts, tags, XFA** (Firefox 84, 88, 89, 93) | AcroForm, a sandboxed PDF JavaScript subset with no network or disk APIs, tagged-PDF exposure, XFA | Staged by telemetry ([HP 7.6, 10.3][HP]) | Precedent for a narrow script API; XFA remains partial ([HP 9.5][HP]) |
| 2021 | **FORCEDENTRY** (CVE-2021-30860) | A PDF in iMessage; JBIG2's segment operations used to build a computer | The decoder ran outside the BlastDoor sandbox ([HP 14.5][HP]) | Every decoder inside a PDF is an attack surface on its own |
| 2023 | **Edge moves to Adobe's engine** | Replaces its PDFium-based viewer | Fidelity, performance, security, text selection, read aloud ([HP 8.2][HP]) | Criticised for branding and paid upsell ([HP 8.3][HP]) |
| 2024 | **Well-Tagged PDF and PDF/UA-2** | Tagging for PDF 2.0; WTPDF free, PDF/UA-2 the ISO version | PDF 2.0 redesigned tagged PDF ([HP 13.3–13.4][HP]) | About 0.9% of web PDFs claim PDF/UA, almost all version 1 ([GO 4.4][GO]) |
| 2025 | **Typst 0.14** | Tagged by default; PDF/UA-1 and every PDF/A part; refuses PDF/UA export when alt text is missing ([WP 3.1–3.3][WP]) | A document tool owns its structure | Sets the bar for producers: tagged by default, conformance on request, refuse or warn on violations |
| 2026 | **Ladybird bundles pdf.js** ([HP 8.6][HP]); **PDF-AAM** still an Editor's Draft ([RT 6.3][RT]) | — | — | No browser engine besides Izcal writes its own PDF engine; the accessibility mapping is still being standardised |

### 2.2 Technology mismatches: assumptions that no longer hold

| Assumption baked into older designs | Why it no longer holds | Consequence for Izcal |
|---|---|---|
| A PDF viewer is an application that may act for the user (open files, launch programs, submit data, run scripts) | Browsers give content zero ambient authority (I-2); the Reader exploit era came from exactly that authority ([HP 5.1–5.4][HP]) | The PDF process holds no authority; every external effect is a typed request a person approves (G118) |
| Parsing is lenient and each reader's leniency is its own business | Parser differentials are an attack class (shadow and wrapping attacks, [GO 6.1–6.3][GO]) and a fidelity problem (13.5% inconsistent, [RT 1.1][RT]) | One specified repair grammar, one resolved view for every consumer (PD-D3, G112) |
| Containment of a C++ engine is enough | PDFium kept producing CVEs inside its sandbox ([HP 6.5][HP]); Reader X's sandbox was "not a panacea" ([HP 5.4][HP]) | Memory safety by construction (no `unsafe`, generated parsers) **and** a sandbox, not one or the other |
| A memory-safe runtime is enough | pdf.js's 2024 bug turned font data into executed JavaScript ([SE 2.14][SE]) | No code is generated from document data; glyph programs are interpreted under fuel (TX-D10); no script engine in the PDF process (G115) |
| Rendering may differ by machine; reftests need tolerances | Izcal's raster is canonical integer arithmetic (GR-D3); Mozilla's PDF reftests suffer anti-aliasing noise ([GO 3.10][GO]) | PDF pages render bit-identically everywhere; differences are bugs, not noise |
| The screen is RGB, so convert CMYK and spot colours to RGB at fill time | PDF's transparency is defined in the group's blending colour space; overprint and spot colours only make sense in their own channels ([FS 6.7][FS]; [RT 2.1–2.2][RT]) | Groups blend in their declared space, including CMYK and spot colorants (PD-D5) |
| Print-to-PDF is a by-product of a 2D drawing library | The writer loses structure, link targets and text mappings when it sees only a canvas ([WP 2][WP]) | The writer reads the display list and the semantic plane directly (PD-D16) |
| Output files may carry the wall clock and random identifiers | Reproducible output is a solved convention (SOURCE_DATE_EPOCH, pdfTeX, Typst) and Izcal is deterministic by design (B1) ([WP 6.1–6.3][WP]) | Writer bytes are a pure function of recorded inputs (G117) |
| Tags can be trusted | Half of web PDFs are tagged and many trees break ISO/TS 32005's rules ([GO 4.1–4.5][GO]) | Author structure and Izcal's repairs are kept apart, as B8 already does for HTML (PD-D11) |

## 3. Today: the landscape and its gaps

**Every browser contains its PDF engine; none removes its bugs.** NVD lists 135 records matching "pdfium" (6 in 2014, 23 in 2019, 1 in 2022, 23 so far in 2026); of their weakness tags, 43 are use-after-free, 29 out-of-bounds write, 13 integer overflow and 11 heap overflow, and 18 to 21 descriptions name the JPEG 2000 decoder (OpenJPEG), while none names forms, fonts appear in 2 and JBIG2 in 1 ([SE 1.1–1.3][SE]; [fact-check FC-M1][FK]). Chrome's own "PDF" component has 31 records, 18 of them use-after-free, and in 2026 one advisory reaches code "outside the sandbox" ([SE 1.4][SE]). pdf.js has a different class: no memory corruption, but a same-origin break exploited in the wild to upload local files (CVE-2015-4495), script injection through Type 4 functions compiled to JavaScript (CVE-2018-5158), and arbitrary JavaScript through a font matrix inserted into generated glyph code (CVE-2024-4367, affecting releases from April 2014 to April 2024) ([SE 1.6, 2.12–2.14][SE]). Edge adopted Adobe's engine with stated hardening but no published sandbox design ([SE 9.5][SE]). Three patterns recur across 18 years: **an image codec reached unexpectedly** (JBIG2 in 2009 and 2021), **a font program parsed with privilege** (CoolType SING 2010, BLEND 2015, Adobe Type Manager 2020), and **code generated from document data** ([SE 2.4–2.14, 3.1–3.3][SE]).

**Standard features are dangerous on their own.** Müller et al. tested 28 applications with 209 files: 17 contacted an attacker's server when a file was opened, 11 leaked form data silently on print or close, 12 of 18 Windows viewers leaked NTLM hashes, 6 launched an executable with no dialog; only 532 of 294,586 crawled PDFs (0.18%) use Launch at all ([SE 4.1–4.7][SE]). A 578-byte stream of chained Flate filters decoded to 10 GB (18,470,265:1) and "fully affected" Chrome and Firefox: per-tab processes stopped infinite loops, not memory exhaustion ([SE 4.3, 7.2][SE]). Safari and Edge (2020) blocked every dangerous path except a link the user clicked ([SE 4.8][SE]). Signature forgery fooled 21 of 22 desktop viewers, shadow attacks 15 of 28 (16 of 29 in the paper), certification attacks 24 of 26, and PDFex exfiltrated decrypted content from 23 of 27 ([SE 5.1–5.5][SE]; [GO 6.1–6.5][GO]). Of 94 crafted edge cases, 63 rendered differently across viewers, which both masks content and fingerprints the viewer ([SE 6.1][SE]). The PDF JavaScript API "is descriptive rather than prescriptive": Acrobat exposes 6,742 properties and PDF-XChange 114, so property enumeration identifies the viewer ([SE 8.1][SE]).

**Fidelity is undefined in practice.** 13.5% of US government PDFs rendered differently in at least one of eight readers, and the study's authors could not say which reader was right for missing fonts ([RT 1.1–1.3][RT]). No browser viewer has a published result on the Ghent Output Suite's overprint, transparency, soft-mask, spot-colour and ICC patches (48 by trade-press accounts, to be confirmed from the suite, [RT 1.6][RT]). No neutral benchmark of render time or memory exists; the one careful study (pdf.js, 2014) found page canvases above 20 MiB each on a Retina display ([GO 3.1–3.3][GO]; [RT 3.1, 3.6][RT]).

**Forms, scripts and XFA divide the browsers.** About 9% of web PDFs carry forms, 2% signatures, 4% optional content and 2.5% are encrypted with an empty password; annotations are dominated by links (265 million) and widgets (20 million) ([GO 1.7–1.10][GO], a vendor study). Firefox runs form JavaScript (since 88) and partial XFA (since 93); Edge supports neither ("JavaScript forms aren't supported"); Chrome has no XFA by default ([RT 11.1–11.4][RT]). Nobody publishes how many web PDFs use JavaScript or XFA ([GO 1.12][GO]), so Izcal measures it (PD-S1).

**Accessibility is half done.** The tagged share passed 50% in July 2025 and grows about 1.5 points a year; about 2% have empty trees; ISO/TS 32005 nesting violations number in the millions; 0.9% claim PDF/UA ([GO 4.1–4.4][GO]). In 2022, Chrome and Edge ignored tags with three screen readers, Firefox followed them partly, and PDFKit and Acrobat did best ([RT 7.1–7.4][RT]). Chrome now OCRs image-only PDFs on the device in 77 languages ([RT 8.4][RT]). The accessibility mapping standard, PDF-AAM, is an Editor's Draft that does not yet cover paragraphs, tables, figures, lists, Alt, ActualText or Lang ([RT 6.3][RT]).

**Print-to-PDF is a by-product.** Chrome 155 writes tagged PDF 1.4 with links and passes JPEG bytes through, but no XMP, no PDF/A or PDF/UA identification, no outline by default, no object streams, Type 3 fallbacks for variable fonts, collection fonts and colour emoji (emoji text lost), sequential subset tags, and differing dates per run; WebKit's output through Quartz is untagged with no links, re-encodes a JPEG 3.4× larger, draws emoji as 32-pixel images, maps one Japanese character to the wrong code point, and writes a random `/ID` ([WP 1.1–1.13, 2.1–2.7, 6.4–6.5][WP]). Typst tags by default and Prince tags whenever a PDF/UA or PDF/A level-a profile is requested; both offer PDF/UA and PDF/A on request ([WP 3.1–3.4, 4.4][WP]).

### 3.1 What current tools cannot see or do, with evidence

| Gap | Evidence | What Izcal can do (Section 6) |
|---|---|---|
| **What a signature covers** and what changed after signing | 15–16 of 28–29 viewers fooled by shadow attacks; viewers warn on added content, rarely on removed ([SE 5.3][SE]) | Render the signed revision and the current one from the same resolved view and diff their display lists (PD-D17) |
| **Which actions and scripts a file carries** before anything runs | 26 of 28 applications exploitable through standard features ([SE 4.1][SE]) | The document-facts panel lists every action, script and external reference; none runs on open (PD-D15, PD-D26) |
| **Which fonts were substituted**, and by what | No viewer reports it; substitution has no standard answer ([RT 1.3][RT]; [GO 9.1][GO]) | Substitution is a logged, explainable decision (PD-D7) |
| **Why a page renders differently** (repaired file, unsupported feature, colour management, overprint) | 13.5% inconsistent; 63 of 94 edge cases diverge ([RT 1.1][RT]; [SE 6.1][SE]) | Repair log, feature log and per-operator spec anchors in the IIP `pdf.*` domain |
| **Untagged and mis-tagged regions** | Millions of ISO/TS 32005 violations ([GO 4.3][GO]) | Author tree and Izcal's inferred tree side by side, with the rule each repair applied (PD-D11) |
| **Deterministic PDF rendering and writing** for tests | Mozilla's PDF reftests suffer anti-aliasing noise ([GO 3.10][GO]); browsers' print output changes per run ([WP 6.4–6.5][WP]) | Canonical pixels (GR-D3) and byte-identical writer output (PD-D21) |
| **Print explainers** (why the page broke here, which fonts were embedded) | No browser exposes it ([GO 7.6][GO]) | Layout's break explainers plus a writer report (PD-D26) |
| **Local reflow of a tagged PDF** | Acrobat's Liquid Mode reflows in Adobe's cloud, with limits ([GO 4.10–4.12][GO]) | A reflow view generated from the (repaired) structure tree, on the device (Stage 2, 6.15) |

## 4. First principles

### 4.1 Users' real tasks

| Task | What it needs from the engine |
|---|---|
| "Show me this document" (read, scroll, zoom, find a page) | Fast first page from partial bytes, every page bit-exact at any scale, bounded memory however large the file |
| "Find this, copy that, read it to me" | One text model per page with correct Unicode, reading order and word boundaries, shared by find, selection, copy, read-aloud and assistive technology |
| "Fill, sign and send this form" | Field editing with the platform's input methods, calculation and validation, appearance generation, saving that preserves the original bytes and signatures, and a submit that the person sees and approves |
| "Is this document what it claims to be?" | What each signature covers, what changed after signing, and which actions and scripts the file carries, before anything runs |
| "Print it" | Vector output with real text, the same pages as on screen, never a soft raster |
| "Make a PDF of this page" | Predictable pagination, real fonts and text, links, headings as bookmarks, tags, small files, identical bytes for identical input |
| "Why does it look wrong?" (author, developer, support) | Which fonts were substituted, which features were skipped or repaired, which clause of ISO 32000-2 governs a drawing operation |

### 4.2 Platform primitives

- **ISO 32000-2:2020 with Errata Collection 3** is the anchor ([FS 1.1–1.2][FS]); ISO 32000-1 is read only for legacy behaviour. Clause numbers are confirmed from the PDF Association's errata site ([FS 0][FS]). Until the licence of the free copy is read, the spec cites and paraphrases, never quotes ([FS 1.4][FS]).
- **Codecs**: RFC 1950/1951, ITU-T T.4, T.6, T.81, T.88, T.800 ([FS 3.3][FS]), built by the media spec.
- **Satellites**: ISO/TS 32001–32004 (hashes, signature algorithms, AES-GCM, integrity), ISO/TS 32005 (structure namespaces), ISO 21757-1 (ECMAScript for PDF; whether it defines the Acrobat `AF*` form functions is unverified, Q-PD-14), ISO 19005-4 (PDF/A-4), ISO 14289-2 (PDF/UA-2), WTPDF 1.0, ETSI EN 319 142-1 (PAdES) ([FS 4–5, 8.4, 9.2, 11][FS]).
- **Web platform**: HTML's PDF viewing support (`navigator.pdfViewerEnabled`, the fixed `navigator.plugins` list, `<embed>` and `<object>` for `application/pdf`) ([HTML Standard, PDF viewing support](https://html.spec.whatwg.org/multipage/system-state.html#pdf-viewing-support)); HTML's printing steps for `window.print()`, `beforeprint` and `afterprint` ([WP 8.7][WP]); MIME Sniffing's `%PDF-` signature ([MIME Sniffing](https://mimesniff.spec.whatwg.org/#identifying-a-resource-with-an-unknown-mime-type)); CSS Paged Media 3 and Fragmentation 3 through the layout spec.
- **Accessibility mapping**: PDF-AAM, a W3C Editor's Draft dated 12 October 2026 that maps headings, links, sections and articles but leaves P, Table, Figure, L, Artifact, ActualText, Alt and Lang open ([RT 6.3][RT]); WCAG 2.2's PDF techniques PDF1–PDF23 ([RT 6.5][RT]).
- **Izcal's own primitives**: the display list and canonical raster (GR-D3, GR-D6), the font stack with fuel (TX-D3, TX-D10), `hg-colour`'s one numeric contract (MD-D28), the semantic plane with a repair overlay (B8), the substrate's recorded inputs (B1, JS-D15), typed IPC (B2).

### 4.3 Izcal's principles that bind this design

- **P1 first.** A PDF is hostile. The process that parses it holds one document and no authority (I-2); everything it produces is checked again by its consumer (IPC-R7's validated reads, G62's generated display-list reader).
- **P2 means ISO 32000-2, not another viewer.** Correctness is the clause, an independent reference renderer and conformance suites; other viewers are black-box differential signals only (M-1 part 4). Where the standard is silent (font substitution, repair, reading order of untagged pages), Izcal writes one rule, records it as a deviation or a black-box observation, and files it with the PDF Association's errata process ([FS 1.9][FS]).
- **P3.** Every damaged file opens to the extent the repair grammar allows, every fuel limit has a specified, deterministic fallback, and a PDF process crash costs one document, never the tab or the browser.
- **P4.** Page rasters dominate memory ([GO 3.1][GO]; [RT 3.1][RT]), so rasters live in the content GPU process under the graphics spec's tile budgets, and the PDF process keeps only the resolved model and bounded decode caches.
- **B1.** Rendering, text extraction, form logic, saving and writing are pure functions of the bytes, the recorded inputs and the build.
- **M-1.** No PDF library, no PDF writer crate, no font subsetter, no codec from another project. Generic crates only, all MIT-available (Section 7).

### 4.4 Assumptions kept and broken

| Inherited assumption | Keep or break | Reason |
|---|---|---|
| A built-in viewer is required, not a download prompt | **Keep** | HTML defines PDF viewing support; every shipping desktop browser has one; Android Chrome added inline viewing only in 2025 and users wanted it ([GO 5.1–5.2][GO]) |
| The viewer runs in a separate, sandboxed process | **Keep, and tighten**: one process per document, not per site or per tab | P1; a hostile PDF from a site should not read the user's other PDFs from the same site (6.1) |
| The viewer is a JavaScript application drawing to a canvas with an invisible DOM text layer | **Break** | The text layer approximates selection and needed `aria-owns`, which, commenters on Mozilla's post note, VoiceOver does not support ([HP 7.7][HP]); Izcal draws pages into its own display list and exposes the semantic tree directly |
| PDF pages render through a PDF-specific rasterizer | **Break** | One canonical rasterizer (GR-D3) gives identical pixels, one set of proofs and fuzzers, and no second attack surface in the GPU path; its missing primitives are added once (PD-D4) |
| Transparency and colour are approximated in RGB | **Break** | ISO 32000-2 clause 11 defines blending in the group's colour space; RGB approximation is the classic divergence ([RT 2.2][RT]) |
| Lenient parsing is each viewer's private heuristic | **Break** | One specified repair grammar, logged and tested (PD-D3) |
| PDF JavaScript runs with an Acrobat-like API | **Break** | The API was the main exploit path ([HP 5.3, 10.2][HP]); the use that matters is form logic, which a declarative profile covers (PD-D13) |
| XFA is part of PDF | **Break** | PDF 2.0 deprecates it ([FS 8.5][FS]; [HP 4.1][HP]); it is a second document model (PD-D14) |
| Signed means trustworthy, shown as a green tick | **Break** | 16 of 29 viewers fell to shadow attacks ([GO 6.1][GO]); Izcal shows coverage and post-signing changes (PD-D17) |
| Printing a PDF hands the original file to the OS print system | **Break** | That puts an unsandboxed PDF interpreter (the OS's or the printer's) in front of hostile bytes; Izcal prints its own re-emitted, normalised PDF or raster (PD-D22) |
| Print-to-PDF output is untagged or tagged as an afterthought, with wall-clock dates | **Break** | Tags from the accessibility tree and deterministic bytes from day one (PD-D18, PD-D21) |
| Linearization hint tables speed up web viewing | **Break (do not depend on them)** | No measured benefit was found; range loading works without them ([HP 12.1–12.4][HP]) |
| Non-embedded fonts are substituted by whatever the system has | **Break** | The substitute is part of the rendering; Izcal uses its own metric-compatible faces, identically everywhere (PD-D7) |

## 5. Lessons learned

| Project | What happened | Lesson | How Izcal applies it (ideas only, M-1) |
|---|---|---|---|
| Acrobat 1.0 and free Reader | Paid reader stalled; free reader won ([HP 1.4–1.5][HP]) | Reach comes from a zero-cost, built-in reader | Built in, no account, no upsell (contrast Edge, [HP 8.3][HP]) |
| Adobe Reader plugin, 2008–2010 | Most-attacked application; the JavaScript API carried many bugs ([HP 5.1–5.3][HP]) | A scriptable viewer with the user's authority is a top target | No authority in the PDF process (G114); no general scripting (PD-D13) |
| Reader X sandbox; Chrome's sandboxed PDFium | Containment helped, CVEs continued ([HP 5.4, 6.5][HP]) | A sandbox contains, memory safety removes | Both: no `unsafe`, generated parsers, and the strictest sandbox tier |
| pdf.js | Safe runtime, speed and memory costs; font data became executed JavaScript in 2024 ([HP 7.4–7.5][HP]; [SE 2.13–2.14][SE]) | Memory safety does not stop logic bugs that turn data into code | Nothing generated from document data is ever executed; fonts are interpreted under fuel |
| pdf.js memory, 2014 | Page canvases dominated memory; 4.4× and 7.8× cuts from caching less ([GO 3.1][GO]) | Budget page rasters in bytes, not pages | Rasters live in the content GPU process's tile budget (6.21) |
| Firefox forms, scripts, tags, XFA (2020–21) | Staged by telemetry; scripts with no network or disk APIs ([HP 7.6, 10.3][HP]) | Let measured use order features; narrow script APIs | Corpus census first (PD-S1); declarative form-logic profile (PD-D13) |
| Chrome and Edge in 2022 | Tags ignored by both with three screen readers ([RT 7.2][RT]) | An accessibility path that bypasses the document's structure fails users | The PDF structure tree feeds the same semantic plane as HTML (PD-D11) |
| Kuchta et al., 2018 | 13.5% inconsistent; Acrobat not ground truth ([RT 1.1–1.3][RT]) | "Match the market leader" is not a definition of correct | A float64 reference renderer and the clause are the oracle (PD-2) |
| Bast and Korzen, 2017 | 14 extraction tools; Type 3 and line-end hyphens unsolved ([RT 5.2–5.4][RT]) | Extraction quality is decided by ligatures, hyphens and reading order | One extractor for find, copy, a11y; hyphen joins checked against TX's dictionaries (PD-D9) |
| XFA in Firefox | A separate layout engine and substitute fonts, still partial ([HP 9.4–9.5][HP]) | A second document model never reaches parity | Decline dynamic XFA; explain it honestly (PD-D14) |
| Shadow, wrapping, certification and PDFex attacks | Legitimate features abused; viewers hid what was signed ([GO 6.1–6.7][GO]) | Show coverage, not a tick | Signed-revision rendering and display-list diff (PD-D17) |
| FORCEDENTRY | A JBIG2 decoder outside BlastDoor became a computer ([HP 14.5][HP]) | Every decoder reachable from a PDF is attack surface | All decoders in the PDF process under G50–G53, fuel in every loop (G113) |
| SafeDocs | Models and corpora, not a safe profile ([HP 14.1–14.3][HP]); 20 parsers disagreed over 1.04 million files ([FS 2.5][FS]) | Tightening comes from tests, specified repair and errata | Arlington and the SafeDocs corpora as test-only inputs; repair grammar published |
| Chrome tagged output (2020) | Tags a decade after print-to-PDF, no XMP, no conformance claim ([HP 11.4][HP]; [WP 1.3][WP]) | Retrofitting structure is slow | Tags and PDF/A-4 identification from the first writer release |
| Typst 0.14 | Tagged by default; refuses PDF/UA with missing alt text ([WP 3.1–3.2][WP]) | Producers can enforce accessibility at export | A writer report of every machine-checkable failure; no automatic PDF/UA claim (PD-D18) |
| Linearization | Benefit never measured; range loading works without it ([HP 12.4][HP]) | Do not design for unmeasured features | Range loading for every file; hint tables ignored (PD-D10) |
| Edge's switch to Adobe (2023) | Five reasons: fidelity, performance, security, selection, read aloud ([HP 8.2][HP]) | Those are what users notice | Each is a falsifiable target (Section 8) |

## 6. Design

### 6.0 Architecture at a glance

```
 BUILD TIME  .hgpdf descriptions (ISO 32000-2 clauses 7, 8.4-8.11, 9, 12, 14) --hg-pdfgen--> hg-pdf-syntax lexer and
             object parser, xref and object-stream readers, typed dictionary accessors, content-operator decoder,
             CMap and Type 4 function parsers, Kani harnesses, fuzz grammars, spec anchors
             character-collection and encoding tables (PD-D8) --hg-pdf-data-build--> engine-image data

 BROKER       navigation, MIME decision (application/pdf), launches the PDF process for the document; parses nothing;
              receives typed action requests (navigate, submit, save, print, copy) and asks the person (G118)
                 | sealed bytes or a range-read grant for one URL                  ^ typed requests
                 v                                                                  |
 PDF PROCESS per open document, chosen and charged by the document's renderer process key; strictest tier
   hg-pdf-syntax  hg-pdf-xref (revisions, repair, the one resolved view, G112)  hg-pdf-crypt (via hg-crypto)
   hg-pdf-filter (Flate/LZW/ASCII/RunLength/predictors; hands DCT/JPX/JBIG2/CCITT to hg-jpeg, hg-jpx, hg-jbig2, hg-ccitt)
   hg-pdf-interp: content streams -> display-list chunks (.hgidl) per page, text runs, marked content
   hg-pdf-font: font dictionaries -> TX faces (hg-fontfmt, hg-font-vm), Type 3 glyph procedures, substitution
   hg-pdf-text (text model, find, selection)  hg-pdf-struct (structure tree -> semantic subtree, inferred structure)
   hg-pdf-forms (AcroForm, annotations, appearances, actions, incremental save)  hg-pdf-sig (CMS, coverage)
   hg-pdf-write (serializer, shared with the writer)
        | display-list deltas (G62)        | semantic subtree (AX interface)   | page geometry, find results,
        v                                  v                                    | outline, form-field events
 CONTENT GPU PROCESS of the key       BROKER accessibility mirror               v
   hg-raster (canonical, with the     (stitched under the viewer's     RENDERER of the key
   PDF group extensions, PD-D4)        page element, like an OOPIF)     hg-pdf-viewer (Izcal HTML application: toolbar,
        |                                                               thumbnails, outline, find bar, document facts);
        v                                                               page surfaces embedded like out-of-process frames;
 COMPOSITOR (one)                                                       (password prompt: the shell's document-password class, 6.1)

 PRINT-TO-PDF (Stage 2 product; core in Stage 1)
   renderer: paged fragment tree (LY-D8) -> hg-paint display lists + structure records from the semantic plane
   PDF WRITER JOB per key (same binary, writer role): hg-pdf-print (display list -> complete objects in a granted
   object-number range, fonts via hg-font-subset, images and encoded JPEGs from the key's media process)
     -> writes its own segment of a broker-granted output file
   BROKER concatenates segments in page order without parsing them
   PRINT JOINER (keyless utility job, typed descriptors only): writes page tree, structure tree root, ParentTree,
   outline links, metadata and the cross-reference stream -> download manager or OS print path
```

### 6.1 Processes and placement (PD-D1)

A PDF's bytes are the most dangerous input a browser shows that is not script: every class in Section 3 (image codecs, font programs, generated code, cycles, bombs, actions) begins with them. The design question is how small and how unprivileged the process that first reads them can be.

| Option | What a compromise or a bomb reaches | Cost | Verdict |
|---|---|---|---|
| (a) Parse and render in the site's renderer (pdf.js's model) | That site's renderer: its script, cookies' effects, every same-site document; a bomb takes the tab | No extra process | Rejected: puts a second large parser in the process that already runs the site's script, and a PDF bug would reach the site's pages |
| (b) A pooled utility process (Stage 1's utility row) | Every site's PDFs in the pool | One process | Rejected: violates I-1 |
| (c) One PDF process per renderer key, like the media process | Every open PDF of that key | One process per key with PDFs | Considered: matches the media spec's keying, but a hostile PDF on a shared host (a mail service's attachments, a drive's shared files, all one site and often one origin) could read the person's other PDFs from it |
| (d) **One PDF process per open document**, chosen and charged by the document's **renderer process key** ((site, container), or origin wherever FP-D8 keys renderers by origin, which is every desktop class), strictest tier | That one document's bytes and state, which the attacker supplied | One small process per open PDF (≤ 3 MiB† empty, PD-F1) | **Recommended** |

**Why per document.** The data a PDF process holds is the person's document, and the realistic attacker is another document from the same site: same-site hosting of user-uploaded PDFs is the common case for mail, drives and learning systems. The cost is one process per open PDF; people open few at once, and a hidden document's process is discarded and rebuilt (6.21). The NDSS authors reach the same advice, "by sandboxing it – similar to a tab in a modern web browser", from the denial-of-service side ([SE 4.9, 12.4][SE]). **The key** is the document's renderer process key, exactly as the graphics and media specs key their per-site processes ([GR 6.1][GR]; [MD-D5][MD]): it decides the partition of range reads, which content GPU process rasters the pages, which renderer hosts the viewer, and whose ledger pays. All three are chosen by that one key, so a PDF never shares a content GPU process with a renderer FP-D8 separated from it. **At most 8† PDF processes per renderer key are live at once** (canonical, FP-D19); further PDF navigables show a placeholder ("PDF: click to view", with the response's size and file name) until one is visible and a slot frees, hidden documents being reclaimed first (6.21). Under memory pressure Izcal reduces per-process cost, never co-locates documents (FP-D8).

**What the PDF process holds (G114).** The document's bytes, as a read-only handle (IPC-R1) to a sealed copy that it parses only after copying or validating as IPC-R7 requires, or a **pinned range-read grant** for the document's one URL in its fetch partition (6.2); its own display-list channel and chunk namespace in its key's content GPU process (6.6); a channel to the viewer; a read-only font endpoint for the Izcal font set and the OS tier that answers only the lookups the substitution rule makes, never enumeration (as the media process has for SVG images, [MD 6.1, MD-D20][MD]); and a request endpoint to the broker for actions (6.13). The image decoders are linked in, as G50 permits. It holds no network socket, no file handle, no clipboard, no device, no storage, and no JavaScript engine (G115). It runs at the security scope's **strictest (utility) tier**, where MD-D5 places the media process: compute and IPC only.

**What it produces.** Display-list chunks per page (6.6), a text model per page (6.9), a semantic subtree (6.10), a document model summary (page count, sizes, outline, page labels, form-field descriptors, actions, signatures, repair and feature logs), and saved bytes (6.12). Every output is a typed message read by a generated reader in its consumer; no consumer parses PDF syntax (G110).

**The viewer (PD-D2).** The viewer UI (toolbar, page navigation, zoom, thumbnails, outline, find bar, document-facts panel, form popups) is **`hg-pdf-viewer`, an Izcal HTML application** like the shell and DevTools frontend (D-S2, D-S11), loaded as the document of the navigable that holds the PDF, in the renderer of the PDF's key, at an **internal opaque origin** so no page of that site can script it. Each page is an **embedded surface** owned by the PDF process, like an out-of-process frame: the compositor composes its tiles, input on it is routed to the PDF process by the broker's hit test over hit-test regions clamped by the broker ([GR 6.0][GR]; [RN2 Q4][RN2]), and its semantic subtree is stitched under the viewer's page element by the accessibility spec's cross-process mechanism, which must accept a non-renderer producer (9.5). **Secrets never enter the viewer.** The document password prompt is the shell's **`document-password` class** (shell 6.4, 9.5): broker-owned, high risk, with the secret going straight to the document process and never to the prompt renderer or the shell, and never recorded (one ledger entry, no secret); choice popups and field text are drawn by the PDF process in its page surface; the viewer receives only the document-model summary, and for encrypted documents that summary omits outline titles, find-result snippets and field values, which the PDF process draws in its own surfaces. With that rule, running the viewer in the key's renderer gives that key nothing it lacks.

| Alternative for the viewer | Why not |
|---|---|
| A canvas plus an invisible DOM text layer (pdf.js) | Selection and accessibility become approximations; needs `aria-owns`, which commenters on Mozilla's post note VoiceOver does not support ([HP 7.7][HP]) |
| Viewer UI in the shell renderer | Document content (outline titles, field names) would sit in the shell's process with every other site's tab titles and more; the shell should hold only what it shows for all tabs |
| PDF process draws the UI too | Puts a UI toolkit and input handling into the most exposed process |

**Embedding.** Izcal supports HTML's PDF viewing: `navigator.pdfViewerEnabled` reflects whether PDFs are currently viewed inline (false when the person chose "always download PDFs" or the emergency switch turned viewing off, and `navigator.plugins` and `mimeTypes` are then empty), and otherwise `navigator.plugins` is HTML's fixed list ([HTML Standard](https://html.spec.whatwg.org/multipage/system-state.html#pdf-viewing-support)); `<iframe>`, `<embed>` and `<object>` with an `application/pdf` resource create a nested navigable showing the viewer; `Content-Disposition: attachment` downloads (PD-D24). The navigable's sandboxing flags restrict the PDF's actions (no navigation of the top level without `allow-top-navigation`, no form submission without `allow-forms`, no form logic without `allow-scripts`). A `blob:` PDF takes its creator's key; a PDF opened from `file:` or `data:` gets an opaque key of its own.

### 6.2 Loading: bytes, ranges and linearization (PD-D10)

The network service fetches the document as a normal navigation (cache, CORS-free, partitioned by the networking spec's key). For a response whose `Accept-Ranges` and length allow it, the broker gives the PDF process a **range-read grant**: the process asks for byte ranges of that one URL, served by the partition service from cache or network with the same partition and credentials mode as the navigation, never another URL. The process reads the trailer first (end of file), then the objects the first visible page needs, then the rest in the background as idle work. **Hint tables are not used**: no measured benefit was found ([HP 12.1–12.4][HP]), they are often wrong and break with incremental updates, and range reads by object offset give the same first-page behaviour on linearized and non-linearized files alike. Linearization dictionaries are still parsed and validated, so a malformed one is a logged repair, not a crash. Range responses are recorded inputs (B1), so a recorded session replays the same arrival order. A server that ignores ranges gets a full download with progressive parsing from the start.

**Range reads are pinned, and the grant is a bounded channel.** A server could answer two reads of the same bytes differently, showing the digest check one revision and the renderer another, and a compromised PDF process choosing the offsets of credentialed requests could encode what it holds (typed field values) in them. So: a range grant is issued only when the first response carries a strong validator; every later request carries `If-Range` with it; a response whose validator, total length or `Content-Range` total differs ends range mode, and the document is fetched whole and re-parsed from that one copy, with the change logged. Every byte read is kept in a **sealed per-document copy** for the document's lifetime (counted in PD-F2) and never fetched again, so every consumer reads identical bytes, and **signature verification runs only after the whole file is in that copy** (6.14). The broker serves only offsets inside the declared length, coalesced to 64 KiB†-aligned blocks, at most 4,096† requests per document, and **revokes the grant when the whole file is present or before the first form-field edit, whichever comes first**: the viewer finishes the download before a field becomes editable. Once the first page is shown, background fetching proceeds in file order at a fixed rate with on-demand blocks mixed in, so the request pattern reveals little beyond the first jump.

### 6.3 Syntax, revisions and repair (PD-D3)

**One generated parser.** The lexer (7.2), object parser (7.3), cross-reference tables and streams (7.5.4, 7.5.8), object streams (7.5.7), stream extents (7.3.8.2) and trailer (7.5.5) are generated by `hg-pdfgen` from `.hgpdf` descriptions written from ISO 32000-2's clauses, each production carrying its clause anchor (G111), the way TX generates font readers (G29) and MD generates container parsers (G51). The generator emits a non-recursive recogniser with an explicit stack, Kani harnesses per accessor (no panic, no overflow, no out-of-bounds read for any input up to a bounded size), structure-aware fuzz grammars, and typed accessors for every dictionary the engine reads (catalog, pages, resources, fonts, images, annotations, fields, structure elements). **Arlington's data is not read at build time** unless the owner rules it usable (Q-PD-3): it is Apache-2.0 ([FS 12.2][FS]); it is used test-only to generate conformance cases and as a validator oracle.

**Revisions, not a merged dictionary.** Incremental updates (7.5.6) are kept as an ordered list of **revisions**, each with its xref section and its byte range. The **resolved view** (the object table at the latest revision) is computed once by `hg-pdf-xref` and is the only way any crate reads objects (G112); a view at an earlier revision is available for signature checking (6.14) and the document-facts panel. Hybrid-reference files (7.5.8.4) are resolved by the clause's rule, and the choice is logged.

**Strict first, then a specified repair grammar.** Every file is first read strictly. If strict reading fails, the **repair grammar** runs, and each repair is a named, numbered rule with a black-box observation behind it (M-1 part 2), never an ad hoc guess:

| Repair | Trigger | Rule (summary; exact text in the repair grammar) |
|---|---|---|
| R1 header search | `%PDF-` not at offset 0 | Accept within the first 1,024 bytes† (a widely reported legacy tolerance, [FS 2.9][FS], Low; measured at PD-M0) |
| R2 xref offset wrong | Entry does not point at `n g obj` | Scan for the object header within a window; else R4 |
| R3 broken `/Length` | `endstream` not where the length says | Use the first `endstream` keyword after the data start, bounded by the next `obj` header |
| R4 xref reconstruction | Missing or unparsable xref | Scan the whole file once for `n g obj` headers and trailers; later definitions win; object streams indexed after the scan |
| R5 trailer recovery | No usable trailer | Take `/Root` from the last catalog found by R4 |
| R6 cycles and dangling references | `/Prev`, `/Parent`, `/Kids`, `/Next`, outline or object-stream `/Extends` loops | Visited sets on every walk (G113); a dangling reference is `null` (7.3.10) |
| R7 page tree damage | Missing `/Count`, wrong `/Type`, inherited attributes missing | Count leaves; inherit from the nearest ancestor; fall back to a Letter `MediaBox`† |

The share of real files that need each repair is unknown ([FS 2.8][FS]); PD-S1 measures it on the untruncated Common Crawl corpus (test-only), and rules that never fire are dropped. A repaired document is marked so: the document-facts panel says so, signature verification reports "document structure was repaired" (6.14), and text extraction proceeds.

**Limits (G113).** Every limit is canonical (identical on every device, FP-D19), has a specified fallback, and is checked in G18's checked arithmetic:

| Quantity | Limit† | On exceeding |
|---|---|---|
| Revisions (`/Prev` chain) | 4,096 | Older revisions ignored; logged |
| Indirect-reference chain while resolving one value | 64 | Value is `null` |
| Objects in the resolved view | 2²⁴ | Document refused with a clear error |
| Decoded bytes per stream | 256 MiB, and output ≤ 1,000× input after the first 1 MiB ([SE 4.3, 7.4][SE]) | Stream truncated; logged |
| Decoded bytes per document, live at once | 512 MiB, inside the PDF process's canonical 1 GiB heap limit (PD-F2) | Least recently used decoded data dropped and re-decoded on demand; reaching the heap limit itself ends the document with an explanation (6.21) |
| Filters chained on one stream | 8 ([SE 7.4][SE]) | Stream treated as empty |
| Form XObject, pattern and Type 3 nesting | 32 combined depth | Inner content skipped |
| Content-stream operators per page render | 10⁷ (fuel) | Rendering of the page stops at that point; the page shows a "too complex" marker; logged |
| Image pixels per image | 2²⁸, after decode-time downscaling to the needed resolution | Image drawn as a grey placeholder |
| Type 4 function program | 64 KiB of operators, operand stack 100 | Function returns its range's lower bounds |
| Mesh shading patches | 2²⁰ per shading | Shading drawn with its `/Background` or not at all |
| Annotation and field count per page | 65,536 | Excess not shown; logged |

These are first-principles placeholders; PD-S1 checks that no file in the corpus that renders correctly elsewhere exceeds them, and the black-box limits of other viewers are measured for comparison ([SE 7.4][SE]).

### 6.4 Filters and decoders

`hg-pdf-filter` implements ASCIIHexDecode, ASCII85Decode, RunLengthDecode and the TIFF and PNG **predictors** (7.4.4.3) itself; FlateDecode uses the networking spec's `hg-compress` inflate under the limits above; LZWDecode (with `EarlyChange`) uses the media spec's `hg-entropy` LZW decoder, which gains the `EarlyChange` parameter (amendment, 9.5); DCTDecode uses `hg-jpeg` (with the Adobe APP14 colour transform and inverted CMYK); JPXDecode uses `hg-jpx` (with `SMaskInData` and the PDF rule that the JPX colour space overrides `/ColorSpace` when absent); JBIG2Decode uses `hg-jbig2` with `/JBIG2Globals`; CCITTFaxDecode uses `hg-ccitt` with PDF's parameters (`K`, `EncodedByteAlign`, `BlackIs1`, `Columns`, `Rows`). All four image decoders are built and staffed by the media spec and follow G50–G53 inside the PDF process (A-FUT-2). They decode to a **new class P (source samples)**, produced only for the PDF process: samples in the image's own colour space (Gray, RGB, CMYK, Lab, N-channel), 8 or 16 bits per component, unconverted, with the Adobe transform and CMYK inversion applied as PDF requires. The media spec's classes S and W (MD-D8) would already have converted CMYK and spot channels to RGB, which PD-D5 cannot use; class P is an amendment to MD-D8 (9.5), and the web's classes are unchanged. Decoding is **at the resolution the page needs**: DCT and JPX decode at reduced scale where their structure allows (DCT's scaled IDCT, JPX resolution levels), so a 600 dpi scan viewed at 100% never materialises at 600 dpi. **Format reachability is an allow-list by filter name**, never by sniffing the stream's bytes, which is what let FORCEDENTRY's PDF reach a JBIG2 decoder through a `.gif` name ([SE 2.9, 6.3][SE]). Crypt filters are applied before any other filter (7.6.6).

### 6.5 Encryption (PD-D25)

The viewer decrypts every standard security handler revision R2 to R6 and the AES-GCM extension of ISO/TS 32003 ([FS 4.1–4.6][FS]), because encrypted files exist (2.5% of web PDFs are encrypted with an empty open password, [GO 1.9][GO]), even though R1–R5 are deprecated in PDF 2.0 ([FS 4.1][FS]). The primitives come from `hg-crypto` (G36): SHA-256, SHA-384 and AES are already pinned there, and SHA-512 (same `sha2` crate) is added for R6. The legacy primitives live in a **separate crate, `hg-crypto-legacy`** (L2s, owned by the networking spec under G36), the only crate that depends on RustCrypto `md-5` 0.11, `rc4` 0.2 and `cbc` 0.2 (MIT OR Apache-2.0, [GO 11.10–11.13][GO]); a crate rather than a Cargo feature, because features unify across a workspace build, and `hg-graph-lint` proves it is in no binary's closure but the PDF process's (G119; NW amendment, 9.5).

**What legacy crypto may do (G119).** Decrypt under any handler the document uses; **encrypt only to append an incremental update to a document already encrypted with that handler**, because a saved filled form must encrypt its new strings and streams with the document's own scheme (7.5.6, 7.6.3), RC4 for R2–R4, AES-128-CBC for AESV2, AES-256-CBC for R6; and AES-256 (R6) for an explicit "save with a password" (Stage 2). Never RC4 or MD5 for a new document.

**The public-key handler** (7.6.5) needs the person's private key, which never enters the PDF process, and the broker never parses hostile CMS. The PDF process parses the recipient structures and sends the broker a typed request: the recipient's issuer and serial, the encrypted key blob (≤ 512 B†), the bytes of all `/Recipients` strings, and the handler's revision and `EncryptMetadata` flag. The broker shows a consent surface naming the document and the certificate; on approval it decrypts the blob through the OS key store, **derives the file encryption key itself** with 7.6.5's hash, and **returns only the derived key**. The raw plaintext never leaves the broker, so the request is not a decryption oracle; requests are bound to one document and limited to one per recipient per document session. It ships at PD-M3.

**Permissions** (print, copy, extract) are shown in the document-facts panel; Izcal honours "no copy" and "no print" for its copying and printing commands. PDF 2.0 deprecates the accessibility-extraction permission bit and tells readers to ignore it (ISO 32000-2 7.6.4.2, Table 22, per secondary sources; confirmed against the no-cost copy at PD-M0, Q-PD-7), so Izcal ignores it, although 0.4% of web PDFs still set it ([GO 1.9][GO]). **PDFex defences** ([SE 5.5][SE]): an encrypted document whose objects are partly unencrypted (other than those the standard exempts) is opened read-only with forms disabled and a warning, cannot be saved with changes, and no decrypted content can leave through actions without a person's approval (G118). Passwords are never stored. The writer emits no encryption by default (PDF/A forbids it).

### 6.6 Content-stream interpretation and the display list (PD-D4)

`hg-pdf-interp` executes a page's content stream (8, 9.4, Annex A) with an explicit graphics-state stack (8.4.2), resolving resources through the resolved view. It never generates code: Type 4 functions and glyph programs are interpreted under fuel (the opposite of pdf.js's compiled functions and glyphs, [SE 2.13–2.14][SE]). Its output is **display-list chunks in the graphics spec's `.hgidl` format**, one sequence per page, content-addressed and sent as deltas to the content GPU process, which reads them with the generated reader and checks their semantic limits (GR-D6, G62). The content GPU process now has more than one producer per key, so **each PDF process has its own display-list channel and chunk namespace** there: the consumer computes or verifies every chunk hash itself, never resolves a hash across namespaces, and charges display-item and tile bytes to the document's own line on the key's ledger with a per-document cap†, so one document cannot poison another's chunks or evict the site's pages (amendment, 9.5). The display list is the only rendering representation: the same chunks feed the screen, thumbnails, printing (6.16), signature diffs (6.14), DevTools and canonical captures.

**Numbers.** PDF numbers are parsed exactly as decimal mantissa and exponent, then converted once to the graphics spec's fixed point (i64 with 32 fractional bits; matrices composed in i128 with one rounding, [GR 6.4][GR]), with saturation and a logged event on overflow. Colour components and blend inputs are fixed point. PDF functions (Types 0, 2, 3, 4) are evaluated in IEEE f64 through `hg-math` with JS 6.9's discipline (no FMA contraction, one `hg-math`, fixed evaluation order; MD G55) and quantised once; **colour-space conversion happens only in `hg-colour` under MD-D28's one f32 contract** (CalGray, CalRGB, Lab, ICCBased, the default CMYK characterisation, and tint transforms sampled into tables), so a profile converts identically in a PDF and in an HTML image. Those are the only floating-point paths to observable output (G116). Results are therefore identical on every OS and ISA.

**Text.** Glyphs reach the display list as `hg-glyph-raster` coverage masks produced in the PDF process for axis-aligned text up to 256 px† em at device scale, and as bounded-format outlines otherwise (any skew or rotation that is not a multiple of 90°, render modes 1, 2 and 4–7, pattern-filled text, and text inside process-colour groups, where coverage is applied as shape). Masks go to a per-document atlas region in the key's content GPU process under the document's own upload budget. No font byte leaves the PDF process (G26, G110).

**Most of PDF lowers directly**: paths with nonzero and even-odd fill (8.5.3), strokes with joins, caps, miter limits and dashes (the graphics spec's stroker, SVG 2's dash algorithm), clipping including text clipping modes 4–7 (glyph outlines in the bounded path format), images with interpolation flags, image masks and stencil masks, axial and radial shadings (radial with two circles, as Canvas 2D's gradient already needs), the 16 blend modes (11.3.5 is the same set Compositing 1 defines), constant alpha, and soft masks of the alpha kind.

**What the display list lacks, and the decision.** ISO 32000-2's imaging model is a superset of CSS's ([FS 6.7][FS]):

| PDF feature | Clause | Missing from the display list today |
|---|---|---|
| Knockout groups | 11.4.6 | Each element composites with the group's initial backdrop, not with earlier elements |
| Non-isolated groups | 11.4.5 | The group starts from its backdrop; CSS treats every stacking context as isolated |
| Blending colour space per group, and spot colorants beside it | 11.3.4, 11.6.6, 11.7.3 | CMYK blending spaces; spot colorants carried with the group |
| Overprint and overprint mode | 8.6.7, 11.7.4 | Per-colorant replace-or-keep semantics |
| Soft masks of luminosity kind with backdrop colour `BC` and transfer `TR` | 11.6.5 | Luminance masks exist; `BC` and `TR` do not |
| Mesh shadings, types 4–7 | 8.7.4.5.5–8.7.4.5.8 | Gouraud triangles and Coons/tensor patches |
| Function-based shading, type 1 | 8.7.4.5.2 | A two-input colour function |
| Tiling patterns | 8.7.3 | A repeated vector cell under its own matrix |
| Type 3 glyphs | 9.6.4 | Glyphs that are content streams (lowered as content, 6.8) |

| Option | What it costs | Verdict |
|---|---|---|
| (a) Approximate: convert all colour to sRGB at fill time, flatten knockout and overprint, rasterize meshes and patterns into images in the PDF process | Little; but wrong exactly where viewers already diverge ([RT 2.1–2.3][RT]), and rasterizing in the PDF process needs a rasterizer there | Rejected: P2 |
| (b) A PDF rasterizer in the PDF process, producing images | A second rasterizer with its own bugs, numerics and fuzzers; pixels differ from the canonical raster; images cost memory at every zoom | Rejected: one canonical rasterizer (GR-D3) |
| (c) **Extend the display list and `hg-raster` with a PDF group profile**: knockout and non-isolated group flags on effect nodes, with `AIS` and text knockout (`TK`) honoured; a blending colour space per group (Gray, RGB or CMYK, device or CIE-based, 11.3.4) with a new **process-colour encoding**: per-pixel 16-bit **shape and opacity** channels kept separately as 11.3.7 and 11.4.8 compute them, beside premultiplied 16-bit colorants of the blending space, plus up to 28† **spot channels** carried beside the process space under 11.7.3's rules (non-separable blend modes act on process channels and Normal on spots); a page needing more distinct spot colorants converts the excess to their alternate space at fill time, logged; used only by groups that need it; overprint as a per-colorant mask on fill items; soft masks with `BC` and a `TR` table; a **colour-mesh item** (triangles with per-vertex colour in the shading's space, interpolated in integer barycentric coordinates, patches subdivided in the PDF process to a 1/4† device-pixel flatness at the page's scale); a **tiled sub-list item** (a cell's display list, step and matrix, rastered once per scale in the content GPU process); function-based shadings sampled by the PDF process into a colour-mesh at the needed resolution | Work in `hg-raster` and the display-list types (graphics spec's crates), done by this spec's imaging track under the graphics owner's review; validation limits for each new item | **Recommended (PD-D4)** |

**Cost.** Process-colour groups are allocated per tile at the channel count the page actually uses (4 process plus shape and opacity is 12 B per pixel; the 28-spot worst case is 68 B), charged to the document's GPU line; a page whose group working set would exceed 64 MiB† renders its groups in bands. Every new item has a CPU path, an integer GPU compute path under G61 and per-ISA SIMD kernels (x86-64, arm64, riscv64) with scalar references under G54, all bit-identical (PD-4).

**Conversion to the screen.** At a process-colour group's end, its channels convert to the page encoding (S8 or E16, [GR 6.5][GR]) through a table built by `hg-colour` from the group's colour space (ICC profile, or the output intent for Device spaces, PD-D6, or the tint transforms of Separation and DeviceN), applied by integer lookup (G61's rule). The cost lands only on pages that use those features: **ISO 32000-2 Annex Q** gives the normative test for whether a page uses transparency ([FS 6.7][FS]), and the interpreter computes the same flags for overprint and spot colours; a page without them never allocates a process-colour group. Spike **PD-S2** measures render time and memory of the process-colour encoding on the Ghent Output Suite and the corpus's CMYK pages; the reversal trigger is a page class where it costs more than 2×† the S8 path's time with no visible difference in the reference renderer's output, in which case groups that need neither overprint nor non-separable blending convert to the page encoding at fill time.

**Rendering rules the standard leaves to the viewer**, each written once, anchored and tested: **stroke adjustment** (10.7.5) snaps strokes thinner than 1 device pixel to whole pixels when `SA` is true; zero-width lines draw 1 device pixel wide (8.4.3.2); **conflation seams** between abutting shapes ([RT 2.4][RT]) are removed by the rasterizer's conflation fallback (16† samples per pixel where a pixel's minimum and maximum winding numbers differ by more than one, [GR 6.4][GR]), so maps and CAD drawings render without hairline gaps; halftones, transfer functions, black generation and undercolour removal (10.4–10.6) are device controls and are ignored on screen and in print, as the clause permits; **rendering intents** (8.6.5.8) select `hg-colour`'s intent for ICC conversions.

### 6.7 Colour (PD-D5, PD-D6)

Every PDF colour space (DeviceGray, DeviceRGB, DeviceCMYK, CalGray, CalRGB, Lab, ICCBased, Indexed, Pattern, Separation, DeviceN; 8.6) resolves to a `hg-colour` descriptor. ICC profiles embedded in the PDF are parsed by the media spec's generated `hg-icc` inside the PDF process (G50), v2 and v4, never iccMAX ([MD 6.5][MD]). DeviceRGB and DeviceGray are sRGB-encoded as the web treats untagged content. Device colour inside a document with a **PDF/A or PDF/X output intent** (14.11.5) is interpreted through that intent's profile. **Overprint simulation is on by default** for documents that set overprint parameters (unlike Acrobat's PDF/X-only default, [RT 2.1][RT], Low), because the author asked for it and process-colour groups make it exact; a viewer setting turns it off.

**The default CMYK characterisation (PD-D6)** is needed for DeviceCMYK without an output intent, the most common case. ISO 32000-2 10.4.2 gives a naive conversion formula; no MIT-licensed CMYK profile exists, MD-D8 forbids shipping ICC-authored profiles, and standard characterisation data sets (CGATS TR 001 for SWOP, FOGRA39) are published by industry bodies whose terms are unchecked ([RT 9.4][RT], Q-PD-4).

| Option | Result | Verdict |
|---|---|---|
| (a) ISO 32000-2's 10.4.2 formula | Defined, simple, but over-saturated and hue-shifted for photographs | Interim only |
| (b) **An Izcal-built CMYK characterisation**: `hg-colour` builds a CLUT from a published characterisation data set (if the owner rules it standards data) or from Izcal's own measurements of a press standard's printed targets, generated in code like MD-D8's built-in spaces | Close to what professional tools show; identical everywhere | **Recommended** |
| (c) The OS's default CMYK profile | Differs by OS; breaks canonical pixels | Rejected |

### 6.8 Fonts in PDF (PD-D7, PD-D8)

**Embedded fonts go through the text stack** (TX-D7, G25): Type 1 (`FontFile`; eexec and charstring decryption are part of the Type 1 reader), bare CFF and CID-keyed CFF (`FontFile3` `Type1C` and `CIDFontType0C`), TrueType (`FontFile2`), and OpenType (`FontFile3` `OpenType`) are read by `hg-fontfmt` and interpreted by `hg-font-vm` with TX-D10's canonical fuel, in the PDF process only (G26 keeps them out of every privileged process). `hg-pdf-font` turns a font dictionary into a TX face plus PDF's mapping from character codes to glyphs: simple-font encodings (`/Encoding`, `/Differences`, the built-in encodings of Annex D), the TrueType cmap selection rules for symbolic and non-symbolic fonts (9.6.5.4, [RT 4.3][RT]), CID-keyed fonts with CMaps and `CIDToGIDMap` (9.7), and vertical writing (`/W2`, `/DW2`). Glyph advances come from the PDF's `/Widths` and `/W` arrays, not the font's metrics, as the standard requires, so text positions are exactly the author's. Legacy files may omit `/Widths` for the standard 14 fonts (allowed before PDF 1.5); their advances then come from Izcal's aliases' advance tables, which TX-8 holds equal to the standard metrics as measured black-box, never from Adobe's AFM files.

**Type 3 fonts are content, not fonts.** A Type 3 glyph is a content stream (9.6.4), so `hg-pdf-interp` executes it, under the shared nesting and operator fuel, with its own resources; `d1` glyphs inherit the text colour as a stencil, `d0` glyphs carry their own colour. Each executed glyph is cached as a display-list sub-list per (font, glyph, matrix class) inside the document. This is an amendment to the fonts spec's text, which lists Type 3 among its formats: the font stack supplies nothing for Type 3 except glyph-cache accounting (9.5).

**Non-embedded fonts** (Kuchta's top inconsistency, [RT 1.3, 2.7][RT]) are substituted by one written rule, logged per font:

1. **The standard 14** (Times, Helvetica, Courier in four styles, Symbol, ZapfDingbats; 9.6.2.2) map to **Izcal's metric-compatible aliases** (TX-D19, TX-D24), whose widths contract already covers the PostScript core families. Symbol and ZapfDingbats have no alias today ([RT 4.11][RT]); the fonts spec adds two faces (about 190 and 200 glyphs) to wave TX-W1 with their built-in encodings, costed by its type-design track (amendment, 9.5). Until they ship, their glyphs render from the Izcal set's symbol and dingbat coverage by Unicode, with a logged approximation.
2. **Other names** pick an Izcal face by the font descriptor's flags (serif, fixed-pitch, script, italic, bold weight from `/FontWeight` or `/StemV`) and the name's family hints, then **scale each glyph horizontally to the `/Widths` advance**, so line lengths and positions stay exact even when shapes differ.
3. **CID fonts without embedding** use the character collection's script to choose an Izcal Han, Hangul or Kana face, or the interim OS tier (TX-D25) until one ships, in which case the page is labelled non-canonical in the facts panel and the IIP and is excluded from PD-4; vertical forms follow the collection's vertical CMap.

PDF rendering is not observable by any page script, so the OS tier's fingerprinting cost does not apply here; the rule still uses Izcal faces first so PDFs render identically everywhere.

**Data the font path needs, and its licence (PD-D8).** Three data sets are needed: the **predefined CMaps** of the four required character collections (Adobe-CNS1-7, Adobe-GB1-5, Adobe-Japan1-7, Adobe-KR-9; a processor "shall support" them, 9.7.5.2, [FS 7.3][FS]), the **CID-to-Unicode mappings** of those collections (for text extraction when no `ToUnicode` exists), and a **glyph-name-to-Unicode list** for simple fonts (the fallback when a font has no `ToUnicode`). Adobe publishes the first two as `cmap-resources` and the third as the Adobe Glyph List, all **BSD-3-Clause** ([RT 4.4–4.5][RT]). The single-byte encodings (StandardEncoding, WinAnsiEncoding, MacRomanEncoding, PDFDocEncoding, Symbol and ZapfDingbats built-ins) are tables in ISO 32000-2 Annex D itself.

The code-to-CID assignments of those collections are **arbitrary orderings Adobe defined**; they cannot be derived from JIS, GB, Big5, KS X 1001, Unicode or WHATWG data, and the Glyph List's roughly 4,000 legacy names (`afii10017`, `commaaccent`) cannot be derived from Unicode names. Taking them from Adobe's technical notes would copy the same data under a stricter licence than the BSD-3 files.

| Option | Consequence | Verdict |
|---|---|---|
| (a) **Owner rules Adobe's CMap resources, CID-to-Unicode tables and Glyph List registry data under OD-L6**, alongside the OpenType registries the licence audit already allows: ISO 32000-2 makes the four collections' CMaps a "shall support" (9.7.5.2) and relies on the glyph list for simple-font text; the data ships unchanged with its BSD-3 notice | One ruling; no data work; Q-PD-2 becomes moot | **Decided** (owner, 20 October 2026; HGBA-2 2.3.3) |
| (b') Generate everything derivable (the code-to-Unicode direction of each CMap from the encoding standards and WHATWG indexes; algorithmic `uniXXXX` names) and ship CID orderings and legacy glyph names only if (a) is granted | Without (a), CID fonts with no embedded CMap and no `ToUnicode` lose text extraction and non-embedded CID fonts fall back by Unicode: a "shall support" broken | Not taken; the fallback only if (a) is ever reversed |
| (c) Support only embedded CMaps and `ToUnicode` | CJK files without embedded CMaps fail to render | Rejected |


### 6.9 Text: one model for find, selection, copy and reading (PD-D9)

The interpreter emits, per page, a **text model**: glyph runs with fixed-point positions, font, size, rendering mode, the marked-content and structure references around them, and a Unicode mapping per glyph. The mapping follows one chain, recorded per glyph so the IIP can show where each character came from: `ActualText` on enclosing marked content (14.9.4) → the font's `ToUnicode` CMap (9.10.3) → the character collection's CID-to-Unicode table (PD-D8) → the simple font's encoding and glyph names (Annex D and the glyph-name table) → the TrueType font's Unicode cmap read backwards → unknown, exposed as U+FFFD and flagged. Bast and Korzen found that ligatures, diacritics drawn as separate glyphs and line-end hyphens separate good extractors from bad ones, and that Type 3 glyphs and mandatory hyphens defeated all 14 tools they measured ([RT 5.1–5.4][RT]). Izcal's rules, each written down and tested:

- **Words and lines** come from an Izcal-defined, deterministic layout analysis over the glyph runs: baseline clustering in the text's own direction, gaps compared with the font's space advance and size, columns found by whitespace separation; the algorithm and its thresholds† are specified, not tuned per file, and are scored on a benchmark built like Bast and Korzen's (test-only, Q-PD-9).
- **Reading order** is the structure tree's order when the page is tagged (14.8.2.3) and the analysis's order otherwise; the viewer, selection, copy and assistive technology all use the same order.
- **Ligature glyphs** copy as their decomposed letters (ﬁ copies as "fi"); search matches across ligatures through `hg-unicode`'s caseless, canonical-equivalence matching (A-S1-75), with options for case and diacritics.
- **Line-end hyphens** are joined only when the hyphen is a soft hyphen or the font's hyphen glyph at a line end **and** the joined word is in the fonts spec's hyphenation data for the run's language (`hg-hyphen`); "sugar-free" keeps its hyphen.
- **Type 3 text** uses `ToUnicode` when present; otherwise the glyphs are flagged as image text for the OCR hook (6.10).

Find, selection highlights, copy (through the viewer renderer to the broker on a user gesture, since the PDF process holds no clipboard authority, G114), read-aloud and the semantic subtree all read this model; there is no second extractor (G112's principle for text). Selection is drawn by the PDF process as display items in the page's chunks; the text never leaves the process except as the person's selection, the find results the viewer shows, or the semantic subtree.

### 6.10 Accessibility (PD-D11)

The PDF process produces a **semantic subtree** per document through the accessibility spec's cross-process interface (the same way an out-of-process frame does), stitched under the viewer's page elements in the broker's mirror. It never synthesises a DOM.

**Tagged documents.** The structure tree (14.7, 14.8), role maps and namespaces (14.7.4, 14.8.6) and ISO/TS 32005's inclusion rules map to roles by one table. PDF-AAM covers headings, links, sections, articles, asides and divisions ([RT 6.3][RT]); for the rest Izcal defines the mapping by analogy with HTML-AAM and files each row with the W3C task force: P → paragraph; L, LI, Lbl, LBody → list, listitem, its marker and content; Table, TR, TH (with `Scope` and `Headers`), TD → table, row, column or row header, cell; Figure with `Alt` → image with its name; Formula with a MathML associated file (14.13) → math through the MathML mapping; Note and FENote → note; Caption → caption; Form and widget annotations → form controls named by `TU`; Artifact → hidden. `Lang` (14.9.2), `Alt`, `ActualText` and `E` (expansion) are honoured at every level. Content items link to the text model, so caret navigation and text ranges reach assistive technology with exact bounds.

**Wrong tags.** ISO/TS 32005 nesting violations are common: P directly under the root in about 1.6 million documents, Figure in 1.1 million, Table in 0.95 million ([GO 4.3][GO]). The author's tree is exposed **as the author tree**, and Izcal's corrections (re-parenting stray paragraphs, adding missing table structure, demoting empty structure) form a **repair overlay** under HGBA-3 B8, each repair labelled with the rule it applied, so a screen-reader user, an author and DevTools can see both and the difference ([GO 10.5][GO]).

**Untagged documents** get an **inferred structure** from the text model (reading order, headings from relative size and weight, lists from label patterns, tables from rules and alignment), exposed in the repair overlay with an "inferred" label, never as author structure. **Image-only pages** (12% of web PDFs by Google's estimate, [RT 8.4][RT]) need OCR: that is an on-device ML service feature (**ML-OCR**) that runs only with the **accessibility grant** on (ML-D4, ML-D5; Stage 1 process map). **OCR starts on a user action in the viewer or an assistive-technology read, never on document open, so an embedded image-only PDF causes no job** (ML-12). On the PDF process's request the content GPU process of the key gives the canonical page raster to the ML service, and recognised runs return to the PDF process's text model with confidence and reading order, labelled as recognised text. The model is the ML service spec's (ML-OCR, extended here to OCR; G-15 closed by ML-D1).

### 6.11 Navigation features, optional content and attachments

Outlines (12.3.3), destinations including structure destinations (12.3.2), page labels (12.4.2), viewer preferences (12.2), article threads and optional content with its usage and configuration dictionaries (8.11) are supported; optional-content state is per viewing session, changes are shown in the document-facts panel, and printing honours `Print` usage. **Embedded files and associated files** (7.11, 14.13) are listed; saving one goes through the download manager with its file-type and reputation checks (Stage 1 13.2), and Izcal never opens them itself, except that a GoToE action to an embedded PDF opens it, after a click, in a new PDF process of the same key; never automatically. **Multimedia and 3D** (13.2–13.7: sounds, movies, screen and rich-media annotations, U3D, PRC, STEP, glTF) are not played or rendered (PD-D23): the annotation's appearance stream (poster) is drawn, the facts panel says what was skipped, and the content can be saved as an attachment. Sounds and movies are deprecated in PDF 2.0 ([FS 10.1][FS]); U3D and PRC have no web counterpart; glTF rich media (ISO/TS 32007) is the one later candidate, reconsidered when Izcal's WebGL and WebGPU paths can render it in the content GPU process and the corpus shows use ([FS 10.3][FS]).

### 6.12 Annotations, forms and saving (PD-D12)

**Annotations** (12.5) are drawn from their appearance streams; Izcal generates deterministic appearances where none exist for the common types (Text, FreeText, Line, Square, Circle, Polygon, PolyLine, Highlight, Underline, Squiggly, StrikeOut, Ink, Stamp by name, Caret). **AcroForm** (12.7): text fields (single-line, multiline, comb, password, maximum length), check boxes, radio groups, combo and list boxes, push buttons and signature fields. Editing runs in the PDF process through `hg-edit-model`'s text-layout and backing traits, which `hg-pdf-forms` implements, so field editing uses the same edit operations, movement rules and IME protocol as web content (editing spec 6.14, 6.19; IME through the broker, no IME code in the PDF process). **Credential mode** for PDF password fields follows the editing model's `CredentialPolicy`: spelling, autocorrect, IME hints, snapshots and recording redaction treat a field that is or was a password exactly as web content does (editing spec G148). choice popups are drawn by the viewer from typed descriptors. **Appearance generation for variable text** (12.7.4.3) follows `DA`, `Q`, `MK` and the default resources, with an Izcal face when the field's font lacks a glyph; `NeedAppearances` is honoured. Field values are part of the session's recorded state, so a replay reproduces them.

**Rich text** (`/RC`, XHTML in annotations and fields) is parsed by the style spec's `hg-xml-syntax` inside the PDF process (G23) and rendered from ISO 32000-2's fixed element and property allow-list through the PDF process's own appearance generation, never the HTML engine; XMP and XFA packets are parsed the same way.

**Saving is an incremental update** (7.5.6): only changed and new objects are appended after the original bytes, which are never rewritten, so existing signatures still verify and the original revision stays recoverable; serialization is deterministic (G117). Saving an encrypted document encrypts the appended objects with its own handler and keeps its `/Encrypt` dictionary (6.5, G119). The save goes to the download manager through the broker (G118). **Annotation authoring** (highlight, underline, strike-out, ink, free text, note, image stamp with alt text or "decorative") ships at PD-M3, after the incremental writer and form editing are proven correct (PD-8, PD-9), because every authored annotation is saved through them; Firefox and Edge show these are the tools people now expect ([RT 12.1–12.4][RT]).

### 6.13 Actions and links (PD-D15)

No PDF construct causes an effect outside the document unless a person caused it (G118). The default posture starts where Safari and Edge stood in 2020, which blocked every dangerous path but a clicked link ([SE 4.8][SE]), and adds visible consent:

| Action or feature (12.6, 12.7) | Izcal |
|---|---|
| GoTo, GoToE, Named (page navigation), Thread, SetOCGState, Hide, ResetForm | Run inside the document |
| URI (12.6.4.8) | Only on a person's activation of a visible link annotation; `http`, `https` and `mailto` only; the navigation's initiator is an **opaque origin** with no referrer, so the request is cross-site: `SameSite=Strict` cookies are withheld, `Lax` cookies go only with a top-level GET as for any cross-site link, and the PDF cannot make a same-site request ([SE 4.4][SE]) |
| GoToR, remote destinations | As URI, for `http`/`https` targets only; file paths and UNC paths never (NTLM leaks, [SE 4.5][SE]) |
| SubmitForm | Only from a person's click on a push button; a broker consent surface shows the destination origin and the fields to be sent; `http`/`https`; opaque initiator; never from open, print, close or page events ([SE 4.6][SE]) |
| Launch, ImportData, Rendition, Sound, Movie, RichMediaExecute, GoTo3DView | Never; listed in the facts panel ([SE 4.7][SE]: 0.18% of crawled PDFs use Launch) |
| JavaScript actions | Only the form-logic profile (6.17) |
| OpenAction, document and page additional actions (`AA`) | Destinations are honoured; nothing else runs on open, close, save, print or page events |
| External streams, external file specifications, remote fonts or images | Never fetched |

Printing, saving, copying and opening an attachment are requests the broker approves on a user activation it observed. The facts panel lists every action in the document before any runs ([GO 10.2][GO]).

### 6.14 Signatures (PD-D17)

Signature *display* is part of the viewer, because a revision chain is visible structure and the attacks of Section 3 exploit what viewers hide. For each signature field `hg-pdf-sig` parses the CMS SignedData (RFC 5652, PAdES profiles of EN 319 142-1) with the networking spec's certificate machinery, checks that the `ByteRange` covers the whole file except `Contents` and ends at that revision's `%%EOF` (12.8, [FS 5.2][FS]), recomputes the digest, verifies the signature with `hg-crypto`'s verification primitives (RSA, ECDSA, and Ed25519 from ISO/TS 32002's EdDSA, [FS 5.4][FS]; Ed448 is reported "algorithm not supported", never "invalid"), only once the whole file is in the pinned copy (6.2), and evaluates DocMDP and FieldMDP permissions against every later revision. Then it does what no viewer does: it **renders the signed revision and the current one from the same resolved view and diffs their display lists**, so the person sees exactly which regions changed after signing and whether the change is one the certification permits ([GO 6.7, 10.3][GO]). The status reads, for example, "Signed by *name in certificate* (identity not checked). Covers revision 1 of 2. Page 3 changed after signing", never a bare tick; a document that needed repair (6.3) reports that the signed bytes could not be checked strictly.

**Trust evaluation** (chain building to a trust list, revocation, timestamps, long-term validation) is a separate decision, because a "valid" badge carries legal meaning under eIDAS and needs trust data Izcal does not have: (a) **integrity and coverage only, with the signer named as unverified** (recommended for PD-M2); (b) add evaluation against the EU Trusted Lists, which member states publish as machine-readable public data, at PD-M3, if their reuse terms fit the licence rule (Q-PD-6); (c) OS trust stores, which are web-PKI-oriented and differ by OS. Revocation fetches reveal which document is open ([FS 5][FS]), so they happen only when the person asks, through the network service in the document's partition.

### 6.15 The viewer and user experience (PD-D2, PD-D24)

`hg-pdf-viewer` gives continuous and paged scrolling, spreads, zoom (fit width, fit page, actual size), rotation, thumbnails, the outline, page labels, find, presentation mode, keyboard and caret navigation, print, save (with filled fields and annotations), "save attachment", and **open with another application** (a download handed to the OS through the download manager). Its **document-facts panel** shows what the parser and interpreter already know and other viewers discard ([GO 9][GO]): PDF version and producer, repairs applied, fonts embedded and substituted (and by what), features skipped (XFA, multimedia, scripts outside the profile), actions and their targets, signatures and coverage, encryption and permissions, tagged or inferred structure, output intent and colour handling. Each item links to its ISO 32000-2 clause. Dark mode is **role-based** (text, vector fill and images remapped through the colour pipeline, images untouched), never pixel inversion ([GO 8.4, 10.10][GO]); a **reflow view** generated from the (repaired) structure tree, on the device, is a Stage 2 feature with the shell's reader view ([GO 4.10–4.12][GO]). Android viewing arrives with Android in Stage 2 (S-8).

**Zoom.** During a zoom, the previous scale's tiles are shown resampled by the compositor and labelled non-canonical until canonical tiles at the new scale arrive (as GR 6.4 does for in-flight transforms).

**Embedding and downloads (PD-D24).** `application/pdf` responses render inline unless `Content-Disposition: attachment` is set, in which case they download; the person can choose "always download PDFs" in settings, which also makes `navigator.pdfViewerEnabled` false. The MIME type is decided as the MIME Sniffing standard specifies, so a PDF served as `text/plain` is not sniffed into the viewer.

### 6.16 Printing a PDF (PD-D22)

Printing a viewed PDF **never hands the original bytes to the operating system's print path**, because that puts an unsandboxed PDF interpreter (the OS's or the printer's) in front of hostile input. Izcal prints its **own re-emitted PDF**: the page display lists, written by `hg-pdf-print` (6.19) as a normalised, vector PDF in which text is re-emitted as CID fonts **regenerated by Izcal's subsetter from the TX faces Izcal interpreted** (fonts Izcal wrote, never the document's font programs; Type 3 only for Type 3 sources), and images are re-encoded from a canonical decode (JPEGs at quality matching the source's quantisation tables, others losslessly), so no font program or image stream from the document reaches the print system; the OS print system (macOS and CUPS take PDF) receives that file. Where an OS path needs raster (Windows' print path at the printer's resolution), the content GPU process rasters the display lists canonically, in bands of 64 rows†. Text stays vector, unlike a fixed 150 dpi raster ([RT 10.1][RT], Low). Optional content honours its print state; annotations print as their `Print` flag says, and filled fields print with their values ([GO 2.5][GO]).

### 6.17 Form logic and JavaScript in PDF (PD-D13)

PDF JavaScript is the historic exploit path ([HP 5.3][HP]; [SE 2.1–2.3, 8.2][SE]), its API has no IDL so viewers differ wildly (6,742 properties in Acrobat, 114 in PDF-XChange, [SE 8.1][SE]), and what people need from it is form logic: validation and calculation ([HP 10.4][HP]). Firefox's sandboxed subset left out network and disk APIs; Edge runs none ([HP 10.3][HP]; [RT 11.3][RT]).

| Option | Compatibility | Risk | Verdict |
|---|---|---|---|
| (a) No scripts; forms work without calculation | Calculated and formatted fields stay blank or unformatted | None | The floor, and the Hardened profile's setting |
| (b) **Declarative form-logic profile**: Izcal implements the Acrobat form functions natively in Rust, anchored to ISO 21757-1 where it defines them and otherwise to Adobe's published forms documentation (the `AFNumber_*`, `AFPercent_*`, `AFDate_*`, `AFTime_*`, `AFSpecial_*`, `AFRange_Validate` and `AFSimple_Calculate` families, and Acrobat's simplified field notation for arithmetic over field values) and a generated recogniser accepts only scripts that consist of calls to them with literal arguments, or one assignment of an arithmetic expression over field values; the evaluator is a small, fuel-bounded interpreter in the PDF process, like a Type 4 function | Covers formatting, validation and simple calculation; PD-S4 measures the share of real form scripts it covers | No script engine, no API surface, no I/O; deterministic | **Recommended** |
| (c) A sandboxed subset of ISO 21757-1 on Izcal's own jitless engine, in a **separate per-document form-script process** at the strictest tier (not the site's renderer, which a PDF author must not reach, and not the PDF process, which links no engine, G115), with only `Field`, `event`, `util` formatting and the AF functions, a fixed property set, no network, file or timers, data copied in and out | Most form scripts | A JS engine reachable from document data; a fixed API still fingerprints as "Izcal" only | The owner's option if PD-S4 shows (b) covers less than 90%† of the corpus's form scripts |
| (d) An Acrobat-compatible API | Highest | The 2008–2010 record | Rejected |

**Evidence and checks.** Each profile function's semantics come from ISO 21757-1 where it specifies them (Q-PD-14), Adobe's published API prose otherwise, and black-box measurement in Acrobat recorded in `compat/observations/`; Acrobat's bundled scripts and other viewers' implementations are never read (M-1 part 3). The recogniser is generated from an ECMAScript grammar subset and, under the style spec's G22, differentially tested in CI against Izcal's own JavaScript parser: every accepted script must parse there to the same call or expression tree. PD-13 checks the values it computes.

Under (b) and (c) scripts run only on field events a person caused (keystroke, format, validate, calculate), never on open, page, print, save or close events; anything outside the profile is not run and the facts panel says so ("3 scripts not run: they would have …" where the recogniser can tell). The environment the profile exposes (viewer name, version, platform) is fixed per build, so it reveals nothing about the device (6.23).

### 6.18 XFA, multimedia and 3D (PD-D14, PD-D23)

XFA is a second document model inside PDF: an XML form language with its own layout engine, scripting (FormCalc and JavaScript) and network submission, never an ISO standard, deprecated in PDF 2.0, and still partial in the one browser that implemented it ([HP 9.1–9.5][HP]; [SE 8.4][SE]).

| Option | Verdict |
|---|---|
| (a) Ignore XFA silently | Rejected: users get Adobe's "Please wait..." page with no explanation ([GO 2.7][GO]) |
| (b) **Render the AcroForm that static XFA documents carry** (ISO 32000-2's behaviour, ignoring XFA), and for dynamic-XFA-only documents show a plain explanation ("This form uses XFA, an Adobe format that PDF 2.0 retired; Izcal does not run it") with "save" and "open with another application" | **Recommended** |
| (c) A static XFA renderer (XFA layout to the display list, no scripts) | Considered when PD-S1 and opt-in reports show dynamic XFA in more than 1%† of the PDFs people open with forms, or a public form estate Izcal's users depend on requires it |
| (d) Full dynamic XFA with FormCalc and scripts | Rejected: a second document engine and script runtime in the most exposed process |

Multimedia and 3D follow 6.11.

### 6.19 The PDF writer (PD-D16, PD-D18, PD-D19, PD-D20, PD-D21)

The writer serves print-to-PDF, `window.print()` to a PDF destination, saving a viewed PDF's changes (incremental, 6.12) and printing a viewed PDF (6.16). Its core (`hg-pdf-write`: object serialisation, cross-reference streams, object streams, deterministic numbering) is needed in Stage 1 by the viewer; the print-to-PDF product ships with Stage 2's print UI (Stage 1 13.2, LY-D8).

**Inputs** (reconciliation 2 Q24): for each page, the **display list** `hg-paint` builds from layout's paged fragment tree (appearance: paint order, clips, effects, [GR 6.27][GR]); the **structure** from the semantic plane and fragments (roles, names, language, reading order, link areas and targets, headings); the **print environment** (sheet size, margins, scale, headers and footers), recorded once (LY 6.7); the **print time**, a recorded input. Two amendments make this possible (9.5): print-mode display items carry the semantic node they paint (to become marked-content references), and text items carry their shaping cluster maps (to become `ToUnicode` and `ActualText`).

**Placement (PD-D16).** The writer embeds opaque cross-origin images, so it is never a renderer (G56), and a page with cross-site frames contains several keys' content, so no one key's process may hold all of it (I-1):

| Option | Verdict |
|---|---|
| (a) The writer in the top document's renderer | Rejected: G56 |
| (b) One writer process of the top document's key that receives every frame's display list | Rejected: I-1 (site B's frame content in site A's process) |
| (c) A writer job per key, plus a keyless assembler that receives every key's serialised bytes and joins them | Works, but needs a new I-1 shared-process exception and one buffer as large as the print |
| (d) **A writer job per key** (the PDF process binary in its writer role) writes that key's content streams, resources, structure elements, annotations and outline items as **complete objects** in an object-number range the broker grants, into **its own segment of a broker-granted output file**; the **broker concatenates the segments** in page order without parsing them (as the download path already writes broker-granted files); a keyless **joiner** (a fresh utility job per print, strictest tier, linking only `hg-pdf-write`) receives only typed descriptors (object numbers, lengths, root references, page order, outline titles' object references) and writes the page tree, structure tree root, `ParentTree`, outline links, metadata and the cross-reference stream | **Recommended.** No process holds two keys' content, so **no I-1 exception** is needed; output streams to a file, never one buffer |
| (e) The content GPU process of the key, which already holds the key's display items and image regions ([GR 6.27][GR]) | Rejected: it would link `hg-fontfmt` and the font subsetter, which G26 keeps out of GPU processes |

The writer job gets the **encoded bytes** of the key's JPEGs, for pass-through, from the key's media process on a print-job grant (a media amendment, 9.5); it parses only Izcal display lists, structure records, and fonts through `hg-fontfmt` (G110).

**Output profile (PD-D18).** Chrome's output is tagged but unidentified PDF 1.4; WebKit's is untagged; document tools tag by default and offer PDF/UA and PDF/A ([WP 10][WP]).

| Option | Verdict |
|---|---|
| (a) PDF 1.7 with tags, no identification (Chrome's level) | Rejected: below what document tools do |
| (b) **PDF 2.0, structure per Well-Tagged PDF in the PDF 2.0 namespace (Annex L), identified as PDF/A-4, with an accessibility report; no automatic PDF/UA-2 claim** | **Recommended**: PDF/A-4 adds only XMP and colour identification once fonts, `ToUnicode` and determinism are right ([WP 4.5][WP]); PDF/UA needs human judgement (alt text quality, reading-order intent) that no machine check can supply, so Izcal reports and never claims |
| (c) As (b), and claim PDF/UA-2 whenever every machine-checkable requirement passes | The owner's alternative; a claim resting on unverified judgement would be a false conformance claim |
| (d) PDF/A-2u or PDF/A-3 on PDF 1.7 | The fallback if PD-S5 finds widely used readers that fail on PDF 2.0 output |

What the writer emits: tags from the semantic plane (headings, paragraphs, lists, tables with header scope, figures with `Alt` or as artifacts when decorative, links with `Link` structure and object references, `Lang`, MathML as an associated file for `Formula`), page-margin boxes and browser headers and footers as pagination artifacts, link annotations for every link fragment (external and in-document), an **outline from headings by default**, `/ID`, XMP, document title from `document.title`, object streams and cross-reference streams (Chrome and WebKit use neither, [WP 7.4][WP]).

**Colour.** Every colour space the page uses is written as an `ICCBased` space whose profile `hg-colour` generates from published parameters (sRGB per IEC 61966-2-1, Display P3, Rec. 2020), deduplicated by hash, as MD-D8 permits; the output intent is the generated sRGB profile. Wide-gamut content is never clipped to sRGB.

**Fonts (PD-D19).** The writer embeds a subset of every font it uses through `hg-font-subset`, a fonts-spec crate this spec proposes (generated from the same table descriptions as `hg-fontfmt`; amendment, 9.5): TrueType outlines as CIDFontType2 (`FontFile2`, Identity-H), CFF as CIDFontType0C, **variable fonts instanced** at each used location (glyph deltas applied; CFF2 converted to CFF), because PDF 2.0 has no variable-font form ([WP 5.5][WP]); **COLRv0 and COLRv1 glyphs as vector Type 3 glyphs** (paths and PDF shadings), sbix and CBDT glyphs as image Type 3 glyphs, OT-SVG glyphs as vector Type 3 glyphs drawn from Izcal's SVG painting, all with `ToUnicode` so emoji text survives, unlike both measured browsers ([WP 1.10, 2.4, 5.7][WP]). **Subset tags** are six letters derived from a hash of (font, glyph set), so identical subsets dedupe and output is deterministic ([WP 5.2][WP]). **Embedding permissions**: OS/2 `fsType` is read with the version-dependent precedence rule ([WP 5.3–5.4][WP]); "restricted license" and "bitmap embedding only" without bitmaps are not embedded, and their text is drawn as outlines inside marked content with `ActualText`, so it stays extractable and the file stays PDF/A-4 conformant; "no subsetting" embeds the whole font. Izcal's own fonts are MIT and always embedded. Whether drawing outlines of a restricted font is permitted is for the legal review (Q-PD-5).

**Images and effects (PD-D20).** JPEG images drawn unmodified are **passed through** as their original bytes (DCTDecode), with the colour profile and orientation handled, as Chrome does and WebKit does not ([WP 1.6, 2.3, 7.6][WP]); lossless sources (PNG, lossless WebP, GIF frames) are written losslessly with Flate and PNG predictors from `hg-compress`'s encoder; other lossy sources (lossy WebP, AVIF, JPEG XL) are decoded and written losslessly at their intrinsic resolution, capped at 300 dpi† at their printed size, with no second lossy generation. Blend modes, opacity groups, clip paths, CSS masks and gradients stay vector; **effects without a vector form** (blur, shadows, SVG filters, `backdrop-filter`, 3D transforms) are rastered by `hg-raster` canonically at 300 dpi† of their printed bounds, as images with soft masks, and each raster fallback is listed in the writer report with its reason. Canvas output uses its canonical raster; WebGL and WebGPU canvases their current frame, labelled non-canonical; video its current frame.

**Determinism (PD-D21).** The output bytes are a pure function of (display lists, structure records, print environment, print time, build) (G117): objects numbered in document order; dictionaries serialised in a fixed key order; a pinned Flate encoder level; hashed subset tags; `/ID` from SHA-256 of the body; dates from the recorded print time in UTC; no user name, host name, path, locale or time zone ([SE 13.5][SE]). For tests and automation the print time is fixed by SOURCE_DATE_EPOCH or an explicit option, as pdfTeX and Typst do ([WP 6.1–6.3][WP]). Byte identity across macOS, Linux and Windows on x86-64, arm64 and riscv64 is a release test (PD-36).

### 6.20 Determinism (B1)

Rendering, text extraction, structure inference, form logic and saving are pure functions of (bytes, recorded inputs, build): no clock, no entropy, no thread-count or device dependence (G113's canonical limits; G116's numbers; GR-D3's canonical raster). Range arrivals, user input (field edits, clicks), the viewer's scroll and zoom, and the print time are recorded inputs, so a recorded session replays exactly, including a form-filling bug report ([GO 10.7][GO]). Background parsing order may vary because results are keyed by object, not by arrival; the canonical-output test runs the corpus under randomised scheduling to prove it, as G70 does for raster placement.

### 6.21 Footprint

Page rasters, not documents, dominate PDF memory ([GO 3.1][GO]; [RT 3.1][RT]), and in Izcal they live in the content GPU process's tiles, under FP-14 and the graphics spec's content GPU budgets ([GR 6.22][GR]), so the PDF process's own budget is small:

| Item | Budget† | Target |
|---|---|---|
| PDF process, empty (process part, before a document) | ≤ 3 MiB charged | PD-F1 |
| Resolved view | ≤ 48 B per object plus the cross-reference data | PD-F2 |
| Decoded data caches (images at needed resolution, fonts, Type 3 sub-lists, decoded streams) | ≤ 64 MiB per document, least-recently-used, discardable and regenerable (FP 6.4 class (a)) | PD-F2 |
| Live decoded data and the pinned byte copy (6.2, 6.3) | Inside the heap limit; decoded data ≤ 512 MiB | PD-F2 |
| PDF process canonical heap limit | 1 GiB, identical on every device (FP-D19); reaching it ends the document with an explanation, never the browser | PD-F2 |
| Live PDF processes per renderer key | ≤ 8 (6.1) | PD-F3 |
| Display-list residency in the content GPU process | Only the visible pages ± 2† keep their chunks; others are evicted and regenerated deterministically on demand; a page whose display list would exceed 32 MiB† is split into spatial bins so only visible bins are resident | PD-F3 |
| Process-colour group working set | Allocated per tile at the channel count used; banded above 64 MiB† per page (6.6) | PD-F3 |
| Hidden document | Caches dropped after 30 s†; the process exits after 10 min† hidden and the document is re-parsed on return, deterministically, **unless it has unsaved changes or a decryption key**, in which case only caches are dropped | PD-F3 |
| Viewer application in the renderer | ≤ 8 MiB† above an empty site process | PD-F4 |
| Installed code of `hg-pdf-*` (excluding the text and media crates it shares) | ≤ 6 MiB† per architecture | PD-F5 |
| PDF data in the engine image (encodings, CMaps, CID and glyph-name tables) | ≤ 4 MiB† on disk, mapped clean | PD-F5 |

**Crashes.** A crash of a document's process with unsaved changes restores field values and annotations from a journal the viewer keeps of the typed edit events (no PDF bytes), then reports anything not journaled; for an encrypted document the person is asked for the password again. The writer job and the joiner run only during a print and exit after it (FP's utility-job rule), and the writer streams to its file segment, never holding the print in memory.

### 6.22 Security

**Threat model.** The attacker controls every byte of a PDF and every byte served for its ranges; can make the person open it (a link, an attachment, an iframe on any site); and wants code execution, data from other documents, sites or the system, network effects (phone-home, CSRF, credential leaks), denial of service, or a signed document that says something other than what was signed.

| Process | Holds | Attacker reach after full compromise |
|---|---|---|
| PDF process (per document) | That document's bytes and state | That document, which they wrote; its display lists and semantic subtree reach consumers only through generated readers with semantic limits (G62) |
| Content GPU process of the key | That key's pixels | As the graphics spec states; the new PDF items have their own validation limits (6.6) |
| Renderer of the key (viewer) | The viewer UI's view of that document | Nothing from the PDF process beyond typed messages |
| Form-script process (only under PD-D13 (c)) | Field values copied in | Those values |
| Writer job (per key, during a print) | That key's page content for one print | That key's part of that print |
| Print joiner (keyless, fresh per print) | Typed descriptors: object numbers, lengths, roots, page order | The structure of one print, no content |

**Layers.** Memory safety by construction (no `unsafe` in any `hg-pdf-*` crate, G5 with no inventory entry; generated parsers with Kani harnesses, G111; no recursion and fuel everywhere, G113; checked arithmetic, G18); a sandbox with no authority (G114) at the strictest tier; no code generated from document data and no script engine in the PDF process (G115); one resolved view for every consumer, closing parser differentials (G112); every external effect through broker consent (G118); legacy crypto confined (G119). **Fuzzing in three layers** from PD-M0 ([SE 11][SE]): structure-aware fuzzing of each generated parser, filter, font type and decoder; generation of adversarial but valid documents guided by Arlington (test-only); and differential runs against black-box viewers to find semantic divergences. Every target runs in CI and on the fuzzing farm (Stage 1 7.5) on x86-64, arm64 and riscv64 builds, with published coverage and crash-free hours (PD-22). **Emergency switches**: new switches, added to UPD-R5's closed list by amendment (9.5), let the update system's emergency channel turn off form logic, any single PDF decoder (JBIG2 and JPX first), signature parsing, or PDF viewing as a whole (PDFs then download and `navigator.pdfViewerEnabled` becomes false) until a fix ships. The **Hardened profile** (S-4a) runs form logic off and opens PDFs with external navigation confirmed per click.

### 6.23 Privacy

Opening, scrolling, printing or closing a PDF makes **no network request** (PD-30) beyond the range reads of the document itself ([SE 13.1][SE]). Navigation from a PDF starts from an opaque origin without referrer (6.13). Rendering is canonical, so the "content masking" fingerprint of viewers ([SE 6.1][SE]) reveals the Izcal build and nothing else; the form-logic profile exposes a fixed environment (6.17). The writer emits no user, host, path, time-zone or locale data (6.19). Agents see a PDF only through the semantic plane under B9's consent and origin scoping, and never its bytes.

### 6.24 Observability and integrations (PD-D26)

An **IIP `pdf.*` domain** (DevTools spec), developer-tool only and never page-exposed: the revision list and cross-reference sections; the object graph with each object's ISO 32000-2 clause; repairs applied (rule numbers); filters and decoders per stream with sizes and limits hit; fonts with their embedding, encoding chain and substitution; colour spaces, output intent and process-colour groups; a per-page operator trace with clause anchors and time per operator class; the text model with each character's mapping source; the author structure tree, the repair overlay and the inferred structure with rule IDs; form fields, scripts, which ran under the profile and which did not; the action ledger; signature coverage and revision diffs; decrypted strings and field values are marked sensitive and shown only through the inspector's consent flow, never to agents without B9 consent; and the **writer report** (fonts embedded, subset and instanced; tags emitted per element; raster fallbacks with reasons; page breaks with layout's explainers; the output's machine-check results against PDF/A-4 and Well-Tagged PDF). All of it is derived from state the process already holds. The **document-facts panel** (6.15) is the user-facing slice. **Recorded sessions** include the PDF process's inputs, so a PDF bug report replays exactly. **Agents** (AI on, B9) read the semantic subtree and text model through the tool registry with origin-scoped consent and audit; they can request a download, never raw bytes from the PDF process.

### 6.25 Crate layout and rules

A new sub-layer **L2d (document formats)**, a sibling of L2e, L2p, L2t, L2m and L2g: pure, sans-IO crates over plain data that depend only on L1, L2 and L2s. Crates that join PDF with fonts, media decoders or graphics sit in L4, because L2 sub-layers never depend on one another and L3 and above join them ([RN P1][RN]; [RN2 Q18][RN2]). The **PDF profile** is tier P's outcome (no panic on any input) **and** its checked-arithmetic lints (G18) with no saturating exceptions, plus budget handles and fuel (G53's discipline); it applies to every `hg-pdf-*` crate, L4 included, because all of them run on hostile data. XML found in PDFs (XMP, `/RC` rich text, XFA packets, MathML associated files) is parsed by the style spec's `hg-xml-syntax` (L2p, G23), linked by the L4 crates inside the PDF process only. `hg-pdf-sig` needs certificate and CMS parsing, which today lives in the networking spec's `hg-pki` (L5); the DER and X.509 parsers move to an L2s crate both use (amendment, 9.5).

| Layer | Crate | Role | Tier |
|---|---|---|---|
| L2d | `hg-pdf-syntax` (generated) | Lexer, object parser, stream extents, typed dictionary accessors (G111) | PDF profile |
| L2d | `hg-pdf-xref` | Cross-reference tables and streams, object streams, revisions, repair grammar, the resolved view (G112) | PDF profile |
| L2d | `hg-pdf-filter` | ASCII filters, RunLength, predictors, Flate through `hg-compress`; a decoder trait the host fills with L2m decoders | PDF profile |
| L2d | `hg-pdf-crypt` | Standard and public-key security handlers, crypt filters, through `hg-crypto` (G119) | PDF profile |
| L2d | `hg-pdf-content` (generated) | Content-stream operator decoder, CMap parser, Type 4 function parser and evaluator | PDF profile |
| L2d | `hg-pdf-write` | Deterministic serialiser, cross-reference and object streams, incremental updates (G117) | P (linked by the keyless joiner) |
| L4 | `hg-pdf-interp` | Graphics state, colour, transparency, shadings, lowering to `.hgidl` | PDF profile |
| L4 | `hg-pdf-font` | Font dictionaries to TX faces; encodings; Type 3 execution; substitution | PDF profile |
| L4 | `hg-pdf-text` | Text model, layout analysis, find, selection | PDF profile |
| L4 | `hg-pdf-struct` | Structure tree mapping, repair overlay, inferred structure, semantic subtree | PDF profile |
| L4 | `hg-pdf-forms` | Annotations, AcroForm, appearances, actions, field editing, saving | PDF profile |
| L4 | `hg-pdf-logic` | Form-logic profile recogniser and evaluator (PD-D13 (b)) | PDF profile |
| L4 | `hg-pdf-sig` | CMS parsing, digests, verification, MDP evaluation, revision diff | PDF profile |
| L4 | `hg-pdf-print` | Display list and structure records to complete objects in a granted range; fonts through `hg-font-subset`; images and encoded JPEGs from the media process | PDF profile |
| L4 | `hg-pdf-host` | PDF process main library: jobs, budgets, pinned range reads, decoder wiring, viewer and broker endpoints | PDF profile |
| L5 | `hg-pdf-viewer` | The viewer application (HTML, CSS, script resources) and its typed endpoints | R (runs as web content) |
| L5 | `hg-print-join` | Print joiner job (typed descriptors only) | P, strictest tier |
| L6 | `hg-pdf-main`, `hg-print-join-main` | Process mains | — |
| tools | `hg-pdfgen` | `.hgpdf` → parsers, accessors, Kani harnesses, fuzz grammars, anchors | T (output held to the PDF profile) |
| tools | `hg-pdf-data-build` | Encodings, CMaps, CID-to-Unicode and glyph-name tables (PD-D8) → engine image | T |
| tools | `hg-pdf-ref` | Independent float64 reference renderer: shares the syntax layer and `hg-fontfmt` (a common-mode risk covered by PD-1 and the fonts spec's own oracles, Section 10) and nothing from interpretation, colour or raster onward | T |
| tools | `hg-pdf-check` | Machine checks of writer output against PDF/A-4 and Well-Tagged PDF, written from the standards | T |
| tools | `hg-pdf-fuzz`, `hg-pdf-corpus`, `hg-pdf-diff` | Fuzzers, corpus census (PD-S1), black-box differential harness | T |

**Rules G110–G119.**

| Rule | Statement | Why |
|---|---|---|
| **G110** | No PDF byte (file, stream, embedded font or image, CMap, XMP or other XML, FDF) is parsed outside the PDF process; no binary of the broker, renderer, GPU processes, network, storage or media processes has an `hg-pdf-*` parser crate in its loaded-library closure; the writer job parses only Izcal display lists, structure records and fonts through `hg-fontfmt`; the joiner parses nothing but typed descriptors | PD-D1; as G26 and G50 |
| **G111** | The PDF lexer, object parser, cross-reference and object-stream readers, typed dictionary accessors, content-operator decoder, CMap and Type 4 parsers are generated by `hg-pdfgen` from checked-in `.hgpdf` descriptions with clause anchors; hand-written byte reads fail CI | One source of truth (G29, G51) |
| **G112** | Every consumer (rendering, text, structure, forms, signatures, saving, the IIP) reads objects only through `hg-pdf-xref`'s resolved view or a revision view; there is no second parser or quick scanner | Parser differentials (shadow and wrapping attacks); G23's one parser per grammar |
| **G113** | No recursion over input structure; explicit stacks with canonical depth limits; fuel in every input-driven loop; visited sets on every graph walk; every limit identical on every device | G21, G53, FP-D19; cycles and bombs ([SE 4.2–4.3][SE]) |
| **G114** | The PDF process holds only: its document's sealed bytes or a pinned one-URL range grant, revoked as 6.2 states; its own display-list channel and namespace in its key's content GPU process; its viewer channel; a read-only font endpoint answering substitution lookups only; and the broker's request endpoint; no network, file, clipboard, device or storage handle | I-2 |
| **G115** | No `hg-pdf-*` crate and no PDF process binary links an L2e crate or generates executable code from document data; PDF scripts run only under PD-D13 | CVE-2018-5158, CVE-2024-4367 |
| **G116** | No `f32` or `f64` on any path whose result reaches a display list, text model, semantic subtree or saved bytes, except number conversion at the boundary, PDF function evaluation through `hg-math` under JS 6.9's discipline, quantised once, and colour-space conversion, which happens only in `hg-colour` under MD-D28 | Cross-OS and cross-ISA identity (G28, G61) |
| **G117** | `hg-pdf-write`, `hg-pdf-print`, `hg-pdf-forms`' save path and the joiner read no clock, entropy, environment, hash-map iteration order or thread count; output bytes are a pure function of declared inputs | B1; PD-36 |
| **G118** | Every effect outside the document (navigation, submission, save, print, copy, attachment export, revocation fetch, key-store use) is a typed request to the broker, which requires a user activation it observed and, for submission, external `mailto` and attachment export, a consent surface; no action runs on open, close, print or page events except in-document navigation | [SE 4][SE] |
| **G119** | RC4, MD5 and unauthenticated AES-CBC are reachable only from `hg-pdf-crypt` through `hg-crypto-legacy`, which no binary but the PDF process links; they decrypt, and encrypt only to append an incremental update to a document already encrypted with that handler; new encryption is only R6 on an explicit request, never RC4 or MD5 | G36; PDF 2.0 deprecations |

### 6.26 Interfaces to other specs

| Interface | Counterpart | Contract |
|---|---|---|
| Display list extensions | Graphics (GR-D6) | Effect-node flags (knockout, non-isolated, `AIS`, `TK`), blending colour space, process-colour encoding with shape and opacity channels and spot channels, overprint masks, soft masks with `BC` and `TR`, colour-mesh and tiled sub-list items, each with validation limits, a canonical raster rule, GPU and SIMD paths; several producers per key with per-producer namespaces and budgets; owned by the graphics spec, written by this spec's imaging track |
| Print display lists | Graphics, layout | Print-mode display items carry their semantic node and text items their cluster maps; the writer consumes them (Q24) |
| Fonts | Fonts and text (TX-D7) | Type 1, CFF, TrueType and OpenType through `hg-fontfmt` and `hg-font-vm`; Type 3 is PDF content; Symbol and ZapfDingbats aliases; `hg-font-subset` with instancing and colour-glyph export |
| Decoders and colour | Images and media (G50–G53, MD-D8, MD-D28) | `hg-jpeg`, `hg-jpx`, `hg-jbig2`, `hg-ccitt`, `hg-entropy` (LZW with `EarlyChange`), `hg-icc` in the PDF process; decode class P; reduced-resolution decode; encoded JPEGs to the key's writer job; `hg-colour` tables for CMYK, Lab, DeviceN tint transforms and the default CMYK characterisation; generated ICC profiles for writer output |
| Semantic subtree | Accessibility (AX) | The cross-process subtree interface used by out-of-process frames; the PDF mapping table; the inferred-structure label in the repair overlay |
| Range reads | Networking | A pinned one-URL range-read grant (strong validator, `If-Range`, block coalescing, request cap, revocation) served by the partition service in the navigation's partition |
| Crypto and certificates | Networking (G36) | `hg-crypto-legacy` (MD5, RC4, AES-CBC); SHA-512; DER and X.509 parsing in L2s for `hg-pdf-sig` |
| Form-field editing | `hg-edit-model` (editing spec 6.14, 6.19) | `hg-pdf-forms` implements its text-layout and backing traits, so field editing uses the same edit operations, movement rules and IME protocol; credential mode for PDF password fields |
| OCR | ML service (ML-OCR; G-15 closed) | Page raster in, recognised text runs out with confidence and reading order; only with the accessibility grant on, on a user action or an AT read, never on document open (ML-12) |
| IIP | DevTools | The `pdf.*` domain (6.24) |

## 7. M-1 compliance

**Dependencies used unmodified (shipped).** None that is PDF-specific. Generic crates, all behind the networking spec's crypto crates (G36, G119): RustCrypto `md-5` 0.11.0, `rc4` 0.2.0 and `cbc` 0.2.1, each **MIT OR Apache-2.0** (crates.io, 19 October 2026, [GO 11.10–11.13][GO]), in the new `hg-crypto-legacy`. Each was resolved alone with `cargo tree` on 19 October 2026: none links `subtle`; `md-5` uses `digest` 0.11 (the family of NW's `sha2` 0.11 pin) and `rc4` and `cbc` use `cipher` 0.5, with `block-buffer`, `crypto-common`, `hybrid-array`, `inout`, `block-padding`, `const-oid`, `typenum` and `cfg-if` beneath; their licences are checked by `tools/deny.toml` like every RustCrypto pin. `sha2` (adding SHA-512) and `aes` are already pinned by the networking spec. Every other building block is an accepted Izcal crate: `hg-compress` (inflate and deflate), `hg-entropy` (LZW), `hg-jpeg`, `hg-jpx`, `hg-jbig2`, `hg-ccitt`, `hg-icc`, `hg-colour`, `hg-math`, `hg-unicode`, `hg-hyphen`, `hg-fontfmt`, `hg-font-vm`, `hg-glyph-raster`, `hg-display-list`, `hg-raster`, `hg-path`.

**Excluded, by M-1 part 1 or the licence rule** ([GO 11][GO]; [WP 9][WP]): pdf.js, PDFium and `pdfium-render`, Poppler, xpdf, MuPDF, qpdf, `lopdf`, `pdf` (pdf-rs), `hayro` and its codec crates, `fax`, `printpdf`, `pdf-writer`, `krilla`, `subsetter`, `allsorts` (Apache-2.0 only), `ttf-parser` and Fontations (G25), `zlib-rs` (Zlib), `jbig2dec` (GPL-3.0), OpenJPEG bindings, DaeDaLus (BSD-3-Clause), PolyFile (Apache-2.0) ([SE 6.5][SE]). The licence of a PDF crate does not matter: a PDF engine or writer is engine work.

**Built new.** Everything in 6.25: the `.hgpdf` descriptions and `hg-pdfgen`, the parser and repair grammar, filters, security handlers, interpreter, display-list and `hg-raster` extensions, PDF font glue and Type 3 execution, the PD-D8 build of Adobe's registry tables into the engine image (option (a), decided), text model and layout analysis, structure mapping, forms and appearances, the form-logic profile, signature checking, the viewer application, the writer, subsetter and instancer (with the fonts spec), the print joiner, the reference renderer, the output checker, fuzzers and census tools.

**Anchored to** (pinned revisions recorded in `hg-anchor`): ISO 32000-2:2020 with Errata Collection 3 (every clause cited in Section 6; Medium-confidence clause numbers checked against the no-cost copy before PD-M0 ends, [FS 0][FS]); ISO 32000-1:2008 for legacy behaviour; ISO/TS 32001, 32002, 32003, 32004, 32005; ISO 21757-1:2020 (the AF function and field semantics of the form-logic profile); ISO 19005-4:2020 (PDF/A-4); ISO 14289-2:2024 and WTPDF 1.0; ETSI EN 319 142-1 V1.2.1 and RFC 5652; RFC 1950/1951; ITU-T T.4, T.6, T.81, T.88, T.800 (through the media spec); ICC.1:2022; OpenType 1.9.1 (`OS/2.fsType`); HTML (PDF viewing support, printing), MIME Sniffing, Fetch (range requests); PDF-AAM (Editor's Draft, 12 October 2026) and WCAG 2.2's PDF techniques; CSS Paged Media 3 and Fragmentation 3 through the layout spec.

**Standards data ([OD-L6][LA]).** The single-byte encodings and character sets of ISO 32000-2 Annex D; WHATWG encoding indexes and Unicode data. Adobe's CMap resources, CID-to-Unicode tables and Glyph List (BSD-3-Clause) ship as standards registry data: the owner decided PD-D8 (a) on 20 October 2026 (HGBA-2 2.3.3; the licence audit's class 4), with the BSD-3 notice. Characterisation data for the default CMYK conversion is shipped only if the owner rules it standards data (PD-D6, Q-PD-4). No ICC-authored profile ships (MD-D8).

**Test-only inputs ([OD-L2][LA]), licences recorded before use; none is in a shipped artifact or a build-time input that emits shipped code:** the Arlington PDF Model (Apache-2.0, [FS 12.2][FS]; test-only, which answers Q-PD-3); veraPDF corpus (CC BY 4.0) and veraPDF itself as a black-box validator (MPL-2.0+/GPL-3.0+) ([FS 13.4][FS]); CC-MAIN-2021-31-PDF-UNTRUNCATED and UNSAFE-DOCS (Common Crawl and Digital Corpora terms, unsanitised: handled as hostile input on isolated machines) ([FS 13.1][FS]; [GO 1.4][GO]); GovDocs1; the Ghent Output Suite 5.0 (terms to check); the PDF Association's PDF 2.0 sample files; pdf-insecurity.org's test files and the NDSS 2021 dangerous-paths set; a Bast-Korzen-style extraction benchmark (Q-PD-9); Adobe's CMaps and Glyph List as validation oracles if PD-D8 (a) is refused; Adobe's AFM metric files as a test oracle for TX-8's alias tables; black-box viewers (Chrome, Firefox, Safari, Edge, Acrobat) and command-line tools such as Poppler's `pdfinfo`, `pdffonts` and `pdftotext` run only as programs.

**M-1 part 3, evidence.** No link in this spec or its research notes points to any third-party project's source code (HGBA-2 2.3.3: engines, PDF libraries, tools, OS components alike); every note records the source-code results it did not open ([HP][HP], [FS][FS], [SE][SE], [RT][RT], [WP][WP], [GO][GO] M-1 notes). Values that only engine source would state (other viewers' recursion limits, decompression caps, rasterisation resolutions for effects, print paths) are black-box measurement questions ([SE 7.4][SE]; [WP 11][WP]; [RT 13][RT]; [GO 12][GO]); every limit in this spec is marked † and set from first principles. **Where the design resembles existing systems** (a separate sandboxed process, range loading, a narrow form-script API, tagged output from the accessibility tree, JPEG pass-through), each choice is derived here from I-1, I-2, B1, P4 and the measured attacks of Section 3, and the resemblance is cited as evidence that it works, never as a template. The writer's black-box tests inspected Chrome's and WebKit's **output** files; Skia's and Quartz's PDF backends were not read. The form-logic profile's semantics come from ISO 21757-1, Adobe's published prose and black-box measurement, never from Acrobat's bundled scripts or another viewer's implementation (6.17).

## 8. Targets

† marks thresholds set by this document from first principles; none is measured. "Best shipping viewer" means the best of stable Chrome, Firefox, Safari (macOS) and Edge (Windows) on the same reference device, corpus and day, run as black boxes. The **PDF corpus** is fixed at PD-M0 from PD-S1: a stratified sample of the untruncated Common Crawl corpus (by page count, producer, version, features), GovDocs1, the veraPDF corpus, the PDF 2.0 samples, the Ghent Output Suite, the signature and dangerous-paths suites, and Izcal's own generated cases.

### 8.1 Correctness

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **PD-1** | Parser conformance | 100% of Arlington-generated valid cases accepted with the right object values; 100% of invalid cases rejected or repaired by a named rule; 0 panics | T1 suite from `hg-pdfgen` + Arlington (test-only) | Ship blocker for the affected clause |
| **PD-2** | Rendering accuracy | Against `hg-pdf-ref` at 150 dpi, after a 1-pixel edge tolerance (a pixel within 1 px of a reference edge is compared with its 3 × 3 neighbourhood's range), ≥ 99.5%† of corpus pages have ≥ 99.9%† of pixels within ΔE00 ≤ 2†; 100% of the PDF 2.0 samples | Lab, every release | Triage per clause; regressions block |
| **PD-3** | Colour and transparency | Every Ghent Output Suite 5.0 patch (48 by trade-press accounts, counted from the suite at PD-M0), including knockout with soft masks, passes by the suite's own marks with overprint simulation on; if the suite's terms bar use (Q-PD-13), Izcal-built patches covering the same cases | Lab | Ship blocker for PD-D5 features |
| **PD-4** | Canonical pages | Page rasters bit-identical across macOS, Linux, Windows; x86-64, arm64, riscv64; CPU and GPU raster; 1 to N threads; excluding only pages labelled non-canonical because they use the interim OS font tier (6.8) | Cross-platform hash test (DET-1 as extended by GR-51) | Ship blocker |
| **PD-5** | Differential triage | Every corpus page where Izcal disagrees with the majority of four black-box viewers is triaged within one release with the governing clause cited; disagreements without a clause-backed explanation ≤ 0.5%† of pages | `hg-pdf-diff` | Staffing review of the imaging track |
| **PD-6** | Text extraction | Word error rate on the extraction benchmark ≤ the best of the four browsers' text (copy-all) and of `pdftotext` run as a black box; ligatures, diacritics and line-end hyphens 0 errors on the synthetic suite | Lab | Ship blocker for read-aloud claims |
| **PD-7** | Substitution keeps layout | Every substituted glyph's advance equals the PDF's `/Widths` or `/W` value (exact) | Unit and corpus | Ship blocker |
| **PD-8** | Forms round trip | Filled values and annotations survive save and reopen in Izcal and in three black-box viewers for 100% of the form corpus, including encrypted strata (R2–R4 RC4, R4 AESV2, R6); appearances regenerate identically | Scripted round trip | Ship blocker for save |
| **PD-9** | Incremental save | The original bytes are an exact prefix of every saved file; every signature valid before a save is valid after it | Corpus | Ship blocker |
| **PD-10** | Encryption | 100% of the corpus's R2–R6, AES-GCM and public-key files open with the right password or key | Corpus | Ship blocker for the handler |
| **PD-11** | Damaged files | Files Izcal opens ≥ files the best shipping viewer opens, on the corpus's damaged stratum | Lab | Repair grammar extended by a named rule |
| **PD-12** | HTML integration | WPT tests for PDF viewing support, `<embed>`/`<object>` of PDFs and printing events pass at Izcal's per-directory floors (C-1) | `hg-wpt` | As C-1 |
| **PD-13** | Form logic computes what authors intended | On PD-S4's corpus of profile-accepted scripts, formatted, validated and calculated values equal Acrobat's black-box results for ≥ 99.5%† of field events; every difference has a recorded reason | Black-box comparison | Profile function fixed or the script class removed from the profile |
| **PD-14** | Selection | On a selection suite (two columns, rotated text, tables, vertical CJK, right-to-left), drag-selection ranges follow the text model's reading order in 100% of cases and copied text equals the selection | Automated | Ship blocker |
| **PD-15** | Read aloud | For the tagged and untagged strata, the OS screen reader's linear reading equals the semantic subtree's order | Automated with AT drivers | Ship blocker for the a11y claim |
| **PD-16** | Printing a PDF | The printed output, re-rastered at 300 dpi, matches the screen's canonical raster within PD-2's metric; size ≤ 2×† the source | Print corpus | PD-D22's reversal |

### 8.2 Security

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **PD-20** | Parsers only in the PDF process | 0 `hg-pdf-*` parser crates in the closure of any other binary (G110); a runtime probe feeding PDF bytes to every other process kind finds 0 parsing paths | `hg-graph-lint`; T2 probe | Ship blocker |
| **PD-21** | No `unsafe` | 0 `unsafe` blocks in `hg-pdf-*`; 0 inventory entries | Lint | Ship blocker |
| **PD-22** | Fuzzing | Every parser, filter, font type, decoder entry, content operator class and the form-logic recogniser is a fuzz target, run on x86-64, arm64 and riscv64 builds; ≥ 24 h† per target per release with 0 new crashes, hangs or limit escapes; coverage published | Fuzz farm (Stage 1 7.5) | Release holds that target's feature behind the emergency switch |
| **PD-23** | Dangerous paths | On the NDSS 2021 set and pdf-insecurity.org's files: 0 network requests, 0 launches, 0 silent submissions, 0 credential leaks, 0 file accesses; an adversarial PDF-process harness (bytes changed between ranges, field values encoded in range offsets, foreign chunk hashes, decryption requests for foreign ciphertext) gets 0 requests after the first edit, 0 cross-namespace chunk resolutions and only consented, SHA-derived keys | T2 probe with the network service recording | Ship blocker |
| **PD-24** | Signatures | 0 files in the forgery, incremental-saving, wrapping, shadow and certification suites shown as "unchanged since signing" when content differs; every post-signing change shown in the revision diff | Suite | Ship blocker for signature display |
| **PD-25** | Bombs and loops | The chained-Flate bomb, cycle files and 10⁷-operator pages stop within their limits in ≤ 1 s† each, with the PDF process inside its heap limit and no effect on other tabs; a page with 500 embedded PDFs keeps ≤ 8 live PDF processes | T2 | Ship blocker |
| **PD-26** | Security response | Fixed release ≤ 72 h† for a PDF bug exploited in the wild, ≤ 7 days† for critical or high; the emergency switch used when a window is missed | Security-response log | As JS-16 |
| **PD-27** | Capability and key audit | The PDF process's granted handles equal G114's list on every OS (no network, file, clipboard, device, storage); its key equals its content GPU process's and viewer renderer's key for 100% of corpus navigations, subdomain-hosted PDFs included; no password or decrypted string reaches the viewer renderer's memory for the encrypted stratum | Sandbox self-test at start; probes | Ship blocker |

### 8.3 Privacy

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **PD-30** | No phone-home | 0 network requests caused by opening, scrolling, printing or closing any corpus PDF, other than the document's own range reads | Network-service log in T2 | Ship blocker |
| **PD-31** | Fixed environment | The form-logic environment is identical across all reference devices and OSes for one build | Probe | Ship blocker for PD-D13 |
| **PD-32** | Writer metadata | 0 occurrences of user name, host name, path, time zone or locale in writer output | Scan of the print corpus | Ship blocker |

### 8.4 Determinism

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **PD-35** | Replay | A recorded viewing and form-filling session replays to identical display-list hashes, text models and saved bytes | DST farm | Ship blocker |
| **PD-36** | Writer bytes | Print-to-PDF output byte-identical across macOS, Linux, Windows and x86-64, arm64, riscv64 for the print corpus with recorded inputs | Cross-platform hash test | Ship blocker for PD-M4 |

### 8.5 Performance and footprint

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **PD-40** | First page | Time from navigation response start to first page fully drawn, p50 and p95, ≤ best shipping viewer on each corpus class (1, 100, 1,000 pages; scanned; form-heavy; CAD; oversized page), local and over a 20 Mbit/s, 50 ms link | Performance lab | Profile and fix before the milestone exits |
| **PD-41** | Scrolling and zoom | p95 frame time during scroll and pinch zoom ≤ the display's frame interval on the reference devices for pages under 10⁵ operators; for heavier pages, first canonical tile at a new scale ≤ 250 ms† p95, with resampled tiles shown meanwhile | Lab | As PD-40 |
| **PD-42** | PDF process memory | PD-F1 and PD-F2 budgets (6.21) held on 100% of the corpus | Lab | Budget review in the decision log |
| **PD-43** | Large documents | Whole-browser memory with a 1,000-page document open ≤ the best shipping viewer's, and ≤ FP-S30's budget | Lab | As PD-42 |
| **PD-44** | Binary size | PD-F5 held | CS-PF-8 report | Size review |
| **PD-45** | Writer speed and size | For 100-page text and image documents, time ≤ Chrome's and bytes per page ≤ Chrome's, with tags and PDF/A-4 on | Black-box comparison | Profile; image and stream settings reviewed |

### 8.6 Accessibility and writer quality

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **PD-50** | Tagged mapping | For 100% of the tagged stratum, the semantic subtree equals the mapping table applied to the author tree (repairs only in the overlay) | Automated | Ship blocker |
| **PD-51** | Screen readers | On an IAG-style matrix (headings, reading order differing from paint order, figures with alt text, tables with headers, forms) with NVDA, JAWS, VoiceOver and Orca, Izcal's verdicts are at least those of the best viewer in [RT 7.4][RT] (Safari, Preview or Acrobat) | Manual test with the accessibility track, each release | Ship blocker for the a11y claim |
| **PD-52** | Users | Task success of screen-reader users on PDF tasks (find a section, fill a form, read a table) ≥ the best shipping viewer's, in a study with the accessibility track's contracted researcher | Study at PD-M2 and PD-M3 | Design review |
| **PD-53** | Writer conformance | 100% of print-corpus outputs, including Display P3 and Rec. 2020 pages, pass `hg-pdf-check`'s PDF/A-4 checks and veraPDF (black box); for pages without author accessibility errors, 0 failures of the machine-checkable WTPDF/PDF/UA-2 requirements | `hg-pdf-check`; veraPDF | Ship blocker for PD-M4 |
| **PD-54** | Writer text | 100% of printed text extracts with correct Unicode in logical order, including emoji, ligatures and fallback fonts, by Izcal's extractor and two black-box extractors | Print corpus | Ship blocker for PD-M4 |

### 8.7 Observability

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **PD-60** | Explained decisions | 100% of substitutions, repairs, skipped features, limit hits, raster fallbacks and actions appear in the IIP `pdf.*` domain with a clause anchor or a deviation record | Coverage check over the corpus | Ship blocker for the IIP domain |
| **PD-61** | Facts panel | The document-facts panel lists 100% of the actions, scripts, signatures, substitutions and repairs the IIP reports | Automated comparison | Ship blocker |

## 9. Milestones and staffing

### 9.1 What is staged, and why

| Item | Reason for staging | Kind |
|---|---|---|
| The imaging extensions of PD-D4 land after PD-S2 measures them | The process-colour encoding's cost is unmeasured | Measurement |
| Form logic (PD-D13) after PD-S4 | Its coverage decides between (b) and (c) | Measurement and security |
| Annotation authoring at PD-M3 | Every authored annotation is saved through the incremental writer and field editing, which must be proven first (PD-8, PD-9) | Correctness |
| Trust evaluation of signatures at PD-M3, if chosen | Needs trust-list terms and a legal reading of "valid" | Correctness (legal) |
| The print-to-PDF product with Stage 2's print UI | Stage 1 13.2's owner decision; the writer core ships in Stage 1 for saving and printing PDFs | Owner decision (13.2) |
| Static XFA rendering only if its trigger fires | No measured need yet | Measurement |
| Reflow view, Android viewing | Stage 2 with the reader view and S-8 | Owner decisions (13.2, S-8) |

### 9.2 Milestones

| ID | Milestone | Exit criterion (demo plus measurement) | Depends on |
|---|---|---|---|
| **PD-M0** | Foundations (month 9†) | **Entry criterion: Q-PD-1 answered** (the no-cost ISO copy's terms permit MIT descriptions derived clause by clause). `.hgpdf` descriptions for clauses 7.2–7.5 and the dictionaries of 7.7–7.9; `hg-pdfgen` with Kani harnesses; strict parser and the first repair rules; **PD-S1** census (feature prevalence: scripts and what they call, XFA static and dynamic, encryption revisions, repairs needed, Type 3, non-embedded fonts, transparency, overprint and spot colours, tags and their errors, actions) on the untruncated Common Crawl corpus; **PD-S3** (per-document process: spawn time and empty memory on three OSes and three ISAs); the corpus fixed; Medium clause numbers and Table 22's accessibility bit checked against the no-cost copy; black-box confirmation of other viewers' script and XFA behaviour; threat model; PDF process skeleton sandboxed on three OSes; fuzz targets running | GR-M0, TX-M0, MD-M0 |
| **PD-M1** | First pages | All filters and security handlers R2–R6 with pinned range reads; interpreter for gray and RGB (device and ICC), paths, images (class P), all embedded font types and substitution, glyph masks and outlines, axial and radial shadings, alpha soft masks, blend modes; text model and find; viewer application with scrolling, zoom, thumbnails, outline and find; per-producer namespaces in the content GPU process; **PD-S2** measured; PD-1, PD-4 (riscv64 hardware or the agreed emulator included), PD-7, PD-20, PD-21, PD-25 at threshold | GR-M1 (display list, raster), TX-M1, MD-M1 |
| **PD-M2** | Viewer complete (Stage 1 M7) | PD-D4's extensions in `hg-raster` (CPU, GPU and SIMD paths); CMYK, Lab, DeviceN, Separation, overprint; meshes and tiling patterns; Type 3; tagged-PDF accessibility with the repair overlay and inferred structure; AcroForm filling, appearances and incremental save (encrypted documents included); annotations displayed; the actions policy; signature integrity, coverage and revision diffs; optional content and attachments; printing a PDF through the writer core and the subsetter's core; IIP `pdf.*` and the facts panel; **PD-S4** and **PD-S6** (screen-reader validation of the mapping) | GR-M2 (30–42 months†), TX-M2, MD-M2, the accessibility spec's cross-process interface |
| **PD-M3** | Stage 1 exit | The form-logic profile (and (c) if chosen); annotation authoring; public-key handler with the broker's derived-key rule; trust evaluation if chosen; every PD target at its Stage 1 threshold; external security audit of the PDF process | Stage 1 M8 |
| **PD-M4** | Writer (Stage 2, with the print UI) | Per-key writer jobs, broker concatenation and the joiner; tags, outline, links, PDF/A-4 with generated ICC profiles; instancer and colour-glyph export; effects fallback; **PD-S5** (PDF 2.0 output in widely used readers); PD-36, PD-45, PD-53, PD-54 | LY-D8's paged layout; GR's print display lists; Stage 2 print UI |

### 9.3 Staffing and duration

No public source gives a team size for a PDF engine. The estimate follows the media and fonts specs' method: a **component inventory** in thousands of lines of production Rust†, excluding generated code and tests, at **3–4 KLOC† of verified, reviewed code per engineer-year**.

| Component | Production KLOC† | Non-code work |
|---|---|---|
| `.hgpdf`, `hg-pdfgen`, proofs; syntax, cross-reference, revisions, repair grammar, pinned range reads | 18–25 | Descriptions from clauses 7 and 12–14; repair observations |
| Filters, security handlers, crypt filters, public-key handler, re-encryption on save | 6–9 | Corpus of encrypted files |
| Interpreter: graphics state, colour spaces, functions, shadings, patterns, transparency lowering, Type 3, glyph path | 25–35 | Ghent suite analysis |
| Display-list and `hg-raster` CPU extensions (process-colour encoding with shape and opacity, knockout and non-isolated groups, overprint, meshes, tiled sub-lists, soft-mask backdrop and transfer) | 12–18 | Review by the graphics owner |
| GPU compute kernels for the same items | 8–12 | Device lab |
| Per-ISA SIMD kernels (x86-64, arm64, riscv64) with scalar references | 3–5 | Differential tests |
| PDF font glue, encodings, CMaps, substitution | 8–12 | — |
| PD-D8 data build and validation | 4–6 | Small: (a) decided |
| Text model, layout analysis, find, selection | 10–14 | Extraction benchmark |
| Structure mapping, repair overlay, inferred structure | 8–12 | Mapping filed with the W3C task force |
| Annotations, AcroForm, appearances, field editing, actions, saving | 18–25 | Form corpus |
| Form-logic profile and its differential recogniser | 5–8 | ISO 21757-1 reading; Acrobat black-box observations; PD-S4 |
| Signatures (CMS, MDP, revision diff) | 6–9 | Signature suites |
| Viewer application and endpoints | 15–20 | UX research with the shell team |
| Writer core, print writer, joiner, tags, PDF/A-4, effects fallback | 18–25 | Validator runs |
| Font subsetter, instancer, colour-glyph export (a fonts-spec crate, built here) | 8–12 | Review by the fonts owner |
| `hg-pdf-ref` reference renderer (own interpreter, compositor, shading, colour and raster) | 25–35 | Independent authorship from the imaging track |
| `hg-pdf-check` | 6–9 | Reading PDF/A-4, WTPDF, PDF/UA-2 |
| Fuzzing, census, differential harness, adversarial process harness | 6–9 | Corpus handling |
| IIP `pdf.*`, facts panel, writer report | 6–9 | With the DevTools track |
| **Total** | **220–319** | |

At 3–4 KLOC per engineer-year that is 55–106 engineer-years† of code. Over the roughly four to five years from PD-M0 to Stage 1 exit, with the writer finishing in Stage 2, and adding review, security response, corpus and accessibility-testing work, the tracks are:

| Track | Scope | Engineers† |
|---|---|---|
| Syntax, security handlers and signatures | `hg-pdfgen`, syntax, cross-reference, repair, range pinning, filters, crypt, CMS | 3–4 |
| Imaging | Interpreter, colour, transparency, shadings, display-list and `hg-raster` extensions on CPU, GPU and three ISAs | 5–7 |
| Reference renderer | `hg-pdf-ref`, kept independent of the imaging track | 2–3 |
| Fonts and text | PDF fonts, Type 3, substitution, PD-D8 data, text model, find and selection | 3–4 |
| Interactive | Annotations, forms, appearances, field editing, actions, saving, form logic | 3–4 |
| Viewer and accessibility | Viewer application, UX, structure mapping, repair overlay, inferred structure, screen-reader testing | 3–4 |
| Writer | Writer core, print writer, joiner, tags, PDF/A-4, subsetter and instancer (with the fonts owner), `hg-pdf-check` | 3–5 |
| Security, robustness and corpus | Fuzzing, census, differential and adversarial harnesses, threat model, a share of the response rota | 2–3 |
| Observability | IIP `pdf.*`, facts panel, writer report, with the DevTools track | 1–2 |
| **Total** | | **25–36†**, plus the accessibility track's contracted researcher for PD-52, a legal review (Q-PD-2, Q-PD-4, Q-PD-5, Q-PD-6, Q-PD-8) and, if Q-PD-4 bars published characterisation data, a press-measurement contract for the default CMYK characterisation |

**Overlap with other specs, line by line.** The media spec counts `hg-jpx`, `hg-jbig2` and `hg-ccitt` (15–22 KLOC there); class P and the encoded-JPEG grant are small additions this spec's imaging track writes under media review. The fonts spec counts Type 1 (its "Type 1 and Type 3 for PDF" row shrinks by Type 3, which moves here) and the Symbol and ZapfDingbats faces (its type-design track). The graphics spec's print track (3–4) counts print display-list production, print raster and preview, and capture; this spec counts display-list-to-PDF lowering, fonts, images, the joiner, and the `hg-raster` extensions it writes under graphics review. The layout spec counts paged layout; the accessibility spec counts the cross-process semantic interface. No engineer is counted twice. Reconciliation 2 listed PDF as uncounted ([RN2 Section 4][RN2]); this spec adds **25–36 engineers†** to that running total. **Duration:** PD-M2 follows GR-M2 (30–42 months†) and Stage 1 M7; PD-M3 is Stage 1 exit (48–60 months†, the combined critical path set by layout, [RN2 Section 4][RN2]); PDF is not on the critical path if PD-M0 starts with the graphics, fonts and media M0s. Hiring people who know the PDF imaging model and its colour science is the schedule risk.

### 9.4 Decisions for the owner

A bare "yes" means the recommendation. Build-versus-reuse is not on this list: the standing answer is "build our own" (M-1, D-S10). **The owner most needs to see PD-D1, PD-D4, PD-D5, PD-D13, PD-D14, PD-D16 and PD-D18** (PD-D8 was decided on 20 October 2026).

| ID | Decision | Options | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|---|
| **★ PD-D1** | Where PDF bytes are parsed | (a) site renderer; (b) pooled utility; (c) one PDF process per renderer key; (d) one per open document, chosen and charged by the renderer process key, strictest tier, ≤ 8† live per key | **(d)** (6.1) | One small process per open PDF (≤ 3 MiB† empty) | PD-S3 shows per-process cost above 2×† the media process's: reduce the baseline (lazy font endpoint, smaller arenas, shared engine image); never co-locate documents for memory (FP-D8) |
| PD-D2 | Viewer architecture | (a) Izcal HTML viewer in the key's renderer, pages as embedded surfaces, secrets kept out; (b) canvas plus DOM text layer; (c) UI in the shell | **(a)** | Surface embedding and semantic stitching with a non-renderer producer | The viewer's renderer memory exceeds PD-F4 on two releases: move the toolbar to the shell's UI renderer with typed descriptors only |
| PD-D3 | Parsing | (a) generated strict parser, one resolved view, named repair grammar; (b) hand-written lenient parser | **(a)** (6.3) | `hg-pdfgen` | PD-11 misses on two releases despite new repair rules: review the strict-first order |
| **★ PD-D4** | Rendering path | (a) approximate in sRGB; (b) a PDF rasterizer in the PDF process; (c) extend the display list and `hg-raster` | **(c)** (6.6) | 23–35 KLOC† in graphics crates (CPU, GPU, SIMD), under graphics review | PD-S2: a page class costs > 2×† with no visible difference: convert groups that need neither overprint nor non-separable blending at fill time |
| **★ PD-D5** | Colour and overprint | (a) group blending in its declared space with shape and opacity, spot channels, overprint simulation on by default, class P decode, `hg-colour`'s f32 contract; (b) RGB at fill time | **(a)** (6.6, 6.7) | Process-colour encoding; MD-D8 amendment | As PD-D4 |
| PD-D6 | Default CMYK characterisation | (a) ISO 10.4.2 formula; (b) Izcal-built CLUT from published characterisation data or Izcal's own press measurements; (c) OS profile | **(b)**, with (a) until data is cleared (Q-PD-4) | Data work or a measurement contract | — (the interim is explicit) |
| PD-D7 | Fonts | (a) TX stack, Type 3 as PDF content, Izcal aliases with width scaling; (b) substitute OS fonts; (c) Izcal faces without width scaling | **(a)** (6.8) | TX amendment (two faces) | PD-5 shows substitution-caused disagreements above 0.1%† of pages: extend the alias set |
| **★ PD-D8** | CMaps, CID-to-Unicode and glyph-name data | (a) owner rules Adobe's BSD-3 tables registry data under OD-L6; (b') generate what is derivable, ship Adobe's orderings only under (a); (c) embedded data only | **(a), decided by the owner on 20 October 2026** (HGBA-2 2.3.3: standards registry data) (6.8) | One ruling (made) | — |
| PD-D9 | Text model | (a) one extractor with a written mapping chain, layout analysis, ligature and hyphen rules; (b) separate extractors per feature | **(a)** (6.9) | — | PD-6 misses: tune the specified algorithm, never fork it |
| PD-D10 | Loading | (a) pinned range reads, hint tables unused; (b) use hint tables; (c) whole-file download | **(a)** (6.2) | — | PD-40 shows hint tables would cut first-page time > 20%† on the linearized stratum |
| PD-D11 | Accessibility | (a) author tree, repair overlay, inferred structure; OCR only through the ML service; (b) trust author tags only | **(a)** (6.10) | Mapping work | PD-51 or PD-52 shows inferred structure misleads users: expose it only on request |
| PD-D12 | Forms and saving | (a) AcroForm editing in the PDF process, incremental save, re-encryption, authoring at PD-M3; (b) full rewrite on save | **(a)** (6.12) | — | — |
| **★ PD-D13** | JavaScript in PDF | (a) none; (b) declarative form-logic profile; (c) jitless sandboxed subset in a per-document form-script process; (d) Acrobat API | **(b)** (6.17) | 5–8 KLOC† | PD-S4: (b) covers < 90%† of corpus form scripts: owner chooses (c) |
| **★ PD-D14** | XFA | (a) ignore silently; (b) AcroForm fallback and plain explanation; (c) static XFA renderer; (d) full XFA | **(b)** (6.18) | — | PD-S1 or opt-in reports: dynamic XFA > 1%† of opened PDFs with forms, or a public form estate requires it: (c) |
| PD-D15 | Actions | (a) the table in 6.13 (no automatic effects, consent for submission, Launch never); (b) Acrobat's action model with prompts | **(a)** | — | — (security floor) |
| **★ PD-D16** | Writer placement | (a) renderer; (b) one process of the top key; (c) per-key writers plus a keyless assembler of bytes; (d) per-key writers into broker-concatenated file segments plus a keyless joiner of typed descriptors; (e) the content GPU process | **(d)** (6.19) | Object-range grants and segment files in the broker | The joiner's descriptor format needs content bytes for some PDF feature: that feature is written by the top key's writer from typed data, never by the joiner |
| PD-D17 | Signatures | (a) integrity and coverage, signer unverified; (b) plus EU Trusted Lists; (c) OS trust | **(a) at PD-M2; (b) at PD-M3 if Q-PD-6 clears** (6.14) | CMS parsing; trust-list ingestion for (b) | Legal review finds "integrity only" display misleading under eIDAS: show integrity in a separate panel from identity |
| **★ PD-D18** | Writer output profile | (a) tagged PDF 1.7, no identification; (b) PDF 2.0 + WTPDF structure + PDF/A-4, report, no PDF/UA claim; (c) as (b) with automatic PDF/UA-2 claims; (d) PDF/A-2u on 1.7 | **(b)** (6.19) | `hg-pdf-check` | PD-S5 finds widely used readers failing on PDF 2.0 output: (d) |
| PD-D19 | Writer fonts | (a) own subsetter and instancer, colour glyphs as Type 3 with `ToUnicode`, `fsType` gate with outline fallback; (b) Type 3 for everything that is not a static TrueType (Chrome's practice) | **(a)** | TX amendment | — |
| PD-D20 | Writer images and effects | (a) JPEG pass-through, lossless otherwise, effects rastered at 300 dpi†; (b) re-encode all images at display resolution | **(a)** | — | PD-45 size target missed on image pages: revisit the lossless policy for lossy sources |
| PD-D21 | Writer determinism | (a) bytes a pure function of recorded inputs, dates from the recorded print time in UTC; (b) wall-clock dates by default with an opt-in reproducible mode | **(a)** | — | — |
| PD-D22 | Printing a PDF | (a) original bytes to the OS; (b) canonical raster only; (c) re-emitted PDF with Izcal-regenerated fonts, raster where the OS needs it | **(c)** (6.16) | Subsetter core at PD-M2 | PD-16: output > 3×† the source size or > 2×† slower than the OS path on the print corpus |
| PD-D23 | Multimedia and 3D | (a) poster only, save as attachment; (b) play through the media stack | **(a)** (6.11) | — | glTF rich media in measured use |
| PD-D24 | Embedding and downloads | (a) inline unless `attachment`; HTML's PDF viewing support; sandbox flags restrict actions; (b) always download | **(a)** (6.15) | — | — |
| PD-D25 | Encryption | (a) decrypt R2–R6 and AES-GCM; re-encrypt only for incremental saves; legacy crypto in its own crate; (b) refuse legacy handlers | **(a)** (6.5) | NW amendment | — |
| PD-D26 | Observability | (a) IIP `pdf.*`, facts panel, writer report; (b) DevTools only | **(a)** (6.24) | 6–9 KLOC† | — |

### 9.5 Amendments this spec proposes to other documents (not applied)

| Document | Amendment |
|---|---|
| Stage 1 design, Section 1 | The utility row's "PDF parsing and rendering" becomes a **PDF process** row: one per open document, chosen and charged by its renderer process key, ≤ 8† live per key, strictest tier, owning parsing, decoding, interpretation, text and structure (PD-D1); a **print joiner** utility row (keyless, fresh per print, typed descriptors only) (PD-D16). Closes C-21 with the media spec's "PDF process" |
| Stage 1 design, 2.2 and G9 | Register G110–G119 and the L2d sub-layer; G9's behaviour-crate list adds `hg-pdf-*` |
| Stage 1 design, 6, 8.1, 8.3, 13.2 | Section 6's PDF row: annotation authoring moves to Stage 1 (PD-M3); M7's "PDF via utility processes" becomes PD-M2; a PDF track of 25–36† engineers; 13.2 states that the writer core ships in Stage 1 (saving and printing PDFs) while print-to-PDF stays Stage 2 |
| Phase 3 ledger and the rule registry | Register G110–G119 for this spec; G-14 and C-21 owned here; A-FUT-2 answered with class P as well as G50–G53; A-FUT-6 answered (PD-D1, PD-D4, PD-D16) |
| HGBA-2, I-1 | The PDF process, its content GPU process and its viewer renderer are keyed by the renderer process key; no I-1 exception is needed for printing (PD-D16 (d)) |
| Fonts and text spec | Type 3 is PDF content executed by `hg-pdf-interp` (6.18, TX-D7's text and the glyph-program staffing row); Symbol and ZapfDingbats aliases (about 190 and 200 glyphs) with their built-in encodings in TX-W1, costed by the type-design track; alias advance tables serve standard-14 fonts without `/Widths`; `hg-font-subset` (generated writers) with variable-font instancing and colour-glyph export, built by this spec's writer track and owned by the fonts spec; glyph masks and outlines for PDF text produced in the PDF process (6.6); print-mode text items carry cluster maps; field editing in the PDF process uses TX-D23's services |
| Images and media spec | **MD-D8 gains class P (source samples)** for the PDF process only; `hg-entropy`'s LZW gains `EarlyChange`; `hg-jpeg` and `hg-jpx` expose reduced-resolution and unconverted CMYK, N-channel and Lab output; the media process gives its key's writer job encoded JPEG bytes on a print-job grant; `hg-colour` builds PDF colour-space tables under MD-D28's f32 contract, holds the default CMYK characterisation (PD-D6) and generates ICC profiles for writer output (sRGB, Display P3, Rec. 2020) |
| Graphics spec | 6.27's "PDF viewing" row: the PDF process's display list, rastered by the content GPU process of the document's renderer key; the content GPU process accepts several producers per key with per-producer chunk namespaces, hash verification and budgets (G62, G71); PD-D4's extensions with shape and opacity channels, spot channels, `AIS` and `TK`, on CPU, GPU and three ISAs; FP-14 and GR 6.22 rows for process-colour working sets; print-mode items carry semantic node ids; the vector print row names per-key writer jobs and the joiner; the print-track and PDF writer-track split (9.3) |
| Layout spec | 6.7: structure records for print (headings for the outline, link areas and targets) come from fragments and the semantic plane, as Q24 states; no change in substance |
| Networking spec | `hg-crypto-legacy` (L2s, G36): `md-5`, `rc4`, `cbc`, linked only into the PDF process, decrypt and re-encrypt-for-incremental-save only (G119); SHA-512 in `hg-crypto`; the pinned one-URL range grant (strong validator, `If-Range`, block coalescing, request cap, revocation at full fetch or first edit); DER and X.509 parsing split from `hg-pki` (L5) into an L2s crate that `hg-pdf-sig` can link; Ed448 stays out |
| HTML and CSS parsing and style spec | G24's sibling list gains L2d; XML in PDF through `hg-xml-syntax` in the PDF process only (G23); G22's differential covers the form-logic recogniser |
| JavaScript engine spec | A parse-tree export of Izcal's JS parser for the recogniser's CI differential (test-only); a jitless build for a per-document form-script process only if PD-D13 (c) is chosen |
| Resource-footprint spec | Process kinds and budgets PD-F1 to PD-F5 (PDF process, viewer, writer job, joiner); ≤ 8† live PDF processes per key; display-list residency; a hidden document with unsaved changes or a key is not reclaimed; the PDF process's canonical heap limit (1 GiB†) |
| Accessibility spec | The cross-process semantic interface accepts a **non-renderer producer** (the PDF process) stitched under a renderer's element, with the same validation and budgets as out-of-process frames; the PDF mapping table; an "inferred" label in the repair overlay; PD-14, PD-15 and PD-51 in its test plan |
| DevTools spec | The IIP `pdf.*` domain (6.24), with decrypted strings and field values marked sensitive |
| Security scope | Rows and threat models for the PDF process, writer job and joiner; the broker's key-store rule (consent, derived file key only, one request per recipient per document); the PDF password prompt as the shell's `document-password` class (shell 6.4; broker-owned, high risk, secret straight to the document process, never recorded); new UPD-R5 switches (form logic, single PDF decoders, signature parsing, PDF viewing); the Hardened profile's PDF settings; SEC-12's metric names the PDF process |
| [Coding standard][CS] | `tiers.toml`: the PDF profile for every `hg-pdf-*` crate |
| Shell UI design | R18's "PDF editing" splits: annotation authoring at PD-M3, page-content editing stays Stage 2; "always download PDFs" also sets `navigator.pdfViewerEnabled` false; the password prompt is the shell's `document-password` class (shell 6.4); the facts panel lives in the viewer |
| Licence audit | PD-D8 (a) as a class 4 / OD-L6 registry-data ruling with the BSD-3 notice; test-only rows for Arlington, veraPDF, the Ghent suite, the corpora and Adobe's AFM files; `md-5`, `rc4`, `cbc` rows; CMYK characterisation data (Q-PD-4) |
| ML service (`reports/Izcal ML service.md`) | OCR for image-only PDF pages under the accessibility grant (ML-D4, ML-OCR), fed by the content GPU process's canonical raster, starting on a user action or an AT read and never on document open (ML-12) |

## 10. Risks

| Risk | Likelihood and impact | Mitigation | Trigger |
|---|---|---|---|
| **The repair grammar never converges**: real files need repairs no rule covers | Medium; fidelity | PD-S1 measures before rules are fixed; PD-11 compares with black-box viewers; each new rule is a named observation | PD-11 misses on two releases |
| **Process-colour groups are too slow or too large** for CMYK-heavy print PDFs | Medium; performance and memory | PD-S2 at PD-M1; Annex Q gating; per-tile channel counts and banding; PD-D4's reversal | PD-S2 result |
| **The form-logic profile covers too few forms**, or computes differently from Acrobat | Medium; compatibility | PD-S4 on the corpus; PD-13; option (c) ready | PD-S4 < 90%†; PD-13 miss |
| **Per-document processes cost too much** on small devices | Low to medium; footprint | PD-S3; ≤ 8† per key; per-process cost reduction | PD-S3 result |
| **PD-D8 (a) is later reversed** (decided 20 October 2026) | Low; CJK text extraction and rendering of non-embedded CID fonts | PD-D8 (b'): generate what is derivable, label the gap | A licence challenge to the registry-data ruling |
| **No MIT CMYK characterisation** | Medium; colour fidelity | ISO 10.4.2 interim; legal review (Q-PD-4); press-measurement contract | Q-PD-4 answered "not usable" |
| **Common-mode bugs between `hg-pdf-ref` and the engine** in the shared syntax layer and `hg-fontfmt` | Medium; correctness | PD-1 and the fonts spec's own oracles cover the shared layers; differential triage against black-box viewers (PD-5) | A PD-5 finding traced to a shared crate |
| **PDF 2.0 output breaks old readers** | Low to medium; writer adoption | PD-S5; PD-D18 (d) fallback | PD-S5 result |
| **Patent position of PDF 2.0** | Low; legal | The 2008 Adobe licence covers PDF 1.7 only ([FS 1.7–1.8][FS]); legal review of ISO's patent declarations for ISO 32000-2 before PD-M2 | Any declared essential claim without royalty-free terms |
| **XFA-dependent forms** that people must fill | Medium; user harm | Plain explanation and "open with another application"; PD-D14's trigger | Trigger in PD-D14 |
| **Hiring** people who know PDF imaging, colour science and form semantics | High; schedule | Train from the standard and the Ghent suite; start PD-M0 with the graphics, fonts and media M0s | PD-M0 slips past month 9† |
| **Logic bugs outside memory safety** (origin confusion, navigation bypass, Chrome's 2026 class, [SE 1.4][SE]) | Medium; security | G114, G118, PD-23, PD-27; the viewer at an internal opaque origin; secrets kept out of the viewer | Any finding of that class |

## 11. Open questions and flagged claims

### 11.1 Open questions

- **Q-PD-1.** The licence of the no-cost ISO 32000-2 and PDF/UA bundles: may test cases quote clause text, and may derived data (descriptions, tests) be published under MIT? A PD-M0 entry criterion ([FS 1.4][FS]).
- **Q-PD-2.** Answered as moot: PD-D8 (a) was decided on 20 October 2026, so whether code-to-CID assignments and glyph-name mappings are copyrightable facts no longer gates shipping.
- **Q-PD-3.** Answered: Arlington's Apache-2.0 data is test-only; no build-time generator reads it ([FS 12.2][FS]).
- **Q-PD-4.** May published CMYK characterisation data sets (CGATS TR 001, FOGRA39) ship as standards data inside a generated CLUT? Otherwise, the cost of Izcal's own press measurements.
- **Q-PD-5.** Does drawing a restricted (`fsType` 0x0002) font's outlines as paths in a PDF respect its licence?
- **Q-PD-6.** The EU Trusted Lists' reuse terms, and what a "valid signature" statement means legally under eIDAS for a browser.
- **Q-PD-7.** Confirm against ISO 32000-2's Table 22 that readers shall ignore the accessibility-extraction bit; if the text differs, the owner decides whether Izcal ignores it anyway.
- **Q-PD-8.** ISO's patent declarations for ISO 32000-2 and the ITU-T patent statements for T.88 and T.800 ([FS 3.5–3.6, 1.8][FS]).
- **Q-PD-9.** A licensed extraction benchmark: Bast and Korzen's data's terms, or Izcal builds its own from TeX sources with permissive licences.
- **Q-PD-10.** Which black-box limits other viewers enforce (decoded size, nesting, operator counts, print resolution), measured at PD-M0 ([SE 7.4][SE]; [RT 13][RT]; [WP 11][WP]).
- **Q-PD-11.** Whether Chrome stable now exposes PDF tags to screen readers (a 2026 re-run of the IAG test) ([RT 7.6][RT]).
- **Q-PD-12.** The share of web PDFs with JavaScript, XFA, Launch and non-embedded fonts, and what form scripts call (PD-S1, PD-S4) ([GO 1.12][GO]).
- **Q-PD-13.** The Ghent Output Suite's terms for test use.
- **Q-PD-14.** Does ISO 21757-1 define the Acrobat `AF*` form functions' formatting and rounding rules?

### 11.2 Claims that are single-source, unverified or conflicting

- Feature prevalence (forms 9.26%, signatures 2.04%, optional content 4.32%, encryption 2.5%) comes from one vendor study (Dual Lab, 20.6 million PDFs, CC-MAIN-2026-25), and the tagged-share and ISO/TS 32005 figures from a different Dual Lab wave (20.5 million PDFs, 2023–2026), posted to a blog and a mailing list, not peer-reviewed ([GO 1.5–1.11, 4.1–4.5][GO]).
- Google's 12% image-only estimate has no published method ([GO 1.13][GO]; [RT 8.4][RT]).
- Shadow-attack counts differ between the paper (16 of 29) and the project site (15 of 28) ([GO 6.2][GO]); both are cited.
- Chrome running PDF JavaScript in V8 and not rendering XFA rest on forum posts ([HP 6.4, 10.5][HP]); black-box confirmation is a PD-M0 task.
- XFA's deprecation in PDF 2.0 rests on vendor sources; the clause was not read ([FS 8.5][FS]).
- Clause numbers not on any errata page and inferred from ISO 32000-1's numbering: 7.4.2 and 7.4.5–7.4.10; 8.4.3.2; 8.6.7; 8.7.4.5.2–8.7.4.5.7; 9.6.5.4; 10.7.5; 11.3.4; 11.4.5; 11.4.6; 11.7.3; 11.7.4; 14.8.2.3 ([FS 0][FS]); all checked at PD-M0.
- The accessibility permission bit's deprecation rests on secondary sources quoting Table 22 (Q-PD-7).
- The Ghent suite's 48-patch count is from trade press ([RT 1.6][RT]).
- The 1,024-byte header tolerance (R1) rests on the researcher's recollection of PDF Reference 1.7 ([FS 2.9][FS]).
- Acrobat's overprint-preview default, Firefox's 150 dpi print raster and Chrome's print path rest on recollection ([RT 2.1, 10.1, 10.5][RT]).
- PDF 2.0 having no variable-font form comes from a 2020 presentation abstract ([WP 5.5][WP]).
- The reason for the 2026 rise in Chrome PDF CVEs is unknown ([SE 1.4][SE]).
- Every number marked † is set here from first principles and is unmeasured.

## Appendix A. Review disposition (revision 0.2)

Phase 8 was done by an independent [fact-checker][FK] and an independent [design critic][CR], neither of whom wrote the spec or its notes. Every item was applied; three in a different form than proposed (marked).

**Fact-check, must-fix**

| ID | Item | Disposition | Where |
|---|---|---|---|
| FC-M1 | PDFium "forms (24)", "JPEG (19)" were substring artefacts | Applied; SE 1.3 and SE 12.2 corrected in the note | 3; [SE][SE] |
| FC-M2 | KLOC total and engineer-years mis-summed | Applied, then superseded by C11 and C21's new rows (220–319 KLOC, 55–106 engineer-years) | 9.3 |
| FC-M3 | Accessibility permission bit is deprecated, not a deviation | Applied; Q-PD-7 reframed | 6.5; 11.1 |
| FC-M4 | JS-D12 → JS 6.9 | Applied | 6.6; G116 |
| FC-M5 | GR-D28 → GR 6.0 and RN2 Q4 | Applied | 6.1 |
| FC-M6 | MD 6.17 → MD 6.1, MD-D20; MD 6.13 → MD 6.5 | Applied | 6.1; 6.7 |
| FC-M7 | FP-D16 → FP 6.4 class (a) | Applied | 6.21 |
| FC-M8 | G110–G119 proposed, not assigned | Applied; registry row in 9.5 | 1; 9.5 |
| FC-M9 | 21 of 22 is the 2019 study across three attack classes | Applied; GO 6.3 corrected in the note | 1; [GO][GO] |
| FC-M10 | `SameSite=Lax` cookies still sent on top-level GET | Applied | 6.13 |

**Fact-check, recommended**

| ID | Disposition |
|---|---|
| FC-R1 | Applied (6.10) |
| FC-R2 | Applied: Chrome 131 November 2024 (1) |
| FC-R3 | Applied (11.2) |
| FC-R4 | Applied (4.4, 6.1) |
| FC-R5 to FC-R9 | Applied (2.1, 5, 3) |
| FC-R10, FC-R11 | Applied: exact quotations (6.1, 1) |
| FC-R12 | Applied (3, PD-3, 11.2) |
| FC-R13 | Applied (11.2) |
| FC-R14 | Applied (6.19) |
| FC-R15 | Applied (summary) |
| FC-R16 | Applied (4.2, 6.17, Q-PD-14) |
| FC-R17 | Applied: SHA-512 added (6.5, 9.5) |
| FC-R18 | Applied: live decoded data 512 MiB inside the 1 GiB heap (6.3, 6.21) |
| FC-R19 | Applied: IPC-R1 and IPC-R7; the strictest (utility) tier (6.1) |
| FC-R20 | Applied: UPD-R5 switches by amendment (6.22, 9.5) |
| FC-R21 | Applied (6.6) |
| FC-R22 | Applied: GR 6.0/6.22, GR-51, G5 with no inventory entry (6.19 removed the exception; 6.21; PD-4) |
| FC-R23 | Applied: RN P1 and RN2 Q18; RN2 Section 4; G-15 extended; `hg-font-subset` proposed here |
| FC-R24 | Applied: header trimmed; `[LA]`, `[FM]` used; security scope, coding standard and shell UI design linked |
| FC-R25 | Applied in the notes (HP 7.5, HP 11.6, FS 8.5 links replaced; unreachable links marked) |
| FC-R26 | Applied in the notes (SE 1.3, SE 12.2, GO 6.3) |

**Design critique**

| ID | Severity | Disposition | Where |
|---|---|---|---|
| C1 | High | Applied: renderer process key everywhere; PD-27 key probe | 6.0, 6.1, G114, PD-27, 9.5 |
| C2 | High | Applied: pinned ranges, sealed copy, signatures after full fetch, bounded grant revoked before the first edit; PD-23 cases | 6.2, 6.14, G114, PD-23 |
| C3 | High | Applied: derived file key only, consent, one per recipient | 6.5, G118, 9.5 |
| C4 | High | Applied: re-encryption for incremental saves; G119 restated; PD-8 strata | 6.5, 6.12, G119, PD-8 |
| C5 | High | Applied: MD-D8 class P; colour through `hg-colour`'s f32 contract; G116 restated | 6.4, 6.6, 6.7, G116, 9.5 |
| C6 | High | Applied: PD-D8 rewritten; (a) recommended | 6.8, 7, 9.4 |
| C7 | Medium | Applied: ≤ 8† per key; reversal never co-locates | 6.1, 6.21, PD-25, 9.4 |
| C8 | Medium | Applied: secrets never enter the viewer; broker password prompt | 6.1, PD-27, 9.5 |
| C9 | Medium | Applied: per-producer namespaces and budgets | 6.6, 9.5 |
| C10 | Medium | Applied: shape and opacity channels, `AIS`, `TK`, spot rules, overflow | 6.6 |
| C11 | Medium | Applied: memory rule, GPU and SIMD rows | 6.6, 6.21, 9.3 |
| C12 | Medium | Applied: residency, resampled zoom, PD-41 restated | 6.15, 6.21, PD-41 |
| C13 | Medium | Applied: glyph masks and outlines from the PDF process | 6.6 |
| C14 | Medium | Applied: evidence sources, G22 differential, PD-13 | 6.17, 7, 8.1 |
| C15 | Medium | Applied: XML through `hg-xml-syntax` in the PDF process | 6.12, 6.25, G110 |
| C16 | Medium | Applied: no reclaim with unsaved changes or a key; edit journal | 6.21 |
| C17 | Medium | Applied: option (d) recommended; no I-1 exception | 6.0, 6.19, 9.4, 9.5 |
| C18 | Medium | Applied: option (e) rejected with reason; encoded-JPEG grant; G110 restated | 6.19, G110, 9.5 |
| C19 | Medium | Applied: generated ICC profiles; PD-53 cases | 6.19, PD-53 |
| C20 | Medium | Applied: `hg-crypto-legacy` crate; `cargo tree` check recorded; Ed448 rule | 6.5, 6.14, 7 |
| C21 | Medium | Applied: split stated; `hg-pdf-ref` 25–35 KLOC, 2–3 engineers, shares only syntax and `hg-fontfmt` | 6.25, 9.3, 10 |
| C22 | Medium | Applied: every decision has options; triggers where one exists, "—" only where a floor is intended | 9.4 |
| C23 | Medium | Applied: PD-14, PD-15, PD-16; PD-2 restated with an edge tolerance | 8.1 |
| C24 | Medium | Applied: `/Widths` fallback, two faces costed, interim tier labelled non-canonical | 6.8, PD-4, 9.5 |
| C25 | Low | Applied | 6.3, 6.21 |
| C26 | Low | Applied: PDF profile defined for all `hg-pdf-*` crates; G24, G9, coding-standard rows | 6.25, 9.5 |
| C27 | Low | Applied | 6.1, 6.15, 6.22 |
| C28 | Low | Applied | 6.11 |
| C29 | Low | Applied (with FC-M10) | 6.13 |
| C30 | Low | Applied | 6.10 |
| C31 | Low | Applied in a different form: CID fonts regenerated by Izcal's subsetter; JPEGs re-encoded from a canonical decode; banded raster. PD-D22's milestone moves the subsetter's core to PD-M2 | 6.16, 9.2 |
| C32 | Low | Applied | 6.2 |
| C33 | Low | Applied | 9.5 |
| C34 | Low | Applied | 6.1 |
| C35 | Low | Applied: Q-PD-1 is a PD-M0 entry criterion; PD-3 has a fallback | 9.2, PD-3 |
| C36 | Low | Applied | PD-4, PD-22, PD-M1 |
| E1–E9 | — | Applied: E2 by C6; E3 by C14; E4 by C20; E5 answered Q-PD-3 and stated in 7; E6 by C24; E7 kept blocked until Q-PD-4 with a measurement fallback costed; E8 by C35; E9 stated in 7 | 7, 9.3 |
| R1–R17 | — | Applied in 9.5, except R2 in a different form (no exception needed under PD-D16 (d)) and R17 merged into the ledger row | 9.5 |
| — | Synthesis author's own finding | `hg-pdf-sig` (L4) could not link `hg-pki` (L5): DER and X.509 parsing move to an L2s crate (amendment) | 6.25, 9.5 |

[H2]: HGBA%202%20browser%20architecture%20spec.md
[H3]: Future%20software%20browser%20engine.md
[S1]: HGBA%20Stage%201%20architecture%20design.md
[JS]: Izcal%20JavaScript%20engine.md
[FP]: Izcal%20resource%20footprint.md
[ST]: Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[TX]: Izcal%20fonts%20and%20text.md
[NW]: Izcal%20networking.md
[LY]: Izcal%20layout.md
[MD]: Izcal%20images%20and%20media.md
[GR]: Izcal%20graphics%20and%20rendering.md
[DT]: Izcal%20DevTools%20spec.md
[RN]: ../research_notes/Phase%202%20reconciliation/reconciliation.md
[RN2]: ../research_notes/Phase%202%20reconciliation/reconciliation_2.md
[LA]: ../research_notes/Phase%202%20reconciliation/licence_audit.md
[LG]: ../research_notes/Phase%203%20amendments/ledger.md
[FM]: ../research_notes/Izcal%20PDF/frame.md
[HP]: ../research_notes/Izcal%20PDF/history_postmortems.md
[FS]: ../research_notes/Izcal%20PDF/formats_specs.md
[SE]: ../research_notes/Izcal%20PDF/security_robustness.md
[RT]: ../research_notes/Izcal%20PDF/rendering_text_accessibility.md
[WP]: ../research_notes/Izcal%20PDF/writer_print.md
[GO]: ../research_notes/Izcal%20PDF/gaps_opportunities.md
[FK]: ../research_notes/Izcal%20PDF/factcheck.md
[CR]: ../research_notes/Izcal%20PDF/critique.md
[SS]: Izcal%20security%20scope.md
[CS]: Izcal%20coding%20standard.md
[UI]: HGBA%20shell%20UI%20design.md
