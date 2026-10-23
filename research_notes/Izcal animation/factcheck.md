# Izcal animation: fact-check (phase 8)

Checked 20 October 2026 against `reports/Izcal animation.md` (draft 0.1) and its eight research notes, by a checker who wrote neither. Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8. Primary sources were re-read directly. The checker used NVD's 2.0 API for every 2026 CVE identifier and the in-the-wild CVEs, plus keyword tallies ("animation", "animate", "ViewTransitions"). csswg-drafts issues #5321, #7440, #9256, #10267, #13817 and #14337 were read through the GitHub API. Prose comments of WebKit bugs 185816, 215745, 250970, 281429 and 304412 were read through WebKit Bugzilla's REST interface, and of Mozilla bugs 1208411, 1218169, 1218620, 1832598 and 1834307 through Mozilla's. The Stack Exchange API gave the 11 Stack Overflow questions, and chromestatus's feature-popularity JSON (2026-10-18) gave the use counters. Editor's drafts were fetched as served today: Web Animations 1 (8 October 2026), Scroll-driven Animations 1 (25 September 2026), View Transitions 1 (16 February 2025) and 2 (31 August 2026), Easing 1 and 2, Contain 2, Animations 2, Color 4 and HR-Time. Also read: the HTML Standard's event loop, navigation-history and browsing-the-web pages; `w3.org/TR` headers (Web Animations 1, Transitions 1, Animation Triggers 1, WCAG 2.2, UAAG 2.0); WCAG 2.2's editor's draft and Understanding 2.3.3; ESET's RomCom analysis; Mozilla's 2016 and 2024 security posts; Google TAG's March 2022 post; the Chromium animation-team highlight pages (April, June and October 2021); dbaron's 2015 post; WebKit's blog; MDN's Firefox 144 notes; Web Almanac 2024 and 2025; State of CSS 2025 and 2026; the Gladman et al. (August 2026) and Romano et al. (ICSE 2021) PDFs; the Box2D, Torchbox, Motion, Playwright, Android, The Register and moyix pages; and the blink-dev SMIL thread. Every URL in the spec and the notes was fetched, and every `#fragment` was searched for in the fetched HTML.

Budget notes: no web search was used (the shared budget was exhausted); every page was fetched directly. About 560 page and API fetches. No engine source code was read. Two scripted fetches touched code-adjacent material, and neither was used as evidence. (1) The REST dump of Mozilla bug 1208411 includes inline code-review comments, which printed in a keyword filter; only comments 3 and 10 (prose) and the bug's metadata are relied on. (2) The ESET article embeds figures of exploit and engine code; only its prose was used. Links to a WebKit commit, a searchfox page and a WPT pull-request diff inside GitHub issues were not opened. No personal data was sent in any request.

