# Izcal PDF: formats and specifications

Compiled 19 October 2026. Primary sources were preferred (specifications, standards bodies' pages, the PDF Association's errata site, papers, licence texts).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

M-1 note: this file records the standards Izcal's PDF viewer and writer anchor to, their clause numbers, their legal terms, and the data sets that can test them. No source code of any browser, PDF engine or PDF library was opened. pdf.js, PDFium, Poppler, xpdf, MuPDF, qpdf and similar appear only as landscape or as black-box oracles named by others' papers. The Arlington PDF Model was read only through its README prose; its TSV files are data, and its C++, Python and Java tools were not opened.

**Method notes**
- About 32 searches and fetches were used. pdfa.org refuses plain fetches (HTTP 403); its pages were read through a reader proxy, so the quoted text is the page text.
- **Clause numbers.** ISO 32000-2:2020 text is copyrighted and was not read in full. Clause numbers marked High were confirmed from the headings of the PDF Association's errata pages ([pdf-issues.pdfa.org/32000-2-2020](https://pdf-issues.pdfa.org/32000-2-2020/)), which reproduce each corrected clause's number and title. A clause that has no erratum does not appear there; those numbers are inferred from neighbouring confirmed numbers or from the ISO 32000-1 layout and are graded Medium, and they are listed under "Claims not verified".
- The Adobe ISO 32000-1 public patent licence was read in full (a 1-page PDF, text extracted locally).

## 0. Clause map of ISO 32000-2:2020 used by Izcal

