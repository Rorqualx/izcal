# Izcal content blocking

*First-principles specification of Izcal's content blocking and tracking protection, end to end: the threat model, the layers that defend against each threat (partitioning, navigation, script and API, network matching, DNS uncloaking, cosmetic hiding), the request matcher's semantics, Izcal's own lists and how they are produced, governed and delivered, user-added third-party lists, breakage handling, per-site controls, the Stage 2 extension surface, performance and memory, determinism and recording, and "why was this blocked". Date: 19 October 2026. Status: draft 0.3, owner decisions recorded. Research notes: `research_notes/Izcal content blocking/` (`frame.md`, `history_postmortems.md`, `tracking_threats.md`, `matching_engines_performance.md`, `list_production_governance.md`, `compat_breakage_ux.md`, `gaps_opportunities.md`; phase 8: `factcheck.md`, `critique.md`). Relates to: HGBA-2 2.3.2 [M-1 all parts, P1–P6, I-1, I-2][H2]; HGBA-3 [P2 scope note, B1, B9, B10, DET-1, AG-0, PRIV-1, PRIV-2][H3]; Stage 1 design [Section 6 rows "Built-in content blocking", "Containers", "Cookie-banner handling", "Anti-fingerprinting", "Extensions"; D-S1, D-S3, D-S8, D-S10; 13.2][S1]; networking spec 0.3.3 [6.8, 6.13, 6.15–6.19, 6.21, NW-D3, NW-D7, NW-D17, NW-D26, NW-22, NW-36, NW-54, 9.3][NW]; HTML and CSS parsing and style spec 0.3.3 [6.16, ST-D17][ST]; resource-footprint spec 0.3 [6.11, 6.14, FP-28, FP-D7, FP-D16, FP-D18][FP]; JavaScript engine spec 0.3 [6.8–6.10, JS-D15][JS]; graphics [GR-D12][GR], fonts and text [TX-D4][TX], images and media [MD-D14][MD] and layout [LY-D11][LY] (the D-S1 amendments); DevTools spec 0.3 [DT-1, DT-10][DT]; shell UI design [4.3][UI]; Phase 2 reconciliation [P21][RN] and licence audit [2.5, OD-L5][LA].*

