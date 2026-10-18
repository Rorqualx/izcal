# Design critique: the two feature essays as input to HGBA

*Independent design review, 14 October 2026. Inputs: `essays.md` (two essays, author unknown), HGBA-3.1 direction spec (`reports/Future software browser engine.md`), HGBA Stage 1 architecture design draft 0.1 (`reports/HGBA Stage 1 architecture design.md`), and the pain-points report (`reports/Browser user pain points 2026.md`). No web research was done. Factual claims in the essays (WebAIM quotes, the 2026 "verified repair" paper, Legato, CVE-2026-95301, overlay timings, lawsuit counts) are being fact-checked elsewhere and are treated here as unverified. Thresholds marked † are set by this review from first principles.*

---

## 0. Verdict in one paragraph

The essays reach a defensible conclusion by a weak route. Accessibility remediation is a reasonable candidate for HGBA's one "unilateral" user-facing feature, and the essays' design constraints (a separate tree, opt-in, one-toggle reversal, a per-correction audit trail, no site detection) are better than what HGBA-3.1 currently says. But the case for it rests on three errors: an oracle that is not strong by HGBA's own definitions, an analogy to B8's dual-trait differential that does not hold, and evidence that mostly comes from model-based repair, which the essays then claim not to need. The most useful thing the essays do is expose a gap in HGBA: B10 and the Stage 1 design treat "deterministic" repairs as safe, unmarked and implicitly default-on, and the pain-points report claims they "carry no risk of a wrong label". That claim is false for heuristic names, and the documents should be corrected. HGBA can do something no competitor can here, but not the thing the essays propose. Its strong oracles (bit-identical replay hashes) can prove that a repair **does not interfere** with the page. They cannot prove that a repair **helps**. Only a user study can show that, and the essays' experiment does not specify one with thresholds.

---

## 1. Logical soundness

### 1.1 Essay 1

**The Arc argument points the wrong way for HGBA.** Essay 1 says Arc's failure "may have been engine cost, not design", and that this "argues for HGBA-3.0's engine reuse". Arc was a Chromium fork: it reused the most complete engine available, with no engine of its own to build. If maintaining a fork of free Chromium was enough to sink a well-funded company, then a new engine assembling eight independently versioned upstreams (V8, Stylo, Taffy, Parley, html5ever, WebRender, wgpu, Wasmtime) faces a strictly larger version of the same cost. HGBA-3.1 already names this standing integration cost as the thing that "decides whether the project survives" (Section 8). The Arc case is a warning to HGBA, not support for it.

**The falsifiers cannot be tested as written.**

| Essay falsifier | Problem |
|---|---|
| "If Arc failed from Chromium maintenance cost, UI space may be more open" | There is no observable that separates the two causes, which the essay itself admits ("the public record does not separate the two"). The consequent ("may be more open") is also hedged, so the falsifier makes no prediction. |
| "If containers were widely used, identity thesis is wrong" | "Widely" has no threshold. The inference is also invalid: low adoption of a *hidden* feature measures discoverability, not value. The essay applies this logic both ways. Containers are ignored, so identity is a dead end. Reader Mode and Forced Colors are ignored, yet remediation is promising. |
| "If banner rejection worked at scale, consent is solved" | "Worked" and "at scale" are undefined. "Solved" conflates browser-side rejection with site-side compliance, which the essay elsewhere calls a coordination problem outside the browser. |
| Essay 2: "If it fails, put the budget into notification brokering" | "Fails" has no metric or threshold (Section 8 of this review supplies them). |

**The three groups are a false trichotomy.** The essay sorts features into "shipped, ignored", "blocked outside the browser" and "unilateral action still works". A feature can be all three: Forced Colors is shipped, rarely used by the general population, and unilateral. The sorting also measures adoption against the wrong denominator. Forced Colors and Reader Mode are "rarely used" across all users, but the question is whether the target population uses them, and the essays give no figure for that. The same objection applies to their proposal. An **opt-in** remediation feature will, by the essays' own logic, join Group 1. The essays need to say why their opt-in will be found when Forced Colors is not, or accept that it serves a self-selecting AT-user population. That is fine, but then the general-population "rarely used" argument is irrelevant.

**"None requires AI" does not match the evidence cited.** Essay 2's evidence of benefit is mostly about model-based repair: LLM HTML restructuring, the "24 improvements against 20 regressions" result, and the verified-repair instrument (whose candidate generator is, from the quote, an assistive agent). The rules-based repairs the essays fall back on (button names, headings, focus order, dialog traps) are supported only by the WebAIM "semantic automation" argument and by Legato's existence, neither of which measures an outcome. So either the feature needs a model, in which case B10's separate accessibility opt-in governs it, or it is rules-only, in which case the essays' evidence of benefit mostly does not apply.

