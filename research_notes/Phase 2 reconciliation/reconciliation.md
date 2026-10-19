# Phase 2 reconciliation: parsing and style, fonts and text, networking

Compiled 19 October 2026 by the reconciliation reviewer, who wrote none of the three specs. Method: phase 8 of the first-principles-spec skill (`.claude/skills/first-principles-spec/SKILL.md`), following the precedent of the JavaScript engine and resource-footprint reconciliation (R1–R20 in the [footprint critique](../Izcal%20resource%20footprint/critique.md) and Appendix A of both accepted specs): for each conflict, one spec owns the design and the others cite it.

**Documents reconciled.** Draft 0.2 of [HTML and CSS parsing and style][ST] (ST), [fonts and text][TX] (TX) and [networking][NW] (NW), against the accepted [JavaScript engine][JS] (JS, draft 0.3) and [resource footprint][FP] (FP, draft 0.3) specs, [HGBA-2 2.3.0][H2] M-1, and the [Stage 1 design][S1]. The JS and FP specs, HGBA-2 and Stage 1 are not edited; every amendment they need is listed in the amendments section of the new spec that owns it and summarized in Section 5 below. Each new spec gains a revision 0.3 line and an Appendix A section for this pass.

**Addendum, HGBA-2 2.3.1 (19 October 2026).** M-1 part 3 now says another engine's source code is never evidence. This note was checked for links to engine source and for unlinked claims read from it: it has none. Its one size estimate (P9's bytes per glyph) is now derived from TX 6.7's output contract and marked †. P25's test-only oracles (HarfBuzz `hb-shape`, Core Text, DirectWrite, FreeType tools, fontTools, rustls-webpki, other engines over WebDriver BiDi) are run as **black-box programs**: their outputs are observations, which 2.3.1 allows; triaging a divergence must not read the oracle's source. The three specs' passes are logged in [ST's](../Izcal%20HTML%20and%20CSS%20parsing%20and%20style/source_evidence_pass.md), [TX's](../Izcal%20fonts%20and%20text/source_evidence_pass.md) and [NW's](../Izcal%20networking/source_evidence_pass.md) `source_evidence_pass.md`; each spec is at revision 0.3.1.

**Result.** 27 items (P1–P27): conflicts between the new specs, conflicts with the accepted JS and FP specs, gaps that no spec covered, consistency fixes and one open measurement (the Kind column of Section 1). Every item is applied to the three new specs. Four need an owner decision because they change an accepted spec's design or budget (Section 6). No footprint budget is exceeded on paper, but the per-(top-level site, container) network services use about a third of FP-S30's headroom and are outside FP's model until FP is amended (Section 3). The five components need about **169–218 engineers† at full strength** (Section 4).

## 1. Summary

| ID | Topic | Kind | Owner after reconciliation | Owner decision |
|---|---|---|---|---|
| P1 | One shared standards layer (L2s) and the layer registry | Conflict (new specs) | ST (layer rule G24) | — |
| P2 | WHATWG Encoding (`hg-encoding`) | Conflict (new specs) | ST | — |
| P3 | Unicode data and algorithms (`hg-ucd`, `hg-unicode`) | Conflict (JS) | TX | **OD-1** |
| P4 | Shared math (`hg-math`) | Conflict (JS) | JS | — |
| P5 | Compression (`hg-compress`) | Gap and conflict (new specs) | NW | — |
| P6 | URL Standard (`hg-url`), Public Suffix List, cookie parser, MIME types | Gap | ST (`hg-url`); NW (the rest) | — |
| P7 | One G-rule registry | Conflict (new specs) | Stage 1 2.2 (registry); each spec its rules | — |
| P8 | `FontFace` and `@font-face` descriptor parsing across layers | Conflict (new specs) | ST parses; TX consumes | — |
| P9 | FP-11's 512 B per node and text-proportional data | Conflict (FP) | ST (split); TX (text line) | — |
| P10 | Network services per (top-level site, container) against FP's model | Conflict (FP) | NW (budgets); FP (model, amended) | **OD-2** |
| P11 | Unbudgeted network helper processes | Gap | NW | — |
| P12 | Font service, sealed font copies, the Izcal font set's mapping | Conflict (FP) | TX | — |
| P13 | Third-party IME code in the broker (TX-D23) against FP-1 | Conflict (FP) | TX | — |
| P14 | The Izcal font set against FP-25 and FP-D13 | Conflict (FP) | TX (TX-D1) | **OD-3** |
| P15 | Glyph atlas size for CJK and emoji (FP-14) | Open measurement | TX (TX-52) | — |
| P16 | CSP renderer and network split | Conflict (new specs) | NW | — |
| P17 | The font-metrics query and numeric hand-off | Conflict (new specs) | TX (API); ST (consumer) | — |
| P18 | Partition keys for in-process caches, `:visited` and nonce partitions | Conflict (new specs and FP) | NW (key type) | — |
| P19 | Cache classes: in-memory classes, canonical bytes and FP-D16 members | Conflict (FP) and gaps | FP (classes, amended); each spec its members | **OD-4** |
| P20 | Determinism: decision points and the two recordings | Consistency | Each spec its points; JS-D15 the rule | — |
| P21 | Cosmetic filters | Gap | NW (compile); ST (apply) | — |
| P22 | Speculative-parser fetches in the network service | Gap | NW | — |
| P23 | Shared-crate delivery dates and the missing layout spec | Gap | Each owner; layout spec unowned | — |
| P24 | Staffing and double counting | Conflict (new specs and JS) | Each spec its tracks | (part of OD-1) |
| P25 | M-1 classifications of oracles and data | Consistency | JS-D9 class; each spec its members | — |
| P26 | Stale cross-references between drafts | Consistency | Each spec | — |
| P27 | Internal inconsistency in TX (class (b) against class (a)) | Conflict (within TX) | TX | — |

## 2. Items

### P1. One shared standards layer (L2s) and the layer registry

**Conflict.** Three new layers were proposed in parallel for overlapping crates. ST adds **L2p** for its parsers and puts `hg-encoding` and `hg-math` there ([ST 6.15][ST]); TX adds **L2u** for `hg-ucd` and `hg-unicode` below L2e and **L2t** for the text stack ([TX 6.15][TX]); NW puts `hg-net-wire` and `hg-compress` in **L2** (contracts), and `hg-cookies` in L5 although renderers link it ([NW 6.21][NW]). JS G6 forbids any crate outside L2e from depending on L2e crates, and G13 lets L2e depend only on L1–L2 ([JS 6.15][JS]), so `hg-ucd` (in L2e) cannot serve TX or ST, and an L4 renderer crate cannot link an L5 crate.

**Resolution.** One sub-layer of L2, **L2s (shared standards)**, holds every crate that implements a standard over plain data and is linked by more than one of the engine, the parsers, the text stack, the network services and the broker: `hg-encoding`, `hg-url` (ST), `hg-math` (JS), `hg-ucd`, `hg-unicode` (TX), `hg-compress`, `hg-net-wire`, `hg-cookie-parse`, `hg-psl` (NW). L2s crates depend only on L1, L2 and each other and are written to tier P, because network services and the broker link several of them. Because L2s is part of L2, JS G13 holds unchanged; L2e, L2p and L2t are siblings that never depend on one another, and L3 and above join them. TX's L2u merges into L2s.

| Layer | Holds | Owner of the layer's text |
|---|---|---|
| L2 (Stage 1) | Contracts: `hg-tree-traits`, `hg-ipc-gen`, `hg-types` | Stage 1 |
| **L2s** (part of L2) | The nine shared crates above | ST (rule G24) |
| L2e | JavaScript and Wasm engine | JS |
| L2p | HTML, XML and CSS parsing, property tables, selectors | ST |
| L2t | Font formats, glyph programs, shaping, matching, loading, text interface, raster | TX |

**Owner.** ST owns the layer rule (new **G24**), because ST first proposed a shared parser layer and owns two of its crates. **Changed.** ST 6.15 (layer table, G24), 9.5; TX 6.6, 6.15, 9.5, TX-D11; NW 6.2, 6.21, 9.5.

### P2. WHATWG Encoding (`hg-encoding`)

**Conflict.** ST builds it and asks who owns it (Q-ST-2, [ST 6.1, 11.1][ST]); TX disclaims it ([TX 9.5][TX], ST row); NW does not mention it, yet NW links `hg-url`, whose query encoding needs it.
**Resolution.** ST owns `hg-encoding` in L2s, tier P. Reasons: its first consumer is the HTML byte-stream decoder, ST already budgets it in its HTML parsing track, and the Encoding Standard's sniffing order is part of HTML parsing. NW and TX cite it. Q-ST-2 is closed. **Changed.** ST 6.1, 6.15, 9.5, 11.1.

### P3. Unicode data and algorithms (`hg-ucd`, `hg-unicode`)

**Conflict.** The accepted JS spec places `hg-ucd`, UAX 29 and UAX 15 in L2e, owned by its Intl and Unicode track ([JS 6.13, 6.15, 9.3][JS]). TX needs them below L2e and proposes moving them to L2u, but leaves segmentation and normalization staffed by JS ([TX 6.6, 9.3][TX]), so one crate would have two owners. TX also declares both crates tier R because "no tier-P binary links them" ([TX 6.15][TX], C30). That premise fails once `hg-url` exists: UTS 46 (IDNA) host processing needs the UCD-derived mapping table and NFC, and the network services and the broker parse URLs (P6).
**Resolution.** **TX owns** `hg-ucd`, `hg-unicode` and the generator `hg-unicode-build`, in L2s, written to tier P. Reasons: the text stack is their largest consumer (bidi, line breaking, itemization, emoji, vertical orientation), TX-1 is the gate for every Unicode upgrade, and one owner must run the yearly upgrade for the engine, the text stack and URLs at once. JS keeps `hg-intl` (formatting, collation, plural rules, calendars), `hg-tz` and `hg-cldr-build`, and consumes L2s; one Unicode version is pinned per build and recorded once. UTS 46 joins TX's algorithm table, with IdnaTestV2 in TX-1. About one engineer moves with the work from JS's Intl and Unicode track (4–5†, to 3–4†) to TX's Unicode for text track (3–4†, to 4–5†); the combined total is unchanged (P24).
**Owner decision OD-1**, because it moves a crate and staff out of an accepted spec. Recommendation: accept. **Changed.** TX 6.6, 6.15, 8.1 (TX-1), 9.3, 9.4 (TX-D11), 9.5, Appendix A; ST 6.1, 6.15. JS amendments (listed in TX 9.5): 6.13, 6.15 crate table, 9.3 track sizes.

### P4. Shared math (`hg-math`)

**Conflict.** ST proposes a new `hg-math` crate for `calc()` and colour and asks the JS spec to move its math functions there ([ST 6.9, 9.5][ST]); JS-D12 and JS 6.9 define one implementation per function on every ISA, inside the engine. TX uses no floating point (G28, was G-TX4) and needs nothing.
**Resolution.** **JS owns** `hg-math`, placed in L2s with a tier-P build for the component host; ST cites it. The semantics stay JS-D12's (one implementation per function, no FMA contraction, no platform libm). **Changed.** ST 6.9, 6.15, 9.5 (JS row restated). JS amendment (listed in ST 9.5): 6.9 and 6.15 name `hg-math` in L2s.

### P5. Compression (`hg-compress`)

**Conflict and gap.** NW builds `hg-compress` (NW-D6) in L2 and TX uses it for WOFF 1.0 and 2.0 in renderers ([NW 6.3, 6.21][NW], [TX 6.2][TX]): agreed. Not covered by either: encoders. Compression Streams (`CompressionStream`), WebSocket `permessage-deflate` on send and PDF writing need a deflate encoder; PNG decoding (FP-D11's decoder process) needs inflate. Timing gap: TX-M1 ships WOFF2 at 15–21 months, while NW's Brotli lands with NW-M2 at 18–24.
**Resolution.** **NW owns** `hg-compress` in L2s, tier P, with the decoders it lists plus the deflate and gzip encoders the Compression Streams standard and `permessage-deflate` need, within its compression track (1–2†). Brotli, deflate and gzip decoders are delivered at NW-M1. **Changed.** NW 6.3, 6.21, 7, 9.2 (NW-M1), 9.3, 9.5; TX 6.2, 9.5.

### P6. URL Standard (`hg-url`), Public Suffix List, cookie parser and MIME types

**Gap.** NW makes `hg-url` "the browser's one URL parser, owned by the HTML and DOM work" ([NW 6.2][NW], NW-G2), but no spec owns or staffs it: ST's scope stops at HTML and CSS byte streams, the JS spec owns no URL code, and the DOM's Stage 1 plan relied on an excluded Servo crate. Three related crates sit in the wrong layer: `hg-psl` (L5) is needed by `document.cookie`'s domain checks and by the URL Standard's registrable-domain concept; `hg-cookies` (L5) is linked by renderers; and MIME type parsing, which ORB, strict MIME checks for style sheets and the `charset` parameter all need, has no named parser.
**Resolution.** **ST owns `hg-url`** (URL Standard parser and serializer, `URL` and `URLSearchParams` backing, host parsing with UTS 46 from L2s), in L2s, tier P. Reasons: it is a parser of a web grammar with the same differential risk as HTML (a URL check that disagreed with the full URL parser was one of the two Chrome 146 sanitizer bypasses, which ST's G22 already cites), it uses `hg-encoding` for query encoding, and ST has the parser-generation and differential-testing machinery. ST's HTML parsing track grows by one engineer† (P24). New target ST-15 (WPT `url/` and `encoding/`). **NW owns** `hg-psl` (moved to L2s), a new `hg-cookie-parse` (the parser and serializer split out of `hg-cookies`, in L2s; the store stays in L5), and the MIME Sniffing standard's MIME type parser and sniffing algorithms inside `hg-net-wire`. **Changed.** ST 6.1, 6.15, 7, 8.1 (ST-15), 9.2, 9.3; NW 6.2, 6.21, 7, 9.5.

### P7. One G-rule registry

**Conflict.** ST numbers its rules G19–G23 as provisional; TX uses G-TX1–G-TX5; NW uses NW-G1–NW-G6; each asks the Stage 1 owner to renumber ([ST 6.15][ST], [TX 6.15][TX], [NW 6.21][NW]). The new layer rule (P1) needs a number too.
**Resolution.** One registry, kept in Stage 1 2.2 and assigned here in spec order. ST keeps its numbers and adds the layer rule; TX and NW are renumbered.

| Rules | Owner | Statement in brief |
|---|---|---|
| G1–G9 | Stage 1 | Unchanged (G1 restated by ST; G6 rewritten by JS) |
| G10–G12 | DevTools spec | Unchanged |
| G13–G18 | JS | Unchanged |
| G19–G23 | ST | Script-free parser and style core; independent reference engine; no recursion on content depth; differentials for fast paths; one parser per grammar |
| **G24** | ST | L2s layer rule (P1) |
| **G25–G29** | TX (were G-TX1–G-TX5) | One font reader; no font code in the broker, GPU, network or storage services; no `unsafe`, G18 extended; no floating point on observable text paths; generated readers and engines only |
| **G30–G35** | NW (were NW-G1–NW-G6) | Sans-IO protocol crates; one parser per format; no `async` or `std::net`; budgets on peer-sized allocation; the socket broker parses nothing; wire identifiers from `Entropy::crypto()` |
| G36 onward | Next spec | Assigned in Stage 1 2.2 |

**Changed.** ST 6.15, 9.5; TX 5, 6.11, 6.12, 6.15, 7, 8, 9.5 (every reference renumbered); NW 6.21, 8.1 (NW-9), 9.5.

### P8. `FontFace` and `@font-face` descriptor parsing across layers

**Conflict.** TX's `hg-font-load` (L2t) implements `FontFace` ([TX 6.5][TX]), whose constructor takes descriptor strings, and canvas text takes a CSS `font` string; parsing them needs ST's generated parsers (L2p). Under P1, L2t cannot depend on L2p, and ST's G22 and G23 forbid a second parser of a CSS grammar.
**Resolution.** Descriptor and `font` strings are parsed once, by `hg-css-props`' generated parsers (descriptor grammars come from webref like property grammars, ST-D6), in the bindings and `hg-cssom` (L3). `hg-font-load` and `hg-text` receive **typed descriptors** only. `@font-face`, `@font-feature-values` and `@font-palette-values` rules parsed from style sheets reach TX by the same typed hand-off. ST owns the parsers; TX owns the typed interface. **Changed.** ST 6.9, 6.16, 9.5; TX 6.5, 6.14, 9.5.

### P9. FP-11's 512 B per node and text-proportional data

**Conflict.** ST splits FP-11's 512 B† per node as DOM 72 / style 80 / layout and paint 360 ([ST 6.11][ST]). FP-11 excludes "text and images" but does not say whether shaped glyph runs, which layout keeps in its inline item lists, are text or layout; TX budgets the text stack per page (≤ 2 MiB†, [TX 6.16][TX]) but not where layout's copies of glyph runs go. At about 8–12 bytes per glyph† (from TX 6.7's output contract: a 16-bit glyph ID, a 16-bit advance, two 16-bit offsets in font units and a cluster index of 16 to 32 bits with the three flags packed in, so 10–12 bytes, or 8 with the offsets stored only when non-zero), glyph data on a text-heavy page would take a large part of the 360 B.
**Resolution.** Shaped glyph runs and segments, wherever held, are charged to FP's existing ledger category **"text shaping"** and to TX-50's text line, never to the per-node budget, so FP-11's text exclusion covers them. ST's split stands (72 + 80 + 360 = 512 B†, with JS-D4's 48 B† columns inside the 72); the layout spec inherits 360 B† for boxes, fragments and paint data only. NW adds no per-node data. **Owner.** ST owns the split, TX the text line. **Changed.** ST 6.11, 9.5; TX 6.16, 9.5. FP amendment (listed in TX 9.5 and ST 9.5): FP-11's "text excluded" includes the text-shaping category.

### P10. Network services per (top-level site, container) against FP's model

**Conflict.** FP's model counts one network service inside the baseline B (≤ 8 MiB† in FP-1, [FP 6.1, 8.2][FP]). NW-D3 makes it one system service, one container service per active container and one partition service per open top-level site, plus a spare ([NW 6.1, 6.17][NW]). NW budgets only the idle size of each (≤ 4, ≤ 2, ≤ 2 MiB†) and amends FP-1, but not FP's model, FP-S30, FP-6, FP-7 or FP-28. FP-D3 (renderer granularity within a site) is not affected: partition services are keyed by top-level site and never split.
**Resolution.** NW adds three budgets: a spare ≤ 1.5 MiB†, a partition service **settled after a corpus page load ≤ 3 MiB†**, and a partition service whose documents are all frozen ≤ 1.25 MiB† (it drops its per-partition state and receive buffer). With these, every FP budget holds on paper (Section 3); the model gains a term for network services; FP-7 needs a scope decision. **Owner decision OD-2** (FP-7's scope). **Changed.** NW 6.17, 8.4 (NW-35), 9.5.

### P11. Unbudgeted network helper processes

**Gap.** NW introduces the OS-adapter process per container, the proxy-resolver utility per (container, route), the route process per routed container and spare network services, with no memory budget and no row in FP 8.2 ([NW 6.0, 6.1, 6.13][NW]).
**Resolution.** NW budgets them: OS-adapter ≤ 2 MiB† idle and exits after 10 s† idle (FP-38's pattern); proxy-resolver ≤ 4 MiB† (a jitless engine with no DOM, within FP-2's empty-process figure), only while PAC is configured; route process ≤ 2 MiB† idle, only for routed containers; spare ≤ 1.5 MiB†. None runs in the default configuration except the spare. **Changed.** NW 6.17, NW-35, 9.5 (FP 8.2 rows).

### P12. Font service, sealed copies and the Izcal font set's mapping

**Conflict.** FP 6.3 rule 5 has a utility process **parse** installed fonts per session; TX-D22 replaces parsing with hashing (TX lists the FP amendment). Two things were left unbudgeted: the font service's own memory, and who is charged for **sealed copies** of non-immutable font files once the service exits ([TX 6.4, 6.16][TX]); and the Izcal font set (200–350 MiB†) is mapped "immutable", which holds only if it is installed and verified like the engine image.
**Resolution.** The font service is a **utility job**: peak ≤ 8 MiB† charged while hashing, exits when done, relaunched for a per-site grant or a mid-session font install (FP-1's "utility jobs exit when done"). Sealed copies exist only for per-site granted user fonts (macOS and Windows OS-tier faces sit on protected volumes; Linux's OS tier is empty), so each is charged once, to the (site, container) whose grant needed it. The Izcal font set files join FP 6.3's image inventory under the engine image's rules (file-backed, signed, SEC-16). **Owner.** TX. **Changed.** TX 6.4, 6.16, 9.5.

### P13. Third-party IME code in the broker against FP-1

**Conflict.** TX-D23 keeps OS input methods in the broker, and TSF loads third-party IME DLLs into the process that owns the focused window ([TX 6.8a][TX]). Their memory is charged to the broker, whose FP-1 line is ≤ 20 MiB†, and no source measures it.
**Resolution.** OS-loaded IME memory is reported as its own attributed line in the broker, outside FP-1's broker line and inside FP-10's explanation of every byte, as GPU driver allocations are; FP-1 is measured with no third-party IME active. The security cost stays as TX states it. **Owner.** TX. **Changed.** TX 6.16, 9.5 (FP-1 and FP-10 rows).

### P14. The Izcal font set against FP-25 and FP-D13

**Conflict.** TX-D1 ships an Izcal font set of an estimated 200–350 MiB† installed in every installation as part of FP-D13's web-observable data set ([TX 6.16, TX-51][TX]). FP-25 budgets that data set at a **24 MiB† placeholder** on disk and the installer at ≤ 50 MiB† plus the data set ([FP 6.12, 8.2][FP]). The data set grows by 9–15 times, the installed total per architecture from about 144 MiB to about 344–494 MiB, and the second half of FP-D13's reversal trigger ("the measured data set more than doubles the installer") fires. Fonts were not among FP-D13's listed items (locales, hyphenation, codecs).
**Resolution.** Keep TX-D1 (a): a page-dependent font download would be a cross-site channel and a fingerprint, which FP-D13 already rejects for locales; option (b) of FP-D13 (data chosen by UI language) would make font fallback reveal the user's languages. FP-25 gains separate installed and compressed lines for the font set (TX-51: ≤ 350 MiB† installed, ≤ 200 MiB† compressed) and allows its delivery as a separate signed data package installed with every installation. **Owner decision OD-3.** Recommendation: extend FP-D13 to fonts and keep (a). **Changed.** TX 9.5 (FP-25 and FP-D13 row restated with numbers).

### P15. Glyph atlas size for CJK and emoji

**Open measurement, no conflict yet.** FP-14's ≤ 2 MiB† atlas holds about 1,800 A8 glyphs at DPR 2 and about 128 RGBA emoji at 64 px ([TX 6.10][TX]). TX-52 measures re-rasterization at TX-M2 and TX asks FP for separate A8 and RGBA pools. Nothing changes until TX-52; any resize stays canonical and device-independent (FP-D19), and FP-D14's partitioning never relaxes. **Owner.** TX (measurement), FP (budget).

### P16. CSP renderer and network split

**Conflict.** ST says the network service "re-checks the directives the broker gives it" ([ST 6.4, 9.5][ST]). NW says the partition service parses the document's `Content-Security-Policy` headers itself, from the response it delivered, and the broker never parses CSP (IPC-R15) ([NW 6.7][NW]).
**Resolution.** NW's text is the design: the renderer runs the pre-request checks with the full policy, including `<meta>` policies only it sees; the partition service re-checks the header-delivered policy it parsed itself, as defence in depth; a compromised renderer can therefore bypass only `<meta>` policies, a limit both specs record for the renderer threat model. Speculative and style-initiated requests carry the document's policy container as data, but the partition service never trusts it for its own check. **Owner.** NW. **Changed.** ST 6.4, 6.6a, 9.5; NW 6.7 (cross-reference).

### P17. The font-metrics query and the numeric hand-off

**Conflict.** ST asks for metrics "for (font list, size, style, language)" ([ST 6.9][ST]); TX answers with metrics as B1 inputs and states that font-size quantization is a used-value rule, never a computed-value one ([TX 6.8, 6.8a][TX]). If the query returned lengths scaled at TX's quantized size, `ex`, `ch`, `ic`, `cap` and `lh` would carry quantization into computed values, contradicting TX. ST never states the numeric type of its computed lengths. Dates: ST-M1 (Stage 1 M2) computes font-relative units, but TX implements real metrics at TX-M1 (Stage 1 M4).
**Resolution.** The query returns, per metric, the **exact ratio** (value in font units of the instance, `unitsPerEm`, with `size-adjust` and the metric overrides applied as exact factors) for the first available font; ST multiplies by the computed font size in its own computed-length type. ST fixes that type at ST-M0 as an IEEE binary type with no FMA contraction, and every conversion into TX's integer contract uses TX-D4's round-half-to-even rule. TX-M0 implements the query over the Izcal font set (its proved tables already include `head`, `hhea`, `OS/2`, `cmap`, `hmtx`), so ST-M1 is not blocked; web fonts join at TX-M1, and each install that changes a result is a mutation-log entry. **Owner.** TX (API), ST (consumer). **Changed.** ST 6.9, 9.2, 9.5; TX 6.8a, 9.2, 9.5.

### P18. Partition keys for in-process caches, `:visited` and nonce partitions

**Conflict.** NW-D7 keys network state by (T, C, F, X, N, n) ([NW 6.8][NW]). ST keys parsed sheets and attribute sets, and TX keys every text cache and the atlas, by "partition", which FP 6.4 defines as (top-level site, container) ([ST 6.6, 6.11][ST], [TX 6.8][TX], [FP 6.4][FP]). A renderer cache coarser than the network key re-creates the leak the network key closes: a credentialless iframe or fenced frame (nonce n) shares a process with other documents of its site, and would see their decoded resources, parsed sheets and shaped text by timing. `:visited` (ST-D15) says nothing about nonce partitions.
**Resolution.** NW owns one **partition key type** (fixed at NW-M0). In-process caches in a renderer key by its projection **(T, C, X, n)**: F is fixed by the process's site lock (origin under FP-D8), and N concerns only navigation requests. ST's sheet and attribute-set sharing, TX's shaping, raster, instance, parsed-font and validation caches, and FP-D14's atlas use that projection. In a nonce partition the broker sends no visited answers, so `:visited` never matches there. ST's per-agent intern tables may span two partitions only when both documents are in one agent, which already lets them script each other, so no new channel arises. **Owner.** NW (key); ST, TX (use). **Changed.** NW 6.8, 9.5; ST 6.6, 6.11, 6.14, 9.5; TX 6.5, 6.8, 9.5. FP amendment (listed in NW, ST and TX 9.5): FP 6.4 and FP-D14 use the projection.

### P19. Cache classes: in-memory classes, canonical bytes and FP-D16 members

**Conflict.** (a) TX makes text caches class (a) but excludes their bytes from canonical accounting, because parallel layout workers insert in varying order ([TX 6.8][TX]); FP-24 requires canonical touched and committed bytes to be bit-identical across runs and has no exempt class ([FP 8.3][FP]). ST takes the other route: style data goes through a deterministic merge and is counted ([ST 6.9][ST]). (b) ST's parsed-sheet store says it can be rebuilt "from the HTTP cache" ([ST 6.11][ST]); a refetch is a new network input and would break FP-12's bit-identical rebuild. (c) Two on-disk formats outside FP-D16's member list: ST-D17's compiled user style sheets ([ST 6.16][ST]) and TX's font-hash cache in the profile ([TX 6.4][TX]). NW's HTTP cache already asks to join FP-D16, bound to format version and a security epoch rather than the build (NW-D11), which ST and TX do not contradict.
**Resolution.** (a) FP gains a **canonical-exempt** sub-class of class (a): caches whose contents may depend on worker interleaving, with a fixed byte cap, observationally transparent (TX-31), on their own ledger line and outside CT and CC. Only TX's shaping, raster and instance caches use it; style stays counted. **Owner decision OD-4**, because it changes FP-24's scope. Recommendation: accept; the alternative (a deterministic merge for every shaping-cache insertion) costs a merge in every layout pass for data that cannot affect results. (b) ST keeps a copy of the decoded bytes with each entry and never refetches to rebuild. (c) Both formats join FP-D16 as members: the compiled user sheet (written by the shell renderer, never the only copy of the user's sheet, which lives in the storage service) and the font-hash cache (written by the font service, never the only copy of anything). **Changed.** TX 6.4, 6.8, 6.14, 9.5; ST 6.11, 6.16, 9.5.

### P20. Determinism: decision points and the two recordings

**Consistency.** All three specs follow JS-D15: ST records parser yields, speculative requests and their reuse, and sheet installs ([ST 6.4, 6.6a][ST]); TX records font installs and uses the substrate clock for `font-display` ([TX 6.5, 6.11][TX]); NW records race winners, pool decisions, cache lookups and DNS answers ([NW 6.16][NW]). Two clarifications were missing. TX said the install point is "replayable" without saying it is logged when recording. And ST treats network chunk boundaries as B1 inputs while NW records at the decryption boundary in the network service: the renderer's view (how many bytes each pipe read returned, and in which task) is a separate input.
**Resolution.** TX's install is a decision point **logged when recording**. Two recordings exist and are joined by request ID: the renderer's, which records data-pipe reads as IPC inputs (Stage 1 3.4, DT-D2), and NW-D2's network recording. OS dependencies are recorded in each spec as already stated (the locale encoding default, the environment snapshot, OS font manifests and session-image identity, OS resolver and proxy configuration). TX's canonical-exempt caches (P19) never feed a recorded or JS-observable value. **Changed.** TX 6.5, 6.11; ST 6.1; NW 6.16.

### P21. Cosmetic filters

**Gap.** FP 6.11 delivers cosmetic filters to renderers as a per-site user-origin style sheet (≤ 256 KiB† per site process, cascade shared per partition). NW says cosmetic rules are "compiled separately for renderers" without an owner ([NW 6.15][NW]); ST never mentions them.
**Resolution.** NW's content-blocking track compiles cosmetic rules, in the same strictest-tier utility job as the request automaton, into per-site CSS subsets; ST parses and applies them as user-origin sheets, shared per partition (P18's projection), visible in DevTools as the user origin. Because cosmetic rules change computed styles, the container's list set is engine configuration, recorded, and covered by ST-D17's global-sheet fingerprint case in PRIV-2. **Changed.** NW 6.15, 9.3; ST 6.16.

### P22. Speculative-parser fetches in the network service

**Gap.** ST's speculative parser issues fetches and reuses their responses ([ST 6.4][ST]); NW's Fetch section covers preconnect, prefetch and prerender but not these ([NW 6.7, 6.11][NW]).
**Resolution.** They are ordinary subresource fetches through the frame's `FetchEndpoint`, marked speculative, at the priority the loader assigns, with every policy check of Section 6.7; their fate is a decision-log entry. **Owner.** NW. **Changed.** NW 6.7; ST 6.4.

### P23. Shared-crate delivery dates and the missing layout spec

**Gap.** The shared crates gate other specs' milestones: NW-M1 (9–12 months) needs `hg-url` and `hg-encoding`, but ST-M1 lands at 15–21; `hg-url`'s IDNA needs normalization from `hg-unicode` at the same time; TX-M1 (15–21) needs Brotli, planned for NW-M2 (18–24); ST-M1 needs the metrics query (P17). Separately, ST and TX both freeze contracts with a layout spec that does not exist (ST-M0 container re-entry, TX-M0 layout contract), and Stage 1's "style and layout" track (3–4) is left holding layout.
**Resolution.** Delivery dates: `hg-encoding` and `hg-url` at ST-M0 (ST-15); normalization and IDNA data first in TX-M0, by month 6; Brotli, deflate and gzip at NW-M1; the metrics query over the Izcal set at TX-M0; `hg-math` with JS-M1. The layout spec has no owner: this note records it as the largest unowned item on Stage 1's critical path; ST's and TX's existing risk rows cover their side. **Changed.** ST 9.2; TX 9.2; NW 9.2.

### P24. Staffing and double counting

See Section 4. Double counts checked and removed or confirmed absent: the content-blocking matcher (NW only; FP-M5's matcher deliverable is NW-M4's work), Stage 1's "style and layout" track (ST replaces its style half, TX its Parley part, layout stays unowned), the Unicode team (P3, one engineer moves), compression (NW only), URL (new, ST, +1), the security on-call (each spec counts only its own share of the security scope's rotation), DevTools transfers (ST removes 2 + 8 contingent from DevTools, NW takes 8 engineer-months, both transfers, not additions), and FP's "2–3† in other tracks" (already inside the component tracks, so not added).

### P25. M-1 classifications of oracles and data

**Consistency.** Every spec classifies the UCD, CLDR, IANA registries and Mozilla's root store as standards data, and WPT as an oracle, the same way. Test-only oracles are proposed in three places: JS-D9 (accepted: ESMeta, SpecTec, engine262 and others, never linked), TX-D9 (HarfBuzz `hb-shape`, Core Text, DirectWrite, FreeType tools, fontTools, as black-box CI oracles) and NW-D19 (rustls-webpki as a differential oracle). ST uses other engines over WebDriver BiDi only. Under HGBA-2 2.3.1 these remain allowed as black-box observation only: a divergence is triaged against the specification and the oracle's observed output, never against its source. Data needing the owner: ST-D6 (`@webref/css`), TX-D9 (USE override files, ICU word lists, hyph-utf8 patterns), TX-D14 (pattern licences).
**Resolution.** TX-D9's oracle part and NW-D19's oracle part are members of the **test-only oracle class** the owner accepted with JS-D9: never linked into a shipping binary, divergences triaged against the specification, never resolved by matching. The owner confirms membership rather than a new class. TX-D14 is consistent with FP-D13 (every language still ships; non-permissive patterns are rebuilt). One Unicode version and one CLDR version per build are recorded once, by `hg-unicode-build` (TX) and `hg-cldr-build` (JS). **Changed.** TX 9.4 (TX-D9), 9.5; NW 7, 9.5.

### P26. Stale cross-references between drafts

TX cited ST and NW as draft 0.1; NW and ST did not cite each other or TX as documents. Each header now cites the others at draft 0.3 and this note. **Changed.** Headers of ST, TX, NW.

### P27. TX: class (b) against class (a)

TX's integration table called text caches "class (b) derived data" ([TX 6.14][TX]) while TX 6.8 and 6.16, after critique C13, make them class (a). Class (a) is right (own arenas, released after the safepoint handshake). **Changed.** TX 6.14.

## 3. Footprint budget results

Charged memory, settled, from the budgets in FP 8.2 and the specs as reconciled. "Holds" means the arithmetic of the stated budgets fits; nothing is measured yet.

| Budget (FP) | Before NW-D3 | With P10 and P11 | Result |
|---|---|---|---|
| FP-1 network line ≤ 8 MiB† | One service ≤ 8 | System ≤ 4 + spare ≤ 1.5 = **5.5 MiB**; the default container service starts with the first navigation | Holds |
| FP-1 broker ≤ 20 MiB† | — | Third-party IME memory reported on its own line (P13) | Holds by definition; IME memory unmeasured |
| FP-S30 at settle ≤ 1.25 GiB† (1,280 MiB) | 30 × FP-5 median 32 = 960, + baseline 64 = 1,024; headroom about **256 MiB** for the three iframe-heavy pages' extra processes | + 30 partition services × 3 settled = 90 (the container service, ≤ 2, fits in the 8 MiB line beside the system service and spare) → about 1,114 MiB; headroom about **166 MiB** | Holds; about a third of the headroom is used |
| FP-6 after 10 min idle ≤ 0.75 GiB† (768 MiB) | 29 × 12 + 32 + 64 = 444 | + 29 × 1.25 frozen + 1 × 3 visible ≈ 39 → about 483 MiB | Holds |
| FP-7 isolation overhead ≤ 5%† corpus | Renderer isolation only | Duplicated network runtime: one page about 2 MiB of about 99 MiB (2%); at FP-S30 about 30 × 1.25 = 37.5 MiB of about 1,114 MiB (3.4%) | **Needs OD-2**: if FP-7 counts it, renderer isolation keeps only 1.6–3% |
| FP-28 matcher ≤ 8 MiB† | One matcher | 256 KiB† per partition service: 7.5 MiB at 30 top-level sites, above 8 MiB from 33 | Amend FP-28 to per service (NW lists it) |
| FP-2 empty site process ≤ 4 MiB† | — | TX adds 0 bytes at start-up; ST's style share ≤ 64 KiB† inside the 0.25 MiB document line (ST-34) | Holds |
| FP-5 median page ≤ 32 MiB† | — | Text stack ≤ 2 MiB† (TX-50); style ≤ 80 B† per element (ST-30) | Holds by construction of the lines |
| FP-11 512 B† per node | — | 72 + 80 + 360; glyph runs in the text-shaping category (P9) | Holds |
| FP-14 atlas ≤ 2 MiB† per partition | — | CJK at DPR 2 and emoji-dense pages may need more (P15) | Open until TX-52 |
| FP-25 data set ≤ 24 MiB† placeholder | — | + 200–350 MiB† Izcal font set (P14) | **Broken; OD-3** |
| New processes not in FP 8.2 | — | OS-adapter ≤ 2 MiB† (exits idle), proxy resolver ≤ 4 MiB† (PAC only), route ≤ 2 MiB† (routed containers only), font service job peak ≤ 8 MiB† (exits), spare network service ≤ 1.5 MiB† | Rows to add (listed in NW and TX 9.5); spellcheck utility (Stage 2) budgeted by the editing spec |
| FP-D3 renderer granularity | — | No change: partition services are keyed by top-level site and never split | No conflict |

**FP amendments needed, by owner of the request:** NW: FP 6.1 model (network services as a term, with N_net = open top-level sites per container plus spares), FP-1 itemization with the spare, FP-7's scope (OD-2), FP-S30 and FP-6 lines, FP-28 per service, 8.2 rows for the helper processes, FP 6.4 partition projection, FP-D16 (HTTP cache, already listed), FP 6.11 and 6.18.1 (already listed). TX: FP-11 text-shaping exclusion, FP-24 canonical-exempt sub-class (OD-4), FP-D13 and FP-25 (OD-3), FP-1 and FP-10 IME line, FP 6.3 inventory (font set, font-service job), FP-D16 (font-hash cache), FP 6.4 and FP-D14 projection, FP-14 pools (already listed). ST: FP-11 split (already listed), FP 6.4 projection, FP-D16 (compiled user sheets).

## 4. Staffing, side by side

Full-strength engineers† as each spec states them after reconciliation. FP has no component of its own; its Stage 1 amendment asks for a footprint track of 3–4† plus 2–3† inside other tracks, which the component tracks already contain.

| Spec | Full strength† | Start (M0)† | Month 12† | Engineer-years to parity† | Parity† |
|---|---|---|---|---|---|
| JavaScript engine (JS) | 61–76 → **60–75** with OD-1 (Intl and Unicode 4–5 → 3–4) | 20–25 | about 40 | about 265–460 | 5–7 years |
| Resource footprint (FP) | 3–4 (footprint track) | 3–4 | 3–4 | about 15–25 (3–4 a year over the same period; FP states no figure) | With Stage 1 exit |
| Parsing and style (ST) | 30–39 → **31–40** (URL, P6) | 13–16 → 14–17 | about 27 | about 130–215 | 4.5–6 years |
| Fonts and text (TX) | 28–37 → **29–38** (Unicode ownership, P3) | 12–15 | about 25 | about 105–165 | 4–5 years |
| Networking (NW) | **46–61** (encoders and cosmetic compiler absorbed) | 18–22 | about 35 | 160–255 | 4–5 years |
| **Combined** | **169–218** | **67–83** | **about 130** | **about 675–1,120** | — |

**Honest reading.** The combined start of 67–83 specialists at M0, rising to about 130 within a year, is itself the largest schedule risk each spec names separately (hiring JIT, GC, shaping, QUIC and style specialists). The total does not include the layout and graphics specs (not written; P23), the DOM, bindings and event loop (Stage 1's 5–6), the shell, storage, media, security and infrastructure tracks, or DevTools. Stage 1 8.3's whole pre-2.3.0 plan was 23–30.5 engineers; these five components alone are about six to seven times that.

## 5. Amendments to the accepted specs, HGBA-2 and Stage 1 (not applied)

Each row is listed in the amendments section of the spec named in the last column.

| Document | Amendment | Item | Listed in |
|---|---|---|---|
| JS 6.13, 6.15 | `hg-ucd` and the UAX 29, UAX 15 and case-mapping code move to L2s, owned by TX; `hg-intl`, `hg-tz`, `hg-cldr-build` stay | P3 (OD-1) | TX 9.5 |
| JS 9.3 | Intl and Unicode track 4–5† → 3–4† | P3, P24 | TX 9.5 |
| JS 6.9, 6.15 | `hg-math` is a JS-owned crate in L2s with a tier-P build | P4 | ST 9.5 |
| JS 6.15 (G13) | No change needed: L2s is part of L2 | P1 | ST 9.5 (stated) |
| FP 6.1, 8.2, FP-1, FP-6, FP-S30, FP-28 | Network services as a model term; itemized lines; per-service matcher | P10 | NW 9.5 |
| FP-7 | Scope: renderer isolation, with network placement reported separately under NW-35 | P10 (OD-2) | NW 9.5 |
| FP 8.2 | Rows: OS-adapter, proxy resolver, route process, spare network service, font-service job | P11, P12 | NW 9.5, TX 9.5 |
| FP 6.3 | Font service hashes, never parses; the Izcal font set in the image inventory | P12 | TX 9.5 |
| FP-1, FP-10 | Third-party IME memory reported on its own line | P13 | TX 9.5 |
| FP-D13, FP-25 | The Izcal font set joins the data set; its own installed and compressed lines | P14 (OD-3) | TX 9.5 |
| FP 6.4, FP-D14 | In-process cache key = (T, C, X, n) projection of NW-D7 | P18 | NW, ST, TX 9.5 |
| FP 6.4, FP-24 | Canonical-exempt sub-class of class (a) | P19 (OD-4) | TX 9.5 |
| FP-D16 | Members: HTTP cache (epoch-bound), compiled user sheets, font-hash cache | P19 | NW, ST, TX 9.5 |
| FP-11 | 72 / 80 / 360 B† split; text-shaping category excluded with text | P9 | ST 9.5, TX 9.5 |
| HGBA-2 M-1 | TX-D9's and NW-D19's oracles join JS-D9's test-only oracle class | P25 | TX 9.5, NW 9.5 |
| Stage 1 2.1 | Layers L2s, L2p, L2t (L2u withdrawn), with their crates | P1 | ST, TX, NW 9.5 |
| Stage 1 2.2 | G-rule registry: G19–G24 (ST), G25–G29 (TX), G30–G35 (NW) | P7 | ST, TX, NW 9.5 |
| Stage 1 8.3 | Combined staffing of Section 4; layout spec unowned | P23, P24 | ST 9.5; cited in TX and NW 9.3 |

## 6. Decisions for the owner

A bare "yes" means the recommendation.

| ID | Decision | Recommendation | Why |
|---|---|---|---|
| **OD-1** | Move `hg-ucd`, `hg-unicode` (UAX 29, UAX 15, case mapping) and about one engineer from the accepted JS spec to TX, in L2s, tier P | **Yes** | One owner for the yearly Unicode upgrade across engine, text and URLs; TX-1 is the gate; IDNA makes the crates tier P anyway |
| **OD-2** | FP-7's scope: renderer isolation only, with network placement reported separately and bounded by NW-35 | **Yes** | Counting both under 5%† would leave renderer isolation 1.6–3%; network isolation has its own budget line and NW-D3's reversal trigger |
| **OD-3** | The Izcal font set joins FP-D13's data set; FP-25 gains its own lines (≤ 350 MiB† installed, ≤ 200 MiB† compressed), deliverable as a separate signed package installed with every installation | **Yes**, keeping TX-D1 (a) | A per-language or on-demand font set is a fingerprint and a cross-site channel; disk is the cost |
| **OD-4** | A canonical-exempt sub-class of class (a) for caches whose insertion order may vary, capped, transparent and outside CT and CC | **Yes** | The cache cannot change any result (TX-31); a deterministic merge for every insertion costs layout time for nothing observable |

[ST]: ../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[TX]: ../../reports/Izcal%20fonts%20and%20text.md
[NW]: ../../reports/Izcal%20networking.md
[JS]: ../../reports/Izcal%20JavaScript%20engine.md
[FP]: ../../reports/Izcal%20resource%20footprint.md
[H2]: ../../reports/HGBA%202%20browser%20architecture%20spec.md
[S1]: ../../reports/HGBA%20Stage%201%20architecture%20design.md
