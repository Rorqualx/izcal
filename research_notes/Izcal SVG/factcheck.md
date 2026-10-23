# Izcal SVG: fact-check (phase 8)

Checked 20 October 2026 against `reports/Izcal SVG.md` (draft 0.1) and its eight research-note files, by a checker who wrote neither the spec nor the notes. Locations are given by **section** (and by SV-, G- or Q- ID where there is one), not by line.

## Method

The claims that carry a decision (SV-D1 to SV-D16), a target (SV-1 to SV-63), a rule (G120 to G129) or the history were checked against primary sources where they could be reached, and otherwise against the research notes:
- **SVG 2.** The Candidate Recommendation's status section and the `use` chapter were read on w3.org through a fetch tool (w3.org answers plain downloads from this host with a Cloudflare challenge, HTTP 429). For text, section numbers and `id` attributes, the SVG 2 **Editor's Draft** (w3c.github.io, generated 13 October 2026, one week after the CR) was downloaded chapter by chapter (conform, types, struct, styling, coords, paths, text, embedded, painting, pservers, linking, interact, implnote, mimereg, animate, changes) and searched locally. Where the ED and the CR differ, this is said.
- **Other specifications.** SVG Integration FPWD (w3.org, read through the fetch tool) and ED (svgwg.org, downloaded); SVG Animations Level 2 ED (svgwg.org, downloaded); SVG-AAM (TR status line read; ED downloaded for `id`s); Filter Effects 1 (TR, tainted-primitives section read); the draft 2026 SVG WG charter.
- **Data.** Chrome use-counter timelines for buckets 501 (`SVGSMILElementInDocument`) and 250 (`SVGSVGElementInDocument`) downloaded from chromestatus as JSON and re-read at the dates the spec uses; the CVE records of CVE-2016-9079, CVE-2012-5112, CVE-2013-0912, CVE-2019-20446, CVE-2020-26870, CVE-2023-38633 and CVE-2026-31807 from the CVE.org API; the CISA Known Exploited Vulnerabilities feed.
- **Prose primary sources, read through the fetch tool:** the blink-dev "Intent to deprecate: SMIL" post (29 April 2015); the www-svg forward of Google's suspension (17 August 2016); Securelist (Kaspersky, SVG phishing); KnowBe4; librsvg's security page; Chrome's blog posts on `data:` in `use` and on hardware-accelerated SVG animations; McCormack's 2013 Firefox SVG text post; the WPE WebKit LBSE post; State of HTML 2024 (content); lyra.horse "SVG clickjacking" (December 2025); svgwg issue #1087.
- **Izcal's own documents.** Every Izcal cross-reference in the spec was checked against the other reports, reconciliation 2 and the Phase 3 ledger by a helper (local files only); the checker reviewed each wrong or partly row.
- **Links.** A second helper extracted every URL from the spec and the notes, classified each by its URL string for source code (none was opened), and checked liveness with HTTP requests. Fragments of the notes' SVG 2, svgwg.org and SVG-AAM links were checked against the downloaded drafts' `id` attributes.

About 26 web fetches were used. **No source code of any project was opened** (M-1 part 3). The helper's URL check flagged three borderline links in the notes (below); they were not opened. No personal data was sent in any request, and no contact detail from any page is reproduced here.

Not re-run: the wpt.fyi pass rates and file counts (US 3, AN 3, RG 1.11, 2.8, 7.4). They are the researchers' computations from wpt.fyi summary files at a stated revision; nothing found contradicts them, and they are graded on the note's evidence ("note").

Verdicts:
- **Correct.**
- **Partly**: the substance holds, but a number, scope, date, attribution or wording is off.
- **Wrong.**
- **Unverifiable**: the cited source does not contain the claim, or could not be reached, and no other source was found.

