# Browser Privacy and Security Pain Points: Claim Verification and Browser-Level Mitigations (as of October 2026)

Research date: 2026-10-14. Older data is dated inline. Where only secondary coverage was reachable, this is flagged.

## Claim 1: "Users see roughly 1,000 cookie banners per year and click Accept All in about two seconds"

### Takeaway
**Verdict: Unverifiable.** I found no primary study that measures banners seen per year. The "~2 seconds" figure is not supported by the best field study, which measured median decision times of roughly 4–8 seconds (means 6.5–21 s) depending on banner design. Users do decide fast, but "2 seconds" and "1,000 per year" look like marketing estimates.

### Cited Findings
- Field study of GDPR consent notices (Utz et al., "(Un)informed Consent", CCS 2019; arXiv 1909.02638). Time to decision by banner type: medians of about 4–8 s, means of about 6.5–21 s. The "no option" banner had the shortest mean at 6.51 s. — [arXiv 1909.02638](https://arxiv.org/pdf/1909.02638)
- Vendor blogs claim things like "70% of users spend <3 s on a banner" and "52% click Accept All within five seconds" without primary citations. Treat these as weak sources. — [flexyconsent](https://flexyconsent.com/blog/consent-fatigue-how-to-reduce-banner-dismissals/); [Gitnux 2026](https://gitnux.org/cookie-consent-statistics/)
- Matte et al. (2020) note that European users "encounter cookie banners on almost every website", which is qualitative and gives no annual count. — [arXiv 1911.09964](https://arxiv.org/abs/1911.09964)

### Inferences
- A "~1,000/year" figure could be derived (for example, ~3 new sites/day × 365), but I found no measurement. Report it as an estimate, not a finding.
- Corrected phrasing: "Users typically decide on a banner within several seconds (median ~4–8 s in a 2019 field study); no reliable annual count exists."

### Gaps
- No peer-reviewed study gives banners encountered per user per year. The HBS Working Knowledge "time cost" piece ([link](https://www.library.hbs.edu/working-knowledge/time-cost-of-customizing-website-cookies-visualized)) was found but not read.

## Claim 2: "An audit found that 55% of websites set cookies even after a user clicks Reject, and 78% of banners do not fulfill the user's choice"

### Takeaway
**Verdict: Partly correct.** The figures come from the **webXray California Privacy Audit (data March 2026, published April 14, 2026)**. However, that audit tested the **Global Privacy Control (GPC) signal, not clicking "Reject" on a banner**. Its own page asserts "78% of Cookie Banners Fail to Protect Users" but gives no derivation. The academic "Reject is ignored" figure is **65.4%** (Bouhoula et al., USENIX Security 2024, 97k EU sites).

### Cited Findings
- webXray audit: 7,634 popular sites, each scanned twice in unmodified Chrome from a California residential IP, with and without `Sec-GPC: 1`. Results:
  - **55% (4,170 sites)** set advertising cookies despite GPC opt-out.
  - 125,106 ad cookies set despite opt-out.
  - 194 of 242 ad-tech vendors (80%) set ad cookies despite GPC.
  - All 11 evaluated CMPs failed. — [globalprivacyaudit.org/2026/california](https://globalprivacyaudit.org/2026/california)
- The "78% of Cookie Banners Fail" line appears in the summary without a formula. Per-CMP failure rates were 77%, 90% and 91%. The $5.8B "liability exposure" is average fine × 4,170 sites, which the authors call an estimate. — [globalprivacyaudit.org](https://globalprivacyaudit.org/2026/california)
- Vendor-level opt-out failure rates: Google 86% (present on 77% of sites), Microsoft 50%, Meta 69%. Auditor is webXray (founder Timothy Libert, ex-Google cookie-compliance lead). Google called it "a fundamental misunderstanding of how our products work". Meta called it a marketing ploy. — [PPC Land, Apr 2026](https://ppc.land/audit-finds-google-meta-and-microsoft-set-ad-cookies-after-users-opt-out/); [TechSpot](https://www.techspot.com/news/112073-clicking-reject-cookies-might-not-actually-do-anything.html)
- **Bouhoula, Kubicek, Zac, Cotrini, Basin (ETH Zurich), USENIX Security 2024:** across 97k EU-popular websites, "65.4% of websites offering a cookie rejection option likely collect user data despite explicit negative consent." — [USENIX](https://usenix.org/conference/usenixsecurity24/presentation/bouhoula)
- **Matte, Bielova, Santos, IEEE S&P 2020** (crawl of 22,949 EU sites using IAB TCF):
  - 141 sites register positive consent before the user chooses.
  - 236 pre-select options.
  - 27 store positive consent after explicit opt-out.
  - In-depth tests of 560 sites found at least one violation in 54%. — [arXiv 1911.09964](https://arxiv.org/abs/1911.09964)
- **Bollinger, Kubicek, Cotrini, Basin, USENIX Security 2022:** ~30k websites, at least one potential GDPR violation in **94.7%** of sites. Their CookieBlock extension filtered ~90% of privacy-invasive cookies on 100 sampled sites. — [USENIX](https://usenix.org/conference/usenixsecurity22/presentation/bollinger)
- University of Amsterdam (2024) reported that 65% of testable websites ignore rejection (news release, not read in full). — [UvA](https://www.uva.nl/shared-content/faculteiten/en/faculteit-der-rechtsgeleerdheid/news/2024/03/control-your-cookies.html)
- PrivAudit (arXiv preprint): on 239 CCPA sites with banners, the median cookie count barely changed after rejection (23 → 22). — [arXiv 2609.09697](https://arxiv.org/pdf/2609.09697)

### Inferences
- Corrected claim: "A 2026 webXray audit of 7,634 sites found 55% set ad cookies despite a GPC opt-out signal, and all 11 CMPs tested failed to block them. Peer-reviewed EU work (USENIX Sec 2024) found 65.4% of sites with a Reject option likely still collect data after rejection."
- Do not use "78%" without caveating that the source does not show how it was derived.

### Gaps
- I could not find the derivation of the 78% figure anywhere.

## Claim 3: "Chrome lacks meaningful native anti-fingerprinting protections"

### Takeaway
**Verdict: Largely correct for normal browsing (as of Oct 2026).** Chrome's only default-on measure is User-Agent reduction, which shrinks passive entropy. IP Protection and fingerprinting-script blocking were scoped to Incognito. In October 2025, Google listed "IP Protection" among the Privacy Sandbox technologies being retired. By contrast:
- Safari 26 turns on Advanced Fingerprinting Protection by default in all modes.
- Firefox 145 (Nov 2025) added phase-2 fingerprinting protections, reported by Mozilla to cut users seen as unique by "almost half". These apply only in Private Browsing / ETP Strict for now.
- Brave randomizes ("farbles") fingerprintable APIs by default. This was not re-verified in this session.

### Cited Findings
- **Chrome UA reduction:**
  - Minor version zeroed from Chrome 101.
  - Desktop OS/platform frozen from Chrome 107.
  - Android model/version frozen from Chrome 110 (Feb 2023). — [Chrome dev blog](https://developer.chrome.com/blog/user-agent-reduction-android-model-and-version); [Privacy Sandbox UA reduction](https://privacysandbox.google.com/protections/user-agent)
- April 22, 2025 (Anthony Chavez): Google planned to launch **IP Protection in Incognito in Q3 2025**. Incognito already blocks third-party cookies by default. — [Privacy Sandbox blog](https://privacysandbox.google.com/blog/privacy-sandbox-next-steps)
- The blink-dev Intent to Ship (July 2025) for IP Protection in Incognito targeted desktop Chrome 140. It is limited to a Masked Domain List and to signed-in Incognito users, and is off by default for enterprise. — [blink-dev](https://groups.google.com/a/chromium.org/g/blink-dev/c/EbdlJ3pc3NM/m/onR2VnBgAgAJ); [GitHub explainer](https://github.com/GoogleChrome/ip-protection)
- Aug 2025: Chrome tested blocking known fingerprinting scripts in Incognito on Windows, again limited to domains on a list. — [Red Hot Cyber](https://www.redhotcyber.com/en/post/google-chrome-goes-privacy-first-a-new-script-blocker-for-incognito-mode)
- Oct 17, 2025: Google listed **IP Protection** among the retired Privacy Sandbox technologies. — [Privacy Sandbox blog](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies)
- **Firefox 145 (released Nov 11, 2025):**
  - Mozilla says the improvements "cut the percentage of users seen as unique by almost half".
  - Secondary outlets report ~65% → ~20% unique/trackable, and up to "70%". Those baselines are unclear.
  - Currently limited to Private Browsing and ETP Strict, with default-on planned after testing. — [SecurityWeek](https://www.securityweek.com/new-firefox-protections-halve-the-number-of-trackable-users/); [BleepingComputer](https://www.bleepingcomputer.com/news/security/mozilla-firefox-gets-new-anti-fingerprinting-defenses/); [PCWorld](https://www.pcworld.com/article/2969961/firefox-browser-adds-privacy-protections-to-erase-digital-fingerprints.html); [TechRadar](https://www.techradar.com/pro/security/firefoxs-new-anti-fingerprinting-protections-cut-user-tracking-by-70-percent)
- **Safari 26 (Sept 2025):**
  - Advanced Fingerprinting Protection is on by default in all browsing modes. It presents "a simplified version of the system configuration so more devices look identical to trackers".
  - Link Tracking Protection (URL parameter stripping) still auto-applies only in Private Browsing and Mail/Messages links. — [9to5Mac](https://9to5mac.com/2025/09/01/security-bite-my-favorite-privacy-features-in-ios-26/); [PPC Land](https://ppc.land/safari-26-tracking-changes-to-impact-marketing-measurement/)

### Inferences
- Corrected claim: "Chrome's default (non-Incognito) mode has no active anti-fingerprinting beyond UA reduction. Its Incognito-only IP Protection was listed for retirement in Oct 2025. Safari (default-on since Safari 26) and Brave (default farbling) protect by default. Firefox's strongest protections remain opt-in (Strict/Private) as of late 2025."

### Gaps
- Current (Oct 2026) status of Chrome Incognito IP Protection and script blocking: whether either shipped before retirement, or was removed, is not confirmed.
- I found no primary source for Brave farbling effectiveness in this session.
- I did not read the original Mozilla blog post. The "almost half" quote comes via SecurityWeek.

## Claim 4: "Google abandoned its plan to reduce third-party cookies in July 2024"

### Takeaway
**Verdict: Partly correct / imprecise.**
- **July 2024:** Google abandoned *deprecation*. It proposed instead a "user choice" prompt, so it had not yet abandoned *reducing* cookies.
- **April 22, 2025:** Google dropped the standalone choice prompt and kept the status quo of cookie controls in Settings.
- **October 17, 2025:** Google retired most Privacy Sandbox APIs (Topics, Protected Audience, Attribution Reporting, IP Protection, Related Website Sets, etc.) for low adoption.

As of October 2026, third-party cookies remain enabled by default in regular Chrome browsing.

### Cited Findings
- July 2024: Google reversed its plan to phase out third-party cookies and proposed an "informed choice" experience instead. — [Computerworld](https://www.computerworld.com/article/3476620/google-abandons-plans-to-drop-third-party-cookies-in-chrome.html); [AlternativeTo](https://alternativeto.net/news/2024/7/google-reverses-plan-to-phase-out-third-party-cookies-in-chrome-updates-privacy-sandbox/)
- April 22, 2025 (Anthony Chavez, VP Privacy Sandbox):
  - Chrome will "maintain our current approach to offering users third-party cookie choice in Chrome's Privacy and Security Settings".
  - It will **not** roll out a standalone prompt, citing "divergent perspectives".
  - Incognito keeps blocking third-party cookies by default. — [Privacy Sandbox blog](https://privacysandbox.google.com/blog/privacy-sandbox-next-steps)
- October 17, 2025 (Chavez): Google **retired** the following, citing low adoption:
  - Attribution Reporting API (Chrome and Android)
  - IP Protection
  - On-Device Personalization
  - Private Aggregation (incl. Shared Storage)
  - Protected Audience (Chrome and Android)
  - Protected App Signals
  - Related Website Sets (incl. requestStorageAccessFor, Related Website Partition)
  - SelectURL
  - SDK Runtime
  - Topics (Chrome and Android)

  Kept: CHIPS, FedCM, Private State Tokens, and work on an interoperable Attribution standard. — [Privacy Sandbox blog](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies)
- Secondary reporting says deprecation intents (Nov 2025) target deprecation starting in Chrome 144 and removal by Chrome 150. Topics was used on ~13% of page loads. — [Windows Report via scour.ing](https://scour.ing/p/https://windowsreport.com/google-is-scrapping-privacy-sandbox-apis-as-chrome-keeps-third-party-cookies-after-all) (not primary)

### Inferences
- Corrected claim: "Google abandoned cookie *deprecation* in July 2024, dropped its planned choice prompt in April 2025, and wound down most Privacy Sandbox APIs in October 2025. Third-party cookies remain on by default in Chrome in 2026."

### Gaps
- I did not verify the exact Chrome removal milestone for each retired API against chromestatus.com.

## Claim 5: "Manifest V3 effectively disabled uBlock Origin"

### Takeaway
**Verdict: Correct for Chrome; wrong as a general statement.**
- In Chrome, MV2 extensions (including uBO) were disabled with no re-enable as of Chrome 138 (July 24, 2025). MV2 support was removed in Chrome 139, and all MV2 items were removed from the Chrome Web Store on **Aug 31, 2026**.
- uBO still works on Firefox. Brave reportedly lets users keep some MV2 extensions. Edge began its consumer MV2 phase-out in August 2026, targeting completion by end of 2026.
- uBlock Origin Lite (MV3) is the Chrome replacement, with reduced capabilities.

### Cited Findings
- **Official Chrome MV2 timeline:**
  - **Jun 3, 2024:** phase-out began in Beta, Dev and Canary.
  - **Oct 9, 2024:** stable disabling began.
  - **Mar 31, 2025:** disabled by default, re-enable still allowed.
  - **Jul 24, 2025:** Chrome 138 disabled MV2 on all channels with no re-enable. The ExtensionManifestV2Availability enterprise policy is removed in Chrome 139.
  - **Aug 31, 2026:** all remaining MV2 extensions removed from the Chrome Web Store. — [developer.chrome.com MV2 timeline](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline)
- MV3 declarativeNetRequest limits (official):
  - Guaranteed minimum static rules: 30,000 per extension, plus a shared global pool.
  - Max enabled static rulesets: 50. Max static rulesets: 100.
  - Dynamic rules: 30,000, of which 5,000 can be "unsafe".
  - Session rules: 5,000. Regex rules: 1,000. — [chrome.declarativeNetRequest](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest)
- uBOL is fully declarative. Per the project description it lacks the following by default:
  - Cosmetic filtering
  - Scriptlet injection
  - redirect/csp/removeparam filters

  The uBOL FAQ notes that rule counts stay under 30K even with many lists enabled, and that the binding constraint has been the number of enabled rulesets. — [uBOL FAQ](https://github.com/uBlockOrigin/uBOL-home/wiki/Frequently-asked-questions-%28FAQ%29); [Vivaldi forum quote](https://forum.vivaldi.net/topic/79446/ublock-origin-lite/7)
- **Edge:** On Aug 7 (2026), Microsoft said it would begin retiring MV2 for consumers that month, rolling out Canary → Dev → Beta → Stable. It aims to finish the consumer transition by end of 2026, with enterprise deprecation in early 2027. Edge Canary had disabled uBO in early 2025, with re-enable possible. — [BetaNews](https://betanews.com/article/firefox-brave-ublock-origin-chrome-edge/); [Gigazine Mar 2025](https://wbgsv0a.gigazine.net/gsc_news/en/20250305-microsoft-edge-turning-off-ublock-origin); [tbreak](https://tbreak.com/microsoft-edge-mv2-ad-blockers/) (secondary)
- **Firefox / Brave:** Firefox continues to support MV2 and full uBO. Brave users can enable selected MV2 extensions under Settings > Extensions. — [Cybernews](https://cybernews.com/security/firefox-ublock-origin-chromium-adblocker/); [BetaNews](https://betanews.com/article/firefox-brave-ublock-origin-chrome-edge/) (secondary)
- TechRadar: "Google Chrome Manifest V2 support has officially ended". — [TechRadar](https://techradar.com/pro/google-chrome-manifest-v2-support-has-officially-ended-adios-ublock-origin)

### Inferences
- Corrected claim: "On Chrome (since July 2025), full uBlock Origin no longer runs, and it was removed from the Web Store on Aug 31, 2026. Users get uBO Lite, which drops cosmetic/scriptlet filtering by default. Full uBO remains available on Firefox. Edge is completing its MV2 shutdown by end-2026."

### Gaps
- Edge and Brave status comes from secondary news, not Microsoft or Brave primary posts. How long Brave's MV2 allowance will last is unverified.
- I found no independent measurement of uBOL vs. uBO blocking efficacy.

## Claim 6: "Over 100 malicious extensions found on the Chrome Web Store," and malware via legitimate stores

### Takeaway
**Verdict: Correct.** The "100+" figure matches the DomainTools report (May 2025): over 100 fake sites and extensions impersonating VPN, AI and crypto tools, active since at least Feb 2024. Later campaigns are larger by user count, notably:
- Koi Security's RedDirection (July 2025, 18 extensions, ~2.3M users)
- ShadyPanda (Dec 2025, ~4.3M Chrome+Edge users)
- A Feb 2026 report of 287 "AI assistant" extensions with 37.4M users (secondary, treat with caution)

### Cited Findings
- **DomainTools (reported May 2025):** 100+ malicious Chrome extensions and lookalike sites impersonating DeepSeek, Manus, DeBank, FortiVPN, YouTube and Site Stats, active since ≥Feb 2024. Capabilities:
  - Cookie theft
  - Remote script execution
  - Proxying and traffic modification
  - Ad injection and redirects

  Google removed many, but some persisted. — [TechRadar](https://www.techradar.com/pro/security/these-dangerous-fake-google-chrome-extensions-spoof-vpns-and-youtube); [Daily Security Review](https://dailysecurityreview.com/security-spotlight/over-100-malicious-chrome-extensions-found-masquerading-as-ai-tools-vpns-and-crypto-utilities/); [HackMag](https://hackmag.com/news/chrome-extensions-malware)
- **Koi Security (July 2025):** 11 Chrome extensions with ~1.7M downloads, several with "Featured"/verified status, turned malicious via updates. Part of "RedDirection": 18 extensions across Chrome and Edge, ~2.3M users. — [Tribune](https://tribune.com.pk/story/2554951/malicious-google-chrome-extensions-with-over-17m-downloads-discovered) (secondary)
- **ShadyPanda (Koi Security, Dec 2025):** a seven-year campaign reaching ~4.3M Chrome and Edge users.
  - Five Chrome extensions (incl. Clean Master; ~300k users) were legitimate since 2018 and got an RCE backdoor in a mid-2024 update.
  - Five Edge extensions from Starlab Technology (>4M installs; WeTab ~3M) collected browsing history and fingerprints.
  - Reports conflict on whether Edge removed them promptly. — [Infosecurity Magazine](https://infosecurity-magazine.com/news/shadypanda-infects-43m-chrome-edge); [TechRadar](https://www.techradar.com/pro/security/4-3-million-have-installed-this-malicious-browser-extension-on-chrome-and-edge-heres-how-to-check); [Security Boulevard](https://securityboulevard.com/2025/12/shadypandas-years-long-browser-hack-infected-4-3-million-users/)
- Dec 2024/Jan 2025: a phishing attack on developer accounts (Cyberhaven and others) led to malicious updates pushed through the Chrome Web Store. — [heise](https://heise.de/-10225127)
- **Feb 2026 (unverified):** LayerX reportedly found 287 malicious extensions posing as AI assistants, with 37.4M users. This comes only via an aggregator. — [ThreatCluster](https://threatcluster.io/cluster/malicious-chrome-extensions-target-300k-users-to-steal-data-6e9f9b14)

### Inferences
- The dominant 2025–2026 pattern is **trusted extensions turning malicious via updates** (ShadyPanda, RedDirection, Cyberhaven), which defeats one-time store review. Some carried Google "Featured"/verified badges.
- Corrected claim: "Multiple 2025 reports found 100+ malicious extensions in a single campaign (DomainTools, May 2025). Update-based hijacks reached millions of users (RedDirection 2.3M, ShadyPanda 4.3M)."

### Gaps
- I did not read the primary reports from DomainTools, Koi or LayerX. The LayerX 287/37.4M figure needs primary confirmation.

## Mitigations: Built-in Content Blocking, Cookie-Banner Handling, GPC, and Extension Security Models

### Takeaway
Engine-level blocking (Brave Shields' Rust engine, Safari ITP/AFP, Firefox ETP) is unaffected by Chrome's MV3 limits. The strongest measured, engine-level gains in 2025–2026 are:
- Firefox's fingerprinting cut (users seen as unique down by "almost half")
- Brave's ~75% adblock-engine memory reduction (a performance gain, not a blocking-rate measurement)

Firefox's built-in cookie-banner reduction **is no longer available** per Mozilla support. Mozilla now points users to uBO cookie-notice lists. GPC gains legal force through California's **AB 566 (Opt Me Out Act)**, which requires browsers to offer an opt-out preference signal from **Jan 1, 2027**. But the webXray audit shows that sites often ignore GPC today.

### Cited Findings
- **Brave Shields (adblock-rust):**
  - The FlatBuffers rewrite cut adblock-engine memory by ~75% (~45 MB saved by default on Android, iOS and desktop) across ~100k default filters, shipped in v1.85.
  - The Register measured ~104 MB (1.85) vs. 162 MB (1.79, May 2025) for browser-process totals. — [Brave](https://brave.com/privacy-updates/36-adblock-memory-reduction); [The Register, Jan 6 2026](https://www.theregister.com/2026/01/06/brave_refurbishes_rust_adblocking_engine/)
- **Firefox cookie banner reduction:**
  - Launched in Firefox 120. It set opt-out cookies or auto-clicked "reject" on a curated site list, and was default-on in private windows in Germany.
  - Mozilla's support page now says the feature is no longer available and suggests uBO with "AdGuard/uBO – Cookie Notices" lists. — [Mozilla Support](https://support.mozilla.org/kb/cookie-banner-reduction)
- **CookieBlock (research client-side enforcement):** filters ~90% of privacy-invasive cookies on 100 sampled sites without significantly breaking functionality. Cookie classifier accuracy is 84.4%. — [USENIX Sec 2022](https://usenix.org/conference/usenixsecurity22/presentation/bollinger)
- **GPC legal status:**
  - California AB 566 was signed Oct 8, 2025 and takes effect Jan 1, 2027. Browsers must include a user-controlled opt-out preference signal setting. It is CPPA-enforced, and browser makers are shielded from liability for businesses' violations. A similar bill had been vetoed the year before. — [McDonald Hopkins](https://www.mcdonaldhopkins.com/insights/news/California-enacts-first-in-the-nation-browser-opt-out-requirements); [Thompson Coburn](https://www.thompsoncoburn.com/insights/cybersecurity-month-legislative-update-californias-opt-me-out-act-sets-new-st-102lpdo/); [MediaPost](https://www.mediapost.com/publications/article/403402/california-bill-requiring-browser-based-opt-out-re.html)
  - Other states requiring businesses to honor opt-out signals (CO, CT, MT, TX, MN, MD, etc.) are listed by a vendor blog only. Verify before citing. — [Captain Compliance](https://captaincompliance.com/education/california-opt-me-out-act-ab-566-law/)
  - Measured GPC compliance: 55% of 7,634 sites set ad cookies despite GPC, and all 11 CMPs failed (webXray, Mar 2026). — [globalprivacyaudit.org](https://globalprivacyaudit.org/2026/california)
- **Firefox fingerprinting protection (ETP Strict/Private, Firefox 145):** users seen as unique cut by "almost half". — [SecurityWeek](https://www.securityweek.com/new-firefox-protections-halve-the-number-of-trackable-users/)
- **Safari 26 AFP:** default in all modes. — [9to5Mac](https://9to5mac.com/2025/09/01/security-bite-my-favorite-privacy-features-in-ios-26/)
- **Chrome:**
  - Third-party cookies are blocked by default only in Incognito. — [Privacy Sandbox blog](https://privacysandbox.google.com/blog/privacy-sandbox-next-steps)
  - MV3 DNR caps ad-blocking extensions at the published limits. — [chrome.declarativeNetRequest](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest)
- **Extension security models:** MV3's declarative, rule-capped model restricts what blockers can do. Store review did not prevent update-based compromise in multiple 2024–2025 incidents, including Featured/verified extensions. — [Infosecurity Magazine](https://infosecurity-magazine.com/news/shadypanda-infects-43m-chrome-edge); [heise](https://heise.de/-10225127)

### Inferences
- For a new browser, the evidence favors:
  - A native, engine-level blocker (Brave-style), independent of extension-API limits.
  - Default-on fingerprinting protection (the Safari 26 model).
  - Sending GPC by default (ahead of the AB 566 deadline).
  - Built-in cookie-notice handling via maintained lists rather than a curated per-site approach (Firefox's curated approach was discontinued).
- For extensions: the threat is malicious *updates* to trusted extensions. Useful mitigations would be update-time re-review, permission-escalation prompts on update, and enterprise allowlists. Their measured effect is not quantified in sources found.

### Gaps
- No primary sources gathered for the following:
  - Brave's cookie-notice blocking ("Block cookie consent notices") and its success rate
  - Consent-O-Matic measured success rates
  - Opera/Vivaldi built-in blockers
  - Safari ITP efficacy numbers
  - Firefox ETP block counts
- No independent study quantifies how often uBOL misses ads compared with full uBO.
- I did not verify the date or reason for removing Firefox's cookie banner reduction.
- There is no source-verified data on Chrome Web Store, AMO or Edge review processes or on permission-model changes in 2025–2026.
