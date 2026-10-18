# Independent review: the two feature-design essays

*Review of two pasted essays ("Arc is the design-layer equivalent of Opera Presto" and "Accessibility Remediation as a Browser-Level Unilateral Action") as design input to HGBA. 14 October 2026. Three independent reviewers: two fact-checkers working from primary sources, and one design reviewer who did no web research and read only the essays and the HGBA documents. Their notes are in `research_notes/Feature design essays review/`.*

## Verdict

The essays reach a defensible conclusion by a weak route. Their destination is worth keeping. Chrome rearrangement is not where a new browser differentiates; accessibility repair is a real opportunity; and any repair must be opt-in or disclosed, reversible and auditable. But the reasoning that gets them there has three serious faults:

1. **The Arc premise is wrong.** The Browser Company's own letter gives product reasons and never cites Chromium maintenance cost.
2. **Several "browsers do nothing here" claims are wrong.** Notifications are already brokered, resource budgets have shipped with measured effects, and Firefox's cookie-banner blocker has been withdrawn.
3. **The proposed oracle is weaker than claimed.** "Strictly fewer checker violations" measures the checker, not the user, and the evidence cited for it is narrower than the essay says.

Essay 2's quotations are mostly real and verbatim, which is better than earlier pasted material. But four are misattributed or stripped of context, one figure is wrong, and the overlay timing figures have no source.

---

## 1. Fact-check: Essay 1 (UI layer and Arc)

| Claim | Verdict | What the record says |
|---|---|---|
| Arc "discontinued as a product in 2025, moving to Dia" | **Partly correct** | Active feature development stopped on 26 May 2025 (Josh Miller, "Letter to Arc members 2025"). Arc still receives Chromium upgrades and security fixes. Dia entered invite-only beta on 11 June 2025, on the same Chromium base. Atlassian bought The Browser Company for about $610M (announced 4 September 2025, closed 21 October 2025) |
| Reasons: chrome rearrangement stayed niche, **and Chromium maintenance consumed the company** | **First half correct, second half wrong** | The letter says Arc looked like "a highly specialized professional tool (like a video editor)", was "too different, with too many new things to learn, for too little reward", "lacked cohesion", was "bloated", and that only 5.52% of daily users regularly used more than one Space. It says "Arc isn't just a Chromium fork", and the technical debt it names is in the app layer (Swift UI frameworks). Dia shipping on the same Chromium base also cuts against the engine-cost reading |
| "The public record does not separate the two failure modes" | **Wrong** | The company's own account separates them, and it points to product, not engine |
| Vertical tabs, split view, workspaces, tab groups shipped in Edge, Vivaldi, Brave and Firefox | **Partly correct** | Edge, Vivaldi and Firefox ship vertical tabs, split view and groups (Firefox split view in 149, March 2026). Brave and Firefox have no native workspaces. The essay omits Chrome, which added split view (February 2026) and vertical tabs (April 2026); that strengthens its point. Microsoft removed Edge Workspaces sharing and is retiring Collections in 2026 |
| Firefox Multi-Account Containers: shipped, hidden, low adoption | **Partly correct** | 411,602 users on addons.mozilla.org (14 October 2026). No Mozilla statement on adoption was found. Total Cookie Protection has been on by default for all desktop users since June 2022, which removed most of the privacy reason to use containers |
| Firefox cookie-banner reduction "ships and works on a subset" | **Wrong** | It was on by default only in German private windows (Firefox 120), was turned off in pre-release builds in early 2025, and Mozilla's support page now says it is no longer available |
| GPC "legally recognized in some jurisdictions"; consent is a coordination problem | **Understated** | Sephora paid $1.2M in 2022 partly for ignoring GPC. Colorado has required honoring it since July 2024 and Connecticut since January 2025. California, Colorado and Connecticut ran a joint GPC enforcement sweep from September 2025. California AB 566 requires **browsers** to offer an opt-out signal from 1 January 2027. A USENIX Security 2025 study found 43–45% of sites that sell or share data honored GPC. This looks like an enforcement gap with regulators actively closing it, not an unsolvable coordination problem |
| Reader mode "widely ignored" | **Unverifiable** | No vendor publishes usage. A 2019 study found Reader View worked on only 41% of popular pages, which is a competing explanation for low use |
| Forced Colors "rarely used" | **Questionable** | Microsoft has said about 4% of active Windows devices use High Contrast |
| Cost budget: "prototype at best"; Chrome Data Saver is the precedent | **Wrong** | Data Saver was a compression proxy, not a budget (desktop extension deprecated in 2019; Lite mode ended in Chrome 100, March 2022). Real budgets have shipped with measured effects. Chrome's heavy-ad intervention unloads ads over 4 MB or over its CPU limits; the 0.3% of ads that exceeded them used 27% of ad data and 28% of ad CPU. Background timer throttling cut CPU up to 5x in Google's tests. Edge sleeping tabs cut memory 32% on average. Opera GX has user-set RAM, CPU and network limits |
| Battery saver modes are binary | **Partly correct** | Chrome Energy Saver is on/off; Chrome Memory Saver has three levels; Edge Efficiency mode has two |
| Browsers give sites "the raw channel" to notifications | **Wrong** | Chrome's quiet permission UI (2020), on-device ML warnings on suspicious notifications on Android (May 2025), auto-revocation for high-volume, low-engagement sites (announced October 2025; under 1% of notifications get any interaction), Edge's quiet and adaptive requests, and per-site OS notification channels in Chrome on Android. **What survives:** no browser batches or summarizes web notifications on a schedule |
| Manifest V3 (implied: uBlock Origin gone from Chrome) | **Mostly correct** | MV2 extensions were disabled in Chrome 138 (July 2025). Chrome users get uBlock Origin Lite. A report that the full uBlock Origin was removed from the Web Store on 31 August 2026 is single-source |

