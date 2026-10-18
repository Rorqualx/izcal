# Essays under review (pasted by the user, 14 October 2026; author unknown, likely another AI)

## Essay 1: "Arc is the design-layer equivalent of Opera Presto"

The Browser Company raised heavily, shipped Arc with a genuinely rethought chrome — sidebar tabs, spaces, split view, no prominent URL bar — and discontinued it as a product in 2025, moving to Dia. The stated reasons are informative: the chrome rearrangement didn't convert beyond a power-user niche, and maintaining a Chromium fork consumed the company.

Two lessons: (1) Chrome rearrangement is a solved space. Vertical tabs, split view, workspaces, sidebars, tab groups are shipped in Edge, Vivaldi, Brave, Firefox. No differentiation left in where the tabs go. (2) Arc's failure may have been engine cost, not design; if the real constraint was Chromium maintenance, that argues for HGBA-3.0's engine reuse. The public record does not separate the two.

Fixed vs free in the UI layer:
- Navigation: URLs/origins are the security primitive (fixed); presentation/memory/scoping of navigation is free.
- Page behavior: sites control scrolljacking, autoplay, popups, login walls, CAPTCHAs (fixed); whether the browser overrides them is free.
- Consent: cookie banners are sites' response to regulation (fixed); whether the browser answers banners for the user is free.
- Accessibility: author errors are root cause (95.9% of homepages fail WCAG) (fixed); whether browser corrects the tree before handing it to AT is free.
- Tabs: platform has no concept of a tab; unit could be tab, task or identity.
- Notifications: sites request, OS delivers; browser could broker, batch, prioritize.
- Cost: pages load what authors chose; browser could expose CPU/network/battery as a budget.
- Identity: cookies and storage partitions; identity could be the primary organizing principle.

