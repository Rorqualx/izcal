# Izcal extensions: fact-check (phase 8)

Compiled 20 October 2026 against `reports/Izcal extensions.md` (revision 0.1, 20 October 2026) and its research notes (`frame.md` plus the seven strands), by a checker who wrote neither. Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8. Each claim was checked first against the note row or section the spec cites (a wrong row citation counts as a finding), then against primary sources where they could be fetched. Sources read this session: developer.chrome.com (service-worker lifecycle, offscreen API, MV2 deprecation timeline, web-accessible resources, manifest reference, declare-permissions, install-extensions), MDN (Content Security Policy for extensions, content scripts, `browser_specific_settings`), Extension Workshop (extensions-and-the-add-on-id, after the MDN redirect), the Mozilla Hacks 2019 add-on-outage post, and the NDSS Carnus paper page and PDF snippet. Cross-references were checked in the real specs on disk: HGBA-2 (I-1, W-1, X-3), HGBA-3 (`Future software browser engine.md`: B1, B2, B9, D12, PRIV-2), the Stage 1 design (D-S1/D-S2/D-S3/D-S8/D-S12, Section 6, 8.3, 9, 13.2), the JavaScript engine spec (JS-D10/D15/D16/H-2/D17, JS-M1), the DOM spec (DM-D16, the 30 s service-worker row), content blocking (CB-D15, CB-D21, 6.3, 6.9–6.11, 6.14–6.16, 9.3, G81/G82/G84), networking (6.5.2, 6.18, 6.19, 6.22, G36), graphics (GR-D28), editing (ED-D10/D11/D21), DevTools (DT-D10, DT-4, DT-6, the `devtools.*` refusal), resource footprint (FP-D9, FP-D16, 8.2), accessibility (AX-32), WebCrypto (the G-rule ceiling at G168/G169), and the Phase 3 amendment ledger (G-19, G-20, C-5/C-6/C-17, Q29).

Budget notes: about 18 page fetches and 4 web searches were used. The USENIX papers (Jagpal 2015, Carlini 2012), the Sjösten CODASPY 2017 paper and Ars Technica's DataSpii piece were not re-fetched; their numbers were taken from the notes' High-confidence rows, which record the primary text as read. The Google Chrome help-centre article for the three site-access modes did not surface (a support-community thread and third-party walkthroughs do); `chrome://` pages are not fetchable. No engine source code was opened, and none is linked from the spec or the notes' cited rows.

Verdicts: **Correct**; **Partly** (the substance holds but a number, scope, wording, citation or attribution is off); **Wrong**; **Unverifiable** (no source found that states it); **Not re-checked** (the note row is Medium/Low or unfetchable and no independent check was made this session). "Must-fix" marks a finding that a decision (EX-D1 to EX-D16) or a target depends on, a misattribution, or a wrong evidence citation. Every other fix is recommended.

## Summary

105 claims checked (a claim cited at several places is counted once, by its worst verdict):

| Verdict | Count |
|---|---|
| Correct | 84 |
| Partly | 12 |
| Wrong | 5 |
| Unverifiable | 0 |
| Not re-checked | 4 |
| **Total** | **105** |

Links: the spec contains **no external URLs at all** — every design-carrying primary fact is carried by the research notes rather than linked inline, which the skill's "inline links to primary sources" recommendation asks against (FC-R1). Its 8 relative links (7 research-note links plus the bare `izcal-capabilities.md` reference in Section 3) all resolve on disk. No `#fragment` links. All EX-D/EX-Target/EX-M/Q-EX/G-rule identifiers are used consistently with their definitions. What holds: the incident numbers (DataSpii ~4M, Great Suspender ~2M, 500+ extensions/1.7M installs, Firefox 57's two-thirds catalogue loss, the 2019 expiry and its sub-9-hour hotfix, 96.5%/25 minutes/50M+, 96 of 108), the privacy numbers (>50% WAR probing; 2 extensions → 77.5% of ~84,000 users; 4 → 94.5%; interaction-triggered probing), the Chrome lifecycle numbers (30 s idle, 5-minute cap, 30 s alarm floor, 15 offscreen reasons with mandatory justification), the MV2 deprecation dates end to end, and the CB 6.15 tier text, NW 6.19 interception wording, ED-D21/D10/D11, DT-D10/DT-4/DT-6, GR-D28, JS-D10/D15/D16/D17, DM-D16, Stage 1 D-S2/D-S3/D-S12/13.2, the G-rule range G170–G179, `hg-crypto`'s Ed25519 and `hg-compress`'s deflate are all confirmed against the real specs and, where fetched, the primaries. The G170–G179 range is free (the ledger's registry leaves G88 onward to successive specs; the highest rule in use elsewhere is WebCrypto's G169). The staffing arithmetic sums exactly (9–14†).

### Must-fix list