**The headline statistic overstates what is reachable.** "95.9% of homepages fail" motivates the feature, but the failure mix (pain-points report, X3) matters more than the headline. Low contrast (83.9%) is a rendering problem that Forced Colors already addresses and that a semantic-tree repair cannot fix. Missing alt text (53.1%) needs a model. Missing form labels (51.0%) are already handled by screen-reader heuristics (JAWS and VoiceOver, per the essay's own WebAIM quote). What remains for deterministic semantic repair is mainly empty links (46.3%), empty buttons (30.6%) and missing document language (13.5%). That is real, but much smaller than the headline suggests.

### 1.2 Essay 2

**The analogy to B8's dual-trait differential is false.** Essay 1 says the oracle is "strong (a11y spec + screen reader), same shape as HGBA-3.0 B8 dual-trait differential". The dual-trait differential (HGBA-3.1 Section 6) compares two implementations of the **same function** and requires identical output. Remediation has no second implementation of "the correct label". The accessibility specs (HTML-AAM, accname) define how to compute a name *from markup*. A corrected tree departs from that computation by design. The spec is therefore an oracle for the **author** tree and says nothing about whether a correction is right. In fact HGBA's existing B8 oracle (differential against Chromium's tree plus WPT accessibility-mapping tests) will *flag* every repair as a discrepancy. The essays never deal with this, but the design must: the differential has to run on the author tree, with repairs applied afterwards as a separate layer.

**The verified-repair result is circular as summarized.** "Accepts a change only if violations strictly decrease" makes regression "impossible by construction" only where regression is defined as *checker-detected violations*. The evaluation then measures the same quantity: 57 of 57 seeded violations detected, 126 of 126 harmful candidates rejected. The seeded violations and the adversarial candidates are presumably of checker-detectable kinds, because otherwise the instrument could not reject them. This shows the instrument enforces its own invariant. It does not show that users are better off. Section 2 develops this point.

**The overlay timing argument proves less than it claims.** The essay explains overlay failure by load timing (the screen reader reads the original for 500–2,500 ms before overlay JS runs; these figures are unsourced) and by conflicts with screen readers. A browser-level repair fixes timing and the conflict between two scripting layers. It does not fix the deeper complaint in the same studies, which is wrong or conflicting guesses. A browser repair that guesses also conflicts with the screen reader's own heuristics (the essay's open question Q-3, and WebAIM's quoted complaints: "no way to know if label text is actually associated", "no option to disable"). The FTC accessiBe order concerned unsupported *claims*, not the mechanism. "Overlays fail, so browser-level repair succeeds" is not a valid inference. What follows is narrower: browser-level repair removes two of several failure causes.

**The maturity label is inconsistent with the evidence.** The essay labels a "corrected a11y tree as primary AT interface" as research. Its own evidence says user agents have shipped "semantic automation" since iOS 5 and OS X Lion, and that Legato ships rules-based repair (if it exists as described). Under P7, rules-based repair by a user agent is **shipped**. What is new is verification and auditability. That actually strengthens feasibility and weakens the "nobody does this" framing.

**The CVE example cuts against the design.** If CVE-2026-95301 is real (the number looks unusual and needs checking), it shows that accessibility-tree data is a cross-origin leak surface. A second, corrected tree containing synthesized text is more data on the same surface. The essay cites the CVE without drawing this conclusion (see Section 4).

---

## 2. Is "violation checker; strictly fewer violations" a strong oracle?

No. Measured against HGBA-3.1's own oracle-strength classes (Section 6 table; EQ-1's strong, moderate and none), it is **moderate at best for non-regression, and absent for correctness**.

| Property | What the checker oracle establishes | HGBA strength class |
|---|---|---|
| The repair does not add *checker-detectable* violations | Yes, by construction | Moderate: a reference tool, not ground truth, and only for detectable classes |
| The repair removes violations | Yes, for detectable classes | Moderate |
| The repaired name, role or structure is *correct* | **No** | None |
| Users perform better | **No** | None |
| The repair does not break the page | **No**: the checker reads the tree, not behavior | None |

**Coverage.** Automated checkers detect only a minority of WCAG success criteria. The commonly cited range is roughly a third to a half, depending on whether criteria or issue instances are counted (fact-check the exact figure). Everything they cannot see, such as label *meaning*, reading order, cognitive load and focus logic, is invisible to the gate.

**Goodhart risk.** A gate on "strictly fewer violations" rewards the cheapest way to clear each rule:

- An empty link given the name "link", or a URL slug such as `click?id=8f3a…`, clears "empty link" and helps nobody.
- An icon button named from its filename, `icon-23.svg` or `btn_primary`, clears "empty button" and misleads.
- A trash-can button named "Submit" (taken from adjacent text) clears the rule and is **actively harmful**: the user deletes something they meant to save. The checker counts this as a strict improvement.
- A contrast fix that recolors text clears the contrast rule and may make an author-meaningful color distinction (error red, disabled grey) disappear.

**A wrong label is worse than a missing one.** With a missing name, the screen reader says "button" (or reads the URL), and an experienced user knows to be cautious. A wrong name is trusted. Any oracle for remediation must therefore **weight harm asymmetrically**. A count of violations treats a removed violation and an invented error as unrelated events, when in fact the second is often how the first was removed.

**What a stronger oracle looks like, using HGBA's strengths.** HGBA has something no overlay vendor has: bit-identical replay hashes (DET-1) over DOM serialization, the fragment tree, the display list and the semantic tree. These give a **strong** oracle for a property the essays care about but never test: **non-interference**. With repairs on, an AT-only repair must leave the DOM serialization, computed styles, fragment hash, display-list hash and JS-observable event log bit-identical to repairs off, for the same recording and seed. That is an exact full-state check, of the same class as projection equality. Benefit then needs separate, honestly labelled oracles: gold labels from human annotators (moderate) and a user study (the only real oracle for benefit, run per release cycle, not per commit). Section 8 sets this out.

---

## 3. Conflict with HGBA-3.1 B10: opt-in or default-on?

### 3.1 What the documents actually say

- HGBA-3.1 P6: accessibility "is never gated by the switch".
- B10 boundary rule: deterministic repairs (forced colors and contrast enforcement, default document language, heuristic names for empty links and buttons) "need no model, so B10 never gates them (P6)".
- Stage 1 design 5.8: these repairs "are part of the default build".
- Pain-points report: deterministic repairs "carry no risk of a wrong label".

"Never gated by the AI switch" is not the same as "on by default", and "in the default build" is not the same as "enabled at runtime". So the documents are **ambiguous rather than contradictory**. A reader would still assume default-on, and nothing in them requires marking, auditing or a toggle for deterministic repairs. The pain-points sentence is simply wrong: URL- and filename-derived names are heuristics, and heuristics produce wrong labels.

Two further problems:

- **"Default document language" may not be deterministic.** If the language is *detected from content*, the usual tools (CLD-class language identifiers) are small neural networks. Under B10's own rule, such a model must either be listed in the ML inventory or not be used. Deterministic sources exist: HTTP `Content-Language`, the `lang` of an ancestor frame, the user's UI locale. These are not the same as detection, so the bullet needs to say which is meant.
- **Injecting `lang` into the DOM is a P2 violation.** `lang` affects `:lang()` matching, font fallback, hyphenation and line breaking, so injecting it changes layout and is script-visible. The repair must be a hint to platform accessibility APIs only, never a DOM or style input.

### 3.2 Who is right, per repair class

Neither document is right across the board. The right default depends on two properties: whether the repair **invents content** (and so can be wrong), and whether it is **observable by the page** (and so engages P2 and fingerprinting).

| Class | Examples | Invents content? | Site-observable? | Default | Marked and audited? |
|---|---|---|---|---|---|
| **R0. Platform preference honoring** | Forced colors, `prefers-contrast`, minimum font size, zoom | No: applies a user or OS preference by spec | **Yes, by spec** (`forced-colors`, `prefers-contrast` media queries) | **Follows the OS setting**, as in every browser. Not "remediation" at all | No: it is a standard UA behavior |
| **R1. Spec-conformant AT metadata** | Document language hint taken from `Content-Language`, frame `lang` or UI locale, exposed only to platform APIs | No: copies existing deterministic signals | No, if never written to the DOM | **On** (AT-only) | Logged; not announced |
| **R2. Heuristic semantic inference** | Names for empty links and buttons from URL, filename, `title`, nearby text; label association; role inference | **Yes** | No, if confined to the semantic plane | **Off by default**; a single accessibility-settings toggle, per-site override. May be promoted to default-on only after A11Y-3 (Section 8) passes, by decision-log record | **Yes**: every repaired node flagged; enumerable; announced (see 3.4) |
| **R3. Structural or behavioral repair** | Fixing keyboard traps, focus order, dialog traps, inserting headings or landmarks, restyling author colors outside forced-colors mode | Yes | **Yes**: changes focus events, tab order, DOM or rendering | **User-invoked per site only**, like the pain-points report's rule for scroll overrides. Not in Stage 1 | Yes |
| **R4. Model-based repair** | Alt text, LLM restructuring | Yes | Depends; must be R2-style (AT-only) | B10's separate accessibility opt-in (unchanged) | Yes (already in B10) |

So the essay is right for R2–R4: anything that invents content must be opt-in or disclosed, reversible and auditable. B10 is right that none of these classes belongs behind the *AI* switch, and it is right for R0 and R1. Where B10 is wrong is in lumping "heuristic names for empty links and buttons" (R2) together with forced colors (R0) as unproblematic.

**Why R2 should not simply be default-on.** Screen readers already apply their own heuristics to unlabeled controls, including reading URLs and associating adjacent text. A default-on browser heuristic stacks on top of these without the user's knowledge, which is exactly the WebAIM complaint the essay quotes ("no option to disable"). Experienced screen-reader users have calibrated expectations of their own AT's behavior, and silently changing what "button" means on 30% of pages breaks that calibration. The essay's opt-in is the conservative choice until the user study says otherwise.

### 3.3 Proposed resolution

1. Rename B10's bullet from "Deterministic accessibility repairs are not AI" to "Rule-based accessibility repairs are not AI". Being rule-based does not make a repair correct.
2. Split it according to the R0–R4 table above. Keep "never gated by the AI switch" for all classes. Define the default per class.
3. Require every R2–R4 repair to live in an **overlay** (Section 4.3), to be enumerable, and to be reversible with one toggle.
4. Correct the pain-points report's "carry no risk of a wrong label".

### 3.4 Feasibility gap in B10's "screen readers announce it"

B10 says machine-generated labels are "marked as machine-generated in the accessibility tree, so screen readers announce it". The platform accessibility APIs (UIA, IAccessible2, AX, AT-SPI) have no standard property that means "this name was inferred by the user agent", and screen readers do not announce custom attributes. The only portable ways to disclose are to change the text (a name suffix, or the accessible description, for example "possibly: Search") or to work with screen-reader vendors on a new property. Both have costs: suffixes add verbosity on every encounter. This is an open question and should be written down as one (proposed Q-10 in Section 7.2), not stated as a property.

---

## 4. "Invisible to the site", security and cost

### 4.1 Can corrections be invisible?

**For R1, R2 and R4 (semantic-plane only): yes, and invisibility is a P1 requirement, not a nicety.** The web platform has no API for reading the computed accessibility tree. AOM's computed-accessibility phase did not ship, and ARIA reflection reflects attributes, not computed names. If the repair never writes to the DOM, style or the data plane, script cannot see it. This matters for privacy. An AT-related setting that a site *could* detect would be a disability signal, which is sensitive personal data, and a fingerprinting bit drawn from a small population. HGBA's B10 already uses this reasoning for WebMCP ("inert, not missing").

There are residual channels:

- **Interaction outcomes.** A user who can now find the button will click it. That is indistinguishable from any other user clicking it, so it is acceptable.
- **Timing.** Repair work runs on the renderer and could in principle shift task timing. Under B1 it can be scheduled off the script thread's critical path, or computed lazily only when AT queries the tree. This needs a measurement, not an assumption.
- **AT-driven activation.** Accessibility actions synthesize events. Their shape is the same with or without repair, but this should be verified.

**For R0: no, and it should not be.** `forced-colors` and `prefers-contrast` are site-visible by design, so that sites can adapt. Hiding them would break the sites that do adapt, which violates P2. They are also already common to all browsers, so they add no new fingerprinting bit beyond what the OS setting already exposes. Anti-fingerprinting (D-S1 uniformity) should not try to hide them, because hiding an accessibility preference trades usability for anonymity on behalf of the users with the most to lose. This should be recorded as a decision rather than left implicit.

**For R3: no.** Changing focus order or trapping behavior changes `focus` and `blur` events, `document.activeElement` and tab-sequence behavior. These are script-observable, and sites may react to them in ways that break. That is why R3 must be user-invoked per site.

**HGBA can test invisibility, and no one else can.** The non-interference oracle in Section 2 is exactly the "site cannot detect" property, stated in a form that can be checked: under DET-1 replay, every JS-observable output is bit-identical with R1, R2 and R4 on and off. This should become a target (A11Y-1).

### 4.2 Security

- **Repair logic in the renderer is the right place, with a caveat.** Running repair in the site's sandboxed renderer means it processes attacker-controlled input inside the attacker's trust zone, which is correct. A compromised renderer can already lie in its own accessibility tree, so repair adds no new authority. The browser-process side (stitching per-frame trees, feeding AccessKit) is an IPC parser of renderer-supplied data and falls under VER-1 Tier 1.
- **Cross-origin repair inputs must be forbidden.** Heuristics such as "name this button from adjacent text" or "associate this label" must never draw text across a frame boundary. Under site isolation, a cross-origin child frame is in another process anyway. But same-process, cross-origin, same-site frames, and any future aggregation step, could leak or combine content across origins: frame B's text becoming frame A's button name. B8 already carries provenance, so a property test can enforce this: every repaired name's source nodes share the repaired node's origin and frame. This belongs in the oracle table.
- **Repairs are a prompt-injection amplifier for agents.** Heuristic names from URLs and filenames promote attacker-controlled strings into the "name" field, which agents trust more than raw text. If the AI switch is on and the agent reads the semantic tree, a filename like `ignore-previous-instructions-and-….svg` becomes a button's name. Rule: **agent observations receive the author tree by default; repaired nodes, if exposed to agents at all, carry a `repaired` provenance tag**. This fits B8's provenance model and B10's separation of the accessibility setting from the AI switch.
- **Privacy of URL-derived names.** Reading a link's URL tokens aloud can disclose session identifiers or email addresses to bystanders. Heuristics should strip query strings and fragments.
- **The cited CVE pattern.** If accessibility-tree data can leak cross-origin through a broadcast mechanism, then synthesized names increase what leaks. The mitigation is architectural and already in HGBA's process map: the accessibility tree is assembled per frame and stitched in the browser process, never broadcast between renderers.

### 4.3 Memory and the semantic plane

The essay's "corrected tree computed alongside author tree" should **not** mean a second full tree. Doubling the semantic plane would be a large cost against an INC-3 budget that is already open. INC-3 also does not explicitly count the semantic plane, which is itself a gap. The right design is a **sparse overlay**:

- `RepairOverlay: Map<StableNodeId, RepairRecord>`, where `RepairRecord = { class: R1|R2|R4, field: Name|Role|Lang|…, value, rule_id, source_nodes: SmallVec<StableNodeId>, provenance }`.
- WebAIM's own numbers (56.1 errors over 1,437 elements, about 4%) suggest the overlay touches a few percent of nodes, so its cost scales with the number of repairs, not with page size.
- The overlay is computed **only when AT is active and the user has enabled the class**, consistent with B10's "computed whenever assistive technology needs it".
- Repairs are pure functions of (author semantic node, same-origin local context), so they fit B5's incremental model: a semantic diff invalidates only the overlay entries whose `source_nodes` changed.
- Repairs must be deterministic, because the semantic-tree hash is part of DET-1. No hash-map iteration order and no time-dependent heuristics.
- The "user can enumerate corrections" requirement falls out of the overlay directly: the overlay *is* the audit log.
- B8's existing differential against Chromium and WPT accessibility-mapping tests runs on the **author tree only**, so repairs never appear as conformance failures.

Recommendation: INC-3 (or a sibling target) should state whether the semantic plane and repair overlay count against renderer memory. Proposed: the semantic plane is reported separately, with the overlay under **1%†** of renderer memory on the corpus when enabled.

---

## 5. Notification brokering and cost budget against Section 6

### 5.1 Notification brokering

Section 6 already contains the part of notifications with the **strongest evidence in the whole pain-points review**: prompts gated on user activation, quiet UI, and auto-revocation. The essays' "brokering" (batching, prioritizing, per-site revocation, mobile-style channels) is a *post-grant* delivery feature, and it is a different thing:

- **Evidence.** The precedent is mobile OSes (Android channels, iOS summaries), where apps *declare* channels and priority. The Notifications API has no channel or priority concept, so the browser would have to infer urgency, which is a heuristic with the same wrong-guess problem as R2. Batching a 2FA code, a calendar alert or a chat message is a real harm.
- **Overlap with the OS.** On desktop, macOS Focus and Windows notification settings already batch and silence by app. Browser brokering duplicates them at site granularity, which is the one thing the OS cannot do.
- **Push semantics.** Push API subscriptions with `userVisibleOnly` expect a notification to be shown for each push. Delaying is permitted; dropping is not. A digest still has to show something.
- **"Backup" framing.** Notification brokering is not an alternative to accessibility remediation. It serves different users, sits in a different layer (shell only) and costs far less. The essay's "if it fails, do notifications instead" is a budgeting non-sequitur. Both can be decided on their own merits.

**Concrete change:** add a Stage 2 shell row, "Notification delivery controls": a per-site mute and snooze from the notification itself, an opt-in per-site digest, honoring the OS focus mode, and never delaying by default. Success is measured by post-grant revocation and complaint rates, not by shipping the feature.

### 5.2 Cost as a budget

Section 6 has tab memory management (discard and freeze). The essay's per-page CPU, network and battery *budget* has two halves:

- **Visibility (a per-site resource meter).** Low risk, shell only, shipped in other forms (task managers, memory hover cards). B1's injectable clock and per-site processes make attribution comparatively clean. Worth a Stage 2 row.
- **Enforcement (throttling a foreground page that exceeds its budget).** This changes timer and rAF cadence and network scheduling, which is script-observable behavior beyond the throttling classes browsers already share (background timers, freezing). Under P2 it is a user-invoked per-site intervention, never a default. The essay's own "right-default problem" and the discontinued Data Saver are evidence against making it default.

**Concrete change:** add "Per-site resource meter" (Stage 2, shell, visibility only). Record "budget enforcement" as a non-goal for Stage 1–2, with a revisit trigger.

---

## 6. "Browser as the user's agent" versus P2

**HGBA already holds this stance in practice, but not in its stated principles.** Section 6 blocks content, partitions third-party cookies, blocks audible autoplay, gates notification prompts, reduces fingerprinting surface and keeps AI off. Every one of these is the browser acting for the user against the site's preference. P1 outranks P2, so the precedence order allows it.

**The inconsistency is in HGBA-3.1 Section 1.** The vision says HGBA does not claim "any change to what web content can observe". That sentence is about engine *internals*, but read literally it contradicts Section 6. Content blocking, cookie partitioning and anti-fingerprinting all change what content observes. The essay's stance exposes this.

**The essay's stance is too broad to adopt as written.** "The browser is the user's agent" would justify overriding scrolljacking, login walls and CAPTCHAs, and the essay concedes that "sites detect and block overrides". PRIV-1 already shows that HGBA has accepted the cost of interventions only with measurement. The workable principle is narrower, and it should be written into P2:

> **P2 scope note.** P2 governs the engine's behavior on unmodified pages under default settings. User-agent interventions that change what content observes are permitted only in three classes: (a) behaviors the platform leaves to UA discretion (permission UI, autoplay policy, storage partitioning, user stylesheets, forced colors); (b) P1 interventions recorded in the decision log with a compatibility measurement (content blocking, anti-fingerprinting; cf. PRIV-1); (c) user-invoked, per-site overrides that are off by default. Interventions confined to embedder-facing outputs that content cannot observe (the semantic plane, including accessibility repair overlays) are outside P2, provided the non-interference oracle (A11Y-1) passes.

**A terminology warning.** "User's agent" collides with HGBA's AI "agent principal" (P1, B9). The documents should say "user-agent interventions" to keep the two apart.

---

## 7. What the essays get right that HGBA misses

1. **Rule-based repairs can be wrong.** HGBA treats "deterministic" as safe. The essays, and the WebAIM complaints they quote, show that heuristic inference needs disclosure and an off switch. This is a real error in the pain-points report and an omission in B10.
2. **A separate tree, not in-place mutation.** HGBA does not say where repairs live. The overlay design (Section 4.3) follows from the essays' requirement and protects B8's conformance oracle.
3. **Auditability as a user feature.** "The user can enumerate corrections" has no counterpart in HGBA. A11y repairs are the one place where the browser speaks *for* the site, so the user should be able to ask what it changed.
4. **Interaction with AT's own heuristics and with site widgets and overlays** (the essays' Q-3). HGBA does not consider double repair at all.
5. **An outcome experiment before claims.** This is consistent with P7 and with the pain-points conclusion that HGBA can "publish measured effects for features competitors ship on faith". But HGBA has no target for accessibility outcomes. Every B8 target is about agents (AG-1, AG-2, AG-5) or conformance.
6. **Chrome rearrangement is not where differentiation lies.** HGBA's Section 6 already implicitly agrees. It contains no tab or chrome-layout features, which is correct, and should say so explicitly so that scope does not creep.

