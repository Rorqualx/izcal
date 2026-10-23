# Izcal content blocking: fact-check (phase 8)

Checked 19 October 2026 against `reports/Izcal content blocking.md` draft 0.1, by an independent fact-checker who did not write the spec or its notes.

**Method.** (1) Claims the design depends on were extracted from the summary, Sections 1–3, 5, 6, 8, 9 and 11, with priority on the summary, Sections 2, 3, 5, 6 and the decision table. Each was checked against the note row it cites and, for load-bearing numbers, against the primary source: the papers were downloaded and their full text searched (Luo et al. IMC 2025; Böttger et al. arXiv 2606.16720; Randall et al. IMC 2022; El Hajj Chehade et al. USENIX Security 2025; Kontaxis and Chew; Snyder et al.; SINBAD; WebGraph; FP-Inspector; Dimova et al.), and the web pages were fetched and their text searched (armin.dev; EFF 2020; WebKit 9661 and 11338; Brave privacy updates 6, 9, 24 and 36 and the 2019 performance post; Brave's web-compatibility wiki page; Privacy Sandbox update; Chrome DNR reference; Chrome MV2 timeline; Chromium heavy-ads post (source of the page); Privacy CG Navigational-Tracking Mitigations; ARES 2026 project page; Nethercote 2015; vzbv; AdGuard's EasyList post; CSS Cascade 5). One count was re-run through the GitHub search API (AdguardFilters issues created in 2025). (2) Every external link in the report was fetched with redirects followed; the one `#fragment` was checked in the page's HTML; every internal link definition was resolved relative to `reports/`. (3) Every URL in the six strand notes and the frame note (288 distinct) was screened for code hosts, commits, diffs, code reviews and raw source. (4) Cross-references were read in the cited specs. (5) Staffing arithmetic was recomputed.

**Grades.** Correct: the claim matches the source. Partly correct: the substance holds but a number, scope or attribution is off. Wrong: the source says otherwise. Unverifiable: no primary source reachable. Not checked: consistent with its note row, the row's source not re-read here.

**Counts.** 118 claims: **91 Correct, 12 Partly correct, 5 Wrong, 2 Unverifiable, 8 Not checked**. Links: **27 distinct external links** in the report (35 occurrences): **27 resolve (HTTP 200)**, 4 through a redirect (the ABP help page and the three RFC pages), **0 dead**; **1 fragment** (`css-cascade-5/#importance`), present. **22 internal link definitions**, all resolve; no reference is used without a definition and none is defined unused. M-1 part 3: **no link to engine or blocker source code**; 6 borderline links in the notes, judged in Section 3. Must-fix: **9** (FC-M1 to FC-M9). Recommended: **14** (FC-R1 to FC-R14).

## 1. Claims

Location codes: S = summary paragraph; numbers are report sections; T = target ID; D = decision ID.