**Reasoning faults the fact-check exposes:**
- Arc's failure is attributed to a cause its own source rejects.
- "Shipped" is treated as "solved".
- Low container use is read as users rejecting identity separation. The essay ignores that the add-on is hidden and that Total Cookie Protection now does most of the job by default.
- Data Saver is chosen as the cost precedent while shipped budgets are ignored.
- The maturity table states usage claims as settled when no source supports them.

---

## 2. Fact-check: Essay 2 (accessibility remediation)

| Claim | Verdict | What the record says |
|---|---|---|
| WebAIM Million 2026: 95.9% (from 94.8%), 56.1 errors per page (+10.1% from 51), first rise in six years | **Correct** | Matches webaim.org/projects/million. These are **automatically detected** failures only |
| WebAIM blog quotes on why barriers persist | **Correct, verbatim** | "Tolerating Inaccessibility", Christopher Phillips, 30 April 2026 |
| Overlay timing: screen readers build the tree at T+200–500 ms, overlays run at T+800–2000 ms | **Unsupported; likely fabricated; mechanism wrong** | No source found. Browsers update the accessibility tree incrementally and notify assistive technology of changes, so overlay edits do reach the screen reader once applied. The real harms are a brief window before the fix, refreshes mid-reading, and the overlay's own reader conflicting with the user's |
| Overlay user "studies", three quotes, "24 respondents" | **Real, misattributed** | All from one paper: Makati, Tigwell and Shinohara, ASSETS 2024 (24 of 47 respondents). The first quote is the authors' summary, not participants' words |
| FTC "fined" accessiBe $1M because claims "were not supported by competent and reliable evidence" | **Partly correct** | The $1M is consumer redress, not a fine; proposed order 3 January 2025, finalized 22 April 2025. The quoted phrase is not in any FTC text: the complaint says "false, misleading, or unsubstantiated", and "competent and reliable evidence" is the standard imposed on accessiBe's *future* claims |
| "Over 800 businesses using AI overlays were sued in 2023–2024" | **Wrong** | UsableNet's figures (via secondary sources; its site did not load) are about 933 in 2023 and 1,023 in 2024, about 1,950 in total, and they cover overlays and widgets generally, not "AI overlays" |
| 2026 assistive-agents paper: 24 improvements vs 20 regressions; verified loop makes regression "impossible by construction"; 57/57 and 126/126 | **Real, overstated** | arXiv 2608.24913 (July 2026); the numbers match. The essay drops "on the automated checks" from the regression claim. The system injects CSS only, its checker covers a small rule set, and the 57/57 and 126/126 results came from stand-in generators, not real model output |
| "Users preferred the repaired version in 26 out of 38 subjects", 17% readability gain | **Real quote, wrong context** | From MFix (ICSE 2018), which repairs **mobile-friendliness** problems. The 38 "subjects" are web pages rated by paid crowd workers. It has nothing to do with screen readers or accessibility |
| Legato exists; developer's quote about sighted testing; free rules, paid AI | **Correct** | Chromium-based macOS browser by Dmytro Liesniak (Qiksy); AppleVis forum quote verbatim (about August 2026). Its "screen reader toggle" is a built-in voice that can be swapped for the user's own screen reader. It also makes Essay 1's "research only" maturity label for corrected trees out of date |
| AOM was designed for authors to modify the tree; W3C quotes on "parallel structures" and "maintain consistency" | **Correct quotes, second misapplied** | Both are verbatim in WAI-ARIA 1.2, but "maintain consistency" concerns how ARIA attributes look across host languages (HTML, SVG), not accessibility-tree versus DOM consistency |
| "Courts have consistently held" runtime fixes do not substitute for source compliance | **Overstated** | The rulings found (Quezada v. US Wings, Angeles v. Grace, both 2021, seen via summaries) hold only that the defendant had not shown the site was fixed. The rest are settlements, and one reported ruling (Erkan) went the other way |
| WebAIM "Semantic Automation" quotes; JAWS and VoiceOver guess labels; VoiceOver added it in iOS 5 and Lion | **Correct, slightly reframed** | Jared Smith, 4 November 2011, verbatim. The iOS 5/Lion point is his own observation, and the "complaints" are his testing concerns, not user complaints |
| LLM HTML restructuring and automatic heading generation research | **Correct, small studies** | arXiv 2502.18701; ACM TACCESS 2024 (DOI 10.1145/3649223) |
| CVE-2026-95301: compromised renderer reads cross-origin accessibility data | **Correct, missing conditions** | Fixed in Chrome 154.0.8037.57 (29 September 2026); Chromium bug 540265100. It requires a compromised renderer **and** a running `chrome.automation` consumer such as ChromeVox, so it mainly affects ChromeOS |
| Correction can be "invisible to the site, like Forced Colors" | **Partly wrong** | `forced-colors` is visible to sites through CSS and `matchMedia` by design. Repairs confined to the accessibility tree can be invisible, because no shipped web API reads the computed tree. Anything that changes focus, layout or the DOM cannot be invisible |
| Browsers do no general repair | **Partly wrong** | Chrome (2019) and Edge (2022) already insert generated image descriptions for unlabeled images, with an on/off control. No browser lets the user switch between the whole author tree and a whole repaired tree, which is the essay's genuinely new idea |