Top-level clauses with errata (and hence confirmed titles): 2 Normative references, 3 Terms, 4 Notation, **7 Syntax, 8 Graphics, 9 Text, 10 Rendering, 11 Transparency, 12 Interactive features, 13 Multimedia features, 14 Document interchange**; Annexes A (operator summary, informative), D (character sets and encodings), E (extending PDF), **F (Linearized PDF, normative)**, H (example files), **L (parent-child rules of the PDF 2.0 standard structure namespace)**, O (fragment identifiers), **Q (method for determining transparency on a page)**. Source: [errata index](https://pdf-issues.pdfa.org/32000-2-2020/), page modified 2026-03-13. High.

| Izcal area | Clauses (High unless marked) |
|---|---|
| Lexer, objects, streams | 7.2 lexical conventions (7.2.2, 7.2.4 comments); 7.3 objects (7.3.3 numbers, 7.3.4 strings, 7.3.7 dictionaries, 7.3.8 streams, 7.3.8.2 stream extent, 7.3.10 indirect objects) |
| Filters | 7.4 (7.4.3 ASCII85Decode, 7.4.4 LZWDecode and FlateDecode, 7.4.4.3 their parameters incl. predictors, 7.4.9 JPXDecode); 7.4.2, 7.4.5–7.4.8, 7.4.10: Medium |
| File structure | 7.5.2 header, 7.5.4 cross-reference table, 7.5.5 trailer, 7.5.6 incremental updates, 7.5.7 object streams, 7.5.8 cross-reference streams (7.5.8.2–7.5.8.4, hybrid-reference in 7.5.8.4); 7.5.3 body: Medium |
| Encryption | 7.6 (7.6.2, 7.6.3 general algorithm, 7.6.4 standard security handler, 7.6.4.3.2 Algorithm 2 (R≤4), 7.6.4.3.3 Algorithm 2.A and 7.6.4.3.4 Algorithm 2.B (R6), 7.6.4.4 password algorithms incl. Algorithm 10 and 13 for R6, 7.6.5 public-key handlers, 7.6.6 crypt filters, 7.6.7 unencrypted wrapper document) |
| Document structure | 7.7 (catalog 7.7.2, page tree 7.7.3, name dictionary 7.7.4); 7.8.2 content streams, 7.8.3 resources; 7.9 common data structures (text strings 7.9.2.2, dates 7.9.4, name and number trees 7.9.6–7.9.7); 7.10 functions (Type 2 7.10.3, Type 4 7.10.5); 7.11 file specifications and embedded files; 7.12 extensions dictionary |
| Graphics | 8.2 graphics objects; 8.4 graphics state (8.4.2 stack, 8.4.4 operators, 8.4.5 ExtGState); 8.5 paths; 8.6 colour (8.6.5.5 ICCBased, 8.6.5.8 rendering intents, 8.6.6 special spaces, 8.6.6.5 DeviceN, 8.6.8 operators); 8.7 patterns (8.7.3 tiling, 8.7.4 shading, 8.7.4.5 shading types, 8.7.4.5.8 Type 7); 8.9 images (8.9.5 dictionaries, 8.9.6 masks, 8.9.7 inline images); 8.10 form XObjects; 8.11 optional content |
| Text and fonts | 9.3 text state; 9.4 text objects and operators; 9.6.2 Type 1, 9.6.2.2 "Standard Type 1 fonts (standard 14 fonts) (PDF 1.0-1.7)"; 9.6.3 TrueType; 9.6.4 Type 3; 9.7 composite fonts (9.7.3 CIDSystemInfo, 9.7.4 CIDFonts, 9.7.5 CMaps, 9.7.5.2 predefined CMaps, 9.7.6 Type 0); 9.8 font descriptors; 9.10.3 ToUnicode CMaps |
| Rendering | 10.3 CIE-based to device; 10.4 device-space conversions; 10.6 halftones; 10.7 scan conversion (10.7.2 flatness) |
| Transparency | 11.3 compositing (11.3.5 blend mode); 11.4 groups (11.4.8 summary); 11.6 specifying transparency (11.6.5 soft masks, 11.6.6 group XObjects); Annex Q |
| Interactive | 12.2 viewer preferences; 12.3 navigation (destinations, outlines, collections); 12.4 page labels, articles, presentations; 12.5 annotations (12.5.5 appearance streams, 12.5.6 types); 12.6 actions (12.6.4.8 URI, 12.6.4.17 ECMAScript actions); 12.7 forms (12.7.3 AcroForm dictionary, 12.7.4 fields, 12.7.5 field types, 12.7.6 form actions, 12.7.8 FDF, 12.7.9 non-interactive forms); 12.8 digital signatures; 12.9 measurement; 12.10 geospatial; 12.11 document requirements |
| Multimedia | 13.2 multimedia; 13.3 sounds and 13.4 movies (deprecated); 13.5 alternate presentations; 13.6 3D artwork; 13.7 rich media |
| Interchange | 14.3 metadata; 14.4 file identifiers; 14.6 marked content; 14.7 logical structure (14.7.4 namespaces); 14.8 Tagged PDF; 14.9 repurposing and accessibility (14.9.2 language, 14.9.4 replacement text); 14.10 web capture; 14.11 prepress (14.11.5 output intents, 14.11.7 OPI); 14.12 document parts; 14.13 associated files |

**Implications:** Izcal's PDF spec can cite clause numbers with confidence for every area it owns; the handful of Medium numbers (filters other than ASCII85/LZW/Flate/JPX, colour-space sub-clauses, shading types 1–6) must be checked against the no-cost copy before the spec is frozen.

## 1. Access to the core specifications and their legal terms

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **ISO 32000-2 is available at no cost from the PDF Association since 5 April 2023**, sponsored by Adobe, Apryse and Foxit. Between 2017 and 2023 it was "published exclusively — and only for purchase — by ISO" | 2023-04-05 | [Sponsored standards](https://pdfa.org/sponsored-standards/); [announcement](https://pdfa.org/announcing-no-cost-access-to-iso-32000-2-pdf-2-0/) | 2023-04-05 | High |
| 1.2 | The no-cost bundle (updated 30 July 2024) holds ISO 32000-2:2020 with errata plus ISO/TS 32001, 32002, 32003 and 32004. The current resource page says the copy now includes **Errata Collection 3**; its cover image is dated June 2026 | 5 documents | [ISO 32000-2 resource page](https://pdfa.org/resource/iso-32000-2/) | 2026 | High |
| 1.3 | **The 2020 edition replaces the 2017 edition.** PDF 2.0 was first published in July 2017 after nine years' work by about 30 experts. "ISO holds the copyright to the PDF specification" | 2017-07; 2020 | same | — | High |
| 1.4 | **The licence terms of the no-cost copy were not found.** No page read states whether it is single-user, whether it may be redistributed, or whether text may be quoted. The errata site gives only "minimal surrounding text … due to ISO copyright" | unknown | [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2026 | High (that the terms were not found) |
| 1.5 | The PDF/UA bundle (ISO 14289-1:2014, ISO 14289-2:2024, ISO/TS 32005:2023) is also no-cost, sponsored by Allyant, axes4, GrackleDocs and TargetStream (updated 2 September 2025) | 3 documents | [Sponsored standards](https://pdfa.org/sponsored-standards/) | 2025-09-02 | High |
| 1.6 | **ISO 32000-1:2008 (PDF 1.7)** is "technically identical" to Adobe's PDF Reference 1.7 (November 2006) and, "by special arrangement with ISO", is available at no charge from adobe.com | 2008-07 | [PDF Association: ISO 32000-1 summary](https://pdfa.org/resource/iso-32000-1-pdf-1-7/) | 2008 | High |
| 1.7 | **Adobe's public patent licence covers ISO 32000-1:2008 only.** It grants "every individual and organization in the world the royalty-free right, under all Essential Claims that Adobe owns, to make, have made, use, sell, import and distribute Compliant Implementations", revocable against a licensee who sues over an Essential Claim. It states that "any updated version of ISO 32000-1 and any specification that incorporates ISO 32000-1:2008 by reference is not the Specification" | 1 standard | [Adobe ISO 32000-1 Public Patent License](https://www.adobe.com/pdf/pdfs/ISO32000-1PublicPatentLicense.pdf) | 2008-12-20 (file date) | High |
| 1.8 | No equivalent public patent licence for ISO 32000-2 was found. ISO's patent declaration database for ISO 32000-2 was not checked | unknown | — | — | Low (absence) |
| 1.9 | **Errata process.** The PDF Association's public GitHub repository "pdf-issues" takes errata reports for PDF 2.0, its TS extensions, PDF/A-4, PDF/X-6, PDF/VT-3, PDF/UA, WTPDF, ECMAScript for PDF and XMP. Resolutions are published by clause at pdf-issues.pdfa.org, marked as ISO-approved, submitted to ISO, or industry-only. Errata on ISO 32000-1 are **not** handled | — | [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2026 | High |
| 1.10 | Errata Collection 3 covers resolutions up to 1 June 2026: 766 edits, 655 (86%) approved by ISO TC 171 SC 2 WG 8, up from 54% in EC2 | 766; 86% | [PDF Association: EC3 now available](https://pdfa.org/pdf-2-0-errata-collection-3-now-available/) (search snippet) | 2026 | Medium |
| 1.11 | Participation in PDF standardisation is open to PDF Association members through its Category A liaison with ISO TC 171 SC 2 | — | [ISO 32000-2 resource page](https://pdfa.org/resource/iso-32000-2/) | 2026 | High |
| 1.12 | The PDF Association publishes PDF 2.0 Application Notes (001 black-point compensation 2018-09-17; 002 associated files 2018-10-26; 003 object metadata streams 2021-03-16) and a small set of PDF 2.0 sample files on GitHub | 3 notes | same | 2026 | High |

**Implications:** Izcal anchors to **ISO 32000-2:2020 with Errata Collection 3** and reads ISO 32000-1 only for legacy behaviour. Until the no-cost copy's licence is read, the spec must cite clause numbers and paraphrase, never quote clause text. Adobe's patent grant does not reach PDF 2.0 features, so the patent position of PDF 2.0-only features (AES-256 R6, new signature profiles, associated files) is an open question for the owner's royalty-free rule.

## 2. File structure and syntax (clause 7) and the lenient-parsing reality

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | File structure is clause 7.5: header 7.5.2, cross-reference table 7.5.4, trailer 7.5.5, **incremental updates 7.5.6, object streams 7.5.7, cross-reference streams 7.5.8** with dictionary (7.5.8.2), data (7.5.8.3), and compatibility for readers that lack compressed references (7.5.8.4, which holds the hybrid-reference trailer's `XRefStm` entry, Table 19) | — | [Errata clause 7](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html) | 2026 | High |
| 2.2 | Corrected tables show where readers diverge: Table 3 (escape sequences in literal strings), Table 5 (stream dictionary entries), Table 8 (LZW and Flate parameters), Table 15 (trailer), Tables 17–19 (xref streams, hybrid trailer) all have industry-approved corrections | 7 tables | [Errata index](https://pdf-issues.pdfa.org/32000-2-2020/) | 2026-03-13 | High |
| 2.3 | **Linearization is normative Annex F.** Errata cover F.3.2 (header; linearization applies to any PDF 1.2 or later), F.3.3 (linearization parameter dictionary, Table F.1) and F.3.6 (hint streams, parts 5 and 10) | — | [Errata Annex F](https://pdf-issues.pdfa.org/32000-2-2020/clauseAnnexF.html) | 2026 | High |
| 2.4 | The extensions dictionary (7.12: BaseVersion, ExtensionLevel, URL) is how files declare Adobe or ISO extensions; Adobe extended PDF 1.7 up to ExtensionLevel 11 by 2017 | 11 | [Errata clause 7](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html); [PDF Association corpus post](https://pdfa.org/new-large-scale-pdf-corpus-now-publicly-available/) | 2023 | High |
| 2.5 | **Lenient parsing is the norm, and parsers disagree.** Galois ran 20 parsers over **1,040,628 PDFs** in about 13 days (96 cores), found 320,717 distinct binary features on a 10,000-file subset, and built rules matching a SafeDocs validity gold standard with 0% false positives and 0.5% false negatives. Root causes found include mislabelled non-embedded fonts and "file is damaged" warnings | 1.04 M files; 20 parsers | [Menon, Woods: Corpus-wide analysis of parser behaviors (LangSec 2023)](https://langsec.org/spw23/papers/Woods_LangSec23.pdf) | 2023 | High |
| 2.6 | The SafeDocs "UNSAFE" corpus (CC-MAIN-2021-31-UNSAFE, over 5 million files incl. about 3.9 million Common Crawl PDFs) is described as containing files that crash or hang parsers and that parse differently across parsers or options | > 5 M | [Digital Corpora: UNSAFE-DOCS](https://digitalcorpora.org/?p=3002) (search snippet) | 2023 | Medium |
| 2.7 | In a 1,000-file sample of the 2021 crawl, pdfinfo reported failures such as "Couldn't find trailer dictionary" and password errors; 4 of 1,045 rows had no readable version. PDF versions in the sample: 1.3 85, 1.4 290, 1.5 262, 1.6 113, 1.7 280 rows; **no PDF 2.0** | 1,045 rows | [PDF Association corpus post](https://pdfa.org/new-large-scale-pdf-corpus-now-publicly-available/) | 2023 | High |
| 2.8 | A share of real-world files that need xref repair (wrong offsets, missing or broken tables) was **not found** in any source read | unknown | — | — | — |
| 2.9 | Acrobat historically accepted a header anywhere in the first 1,024 bytes and other deviations, documented in the PDF Reference 1.7's implementation notes (Appendix H) | 1,024 bytes | PDF Reference 1.7, Appendix H (recollection, not re-read) | 2006 | Low |

**Implications:** Izcal needs two parsers in one design: a strict ISO 32000-2 parser that defines "valid", and a separately specified, bounded **repair mode** whose every recovery is named, counted and testable (no silent guessing). The repair rules must come from black-box measurement on the corpora of Section 13, since no published rate or rule set was found.

## 3. Filters (7.4) and the codec standards behind them

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | Filter clauses: 7.4.3 ASCII85Decode, 7.4.4 LZWDecode and FlateDecode (7.4.4.3 parameters: Predictor, Colors, BitsPerComponent, Columns, EarlyChange), 7.4.9 JPXDecode | — | [Errata clause 7](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html) | 2026 | High |
| 3.2 | By the clause order: 7.4.2 ASCIIHexDecode, 7.4.5 RunLengthDecode, 7.4.6 CCITTFaxDecode, 7.4.7 JBIG2Decode, 7.4.8 DCTDecode, 7.4.10 Crypt | — | inferred from 3.1 and ISO 32000-1 layout | — | Medium |
| 3.3 | Codec standards: **Flate** = zlib/deflate, RFC 1950 and RFC 1951 (1996); **DCT** = ITU-T T.81 (1992); **JPX** = JPEG 2000 Part 1, ITU-T T.800 / ISO/IEC 15444-1; **JBIG2** = ITU-T T.88 (2000; current edition 08/2018) / ISO/IEC 14492; **CCITT** = ITU-T T.4 (Group 3) and T.6 (Group 4) | — | [ITU-T T.88 page](https://www.itu.int/ITU-T/recommendations/rec.aspx?rec=13688); [JPEG 2000 overview](https://jpeg.org/jpeg2000/) | 2018; 2026 | Medium |
| 3.4 | **LZW patents are expired**: Unisys US 4,558,302 expired 20 June 2003; UK, French, German and Italian counterparts 18 June 2004; Japanese 20 June 2004; Canadian 7 July 2004. IBM's related US patent expired in 2006 (sources differ: 11 August or 1 October) | 2003–2006 | [endsoftwarepatents wiki (quoting Unisys)](https://wiki.endsoftwarepatents.org/wiki/LZW_algorithm_patent); [GNU: Why there are no GIF files](https://www.gnu.org/philosophy/gif); images note [formats_codecs 10.3](../Izcal%20images%20and%20media/formats_codecs.md) | 2003–2006 | Medium |
| 3.5 | **JPEG 2000**: "The JPEG 2000 standard is a collaboration between ITU-T and ISO/IEC and has a royalty-free objective." The page states no Part 1 patent terms. A 2001 paper says Part 1 was meant to be usable without royalties or fees; Michael Adams noted the effort may not have fully succeeded | royalty-free objective | [jpeg.org JPEG 2000](https://jpeg.org/jpeg2000/); [Adams, JPEG 2000 still image compression standard](https://www.ece.uvic.ca/~frodo/publications/jpeg2000.pdf) | 2026; 2001 | Medium |
| 3.6 | **JBIG2**: patents were held by IBM and Mitsubishi. A secondary source (Kuhn) says IBM agreed to license its arithmetic-coding patents free of charge to JBIG2 implementers, as recorded in an annex of the final draft. The ITU T.88 pages link "patent statement(s)" that were not read. JBIG2 dates from 2000, so patents filed before it are likely expired by 2026, but no expiry list was found | 2000 | [Kuhn: JBIG-KIT patents](https://www.cl.cam.ac.uk/~mgk25/jbigkit/patents/); [Wikipedia: JBIG2](https://en.wikipedia.org/wiki/JBIG2) | 2013 | Low |
| 3.7 | CCITT T.4 (1980) and T.6 (1984) and JPEG baseline T.81 (1992) are old enough that any essential patents have expired (20-year terms) | > 30 years | inference from publication dates | — | Medium |
| 3.8 | Izcal's images and media spec already assigns `hg-jpx`, `hg-jbig2` and `hg-ccitt` (built by the media team, run in the PDF process under G50–G53) and records FORCEDENTRY (CVE-2021-30860) as a JBIG2 integer-overflow exploit delivered in a PDF | 3 crates | [Izcal images and media 6.x, Table of codecs](../../reports/Izcal%20images%20and%20media.md); [Project Zero](https://projectzero.google/2021/12/a-deep-dive-into-nso-zero-click.html) | 2021-12-15; 2026 | High |

**Implications:** Every PDF filter can be built royalty-free on today's evidence, but the JBIG2 and JPEG 2000 rows rest on secondary or "objective" statements. Before the spec is frozen, the ITU-T patent statements for T.88 and T.800 must be read and recorded in the patent table that MD-D3 established.

## 4. Encryption (7.6) and its extensions

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **"Use of security handler revisions 1, 2, 3, 4 and 5 is deprecated in PDF 2.0."** Only revision 6 is current | R1–R5 deprecated | [Errata clause 7, 7.6.4](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html) | 2026 | High |
| 4.2 | R6 algorithms: 7.6.4.3.3 Algorithm 2.A (retrieve the file key, R6 and later), 7.6.4.3.4 **Algorithm 2.B (computing a hash, R6 and later)**, 7.6.4.4.9 Algorithm 10 (Perms value), 7.6.4.4.12 Algorithm 13 (validate permissions) | 4 algorithms | same | 2026 | High |
| 4.3 | R2–R4 use RC4 (R4 may use AES-128 via crypt filters, PDF 1.6); R6 uses AES-256, and Algorithm 2.B iterates SHA-256, SHA-384 and SHA-512 with AES-128 rounds. R5 was an Adobe extension (ExtensionLevel 3) superseded by R6 | — | recollection of ISO 32000-2 7.6.4; titles confirmed in 4.2 | — | Medium |
| 4.4 | The errata keep a note that the RC4 name is an RSA trademark; AES is defined by FIPS 197 and was introduced in PDF 1.6 | — | [Errata clause 7, 7.6.3.1](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html) | 2026 | High |
| 4.5 | Public-key security handlers are 7.6.5 (dictionary 7.6.5.2, permissions 7.6.5.3, algorithms 7.6.5.4); crypt filters 7.6.6; the **unencrypted wrapper document** 7.6.7 (PDF 2.0) | — | same | 2026 | High |
| 4.6 | **ISO/TS 32003:2023 adds AES-GCM** (128-, 192- and 256-bit keys) to the Encrypt dictionary, giving authenticated encryption | 3 key sizes | [PDF Association ISO/TS 32003 summary](https://pdfa.org/?p=126660) (search snippet) | 2023 | Medium |
| 4.7 | **ISO/TS 32004 "Integrity protection in encrypted documents in PDF 2.0"** (errata site dates it 2023; one listing says 2024) | — | [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2023 | High (title); Medium (date) |
| 4.8 | The FDF encryption mechanism (12.7.8) is separate and uses 40-bit RC4 keyed by MD5 of a padded password, revision 1 only | 40-bit | [Errata clause 12](https://pdf-issues.pdfa.org/32000-2-2020/clause12.html) | 2026 | High |

**Implications:** The viewer must decrypt R2–R6 and AES-GCM to open real files, but the writer emits only **R6 (AES-256)**, or AES-GCM once its readership is measured. RC4 and MD5 exist only on the decode path, inside the PDF process.

## 5. Digital signatures (12.8), PAdES and hash extensions

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | 12.8 Digital signatures: 12.8.2 transform methods (12.8.2.2 **DocMDP** incl. 12.8.2.2.2 validation; 12.8.2.4 FieldMDP); 12.8.3 interoperability (12.8.3.4.4 profiles of ETSI.CAdES.detached); 12.8.4 **long-term validation** (12.8.4.3 **Document Security Store (DSS)**, 12.8.4.4 VRI); 12.8.5 document timestamps; 12.8.6 permissions | — | [Errata clause 12](https://pdf-issues.pdfa.org/32000-2-2020/clause12.html) | 2026 | High |
| 5.2 | The signed ByteRange runs from the `%PDF-` comment to the end of the `%%EOF` of the incremental update that holds the signature (excluding Contents), so each signature covers one revision | — | same | 2026 | High |
| 5.3 | **ETSI EN 319 142-1 V1.2.1 (2024-01)**, "PAdES digital signatures; Part 1: Building blocks and PAdES baseline signatures", defines four levels B-B, B-T, B-LT, B-LTA, each including the lower | 4 levels | [EVS listing](https://evs.ee/en/evs-en-319-142-1-v1-2-1-2024) (search snippet); level detail from [PDF Tools](https://www.pdf-tools.com/pdf-knowledge/pades-pdf-advanced-electronic-signature/) | 2024-02-01 | Medium |
| 5.4 | **ISO/TS 32001:2022** adds SHA3-256, SHA3-384, SHA3-512 and SHAKE256 for signatures; **ISO/TS 32002:2022** adds ECDSA on NIST and Brainpool curves and EdDSA (Ed25519, Ed448) | 4 hashes; 2 EdDSA curves | [PDF Association ISO/TS 32001](https://pdfa.org/resource/iso-ts-32001/) (search snippet); [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2022 | Medium |

**Implications:** Signature *display* (who signed, which revision, whether later revisions changed the document within DocMDP) is a viewer requirement because the revision chain is visible structure. Full PAdES validation needs a trust store, revocation fetches and a timestamp authority model; it is a distinct component with its own privacy question (revocation fetches leak which document is open).

## 6. Content streams, graphics, colour, shading and transparency (clauses 8, 10, 11, 14.11)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Operators are summarised in informative Annex A (Table A.1, corrected); graphics-state operators in Table 56; ExtGState in Table 57; device-dependent state in Table 52 | — | [Errata index](https://pdf-issues.pdfa.org/32000-2-2020/) | 2026 | High |
| 6.2 | Colour: 8.6.5.5 ICCBased (Table 66 corrected to "ICC profile versions supported by ICCBased colour spaces"), 8.6.5.8 rendering intents, 8.6.6 special colour spaces with 8.6.6.5 DeviceN, 8.6.8 colour operators | — | [Errata clause 8](https://pdf-issues.pdfa.org/32000-2-2020/clause08.html) | 2026 | High |
| 6.3 | Remaining colour sub-clauses by layout: 8.6.4 device spaces (DeviceGray, DeviceRGB, DeviceCMYK); 8.6.5 CIE-based (CalGray, CalRGB, Lab, ICCBased); 8.6.6.2 Pattern, 8.6.6.3 Indexed, 8.6.6.4 Separation | — | inferred from 6.2 and ISO 32000-1 | — | Medium |
| 6.4 | Shadings: 8.7.4.5 shading types, with **Type 7 (tensor-product patch mesh) at 8.7.4.5.8**; by order Types 1–6 are 8.7.4.5.2–8.7.4.5.7 (function-based, axial, radial, free-form Gouraud, lattice Gouraud, Coons patch) | 7 types | [Errata clause 8](https://pdf-issues.pdfa.org/32000-2-2020/clause08.html) | 2026 | High (Type 7); Medium (others) |
| 6.5 | Images: 8.9.5 image dictionaries (8.9.5.4 alternate images), 8.9.6 masked images (8.9.6.2 stencil masking), 8.9.7 inline images; form XObjects 8.10; **optional content 8.11** (configuration and usage dictionaries corrected) | — | same | 2026 | High |
| 6.6 | Rendering clause 10 covers CIE-to-device conversion (10.3), device-space conversions incl. Gray↔RGB and Gray↔CMYK (10.4.2.2–10.4.2.3), halftones (10.6, Types 5 and 6 corrected) and flatness (10.7.2) | — | [Errata clause 10](https://pdf-issues.pdfa.org/32000-2-2020/clause10.html) | 2026 | High |
| 6.7 | Transparency clause 11: blend modes 11.3.5; groups 11.4 (isolated and knockout semantics, summary 11.4.8); soft masks 11.6.5 (soft-mask images 11.6.5.2, Table 143); group XObjects 11.6.6 (Table 145). **Annex Q** gives the normative method for deciding whether a page uses transparency | — | [Errata clause 11](https://pdf-issues.pdfa.org/32000-2-2020/clause11.html) | 2026 | High |
| 6.8 | **Output intents** are 14.11.5 (DestOutputProfileRef, Table 402); PDF 2.0 allows page-level output intents | — | [Errata clause 14](https://pdf-issues.pdfa.org/32000-2-2020/clause14.html) | 2026 | High (clause); Medium (page-level) |
| 6.9 | Functions Types 2 and 4 (PostScript calculator, 7.10.5, Table 42 operators corrected) are needed by shadings, separations and transfer functions | — | [Errata clause 7](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html) | 2026 | High |
| 6.10 | PDF 2.0 Application Note 001 defines black-point compensation behaviour | — | [ISO 32000-2 resource page](https://pdfa.org/resource/iso-32000-2/) | 2018-09-17 | High |

**Implications:** PDF's imaging model (non-separable blend modes on CMYK, knockout groups, DeviceN with overprint, Type 4 functions, patch meshes) is a superset of what CSS and Izcal's display list carry. The spec must either extend the display list with these primitives or render PDF pages to a PDF-specific layer; Annex Q is the cheap test for "does this page need the transparency path".

## 7. Text and fonts (clause 9)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | Simple fonts: Type 1 9.6.2, TrueType 9.6.3, Type 3 9.6.4; composite fonts 9.7 (CIDSystemInfo 9.7.3, CIDFonts 9.7.4, CMaps 9.7.5, predefined CMaps 9.7.5.2, Type 0 9.7.6); font descriptors 9.8; ToUnicode 9.10.3 (Table 125a added for ToUnicode stream dictionaries) | — | [Errata clause 9](https://pdf-issues.pdfa.org/32000-2-2020/clause09.html) | 2026 | High |
| 7.2 | The standard-14 clause is titled **"Standard Type 1 fonts (standard 14 fonts) (PDF 1.0-1.7)"**: the special status of the 14 fonts is a PDF 1.x feature; PDF 2.0 writers are expected to embed | 14 fonts | same | 2026 | High (title); Medium (reading) |
| 7.3 | A PDF processor "shall support Adobe-CNS1-7, Adobe-GB1-5, Adobe-Japan1-7 and Adobe-KR-9"; Adobe-Japan2-0 and Adobe-Korea1-2 are deprecated (2020) | 4 collections | same | 2026 | High |
| 7.4 | CFF and OpenType font programs are embedded through FontFile3 subtypes (Type1C, CIDFontType0C, OpenType); CIDFontType0 carries CFF CID-keyed outlines and CIDFontType2 carries TrueType | — | recollection of 9.9 (embedded font programs) | — | Medium |
| 7.5 | Mislabelled non-embedded fonts were a measured root cause of rendering differentials across parsers | — | [Menon, Woods (LangSec 2023)](https://langsec.org/spw23/papers/Woods_LangSec23.pdf) | 2023 | High |

**Implications:** The viewer needs Type 1 and Type 3 (PDF only, as the fonts spec TX already ruled), CFF, TrueType and OpenType through Izcal's own font stack, the four predefined CJK collections' CMaps as data, and a defined substitution for non-embedded fonts. The writer embeds and subsets every font, so it never relies on the standard 14.

## 8. Annotations, forms, actions, JavaScript and XFA (clause 12)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | Annotations 12.5: dictionaries 12.5.2, **appearance streams 12.5.5**, types 12.5.6 (markup, link, free text, line, square/circle, polygon, caret, ink, movie, screen, projection all have errata) | — | [Errata clause 12](https://pdf-issues.pdfa.org/32000-2-2020/clause12.html) | 2026 | High |
| 8.2 | Forms 12.7: interactive form dictionary 12.7.3, fields 12.7.4 (variable text 12.7.4.3), types 12.7.5 (buttons, text, choice, signature), actions 12.7.6 (submit, reset), FDF 12.7.8, non-interactive forms 12.7.9 | — | same | 2026 | High |
| 8.3 | Actions 12.6.4: Go-To, embedded Go-To, **Launch** (12.6.4.6; Table 208 "Microsoft Windows launch parameter dictionary"), **URI** 12.6.4.8, **ECMAScript actions 12.6.4.17**, Rich-Media-Execute 12.6.4.18; the document-level JavaScript name tree maps to 12.6.4.17 | — | same; [Errata clause 7, 7.7.4](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html) | 2026 | High |
| 8.4 | **ECMAScript for PDF 2.0 is a separate standard, ISO 21757-1:2020**, with its own API clause (10) and errata | — | [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2020 | High |
| 8.5 | **XFA was not carried into PDF 2.0**; several vendors state PDF 2.0 readers ignore XFA. No ISO text was read for this, and the clause that handles it was not confirmed | — | [Datalogics: XFA deprecation](https://www.datalogics.com/xfa-form-deprecation-what-it-means-and-what-to-do); [Mapsoft](https://mapsoft.com/posts/pdf-format-spec.html) | — | Low |

**Implications:** AcroForm with appearance-stream generation (12.7.4.3 variable text) is in scope for a viewer that lets users fill forms. JavaScript, Launch and XFA are decisions for the spec, not parsing questions: each has a separate standard or none, and each enlarges the attack surface.

## 9. Logical structure, Tagged PDF, metadata and associated files (clause 14)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | Marked content 14.6; **logical structure 14.7** (hierarchy 14.7.2, types 14.7.3, **namespaces 14.7.4**, content items 14.7.5, attributes 14.7.6); **Tagged PDF 14.8** (real content vs. artifacts 14.8.2.2, layout model 14.8.3, standard structure types 14.8.4 incl. ruby and warichu, tables, captions, formulas; attributes 14.8.5; standard namespaces and role maps 14.8.6) | — | [Errata clause 14](https://pdf-issues.pdfa.org/32000-2-2020/clause14.html) | 2026 | High |
| 9.2 | Annex L gives normative parent-child rules for the PDF 2.0 structure namespace; **ISO/TS 32005:2023** gives the inclusion rules when PDF 1.7 and PDF 2.0 namespaces are mixed | — | [Errata index](https://pdf-issues.pdfa.org/32000-2-2020/) | 2023 | High |
| 9.3 | Accessibility support 14.9: natural language 14.9.2, replacement text 14.9.4 | — | [Errata clause 14](https://pdf-issues.pdfa.org/32000-2-2020/clause14.html) | 2026 | High |
| 9.4 | Metadata: XMP metadata streams 14.3.2 and the document information dictionary 14.3.3; file identifiers 14.4. XMP itself is ISO 16684-1:2019 | — | same; [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2026 | High |
| 9.5 | **Associated files 14.13** (incl. 14.13.5 linked to graphics objects); Application Note 002 explains them | — | same; [ISO 32000-2 resource page](https://pdfa.org/resource/iso-32000-2/) | 2018 | High |
| 9.6 | Document parts (14.12, DPart) and web capture (14.10) also live in clause 14 | — | same | 2026 | High |

**Implications:** Izcal's viewer maps the structure tree (14.7–14.8) into its accessibility tree; its writer emits a PDF 2.0 structure tree in the PDF 2.0 namespace (Annex L) generated from the DOM, which is where a browser has an advantage no PDF engine has.

## 10. Multimedia, 3D and rich media (clause 13): what to drop

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | 13.3 Sounds and 13.4 Movies "describe deprecated features superseded by 13.7, Rich media" | 2 deprecated | [Errata clause 13](https://pdf-issues.pdfa.org/32000-2-2020/clause13.html) | 2026 | High |
| 10.2 | 3D artwork (13.6) embeds **U3D** or **PRC** streams (PRC is ISO 14739-1:2014); rich media (13.7.2) can carry scripts via a `Scripts` array of JavaScript files in the Assets name tree | 2 formats | same; [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2026 | High |
| 10.3 | New TS extensions add rich-media annotations for STEP AP 242 (ISO/TS 24064:2023) and **glTF** (ISO/TS 32007:2024) | 2 TS | [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2024 | High |
| 10.4 | The PDF Association has asked publicly whether "3D PDF can thrive without JavaScript" | — | [PDF Association front page link](https://pdfa.org/announcing-no-cost-access-to-iso-32000-2-pdf-2-0/) (headline only) | 2026 | Low |

**Implications:** Drop U3D, PRC, STEP, Flash-era rich media and deprecated sounds and movies from the viewer: show the poster appearance stream and an "unsupported content" notice. glTF rich media is the one 3D path that maps onto web technology already in Izcal; it is a later, measured decision.

## 11. Related standards: PDF/A, PDF/UA, WTPDF, PDF/X, print

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 11.1 | **PDF/A** (ISO 19005): PDF/A-1 2005, PDF/A-2 2011, PDF/A-3 2012, **PDF/A-4 2020** (based on PDF 2.0, with errata on its clause 6) | 4 parts | [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2020 | High |
| 11.2 | **PDF/UA** (ISO 14289): PDF/UA-1:2014 (PDF 1.7), **PDF/UA-2:2024** (PDF 2.0); both now no-cost | 2 parts | [Sponsored standards](https://pdfa.org/sponsored-standards/) | 2024 | High |
| 11.3 | **WTPDF 1.0** ("Using Tagged PDF for Accessibility and Reuse in PDF 2.0") was published 29 February 2024; PDF/UA-2 is the ISO standardisation of its accessibility conformance level, published about mid-March 2024 (13 or 15 March by different listings) | 2024-02-29 | [Sponsored standards](https://pdfa.org/sponsored-standards/); [axes4](https://www.axes4.com/en/blog/post/2024/pdf-ua-2-vs-well-tagged-pdf) | 2024 | High (WTPDF link); Medium (dates) |
| 11.4 | **PDF/X** (ISO 15930): X-3 2003, X-4 and X-5 2010, **X-6 2020** (PDF 2.0); **PDF/VT-3** ISO 16612-3:2020; **PDF/R-1** ISO 23504-1:2020 | — | [pdf-issues overview](https://pdf-issues.pdfa.org/) | 2020 | High |
| 11.5 | **CSS Paged Media Level 3** is a W3C Working Draft (latest 14 September 2023); **CSS GCPM Level 3** is a Working Draft (January or May 2024). Neither has reached CR | 2 WDs | [W3C css-page-3](https://www.w3.org/TR/css-page/); [W3C css-gcpm-3](https://www.w3.org/TR/css-gcpm-3/) | 2023; 2024 | Medium |

**Implications:** The writer's natural targets are **PDF 2.0 + WTPDF/PDF/UA-2** (a DOM-derived structure tree) and, optionally, **PDF/A-4**, which is PDF 2.0-based and so needs no second writer. PDF/X is a print-shop format and out of scope. Paged layout comes from the layout spec's CSS Paged Media work, which is still a draft standard.

## 12. Arlington PDF Model

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 12.1 | A machine-readable definition of every PDF object, key, type, version and constraint, in TSV files (one object per file, one key per row, with predicates), based on ISO 32000-2:2020 plus resolved errata; earlier-version sets (PDF 1.0–1.7) are derived by version predicates | 1.0–2.0 | [GitHub: arlington-pdf-model README](https://github.com/pdf-association/arlington-pdf-model) | 2026 | High |
| 12.2 | **Licence: Apache-2.0** (repository licence). Developed under DARPA SafeDocs; the README says it is "not a replacement for the standard" | Apache-2.0 | same; [PDF Association: Arlington](https://pdfa.org/resource/arlington-pdf-model/) | 2026 | High |
| 12.3 | Checkers built on it: veraPDF, PDFix, BFO, JPedal; a C++ TestGrammar tool compares it with Adobe's DVA | 4 | same | 2026 | High |

**Implications:** Apache-2.0 is not MIT, so Arlington **cannot ship** inside Izcal unless the owner rules it "standards data from a standards body" (the PDF Association is an ISO liaison, not a standards body). It is fully usable **test-only**: Izcal can generate parser conformance tests and a validator oracle from it. Izcal's own object model must be written from the ISO text; whether a build-time code generator may read Arlington TSVs to produce MIT code is an owner decision.

## 13. Black-box test corpora

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 13.1 | **CC-MAIN-2021-31-PDF-UNTRUNCATED**: 7.9 million unique PDFs (8.3 million URLs), about 8 TB, from Common Crawl's July–August 2021 crawl, refetched in full by NASA JPL (Common Crawl truncates at 1 MB; 25.6% of PDFs in CC-MAIN-2023-06 were truncated). 7,933 ZIPs; provenance and pdfinfo metadata. **No single licence**: users must follow Common Crawl's terms and Digital Corpora's Terms of Use. Not sanitised | 7.9 M; 8 TB | [PDF Association](https://pdfa.org/new-large-scale-pdf-corpus-now-publicly-available/) | 2023 | High |
| 13.2 | The SafeDocs **Stressful PDF Corpus** mines open-source PDF projects' issue trackers for malformed files; it "does not accurately represent PDF files in general" | — | same | 2023 | High |
| 13.3 | **GovDocs1**: nearly 1 million files from .gov web servers (2009), described as freely redistributable "to the best of our knowledge"; no formal licence text found | ~1 M | [Digital Corpora: GovDocs1](https://digitalcorpora.org/?p=94) (search snippet) | 2009 | Medium |
| 13.4 | **veraPDF corpus**: CC BY 4.0. Covers PDF/A-1a/b, 2a/b/u, 3a/b/u, 4, 4e, 4f, PDF/UA-1 and UA-2, plus ISO 32000-1 and 32000-2 files; complements the Isartor and BFO PDF/A-2 suites. The veraPDF software is MPL-2.0+/GPL-3.0+ | CC BY 4.0 | [GitHub: veraPDF-corpus](https://github.com/veraPDF/veraPDF-corpus) | 2026 | High |
| 13.5 | **Isartor** (PDF/A-1b negative test suite, PDF Association): licence not found | — | — | — | Low |
| 13.6 | PDF Association **PDF 2.0 sample files** on GitHub (one feature per file); licence not checked | — | [ISO 32000-2 resource page](https://pdfa.org/resource/iso-32000-2/) | 2026 | Medium |

**Implications:** All of these are test-only inputs, which the owner's rule allows under any licence; none ships. The 2021 crawl sample has no PDF 2.0 files, so PDF 2.0 coverage must come from the samples, veraPDF and Arlington-generated cases. Black-box oracles (other viewers run as programs) plus these corpora replace any reading of engine code.

## 14. Synthesis: what Izcal anchors to and what it drops

| Area | Anchor | Viewer | Writer |
|---|---|---|---|
| Core | ISO 32000-2:2020 + EC3; ISO 32000-1 for legacy | Read 1.0–2.0 | Emit PDF 2.0 |
| Syntax | 7.2–7.5, Annex F | Strict parser plus specified repair mode | Xref streams, object streams, optional linearization |
| Filters | 7.4; RFC 1950/1951, T.81, T.800, T.88, T.4/T.6 | All, in the PDF process | Flate (+ predictors), DCT pass-through |
| Encryption | 7.6; TS 32003, 32004 | R2–R6, public key, AES-GCM | R6 AES-256 only (if asked) |
| Signatures | 12.8; EN 319 142-1; TS 32001, 32002 | Show, check integrity and DocMDP; full validation is a decision | None at first (decision) |
| Graphics | 8, 10, 11, Annex Q, 14.11.5 | Full | Subset produced by the display list |
| Fonts | 9; CJK collections | All types | Embedded, subsetted CFF/TrueType |
| Forms | 12.5, 12.7 | AcroForm with appearance generation | Static (non-interactive) |
| JavaScript | 12.6.4.17; ISO 21757-1 | Decision (most likely off) | Never |
| XFA | Not in PDF 2.0 | Drop | Never |
| Multimedia, 3D | 13 | Drop (poster only); glTF later | Never |
| Structure | 14.7–14.8, Annex L, TS 32005 | Into the accessibility tree | WTPDF / PDF/UA-2 from the DOM |
| Archival | ISO 19005-4 | — | PDF/A-4 as an option |

## Open questions and black-box measurements needed

1. The licence terms of the no-cost ISO 32000-2 and PDF/UA bundles (quotation, redistribution, use in derived data such as tests).
2. The patent position of PDF 2.0 itself: ISO's patent declarations for ISO 32000-2, since Adobe's public licence (1.7) excludes later versions.
3. ITU-T patent statements for T.88 (JBIG2) and T.800 (JPEG 2000 Part 1): record holders and terms in the MD-D3 patent table.
4. Whether Arlington's Apache-2.0 TSVs may feed a build-time generator for MIT code, or are test-only (owner decision).
5. The share of real-world PDFs that need xref repair, and which repairs: measure with Izcal's strict parser on CC-MAIN-2021-31-PDF-UNTRUNCATED, with other viewers as black-box oracles.
6. Feature prevalence in the 2021 crawl: encryption revisions, JBIG2, JPX, CCITT, Type 3, XFA, JavaScript, 3D, signatures, Tagged PDF (pdfinfo metadata gives some of this).
7. The clause in ISO 32000-2 that states XFA's status.
8. Isartor and PDF 2.0 sample-file licences.

## Claims not verified

- 0 and 3.2: filter clauses 7.4.2, 7.4.5–7.4.8, 7.4.10; 7.5.3 body; 6.3 colour sub-clauses 8.6.4, 8.6.5, 8.6.6.2–8.6.6.4; 6.4 shading Types 1–6 sub-clause numbers (all inferred from neighbours and the ISO 32000-1 layout).
- 1.8 absence of a PDF 2.0 public patent licence (not searched exhaustively); 1.10 EC3 counts (search snippet).
- 2.6 UNSAFE corpus size; 2.9 the 1,024-byte header tolerance (recollection of PDF Reference 1.7 Appendix H).
- 3.3 standard edition years; 3.4 LZW dates (quoted copies of Unisys's page); 3.5 JPEG 2000 Part 1 royalty status; 3.6 JBIG2 patent holders and IBM's free licence (secondary, 2013); 3.7 expiry by age (inference).
- 4.3 algorithm contents of R2–R6 and R5's status (recollection); 4.6 AES-GCM key sizes and 4.7 TS 32004 date (snippets).
- 5.3 PAdES level contents (vendor summaries); 5.4 TS 32001/32002 algorithm lists (snippets).
- 6.8 page-level output intents in PDF 2.0.
- 7.2 reading of the standard-14 "(PDF 1.0-1.7)" qualifier; 7.4 FontFile3 subtypes (recollection).
- 8.5 XFA status in PDF 2.0 (vendor sources only).
- 10.4 the PDF Association 3D/JavaScript article (headline only).
- 11.3 PDF/UA-2 exact date; 11.5 GCPM's latest WD date.
- 13.3 GovDocs1 terms; 13.5 Isartor licence; 13.6 sample-file licence.
