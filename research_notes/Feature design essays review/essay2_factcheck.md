# Fact-check: Essay 2 ("Accessibility Remediation as a Browser-Level Unilateral Action") and the accessibility parts of Essay 1

Checked 14 October 2026 against primary sources wherever they could be reached. Verdicts: correct / partly correct / wrong / unverifiable / likely fabricated.

**Overall:** The essay is mostly built on real sources. Almost every quotation exists word for word. The main problems are misattribution and loss of context:
- The "26 of 38" result comes from a mobile-friendliness paper and is not about accessibility. Its "subjects" are web pages, not people.
- The study quotes all come from one paper, and one of them is the authors' summary, not a user's words.
- The FTC "quote" is a paraphrase, and the payment was consumer redress, not a fine.
- The overlay lawsuit count is too low.
- The claim about courts overstates rulings that were really about mootness, plus settlements.
- The overlay timing figures have no source.
- CVE-2026-95301 is real. The essay's description matches the public Chromium bug, but leaves out an important precondition.

---

## 1. WebAIM Million 2026 figures

| Claim | Verdict | Primary-source text / correction |
|---|---|---|
| 95.9% of homepages had detectable WCAG 2 failures, up from 94.8% | **Correct** | "95.9% of home pages had detected WCAG 2 failures. This number increased from 94.8% in 2025". Note: 95.9% is the same figure as 2024, so 2026 went back to the 2024 level. The scan ran in February 2026. |
| 56.1 errors per page, up 10.1% from 51 | **Correct** | "56,114,377 distinct accessibility errors were detected—an average of 56.1 errors per page. The number of detected errors increased 10.1% since the 2025 analysis which found 51 errors/page." |
| "First increase in six years" | **Correct (paraphrase)** | WebAIM's wording is "reversing a trend of small improvements each of the previous 6 years." |
| Essay 1: "95.9% of homepages fail WCAG" | **Partly correct** | It is "detected WCAG 2 failures" from WAVE automated testing. WebAIM says the real non-conformance rate is "certainly" higher. Saying 95.9% "fail WCAG" understates the problem rather than overstating it. |

Source: https://webaim.org/projects/million/ (2026 report, data from February 2026)

### WebAIM blog quotes
All four quotes are **correct and verbatim**. They come from **"Tolerating Inaccessibility", Christopher Phillips, WebAIM blog, 30 April 2026**: https://webaim.org/blog/tolerating-inaccessibility/
- "Despite an increase in interest and the tireless efforts of digital accessibility professionals all over the world, the data indicate that the web is less accessible than it was a year ago."
- "Digital inaccessibility is often framed as an awareness, technical, training, or legal problem—and it is all of those things. But that is only part of the story; accessibility barriers persist not only because of these challenges, but also because their impact on disabled users remains easy for so many organizations to overlook."
- "It is incredibly frustrating to see the same set of common issues persist year after year, when so many proven technical solutions are readily available."

The essay should name the post and author. "WebAIM blog" on its own is too vague.

---

## 2. Overlay timing claim: screen reader at T+200–500 ms, overlay JS at T+800–2000 ms, SR reads the original for 500–2500 ms

**Verdict: unverifiable; probably invented numbers; the mechanism is technically misleading.**

- No primary source gives these numbers. Searches found only general statements from anti-overlay vendor blogs (TestParty) that overlay scripts run "after parse and often after first paint". None of them includes measurements.
- The mechanism is wrong. Screen readers do not "hook into" the tree once at a fixed time and then keep reading a stale copy. The browser keeps the platform accessibility tree up to date as the DOM changes. Chromium's docs say Blink marks nodes "dirty", and "The infrastructure crawls the dirty nodes and creates a tree mutation and propagates it to all client interfaces". Events are generated "based on changes to the accessibility tree". (https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/browser/how_a11y_works_3.md)
  - Overlay DOM patches therefore do reach assistive technology (AT) once they are applied.
  - The real problems are different:
    - a short window before the patch is applied;
    - virtual-buffer refreshes that happen in the middle of reading (JAWS and NVDA browse-mode buffers);
    - pages that change dynamically after the overlay's first pass;
    - the overlay's own UI and its "screen reader mode" conflicting with the user's screen reader. This last one is the main harm reported in the user research (see section 3).
- **Correction:** Say that overlays apply fixes after load, through script, and can conflict with the user's AT. Remove the millisecond figures unless a measurement is cited.

---

## 3. Overlay user-study quotes