Verdicts: **Correct**; **Partly** (the substance holds, but a number, scope, quote or wording is off); **Wrong**; **Unverifiable** (the cited source does not contain the claim and no other source was found); **Not checked** (left to the notes' own grading, or checking would need code). "Must-fix" marks a finding that a decision (AN-D1 to AN-D13) or a target depends on, a misstatement of a specification, or an arithmetic error in the spec. Every other fix is recommended.

## Summary

97 claims checked (a row that bundles several sub-claims is counted once, by its worst verdict):

| Verdict | Count |
|---|---|
| Correct | 77 |
| Partly | 6 |
| Wrong | 7 |
| Unverifiable | 1 |
| Not checked | 6 |
| **Total** | **97** |

**Links.**
- The spec has 27 inline URLs; all return 200, and its 5 `#fragment`s (`#combining-values`, `#interpolation`, `#update-the-rendering`, `#reveal`, `#event-loop-processing-model`) all exist. All 22 reference-style definitions resolve to files on disk.
- The notes and spec together hold 513 distinct URLs, and **none is genuinely broken**:
  - 495 return HTTP 200.
  - 14 return 403 to scripted fetches on pages that exist: 11 Stack Overflow questions (all 11 confirmed through the Stack Exchange API), Fast Company, QxMD and connect.mozilla.org.
  - `crates.io/crates/rug` returns 404 without an `Accept: text/html` header and 200 with it (a single-page app).
  - PubMed returns 203.
- **Fragments.** Of 208 `#fragment` links in the notes, 203 exist. **2 are broken:** `css-view-transitions-1/#security-considerations` and `css-view-transitions-2/#security-considerations`, where the id is `#sec`. **3 cannot be checked statically:** two CDP protocol pages and a GitHub README anchor are rendered client-side, but the targets exist.
- All 10 relative links in the notes resolve.

**The claims that carry the design mostly hold.**
- **CVEs.** All 16 2026 CVE identifiers exist on NVD and say what SR 1.3–1.5 claim, including the three flagged: CVE-2026-110166 (Chrome "Buffer overflow in Animation", published 2026-10-13), CVE-2026-79244 ("Use after free in Animation", 2026-08-25) and CVE-2026-106180 ("Observable discrepancy in Animation … cross-origin data", 2026-10-06). The three in-the-wild exploits match their primary sources: CVE-2016-9079 (Tor deanonymization; Mozilla fix about 1:30 pm Pacific on 30 November 2016), CVE-2022-0609 (two North Korean groups; TAG found it on 10 February 2022) and CVE-2024-9680 (ESET: a `then` getter on `Object.prototype`, four Animation objects on one AnimationTimeline, a fix in 25 hours).
- **Browser history.** WebKit enabled threaded animation resolution by default on 2026-02-13 (bug 250970, comments 6 and 7). Firefox's compositor-vsync fix landed in mozilla50 (bug 1208411, resolved 2016-06-29). Firefox's 60% → 1% hidden spinner and every chromestatus counter match: SMIL 2.896%, rAF 65.128%, `@property` 19.444%, `@view-transition` 12.272%, Animation Worklet 0.
- **Resolutions and accessibility texts.** The CSSWG resolutions of 2021-09-02, 2022-11-30 and 2024-09-24 match their minutes word for word, and UAAG 2.10.5 says "three or more seconds".
- **HTML timing.** HTML's frameTimestamp = last render opportunity time, passed to both "update animations and send events" and rAF.
- **Experiments.** Both of the spec's arithmetic experiments reproduce. 8.7% reproduces exactly: 8,656 of 100,000 with Python's `random.Random(1)`. 7.3% reproduces as 648 of 8,850 for durations of 0.1–1,000 ms.

**What fails.** Seven claims are wrong and two more partly-correct claims carry decisions:
- the basis of AN-D4 (there were two Chrome field trials, and the later one reported "very good results on throughput metrics");
- the spec-order argument for G130 (a `then` getter runs synchronously when a promise is resolved with an object);
- "one F for every document" (HTML passes each document its own relative time);
- invisible-animation skipping in `content-visibility` subtrees (Contain 2 freezes those animations, which is observable);
- inactive scroll timelines (a CSSWG resolution of 27 May 2026 reversed the text the spec follows);
- the easing error bound (false for legal overshooting curves);
- the engineer-year arithmetic;
- WCAG's blur exclusion (amended by erratum);
- the 55/43/35 wording (the 43 includes a policy bug).

### Must-fix list

1. **AN-D4's basis: "the one field trial found no smoothness gain".** There were two Chrome trials. June 2021 (Finch study): "more animations being composited", "we don't see improvement in the 'PercentDroppedFrame'", and a memory regression (crbug 1210221) ([Chromium animations highlights, June 2021](https://chromium.org/teams/animations/highlights-archive/june-2021---new-features-stability-fixes-interop-fixes-and-more)). October 2021 (stable channel, 1% of users): "Overall, we got very good results on throughput metrics, but slightly more memory consumption on Windows (~1.5%)", no FCP or LCP regression ([October 2021](https://www.chromium.org/teams/animations/highlights-archive/october-2021---2-sprints-worth-of-highlights)). CF 1.7 records the second trial; HP 6.6 and the spec use only the first. Change these six places:
   - The summary paragraph: "with colour and other uniform paint properties held for measurement because the one field trial found no smoothness gain".
   - Section 2.1, compositor row: "Chrome's composited `background-color` (2021) gave **no improvement in dropped frames** and a memory regression".
   - Section 4.4: "because the one field trial of composited colour showed no gain".
   - Section 6.5, class U Evidence: "Chrome's field trial found **no improvement in dropped frames** … and a memory regression".
   - Section 6.18, AN-D4 row: "One field trial found no smoothness gain and a memory cost".
   - Section 9.1: "the one field trial found none".

   Correct statement: "Chrome's two 2021 field trials of composited `background-color` disagree: the first saw no improvement in PercentDroppedFrame (the animations are mostly on small buttons) and a memory regression; the second, on 1% of stable users, reported 'very good results on throughput metrics' with about 1.5% more memory on Windows, and published no figure; the feature's later fate is unconfirmed ([CF 1.7][CF]; [HP 6.6–6.7][HP])". AN-D4 can stand as "held for measurement because the evidence conflicts and no figure was published", but not as "found no gain".
2. **6.3, step 2 and the paragraph after it.** The spec says: "a `then` getter on `Object.prototype` runs here, after the walk, exactly where the specification's microtask checkpoint already lets script run" and "Spec order is preserved: the specification already runs script only from step 3 of its procedure onwards, and promise reactions are microtasks either way". This is wrong. Web Animations resolves the ready promise "with animation" ([WA1 §4.5](https://drafts.csswg.org/web-animations-1/)). ECMAScript's promise resolve function reads `then` from an object resolution synchronously, so a getter on `Object.prototype.then` runs at the moment of resolution, inside step 1 ("update the current time of all timelines") of the specification's own order. That is exactly how CVE-2024-9680 freed the animation during the tick ([ESET](https://www.welivesecurity.com/en/eset-research/romcom-exploits-firefox-and-windows-zero-days-in-the-wild/)). Correct statement: "In the specification's order a `then` getter can run synchronously inside step 1, when a ready promise is resolved with its animation. Izcal defers every settlement until after the walk (G130), so the getter runs after all timelines are updated and before the microtask checkpoint. The specification leaves the ready moment to the implementation ([SP 2.7][SP]), so this ordering is permitted. It is still an Izcal choice, tagged under G135, and not 'spec order preserved'." SR 2.2's implication ("script runs between the timeline update and event dispatch by design") understates the same point.
3. **6.3, "The same F is used for every document of the event loop", and AN-54 "rAF arguments and timeline times identical across every document of an event loop".** HTML passes "relative high resolution time given frameTimestamp and doc's relevant global object" to both steps ([HTML update the rendering](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering)). Each document's value is the shared tick relative to its own time origin, so values differ between documents with different time origins. Each document's cross-origin isolated capability also depends on that document being allowed to use the "cross-origin-isolated" feature ([HTML](https://html.spec.whatwg.org/multipage/nav-history-apis.html)), so one event loop can hold documents on the 100 µs and the 5 µs grids. Correct statement: "Every document of the event loop, every timeline and every off-main-thread sample of a frame uses **the same tick**; each document's F is that tick relative to its own time origin, quantized down on its own grid, with no jitter." Restate AN-54 as "rAF arguments and timeline times of every document derive from one tick (equal after adding each document's time origin, up to its grid), and the off-main-thread sample uses the same tick". Make the same change in AN-D3, 6.17's first row and the 9.5 DM-D10 amendment ("identical for every document of the event loop"). The hand-off record already carries "the document's time origin and coarsening grid", so the design needs no change.
4. **6.11, invisible animation: "in a skipped `content-visibility` subtree (where CSS Contain 2 already says animations do not update) … Its timeline still advances … so skipping is unobservable to the model."** For skipped subtrees, Contain 2 specifies observable behaviour: "existing animations do not advance in their timeline. Running animations on the element do not end", new ones are not created, and when the element stops being skipped "animations and transitions are sampled and then resume advancing on their timelines as normal from that point". A style query samples them "according to the styles at the time of that style change event" ([css-contain-2](https://drafts.csswg.org/css-contain-2/)). MA 8.5 has this right; the spec merged it with the unobservable cases. Correct statement: "Skipped `content-visibility` subtrees follow Contain 2: their animations do not advance or end and no new ones start; they are sampled on a style query and resume from that point. The other invisible cases (offscreen, `visibility: hidden`, throttled navigables, hidden documents) are skipped unobservably as described." AN-30's "in skipped content" case and the energy table (AN-35) then measure a spec behaviour, not an optimization.
5. **6.6, "Inactive timelines. An animation on a timeline that is inactive (zero scroll range) stays pending; … Izcal follows the text … and files the open issues it depends on (Q-AN-5)"**, and Q-AN-5. The CSSWG resolved on 27 May 2026, in #9256: "RESOLVED: Scroll and view timelines don't become inactive when there's no scroll range". On 3 June 2026, in #13817, it resolved that auto-rewind "has no effect if the timeline is unresolved" ([#9256](https://github.com/w3c/csswg-drafts/issues/9256), [#13817](https://github.com/w3c/csswg-drafts/issues/13817)). The ED of 25 September 2026 still says the timeline "is inactive" when the 0% and 100% positions coincide, and a 31 August 2026 comment reports WPT tests "already being updated based on that". SP 13.2 lists both issues as open without the resolutions. Correct statement: "The CSSWG resolved on 27 May 2026 that scroll and view timelines do not become inactive at zero scroll range (#9256), and on 3 June 2026 how auto-rewind treats unresolved timelines (#13817); the ED text is not yet updated and WPT is moving to the resolution. Izcal implements the resolutions, as it does the reduced-motion cross-fade (6.7), and files for the text." Rewrite Q-AN-5 the same way. This also affects AN-1's `scroll-animations` floor.
6. **6.2, Easing: "24 steps fix t to 2⁻²⁴; y is evaluated exactly at that t and rounded once to Q32, giving an output error below 2⁻¹⁹ for every legal curve† (below 0.002 px on a 1,000 px move)".** The y control values of `cubic-bezier()` are unbounded, so the error is at most max|dy/dt| × 2⁻²⁴, plus the Q32 rounding. Here dy/dt = 3(1−t)²y₁ + 6(1−t)t(y₂−y₁) + 3t²(1−y₂), so max|dy/dt| ≤ 3·max(|y₁|, |y₂−y₁|, |1−y₂|). That gives ≤ 3 × 2⁻²⁴ when y₁, y₂ ∈ [0, 1], but 33 × 2⁻²⁴ ≈ 1.97×10⁻⁶ > 2⁻¹⁹ for `cubic-bezier(0.5, 10, 0.5, −10)` and 300 × 2⁻²⁴ for y₁ = 100 (checker's computation). Correct statement: "error ≤ 3·max(|y₁|, |y₂ − y₁|, |1 − y₂|)·2⁻²⁴ + 2⁻³³; below 2⁻²¹ when both y control values are in [0, 1]†". Then either let the step count grow with the curve's y range (k = 24 + ⌈log₂ max(1, |y₁|, |y₂−y₁|, |1−y₂|)⌉, checking DR 5.5's i128 headroom) or state the bound only for y ∈ [0, 1]. AN-D2's "fixed-point fails WPT tolerances" trigger is unaffected for ordinary curves.
7. **9.3, engineer-years: "about 20 in year one and 33–37 in year two, then 29–39 a year: 20 + 33 + 2 × 29 ≈ 111 … 20 + 37 + 3 × 39 ≈ 174".** Year two's low figure (33) exceeds the low full-strength team (29), which is impossible. With the stated ramp (12–15 at AN-M0, full strength by month 18, linear), year one averages about 18–23 and year two about 28–37. So the totals are about 18 + 28 + 2 × 29 ≈ **104** at 48 months and 23 + 37 + 3 × 39 ≈ **177** at 60 months: **about 105–175 engineer-years†**. Change the sentence, or change the ramp so that the year sums agree.
8. **6.10, "WCAG defines motion as change of position, size or shape, not of colour, opacity or blur", and level S's "colour, opacity and blur animations are unchanged".** WCAG 2.2's published glossary excludes "color, blurring, or opacity", but an erratum is listed against it: "Amending the definition to not exclude blurring". The editor's draft now reads "Motion animation does not include changes – such as changes of color or opacity – that do not alter the perceived size, shape, position, or distance/depth of the element" ([Understanding 2.3.3](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html); [WCAG 2.2 ED](https://w3c.github.io/wcag/guidelines/22/)). Correct statement: "WCAG defines motion animation as change of perceived size, shape, position or distance/depth, not colour or opacity (an erratum removed blur from the exclusion)". Level S must then decide blur explicitly, for example by treating `filter: blur()` and `backdrop-filter` blur as motion-inducing, or by keeping them unchanged as an "Izcal choice" with a reason. MA 3.3–3.4 need the same correction.

### Recommended (not blocking)

- **Section 1, "one small continuous animation draws about 8× an idle browser's power on one laptop"** (and 6.11's "about 200 mW for a typical CSS animation"). The Torchbox post gives idle "roughly 25mW" and "CSS and SVG animations: 100mW to 500mW"; "about 200 mW" is MA 7.1's reading of a chart. Say "4–20× (100–500 mW against 25 mW idle) on one M1 laptop".
- **2.1, Web Animations row: groups were cut "to iterate more quickly".** The words are "In the interests of iterating more quickly" ([public-fx, 2 February 2015](https://lists.w3.org/Archives/Public/public-fx/2015JanMar/0032.html)). Quote them exactly.
- **2.1 and 6.9: "WebKit's own threaded sampler replaced Core Animation by default on 2026-02-13".** Bug 250970 says "enabled by default as of 307441@main", which is WebKit's main branch, not a Safari release. Say "enabled by default in WebKit's main branch on 2026-02-13".
- **6.7, "`pageswap` and `pagereveal` activation information is null across origins, as HTML defines".** Only `PageSwapEvent` has `activation`. It is null unless the target document is same origin and was not created via cross-origin redirects (or its latest entry is non-null). `PageRevealEvent` has only `viewTransition`; the new document sees `navigation.activation.from` as null when the previous document was not same origin. Say so.
- **6.7, eligibility.** Correct as stated, but "history restoration excepted" is, normatively, "unless the new document's latest entry is non-null (a BFCache restore)". VT2 §12.5.1.1 also lets the UA return false when it shows "an implementation-defined navigation experience". Add both clauses.
- **G137's "Why", "The spec's 'must not be accessible'".** VT1 says "The implementations must ensure this data can not be accessed by the Document". Quote it exactly.
- **6.14 and G131, "35 of the 43 memory-class animation CVEs"**, and Section 5's "35 of 43 memory-class animation CVEs were lifetime bugs". The 43 are the rows of SR 1.2, 1.3 and 1.5. They include CVE-2026-8537 (a ViewTransitions policy leak) and two crash or unclassified entries, so not all 43 are memory-class. Say "35 of the 43 SMIL, CSS/Web Animations and View Transitions CVEs". The 55 total and the class sums are arithmetically consistent, and every identifier listed in SR 1.2–1.5 exists on NVD with the stated prose.
- **11.2, "2026 Chrome and Firefox animation CVE identifiers and counts … several 2026 identifiers to be re-checked by the fact-check"**. All 16 are confirmed on NVD today (Section 3, rows 87–88). Change the status to "identifiers confirmed on NVD 2026-10-20; counts are keyword tallies (lower bounds)".
- **6.2, "conformance tests allow ±1 µs on times and 0.02 on easing output ([DR 4.3, 5.1])"**. DR 4.3 and 5.1 read these from WPT's `web-animations/testcommon.js` and from the test file's script. See Section 4, item 1.
- **6.2, the 7.3% experiment.** It reproduces, but the result depends on the durations: 6.3% to 14.1% over the duration sets the checker tried. DR 4.8 does not record its five durations. Record them, and say "7.3% (6–14% depending on durations)".
- **6.2, "the product of an i64 and a double formed exactly in i128 and rounded once"**. That holds only while the double's exponent keeps the product within i128 (an i64 times a 53-bit mantissa is 116 bits). A `playbackRate` of 1e30 or 1e−30 needs the far path too. Extend the far-path condition from "a double whose magnitude exceeds 2⁶² ns" to "any time or rate whose exact product does not fit". 9.5's G61 amendment ("`hg-anim-time` joins G61's list (integer only except boundary conversions)") should also name the f64 far path as an exception, or move the far path out of `hg-anim-time`.
- **6.14, "at most one event of each type per animation per frame, [SR 7.4]"**. Correct, but SR 7.4 grades the per-frame count Low. Cite CSS Animations 2's event table instead: one `animationiteration` when "the current iteration … has changed since the previous animation frame" ([css-animations-2](https://drafts.csswg.org/css-animations-2/)).
- **Unmarked first-principles thresholds in Section 8.** AN-30's "over 60 s", AN-31's "per minute" window and AN-32's "500 ms" block are set by this document; mark them †. AN-71's 5 s comes from WCAG 2.2.2 and needs no mark.
- **UAAG 2.0** is a W3C Working Group Note (15 December 2015) that closed before CR, not a Recommendation. Say "UAAG 2.0 (a W3C Note)" where the spec says "UAAG asks".
- **Note fixes.** The spec does not repeat these:
  - **SR 5.1, 5.2.** `#security-considerations` → `#sec` (both VT drafts).
  - **SR 13.2.** Add the 27 May 2026 (#9256) and 3 June 2026 (#13817) resolutions.
  - **HP 6.6.** Add the October 2021 trial (CF 1.7 has it).
  - **CF 1.8.** The comment is dated 2026-01-29 (UTC).
  - **DR 1.11.** moyix scanned "2.4 million" shared libraries, not 270,000 wheels.
  - **DR 9.3.** The counts are in the paper's Table IV, not Table II.
  - **SR 6.4.** Bug 1834307 was fixed on 2024-09-11 (Firefox 132), not in 2023.
  - **MA 3.3–3.4.** Add the blur erratum (must-fix 8).
  - **MA 7.1.** "most CSS and SVG animations ≈200 mW" is a chart reading; the text gives 100–500 mW.
  - **HP 2.10** stays Unverifiable: the blink-dev thread as served shows messages only to 30 September 2015.

## 1. Internal consistency

| # | Check | Result | Fix |
|---|---|---|---|
| C1 | 9.3 track rows sum to the total | Correct: minimum 4+3+4+3+2+3+2+1+2+4+1 = 29, maximum 5+4+5+4+3+4+3+2+3+5+1 = 39 | — |
| C2 | 9.3 engineer-years against the ramp | **Wrong**: year two's low (33) exceeds full strength (29) | Must-fix 7 |
| C3 | 2⁶² ns ≈ 146 years (6.2) | Correct: 4.61×10⁹ s = 146.1 years | — |
| C4 | Easing bound "below 0.002 px on a 1,000 px move" from 2⁻¹⁹ | Arithmetic correct (1,000 × 2⁻¹⁹ = 0.0019); the bound itself fails for overshooting curves | Must-fix 6 |
| C5 | AN-35's "≤ 2.2† wakeups per second" against 6.11's "wakes twice a second" for a 1 s `steps(1)` caret | Consistent | — |
| C6 | 6.3's "same F for every document" against 6.5's hand-off record carrying "the document's time origin and coarsening grid" | Inconsistent: the record anticipates per-document F; 6.3 and AN-54 do not | Must-fix 3 |
| C7 | 6.11's "skipping is unobservable" against MA 8.5 (which the spec cites) | Inconsistent | Must-fix 4 |
| C8 | 6.7 (implements the 2024 resolution ahead of text) against 6.6 (follows the text, ignores the 2026 resolution) | Inconsistent policy | Must-fix 5 |
| C9 | G61 "integer only except boundary conversions" (9.5) against `hg-anim-time`'s f64 far path (6.2) | Inconsistent | Recommended |
| C10 | DM-D10 jitters per realm (DM 6.7, 100 µs / 5 µs) and the spec's amendment | Matches DM's text | — |
| C11 | GR-D13 already puts `filter` and `clip-path` re-raster in the content GPU process; GR-D5 caps compositor-mutable nodes at 16† | Matches GR's text (6.5 class G and rule 5) | — |

## 2. Links and fragments

| Scope | Checked | Result |
|---|---|---|
| Spec inline URLs | 27 | 27 × 200 |
| Spec fragments | 5 (`#combining-values`, `#interpolation`, `#update-the-rendering`, `#reveal`, `#event-loop-processing-model`) | All ids exist |
| Spec reference definitions | 22 (8 notes, 12 reports, reconciliation 2, ledger) | All files exist |
| Notes' distinct URLs (incl. spec) | 513 | 495 × 200 (three very large drafts needed a longer timeout; two GitHub pages returned 429, then 200); 14 × 403 bot blocks (11 Stack Overflow, all confirmed through the Stack Exchange API; Fast Company; QxMD; connect.mozilla.org); crates.io SPA (200 with `Accept: text/html`); PubMed 203. **0 broken** |
| Notes' fragments | 208 | 203 exist. **Broken: 2**: `drafts.csswg.org/css-view-transitions-1/#security-considerations` and `…-2/#security-considerations` (SR 5.1–5.3; the id is `#sec`). **Unverifiable statically: 3**: `chromedevtools.github.io/…/Emulation/#method-setVirtualTimePolicy` and `…/HeadlessExperimental/#method-beginFrame` (client-rendered; the method names are on the page), `github.com/gre/bezier-easing#readme` (GitHub's client-side anchor) |
| Notes' relative links | 10 | All exist (`m0/determinism_crossplatform.md`, other notes, reports) |

## 3. Claims checked

| # | Claim | Spec location | Source checked | Verdict | Note |
|---|---|---|---|---|---|
| 1 | Every engine has two animation engines, the compositor one partial with silent fallback | Summary; 2.2; 4.4 | HP 6.8 (Motion tier list, vendor claim) | Not checked | Note grades it Medium; plausible, single vendor |
| 2 | WebKit replaced Core Animation with its own threaded sampler, filed 2023-01-22, default 2026-02-13 | Summary; 2.1; 5 | WebKit bug 250970 comments 0, 6, 7 | Correct | "Enabled by default as of 307441@main" (main branch) |
| 3 | Core Animation capped `steps()`, `linear()`, additive, motion path, `playbackRate ≠ 1` | 2.2; 5 | Bug 250970 comment 2; bug 281429 comment 2 | Correct | Also out-of-range `cubic-bezier()` |
| 4 | Firefox compositor sampled one vsync behind; fixed by sampling at the main thread's frame time | Summary; 5; 6.3 | Bug 1208411 comment 3 (prose), target mozilla50, resolved 2016-06-29 | Correct | — |
| 5 | Firefox 41 (2015) ran transform and opacity, with geometry exceptions; desync "much more noticeable" for geometry | 2.1; 5 | dbaron 2015-09-16; bug 1208411 comment 10 (Birtles) | Correct | — |
| 6 | Firefox moves opacity to the main thread with a non-compositable transform (group rule) | 6.5 rule 1 | Bug 1218620 title; mozilla48 | Correct | — |
| 7 | Three in-the-wild exploits in eight years: Tor, two state groups, zero-click | Summary; 6.14 | NVD; Mozilla 2016-11-30; TAG 2022-03-24; ESET 2024-11-26 | Correct | Nov 2016 → Oct 2024 |
| 8 | CVE-2024-9680: promise settlement mid-tick; `then` getter; four animations on one timeline | Summary; 4.4; 5; G130 | ESET | Correct | The getter runs synchronously at resolution (see 69) |
| 9 | CVE-2016-9079 used to deanonymize Tor users; fixed 30 Nov 2016 | 6.14 via SR 3.1 | Mozilla blog; NVD | Correct | — |
| 10 | CVE-2022-0609: two North Korean groups | Summary; 6.14 via SR 3.2 | TAG blog; NVD | Correct | Found 10 Feb 2022 |
| 11 | Layout had no in-the-wild exploit in the same record | Summary | SR 3.4 → layout note | Not checked | Left to the layout fact-check |
| 12 | Style change event timing, solver, time precision, ready moment, snapshot limits and timeouts are implementation-defined | Summary; 3 | Transitions 1 §3; Easing 2 §2.2.2; WA1 §4.5.6, §8.1; VT1 §7 | Correct | WA1 recommends µs, does not require |
| 13 | Easing 1 and Easing 2 disagree on `steps()` | Summary; 6.2 | Easing 1 ED (16 Dec) and Easing 2 ED (23 Feb 2026) step algorithms | Correct | Easing 2: "If the before flag is true, return 0" |
| 14 | UAAG 2.10.5 asks to pause animation lasting three seconds or more | Summary; 1; 6.10 | UAAG 2.0 | Correct | "stop, pause, and resume … three or more seconds", Level A; a 2015 WG Note |
| 15 | No shipping browser lets users pause author animation | Summary; 1; 4.4 | MA 3.7, 4.7 | Not checked | Absence; note Medium |
| 16 | Hidden spinner cost 60% CPU in Firefox until invisible animations stopped updating (→ 1%) | 1; 5; 6.11 | Bug 1218169 comments 0 and 72 | Correct | Fixed 2018-06-27 |
| 17 | One small continuous animation ≈ 8× idle power (25 vs ~200 mW) | 1; 6.11 | Torchbox (Hancock, 2025-02-18) | Partly | Text: idle "roughly 25mW"; "CSS and SVG animations: 100mW to 500mW" |
| 18 | Half of sites contain `prefers-reduced-motion` | 1 | Web Almanac 2024 | Correct | 49% desktop, 50% mobile |
| 19 | Two most-viewed animation questions ≈ 4.6 M views (`display`, `height: auto`) | 1 | Stack Exchange API | Correct | 2,618,014 + 1,970,004 |
| 20 | "Animations" top Interactions pain point, State of CSS 2026 | 1 | 2026.stateofcss.com | Correct | 119 answers against 118 for "Browser vendor issues" |
| 21 | Scroll-driven and View Transitions 2nd and 3rd incompatibilities, 2025 | 1 | 2025.stateofcss.com | Correct | 50 and 49 answers |
| 22 | Animation timing caused 26 of 235 flaky UI tests | 1 | Romano et al. PDF | Correct | 17 web + 9 mobile; Table IV (DR says Table II) |
| 23 | Playwright "stable" = same bounding box for two frames | 1 | Playwright actionability | Correct | — |
| 24 | WebDriver BiDi has no animation command | 1; 6.13 | BiDi ED module list | Correct | 10 modules, none for animation |
| 25 | f64 step boundaries wrong in 7.3% of exact-boundary cases | 1; 6.2; 6.18 | Re-run in `python3 -I` | Correct | 648/8,850 (7.32%) for 0.1–1,000 ms; 6.3–14.1% for other duration sets |
| 26 | jQuery 1.6 adopted rAF, 1.6.3 removed it (hidden-tab pile-up) | 2.1; 5 | jQuery blog 2011-09-01 | Correct | — |
| 27 | O'Callahan: "there is no 'right' timeout value" | 2.1 | O'Callahan 2010-08-14 | Correct | — |
| 28 | IE: SMIL "Never" (2015) | 2.1 | SVG WG minutes 2015-04-30 | Correct | Rossen distinguished IE from Edge |
| 29 | Blink deprecation intent cited 0.0403% | 2.1 | blink-dev thread | Correct | 0.006% inside images |
| 30 | SMIL deprecation suspended in 2016 | 2.1; 11.2 | blink-dev thread as served (to 30 Sep 2015) | Unverifiable | Already flagged Low |
| 31 | SMIL on 2.90% of Chrome page loads; counting change | 2.1; 4.4; 6.8 | chromestatus JSON, 2026-10-18 | Correct | `SVGSMILElementInDocument` 2.896% |
| 32 | CSS Transitions 1 still a WD in 2026 | 2.1 | TR header | Correct | WD 8 January 2026 |
| 33 | rAF on 65.1% of page loads | 2.1 | chromestatus | Correct | 65.128% |
| 34 | WebKit needed "well over 2 years" to unify | 2.1; 5 | WebKit blog 10266 | Correct | — |
| 35 | Groups cut in 2015 "to iterate more quickly" | 2.1 | public-fx 2015-02-02 | Partly | Words are "In the interests of iterating more quickly" |
| 36 | The one field trial of composited colour found no smoothness gain | Summary; 2.1; 4.4; 5; 6.5; 6.18; 9.1 | Chromium highlights June and October 2021 | **Wrong** | Two trials; the second reported "very good results on throughput metrics". Must-fix 1 |
| 37 | Chrome checked eligibility twice; a missed input crashed; system colours fell back | 5 | April and October 2021 highlights | Correct | — |
| 38 | Composited `clip-path` "close to launching" (January 2026), unconfirmed | 2.1; 11.2 | WebKit bug 185816 comment 4 | Correct | Dated 2026-01-29 UTC |
| 39 | Animation Worklet: use counter 0 | 2.1; 6.1 | chromestatus | Correct | 0.0, last 2023-11-09 |
| 40 | `@property` on 19.4%; in all three engines by 2024 | 2.1 | chromestatus; HP 8.4 | Correct | 19.444% |
| 41 | WebKit threaded scroll-driven animations rendered continuously when idle (fixed 2025) | 2.1; 5; 6.6 | WebKit bug 304412 | Correct | Fixed 2025-12-19 |
| 42 | Scroll-driven: Chrome 115, Safari 26, Firefox preview; Interop 2026 | 2.1; 6.9 | WebKit Interop 2026 post (focus list) | Correct | Versions from BCD not re-fetched |
| 43 | VT same-document in all three (Firefox 144, Oct 2025); cross-document Chrome 126, Safari 18.2 | 2.1; 6.9 | MDN Firefox 144 (14 Oct 2025); WebKit Safari 18.2 post | Correct | — |
| 44 | `@view-transition` on 12.3% | 2.1 | chromestatus | Correct | 12.272% |
| 45 | Two Chrome VT CVEs in 2026, one a cross-origin leak | 2.1 | NVD CVE-2026-8537, CVE-2026-11646 | Correct | — |
| 46 | Motion: 4.5 million weekly downloads | 2.1 | Motion blog | Correct | "over 4.5 million" |
| 47 | Android ARR: Normal ≈ 60 Hz, max rate "results in high power consumption" | 2.2; 6.11 | Android docs | Correct | — |
| 48 | f32 time drifts by seconds per hour | 2.2 | Re-run | Correct | +5,399 ms in f32; −1.0×10⁻⁵ ms in f64 |
| 49 | Coarsening floor 100 µs / 5 µs; timing channels "practically impossible" to remove | 2.2; 6.3 | HR-Time ED | Correct | — |
| 50 | WA1 not snapshotted since June 2023 | 3 | WA1 ED header | Correct | Last TR 2023-06-05; ED 8 Oct 2026 |
| 51 | Animation Triggers 1 FPWD on 20 October 2026 | 3; 6.9 | TR | Correct | — |
| 52 | Scroll-anim cites HTML steps that no longer exist; HTML has no stale-timeline step | 3; 6.6 | Scroll-anim ED §5.1 ("step 7.10", "7.14.1"); HTML | Correct | — |
| 53 | #14337 argues resized subjects should be stale | 6.6 | GitHub API | Correct | Open, created 2026-08-17 |
| 54 | 40% of mobile pages run a non-composited animation | 3 | Web Almanac 2025 | Correct | 44% desktop |
| 55 | Rust `+ − × ÷` correctly rounded; `algebraic_*` opt-in | 6.2 | Rust std f32 docs (1.99.1) | Correct | — |
| 56 | NaN bits differ across ISAs | 6.2 | Rust NaN rules (read); RISC-V canonical NaN (DR 1.8) | Correct | RISC-V clause not re-fetched |
| 57 | A loaded library can turn on FTZ for a process | 6.2 | Clang manual; moyix 2022 | Correct | Note's "270,000 wheels" should be "2.4 million" libraries |
| 58 | Conformance tests allow ±1 µs and 0.02 | 6.2 | DR 4.3, 5.1 (WPT harness code) | Not checked | Reading the constants means reading test code (Section 4) |
| 59 | WA1 recommends microsecond precision | 6.2 | WA1 §8.1 | Correct | — |
| 60 | `A + p(B − A)` misses B at p = 1 in 8.7% of random pairs | 6.2 | Re-run | Correct | 8,656/100,000 exactly with `random.Random(1)`; `(1−p)A + pB` is exact at p = 1 by construction |
| 61 | Easing error below 2⁻¹⁹ "for every legal curve†" | 6.2 | Checker's derivation | **Wrong** | Fails for y control values outside about ±10. Must-fix 6 |
| 62 | 2⁶² ns ≈ 146 years | 6.2 | Arithmetic | Correct | — |
| 63 | Box2D v3: no contraction, own trig, CI hash | 5 | box2d.org 2024-08 | Correct | Hash 0x5e70e5fe, step 310 |
| 64 | HTML passes one frameTimestamp (last render opportunity time) to "update animations" and rAF | 4.2; 6.3 | HTML event loop | Correct | — |
| 65 | Document timeline time equals the rAF argument and is constant within a task | 6.1; 6.3 | WA1 §4.3.1.2 | Correct | — |
| 66 | The same F for every document of the event loop; AN-54 identical values | 6.3; AN-54; 9.5 | HTML ("relative high resolution time given frameTimestamp and doc's relevant global object"); cross-origin isolated capability per Document | **Wrong** | Must-fix 3 |
| 67 | Ready: setup "including rendering the first frame"; never synchronous | 6.3 | WA1 §4.5.6, §4.5.8 | Correct | "must not perform the task synchronously"; may run at a microtask checkpoint |
| 68 | WA1 puts a microtask checkpoint inside "update animations and send events" | 6.3 via SR 2.2 | WA1 §4.4 | Correct | — |
| 69 | A `then` getter runs only at the checkpoint; "spec order is preserved" | 6.3 | WA1 ("Resolve animation's current ready promise with animation"); ECMAScript promise resolution; ESET | **Wrong** | Must-fix 2 |
| 70 | Hidden documents are filtered out; timelines jump on show | 6.3 | HTML; SP 13.9 | Correct | Implied, as the note says |
| 71 | Inactive timelines keep animations pending; issues open | 6.6; Q-AN-5 | #9256 (resolved 2026-05-27), #13817 (resolved 2026-06-03); scroll-anim ED | **Wrong** (outdated) | Must-fix 5 |
| 72 | VT limits and timeouts implementation-defined; "at the very least, some low-quality representation" | 6.7 | VT1 ED | Correct | — |
| 73 | Cross-document eligibility: same origin, not reload, no cross-origin redirect (restore excepted), not browser UI for push/replace | 6.7 | VT2 §12.5.1.1 | Correct | Omits the UA's own-navigation-experience clause |
| 74 | `pageswap` and `pagereveal` activation null across origins | 6.7 | HTML "fire the pageswap event"; `PageRevealEvent` IDL; `navigation.activation.from` | Partly | `pagereveal` has no activation |
| 75 | VT1 says snapshots "must not be accessible" | G137 | VT1 §Security | Partly | "must ensure this data can not be accessed by the Document" |
| 76 | CSSWG 2024-09-24: cross-fade under reduced motion; not in drafts | 6.7 | #10267 | Correct | Re-resolved 2024-10-23 |
| 77 | Chrome's 4 s VT timeout unverified | 6.7; Q-AN-6 | SR 5.8 | Correct (as flagged) | Spec correctly marks it 4 s† and unverified |
| 78 | SMIL: 18 lifetime CVEs | 6.8 | NVD prose for SR 1.2's 19 | Correct | Opera CVE-2011-2620 is the unspecified crash |
| 79 | `accessKey` timing captured keystrokes without script | 6.8 | NVD CVE-2011-3663 | Correct | — |
| 80 | CSSWG 2021-09-02: discrete motion mode | 6.10 | #5321 | Correct | "where keyframes are sufficiently separated in time" |
| 81 | CSSWG 2022-11-30: "more granular animation control" | 6.10 | #7440 | Correct | — |
| 82 | WCAG motion excludes colour, opacity and blur | 6.10 | WCAG 2.2 Rec; Understanding 2.3.3 erratum; WCAG 2.2 ED | Partly | Blur no longer excluded; "distance/depth" added. Must-fix 8 |
| 83 | Firefox's reduced-motion spoof leaked through smooth scrolling | 5; 6.10; AN-55 | Bugs 1832598, 1834307 (titles) | Correct | Fixed 2023-05-23 and 2024-09-11 |
| 84 | Skipped `content-visibility` animations: timeline advances, skipping unobservable | 6.11 | Contain 2 ED | **Wrong** | Must-fix 4 |
| 85 | iOS Low Power Mode caps rAF at 30 fps, intentionally | 6.11 | WebKit bug 215745 comment 2 (2020-08-24) | Correct | — |
| 86 | A blinking caret cost an editor 13% CPU | 6.11 | The Register 2017-03-23 | Correct | "13 per cent"; Electron/Chromium |
| 87 | 55 animation CVEs; 35 of 43 memory-class are lifetime | 5; 6.14; G131 | NVD keyword tallies; SR 1.1–1.5, 2.8 | Partly | Sums consistent; the 43 includes CVE-2026-8537 (policy) and a crash |
| 88 | The 2026 identifiers exist (incl. CVE-2026-110166, -79244, -106180) | 2.1; 11.2 | NVD API, all 16 | Correct | Every description matches SR 1.3–1.5 |
| 89 | At most one event of each type per animation per frame | 6.14 | CSS Animations 2 event table | Correct | Cite it rather than SR 7.4 (Low) |
| 90 | Scroll-driven Animations: "no known privacy/security impacts" | 6.14 via SR 4.12 | Scroll-anim ED §6, §7 | Correct | — |
| 91 | Interop 2025 VT: 90.8% overall, Safari 99.2% | 6.9 via HP 10.5 | WebKit Interop 2026 post | Correct | — |
| 92 | Staffing rows total 29–39 | 9.3 | Arithmetic | Correct | — |
| 93 | Engineer-years 20 + 33–37 + … ≈ 110–175 | 9.3 | Arithmetic | **Wrong** | Must-fix 7 |
| 94 | Kotcher et al., CCS 2013 | 11.2 | SR 4.4 | Not checked | Already flagged |
| 95 | `Element.animate` 2014 (Chrome) to 2020 (Safari) | 2.1 | WebKit blog (Safari 13.1); BCD not re-fetched | Not checked | Safari 13.1 confirmed |
| 96 | Scroll-driven and VT are Interop 2026 areas | 8.1 AN-2 | WebKit Interop 2026 post | Correct | — |
| 97 | HTML does not mandate a rendering-opportunity model | 2.1; 2.2 | HTML event loop | Correct | "does not mandate any particular model" |

## 4. M-1 part 3: source evidence

No link in the spec points to source code: its 27 URLs are specifications and standards, and its 22 references are Izcal files. The notes were scanned for code hosts, commit, diff and blob paths, and source browsers. Every hit is handled below.

1. **WPT harness constants (DR 4.3, DR 5.1; repeated in spec 6.2).** DR 4.3 reads `TIME_PRECISION = 0.0005` and `assert_times_equal`'s ±0.001 ms from `web-animations/testcommon.js`, a JavaScript file of the test suite. DR 5.1 reads `epsilon = 0.02` and its source comment from `cubic-bezier-timing-functions-output.html`. WPT is a test-only input that the licence rule allows, but reading constants out of test-harness code is not a black-box use. **Fix**: either record an owner ruling that WPT test files count as test inputs that may be read (as the layout spec treats them), or restate these as black-box facts ("WPT's animation tests pass implementations that differ by up to 1 µs"), established by running the tests. The spec's design does not depend on the exact values.
2. **Mozilla bug 1208411 (CF 1.11, 3.6).** The thread contains inline code-review comments (diff excerpts). CF 3.6 quotes the patch's title ("Delay vsync timestamps on the compositor to make async animations more sync"), which is commit metadata. **Fix**: cite comment 3's prose for the mechanism, comment 10 for the rationale and the bug's milestone (mozilla50) for the release, as SR does for bugs 1478158 and 1832598. Drop the patch-title quote.
3. **ESET's RomCom article (SR 2.1, 3.3).** The article embeds figures of exploit code and of Firefox's `AnimationTimeline::Tick`, and describes the fix "as suggested by the diff". SR 2.1's last sentence ("the fix made the timeline hold its animations alive while iterating") is ESET's reading of a diff, which makes it second-hand source evidence. The spec does not use it: G130 rests on the mechanism, not on the fix. **Fix**: drop that sentence from SR 2.1, or mark it as ESET's description of a diff, unused. Extend SR's M-1 note to say the article's code figures were not used.
4. **`chromium.googlesource.com/…/docs/how_cc_works.md` (CF 1.3, 4.3).** A prose Markdown document served by a source browser (gitiles). Acceptable under OD-7 if only that page was opened. The spec cites neither row directly.
5. **`github.com/riscv/riscv-isa-manual` (DR 1.8, 1.9),** citing `src/unpriv/*.adoc`. These are the AsciiDoc sources of the ISA specification text, which is prose. Acceptable under OD-7.
6. **Repository READMEs and docs** (flackr/reduce-motion, bramus's extension, web-animations-js README and `docs/experimental.md`, gre/bezier-easing README, the interop READMEs, WICG's view-transitions explainer). Prose; acceptable. DR 5.2's lineage claim comes from bezier-easing's prose about engines; the spec derives integer bisection from Easing's monotonicity instead (7, "Evidence rule"), which is correct handling.
7. **GitHub issues that link to code** (#14337 links a WebKit commit and a searchfox page; #9256 links a WPT pull-request diff). The notes cite the issues' prose only; the checker did not open the code links. Acceptable.
8. **Values.** No value in the spec was found to come from source. The 4 s timeout is flagged and marked †. Engine caps are black-box questions (Q-AN-8), and solver tolerances are Q-AN-2.

## 5. First-principles numbers (†) and measured figures

- Every limit in 6.14 (65,536†, 10,000†, 1,024†, 4,096†), every View Transition limit (1,024†, one viewport of margin†, half resolution†, 4 s†), the motion-level parameters (1 s†, one scrollport length†, 1 Hz†), the 30 Hz† battery cap, the budgets in 6.15, every Section 8 percentage threshold, the milestone months and every staffing figure carry †. "Every limit marked †" (6.19) holds for 6.14.
- **Unmarked thresholds set by this document:** AN-30's 60 s window, AN-31's per-minute window and AN-32's 500 ms block (recommended fix above). The easing bound 2⁻¹⁹ is marked † but is wrong as stated (must-fix 6).
- **Figures presented as measured** are all attributed: the 7.3% and 8.7% are labelled "in one experiment" and reproduce; 25 vs ~200 mW is labelled as one lab blog (but see row 17); "40%", "4.6 million", "26 of 235", "12.3%", "2.90%", "19.4%", "65.1%" are third-party measurements and check out. No invented number is presented as measured. Section 9.3's engineer-years are marked † but contain an arithmetic error (must-fix 7).