| # | Claim as stated | Location | Source checked | Grade | Correction |
|---|---|---|---|---|---|
| 1 | About half of fingerprinting is first-party-served | S | Luo et al. IMC 2025 §5: 49% (top 20K), 52% (tail) of canvas-fingerprinting sites | Correct | — |
| 2 | EasyList-based blockers cut fingerprinting by only about 5% in a 2025 measurement | S | Luo et al.: canvases and sites "only decreased by about 5%" under ABP and uBO | Correct | — |
| 3 | First-party and server-side tracking on more than 54% of sites | S, 3 | Böttger et al. abstract: "over 54% of analyzed sites" | Correct | — |
| 4 | "of which the dominant lists match 27.5%" | S | Böttger et al. §5: 33,081 (27.5%) of the **URLs serving FPT-related JavaScript** were flagged by EasyList or EasyPrivacy; not 27.5% of the sites | Partly correct | FC-M6 |
| 5 | Identifiers carried through navigations on 8–11% of navigation paths | S | Randall et al.: 8.11% of unique URL paths; 10.8% with bounce tracking | Correct | — |
| 6 | ITP's classifier and Privacy Badger's local learning were both shown to leak | S | WebKit 9661; EFF 2020-10-07 | Correct | — |
| 7 | "both projects moved to uniform rules and one list shipped to everyone" | S | WebKit moved to uniform, classifier-independent rules; Privacy Badger moved to one pre-trained list. ITP ships no list | Partly correct | FC-R9 |
| 8 | A user's list choice identifies 84% of lists "with 45 probe rules and no script" | S | Double-Edged Shield abstract: attacks "identify 84% of filter lists" and "reduce the relative anonymity set ... to a median of 48 users ... using only 45 rules out of 577K". The 45 rules belong to the anonymity-set result, not to the 84% | Partly correct | FC-M5 |
| 9 | Every blocker that let lists run code ended up with a trust tier by provenance | S, 6.6 | ME 1.2, 1.5, 1.6, 4.2, 4.4 (ABP, uBO, AdGuard documentation) | Correct | — |
| 10 | The one that did not shipped a code-execution vulnerability to more than 100 million users | S | armin.dev: "The affected extensions have more than 100 million active users" | Correct | — |
| 11 | ABP `$rewrite` let a list author execute code on Google sites and target chosen IP addresses | 5, 6.6 | armin.dev: Google Maps steps; targeting "based on the IP addresses from which the updates are requested" | Correct | — |
| 12 | The 2019 code-execution CVEs | D CB-D2 | armin.dev: CVE-2019-11593, -11594, -11595 | Correct | — |
| 13 | Paid allowlisting, ad data sold to the ad industry and an undisclosed contractual exception each cost the project more than a coverage gap | S | HP 2, 3, 11, 14 (interpretive synthesis of HP rows) | Not checked | — |
| 14 | Networking spec staffs 5–9† of content-blocking work; this spec adds 4–7†; 9–16† in all | S, 9.3 | NW 9.3: Content blocking 2–3, Lists 4–7 of which 3–6 block lists; 2–3 + 3–6 = 5–9; table rows sum to 4–7 | Correct | — |
| 15 | About 2–4† of them are curators, reviewers, triagers and policy staff | S, 9.3 | NW 9.3 Lists track "about half engineers": 1.5–3 of 3–6, plus governance 0.5–1 = 2–4 | Correct | — |
| 16 | About half of users facing breakage give up on the site or the blocker | 1 | CU 1.2 (about 50% would abandon the blocker); CU 1.4 (91/180 went elsewhere) | Correct | — |
| 17 | 38–66% of forum reports are still reproducible when examined | 1 | SINBAD: 41% EasyList, 38% uBlock, 66% AdGuard | Correct | — |
| 18 | `net::ERR_BLOCKED_BY_CLIENT` names no rule, list or reason | 1 | CU 7.3 (Chrome DevTools Network reference) | Not checked | — |
| 19 | Replica ad domains survived about 410 days before listing | 1 | LG 4.7 (search snippet only; flagged in 11.2) | Unverifiable | — |
| 20 | Same-priority order is undefined in every published dialect | 1 | ME 1.9 (Safari JSON: "Order matters", order-defined), ME 1.12, 1.14, 1.7 (DNR and AdGuard say undefined; Firefox orders by ruleset kind; ABP and uBO documents are silent) | Wrong | FC-M2 |
| 21 | HGBA-3's P2 scope note allows default-on interventions only as class (c) "privacy protections on by default that carry a recorded P1 analysis" | 1 | HGBA-3 Section 2 P2 scope note, quoted exactly | Correct | — |
| 22 | The scope note requires publishing compatibility cost per release | 1 | Same: "publishes its compatibility cost (WPT and corpus breakage) per release" | Correct | — |
| 23 | Hosts files survive as DNS sinkholes (Pi-hole, 2014) that see no path, frame or first party | 2.1 | HP 12.2 | Correct | — |
| 24 | Adblock then Adblock Plus, 2002–2006 | 2.1 | HP 1.1 | Correct | — |
| 25 | EasyList is volunteer-run (4 named maintainers today) | 2.1 | LG 1.1 (4 maintainers); LG 1.4: Brave has paid the top maintainer since June 2019 | Partly correct | FC-R2 |
| 26 | 90.16% of EasyList's network rules never fired in a 74-day crawl of 10,000 sites | 2.1 | Snyder et al.: "90.16% of the resource blocking" rules; 10,000 sites, 74 days | Correct | — |
| 27 | Ghostery's GhostRank fed ad-industry data; the product was sold twice | 2.1 | HP 3.3–3.5 (Evidon, Cliqz) | Not checked | — |
| 28 | Disconnect powers Firefox and Edge | 2.1 | HP 3.6, 10.2; LG 1.15 | Correct | — |
| 29 | Acceptable Ads survived competition law (BGH 2018) | 2.1 | HP 2.5; CU 5.9 (BGH PM 78/2018, I ZR 154/16) | Correct | — |
| 30 | Privacy Badger local learning off by default in October 2020 after Google showed it could be steered and read back | 2.1 | EFF 2020-10-07 | Correct | — |
| 31 | uBO disabled on Chrome by Manifest V3 (Chrome 138, July 2025) | 2.1 | Chrome MV2 timeline: "Jul 24th 2025: Manifest V2 is disabled everywhere" (Chrome 138); disabling began 2024-10-09 | Correct | — |
| 32 | Safari content blockers (2015): declarative JSON compiled ahead of time, run in the network subsystem; the extension never sees URLs | 2.1 | WebKit blog 3476 via ME 1.9, 2.10, HP 6.1 | Correct | — |
| 33 | Safari rule caps 50,000, later 150,000 per blocker | 2.1 | ME 2.12, HP 6.2 (vendor pages; date flagged in 11.2) | Correct | — |
| 34 | Firefox Tracking Protection measured 44% median page-load reduction, 39% less data on news sites | 2.1 | Kontaxis and Chew: "44% median reduction in page load time and 39% reduction in data usage" (Alexa top 200 news) | Correct | — |
| 35 | ETP default-on from 2019 | 2.1 | HP 7.3 (snippet) | Not checked | — |
| 36 | WebKit moved to full third-party cookie blocking, March 2020 | 2.1 | HP 6.7 (WebKit blog 10218, 2020-03-24) | Correct | — |
| 37 | DNR: caps raised only with blocker vendors' data | 2.1 | HP 5.6; ME 2.15 (Chrome blog 2023-11-15) | Correct | — |
| 38 | DNR had no top-level context until Chrome 145 | 2.1, 3 | DNR reference: `topDomains` "Chrome 145+" | Correct | — |
| 39 | Brave Rust rewrite 2019, 69× faster | 2.1 | Brave blog 2019: "69x" | Correct | — |
| 40 | Brave CNAME uncloaking 2020; debouncing 2021 | 2.1 | HP 8.2, 8.3; Brave privacy update 6 (2020) | Correct | — |
| 41 | Brave zero-copy store (2026): −75% memory, about 45 MB | 2.1, 6.3 | Brave privacy update 36: "by 75%", "roughly 45 MB of memory savings" | Correct | — |
| 42 | Brave stopped first-party network blocking by default in 2021 because breakage made users turn Shields off | 2.1, 6.2 | Brave privacy update 9: version 1.30; reasons: other protections cover it, and fewer "drop shields" | Correct | — |
| 43 | Tracker Radar data CC BY-NC-SA; undisclosed contractual Microsoft exception (2022) | 2.1 | LG 1.13, 4.4; HP 11.2 | Correct | — |
| 44 | Most Privacy Sandbox APIs retired 17 October 2025 for "low levels of adoption"; CHIPS, FedCM and Private State Tokens survived | 2.1 | Privacy Sandbox update (2025-10-17), quoted exactly | Correct | — |
| 45 | Firefox SmartBlock, query stripping and bounce tracking protection, 2021–2025 | 2.1 | HP 7.5–7.7 | Correct | — |
| 46 | AFP default in all browsing from Safari 26 (secondary sources) | 2.1 | GO 7.5 (secondary, flagged) | Unverifiable | — |
| 47 | 39% of requests were blocked in Brave's 2019 data | 2.2 | Brave blog 2019: "39% were blocked" with EasyList and EasyPrivacy | Correct | — |
| 48 | APIs were removed under blockers (NoScript 2017, uBO on Chrome 2025) | 2.2 | HP 3.2, 5.8 | Correct | — |
| 49 | Brave's 2026 gain came from the storage format, not matching | 2.2 | Brave update 36 also reports +13% matching speed; the memory gain is from the format | Correct | — |
| 50 | Lists can carry code and can be served selectively by IP | 2.2 | armin.dev | Correct | — |
| 51 | Cookie syncing among 45 of the top 50 third parties (2016) | 3 | TT 1.3 (Englehardt and Narayanan §5.6) | Not checked | — |
| 52 | Third-party fingerprinting on 10–25% of top sites | 3 | FP-Inspector: 10.18% of top 100K; over a quarter of top 10K | Correct | — |
| 53 | First-party or bundled fingerprinting on 49–52% of canvas-fingerprinting sites | 3, 6.6 | Luo et al. | Correct | — |
| 54 | 23% of canvas-fingerprinting sites in the top 20K use one vendor's bot-detection canvas | 3 | Luo et al.: Akamai "485 (23%)" | Correct | — |
| 55 | 8.1% of navigation paths smuggle UIDs, 10.8% with bounce tracking; 41% of dedicated smugglers missing from Disconnect | 3, 6.5 | Randall et al.: 8.1%; 10.8%; "41% of them (11 out of 27)" | Correct | — |
| 56 | Dedicated smugglers were 16 of the 30 most common redirectors in 2022 | 6.5 | Randall et al.: "16 of the 30 most common redirectors are dedicated smugglers" | Correct | — |
| 57 | CNAME cloaking on 9.98% of the Tranco top 10K; cookie leaks on 95% of those sites | 3, 6.4 | Dimova et al.: 9.98%; leaks on 7,377 of 7,797 sites (95%) | Correct | — |
| 58 | Parameter key sets matched 63% of what lists matched 0% of; 3% false positives; 1 of 2,300 sites visibly broken | 3, 6.2 | Böttger et al. §5.5–5.6: 5,482 (63%) of V_FPT "while EasyList and EasyPrivacy flagged none"; 260 (3%) of V_Other; 1 page of the Tranco top 2,300 | Correct | — |
| 59 | Malvertising about 1% of ads (2014) | 3 | TT 6.1 (Zarras et al.) | Not checked | — |
| 60 | Forced redirects 81% of malicious ads (vendor, October 2024) | 3, 6.5 | TT 6.3 (vendor; flagged in 11.2) | Correct | — |
| 61 | Cryptojacking on 0.011% of domains (2019) | 3 | TT 7.3 (Bijmans et al., abstract) | Correct | — |
| 62 | Cryptojacking "collapsed after Coinhive closed in 2019" | 3 | TT 7.5 is graded Low and "Not checked this session"; no prevalence measurement after 2020 | Partly correct | FC-R10 |
| 63 | Filter-list fingerprinting: 84% of lists identified; median anonymity set 48 users | 3 | Double-Edged Shield abstract | Correct | — |
| 64 | DNR rations rule-match feedback to 20 calls per 10 minutes | 3, 5 | DNR reference: `MAX_GETMATCHEDRULES_CALLS_PER_INTERVAL` 20; interval 10 (minutes) | Correct | — |
| 65 | AdGuard's filter tracker received 24,389 issues in 2025 | 3 | GitHub search API, re-run 2026-10-19: `is:issue created:2025` = 24,389 | Correct | — |
| 66 | No DNR implementation uncloaks CNAMEs | 3 | ME 6.6 | Correct | — |
| 67 | Inherited-origin frames bypass blockers (Local Frames, CCS 2025) | 3 | GO 10.3 (title only; Medium) | Not checked | — |
| 68 | No list publishes per-entry evidence, an appeal path or a response time | 3 | LG 4 implications (absence) | Correct | — |
| 69 | Separator `^`: anything other than ASCII letter, digit, `_ - . %`, or end of URL, as DNR documents | 6.2 | DNR reference, quoted | Correct | — |
| 70 | "In every published dialect an exception in any list cancels a block from any other list" | 6.2 | DNR reference: across extensions "block > redirect or upgradeScheme > allow", so an allow in one extension never cancels another extension's block; ME 2.12: in Safari "exceptions in a different blocker from their block rule may not work" | Wrong | FC-M1 |
| 71 | `domain=` on imported lists matches the frame's or the top-level site's host, as NW 6.15 states | 6.2 | NW 6.15 "Partition semantics" | Correct | — |
| 72 | "Third-party" is relative to F, as NW 6.15 defines | 6.2 | NW 6.15 | Correct | — |
| 73 | A response-phase rule cannot stop the request reaching the server, as DNR documents | 6.2 | GO 1.6, quoting the DNR reference | Correct | — |
| 74 | Safari compiled 40,000 rules in 7.4–11.6 s on iPhones in 2018 | 6.3 | Snyder et al.: 7.4 s (iPhone X), 11.62 s (iPhone 6s) | Correct | — |
| 75 | Published per-request means are 5–20 µs, and no engine publishes a tail | 6.3 | ME 2: Brave 5.7 µs mean; Ghostery, uBO, ABP figures are medians; Snyder et al. publish a p90 (0.30 ms for an ABP-style JS matcher, ME 8.3) | Partly correct | FC-R4 |
| 76 | DNR's and Safari's caps stranded list authors and were raised only with vendors' data | 6.3 | HP 5.6 (DNR); HP 6.2 (Safari, AdGuard's account) | Correct | — |
| 77 | No cross-engine conformance suite exists | 6.3 | ME 7.1 (absence) | Correct | — |
| 78 | Snyder et al.'s hybrid matcher lets first requests through | 6.3 | ME 8.3 | Correct | — |
| 79 | Brave's 2026 correction "shows intermediates matter" | 6.4 | Brave privacy update 6, note of 11 Feb 2026: uncloaking is "applied to the initial and canonical domain names, but not any possible intermediate CNAME domains". It shows Brave does not match intermediates; it does not show they matter | Wrong | FC-M7 |
| 80 | A-record evasion is predicted and observed | 6.4 | TT 4.4 (Dimova: 35 publishers already did it) | Correct | — |
| 81 | WebKit compares the CNAME against the first party's and top frame's, so self-hosted CDNs are not penalized | 6.4 | HP 6.8 (WebKit blog 11338) | Correct | — |
| 82 | Cookies from cloaked responses capped at 7 days, Safari's figure | 6.4 | WebKit 11338: "caps the expiry of cookies set in ... CNAME-cloaked HTTP responses to 7 days" | Correct | — |
| 83 | Bot-detection and anti-fraud scripts protect user accounts and blocking raises challenge rates | 6.4 | TT 2.10 | Correct | — |
| 84 | The Nav-Tracking draft assumes third-party cookies are blocked; Izcal partitions them, "so its precondition always holds" | 6.5 | Draft §6: "written for the case where ... All third-party cookies are blocked. All cookies written by embedded third-party sites are partitioned. All storage ... is partitioned." Izcal partitions but does not block third-party cookies (NW-D7) | Wrong | FC-M3 |
| 85 | Draft values: client bounce 10 s, grace 1 h, activation lifetime 45 days; WebAuthn assertions count | 6.5 | Draft notes: "10 seconds", "1 hour", "45 days"; §6.2.2 Web Authentication monkey patch | Correct | — |
| 86 | The draft warns that deletion timing leaks interaction state | 6.5 | Draft: the global timer adds fuzz that "mitigates adversarial leaks of interaction and bounce times" | Correct | — |
| 87 | The draft has no consensus on its algorithm | 4.2, 6.5 | Draft: "does not yet reflect any consensus in the PrivacyCG" | Correct | — |
| 88 | "Safari's and Firefox's lists are not published" | 6.5 | TT 3.7 says Safari's list is not published; no row says Firefox's is unpublished (HP 7.6 quotes Firefox's examples) | Partly correct | FC-R1 |
| 89 | Safari users reported `window.location` losing parameters | 6.5 | HP 6.9 says developers reported it (Low) | Partly correct | FC-R1 |
| 90 | FP-Inspector: targeted per-script restriction broke about half as much as blanket API restriction | 6.6 | FP-Inspector §IV-B: targeted countermeasures "reduce breakage by a factor of 2"; blanket breaks over two-thirds, targeted none major on about 75% (61 breakage-prone sites) | Correct | — |
| 91 | Fingerprinting scripts use timing, frame-rate estimation, sensors and `canPlayType` | 6.6 | TT 2.5 (FP-Inspector §VI) | Correct | — |
| 92 | Disconnect and AdGuard exempt bot-detection and anti-fraud scripts | 6.6 | LG 1.7: AdGuard exempts anti-fraud that would break a site. LG 1.15: Disconnect has an "Anti-fraud" category; no row says it is exempt from blocking | Partly correct | FC-R3 |
| 93 | German courts treat anti-adblock walls as the publisher's legitimate response | 6.6 | CU 5.9 (BGH PM 78/2018) | Correct | — |
| 94 | Miners use Wasm and Stratum and throttle and randomize URLs | 6.6 | TT 7.1 (MineSweeper) | Correct | — |
| 95 | SugarCoat showed replacements can be generated from a policy | 6.6 | ME 4.9, CU 2.12 | Correct | — |
| 96 | Generic selector sheets cost about 4 MiB per document in 2014 Firefox before sharing | 6.7 | Nethercote: "an overhead of about 4 MiB per iframe"; fix saved 3.62 MiB per document | Correct | — |
| 97 | User-origin `!important` beats author `!important` | 6.7 | CSS Cascade 5 §6.4 (`#importance`) | Correct | — |
| 98 | Procedural operators run as page-injected script with MutationObservers whose cost no one publishes | 6.7 | ME 3.4, 3.6 (3.6 is inference, Low) | Correct | — |
| 99 | Scriptless list-fingerprinting works through what hiding changes (background-image and lazy-image fetches) | 6.7 | TT 8.4, 8.5 | Correct | — |
| 100 | The BGH's July 2025 remand leaves open whether changing DOM and CSSOM modifies a protected program | 5, 6.7 | vzbv: BGH 31.07.2025, I ZR 131/23, partly set aside and remanded to OLG Hamburg | Correct | — |
| 101 | Privacy Badger's floor of 3 sites | 6.8 | LG 1.19, HP 4.1 | Correct | — |
| 102 | Chrome heavy-ad evidence: 0.3% of ads cause 27–28% of ad data and CPU; 99.9th percentile | 5, 6.8 | Chromium blog 2020-05-14 (page source): "0.3% of ads exceed this threshold ... 27% of network data used by ads and 28% of all ad CPU usage" | Correct | — |
| 103 | Headless Chromium blocked on 15.2% of the Tranco top 10K against about 7% headed; correlated with CDN; 83% of 81 papers did not report block rates | 6.8 | ARES 2026 project page: 15.2%; others 6.8–7.2%; 83% of 81; per-provider rates | Correct | — |
| 104 | Third parties per site rose from 22 to 34 with 4 internal pages | 6.8 | TT 1.2 (Englehardt and Narayanan §5) | Not checked | — |
| 105 | Evasion of a content-feature classifier reached 96.6% with first-party collusion, against 8.3% for a flow-feature one | 6.8 | WebGraph: 96.62% (AdGraph, collusion); 8.34% (WebGraph) | Correct | — |
| 106 | Disconnect's, Tracker Radar's and Ghostery's entity data are non-commercially licensed | 6.8 | LG 1.13, 1.16, 1.25 (TrackerDB from a mirror, Medium) | Correct | — |
| 107 | A 2017 DMCA notice removed a domain from EasyList | 6.8 | LG 4.5 (TorrentFreak) | Correct | — |
| 108 | uBO adopted version-addressed diffs in 2023 to cut bandwidth | 6.9 | LG 6.2 (uBO 1.54.0 release notes) | Correct | — |
| 109 | EasyList's traffic rose 10–20× "from one browser re-downloading on every start" | 6.9 | AdGuard blog: 10–20×; an open-source Android browser "forked by a couple of other browsers that are very popular in India" downloading on every startup | Partly correct | FC-R8 |
| 110 | Firefox and Brave strip query and fragment from breakage reports | 6.11 | Brave wiki: "query parameters and fragments are omitted"; CU 4.1: Firefox strips the query string (fragment not stated) | Partly correct | FC-R1 |
| 111 | Brave keeps reports 30 days (basis for ≤ 30 days†) | 6.11 | Brave wiki: "deleted from Brave's servers after 30 days" | Correct | — |
| 112 | 25–44% of user-reported breakage appears only after interaction; CSS-hiding rules about 53% of rule-attributed breakage | 5, 6.11 | SINBAD: dynamic breakage 25% (EasyList) to 44% (uBlock); "≈ 53 % of rules causing breakage" | Correct | — |
| 113 | Breakage predictors with AUC 0.88 | 6.11 | CU 1.6 (Smith et al.) | Correct | — |
| 114 | Brave retired Strict fingerprinting mode at under 0.5% use | 6.12 | Brave privacy update 28: "Fewer than 0.5% of Brave users" (1.64) | Correct | — |
| 115 | Brave's sign-in permission is narrow and can be limited to the tab | 6.8, 6.12 | Brave privacy update 24 (Google login URLs only); CU 2.8 for the tab scope | Correct | — |
| 116 | Edge documents that request counts fall as protection rises | 4.4, 6.12 | CU 7.7 (Microsoft Learn) | Correct | — |
| 117 | SmartBlock's and uBO's catalogues are 14–30 scripts | 9.3 | CU 2.6: uBO 14 URL-specific surrogates; ME 4.9: uBO replacements for 27 scripts (SugarCoat); no source counts SmartBlock's (CU 2.1, GO 7.6: "no count published") | Partly correct | FC-R5 |
| 118 | Person-years for the added work: about 16–30† to Stage 1 exit with a ramp | 9.3 | 4–7 people for 36–48 months (NW-M5) is at most 12–28 person-years at full strength; a ramp lowers it. 16–30 exceeds the no-ramp maximum | Wrong | FC-M4 |

Additional checks graded inside the cross-reference table (Section 4) are not counted above.

**Faithfulness to the notes.** No number drifted between a note row and the report except where listed above (rows 4, 8, 25, 88, 92, 109, 117). Three report statements inherit an error from the note itself: row 84 (TT 3 implications: "the CG algorithm's precondition always holds"), row 79 (GO 10 implications: "Brave's 2026 correction shows the intermediates matter") and row 20 (ME 7.3 lists DNR, Firefox and AdGuard, and the report widened it to "every published dialect").

## 2. Link check

**External links in the report** (27 distinct; fetched 19 October 2026 with redirects followed):

| Link | Result |
|---|---|
| sysnet.ucsd.edu canvas-imc25.pdf; arxiv.org 2606.16720, 1506.04104; brave.com UID-smuggling PDF; webkit.org 9661, 3476; eff.org 2020/10; notify.cispa.uni-saarland.de double-edged PDF; armin.dev 2019/04 | 200 |
| fetch.spec.whatwg.org; url.spec.whatwg.org; html.spec.whatwg.org browsing-the-web; privacycg.github.io nav-tracking-mitigations; publicsuffix.org | 200 |
| w3.org TR css-cascade-5, selectors-4, CSP3, permissions-policy-1 | 200 |
| `w3.org/TR/css-cascade-5/#importance` | 200; `id="importance"` present |
| github.com gorhill/uBlock wiki Static-filter-syntax; adguard.com create-own-filters; developer.chrome.com DNR reference; developer.mozilla.org DNR | 200 |
| help.adblockplus.org `.../360062733293-How-to-write-filters` | 200 after 1 redirect to `help.adblockplus.org/adblock-plus-help-center/how-to-write-filters` |
| rfc-editor.org rfc1034, rfc6962, rfc9458 | 200 after 1 redirect each to `rfc-editor.org/info/rfcNNNN/` |

Dead links: **0**. Broken fragments: **0**.

**Internal link definitions** (22, resolved relative to `reports/`): HP, TT, ME, LG, CU, GO, FR (the seven notes), H2, H3, S1, NW, ST, FP, JS, GR, TX, MD, LY, DT, UI (the specs) and RN, LA (`research_notes/Phase 2 reconciliation/`): **all exist**. Every `[X][X]` use has a definition and every definition is used.

## 3. M-1 part 3: source-code check

**Report.** No link points to source code. The one GitHub link is the uBO wiki's "Static filter syntax", which is prose documentation (allowed).

**Notes (288 distinct URLs screened).** No link to `source.chromium.org`, `chromium.googlesource.com`, `searchfox.org`, `hg.mozilla.org`, `phabricator`, `codereview`, a `/blob/`, `/tree/`, `/commit/` or `/pull/` path on GitHub, or a code file extension. No note links Brave's `adblock-rust`, uBO's code, Ghostery's engine, AdGuard CoreLibs or WebKit's content-extension compiler. ME's header records that search results pointing at WebKit compiler source were not opened, and LG's header that an ABP Rietveld code review was not opened.

Borderline links, with judgement:

| Link (note) | What it is | Judgement |
|---|---|---|
| `git.nixnet.services/curben/urlhaus-filter/commit/ebff982...` (LG 2.5) | A **commit page** in a filter-list (data) repository, cited for its commit message labelling Tranco "MIT" | Not engine or blocker source, but a commit page shows a diff, which M-1 part 3 names. Replace or drop (FC-R11); the claim is already graded Low and is not used by the report |
| `bugzilla.mozilla.org/show_bug.cgi?id=1210192` (HP 3.7) | Bug "Update Disconnect GPL notice in about:license"; its one attachment is a MozReview code-review request (licence text, not engine code) | Issue discussion is allowed; acceptable, but the cited fact (a GPL-notice discussion) does not need the bug. Keep with care, or cite the Mozilla blog only (FC-R11) |
| `github.com/gorhill/uBlock/releases/tag/1.46.0` and `/1.54.0` (ME 4.3, LG 6.2) | GitHub release pages: release-note prose, plus source-archive download links | Allowed as release notes; nothing was taken from the archives |
| `github.com/AdguardTeam/DiffBuilder` (LG 6.3) | Front page of a **code** repository (a tool), read for its README | Allowed by the README-prose rule, but it is a blocker vendor's tool repository; prefer the `ameshkov/diffupdates` spec alone for the 7-day retention, or mark the README as the only part read (it is) |
| `raw.githubusercontent.com/duckduckgo/tracker-radar/main/docs/DATA_MODEL.md` (LG 1.12, 3.2) | Raw Markdown documentation in a data repository | Allowed: prose documentation of a dataset, not code |
| Repository front pages of list (data) repositories: easylist, uAssets, AdguardFilters, brave/adblock-lists, StevenBlack/hosts, disconnect-tracking-protection, tracker-blocklists, tracker-radar | READMEs and licence labels | Allowed (data, README prose) |

**Values derived from source.** None found. Rows whose values exist only in source are listed as black-box questions (HP 17, ME 10, GO question 10), and the report says no design depends on them.

## 4. Cross-references to other Izcal specs

| Reference in the report | Checked in | Result |
|---|---|---|
| NW 6.15: matcher L5 tier P, sealed automaton per partition service, ≤ 256 KiB† charged, p99 ≤ 20 µs†, cosmetic per-site subsets | `Izcal networking.md` 6.15 | Correct |
| NW-22: blocked request byte-identical to other failures | NW-22 row | Correct |
| NW-D17: system service only, OHTTP for user-specific requests, public files fetched plainly | NW-D17 row | Correct |
| NW-D26: Izcal builds its own lists; first tracking list about 1 year† after the track starts | NW-D26 row and 6.15 "Cost" | Correct |
| NW-54: evidence per entry, ≥ 90%†, ≤ 1%†, triage ≤ 7 days† | NW-54 row | Correct |
| NW 6.15 breakage report sends "only the URL and the rules that fired" | NW 6.15 | Correct |
| NW 9.3: Content blocking 2–3†; Lists 4–7† with 3–6† for block lists | NW 9.3 table | Correct |
| NW-M4 at months 30–36†; NW-M0 to NW-M5 36–48 months† | NW 9.2 duration paragraph | Correct |
| NW 6.17 daily† wake-grid list checks, deferred on battery saver | NW 6.17 | Correct |
| NW 6.18 decision log records the content-blocking rule; "who received what"; FP-D18 24 h† detail, cleared with history, memory-only in private | NW 6.18; FP-D18 row | Correct |
| NW 6.19 IIP scoped, audited interception | NW 6.19 | Correct |
| NW-D3 container service owns first-party jars; NW-D7 partition key (T, C, F, X, N, R, n); NW-D8 own stub; NW-D18 no cross-container side effects; NW-D25 12-week re-check; NW-36 quiet browser | Respective rows | Correct |
| FP-28 p99 ≤ 20 µs†, 0 allocations | FP-28 row | Correct |
| FP-28 "Matcher ≤ 8 MiB†" | FP-28 row; FP 6.11: "matcher ≤ 8 MiB† **charged** in the network service for the default lists" | **Changed without an amendment**: CB-18 and 6.3 turn it into "compiled default bundle ≤ 8 MiB† (clean, shared)" plus ≤ 256 KiB† charged; 9.5's FP row does not say so (FC-M8) |
| FP 6.11 strictest-tier utility process, cosmetic subset ≤ 256 KiB†, "List governance ... belong to the shell and policy work" | FP 6.11 | Correct (the quoted sentence exists verbatim) |
| FP-D7 script-attributed interventions on sites where blocking is off; CPU, memory-time and wakeup thresholds at the 99.9th percentile; calibration corpus FP-M5 | FP 6.14, FP-D7, FP-M5 | Correct |
| FP-D16 regenerable-cache class includes compiled filter automata | FP-D16 row | Correct |
| FP-D9 cited for "No telemetry" | FP-D9 row: corpus results **plus opt-in aggregates** | Partly correct (FC-R6) |
| ST 6.16 cosmetic rules as user-origin sheet ≤ 256 KiB†, PRIV-2 global-sheet case; P21 | ST 6.16; reconciliation P21 | Correct |
| GR-D12: no noise on 2D canvas readback; WebGL and WebGPU noise once where pixels leave | GR-D12 row; GR 9.5 D-S1 row | Correct |
| TX-D4: `measureText` without noise | TX-D4 row | Correct |
| MD-D14: no seeded noise on Izcal-rendered audio (once MD-30 passes) | MD-D14 row | Correct (the MD-30 condition is omitted; acceptable) |
| LY-D11: Izcal scrollbar and form-control metrics, no OS metric in layout | LY-D11 row | Correct |
| D-S1 hybrid, seeded per (site, container, session); uniformity opt-in | Stage 1 Section 6 and D-S1 row | Correct |
| D-S3 no extensions in Stage 1; Stage 2 Wasmtime | D-S3 row | Correct |
| D-S8 funding options include search revenue share | D-S8 row | Correct |
| D-S10 no code from another browser | D-S10 row | Correct |
| Stage 1 13.2 TUF-based update client with signed metadata, rollback and freeze protection, key rotation | Stage 1 13.2 "Updates" row | Correct |
| Stage 1 13.2 iOS out of scope; 13.1a protected-media host blocks through `WKContentRuleList` | Stage 1 13.2, 13.1a | Correct |
| Stage 1 Section 6 rows "Built-in content blocking", "Containers" (block-list set per container), "Cookie-banner handling", "Extensions", GPC sent by default | Stage 1 Section 6 | Correct; the cookie-banner row says "rule-based auto-reject", which CB-D22 changes to hide-first, and 9.5 lists that amendment |
| HGBA-3 P2 scope note classes (a)–(c) | HGBA-3 Section 2 | Correct |
| PRIV-1: challenge rate with defaults vs Chrome and Firefox | PRIV-1 row | Correct |
| PRIV-2's rule "move the responsible surface to normalization" (CB-D3 reversal) | PRIV-2 row: "If (a) fails, the surface responsible is found by ablation and moved from noise to normalization" | Correct |
| "PRIV-2's surface list", "every surface PRIV-2 lists" (6.6, CB-12, I5) | PRIV-2 row | **Does not exist**: PRIV-2 names libraries and discrete surfaces in general, with no surface list; the inventory is this spec's proposed amendment (9.5) (FC-M9) |
| DT-1(c): divergences flagged, nothing past one shown as exact | DT-1 row | Correct |
| DT-10 redaction of recordings | DT-10 row | Correct |
| DT-D5 session rules for network views | DevTools spec DT-D5 | Correct |
| Shell UI 4.3 "Blocked here 23 trackers, 4 ads" | Shell UI 4.3 | Correct |
| HGBA-2: Brave's `adblock` excluded by name | HGBA-2 revision 2.3.0 (not 2.3.2) | Partly correct (FC-R12) |
| [RN] Section 7: matcher and lists counted only in NW | Reconciliation Section 7 says the lists track exists only in NW; the matcher is in P24 (Section 2) | Partly correct (FC-R7) |
| LA 2.5 and OD-L5; OD-L2 test-only | Licence audit | Correct |
| G80–G86 | No other spec uses G79–G89; frame says G1–G78 are taken | Correct (G79 is skipped, harmless) |

## 5. Arithmetic

- Added staffing rows: minima 1 + 1 + 0.5 + 0.5 + 0.5 + 0.5 = 4; maxima 1.5 + 1.5 + 1 + 1 + 1 + 1 = 7. **4–7† correct.**
- NW share 2–3 + 3–6 = 5–9; total 5–9 + 4–7 = 9–16. **Correct.**
- Non-engineers: 1.5–3 (half of NW's 3–6 list share) + 0.5–1 (governance) = 2–4. **Correct.**
- Person-years "4–7† a year once staffed": correct. "About 16–30† to Stage 1 exit with a ramp": **wrong**. Full strength for 3–4 years (36–48 months) is 12–28; a one-year linear ramp from zero gives (0.5 + 2 to 3 years) × 4–7 = 10–24.5. See FC-M4.
- Budgets: 6.3, CB-18 and CB-23 agree with each other (≤ 2 s†, ≤ 10 s† per 300,000 rules, ≤ 256 MiB† compiler, ≤ 8 MiB† default bundle, ≤ 32 MiB† per container, ≤ 256 KiB† charged, ≤ 256 KiB† cosmetic). CB-D15's 32 MiB† matches 6.3.

## 6. Must-fix

- **FC-M1 (6.2, "Why list-scoped exceptions").** Replace "In every published dialect an exception in any list cancels a block from any other list." with: "In ABP, uBO and AdGuard, an exception in any enabled list cancels a block from any other list ([ME 1][ME]). The declarative APIs already scope exceptions: in Chrome's DNR an `allow` in one extension never overrides another extension's `block` ([DNR](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest)), and in Safari an exception in a different content blocker from its block rule may not apply ([ME 2.12][ME])." Then let the rest of the paragraph present list scoping as following DNR's and Safari's precedent.
- **FC-M2 (1, engineers row; 3, "Correctness").** Replace "same-priority order is undefined in every published dialect" with "same-priority order is undefined in DNR and AdGuard, ordered by ruleset kind in Firefox, order-defined in Safari's JSON, and not documented by ABP or uBO ([ME 1.7, 1.9, 1.12, 1.14, 7.3][ME])", and in Section 3 replace "undefined in DNR, Firefox and AdGuard" with "undefined in DNR and AdGuard and only partly defined in Firefox".
- **FC-M3 (6.5, first bullet; also TT 3 implications).** Replace "The draft assumes third-party cookies are blocked; Izcal partitions them by default, so its precondition always holds." with: "The draft is written for browsers that block all third-party cookies and partition third-party cookies and storage. Izcal partitions both by default (NW-D7) but does not block partitioned third-party cookies, so it meets two of the three conditions. A bounce tracker can still use its partitioned third-party jar, but that jar cannot join sites, so Izcal applies the algorithm to first-party state and records this deviation from the draft." The design critic should confirm the deviation is sound.
- **FC-M4 (9.3, Person-years).** Replace "about 16–30† to Stage 1 exit with a ramp" with "about 10–25† to Stage 1 exit (36–48 months†) with a one-year linear ramp; 12–28† at full strength".
- **FC-M5 (summary).** Replace "a user's choice of filter lists identifies 84% of lists with 45 probe rules and no script" with "scriptless probes identify 84% of a user's filter lists, and 45 probe rules shrink a user's anonymity set to a median of 48 users".
- **FC-M6 (summary).** Replace "first-party and server-side tracking on more than 54% of sites, of which the dominant lists match 27.5%" with "first-party and server-side tracking on more than 54% of sites, whose scripts EasyList and EasyPrivacy match only 27.5% of the time".
- **FC-M7 (6.4, first paragraph; also GO 10 implications).** Replace "because Brave's 2026 correction shows intermediates matter ([GO 10.2][GO])" with "because a tracker can sit at any hop, and the one browser that publishes its rule matches only the first and canonical names (Brave's correction of February 2026, [GO 10.2][GO]); CB-13 counts how many requests only an intermediate name uncloaks".
- **FC-M8 (6.3, CB-18, 9.5).** FP-28 and FP 6.11 set "matcher ≤ 8 MiB† charged in the network service". This spec makes it "compiled default bundle ≤ 8 MiB† (clean, shared)" plus "≤ 256 KiB† charged per partition service". Add to 9.5's resource-footprint row: "FP-28's and 6.11's 'matcher ≤ 8 MiB† charged' becomes 'compiled default bundle ≤ 8 MiB†, clean and shared; charged matcher state ≤ 256 KiB† per partition service (NW 6.15)'", and say in CB-18 that it restates FP-28 as amended.
- **FC-M9 (6.6, I5, CB-12, CB-D3).** PRIV-2 has no surface list. Replace "PRIV-2's surface list must cover", "the surfaces PRIV-2 lists" and "every surface PRIV-2 lists" with "the surface inventory this spec adds to PRIV-2 (Section 9.5: [TT 2.5][TT] plus D-S1's discrete and readback surfaces)".

## 7. Recommended

- **FC-R1 (6.5, 6.11).** "Safari's and Firefox's lists are not published" → "Safari's list is not published ([TT 3.7][TT])". "Safari's users reported" → "developers reported (Low)". "as Firefox and Brave strip them" → "as Brave strips both and Firefox strips the query ([CU 4.1, 4.2][CU])".
- **FC-R2 (2.1, EasyList row).** "volunteer-run (4 named maintainers today)" → "hand-curated by 4 named maintainers, one funded by Brave since 2019 ([LG 1.1, 1.4][LG])".
- **FC-R3 (6.6, point 2).** "Disconnect and AdGuard exempt them too" → "AdGuard exempts anti-fraud scripts that would break a site, and Disconnect files them in a separate Anti-fraud category ([LG 1.7, 1.15][LG])".
- **FC-R4 (6.3).** "Published per-request means are 5–20 µs and no engine publishes a tail" → "Published per-request means and medians are 5–20 µs; the only published tail is a p90 for a JavaScript matcher in a 2020 study ([ME 2, 8.3][ME]), and no shipping engine publishes one".
- **FC-R5 (9.3, surrogates row).** "SmartBlock's and uBO's catalogues are 14–30 scripts" → "uBO's surrogate catalogue is 14–27 scripts ([CU 2.6][CU], [ME 4.9][ME]); Firefox publishes no SmartBlock count".
- **FC-R6 (6.16, Privacy).** "No telemetry (FP-D9)" → "No default-on telemetry (NW 6.19; FP-D9 allows only opt-in aggregates under a privacy review)".
- **FC-R7 (9.3).** "([RN] Section 7)" → "([RN] P24 and Section 7)".
- **FC-R8 (6.9).** "from one browser re-downloading on every start" → "from an open-source Android browser and its forks re-downloading on every start".
- **FC-R9 (summary).** "both projects moved to uniform rules and one list shipped to everyone" → "WebKit moved to uniform, classifier-independent rules and Privacy Badger to one pre-trained list shipped to everyone".
- **FC-R10 (3, cryptojacking row; 11.2).** Add "Cryptojacking collapsed after Coinhive closed (2019) ([TT 7.5][TT]) | Not checked; no prevalence measurement after 2020" to 11.2.
- **FC-R11 (notes, M-1 part 3).** In LG 2.5 drop or replace the `git.nixnet.services` commit link (a commit page shows a diff); in HP 3.7 the Bugzilla link may stay as issue discussion, but the Mozilla blog alone supports the row.
- **FC-R12 (1, "What Izcal's specs already decide").** "HGBA-2 2.3.2 M-1: ... Brave's `adblock` is excluded by name" → "HGBA-2 M-1 (2.3.0 and 2.3.2): ... Brave's `adblock` is excluded by name (2.3.0)".
- **FC-R13 (links).** Update the four redirecting URLs to their final targets: `https://help.adblockplus.org/adblock-plus-help-center/how-to-write-filters` and `https://www.rfc-editor.org/info/rfc1034/` (and the same form for 6962 and 9458), or keep them, since they redirect cleanly.
- **FC-R14 (11.2).** Add the two report statements that inherit note errors (FC-M3, FC-M7) to the flagged-claims table until fixed, and correct TT 3 and GO 10 implications in the notes so the evidence trail matches the spec.