- **FC-M1. The identity-derivation precedent is misattributed to Firefox** (6.4, EX-D4). The spec derives the extension ID "from its publisher signing key and package name at first install (the documented Firefox derivation pattern)". No such Firefox pattern is documented: MDN's `browser_specific_settings` page says AMO "assigns a GUID to the extension when it is signed" for MV2 and requires an explicit ID for MV3, and Extension Workshop's add-on-ID page describes only AMO-assigned and temporary random IDs — no key derivation anywhere. The documented key-derived ID is **Chrome's**: the manifest `key` field "specifies your extension's ID", and the SHA-256-of-public-key → first-16-bytes → a–p derivation is the documented CRX/`key` mechanism. The design (signature-derived identity) is unaffected; the parenthetical justification is wrong. Correct statement: "…derived from its publisher signing key, the documented pattern behind Chrome's `key` field and CRX identity; Firefox instead documents AMO-assigned IDs".
- **FC-M2. Two evidence citations for EX-D5 point at the wrong rows of the permissions note** (6.5). (a) "comprehension research shows warning fatigue above roughly a 50% ask rate … (permissions_prompts 2.x, High)": section 2 of that note is **Firefox's model**; the warning-fatigue research is section 4 (rows 21–24, Felt et al. 2011). (b) "the invisible-enterprise-grant gap the standards debate left open (permissions_prompts 8.3, High)": the note numbers its rows continuously 1–44, so "8.3" names nothing; the invisible-grant material is section 6 row 34 (WECG #260: grants invisible to `permissions.contains`/`getAll`) and is echoed in section 8's second row (43). Correct citations: "(permissions_prompts 4.x, High)" and "(permissions_prompts 6.34 / 8.2, High)".
- **FC-M3. The ceiling/convergence sentence cites a range that does not exist** (6.5, EX-D5). "…the invariant the platform with the most users has converged on, alongside the opposite-direction convergence of Firefox and Safari … (permissions_prompts 8.1–8.4, High)". The note's section 8 has exactly three rows (42–44), continuously numbered; the convergence-and-ceiling content is row 42 and the intersection rule is section 1 row 7. Cite "(permissions_prompts 8 and 1.7, High)".
- **FC-M4. The decision-log pattern is cited to CB's staffing section** (6.12, EX-D10). "a published decision log — each accept, reject or removal entry with its reason, in the pattern of NW's list governance (CB 9.3)". CB 9.3 is "Staffing and duration". The governance pattern (witnessed transparency log, published evidence, contest path) is CB 6.9. Cite "(CB 6.9)".
- **FC-M5. The proposed CB amendment cites the wrong section for the container route** (9.5, CB row). "tier-1 extension list updates ride the container route per CB 6.10". The container-route rule ("fetched by that container's own network service, through the container's route, with no cookies … at the same daily slot") is in **CB 6.9**; CB 6.10 is the list formats. Section 6.7 of this spec cites it correctly as "(CB 6.9/6.10)"; the amendment text should say CB 6.9, or it would propagate a wrong pointer into the content-blocking spec.
- **FC-M6. Content-script CSP is stated as one documented semantics; MDN documents browser divergence** (6.13, EX-D11). "Content scripts are subject to the **page's** CSP for resource loads they cause, not for their injection — the documented MV3 semantics". MDN's extensions-CSP page: MV3 content scripts "share the default CSP as extensions", and "The extent to which the CSP controls loads from content scripts **varies by browser**. In Firefox … most DOM-based APIs are subjected to the CSP of the web page. In Chrome, many DOM APIs are covered by the extension CSP instead of the web page's CSP." Chrome's own page documents a browser-fixed isolated-world CSP (`script-src 'self' 'wasm-unsafe-eval' …`) and applies the page's CSP only to MAIN-world injection. Izcal may pick one semantics, but the sentence should say it is choosing, not cite "the documented MV3 semantics". Correct statement: "Chrome and Firefox divide these loads differently (MDN); Izcal fixes one semantics — page CSP for DOM-inserted loads, the browser-set extension CSP for the content script's own code — and publishes the divergence."
- **FC-M7. "Uniformity (D-S1)" misstates the accepted D-S1** (4.3; 6.15; 10). The report's P2 says "uniformity (D-S1) is not for sale in the extension store", and 6.15 says "the engine's D-S1 uniformity is not extension-dependent". Stage 1's D-S1 as accepted is "**Hybrid, randomization by default**: seeded per-site, per-session noise …; full uniformity (RFP-style) as a user opt-in" — uniformity is the opt-in, not the model. The intended claim (an extension cannot weaken the anti-fingerprinting posture) stands; write "the engine's anti-fingerprinting posture (D-S1)" or "D-S1's noise and normalization".

### Recommended (not blocking)

