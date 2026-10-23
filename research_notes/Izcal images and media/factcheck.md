# Izcal images and media: fact-check (phase 8)

Checked 19 October 2026 against `reports/Izcal images and media.md` (draft 0.1) and its eight research notes, by a checker who wrote neither. The report was revised after the design critique while this check ran, so locations are given by **section**, not by line. Every flagged string was re-checked against the revised text before this file was written: all of them were still present.

## Method

The claims that carry a decision (MD-D1 to MD-D30), a target (MD-1 to MD-60) or the history were checked against their primary sources, read directly. The sources were:
- **Spreadsheets and bug trackers**: the Project Zero 0-day sheet, exported as xlsx and parsed locally; Project Zero RCAs and blog posts; Mozilla Bugzilla through its REST API (bugs 195280, 680385, 1207429 and 1213499).
- **Vendor posts and release notes**: the Chrome, Android, Microsoft Edge, Microsoft Research, image-rs, Rust and memorysafety.org posts; the Safari release notes as Apple's JSON; the Ladybird newsletter; the GitHub API for repository metadata; the blink-dev and dev-platform threads.
- **Licence and pool terms**: Via LA's AVC, AAC and HEVC/VVC pages; Access Advance's press releases; the AOM Patent License 1.0; the OpenH264 FAQ; LWN.
- **Platform documentation**: Microsoft Learn (Media Foundation formats, IMFContentDecryptionModule, Protected Media Path, Advanced Color, low-latency audio, screen capture); Apple's documentation JSON for VideoToolbox.
- **Specifications and RFCs**: RFC 6386, 6716, 7742 and 8251 as text, prose sections only (no Appendix A code was read); the PNG Third Edition; CSS Color 4 and CSS Color HDR; HTML; Web Audio 1.1; WebCodecs; Media Capabilities; Media Capture and Streams; webrtc-stats; ITU-R BT.1359-1; the ITU-T H.264 page; the AV1, AVIF and VP9 specification pages; the Vorbis I specification; color.org's licence pages.
- **Measurements and papers**: Web Almanac 2024 (Media) and 2025 (Performance); Englehardt and Narayanan (CCS 2016); arXiv 2402.09001.
- **Izcal's own documents**: the other Izcal specs on this branch, plus the security scope and coding standard in the main repository.

No browser or codec source code was opened (M-1 part 3). One source a helper found while checking AAC-LC was a merge request on a codec library; it is **not** used as evidence here. About 20 web searches were used in total. No personal data was sent in any request.

Verdicts:
- **Correct.**
- **Partly**: the substance holds, but a number, scope, date, attribution or wording is off.
- **Wrong.**
- **Unverifiable**: the cited source does not contain the claim, or could not be reached, and no other source was found.