---

## 8. Proposed experiment: accessibility remediation under HGBA's oracle discipline

**Question.** Do R1 and R2 rule-based repairs, delivered through the overlay, improve task outcomes for experienced screen-reader users without introducing harm or interfering with pages?

**Corpus (pinned and recorded under B1, so every arm sees byte-identical pages):**

| Stratum | Size† | Purpose |
|---|---|---|
| Random sample from a top-1M list, stratified by WebAIM-style error density | 300 pages | Representativeness |
| Task-critical sites: government, banking, health, shopping checkout, travel booking | 100 pages | Harm in high-stakes flows |
| Sites with their own ARIA widget libraries, or with overlay scripts installed | 50 pages | Conflict (essays' Q-3) |
| Adversarial and synthetic: misleading filenames, URL-only links, cross-origin and same-site frames, injection strings in icon names, seeded violations | 50 pages | Security and Goodhart probes |

The corpus is split into a **development set (60%)** and a **sealed holdout (40%)** that repair authors never see. All reported numbers come from the holdout.

**Gold labels.** For every node the repairs touch on the holdout, two independent annotators assign the correct accessible name and role. At least one annotator is an experienced screen-reader user. Adjudication is by a third. Inter-rater agreement must reach Cohen's κ ≥ **0.7†** before the labels are used.

**Metrics and pass/fail thresholds:**

| ID | Metric | Oracle class | Pass† | If missed |
|---|---|---|---|---|
| A11Y-1 | **Non-interference.** DOM serialization, computed styles, fragment hash, display-list hash and JS-observable event log are bit-identical with R1/R2 on and off, under DET-1 replay | **Strong** (exact full-state) | 1,000 of 1,000 replays on 100% of corpus pages | Ship blocker. The offending rule is not AT-only and moves to R3 |
| A11Y-1b | **Origin containment.** Every repaired name's source nodes share the repaired node's origin and frame | Moderate (property test) | 100% | Ship blocker |
| A11Y-2a | **Checker non-regression.** Violations introduced, measured by a checker **not** used during development (two independent engines) | Moderate | 0 introduced on ≥ 99% of pages; removals reported, not gated | Fix the rule. A held-out checker failure counts double |
| A11Y-2b | **Label accuracy against gold.** Each repair scored correct, acceptable, unhelpful, wrong or **harmful** (misleading about the action or destination) | Moderate (human reference) | Per rule: wrong + harmful ≤ **10%**; harmful ≤ **1%**, with the 95% CI upper bound ≤ 2% | **That rule is deleted.** Rules are judged individually so a good rule cannot mask a bad one (the EQ-1 principle) |
| A11Y-2c | **Coverage.** Share of checker-detectable empty-name nodes repaired as correct or acceptable | Reported | No gate; published | — |
| A11Y-3 | **User outcome** (study below) | The only benefit oracle | See below | R2 stays opt-in, or is withdrawn |
| A11Y-4 | **Overlay cost.** Overlay memory and CPU when enabled | Measured | ≤ 1% renderer memory; no script-thread task delayed by > 1 ms (p99) | Lazy or offloaded computation; otherwise drop the rule |

**User study (A11Y-3):**

- **Participants:** at least **24** daily screen-reader users, at least 6 each on NVDA, JAWS and VoiceOver (plus Orca if recruitment allows), and a mix of expertise. Recruited through disability organizations, paid, under ethics review. A separate cohort of at least 8 low-vision users is needed only if R0 or R3 visual repairs are in scope, which they are not for this experiment. Final sample size is fixed by a pre-registered power analysis; 24 is the floor.
- **Design:** within-subject and counterbalanced, three arms: (A) repairs off; (B) R1 + R2 on, with disclosure; (C) R1 + R2 on, without disclosure (this isolates the cost of announcing "inferred"). Participants are blind to which arm is active. 12 tasks per participant drawn from the holdout corpus: find and activate a named control, complete a form, reach a page section, complete a checkout step.
- **Measures:** task success, time on task, **wrong activations** (activating the wrong control, the primary harm measure), self-reported confidence and trust, and preference. A post-session interview covers conflicts with each participant's own screen-reader heuristics.
- **Pass (pre-registered):** in arm B versus A, task success improves by ≥ **10 percentage points** with a 95% CI excluding zero. The wrong-activation rate does not rise: its 95% CI upper bound for the difference is ≤ **+2 pp**. At least **60%** of participants prefer B to A. If C beats B on success with no rise in wrong activations, disclosure is redesigned rather than dropped.
- **Outcomes:** if every threshold passes, R2 may be proposed for default-on with a decision-log record, and only with disclosure. If success improves but wrong activations rise, R2 stays opt-in and the harmful rules are removed. If there is no improvement, R2 is withdrawn, R1 stays, and the essays' fallback applies, with notification delivery controls judged on their own merits (Section 5.1).

**Claims discipline.** Results are published per release, and repairs are described as "an aid to the user", never as compliance (B10, FTC). A11Y-1 and A11Y-2 rerun on every release. A11Y-3 reruns when a rule is added or changed materially.

---

## 9. Recommended edits

### 9.1 Stage 1 design document, Section 6 (shell features table)

- **Add an "Evidence or metric" column**, so each row names the measurement that would show the feature works. This follows the pain-points conclusion that HGBA's differentiator is publishing effects.
- **Add a row before "Accessibility model opt-in":**

  > | Accessibility repairs, rule-based | Engine (semantic-plane overlay) plus shell (settings, inspector) | 1: R0 (OS preferences) and R1 (AT-only language hint). 2: R2 heuristic names, opt-in | Sparse repair overlay keyed by stable node ID (`hg-semantic-repair`); never writes DOM, style or data plane; computed only when AT is active and the class is enabled; every R2 repair enumerable in a "What the browser changed on this page" inspector and reversible with one toggle; per-site override; agents receive the author tree by default | Accessibility (19, 20) | A11Y-1 to A11Y-4 |

- **Amend the "Accessibility model opt-in" row:** "B10 boundary rule: separate setting, local model, every repair recorded in the same overlay and inspector as rule-based repairs; disclosure mechanism per Q-10."
- **Add a row:** "Notification delivery controls | Shell | 2 | Per-site mute and snooze from the notification; opt-in per-site digest; honors OS focus modes; never delays by default; no inferred urgency | Notification spam | Post-grant revocation and complaint rate versus Stage 1 baseline".
- **Add a row:** "Per-site resource meter | Shell plus browser process | 2 | Attributed CPU, memory and network per site; visibility only, no enforcement | Page weight, battery | Not gated; usage reported only if telemetry is opted in".
- **Add an explicit out-of-scope line** under the table: "Out of scope for Stages 1–2: chrome rearrangement (vertical tabs, spaces, split view) as differentiation; foreground resource-budget enforcement; R3 structural accessibility repairs; automatic overrides of scrolljacking, login walls or CAPTCHAs."
- **Amend 5.8:** replace "Deterministic accessibility repairs (forced colors, default language, heuristic names) are part of the default build (HGBA-3.1 B10)" with: "Accessibility repairs live in `hg-semantic-repair`, a sparse overlay over the author semantic tree. The crate depends only on `hg-tree-traits` and the semantic tree's read interface, and cannot name DOM mutation or data-plane write APIs (new crate-graph rule R5, enforced by `hg-graph-lint`). Forced colors and contrast preferences are style-system behavior (R0) and do not belong to this crate. The B8 Chromium differential and WPT accessibility-mapping tests run on the author tree only."
- **Section 9 backlog:** Stage 1 shell adds "R0/R1 accessibility"; Stage 2 shell adds "R2 opt-in repairs, inspector, notification delivery controls, resource meter".
- **Section 8.3:** the accessibility work and the A11Y-3 user study need named ownership. The shell track's 1–2 engineers† cannot also run a pre-registered study; budget for a contracted accessibility researcher.

### 9.2 HGBA-3.1

**Section 1 (vision).** Replace "or any change to what web content can observe" with: "or any change to what web content can observe **through the engine's internal design**. User-agent interventions that change what content observes (content blocking, partitioning, anti-fingerprinting) are policy decisions governed by the P2 scope note."

**P2.** Add the scope note from Section 6 of this review.

**B8, new paragraph "Repair overlay":**

> The semantic plane has two layers: the **author tree**, computed exactly as HTML-AAM and accname specify and checked by the Chromium differential and WPT, and an optional **repair overlay** keyed by stable node ID. Each overlay entry records its class, rule, source nodes and provenance. The overlay never feeds back into the DOM, style, layout or the data plane, which target A11Y-1 checks bit-for-bit. Its sources must share the repaired node's origin and frame. Agent observations use the author tree. Repaired values reach an agent only with a `repaired` provenance tag, because heuristic names promote attacker-controlled strings such as filenames and URLs.

**B8 oracle table, add rows:**

| Component | Oracle | Strength | Residual gap |
|---|---|---|---|
| Repair overlay: non-interference | Bit-identical DOM, style, fragment, display-list and JS-event outputs with repairs on and off under DET-1 replay (A11Y-1) | Strong | None for observable state; timing measured separately (A11Y-4) |
| Repair overlay: origin containment | Property test over provenance (A11Y-1b) | Moderate | — |
| Repair overlay: correctness | Gold-labelled holdout corpus (A11Y-2b); held-out checker (A11Y-2a) | Moderate | Checkers see a minority of WCAG criteria; gold labels cover only the corpus |
| Repair overlay: benefit | Pre-registered screen-reader user study (A11Y-3) | Weak in cadence (per change, not per commit); the only oracle for benefit | Population and task coverage |

**B8 risk.** Add: "Repairs that reduce checker-detected violations can still harm users; a wrong name is worse than a missing one. No checker-based gate is presented as evidence of benefit."

**B10, replace the "Deterministic accessibility repairs are not AI" bullet with:**

> **Rule-based accessibility repairs are not AI, but not all are default-on.** No accessibility repair is gated by the AI switch (P6). Defaults depend on whether a repair invents content and whether content can observe it. **R0**: forced colors, contrast and other OS accessibility preferences follow the OS setting and are exposed to content exactly as the specs require. **R1**: a document-language hint taken only from deterministic sources (`Content-Language`, an ancestor frame's `lang`, the UI locale) and exposed only to platform accessibility APIs, never written to the DOM, is on. Content-based language *detection* that uses a classifier is an ML-inventory entry, not R1. **R2**: heuristic names and roles for unlabeled controls invent content and can be wrong. They are off by default, live in B8's repair overlay, are enumerable and reversible with one toggle, and become default-on only by a decision-log record after A11Y-3 passes. **R3**: repairs that change focus, structure or rendering are user-invoked per site only. Model-based repairs (R4) follow the rule below.

**B10, model-based repair bullet.** Replace "so screen readers announce it" with "and disclosed to the user by a mechanism decided under Q-10".

**Section 8, new Stage B targets** A11Y-1 to A11Y-4, as specified in Section 8 of this review.

**Section 12, new open question:**

> **Q-10. How can a user agent disclose an inferred accessible name?** Platform accessibility APIs have no standard "inferred by user agent" property, and screen readers do not announce custom attributes. Options: a name or description suffix, which is verbose; or a new property agreed with screen-reader vendors, which is slow. A11Y-3's arm C measures the cost of disclosure.

**INC-3.** State whether the semantic plane and repair overlay count against renderer memory. Proposed: they are reported separately, with the overlay at ≤ 1%† when enabled.

**D-S1 (Stage 1 design) or the decision log.** Record that `forced-colors` and `prefers-contrast` are not normalized by anti-fingerprinting uniformity, because hiding an accessibility preference trades usability for anonymity on behalf of the users who rely on it.

### 9.3 Pain-points report

Correct "Deterministic repairs need no model and carry no risk of a wrong label" to: "Rule-based repairs need no model, but heuristic names can be wrong or misleading; only preference-honoring repairs (forced colors) and AT-only metadata copied from deterministic sources carry no risk of a wrong label."

---

## 10. Open items for the fact-checkers

These claims bear directly on the conclusions above:

- The 2026 "verified repair" paper: what generated the candidates, and whether its evaluation used a different checker from its gate.
- Whether Legato exists and ships rules-based repair as described. If so, the maturity of rule-based UA repair is "shipped".
- CVE-2026-95301: whether it exists, and the mechanism ("accessibility broadcast").
- The overlay timing figures (T+200–500 ms and T+800–2,000 ms).
- The automated-checker coverage figure to cite for WCAG criteria.