- **FC-R1. Inline primary links.** The spec has zero external links. The skill asks for inline primary links where a decision or target depends on a claim. The heaviest dependences: the MV2 dates (6.2, 9.5), the 30 s/5-minute/offscreen numbers (2.2, 3, 6.3), the Carnus numbers (6.14), the Jagpal and Palant numbers (3, 6.12), the native-messaging no-prompt claim (6.11) and the site-access model (6.5) should carry links where they are used, as the graphics, editing and layout specs now do.
- **FC-R2. Tor is overstated as shipping no extensions** (1 "Tor Browser's no-extensions policy"; 6.14 "Tor Browser's answer is to ship no extensions at all"). Tor's documented policy is that installing add-ons is "strongly discouraged"; it ships and supports the bundled NoScript (privacy note row 39). Write "no third-party extensions".
- **FC-R3. The 50% warning-fatigue threshold is an inference** (6.5). The note's Felt rows say frequent warnings desensitize and that warnings only inform when the request is anomalous; the >50% figure is the share of extensions requesting history/all-hosts (the anomaly baseline), not a measured fatigue threshold above a 50% ask rate. Say "warnings inform only where the request is anomalous; history and all-hosts warnings are requested by more than half of extensions and are therefore read by nobody (Felt 2011)".
- **FC-R4. "11 days of releases" and a note conflict on the hotfix latency** (2.1, 5). The primary Mozilla Hacks post confirms "shipping the fix in Normandy at 2:44 AM, or after less than 9 hours" (plus 6–12 h to propagate), matching the report. But the distribution note (6.34) says the Studies hotfix took "~24 h" — the two notes disagree. The report's number is the right one; the note row should be corrected by its owner. "11 days" is defensible (66.0.4 on May 5, back-ports to Firefox 47–65 by May 14), but say "back-ports completed 10–11 days later".
- **FC-R5. Safari's distribution is not App-Store-only** (1, 2.1, 5). A Developer ID + notarization path outside the Mac App Store exists (distribution note 3.19, Medium). "App-Store-distributed" and "App-Store-only model" describe the dominant channel, not the only one; say "App-Store-mediated".
- **FC-R6. Two distribution-note citations point at the wrong sections** (6.12). "(distribution note 4.x, High)" for the CA-incident-playbook/rotation lesson: the 2019-expiry analysis the sentence leans on is distribution 6.34 (4.x is update-channel incidents, which supports it only partly). "(distribution note 6.x, High)" for ownership transfer: distribution 6 is store-review failures; the bought/trojanized evidence is distribution 4.23–26 and privacy 4.22–23. Likewise "(security 5, distribution 6)" in the Section 3 review row: the no-published-decisions evidence is distribution **5** (rows 30–31).
- **FC-R7. "G84's separation" is a loose analogy** (6.12). G84 is "no third-party list is ever an input to Izcal's entries" — Izcal-own versus third-party input, not store tier versus enterprise tier. The enterprise-tier design is sound on its own; either drop the G84 reference or say "the G84 discipline (one trust domain never mixing with another), applied to code".
- **FC-R8. "Section 9.4 lists the two refusals most likely to be reversed on evidence"** (6.1). Section 9.4 lists decisions with reversal triggers; it does not name or rank "the two refusals most likely to be reversed". Either name them here (EX-D1's ring table and EX-D2's MV2 refusal, whose triggers 6.20 and 9.4 state) or point at 6.20.
- **FC-R9. The trojanized-wave row drops the source's hedge** (2.1 "16 bought extensions, 3.2M users"). GitLab Threat Intelligence says the operator "likely bought" the extensions (privacy note 4.23, High for the campaign, hedged for the acquisition channel). Keep "16 extensions, 3.2M users; the operator likely bought them".

### Notes

- **Ring C mixes refusals with Stage-3 staging** (6.1). `tabCapture`, `tts`/`ttsEngine`, `printing*` and `sidebar_action`/`sidePanel` sit in the Ring C bullet annotated "(Stage 3…)", while the Ring C table row defines C as "Izcal does not implement it" and 9.1 lists the same items as staged, not refused. Publish them as a fourth state ("staged") or a Ring C sub-column, or the compatibility table contradicts itself.
- **`activeTab` is declared Ring A in 6.5 but is absent from 6.1's Ring A bullet list.** The list is explicitly "the load-bearing entries", so this is not a contradiction, but the machine-readable ring index must include it.
- **`pageAction` "aliased to `action`" is a divergence from both Chrome and Firefox**, which keep the namespace separate (surface manifest §2a). Publish it as a documented divergence like the storage.sync one.
- **Claims resting on Medium/Low note rows are stated as fact in the spec**: DataSpii's "the store did not act for an extended period" (note Medium), the Web Developer sale mechanism (note Low, sources conflict), the userscript manager's default-on analytics (note Low, community reports). 11.2 flags the fingerprinting numbers but not these three. One sentence in 11.2 would cover them.
- **Report-internal consistency is good**: the EX-D numbering (D1–D16, with D15/D16 living in 6.9/6.10 ahead of D9–D14) is used consistently in 6.x, 9.4 and 5; G170–G179 are each defined once in 6.18 and cited consistently (G171–G175, G178, G179 in 9.5; G174/G175 in 6.12 and the attacker table); EX-M0's Stage 1 ask matches 9.5's Stage 1 row; EX-4's 2 MiB† appears identically in 9.5's footprint amendment.
- **The stale "Wasmtime" texts the report proposes to amend all exist verbatim**: Stage 1 D-S3 ("extensions arrive in Stage 2 via Wasmtime"), Stage 1 Section 6's extensions row and Section 9's "Wasmtime (Stage 2)" row, NW 6.22 ("extensions (Stage 2, Wasmtime)"), CB 6.15 ("Stage 2's Wasmtime extensions"), HGBA-2 W-1 and X-3 (ledger C-17/G-5). The amendment list in 9.5 is accurate about what it is editing.
- **Q29's 48–60 months† and C-6's counting rule are correctly used** (9.2, 9.5; ledger rows C-6/C-7 and G-3).

## 1. Claims checked

### 1.1 History and incidents (Sections 1, 2.1, 2.2, 5)