**Source found:** Tlamelo Makati, Garreth W. Tigwell, Kristen Shinohara, *"The Promise and Pitfalls of Web Accessibility Overlays for Blind and Low Vision Users"*, ASSETS 2024 (October 2024). DOI 10.1145/3663548.3675650. Full text: https://dl.acm.org/doi/fullHtml/10.1145/3663548.3675650. Summary page: https://create.uw.edu/accessibility-overlays-can-make-websites-less-accessible (31 October 2024). The study had 47 blind and low-vision survey respondents and 12 interviewees.

| Claim | Verdict | Exact text |
|---|---|---|
| SR users reported overlays "increased the inaccessibility of the website, especially when it clashed with their screen readers" | **Quote verbatim; attribution partly wrong** | This is the **authors' summary** in the Discussion (§6.1), not words from users: "Instead, the experiences of many participants was that overlays increased the inaccessibility of the website, especially when it clashed with their screen readers." |
| Overlay SR mode "interrupted their screen reader and often, the two were not compatible" | **Correct (verbatim)** | "...they described how overlays made it more challenging to navigate the website as most of the time, the overlay screen reader interrupted their screen reader and often, the two were not compatible." |
| "24 respondents" cited overlay SR conflicting with their own SR | **Correct** | "overlay screen reader conflicting with personal screen reader (24; 51.06%)". That is 24 of 47. |
| "Studies" (plural) | **Partly correct** | All three items come from **one** study. Other studies support the general finding but are not the source of these quotes, for example *Evaluating AI-Powered Website Accessibility Overlays* (ACM, 2025, https://dl.acm.org/doi/full/10.1145/3663547.3759761). |

---

## 4. FTC and accessiBe

| Claim | Verdict | Correction |
|---|---|---|
| "FTC fined accessiBe $1 million" | **Partly correct** | The amount is $1 million, but it is not a fine or civil penalty. The FTC calls it a "$1 million consumer redress payment" that "may be used to provide refunds to consumers". The proposed order was announced on **3 January 2025** and the final order was approved 3-0 on **22 April 2025**. |
| Claims "were not supported by competent and reliable evidence" | **Misquoted (paraphrase shown as a quote)** | That wording appears in neither press release nor the order package. The complaint language is that the claims were "false, misleading, or unsubstantiated, in violation of the FTC Act". "Competent and reliable evidence" is the **order's forward-looking requirement**: accessiBe "must possess competent and reliable evidence to substantiate" future compliance claims. |

Sources:
- https://www.ftc.gov/news-events/news/press-releases/2025/01/ftc-order-requires-online-marketer-pay-1-million-deceptive-claims-its-ai-product-could-make-websites (3 January 2025)
- https://www.ftc.gov/news-events/news/press-releases/2025/04/ftc-approves-final-order-requiring-accessibe-pay-1-million (22 April 2025)
- https://www.ftc.gov/system/files/ftc_gov/pdf/letters.222_3156_-_accessibe_final_consent_package_without_signatures-3.pdf

## 5. "Over 800 businesses using AI overlays were sued in 2023–2024"

**Verdict: wrong. The figure is understated, and the scope is mislabelled.**

UsableNet's year-end reports, as quoted in secondary sources, give these figures:
- **2023:** 933 businesses with an overlay or widget sued, about 30% of tracked suits.
- **2024:** 1,023 companies with "accessibility overlays or widgets" sued, about 25%.

That is roughly **1,950 over the two years**, or more than 900 in *each* year. The figures cover overlays and widgets generally, not specifically "AI overlays".

UsableNet's own site could not be reached from here, so these come from:
- QualityLogic, January 2025: https://www.qualitylogic.com/knowledge-center/accessibility-industry-update-january-2025/
- Accessibility.works: https://www.accessibility.works/blog/2023-ada-website-lawsuits-legal-statistics/

The 2025 report reportedly gives more than 1,400. Check the UsableNet PDFs before quoting exact figures.

---

## 6. The 2026 "assistive agents" paper

**Verdict: correct paper, but one important qualifier is dropped.**

The paper is Wanscher, Lorensen, Shafiq, Tourchi Moghaddam, Alipour, *"From Blind Edits to Verified Repair: Building Trustworthy User-Side LLM Agents for Web Accessibility"*, arXiv:2608.24913v1, submitted 26 July 2026. Links: https://arxiv.org/abs/2608.24913 and https://arxiv.org/html/2608.24913v1

- "Assistive agents": verbatim. The abstract opens "Assistive agents that adapt web pages on the user's side...".
- "24 improvements against 20 regressions across the 100 trials": verbatim. The full phrase continues "...of the five models that produced injectable CSS". Six models (7–14B parameters) were tested on 20 live sites.
- "fixing typography while breaking perception-dependent properties": verbatim.
- "audit-inject-verify loop that accepts a change only if violations strictly decrease": verbatim.
- "57 of 57 seeded violations with no false positives" and "126 of 126 adversarially harmful candidates": verbatim.
- **Dropped qualifier:** the paper says "regression **on the automated checks** is impossible by construction". The essay presents this as an unqualified guarantee.
  - The paper limits the claim. Residual risk is "confined to regressions its checks cannot see".
  - The auditor checks only part of the problem: 2 axe-core rules plus 5 custom probes.
  - The system only **injects CSS**. It does not repair names, roles, or the accessibility tree.
  - The 57/57 and 126/126 results came from stub generators, not from LLM output.
  - The essay's design rule, "accepts only if it strictly reduces violations and adds none", is therefore only as strong as the auditor. It cannot catch harmful meaning changes.

## 7. "Users preferred to use the repaired version in 26 out of 38 subjects", 17% readability improvement

**Verdict: the quote is verbatim, but it is misattributed or misused. The paper is not about accessibility.**

Source: Mahajan, Abolhassani, McMinn, Halfond, *"Automated Repair of Mobile Friendly Problems in Web Pages"* (MFix), ICSE 2018. PDF: https://eprints.whiterose.ac.uk/127563/8/paper.pdf

- Text: "we found that the users preferred to use the repaired version in 26 out of 38 subjects, three subjects received equal preference... and only nine subjects received a preference for using the original version". Also: "the readability rating of the repaired pages showed a 17% improvement over original versions (original = 5.97, repaired = 6.98)".
- "Subjects" means **38 web pages**, not 38 people. Ratings came from paid crowd workers ($0.65 per survey, 240 surveys), not AT users.
- The repair fixes **mobile-friendliness** (Google Mobile-Friendly Test issues: font size, tap targets, viewport). It is not screen-reader accessibility.
- In the second survey variant, users preferred the *original* page in 22 of 38 subjects.

It cannot support claims about AT users preferring repaired pages.

## 8. Legato browser

**Verdict: exists. The quote is correct. Some details are slightly off.**

- AppleVis forum post by **Dmytro Liesniak**, about late August 2026: "I built a Mac browser that repairs pages for VoiceOver — and I need blind testers, because mine were all sighted". https://www.applevis.com/forum/macos-mac-apps/i-built-mac-browser-repairs-pages-voiceover-i-need-blind-testers-because-mine
- The quote is verbatim: "Almost all of it has been done by sighted people, including me, with VoiceOver switched on — and that is not the same thing as using it."
- Maker: Qiksy / Qiksy Studio (https://qiksy.app/legato/). It is a "Chromium browser with a repair engine" for macOS.
- Free: button names, real headings, focus order, dialog escapes, and a built-in voice. The free features are on-device, with "no account and no key". Paid ("Care", $49 per year or $6 per month): AI features only (image descriptions, plain-language rewrite, form explanation), with a 30-day trial.
- "Built-in screen reader toggle" is **partly correct**. It has its own TTS voice and one shortcut to switch between that voice and the user's reader (NVDA/JAWS/VoiceOver/Orca). It does not replace the screen reader.
- Some product details are inconsistent:
  - The site claims support for NVDA/JAWS/Orca, but the forum post says macOS only.
  - Voice and language counts differ: 21 languages and 27 voices in the forum post, 22 and 46 on the site.
- **Effect on Essay 1:** this is a shipping product. The maturity table's "corrected a11y tree as primary AT interface: research" is out of date, although the product has no evidence of testing with blind users.

## 9. AOM design intent and the W3C quotes

| Claim | Verdict | Source |
|---|---|---|
| AOM was designed to let authors, not browsers, modify the a11y tree | **Correct** | WICG AOM README: "This effort aims to create a JavaScript API to allow developers to modify (and eventually explore) the accessibility tree for an HTML page." https://github.com/WICG/aom. The virtual-nodes part was dropped over "privacy concerns". |
| "the accessibility tree and the DOM tree are parallel structures" | **Correct (verbatim)** | WAI-ARIA 1.2 §7 "Accessibility Tree" (https://www.w3.org/TR/wai-aria-1.2/); also in the 2012 WAI-ARIA UA Implementation Guide §1.2, which adds "Roughly speaking the accessibility tree is a subset of the DOM tree." |
| "the intent is to maintain consistency in how the WAI-ARIA information looks to authors and how it is manipulated in the DOM by scripts" | **Verbatim, but taken out of context** | WAI-ARIA 1.2 §8 "Implementation in Host Languages". It concerns how host languages (HTML, SVG, etc.) should present ARIA *attributes* consistently. It says nothing about keeping the a11y tree consistent with the DOM, and nothing against UA repair. |

## 10. "Courts have consistently held runtime fixes do not substitute for source-code compliance"

**Verdict: wrong / overstated.**

No ruling was found that holds this in general. What exists is:
- **Procedural mootness rulings.** The defendant failed to carry its burden of proving the site was fixed. Examples:
  - *Quezada v. U.S. Wings* (S.D.N.Y., December 2021): the defendant "has not shown that they have undoubtedly fixed accessibility issues". accessiBe audit reports were offered as evidence.
  - *Angeles v. Grace Products* (S.D.N.Y., September 2021).
- **Settlements and complaints:**
  - *Murphy v. Eyebobs* (settled October 2021).
  - DRA v. ADP (settled December 2021). This agreement says overlays "will not suffice", but that is a contract term, not a court finding.
- **At least one ruling the other way.** *Erkan v. David A. Hidalgo, MD, P.C.* (E.D.N.Y.) was reportedly dismissed partly on remediation that included accessWidget. That account comes from accessiBe's own marketing and the primary opinion was not checked.

Sources:
- https://www.lflegal.com/2021/11/overlay-legal-update/
- https://www.kaufmandolowich.com/news-resources/examining-new-york-court-decisions-on-website-accessibility-claims-12-13-24-new-york-law-journal-by-jennifer-e-sherven-esq-and-megan-beloff-attorney-bar-admission-pending/ (12 December 2024)
- https://accessibe.com/blog/court-affirms-accesswidget-role-in-accessibility (vendor-sourced)

## 11. WebAIM "Semantic Automation" article

**Verdict: correct. All quotes are verbatim, with two points of context.**

Source: Jared Smith, "Semantic Automation", WebAIM blog, **4 November 2011**. https://webaim.org/blog/semantic-automation/

- "Semantic automation is when user agents, such as browsers and screen readers, create meaning and relationships where the presented meaning and relationships are missing, ambiguous, or incorrect." This is verbatim.
- JAWS and VoiceOver "will implement algorithms to auto-associate adjacent text to the control": correct.
- "With the release of iOS5 and Lion, VoiceOver will now auto-associate the adjacent text." This is correct as the author's observation. A commenter (Paul J. Adam) notes that Apple did not document this as a new feature.
- "When computers guess, the results are often not very good. But guessing is usually better than nothing.": verbatim.
- "there's no way to know if label text is actually associated or if VoiceOver or JAWS is just assuming it should be. And there's no option to disable this functionality.": verbatim.
- **Context:** these "complaints" are the author's concern about **evaluation and testing**, not complaints from users. The article is 15 years old.

## 12. LLM-driven HTML restructuring and automatic heading generation

| Claim | Verdict | Source |
|---|---|---|
| LLM restructuring: "significant improvements over the original webpages regarding screen reader navigation experience" | **Correct (verbatim)** | *LLM-Driven Optimization of HTML Structure to Support Screen Reader Navigation*, arXiv:2502.18701 (February 2025): https://arxiv.org/abs/2502.18701. Published as *From Cluttered to Clear: Improving the Web Accessibility Design for Screen Reader Users in E-commerce With Generative AI* (ACM, 2025, DOI 10.1145/3663547.3746353). The evaluation is small and covers e-commerce pages. |
| Automatic heading generation via extensions that inject headers and in-page navigation | **Correct** | Resende Silva et al., *In-Page Navigation Aids for Screen-Reader Users with Automatic Topicalisation and Labelling*, ACM TACCESS 2024, DOI 10.1145/3649223. Small studies: 8 screen-reader users, then 7 blind participants testing the browser extension. |

## 13. CVE-2026-95301

**Verdict: exists. The essay's description is accurate to the public bug report but leaves out a key precondition.**

The doubt about high CVE numbers does not apply here. Chrome's CNA (the body that assigned this CVE ID) uses high ID blocks, and the ID is valid.

- **CVE record and NVD.** Published 29 September 2026. "Missing authorization in Extensions in Google Chrome prior to 154.0.8037.57 allowed a remote attacker who had compromised the renderer process to bypass site isolation via a crafted HTML page. (Chromium security severity: High)". CWE-862, CVSS 8.1.
  - https://www.cve.org/CVERecord?id=CVE-2026-95301
  - https://nvd.nist.gov/vuln/detail/CVE-2026-95301
- **Chrome Stable release notes** (September 2026): "High CVE-2026-95301: Missing authorization in Extensions. Reported by OGINOME Tomohito on 2026-07-29 [$5,000]". https://chromereleases.googleblog.com/2026/09/stable-channel-update-for-desktop_0856730748.html
- **Chromium issue 540265100** (public, Fixed/Verified). Title: "Site Isolation bypass: the ungated extensions RendererAutomationRegistry handler lets a compromised web renderer read cross-origin/cross-profile accessibility trees (page text, frame URLs)." The body says the browser "broadcasts every tab's full serialized accessibility tree — visible text, non-password form-field values, ARIA, cross-site frame URLs". https://issues.chromium.org/issues/540265100
- **Omitted condition:** the leak only happens "while any legitimate `chrome.automation` consumer (e.g. ChromeVox on ChromeOS) is active". The bug calls the realistic impact "ChromeOS-centric". It also requires a compromised renderer.
- **Relevance to the design:** shipping accessibility trees across process boundaries is a real attack surface. That supports keeping correction logic local to the renderer, but every extra consumer of the tree increases exposure.

## 14. Do browsers already repair accessibility? Can users switch between author and repaired trees?

**Verdict: browsers already do targeted repair, so Essay 1's "precedent partial" understates it. No full author/repaired tree switch was found.**

Existing features:
- **Chrome, "Get image descriptions from Google"** (October 2019). It sends unlabeled images to Google and inserts descriptions into what the screen reader reads. It can be turned on or off from the context menu or Settings. https://support.google.com/chrome/answer/9311597 and https://blog.google/outreach-initiatives/accessibility/get-image-descriptions/
- **Edge, "Get image descriptions from Microsoft for screen readers"** (Insider December 2021, Stable 2022). It uses Azure Computer Vision for images that have no alt text, and skips images marked decorative. https://winaero.com/microsoft-edge-can-now-automatically-generate-alt-descriptions-for-images
- **Firefox, alt text in the PDF editor** (Firefox 130, 2024). An on-device model (about 182M parameters) *suggests alt text to the author* when they add an image to a PDF. It is an **authoring aid, not repair** of pages for AT users. https://hacks.mozilla.org/2024/05/experimenting-with-local-alt-text-generation-in-firefox-nightly/
- **Apple VoiceOver Screen Recognition** (iOS 14, 2020). It is an OS-level feature that infers UI elements in inaccessible apps. It belongs to the OS and screen reader, not to Safari.
- **Screen-reader heuristics** such as JAWS and VoiceOver label guessing (section 11) are long-standing repair done in the AT layer.

On switching: the image-description features have a per-feature on/off control. No browser offers the essay's design, a user switch between the whole author tree and a whole repaired tree. Legato's site does not say it keeps two trees either.

## 15. Is "invisible to the site" achievable?

**Verdict: only partly. Forced Colors is definitely visible to sites.**

- `forced-colors` is a standard CSS media feature that authors can query: "used to detect if the user agent has enabled a forced colors mode". It can be read in CSS and through `matchMedia`. It has been Baseline across browsers since September 2022. https://developer.mozilla.org/en-US/docs/Web/CSS/@media/forced-colors
  - So Essay 1's use of Forced Colors as a precedent for an override the site cannot detect is weak.
  - `prefers-contrast` and `prefers-reduced-motion` are also exposed to sites.
- Corrections made only in the platform a11y tree, never written back to the DOM, are not directly readable through the DOM. Chrome's image descriptions work this way. AOM's privacy discussion shows that standards bodies avoid APIs that reveal AT use.
- Some leaks are still possible:
  - behaviour-based inference, such as focus and navigation patterns changed by corrected focus order or headings;
  - timing and performance side-channels from computing the tree;
  - any correction that touches the DOM or CSS. Legato and the arXiv paper both inject into the page.
- Treat "should not be able to detect" as a goal to test, not a property already guaranteed.

## 16. Other accessibility claims in Essay 1

- **"Forced Colors widely available, rarely used": partly wrong.** Microsoft reported about **4% of active Windows devices** using High Contrast Mode, which it described as the most-used built-in Windows AT. Source: CSSWG F2F slides, February 2019, https://lists.w3.org/Archives/Public/www-archive/2019Feb/att-0000/CSSWG_F2F_High_Contrast.pdf. Secondary: https://smashingmagazine.com/2022/06/guide-windows-high-contrast-mode/. That is not "rarely used" in absolute terms.
- **"Reader mode rarely used": unverifiable.** No primary usage data was found or checked.
- **Maturity "corrected a11y tree as primary AT interface: research": out of date.** Legato is a shipping product (August 2026), and Chrome/Edge have done narrow tree repair since 2019 and 2022.