Group 1 (shipped, ignored): Firefox Multi-Account Containers (shipped for years, hidden, low adoption); automatic banner rejection (GPC legally recognized in some jurisdictions; Firefox cookie banner reduction works on a subset); reader mode (shipped everywhere, rarely used); tab grouping/workspaces (cosmetic because browser can't tell when a task is done).
Group 2 (blocked outside browser): consent enforcement needs sites to honor signal (coordination problem); CAPTCHAs/login walls/scrolljacking/autoplay are author decisions, override breaks business model; task-as-unit needs observing completion, AI could infer it but AI is off.

Where unilateral action still works:
1. Accessibility remediation: browser computes corrected semantic tree (labels, empty buttons, low contrast, keyboard traps) and hands it to AT. Precedent partial (Reader Mode, Forced Colors). Oracle strong (a11y spec + screen reader), same shape as HGBA-3.0 B8 dual-trait differential. Risk: silent meaning changes; needs per-correction audit trail.
2. Cost as first-class budget: per-page CPU/network/battery budget. Precedent weak: Chrome Data Saver shipped and discontinued; battery saver binary. Risk: right-default problem.
3. Notification brokering: batch, prioritize, revoke per site. Precedent strong in mobile OSes (Android notification channels, iOS summaries); browsers give raw channel.

Stance: browser is the user's agent, not a neutral renderer; Chrome claims neutrality but drifts toward being the site's agent (Manifest V3, AI-in-chrome). Overrides break "author is sovereign"; sites detect and block overrides. None requires AI.

Maturity table: corrected a11y tree as primary AT interface (research); cost budget (prototype at best); identity as organizing principle (shipped, Firefox Containers; blocked by adoption); notification brokering in a browser (shipped in mobile OSes); task as unit (research); consent auto-answer with enforcement (partial, GPC, Firefox).

Falsifiers: if Arc failed from Chromium maintenance cost, UI space may be more open; if containers were widely used, identity thesis is wrong; if banner rejection worked at scale, consent is solved. Author says it reasons from general knowledge, not fresh search; Arc details, container adoption and reader-mode adoption need checking.

Conclusion: don't write a spec; pick one unilateral area (accessibility remediation strongest) and test its falsifier.

## Essay 2: "Accessibility Remediation as a Browser-Level Unilateral Action"

Claims:
- WebAIM Million 2026: 95.9% of homepages had detectable WCAG 2 failures, up from 94.8%; 56.1 errors/page, up 10.1% from 51; first increase in six years. WebAIM blog: "the data indicate that the web is less accessible than it was a year ago". Quotes: inaccessibility "is often framed as an awareness, technical, training, or legal problem" ... barriers persist because "their impact on disabled users remains easy for so many organizations to overlook"; issues persist "when so many proven technical solutions are readily available".
- Overlays: screen readers hook into the a11y tree as DOM is constructed, typically T+200ms to T+500ms, overlay JS executes T+800ms to T+2000ms, so the SR navigates the original for 500–2500 ms.
- Studies: SR users reported overlays "increased the inaccessibility of the website, especially when it clashed with their screen readers"; overlay SR mode "interrupted their screen reader and often, the two were not compatible"; 24 respondents cited overlay SR conflicting with their own SR.
- FTC fined accessiBe $1 million because claims "were not supported by competent and reliable evidence". Over 800 businesses using AI overlays were sued in 2023–2024.
- A 2026 research paper on assistive agents: "unverified generation improved and regressed pages at similar rates (24 improvements against 20 regressions across the 100 trials)"; LLMs fixed typography while breaking perception-dependent properties. A verified repair instrument, "audit-inject-verify loop that accepts a change only if violations strictly decrease", made regression "impossible by construction"; in a real browser it detected 57 of 57 seeded violations with no false positives and rejected 126 of 126 adversarially harmful candidates.
- Page repair research: "users preferred to use the repaired version in 26 out of 38 subjects" with a 17% readability improvement.
- Legato browser exists and targets this; developer forum post: "Almost all of it has been done by sighted people, including me, with VoiceOver switched on — and that is not the same thing as using it..." Legato: rules-based repair (button names, headings, focus order, dialog traps) free, AI portion paid; built-in screen reader toggle.
- Reader Mode and Forced Colors widely available, rarely used.
- AOM proposal designed to let authors modify the a11y tree, not browsers. W3C: "the accessibility tree and the DOM tree are parallel structures"; "the intent is to maintain consistency in how the WAI-ARIA information looks to authors and how it is manipulated in the DOM by scripts".
- Courts have consistently held runtime fixes do not substitute for source-code compliance.
- WebAIM "semantic automation": "user agents, such as browsers and screen readers, create meaning and relationships where the presented meaning and relationships are missing, ambiguous, or incorrect". JAWS and VoiceOver auto-associate adjacent text with form controls; VoiceOver added this in iOS 5 and Lion. "When computers guess, the results are often not very good. But guessing is usually better than nothing". Complaints: "there's no way to know if label text is actually associated..." and "there's no option to disable this functionality".
- Research on LLM-driven HTML restructuring found "significant improvements over the original webpages regarding screen reader navigation experience"; research on automatic heading generation via extensions injecting headers and in-page navigation.
- CVE-2026-95301: Chrome vulnerability allowing a compromised renderer to read cross-origin accessibility tree data (page text, form values, ARIA, frame URLs) by joining the accessibility broadcast.
- Proposed design: corrected tree computed alongside author tree (not in place); opt-in per site or globally, one-toggle reversible; every correction accepted only if it strictly reduces violations and adds none; user can enumerate corrections; correction logic renderer-local; exposed via standard platform APIs (NVDA, JAWS, VoiceOver, Orca). Site should not be able to detect correction.
- Open: Q-1 false-positive rate on organic violations; Q-2 does it improve AT user experience; Q-3 interaction with site's own a11y widgets.
- Next step: experiment on a corpus: violations corrected, new violations introduced, SR user preference. If it fails, put the budget into notification brokering instead.