*Revision 0.3 (19 October 2026): the owner accepted every recommendation in Section 9.4 (CB-D1 to CB-D24. Under CB-D16, Izcal's funding takes no revenue from a party the lists name in a way that could shape them, so a search-revenue deal with a listed party is ruled out, and the German release gate waits on legal advice), and the reconciliation's OD-5 to OD-8. The amendments of Section 9.5 are still to be applied in the documents they amend.*

*Revision 0.2 (19 October 2026): phase 8 applied. The independent fact-check checked 118 claims (91 correct, 12 partly correct, 5 wrong, 2 unverifiable, 8 not checked) and 27 external links (none dead), and found no link to engine or blocker source code; its 9 must-fix and 14 recommended items are applied. The independent design critique raised 36 findings (7 High, 19 Medium, 10 Low); every one is applied, one in a different form than proposed (Appendix A). What changed: **exceptions are scoped by trust domain** (Izcal's bundle, the user's own rules, each third-party list), so Izcal's own breakage exceptions work and no third-party list can unblock Izcal's entries (C2); **`csp=` rules cannot report or name a host**, and cosmetic rules compile to a selector plus one fixed declaration (C4, C25); **the transparency log is witness-cosigned** and verified by the broker's update client (C5); **the restrict action** needs API-call attribution with creator inheritance from the JS engine and is stated as a mitigation, not a guarantee (C3); **the Ads list admits no entry on resource cost alone** and ships with the first lists (C7, C24); **bounce tracking** keeps NW 6.9's 7-day rule beside the Privacy CG draft's, with a Storage Access exemption and a defined deletion path (C1, FC-M3); stripping and debouncing moved to the partition service, and debouncing admission is strict (C11, C15); **user-added lists are fetched through their container's route** (C16); reports default to scheme and host and follow NW-D17's accepted fallback (C14); fetch independence is narrowed to backgrounds and fonts, with lazy images, container queries and computed style measured as residuals (C13); paint suppression moves to the graphics spec and never follows a blocked link (C26); FP-28 stays per request, with a cold-start case (C8, FC-M8); new targets CB-26 to CB-28, new rule G87, new decision CB-D24; staffing re-derived to about 10–19 people† in all, 5–9.5† added to NW's 5–9† (C21, FC-M4).*

*Revision 0.1 (19 October 2026): first draft, written by the synthesis author from the six research notes and the frame note.*

The networking spec already places a compiled request matcher in every partition service and decides that Izcal builds its own lists (NW-D26). This spec builds on it and answers what the matcher is for. The record says four things. **Blocking URLs is not where privacy is won.** Partitioning removed the shared third-party jar; what is left is fingerprinting, about half of it first-party-served, which EasyList-based blockers cut by only about 5% in a 2025 measurement ([Luo et al., IMC 2025](https://www.sysnet.ucsd.edu/~voelker/pubs/canvas-imc25.pdf)); first-party and server-side tracking on more than 54% of sites, whose scripts EasyList and EasyPrivacy match only 27.5% of the time ([Böttger et al. 2026](https://arxiv.org/abs/2606.16720)); and identifiers carried through navigations on 8–11% of navigation paths ([Randall et al., IMC 2022](https://brave.com/research/files/measuring-uid-smuggling-in-the-wild.pdf)) ([TT 2–4][TT]). **Per-user protection state is a fingerprint.** ITP's on-device classifier and Privacy Badger's local learning were both shown to leak, and WebKit moved to uniform, classifier-independent rules and Privacy Badger to one pre-trained list shipped to everyone ([WebKit 2019](https://webkit.org/blog/9661/preventing-tracking-prevention-tracking/), [EFF 2020](https://www.eff.org/deeplinks/2020/10/privacy-badger-changing-protect-you-better)); scriptless probes identify 84% of a user's filter lists, and 45 probe rules shrink a user's anonymity set to a median of 48 users ([El Hajj Chehade et al., USENIX Security 2025](https://notify.cispa.uni-saarland.de/papers/elhajjchehade2025double.pdf)) ([HP 4, 6, 13][HP], [TT 8.4][TT]). **A list is executable policy.** Every blocker that let lists run code ended up with a trust tier by list provenance, and the one that did not shipped a code-execution vulnerability to more than 100 million users ([Sebastian 2019](https://armin.dev/blog/2019/04/adblock-plus-code-injection/)) ([ME 4][ME], [LG 5][LG]). **Trust follows money and disclosure.** Paid allowlisting, ad data sold to the ad industry and an undisclosed contractual exception each cost the project that did it more than any coverage gap ([HP 2, 3, 11, 14][HP]). **The central idea: protection is a function of the shipped list version and the user's explicit settings, nothing else; each threat is assigned to the layer that can actually defeat it; and every decision is a typed, recorded, explainable event that the page cannot distinguish and the user and developer can trace to a published entry with its evidence.** The decisions that matter most: (1) **no rule of any list, Izcal's own included, can carry code or rewrite a body**; script-level interventions are a closed, typed vocabulary written in the engine (CB-D2); (2) **fingerprinting is defended at the API by D-S1 and its amendments**, with list blocking of third-party fingerprinting scripts as a complement and per-script API normalization held behind a detectability gate (CB-D3); (3) **no learning on the user's device**; Izcal learns centrally on a crawler that is Izcal itself and ships one signed, logged bundle identical for every user (CB-D4, CB-D17); (4) **navigation is its own defence layer**: bounce tracking, link decoration and forced redirects are handled in navigation, not in the request matcher (CB-D8); (5) **Stage 2 extensions contribute rules as data with zero read capability**; there is no blocking request-interception API (CB-D15). Content blocking is staffed inside the networking spec's lists and content-blocking tracks (5–9†) plus about **5–9.5 people†** of work this spec adds (navigation defences, renderer interventions, surrogates, breakage replay, the user-facing surfaces, the second review, the log, witnesses and entity data, and governance), so about **10–19 people†** in all, of whom about 2.5–5.5† are curators, reviewers, triagers and policy staff rather than engineers (Section 9.3).

Note sources are cited as **HP** ([history and postmortems][HP]), **TT** ([tracking threats][TT]), **ME** ([matching engines and performance][ME]), **LG** ([list production and governance][LG]), **CU** ([compatibility, breakage and UX][CU]), **GO** ([gaps and opportunities][GO]) and **FR** ([frame][FR]), with the row number. Primary sources are linked inline for the claims a decision or target depends on.

## 1. The problem and who has it

| User | Task, in their words | What fails today |
|---|---|---|
| **People browsing, default settings** | "Sites should not follow me around, ads should not eat my battery, and pages should still work" | Lists catch third-party URLs; first-party, bundled and server-side tracking pass ([TT 2.8, 4.7][TT]); about half of users facing breakage give up on the site or the blocker ([CU 1.2, 1.4][CU]) |
| **People who know what they want** | "Block more here, block nothing there, add the list I trust" | Controls are all-or-nothing per site; users "drop shields" and forget to raise them ([GO 15.3][GO]); a custom list set is a fingerprint nobody tells them about ([TT 8.4][TT]) |
| **People reporting breakage** | "Fix it without making me send you my browsing" | Reports go unanswered; vendors publish no triage times; 38–66% of forum reports are still reproducible when examined ([CU 1.8, 4.3, 4.6][CU]) |
| **Web developers and site operators** | "Why did my request fail in this browser, and how do I contest it?" | `net::ERR_BLOCKED_BY_CLIENT` names no rule, list or reason; developers bisect by disabling extensions ([CU 7.3, 7.4][CU]); no list publishes per-entry evidence or an appeal path ([LG 4][LG]) |
| **Izcal's list curators** | "Find what tracks, prove it, publish it, catch evasion fast" | Research classifiers needed forked, instrumented browsers ([GO 6][GO]); replica ad domains survived about 410 days before listing ([LG 4.7][LG]) |
| **Izcal's engineers** | "Fast, small, deterministic, never panics on a hostile list, replays exactly" | No conformance or differential suite exists across filter engines; same-priority order is undefined in DNR and AdGuard, ordered by ruleset kind in Firefox, order-defined in Safari's JSON, and not documented by ABP or uBO ([ME 1.7, 1.9, 1.12, 1.14, 7.3][ME]) |

**What Izcal's specs already decide.** HGBA-2 M-1 (2.3.0 and 2.3.2): content blocking is among what Izcal builds; Brave's `adblock` is excluded by name (2.3.0); everything shipped is MIT; another engine's or blocker's source code is never evidence. HGBA-3's P2 scope note allows default-on interventions only as **class (c)**, "privacy protections on by default that carry a recorded P1 analysis", and requires each to publish its compatibility cost per release ([H3][H3]). Stage 1 Section 6 builds blocking into the engine (network service and style), not on an extension API, with cosmetic filters as a user-origin sheet, the block-list set as per-container policy, and no extensions in Stage 1 (D-S3). The networking spec owns the matching semantics, the matcher (`hg-content-block`, L5, tier P), the cosmetic compiler and Izcal's own lists (NW-D26); places the sealed automaton in each partition service (≤ 256 KiB† charged, p99 ≤ 20 µs†, NW 6.15); makes a blocked request a `FailureReason::ContentBlocked` that the page cannot distinguish (NW-22); routes Izcal's own traffic through the system service with OHTTP for user-specific requests (NW-D17); and sets NW-54 (every entry with evidence, ≥ 90%† of classifier-flagged third-party requests blocked, ≤ 1%† corpus breakage, triage ≤ 7 days†). The style spec applies cosmetic rules as user-origin sheets ([ST 6.16][ST], [P21][RN]). The footprint spec treats blocking as a measured lever (FP-28) and defines script-attributed interventions (FP-D7). D-S1's fingerprinting model is amended by TX-D4 (no `measureText` noise), GR-D12 (no 2D canvas noise; WebGL and WebGPU noise once where pixels leave), MD-D14 (no noise on Izcal-rendered audio, once MD-30 passes) and LY-D11 (geometry normalized by construction). The full list is in [FR 2][FR].

**What success means.** Protection measured, not asserted (Section 8.3); breakage measured, including interaction-triggered and cosmetic breakage, and small (CB-15); every entry and every decision explainable (CB-19); FP-28 met per service with zero allocations per match (CB-18); no new fingerprinting surface from lists, settings or learned state (CB-8, CB-9); exact replay of every decision (CB-3); zero shipped bytes of third-party lists or engines and zero citations of their source (Section 7).

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1990s | Hosts files | Mapped ad hosts to a dead address | The only interception point a user controlled | Survives as the domain-list format and as DNS sinkholes (Pi-hole, 2014), which see no path, frame or first party ([HP 12][HP]) |
| 2002–2006 | Adblock, then Adblock Plus | URL-pattern rules and element hiding in an extension | Mozilla's extension model saw every load; lists separated from the engine so one curator could serve millions | ABP's syntax became the de facto standard every blocker reads ([HP 0, 1][HP]) |
| 2005 | EasyList | One shared, subscribable list | Users should not each write rules | Lives on, hand-curated by 4 named maintainers, one funded by Brave since 2019, with no evidence per entry; 90.16% of its network rules never fired in a 74-day crawl of 10,000 sites ([LG 1.1][LG], [ME 8.1][ME]) |
| 2009–2011 | Ghostery; Disconnect; Acceptable Ads | Tracker identity by company; category lists; paid default-on allowlist | Visibility; selective blocking; a business model | Ghostery's GhostRank fed ad-industry data and the product was sold twice; Disconnect powers Firefox and Edge; Acceptable Ads survived competition law (BGH 2018) at a lasting trust cost ([HP 2, 3][HP]) |
| 2014 | Privacy Badger | Learned trackers from behaviour, no list | Lists are subjective and stale | Local learning **off by default in October 2020** after Google showed it could be steered and read back; one crawler-built list ships to everyone ([HP 4][HP]) |
| 2014 | uBlock Origin | Fast, low-memory blocker; extended syntax (`$removeparam`, scriptlets, redirect resources) | Matching cost and memory, not lists, made blockers heavy | Thrives on Firefox; disabled on Chrome by Manifest V3 (Chrome 138, July 2025) ([HP 5][HP]) |
| 2015 | Safari content blockers | Declarative JSON rules compiled ahead of time, run in the network subsystem | Latency, energy, memory, and privacy: the extension never sees URLs | The model reached Chrome; rule caps (50,000, later 150,000 per blocker) and multi-second compiles on phones ([ME 2.10–2.12][ME]) |
| 2015 | Firefox Tracking Protection | Disconnect-listed domains blocked | Measured: 44% median page-load reduction, 39% less data on news sites | Grew into ETP, default-on from 2019 ([Kontaxis and Chew](https://arxiv.org/abs/1506.04104), [HP 7][HP]) |
| 2017–2020 | Safari ITP | On-device classifier; storage purge and partitioning | No list, nothing to configure | The classifier was a side channel; WebKit moved to uniform, classifier-independent rules (full third-party cookie blocking, March 2020), keeping the classifier where its effect is hard to observe ([HP 6][HP]) |
| 2018–2025 | Chrome Manifest V3, declarativeNetRequest (DNR) | Capped declarative rules replaced blocking request interception | Stated: privacy and security of request modification, extension performance | Caps raised only with blocker vendors' data; updates tied to store review; no top-level context until Chrome 145; non-network features fell back to page JavaScript ([HP 5][HP], [GO 1][GO]) |
| 2019–2026 | Brave Shields | Engine-native blocker; Rust rewrite (2019, 69× faster); CNAME uncloaking (2020); debouncing (2021); zero-copy store (2026, −75% memory, about 45 MB) | The browser's own business; no extension limits | Successful, but it took two rewrites to reach a mapped, zero-copy store; stopped first-party network blocking by default (2021) because breakage made users turn Shields off ([HP 8][HP], [CU 2.7][CU]) |
| 2020 | DuckDuckGo Tracker Radar | Crawler-derived dataset with prevalence, API use and entities | Hand lists stale and break sites | The closest precedent for Izcal's lists; data CC BY-NC-SA; an undisclosed contractual exception for Microsoft (2022) cost trust ([HP 11][HP], [LG 1.11–1.14][LG]) |
| 2020–2025 | Chrome third-party cookie phase-out; Privacy Sandbox | Promised removal, then reversal; most Sandbox APIs retired 17 October 2025 for "low levels of adoption" | Ads without cross-site cookies | A vendor whose revenue depends on tracking could not ship default protection; CHIPS, FedCM and Private State Tokens survived ([HP 9][HP]) |
| 2021–2025 | Firefox SmartBlock, query stripping, bounce tracking protection | Stand-in scripts for blocked trackers; listed parameters stripped; bounce trackers' data purged | Blocking broke sites; trackers moved to URLs and navigations | Layered defence became the norm ([HP 7][HP], [CU 2.1–2.4][CU]) |
| 2023–2025 | Safari Link Tracking Protection and Advanced Fingerprinting Protection | Parameter stripping; per-site noise on readback, fixed screen metrics | Fingerprinting moved off lists | AFP default in all browsing from Safari 26 (secondary sources) ([GO 7.4, 7.5][GO]) |

### 2.2 Technology mismatches: assumptions that no longer hold

| Assumption | Where it came from | Why it no longer holds |
|---|---|---|
| **Trackers are third-party URLs** | Hosts files, EasyList | First-party serving and bundling, CNAME and A-record cloaking, reverse proxies and server-side tagging ([TT 4, 5][TT]) |
| **The third-party cookie is the tracking mechanism** | 2000s ad tech | Izcal partitions every third-party cookie by default (NW-D7); identifiers moved into URLs, first-party jars and fingerprints ([TT 1.7, 3, 5.3][TT]) |
| **Blocks are rare, so optimize the miss path** | Brave's C++ Bloom filter | 39% of requests were blocked in Brave's 2019 data ([ME 2.2][ME]) |
| **The blocker is an extension that sees requests through an API** | Firefox and Chrome extension models | APIs are removed under blockers (NoScript 2017, uBO on Chrome 2025); an extension sees no partition, CNAME or initiator chain ([HP 5][HP], [ME 6.6][ME]) |
| **Rule count is the cost** | DNR, Safari caps | The cost is memory and compile time; Brave's 2026 gain came from the storage format, not matching ([ME 2.8][ME]) |
| **Learning on the device is private** | ITP 2017, Privacy Badger 2014 | Learned state is observable and steerable ([HP 13][HP]) |
| **Lists are trusted data** | EasyList's volunteer model | Lists can carry code and can be served selectively by IP ([LG 5.1][LG]) |
| **A filter list is a privacy feature with no privacy cost** | Everyone | The list set is a fingerprint ([TT 8.4][TT]) |
| **Breakage is found by users** | Every list | Users rarely report; reports rot within weeks ([CU 1.2, 1.8][CU]) |

## 3. Today: the landscape and its gaps

**What the threats are, and which layer can defeat each.** The full threat table is [TT, threat-model table][TT]; its conclusions:

| Threat | Prevalence (measured) | Layer that can defeat it |
|---|---|---|
| Third-party cookies and cookie syncing | Syncing among 45 of the top 50 third parties (2016) | **Partitioning** (NW-D7), already default |
| Fingerprinting, third-party script | 10–25% of top sites; lists remove about 5% of test canvases in practice | **API level** (D-S1), then the network list |
| Fingerprinting, first-party or bundled | 49–52% of canvas-fingerprinting sites | **API level only** |
| Bot-detection fingerprinting | 23% of canvas-fingerprinting sites in the top 20K use one vendor's bot-detection canvas | API level, preserving the bot signal; never blocked by default (PRIV-1) |
| Bounce tracking, UID smuggling, link decoration | 8.1% of navigation paths smuggle UIDs, 10.8% with bounce tracking; 41% of dedicated smugglers missing from Disconnect | **Navigation** |
| CNAME and A-record cloaking | 9.98% of the Tranco top 10K; cookie leaks on 95% of those sites | **DNS uncloaking** plus cookie policy |
| First-party proxying and server-side tagging | > 54% of sites; lists match 27.5% of such scripts | **Request shape** (parameter key sets matched 63% of what lists matched 0% of), script attribution, cookie policy; off-device forwarding cannot be blocked, only made visible |
| Malvertising | About 1% of ads (2014); forced redirects 81% of malicious ads (vendor, 2024) | **Navigation** (forced redirects), network list |
| Cryptojacking | 0.011% of domains (2019); collapsed after Coinhive closed in 2019 | **Behaviour** (CPU budgets, FP-D7), list |
| Filter-list fingerprinting | 84% of lists identified; median anonymity set 48 users | **Uniform list sets**, side-channel-free hiding |

Sources: [TT 1.3, 2.3, 2.7, 2.8, 2.10, 3.1–3.2, 3.8, 4.1–4.2, 4.7, 6.1, 6.3, 7.3, 8.4][TT].

**What current tools cannot see or do.**
- **Why something was blocked.** No blocker shows the list version, the entry and its evidence, the partition or the initiating script, and none keeps history beyond an open window; uBO's logger is the richest, and it is an extension. Chrome reports `ERR_BLOCKED_BY_CLIENT`; DNR rations rule-match feedback to 20 calls per 10 minutes ([GO 1.7, 5][GO]).
- **What would have happened without the rule.** No blocker reproduces breakage; maintainers reload live sites and bisect by hand; AdGuard's filter tracker received 24,389 issues in 2025 ([GO 9, 15.1][GO]).
- **Context.** DNR gained top-level-site conditions only in Chrome 145; no DNR implementation uncloaks CNAMEs; inherited-origin frames (`about:blank`, `srcdoc`) bypass blockers (Local Frames, CCS 2025) ([GO 10][GO], [ME 6.6][ME]).
- **Evidence and process.** No list publishes per-entry evidence, an appeal path or a response time; every controversy came from an undisclosed exception ([LG 4][LG]).
- **Correctness.** No differential or conformance suite exists across filter engines; same-priority order is undefined in DNR and AdGuard and only partly defined in Firefox ([ME 7][ME]).
- **Cost of the list set.** No browser tells the user that a custom list set makes them identifiable ([TT 8.4][TT]).

**What Izcal can expose that no one does** (ranked in [GO, Opportunities][GO]): replay-backed breakage triage (B1, DET-1); a typed reason resolving to (list, version, entry, evidence, partition, initiator, CNAME chain) from the decision log (NW 6.18, JS 6.10); per-script attribution of API calls in the engine itself, which research needed forked browsers for; a crawler that is Izcal in recording mode, with flow-based classification; partition-complete matching on (T, C, F); per-entry energy and memory evidence from the footprint ledger; and a Stage 2 extension surface that never needs per-request extension code.

## 4. First principles

### 4.1 Users' real tasks

A person browsing wants to be **unlinkable across sites and contexts**, to **not pay** in battery, bandwidth or security for third parties they did not choose, and to have **every page work**. A curator wants to **know what a party does**, prove it and keep up with evasion. A developer wants **the reason at the point of failure**. None of these is "block URLs matching a list"; that is one mechanism among several.

### 4.2 Platform primitives

- **Fetch** gives every request a destination, initiator, client and mode, and a network error that reveals nothing ([Fetch](https://fetch.spec.whatwg.org/)). Content blocking is a network error at the partition service (NW 6.7).
- **The partition key** (T, C, F, X, N, R, n) is richer than any blocker's context (NW-D7).
- **HTML navigation** decides top-level navigation, user activation and sandboxing ([HTML](https://html.spec.whatwg.org/multipage/browsing-the-web.html)); the Privacy CG's **Navigational-Tracking Mitigations** draft defines bounce tracking and link decoration, with no consensus on its algorithm ([Privacy CG](https://privacycg.github.io/nav-tracking-mitigations/)).
- **CSS Cascade** defines the user origin; **Selectors 4** has `:has()` natively in every engine ([Cascade 5](https://www.w3.org/TR/css-cascade-5/), [Selectors 4](https://www.w3.org/TR/selectors-4/)).
- **DNS** (RFC 1034) exposes the CNAME chain to Izcal's own stub resolver (`hg-dns`, NW-D8).
- **The engine's own facts**: script attribution of every API call and of time and allocation (JS 6.10), the decision log (NW 6.18), recorded B1 inputs (JS-D15, NW 6.16), the semantic plane (B8).
- No standard governs content blocking itself; filter syntax is documented by its vendors, which M-1 part 2 treats as interoperability evidence, recorded as black-box observations where the documents are silent ([ME 1.16][ME]).

### 4.3 Izcal's principles that bind this design

P1 (security and privacy first) puts unlinkability and no-code-from-lists above coverage. P2 with its scope note makes every default-on intervention a class (c) intervention with a recorded P1 analysis and published compatibility cost. P3 makes each decision replayable and each component oracle-checked. M-1 makes the matcher, compiler, lists, surrogates and crawler Izcal's own, MIT, and evidence only from prose and black-box tests. B10 and AG-0: the browser runs no model unless AI is on; a classifier in Izcal's lab is not a browser feature.

### 4.4 Assumptions kept and broken

| Assumption | Keep or break | Why |
|---|---|---|
| Lists separate from the engine | **Keep** | One curated, reviewed artifact serves everyone; it is also what makes the list set uniform ([HP 1][HP]) |
| ABP-family syntax as the user-list format | **Keep (published subset)** | Twenty years of lists use it; the interoperable core is shared by every dialect and DNR ([ME 1][ME]) |
| Lists may carry scriptlets, body rewrites, header removal "from trusted lists" | **Break** | A trust tier is a patch over list-supplied code; Izcal writes interventions in the engine instead (CB-D2) |
| Blocking is the primary privacy defence | **Break** | Partitioning, API-level defences and navigation defences do most of the work; lists address the rest (CB-D1) |
| Learning trackers on the user's device | **Break** | Observable and steerable ([HP 13][HP]); learn centrally (CB-D4) |
| "Third-party" relative to an entity (same owner = same party) | **Break** | Site-based, as NW 6.15 defines; entity exemptions favour conglomerates (CB-D7) |
| A rule-count cap | **Break** | Memory and compile budgets instead (CB-D11) |
| Undefined order among equal rules | **Break** | One total order, needed for replay (CB-D11, G83) |
| Per-site "off" as the only control | **Break** | Narrower, expiring overrides first (CB-D19) |
| Counting blocked requests | **Break** | Count distinct parties prevented; Edge documents that request counts fall as protection rises ([CU 7.7][CU]) |
| Generic cosmetic selector sheets | **Break for Izcal's lists, keep for user lists** | "Collapse what was blocked" needs no selectors; generic rules are the easiest list fingerprint and the largest cosmetic cost ([TT 8.5][TT], [ME 3.1][ME]) |
| Breakage is triaged on live sites | **Break** | Replay with and without the rule (CB-D18, CB-D20) |

## 5. Lessons learned

Ideas and evidence only, never code or a template (M-1).

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| EasyList | Volunteer list; no evidence per entry; 90% of network rules never fired | Lists rot without evidence and expiry | Every entry carries evidence, two reviews and a last-fired date, and is re-validated every 12 weeks† (CB-D16, CB-21) |
| Acceptable Ads, GhostRank, DuckDuckGo's Microsoft exception | Money or contract shaped what was blocked, undisclosed | Hidden exceptions cost more trust than gaps | No payment of any kind for any entry or exception; every exception is a published entry; legal removals published with the demand (CB-D16) |
| Privacy Badger, ITP | Per-user learned state was steerable and readable | Ship the same decisions to everyone | No on-device learning; protection is a function of list version and explicit settings (CB-D4, G86) |
| ABP `$rewrite` (2019) | A list rule could execute code on Google sites; lists can be served selectively by IP | A list is executable policy | No code-bearing rule kinds at all; signed, logged, identical releases (CB-D2, CB-D17) |
| uBO, AdGuard, ABP trust tiers | Every vendor restricted scriptlets and header removal by list provenance | The capability itself is the risk | Typed engine interventions instead of scriptlets; header changes from a closed list that never weakens security (CB-D2, CB-D11) |
| Firefox Tracking Protection (2015) | Published measured gains | Measure and publish | Per-release protection, breakage, bytes, energy and CAPTCHA numbers (Section 8) |
| Firefox SmartBlock, Brave replacements | Blocking scripts broke pages; stand-ins fixed them | Breakage needs a built-in remedy | Izcal-written surrogates and neutral resources, shipped with the build (CB-D2) |
| Brave Standard mode | First-party blocking broke sites and made users turn Shields off | Default scope matters more than list quality for breakage | Default lists block third-party requests, plus first-party only for uncloaked trackers, evidenced request shapes and security entries (CB-D6) |
| Brave adblock-rust (2019, 2026) | A design for rare blocks; heap structures cost about 45 MB | Design for the real block rate and for many processes sharing one store | Token-indexed, zero-copy, sealed file from day one (CB-D11) |
| Chrome DNR | Caps set by the vendor; updates tied to store review; no top context; explanations rationed | A declarative API is right; its limits were not | Stage 2: lists as data, compiled by Izcal, updated as data, explained locally (CB-D15) |
| Safari content blockers | The extension never sees URLs, and WebKit records nothing | Privacy and explanation are not in conflict if explanations stay local | Decision log local, never sent; list authors learn nothing (CB-D20) |
| Chrome heavy-ad intervention | 0.3% of ads caused 27% of ad data and 28% of ad CPU | A small attributable set causes most cost | FP-D7's script-attributed thresholds, calibrated on Izcal's corpus |
| Safari ITP CNAME defence; Brave uncloaking | Trackers moved behind first-party CNAMEs | DNS evidence is needed for first-party-looking requests | Every hop of the chain and IP entries matched; cookie lifetime cap for cloaked responses (CB-D9) |
| Privacy CG bounce tracking; Brave debouncing; Firefox BTP | Tracking moved to navigations | Navigation is a separate surface | Navigation defences with their own targets (CB-D8, CB-10) |
| Double-Edged Shield (2025) | Scriptless probes identify list sets | The list set is a fingerprint | Uniform default list sets; user lists labelled as a cost; user-origin hiding never changes fetches (CB-D13) |
| SINBAD, Smith et al. | 25–44% of breakage appears only after interaction; CSS hiding causes about 53% of rule-attributed breakage | Load-time diffs miss most breakage | Interaction-driven corpus checks with engine-level comparison (CB-15) |
| Axel Springer v eyeo (BGH, 31 July 2025, remanded) | Whether changing DOM and CSSOM modifies a protected program is open | The cosmetic mechanism is legally contested in Germany | Cosmetic rules are a user-origin sheet applied by the engine; German legal opinion before shipping there (CB-M4) |
| WebGraph, Khaleesi | Content-feature classifiers are evadable; flow features are robust | Classify by information flow | Offline flow classification in the crawler, human review (CB-D16) |
| Mathur 2018, Nisenoff 2023 | Users rarely report; half abandon | The off switch is the reporting moment | Offer a narrow fix and a consented report when protection is turned off (CB-D19) |

## 6. Design

### 6.0 Architecture at a glance

```
 Izcal lab (tools, never shipped)                         Izcal servers (public files)
 ┌───────────────────────────────────────────┐            ┌──────────────────────────────┐
 │ crawler = Izcal in recording mode, headed, │  reviewed  │ list bundle (TUF target,     │
 │ multi-region, paired arms (with/without)   │──entries──▶│ signed, MIT data) + diffs    │
 │ flow classifier · review queue (2 people)  │            │ witnessed transparency log   │
 │ evidence records · breakage replay · triage│◀─reports───│ OHTTP gateway (reports only) │
 └───────────────────────────────────────────┘   (OHTTP)  └──────────────┬───────────────┘
                                                                         │ daily†, wake grid
 Browser                                                                 ▼
 ┌─────────────────────────────────────────────────────────────────────────────────────────┐
 │ system service: fetch bundle → broker update client verifies TUF + witnessed checkpoint │
 │ container service (or its route): fetch that container's user-added list URLs            │
 │        │ sealed source handles                                                           │
 │        ▼                                                                                 │
 │ utility process (strictest tier): hg-cb-syntax parses · hg-cb-compile builds one sealed  │
 │   file per distinct per-container list set: network automaton, cosmetic per-site subsets,│
 │   intervention table, entry metadata (hg-cb-format)                                      │
 │        │ sealed, read-only file (regenerable cache, FP-D16)                              │
 │   ┌────┴───────────────────────────────┬───────────────────────────────┐                 │
 │   ▼                                    ▼                               ▼                 │
 │ partition service (T, C)            container service (C)          renderer (site, C)    │
 │  hg-content-block: phase A (URL,     hg-nav-protect: bounce-tracker hg-style: cosmetic   │
 │  context) · phase B (CNAME chain,    decisions (site IDs, times),   subset as user-origin│
 │  IP) · phase C (response headers)    deletion orders; cookie        sheet; paint-suppress│
 │  → block / redirect-to-built-in /    policy hooks (CB-D9, CB-D10)   (graphics); restrict │
 │  removeparam / restrict / allow                                     profiles (JS, CB-D10)│
 │  stripping and debouncing on each   (interaction record in the                          │
 │  navigation hop                      storage service)                                    │
 │  decision log record (G85)                                                               │
 │                                                                                          │
 │ broker: navigation events (typed) → container service; forced-redirect rule (CB-D8)      │
 │ shell: site panel, per-site controls, report flow; DevTools: Protections view (IIP)      │
 └─────────────────────────────────────────────────────────────────────────────────────────┘
```

**Who owns what.** The networking spec owns `hg-content-block` (the matcher), the request-matching semantics and the lists (NW 6.15, NW-D26); this spec specifies those semantics in detail (Sections 6.2–6.4), the list operation (6.8–6.9), and adds the navigation layer (`hg-nav-protect`, split between the partition service and the container service, Section 6.5), the intervention vocabulary and surrogates, the compiled format, the user-facing surfaces and the Stage 2 extension surface. The style spec applies cosmetic subsets (ST 6.16); the graphics spec owns paint suppression (Section 6.7); the JavaScript engine spec owns the attribution restricted scripts rely on (Section 6.4); the footprint spec owns FP-28 and FP-D7's thresholds; the shell owns the settings UI (NW 6.15). Amendments are in Section 9.5.

### 6.1 The layered model: each threat to the layer that can defeat it (CB-D1)

Content blocking in Izcal is not one mechanism. It is six layers, and each threat is assigned a **primary layer**, the one that can actually defeat it, with the others as complements:

| Layer | Mechanism | Owner | Primary for |
|---|---|---|---|
| **P** Partitioning and cookie policy | Partition key (T, C, F, X, N, R, n); CHIPS-style third-party jars; cookie policy for cloaked responses and restricted scripts | NW-D7, NW 6.9; CB-D9, CB-D10 | Third-party cookies, cookie syncing, cookie leaks to cloaked endpoints |
| **V** Navigation | Bounce-tracking state deletion, link-decoration parameter stripping, debouncing, forced-redirect blocking | This spec, CB-D8 | Bounce tracking, UID smuggling, malvertising redirects |
| **S** Script and API | D-S1 and its amendments (normalization, canonical output, seeded noise on WebGL and WebGPU readback); restricted scripts; FP-D7 CPU and wakeup budgets; surrogates | D-S1, GR-D12, TX-D4, MD-D14, LY-D11, FP-D7; CB-D2, CB-D3, CB-D10 | Fingerprinting (first- and third-party), cryptojacking, heavy ads |
| **N** Network matcher | Phase A (URL and context), phase C (response headers) | NW 6.15; Sections 6.2–6.3 | Third-party trackers, ad and malvertising endpoints, first-party proxies by request shape |
| **D** DNS uncloaking | Phase B: every name in the CNAME chain, and reviewed IP-range entries | `hg-dns`, NW-D8; CB-D9 | CNAME and A-record cloaking |
| **C** Cosmetic | Paint suppression of blocked elements; per-site user-origin hide rules | ST 6.16; CB-D12, CB-D13 | Empty ad slots, visible residue; user lists' cosmetic rules |

**Why layers, and why this assignment.** The evidence in Section 3 says that a URL list cannot reach half of fingerprinting, most first-party tracking, or navigational tracking. Making the list the centre would set Izcal's protection by its weakest layer's coverage. Each layer gets its own targets (Section 8.3), so a gap in one is visible rather than averaged away. **What the network list is for**, after partitioning and D-S1: third-party scripts and endpoints whose behaviour joins users or harms them (tracking, fingerprinting by third-party scripts, malvertising, mining), the bytes and energy of third-party ads (FP-28), and first-party proxies identifiable by request shape.

### 6.2 Network matching semantics (CB-D6, CB-D7, CB-D11)

**The request context.** Every request that leaves a partition service, on any channel (HTTP fetch of any destination, navigation, WebSocket, WebTransport, `sendBeacon` and `ping`, service-worker `fetch()`, prefetch and speculative fetches, WebRTC's TURN and STUN servers as host-only), is matched over one typed context:

| Field | Source | Notes |
|---|---|---|
| URL | `hg-url`, after parsing | Host in ASCII (IDNA to punycode), lowercased; path and query as the URL Standard serializes them; no further decoding (Q-CB-6 records how other blockers normalize, black-box) |
| Resource type | Fetch's request destination and mode | One mapping table, published in the rule-syntax page: `document`, `subdocument`, `script`, `stylesheet`, `image`, `font`, `media`, `xhr` (fetch, XHR, EventSource), `websocket`, `webtransport`, `ping` (beacon, ping, reports), `webrtc`, `other` |
| Method | Request | |
| T, C, F, X | Partition key (NW-D7) | F is the **initiating frame's site**; for frames with inherited origins (`about:blank`, `srcdoc`, `blob:`) F and the initiator are those of the creating document, as HTML's origin inheritance defines, so the Local Frames bypass class does not apply (CB-6's suite) |
| Initiator | Frame and, where the engine attributes it, the script (JS 6.10) | Used for explanations and for restricted-script rules; never needed for a block decision on an unattributed request |
| CNAME chain and resolved addresses | `hg-dns` (phase B) | Every name in the chain, in order |
| Navigation flags | Broker | Top-level or subframe, user activation present |
| Service-worker route | NW 6.7 | A service worker's own `fetch()` is matched in the worker's context (F is the worker's site); responses a worker builds itself or serves from Cache Storage are not network requests and are not matched (Section 6.16) |

**Party.** A request is **third-party** when its site differs from F, as NW 6.15 defines. "Site" is the registrable domain from the Public Suffix List (standards data). Same-entity requests are **not** exempt (CB-D7): `google.com` loading `doubleclick.net` is third-party. Izcal keeps its own entity data for explanation and grouping only (Section 6.8).

**First-party scope of the default lists (CB-D6).** Izcal's default lists block third-party requests, and first-party (same-site) requests only in three cases, each needing its own evidence: (1) the request's CNAME chain or resolved address matches a tracking entry (phase B); (2) an evidenced **request-shape** entry for a first-party proxy or server-side tagging endpoint, matching path patterns and query-parameter **key sets**, the approach that matched 63% of first-party tracking requests that EasyList and EasyPrivacy matched 0% of, with 3% false positives and 1 of 2,300 sites visibly broken ([TT 3.8][TT]); (3) security entries (malvertising, mining). Brave stopped general first-party network blocking in 2021 because the breakage made users turn protection off ([CU 2.7][CU]); Izcal's narrower first-party scope keeps the cases with evidence and leaves the rest to layers P and S.

**The rule language.** Izcal publishes a rule-syntax page (NW 6.15) for **Izcal filter syntax**, which is the interoperable core of the published ABP-family syntax plus explicit context fields:

| Family | Form | Notes |
|---|---|---|
| Pattern | `*`, `^` (separator: any character other than ASCII letter, digit, `_ - . %`, or end of URL, as DNR documents it), `\|` start or end anchor, `\|\|` host anchor | Compiled to fixed match primitives (Section 6.3) |
| Regex | `/…/` | Linear-time subset only: no backreferences, no lookaround, bounded repetition; each compiled automaton ≤ 4 KiB†; anything else is rejected with its reason ([ME 2.13, 2.17][ME]) |
| Options | resource types and `~` negation; `third-party` / `first-party`; `domain=` (imported lists: matches when the frame's or the top-level site's host matches, as NW 6.15 states); Izcal's explicit `top=` and `frame=`; `method=`; `match-case`; `to=` (request host set); `denyallow=`; `important`; `badfilter` | `domain=` semantics are recorded black-box against other blockers (Q-CB-6) |
| Exceptions | `@@` | **Scoped by trust domain**: an exception cancels only blocks from its own domain (below); `badfilter` and `$document` are scoped the same way |
| Actions | block (default); `redirect=<built-in name>`; `removeparam=<literal name>` or a linear-time regex; `csp=<directives>` (adds a policy built from an allow-list of fetch and navigation directives, `script-src`, `img-src`, `frame-src`, `connect-src`, `object-src`, `worker-src`, `media-src`, `font-src`, `style-src`, `base-uri`, `form-action`, `upgrade-insecure-requests`, `block-all-mixed-content`, with source expressions limited to keywords and `'none'`; `report-uri`, `report-to` and host sources are rejected, so a policy can neither name a host nor report anywhere); `permissions=<policy>` (adds, restrictive only, no `report-to`); `restrict=<profile>` (Izcal lists only, CB-D10) | No action can name an external URL, carry script text or remove a header |
| Response phase | `header=<name>[:<value pattern>]` | Evaluated after response headers (phase C); can only block delivery of the body. The request has already reached the server, as DNR documents for its equivalent ([GO 1.6][GO]) |
| Cosmetic | `##selector`, `#@#selector` | Section 6.7 |

**Precedence: one total order (G83).** For each request the matcher computes the set of matching rules and decides by a single key, so ties never depend on hash order and replay is exact ([ME 7.3][ME]):

1. **The user's explicit per-site override** for this (site, container) (CB-D19).
2. **Within each trust domain separately**: `important` blocks, then exceptions, then redirect-blocks, then blocks, then modifiers (`removeparam`, `csp`, `permissions`, `restrict`); ties by rule ID. The trust domains are: **Izcal's bundle** (all of Izcal's lists together, so an entry in Izcal's Exceptions list cancels an Izcal Tracking or Ads block; a Security entry is cancelled only by an exception that names the same entry ID); **the user's own rules**, whose exceptions may cancel blocks in every domain, because the user is the principal; and **each user-added or extension list**, its own domain.
3. **Across domains**: the request is blocked if any domain's own outcome is a block that no user rule cancels. Domains are ordered (the user's own rules, Izcal's bundle with Security, Tracking and Ads in that order, then user-added lists in the order added) only to choose which entry the explanation names and which redirect resource is served.

**Why exceptions are scoped by trust domain.** In ABP, uBO and AdGuard an exception in any enabled list cancels a block from any other list ([ME 1][ME]), so a user-added list, or a compromised one, could unblock Izcal's security entries with one line. The declarative APIs already scope exceptions: in Chrome's DNR an `allow` in one extension never overrides another extension's `block` ([DNR](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest)), and in Safari an exception in a different content blocker from its block rule may not apply ([ME 2.12][ME]). Izcal follows that precedent with an explicit unit: Izcal's bundle is one domain (so its own breakage exceptions and login-flow entries work across its lists), the user's own rules can cancel anything, and each third-party list governs only itself. The cost: a user-added list written to correct another user-added list's over-blocking no longer can; the user's own rules and per-site overrides still can.

**Phases.** **A** runs on the URL and context before any connection. **B** runs after DNS resolution, on every name in the CNAME chain and on the resolved addresses, only if A allowed the request; DNS is needed for the connection anyway, so B adds lookups, not round trips; for a reused or coalesced connection the partition's DNS cache entry holds the chain (NW 6.6). **C** runs on response headers. Each phase's decision is a decision-log record (G85).

**Popups.** A popup navigation (`window.open`, `target=_blank`) is matched as a `document` request whose F and T are those of the opener's frame and top-level site; `$popup` rules block it before the new top-level traversable is created.

**Blocked means a network error, and nothing else visible.** A blocked request is `FailureReason::ContentBlocked`, byte-identical to other network errors for the page (NW-22). Two consequences this spec adds. (1) **Generic hiding of blocked elements is paint-only** (Section 6.7): a blocked image or frame is not drawn, but its box and layout are what any failed load produces, because collapsing it in layout would let the page tell "blocked" from "failed" through geometry; the "blocked" flag is a paint-only input that no API exposes. (2) A blocked request fails sooner than most network failures, so **timing still reveals that some list blocks this URL**. With uniform list sets (CB-D13) that reveals only "an Izcal user with the default lists", which the user agent string already says; it is recorded as a residual channel and measured (CB-8).

### 6.3 The compiled form, the compiler and the matcher (CB-D11)

**Compile, don't interpret.** Lists are compiled at list-update time in a utility process at the strictest tier (FP 6.11) by `hg-cb-compile`; `hg-cb-syntax` is the only parser of list text (G80). The output is one **sealed, position-independent file per distinct per-container list set** (two containers with the same set share it), with offsets, never pointers, verified once by `hg-cb-format`'s structural verifier in the utility process and mapped read-only by every partition service of that container (clean, uncharged pages). Brave's 2026 move to a zero-copy format cut its blocker's memory by about 75% (about 45 MB) ([ME 2.8][ME]); Izcal starts there.

| Section of the file | Content |
|---|---|
| Header | Format version, compiler version, the hash and version of each source list, the bundle signature it was compiled from, section offsets and checksums |
| Host table | `\|\|host^`-style rules (the majority), as a perfect hash over reversed host labels with suffix walk |
| Token index | Every other rule filed under its rarest token over the whole compiled corpus ([ME 2.1, 2.5][ME]); token-less rules in a bucket that is a compile-time warning, its size reported |
| Pattern programs | Each pattern as a short sequence of fixed primitives (literal, any run, separator, anchors), never a general regex |
| Regex automata | Linear-time automata, each ≤ 4 KiB† |
| Option bitsets and site sets | Resource-type, party, method bitsets checked before patterns; `domain=`, `top=`, `frame=` and `to=` as perfect-hashed site-ID sets |
| IP-range table | Reviewed address ranges for A-record cloaking (CB-D9) |
| Cosmetic index | Site → offset of that site's CSS subset; the generic subset for user lists |
| Intervention table | Entry → typed intervention ID and validated parameters (Section 6.6) |
| Entry metadata | (list ID, list version, entry ID, rule ID) per compiled rule, for explanations (G85) |

**The match path allocates nothing and cannot panic.** The URL is tokenized as a stream (each token hashed and looked up as it is produced, with no buffer), options are checked as bitsets, patterns run as primitive programs with bounds-checked reads, and decision-log records go into a preallocated ring per service. The hot path is tier P and lint-checked (G82). Published per-request means and medians are 5–20 µs; the only published tail is a p90 for a JavaScript matcher in a 2020 study, and no shipping engine publishes one ([ME 2, 8.3][ME]). FP-28's p99 ≤ 20 µs† applies **per request, all phases summed**, measured on Izcal's corpus request stream with the default bundle and with a 300,000-rule user list, and on a cold start (the first requests of a freshly bound partition service after the file has left the page cache) (CB-18).

**Keyed by content, not by partition.** The compiled file holds no user-, site- or browsing-derived state: nothing a partition service or renderer writes ever enters it. It is therefore keyed by the hashes of its source lists and the compiler version, not by (T, C) as FP-D16's other members are; a file compiled from a user-added list is additionally keyed per container, because the list set is that container's user state (FP-D16 amendment, Section 9.5). A partition service maps at most **two** versions at once (the pinned version of its oldest page and the current one, Section 6.14), and both count in CB-18.

**Budgets, not rule caps.** DNR's and Safari's caps stranded list authors and were raised only with vendors' data ([HP 5.6][HP]); Safari compiled 40,000 rules in 7.4–11.6 s on iPhones in 2018 ([ME 2.11][ME]). Izcal has no rule-count limit. It has **budgets**, each with a refusal path that names the offending list and rule class: compile time (default bundle ≤ 2 s† on a DC-4 reference device; any user list ≤ 10 s† per 300,000 rules), compiler peak memory ≤ 256 MiB†, compiled size (default bundle ≤ 8 MiB†, clean and shared, which restates FP-28's "matcher ≤ 8 MiB† charged" now that the matcher is a mapped file; all lists of one container ≤ 32 MiB†), charged matcher state ≤ 256 KiB† per partition service (NW 6.15), the per-site cosmetic subset ≤ 256 KiB† per site process (FP 6.11). User lists' **generic** cosmetic rules are not copied into each site's subset: they form one shared, mapped, read-only sheet per list set, indexed by the ids and classes each rule requires, charged once; its budget is set at CB-M0 from the measured generic sections of the common public lists, so that adding EasyList is not refused by a number chosen in advance. A hostile list that would exceed a budget is refused, not truncated, so the result is never order-dependent; the refusal names the budget and the amount by which it was exceeded. The compiler is a pure function of its inputs: the same lists and compiler version give a byte-identical file on every OS (CB-28), which replay needs.

**The oracle.** `hg-cb-ref` (tools) is a deliberately naive interpreter of the rule-syntax page: for a rule list and a request context it evaluates every rule and applies the precedence key literally. The compiled matcher is differentially fuzzed against it on generated rules and contexts and on the corpus request stream (CB-1). No cross-engine conformance suite exists ([ME 7.1][ME]); Izcal builds the reference first (ORC-1), and runs other blockers only black-box, to record where dialects disagree (M-1 part 2, Q-CB-6).

**Dead and shadowed rules.** For user-added lists, the compiler reports duplicates, rules shadowed by broader ones and unreachable option combinations in the list's compile report. For Izcal's lists, entries are pruned by evidence (Section 6.8). Snyder et al.'s hybrid of a synchronous matcher on the useful subset with an asynchronous check of the rest ([ME 8.3][ME]) is rejected: it lets first requests through and makes decisions order-dependent, which DET-1 forbids.

### 6.4 DNS uncloaking and cookie policy (CB-D9, CB-D10)

**Every hop, and addresses.** Phase B matches every name in the CNAME chain, not only the first and last: a tracker can sit at any hop, and the one browser that publishes its rule matches only the first and canonical names (Brave's correction of February 2026, [GO 10.2][GO]); CB-13 counts how many requests only an intermediate name uncloaks. Phase B also matches resolved addresses against **reviewed IP-range entries** for trackers that publishers point A records at, a predicted and observed evasion ([TT 4.4][TT]). `hg-dns` is Izcal's own stub (NW-D8), so the chain costs nothing extra.

**Where phase B is unavailable.** Izcal sees no chain and no address when resolution happens elsewhere: the OS-resolver fallback (NW 6.6), SOCKS with remote resolution, HTTP proxies reached with CONNECT, the Tor route, and VPN clients that resolve remotely. Izcal never resolves outside the container's route to recover the chain, because that would leak the browsing the route hides (NW 6.8's reason for R in every key). The site panel's protection line says that uncloaking is off on this route; CB-13 is scoped to direct routes with Izcal's stub and reports the unavailable share.

**Cookies from cloaked responses (CB-D9).** A response from a host whose CNAME chain or address leads to a site other than T's, and other than where T's own host's chain leads (the comparison WebKit makes, so that a site served from its own CDN is not penalized, [HP 6.8][HP]), may set cookies only with a lifetime capped at **7 days**, Safari's precedent ([TT 4.3, 5.4][TT]); Izcal keeps the figure because it bounds a cloaked identifier to about one week of linkage while a first-party session cookie set by the site's own servers is unaffected. If the canonical name is on a list, the request is blocked in phase B and sets nothing. This is a class (c) intervention; its compatibility cost is measured on the corpus (CB-15). It answers the cookie-leak half of cloaking (95% of cloaking sites leaked first-party cookies, [TT 4.2][TT]) for cloaked endpoints no list names yet.

**Restricted scripts (CB-D10).** Some third-party scripts track and are also needed: the site breaks without them, or they are bot-detection and anti-fraud code that protects the user's account ([TT 2.10][TT]). Lists today can only block or allow them. Izcal adds a third action, available only to Izcal's lists: **restrict**, which lets the script load and run under a typed profile enforced by the engine:
- cookies the script writes through `document.cookie` or Cookie Store get a lifetime of at most 7 days (the same precedent as CB-D9);
- its CPU time and hidden-page wakeups are capped through the broker's CPU budget at FP-D7's thresholds, **without** FP-D7's Reporting API report, because a restrict cap is list policy rather than an outlier intervention, and a page-visible report would turn it into a detector (FP-D7 amendment, Section 9.5);
- with CB-D3's gate passed, the surfaces in PRIV-2's inventory return the values Izcal's uniformity mode returns (Section 6.6).

**What attribution can and cannot bind.** The JavaScript engine spec attributes time and allocation to scripts (JS 6.10) but defines no attribution of API calls. This spec asks it for one (Section 9.5): the **responsible script** of an API call is the attributed script of the innermost non-builtin frame, and code a script creates **inherits its restriction**: code from `eval`, `Function`, string timers, `javascript:` URLs, inline scripts it inserts, and same-origin child documents it writes. The record lives outside the cage, and its cost is inside JS-60's budget. That closes the one-line escapes (CB-27 tests each). It does not close **laundering through first-party code**: a restricted script that calls a first-party helper, such as a consent manager's `setCookie(name, value)`, writes as the first party. Restriction is therefore a mitigation that raises a tracker's cost, not a guarantee; layer S's D-S1 defences still cover what attribution misses, and CB-27 reports the first-party-helper case rather than hiding it. A restricted script can observe its own restriction, and so can its site; the profile is the same for every user of the list set, so what it reveals is the list set (CB-D13).

### 6.5 Navigation defences (CB-D8)

Bounce tracking, UID smuggling and link decoration are invisible to a request matcher: the tracker is a top-level navigation, and the identifier rides in a URL ([TT 3][TT]). Izcal handles them in a navigation layer, `hg-nav-protect`, split by what each part touches. **Parameter stripping and debouncing** are pure functions over the sealed Parameters and Redirectors tables, run in the **partition service** that handles each navigation hop, which already parses the URL and maps the sealed file. **Bounce-tracker decisions** (site IDs, times and deletion orders, never URLs) run in the **container service**. The container service holds every first-party jar of its container (NW-D3), so it takes no new hostile-input parsing; the broker passes it typed navigation events and parses no web content (IPC-R15).

**Bounce tracking: two rules, one component.** NW 6.9 already clears "storage of a site the user never interacted with as a top-level site ... after 7 days†". This spec keeps that rule and adds a faster one, both in `hg-nav-protect`, sharing one interaction record (amendment to NW 6.9, Section 9.5):
1. **Bounce trackers**, anchored to the Privacy CG's **Navigational-Tracking Mitigations** draft at a pinned revision ([Privacy CG](https://privacycg.github.io/nav-tracking-mitigations/)). A site in an extended navigation that redirected (server-side, or client-side within 10 s, the draft's suggested value) and wrote state (cookies, storage, a service-worker registration) in this container without user activation on that site in the last 45 days (the draft's value; WebAuthn assertions count as activation) has its first-party state in this container deleted after a 1 h grace period (the draft's value).
2. **The backstop** (NW 6.9): first-party state of any site never activated as a top-level site is deleted after 7 days†.

**Where Izcal departs from the draft.** The draft is written for browsers that block all third-party cookies and partition third-party cookies and storage. Izcal partitions both by default (NW-D7) but does not block partitioned third-party cookies, so it meets two of the three conditions. A bounce tracker can still use its partitioned third-party jar, but that jar cannot join sites, so Izcal applies the algorithm to first-party state only and records this deviation. A site that holds a **Storage Access grant** under any top-level site in the container is exempt while the grant lives: the user granted it through a prompt (NW-D16), which is the interaction the draft's precondition requires.

**How deletion reaches the state.** The state lives in the storage service and in the bouncer's partition services (NW 6.8), not in the container service. The storage service and partition services send `hg-nav-protect` typed write notifications (site ID, kind, time; no data). Deletion is a capability the broker grants `hg-nav-protect`, scoped to its container (IPC-R2), and executed by the services that hold the state, along the path Clear-Site-Data uses within one container (NW-D18). The deletion time is drawn from a seeded `Entropy` stream within the grace window and aligned to the wake grid, because the draft warns that deletion timing leaks interaction state; every deletion is a recorded B1 input. The **interaction record** is browsing history: it lives in the storage service per container, holds site IDs and activation times only, keeps 45 days, is cleared with browsing history and with a site's data, and is memory-only in private windows (FP-D18's rules).

**Link-decoration stripping.** Izcal's **parameter list** names query parameters that carry cross-site identifiers, each entry with evidence (the parameter recurring with user-specific values across sites on the crawl) and a scope (global, or per destination site). Listed parameters are removed from top-level and subframe navigations before the request leaves (and from subresource requests through `removeparam`), and from links the user copies with "copy clean link". Safari's list is not published ([TT 3.7][TT]); Izcal publishes its own with evidence. Stripping is visible to the destination by design; its compatibility cost is measured (developers reported `window.location` losing parameters under Safari's stripping, Low, [HP 6.9][HP]).

**Debouncing.** For a navigation to a URL matching a **redirector entry**, Izcal navigates straight to the destination carried in a named parameter, decoded once, which must be an `http` or `https` URL and not another redirector entry. Skipping a redirector also skips its checks, so admission is strict: an entry needs black-box evidence, from the crawl with random destinations, that the redirector forwards **unconditionally** to arbitrary parameter values with no interstitial; **link-scanning services and signed redirectors** are a published excluded category, because they exist to check the destination; Safe-Browsing-class checks, once they exist (Q-CB-5), apply to the destination. The debounced navigation carries the **redirector's origin as its referrer**, what the destination would have seen, never the original page's. Dedicated smugglers were 16 of the 30 most common redirectors in 2022, and 41% were missing from Disconnect's list ([TT 3.2][TT]), so the list is built from Izcal's crawl. CB-10 tests each rule.

**Forced redirects.** A cross-origin subframe that navigates the top-level traversable without transient user activation is refused. HTML allows the navigation for a non-sandboxed frame, so this is a deviation, a class (c) intervention whose P1 analysis is security: forced redirects were 81% of malicious ads in 2024 vendor data ([TT 6.3][TT]). The page observes a **silent no-op** plus a console message, so the refusal tells it nothing beyond "Izcal". Transient activation is the **broker's own record**, updated by the input routing the broker performs, never the subframe renderer's claim. Every WPT result that changes with Izcal's default protections on versus off is listed and published per release, as the P2 scope note requires (CB-26). It does not depend on any list.

### 6.6 Script and API level: the intervention vocabulary (CB-D2, CB-D3)

**No list carries code (CB-D2, G81).** Every blocker that let lists run code ended up restricting it by list provenance (uBO's trusted scriptlets, AdGuard's trusted filters, ABP's vetted snippet list), and the one feature left unrestricted, ABP's `$rewrite`, let a list author execute code on Google sites and target chosen IP addresses with short-lived rules ([ME 4.2–4.6][ME], [LG 5.1][LG]). Izcal does not adopt a trust tier. It removes the capability: **no rule kind in any list, Izcal's own included, can inject script, rewrite a response body, remove a header, or name an external redirect target.** What lists do instead is select from a **closed vocabulary of typed interventions**, each written in Rust in the engine (or, for surrogates, as Izcal-authored scripts shipped and signed with the build), with validated parameters, a decision-log record and a plain-language explanation:

| ID | Intervention | Where it runs | Parameters |
|---|---|---|---|
| I1 | Block (network error) | Partition service | — |
| I2 | Redirect to a **neutral resource** built into Izcal: empty script, style, JSON, text and HTML; 1×1 GIF and PNG; silent audio and video of fixed lengths; empty VAST and VMAP | Partition service | Resource name from a closed list |
| I3 | Serve a **surrogate**: an Izcal-written stand-in for a named library's public API (an analytics `ga()` or tag-manager queue that does nothing and sends nothing), so pages that call the library keep working, as Firefox's SmartBlock and Brave's replacements do ([CU 2.1–2.6][CU]) | Partition service (served as a script response) | Surrogate name from the build's catalogue |
| I4 | Strip parameters | Partition service | Parameter names or a linear-time pattern |
| I5 | Restrict a script (cookie-write cap, CPU and wakeup budget, and with CB-D3's gate, uniform fingerprinting surfaces) | Renderer (JS engine, bindings) | Profile ID |
| I6 | Hide (cosmetic, user-origin `display: none !important`) | Renderer (style) | Selector |
| I7 | Paint-suppress an element whose load was blocked | Renderer (paint) | — (generic) |
| I8 | Delete bounce-tracker state | Container service | — |
| I9 | Refuse a forced top-level redirect | Broker | — |
| I10 | Add a restrictive policy header (`csp`, `permissions`) | Partition service | A directive set from Section 6.2's allow-list; no reporting directive, no host |
| I11 | Heavy-frame and heavy-script intervention | Renderer, broker | FP-D7's thresholds |

**Surrogates are engine code, not list data.** They are written by Izcal from the replaced library's **public documentation and black-box observation** of what calling pages expect, never from the library's source; reviewed like engine code; tested black-box against pages that use the library; versioned and signed with the browser build, and listed in the published catalogue with the behaviour each preserves. A list entry names a surrogate; it cannot supply one. Generators that take the replaced script's own code as input are not used (SugarCoat rewrites the original script, [ME 4.9][ME]), because the output would derive from that code; a generator whose only inputs are public API documentation and black-box call traces may be used, with the same review. Every surrogate carries a provenance manifest listing its inputs, and no third-party script text may be one (G87).

**What this gives up.** Scriptlets are how uBO-family lists defeat anti-adblock scripts, prune JSON ad payloads and defuse timers. Izcal's lists do not defeat anti-adblock walls, which German courts treat as the publisher's legitimate response ([CU 5.9][CU]), and Izcal's lists need no JSON pruning because ad endpoints are blocked by evidence. For **user-added lists**, scriptlet, body-rewrite and header-removal rules are counted in the list's compile report as "not supported: runs code" and ignored. The measured share of rules this affects in the common lists is published (CB-2). The reversal trigger in CB-D2 covers breakage that surrogates and exceptions cannot fix.

**Fingerprinting (CB-D3).** After the D-S1 amendments, much of the classical surface is identical by construction for every user of a build on an OS family: 2D canvas readback is canonical (GR-D12), text metrics are exact (TX-D4), Izcal-rendered audio is bit-identical and loses its noise once MD-30 passes (MD-D14), form controls and scrollbars are Izcal constants (LY-D11). What remains is WebGL and WebGPU readback (noised once where pixels leave, GR-D12), the discrete surfaces D-S1 normalizes, and the APIs fingerprinting scripts use beyond canvas and audio (timing, frame-rate estimation, sensors, `canPlayType`, [TT 2.5][TT]), which the surface inventory this spec adds to PRIV-2 must cover (Section 9.5: [TT 2.5][TT] plus D-S1's discrete and readback surfaces). Izcal's answer has three parts:
1. **API level, for everyone** (D-S1 as amended): the primary defence, because it reaches first-party and bundled fingerprinting that no list reaches (49–52% of canvas-fingerprinting sites, [TT 2.8][TT]).
2. **Network level**: the tracking list includes **third-party fingerprinting scripts** classified by behaviour on the crawl (an API-sequence classifier in the lab, the idea FP-Inspector published, [TT 2.3][TT]), each entry with evidence. **Bot-detection and anti-fraud scripts are a published category that is not blocked by default** (AdGuard exempts anti-fraud scripts that would break a site, and Disconnect files them in a separate Anti-fraud category, [LG 1.7, 1.15][LG]), because blocking them raises challenge and denial rates that PRIV-1 measures.
3. **Per-script uniformity, behind a gate.** The I5 profile "uniform surfaces" makes a restricted script see the values of Izcal's opt-in uniformity mode for the surfaces in that inventory, while other scripts on the page see D-S1's defaults. FP-Inspector measured that targeted, per-script restriction broke about half as much as blanket API restriction ([TT 2.4][TT]). It is **built**, and **enabled** when two measurements pass: corpus breakage no higher than the D-S1-only arm, and a PRIV-2 run in which a page cannot tell, across sites, anything about the user beyond the list set. Holding it is a measurement gate, not a stage.

**Cryptominers.** Security-list entries for mining endpoints seen on the crawl, plus FP-D7's CPU and wakeup budgets, which catch throttled, URL-randomized miners without a list ([TT 7.1][TT]). The crawler detects mining by behaviour (sustained Wasm hashing, Stratum framing over WebSocket); the browser runs no detector.

### 6.7 Cosmetic filtering (CB-D12, CB-D13)

**Izcal's lists hide almost nothing by selector.** The generic answer to an ad slot is **I7**: an element whose image, media or frame load was blocked is not painted, with layout unchanged (Section 6.2). It stays hit-testable as any failed load is, except that a click on a suppressed element whose nearest link's target is itself a blocked URL is not followed, so blank space cannot take the user to an ad; its alt text stays in the accessibility tree, as for any failed image, so assistive technology hears what any failed load gives. Paint suppression happens at display-list generation, which the graphics spec owns (amendment, Section 9.5). It needs no rule, fires only where Izcal blocked something, and gives a page probe nothing it could not learn from the failed load. Per-site hide rules exist only where the corpus shows visible residue (text ads served first-party, empty labelled slots), each with evidence. Generic selector sheets, which cost about 4 MiB per document in 2014 Firefox before sharing ([ME 3.1][ME]) and are the easiest rules to fingerprint ([TT 8.5][TT]), are not in Izcal's lists.

**User-added lists' cosmetic rules** compile into per-site subsets and a generic subset, delivered to each site process as one user-origin sheet ≤ 256 KiB† (ST 6.16, FP 6.11), with `display: none !important` at the user origin, which beats author `!important` in the cascade ([Cascade 5](https://www.w3.org/TR/css-cascade-5/#importance)). Selectors are Selectors 4 as `hg-css-syntax` parses them, `:has()` included natively. **Each cosmetic rule compiles to a parsed selector and one fixed action**: the subset is produced by Izcal's serializer from the selector syntax tree, and `hg-cb-format`'s verifier rejects any subset whose re-parse contains a declaration other than `display: none !important` or anything outside selectors, so a rule such as `##div{}a{background:url(…)}` cannot inject CSS that exfiltrates page content (CB-5).

**Procedural operators (CB-D12).** Text and attribute operators have no CSS equivalent, and blockers run them as page-injected script with MutationObservers whose cost no one publishes ([ME 3.4, 3.6][ME]). Izcal implements a **small, pure subset inside `hg-style`**, as user-origin-only pseudo-classes that author sheets cannot use: `:has-text()` (a linear-time pattern over the element's text content), `:matches-attr()`, `:min-text-length()`, with `:upward()` and procedural `:has()` rewritten to native `:has()`. Each is a pure function of DOM state, so the style engine's invalidation (character-data and attribute mutations) covers it, and no script runs. Operators that depend on computed style or layout (`:matches-css`, `:matches-prop`), on XPath, or that act (`:remove`, `:remove-attr`, `:remove-class`, `:style`) are not supported: they either need a second style pass or mutate the page. This is an amendment to the style spec (Section 9.5).

**Hiding without a side channel (CB-D13).** The scriptless list-fingerprinting attacks work through what hiding changes: a hidden element's background image or lazy image is not fetched, so the probe's server learns which rule matched ([TT 8.4, 8.5][TT]). Izcal's rule is the part that is feasible: **user-origin hiding does not change which background images and fonts are fetched.** For an element hidden only by a user-origin hide rule, background-image and font eligibility are computed from its style as if the rule did not apply, a style-only counterfactual for those elements; real ad images under hidden elements are almost always blocked by network rules anyway. Three channels remain and are measured, not claimed closed (CB-8): **lazy images** under or after hidden elements, whose loading depends on layout; **container-query probes** (hiding changes a container's size, which changes a sibling's style); and, with script on, **computed style and geometry**, which any script can read. The defence for all three is **uniformity**: Izcal ships **one default list set**, the same in every named container, with no per-list toggles, so a page that probes Izcal's lists learns "Izcal with defaults"; a user-added list is shown in settings as "makes you more recognizable to sites" with the reason. The cost of the counterfactual is measured by the style owner at CB-M2 (Section 9.5).

**Accessibility and focus.** `display: none` removes the element from the accessibility tree too, so a hidden ad is consistent for screen-reader and sighted users. If the focused element is hidden, HTML's focus fixup rule applies. Leftover side effects of hiding overlays (scroll lock, `inert`, focus traps) belong to the Stage 2 cookie-banner work (CB-D22), where they are checked mechanically after a rule applies (CB-17).

**Legal note.** The BGH's July 2025 remand leaves open whether an ad blocker changing a page's DOM and CSSOM modifies a protected program ([HP 2.8][HP]). Izcal's cosmetic rules are a user-origin sheet that the engine applies as part of the cascade the user agent computes, not a modification of the site's code; Izcal still takes a German legal opinion before shipping in Germany (CB-M4).

### 6.8 Izcal's lists: what they contain and how entries are made (CB-D5, CB-D16, CB-D23, CB-D7)

**The list set.** One bundle, published under **MIT** as data:

| List | Content | Default |
|---|---|---|
| **Security** | Endpoints the crawl observes serving malvertising, forced-redirect chains, scam redirectors and miners | On |
| **Tracking** | Third-party hosts and paths whose behaviour joins users across sites (below); third-party fingerprinting scripts; CNAME and IP entries; first-party request-shape entries | On |
| **Ads** | Ad-serving, auction and ad-measurement endpoints, **each with P1 evidence** (CB-D5) | On (from CB-M3) |
| **Parameters** | Link-decoration parameters | On |
| **Redirectors** | Debounce entries | On |
| **Cosmetic** | Per-site hide rules where the corpus shows residue | On |
| **Restrict** | Scripts loaded under an I5 profile | On |
| **Exceptions** | Breakage exceptions and login-flow entries, each with its reason | On |
| **Anti-fraud** | Bot-detection and anti-fraud scripts, published for transparency, **not blocked** | Informational |

**What counts as tracking.** NW 6.15 defines the tracking list by measured behaviour. This spec makes it operational: a third-party host or path pattern is a tracking candidate when, on **at least 3† distinct sites** of the crawl (Privacy Badger's precedent, [LG 1.19][LG]; Izcal keeps it because two sites can be one operator's pair, and three is the smallest count at which a host joins sites that are not a pair; it is a floor, and a reviewer may require more), it does at least one of: stores or reads an **identifier-like value** in a third-party or cloaked context (persistent across visits, long enough, distinct across fresh profiles and stable within one, the published ID-detection heuristics, [TT 1.4][TT]); receives identifier-like values in URL parameters that recur across sites; takes part in identifier syncing between parties; or runs fingerprinting behaviour attributed to its script. Each candidate's evidence records which heuristic fired. Whether the party ignores `Sec-GPC: 1` is recorded as evidence, never as a criterion ([GO 13][GO]). Known malware and mining endpoints, which NW 6.15 put in the tracking list, are in the Security list here; and NW-54's denominator excludes requests matched by Anti-fraud and Restrict entries, which are reported separately (amendments, Section 9.5).

**The ads list stays a privacy and security list (CB-D5).** HGBA-3's P2 scope note allows default-on interventions only as privacy protections with a recorded P1 analysis. An ads-list entry therefore needs at least one of: the tracking criteria above; malvertising served on the crawl; or an auction or measurement endpoint that receives identifier-like values across sites. **Resource cost alone never qualifies**: bytes and CPU are a footprint concern, and ad frames and scripts above the 99.9th percentile are handled by FP-D7's intervention, which is announced and explained (Chrome's heavy-ad evidence that 0.3% of ads cause 27–28% of ad data and CPU, [HP 9.2][HP]); an entry's measured resource effect appears in its record only as effect evidence. Contextual, non-tracking ads served by the site itself are not blocked. Because no criterion needs FP-D7's calibration corpus, the Ads list ships with the first lists (CB-M3). The P1 analysis is recorded once in the decision log and the list's measured compatibility cost is published per release.

**The crawler.** Izcal itself, in recording mode, in the lab:
- **Looks like a user.** Headed, with realistic profile shapes, from several regions and networks. Headless Chromium was blocked on 15.2% of the Tranco top 10,000 against about 7% headed, correlated with the CDN and not with popularity, and 83% of 81 measurement papers did not report their block rate ([LG 2.2][LG]); Izcal publishes its crawler's block rate per CDN and region every crawl (CB-24).
- **Visits enough.** The Tranco list (list IDs recorded; its own licence is unstated and one input is non-commercial, so a legal reading comes first, Q-CB-2) plus a **Certificate Transparency-derived** per-region sample of live sites ([RFC 6962](https://www.rfc-editor.org/rfc/rfc6962)), which reaches sites no popularity list ranks. Homepage-only crawls undercount (third parties per site rose from 22 to 34 with 4 internal pages, [TT 1.2][TT]), so each site gets its home page and at least 4† internal pages, and a scripted interaction set (scroll, open login, play media, open comments) for breakage. Cadence: top 10,000 weekly†, top 100,000 monthly†, a 1-million-site sample quarterly†, regional samples monthly†.
- **Separates identifiers from session IDs** with several coordinated profiles (fresh and aged, two machines), as CrumbCruncher did ([TT 3.1][TT]).
- **Records both arms.** Every page is recorded with and without each candidate list version, so breakage is compared on engine facts (DOM, semantic tree, display-list hash, network outcomes, console errors, layout of main content) and replayed exactly (Section 6.11).

**The classifier** uses **information-flow features** (who stores which identifier, who sends it where, redirect chains, attributed API sequences), computed natively from the decision log and the engine's attribution. Content features (URL strings) are what trackers control: evasion against a content-feature classifier reached 96.6% with first-party collusion, against 8.3% for a flow-feature one ([TT 9.2, 9.3][TT]). Labels come from Izcal's own reviewed entries, never from EasyList or other third-party lists, whose gaps and licences a classifier would inherit ([TT 9.7][TT]). The classifier runs only in the lab; it proposes, people decide.

**Review and the entry record.** Every new entry, every change and every exception is reviewed by **two people** (a curator and a second reviewer) before publication. The review load is derived, not assumed: candidates per week from the crawl (estimated at CB-M0, measured at CB-M2) × 2 reviews × minutes per review, plus the 12-weekly re-validation of every entry; Section 9.3 costs the second reviewer. An entry is a record, not a line ([LG, design idea 2][LG]):

| Field | Content |
|---|---|
| ID, list, category, action | `trk-004213`, Tracking, analytics, block |
| Rules | The compiled rule text |
| Evidence | Sites observed (count, and a sample of public sites), heuristics that fired, identifier and API classes observed (classes, never values), crawl IDs, first seen, last fired |
| Effect | Bytes, requests, memory-time and wakeups removed across the corpus (FP 6.11, [GO 14][GO]) |
| Breakage | Corpus result for both arms, interaction-inclusive |
| Review | Reviewers, date, linked issues |
| Exceptions | Linked exception entries and their reasons |

The text lists are compiled from the records. Entries are **re-validated every 12 weeks†** (the HSTS list's cadence, NW-D25), and an entry that has not fired on the crawl in that time is reviewed for removal, because most rules in mature lists never fire ([ME 8.1][ME]). Twelve weeks is NW-D25's cadence, kept so one crawler schedule serves both lists and so a host that changes hands is re-checked within a quarter.

**Exceptions are entries.** Each has a reason (a breakage report, a corpus diff), is as narrow as possible, and is published. **Login-flow entries** (for example a federated sign-in provider's endpoints) are exceptions that apply only when the user starts a sign-in from the site panel or the page's sign-in control, for that tab, until it closes or 24 h† pass, the narrow, short-lived shape Brave's sign-in permission uses ([CU 2.8][CU]).

**Entity data (CB-D7).** Izcal builds its own map of which organization operates which domains, from RDAP and WHOIS, certificate subject alternative names, CNAME chains and corporate records, published with evidence like any entry. Each input is listed with its licence and reviewed (corporate-record databases can carry share-alike database licences, Q-CB-1's legal request covers them); only organisation-level facts are stored, never registrant personal data, with provenance per entity. It is used to group and explain ("23 trackers from 9 companies"), never to exempt a same-owner request. Disconnect's, Tracker Radar's and Ghostery's entity data are non-commercially licensed ([LG 1.13–1.16, 1.25][LG]).

**Security list scope (CB-D23).** The security list holds what the crawl observes. Phishing and malware URL reputation (a Safe-Browsing-class service with frequent updates and per-URL checks) is a different system with different privacy trade-offs ([GO 11.2][GO]); it is out of this spec's scope and needs an owner (Q-CB-5).

**Governance and transparency.**
- **No payment, ever**, for any entry, exception or ordering; no "acceptable ads" programme; staff working on the lists may hold no interest in a listed party. Izcal's funding (D-S8) must not include revenue from a party the lists name in a way that could shape them; a search-revenue deal with a listed party would be such a conflict, and is put to the owner (CB-D16).
- **Published criteria**, entry records, a change log with the reason for every change, and a quarterly transparency report: entries added and removed, exceptions, appeals and outcomes, legal demands.
- **Appeals.** A site operator can contest an entry through a public issue or a private form; a named reviewer answers within 7 days†; the decision and its reason are published. **Legal demands** (DMCA notices, court orders) are published with the demand, as GitHub publishes takedowns; a 2017 DMCA notice removed a domain from EasyList ([LG 4.5][LG]).
- **Evidence is factual and correctable.** Each evidence statement is dated, reproducible from the crawl ID and corrected when wrong, because a vendor's published claim about a company is a larger legal target than a volunteer's list line ([LG 8][LG]).
- **Lab use of third-party lists** is test-only (OD-L2) and limited to coverage measurement: GPL, CC BY-SA and MPL lists are allowed; the non-commercial datasets (Disconnect, Tracker Radar, TrackerDB) are not used even in the lab until a legal opinion says how their licence applies to developing a commercial product's lists (Q-CB-1). No third-party list is ever an input to Izcal's entries (G84): coverage results are published only as aggregates (shares of requests and categories), never as host or rule lists, and are not readable by `hg-list-build`'s candidate and review components; candidates come only from the crawl's own heuristics and from user reports.

### 6.9 Delivery and updates (CB-D17)

**One bundle, the same for everyone.** All of Izcal's lists, for every region, are published as **one bundle**: a regional user and a default user fetch the same bytes, so the fetch reveals nothing per user beyond what any update check reveals (NW-D17). Regional lists would split the anonymity set and reveal a choice; the bundle's cost is size, which diffs make small.

**Signed through the update system, verified by the broker.** The bundle and its diffs are **targets in Izcal's TUF-based update repository** (Stage 1 13.2), so they inherit signed metadata, rollback and freeze protection and key rotation. The **system service only fetches bytes**; the **broker's update client** (Stage 1 Section 1) verifies them and hands the sealed source to the utility process. The broker parses no list text (IPC-R15); it checks signatures and hashes only.

**A witnessed transparency log.** Each bundle version's hash, and each diff's result hash, is appended to a public log. A log Izcal operates alone does not stop a targeted list: the operator could show a targeted client a second view in which the targeted bundle is included. So the client accepts a bundle only with a **checkpoint cosigned by k of n independent witnesses** whose keys ship in the TUF root, together with a **consistency proof** from the last checkpoint it accepted; published monitors compare the log with the bytes the CDN serves. A list operator who served a different list to chosen addresses (the attack the 2019 `$rewrite` disclosure described, [LG 5.1][LG]) would have to put it in the one log every client and witness sees. The witness operators are an open question (Q-CB-15) and a non-staff cost.

**Diffs by version.** Clients fetch version-addressed diffs of the source bundle (the approach uBO adopted in 2023 to cut bandwidth, [LG 6.2, 6.3][LG]) with a full sync every 7 days†, and recompile in the utility process. Schedule: one daily† check at a wake-grid time that is the same for every client, never user-specific, deferred on battery saver (NW 6.17). **Security additions** that must reach users faster ride the update system's emergency channel (UPD-R5), never a new poll, and apply even to pages already open (Section 6.14).

**Why not Oblivious HTTP for the lists.** NW-D17 routes requests that carry user-specific data through an OHTTP relay and fetches public files plainly. The bundle is a public file identical for every client, fetched on a schedule identical for every client; OHTTP would hide only the client's IP address from Izcal's CDN, which every update check already reveals, at the cost of relaying megabytes through a third party. Izcal uses the OHTTP channel where it protects something: **breakage reports** (Section 6.11). If the bundle ever had to vary by user (it must not, CB-D13), its fetch would move to OHTTP.

**User-added lists follow the container's route.** A user-added list belongs to a container, so it is fetched by **that container's own network service, through the container's route**, with no cookies and no partition state, at the same daily slot, never by the system service. Fetching it directly from a Tor- or WireGuard-routed container would show the list host the user's real address on a schedule tied to that container's list set, linking the routed identity to the direct one, the linkage NW 6.8 puts R in every key to prevent. The same list in containers on different routes is fetched once per route. The settings page says that the list's host sees the fetch from the container's route. This moves third-party list URLs out of the system service's endpoint list in NW-D17 (amendment, Section 9.5). Hosting of Izcal's bundle is sized for hostile clients: EasyList's traffic once rose 10–20× from an open-source Android browser and its forks re-downloading on every start ([LG 5.6][LG]).

### 6.10 User-added third-party lists (CB-D14)

A user may add lists per container, by `https` URL or from a local file, and write their own rules. Izcal reads these formats, from their **published documentation**, recorded as black-box observations where the documents are silent (M-1 part 2):

| Format | Source of the syntax |
|---|---|
| ABP-family filter lists (EasyList, uBO and AdGuard dialects) | [Adblock Plus filter documentation](https://help.adblockplus.org/hc/en-us/articles/360062733293-How-to-write-filters), [uBO static filter syntax](https://github.com/gorhill/uBlock/wiki/Static-filter-syntax), [AdGuard filter rules](https://adguard.com/kb/general/ad-filtering/create-own-filters/) |
| Hosts files | Their conventional format |
| declarativeNetRequest JSON rulesets | [Chrome DNR reference](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest), [MDN](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/declarativeNetRequest) |
| Safari content-blocker JSON | [WebKit, Introduction to content blockers](https://webkit.org/blog/3476/content-blockers-first-look/) |

**Supported**: everything in Izcal filter syntax (Section 6.2), cosmetic `##` and `#@#` rules, the procedural subset (CB-D12), `!#include` within the same origin, `$popup` as a block of the popup navigation, `$document` exceptions (which, list-scoped, can only disable that list on a site). **Ignored, counted and explained**: scriptlets and snippets (`##+js`, `#%#`, `#$#`), `$replace` and HTML filtering, header removal, `$redirect` to a name not in Izcal's neutral-resource catalogue (served as a plain block instead), `$rewrite`, `$app`, `$cookie`, procedural operators outside the subset, regexes outside the linear-time subset. Every list gets a **compile report** in settings (rules compiled, ignored by reason, rejected by reason, shadowed), and Izcal publishes the same table for the common public lists, measured test-only (CB-2, [GO, question 7][GO]).

User-added lists are untrusted input: they are parsed only by `hg-cb-syntax` in the utility process (G80), fuzzed as a tier-P parser, compiled within budgets, and can neither carry code (G81) nor unblock another list's entries (Section 6.2).

### 6.11 Breakage: prevention, reports and replay (CB-D18)

**Prevent.** No entry ships without its corpus breakage run in both arms, interaction-inclusive, compared on engine facts (Section 6.8). Load-time checks miss most breakage: 25–44% of user-reported breakage appears only after interaction, and CSS-hiding rules cause about 53% of rule-attributed breakage ([CU 1.8][CU]). Breakage categories are weighted by how much users care: pages that do not load, logins, streaming and payments above missing images on news sites ([CU 1.3][CU]). Published breakage predictors (bytes removed, share of sub-document requests blocked, DOM nodes injected by blocked scripts, AUC 0.88, [CU 1.6][CU]) are triage aids, not gates.

**Report.** From the site panel, or when a user turns protection off for a site (the moment users actually act, [CU 3][CU]), Izcal offers a one-click report and shows exactly what it sends:
- **scheme and host** by default; the **path** only as a separate field the user previews and may edit, with high-entropy segments (16 or more characters mixing case and digits, or matching a token pattern) replaced by a placeholder before display, because paths carry document IDs, reset tokens and user IDs; never the query or fragment (Brave strips both and Firefox the query, [CU 4.1, 4.2][CU]);
- the bundle version and an **enumerated settings vector**: protection on or off for the site, the site's overrides, and whether any user-added list is present (a yes or no, never which); Izcal's lists cannot be toggled, so there is no per-list field;
- the IDs of the entries that fired on that load;
- the Izcal version, the date (no time of day) and an optional description.

Optionally, with a second, separate consent, a **redacted recording**. A recording starts at process launch (DT-1), so the flow is "reload this page in recording mode, reproduce, then report", with the DevTools spec's recording and DT-10's redaction. Reports go through the **OHTTP relay** (NW-D17, Q-NW-6); until an independent relay operator exists, they follow NW-D17's accepted fallback, direct to Izcal with a published logging policy, and the report dialog says which path applies. Reports are kept at most 30 days† and then deleted; the public issue keeps only the site, the entries and the resolution. NW 6.15's "only the URL and the rules that fired" changes to the host by default plus the version and settings vector, because triage needs them; Section 9.5 lists the amendment.

**Replay.** Every blocking decision is a recorded input (Section 6.14), so triage is mechanical: replay the recording with the suspect entry removed and diff the outputs; bisect entries automatically in a replay host. Replay is exact until the first request the original load blocked and the modified load needs, whose response was never recorded; past that point the replay is marked "exact until" (DT-1(c)) and triage moves to the lab, which re-crawls the URL with both arms recorded. No blocker reproduces breakage today ([GO 9][GO]).

**Fix and close the loop.** A fix is the narrowest that works: a surrogate, a narrow exception entry, or an entry change, each reviewed and published. The site panel tells the user who reported, or who turned protection off on that site, that a fix shipped in bundle version N and offers to turn protection back on. Median and p90 triage time and report volumes are published per release (CB-16); no vendor publishes these today ([CU 4.6][CU]).

### 6.12 Per-site controls (CB-D19)

The site panel's protection row (shell UI 4.3) offers, per (site, container), from narrowest to widest:
1. **Allow one party here**: from the list of what was blocked, unblock a party on this site: every entry, in every domain, that matched requests to that party's hosts on this page, as one override.
2. **Allow sign-in with a provider, for this tab**: activates the provider's login-flow exception entries until the tab closes or 24 h† pass.
3. **Hiding off here**: cosmetic rules off on this site, network protection unchanged.
4. **Protection off here**: every layer this spec adds, for this site; partitioning (layer P) and D-S1 are never switched off by this control.

Offering the narrow options first answers the "drop shields and forget" pattern ([GO 15.3][GO]). Overrides persist until the user changes them; the site panel badges any site with one; when a fix ships for that site, the panel offers to restore protection. **Levels**: Izcal ships one protection level, the default list set, the same in every named container, and "off" per site; there is no Strict level in Stage 1, because every additional level splits the anonymity set (CB-D13) and Brave retired its Strict fingerprinting mode at under 0.5% use ([GO 7.2][GO]). Izcal's lists cannot be toggled individually, so a report's settings vector has no per-list field. Users who want more add lists. In private windows, overrides set there and bounce-tracking bookkeeping are memory-only and discarded with the window. Every override and setting is engine configuration recorded at the start of a recording (Section 6.14). Engagement-based relaxation, as Edge does, is not used: it is per-user, history-derived state ([HP 10][HP]).

**What the panel counts.** "Blocked here: N parties from M organisations", counting distinct entries and organisations, never requests; Edge documents that request counts fall as protection rises ([CU 7.7][CU]). The number links to the list of parties, each with its entry and evidence.

### 6.13 Observability: why was this blocked (CB-D20)

**The decision log record** (G85), emitted by every phase and intervention into the service's preallocated ring: request ID; partition (T, C, F); initiator frame and attributed script; destination; phase (A, B, C, navigation, renderer); intervention ID; list ID and version; entry ID and rule ID; for phase B, the matched name and its position in the CNAME chain. It joins the networking decision log (NW 6.18), so the "who received what" view shows what was prevented beside what was sent.

**For developers** (`hg-cb-observe`, an IIP `Protections` domain, read capability on the origin, DT-D5's session rules):
- the Network view's status for a blocked request: "Blocked by Izcal Tracking 2026.10.19, entry trk-004213", linking to the entry's public record and its contest page; for phase B, the CNAME hop that matched;
- a **Protections** view: a timeline of everything Izcal changed on the page (blocks, redirects to neutral resources, surrogates, parameter stripping, restricted scripts and what was capped, hides, paint suppression, refused redirects, bounce deletions), each with its entry;
- a **site self-test**: a documented headless run of a URL with a chosen bundle version that prints the decision log, so an operator can check before contesting ([CU 9.2][CU]).

**Agents** (B9, AI on only): agent tabs get the default list set; an agent session may read the `Protections` domain only for origins in its granted set and may not change protection settings or overrides; with AI off none of this exists (B10).

**For people browsing**: the site panel's list of parties prevented, each with its entry and evidence, and the per-site controls. **For the page: nothing** (NW-22). Detail is held locally under FP-D18's attribution-history rules (24 h† of detail, cleared with history, memory-only in private windows) and never sent. Safari shows that a declarative design can keep the list author ignorant of the user's browsing; Izcal keeps that property and adds the explanation, which stays on the device ([GO 2.1][GO]).

### 6.14 Determinism and recording (CB-D21)

**Matching is a pure function** of (request context, sealed file, settings), with no clock, entropy, allocation or hash-map iteration order on the path (G82), and the compiler is a pure function of its inputs (CB-28). A recording records, as B1 inputs, the bundle version and compiled-file hash per container, the **sources of any user-added lists** (encrypted with the recording, since a list's URL content changes later, and redacted from reports), and the settings snapshot at its start and at every change; replay recompiles, recomputes every decision and compares it with the log, so a divergence is found at the first differing decision.

**When a new bundle applies.** A top-level page keeps the bundle version it was loaded with, so an ordinary list update never changes decisions mid-page. Two exceptions: **Security additions** apply to open pages at the next task boundary, as a recorded B1 input, because a long-lived tab (webmail, a dashboard) must not miss a security entry for weeks; and **workers** (shared and service workers outlive pages) use the version current at their start, re-pinned at each client navigation that reaches them. A partition service maps at most two versions at once (Section 6.3).

**Timed decisions** (bounce-tracking deletion times, forced-redirect refusals, restricted-script caps, FP-D7 interventions) are recorded decision points (JS-D15's pattern). The total precedence order (G83) means no decision depends on rule order in memory.

### 6.15 The Stage 2 extension surface (CB-D15), and cookie banners (CB-D22)

Stage 1 has no extensions (D-S3). For Stage 2's Wasmtime extensions, the evidence says the threat is broad read access plus silent updates, and that an ad blocker is the most natural disguise for it ([GO 4][GO]); and that DNR's model is right in being declarative but wrong in coupling list updates to store review, withholding top-level context and rationing explanations ([GO 1][GO]). Izcal's surface has three tiers:
1. **List providers**, with **zero read capability**: an extension supplies rule lists (Izcal filter syntax, the ABP-family subset, DNR JSON) as data, in its package and from a declared update URL fetched like a user-added list (Section 6.9). Izcal compiles and applies them with the same rules (no code, its own trust domain, no reporting directives, budgets). The extension receives no feedback about matches; the user sees Izcal's explanations, which name the extension's list. List updates are data and need no code review, because they cannot carry code (G81).
2. **Typed interventions as data**: cosmetic rules and redirects to Izcal's neutral resources, within the same vocabulary.
3. **Scoped, audited interception** through IIP (NW 6.19) for the rare extension that must read requests: granted per origin set and container, badged, audited, and an update that widens capability asks the user again.

There is no blocking request-interception API equivalent to MV2's `webRequest`. Extension lists count against the container's compiled-size budget (32 MiB†), not a rule count.

**Cookie banners** (Stage 1 Section 6, Stage 2) reuse this pipeline: the crawler finds banners per region, people review proposed rules, entries carry evidence. Handling is **hide-first** with typed cleanup (scroll lock, overlays) checked mechanically after the rule applies (CB-17), never auto-accept, and no clicking through a consent manager in the first version, because clicking records a preference with a system that may itself track ([CU 5.5, 5.6][CU]). A typed "reject through the consent manager" intervention, where hiding leaves a site unusable or a legal regime gives the refusal weight, is a later decision (Q-CB-8).

### 6.16 Security, privacy and consent

| Attacker | Goal | What stops it |
|---|---|---|
| A hostile or compromised list (user-added, extension-supplied, or Izcal's own) | Run code on pages; redirect users to affiliate or phishing URLs; unblock Izcal's security entries; exhaust the compiler | No code-bearing rule kinds (G81); redirects only to built-in resources; `csp=` without reporting directives or hosts; cosmetic rules re-serialized from parsed selectors; trust-domain-scoped exceptions; tier-P parser in the utility process, fuzzed (CB-4); budgets with refusal |
| An attacker on the delivery path, or Izcal's own operator serving a targeted list | Serve one user a different list | TUF signatures; a witness-cosigned transparency log with consistency proofs; one bundle for everyone (CB-22) |
| A hostile page | Learn the user's list set or settings; steer protection state; bypass by inherited-origin frames or alternate channels | Uniform list set; no learned state (CB-9); fetch-independent hiding; paint-only generic hiding; initiator attribution for inherited-origin frames; every channel matched in the network service |
| A compromised renderer | Skip cosmetic rules or restrictions | Only affects its own site's rendering; network matching runs in the partition service, outside the renderer |
| A compromised partition service | Change matching for other sites | It maps a read-only file; it holds only its (T, C) |

**Residual channels**, measured rather than hidden (CB-8): the speed of a blocked request's failure; lazy images and container-query probes of hidden elements; computed style and geometry of hidden elements, readable by script; a restricted script's view of its own restriction. Each reveals at most the list set, which is uniform for default users.

**Out of scope, stated.** Responses a service worker builds itself or serves from Cache Storage are not matched; their network fetches were. Server-side forwarding of data after a first-party request cannot be blocked; Izcal makes the first-party request visible in "who received what" (NW 6.18) and limits what it carries (layers P and V).

**Blocked top-level navigations.** A top-level navigation blocked by a Security entry shows Izcal's interstitial in the shell's warning design (UI-9), naming the entry and its evidence; "continue anyway" sets a per-site override.

**Privacy.** No default-on telemetry (NW 6.19; FP-D9 allows only opt-in aggregates under a privacy review). Breakage reports are per-report opt-in, show their payload, and go through OHTTP. Evidence is gathered only by Izcal's crawler on public pages, never from users. Opt-in aggregate rule statistics through a distributed aggregation protocol would help curators find dead rules and common overrides ([GO 11.3–11.6][GO]); they need a P1 analysis and conflict with "no telemetry" unless opt-in and published, and the protocol is still a draft, so they are an open question (Q-CB-9), not a design element.

### 6.17 Crate layout and rules

| Layer | Crate | Role | Tier |
|---|---|---|---|
| L5 | `hg-content-block` (NW) | The matcher: phases A, B, C over the sealed file; decision-log records | P |
| L5 | `hg-cb-syntax` | The only parser of list text: Izcal filter syntax, the ABP-family subset, hosts files, DNR JSON, Safari JSON; the procedural cosmetic subset | P |
| L5 | `hg-cb-compile` | Compiler to the sealed file: host table, token index, pattern programs, linear-time regex automata, site sets, cosmetic subsets, intervention table, entry metadata; budgets; compile reports | P |
| L2s | `hg-cb-format` | The sealed file format: structural verifier and bounds-checked readers, linked by network services and by renderers (`hg-style` links its reader for the procedural automata, G24) | P |
| L5 | `hg-cb-resources` | Neutral resources and the surrogate catalogue with per-surrogate provenance manifests (G87), served by partition services | P |
| L5 | `hg-nav-protect` | Parameter stripping and debouncing as pure functions over the sealed tables, in the partition service of each navigation hop; bounce-tracker decisions (site IDs, times, deletion orders, no URLs) in the container service | P |
| L5 | `hg-cb-observe` | Decision-log schema for protections; the IIP `Protections` domain | P |
| L6 | `hg-cb-compile-main` | The utility-process binary for compilation | P |
| tools | `hg-cb-ref` | Reference matcher (oracle) and differential fuzz targets | T |
| tools | `hg-list-build` (NW) | Crawl orchestration, flow classifier, review queue, entry records, breakage comparison, replay triage, publication, transparency report | T |
| tools | `hg-cb-export` | Emits the Safari content-blocker JSON subset of a bundle for the protected-media host (Stage 1 13.1a), which can only apply rules through the host's own API | T |

**Rules**, starting at G80 as the owner directed (the registry's next free number was G73; G73–G79 stay unassigned and are recorded so in Stage 1 2.2, Section 9.5):

| Rule | Statement | Why |
|---|---|---|
| **G80** | Only `hg-cb-syntax` parses filter-list text, and it runs only in the strictest-tier utility process; the broker and network services receive only sealed files, and renderers receive the sealed cosmetic subset and parse only the serialized CSS the verifier accepted; an `ast-grep` check rejects other list parsers | IPC-R15; one hostile-input parser |
| **G81** | No compiled rule or intervention can hold script text, a body rewrite, a header removal, an external URL, a reporting directive or a host source; cosmetic rules are a selector plus the one fixed declaration; `hg-cb-format`'s verifier rejects anything else, and the format has no type that could carry it | CB-D2 |
| **G82** | `hg-content-block`'s match path reads no clock or entropy, allocates nothing, iterates no hash map and cannot panic; an `ast-grep` rule flags allocation, `HashMap` iteration and `unwrap` in its match module | FP-28, DET-1, tier P |
| **G83** | The precedence order of Section 6.2 is defined once, in `hg-cb-format`, and every consumer (matcher, reference matcher, compiler reports, explanations) uses it | Replay; explanations that match decisions |
| **G84** | No third-party list text enters a shipped artifact or an input of Izcal's list build, except the coverage job, marked test-only, whose results leave it only as aggregates that the candidate and review components cannot read; `hg-list-build` records the provenance of every input and fails on an unmarked third-party source | M-1 licence rule; OD-L5; NW 6.15 "never as a source of entries" |
| **G85** | Every decision of every phase and intervention emits a decision-log record with (list, version, entry, rule), into a preallocated ring | Explanations; replay comparison |
| **G86** | No shipped crate may derive blocking state from browsing; protection is a function of the bundle version and explicit settings, plus the bounce-tracking state CB-D8 defines; enforced by design review and by CB-9's attack suite | ITP and Privacy Badger lessons |
| **G87** | Every surrogate and neutral resource in `hg-cb-resources` has a provenance manifest listing its inputs (public documentation, recorded black-box traces); no third-party script text may be an input; checked in CI like G84 | M-1 part 3; CB-D2 |

### 6.18 Alternatives and interfaces for later stages

Each decision's options, cost and reversal trigger are in Section 9.4. **Rejected outright**: any third-party blocking engine (Brave's `adblock-rust`, uBlock Origin's code, Ghostery's engine, Safari's compiler) under M-1 and the owner's direction; shipping or deriving entries from EasyList, EasyPrivacy, uBO, AdGuard, Disconnect, Tracker Radar or any other third-party list (licences and OD-L5); DNS-only blocking (no path, frame or partition); building blocking on an extension API (Stage 1 Section 6); paid allowlisting of any kind.

**Interfaces for later stages.** Android (S-8) uses the same placement. The **embedding API** (Stage 2) exposes lists as data, like tier 1 extensions. The **protected-media host** (D-S7, Stage 1 13.1a) runs the OS's own web engine. On macOS it blocks through `WKContentRuleList`, fed by `hg-cb-export`; Izcal classifies that API as an **OS interface** (the host is the OS's engine, and Izcal's rules are its input), a classification put to the owner (CB-D24). On Windows, WebView2's request-interception callbacks are routed to a partition service for the page's (T, C), which runs `hg-content-block` (never the broker), at the cost of an IPC round trip per request. What the host's rule subset cannot express (restrict, phase B, `removeparam` on every channel) is published, and the label already says Izcal's protections apply only where the host allows. iOS is out of scope (Stage 1 13.2).

## 7. M-1 compliance

**Part 1, original and MIT.** HGBA-2 2.3.2: everything Izcal ships, code and data, must be available under MIT; anything else Izcal builds; another engine's or blocker's code is never copied, ported or translated.

**Dependencies used, unmodified:** none beyond what the networking, style and JavaScript specs already list (Rust std and their MIT-available generic crates). No blocking engine, filter-list library, regex engine for filter rules or tracker dataset is a dependency.

**Built new by Izcal:** the matcher's semantics and phases, the compiler and the sealed format, the reference matcher, the parsers of the published list formats, the linear-time regex subset, the cosmetic compiler and the procedural subset in `hg-style`, the navigation layer, the intervention vocabulary, the neutral resources and every surrogate, the crawler's classification and review tooling, the entry records, the delivery path on Izcal's TUF repository and the transparency log, the decision-log schema and the explanation surfaces, and the Stage 2 extension tiers.

**Data Izcal ships, all its own, published under MIT:** the list bundle (Security, Tracking, Ads, Parameters, Redirectors, Cosmetic, Restrict, Exceptions, Anti-fraud), the entry records and their evidence, the entity data, the neutral resources and the surrogate catalogue. **Allowed as standards data** (HGBA-2 2.3.2 M-1 part 1): the Public Suffix List (site computation), the CT log list (via the networking spec). **Not shipped and never an input to Izcal's entries** (G84): EasyList, EasyPrivacy (GPL-3.0+ or CC BY-SA 3.0+), uBO's and AdGuard's lists (GPL-3.0), Brave's lists (MPL-2.0), Disconnect, DuckDuckGo Tracker Radar and Ghostery TrackerDB (CC BY-NC-SA 4.0), Peter Lowe's list and the hosts aggregations ([LG, licence summary][LG]).

**Test-only inputs** (M-1 part 1; OD-L2): third-party lists for coverage measurement only (the GPL, CC BY-SA and MPL lists; the non-commercial datasets only after Q-CB-1's legal opinion); other browsers and blockers as black-box subjects for the dialect comparison (Q-CB-6) and for the PRIV-2 probe suites; the published research corpora (the "Double-Edged Shield" probes, the IMC 2025 test canvases, Local Frames cases) as test inputs.

**Part 2, spec-anchored.** Each part and its anchor:

| Part | Anchored to |
|---|---|
| Request context, destinations, network error | [Fetch](https://fetch.spec.whatwg.org/) (request destination, initiator, client; "network error"), at the networking spec's pinned commit |
| URLs and hosts | [URL Standard](https://url.spec.whatwg.org/) through `hg-url`; [Public Suffix List](https://publicsuffix.org/) for sites |
| Navigation defences | [HTML navigation](https://html.spec.whatwg.org/multipage/browsing-the-web.html) and user activation; the Privacy CG's [Navigational-Tracking Mitigations](https://privacycg.github.io/nav-tracking-mitigations/) draft at a pinned revision |
| Cosmetic rules | [CSS Cascade 5](https://www.w3.org/TR/css-cascade-5/) (user origin, importance), [Selectors 4](https://www.w3.org/TR/selectors-4/) |
| Policy-adding actions | [CSP 3](https://www.w3.org/TR/CSP3/) (multiple policies only restrict), [Permissions Policy](https://www.w3.org/TR/permissions-policy-1/) |
| DNS uncloaking | [RFC 1034](https://www.rfc-editor.org/rfc/rfc1034) (CNAME) through `hg-dns` |
| Delivery | Stage 1 13.2's TUF-based update client; [RFC 9458](https://www.rfc-editor.org/rfc/rfc9458) (OHTTP) for reports |
| Crawl sample | [RFC 6962](https://www.rfc-editor.org/rfc/rfc6962) (Certificate Transparency) |
| Filter syntax | No standard governs it. The published documentation of the ABP, uBO and AdGuard dialects, the DNR reference and WebKit's content-blocker post are **interoperability evidence** (M-1 part 2); where they are silent (tokenization beyond ASCII, case folding, normalization, `domain=` scope, tie order, [ME 1.16][ME]) Izcal's rule-syntax page decides and records the observed behaviour of other blockers as black-box observations (Q-CB-6) |

**OS interface, for the owner (CB-D24):** `WKContentRuleList` in the protected-media host on macOS, used as the input API of the OS's own engine, never as Izcal's blocker.

**Entity data inputs:** RDAP and WHOIS, certificate subject alternative names, CNAME chains and corporate records, each with its licence recorded and reviewed (Q-CB-1); only organisation-level facts are stored.

**Part 3, evidence.** Every claim in this spec and its notes rests on published prose (papers, documentation and wiki pages that describe behaviour, blog posts, release notes, court summaries, issue discussion) or black-box observation. No note links to or cites the source code of any engine or blocker; several researchers declined search results that pointed at source snapshots ([ME, header][ME], [LG, method note][LG]). Values only source states (Chrome's global static-rule pool, Safari's compiler internals) are questions for black-box measurement, and no design here depends on them ([HP 17][HP], [ME 10][ME], [GO, question 10][GO]). Surrogates are written from the replaced libraries' public documentation and black-box observation, never from their source, and no generator that takes their code as input is used (G87).

**Part 4, oracles.** `hg-cb-ref` decides the matcher's correctness (CB-1); the crawler's paired recordings decide breakage (CB-15); PRIV-2's suites decide privacy (CB-8, CB-9); WPT decides that cosmetic rules are WPT-neutral (Stage 1 Section 6). Other blockers are differential signals only: a disagreement is resolved against Izcal's rule-syntax page, never by matching their output.

## 8. Targets

Thresholds marked † are set by this document from first principles, not measured. Values taken from another product or draft (7-day cookie cap, 3-site floor, the Privacy CG draft's 10 s, 45 days and 1 h) are marked as precedents in the text, with the reason Izcal keeps each.

### 8.1 Correctness and determinism

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CB-1** | Compiled matcher equals the reference | **0** decision differences between `hg-content-block` and `hg-cb-ref` over 10⁹† generated (rule set, context) cases per week and the full corpus request stream, for the default bundle and for each common public list (test-only), including trust-domain cases (an Izcal exception unblocks an Izcal Tracking entry; a user-added exception cannot unblock any Izcal entry; a user rule can) | T2 differential fuzzing | Ship blocker for the matcher change |
| **CB-2** | User-list compatibility is published | For EasyList, EasyPrivacy, uBO's filters and AdGuard's base list (test-only): share of rules compiled, ignored and rejected, by rule family and reason, published per release; **≥ 99%†** of rules in the interoperable network core (Section 6.2) compile | Coverage job (G84) | Fix the parser or record the gap on the rule-syntax page |
| **CB-3** | Decisions replay exactly | **1,000 of 1,000** replays of recorded corpus loads reproduce every blocking, stripping, restricting and navigation decision, with the decision log byte-identical, including recordings with user-added lists | DET-1 harness | Ship blocker |
| **CB-28** | The compiler is a pure function | **1,000 of 1,000** rebuilds of the same inputs give byte-identical files on macOS, Linux and Windows | T1 | Ship blocker for the compiler change |

### 8.2 Security

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CB-4** | The compiler survives hostile lists | **0** open crashes, hangs or budget overruns older than 14 days† from structure-aware fuzzing of `hg-cb-syntax` and `hg-cb-compile`; every budget tested at limit − 1, limit and above | T2 fuzzing | Ship blocker for the parser |
| **CB-5** | No list carries code | **0** compiled files containing script text, body rewrites, header removals, external redirect targets, reporting directives, host sources or any CSS beyond selectors with the fixed declaration (G81); a planted-rule suite covering 100% of the code-bearing families in [ME 1][ME], CSS-injection rules and `csp=` with `report-uri` or `report-to` is ignored or rejected | Format verifier; planted suite | Ship blocker |
| **CB-6** | No bypass by context | **100%** of a published bypass suite blocked: inherited-origin frames (Local Frames cases), popups, service-worker `fetch()`, WebSocket, WebTransport, beacons, prefetch, CNAME at every hop, A-record cloaking with listed ranges, user-added exceptions against any Izcal entry; no list can cause a request to a host the page did not request; for the protected-media host, what its rule subset cannot express is published | T1 suite | Ship blocker for the failing channel |
| **CB-7** | Security headers never weakened | **0** rules in any list can remove or weaken CSP, framing, CORS or HSTS headers; policy-adding actions only restrict and carry no reporting directive, checked by a property test over generated policies | T1 property test | Ship blocker |
| **CB-27** | Restriction resists the one-line escapes | **0** bypasses of the restrict profile's cookie cap through `eval`, `Function`, string timers, `javascript:` URLs, inserted inline scripts and same-origin child documents; the first-party-helper case reported per release, not hidden | T1 adversarial suite | Ship blocker for the restrict action |

### 8.3 Privacy and protection

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CB-8** | The list set is not a fingerprint beyond the default | PRIV-2 run of the "Double-Edged Shield" scriptless probes, blocked-endpoint probes, lazy-image, container-query and computed-style probes, and an IntersectionObserver v2 probe of paint suppression, against default containers: a classifier over probe results cannot separate two default profiles above chance at α = 0.01 over ≥ 1,000† runs per pair; with a user-added list, the residual bits are measured and the settings label shown whenever they exceed 0 | PRIV-2 harness | Above chance for defaults: fix the channel before shipping |
| **CB-9** | No learned or steerable state | The ITP and Privacy Badger attack classes ([HP 4, 6][HP]) and attempts to read per-site overrides or bounce-tracking state cross-site: **0** successes | PRIV-2 suite | Ship blocker (G86) |
| **CB-10** | Navigation defences work | On a CrumbCruncher-style crawl of decorated links and redirect chains: **≥ 90%†** of smuggled identifiers either stripped or not persisted at the destination. In the test suite: **100%** of bounce-tracker state deleted within the grace window; sites with Storage Access grants exempt; private-window state discarded; **100%** of forced top-level redirects without broker-recorded activation refused with no page-visible difference beyond the console; debouncing never admitted for a redirector that conditions on its parameter, never for an excluded category, and always with the redirector's origin as referrer | Crawl; T1 suite | Lists and navigation track |
| **CB-11** | Cross-site identifier flows are prevented | On a held-out crawl, against an **independent oracle** (a hand-labelled site set labelled by people who do not curate, plus published research detectors run black-box, agreement reported): **≥ 95%†** of third-party identifier flows prevented by some layer (P, V, N, D), reported per layer; NW-54 as amended (Section 9.5) | Held-out crawl | Lists track; per-layer gap reported |
| **CB-12** | Fingerprinting is neutralized | On the held-out crawl, against the same kind of independent oracle: **≥ 90%†** of fingerprinting scripts are blocked, restricted or see only D-S1's normalized or canonical values on every surface in PRIV-2's inventory (Section 9.5); first- and third-party reported separately | Held-out crawl with attribution | D-S1 owners and lists track |
| **CB-13** | Uncloaking is complete where it can run | On direct routes with Izcal's stub: **100%** of test zones with tracker CNAMEs at any hop, and A records into listed ranges, are matched; on the corpus, requests uncloaked only by an intermediate name, and the share of traffic on routes where phase B is unavailable, are published | T1 suite; corpus | Matcher fix |
| **CB-14** | Protection is measured and published | Per release: corpus bytes, requests, memory-time and energy removed by the default bundle, with and without (FP-28's per-site savings); PRIV-1's challenge rate with protection on and off from the same IP and session, per challenge provider | Lab A/B; PRIV-1 harness | Publication blocker for the release notes |

### 8.4 Breakage and user experience

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CB-15** | Little breakage, measured honestly | Corpus pages with user-visible breakage from the default bundle **≤ 1%†** (NW-54), where breakage includes load-time, interaction-triggered and cosmetic cases, weighted toward page-load, login, media and payment failures | Paired crawl arms with interaction scripts | Breakage first: narrow the entries |
| **CB-16** | Reports are answered | **≤ 7 days†** to triage for 95%† of reports; median and p90 triage time, report volume and share not caused by Izcal published per release | Triage service | Lists track staffing review |
| **CB-17** | Hiding leaves nothing behind | Stage 1, for Izcal's per-site hide rules: **0** focus trapped in hidden content and **0** main content made inert on the corpus; Stage 2 extends it to banner rules (no scroll lock or overlay left) | Engine-level checks after rule application | Rule fix |
| **CB-20** | The panel is honest | The site panel counts distinct entries and organisations, never requests; in a comprehension study before Stage 1 exit, **≥ 80%†** of participants answer both questions (what the count means; what "off for this site" does) correctly | UI review; study | Wording fix |
| **CB-26** | WPT cost is published | Every WPT subtest whose result changes with Izcal's default protections on versus off is listed per release, with the intervention responsible (the P2 scope note's requirement) | WPT runs in both arms | Publication blocker |

### 8.5 Performance and footprint

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CB-18** | FP-28, per service | p99 match time **≤ 20 µs†** per request, all phases summed, on the corpus request stream (default bundle, and with a 300,000-rule user list) on every reference device; cold start: p99 **≤ 250 µs†** over the first 50 requests of a freshly bound partition service after the file has left the page cache, on DC-4; **0** allocations on the match path; charged matcher state **≤ 256 KiB†** per partition service, including page-table cost of the mapping on each OS and two mapped versions; compiled default bundle **≤ 8 MiB†** (clean, shared; FP-28 as amended); per-site cosmetic subset **≤ 256 KiB†**; decision-log cost included | Footprint ledger; CS-PF-3 | Matcher redesign (FP-28's rule) |
| **CB-23** | Compiles are bounded | Default bundle **≤ 2 s†** on a DC-4 reference device; **≤ 10 s†** per 300,000 rules of user lists; compiler peak **≤ 256 MiB†**; no list update recompiles while a page loads on battery saver | Lab | Compiler fix; never truncate |

### 8.6 Observability and list operation

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CB-19** | Every decision is explainable | **100%** of blocked, redirected, stripped, restricted, hidden and refused events in DevTools resolve to (list, version, entry, rule, published record URL); **100%** of entries have a public record | DevTools and publication checks | Ship blocker for the surface |
| **CB-21** | Entries are evidenced and current | **100%** of entries have evidence, two reviewers and a breakage run; **100%** re-validated within 12 weeks†; entries not fired in 12 weeks† reviewed; **0** entries whose first evidence predates their first crawl observation | `hg-list-build` reports | Lists track |
| **CB-22** | One list for everyone, provably | **100%** of clients refuse a bundle or diff without a checkpoint cosigned by k of n witnesses and a consistency proof from their last accepted checkpoint, tested with a planted split view; a multi-region lab check of one hash per version as a smoke test | Client tests; lab | Ship blocker for the bundle |
| **CB-24** | The crawl reports its own bias | The crawler's block rate per CDN and per region published with every crawl; headed and headless rates compared at each major crawler change | Crawl reports | Crawler fix |
| **CB-25** | Governance is public | Quarterly transparency report (entries added and removed, exceptions, appeals with outcomes and times, legal demands); **0** entries or exceptions with any payment or contractual origin, shown by an annual signed attestation from list staff and funders and an external review of exceptions | Report; attestation; review | Owner review |

## 9. Milestones and staffing

### 9.1 What is ordered or held, and why

Staging is allowed only for correctness, security or measurement:

| Item | Reason | Kind |
|---|---|---|
| The rule-syntax page, sealed format, precedence order and `hg-cb-ref` come before the compiled matcher | Oracles before code (ORC-1); the order cannot be retrofitted without changing decisions | Correctness |
| The governance charter (criteria, two-person review, appeals, no payment, transparency) comes before the first entry | Entries made without it would have to be re-reviewed | Correctness |
| Bundle delivery to users waits for the witnessed log (k witnesses in post) | Without witnesses a targeted list cannot be detected (CB-22) | Security |
| Per-script uniformity (CB-D3) is enabled when its two measurements pass | Breakage and detectability must be shown first | Measurement gate |
| Shipping in Germany waits for a German legal opinion on cosmetic filtering after the BGH's 2025 remand | An unknown legal exposure; neither correctness, security nor measurement, so put to the owner (CB-D16) | Owner decision |
| Extensions (CB-D15) and cookie banners (CB-D22) are Stage 2 | Stage 1 D-S3 and Section 6 | Accepted stage decision |

### 9.2 Milestones

| ID | Milestone | Exit criterion | With |
|---|---|---|---|
| **CB-M0** | Foundations | Rule-syntax page; sealed format and verifier; precedence order and trust domains (G83); `hg-cb-ref`; decision-log schema; intervention vocabulary and surrogate catalogue design with provenance manifests (G87); crawler design and the review-load estimate; governance charter; witness recruitment started; the generic cosmetic budget set from the measured common lists; the JS attribution amendment agreed with the JS owner; PRIV-2 additions designed (CB-8, CB-9); legal opinions requested (Q-CB-1 to Q-CB-3) | NW-M0 |
| **CB-M1** | Matching | Phases A–C in partition services; compiler in the utility process with budgets (CB-23, CB-28); differential fuzzing green (CB-1); user-added lists with compile reports and the coverage table (CB-2); DevTools reasons (CB-19); crawler running headed from several regions, publishing its block rate (CB-24); first evidence records | NW-M2 |
| **CB-M2** | Layers | Flow classifier and two-person review queue; navigation defences with the NW 6.9 backstop (CB-10); restrict profiles with creator inheritance (CB-27); cosmetic subset with the procedural subset and the background-and-font counterfactual, its cost measured; paint suppression (graphics); first surrogates; replay triage; review and triage throughput measured and staffing re-estimated | NW-M3 |
| **CB-M3** | First lists | Security, Tracking, Ads, Parameters and Redirectors lists published with records (NW-54, CB-21); bundle delivery on TUF with the witnessed log (CB-22); site-panel controls; reports through OHTTP or NW-D17's fallback; PRIV-1 arm (CB-14); CB-8, CB-9 and CB-26 green | NW-M4 |
| **CB-M4** | Stage 1 exit | Every Stage 1 target met; CB-D3's gate evaluated on measurement; per-release publication (CB-14, CB-16, CB-25, CB-26); German legal opinion received before any German release | NW-M5 |
| **CB-M5** | Stage 2 | Extension tiers (CB-D15); cookie banners (CB-D22) with CB-17's Stage 2 scope; opt-in aggregate statistics if Q-CB-9 is decided | Stage 2 |

NW-D26 puts a useful first tracking list about 1 year† after the lists track starts; CB-M3 at NW-M4 (months 30–36†) leaves room for the crawler and review queue to run first.

### 9.3 Staffing and duration

The networking spec already staffs two tracks this spec relies on, and they are **not** counted again here: **Content blocking**, 2–3† (`hg-content-block`, rule syntax, automaton and cosmetic compilers), and the content-blocking share of **Lists**, 3–6† of its 4–7† (crawl, classifier, first review, cosmetic rules, breakage triage, evasion response) ([NW 9.3][NW]). So 5–9† of the work is NW's. The reconciliation counts the matcher and the lists only in NW ([RN] P24 and Section 7). This spec adds work NW's tracks do not cover; each row is a first-principles estimate† from its deliverables, replaced at CB-M2 by measured throughput (entries reviewed per reviewer-week, reports triaged per triager-week):

| Work this spec adds | Scope | Size driver | People† |
|---|---|---|---|
| Navigation defences | `hg-nav-protect` in both services, write notifications and the deletion capability, the interaction record, stripping and debouncing, forced-redirect refusal, tracking the Privacy CG draft | One draft specification, three mechanisms, container-scoped state and timers | 1–1.5 |
| Renderer interventions | Restrict profiles and creator inheritance in the JS bindings (with the JS track), paint suppression (with the graphics track), the background-and-font counterfactual and the procedural subset in `hg-style` (with the style track) | Four mechanisms across JS, style and paint | 1–1.5 |
| Surrogates, importers and export | The surrogate catalogue with provenance manifests, black-box tests and maintenance; the DNR JSON and Safari JSON importers; `hg-cb-export` and the WebView2 routing for the protected-media host | uBO's surrogate catalogue is 14–27 scripts ([CU 2.6][CU], [ME 4.9][ME]); Firefox publishes no SmartBlock count | 0.5–1 |
| Breakage replay and measurement | Paired-arm comparison on engine facts, interaction scripts, replay bisection, the CB-8 to CB-15 and CB-26 harnesses, the independent labelled set for CB-11 and CB-12 | Corpus scale; interaction coverage | 0.5–1 |
| User-facing surfaces | Site-panel controls and counts, the report flow, settings and compile reports, the DevTools Protections view (with the shell and DevTools tracks) | Three surfaces | 0.5–1 |
| Second review | The second reviewer for every entry, change and exception, beyond NW's single review | Candidate rate × 2 reviews × minutes per review, plus 12-weekly re-validation, estimated at CB-M0 | 0.5–1.5 (not engineers) |
| Log, witnesses and entity data | Operating the transparency log and coordinating witnesses; the entity-data pipeline (RDAP, certificates, CNAME chains, corporate records) with its licence review | Two operated services; tens of thousands of domains | 0.5–1 |
| Governance | Appeals reviewer, policy and legal liaison, transparency reports and the annual attestation | Appeals volume; legal demands | 0.5–1 (not engineers) |
| **Total added** | | | **5–9.5†** |

**Total content blocking: about 10–19 people†** (NW's 5–9† plus 5–9.5†), of whom about 2.5–5.5† are curators, reviewers, triagers and policy staff rather than engineers (half of NW's 3–6† lists share, the second reviewers and governance). **Overlap with NW's list staffing: yes, by design**; this table adds only work NW does not list, and the format importers are counted here, not moved into NW's track. **Duration**: aligned with NW-M0 to NW-M5 (36–48 months† to Stage 1 exit); the lists, surrogates, log and governance work does not end at parity. **Person-years** for the added work: 5–9.5† a year at full strength; about **12–33†** to Stage 1 exit with a one-year linear ramp ((0.5 + 2 to 3 years) × 5–9.5), 15–38† at full strength from the start. **Non-staff costs** for D-S8: crawler compute and egress in several regions, bundle hosting sized for several terabytes a day at scale ([LG 7.4][LG]), the OHTTP relay (shared with NW-D17), the witnesses, and legal opinions (Q-CB-1 to Q-CB-3).

### 9.4 Decisions for the owner

**Owner decision (19 October 2026): every recommendation below is accepted.**

A bare "yes" means the recommendation. Build-versus-reuse is not on this list: the standing answer is to build Izcal's own (M-1, D-S10), and the owner's direction excludes every third-party blocking engine and list. **The owner most needs to see the starred rows**: CB-D2 (no list code, which costs user-list compatibility), CB-D3 (fingerprinting at the API, per-script uniformity gated), CB-D5 (the Ads list on by default under the P2 scope note), CB-D8 (adds a bounce-tracking rule to NW 6.9's accepted one), CB-D10 (a new "restrict" action, which needs API-call attribution from the JS engine), CB-D13 (one default list set in every named container, narrowing Stage 1's per-container block-list policy), CB-D15 (Stage 2 has no blocking interception API), CB-D16 (governance, including the funding conflict and the German release gate), CB-D17 (how this spec reads NW-D17, and user lists through the container's route), CB-D18 (changes NW 6.15's report contents), CB-D22 (changes Stage 1's cookie-banner row) and CB-D24 (an OS-interface classification).

| ID | Decision | Options | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|---|
| **CB-D1** | Protection model | (a) six layers, each threat assigned a primary layer with its own target (6.1); (b) list-centred, as most blockers | **(a)**: the list cannot reach half of fingerprinting, most first-party tracking or navigational tracking | Per-layer targets and measurement | A threat class whose primary layer misses its target for two releases while another layer meets it: reassign the primary layer |
| **★ CB-D2** | List-supplied code | (a) no rule kind in any list can carry code, rewrite bodies, remove headers, name external redirects or report anywhere; a closed, typed intervention vocabulary in the engine; surrogates are Izcal-written build artifacts with provenance manifests; (b) as (a), plus typed native "property interventions" (scriptlet equivalents written in Rust) for Izcal's lists only; (c) a trust tier as uBO and AdGuard have | **(a)**: removes the capability that produced the 2019 code-execution CVEs, instead of patching it with a tier | User-added lists lose scriptlets, body rewrites and header removal (published share, CB-2); Izcal does not defeat anti-adblock walls | Corpus breakage that surrogates and narrow exceptions cannot fix: (b) for Izcal's lists only, owner decision with the cases |
| **★ CB-D3** | Fingerprinting | (a) API level for everyone (D-S1 as amended) is primary; third-party fingerprinting scripts on the tracking list by behaviour; bot-detection and anti-fraud published but not blocked; per-script uniformity built and enabled when breakage ≤ the D-S1-only arm and PRIV-2 shows nothing beyond the list set; (b) as (a), per-script uniformity on at once; (c) list blocking only | **(a)** | A restrict profile in the bindings; two measurements | PRIV-2 (a) fails with lists on: move the responsible surface to normalization (PRIV-2's own rule) |
| **CB-D4** | Learning | (a) none on the device; Izcal learns on its crawler and ships one bundle; protection depends only on the bundle version and explicit settings (G86); (b) opt-in local learning, as Privacy Badger keeps | **(a)**: learned state was steerable and readable in both projects that tried it | Slower reaction than a local learner to a new tracker; offset by crawl cadence | A design for local learning that CB-9's attack classes cannot read or steer, shown in PRIV-2: put (b) to the owner |
| **★ CB-D5** | The Ads list | (a) on by default from CB-M3, every entry with privacy or security evidence (tracking, malvertising, or identifier-receiving auction and measurement endpoints), P1 analysis recorded per HGBA-3's P2 scope note; resource cost alone never qualifies (FP-D7 handles it); contextual first-party ads not blocked; (b) Tracking and Security on, Ads opt-in; (c) every ad-serving endpoint, by taste | **(a)**: keeps the list a class (c) privacy and security protection; malvertising arrives through the ad chain ([TT 6][TT]) | Advertiser and publisher objection; German legal exposure (CB-D16) | Any Ads entry found whose only evidence is resource cost: remove it |
| **CB-D6** | First-party scope | (a) third-party by default, first-party only for uncloaked trackers, evidenced request shapes and security entries; (b) first-party paths too (Brave Aggressive); (c) third-party only | **(a)** | Some first-party tracking passes; layers P and S cover part of it | Request-shape entries' measured breakage above CB-15: narrow them |
| **CB-D7** | Party and entities | (a) third-party by site, no entity exemptions; Izcal's own entity data, licence-reviewed, for explanation only; (b) entity exemptions (Firefox, Edge) | **(a)**: an entity exemption lets a conglomerate track across its own properties; entity data elsewhere is non-commercially licensed | Some same-owner breakage, fixed by exceptions | Same-owner breakage exceeding 20%† of exceptions after a year: reconsider narrowly, with evidence |
| **★ CB-D8** | Navigation layer (amends NW 6.9) | (a) the Privacy CG draft's bounce-tracker deletion on first-party state, with the recorded deviation and the Storage Access exemption, plus NW 6.9's 7-day backstop, in one component; stripping and debouncing in the partition service, bounce bookkeeping in the container service; strict debouncing admission; forced-redirect refusal with broker-owned activation; (b) NW 6.9's rule only; (c) request matching only | **(a)** | Two services' work, write notifications, a deletion capability, an interaction record | The CG draft reaches consensus on a different algorithm: follow it |
| **CB-D9** | Uncloaking and cloaked cookies | (a) every CNAME hop and reviewed IP ranges where Izcal resolves; cookies from cloaked responses capped at 7 days, compared against T's own chain; no resolution outside the route; (b) first and last names only, no cookie policy | **(a)** | Lookups per hop; cookie policy in NW 6.9 (amendment) | Corpus breakage from the cap above 0.1%†: narrow the comparison |
| **★ CB-D10** | Restricted scripts | (a) a "restrict" action for Izcal's lists: the script loads under an engine-enforced profile (cookie-write lifetime ≤ 7 days, CPU and wakeup budget without a page-visible report, CB-D3's uniformity once gated), with API-call attribution and creator inheritance from the JS engine; (b) block or allow only | **(a)**: a middle ground no blocker has; a mitigation that raises a tracker's cost, not a guarantee, because first-party helpers launder | JS attribution work (amendment); bindings and cookie-store hooks | Breakage from the cookie cap on the corpus above 0.1%†: lengthen the cap or drop it per entry; laundering through first-party helpers found common: report it and rely on D-S1 |
| **CB-D11** | Matcher and compiled form | (a) the rule language, total precedence order, trust-domain-scoped exceptions, phases A–C, sealed zero-copy file keyed by content, budgets instead of rule caps, reference matcher (6.2–6.3); (b) DNR-like caps; (c) interpret rules at match time | **(a)** | Compiler and verifier; the reference oracle | CB-18 missed after redesign: profile and change the index, never the semantics |
| **CB-D12** | Procedural cosmetic operators | (a) a pure, user-origin-only subset in `hg-style` (`:has-text`, `:matches-attr`, `:min-text-length`; `:upward` and procedural `:has` rewritten to native `:has()`); (b) CSS only; (c) page-injected script | **(a)**: keeps the common user-list rules working with no script and normal invalidation | Style-engine work and invalidation on text changes (style amendment) | Measured style cost on pages with large user lists above ST's budgets: restrict the subset |
| **★ CB-D13** | Hiding and the list-set fingerprint | (a) one default list set in every named container, no per-list toggles; hiding does not change background or font fetches; lazy images, container queries and computed style measured as residuals; generic hiding of blocked elements is paint-only; user lists labelled as a recognizability cost; (b) per-container default sets as Stage 1's row allows; (c) hiding as other blockers do | **(a)**: every distinct default set splits the anonymity set ([TT 8.4][TT]) | A style-only counterfactual for hidden elements; blank space where ad slots were; containers cannot differ in default lists | CB-8 finds a channel the rules miss: close it in the engine |
| **CB-D14** | User-added lists | (a) per container, by URL or file; ABP-family subset, hosts, DNR JSON and Safari JSON; code-bearing and unsafe rules ignored and reported; compile reports; fetched through the container's route; (b) Izcal syntax only | **(a)**: users judge a blocker by whether the lists they trust work ([GO 3][GO]) | Parsers for four formats; a published coverage table | A format's measured compile share on CB-2 below 90%†: drop the format or document the gap |
| **★ CB-D15** | Stage 2 extensions | (a) three tiers: list providers with zero read capability, typed interventions as data, scoped and audited IIP interception with re-prompting on capability growth; no blocking interception API; (b) a DNR-compatible API; (c) MV2-style blocking interception | **(a)**: keeps DNR's privacy benefit without its update, context and explanation failures, and refuses the read-everything disguise | Blockers needing script injection cannot run on Izcal | A class of user need that lists, interventions and audited interception cannot meet, shown with cases |
| **★ CB-D16** | List operation and governance | (a) behaviour criteria, crawler as in 6.8, flow classifier in the lab, two-person review, entry records under MIT, re-validation every 12 weeks, exceptions as entries, no payment ever, published appeals and legal demands, coverage results aggregate-only, NC datasets unused until a legal opinion, funding free of listed parties, German release after a legal opinion; (b) single review, no published evidence | **(a)**; the owner decides the two points that are not engineering: whether any funding source (for example a search deal with a listed party) is acceptable, and the German release gate | Two reviewers per entry (Section 9.3); legal opinions; publication work | Measured review throughput too low for NW-54's coverage: staff up, never drop the second review for blocks |
| **★ CB-D17** | Delivery | (a) one bundle for all regions, TUF targets with diffs, verified by the broker's update client, a witness-cosigned transparency log, one daily† slot for every client, plain public fetch per NW-D17; Security additions through the emergency channel; user-added lists through the container's route; OHTTP for reports; (b) per-list or per-region fetches; (c) the bundle through OHTTP | **(a)**: the bundle and schedule are the same for every client, so OHTTP would hide only an address every update check reveals | Bundle size for regional entries; log and witness operation | The bundle has to vary by user: move its fetch to OHTTP |
| **★ CB-D18** | Breakage reports (amends NW 6.15) | (a) scheme and host by default, a previewed and redacted path only if the user includes it, bundle version, enumerated settings vector, fired entries, the date; a recording only by reload in recording mode with separate consent; OHTTP, or NW-D17's accepted fallback until a relay exists; ≤ 30 days† retention; (b) NW 6.15's URL and rules only | **(a)**: triage needs the version and settings; the host-only default sends less than Firefox and Brave | A triage service; the relay dependency | Reports re-identify users in a review: drop the field responsible |
| **CB-D19** | Per-site controls | (a) narrow first (one party, sign-in for this tab, hiding off), then off; one protection level; no per-list toggles; no engagement-based relaxation; private-window overrides memory-only; (b) levels (Standard, Strict) | **(a)** | Shell UI work | CB-20's study fails: simplify the wording, not the model |
| **CB-D20** | Explanations | (a) decision-log record per decision; DevTools reason and Protections view; site self-test; site panel parties; agents read only their origin set; local only, FP-D18 retention; (b) counts only | **(a)** | Ring buffer per service, within CB-18 | Decision-log cost breaks CB-18: sample non-block decisions, never blocks |
| **CB-D21** | Determinism | (a) pure matching and compilation; bundle version, user-list sources and settings recorded; version pinned per top-level page except Security additions at task boundaries; worker versions defined; timed decisions recorded; (b) record every decision as an input | **(a)**: recomputation plus comparison finds divergences | Recording size (user-list sources) and recompilation time in replay | CB-3 fails: record the failing decision class as inputs |
| **★ CB-D22** | Cookie banners (Stage 2; changes Stage 1's "rule-based auto-reject") | (a) the same pipeline; hide-first with typed cleanup; no clicks in the first version; never accept; (b) reject through the consent manager by default, as Stage 1's row says | **(a)**: clicking records a preference with a system that may itself track ([CU 5.5][CU]) | Stage 2 work | Sites unusable with hiding: Q-CB-8 |
| **CB-D23** | Security list scope | (a) what the crawl observes (malvertising, forced redirects, scam redirectors, miners); URL reputation for phishing and malware is a separate system; (b) include a reputation service here | **(a)** | Phishing protection needs its own spec and owner (Q-CB-5) | Q-CB-5 unowned at CB-M3: owner decision |
| **★ CB-D24** | Protected-media host (M-1 classification) | (a) on macOS, `WKContentRuleList` classified as an OS interface fed by `hg-cb-export`; on Windows, WebView2 interception routed to `hg-content-block` in a partition service; what the host cannot express published; (b) no content blocking in the protected-media host, labelled | **(a)**: the host is the OS's own engine, and the label already says protections apply only where the host allows | An export path and a per-request IPC on Windows | The owner rejects the classification: (b) |

### 9.5 Amendments this spec proposes to other documents (not applied)

| Document | Section or ID | Amendment | From |
|---|---|---|---|
| Networking spec | 6.15 | Semantics as Section 6.2: request context, trust-domain-scoped exceptions, the total order, phases A–C, popups, `restrict` and `permissions` actions, `csp=` from an allow-list without reporting directives or hosts, the linear-time regex subset; first-party scope (CB-D6); CNAME at every hop and IP-range entries (CB-D9); "collapse" becomes paint-only suppression to keep NW-22; DNR JSON and Safari JSON import; the list set of Section 6.8, with malware and mining endpoints in Security and bot-detection scripts published but not blocked | CB-D6, CB-D9, CB-D11, CB-D13, CB-D14 |
| Networking spec | 6.15 breakage bullet; NW-54 | Report contents per CB-D18; NW-54's breakage includes interaction-triggered and cosmetic breakage (CB-15); its denominator excludes requests matched by Anti-fraud and Restrict entries, reported separately, and its numerator is "prevented by layer N or D" | CB-D18, critique C10 |
| Networking spec | 6.9 cookies | The bounce-tracking sentence gains CB-D8's faster bounce-tracker rule beside the 7-day backstop, both in `hg-nav-protect`, with the Storage Access exemption; cookie-lifetime caps for cloaked responses (CB-D9) and for writes attributed to restricted scripts (CB-D10) | CB-D8, CB-D9, CB-D10 |
| Networking spec | 6.13, NW-D17, NW-36 | The bundle as a TUF target with the witnessed log; third-party list URLs leave the system service's endpoint list and are fetched by each container's network service through its route; the report endpoint through OHTTP, with NW-D17's fallback until a relay exists | CB-D17, CB-D18 |
| Networking spec | 6.21 | New crates `hg-cb-syntax`, `hg-cb-compile`, `hg-cb-format`, `hg-cb-resources`, `hg-nav-protect`, `hg-cb-observe`, `hg-cb-compile-main`, `hg-cb-ref`, `hg-cb-export`; rules G80–G87 | 6.17 |
| Networking spec | 9.3 | The lists track's size confirmed at CB-M2 from measured review throughput; the second review, importers, log, witnesses and entity data are counted in this spec, not NW's | 9.3 |
| JavaScript engine spec | 6.10, JS-60 | API-call attribution: the responsible script of an API call is the attributed script of the innermost non-builtin frame, and code created by `eval`, `Function`, string timers, `javascript:` URLs, inserted inline scripts and same-origin child documents inherits its creator's restriction; held outside the cage; cost inside JS-60's budget | CB-D10, critique C3 |
| HTML and CSS parsing and style spec | 6.16 | User-origin-only procedural pseudo-classes (CB-D12); cosmetic subsets are re-serialized selectors with one fixed declaration (G81); background-image and font eligibility of elements hidden only by user-origin rules computed from a style-only counterfactual, its cost measured with the layout owner at CB-M2 (lazy images stay a residual) | CB-D12, CB-D13 |
| Graphics and rendering spec | Display-list generation | Paint suppression of elements whose load content blocking refused: not drawn, layout unchanged, the "blocked" flag a paint-only input exposed to no API (IntersectionObserver v2 included); a click on a suppressed element whose nearest link targets a blocked URL is not followed | CB-D13, critique C26 |
| Resource-footprint spec | 6.11, FP-28 | "Matcher ≤ 8 MiB† charged in the network service" becomes "compiled default bundle ≤ 8 MiB†, clean and shared; charged matcher state ≤ 256 KiB† per partition service (NW 6.15)"; p99 ≤ 20 µs† per request with all phases summed, plus a cold-start case; compile budgets (CB-23); "List governance, breakage and the default list choice belong to the shell and policy work" becomes "to the content-blocking spec" | CB-18, CB-23, FC-M8, C8 |
| Resource-footprint spec | FP-D16 | Compiled filter files are keyed by the hashes of their source lists and the compiler version, not by (T, C), because they hold no user- or site-derived state; files from user-added lists are keyed per container | CB-D11, critique C9 |
| Resource-footprint spec | 6.14, FP-D7 | FP-D7's CPU and wakeup budget also applies to restricted scripts (CB-D10), without the Reporting API report, because a restrict cap is list policy | CB-D10, critique C3 |
| Stage 1 design | Section 6, "Built-in content blocking" | Izcal's matcher and lists per this spec (replacing Brave's `adblock`); six layers; one default list set in every named container | CB-D1, CB-D13 |
| Stage 1 design | Section 6, "Containers" | The block-list set is per container, and Izcal's named containers share one default set (uniformity) | CB-D13 |
| Stage 1 design | Section 6, "Extensions"; D-S3 note | Stage 2's three tiers; no blocking interception API | CB-D15 |
| Stage 1 design | Section 6, "Cookie-banner handling" | Reuses the list pipeline; hide-first; no clicks in the first version (replacing "rule-based auto-reject") | CB-D22 |
| Stage 1 design | 13.2 | Add a content-blocking row: Izcal's matcher and lists (with NW's amendment); bundle on TUF with a witnessed log; reports through OHTTP | CB-D17, CB-D18 |
| Stage 1 design | 13.1a, D-S7 | The protected-media host's blocking: `WKContentRuleList` as an OS interface on macOS; WebView2 interception routed to a partition service on Windows | CB-D24 |
| Stage 1 design | 2.2 (rule registry) | G80–G87 registered to this spec; G73–G79 recorded as unassigned (the owner directed this spec to start at G80) | 6.17, critique C31 |
| Stage 1 design | D-S1 | Per-script uniformity for restricted scripts, gated by CB-D3's measurements; an addition, not a change to D-S1's defaults | CB-D3 |
| Stage 1 design | 8.3, D-S8 | Fund the added 5–9.5† people, crawler compute and egress, hosting, the witnesses and legal opinions; record that funding from a party the lists name conflicts with CB-D16 | 9.3, CB-D16 |
| HGBA-3 | PRIV-2 | Add a surface inventory (the fingerprinting APIs of [TT 2.5][TT] plus D-S1's discrete and readback surfaces), the list-set probes (CB-8) and the learned-state attack classes (CB-9) | CB-8, CB-9, FC-M9 |
| HGBA-3 | PRIV-1 | Measure with protection on and off from the same IP and session, per challenge provider | CB-14 |
| DevTools spec | Network view; IIP domains | The typed reason for blocked requests and the `Protections` domain and view; agents read it only for their origin set | CB-D20 |
| Shell UI design | 4.3 site panel | Counts of parties and organisations; graduated controls; the report flow; the "recognizability" label for user-added lists; the Security interstitial | CB-D13, CB-D19 |
| Security scope (in review) | IPC-R15, fuzzing, UPD-R5 | List text is a web-format input parsed only in the utility process; `hg-cb-syntax` and `hg-cb-compile` as tier-P fuzz targets; Security-list additions ride the emergency channel | G80, CB-4, CB-D17 |
| Licence audit | 2.5 | Record that the non-commercial datasets are not used even test-only until Q-CB-1 is answered, that Tranco's licence is unstated, and that entity-data inputs are licence-reviewed | Q-CB-1, Q-CB-2 |

## 10. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Izcal's lists are weaker than mature lists at first | High | Less protection than users of other blockers expect | Layers P, V and S carry most of the privacy load; CB-11 and CB-12 measure per layer; users may add lists (NW 6.15's existing risk row) |
| No scriptlets makes Izcal worse on anti-adblock and annoyance sites | High | Users of uBO-family lists see walls Izcal does not defeat | Stated cost (CB-D2); published coverage (CB-2); reversal trigger for Izcal's own lists |
| Legal action against Izcal's lists or cosmetic filtering (Germany's remand, competition claims, defamation claims about evidence) | Medium | Shipping blocked in a market; legal cost | Behaviour-based criteria; factual, dated, correctable evidence; German legal opinion before release; published legal demands |
| The crawler is detected and served a different web | High | Missed trackers, wrong breakage results | Headed, realistic profiles, several regions; published block rate (CB-24); consented user reports as a second signal |
| Review throughput cannot keep up with evasion | Medium | Coverage below NW-54 | Flow features resist evasion; throughput measured at CB-M2; staffing decided by the owner with the number |
| Restricted scripts launder through first-party helpers, or attribution misses bundled code | High | The restrict profile binds less than it appears to | Creator inheritance closes the one-line escapes (CB-27); D-S1 covers everyone; the helper case is reported; CB-12 reports first- and third-party separately |
| Residual channels identify list users | Medium | User-added lists become a fingerprint | Uniform defaults; label in settings; CB-8 measures every residual |
| Navigation defences break sign-in or payment flows | Medium | Users turn protection off | Activation, WebAuthn and Storage Access grants count; login-flow entries; CB-15 weights login and payment |
| No independent witnesses or relay operator | Medium | Bundle delivery or private reports held | Witness recruitment from CB-M0; reports use NW-D17's accepted fallback until a relay exists |

## 11. Open questions and flagged claims

### 11.1 Open questions

- **Q-CB-1.** Can the non-commercially licensed datasets (Disconnect, Tracker Radar, TrackerDB) be used even test-only to measure the coverage of a commercial product's lists? And which entity-data inputs (corporate-record databases) may feed an MIT-published entity map? Legal opinion needed; until then neither is used ([LG 1][LG]).
- **Q-CB-2.** Tranco states no licence for its own list and one of its inputs is CC BY-NC 4.0; CrUX's licence is stated differently by Google and Tranco. A legal reading before the crawler uses Tranco list IDs ([LG 2.4–2.6][LG]).
- **Q-CB-3.** German legal opinion on cosmetic filtering as a user-origin sheet, after the BGH's 31 July 2025 remand; and whether the owner accepts a German release gate (CB-D16).
- **Q-CB-4.** The OHTTP relay operator for reports (Q-NW-6).
- **Q-CB-5.** Who owns phishing and malware URL reputation (a Safe-Browsing-class service), and with what privacy design? (CB-D23)
- **Q-CB-6.** How do ABP, uBO, AdGuard, Safari and Chrome's DNR each read the areas their documents leave silent: non-ASCII separators, case folding, URL normalization, `domain=` scope, third-party relative to frame or top, tie order? Black-box measurement for the rule-syntax page ([ME 1.16, Q1][ME]).
- **Q-CB-7.** GPC: Izcal sends it by default (Stage 1 Section 6), but the GPC specification notes that many US states hold that a user agent may not send an opt-out by default, and California AB 566 requires a browser setting from 1 January 2027 (law-firm summary only). The networking and shell owners decide the per-jurisdiction default and the setting; the crawler records GPC non-compliance as evidence ([GO 13][GO], [TT 10.5][TT]).
- **Q-CB-8.** Cookie banners: when, if ever, should Izcal reject through a site's consent manager rather than hide? (CB-D22)
- **Q-CB-9.** Opt-in aggregate rule statistics through a distributed aggregation protocol: worth a P1 analysis once the protocol is an RFC? ([GO 11.3][GO])
- **Q-CB-10.** Should Izcal support Privacy Pass tokens (RFC 9576–9578) to reduce challenges caused by privacy defaults, and with what privacy review? ([CU 6.2, 6.3][CU])
- **Q-CB-11.** An ITP-like lifetime cap on all script-written first-party cookies, not only those of restricted scripts: measure its breakage on the corpus before deciding.
- **Q-CB-12.** Should anti-fraud and bot-detection scripts be restricted (CB-D10) rather than allowed, once PRIV-1 shows the challenge cost of each choice?
- **Q-CB-13.** Review and triage throughput and the lists track's size, measured at CB-M2.
- **Q-CB-14.** Can layout provide a counterfactual for lazy images under or after hidden elements at an acceptable cost, closing that residual? Measured with the style and layout owners at CB-M2.
- **Q-CB-15.** Who operates the transparency log's witnesses (k of n, independent of Izcal), and under what agreement?

### 11.2 Claims that are single-source, unverified or conflicting

| Claim | Status |
|---|---|
| Lists match 27.5% of first-party tracking scripts; > 54% of sites use first-party or server-side tracking ([TT 4.7][TT]) | Preprint (June 2026), not yet peer reviewed |
| Forced redirects were 81% of malicious ads in October 2024 ([TT 6.3][TT]) | Vendor data, no published method |
| Brave's −75% memory and about 45 MB ([HP 8.5][HP]) | Vendor-reported, Brave's own corpus |
| Safari's 50,000 → 150,000 rule cap: iOS 15 (2021) or 2022 ([ME 2.12][ME]) | Two vendor pages disagree; no Apple source read |
| Safari 26 made Advanced Fingerprinting Protection the default ([GO 7.5][GO]) | Secondary sources only, conflicting on parameter stripping |
| The WhoTracks.me performance study ([HP 5.4][HP]) | By a competing blocker's authors; original URL returns 404 |
| California AB 566's browser requirement ([GO 13.1][GO]) | Law-firm summary; statute page not read |
| The BGH's 2025 remand: the press-release number ([HP 2.8][HP]) | Holding High; release number from a snippet |
| uBO Lite's cosmetic filters about 20% less effective ([HP 5.10][HP]) | Read via Wikipedia only; not used in a decision |
| CrUX licence (CC BY 4.0 per Google, CC BY-SA 4.0 per Tranco) ([LG 2.6][LG]) | Conflicting |
| Replica ad domains survived about 410 days before blocking ([LG 4.7][LG]) | Search snippet only |
| Breakage counts for user reports (37.5% vs 91–100% experienced breakage) ([CU 1.5][CU]) | Studies disagree; the question's wording explains it |
| No public measurement of CAPTCHA cost caused by blocking itself ([CU 6.6][CU]) | Absence of evidence; PRIV-1 must measure |
| Cryptojacking collapsed after Coinhive closed (2019) ([TT 7.5][TT]) | Not checked; no prevalence measurement after 2020 |
| The Privacy CG draft's precondition (TT 3 implications) and Brave's intermediate-CNAME rule (GO 10 implications) | Both were misstated in the notes and in draft 0.1; corrected in the notes and in Sections 6.4 and 6.5 (FC-M3, FC-M7) |

## Appendix A. Review disposition (revision 0.2)

Sources: the independent fact-check ([factcheck.md][FK]: 118 claims, 91 correct, 12 partly correct, 5 wrong, 2 unverifiable, 8 not checked; 27 external links, none dead; 22 internal links, all resolve; no link to engine or blocker source code) and the independent design critique ([critique.md][CR]: 36 findings, 7 High, 19 Medium, 10 Low). Neither reviewer wrote the spec or its notes. Every item is applied; where the fix differs from the one proposed, the row says so. IDs: **FC-M** fact-check must-fix; **FC-R** fact-check recommended; **C** critique finding.

**Fact-check, must-fix**

| ID | Finding | Disposition | Where |
|---|---|---|---|
| FC-M1 | "Every published dialect" lets an exception in any list cancel any block: wrong for DNR and Safari | Applied, with the corrected precedent cited; combined with C2's trust domains | 6.2 |
| FC-M2 | "Same-priority order is undefined in every published dialect": wrong | Applied as proposed | 1, 3 |
| FC-M3 | The Privacy CG draft's precondition does not "always hold" under partitioning | Applied: deviation stated and recorded; the TT 3 implications line corrected in the note | 6.5; TT 3 |
| FC-M4 | Person-years to Stage 1 exit wrong | Applied, recomputed with the larger added staffing of C21: about 12–33† with a ramp | 9.3 |
| FC-M5 | 84% and 45 rules conflated | Applied as proposed | Summary |
| FC-M6 | 27.5% is a share of scripts, not sites | Applied as proposed | Summary |
| FC-M7 | Brave's correction says it does **not** match intermediates | Applied; the GO 10 implications line corrected in the note | 6.4; GO 10 |
| FC-M8 | FP-28's 8 MiB charged changed without an amendment | Applied: restated in 6.3 and CB-18, amendment row added | 6.3, CB-18, 9.5 |
| FC-M9 | PRIV-2 has no surface list | Applied: the inventory is this spec's PRIV-2 amendment, referenced as such | 6.4, 6.6, CB-12, 9.5 |

**Fact-check, recommended**

| ID | Finding | Disposition | Where |
|---|---|---|---|
| FC-R1 | Safari's and Firefox's lists; report stripping | Applied | 6.5, 6.11 |
| FC-R2 | EasyList maintainers | Applied | 2.1 |
| FC-R3 | Disconnect files anti-fraud separately | Applied | 6.6 |
| FC-R4 | Published tails | Applied | 6.3 |
| FC-R5 | Surrogate catalogue counts | Applied | 9.3 |
| FC-R6 | FP-D9 allows opt-in aggregates | Applied | 6.16 |
| FC-R7 | Reconciliation reference | Applied ([RN] P24 and Section 7) | 9.3 |
| FC-R8 | EasyList traffic source | Applied | 6.9 |
| FC-R9 | WebKit and Privacy Badger fixes differ | Applied | Summary |
| FC-R10 | Cryptojacking claim unchecked | Applied: added to 11.2 | 11.2 |
| FC-R11 | Borderline links in the notes | Applied: the commit link in LG 2.5 removed and its claim restated without it; the Bugzilla link in HP 3.7 kept as issue discussion (allowed), with the Mozilla blog as the primary support | LG 2.5; HP 3.7 |
| FC-R12 | Brave's `adblock` excluded in 2.3.0 | Applied | 1 |
| FC-R13 | Redirecting URLs | Kept: all four redirect cleanly to the same documents | — |
| FC-R14 | Note-inherited errors | Applied: 11.2 row; notes corrected | 11.2; TT 3; GO 10 |

**Design critique**

| ID | Severity | Finding | Disposition | Where |
|---|---|---|---|---|
| C1 | High | Two bounce-tracking rules; NW 6.9 not amended; Storage Access; deletion path; interaction record | Applied: both rules kept in one component, NW 6.9 amended (starred CB-D8); Storage Access exemption; write notifications and a broker-granted deletion capability; the interaction record's holder, retention and private-window behaviour | 6.5, CB-D8, CB-10, 9.5 |
| C2 | High | List-scoped exceptions disable Izcal's own exceptions; user rules; "allow one party" | Applied as proposed: trust domains; user rules cancel any domain; Security narrowed only by same-ID exceptions; "allow one party" acts on a party; CB-1 cases | 6.2, 6.12, CB-1, CB-6 |
| C3 | High | Restrict rests on undefined attribution; one-line escapes; FP-D7 report | Applied: JS amendment for API-call attribution with creator inheritance; honest threat model; CB-27; no FP-D7 report for restrict caps | 6.4, CB-D10, CB-27, 9.5 |
| C4 | High | `csp=` can report to an external URL | Applied as proposed: directive allow-list, no reporting directives or hosts; G81, CB-5, CB-6, CB-7 | 6.2, 6.6, G81, 8.2 |
| C5 | High | The log does not stop a targeted list; verifier process | Applied: witness-cosigned checkpoints with consistency proofs; the broker's update client verifies; CB-22 restated; witnesses as Q-CB-15 and a cost | 6.0, 6.9, CB-22, 9.1, 9.3 |
| C6 | High | Surrogate generation from tracker source | Applied: source-based generators excluded; provenance manifests (G87) | 6.6, 6.17, 7 |
| C7 | High | Resource-only Ads entries outside class (c) | Applied, option (a): resource cost never qualifies; FP-D7 handles outliers | 6.8, CB-D5 |
| C8 | Medium | FP-28 per phase; cold start | Applied: per request, all phases summed; cold-start case; page-table cost | 6.3, CB-18, 9.5 |
| C9 | Medium | FP-D16 partitioning | Applied: content-keyed files, user-list files per container; FP-D16 amendment | 6.3, 9.5 |
| C10 | Medium | NW-54's denominator and NW 6.15's list composition | Applied: amendment row | 6.8, 9.5 |
| C11 | Medium | Debouncing as an open redirect; referrer | Applied as proposed | 6.5, CB-10 |
| C12 | Medium | Forced-redirect refusal: outcome, activation, P2 class, WPT | Applied: silent no-op and console message; broker-owned activation; class (c) under P1 security; CB-26 | 6.5, CB-26 |
| C13 | Medium | Fetch independence infeasible for lazy images; script-visible hiding | Applied: narrowed to backgrounds and fonts; lazy images, container queries and computed style as measured residuals; Q-CB-14 | 6.7, 6.16, CB-8, 9.5 |
| C14 | Medium | Report re-identification; recording flow; NW-D17 fallback | Applied: host-only default, redacted path opt-in, date only; reload-to-record flow; NW-D17's accepted fallback followed (the hold in 9.1 removed) | 6.11, CB-D18, 9.1 |
| C15 | Medium | Stripping and debouncing in the container service | Applied: moved to the partition service; only bounce bookkeeping in the container service | 6.0, 6.5, 6.17 |
| C16 | Medium | User lists fetched outside the container's route | Applied: fetched by the container's network service through its route; NW-D17 amendment | 6.9, CB-D14, CB-D17, 9.5 |
| C17 | Medium | Version pinning, workers, user lists in recordings, compiler determinism, security cadence | Applied: Security additions mid-page as recorded inputs via the emergency channel; worker versions; two mapped versions; user-list sources recorded; CB-28 | 6.3, 6.9, 6.14, CB-3, CB-28 |
| C18 | Medium | Phase B unavailable behind proxies and remote resolution | Applied: configurations listed, no resolution outside the route, CB-13 scoped | 6.4, CB-13 |
| C19 | Medium | Protected-media host uses WebKit's blocker; Windows path | Applied: OS-interface classification put to the owner (CB-D24); WebView2 routed to a partition service | 6.18, 7, CB-D24, 9.5 |
| C20 | Medium | Coverage job as a candidate source | Applied: aggregate-only results unreadable by the candidate pipeline (G84); CB-21 check | 6.8, G84, CB-21 |
| C21 | Medium | Staffing: second reviewer, person-years, importers, missing work | Applied: second-review and log, witness and entity rows; importers counted here only; person-years recomputed; total 10–19† | 9.3, summary |
| C22 | Medium | Targets not falsifiable or self-measured | Applied: independent oracles for CB-11 and CB-12; statistics for CB-8; thresholds for CB-20; CB-17 rescoped; CB-25 method | 8 |
| C23 | Medium | Cosmetic budget may refuse EasyList | Applied: generic rules in one shared, mapped sheet charged once; budget set from measurement at CB-M0; refusal message | 6.3, CB-M0 |
| C24 | Medium | Whole Ads list held for one criterion | Applied (moot after C7): Ads ships at CB-M3 | 9.1, 9.2 |
| C25 | Medium | Cosmetic CSS injection; G80 wording | Applied as proposed | 6.7, G80, G81, CB-5 |
| C26 | Medium | Paint suppression: invisible links, accessibility, flag exposure, owner | Applied: blocked-link clicks not followed; alt text kept as for any failed image; flag paint-only with a CB-8 probe; amendment moved to the graphics spec | 6.2, 6.7, CB-8, 9.5 |
| C27 | Low | Two Stage 1 rows changed without a star | Applied: CB-D13 and CB-D22 starred with what they change | 9.4 |
| C28 | Low | Per-list toggles contradictory | Applied: no per-list toggles; the settings vector has no per-list field | 6.7, 6.11, 6.12 |
| C29 | Low | Missing reversal triggers and cost | Applied | 9.4 |
| C30 | Low | † on borrowed values | Applied: precedent values marked as such, each with Izcal's reason; † kept only on Izcal's own numbers | 6.4, 6.5, 6.8, Section 8 note |
| C31 | Low | Registry bookkeeping; 13.2 row | Applied: G73–G79 recorded unassigned; Stage 1 2.2 row; 13.2 "add a row" | 6.17, 9.5 |
| C32 | Low | Commit link in LG 2.5 | Applied (FC-R11) | LG 2.5 |
| C33 | Low | MD-D14 conditional | Applied | 1, 6.6 |
| C34 | Low | Agents, private windows, popups, blocked top-level navigation, witnesses, system-service fetches | Applied: decided in the text (6.2 popups, 6.12 private windows, 6.13 agents, 6.16 interstitial, 6.9 per-container fetches) and Q-CB-15 for witnesses | 6.2, 6.9, 6.12, 6.13, 6.16, 11.1 |
| C35 | Low | Entity-data licences and personal data | Applied: inputs licence-reviewed, organisation-level facts only, provenance per entity; in Q-CB-1 | 6.8, 7, Q-CB-1 |
| C36 | Low | `hg-cb-resources` layer | Applied: moved to L5; `hg-style` linking `hg-cb-format`'s reader stated | 6.17 |

**M-1 part 3 result.** Neither reviewer found a link to engine or blocker source code in the report; the notes' one commit link (LG 2.5) is removed. No design value is derived from another engine's or blocker's source.

[HP]: ../research_notes/Izcal%20content%20blocking/history_postmortems.md
[TT]: ../research_notes/Izcal%20content%20blocking/tracking_threats.md
[ME]: ../research_notes/Izcal%20content%20blocking/matching_engines_performance.md
[LG]: ../research_notes/Izcal%20content%20blocking/list_production_governance.md
[CU]: ../research_notes/Izcal%20content%20blocking/compat_breakage_ux.md
[GO]: ../research_notes/Izcal%20content%20blocking/gaps_opportunities.md
[FR]: ../research_notes/Izcal%20content%20blocking/frame.md
[H2]: HGBA%202%20browser%20architecture%20spec.md
[H3]: Future%20software%20browser%20engine.md
[S1]: HGBA%20Stage%201%20architecture%20design.md
[NW]: Izcal%20networking.md
[ST]: Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[FP]: Izcal%20resource%20footprint.md
[JS]: Izcal%20JavaScript%20engine.md
[GR]: Izcal%20graphics%20and%20rendering.md
[TX]: Izcal%20fonts%20and%20text.md
[MD]: Izcal%20images%20and%20media.md
[LY]: Izcal%20layout.md
[DT]: Izcal%20DevTools%20spec.md
[UI]: HGBA%20shell%20UI%20design.md
[RN]: ../research_notes/Phase%202%20reconciliation/reconciliation.md
[LA]: ../research_notes/Phase%202%20reconciliation/licence_audit.md
[FK]: ../research_notes/Izcal%20content%20blocking/factcheck.md
[CR]: ../research_notes/Izcal%20content%20blocking/critique.md