**Must-fix (FC-M#)** marks a wrong or unsupported fact that a decision or target depends on, or a plain error in a load-bearing sentence. **Recommended (FC-R#)** marks every other fix.

## Summary

| Verdict | Count |
|---|---|
| Correct | 120 |
| Partly | 42 |
| Wrong | 11 |
| Unverifiable | 6 |
| **Total claims** | **179** |

Must-fix: 13. Recommended: 41.

**Links.** 517 distinct external URLs (34 in the spec, all of them also in the notes; 483 only in the notes) were requested with `curl -L` (6 in parallel, failures retried one at a time with a second user agent).
- **Working: 489** returned HTTP 200. Four of these needed a retry or another route: a Chromium audio README (503, then 200), two blog.webmproject.org posts (HTTPS fails from this host; HTTP gives the right titles) and snook.ca (certificate mismatch on `www`). No soft-404 titles were found.
- **Redirected to the wrong page: 3.** The Opera 10.60 post (HP 7.3, 8.4) now lands on the Opera news homepage. Apple developer forum threads 739953 (OS) and 776802 (HP 3.5) land on a bot check.
- **Bot-walled: 23**, all 403 to both user agents. They are 9 iso.org pages, une.org, 3 Phoronix pages, 2 connect.mozilla.org pages, helpx.adobe.com, Adobe's Flash end-of-life page, Packt, support.brave.app, the EBU tech page, Windows Report, TechRepublic and Bitmovin. None was counted as dead.
- **Not checked: 1.** Mozilla bug 1066467 returned 200 once, but the REST API stayed rate-limited.
- **Dead: 5.** None of them is in the spec.
  - `apple.com/hotnews/thoughts-on-flash/` (HP 6.4): 404, on the Wayback Machine.
  - The Cisco blog post "open-source-h-264-removes-barriers-webrtc" (FC 9.x): 404, on the Wayback Machine.
  - `via-la.com/?p=204` (FC 9.9): 404. The live page is `https://www.via-la.com/licensing-programs/aac/`.
  - `almanac.httparchive.org/en/2025/media` (FC 6, GO 1): 404. The notes already record that this page does not exist.
  - `ns.adobe.com/hdr-gain-map/1.0/` (CH 6): an XMP namespace identifier, not a page.

The note fixes are FC-R41.

**Fragments.** 23 of the 24 `#fragment` targets exist. They include the spec's `css-color-4#tagged-images`, SVG 2 `conform.html#processing-modes` and `rfc8251#section-11`. One is missing: `en.wikipedia.org/wiki/HTML_video#History` (HP 7.1); the section's id is now `History_of_<video>_element`.

**Internal links.**
- 19 of the 20 reference-style definitions resolve. `[SS]` (`Izcal security scope.md`) is not in this worktree's `reports/`. It lives in the main repository's `reports/` and is linked as a sibling, as the other Phase 2 specs do; noted, not counted as broken.
- Every label used in the body is defined, and none is unused.
- The notes' one relative link, `[frame.md](frame.md)`, resolves.

**M-1 part 3 (links to source code).** No link in the spec or the notes points at browser or codec source code: no code files, no `/blob/` or `/+/` paths to code, no commits, diffs, pull requests or code reviews, no searchfox or cs.chromium links.

Prose documents inside repositories (the grey zone), and whether a spec value depends on each:

| Document | Cited at | Value in the spec depends on it? |
|---|---|---|
| Chromium `docs/security/rule-of-2.md` | SE 9.8, 12.1, 12.4 | No: placement reasoning only (6.1, 7, 11.2) |
| Chromium `services/audio/README.md` | MP 9.11 (and an uncited OS copy) | No (7, 11.2) |
| WebRTC NetEq g3doc design document | MP 8.6 | No. But Section 7 cites [MP 9.11, 8.6] while 11.2 and Q-MD-14 say the NetEq document "is not used" (FC-R39) |
| `annevk/orb` README (ORB explainer) | SE 11.4 | Yes, a design requirement, not a number: the "subsequent range" marker is set only by the partition service (6.14; 9.5 networking row) |
| `google/wuffs` README | SE 6.6–6.8, GO 8.8 | Yes: "Wuffs since M93" (summary, 2.1). The spec cites SE 6.1–6.4, which do not contain M93 (FC-R40) |
| Compact-ICC-Profiles README (CC0 project) | CH 2.5 | Yes: "about 0.5 KB" (6.5) |
| `servo/media` README | GO 8.12 | Qualitative only: Servo's media was GStreamer-only and is archived (summary, 2.1, 5, 10) |
| MicrosoftEdge `videotest` README | MP 11.1 | Indirectly: 6.16 cites MP 11.1–11.4, and the stream run times come from 11.1 |
| `rav1d-safe` on docs.rs (crate documentation prose) | FC 11.4 | Yes: the 1.56× figure (6.4), already graded Low |
| ColorWeb-CG `hdr_html_canvas_element.md` | CH | No (not cited by the spec; a W3C community-group proposal) |
| oss-fuzz README, intel/media-driver README, iccDEV README, castLabs EVS wiki, xbmc VMP wiki | SE 5.4; OS; MP; CH; HP 10.4 | No (not cited by the spec) |

None of these is code, but 11.2's sentence "Two prose design documents kept in browser repositories are cited … no value comes from them" undercounts them (FC-R39). The Quarkslab CVE-2025-43300 write-up analyses Apple's binary patch. It is a researcher's prose analysis, not source code, and is acceptable.

**Personal data.** No personal email address appears in the spec or the notes. The only address-like string is a Fedora mailing-list name inside a list-archive URL (FC 9.x).

**Cross-references.**
- **Defined once, and every citation resolves**: MD-1 to MD-11, MD-9b, MD-20 to MD-26, MD-30 to MD-32, MD-40 to MD-42, MD-50 to MD-56, MD-60, MD-D1 to MD-D30 and Q-MD-1 to Q-MD-14.
- **Spikes**: MD-S5 is the exception. It appears in 6.11 and in the 9.5 D-S7 row but in no milestone (FC-R33).
- **G-rules**: G50 to G57 collide with nothing. RN P7 and RN Section 7 give G1–G9 to Stage 1, G10–G12 to DevTools, G13–G18 to JS, G19–G24 to style, G25–G29 to fonts and G30–G36 to networking. The layout draft uses G37–G43, and G44–G49 are unassigned.
- **Arithmetic**: the KLOC rows sum to 617–861 (stated 620–860), the track rows to 55–72, and there are 12 track rows (as Section 10 says). The arithmetic built on these totals has three errors (FC-M11, FC-M12, FC-M13).

## Must fix

1. **FC-M1. The Project Zero denominator** (Summary; Section 3, "Attack surface, measured"; 11.2; SE 1.1 and its method note). The sheet's "All" tab has 413 non-empty rows: 412 unique CVEs plus one row reading "-". The fonts fact-check found the same 412. The 414 comes from adding the year tabs, which repeat CVE-2022-22265. The other figures check out: all 22 listed CVEs are present, 14 of them are from 2021 or later, the 2021–2026 tabs hold 270 rows, and 8 of the 22 are OS frameworks. Fix: replace "22 of 414" with **"22 of 412"** everywhere (5.3%); keep "14 of 270 since 2021". In SE 1.1, write "412 unique CVE rows (the year tabs add one duplicate)".
2. **FC-M2. A misquote in the JPEG XL rationale** (2.1, 2022–2026 row; the MD-D2 rationale depends on it; HP 4.10). The Chrome post does not contain "a memory-safe decoder eliminates risk at the source". It says: "To eliminate these security risks at the source, we have integrated jxl-rs, a pure Rust implementation of the JPEG XL decoder." Fix: replace `because "a memory-safe decoder eliminates risk at the source"` with `with jxl-rs, "a pure Rust implementation", "to eliminate these security risks at the source" ([Chrome](https://developer.chrome.com/blog/jpeg-xl-in-chrome))`. Make the same change in HP 4.10.
3. **FC-M3. "58% to 88% of Linux users able to play H.264"** (Summary "Third"; 2.1, 2013–2015 row; 2.2 "Frameworks give coverage"; Section 5, Firefox GStreamer row; the 6.2 MD-D3 recommendation). Neither cited bug contains a figure. Bug 1207429's comments have no telemetry, and bug 1213499 says only that the change "will allow Linux users to play H.264 video". HP 8.2 already graded the figures Medium. The number carries the "coverage decides" argument and the MD-D3 rationale. Fix, unless a primary source for the numbers is found:
   - Summary: "Firefox moved from GStreamer to FFmpeg for coverage (H.264 playback had depended on which GStreamer plug-ins users had installed)".
   - 2.1: "GStreamer replaced because coverage depended on installed plug-ins; Firefox 43 used system FFmpeg 'to allow Linux users to play H.264 video' ([Mozilla bug 1213499](https://bugzilla.mozilla.org/show_bug.cgi?id=1213499))".
   - 2.2: "(playback depended on which plug-ins were installed)".
   - Section 5: "Coverage then depended only on FFmpeg being installed".
   - 6.2 Recommendation: "the Linux coverage gap that drove Firefox's move to FFmpeg".
   - Add the figure to 11.2 as unverified.
4. **FC-M4. The Edge and Chrome wattage** (Section 5, last row; 6.16 first paragraph; MP 11.2). Microsoft's 2016 post gives 2,068 mW for Edge and 2,819 mW for Chrome from its **mixed lab test**, which automated "opening websites, scrolling through articles" and video. The same-stream test gave **run times**: Edge 7 h 22 min, Chrome 4 h 20 min. "2.07 W vs 2.82 W for the same stream" is therefore wrong, and it is evidence for MD-D24. Fixes:
   - Section 5: "Microsoft's 2016 test: Edge ran 7 h 22 min and Chrome 4 h 20 min streaming the same video on identical laptops (Edge drew 2.07 W and Chrome 2.82 W in a mixed browsing lab test); Firefox 70's Core Animation change cut 28.4 W to 16.4 W from compositing alone".
   - 6.16: "for the same stream on identical Surface Books, Edge ran 7 h 22 min and Chrome 4 h 20 min in Microsoft's 2016 test (in its mixed browsing lab test Edge drew 2.07 W and Chrome 2.82 W)".
   - MP 11.2: correct it the same way.
5. **FC-M5. "HEVC's pools are … not capped for software"** (6.2, the MD-D3 recommendation). No source says this. Via LA's HEVC/VVC programme has passed to Access Advance as VCL Advance (15 December 2025): $0.30 per unit in region R1, $0.20 in R2, and a $30 M annual cap for all products. HEVC Advance publishes "royalty rates and caps" and licenses "HEVC Software Sold separately". Fix: "because HEVC has two pools, both now run by Access Advance (VCL Advance at $0.30 per unit in R1 with a $30 M annual cap; HEVC Advance with its own rates and caps, which licenses separately sold software), and infringement suits are pending in the US and Brazil ([OS 4.11–4.13][OS])".
6. **FC-M6. HEVC on Windows "when the HEVC extension or hardware is present"** (6.2 recommendation; 4.4, "Patent-encumbered codecs" row by implication; 6.15 privacy row). Media Foundation's HEVC full decoder is the HEVC Video Extensions MFT, or an OEM-preinstalled one. Hardware alone is reachable only through D3D11 or D3D12 Video with Izcal's own parser, which G57 forbids. As written, the sentence contradicts G57. Fix: "(macOS always; Windows only when the HEVC Video Extensions decoder, or an OEM-preinstalled one, is installed; HEVC hardware is never driven through a low-level API, G57)". Reword the 6.15 HEVC bit the same way: "depends on whether the HEVC decoder MFT is installed".
7. **FC-M7. AV1 exactness "film grain included"** (4.2, fact 1; the 6.2 table AV1 row; replay in 6.13; MD-1, MD-42). The AV1 specification gives film-grain conformance two options. In one, the decoder's output is exact. In the other, the pre-grain frames must be exact but the grain need only show no "perceptually significant differences". Hardware decoders may take the second option, which affects MD-42 and hardware-to-software replay. Fixes:
   - 6.2 table, AV1 exactness: "Exact without film grain; with film grain, exact for decoders taking the first conformance option (the second requires exact pre-grain frames only); Izcal implements the film-grain synthesis process exactly".
   - 4.2, fact 1: add "(for AV1 with film grain, only if both apply the grain process exactly; otherwise replay compares pre-grain frames and applies Izcal's own grain)".
   - MD-42: compare hashes with film grain disabled, or on pre-grain output, where the hardware path exposes it.
8. **FC-M8. A stale normative anchor for H.264** (6.2 table, H.264 row; Section 7 anchor table). ITU-T lists H.264 (08/2024) as **Superseded**. The edition in force is H.264 (06/2026), approved 13 June 2026. Fix: replace "ITU-T H.264 V15" and "ITU-T H.264 V15 (2024-08-13)" with "ITU-T H.264 (06/2026), approved 2026-06-13", and give no version number until one is confirmed.
9. **FC-M9. Untagged-video defaults** (6.5, "Inputs to a descriptor"; this feeds the colour descriptor and MD-5). CSS Color 4 §3.5 (CRD of 9 October 2026) gives four cases: BT.601 below 720p; SMPTE ST 296 at 720p; BT.709 at 1080p; BT.2020 at 4K and above for SDR video. Fix: "untagged video is BT.601 below 720p, SMPTE ST 296 at 720p, BT.709 at 1080p, and BT.2020 at 4K and above for SDR video (CSS Color 4 §3.5)".
10. **FC-M10. The HTML autoplay viewport rule** (6.12, "Autoplay (MD-D13)"; MD-D13 row). HTML pauses only a `video` element that is autoplaying through its `autoplay` attribute when it leaves the viewport. Media started by `play()` is not covered. Fix: "muted or audio-less media may autoplay; a `video` autoplaying through its `autoplay` attribute is paused when it stops intersecting the viewport (HTML, [MP 1.9][MP]), and Izcal extends the same pause to muted media started by `play()` as its own UA policy". Make the same change in 6.16's hidden-tabs row and MD-D23 ("offscreen muted autoplay is paused").
11. **FC-M11. Stale combined staffing** (9.3, "Effect on Stage 1's plan"; 9.5, Stage 1 8.3 row). The reconciliation's owner addendum (RN Section 8) puts the five earlier specs at **231–290†** combined, not 216–285† (the fonts track became 79–90). Fix: "with the reconciliation's combined 231–290† for the five specs before this one ([RN][RN] Section 8), this spec adds 55–72†, so the six component specs need about **286–362† people**".
12. **FC-M12. Engineer-years and the ramp do not add up** (9.3, total paragraph and "Hiring ramp†"; 9.2, MD-M4).
   - "Half again" on 155–287 engineer-years gives 232–430, not 240–420.
   - The ramp gives 20 + 40 + 55 × 3 = 225 engineer-years over five years at the low end. At the high end, 20 + 40 + 72 × 4 = 348 over six years. With full strength reached only at month 30, even 348 is high.
   - 420 engineer-years therefore needs about seven years at 72 a year, so "420 over six" and "parity at 5–6 years" cannot both hold.

   Fixes:
   - Total: "about **230–430 engineer-years†** to parity".
   - Ramp: "roughly 225 over five years at the low end and 430 over about seven at the high end".
   - Duration: "MD-M4 (parity) at 5–7 years†", in 9.3 and in the 9.2 MD-M4 row.
13. **FC-M13. MD-D3's cost "about 5–7† engineers inside the codec tracks"** (6.2, option (b) cost; MD-D3 row). The 5–7 is the whole VP9, VP8 and H.264 track, and AAC-LC sits in the audio-codec track. The H.264 and AAC-LC row is 40–55 KLOC†, which at 3–4 KLOC† per engineer-year is 10–18 engineer-years, or about 2–4 engineers over the build. Fix: "about 2–4† engineers (40–55 KLOC†, 10–18 engineer-years†) inside the VP9/VP8/H.264 and audio-codec tracks".

## Recommended

**Sections 1–3 and the summary**

1. **FC-R1** (Summary, "the hardest ... were found by review, not fuzzing"). Isosceles says "I suspect", and the P0 RCA says "probably". Write: "were probably found by manual review, and fuzzing had missed them".
2. **FC-R2** (Summary; Section 3; 6.14 table, "5 of 5"). CVE-2018-4990, which has a published third-party analysis (srcincite), fits the same class: a loop bound from the image overruns a fixed buffer. Write: "in every one with a published root cause (5 of 5 with a P0 or named-analyst root cause; CVE-2018-4990's analysis fits too)".
3. **FC-R3** (Section 5, FORCEDENTRY row: "outside the sandbox"). IMTranscoderAgent has its own sandbox, so the path ran outside **BlastDoor**, not outside any sandbox. Write "outside BlastDoor".
4. **FC-R4** (2.1, Android row). Write: "about 80% of critical and high-severity media bugs in 2018 were in software codecs".
5. **FC-R5** (2.1, AV1 row). Apple's first AV1 hardware decode was the A17 Pro (September 2023), not the M3. Write "Apple from the A17 Pro and M3, 2023". The same applies to 6.9's "AV1 on macOS (M3 and later)", which is correct for Macs.
6. **FC-R6** (2.1, WebCodecs row). Write "Chrome 94, Safari 16.4 (video only), Firefox 130 (desktop)".
7. **FC-R7** (Section 3, first paragraph: "All four major browsers ship dav1d"). That is AOM's promotional claim, and Safari plays AV1 only with a hardware decoder (2.1). Write: "AOM says Chrome, Firefox, Edge and Safari use dav1d, though Safari plays AV1 video only with a hardware decoder ([FC 7.3, 7.6][FC])".
8. **FC-R8** (Section 3: "Safari decodes images through ImageIO, so its format set follows OS versions"). The cited WebKit post does not mention ImageIO, and Safari 17's JPEG XL also reached macOS Monterey. Find a primary source, or write: "Safari's new image formats arrive with OS releases (HEIC and JPEG XL with Safari 17), consistent with decoding through the OS image stack".
9. **FC-R9** (Section 1, row 2). The 14–20% figure counts `sizes` attributes, not pages. Write: "`sizes` is inaccurate enough to change the chosen candidate for 14% (mobile) to 20% (desktop) of `sizes` attributes".
10. **FC-R10** (Section 3 and 2.1: "Chrome 137" HDR images). Chrome 137's release notes do not list it, and only a test page is cited. 11.2 already flags it, which is enough; add it to Q-MD-11's black-box list.

**Sections 4–6 (design evidence)**

11. **FC-R11** (4.2, fact 2). The Vorbis I specification sets no numeric conformance tolerance; it gives only precision advice. Write: "JPEG, MP3 and AAC define output only within a tolerance; Vorbis sets no numeric tolerance; Opus partly".
12. **FC-R12** (6.3, ".hgbits" paragraph). "Three of the four classes ... ([SE 8.7])" contradicts SE 8.7, which counts 4 of 7 classes stopped by generation. Write: "Four of the seven root-cause classes in the record (specification ranges, inconsistent fields, layout assumptions and size arithmetic) are rejected by construction; entropy-table construction (G52), cross-thread races (6.8) and lifecycle bugs (MD-22's state machines) need the other rules ([SE 8.7][SE])".
13. **FC-R13** (6.3, EverParse sentence). The blog's words are "didn't introduce any noticeable performance overhead", and the post is dated 3 May 2021. Drop the quote marks or quote it exactly. In SE 8.1, change the date from 2021-12 to 2021-05-03.
14. **FC-R14** (6.4, SIMD paragraph; SE 6.2 and its Implications). Rust **1.87** made most `std::arch` intrinsics callable in safe code inside matching `target_feature` contexts; the 1.88 notes say nothing on this. Write: "Rust 1.87 made most SIMD intrinsics callable without `unsafe` inside matching target-feature contexts ([Rust 1.87.0](https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/))".
15. **FC-R15** (6.4, SIMD paragraph; 11.2). The 1.56× figure is one 4K AVIF still (120.7 vs 187.8 ms), and the page labels it an older measurement. Write: "reports 1.56× slower on one 4K AVIF still (historical measurement, Low confidence)".
16. **FC-R16** (6.2, VP9 row; Section 7). Write "VP9 Bitstream & Decoding Process Specification v0.7 (draft, 22 February 2017; the WebM Project says it is not final)".
17. **FC-R17** (6.2, MP3 row). Technicolor's mp3 licensing programme was terminated on 23 April 2017; the patents themselves are a separate matter. Write: "Technicolor's licensing programme ended 23 April 2017 ([FC 10.1][FC])".
18. **FC-R18** (6.2, ISO-BMFF row; Section 7). Catalogue listings show ISO/IEC 14496-12:2026 (published 21 April 2026); iso.org refused automated access. Write "ISO/IEC 14496-12 (edition confirmed when bought at MD-M0; a 2026 edition appears to supersede 2022)".
19. **FC-R19** (6.2, MPEG-2 TS row). In the Almanac, `.ts` files are 9% of video files; `.ts`, `.m4s` and extension-less adaptive-streaming files together are 27%. Write: "9% of video files are `.ts` segments, and about 27% are adaptive-streaming segments of any kind ([FC 6.5][FC])".
20. **FC-R20** (6.5, class S paragraph). The Almanac counts CICP-tagged images under "No ICC profile". Write: "87.7% of images carry no ICC profile (CICP-tagged images are counted there too, [FC 6.4][FC])". Also note that "1 in 80" is the Almanac's floor ("at least").
21. **FC-R21** (6.5, HDR paragraph; CH 13.3). The "Negative" positions on Chrome's headroom query that Chromestatus lists are Firefox's and Safari's positions on the Multi-Screen Window Placement API, not on headroom itself. Write: "Chrome's proposed headroom query (an extension of the Window Management API, on which Mozilla and WebKit have negative positions) is not implemented".
22. **FC-R22** (6.8, "Device facts are normalised"). The spec's rule is "MUST not be updated more than once per second", and it applies to a document that is visible **or** has microphone permission. Write: "`playbackStats` updates at most once per second and only while the document is visible or holds microphone permission, as Web Audio 1.1 requires; Izcal applies the visibility condition only ([MP 5.6][MP])".
23. **FC-R23** (6.2, "H.264, AAC and HEVC" bullet 2; G57's rationale). The quote is Tom Callaway's, quoted by LWN; it is not a Red Hat statement. Fedora disabled Mesa's VA-API H.264 and H.265 decode **and encode** in September 2022 (Fedora 37). Write: "Fedora, on Red Hat's legal advice, disabled Mesa's VA-API H.264 and H.265 decode and encode in September 2022; Fedora's former legal liaison explained that 'the implementation as delivered potentially infringes' ([LWN](https://lwn.net/Articles/910978/), [OS 4.9][OS])".
24. **FC-R24** (6.2, option (b) cost). Write "then $0.20 per unit up to 5 million and $0.10 above, capped at $9.75 M a year", as the MD-D3 row already says.
25. **FC-R25** (6.2, "AV1 and VP9 patents": "it ends for anyone who sues"). Write: "it ends for a licensee who sues claiming an AV1 implementation infringes Necessary Claims (counter-suits excepted), and licensees must license their own Necessary Claims on the same terms ([AOM licence](https://aomedia.org/license/patent-license/) §1.2.1, §1.3)".
26. **FC-R26** (6.2 table, Opus row; MD-2; Section 7 Opus oracle row). RFC 8251 states only the calibration in prose (quality 0 is about 48 dB SNR); the thresholds themselves are in `opus_compare.c` in RFC 6716 Appendix A. "Re-implemented from the RFC" therefore depends on MD-D4. Add "(depends on MD-D4)" to the Section 7 Opus row and to MD-2.

**Sections 6–9 (cross-spec and arithmetic)**

27. **FC-R27** (header). Draft numbers: networking spec **0.3.3** (not 0.3.2), fonts and text **0.3.4** (not 0.3.3), HTML and CSS **0.3.3** (not 0.3.2); JS and FP are draft 0.3 at revision 0.3.1. The `[SS]` link target is not on this branch; it lives in the main repository's `reports/`, as for other Phase 2 specs. Note this, and make sure it lands before merge. Also write "from the frame and the seven research notes" in the revision line, since the header lists eight files.
28. **FC-R28** (Section 3; header). "FP-17 to FP-23 energy" should be "FP-17 to FP-20 (wakeups) and FP-21 to FP-23 (energy)". The header's FP section list should match the body: 6.4, 6.7, 6.14, 8.1 and 8.2 are used in the body, while 6.6 is listed but never used.
29. **FC-R29** (4.3; 6.3; 6.6; MD-D17). FP-D19 decides canonical **heap and buffer limits**. Applying it to decode limits and downscaling extends it. Write "following FP-D19's principle (canonical limits, no device-dependent observable values)", and list MD-D17 as an FP-D19 extension in 9.5.
30. **FC-R30** (6.2, JPEG XL paragraph; 9.1; MD-M1). FP 6.5 requires **AVIF and** JPEG XL peak decode memory to be measured before those decoders are specified (FP-M3). Add AVIF to the gate, and add the measurement to MD-M0.
31. **FC-R31** (6.1, audio service: "30 s† ... (FP-1 utility pattern)"). FP-1's baseline row says utility jobs "exit when done"; the 30 s keep-warm is FP 6.10's site-process figure. Write "exits after 30 s† with no stream open (a value this spec sets)".
32. **FC-R32** (6.4, Threads: "as JS G13 requires for the engine"). The general rule is Stage 1 G3; JS G13 covers L2e crates. Write "(Stage 1 G3; JS G13 for the engine)". In 4.3 and 6.4, "JS-D12: no FMA contraction, `hg-math`": no-FMA is in JS 6.9 (JS-D12 covers NaNs and relaxed SIMD), and `hg-math` is assigned to JS by RN P4, not named in the JS spec. Cite "JS 6.9 and JS-D12; `hg-math` per [RN P4]".
33. **FC-R33** (9.2, MD-M2). MD-S5 (the signed-in playback tests D-S7 lacks) is in no milestone. Add "MD-S5 passed" to MD-M2's PlayReady exit.
34. **FC-R34** (6.1 vs MD-26). 6.1 says the restart delay doubles "up to 20 minutes", and MD-26 says "to ≥ 20 minutes": a cap against a minimum. Write MD-26 as "doubles per crash per (site, container) up to a cap of 20 minutes†".
35. **FC-R35** (6.13 table). NW 6.12 never records SRTP payloads, so WebRTC media cannot replay exactly. Add a row: "WebRTC media payloads: not recorded (NW 6.12); replay marked 'not exact'".
36. **FC-R36** (9.5, Security scope and Stage 1 rows). Three more security-scope items need rewriting: TB-6 (OpenH264, RLBox), SEC-12 (C decoders and OpenH264 at M7) and SEC-30 (a) ("media in utility processes"). Q-SEC-13 asks **which** decoders, so write "answered by MD-D1, MD-D6 and MD-D5". Stage 1 13.2 also routes H.264, AAC and HEVC through "VA-API or GStreamer" on Linux; add "that route is removed (M-1, G57)".
37. **FC-R37** (several smaller corrections).
    - Summary, first sentence: HGBA-2 names no codec library. Write "exclude every library that does browser-engine work, which includes every library that decodes images, …".
    - MD-M4: "S-8" is Android isolation. Write "(Stage 2 Android; HAL `MediaCodec`)".
    - 9.2: Stage 1 M4 has no images today. Write "this spec proposes images at M4 (amendment, 9.5)".
38. **FC-R38** (6.2 and Section 7, the AOM licence notice ships). HGBA-2 M-1 does not address licence-condition notices, and the licence audit has no AOM entry. Add an owner question (or a clause in MD-D4): "the AOM Patent License text is a licence-condition notice, not a dependency or data set (owner to confirm)", and add it to the licence audit.
39. **FC-R39** (Section 7, "M-1 part 3, checked"; 11.2). The two sections disagree about the NetEq document: Section 7 cites [MP 9.11, 8.6], which includes it, while 11.2 and Q-MD-14 say it "is not used". 11.2's "Two prose design documents" is also too low. Fixes:
    - Section 7: cite "[SE 9.8][SE], [MP 9.11][MP]" only.
    - 11.2: write "Prose documents kept in repositories are cited: Chromium's Rule of 2 and audio-service README (placement only, no value), the ORB explainer (the subsequent-range marker rule), the Wuffs README (its M93 ship date), the Compact-ICC-Profiles README (profile size) and the servo/media README (its GStreamer design); none is code, and no constant comes from a codec library".
40. **FC-R40** (Summary, "its GIF decoder has been Wuffs since M93 ([SE 6.1–6.4])"). M93 is in SE 6.6 and GO 8.8, not in SE 6.1–6.4. Cite "[SE 6.4, 6.6][SE]".
41. **FC-R41** (notes only, links).
    - HP 6.4: replace the dead Apple "Thoughts on Flash" link with its Wayback copy.
    - FC 9.x: replace the dead Cisco blog link with its Wayback copy.
    - FC 9.9: replace `via-la.com/?p=204` with `https://www.via-la.com/licensing-programs/aac/`.
    - HP 7.1: change `#History` to `#History_of_<video>_element`.
    - HP 7.3 and 8.4: the Opera 10.60 link now redirects to a homepage; use an archive copy.
    - Wrong figures and dates in the notes: SE 6.2 should say Rust 1.87, SE 8.1's date should be 2021-05-03, SE 1.1 should say 412, MP 11.2 should be corrected as FC-M4 describes, and HP 4.10's quote should be corrected as FC-M2 describes.

## Claims

| # | Claim | Location in the report | Verdict | Source checked | Note |
|---|---|---|---|---|---|
| 1 | 22 of 414 in-the-wild 0-days are image or media bugs | Summary; 3; 11.2; SE 1.1 | Partly | [P0 sheet](https://docs.google.com/spreadsheets/d/1lkNJ0uQwbeC1ZTRrxdtuPLCIl7mlUreoKfSIgajnSyY/) xlsx export | 412 unique CVEs on "All" (413 rows incl. "-"); year tabs add CVE-2022-22265 twice. FC-M1 |
| 2 | 14 of 270 since 2021; all 22 listed CVEs present | Summary; SE 1.1 | Correct | same | 2021–2026 tabs hold 270 rows; 14 of the 22 are 2021 or later |
| 3 | 8 of the 22 are in OS media frameworks a browser cannot patch | 3; 6.9; 6.14 | Correct | same, vendor column | Apple 8 (1 is the Messages logic bug) → 7 OS frameworks + Samsung 1 |
| 4 | Every published root cause (5 of 5) is a file size or count that disagrees with an allocation | Summary; 3; 6.14 | Partly | P0 RCAs; sheet "Analysis URL" column; srcincite on CVE-2018-4990 | Holds; denominator undercounts third-party analyses (4990 fits). FC-R2 |
| 5 | libwebp CVE-2023-4863: tables sized from worst-case values that assume valid trees; fuzzing could not reach it | 5; 6.3 | Correct | [Isosceles](https://blog.isosceles.com/the-webp-0day/) | "calculates the maximum value for valid and complete codes"; Google's new fuzzer "predictably did not find" it |
| 6 | The hardest bugs were found by review, not fuzzing | Summary | Partly | Isosceles; P0 RCAs 22675, 2294 | Sources say "I suspect" / "probably". FC-R1 |
| 7 | AppleAVD CVE-2022-22675: 32-element array, `cpb_cnt_minus1` up to 255; days of fuzzing found no crash | 5; 6.3; 6.9 | Correct | [P0 RCA](https://googleprojectzero.github.io/0days-in-the-wild/0day-RCAs/2022/CVE-2022-22675.html) | "fixed size of 32 elements … maximum value of 255"; patch checks < 32 |
| 8 | H.264 bounds `cpb_cnt_minus1` to 0..31 | 5; 6.3 | Correct | P0 RCA (patch) and SE 3.4 (H.264 E.2.2) | ITU text not re-fetched; consistent with the 32-entry fix |
| 9 | FORCEDENTRY: a ".gif" reached over 20 codecs through sniffing | 5; 6.1; 6.14 | Correct | [P0 deep dive](https://projectzero.google/2021/12/a-deep-dive-into-nso-zero-click.html) | "over 20 image codecs are suddenly part of the iMessage zero-click attack surface" |
| 10 | … "outside the sandbox" | 5 | Partly | same | Outside BlastDoor; IMTranscoderAgent is sandboxed. FC-R3 |
| 11 | CVE-2025-43300: DNG `SamplesPerPixel` must equal the JPEG SOF component count | 6.3 | Correct | [Quarkslab](https://blog.quarkslab.com/patch-analysis-of-Apple-iOS-CVE-2025-43300.html); msuiche write-up | Two header fields not cross-checked |
| 12 | CVE-2020-15999: FreeType `Load_SBit_Png` 16-bit truncation | SE 2.7, 4.2 | Correct | [P0 RCA](https://googleprojectzero.github.io/0days-in-the-wild/0day-RCAs/2020/CVE-2020-15999.html) | "truncates the image height and width to 16-bit integers" |
| 13 | Stagefright: "Most of the vulnerabilities … were heap overflows resulting from unsigned integer overflows" | 2.1 | Correct | [Android 2016](https://android-developers.googleblog.com/2016/05/hardening-media-stack.html) | Verbatim |
| 14 | 80% of critical media bugs were in software codecs | 2.1 | Partly | [Android 2019](https://android-developers.googleblog.com/2019/05/queue-hardening-enhancements.html) | "In 2018, approximately 80% of the critical/high severity vulnerabilities in media components". FC-R4 |
| 15 | Android split mediaserver into unprivileged processes; `media.swcodec` has no device access | 2.1; 6.1 | Correct | Android 2016 and 2019 posts | — |
| 16 | BlastDoor doubles the restart delay to a 20-minute cap and re-randomises | 5; 6.1 | Correct | [P0 iMessage iOS 14](https://projectzero.google/2021/01/a-look-at-imessage-in-ios-14.html) | "up to an apparent maximum of 20 minutes" |
| 17 | Chromium's default PNG decoder is the safe-Rust `png` crate since M139 | Summary; 2.1 | Correct | [image-rs blog](https://blog.image-rs.org/2026/06/18/png-adoption.html) | "since M139 (August 2025) across all platforms" |
| 18 | … at about 1.9× libpng's speed | Summary; 5 | Correct | same | 339.9 vs 180.4 MP/s geomean, Ryzen 9 7950X (1.88×); maintainers' benchmark, not Chromium's |
| 19 | Rust 1.88 made SIMD intrinsics callable without `unsafe` | 6.4; SE 6.2 | Wrong | Rust 1.87.0 and 1.88.0 announcements | 1.87: "Safe architecture intrinsics"; 1.88 silent. FC-R14 |
| 20 | Chrome's GIF decoder has been Wuffs since M93 | Summary; 2.1 | Correct | [Wuffs README](https://github.com/google/wuffs) | "shipped in the Google Chrome web browser … (milestone M93)" |
| 21 | Rust BMP cut decode time 20–50%; JPEG/PNG-inside-BMP removed | 2.1; 6.2 | Correct | [Edge: Rustifying image codecs](https://microsoftedge.github.io/edgevr/posts/Rustifying-Image-Codecs-in-Chromium/) | "from 20% to 50%"; "we removed it from Chromium" |
| 22 | Chrome 155 ships JPEG XL with a pure-Rust decoder (6 October 2026) | Summary; 2.1; 6.2 | Correct | [Chrome blog](https://developer.chrome.com/blog/jpeg-xl-in-chrome) | "Published: October 6, 2026"; "jxl-rs, a pure Rust implementation" |
| 23 | … because "a memory-safe decoder eliminates risk at the source" | 2.1; HP 4.10 | Wrong | same | Phrase absent; actual: "To eliminate these security risks at the source". FC-M2 |
| 24 | JPEG XL removed from Chrome in 2022 for "not enough interest" | 2.1 | Correct | [The Register](https://www.theregister.com/2022/10/31/jpeg_xl_axed_chrome/) | "There is not enough interest from the entire ecosystem"; executed in M110 |
| 25 | Safari 17 added JPEG XL; Firefox plans it for 158 | 2.1; 6.2; MD-D2 | Correct | WebKit Safari 17.0 post; [dev-platform thread](https://groups.google.com/a/mozilla.org/g/dev-platform/c/3YMV4MS34KA) | "enabled by default in the 158 train" |
| 26 | Lossless JPEG XL decoded 15–20× slower than lossless WebP for about 10% smaller files | 6.2; 9.1 | Correct | dev-platform thread | "15x to 20x, depending on the hardware"; one participant, one image, single-threaded |
| 27 | Theora: "zero day attacks against media codecs have spiked"; turn-down from M120 | 2.1 | Correct | [blink-dev](https://groups.google.com/a/chromium.org/g/blink-dev/c/qqDdLkeyk7Y/m/ajHePzglAwAJ) | Verbatim |
| 28 | Arithmetic JPEG: "we won't implement this without evidence that it's going to be interoperable" | 2.1; 5 | Correct | Bugzilla REST, bug 680385 | Comment 17 |
| 29 | … about 7.7% smaller, decode time doubled | 2.1 | Correct | same | Comment 31 "average 7.7%"; comment 36 "more than doubling the compute time" |
| 30 | MNG removed: decoder "roughly the same size as all the other image decoders … combined", no maintainers, little used, not a W3C Recommendation | 2.1; 5 | Correct | Bugzilla REST, bug 195280 | Comment 0 |
| 31 | Unisys's LZW royalties announced 28 December 1994 | 2.1 | Correct | [Roelofs](http://www.libpng.org/pub/png/pnghist.html) | "quietly announced on 28 December 1994" |
| 32 | PNG Third Edition REC 24 June 2025, with APNG and `cICP` | 2.1; 6.2; 7 | Correct | [PNG-3](https://www.w3.org/TR/png-3/) | "W3C Recommendation 24 June 2025" |
| 33 | PNG-3 chunks `cICP`, `mDCV`, `cLLI`, `eXIf` | 6.2 | Correct | same | All defined |
| 34 | Colour precedence `cICP` > `iCCP` > `sRGB` > `cHRM`+`gAMA` | 6.5 | Correct | same, Table 1 | Priorities 1–4; "lowest Priority number should take precedence" |
| 35 | PNG dimensions up to 2³¹−1; APNG frames up to 2³¹−1, `num_plays` 0 = forever | 6.3 | Correct | same | — |
| 36 | NPAPI "a leading cause of hangs, crashes, security incidents, and code complexity"; Flash ended 31 December 2020 | 2.1 | Correct | [Chromium blog](https://blog.chromium.org/2013/09/saying-goodbye-to-our-old-friend-npapi.html) | Flash date per Adobe EOL page (HP 6.5), not re-fetched |
| 37 | Hickson removed every codec requirement on 29 June 2009 | 2.1 | Correct | [WHATWG list](https://lists.whatwg.org/pipermail/whatwg-whatwg.org/2009-June/020620.html) | "Mon Jun 29 21:50:31 PDT 2009" |
| 38 | Eich: H.264's licence "prevent[s] distributing it with open source products" | 2.1 | Correct | [Eich 2013](https://brendaneich.com/2013/10/ciscos-h-264-good-news/) | "terms that prevent distributing it with open source products including Firefox" |
| 39 | Firefox on Linux: H.264-capable users rose from 58% to 88% | Summary; 2.1; 2.2; 5; 6.2 | Unverifiable | Bugzilla REST, bugs 1207429 and 1213499 | No figure in either bug. FC-M3 |
| 40 | MSE Recommendation 17 November 2016 | 2.1 | Correct | W3C REC URL and page | — |
| 41 | EME Recommendation 18 September 2017; EFF resigned that day | 2.1 | Correct | W3C REC; [EFF letter](https://www.eff.org/deeplinks/2017/09/open-letter-w3c-director-ceo-team-and-membership) | "Effective today, EFF is resigning" |
| 42 | Mozilla: "Browsers must provide the ability to watch video or the browser becomes less and less the tool users need" | 2.1 | Correct | [Mozilla 2014](https://blog.mozilla.org/blog/2014/05/14/drm-and-the-challenge-of-serving-users/) | Verbatim |
| 43 | Widevine refused an open-source browser | 2.1; 5 | Correct | [Maddock](https://blog.samuelmaddock.com/posts/google-widevine-blocked-my-browser/) | "we're not supporting an open source solution like this" |
| 44 | AV1 1.0 on 28 March 2018 | 2.1 | Correct | AOMedia press release | datePublished 2018-03-28 |
| 45 | Apple AV1 hardware decode from the M3 (2023) | 2.1; 6.9 | Partly | Safari 17 notes JSON; Apple product facts | A17 Pro (September 2023) came first; M3 is right for Macs. FC-R5 |
| 46 | Safari decodes AV1 only in hardware | 2.1 | Correct | Safari 17 notes | "for devices with hardware support" |
| 47 | Safari 26 HDR images; Safari 26.2 fixed HDR images in CSS backgrounds and SVG images | 2.1 | Correct | Safari 26 and 26.2 notes JSON | "Fixed HDR images in CSS backgrounds, CSS borders and inside SVG images" |
| 48 | Chrome 137 shows HDR gain-map images | 2.1; 6.5 | Unverifiable | Chrome 137 release notes; Eric Chan test page | Already flagged in 11.2. FC-R10 |
| 49 | WebCodecs: Chrome 94, Safari 16.4, Firefox 130 | 2.1 | Partly | New in Chrome 94; WebKit 16.4; Firefox 130 notes; MDN BCD | Safari 16.4 video only; Firefox desktop only. FC-R6 |
| 50 | Ladybird moved decode (2024) and demux (March 2025) to FFmpeg | Summary; 2.1; 5 | Correct | [Ladybird March 2025](https://ladybird.org/newsletter/2025-03-31/) | "replaced our Matroska demuxer with FFmpeg's libavformat" |
| 51 | Servo's GStreamer media crate archived 23 April 2026 | Summary; 2.1 | Correct | GitHub API, repository banner | archived=true; "archived by the owner on Apr 23, 2026" |
| 52 | rav1d: >20 person-months against an estimate of 7 for the C part | Summary; 5; 9.3; 10 | Correct | memorysafety.org "porting C to Rust for AV1" | "estimated … 7 person-months … more than 20 person-months" |
| 53 | rav1d about 5% slower | 5 | Correct | memorysafety.org rav1d performance bounty | "about 5% slower" |
| 54 | dav1d ~50,000 lines of C and 250,000 of assembly, over ~6 years | Summary; 6.4; 9.3 | Correct | same | "under development for 6 years … 50k lines of C code and 250k lines of assembly" |
| 55 | FFmpeg's security page lists 518 CVE IDs | 2.2 | Correct | [FFmpeg security](https://ffmpeg.org/security.html) | 518 unique IDs; per-year counts match SE 5.1 |
| 56 | AudioContext fingerprint about 5.4 bits | 2.2; 6.8; 6.15 | Correct | [Englehardt & Narayanan](https://www.cs.princeton.edu/~arvindn/publications/OpenWPM_1_million_site_tracking_measurement.pdf) | "18,500 devices hashed to … 713 … entropy … 5.4 bits" |
| 57 | EverParse: Hyper-V's 100+ message kinds, "no noticeable performance overhead" | 6.3 | Partly | [MSR blog](https://www.microsoft.com/en-us/research/blog/everparse-hardening-critical-attack-surfaces-with-formally-proven-message-parsers/) | Substance right; words differ; post dated 3 May 2021 (SE says 2021-12). FC-R13 |
| 58 | "Three of the four classes of root cause … are rejected by construction" (SE 8.7) | 6.3 | Wrong | SE 8.7, SE 4.2–4.6 | SE 8.7 says 4 of 7. FC-R12 |
| 59 | CVE-2025-48530 (CrabbyAVIF, unsafe Rust) is Android's one Rust media near-miss | 3; 6.14 | Correct | [Google: Rust in Android](https://blog.google/security/rust-in-android-move-fast-fix-things/); CVE record | — |
| 60 | WebCodecs' security text asks for fuzzing of random control sequences; reclamation after 10 s without progress or in background | 6.7 | Correct | [WebCodecs](https://www.w3.org/TR/webcodecs/) §11–12 | "random inputs and control method invocations"; "past 10 seconds" |
| 61 | WebP reached every engine with Safari 14 (2020) | 2.1 | Correct | Safari 14 notes (HP 3.2) | — |
| 62 | Via LA AVC: first 100,000 units free, then $0.20 and $0.10, cap $9.75 M a year | 6.2 bullet; MD-D3; 9.3 | Correct | [Via LA AVC](https://www.via-la.com/licensing-2/avc-h-264/) | "100,001 to 5,000,000 $0.20 … 5,000,001 and more $0.10"; "$9.75M per year in 2017 and after" |
| 63 | Option (b) cost: "then $0.20 per unit, capped at $9.75 M" | 6.2 option (b) | Partly | same | Leaves out the $0.10 tier. FC-R24 |
| 64 | The 2010 free-internet-video waiver covers content, not decoders | 6.2 | Correct | OS 4.4's 2010 source (Medium) | Not on the current Via LA page |
| 65 | The OS vendor's licence lets end users use the decoder "without remuneration" | 6.2 | Correct | Via LA AVC page | Quote exact; coverage of a third-party app is an inference, held by Q-MD-3 |
| 66 | Red Hat pulled Mesa's H.264 and HEVC front ends because "the implementation as delivered potentially infringes" | 6.2 | Partly | [LWN 910978](https://lwn.net/Articles/910978/) | Quote is Tom Callaway's; decode and encode disabled, Fedora 37. FC-R23 |
| 67 | Windows ships H.264 and AAC decoders and encoders in the box | 6.2; 6.4 | Correct | Microsoft Learn, Supported Media Formats in Media Foundation | Listed since Windows 7 |
| 68 | Linux guarantees no OS decoder or encoder for H.264, HEVC or AAC | 6.2 | Correct | LWN (Fedora also disabled the encoders); OS 3.1 | — |
| 69 | RFC 7742 requires VP8 and H.264 Constrained Baseline | 6.2 | Correct | [RFC 7742](https://www.rfc-editor.org/rfc/rfc7742) §5 | "MUST implement the VP8 video codec … and H.264 Constrained Baseline" |
| 70 | The AAC pool has no free tier: $15,000 plus $0.98 per unit for the first 500,000 | 6.2 | Correct | Via LA AAC page | "initial fee of $15,000"; "1 to 500,000 units $0.98" |
| 71 | AAC-LC believed patent-free by distributors' counsel; no pool statement | 6.2; 11.2 | Correct | Fedora legal review (bug 1501522), via search | Fair as worded |
| 72 | HEVC: two pools; VCL Advance (former Via LA) $0.30 per unit with a $30 M cap | 6.2 | Correct | Via LA HEVC/VVC page; Access Advance release | Acquired 15 December 2025; R1 $0.30, R2 $0.20; $30 M cap |
| 73 | HEVC pools are "not capped for software" | 6.2 recommendation | Wrong | same | VCL Advance's cap covers all products; HEVC Advance publishes caps. FC-M5 |
| 74 | HEVC on Windows "when the HEVC extension or hardware is present" | 6.2 recommendation | Partly | Microsoft Learn, H.265/HEVC decoder | A full decoder needs the MFT; hardware alone means a low-level API, which G57 forbids. FC-M6 |
| 75 | OpenH264: coverage only for Cisco's binary; BSD-2; Firefox downloads it on demand | 6.2 | Correct | [OpenH264 FAQ](https://www.openh264.org/faq.html) | "Cisco is only covering the licensing fees for its own binary module" |
| 76 | AOM licence covers a decoder that "conforms fully with all non-optional portions"; the licence text must be reproduced | 6.2; 7 | Correct | [AOM Patent License 1.0](https://aomedia.org/license/patent-license/) | §1.2.1 "must reproduce this License with any Implementation" |
| 77 | … "it ends for anyone who sues" | 6.2 | Partly | same, §1.3 | Ends only for a licensee suing over Necessary Claims; counter-suits excepted. FC-R25 |
| 78 | Access Advance's exploratory AV1/AV2 pool (25 August 2026) | 6.2; 10 | Correct | Access Advance press release, 25 August 2026 | "exploratory phase for a new patent pool … AV2 … as well as its predecessor AV1" |
| 79 | VP9 on macOS needs `VTRegisterSupplementalVideoDecoderIfAvailable` (macOS 11 and later) | 6.9 | Correct | Apple documentation JSON | introducedAt macOS 11.0; Apple's page does not name VP9 (OS 1.5 says so) |
| 80 | VideoToolbox names "busy" as a reason (require-hardware key) | 6.9 | Correct | Apple, kVTVideoDecoderSpecification_RequireHardwareAcceleratedVideoDecoder | "The hardware decoding resources on the machine are busy" |
| 81 | `IMFContentDecryptionModule` needs Windows 10 2004 and later | 6.11 | Correct | Microsoft Learn | "Windows 10, version 2004 (10.0; Build 19041)" |
| 82 | PMP: "The application … does not have access to content after it is passed to the PMP"; HDCP enforced by the OS | 6.11 | Correct | Microsoft Learn, Protected Media Path | Ellipsis fair |
| 83 | DRM-1 and DRM-2: WebView2 grants Widevine software-level only and PlayReady up to SL3000; WKWebView grants FairPlay | 6.11 | Correct | OS 7.1–7.2; Stage 1 design (D-S7) | Internal spike results, consistent |
| 84 | MDN: Firefox relies on OS H.264 decoders "to avoid patent concerns" | 3 | Correct | [MDN video codecs](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Video_codecs) | "in order to avoid patent concerns" |
| 85 | All four major browsers ship dav1d | 3 | Partly | AOM "What is dav1d" | Promotional claim; conflicts with Safari's hardware-only AV1 playback. FC-R7 |
| 86 | Safari decodes images through ImageIO, so its formats follow OS versions | 3 | Unverifiable | WebKit Safari 17 post | Post does not mention ImageIO. FC-R8 |
| 87 | `enumerateDevices` before a grant: at most one device per kind, empty IDs, a two-bit floor | 6.10; 6.15 | Correct | [Media Capture and Streams](https://www.w3.org/TR/mediacapture-streams/) | "limit exposure to two bits of information" |
| 88 | `GraphicsCapturePicker` with a yellow border; ScreenCaptureKit picker; portals | 6.1 | Correct | Microsoft Learn, Screen capture | "a yellow notification border is drawn by the system" |
| 89 | Media Capabilities: "significant entropy" for HDR fields | 6.15 | Correct | [Media Capabilities](https://www.w3.org/TR/media-capabilities/) | "potential to add significant entropy" |
| 90 | Windows AV1 and VP9 Store extensions preinstalled; HEVC extension price | 11.2 | Unverifiable | OS 2.3–2.4 | No Microsoft source; already flagged |
| 91 | AV1, VP9, VP8, H.264, PNG, FLAC and lossless WebP define exact output | 4.2 | Correct | RFC 6386 ("exact values for reconstructed pixels"); webmproject VP9 (Argon "identical") | H.264 and FLAC from FC 3.5 and 4.3 |
| 92 | AV1 exact, "film grain included" | 4.2; 6.2; 6.13; MD-1; MD-42 | Partly | [AV1 specification](https://aomediacodec.github.io/av1-spec/) | Second film-grain conformance option is perceptual only. FC-M7 |
| 93 | JPEG, MP3 and AAC define output only within a tolerance; Opus partly | 4.2 | Correct | ITU T.83 page; RFC 6716 §6 | — |
| 94 | Vorbis defines output within a tolerance | 4.2 | Partly | [Vorbis I spec](https://xiph.org/vorbis/doc/Vorbis_I_spec.html) | No numeric tolerance; precision advice only. FC-R11 |
| 95 | RFC 6386: the "bitstream is defined by the reference source code and not this narrative" | 6.2 (MD-D4) | Correct | [RFC 6386](https://www.rfc-editor.org/rfc/rfc6386) §1 | Verbatim |
| 96 | RFC 6716's normative part is the C decoder in Appendix A | 6.2 (MD-D4) | Correct | [RFC 6716](https://www.rfc-editor.org/rfc/rfc6716) §1 | "The primary normative part of this specification is provided by the source code in Appendix A" |
| 97 | Opus conformance: the comparison tool plus the final range-coder state | 6.2; MD-2 | Correct | RFC 6716 §6; RFC 8251 §11 | "MUST have the same final range decoder state" |
| 98 | MD-2's Opus thresholds re-implemented "from the RFC" | 7; MD-2 | Partly | RFC 8251 §11 | Thresholds exist only in `opus_compare.c` (Appendix A); depends on MD-D4. FC-R26 |
| 99 | VP9 specification v0.7 | 6.2; 7 | Partly | [webmproject VP9](https://www.webmproject.org/vp9/) | v0.7 is a 2017 draft; "not final". FC-R16 |
| 100 | ITU-T H.264 V15 (2024-08-13) | 6.2; 7 | Wrong | [ITU-T H.264](https://www.itu.int/rec/T-REC-H.264) | 08/2024 is Superseded; 06/2026 is in force. FC-M8 |
| 101 | AV1 specification, approved PDF with Errata 1 | 6.2; 7 | Correct | aomedia.org/specifications/av1 | Release `v1.0.0-errata1` |
| 102 | AOM AVIF 1.2.0 (2025-10-16), with `tmap` and sample-transform items | 6.2; 7 | Correct | [AVIF](https://aomediacodec.github.io/av1-avif/) | "v1.2.0 AOM Final Deliverable, 16 October 2025" |
| 103 | ISO/IEC 18181-1 and -2, 2nd editions (2024) | 6.2; 7 | Correct | Search snippets (iso.org blocked) | Ed. 2 July and June 2024; reconfirm at MD-M0 |
| 104 | MP3 patents ended 23 April 2017 | 6.2 | Partly | Fraunhofer IIS mp3 page | It was Technicolor's licensing programme that terminated. FC-R17 |
| 105 | ISO/IEC 14496-12 (2022) | 6.2; 7 | Unverifiable | Catalogue snippets; iso.org and SCC 403 | A 2026 edition appears listed. FC-R18 |
| 106 | Images are the LCP element on 76% of mobile and 85% of desktop pages | 1 | Correct | [Almanac 2025 Performance](https://almanac.httparchive.org/en/2025/performance) | "85.3% of desktop … 76% of mobile" |
| 107 | `<video>` on 6.7% of mobile and 7.7% of desktop pages | 1 | Correct | [Almanac 2024 Media](https://almanac.httparchive.org/en/2024/media) | Fig. 5.37 |
| 108 | JPEG 32.4%, PNG 28.4%, GIF 16.8% (32% animated), WebP 12%, AVIF 1.0% (+386%) | 2.1; 6.2 | Correct | same, Figs 5.11, 5.12, 5.18 | Mobile shares, 2024 |
| 109 | 87.7% of images untagged → class S | 6.5 | Partly | same, Fig. 5.10 | CICP-tagged images are counted as "No ICC profile". FC-R20 |
| 110 | About 1 in 80 images is wide gamut | 6.5 | Correct | same | "at least 1 in 80"; a floor |
| 111 | 68% of video files are MP4 | 6.2 | Correct | same, Fig. 5.39 | By file extension |
| 112 | About 13% of video responses are segments | 6.2 | Partly | same | `.ts` 9% + `.m4s` 4%; extension-less ABR files add 14%. FC-R19 |
| 113 | Half of pages send a largest image bigger than a phone can show | 1 | Correct | Almanac 2024 Media | "50% of pages that sent images even larger than that" |
| 114 | `sizes` picks the wrong candidate on 14–20% of pages | 1 | Partly | same, Fig. 5.28 | Unit is `sizes` attributes. FC-R9 |
| 115 | ICC v2 profiles copyable but not alterable under ICC's name; sRGB v4 passed on only unchanged; a CC0 minimal profile is ~0.5 KB | 6.5; 7 | Correct | color.org profile-library and sRGB pages; Compact-ICC-Profiles README | 410–736-byte profiles |
| 116 | HDR reference white 203 cd/m² (BT.2408); CSS Color HDR cites BT.2408 and BT.2390 | 6.5; MD-D9 | Correct | [CSS Color HDR](https://drafts.csswg.org/css-color-hdr-1/) (ED 8 Oct 2026) | BT.2390 is cited for HLG and §5.4.1 toe/knee, not as the presentation tone map |
| 117 | Windows scRGB 1.0 = 80 cd/m² | 6.5 | Correct | Microsoft Learn, HDR and Advanced Color | Holds on HDR displays; on SDR Advanced Color 1.0 = reference white |
| 118 | CSS Color HDR's privacy text: numeric headroom never exposed | 6.5; 6.15 | Correct | CSS Color HDR, Privacy | "numerical values of HDR headroom are not directly exposed" |
| 119 | Chrome's headroom query is opposed by Firefox and Safari | 6.5 | Partly | Chromestatus 5172386273689600 | Linked positions are on the Window Placement API. FC-R21 |
| 120 | Wayland image description carries a reference luminance; KWin uses pure gamma 2.2 in HDR mode | 6.5 | Correct | wayland.app color-management-v1; KWin developer's blog (2023-12-18) | `set_luminances(…, reference_lum)`; "gamma 2.2 instead of the sRGB piece-wise" |
| 121 | Untagged video: BT.601 below 720p, BT.709 above | 6.5 | Partly | [CSS Color 4](https://www.w3.org/TR/css-color-4/#untagged) §3.5 | Four cases (601, ST 296, 709, 2020). FC-M9 |
| 122 | Malformed profile → untagged (§3.4); untagged → sRGB (§3.5) | 4.1; 6.5 | Correct | CSS Color 4 §3.4, §3.5 | Both section numbers right |
| 123 | CSS Color 4 allows three gamut-mapping algorithms; one must be used wherever mapping is mandated | 4.2; 6.5 | Correct | CSS Color 4 §14 | — |
| 124 | HTML: `timeupdate` 15–250 ms; `progress` 350 ± 200 ms; `stalled` about 3 s | 6.7 | Correct | [HTML media](https://html.spec.whatwg.org/multipage/media.html) | — |
| 125 | BT.1359 detectability +45/−125 ms | 6.7 | Correct | ITU-R BT.1359-1 | "+45 ms to –125 ms" |
| 126 | ATSC IS-191 +15/−45 ms | 6.7; MD-7 | Unverifiable | Secondary only | Already flagged Low in 11.2 |
| 127 | Web Audio 1.1 WD 22 September 2026; `renderSizeHint: "hardware"` flagged for fingerprinting | 6.8; 7 | Correct | [Web Audio 1.1](https://www.w3.org/TR/webaudio-1.1/) | "can be used for fingerprinting" |
| 128 | `playbackStats` "once-per-second, visible-only rule" | 6.8 | Partly | same | At most once per second; visible **or** microphone permission. FC-R22 |
| 129 | MMCSS "Audio"/"Pro Audio", audio workgroups, RTKit or portal | 4.2; 6.8 | Correct | Microsoft Learn, low-latency audio; MP 9.5–9.8 | — |
| 130 | Microsoft: "low latency means higher power consumption" | 6.8 | Correct | same | Verbatim |
| 131 | Smith's windowed-sinc resampler | 6.8 | Correct | CCRMA, jos/resample | — |
| 132 | Chrome's MEI: unpublished threshold; >7 s playback, >200×140 | 5; 6.12 | Correct | [Chrome autoplay](https://developer.chrome.com/blog/autoplay) | — |
| 133 | HTML allows pausing muted media that does not intersect the viewport | 6.12; MD-D13 | Partly | HTML autoplay steps | Only `video` autoplaying via the attribute. FC-M10 |
| 134 | Edge 2.07 W vs Chrome 2.82 W "for the same stream" | 5; 6.16 | Wrong | Microsoft Windows blog, 2016-06-20 | Watts are from the mixed lab test; the stream test gave 7:22 vs 4:20. FC-M4 |
| 135 | Firefox 70 Core Animation: 28.4 W → 16.4 W above idle | 5; 6.16 | Correct | Mozilla Gfx blog, 2019-10-22 | "before: 28.4W, after: 16.4W" |
| 136 | Apple: 5 more hours of 1080p streaming in Safari than Chrome | 6.16 | Correct | apple.com/safari | Vendor claim; already flagged |
| 137 | Browsers differ by 38–70% in battery life for the same video | 1 | Correct | MP 11.1; GO 9.2 | 7:22/4:20 = 1.70; 38% is derived (Low) |
| 138 | Hardware decoders use under 9% of the energy of software decoders | 2.2; 4.4; 6.9 | Correct | arXiv 2402.09001 abstract | "less than 9%" |
| 139 | WebCodecs hints may be ignored; webrtc-stats `freezeCount` definition | 6.7; 6.10 | Correct | WebCodecs; webrtc-stats | — |
| 140 | Safe-Rust AV1 fork 1.56× slower at 4K | 6.4; 11.2 | Partly | docs.rs, rav1d-safe (README prose) | One 4K AVIF still; labelled historical. FC-R15 |
| 141 | Chrome's JPEG XL decoder uses a portable SIMD layer and no assembly | 6.4 | Correct | Chrome blog | `jxl_simd`; "no assembly" implied by "pure Rust" |
| 142 | Header: JS 0.3, FP 0.3, NW 0.3.2, TX 0.3.3, ST 0.3.2, DT 0.3, SS 0.2 | Header | Partly | Each file's status line | NW 0.3.3, TX 0.3.4, ST 0.3.3; JS and FP rev 0.3.1. FC-R27 |
| 143 | FP-D11 (b): decoder per (site, container), never pooled; 10 s idle exit; FP-38 ≤ 2 MiB | 6.1; MD-D5 | Correct | FP decision table; FP-38 | — |
| 144 | FP-13 counts 4 B per pixel | 6.16; MD-52 | Correct | FP-13 | — |
| 145 | FP-17 to FP-23 are energy | 3 | Partly | FP targets | 17–20 wakeups, 21–23 energy. FC-R28 |
| 146 | FP-D19 implies canonical decode limits and display-only downscaling | 4.3; 6.3; 6.6; MD-D17 | Partly | FP-D19 | FP-D19 is "Heap and buffer limits". FC-R29 |
| 147 | FP 6.5 asks for JPEG XL peak memory before the decoder is specified | 6.2; 9.1 | Partly | FP 6.5 | It names AVIF and JPEG XL. FC-R30 |
| 148 | Audio service exits after 30 s, "FP-1 utility pattern" | 6.1 | Partly | FP-1 row; FP 6.10 | FC-R31 |
| 149 | FP-D13 (a), FP-D4, FP-D16, FP-12, FP-18, FP-22, FP 6.4/6.7/6.8/6.13/6.14 | Various | Correct | FP spec | — |
| 150 | G50–G57 follow the layout spec's range without collision | 6.18; 9.5 | Correct | RN P7, RN 7; layout draft | Layout G37–G43; G44–G49 free |
| 151 | JS G13 makes the substrate the only thread source | 6.4 | Partly | JS G13; Stage 1 G3 | FC-R32 |
| 152 | JS-D12 = no FMA contraction and `hg-math` | 4.3; 6.4; 6.5; 6.8 | Partly | JS-D12; JS 6.9; RN P4 | FC-R32 |
| 153 | JS-D9, JS-D15, TX-D4, TX-D18, NW-D2, NW 6.12 ("decoded media never enters it"), RN P5, P18, P20 | Various | Correct | Each spec | NW 6.12 also omits SRTP payload recording. FC-R35 |
| 154 | Reconciliation's combined staffing 216–285† (RN Section 7) | 9.3 | Wrong | RN Section 8 owner addendum | Now 231–290†. FC-M11 |
| 155 | Six component specs need 271–357† people | 9.3 | Wrong | Arithmetic | 231–290 + 55–72 = 286–362. FC-M11 |
| 156 | LA class 1 and class 5; "software codecs written by Izcal are possible; hardware decode is not" | 7; header | Correct | Licence audit | — |
| 157 | HGBA-3 DET-1 excludes "GPU-dependent output, media and audio pipelines"; VER-1; PRIV-2 | 6.13; 9.5 | Correct | HGBA-3 | — |
| 158 | HGBA-2 2.3.0–2.3.2, I-1, I-2 note, HAL `MediaCodec`, 5.5, S-4a, S-5, C-1 | Various | Correct | HGBA-2 | — |
| 159 | HGBA-2 names the codec libraries it excludes | Summary | Partly | HGBA-2 M-1 part 1 | Excluded by the general rule, not by name. FC-R37 |
| 160 | S-8 for Android MediaCodec | MD-M4 | Partly | HGBA-2 S-8 | S-8 is "Android isolation". FC-R37 |
| 161 | Stage 1: process map, 2.2, 3.3, 7.5, M7, 8.3, D-S1, D-S7, 13.1a | Various | Correct | Stage 1 design | — |
| 162 | Stage 1 M4 includes images ("images are on Stage 1's critical path at M4") | 9.2 | Partly | Stage 1 8.1 | M4 has no images today. FC-R37 |
| 163 | Stage 1 13.2 amendment covers the codec row | 9.5 | Partly | Stage 1 13.2 | Also the Linux "VA-API or GStreamer" route. FC-R36 |
| 164 | Security scope IDs (ATK-1, ATK-3, TB-6, TB-9, 6.10, 6.10a, SEC-12, SEC-30, SEC-33, Q-SEC-4, Q-SEC-17) exist | Header; various | Correct | Security scope draft 0.2 | — |
| 165 | Q-SEC-13 "answered by MD-D5" | MD-D5; 9.5 | Partly | Security scope Q-SEC-13 | It asks which decoders. FC-R36 |
| 166 | Coding standard CS-ER-3: tier P never panics | 6.1 | Correct | Coding standard | — |
| 167 | KLOC rows total 620–860 | 9.3 | Correct | Arithmetic | 617–861 |
| 168 | 155–285 engineer-years of code at 3–4 KLOC per engineer-year | 9.3 | Correct | Arithmetic | 155–287 |
| 169 | Half again gives about 240–420 engineer-years | 9.3 | Wrong | Arithmetic | 232–430. FC-M12 |
| 170 | Track rows total 55–72; twelve tracks | 9.3; 10 | Correct | Arithmetic | — |
| 171 | Ramp: ~240 over five years (low), 420 over six (high); parity at 5–6 years | 9.2; 9.3 | Wrong | Arithmetic | 225 over five; 348 at most over six. FC-M12 |
| 172 | MD-D3 (b): about 5–7† engineers inside the codec tracks | 6.2; MD-D3 | Wrong | 9.3 rows | 40–55 KLOC = 10–18 engineer-years ≈ 2–4 engineers. FC-M13 |
| 173 | MD-D2 35–50 KLOC, MD-D16 20–30 KLOC, HRTF 0.25 engineer-years match 9.3 | 9.4 | Correct | 9.3 rows | — |
| 174 | MSE 128 MiB ≈ 50 s of 4K AV1 at 20 Mbit/s | 6.7 | Correct | Arithmetic | 53.7 s |
| 175 | 2²⁸ px = 268 M px = 1 GiB at 4 B/px | 6.3 | Correct | Arithmetic | — |
| 176 | MD-26 "≥ 20 minutes" vs 6.1 "up to 20 minutes" | 6.1; MD-26 | Partly | Internal | Cap vs minimum. FC-R34 |
| 177 | Every MD target, MD-D, Q-MD and spike cited is defined | Throughout | Partly | Internal | All defined except MD-S5 in no milestone. FC-R33 |
| 178 | "written … from the seven research notes" | Revision line | Partly | Header | Eight files (frame + seven). FC-R27 |
| 179 | The AOM licence notice ships consistently with M-1's MIT-only rule | 6.2; 7 | Partly | HGBA-2 2.3.2; licence audit | Not addressed; needs an owner ruling. FC-R38 |