**Must-fix (FC-M#)** marks a wrong or unsupported fact that a decision, target or rule depends on, or a plain error in a load-bearing sentence. **Recommended (FC-R#)** marks every other fix.

## Summary

| Verdict | Count |
|---|---|
| Correct | 121 |
| Partly | 17 |
| Wrong | 5 |
| Unverifiable | 3 |
| **Total claims** | **146** |

The total is the 134 numbered claims plus the 12 cross-reference rows graded wrong or partly (X1–X12: 4 wrong, 8 partly). The 52 cross-references found correct are listed in one paragraph and not counted.

Must-fix: 11. Recommended: 24.

The research notes are careful, and the spec mostly reflects them faithfully. The numbers that carry the design (68.5% inline SVG, 57.5% SVG images, 2.90% SMIL, 1.67% SMIL in images, 891 of 1,405 three-engine passes, the SVG 2 CR of 6 October 2026, the Blink intent's 0.0403%, the 2016 suspension quote, Kaspersky's 2,825, KnowBe4's 6.6%, the librsvg caps, CVE-2016-9079 in CISA KEV) all check out. The serious problems are:
1. **The `use` shadow tree changed after the CR.** The CR says "open (inspectable by script), but read-only"; the Editor's Draft of 13 October 2026 records "Made the use-element shadow tree closed instead of open and read-only". SV-D5, 6.2, 6.7 and the DOM amendment rest on the CR wording (FC-M1).
2. **"Sixty-fold" SMIL growth** compares 2.90% with a counter value (0.047%) that is not the figure Blink cited (0.0403%), across a counter that is discontinuous in 2015–2017 (FC-M2).
3. **Five Izcal cross-references point at the wrong rule or claim a rule that does not exist** (TX G26, ST G31, GR-D17, the DOM spec "already" blocking top-level `data:`, and the anchors table's SVG 2 section numbers) (FC-M3 to FC-M7).
4. **The 2025 SVG clickjacking work is not a timing attack** and does not leak pixels to script; the spec groups it with filter-timing leaks (FC-M8).
5. **librsvg's caps are not tied to CVE-2019-20446** by any source, and that CVE is about nested patterns (FC-M9).
6. **The spec has one external inline link.** Phase 7 requires inline primary links for every claim a decision or target depends on (FC-M10).
7. **The SVGZ bomb limits** are presented as the networking spec's when NW gives no numbers (FC-M11).

Links: 1 external inline link in the spec (live); 26 reference definitions, all resolving to existing files (2 unused). The spec has no `#fragment` links. In the notes, 372 unique URLs: 6 dead or unreachable, 3 borderline under the source-code rule, 0 clear source-code links; 127 fragments checked, 0 missing.

## Claims

Abbreviations: Sum = the summary before Section 1; S1 to S11 = spec sections. "Note" in the source column means the claim was checked against the research note only.

### Usage and interoperability

| # | Claim (spec) | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 1 | Inline `<svg>` on 68.5% of Chrome page loads | Sum; 3 | Correct | chromestatus bucket 250, 2026-10-18: 68.52% | — |
| 2 | At least one SVG image decoded on 57.5% | Sum | Correct | Note US 1.4 (bucket 5492) | Not re-fetched |
| 3 | Inline SVG grew from 10.1% on 1 January 2015 | 3 | Correct | Bucket 250, 2015-01-01: 10.06% | — |
| 4 | SVG is 6.4% of image requests on mobile | 3 | Correct | Note US 1.5 (Web Almanac 2024 Media) | — |
| 5 | `filter: url()` on 6.19% of page loads | 3; 6.5 | Correct | Note US 2.8 (CSS bucket 1014) | — |
| 6 | `foreignObject` 8.24%, from 0.32% in January 2025, cause unknown; 0.59% inside images | 3; 6.8; Q-SV-12 | Correct | Note US 2.5 | — |
| 7 | External SVG resources on 4.49% | 3 | Correct | Note US 2.7 | Counter semantics Medium, as 11.2 says |
| 8 | Lighting, convolution, tile filters, `SVGPathSeg`, `xml:base`, SVG fonts, `<discard>` near zero | 3 | Correct | Note US 2.9–2.12, 4.14 | — |
| 9 | SMIL elements on 2.90% of page loads (October 2026) | 6.9; 2.1 | Correct | Bucket 501, 2026-10-18: 2.896% | — |
| 10 | SMIL "sixty times the 0.047% that led Blink to propose deprecating it" | 6.9; 2.2 ("grew sixty-fold"); Sum | **Partly** | Bucket 501; blink-dev intent | Blink's intent cited **0.0403%** of page views (→ about 72×). 0.047% is the counter on 1 January 2015. The counter is discontinuous: 0.049% (28 April 2015), 0.51% (1 September 2015), 0.46% (1 January 2016), 0.089% (1 January 2017), 0.58% (1 January 2019). FC-M2 |
| 11 | SMIL in SVG images on 1.67% | 6.9 | Correct | Note US 2.3, AN 6.2 (bucket 768) | — |
| 12 | `SVG1DOMUriReference` 1.91%, `ObsoleteSVG1DOMFilter` 1.26% | 6.2 | Correct | Note US 2.17 | — |
| 13 | `<discard>` use counter 0.0008% | 6.9 | Correct | Note US 2.4, AN 3.6 | — |
| 14 | `svg/` directory 1,411 files; 87.2% Chrome, 88.4% Firefox, 73.3% Safari, 49.5% Ladybird, 24.4% Servo | 3 | Correct | Note US 3.1 | Not re-run |
| 15 | Only 891 of 1,405 files (63%) pass in all three engines | Sum; 3.1 | Correct | Note US 3.18 (891/1405 = 63.4%) | 1,405 is the Chrome run's count, 1,411 the directory's; say so once (FC-R14) |
| 16 | `svg/path` 54.7% Chrome, 38.3% Safari | Sum | Correct | Note US 3.10 | — |
| 17 | Path, text, linking and styling are the weak directories | 3 | Correct | Note US 3.4–3.10 | — |
| 18 | Vendors turned down whole-SVG Interop proposals four years running | Sum | Correct | Note US 5.2–5.6 (2023, 2024, 2025, 2026) | — |
| 19 | "SVG issues" the top content pain point in State of HTML 2024 | Sum | Correct | State of HTML 2024: 73 of 398, next "Image management" 68 | — |
| 20 | SVG 1.1 test suite not automated in WPT | 3 | Correct | Note US 3.19 (Interop #214) | — |
| 21 | Safari lacks `d` and `context-fill`; only Firefox ships `getBBox` options and `getPathData`; none wraps text | 3; 6.2; 6.6 | Correct | Note US 4.1, 4.2, 4.10, 4.11; RG 1.11 | — |
| 22 | Chrome and Safari fail subtests in six of ten `getBBox` files | S1 table | Correct | Note RG 2.8 (Chrome 03, 05–09; Safari 02, 05, 06, 07, 09, 10) | — |
| 23 | Safari fails 68 of 321 `svg/animations` files, clustered in syncbase and event timing | S1 table | Correct | Note AN 3.2 | Path interpolation is the other cluster; fine |
| 24 | `use` without a fragment shipped in Chrome 137; on the open Interop 2027 proposal | 6.7 | Correct | Note US 4.16, 5.7 (#1469) | — |
| 25 | `getPathData` on the open Interop 2027 proposal; Firefox 137; Blink intent to prototype 5 June 2026 | 6.2; SV-D13 | Correct | Note HP 3.5 (I2P date Medium), US 4.11, 5.7 | — |
| 26 | Published SVG ceilings "a few thousand individual marks"; no engine publishes element-count scaling | S1 table; 6.16 | Correct | Note GO 3.1, 3.4 | — |
| 27 | One study found SVG and canvas about equal; a micro-benchmark found canvas ten times faster for 1,000 objects | 6.16 | Correct | Note GO 3.2–3.3, US 7.4 (Medium/Low, snippets) | Grade of the evidence is Low; fine as background |
| 28 | Chrome composites SVG animations since version 89 | 6.16 | Correct | Chrome blog: "As of Chromium 89 … hardware-acceleration by default on SVG animations" | — |
| 29 | SVG favicons rendered secure static; favicon in the secure-static row | 6.1 table | Correct | Note US 8.1 | — |

### History and postmortems

| # | Claim (spec) | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 30 | 1998: PGML (Adobe, IBM, Netscape, Sun) and VML (Microsoft, Macromedia, HP, Autodesk, Vision); SVG WG chartered 24 July | 2.1 | Correct | Note HP 1.1–1.2 | Submissions Low (Wikipedia); charter date High |
| 31 | VML stayed Microsoft's only vector format until IE9 | 2.1 | Correct | Note HP 1.1, 2.2 | Low |
| 32 | Adobe SVG Viewer end of support 1 January 2009, not on Vista, no official reason | 2.1 | Correct | Note HP 2.1 (Medium) | — |
| 33 | SVG 1.0, 1.1, Tiny and Basic 2001–2003; 3GPP mandated Tiny; Tiny 1.2 (2008) never implemented whole | 2.1; 5 | Correct | Note HP 1.3–1.4 | Medium/Low, recollection-based in parts |
| 34 | SMIL Animation (2001) still the normative definition SVG animations defer to | 2.1; 3 | Correct | SVG Animations L2 ED (downloaded): "the normative definition … is the SMIL Animation specification" | — |
| 35 | Native support KSVG 2004, Opera 8 2005, Firefox 1.5 2005, WebKit 2006, IE9 2011 | 2.1 | Correct | Note HP 2.2 (Low, Wikipedia) | — |
| 36 | Inline SVG via the HTML5 parser 2010–2011 | 2.1 | Correct | Note HP 2.3 (caniuse) | — |
| 37 | Firefox 2013 SVG text on the HTML text engine: 50 patches, decorations, find, selection, bidi | 2.1; 5 | Correct | McCormack 2013: "a series of 50 patches", bug 655877, "reuse the same text layout and rendering code that it uses for HTML" | — |
| 38 | Quote "greatly non-interoperable and somewhat underspecified" | 2.1 | Correct | Note RG 1.15 (2011 www-svg, Medium, snippet) | — |
| 39 | SVG fonts removed from Chrome 38 because Microsoft would implement neither SVG fonts nor SMIL; low use | 2.1 | **Partly** | Note HP 3.1 | The Microsoft account is the WG's 2015 recollection; "low use" was Mozilla's 2010 objection. Chrome's own reason is not documented. FC-R9 |
| 40 | SVG fonts never in Firefox, IE or Edge; still in Safari; removed from SVG 2 | 2.1; SV-D7 | Correct | Note HP 3.1, US 4.25; SVG 2 ED changes | — |
| 41 | SVG Integration first draft 2014, defines processing modes, never advanced | 2.1; 3 | Correct | SVG Integration on w3.org: "W3C First Public Working Draft 17 April 2014", five modes | — |
| 42 | Blink's intent to deprecate SMIL, 29 April 2015, at 0.0403% of page views | 2.1 | Correct | blink-dev: "low but stable at 0.0403% of pageviews"; 0.006% inside images | — |
| 43 | Blink's reasons: "significant complexity", tear-offs, Oilpan | 2.1; 6.9 | Correct | blink-dev: "SMIL adds significant complexity to Blink"; tear-off rewrite and Oilpan named | — |
| 44 | Suspended in August 2016: "use cases … that just don't have high-fidelity replacements yet" | 2.1; 6.9; 11.2 | Correct | www-svg 2016Aug/0037, 17 August 2016, "SVG SMIL news from Google": "we've decided to suspend our intent to deprecate" | The 11.2 flag (forward, original not read) stays accurate |
| 45 | SVGPathSeg removed in Chrome 48 and Firefox 59 | 2.1; 6.2 | Correct | Note HP 3.5, US 4.12 (MDN data) | — |
| 46 | `getPathData` "took nine years" (2016–2025) | 5 | Correct | Note HP 3.5 | — |
| 47 | librsvg's port from C to Rust 2016–2020 | 2.1 | Correct | Note HP 4.1 (maintainer's posts) | — |
| 48 | librsvg's property model "never … matched the SVG/CSS specs" | 2.1; 5; 6.1 option (c) | Correct | Note HP 4.2 (maintainer post, High) | — |
| 49 | The C line misread compact arc flags | 6.2 | Correct | Note HP 4.3 | — |
| 50 | SVG 2 re-CR 2018 cut mesh, hatch, `solidcolor` for lack of implementations | 2.1 | Correct | Note HP 1.7; SVG 2 ED changes appendix | — |
| 51 | Then eight years without a publication; CR again 6 October 2026 | Sum; 2.1 | Correct | Note HP 1.5; CR status: "W3C Candidate Recommendation Snapshot 06 October 2026" | — |
| 52 | Recommendation expected no earlier than 1 December 2026 | 2.1 | Correct | CR status: "no earlier than 01 December 2026" | — |
| 53 | Cross-origin `use` deferred 2019 | 2.1; 6.7 | Correct | Note GO 6.3 (public-svg-issues, Medium) | — |
| 54 | `data:` in `use` removed from Chrome 120 and Firefox 122 after Trusted Types and Sanitizer bypasses | 2.1; 6.7; 6.13 | Correct | Chrome blog ("expect to remove", Chrome 120, December 2023, Trusted Types and Sanitizer API); note US 4.17 (MDN data: removed in 120 and 122) | 11.2 still calls Firefox 122 and Chrome 120 unverified; US 4.17 settles both. FC-R7 |
| 55 | WebKit's layer-based engine: "the invalidation chain depended on the DOM order"; style system decides; not default as of 2024 | 2.1; 5 | Correct | WPE WebKit blog, 21 May 2024 | — |
| 56 | SVG WG rechartered for maintenance only | 2.1; 3 | Correct | Draft 2026 charter: "The mission … is to maintain SVG specifications"; "will not introduce new features" | — |
| 57 | Three at-risk features in the 2026 CR | 2.1; 3 | Correct | CR status: multiple `title`/`desc`, nested links, `vector-effect` other than `non-scaling-stroke` | — |
| 58 | SVG 2 makes `animVal` an alias; "engines called the tear-offs 'costly to maintain'" | 2.2 | **Partly** | SVG 2 ED §4.6: "an alias of baseVal"; note HP 3.4 | "Costly to maintain" was one WebKit participant at the 2015 WG meeting; Blink said "significant complexity". FC-R10 |
| 59 | SMIL "is the only animation inside SVG images" | 2.2; 6.9 option (d) | **Partly** | Note AN 3.8 | CSS Animations also run in SVG images; 6.9 itself says "SMIL and CSS animations are the only way". Fix 2.2 and 6.9 (d) to match. FC-R11 |
| 60 | Blink's SMIL deprecation "proposed at 0.04%" | 5 | Correct | blink-dev 0.0403% | — |
| 61 | SVG 2's text pseudo-code has five defects (`new_check_count`, `resolve_dx` from `dy`, `lrt`, index 1, undefined `i`/`k`) | Sum; 5; 6.10; Q-SV-1 | Correct | SVG 2 ED (13 October 2026) text chapter: `new_check_count`, "set resolve_dx[index + j] to dy[i]", `lrt`, "Set index = 1" all still present | The RG note dates these rows to the 2018 CR; they persist in the 2026 ED. FC-R15 |
| 62 | `inline-size`: no engine implements it; WPT reftests fail in all three; open issue #1087 (April 2026) proposes removing it | 6.10 | Correct | svgwg #1087 (7 April 2026): "Drop inline-size attribute … This is not implemented"; note RG 1.11 | — |
| 63 | Blink SVG text on LayoutNG claims "not used" | 11.2 | Correct | Note HP 3.10, RG 1.16 | Properly excluded |

### Specification statements

| # | Claim (spec) | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 64 | Four switchable features; disabled external references "as if a network error occurred" | 6.8; 6.13 | Correct | SVG 2 ED §2.2 (conform) | 6.8's "load anything but `data:` URLs" also omits same-document references, which stay allowed; fine in context |
| 65 | Disabled animation is not the same as pausing at 0 s | 6.9 (SV-D16) | Correct | SVG 2 ED §2.2: "This is not the same as pausing the document's animated state at 0s" | — |
| 66 | `img`/CSS images secure animated, or secure static where the context cannot animate | 6.1 table | Correct | SVG 2 ED §2.3 ('image' references and HTML `img` "expected") | — |
| 67 | Resource documents secure static in SVG 2; SVG Integration says secure animated and asks the question | 6.1 table; 3; 11.2 | Correct | SVG 2 ED §2.3: `use` and paint-server cross-references and style-property references "must be processed in secure static mode"; Integration FPWD: resource document mode "must use the secure animated processing mode"; note SS 1.8 for the ED's question | — |
| 68 | OT-SVG glyph documents secure static | 6.1 table | Correct | SVG 2 ED §2.3: fonts secure animated if animated glyphs are supported, else secure static; TX 6.3 disables animation | — |
| 69 | Presentation attributes are author-level, specificity 0, before other author declarations | 6.2 | Correct | SVG 2 ED §6.6: "contribute to the author level of the cascade, followed by all other author-level style sheets, and have specificity 0" | — |
| 70 | Geometry properties are presentation attributes only on listed elements; not `x` on `text` or `r` on `radialGradient` | 6.2 | Correct | Note SS 2.3 | — |
| 71 | `d` is `none \| <string>`; new `path-length` | 6.2 | Correct | Note SS 2.4; ED changes ("path-length … &lt;length&gt; per CSSWG resolution", May 2026) | — |
| 72 | SVG 2 has 60 interfaces and 5 mixins | 6.2 | Correct | Note SS 2.7 | — |
| 73 | `baseVal`/`animVal` objects keep identity; list items live and detach on removal | 6.2 | Correct | Note SS 2.11–2.12 | — |
| 74 | `animVal` "an alias of baseVal" | 2.2; 6.2; SV-D4 | Correct | SVG 2 ED §4.6: "animVal no longer represents the current animated value and is instead an alias of baseVal" | — |
| 75 | `SVGElementInstance`, `instanceRoot`, `getTransformToElement` removed | 6.2 | Correct | Note HP 3.6; ED changes | — |
| 76 | HTML parser: case adjustments; `foreignObject`, `desc`, `title` are HTML integration points | 6.3 | Correct | Note SS 7.1–7.2 | — |
| 77 | Path data renders up to the command with the first error; complete segments of a short command render | 6.2; 6.17 | Correct | Note RG 2.1 (SVG 2 §9.5.4) | — |
| 78 | Percentages: normalized diagonal √((w² + h²)/2) | 6.4 | Correct | Note SS 3.1, RG 3.2: √(w² + h²)/√2, which is equal | — |
| 79 | `auto-start-reverse` reverses the first marker | 6.4 | Correct | Note SS 5.1 | — |
| 80 | DOM exposes geometry as `float` and `double` | 6.6 | Correct | Note RG 3.4 | — |
| 81 | SVG 2 requires double precision for CTMs | 2.2 | Correct | Note RG 3.3 (conform §2.5.5–2.5.6) | — |
| 82 | `getBBox(options)` with `fill`, `stroke`, `markers`, `clipped` | 6.6 | Correct | Note SS 2.8 | — |
| 83 | `isPointInFill`/`isPointInStroke` in user space; `isPointInStroke` includes dashes | 6.6; 6.11 | Correct | Note SS 2.9, RG 2.4 | — |
| 84 | Elliptical arcs to centre form with radii scaled up per implementation notes | 6.6 | Correct | Note RG 2.2 (Appendix B.2.4–B.2.5) | — |
| 85 | `use` shadow tree "open (inspectable by script), but read-only", `NoModificationAllowedError` | 6.7; SV-D5; 9.5 DM row | **Partly** | CR (w3.org): "The shadow tree is open (inspectable by script), but read-only"; **ED of 13 October 2026**: "The shadow tree is closed (not inspectable by script): its shadow root's mode is 'closed'", and the changes appendix: "Made the use-element shadow tree closed instead of open and read-only" | Correct for the pinned CR; superseded in the ED a week later. FC-M1 |
| 86 | Instances inherit from `use`; outer rules do not match; sheets cloned; `:hover` per instance | 6.7 | Correct | Note SS 4.4 | — |
| 87 | Listeners mirrored; events retargeted; SMIL mirrored; Web Animations via `ShadowAnimation` | 6.7 | Correct | SVG 2 ED §5.5.4–5.5.5 (`ShadowAnimation` read-only; `NoModificationAllowedError` on listeners) | — |
| 88 | Templates without children clone children "like `use`" | 6.7 | Correct | Note SS 5.2 ("no non-descriptive children") | — |
| 89 | Referencing an ancestor is an invalid circular reference; indirect cycle "in error and must not be rendered" | 6.7 | Correct | SVG 2 ED §5.5: "(shadow-including) ancestor … invalid circular reference and the 'use' element is in error" | — |
| 90 | SVG 2 sets no limit on `use` depth or instance count | Sum; 2.2; 6.7; 6.13 | Correct | Note SS 4.7 | — |
| 91 | SVG 2 "does not allow cross-origin and data URL resource requests in 'use'" | 4.4; 6.7 | Correct | SVG 2 ED §5.5 (exact words) | — |
| 92 | Fetch modes: "anonymous" for style properties and presentation attributes; CSS Masking anonymous with network error → `none`; animation `href` same-document | 6.8; SV-D11 | Correct | SVG 2 ED §16.1.7 ("For a reference from a style property or presentation attribute, the 'anonymous' state"); note SS 4.9, 5.3 | — |
| 93 | Resource documents immutable; missing `id` invalid | 6.8 | Correct | Note SS 4.8 | — |
| 94 | Invalid references: no clip; transparent-black mask; filter chain ignored; no marker; paint fallback | 6.8 | Correct | Note RG 4.4 | — |
| 95 | Mask luminance coefficients conflict (0.2125/0.7154/0.0721 against Rec. 709) | 3; Q-SV-5 | Correct | Note RG 4.5 | — |
| 96 | Filter Effects' constant-time rule for tainted inputs | 5; 6.13; SV-16 | Correct | Filter Effects 1 (WD 18 December 2018): operations "must be implemented in such a way that they always take the same amount of time regardless of the pixel values" when an input is tainted | — |
| 97 | SVG Animations Level 2 is an ED that defers to SMIL; Web Animations says it is "intended" | 3; 6.9 | Correct | SVG Animations L2 ED (downloaded); note SS 6.5 | — |
| 98 | `<discard>` is in neither specification; both engines that shipped it withdrew it | 6.9 | Correct | Note SS 6.2; US 4.14 (Chrome removed 81, Firefox 136–138 then removed) | — |
| 99 | `setCurrentTime` takes a `float` in seconds | 6.9 | Correct | Note AN 5.4 | — |
| 100 | Media fragment `t=` seeks "as if setCurrentTime" | 6.9 | Correct | Note SS 3.5 | — |
| 101 | Fragment timeline begins at the SVG load event or `timelinebegin="loadbegin"` | 6.9 | Correct | Note AN 5.2; SS 6.3 | — |
| 102 | No specification says when an SVG image's timeline starts or whether two `<img>` share it | 6.9; Q-SV-4 | Correct | Note AN 3.9 (absence) | — |
| 103 | `textPath`: `path` attribute, `side="right"` reverses, `startOffset` negative or above 100%, mid-point rule, one loop on a closed subpath | 6.10 | Correct | Note SS 5.7, RG 1.7 | — |
| 104 | `method="stretch"`, `spacing="auto"` have no algorithm; "free to make any additional adjustments" | 6.10 | Correct | Note RG 1.8 | — |
| 105 | `shape-inside` removed from SVG 2 | 6.10 | Correct | Note SS 5.6 | — |
| 106 | `glyph-orientation-vertical` alias (`auto`→`mixed`, `0deg`→`upright`, `90deg`→`sideways`) | 6.10 | Correct | Note RG 1.12 | — |
| 107 | Search and selection of SVG text a conformance requirement in browsers | 6.10 | Correct | Note RG 1.14 (conform) | — |
| 108 | `pointer-events`: "all eleven values" | 6.11 | Correct | SVG 2 ED §15.6: `auto`, `bounding-box`, `visiblePainted`, `visibleFill`, `visibleStroke`, `visible`, `painted`, `fill`, `stroke`, `all`, `none` | — |
| 109 | Markers never hit; text by character cell; images by rectangle | 6.11 | Correct | Note RG 7.2 | — |
| 110 | SVG-AAM a Working Draft dated 12 October 2026 | 3 | Correct | TR: "W3C Working Draft 12 October 2026" | — |
| 111 | SVG-AAM inclusion triggers, exclusion, "hidden" = invisible and not pointer-interactive, `use` shadow content as children | 6.12 | Correct | SVG-AAM ED (`#include_elements`, `#exclude_elements`, `#mapping_use_shadow_tree`); note SS 8.3–8.5 | Never-rendered elements are excluded in the AAM's general rule ("never directly rendered"), not as a list; fine |
| 112 | `svg` → `graphics-document`, shapes → `graphics-symbol` when included | 6.12 | Correct | Note SS 8.2 | — |
| 113 | OpenType `SVG ` table pins SVG 1.1 and discourages `context-fill`; animation optional | 3; Q-SV-15; 9.5 TX row | Correct | Note SS 7.7 | — |
| 114 | `systemLanguage` and `requiredExtensions` named in SVG 2's privacy note | 6.14 | Correct | Note SS 7.6 | — |
| 115 | SVG Integration sizing: 300 × 150 default without size or `viewBox` | 6.8 | Correct | Note SS 1.9 | — |
| 116 | The SVGZ magic-number rule "is not in a specification" | 6.3; SV-D8 | **Partly** | SVG 2 ED conform: implementations "must correctly support gzip-encoded … and deflate-encoded data streams, for any content type"; note SS 7.7 (OpenType SVG documents may be gzip, `1F 8B 08`) | The OpenType table does define gzip detection by magic bytes for glyph documents, and SVG 2's requirement is broader than HTTP. Reword. FC-R12 |
| 117 | Anchors table: grammars §4, §9.3–9.5, §8; presentation attributes §6.6; viewports §8; text §11; hit testing §15.5–15.6; templates §14.1.1; URL processing §16.1.7 | 7 | Correct | SVG 2 ED table of contents | — |
| 118 | Anchors table: "§4.5.2 reflection rules"; "`use`, templates: §5.5–5.7"; "Processing modes: §2.3–2.4" | 7 | **Wrong** | SVG 2 ED table of contents | §4.5.2 is "Interface SVGLength"; reflection is §4.3.4–4.3.6 and §4.6. §5.6 is conditional processing and §5.7 `desc`/`title`; `use` is §5.5 alone. §2.4 is document conformance classes; processing modes are §2.2–2.3. FC-M7 |

### Security and privacy

| # | Claim (spec) | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| 119 | CVE-2016-9079: SMIL timing use-after-free, exploited in the wild against Tor Browser | Sum; 5; 6.13 | Correct | CVE.org: "use-after-free vulnerability in SVG Animation … exploit … in the wild targeting Firefox and Tor Browser users"; CISA KEV added 2023-06-22 | Root cause (list mutated by a callback) rests on Qualys, Medium |
| 120 | CVE-2012-5112 and CVE-2013-0912 as SVG engine bugs | 6.13 | Correct | CVE.org: CVE-2012-5112 "Use-after-free … in the SVG implementation in WebKit"; CVE-2013-0912 "type confusion" (SVG per Pwnies, note SR 1.4) | — |
| 121 | `use` instance trees and resource client lists recur in engine bugs | Sum; 5; 6.5 | **Partly** | Note SR 1.7 | The note says classes (a) and (c) are "inferred from the history of the bugs, not from a published root-cause study" (Low). Say "inferred". FC-R13 |
| 122 | Nested `<use>` is SVG's "billion laughs" | Sum; 6.7 | Correct | Note SR 2.1 (librsvg #323 "similar to a billion laughs attack") | — |
| 123 | librsvg: "no total budget; caps added after CVE-2019-20446"; "CVE-2019-20446 and its 1,000,000-element and 500,000-reference caps" | 5; 6.13 | **Unverifiable** | librsvg security page: caps of 1,000,000 elements and 500,000 references and "no built-in limits on the total amount of memory or CPU time" confirmed; the page lists CVE-2019-20446 ("guard against exponential growth of CPU time") but "does not say when the limits were added or tie them … to this CVE"; CVE.org: CVE-2019-20446 is **nested patterns** | The caps and the no-total-budget quote are correct; the causal "added after" is unsupported. FC-M9 |
| 124 | librsvg CVE-2023-38633: two URL parsers disagreed and read local files | 2.1; 5; 6.13 | Correct | CVE.org: directory traversal in the URL decoder (`.?../../…/etc/passwd`); note SR 7.3 (Canva) | — |
| 125 | SVG filters have leaked cross-origin pixels for a decade; filter timing "from 2013 to 2025"; threat row lists the 2025 SVG clickjacking logic gates as pixel stealing through filter timing | Sum; 2.2; 6.13 | **Partly** | Stone 2013 to GPU.zip 2023 confirmed by note SR 3.1–3.6; lyra.horse (4 December 2025): its technique works "as long as it is not timing-based", and pixel data cannot reach script; results only drive the rendered output | Timing leaks run 2013–2023. The 2025 work reads framed cross-origin pixels inside the filter for clickjacking, without timing. GR-D18 answers it; the constant-time rule does not. FC-M8 |
| 126 | `data:` in `use` bypassed Trusted Types and the Sanitizer API | 5; 6.13 | Correct | Chrome blog | — |
| 127 | PortSwigger animated-`href` vector; sanitizer bypasses into 2026 (CVE-2026-31807) | 6.9; 6.13; 11.2 | Correct | CVE.org CVE-2026-31807 (SiYuan SVG sanitizer); note SR 4.2 | 11.2's "single-source, aggregator" can go: the CVE record exists. FC-R7 |
| 128 | DOMPurify CVE-2020-26870, namespace confusion on reparse | 5; 6.13 | Correct | CVE.org: "serialize-parse roundtrip does not necessarily return the original DOM tree, and a namespace can change from HTML to MathML" | The note's Securitum link is dead (404). FC-R2 |
| 129 | SVG as "fully functional, one-file web applications" | 6.13 | **Unverifiable** | Note SR 4.1 (abstract, Medium) | The RUB page the note cites refuses connections from this host; the quote could not be re-read. Cite the ACM CCS 2011 DOI (FC-R2) |
| 130 | Kaspersky: 2,825 SVG phishing emails in Q1 2025 | S1 table; 6.13 | Correct | Securelist, 21 April 2025: "We found 2,825 of these emails in just the first quarter" | — |
| 131 | KnowBe4: SVG 6.6% of malicious attachments | S1 table; 6.13 | Correct | KnowBe4: "6.6% of malicious attachments", 1 January to 5 March 2025, +245% on Q4 2024 | — |
| 132 | All engines taint a canvas for an SVG image with `foreignObject` "because its HTML could render user-specific state: `:visited`, autofill colours, spelling marks" | 6.8; SV-D10 | **Partly** | Note SR 5.3–5.5 | Tainting by all engines over HTTP is Medium (release notes); the list of user-state sources is the researcher's synthesis (SR 5.5, Low), and Gecko did not taint `blob:` SVG. Say so in 6.8 and 11.2. FC-R13 |
| 133 | Top-level `data:` SVG excluded from Firefox's `data:image/*` exemption | 6.13 | Correct | Note SR 4.6 (Mozilla Security Blog, High) | The second half of the sentence is FC-M6 |
| 134 | Font-metrics fingerprinting made 34% of over 1,000 browsers unique | 6.14 | **Unverifiable** | Note GO 4.1 (Medium) | The cited page (blues.cs.berkeley.edu) times out from this host; the paper (Fifield and Egelman, FC 2015) should be cited by its proceedings DOI. FC-R2 |

### Izcal cross-references

64 cross-references were checked (helper sweep of local files, reviewed). 52 are correct and are not counted above, among them: JS-D4 (node columns outside the cage, traced as a node space), JS G6's crate list, JS 6.2 external traced slots, JS-D12 NaN rule, JS 6.9 `hg-math` without FMA contraction, JS-D15, JS-D17; FP-11 (512 B†), FP-13, FP-17, FP-D4, FP-36, FP-38, FP-D8, FP-D19, FP 6.4 (class (a), sharing within a partition); GR 6.3 (24.8, ±8,388,607, 65,536† segments, ≤ 64† filter primitives), GR i64 Q32 matrices, GR-D3, GR-D18, G69, G70, GR 6.15, 6.16, 6.24 (`hg-svg-paint`, counted 3–4), 6.28 ("Typed records"), GR-S3, GR-M2 at 30–42 months; LY 6.5.10, LY 6.5.8, LY 6.14, LY-D3, LY G1/G41/G42, Stage 1 exit at 48–60 months; MD-D20's quoted crates, MD-S10/Q-MD-18's "without a JS agent heap", MD-D5, MD-D19, G50, G56; TX-D4, TX-D7 ("Izcal's SVG engine"), TX 6.3 (≤ 10,000† elements, no animation), Q-TX-9; ST-D5, Q-ST-19, ST G1/G21/G22/G23, ST 6.9–6.11, ST-13; DM 6.12, DM 6.18 ("SVG DOM, SMIL | Future SVG spec (G-12)"), DM G91, G98, DM 6.5; AX 6.2, G100, G103, AX 6.12, AX-1; PD-D19, PD-D20, PD G116; RN2 Q20, Q25 ("counted nowhere"), Section 4; ledger G-12, G-13, A-FUT-1, A-FUT-5; D-S1 as amended by OD-6; D-S10; the IIP (DevTools spec); G120–G129 free in every other document; the rule ranges G19–G24, G25–G29, G37–G45, G60–G72, G90–G99, G100–G109. The layout box record is 32 B (LY), as 6.4 says.

The 12 rows graded wrong or partly are counted here:

| # | Claim (spec) | Where | Verdict | Source checked | Note / fix |
|---|---|---|---|---|---|
| X1 | "a page with 50 SVG icons would pay about 38 MiB if each image had one" agent heap | S1 table; 6.1 option (b) | Partly | FP-34: "≤ 768 KiB† in total" per agent | A ceiling: write "up to about 38 MiB" (50 × 768 KiB = 37.5 MiB). FC-R5 |
| X2 | Static name indices in FP 6.3 | 6.2; 6.3 | Partly | FP 6.3 holds the name tables; the 16-bit indices are FP 6.4 ("Narrow references") | Cite FP 6.3 and 6.4. FC-R5 |
| X3 | 16† compositor-mutable cap "under GR-D13" | 6.16 | Partly | The cap is GR-D5 and GR 6.3 Layerization; GR-D13 is compositor scrolling and animation | Cite GR-D5. FC-R5 |
| X4 | "Filters never lose resolution under pressure (GR-D17)" | 6.13 | Wrong | GR-D17 is UA-defined pixels (snapping, borders, dashes, dithering, image filtering, cusp continuity); no GR text says this | FC-M5 |
| X5 | Forcing table at "LY 6.1" | 6.6; 6.7; 6.19 | Partly | The forcing table is `hg-layout-query`'s, LY 6.15 (DM 6.18 says so) | FC-R5 |
| X6 | Layout counts "the SVG boundary (2†)" | 9.3 | Partly | LY 9.3: "MathML and SVG boundary … 2" | Say it is shared with MathML. FC-R5 |
| X7 | "MD-56's re-raster latency target" | 6.8; SV-43 | Partly | MD-56 is "The LCP image is not late because of decode"; only MD-D20's reversal trigger reuses it for re-raster | FC-R5 |
| X8 | `data:` fonts in opaque images; MD row of 9.5 | 6.8; 9.5 | Partly | MD 6.1: fonts in such images are "the Izcal font set and the OS tier only" | The spec changes MD-D20 but 9.5's MD row omits it. FC-M3 |
| X9 | "TX G26 does not yet allow" `hg-fontfmt` in the media process; a G26 amendment | 6.8; 6.19; 9.5 TX row | Wrong | TX G26 lists only the broker, GPU service, network service and storage service; RN2 Q25: "TX G26 is unaffected (the media process is not listed…)" | FC-M3 |
| X10 | "ST-D20's entity budget and its 5,000† nesting cap" | 6.3 | Partly | ST-D20: 1,048,576 characters and 64 entity nesting levels; the 5,000† element-depth cap is ST 6.14 | FC-R5 |
| X11 | "`hg-url` only (ST G31)" | 5; 6.13 | Wrong | G31 is the networking spec's rule ("only `hg-url` parses URLs"); ST only refers to it | Write NW G31. FC-M4 |
| X12 | "the navigation owner (DOM spec) already blocks" top-level `data:` | 6.13 | Wrong | No report (DM, NW, ST) mentions blocking top-level `data:` navigation | FC-M6 |

Two further cross-reference observations are recorded as recommended items without a separate grade: ST-D18 is a **CSS** feature-admission rule, so applying it to SVG elements and DOM members is an extension the owner must record, not only the `getPathData` exception (FC-R6); and LY-D3's options table rejects floating-point geometry because "cross-ISA identity needs a strict discipline everywhere", which SV-D2 adopts without discussing LY's rejected option (FC-R16, for the design critique).

### Arithmetic

| Check | Spec value | Recomputed | Verdict |
|---|---|---|---|
| KLOC total (9.3) | 117–172 | Low: 4+6+6+12+25+10+6+12+5+5+15+6+5 = 117; high: 6+9+9+18+35+14+9+18+8+7+22+9+8 = 172 | Correct |
| Engineer-years at 3–4 KLOC per year | 29–57 | 117/4 = 29.3; 172/3 = 57.3 | Correct |
| Track total (9.3) | 15–18 | 3+3+2+1+2+2+1+1 = 15; 4+4+2+2+2+2+1+1 = 18 | Correct |
| "Roughly four years from SV-M0 to Stage 1 exit" | ~4 years | SV-M0 at month 9†, exit at 48–60 months: 39–51 months (3.3–4.3 years) | Correct |
| 50 × 768 KiB | ≈ 38 MiB | 38,400 KiB = 37.5 MiB | Correct (as a ceiling, X1) |
| SMIL growth | 60-fold | 2.896/0.047 = 61.6; 2.896/0.0403 = 71.9; 2.896/0.577 (2019) = 5.0 | Arithmetic right, baseline wrong (#10) |
| Three-engine pass share | 63% | 891/1,405 = 63.4% | Correct |
| f64 range and precision | ±1.8×10³⁰⁸, 53 bits | IEEE-754 binary64 | Correct |
| Q32.32 range; bits kept at 10⁻⁵ | ±2.1×10⁹; about 15 bits | 2³¹ = 2.147×10⁹; 10⁻⁵ × 2³² = 42,950 ≈ 2^15.4 | Correct |
| f32 precision | 24 bits | binary32 significand | Correct |
| Ledger limits | 131,072; 1,048,576; 16,777,216 | 2¹⁷; 2²⁰; 2²⁴ | Correct |
| Normalized diagonal | √((w² + h²)/2) | = √(w² + h²)/√2 | Correct |
| Kaspersky 2,825 in Q1; KnowBe4 +245% | as stated | 6.6% against 1.9% in Q4 2024 = +247% (KnowBe4 rounds to 245%) | Correct |

## Links

### In the spec

- **External inline links: 1.** `https://www.w3.org/TR/svg-integration/` (6.0) is live (200 to plain HTTP; content read through the fetch tool: the 2014 FPWD). It is a "latest" URL; pin the dated one (FC-R8).
- **Reference definitions: 26**, all resolving to existing files in `reports/` or `research_notes/`. `[RN]` and `[LA]` are defined but never used (FC-R14).
- **Fragments: 0.** The spec has no `#fragment` link, so none can be missing. That is itself the problem in FC-M10: the reader cannot reach a primary source for any decision without opening a note.

### In the research notes

- **372 unique URLs** (459 occurrences): animation_smil 47, gaps_opportunities 59, history_postmortems 41, rendering_text_geometry 91, security_robustness 54, specs_standards 112, usage_interop 54, frame 0.
- **Fragments checked: 127, missing: 0.** 118 SVG 2 (single-page and chapter), svgwg.org Integration and Animations fragments, against the downloaded Editor's Drafts (single-page `chapter-id` anchors mapped to the chapter files); 9 SVG-AAM fragments against the SVG-AAM ED. Filter Effects, CSS Masking, HTML and Web Animations fragments (about 27) were not checked (w3.org blocks downloads; the fxtf drafts moved).
- **Dead or unreachable (6):**

| URL | Note, row | Status | What it supports |
|---|---|---|---|
| securitum.com …/mutation-xss-via-namespace-confusion-dompurify | SR 4.4 | 404 | DOMPurify 2020; CVE.org confirms the CVE |
| www.nds.ruhr-uni-bochum.de/research/publications/SVG-security-risks | SR 4.1, 5.2 | Connection refused | Crouching Tiger quote used in 6.13 (#129) |
| blues.cs.berkeley.edu/?p=152 | GO 4.1 | Timeout | 34% font-metrics figure used in 6.14 (#134) |
| brutelogic.com.br/blog/chrome-xss-auditor-svg-bypass | SR 4.3 | DNS failure | XSS Auditor bypass (background) |
| wpml.org/forums/topic/svg-text-not-translating | GO 2.7 | 404 | Translation tools and SVG text |
| phabricator.wikimedia.org/T231143 | GO 2.7 | 502 | Probably transient |

w3.org and lists.w3.org links answer 403/429 to browser-like clients from this host (bot protection) and were confirmed live by plain HTTP or the fetch tool. Two chromestatus URLs in US's method notes are `<id>` templates, not links.

### Source code (M-1 part 3)

- **Spec: 0 links to source code.**
- **Notes: 0 clear source-code links.** Three borderline links, none opened:
  1. `dvcs.w3.org/hg/svg2/rev/c144d4186eeb` (GO 1.4, "SVG2 ACTION-3581") is a **Mercurial changeset** of the SVG 2 specification repository. It is specification text, not engine code, but it is a commit, which the owner's rule names. Replace with the WG minutes or action page (FC-R1).
  2. `github.com/herrstrietzel/svg-getpointatlength` (RG 2.5) is the root of a **code library's repository**, cited for its README claim that engines disagree. Drop it or cite the claim as unsourced; the bug and the WG thread already carry the row (FC-R1).
  3. `github.com/linebender/resvg` (HP 1.4, 2.4, 4.4) is a repository root cited for README prose (the "900 pages" quote, the scope, the ~1,600 regression tests). It renders the README, but it is a code repository page; link the published README or documentation page if one exists, or record the exception (FC-R1).
- The notes' own disclosures of unopened source results (Chromium code reviews and commits, a librsvg source file, a Firefox header, WebKit changesets, source-derived third-party pages) are present in the headers of HP, SR, RG and GO and match. None of those URLs appears in any file.
- One CVE description quoted through the notes names a source file (`xml.rs`, in CVE-2019-20446's record). It is not a link and nothing rests on it.
- Section 7's sentence "No link in this spec or its research notes points to source code of any project (code files, headers, commits, diffs or code reviews)" is therefore **partly** true until item 1 is replaced.

## Must-fix

- **FC-M1 (6.7, SV-D5, 6.2, 9.5 DM row, 11.2).** The `use` shadow tree's mode changed after the CR: the SVG 2 Editor's Draft of 13 October 2026 says "The shadow tree is closed (not inspectable by script): its shadow root's mode is 'closed'", and its changes appendix records "Made the use-element shadow tree closed instead of open and read-only". The CR of 6 October 2026 still says open and read-only. Add a **Conflicting** entry to 11.2 (CR open/read-only against ED closed), add an open question (which mode Izcal ships at SV-M1, measured black-box against the three engines), make 6.7's design state that the root's mode is one flag on the user-agent shadow root kind, and adjust the DM amendment ("a read-only user-agent shadow root kind with SVG 2's rules" → "…with the mode the pinned SVG 2 text gives; the ED now says closed"). `correspondingElement`, `ShadowAnimation` and the `NoModificationAllowedError` rules for listeners and animations remain in the ED.
- **FC-M2 (Sum, 2.1 SMIL row, 2.2 "SMIL is on its way out", 6.9 "The evidence").** Replace "sixty times the 0.047% that led Blink to propose deprecating it" and "Use grew sixty-fold" with: "2.90% of page loads in October 2026, against the 0.0403% of page views Blink's 2015 intent cited; the counter is discontinuous in 2015–2017, and has risen steadily from 0.58% (January 2019) to 2.90%." Cite the intent and the chromestatus timeline inline.
- **FC-M3 (6.8, 6.19, 9.5 TX and MD rows, SV-D12).** TX G26 does not cover the media process (RN2 Q25: "TX G26 is unaffected"). The constraint on `data:` fonts in opaque SVG images is MD-D20's rule ("the Izcal font set and the OS tier only") and the read-only font endpoint. Retarget the request to the MD owner, move it from the TX row to the MD row of 9.5, and rewrite SV-D12's "through a TX G26 amendment".
- **FC-M4 (5 librsvg CVE-2023-38633 row, 6.13 URL-parser row).** "ST G31" → "NW G31" (the networking spec's `hg-url` rule).
- **FC-M5 (6.13 "Filter regions are exact").** Delete "(GR-D17)": GR-D17 is UA-defined pixels. State "filters never lose resolution under pressure" as this spec's rule and add it to the GR amendment in 9.5, or drop it.
- **FC-M6 (6.13 "Top-level `data:` SVG").** No Izcal spec blocks top-level `data:` navigation today. Replace "the navigation owner (DOM spec) already blocks it" with a DM (or NW) amendment in 9.5, citing Chrome 60 and Firefox 59 (SR 4.6).
- **FC-M7 (7 Anchors table).** SVG 2 section numbers: DOM "§4.5.2 reflection rules" → §4.3.4–4.3.6 and §4.6; "`use`, templates §5.5–5.7, §14.1.1" → §5.5, §14.1.1; "Processing modes §2.3–2.4" → §2.2–2.3. These anchors are what G9 makes algorithms cite.
- **FC-M8 (Sum, 2.2 filters row, 6.13 pixel-stealing row, 5 lessons row).** The 2025 SVG clickjacking work (lyra.horse, 4 December 2025) is not timing-based and does not get pixels to script; it reads a framed page's pixels inside a filter to drive what is drawn. Write "filter timing leaked cross-origin pixels from 2013 (Stone) to 2023 (GPU.zip); in 2025 filter logic over a framed cross-origin page enabled clickjacking". In the 6.13 answer, attribute the 2025 case to GR-D18 and G69 (cross-origin pixels never feed page effects), not to the constant-time rule.
- **FC-M9 (5 librsvg row, 6.13 amplification row).** No source ties librsvg's 1,000,000-element and 500,000-reference caps to CVE-2019-20446, and CVE.org describes CVE-2019-20446 as exponential growth through **nested patterns**. Write "librsvg publishes caps of 1,000,000 elements and 500,000 resolved references and 'no built-in limits on the total amount of memory or CPU time'; CVE-2019-20446 (nested patterns) and issue #323 (nested `use`) show the attack" and change the lesson from "caps added after" to "caps without a total budget".
- **FC-M10 (whole spec; phase 7 rule).** The spec has one external inline link. Add inline primary links for the claims SV-D1 to SV-D16 and the targets rest on, at least: the SVG 2 CR (dated URL) for processing modes (§2.2–2.3), the `use` rules (§5.5), `animVal` (§4.6), fetch modes (§16.1.7) and `pointer-events` (§15.6); SVG Integration (dated FPWD and ED); Filter Effects' tainted primitives; SVG Animations Level 2; SVG-AAM WD 2026-10-12; the chromestatus timelines 250, 501, 768 and 325; the blink-dev intent and the www-svg suspension forward; CVE-2016-9079 and its KEV entry; librsvg's security page; Securelist and KnowBe4; the wpt.fyi `svg/` results view.
- **FC-M11 (6.3 SVGZ, SV-D8).** "the same bomb limits as content encoding (a 1:1,024† ratio and a 64 MiB† absolute cap)": the networking spec defines a decompression budget ("output ratio and an absolute cap per destination") but gives neither number. Say these values are set here (†) and ask NW to adopt them for `image/svg+xml`, or cite NW's values once they exist.

## Recommended

- **FC-R1 (notes GO 1.4, RG 2.5, HP 1.4/2.4/4.4; spec Section 7).** Replace the `dvcs.w3.org` changeset link (a commit of the SVG 2 spec repository) with the WG minutes or tracker page; drop the `svg-getpointatlength` repository link; link resvg's README or documentation page rather than the repository root, or record the exception. Until then Section 7's "no link … points to … commits" is partly true.
- **FC-R2 (notes SR 4.1, 4.3, 4.4, GO 2.7, 4.1).** Replace dead links: Crouching Tiger by its ACM CCS 2011 DOI; Fifield and Egelman by the FC 2015 proceedings DOI; DOMPurify by the CVE record CVE-2020-26870 (or the PortSwigger Daily Swig article the note already cites); drop or archive the WPML and Brute Logic links.
- **FC-R3 (S1 "Accessibility users", 3.1 table, 6.17).** "`use` instance provenance … shown only behind a setting" (GO 1.5) and "developers bisect markup" (GO 1.6) are the researcher's recollection (Low). Mark them so in 11.2 or soften the wording.
- **FC-R4 (6.13 threat table).** The memory-safety row cites CVE-2016-9079 as "exploited"; add that CISA added it to KEV on 22 June 2023 (a stronger, checkable fact).
- **FC-R5 (cross-references X1, X2, X3, X5, X6, X7, X10).** "up to about 38 MiB"; FP 6.3 and 6.4; GR-D5 for the 16† cap; LY 6.15 for the forcing table; "the MathML and SVG boundary (2†, shared)"; "the re-raster bound MD-D20's reversal trigger takes from MD-56" (or ask MD for a re-raster target); "ST-D20's entity budget and ST 6.14's 5,000† depth cap".
- **FC-R6 (S1 "Admission", 6.2, 6.10, SV-D13, 9.1).** ST-D18 is a CSS admission rule. Ask the owner to record its extension to SVG elements, attributes and DOM members, not only the `getPathData` exception.
- **FC-R7 (11.2).** Drop "Unverified: Firefox 122 … Chrome 120": US 4.17 (MDN compatibility data) records removal in Chrome 120 and Firefox 122. Drop "single-source, aggregator" for CVE-2026-31807: the CVE.org record exists.
- **FC-R8 (6.0, 4.2).** Pin dated URLs: `https://www.w3.org/TR/2026/CR-SVG2-20261006/` and `https://www.w3.org/TR/2014/WD-svg-integration-20140417/`, as M-1 part 2 and the SS note ask.
- **FC-R9 (2.1 SVG fonts row).** "Microsoft would implement neither SVG fonts nor SMIL" is the SVG WG's 2015 recollection; "low use" was Mozilla's 2010 objection; Chrome's own reason for removal in 38 is not documented. Say so.
- **FC-R10 (2.2 `animVal` row).** "engines called the tear-offs 'costly to maintain'" → "a WebKit participant called the tear-off support 'costly to maintain' and Blink cited 'significant complexity'".
- **FC-R11 (2.2 SMIL row, 6.9 option (d), 4.4 "SMIL can be dropped").** "the only animation in SVG images" → "with CSS animations, the only animation in SVG images" (6.9's own evidence paragraph says this).
- **FC-R12 (6.3 SVGZ).** "The magic-number rule is not in a specification": SVG 2 requires gzip and deflate support "for any content type", and OpenType's SVG table defines gzip glyph documents by their `1F 8B 08` prefix. Say that no specification defines sniffing gzip in an `image/svg+xml` body or a local `.svgz` file.
- **FC-R13 (Sum, 5, 6.5, 6.8).** Mark two syntheses as such: `use`-instance and client-list bug classes are inferred (SR 1.7, Low); the list of user-state sources behind `foreignObject` taint is inferred (SR 5.5, Low), and Gecko did not taint `blob:` SVG (SR 5.4).
- **FC-R14 (3, 3.1; reference list).** State once that 1,405 is the Chrome run's file count and 1,411 the directory's. Remove the unused `[RN]` and `[LA]` definitions.
- **FC-R15 (note RG 1.x, 2.x, 3.x, 7.x).** The note says SVG 2 was read in full on 2026-10-20 but dates its rows "CR 2018-10-04"/"2018". Re-date them to the 2026 CR; the five text-algorithm defects are confirmed present in the 13 October 2026 Editor's Draft.
- **FC-R16 (6.6, SV-D2; for the design critique).** LY-D3's options table rejects floating-point geometry because "cross-ISA identity needs a strict discipline everywhere". SV-D2 adopts exactly that discipline for SVG; say why SVG's case differs (unbounded user units) and cite LY-D3's rejected option.
- **FC-R17 (6.1 option (b) and S1).** The 768 KiB figure includes, for a window agent, "an empty document" (FP-34). Note that the scriptless host still pays for its static document (SV-51's 48 KiB† base), so the saving is the agent heap minus that base.
- **FC-R18 (6.12 table).** "never-rendered subtrees (`defs`, `clipPath`, `mask`, `marker`, `pattern`, un-instanced `symbol`) ([SS 8.4])": SVG-AAM excludes elements "never directly rendered" by a general rule and the element mapping tables, not by this list. Cite SVG-AAM §4.1.1 and the mapping tables.
- **FC-R19 (6.9 "Timelines").** "`<discard>` … both engines that shipped it withdrew it" rests on US 4.14 (MDN data: Chrome removed in 81, Firefox 136–138 removed in 139); AN 3.6 says Firefox support was "not confirmed". Cite US 4.14 rather than SS 6.2 for the engine half.
- **FC-R20 (2.1 2024–2026 row).** The 2024 charter extended the group while a maintenance charter was drafted; the mission sentence "to maintain SVG specifications" is in the **draft 2026** charter. Cite it.
- **FC-R21 (6.8 "What an SVG image cannot do").** "Load anything but `data:` URLs": same-document references are also allowed by SVG 2 §2.2; add them.
- **FC-R22 (6.16 "Compositing").** Chrome's blog lists opacity, filter and transform as composited; the spec says "transform and opacity". Fine for Izcal's own rule, but do not imply Chrome composites only those two.
- **FC-R23 (2.1 "SVG 2 re-CR" row, 6.7).** The ED's changes appendix also lists March 2026 removals (`stroke-linejoin` `miter-clip` and `arcs`, `SVGElementInstance` mixin, issue 1070). Check the pinned CR snapshot against the ED's changes list once more at SV-M0, because the ED moved again after 6 October (FC-M1).
- **FC-R24 (11.2).** Add: "Spec text checked against the 13 October 2026 Editor's Draft where w3.org could not be downloaded; one difference found (FC-M1)."

## Counts

- Numbered claims 1–134: 121 correct, 9 partly (#10, #39, #58, #59, #85, #116, #121, #125, #132), 1 wrong (#118), 3 unverifiable (#123, #129, #134).
- Cross-reference rows X1–X12: 8 partly, 4 wrong. A further 52 cross-references were found correct and are not counted.
- Arithmetic: 13 checks, all correct except the SMIL growth baseline (#10).

**Summary line: 146 claims checked: 121 correct, 17 partly correct, 5 wrong, 3 unverifiable (plus 52 cross-references correct and uncounted). Links: 1 external inline link in the spec (live, no fragment) and 26 reference-style links (all resolve; 2 unused); 372 unique URLs in the notes, 6 dead or unreachable; fragments: 0 in the spec, 127 in the notes checked, 0 missing; engine- or library-source links: 0 (3 borderline repository or commit links in the notes, FC-R1). Must-fix 11, recommended 24.**
