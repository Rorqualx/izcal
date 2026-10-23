# Izcal PDF: frame

Written 19 October 2026 (phase 1 of the first-principles-spec method).

## The problem and who has it

Izcal has no owner for PDF. The Phase 3 ledger records it as gap **G-14** (writer, viewer and PDF process) and conflict **C-21** (Stage 1 13.2's utility process against the media spec's "PDF process"); reconciliation 2's **Q24** settled paged layout and print raster but left the PDF writer and PDF viewing to "a future PDF spec, unowned and unstaffed". Four accepted specs already call into it: fonts and text (TX-D7: Type 1 and Type 3 for PDF only, in the PDF process; print-to-PDF subsets fonts and honours `fsType`), images and media (MD R18: `hg-jpx`, `hg-jbig2`, `hg-ccitt` built by MD, run inside the PDF process under G50–G53), graphics (GR 6.27: PDF viewing uses "the future PDF spec's display list, rastered by these crates"; vector print output takes appearance from the display list), and layout (LY-D8, LY 6.7: paged layout in Stage 1; vector output's structure from fragments and the semantic plane). Stage 1's M7 lists "PDF via utility processes"; 13.2 keeps "printing and print-to-PDF" in Stage 2.

Scope:
- **Viewer**: ISO 32000-2 parsing (objects, xref tables and streams, object streams, incremental updates, hybrid files, linearization, repair of damaged files), filters (Flate, LZW, ASCIIHex, ASCII85, RunLength, CCITT, JBIG2, DCT, JPX, Crypt), encryption (standard security handler R2–R6, public-key handler), content-stream interpretation, fonts in PDF (Type 1, Type 3, bare and CID-keyed CFF, TrueType, OpenType, CMaps, ToUnicode, standard 14 substitution), images and colour spaces (ICC, Lab, Indexed, Separation, DeviceN, patterns, shadings 1–7), transparency (groups, soft masks, knockout, blend modes), overprint, optional content, annotations and AcroForm with appearance generation, the XFA decision, the JavaScript-in-PDF decision, signatures (display and verification), text extraction, search and selection, tagged PDF accessibility (PDF/UA, coordinate with the accessibility spec AX-), printing a PDF, saving and incremental save of filled forms and annotations.
- **Process and sandbox**: where parsing, interpretation and decoding run; which key; capabilities; resource budgets.
- **Rendering**: through the graphics spec's display list and `hg-raster`; what the display list lacks for PDF.
- **Writer**: print-to-PDF and `window.print()` → PDF (LY-D8 paged fragment tree; GR display list for appearance; structure from fragments and the semantic plane, Q24), tagged PDF output (PDF/UA-2, WTPDF), PDF/A decision, font embedding and subsetting of Izcal's own fonts and web fonts (`fsType`), link annotations, outlines, determinism of output bytes, size.
- **Footprint and observability**: budgets for the PDF process and viewer; an IIP `pdf.*` domain; explainers for users.

Not in scope: the print UI and OS print dialog integration (shell, Stage 2), PDF editing beyond forms and annotations (shell design R18, Stage 2 product feature; this spec gives it the save path), OCR of scanned PDFs (an ML service feature under B10; this spec gives the hook), EPUB.

## What Izcal's specs already decide or constrain

- **HGBA-2 2.3.2**: P1–P6, M-1 (no engine libraries: pdf.js, PDFium, Poppler, MuPDF, qpdf, lopdf, pdf-rs, hayro, pdf-writer, krilla are excluded; MIT only; spec anchors; no engine source as evidence), I-1 (process per (site, container)), I-2 (zero ambient authority), I-3.
- **HGBA-3**: B1 (determinism), B2 (typed IPC), B7, B8 (semantic plane), B9/B10 (agents, AI off).
- **Stage 1**: process map (utility row "PDF parsing and rendering"), G1–G9, M7, 13.2 (printing Stage 2).
- **JS**: JS-D13 (tiers; jitless), JS-D15 (decision points), G13–G18 (G18 checked arithmetic).
- **FP**: FP-1 (64 MiB baseline), FP-38 (decoder process ≤ 2 MiB idle), FP-D11, FP-D16, FP-D19 (identical limits on every device), FP-25 (binary size).
- **ST**: G19–G24 (no recursion on content depth G21; one parser per grammar G23; L2s G24).
- **TX**: TX-D2, TX-D3, TX-D7, TX-D10 (fuel), TX-D19/D24 (metric-compatible aliases), G25–G29 (G25 only `hg-fontfmt` reads font bytes; G28 no floats on observable text paths).
- **NW**: `hg-compress` (deflate encoder and decoders, P5), `hg-crypto` (G36: only crate with crypto primitives), G30–G35.
- **LY**: LY-D8, LY 6.7, G37–G45.
- **MD**: MD-D5, MD-D8, MD-D28 (`hg-colour`), `hg-icc`, `hg-jpeg`, L2m crates, G50–G57 (G50 permits the PDF process; G51 generated parsers; G53 fuel, no recursion).
- **GR**: GR-D3 (canonical integer pixels), GR-D5/D6 (display list, `.hgidl`), GR-D16 (blending in encoded sRGB; S8/E16), 6.27, G60–G72.
- **CB**: G73–G80 (approximately).
- **DevTools**: IIP domains per spec.
- **Accessibility (in progress, AX-)**: owns the interface PDF content uses to reach the tree.
- **G-rule range for this spec: G110–G119. ID prefix: PD-, PD-D#, PD-M#.**

## What success looks like (measurable)

- **Correctness**: rendering of a conformance corpus (PDF Association and ISO test files, Ghent Output Suite, Isartor/veraPDF for structure) matches a float64 reference renderer within a stated tolerance, and differential triage against black-box viewers resolved by the ISO text, not by matching an engine.
- **Security**: zero memory-safety bugs by construction (no `unsafe`), no parser or decoder outside the PDF process, the PDF process holds one (site, container)'s data and no network or file authority; fuzzing hours; time to fix.
- **Robustness**: every damaged file in the corpus opens with a deterministic repair or a clear error; no hang (fuel), no OOM (budgets).
- **Performance**: first page visible, page-turn latency and memory at or better than the best shipping viewer on a large-PDF corpus.
- **Accessibility**: tagged PDFs exposed with their structure; untagged PDFs get a reading order; screen-reader task success.
- **Writer**: output passes PDF/UA-2 and (if chosen) PDF/A checks with a black-box validator; byte-identical output for identical inputs across OSes and ISAs; size and time at or better than shipping browsers.
- **Privacy**: a PDF cannot phone home without a user action; no fingerprinting surface beyond the build.
