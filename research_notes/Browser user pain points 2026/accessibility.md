# Web Accessibility in 2026 and Browser-Level Repair / Mitigation

Notes compiled 2026-10-14. Older data carries its date. Every claim has a URL; anything unsourced is in Gaps.

## Q1. Fact-check of WebAIM Million 2026 claims

### Takeaway
All four numeric claims match the primary source, the WebAIM Million 2026 report (data crawled February 2026, page last updated Mar 30, 2026). The "most common errors" list is correct but incomplete, and the report itself does **not** single out keyboard navigation; that framing comes from other sources (see Q2).

### Verdict table
| Claim | Verdict | Primary source / date | Current figure |
|---|---|---|---|
| 95.9% of homepages had detectable WCAG 2 failures in 2026, up from 94.8% in 2025 | **Correct** | [WebAIM Million 2026](https://webaim.org/projects/million/) (Feb 2026 crawl) | 95.9% (2026) vs 94.8% (2025); 97.8% in 2019. WebAIM says this is "reversing a trend of small improvements each of the previous 6 years" |
| Average 56.1 errors per page | **Correct** | [WebAIM Million 2026](https://webaim.org/projects/million/) | 56.1 (2026), up 10.1% from 51 (2025). It was 56.8 in 2024, so 2026 is back to roughly 2024 levels |
| Most common errors: low contrast (83.9%), missing alt text, missing form labels, empty buttons/links | **Correct, but incomplete** (no percentages for the others, and missing document language is left out) | [WebAIM Million 2026](https://webaim.org/projects/million/) | Low contrast 83.9%; missing alt text 53.1%; missing form labels 51.0%; empty links 46.3%; empty buttons 30.6%; missing document language 13.5%. These six types make up 96% of all detected errors |
| Average homepage has 1,437 elements, a 14.3% increase in one year | **Correct** | [WebAIM Million 2026](https://webaim.org/projects/million/) | 1,437 elements (Feb 2026) vs 1,257 (2025), +14.3% |
| Keyboard navigation failures as a pressing issue | **Partly correct / not from WebAIM Million.** The Million report has no keyboard section, and automated WAVE tests cannot detect most keyboard failures. Screen reader users do rank lack of keyboard access among their top problems | [WebAIM Million 2026](https://webaim.org/projects/million/); [WebAIM Screen Reader Survey #10](https://webaim.org/projects/screenreadersurvey10/) (Dec 2023–Jan 2024) | "Lack of keyboard accessibility" is ranked #5 of 12 problem items (survey #10). Related Million 2026 indicators: 30.4 tabindex instances per page (+17%); only 17.1% of pages have a skip link, and 1 in 10 of those is broken |

### Other cited findings from WebAIM Million 2026 (useful context)
- 82.7% of home pages used ARIA (up from 79.4% in 2025). The report counted about 133 ARIA attributes per page, up 27% in a year. Pages with ARIA averaged 59.1 errors vs 42 without, though WebAIM cautions those pages are also more complex — [WebAIM Million 2026](https://webaim.org/projects/million/)
- 5.7% of pages had an ARIA menu, and 22% of those menus introduced barriers. Pages averaged 23.3 `aria-hidden="true"` attributes (+30%) — [WebAIM Million 2026](https://webaim.org/projects/million/)
- WebAIM links the rise in errors to more reliance on third-party frameworks and libraries and to "automated or AI-assisted coding practices ('vibe coding')" — [WebAIM Million 2026](https://webaim.org/projects/million/)
- Error counts vary widely by domain and language:
  - .gov pages averaged 18.5 errors; .ua 103.1; .cn 82.6 — [WebAIM Million 2026](https://webaim.org/projects/million/)
  - English pages averaged 46.0 errors; Chinese pages 136.2 — [WebAIM Million 2026](https://webaim.org/projects/million/)
- Caveat: "Absence of detected errors does not indicate that a page is accessible or conformant." Automated tools cannot detect every failure, so 95.9% is a **lower bound** — [WebAIM Million 2026](https://webaim.org/projects/million/)

### Inferences
- Element count rose 14.3% while errors rose 10.1%, so errors per element fell slightly. Read the error rise partly as a result of more complex pages rather than only declining author care.
- The 2026 reversal happened in the year after the EAA took effect (June 2025). This suggests the EAA has not yet shifted the global top-million homepages in a way automated tests can see (correlation only; the Million is global, not EU-specific).

### Gaps
- The WebAIM Million does not report keyboard-specific failure rates, so no automated headline figure for keyboard failures exists in this source.
- I could not confirm an exact original publication date for the 2026 report. The page shows only "last updated Mar 30, 2026" and a February 2026 crawl.

## Q2. Keyboard and sensory/cognitive barriers (including autistic users)

### Takeaway
Screen reader users' own rankings show the biggest problems are CAPTCHA and interactive widgets (menus, tabs, dialogs) that don't behave as expected; lack of keyboard access ranks 5th. Research on autistic users is small, mostly from W4A rather than CHI/ASSETS, and points to distraction, cognitive overload and higher cognitive cost rather than outright task failure.

### Cited Findings
- WebAIM Screen Reader Survey #10 (Dec 2023–Jan 2024, 1,539 responses) ranked these problem items in order: CAPTCHA; interactive elements that don't behave as expected; links/buttons that don't make sense; unexpected screen changes; lack of keyboard accessibility; missing/improper alt text; complex forms; and so on. The sample was not controlled — [WebAIM SRS #10](https://webaim.org/projects/screenreadersurvey10/); [WebAIM blog summary](https://webaim.org/blog/screen-reader-user-survey-10-results/)
- In the same survey, 71.6% of respondents navigate mainly by headings, and 91.3% use a screen reader on mobile — [WebAIM blog](https://webaim.org/blog/screen-reader-user-survey-10-results/)
- Eraslan, Yaneva, Yesilada and Harper (W4A 2017), "Do Web Users with Autism Experience Barriers When Searching for Information Within Web Pages?": an eye-tracking study found autistic users take more variable, less efficient search paths and are distracted by irrelevant elements — [METU record](https://avesis.ncc.metu.edu.tr/yayin/bc5871ed-2d86-4d81-82cb-87a44ae6cd5a/do-web-users-with-autism-experience-barriers-when-searching-for-information-within-web-pages); [PDF](https://open.metu.edu.tr/bitstream/handle/11511/67790/index.pdf)
- Yaneva et al. (W4A 2019): high-functioning autistic adults reached the same outcomes as neurotypical users but at a higher cognitive cost (reported via search-result summary of the paper; see [a11y-paradise review](https://a11y-paradise.onrender.com/reviews/69baf4c2caf48da433fe9754))
- A 2022 arXiv study looked at irrelevant on-screen animations as distractors for autistic vs non-autistic users during web search — [arXiv 2211.11993](https://arxiv.org/pdf/2211.11993v1)
- A mapping review of 23 papers (SBSI, Brazilian Computer Society) on web/mobile accessibility for autistic adults found barriers cluster around cognitive overload, processing, distinguishability and broken expectations. It found no studies at all on system-status visibility, predictability, response to actions or touchscreen interaction — [SBC mapping review](https://sol.sbc.org.br/index.php/sbsi/article/view/34351)
- An earlier literature review notes that only one autism web-accessibility paper it reviewed included an empirical study, and that study had only 4 autistic participants — [Uniud ACHI 2016 paper](https://air.uniud.it/retrieve/e27ce0c3-592b-055e-e053-6605fe0a7873/achi_2016_7_20_20377.pdf)

### Inferences
- Browser features that cut distraction (Reader Mode, reduced-motion preferences, blocking autoplay) target exactly the barriers the autism literature names. However, I found no study directly measuring these browser features with autistic users.

### Gaps
- I found no CHI/ASSETS 2025–2026 paper on autistic web users in the time available. The evidence base is thin and mostly pre-2023.
- The WebAIM SRS #10 page gives the problem ranking only, with no per-item percentages.

## Q3. Browser-level repair and mitigation features, with evidence of effect

### Takeaway
Browsers ship several repair features: AI image descriptions (Chrome since 2019, Edge since about 2022, Firefox on-device alt text in its PDF editor since Firefox 130), text and page zoom, and user-preference features. Independent evidence of their effect is sparse. The strongest measured result is Apple's pixel-based "Screen Recognition" (CHI 2021 Best Paper, for iOS apps, not the web). Independent tests of AI alt text show good results on photos but generic or wrong labels on charts, art and comics. Every vendor warns that its descriptions may be inaccurate.

### Cited Findings

**Chrome: "Get image descriptions from Google"**
- Launched October 11, 2019. Google said it would "only provide a description if we have reasonable confidence it's correct," and otherwise gives a simpler answer or nothing at all — [Google blog, 2019-10-11](https://blog.google/outreach-initiatives/accessibility/get-image-descriptions/)
- Usage during testing: more than 10 million descriptions, with "hundreds of thousands being added every day." This is a volume figure, not an accuracy rate — [Google blog, 2019](https://blog.google/outreach-initiatives/accessibility/get-image-descriptions/)
- Google's current help page warns descriptions may not be fully accurate. The screen reader says "No description available" when none is produced. Images are sent to Google servers but not stored. It supports about 16 languages — [Chrome Help](https://support.google.com/chrome/answer/9311597)

**Edge: automatic image descriptions ("Get image descriptions from Microsoft for screen readers")**
- Sends images without alt text to Azure Computer Vision. It skips images marked decorative, images under 50×50 px, very large images and explicit content. At launch it ran on Windows, Mac and Linux but not mobile (coverage from about 2022) — [Winaero](https://winaero.com/microsoft-edge-can-now-automatically-generate-alt-descriptions-for-images); [Windows Central](https://windowscentral.com/microsoft-edge-can-now-auto-generate-image-descriptions-improving-accessibility)
- Independent test by Fondazione LIA (2022-04-26):
  - Photos were described "quite well" to "very well," but some descriptions were wrong (a dog labeled as a cat).
  - Charts were identified only by type, and infographics were labeled simply "diagram."
  - Results on illustrations were inconsistent, and a comic was labeled "diagram."
  - Art descriptions were generic: Michelangelo's Pietà became "a statue of a man sitting on a throne."
  - [Fondazione LIA](https://www.fondazionelia.org/en/research-and-development/we-tested-edges-new-feature-to-provide-automatic-alt-texts-for-images/)

**Firefox: local, on-device alt-text generation**
- Firefox 130 (2024) added AI alt text for images in its built-in PDF editor. It uses a roughly 182M-parameter model (ViT encoder plus distilled GPT-2) running locally via Transformers.js/ONNX, so images never leave the device. Mozilla's stated end goal is general browsing for screen reader users — [Mozilla Hacks, May 2024](https://hacks.mozilla.org/2024/05/experimenting-with-local-alt-text-generation-in-firefox-nightly/embed/); [Simon Willison](https://simonwillison.net/random/onnx/)
- Mozilla acknowledges the model will make mistakes, especially on complex images. Its design makes authors review the text, labels it as auto-generated, and shows a disclaimer to readers outside Firefox — [Hidde de Vries summary](https://hidde.blog/links/alt-generation-in-firefox/); [Mozilla blog: help improve the model](https://blog.mozilla.org/en/?p=77294)

**Chrome for Android: Page Zoom / text scaling**
- Announced May 15, 2025 (Global Accessibility Awareness Day). Page Zoom enlarges text "without affecting the webpage layout," and can be set for all sites or per site from the overflow menu. It replaces the older global Text Scaling setting, which tended to break layouts — [9to5Google, 2025-05-15](https://9to5google.com/2025/05/15/google-global-accessibility-awareness-day-2025/); [Winaero](https://winaero.com/google-is-finally-improving-text-scaling-in-chrome-for-android); [Android Police](https://www.androidpolice.com/chrome-for-android-might-be-getting-this-feature-previously-reserved-for-desktop/)

**Accessibility-tree repair research (heuristic or ML labeling of unlabeled controls)**
- Screen Recognition (Zhang et al., Apple/CMU, CHI 2021 Best Paper) infers accessibility metadata from screen pixels for VoiceOver. It was trained on 77,637 screens from 4,068 iPhone apps — [arXiv 2101.04893](https://arxiv.org/pdf/2101.04893); [Apple ML Research](https://machinelearning.apple.com/research/creating-accessibility-metadata); [CMU news](https://www.cs.cmu.edu/news/2021/cmu-apple-team-improves-ios-app-accessibility)
  - A study with 9 screen reader users found it made existing apps more accessible, including previously inaccessible ones — [arXiv 2101.04893](https://arxiv.org/pdf/2101.04893)
  - A third-party summary (verify against the paper) reports:
    - 71.3% mAP across 13 UI element classes
    - grouping cut the elements to navigate per screen by 48.5% (21.8 to 12.1)
    - usability rated 3.73 vs 2.08 for standard VoiceOver
    - [a11y-paradise review](https://a11y-paradise.onrender.com/reviews/69cfdf77eeca48830a9af62d)
- AccessGuru (arXiv 2507.19549, 2025) combines accessibility checkers with LLMs to detect and correct WCAG violations in HTML — [arXiv 2507.19549](https://arxiv.org/pdf/2507.19549v1)
- A11y-Bench (TU Wien thesis, 2025) tested three open-weight LLMs on 51 real accessibility bug fixes, including missing ARIA labels. Only one issue was fully resolved, and the models often produced partial fixes or regressions — [TU Wien repositum](https://repositum.tuwien.at/handle/20.500.12708/224566)
- A11YRepair (arXiv 2606.21926, June 2026) argues that current automated program repair tools perform poorly on real accessibility violations, which often span multiple files — [Emergent Mind listing](https://www.emergentmind.com/papers/2606.21926)

**Overlays and the accessiBe FTC case**
- January 3, 2025: the FTC announced a proposed order (5-0 vote). accessiBe was to pay $1M over claims that accessWidget could make any website WCAG-compliant, and over paid reviews presented as independent — [FTC press release, 2025-01-03](https://www.ftc.gov/news-events/news/press-releases/2025/01/ftc-order-requires-online-marketer-pay-1-million-deceptive-claims-its-ai-product-could-make-websites)
- April 22, 2025: the FTC approved the final order (3-0). It bars unsubstantiated claims of WCAG compliance — [FTC press release, 2025-04-22](https://www.ftc.gov/news-events/news/press-releases/2025/04/ftc-approves-final-order-requiring-accessibe-pay-1-million); [FTC case page](https://www.ftc.gov/legal-library/browse/cases-proceedings/2223156-accessibe-inc)
- accessiBe denied the allegations, and it was a no-admission settlement — [Data Privacy & Security Insider](https://www.dataprivacyandsecurityinsider.com/2025/04/ftc-settles-with-accessibe-for-misleading-statements-about-wcag-compliance/)
- User sentiment:
  - About 67% of respondents in a WebAIM survey rated overlays/plugins as not effective. This is a secondary citation; I could not locate the figure on WebAIM's own survey pages — [CU Boulder Accessibility Minute, May 2023](https://www.colorado.edu/digital-accessibility/accessibility-newsletter/accessibility-minute-may-2023)
  - An ASSETS 2024 study of blind and low-vision users reports that up to 42% stopped using websites with overlays — [UW CREATE](https://create.uw.edu/accessibility-overlays-can-make-websites-less-accessible)

### Inferences
- Client-side AI repair works best where failures are local and visual (missing alt text on photos). It is weakest where meaning depends on context (charts, art, comics, button purpose). Missing alt text (53.1%) and empty links/buttons (46.3% / 30.6%) are high-frequency, so even partial repair reaches most pages. But a wrong label can be worse than a missing one, because users can't tell it is wrong. That is why the vendors use confidence thresholds or "may be inaccurate" disclaimers.
- Low contrast (83.9%) is the most common failure and is the one most fixable purely in the browser (forced colors / high-contrast modes, Reader Mode), without AI and without the risk of wrong labels.
- The FTC case makes clear that site-side overlays cannot be sold as achieving compliance. Browser-side repair is a user-controlled aid rather than a compliance claim, which is a distinct positioning.

### Gaps
- I found no recent primary source or measured user study, as of Oct 2026, for:
  - forced colors / Windows High Contrast effects or usage share
  - minimum font size settings
  - Reader Mode effects on comprehension for disabled users
  - `prefers-reduced-motion` adoption
  - focus-indicator enhancements
  - caret browsing
- I found no independent accuracy benchmark for Chrome's image descriptions; only Google's own statements exist.
- I could not confirm whether Firefox's local alt-text generation has expanded beyond the PDF editor into general browsing as of Oct 2026.
- I could not confirm whether the Edge image-description feature is still present and unchanged in 2026; sources date from 2021–2022.
- I could not locate the exact WebAIM survey wording or the source of the 67% overlay figure.

## Q4. European Accessibility Act (applies from 28 June 2025): effects on site compliance

### Takeaway
There is no EU-wide measurement of EAA-driven improvement as of Oct 2026. The only hard compliance figure found is a Spanish study (2% of 204 corporate sites fully compliant). Commentary says enforcement was light in 2025 and is moving to market-surveillance inspections in 2026. Globally, WebAIM's 2026 numbers got worse, not better.

### Cited Findings
- Tech4Access audited 204 Spanish corporate websites with axe-core: only 2% were fully compliant, and nearly 1 in 4 failed essential WCAG criteria. I saw only the axe-con session description, not the full report — [Deque axe-con session](https://www.deque.com/axe-con/?p=526)
- Clifford Chance (May 2026) briefing on the "accessibility gap" and the EAA's strategic implications — [Clifford Chance, May 2026](https://www.cliffordchance.com/briefings/2026/05/bridging-the-accessibility-gap-the-strategic-imperative-of-the-european-accessibility-act.html)
- Vendor and consultancy commentary:
  - 2025 enforcement was mostly guidance and warnings, with market-surveillance inspections ramping up in 2026 — [PageCrawl](https://pagecrawl.io/blog/european-accessibility-act-compliance-monitoring-wcag.md); [Kolonell](https://kolonell.com/en/blog/eaa-accessibility-audit-remediation-budget-b2b-site-2026)
  - Penalty ceilings vary by member state (sources cite up to EUR 60,000 in Ireland and up to EUR 500,000 elsewhere; vendor figures are inconsistent) — [Kolonell](https://kolonell.com/en/blog/eaa-accessibility-audit-remediation-budget-b2b-site-2026); [Skynet Technologies](https://www.skynettechnologies.com/blog/european-accessibility-act-faqs)
- Transition: some companies cite a 28 June 2030 deadline for existing services, e.g. Falck's statement that its sites are "partially compliant" — [Falck](https://www.falck.us/accessibility/)
- WebAIM Million 2026 does not mention the EAA. Global homepage failures rose to 95.9% in the first post-EAA crawl (Feb 2026) — [WebAIM Million 2026](https://webaim.org/projects/million/)

### Inferences
- The transitional period to 2030 for existing services, together with slow-starting enforcement, plausibly explains why no measurable compliance jump shows up yet (inference, not measured).

### Gaps
- No EU-wide or EU-TLD-specific before/after compliance dataset was found. Most 2026 EAA material comes from vendors with a commercial interest.
- I did not verify the penalty figures against national transposition laws.