| # | Claim (spec section) | Verdict | Evidence | Fix |
|---|---|---|---|---|
| 1 | DataSpii exfiltrated ~4M people's histories (1, 2.1, 5) | Correct | HP 4.3 (Ars; ~4M) | — |
| 2 | The Great Suspender: sold, ~2M users, remote kill (1, 2.1, 5) | Correct | HP 4.1–4.2 | — |
| 3 | 500+ extensions, 1.7M installs, surfaced by an outside researcher (1, 2.1, 3, 5) | Correct | HP 4.4 (Kaya/Duo; Sophos) | — |
| 4 | Firefox 57 killed XUL/XPCOM; e10s; about two-thirds of the catalogue lost (2.1, 5) | Correct | HP 1.1, 1.3 (note says two-thirds to three-quarters; the spec's "about two-thirds" is the conservative end) | — |
| 5 | The 2015 rebuild announcement drew hundreds of angry comments (5) | Correct | HP 1.1 (471 comments) | — |
| 6 | 2019 signing-intermediate expiry disabled the ecosystem overnight; hotfix in under 9 hours (2.1, 5) | Correct | HP 5.2–5.3; [Mozilla Hacks](https://hacks.mozilla.org/2019/05/technical-details-on-the-recent-firefox-add-on-outage/): "shipping the fix in Normandy at 2:44 AM, or after less than 9 hours" | FC-R4 (the distribution note's "~24 h" contradicts; fix the note) |
| 7 | "11 days of releases" (5) | Partly | DS 6.34: 66.0.4 May 5, back-ports to Firefox 47–65 by May 14 (≈10–11 days) | FC-R4 |
| 8 | MV2 deprecation dates: store cutoffs Jan/Jun 2022; warnings 2024-06-03; stable disabling 2024-10-09; default-off 2025-03-31; everywhere 2025-07-24 (Chrome 138); enterprise policy removed with Chrome 139; store purge 2026-08-31; ≈4.5 years (2.1, 5; HP 3.2–3.5) | Correct | [MV2 deprecation timeline](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline), fetched: every date matches | — |
| 9 | uBlock Origin left Chrome; Firefox kept MV2 and its own MV3 shape (2.1, 5) | Correct | HP 3.7–3.8 | — |
| 10 | 197 remote-code Firefox bans (2.1) | Correct | HP 4.5 | — |
| 11 | Honey: policy-compliant affiliate expropriation; litigation 2024– (2.1, 5) | Correct | HP 4.7; PR 26–27 | — |
| 12 | 2025 trojanized wave: 16 extensions, 3.2M users (2.1) | Correct | PR 4.22 (GitLab Threat Intelligence) | FC-R9 ("likely bought") |
| 13 | Safari: App-Store-distributed, thinnest catalogue; WebKit moved the API engine-level in 2025 (1, 2.1, 5) | Partly | HP 6.1–6.3 (Safari 18.4, 2025, correct); the Developer ID/notarization path also exists (DS 3.19) | FC-R5 |
| 14 | ~75% drop in uninstall-support requests after the 2014–15 install restrictions (2.1) | Correct | HP 2.3 | — |
| 15 | Chrome extensions 2009–10; Web Store 2010 (2.1) | Correct | HP 2.1–2.2 | — |
| 16 | Tor Browser's no-extensions policy (1); "ships no extensions at all" (6.14) | Partly | PR 39: third-party add-ons "strongly discouraged"; bundled NoScript is shipped and supported | FC-R2 |
| 17 | One userscript manager shipped analytics on by default (5) | Not re-checked | PR 44 (Low: community reports, 2017) | See Notes |
| 18 | The extension-less cost lands on DNS proxies and userscripts (2.2, 5) | Correct | PR 43–45 (design inference graded High in the note) | — |

### 1.2 Surface, permissions, privacy (Sections 3, 6.2, 6.5, 6.8, 6.13, 6.14)

| # | Claim (spec section) | Verdict | Evidence | Fix |
|---|---|---|---|---|
| 19 | Chrome's install-time all-or-nothing model made "read everything, for ever" cheapest (1, 5) | Correct | PP 1–2 (Felt 2011) | — |
| 20 | Every vendor has since moved to runtime, per-origin grants (2.2, 4.4, 6.5) | Correct | PP 42 | — |
| 21 | Chrome's three site-access modes (on click / on specific sites / on all sites) (6.5) | Correct | PP 6; Chrome's extension Details "Site access" UI documents exactly these three options (available since Chrome 70) | — |
| 22 | Effective grant = declared ∩ granted (the intersection invariant) (6.5) | Correct | PP 7 (Chromium permissions.md) | — |
| 23 | One platform silently re-approves a removed permission (granted-vs-active split) (6.5) | Correct | PP 38 | — |
| 24 | Firefox: MV3 hosts optional by default; update-added host permissions not shown (documented in its own tracker) (6.5) | Correct | PP 10–11, 30 (bug 1893232, quoted in MDN) | — |
| 25 | Safari: per-site Ask/Allow/Deny; private-window and profile defaults; no enterprise force-install (4.4, 6.5) | Correct | PP 15–19 | — |
| 26 | No vendor ships extension-grant auto-expiry (6.5) | Correct | PP 41 (High, for the negative) | — |
| 27 | The 2024 hygiene-pass pattern for web permissions (6.5) | Correct | PP 36 (Chrome Safety Check, Sept 2024; web site permissions only — the spec's wording is careful) | — |
| 28 | "warning fatigue above roughly a 50% ask rate" (6.5) | Partly | PP 21–22: desensitization from frequent warnings; >50% is the anomaly baseline, not a fatigue threshold | FC-R3 |
| 29 | 2 detected extensions → 77.5% unique among ~84,000 users; 4 → 94.5%; sets reveal religion, health, politics (3, 6.14) | Correct | PR 5–6; Carnus paper text: "77.5% chance of uniquely identifying the user within a set of almost 84 thousand users" ([NDSS 2020](https://www.ndss-symposium.org/ndss-paper/carnus-exploring-the-privacy-threats-of-browser-extension-fingerprinting)); 94.5% consistent with the paper, not verbatim re-verified | — |
| 30 | WAR probing identifies >50% of top extensions (3, 6.14) | Correct | PR 1 (Sjösten et al., CODASPY 2017) | — |
| 31 | Interaction-triggered probing reaches extensions static probing misses (3, 6.14) | Correct | PR 7 (36% not detectable by prior art) | — |
| 32 | Chrome's only shipped countermeasure is per-session resource URLs, covering the static vector (6.14) | Correct | PR 10 + implications | — |
| 33 | `use_dynamic_url`: per-session, regenerated on restart/reload (6.14) | Correct | [Web accessible resources](https://developer.chrome.com/docs/extensions/reference/manifest/web-accessible-resources), fetched | — |
| 34 | Content-script CSP semantics (6.13) | Partly | MDN: "varies by browser"; Chrome covers many DOM-API loads under the extension CSP; Chrome documents a browser-fixed isolated-world CSP and page CSP for MAIN world only | FC-M6 |
| 35 | MV3 CSP floor non-relaxable; sandboxed pages the API-less relaxation (6.13) | Correct | SEC 6.1–6.3 | — |
| 36 | ~85 documented MV3 namespaces; ~16 security/privacy-relevant (3) | Correct | SM §2 (85 counted, High; ★~15 plus `webAuthenticationProxy`) | — |
| 37 | Promises and `browser.*` are the cross-browser norm (3) | Correct | SM 6.1–6.3 | — |
| 38 | userScripts: developer-mode gated; USER_SCRIPT world with no extension APIs beyond messaging (6.8) | Correct | SM 4.10 | — |
| 39 | Match-pattern grammar, `run_at` values, `world: "MAIN"` page-authority semantics (6.8) | Correct | SM 4.2, 4.5–4.7; 1.5 | — |
| 40 | MV2 capabilities MV3 lacks = blocking interception, persistent background pages with DOM, remote code; the renames (6.2) | Correct | SM 3.1–3.2, 5.x, §1 | — |

### 1.3 Review, distribution, security (Sections 1, 3, 5, 6.11, 6.12, 6.15)

| # | Claim (spec section) | Verdict | Evidence | Fix |
|---|---|---|---|---|
| 41 | 96.5% detection, half within 25 minutes, <100 evasive extensions reached 50M+ users (3, 6.12) | Correct | SEC 5.1 (Jagpal et al., USENIX 2015) | — |
| 42 | 96 of 108 reported extensions still live 19 months later (3, 6.12) | Correct | SEC 5.5 (Palant, Jan 2025; June 2023 report → Jan 2025 = 19 months) | — |
| 43 | No operator publishes decisions, statistics or precedent (3, 6.12) | Correct | DS 5.30, 7.39 | FC-R6 (the "(security 5, distribution 6)" citation should read distribution 5) |
| 44 | Copyfish, Web Developer, the 2017 sprees: every major incident arrived by update or sale (3, 2.1) | Correct | DS 4.23–24 (the Web Developer mechanism conflicts across sources, but both accounts agree on update/sale) | See Notes |
| 45 | Native messaging: one registry key → OS code execution; no connect-time prompt documented on any platform (3, 6.11) | Correct | SEC 3.3–3.5 | — |
| 46 | Host-manifest contents; length-prefixed JSON wire format (6.11) | Correct | SEC 3.1–3.2 | — |
| 47 | The documented CVE class is contract confusion: header rewrite (12395), content scripts in the wrong context (12396), a missing prompt (12397) (3, 6.15) | Correct | SEC 8.3 (MFSA 2018-27) | — |
| 48 | Carlini: 70 vulnerabilities in 40 extensions; CSP does not stop DOM-injection XSS (6.15) | Correct | SEC 6.6, 8.1 | — |
| 49 | Force-install policy as the historical malware channel (1) | Correct | SEC 9.2, PP 35 (note grades Medium/Low; well attested) | See Notes |
| 50 | CA incident playbook / rotation lesson, cited "(distribution note 4.x, High)" (6.12) | Partly | The supporting row is DS 6.34; 4.x covers update-channel incidents only | FC-R6 |
| 51 | Ownership transfer as review event, resting on the 2025 wave, cited "(distribution note 6.x, High)" (6.12) | Partly | Substance correct (PR 4.22–23; DS 4.26); the cited section is store-review failures | FC-R6 |
| 52 | UPD-R5 emergency channel exists (6.12) | Correct | Ledger A-S1-58 (UPD-R1–R5); CB 6.9 uses the same reference | — |

### 1.4 Izcal cross-references, licences, arithmetic (Sections 1–4, 6, 7, 9, 10)

| # | Claim (spec section) | Verdict | Evidence | Fix |
|---|---|---|---|---|
| 53 | Ledger G-20: "Extensions (Stage 2 surface, JS-D10 component host)", pointers CB-D15, CB 6.15, NW 6.19, Stage 1 D-S3 (1, 9.3) | Correct | Ledger row 107, verbatim | — |
| 54 | Q29 places Stage 1 exit at 48–60 months† (9.2) | Correct | Ledger C-7/G-3 rows | — |
| 55 | C-17 settles the host; stale "Wasmtime" text in HGBA-2 W-1/X-3 (G-5), Stage 1 D-S3, CB 6.15, NW 6.19/6.22 (7, 9.5) | Correct | Ledger C-17; all stale texts verified verbatim in the named specs | — |
| 56 | W-1 says "no extension code in renderer processes"; the proposed structural restatement (6.3, 9.5) | Correct | HGBA-2 W-1 row, verbatim | — |
| 57 | D12: an agent is a principal, never a privileged extension (1, 4.3, 6.10) | Correct | H3 445 | — |
| 58 | B1 recorded inputs; B2 typed capability IPC; B9 agent principal (4.2, 6.0, 6.19) | Correct | H3 141 ("typed capability channels, simulation transport (B2)"), B1 sections, ED 699/723 | — |
| 59 | I-1's isolation key is (site, container) (2.2, 6.4) | Correct | HGBA-2 I-1 extension (a), rev 2.1.2 | — |
| 60 | JS-D10 component host with WASI embedding, jitless for this embedder; JS-D15 recorded decision points; JS-D16/H-2 untrusted-read; JS-D17 one cage per process, per-agent blocks; JS-M1's debugger scope (4.2, 6.0, 6.3, 6.15, 6.17, 6.19) | Correct | JS 13, 42, 169, 213, 247 — all as the spec describes | — |
| 61 | DM-D16: isolated worlds, per-world side tables, Xray-style views; the DOM spec sets a 30 s† idle default for page service workers (4.2, 6.0, 6.3, 6.8) | Correct | DM 321–327; DM 533 ("Service worker idle termination … 30 s†"; "Chrome documents 30 s only for extensions") | — |
| 62 | CB-D15's three tiers, zero read, no blocking interception, DNR-as-data, re-prompt on growth; CB-D21 pure-function matching and recorded inputs; CB 6.3's sealed pipeline (4.2, 6.7, 6.16) | Correct | CB 262, 490–493, 498–508, verbatim | — |
| 63 | Extension list fetches ride the container's route, no cookies, the daily slot (6.7, citing CB 6.9/6.10) | Correct | CB 6.9, verbatim | FC-M5 (the 9.5 amendment cites 6.10 alone) |
| 64 | CB 6.10's accepted list formats: ABP-family, DNR JSON, Safari content-blocker JSON, hosts (6.1) | Correct | CB 6.10 table | — |
| 65 | Abuse reports through the OHTTP channel (6.12, "the CB 6.11 pattern") | Correct | CB 6.11 ("Reports go through the OHTTP relay") | — |
| 66 | CB 6.14's redaction pattern (encrypted in the recording, redacted in reports) (6.16) | Correct | CB 6.14, verbatim | — |
| 67 | CB 6.16's table applies to extension-supplied lists (6.7) | Correct | CB 6.16 row 1 names "extension-supplied" lists | — |
| 68 | G81 (no code-bearing rules; data-only updates skip re-review), G82 (pure function), G84 (no third-party list as input to Izcal's entries) (6.7, 6.12, 6.18) | Correct | CB 324, 279, 418, 548 | FC-R7 (G84's reuse for enterprise tiers) |
| 69 | NW 6.19: interception scoped to origin set and container, typed by intent (observe, mock, block, modify), badged, audited, bodies/cookies/credentials as sensitive classes (1, 6.7) | Correct | NW 6.19, verbatim | — |
| 70 | NW 6.18's "who received what" ledger; NW 6.22's stale "Wasmtime" text; NW 6.5.2's root-program analogy (3, 6.12, 6.17, 6.19) | Correct | NW 583–591, 656, 170/336 | — |
| 71 | GR-D28: prompts in an OS layer the shared compositor cannot draw over, broker-clamped input (4.2, 6.5, 6.9) | Correct | GR rev 0.4 and 205 | — |
| 72 | ED-D10 (paste-press reads), ED-D11 (provenance and the planted-command warning), ED-D21 (typed edit intents producing trusted `beforeinput`/`input`, never minting activation; already names extensions) (6.8, 6.10, 9.5) | Correct | ED 358, 360, 431, 699 | — |
| 73 | DevTools: no WebExtension `devtools.*` API; IIP is a typed, audited, scoped-principal protocol; DT-D10 Option B; DT-4's 12-months-plus-two-releases removal discipline; DT-6's published pass rates (3, 4.4, 6.1, 6.17) | Correct | DT 147–148, 175, 277, 354–356; JS 42 | — |
| 74 | FP-D9 (no default-on telemetry; opt-in aggregates under a privacy review), FP-D16 (regenerable-cache class), FP 8.2, DC-4, CC/CT measures (6.12, 6.14, 8, 9.5) | Correct | FP 1050, 551/583; CB ("DC-4 reference device"); JS 13 | — |
| 75 | Stage 1 13.2: a permissions row (store, auto-revocation) and an updates row (TUF) exist for 6.5/6.12/6.19 to consume | Correct | S1 523–545 | — |
| 76 | "Uniformity (D-S1)" (4.3, 6.15, 10) | Partly | D-S1 as accepted is hybrid randomization-by-default; uniformity is the user opt-in | FC-M7 |
| 77 | D-S2: the shell is sandboxed web content; user theming CSS-only; scripted customization via W-1 (4.4, 6.9) | Correct | S1 461, verbatim | — |
| 78 | D-S8 covers funding/governance (Q-EX-6) | Correct | S1 502 | — |
| 79 | D-S12: recorded black-box observations in `compat/observations/` (7) | Correct | S1 501 | — |
| 80 | Storage spec (G-19) "in progress" (6.6, 6.19, 9.5) | Correct | Ledger row 106; `research_notes/Izcal storage/` exists with no report yet | — |
| 81 | G170–G179 are "the range assigned to this spec" (6.18) | Correct | Ledger C-5 final registry ("G88 onward the next spec"); the highest G-rule in use elsewhere is WebCrypto's G168/G169; no collision | — |
| 82 | Signatures from `hg-crypto`'s pinned Ed25519 (G36); ZIP/deflate via `hg-compress` "already shipped for HTTP"; no new dependencies (6.4, 7) | Correct | NW 9/307/650 (Ed25519 written on fiat-crypto; G36); NW 279 (deflate/gzip encoders in `hg-compress`) | — |
| 83 | PRIV-2, SEC-28, AX-32 exist and mean what the spec says (6.15, 8, 9.5) | Correct | H3 432; NW 595; AX 627 | — |
| 84 | The 30 s idle number is "the number the DOM spec already sets for page service workers and Chrome documents for extension workers" (6.3) | Correct | DM 533; [Chrome lifecycle](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle): "After 30 seconds of inactivity" | — |
| 85 | 5-minute single-event cap; fetch-timeout kill; offscreen: 15 reasons, mandatory justification, 30 s AUDIO_PLAYBACK close, one document, `runtime` only (2.2, 3, 6.3, 6.14's "15-reason workaround") | Correct | SM 3.3–3.7; the lifecycle and [offscreen](https://developer.chrome.com/docs/extensions/reference/api/offscreen) pages, fetched: 15 reasons, justification required, 30 s audio rule, one document, runtime-only | — |
| 86 | Alarms: 30 s minimum period (6.3) | Correct | SM 3.4; lifecycle page: Chrome 120, "minimum period of 30s" | — |
| 87 | Staffing rows sum to 9–14† (9.3) | Correct | Recomputed: 2+2+1+1+1+1+1 = 9; 3+3+2+2+1+1.5+1.5 = 14 | — |
| 88 | "Section 9.4 lists the two refusals most likely to be reversed on evidence" (6.1) | Partly | 9.4 lists decisions and reversal triggers; it neither names nor ranks two refusals (the candidates are EX-D1/EX-D2, with triggers in 6.20 and 9.4) | FC-R8 |
| 89 | EX-D1–D16, EX-1–10, EX-M0–M5, Q-EX-1–8, G170–G179 all defined and used consistently (throughout) | Correct | Checked pairwise | — |
| 90 | `izcal-capabilities.md` (3) resolves | Correct | `.claude/skills/first-principles-spec/references/izcal-capabilities.md` | — |
| 91 | `externally_connectable` enforced broker-side; `runtime.id` extension-contexts-only; per-(site, session) WAR randomization as default (6.2, 6.14) — design on documented bases | Correct | SM 4.9 (Chrome 106's `chrome.runtime` undefined), PR 10 | — |
| 92 | Chrome's 2021–2025 host-permission reform; "install expresses intent; running expresses grant" (4.4, 5) | Correct | PP 5, 44 (the phrase is Chromium's own design sentence) | — |
| 93 | Ring C refusals map onto the notes' dangerous-namespace list (6.1) | Correct | SM §2 summary (the ~15★ list) | Notes (Ring C / Stage-3 mixing) |
| 94 | The Firefox-signing lesson: two signatures, offline roots, rotation with overlap, an emergency channel independent of opt-in telemetry (5, 6.12) — design consistent with HP 5.3's commitments | Correct | HP 5.3 | — |
| 95 | Extension ID "derived from its publisher signing key … (the documented Firefox derivation pattern)" (6.4) | **Wrong** | MDN `browser_specific_settings` and Extension Workshop: AMO assigns a GUID (MV2) or requires an explicit ID (MV3); no key derivation. The documented key-derived ID is Chrome's (`key` field / CRX public-key derivation) | FC-M1 |
| 96 | "permissions_prompts 2.x" for the warning-fatigue research (6.5) | **Wrong** | The note's section 2 is Firefox's model; the research is section 4 (rows 21–24) | FC-M2 |
| 97 | "permissions_prompts 8.3" for invisible enterprise grants (6.5) | **Wrong** | The note's rows are numbered continuously 1–44; "8.3" names nothing. Content: row 34 (section 6) and row 43 (section 8's second row) | FC-M2 |
| 98 | "permissions_prompts 8.1–8.4" for the ceiling/convergence sentence (6.5) | Partly | Section 8 has rows 42–44 only; the content is row 42 plus row 7 (section 1) | FC-M3 |
| 99 | "(CB 9.3)" for NW's list governance (6.12) | **Wrong** | CB 9.3 is "Staffing and duration"; the governance/transparency-log pattern is CB 6.9 | FC-M4 |
| 100 | 9.5's CB amendment: "ride the container route per CB 6.10" | **Wrong** | The container-route rule is CB 6.9; CB 6.10 is formats | FC-M5 |
| 101 | "(distribution note 6.x, High)" for ownership transfer (6.12) | Partly | Content is DS 4.23–26 and PR 4.22–23 | FC-R6 |
| 102 | "Honey … 16 bought extensions" — the acquisition hedge (2.1) | Partly | GitLab: "likely bought" (PR 4.23) | FC-R9 |
| 103 | DataSpii: "the store did not act for an extended period" (5) | Not re-checked | HP 4.3 (Medium) | See Notes |
| 104 | Web Developer: sold, ~1M users, spyware update (3) | Not re-checked | DS 4.24 (Low; sources conflict on mechanism; update/sale agreed) | See Notes |
| 105 | Honey litigation status (2.1) | Not re-checked | HP open questions (coverage conflicts) | See Notes |

## 2. Link report

**Spec.** 0 external URLs. 7 relative links into `research_notes/Izcal extensions/` (history_postmortems, distribution_signing, privacy, surface_manifest, security_robustness, permissions_prompts, gaps_opportunities — each cited twice, in the Section 2 and Section 3 lead-ins), all resolving on disk with the `%20` encoding. The bare `izcal-capabilities.md` in Section 3 resolves to `.claude/skills/first-principles-spec/references/izcal-capabilities.md`. No `#fragment` links. Nothing is dead because nothing external is linked; the cost is FC-R1.

**Note citations.** The report cites notes by section, not row ("history 4", "privacy 1.x", "security 5", "gaps 2", "distribution 4", "permissions_prompts 6.x/7.x/2.x/8.x"). Section-level citations all name sections that exist and, except where listed above (FC-M2, FC-M3, FC-R6), contain the claimed content. The permissions note is the one strand where the section-style notation breaks down, because its rows are numbered continuously across sections — that is what produced FC-M2 and FC-M3. Wrong-content citations found: those listed in the must-fix and recommended lists; no others.

## 3. M-1 part 3 report

**No link in the spec points to any project's source code, and no value appears to come from one.** The spec contains no external URLs at all; its evidence is carried by the notes. Values the spec states that only a vendor document could state — the 30 s idle and 5-minute cap, the 15 offscreen reasons, the 30 s alarm floor, the 85-namespace count, the MV2 dates, `use_dynamic_url`'s per-session behaviour — were all confirmed against vendor prose, not source. The notes disclose their two uses of prose documents inside source trees (Chromium's `extensions/docs/permissions.md`, a design doc, in the permissions note's method line; `firefox-source-docs.mozilla.org` in the privacy and gaps notes), which the skill's prose-document allowance covers; neither feeds a value the spec states as measured. Section 7's licence claims check out: `hg-crypto` writes Ed25519 (NW-D23, G36), `hg-compress` ships the deflate and gzip encoders for HTTP's codings (NW 6.x), and the spec introduces no new dependency or data set (the 6.18 crate table is all `hg-ext-*` crates). The one licence statement worth tightening: 7 says rule formats are "formats, not works", citing CB 6.10's practice — that is exactly what CB 6.10 does, so the statement is anchored.

## Verdict

The spec's load-bearing record is sound. Every incident number, deprecation date, Chrome lifecycle figure and privacy statistic I could reach reproduces its note row and, where fetched, the primary source, and the report is noticeably more careful than most of its siblings about marking first-principles thresholds with † and confessing them in 11.2. The cross-reference work is also largely right: the CB-D15 tiers, NW 6.19's interception wording, ED-D21, DT-4/DT-6, GR-D28, the JS-D10/D15/D16/D17 chain, Stage 1's D-S2/D-S3/D-S12 and 13.2 rows, the free G170–G179 range, and the stale "Wasmtime" texts all say what the report claims they say. What is wrong is concentrated in one kind of sentence: attributions and evidence citations. Five are simply wrong — the Firefox identity-derivation precedent (it is Chrome's documented pattern), two permissions-note row citations, the CB 9.3 governance pointer, and a proposed amendment that would install a wrong section citation into the content-blocking spec — and one mischaracterizes another spec's accepted decision (D-S1 is hybrid randomization, not uniformity). One further sentence presents browser-divergent CSP behaviour as a single documented semantics. None of the seven must-fix items changes a design decision; all seven change the stated justification for one, and two would propagate into other documents if 9.5 were applied as written. Fix the citations, add inline primary links, and decide how Ring C's Stage-3 residents are labelled, and this spec is as checkable as it claims to be.
