# Izcal content blocking: history and postmortems

Compiled 19 October 2026. Primary sources were preferred (papers, specifications, maintainers' posts, release notes, court press summaries, issue trackers, official documentation).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data, or a well-known fact not re-read this session; **Low** = secondary source only.

**M-1 part 3.** No engine or blocker source code was read, linked or cited. Every source below is prose: blog posts, papers, release notes, documentation and wiki pages that describe behaviour, court summaries, bug-tracker discussion and press. Where a value seems to exist only in source code, it is listed in Section 17 as a question for black-box measurement.

**Search budget.** The shared web-search budget ran out partway through. After that, only known primary pages were read directly. Rows that could not be checked are graded Medium or Low and say why.

---

## 0. Chronology

| Year | Project | What it did | Why that way | Limits | Fate and why | What survived |
|---|---|---|---|---|---|---|
| 1990s–2000s | **Hosts files** (Fravia's pages; later community hosts lists) | Mapped ad and tracker hostnames to a dead address in the OS hosts file | No browser extension APIs existed. The hosts file was the only interception point a user controlled | Whole hostnames only, no paths or wildcards, no page context, edited by hand. Malware also rewrote it | Still used as a list format. Superseded for browsing by in-browser blockers and DNS sinkholes | The **domain list** as the simplest rule unit |
| 2002 | **Adblock** (Henrik Aasted Sørensen, Firefox/Mozilla) | Hid and blocked page elements by URL pattern | Mozilla's XUL extension model let an add-on see every load | One author; the project went quiet ("practically abandoned", per EasyList) | Forked and replaced by Adblock Plus | The idea of URL-pattern rules |
| 2005 | **NoScript** (Giorgio Maone) | Default-deny JavaScript, with per-site allow | Security, not ads: most browser exploits then needed script | Breaks most of the modern web; needs constant user decisions | Rewritten as a WebExtension in Nov 2017 when Firefox dropped legacy add-ons; some features were lost. Still maintained (2026) | **Per-site trust** and default-deny as an expert mode |
| 2005 | **EasyList** (Rick Petnel, "rick752") | A shared, subscribable filter list | Users should not each write their own rules. One curated list served everyone | Volunteer-run; English-centric; grew to tens of thousands of rules, many stale | Lives on. Recommended by ABP from 2006; the maintainer died in 2009 and others took over; Fanboy's list merged in 2013; EasyPrivacy split off (2008) | **Separating the list from the engine**: the dominant model to this day |
| 2006 | **Adblock Plus** (Wladimir Palant; later eyeo) | Rewrite of Adblock with filter subscriptions and element hiding | Made lists the product and the extension a generic engine | Slow matching in early versions; memory per tab; later the business model | Large install base. Funded by Acceptable Ads (2011–) and has been in court since 2014 | The **ABP filter syntax**, the de facto standard that every later blocker reads |
| 2009 | **Ghostery** (David Cancel; then Evidon; Cliqz 2017; Ghostery GmbH 2020) | Showed and blocked trackers by company | Visibility first: "what tracking technologies were present" | GhostRank (opt-in) sent data about blocked ads to the ad industry, which was a conflict of interest | Sold to Cliqz in 2017. Cliqz closed in April 2020 and former Cliqz leaders took Ghostery on. Went closed-source in 2016, open again in 2018 | **Tracker identity by company**; WhoTracks.me data |
| 2011 | **Disconnect** (Brian Kennish, ex-Google/DoubleClick) | A tracker list organised by entity and category | Trackers, not ads; categories (social, analytics, advertising) let browsers block selectively | Curated by hand; GPL notice on the list caused licensing discussion at Mozilla | The list powers Firefox's tracking protection (2015–) and Edge's (2019–) | **Category-structured tracker lists** used as browser defaults |
| 2011 | **Acceptable Ads** (eyeo) | Let "non-intrusive" ads through by default; large entities pay | A business model and an argument that the web needs some ads | Paid allowlisting is a conflict of interest, and the default-on choice is eyeo's, not the user's | Governance moved to an "independent" committee in 2017; repeated German litigation (Section 4) | Ad **quality criteria** as an idea; the controversy is the lesson |
| 2014 | **Privacy Badger** (EFF) | Learned trackers from behaviour, without a list | Lists are subjective and stale; behaviour (cross-site cookies, supercookies, canvas) is objective | Per-user learned state is itself observable | Local learning **off by default in Oct 2020** after Google Security Team disclosures; it now ships a pre-trained list (Badger Sett) | **Behavioural classification run centrally** as a crawler, not per user |
| 2014 | **uBlock / uBlock Origin** (Raymond Hill) | Fast, low-memory blocker with ABP syntax plus extensions | Showed that matching cost and memory, not lists, made blockers heavy | Depends on a blocking request-interception API | Thrives on Firefox; disabled on Chrome 2024–2025 by MV3; uBO Lite is the declarative successor | Token-indexed matching (later adopted by Brave); the **extended syntax** ($removeparam, scriptlets, redirect resources) |
| 2014 | **Pi-hole** (Jacob Salmela) | DNS sinkhole for a whole network | Covers devices that cannot run an extension (TVs, phones, apps) | Domain-level only; cannot block same-domain ads (for example YouTube); bypassed by apps with hard-coded DNS or DoH | Popular in homes; DNS services (NextDNS 2019 and others) commercialised it | **Network-wide domain blocking** as a separate layer |
| 2015 | **Safari Content Blockers** (WebKit, Safari 9 / iOS 9) | Declarative JSON rules compiled ahead of time into bytecode | Performance, energy, memory, and privacy: the extension never sees URLs | Regex subset; per-blocker rule cap (50,000, later 150,000); no scriptlets; no request-time logic | Still the only content-blocking API on iOS; Brave iOS and AdGuard live within it | **Compile rules ahead, match in the engine**, and the privacy argument for declarative APIs |
| 2015 | **Firefox Tracking Protection** (Kontaxis and Chew) | Blocked Disconnect-listed tracker domains (Private Browsing default from Firefox 42) | Measured: 44% median page-load gain, 39% less data, 67.5% fewer cookies on news sites | List-based; breakage in Strict | Grew into ETP | **Measured, published effect of blocking** |
| 2017 | **ITP** (WebKit) | On-device ML classifier; purge and partition storage of classified domains | No list to maintain; works for every user without a crowd | The classification state could be detected (Google, 2019) | Moved step by step to **classifier-independent** rules (full third-party cookie blocking, March 2020); the classifier now drives narrower defences | **Caps on script-writable storage**, link-decoration caps, CNAME and bounce defences |
| 2018 | **Chrome Better Ads filter** | Blocked ads on sites that fail the Coalition for Better Ads standards, using EasyList-based patterns | Penalise the worst ad formats without blocking all ads (Google is an ad company) | Only for failing sites; criteria set by an industry coalition | Still on | **Site-level enforcement** with a site-owner report and re-review |
| 2018–19 | **Firefox ETP** | Default blocking of tracking cookies, then fingerprinters and cryptominers | "Changing our approach": protection by default, not by expert opt-in | Disconnect list dependency; breakage | On by default for all from 2019; TCP (2021), SmartBlock (2021), query stripping (2022), bounce tracking protection (2024) followed | **Default-on, list-driven, with shims and partitioning** |
| 2018–2025 | **Chrome Manifest V3 / declarativeNetRequest** | Replaced blocking webRequest with a capped declarative rule API | Stated reasons: security, privacy, performance of extensions | Rule caps; no top-context matching (until partial support in Chromium 145); list updates tied to store releases; many uBO filters not convertible | MV2 disabled for all users in Chrome 138 (July 2025) and removed in 139; store removal 31 Aug 2026 | DNR as a cross-browser WebExtensions baseline; uBO Lite |
| 2019 | **Brave adblock-rust** | Rust rewrite of Brave's C++ engine, 69× faster on average | The Bloom-filter design assumed blocks are rare; 39% of requests were blocked | Memory grew with lists | 2025–26: moved to a zero-copy binary format, **−75% memory (~45 MB)** | **Token-indexed, zero-copy, mapped rule store** inside the engine |
| 2019 | **Edge tracking prevention** | Disconnect lists in three levels, relaxed by site engagement in Balanced | Compatibility: unblock organisations the user has a relationship with | Engagement scores are per-user state | Still on | Levels; **engagement-based relaxation** |
| 2020 | **DuckDuckGo Tracker Radar** | Crawler-built tracker data set with entity, prevalence and API use | Hand lists stale and break sites | Data licence non-commercial; a 2022 contractual exception for Microsoft | Still the base of DDG's apps | **Crawler-derived, evidence-attached lists** |
| 2020–2025 | **Chrome third-party cookie phase-out** | Promised removal "within two years"; tested on 1% in Jan 2024 | Privacy Sandbox replacements first | Competition-law oversight; ad-industry dependence | Reversed: July 2024 "user choice", April 2025 no prompt; most Privacy Sandbox APIs retired Oct 2025 for "low adoption" | CHIPS, FedCM, Private State Tokens |

---

## 1. Origins: Adblock, Adblock Plus and EasyList

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | The original **Adblock** was created in 2002 by Henrik Aasted Sørensen. Wladimir Palant rewrote it as **Adblock Plus**, releasing 0.6 in January 2006. ABP blocks requests by source and context, and hides elements ("element hiding"). | 2002; Jan 2006 | [Adblock Plus, Wikipedia](https://en.wikipedia.org/wiki/Adblock_Plus) | read 2026-10-19 | Medium |
| 1.2 | **EasyList** was created in spring 2005 by Rick Petnel ("rick752") as a simpler subscription for Adblock, whose owner had "practically abandoned" it, and later for ABP, which officially recommended it. | 2005 | [EasyList: about](https://easylist.to/pages/about.html) (via search snippet) | — | Medium |
| 1.3 | Petnel died in 2009, aged 56. Palant appointed a new maintainer. The list depended on one volunteer. | 2009 | [Slashdot, 2009-04-06](https://tech.slashdot.org/story/09/04/06/148207/ad-block-plus-filter-maintainer-rick752-dies-at-56) | 2009-04-06 | Medium |
| 1.4 | **Fanboy's List merged into EasyList in early 2013**; EasyPrivacy was split off in 2008. EasyList accumulates **stale rules**: Brave measured the mounting cost. | 2013 | [Brave, "The mounting cost of stale ad blocking rules"](https://brave.com/blog/the-mounting-cost-of-stale-ad-blocking-rules/) (snippet) | — | Medium |
| 1.5 | Chrome's own **ad filter (2018)** used patterns "based on the public EasyList filter rules": a single volunteer list ended up as input to Google's browser. | 2018-02-15 launch | [Chromium Blog, Feb 2018](https://blog.chromium.org/2018/02) (via snippets) | 2018-02 | Medium |

**Why that way.** In 2002–2006 the only interception point was an extension that saw every load. Lists were separated from engines so that one curator could serve millions of users. **What survived** is the ABP syntax and the list/engine split. **What failed** is governance: volunteer lists with no evidence attached to entries, no review record and no removal of stale rules.

**Implications:** Izcal's own lists (NW-D26) should keep the list/engine split but fix what EasyList never had: evidence per entry, a review record, and expiry of entries that no longer fire on the crawl. Izcal should read ABP syntax for user-added lists, because 20 years of lists use it.

---

## 2. Acceptable Ads, and the German court cases

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | Acceptable Ads was introduced in **ABP 2.0** (Dec 2011): some ads allowed **by default**, and large companies pay to be allowlisted. It drew "significant criticism". | v2.0 | [Adblock Plus, Wikipedia](https://en.wikipedia.org/wiki/Adblock_Plus) | read 2026-10-19 | Medium |
| 2.2 | The fee applies to "large entities" (more than **10 million additional impressions/month** from the programme) and "normally represents **30%** of the additional revenue". Around **90%** of licences are free. Every allowlisted entity must meet the same criteria. | 10 M; 30%; 90% | [About Adblock Plus](https://adblockplus.org/about) (snippet); [TechCrunch 2016-05-09](https://techcrunch.com/2016/05/09/adblock-plus-closes-in-on-a-billion-downloads) | 2016–2026 | Medium |
| 2.3 | In **2017** governance moved to the **Acceptable Ads Committee**, "online industry experts from companies and nonprofit organizations". Acceptable Ads is "on by default". | 2017 | [Allowing acceptable ads](https://adblockplus.org/en/acceptable-ads) | read 2026-10-19 | High |
| 2.4 | **OLG Köln (June 2016)**: ad blocking is lawful, but paid allowlisting was an **aggressive commercial practice** (§ 4a UWG); eyeo had to admit Springer free of charge. | — | [netzpolitik.org, 2016](https://netzpolitik.org/2016/oberlandesgericht-koeln-adblocking-legal-bezahltes-whitelisting-verboten/) | 2016-06 | Medium |
| 2.5 | **BGH, 19 April 2018, I ZR 154/16** ("Werbeblocker II"): offering ABP, including paid allowlisting, is **not unfair**. Using it is "the autonomous decision of internet users". This reversed OLG Köln. Publishers may refuse access to users with blockers. | — | [Fieldfisher](https://www.fieldfisher.com/en/services/intellectual-property/intellectual-property-blog/german-federal-court-of-justice-ruling-ad-blockers-are-allowed); [otto-schmidt.de](https://www.otto-schmidt.de/news/wirtschaftsrecht/adblock-plus-angebot-des-werbeblockers-nicht-wettbewerbswidrig-2018-04-19.html); BGH PM 078/2018 (reproduced, not read) | 2018-04-19 | Medium |
| 2.6 | Springer's constitutional complaint was not accepted (1 BvR 921/19, 22 Aug 2019). This comes from a single law-firm article. *Conflict:* Wikipedia says the "Federal Constitutional Court ruled in Eyeo's favor in April 2018", which mixes up the BGH and the BVerfG. | — | [wbs.legal](https://www.wbs.legal/wettbewerbsrecht/adblock-plus-bald-illegal-bgh-entscheidet-ueber-verkaufsverbot-23479/) | 2019-08-22 | Low |
| 2.7 | **Copyright track.** LG Hamburg (2022) and OLG Hamburg (2023) dismissed Springer's claim that ABP makes an unauthorised **modification of a computer program** (§ 69c No. 2 UrhG). The OLG held that the blocker acts on the browser's temporary data structures (DOM, CSSOM), not on the program's substance. | — | [heise, 2024-07-26](https://heise.de/en/news/Axel-Springer-vs-Adblock-Plus-Federal-Court-of-Justice-awaits-ECJ-ruling-9813824.html) | 2024-07-26 | High |
| 2.8 | **BGH, 31 July 2025, I ZR 131/23** ("Werbeblocker IV"): partly set aside the OLG Hamburg judgment and **remanded** it. The OLG had not ruled out interference with the program's protected scope, had not identified the protected subject matter, and had not dealt with Springer's argument that DOM and CSSOM structures are executable control instructions ("bytecode"), a protected form of expression. Open: whether that bytecode is protected, and whether the blocker changes it. | — | [vzbv summary](https://www.vzbv.de/urteile/zur-urheberrechtlichen-zulaessigkeit-von-werbeblockern); BGH PM 148/2025 (via [jura.cc](https://www.jura.cc/rechtstipps/urheberrechtliche-zulaessigkeit-eines-werbeblockers/), snippet) | 2025-07-31 | High (holding); Medium (PM number) |
| 2.9 | Mozilla's legal director warned that such a requirement could affect "not only ad blockers, but also other extensions that change the display of pages". The remand is in the Hamburg courts, expected to take 1–2 years. | — | [CyberInsider](https://cyberinsider.com/ad-blockers-face-possible-ban-in-germany-after-court-ruling/) | 2025-08 | Low |
| 2.10 | *Unverified:* one secondary source mentions a **2025 OLG Köln** competition ruling against paid allowlisting (case 6 U 149/15). That number is the 2016 case, so this is probably a confusion. No primary source was found. | — | [CyberInsider](https://cyberinsider.com/ad-blockers-face-possible-ban-in-germany-after-court-ruling/) | — | Low |

**Why it was built that way.** eyeo needed revenue and argued that blocking every ad would starve the web. Making the allowlist default-on maximised its value to advertisers. **Fate.** The business survived competition law (BGH 2018), but the trust cost was permanent: uBlock Origin's refusal of donations, Brave's "block ads to block tracking" policy and many users' moves away from ABP are reactions to it. The copyright question (2025) is open, and it targets the **mechanism** itself (changing the DOM and CSSOM), not the business.

**Implications:** Izcal must never take money for list entries or exceptions. Every exception gets the same evidence and review as a block, and is published (NW-54). The BGH remand is a legal risk for **cosmetic filtering** in Germany. Izcal's design, cosmetic rules as a **user-origin style sheet** applied by the engine (Style 6.16), is a different mechanism from an extension rewriting the DOM, and the spec should record it as such. Legal review before Izcal ships in Germany is a milestone, not a footnote.

---

## 3. NoScript, Ghostery, Disconnect

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **NoScript** (Giorgio Maone, released 13 May 2005) blocked scripts from untrusted sites "pre-emptively", then grew into a security suite (XSS, CSRF, clickjacking). | 2005 | [NoScript, Wikipedia](https://en.wikipedia.org/wiki/NoScript); [Mozilla Add-ons blog, 2017-08-01](https://blog.mozilla.org/addons/2017/08/01/noscripts-migration-to-webextensions-apis/) | 2017 | Medium |
| 3.2 | NoScript 10 (announced 14 Nov 2017) was a WebExtension rewrite because "legacy add-on technology… is going to be banned very soon". Users reported lost functionality. | Nov 2017 | Same | 2017 | Medium |
| 3.3 | **Ghostery** began as David Cancel's weekend project to see "what tracking technologies were present". About a year later it went to **Evidon**. It joined **Cliqz** (Munich) in 2017. | — | [Ghostery, "10 years and counting"](https://www.ghostery.com/blog/10-years-and-counting) | 2019 | High |
| 3.4 | **GhostRank** (opt-in) passed data on ads users saw and blocked to advertisers, "who could change their ads to avoid being blocked". Critics said Evidon had "a financial incentive to encourage the program's adoption". | — | [Ghostery, Wikipedia](https://en.wikipedia.org/wiki/Ghostery) | read 2026-10-19 | Low |
| 3.5 | Cliqz GmbH closed (announced 29 April 2020). Ghostery GmbH, run by former Cliqz people, continues the business. Ghostery went closed-source in 2016 and open again in March 2018. | 2020 | [Ghostery, Wikipedia](https://en.wikipedia.org/wiki/Ghostery) | read 2026-10-19 | Low |
| 3.6 | **Disconnect** was founded in 2011 by Brian Kennish, a former Google and DoubleClick engineer. Its lists power Firefox's tracking protection: "350M+ users protected by default". | 2011 | [Disconnect blog](https://blog.disconnect.me/our-tracker-protection-lists-now-power-privacy-for-300-million-firefox-users/) (snippet) | — | Medium |
| 3.7 | Mozilla's Firefox 42 Private Browsing tracker list (Dec 2015) was "based on Disconnect's blocklist", and Mozilla called Disconnect "our partner". A Mozilla bug discussed the list's **GPL** notice versus Firefox's licence. | 2015 | [Mozilla blog, 2015-12-15](https://blog.mozilla.org/blog/2015/12/15/firefox-gives-you-more-control-over-your-data-in-private-browsing/); [Bugzilla 1210192](https://bugzilla.mozilla.org/show_bug.cgi?id=1210192) (snippet) | 2015 | Medium |

**Implications:** Ghostery shows that a privacy tool's data business can contradict its purpose. Izcal's crawler output and breakage reports must never become a product (NW-D17 already sends only URL and rules fired). Disconnect shows that browsers outsource lists because building them is expensive, and then inherit someone else's licence and policy. Izcal has already decided to build its own (OD-L5), and this history supports that choice.

---

## 4. Privacy Badger: heuristic learning and why it was turned off

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | Privacy Badger (EFF, 2014) blocks "non-consensual tracking" **without human-edited lists**. It watches for third-party cookies, local-storage "supercookies" and canvas fingerprinting, and blocks a domain once it has been seen tracking on **three** sites. It replaces social widgets with click-to-activate placeholders. | 3 sites | [EFF, "Badger evolution"](https://www.eff.org/badger-evolution) | read 2026-10-19 | High |
| 4.2 | **February 2020**: Google Security Team sent "a set of security disclosures". One serious issue was fixed at once by removing a feature (release 2020.2.19). Another class let "a dedicated attacker… manipulate the way Privacy Badger acts", forcing it to block or allow chosen domains, which **could be used to fingerprint** users. EFF said these were proof-of-concept only, with no evidence of use in the wild. | Feb 2020 | [EFF Deeplinks, 2020-10-07](https://www.eff.org/deeplinks/2020/10/privacy-badger-changing-protect-you-better) | 2020-10-07 | High |
| 4.3 | **7 October 2020**: local learning turned **off by default** "as it may make you more identifiable to websites or other actors". All default users now share one pre-trained list from **Badger Sett**, an automated crawler of thousands of popular sites. EFF: "the majority of Privacy Badger's protection is already captured by the pre-trained list." Local learning stays an opt-in, "a reasonable tradeoff that users should be able to make for themselves". | — | Same; [EFF, "Badger evolution"](https://www.eff.org/badger-evolution) | 2020-10-07 | High |
| 4.4 | EFF says the attacks resemble the ITP vulnerabilities that Apple patched in late 2019 (Section 6): learning in the client has the same class of weakness. | — | [EFF Deeplinks](https://www.eff.org/deeplinks/2020/10/privacy-badger-changing-protect-you-better) | 2020-10-07 | High |
| 4.5 | Detection of first-to-third-party cookie sharing (July 2019) was later **removed from local learning after a vulnerability**, but is still used in Badger Sett. | 2019 | [EFF, "Badger evolution"](https://www.eff.org/badger-evolution) | — | High |

**Why it was built that way.** Lists are subjective and stale; behaviour is objective. **Why it was changed.** Any per-user blocking state that differs between users, and that content can both influence and observe, is a fingerprint and a history oracle. **What survived** is behavioural classification run **centrally** on a crawler, with the same list shipped to everyone.

**Implications:** This is the most design-relevant postmortem for Izcal. Izcal's tracking list should be learned from behaviour by Izcal's own crawler (frame, §1) and **shipped identically to every user**. Any per-user learning, if offered at all, is an opt-in with a stated linkability cost measured under PRIV-2. Per-site user overrides are also state; Section 17 asks how observable they are.

---

## 5. uBlock Origin and Manifest V3 / declarativeNetRequest

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | uBlock came out in June 2014 (from HTTP Switchboard, 2013). On 3 April 2015 Hill transferred "uBlock" to Chris Aljoudi and renamed his own fork **uBlock Origin** on 6 April 2015. uBO refuses donations and points supporters to list maintainers. | 2014–15 | [uBlock Origin, Wikipedia](https://en.wikipedia.org/wiki/UBlock_Origin) | read 2026-10-19 | Medium |
| 5.2 | The **Manifest V3** changes were announced in **October 2018**. Developers raised alarm in January 2019. Google (Devlin Cronin) said DNR would handle "more than 30,000 rules" but could not be unbounded "for performance reasons". Stated aim: "Users need to have greater control over the data their extensions can access." Google added dynamic rules and promised no MV2 removal "until the platform is proven". | 30,000 | [The Register, 2019-02-16](https://www.theregister.com/2019/02/16/google_content_blocking_code/) | 2019-02-16 | Medium |
| 5.3 | Popular list sets used about **75,000** rules, so EasyList alone could approach the cap. uBO's author argued that uBO and uMatrix were incompatible with "the basic matching algorithm picked, ostensibly designed to enforce EasyList-like filter lists". | ~75,000 | [The Register](https://www.theregister.com/2019/02/16/google_content_blocking_code/); [Android Police, 2019-05-29](https://www.androidpolice.com/2019/05/29/google-still-plans-to-kill-chromes-existing-adblock-apis) (snippets) | 2019 | Low |
| 5.4 | **The performance argument did not hold**: WhoTracks.me (Cliqz/Ghostery, Feb 2019) measured sub-millisecond **median** decision time per request for all major blockers except DuckDuckGo's (Ghostery 0.007 ms, uBO 2.7× slower, ABP 2.9×, Brave's old engine 6.3×). The authors build a competing blocker, and the privacy side was excluded. | 0.007 ms median | [Ghostery/WhoTracks.me, "Adblockers performance study"](https://www.ghostery.com/blog/adblockers-performance-study) (snippet; original URL now 404) | 2019-02 | Medium |
| 5.5 | **Pause and resumption.** Chrome paused the MV2 phase-out in **December 2022** to address migration gaps. It resumed in **November 2023** after adding offscreen documents, longer service-worker lifetimes, a User Scripts API and "more generous" DNR limits. | — | [Chrome, "Resuming the transition to MV3"](https://developer.chrome.com/blog/resuming-the-transition-to-mv3) | 2023-11-16 | High |
| 5.6 | **Limit increases** (Chrome 120/121): enabled static rulesets 10 → **50**, total 50 → **100**; "safe" dynamic rules (block, allow, allowAllRequests, upgradeScheme) **30,000** (Chrome 121); other dynamic rules stay at **5,000**. Evidence offered: AdGuard counted **>2,600 changes per week** to popular lists, and one in four custom-list users had >5,000 dynamic rules. 98–99% of AdGuard rules fit the safe class. Firefox and Safari supported the higher static limits. | see text | [Chrome, "Improvements to content filtering in MV3"](https://developer.chrome.com/blog/improvements-to-content-filtering-in-manifest-v3) | 2023-11-15 | High |
| 5.7 | **Current limits**: guaranteed minimum **30,000** static rules per extension, beyond which a **global** pool applies (uBOL's FAQ gives **330K**); 1,000 regex rules; session rules 5,000. From Chrome 128, disabled extensions' static rules no longer count against the global pool. | 30k / 330k / 1k | [Chrome DNR reference](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest); [uBOL FAQ](https://github.com/uBlockOrigin/uBOL-home/wiki/Frequently-asked-questions-%28FAQ%29) | read 2026-10-19 | High |
| 5.8 | **MV2 removal timeline**: no new public MV2 items from Jan 2022; warnings from 3 June 2024; disabling in stable from **9 Oct 2024**; disabled by default on all channels on 31 Mar 2025; **Chrome 138** (24 July 2025) is "the final version of Chrome to support Manifest V2". The enterprise policy was removed in 139. Remaining MV2 items left the store on **31 Aug 2026**. | — | [Chrome, MV2 deprecation timeline](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline) | read 2026-10-19 | High |
| 5.9 | **uBO Lite's limits, in its author's words**: filter lists update only "when the extension updates"; uBOL "never makes network requests to any remote servers"; many uBO filters cannot be converted, so uBOL is "less effective at dealing with websites using anti-content blocker or minimizing website breakage"; no generic cosmetic filtering by default; dynamic filtering is impossible because DNR "does not support the ability to enforce rules according to the top context" (Chromium 145 added partial support); many regex filters, `replace=`, `ipaddress=` and regex `removeparam=` cannot be expressed. The default ruleset is about 17K rules. A June 2026 edit says custom filters and external list subscriptions are now available. | 17K; Chromium 145 | [uBOL FAQ (wiki)](https://github.com/uBlockOrigin/uBOL-home/wiki/Frequently-asked-questions-%28FAQ%29) | read 2026-10-19 | High |
| 5.10 | uBO's author says "uBlock Origin works best on Firefox". Chromium cannot uncloak CNAMEs for extensions; Firefox exposes a DNS API. A 2025 study found uBOL blocking on par with uBO, but its cosmetic filters about 20% less effective. | ~20% | [uBlock Origin, Wikipedia](https://en.wikipedia.org/wiki/UBlock_Origin) | read 2026-10-19 | Low |

**Why it was built that way.** A blocking webRequest API runs extension code on the critical path of every request and lets the extension read every URL. Google framed DNR as a security and privacy fix; critics read it as a capability cut by an ad company. **Technology mismatch:** DNR's expressiveness was set by what a **browser vendor** chose to match, not by what list authors write. Every feature list authors had built since 2014 (context-dependent rules, scriptlets, `$removeparam` regex, CNAME) became either impossible or a negotiated addition. **What survived** is the declarative model (Safari had it first, in 2015) and, in the end, higher limits won with data.

**Implications:** Izcal's matcher is in the engine (Stage 1 §6), so it does not carry DNR's extension-API limits. Its compiled automaton must accept the syntax list authors actually use (frame: `$domain`, `$removeparam`, `$redirect`). Any cap must be a **memory budget** (FP-28), not a rule count that strands lists. When Stage 2 extensions arrive, Izcal should give them a **declarative list input to the same matcher**, so that an extension never needs per-request code to block. That keeps the privacy benefit of DNR without its expressiveness gap.

---

## 6. Safari: Content Blockers and Intelligent Tracking Prevention

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **Content Blockers (June 2015)**: JS-based blockers "add latency to each resource load, use significant energy, and can consume large amounts of memory". Rules are JSON trigger/action pairs, compiled by WebKit into **bytecode** run in the network subsystem. "We do not track you by design": the extension never sees URLs. The compiler **rejects** rule sets that would hurt user experience. Regex is a strict subset; `^`/`$` only at the ends; ASCII only; quantifiers are costly. Actions: block, block-cookies, css-display-none, ignore-previous-rules. | — | [WebKit, "Introduction to WebKit Content Blockers"](https://webkit.org/blog/3476/content-blockers-first-look/) | 2015-06-12 | High |
| 6.2 | The per-blocker rule cap rose from **50,000 to 150,000** (iOS 15 per one AdGuard page, 2022 per another). AdGuard had split its filters across six blockers to work around the cap; Apple first refused on performance grounds. | 50k → 150k | [AdGuard KB](https://adguard.com/kb/adguard-for-ios/features/safari-protection/); [AdGuard blog](https://adguard.com/en/blog/adguard-for-safari-1-11.html) (snippets) | 2021–22 | Low |
| 6.3 | **ITP 1.0 (June 2017)**: on-device ML classifier over registrable domains. Strongest signals: subresource loads under unique domains, subframe loads under unique domains, and the number of unique domains redirected to. A 24-hour first-party interaction window keeps cookies; 30 days without interaction purges the data; otherwise storage is partitioned. "All data collection and classification happens on-device." | 24 h / 30 d | [WebKit, ITP](https://webkit.org/blog/7675/intelligent-tracking-prevention/) | 2017-06-05 | High |
| 6.4 | **ITP 2.1 (Feb 2019)**: persistent cookies created through `document.cookie` capped at **7 days**, because trackers were writing IDs into first-party jars. HttpOnly cookies set by the server were not affected. Classified domains lost partitioned cookies and had to use the **Storage Access API**. | 7 d | [WebKit, ITP 2.1](https://webkit.org/blog/8613/intelligent-tracking-prevention-2-1/) | 2019-02-21 | High |
| 6.5 | **ITP 2.2 (Apr 2019)**: if a classified domain navigated the user and the URL has a query or fragment (**link decoration**), script-set cookies on that page are capped at **1 day**. | 1 d | [WebKit, ITP 2.2](https://webkit.org/blog/8828/intelligent-tracking-prevention-2-2/) | 2019-04-24 | High |
| 6.6 | **Dec 2019, after Google's report**: tracking prevention "must make it hard or impossible to detect which content and data is treated as capable of tracking". WebKit trimmed **all** cross-site referrers to the origin and blocked cookies for **all** third parties without prior interaction, regardless of classification. | — | [WebKit, "Preventing Tracking Prevention Tracking"](https://webkit.org/blog/9661/preventing-tracking-prevention-tracking/) | 2019-12-10 | High |
| 6.7 | **Full third-party cookie blocking (March 2020)**, partly for "removing statefulness": ITP's state "could itself be used as a tracking vector". It also disables login fingerprinting. Script-writable storage is deleted after **7 days** of Safari use without interaction. The classifier continues for bounce trackers, collusion and link decoration. | 7 d | [WebKit, "Full third-party cookie blocking and more"](https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/) | 2020-03-24 | High |
| 6.8 | **CNAME cloaking defence (Nov 2020, Big Sur)**: cookies set in responses from third-party CNAME-cloaked subresources are capped at 7 days. Detection compares the subresource's CNAME with the first party's and the top frame's CNAMEs. **Bounce tracking**: after 10 unique navigational redirects, a tracker's cookies become SameSite=Strict (the domain list was empty at publication because the one reported domain stopped). | 7 d; 10 | [WebKit, "CNAME cloaking and bounce tracking defense"](https://webkit.org/blog/11338/cname-cloaking-and-bounce-tracking-defense/) | 2020-11-12 | High |
| 6.9 | **Link Tracking Protection (Safari 17 / iOS 17, June 2023)**: strips known tracking query parameters (for example fbclid, gclid, msclkid) during navigation and on copy, in Private Browsing, Mail and Messages. The list is static; one report counts about 25 parameters. Developers reported `window.location` losing parameters. | ~25 params | [mjtsai, 2023-06-19](https://mjtsai.com/blog/2023/06/19/safari-17-link-tracking-protection/); [Apple Developer Forums](https://developer.apple.com/forums/thread/746525) | 2023 | Low |

**Why it was built that way.** Apple has no crowd of list authors and does not want users to configure anything. A classifier on the device needs no list. **Fate.** The classifier proved to be a side channel (Google, 2019), so WebKit moved to **uniform, classifier-independent rules** and kept the classifier only where its decisions are hard to observe. **What survived**: caps on script-writable storage, link-decoration caps, CNAME and bounce defences, and the principle that protection must not be detectable.

**Implications:** Izcal's per-partition design already removes most third-party cookie questions. The transferable rules are these. (a) **Uniform treatment beats classified treatment** wherever a difference is page-observable. (b) The CNAME defence should compare against the first party's own CNAME, as WebKit's does, to avoid punishing self-hosted CDNs. (c) Bounce and link-decoration defences belong in navigation, not only in request matching.

---

## 7. Firefox: Tracking Protection, ETP, SmartBlock, TCP, query stripping, bounce tracking

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **Tracking Protection paper (2015)**: on the Alexa top 200 news sites, blocking tracking domains gave a **44% median page-load reduction**, **39% less data** and **67.5% fewer cookies**. | 44% / 39% / 67.5% | [Kontaxis and Chew, arXiv:1506.04104](https://arxiv.org/abs/1506.04104) | 2015-06-12 | High |
| 7.2 | **"Changing our approach to anti-tracking" (Aug 2018)**: three pillars, namely performance (citing Ghostery's finding that 55.4% of load time went to third-party trackers), removing cross-site tracking (cookie and storage blocking for Disconnect-listed trackers), and mitigating fingerprinting and cryptomining. Default blocking was compared to the 2004 pop-up blocker. Rolled out through shield studies. | 55.4% | [Mozilla Future Releases blog](https://blog.mozilla.org/futurereleases/2018/08/30/changing-our-approach-to-anti-tracking/) | 2018-08-30 | High |
| 7.3 | ETP on by default for new users in June 2019 and for all users later in 2019 (Firefox 69, Sept 2019). | — | [Help Net Security](https://www.helpnetsecurity.com/?p=123182) (snippet) | 2019 | Medium |
| 7.4 | **Total Cookie Protection (Feb 2021, Firefox 86, Strict)**: a cookie jar per site. Cross-site access is granted for non-tracking purposes such as login "only when it detects you intend to use that provider". | — | [Mozilla Security blog](https://blog.mozilla.org/security/2021/02/23/total-cookie-protection/) | 2021-02-23 | High |
| 7.5 | **SmartBlock (Mar 2021, Firefox 87)**: bundled **stand-in scripts** for blocked trackers, so that pages which call the tracker's API still work and no tracker content loads. Motivation: blocking broke sites (missing images, broken features, pages that did not load). | — | [Mozilla Security blog](https://blog.mozilla.org/security/2021/03/23/introducing-smartblock/) | 2021-03-23 | High |
| 7.6 | **Query parameter stripping (Firefox 102, June 2022)**: in ETP Strict and Private Browsing, with a short list (for example fbclid, mkt_tok); strip-on-share for copied links. | — | [Firefox docs: query stripping](https://firefox-source-docs.mozilla.org/toolkit/components/antitracking/anti-tracking/query-stripping/index.html); [ghacks](https://www.ghacks.net/?p=179447) (snippets) | 2022 | Medium |
| 7.7 | **Bounce Tracking Protection** shipped in **Firefox 133** (26 Nov 2024) in ETP Strict: it detects bounce trackers by redirect behaviour and periodically purges their data. It was in Nightly from 127. | — | [Firefox 133 release notes](https://www.mozilla.org/firefox/133.0/releasenotes) (snippet) | 2024-11-26 | Medium |
| 7.8 | The current privacy matrix (Standard / Strict / Private) lists separate switches for tracking protection, fingerprinting blocking and protection, cryptominers, social trackers, email tracking, cookie partitioning, query stripping, a "Level 2" list, referrer policy, bounce tracking and consent-manager skipping. Standard Normal blocks tracking **content** only in Private windows. | — | [Firefox docs: Privacy capabilities matrix](https://firefox-source-docs.mozilla.org/toolkit/components/antitracking/anti-tracking/etp-matrix/index.html) | read 2026-10-19 | High |

**Why it was built that way.** Mozilla had no list-building operation, so it licensed Disconnect's. It added partitioning (TCP) because no list is complete, and shims (SmartBlock) because blocking breaks sites. **What survived**: layered defence (list + partitioning + navigation defences + shims) and default-on with per-site exceptions.

**Implications:** Firefox's 2015 numbers are the template for Izcal's "measured, not asserted" success criterion: publish page-load, bytes and cookie deltas per release (FP 6.11). **Stand-ins** are the standard answer to the breakage that blocking scripts causes. Izcal needs its own neutral resources (`$redirect`, frame §2), written from public API documentation of the scripts they replace, and tested by black-box comparison.

---

## 8. Brave Shields

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **Rust rewrite (June 2019)**: the previous C++ engine relied on a **Bloom filter** to rule out non-blocked requests quickly, but **39%** of requests in the test set were blocked, so the fast path was often missed. The new engine hashes alphanumeric tokens, files each rule under its **least popular token**, and evaluates in phases (important, redirect, normal, exceptions). It drew on uBO's and Ghostery's approaches. **69×** faster on average; about **5.7 µs** per request with EasyList+EasyPrivacy (>55,000 network rules). EasyPrivacy blocked 70% more requests than EasyList alone. | 69×; 5.7 µs; 39% | [Brave blog](https://brave.com/blog/improved-ad-blocker-performance/) | 2019-06-26 | High |
| 8.2 | **CNAME uncloaking (Brave 1.17, Nov 2020)**: for requests not already blocked, Brave resolves the CNAME with its embedded resolver and blocks if the canonical name would be blocked. It is the first Chromium-based browser to do this by default; uBO on Firefox had it earlier through Firefox's DNS API. A later CVE (fixed in 1.33.106) involved CNAME blocking combined with a proxy extension. | — | [Brave, "Fighting CNAME trickery"](https://brave.com/privacy-updates/6-cname-trickery/); [ghacks](https://www.ghacks.net/?p=166253); [APNIC blog](https://blog.apnic.net/2020/08/04/characterizing-cname-cloaking-based-tracking/) (snippets) | 2020 | Medium |
| 8.3 | **Debouncing (desktop 1.32, ~Nov 2021)**: on navigation to a known bounce-tracking URL whose parameter carries the destination, go straight to the destination. The list comes from crowd-sourcing and open projects (ClearURLs and others). Later: "unlinkable bouncing" and a bounce-tracking interstitial. | — | [Brave privacy updates 11](https://brave.com/privacy-updates/11-debouncing/); [16](https://brave.com/privacy-updates/16-unlinkable-bouncing/) (snippets) | 2021–22 | Medium |
| 8.4 | **Policy**: block third-party ads because ads and tracking cannot be separated; "will always attempt to block anti-adblock behavior". **Standard** mode does no network blocking of first-party requests (to limit breakage). **Aggressive** mode blocks first-party requests too and enables CNAME uncloaking. iOS uses a weekly "Slim List" compiled into a Content Blocker extension because of WKWebView. | — | [Brave wiki, "Blocking goals and policy"](https://github.com/brave/brave-browser/wiki/Blocking-goals-and-policy) | read 2026-10-19 | High |
| 8.5 | **Memory rewrite (v1.85, announced 5 Jan 2026)**: the ~100,000 default filters moved from heap structures (vectors, hash maps) into a **zero-copy binary format** (FlatBuffers), done iteratively. Result: **−75% memory, ~45 MB** saved by default on Android, iOS and desktop; 19% fewer allocations; 13% faster matching; ~15% faster builds; 30% less memory for resource storage. Built into the browser, so "unaffected by Manifest V3". | −75%; 45 MB; ~100k filters | [Brave privacy updates 36](https://brave.com/privacy-updates/36-adblock-memory-reduction) | 2026-01-05 | High |

**Why it was built that way.** Brave's business is the browser, so blocking is built into the engine and free of extension limits. Its default lists are community lists (EasyList, EasyPrivacy, uBO lists) plus its own. **Fate.** Successful, but it took seven years and two rewrites to reach a memory-mapped, zero-copy store: the first engine was designed for a world in which blocks were rare and lists were small.

**Implications:** Brave's 2026 result is independent support for the sealed, read-only, mapped automaton in NW-D26/FP-28: ~45 MB of heap per process was the cost of **not** doing it. Izcal starts there. Brave's Standard mode (no first-party network blocking) is the clearest published breakage policy; Izcal should state its own first-party policy just as plainly. Brave's numbers come from Brave and are measured on its own corpus, so Izcal must re-measure, not copy, them.

---

## 9. Chrome: Better Ads filter, heavy-ad intervention, third-party cookies, Privacy Sandbox

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | **Ad filter (15 Feb 2018)**: Chrome first checks whether a site **fails the Better Ads Standards** (from Coalition research with >40,000 users; worst: prestitials and flashing animated ads). If it fails, requests are matched against EasyList-based ad patterns, including Google's own AdSense and DoubleClick. Site owners see violations in the **Ad Experience Report** and request re-review. | >40,000 | [Chromium Blog, Feb 2018](https://blog.chromium.org/2018/02); [betanews](https://betanews.com/2018/02/14/google-chrome-ad-filtering) (snippets) | 2018-02 | Medium |
| 9.2 | **Heavy-ad intervention (Chrome 85, Aug 2020)**: an ad frame the user has not interacted with is unloaded if it uses **>4 MB** of network, **>15 s** of main-thread time in any 30 s window, or **>60 s** in total (descendant frames included). Ads are identified by **filter-list matching**; non-ads can be tagged by mistake. **0.3%** of ads exceed the thresholds but account for **27%** of ad network data and **28%** of ad CPU. | 4 MB / 15 s / 60 s; 0.3% | [Chrome developers, heavy-ad interventions](https://developer.chrome.com/blog/heavy-ad-interventions); [Chromium Blog, 2020-05-14](https://goo.gle/2Xj8HJD) (snippet) | 2020-05-14 | High (thresholds); Medium (shares) |
| 9.3 | **Third-party cookies**: Chromium announced a path to make them obsolete on **14 Jan 2020**. Delays were announced in June 2021 (to mid-2023, "subject to our engagement with" the UK CMA) and July 2022 (to H2 2024). Chrome restricted them for **1%** of Stable users on 4 Jan 2024. | 1% | [Chromium Blog, 2020-01-14](https://blog.chromium.org/2020/01/building-more-private-web-path-towards.html) (title/date only); [The Hacker News, 2022-07](https://thehackernews.com/2022/07/google-delays-blocking-3rd-party.html) (snippets) | 2020–2024 | Medium |
| 9.4 | **22 July 2024**: "A new path": instead of removing third-party cookies, a Chrome experience to let users choose. **22 April 2025**: Chrome will **not** roll out a standalone prompt and keeps the existing settings. Reasons: "divergent perspectives" across publishers, developers, regulators and the ad industry, and changed circumstances since 2019. Incognito still blocks third-party cookies by default; IP Protection was planned for Q3 2025. | — | [Privacy Sandbox, "A new path"](https://privacysandbox.google.com/blog/privacy-sandbox-update) (snippet); [Privacy Sandbox, "Next steps"](https://privacysandbox.google.com/blog/privacy-sandbox-next-steps) | 2024-07-22; 2025-04-22 | High (2025); Medium (2024) |
| 9.5 | **17 Oct 2025**: most Privacy Sandbox technologies **retired**: Attribution Reporting, IP Protection, On-Device Personalization, Private Aggregation (with Shared Storage), Protected Audience, Protected App Signals, Related Website Sets, SelectURL, SDK Runtime, Topics. Reason: "low levels of adoption" after "ecosystem feedback". **Kept**: CHIPS, FedCM, Private State Tokens; attribution work moves to W3C PATWG. | 10 retired | [Privacy Sandbox, "Update on plans"](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies) | 2025-10-17 | High |

**Why it was built that way.** Google sells ads and runs the largest browser. Every Chrome intervention targets the **worst** ads or resource abuse, and keeps the ad market working: Better Ads (format quality), heavy ads (resources), Privacy Sandbox (ads without cross-site cookies). **Fate.** The interventions that do not threaten revenue survived. The cookie phase-out failed under the combined weight of competition-law oversight (the CMA worried that Google would advantage itself) and an ad industry that did not adopt the replacements. **What survived** are the mechanisms that are useful whatever the business model: CHIPS (partitioned cookies), FedCM (login without third-party cookies), and resource caps on ad frames.

**Implications:** A browser vendor whose revenue depends on tracking cannot ship default tracking protection. Izcal has no such conflict, which is why HGBA-3's class (c) interventions are possible. The **heavy-ad thresholds** are a published, measurable model for FP-D7's script-attributed interventions on sites where blocking is off. The 0.3% / 27% / 28% figures say that a small, attributable set of frames causes most of the cost. Izcal should test this on its own corpus before choosing thresholds.

---

## 10. Microsoft Edge tracking prevention

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | Edge (Chromium-based, preview June 2019) has three levels: **Basic** (malicious trackers only), **Balanced** (default; third-party trackers plus malicious), **Strict** (largest set of categories, blocking storage access and resource loads). | 3 levels | [Edge blog, 2019-06-27](https://blogs.windows.com/msedgedev/2019/06/27/tracking-prevention-microsoft-edge-preview/) (snippet) | 2019-06-27 | Medium |
| 10.2 | Powered by **Disconnect's lists**. Edge 79 blocked ~25% more trackers than 78. Balanced **relaxes blocking for organisations the user engages with**, computed on the device from engagement scores; brief visits or pop-ups are not enough. Strict has no mitigations. | +25% | [Edge blog, 2019-12-03](https://blogs.windows.com/msedgedev/2019/12/03/improving-tracking-prevention-microsoft-edge-79/) (snippet) | 2019-12-03 | Medium |

**Implications:** Engagement-based relaxation trades privacy for compatibility using **per-user state**, the same structure that Privacy Badger and ITP found observable. If Izcal ever relaxes protection based on user behaviour, PRIV-2 must measure whether a site can detect the relaxation.

---

## 11. DuckDuckGo Tracker Radar and protections

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 11.1 | **Tracker Radar (5 Mar 2020)**: "automatically generated and maintained through continuous crawling and analysis". There is one file per third-party domain (prevalence, ownership, fingerprinting, cookies, performance) and one per entity (ownership cross-checked against WHOIS and certificates). It was built because hand-curated lists were "stale and incomplete" and some broke sites. Data licence **CC BY-NC-SA 4.0**; generator Apache 2.0; it includes rules and **breakage exceptions**. DDG's own apps use a list derived from it. | — | [Spread Privacy](https://spreadprivacy.com/duckduckgo-tracker-radar/) | 2020-03-05 | High |
| 11.2 | **May 2022**: researcher Zach Edwards found that DDG's mobile browsers let some **Microsoft** trackers (Bing, LinkedIn) through. The CEO said the **search syndication contract** prevented more, and its confidentiality terms prevented disclosure. The contract was later changed. | — | [The Next Web, 2022-05-26](https://thenextweb.com/news/duckduckgo-microsoft-tracking-sparks-backlash) (snippet) | 2022-05 | Medium |
| 11.3 | DDG later shipped App Tracking Protection on Android (beta from 2021). Not checked this session (search budget exhausted). | — | — | — | Low |

**Implications:** Tracker Radar is the closest published precedent for Izcal's planned list (a crawler, behavioural evidence, entities, breakage exceptions). Its NC licence means Izcal cannot ship its data (consistent with OD-L5). The Microsoft episode shows that an **undisclosed exception destroys trust faster than a gap**. Izcal should publish every exception with its reason, including any created by a contract (and should have none).

---

## 12. DNS-level blocking: hosts files, Pi-hole, NextDNS

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 12.1 | Hosts files were used to block ads early (first published on Fravia's anti-advertising pages). Malware also modifies hosts files (Mydoom.B blocked security sites). | — | [hosts (file), Wikipedia](https://en.wikipedia.org/wiki/Hosts_(file)) | read 2026-10-19 | Low |
| 12.2 | **Pi-hole** (Jacob Salmela, 2014, as "an open source alternative to AdTrap") is a network DNS server that answers listed domains with a dummy address. Limits: only listed domains; blocking can break sites; only covers devices that use it. | 2014 | [Pi-hole, Wikipedia](https://en.wikipedia.org/wiki/Pi-hole) | read 2026-10-19 | Low |
| 12.3 | DNS filtering "cannot block ads delivered via the same domain as the content" (for example YouTube), and apps with hard-coded DoH bypass it. | — | [Pi-hole forum](https://discourse.pi-hole.net/t/so-youtube-ads-cant-really-be-blocked-properly-with-pi-hole/34986/2) (snippet) | — | Low |
| 12.4 | NextDNS (2019) and similar services offer the same thing as a hosted resolver. Not checked this session. | — | — | — | Low |

**Implications:** DNS blocking survives because it covers apps and devices that browsers cannot. It cannot see paths, frames or first-party ads. Izcal's DNS (`hg-dns`) should use its view of the **CNAME chain** to uncloak trackers (NW-D26), but blocking itself belongs at the request matcher, where the full context is known. Users running Pi-hole or a filtering resolver must not be broken by Izcal's DoH choices; that is a networking question to record.

---

## 13. Cross-cutting: state as a side channel

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 13.1 | Three projects independently found that **per-user, behaviour-dependent protection state is observable**: ITP (Google, 2019, which led WebKit to uniform rules), Privacy Badger (Google, 2020, local learning off), and Privacy Badger's cookie-sharing heuristic (removed after a vulnerability). | 3 cases | 6.6–6.7, 4.2–4.5 above | 2019–2020 | High |
| 13.2 | The fixes were the same in each case: **ship the same decisions to everyone** (pre-trained list; classifier-independent cookie policy), and keep learning centralised or opt-in. | — | Same | — | High |

**Implications:** This is the rule for Izcal's defaults. Protection must be a function of the shipped list version and the user's explicit settings, never of what the user browsed. This also makes DET-1 replay simple, because the list version and settings are the only inputs.

---

## 14. Cross-cutting: who pays for the list

| Project | Who pays | Conflict that resulted | Outcome |
|---|---|---|---|
| EasyList | Volunteers; later partly eyeo | Single points of failure (2009); stale rules | Survives, but with no evidence per entry |
| ABP / eyeo | Advertisers (Acceptable Ads) | Paid allowlisting | Lawful in Germany (BGH 2018), lasting trust cost |
| Ghostery / Evidon | Ad-industry data (GhostRank) | Data flowing to the ad industry | Sold twice; rebuilt as open source |
| DuckDuckGo | Search ads syndicated by Microsoft | Contractual tracker exception | Contract changed after disclosure |
| Chrome | Google ads | Could not remove third-party cookies | Phase-out reversed; Sandbox retired |
| Brave, Firefox, Safari | Browser business | None reported for blocking | Default-on protection shipped |

**Implications:** Content blocking works when the party that decides what is blocked has no financial interest in what is blocked. Izcal's lists must be funded as part of the browser (Networking 9.3 staffing), with no revenue from any listed party.

---

## 15. Lessons (ideas only, never code)

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| EasyList | Volunteer list, no evidence per entry, stale rules | Lists rot without evidence and expiry | Every Izcal entry has evidence, a review and a last-fired date; entries that stop firing on the crawl are reviewed for removal |
| Adblock Plus / Acceptable Ads | Paid default-on allowlist | Money for exceptions destroys trust | No paid entries or exceptions, ever; exceptions published with evidence |
| Axel Springer v eyeo (BGH 2025) | Copyright theory against changing DOM/CSSOM; remanded | The mechanism of cosmetic filtering is legally contested in Germany | Record that Izcal applies cosmetic rules as a user-origin sheet in the engine; legal review before shipping in Germany |
| Privacy Badger | Local learning made users identifiable | Per-user learned state is a fingerprint | Learn centrally on Izcal's crawler; ship one list to all; any local learning is opt-in with measured linkability |
| ITP | Classifier state was detectable | Uniform rules beat classified rules where differences are observable | Prefer partitioning and uniform caps; use classification only where its effect is not page-observable |
| Firefox Tracking Protection (2015) | Published measured gains | Protection should be measured and published | Publish per-release page-load, bytes, cookie and breakage deltas |
| Firefox SmartBlock | Blocking broke sites; stand-ins fixed it | Breakage needs a built-in remedy, not only an allowlist | Neutral replacement resources written from public API documentation, tested black-box |
| Safari Content Blockers / Chrome DNR | Declarative, compiled rules with caps | Declarative matching keeps URLs private, but rule-count caps strand list authors | Compile to a sealed automaton with a memory budget, not a rule cap; Stage 2 extensions feed the same matcher declaratively |
| WhoTracks.me study | Matching took sub-ms at median | Matching speed was never the bottleneck for good engines | Spend design effort on memory, update latency and expressiveness; keep FP-28 targets but do not over-optimise the hot path at the expense of syntax |
| Brave adblock-rust (2019, 2026) | Bloom filter mismatched a 39% block rate; heap structures cost ~45 MB | Design for the real block rate and for many processes sharing one store | Token-indexed, zero-copy, read-only mapped automaton from day one |
| Brave Standard mode | No first-party network blocking by default | State the first-party policy explicitly | Izcal's spec states what first-party requests the default list may block, and why |
| Brave / ITP CNAME defences | Trackers moved behind first-party CNAMEs | First-party-looking requests need DNS-level evidence | `hg-dns` reports the CNAME chain to the matcher; compare with the first party's own CNAME |
| Brave debouncing / ITP bounce / Firefox BTP | Trackers moved to navigations | Navigation tracking is a separate attack surface | Bounce and link-decoration defences in navigation, with their own targets |
| Chrome third-party cookies | Ad-funded vendor reversed its own plan | Conflicted vendors cannot ship default privacy | Izcal states its funding independence as a design premise |
| Chrome heavy ads | 0.3% of ads cause ~27–28% of cost | A small, attributable set of frames causes most cost | FP-D7 interventions use attributed resource thresholds, measured on Izcal's corpus |
| Edge Balanced mode | Relaxation by on-device engagement | Compatibility relaxations are per-user state | Measure observability under PRIV-2 before any engagement-based relaxation |
| DuckDuckGo Microsoft exception | Undisclosed contractual exception | Hidden exceptions cost more trust than gaps | Every exception is public with its reason |
| Ghostery / GhostRank | Blocking data sent to the ad industry | A blocker's telemetry can feed evasion | Breakage reports carry only URL and rules fired; no aggregate block data is sold or shared |
| NoScript, uBO on Chrome | Extension APIs were removed under them | Building blocking on someone else's extension API is fragile | Blocking is in the engine (Stage 1 §6, D-S3), not on an extension API |
| Pi-hole / DNS | Covers all devices, but only whole domains | DNS is a complement, not a substitute | Do not rely on DNS blocking; do not break users' own DNS filters |

---

## 16. Flagged claims and conflicts

| Claim | Problem |
|---|---|
| "Federal Constitutional Court ruled for eyeo in April 2018" (Wikipedia) | The April 2018 ruling was the **BGH** (I ZR 154/16); the BVerfG declined the complaint in 2019 (single source) |
| "OLG Köln 2025 ruling against paid allowlisting" (CyberInsider) | Case number matches the 2016 case; no 2025 primary source found |
| Safari 150,000-rule cap: iOS 15 (2021) vs 2022 | Two AdGuard pages disagree; no Apple primary source read |
| WhoTracks.me study | Run by a competing blocker's authors; original URL now returns 404; read only via snippets |
| Brave memory figures (−75%, 45 MB) | Vendor-reported, Brave's own corpus |
| uBOL cosmetic filters "~20% less effective" (2025 study) | Read via Wikipedia only; study not read |
| Chrome 14 Jan 2020 "within two years" wording | The page loaded without its body; wording not checked |
| Firefox ETP default dates (June/Sept 2019) and TCP default for all (2022) | Not checked against Mozilla primary pages this session |

---

## 17. Questions for black-box measurement

Each of these values is, as far as this research found, stated only in source code or not at all in prose. Izcal must measure them with black-box tests and never take them from source (M-1 part 3).

| # | Question | Why it matters | How to measure (black box) |
|---|---|---|---|
| Q1 | What is Chrome's actual global static-rule pool across extensions today, and how does it behave when exceeded? (uBOL's FAQ says 330K; Chrome's docs give no number.) | Calibrates what list sizes real users run | Install extensions with large static rulesets; call `getAvailableStaticRuleCount()` until refused |
| Q2 | What regex subset and rule-set complexity limits does Safari's content-blocker compiler enforce today, and at what point does it reject a set? | Shows where a compiled design hits limits | Feed generated rule sets of growing size and quantifier count; record compile errors and time |
| Q3 | What is the per-request matching latency and per-process memory of Brave, uBO (Firefox) and Safari content blockers on the same corpus? | Baseline for FP-28 (p99 ≤ 20 µs†) | Instrumented page loads with timing from outside the engine (network-level timestamps, OS memory accounting) |
| Q4 | Which query parameters do Safari LTP, Firefox query stripping and Brave strip today? | Coverage baseline for `$removeparam` | Navigate to URLs carrying candidate parameters; record the landed URL |
| Q5 | What thresholds trigger Firefox's bounce-tracking purge, and how long until purge? | Design of Izcal's navigation defences | Build redirect chains with controlled counts and timing; observe storage after purge intervals |
| Q6 | Is ITP's bounce-tracking "SameSite=Strict jail" domain list still empty, and does the 10-redirect threshold still hold? | Same | Controlled redirect sites, observe cookie attributes |
| Q7 | Can a page detect whether a browser's per-site override (shields off, ETP exception) is set, and with how many bits? | PRIV-2 linkability of user settings | Cross-site probes for load success of listed resources under each setting |
| Q8 | Does Edge's Balanced engagement relaxation become visible to third parties? | Same | Engage with one entity's site; probe its third-party resources from another site |
| Q9 | Which CNAME comparisons does each engine with uncloaking apply (subresource vs top frame vs first-party CNAME), and what does it do with multi-hop chains? | Rules for `hg-dns` uncloaking | DNS zones under test control with crafted CNAME chains |
| Q10 | How often do Brave, Firefox (Disconnect lists) and Safari update their lists in practice, and how large are the deltas? | Daily† update schedule (Networking 6.17) and delta format | Observe update fetches from a clean profile over weeks |
| Q11 | For Chrome's heavy-ad intervention, how are ad frames tagged today (which list version), and how often do non-ad frames get tagged? | Ad tagging for FP-D7 | Pages with known ad and non-ad frames crossing thresholds |
| Q12 | What share of corpus pages show user-visible breakage under Firefox Strict, Brave Standard/Aggressive and Safari defaults? | Baseline for NW-54's ≤ 1%† | Screenshot and DOM diffing on Izcal's corpus, with and without protection |