---

## 3. Design review

### 3.1 Where the argument fails

- **Arc points the other way for HGBA.** If maintaining a fork of a free engine had been the problem, a new engine composed from eight upstreams (V8, Stylo, Taffy, Parley, html5ever, WebRender, wgpu, Wasmtime) would face a larger version of it. In fact the problem was product, which says nothing about engine reuse in either direction. The lesson HGBA can take is narrower: a different frame around the page is not a purchase reason, and features must earn adoption by default behaviour, not discovery.
- **The falsifiers cannot be checked as written.** "Widely used", "worked at scale" and "if any of them is the wrong answer" have no thresholds.
- **Opt-in undercuts the essay's own evidence.** The essays dismiss containers, Reader Mode and Forced Colors because opt-in features go unused, then propose an opt-in feature without addressing the same adoption problem. The partial answer is that accessibility users are a population that actively configures assistive settings (4% of Windows devices use High Contrast), so opt-in adoption in the target group may be much higher than in the general population. That is a hypothesis the experiment should measure.
- **"None of this requires AI" is only half true.** The strongest evidence of benefit (LLM restructuring, the assistive-agents paper, Legato's paid tier) is model-based. Rule-based repair has the weakest evidence of benefit.
- **The 95.9% headline overstates what rules can fix.** Rule-based repair mainly reaches empty link and button names, missing document language and some label associations. Low contrast (83.9% of pages) is already handled by forced colors and contrast enforcement, which already ship and are visible to sites.

### 3.2 The oracle is moderate, not strong

"Accept a change only if violations strictly decrease" judges the change by the same checker that defines success. Under HGBA-3.1's oracle-strength classes, it is **moderate** for "adds no detectable violations" and offers **nothing** on whether the repair is correct or helps the user.
- **It can be gamed.** Naming an icon-only trash button "Submit" removes an empty-name violation and makes the page worse, because a wrong label is worse than a missing one.
- **It is not the dual-implementation differential.** That compares two implementations of one function. The accessibility mapping specs (HTML-AAM, accname) define the *author* tree, so they can check the author tree but cannot say what a correction should be.

HGBA can supply a strong oracle for a different property: **non-interference**. Under DET-1 replay, DOM serialization, computed style, fragment, display-list and JS-observable event hashes must be bit-identical with repairs on and off. That proves the site cannot observe tree-only repairs, which is also the privacy requirement, because a detectable repair reveals that the user has a disability. Benefit still needs human-labelled correctness and a user study.

### 3.3 Conflict with HGBA-3.1 B10, and the resolution

HGBA-3.1 B10 says deterministic repairs (forced colors and contrast, default document language, heuristic names for empty links and buttons) are "not AI" and "never gated by the B10 switch". The essays say every repair must be opt-in, reversible and auditable. **Both are partly right.** "Not gated by the AI switch" does not have to mean "on by default". The pain-points report's claim that rule-based repairs carry "no risk of a wrong label" is wrong for heuristic names. Writing a default `lang` into the page would change layout and break P2, and detecting a page's language from its text usually needs a small classifier.

The design reviewer proposes five repair classes, none gated by the AI switch:

| Class | Examples | Default | Site-visible? |
|---|---|---|---|
| R0 | Forced colors and contrast, following the OS preference | Follows the OS setting | Yes, by spec |
| R1 | Language hint for assistive technology only (not written into the DOM) | On | No |
| R2 | Heuristic names for unlabeled controls, label association | **Off**; one accessibility-settings toggle; per-site override | No |
| R3 | Structural repairs: focus order, keyboard traps, headings | User-invoked per site only | Yes (focus events) |
| R4 | Model-based repairs such as alt text | HGBA-3.1's separate accessibility opt-in | No |

R2 to R4 live in a sparse **repair overlay** keyed by stable node ID, not a second full tree. Every entry records its class, rule and source nodes. A repair inspector lists every repair, and one toggle reverts them all. Two security rules follow from provenance: **a repair may use only text from the same origin and frame as the repaired node**, and **agents receive the author tree by default**, because heuristic names turn attacker-controlled file names and URLs into trusted-looking labels.

One feasibility gap needs a new open question (Q-10). Platform accessibility APIs have no standard property for "inferred by the user agent", so B10's requirement that screen readers *announce* a generated label has no mechanism today. The options are a suffix on the name or description (verbose, but it works with every screen reader) or work with screen-reader vendors.

### 3.4 The other features

- **Notification brokering** is not a fallback for accessibility remediation, because it serves different users and has a different oracle. After the fact-check, what is genuinely open is narrow: scheduled batching or digests, per-site mute and snooze, and one-click revocation. Gesture-gated prompts are already in the Stage 1 design. **Stage 2, never delaying by default.**
- **Cost budget** shrinks to a **per-site resource meter** (CPU, network, energy, memory) shown on request, plus the interventions that already have evidence: heavy-ad style limits and background throttling. User-set budget enforcement is out of scope, because the "right default" problem has no evidence base.
- **"Browser as the user's agent"** is a stance HGBA already holds in practice: built-in content blocking, partitioned cookies, gesture-gated permissions, autoplay blocking. But HGBA-3.1's vision text implies no changes to what content can observe. The reviewer proposes a **P2 scope note**: P2 governs unmodified pages under default settings. Interventions that content can observe are allowed only in listed classes: behaviour the platform leaves to the user agent, user-invoked overrides, and privacy protections with a recorded P1 analysis. They are called **"user-agent interventions"**, to avoid confusion with AI agents.

### 3.5 What the essays get right that HGBA missed

- The UI chrome is not a differentiator, so HGBA should not spend roadmap on it.
- Repairs need an audit trail and one-toggle reversal. HGBA-3.1 B10 has neither.
- The accessibility tree is security-sensitive (CVE-2026-95301 is real). Repairs must stay inside the renderer and respect origins.
- No browser offers a whole repaired tree that the user can switch to and from.

---

## 4. Recommended changes

**Stage 1 design document (draft 0.2 → 0.3)**
- **Section 6:** replace the single accessibility row with R0 and R1 in Stage 1, and the R2 overlay plus inspector as an opt-in in Stage 2.
- **Section 6 notifications row:** add digests, mute and snooze in Stage 2.
- **Section 6 resource meter:** add a per-site resource meter.
- **Section 6 out-of-scope line:** UI chrome as differentiation, budget enforcement, and R3 repairs other than user-invoked ones.
- **Section 5.8:** define the semantic plane as an author tree plus a repair overlay.
- **Section 8.3:** name an accessibility owner and budget for a contracted accessibility researcher to run the user study.

**HGBA-3.1 (as 3.1.1, together with the Section 12 amendments in the design document)**
- B10 and P6: replace the "deterministic repairs" bullet with the R0–R4 classes.
- B8: the author tree plus repair overlay, the origin-containment rule, and agents receiving the author tree by default.
- P2: the scope note on user-agent interventions.
- Add Q-10 (how to disclose an inferred name).
- New Stage B targets:
  - **A11Y-1:** non-interference. Bit-identical outputs with repairs on and off, over 1,000 of 1,000 replays. Strong oracle; ship blocker.
  - **A11Y-1b:** origin containment. 100%; ship blocker.
  - **A11Y-2a:** no new violations under a checker **not** used during development, on at least 99% of pages.
  - **A11Y-2b:** label accuracy against human-labelled pages. Per rule, wrong plus harmful at most 10%† and harmful at most 1%†. A rule over the harmful bound is deleted.
  - **A11Y-3:** a user study (below).
  - **A11Y-4:** overlay cost. At most 1%† of renderer memory, and no script-thread task delayed by more than 1 ms at p99.

**The experiment** (proposed by the design reviewer; thresholds †):
- **Pages:** 500 recorded pages, with 40% sealed as a holdout.
- **Labels:** each repair rated by two independent raters.
- **Participants and design:** at least 24 daily screen-reader users (at least 6 each on NVDA, JAWS and VoiceOver), paid, with ethics review. A blind, counterbalanced three-arm study: repairs off; repairs on with disclosure; repairs on without disclosure.
- **Pass, which allows R2 to be proposed for default-on with disclosure:** task success up by at least 10 points, wrong activations up by at most 2 points, and at least 60% preference.
- **Otherwise:** R2 stays opt-in, or is withdrawn.

**Pain-points report:** correct the claim that rule-based repairs carry no risk of a wrong label.
