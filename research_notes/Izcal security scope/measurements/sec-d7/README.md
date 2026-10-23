# SEC-D7 list source: which phishing, malware-URL and download-reputation sources fit (b)

**Status: FINAL (2026-10-19, about 02:20 UTC).** This version uses the desk research and **five** feed samples taken from 2026-10-18T22:41:14Z to 2026-10-19T02:07:33Z (3 h 26 min). An interim version, written from sample 1 only, was sent to the owner at about 23:00 UTC. The recommendation has not changed since then. Measurements added since: churn (4.3), cross-feed lag (4.4) and Phishunt. PhishTank's measured intake is now lower than the interim suggested.

Scope: SEC-D7 is decided as (b). Izcal writes its own local-list protection from a published privacy-preserving lookup protocol and ships the list as a TUF data channel (UPD-R1). The same list serves download reputation, and the protection is required for agent tasks (B9). This note chooses the **list source**. Constraints come from the spec:

- The protocol and the source each need a recorded privacy analysis (P1, B10's classifier rules).
- Fallback trigger: "the chosen list source requires sending browsing data the privacy analysis rejects".
- Izcal is distributed publicly and may become commercial, so the licence must allow **commercial redistribution inside a browser**.

Legend: **UNVERIFIED** means a claim I could not confirm against a primary source today. "Measured" means I downloaded the feed file and analysed it as text; no listed URL was fetched or resolved.

---

## 1. Recommendation

No single source is all three of global in coverage, free and licensed for commercial redistribution. Google's lists are the strongest technically, but both Google paths forbid what Izcal needs (Section 3.1). The recommendation is a **two-tier source**.

**Tier 1: ship now, $0. An open, commercially redistributable phishing composite, matched locally.**
- **CERT Polska warning list** (hole.cert.pl) is the primary.
  - **Licence:** may be used "for any purpose, including commercial purposes", with no attribution required (https://cert.pl/en/warning-list/).
  - **Operator and size:** run by a state body (NASK-PIB); 124,483 domains (2.56 MB) measured.
  - **Freshness:** the file was modified 73 s before my fetch. CERT Polska claims a 5-minute refresh.
  - **Caveat:** its scope is phishing aimed at Polish users. It does not provide global coverage on its own.
- **Phishunt** (CC0, https://phishunt.io/tos/) and **URLAbuse** (CC BY 4.0 with attribution, https://urlabuse.com/terms.html) add global phishing URLs.
  - URLAbuse's full feed needs a free account, which I did not create. Its sample was assessed from documentation only.
  - Phishunt measured: 770–779 URLs (33 KB), 2–11 new URLs per hour, and 96% of its hosts are already in Phishing Army, which aggregates it. Only 2.2% overlap with CERT Polska and 8.1% with PhishTank, so it is complementary. Its false-positive proxy is clean: 2 popular-apex hits, both lookalikes (Section 4.6).
- **PhishTank** joins only after Cisco Talos gives written confirmation of redistribution rights.
  - The FAQ says commercial use is "OK", but no data licence exists and new registration is closed (https://phishtank.org/faq.php, https://phishtank.org/register.php).
  - It is the largest standing global input: 59,685 verified-online URLs, only 8% host overlap with CERT Polska, median verification 11.7 min.
  - Its **intake is small**, though: 7 new URLs in 3 h 26 min (0–5 per hour) against 21–28 removed per hour. All 7 new hosts were already in CERT Polska (Allegro-targeted Polish phishing). Its value is the standing global backlog more than freshness, which lowers its priority.
- **Privacy:** Izcal ships the whole list (hashed or plain) through TUF and matches entirely on the device. **No browsing data leaves the device.** The only exposure is that a client fetched the list, which the TUF channel exposes for every data channel anyway.

**Tier 2: before the commercial release, buy a global licensed phishing feed with written redistribution rights.**
- The proven model is DuckDuckGo's: a licensed Netcraft feed served from DuckDuckGo's own servers, with local hash prefixes (https://duckduckgo.com/duckduckgo-help-pages/threat-protection/scam-blocker). The alternative is APWG eCX at the Sponsoring or Premium tier ($7,500 or $15,000 a year; redistribution terms are under an agreement that is not public; https://apwg.org/membership/).
- Literature context: eCX was the most complete list on free-builder phishing at 32.9%, against GSB's 18.4% (https://arxiv.org/html/2212.02563v2).
- Netcraft pricing and terms are not public (**UNVERIFIED**; sales only).

**Malware URLs: abuse.ch URLhaus, under a Spamhaus commercial licence. No free source is licensed for this.**
- URLhaus is the best open malware-URL source measured:
  - 27,723 URLs online in its online export, from 1,984 hosts.
  - 32,284 URLs in its 30-day "recent" file. The newest entry was 7 minutes old when I fetched it.
  - It shares data with GSB, the Spamhaus DBL and SURBL (https://urlhaus.abuse.ch/about/).
- The abuse.ch Terms of Use (updated 4 November 2025) allow free use only "for not-for-profit purposes". Commercial use "may require a paid subscription, which will be managed by Spamhaus", and §7.3 bars derivative works (https://abuse.ch/terms-of-use/). The price is by quote (https://www.spamhaus.com/data-access/abusech-api/), so the cost is **UNVERIFIED**.
- Until the licence is signed, malware-URL protection is the documented gap. Option (a) applies for malware URLs only.

**Download reputation: URL and host reputation of the download source, plus SH-R7's type rules. A hash list is secondary.**
- **No public hash-reputation API for browsers:** Google's binary-reputation service is "internal access only" (https://wiki.mozilla.org/Security/Application_Reputation), and SmartScreen has no third-party API.
- **MalwareBazaar recent hashes are a weak blocking signal:** the export held 900 SHA-256 values, and new malware is mostly polymorphic.
  - Measured: 6–11 new hashes per hour; the export is a rolling window of about 900 hashes, rewritten hourly. An exact-hash match only catches a file already submitted to MalwareBazaar; whether new malware is mostly polymorphic is an inference, not measured.
  - The export falls under the same abuse.ch/Spamhaus terms.
- **Primary download check:** match the final download URL and every redirect host against URLhaus `malware_download` entries, which make up 100% of URLhaus's threat class. Add MalwareBazaar and ThreatFox SHA-256 values under the same licence.

**Rejected as sources for (b):**
- **Google Safe Browsing API (v4/v5):** "you may not use the Safe Browsing API for commercial purposes" without a separate agreement, plus a 30-minute "received from Google" freshness rule (https://developers.google.com/safe-browsing/terms).
- **Google Web Risk:** "must not be redistributed" (https://docs.cloud.google.com/web-risk/docs/overview).
- **OpenPhish community:** non-commercial only; redistribution needs written consent (https://openphish.com/terms.html).
- **Phishing Army and Hagezi:** pass on no rights to upstream data that is non-commercial (https://phishing.army/license/).
- **PhishStats:** no redistribution without written permission; CSV dump gone (https://phishstats.info/terms).
- **urlscan.io:** written permission required (https://urlscan.io/terms/).
- **Phishing.Database:** MIT licence, but stalled since 2026-10-02, with whole popular domains listed (Section 4.5).
- **Spamhaus DBL and SURBL as a DNS query:** each lookup sends the domain to a third party; free tiers exclude products.
- **SmartScreen, Apple and Yandex:** no usable third-party access, or no verified terms.

**Cost**
- **Tier 1:** $0 in licences, plus Izcal's own aggregation, allowlisting and TUF publishing work.
- **Tier 2:** APWG $7.5k–15k a year, or Netcraft by quote.
- **Malware:** Spamhaus/abuse.ch by quote.

**Reversal triggers.** The spec's trigger stays. Source-specific triggers:
1. Any chosen source requires a per-URL online lookup, or its contract requires sending URLs or telemetry. Drop that source; if none remains, fall back to (a) for browsing and keep the protection for agent tasks.
2. CERT Polska changes its terms or stops publishing for more than 7 days.
3. A licence term is withdrawn: PhishTank denies redistribution, or Spamhaus pricing makes the malware tier unaffordable. Remove that input and record the gap.
4. Measured false positives on popular sites exceed a threshold set in SEC targets. Suggested: any listed apex in the Tranco top 10k that is not on the allowlist blocks the release of that channel version.

---

## 2. Comparison table

| Source | Active (2026-10-18) | Coverage | Freshness | Privacy model if used as (b) | Terms for a commercial browser | Fit with (b) |
|---|---|---|---|---|---|---|
| **Google Safe Browsing v4** | Deprecated, no shutdown date; Firefox migrating off ([v4](https://developers.google.com/safe-browsing/v4), [FF 147](https://www.firefox.com/en-US/firefox/147.0/releasenotes/)) | Phishing, malware, unwanted, social engineering | List refresh 20–50 min (Google) | Local hash prefixes; on match, prefix(es) plus IP sent to Google | Non-commercial only without an agreement | **No** (terms, deprecated) |
| **Google Safe Browsing v5** | Active; v5 docs live | Same plus Global Cache | Real-time mode; "majority" of unsafe sites live under 10 min ([blog](https://security.googleblog.com/2024/03/blog-post.html)) | 4-byte prefixes, up to 30 per lookup, decoys allowed; optional OHTTP relay hides IP ([ref](https://developers.google.com/safe-browsing/reference)) | Same ban; 30-min freshness rule; "Advisory provided by Google" required ([terms](https://developers.google.com/safe-browsing/terms)) | Protocol: **yes, a good model**. Source: **no** without a separate Google deal; no self-hosted mirror |
| **Google Web Risk** | Active | Same as GSB | Same | Lookup sends the URL; Update API is prefix-based | Paid ($0.50 per 1k lookups; $50 per 1k `hashes.search`); **"must not be redistributed"** ([pricing](https://cloud.google.com/web-risk/pricing), [overview](https://docs.cloud.google.com/web-risk/docs/overview)) | **No**: no redistribution; consumer-browser use **UNVERIFIED** |
| **Microsoft SmartScreen** | Active (Edge/Windows only) | Phishing, malware, download reputation | Online | Online lookup of URL/file data; 2019 reports of full URLs ([MS Learn](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-security-smartscreen)) | No third-party API found | **No** |
| **Apple (Safari)** | Active | Uses GSB, or Tencent in mainland China; Apple proxies since iOS 14.5 ([9to5mac](https://9to5mac.com/2021/02/11/fraudulent-website-warning/)) | n/a | Prefixes via Apple proxy | No third-party access | **No** (only a design precedent: a proxy in front of the lookup) |
| **Yandex Safe Browsing** | Docs live ([docs](https://yandex.com/dev/safebrowsing/doc/en/)) | Phishing, malware | **UNVERIFIED** | v4-style prefixes to Yandex; DSN 2016 found its lists contain concrete URLs that are easy to re-identify | Free key; terms **UNVERIFIED** | **No** (unverified terms, privacy finding, jurisdiction) |
| **Netcraft** | Active; licensees reportedly include DuckDuckGo ([DDG](https://duckduckgo.com/duckduckgo-help-pages/threat-protection/scam-blocker)) | Phishing, scams, malware | JSON feed every 5 min (vendor, snippet; **UNVERIFIED**) | DDG model: local 8-hex prefixes; uncommon threats send only 4 hex characters to DDG's own server | Commercial, by quote; redistribution negotiable (**UNVERIFIED**) | **Yes, if licensed** (Tier 2 candidate) |
| **APWG eCX** | Active ([eCX](https://apwg.org/ecx/)) | Phishing, malicious IPs, domains | Real-time; best free-builder coverage, 32.9% (2023) | Feed only; Izcal decides | Dues $7.5k/$15k a year for phish modules; data-sharing agreement not public ([membership](https://apwg.org/membership/)) | **Possible** (Tier 2), if redistribution is granted in writing |
| **CERT Polska** | Active; file modified 73 s before fetch | Phishing domains (PL-targeted); subdomains included | Claims 5 min; entries kept 6 months | Full local list: **no leak** | Any purpose including commercial; no attribution; do not imply endorsement ([cert.pl](https://cert.pl/en/warning-list/)) | **Yes** (Tier 1 primary) |
| **Phishunt** | Active; feed.txt modified 2026-10-18 22:47Z | Phishing URLs | Real-time (claim); measured 2–11 new URLs/h | Full local list | CC0; do not claim entries are "confirmed phishing" ([tos](https://phishunt.io/tos/)) | **Yes** (Tier 1; single maintainer) |
| **URLAbuse** | Active; about 23.5k records in 30 days ([home](https://urlabuse.com/)) | Phishing and malware URLs | **UNVERIFIED** (no account) | Full local list | CC BY 4.0, commercial allowed, attribution ([terms](https://urlabuse.com/terms.html)) | **Yes** (Tier 1; needs a free account, which the owner must create) |
| **PhishTank** | Active (new IDs about 9,546,200; file modified 22:21Z); **registration closed** ([register](https://phishtank.org/register.php)) | Phishing URLs, community-verified | Hourly dumps; measured verification median 11.7 min, p90 207 min | Full local list | FAQ: commercial use "OK"; no data licence; redistribution unaddressed ([faq](https://phishtank.org/faq.php)) | **Conditional**: needs written OK from Talos |
| **OpenPhish community** | Active; GitHub commits every 12 h ([repo](https://github.com/openphish/public_feed)) | 300 phishing URLs | 12 h (community); 5 min (premium) | Full local list | **Non-commercial**; no redistribution without written consent ([terms](https://openphish.com/terms.html)) | **No** (community); premium by quote |
| **abuse.ch URLhaus** | Active; export modified 4 min before fetch | Malware-download URLs | Newest entry 7 min old; 30-day window | Full local list | Not-for-profit only; commercial via Spamhaus, by quote; §7.3 bars derivatives ([ToU](https://abuse.ch/terms-of-use/)) | **Yes, if licensed** (malware primary) |
| **abuse.ch ThreatFox** | Active | Command-and-control, payload delivery IOCs (URL, domain, IP:port, hashes) | Newest 10 min old | Full local list | Same as URLhaus | Secondary (mostly IP:port C2, low browser value) |
| **abuse.ch MalwareBazaar** | Active; export modified 22:05Z | Malware sample hashes | Recent export (900 SHA-256) | Local hash match on downloaded file: **no leak** | Same as URLhaus | Secondary download signal |
| **Phishing.Database** | **Stalled**: last commit 2026-10-02; month-long gaps in Aug–Sep ([repo](https://github.com/Phishing-Database/Phishing.Database)) | 392,071 "active" domains | Not fresh | Full local list | MIT | **No** (stale, false positives, Section 4.5) |
| **Phishing Army** | Active; generated 22:00Z; 4 h expiry; daily allowlisting commits | 153,648 domains (aggregate) | Aggregate of others | Full local list | Adds no restriction, but **grants no upstream rights**; includes OpenPhish and urlscan ([license](https://phishing.army/license/), [SOURCES](https://github.com/drego85/PhishingArmyCore/blob/master/SOURCES.md)) | **No** as a source; its allowlist practice is a model |
| **Hagezi TIF** | Active (several releases a day) | Aggregate domains | Aggregate | Local | GPL-3.0 for lists, but "every source has its own license" ([sources](https://github.com/hagezi/dns-blocklists/blob/main/sources.md)) | **No** (same problem) |
| **PhishStats** | Active (API); **phish_score.csv returns 404** | Phishing URLs | Updated every 90 min (claim); measured about 14 entries/h | API lookup or bulk | No redistribution without written permission; 50 requests/day anonymous ([terms](https://phishstats.info/terms)) | **No** |
| **urlscan.io** | Active | Scans, phishing verdicts | Real-time | n/a | Commercial use needs written permission ([terms](https://urlscan.io/terms/)) | **No** without a contract |
| **Spamhaus DBL / SURBL** | Active | Spam, phishing, malware domains | Near real-time | **DNS query per domain**: domain sent to resolver and operator | Free non-commercial or small use only; products need a paid feed ([Spamhaus](https://www.spamhaus.org/organization/dnsblusage/), [SURBL](https://www.surbl.org/usage-policy)) | DNS mode **no**; a paid rsync bulk feed is possible (**UNVERIFIED** price) |
| **CERT-AgID (IT)** | Active | IoCs | n/a | n/a | Accredited bodies only ([feed](https://cert-agid.gov.it/feed/)) | **No** |
| **Other national CERT feeds (NL, NCSC, CISA AIS)** | **UNVERIFIED** | | | | | Not assessed |
| **Quad9 / Cloudflare 1.1.1.2 (model)** | Active | About 20 sources (Quad9), including CERT.PL, URLhaus, URLAbuse, OpenPhish ([partners](https://quad9.net/about/partners/)) | Resolver-side | Every domain lookup goes to the resolver | n/a | Model only: shows the composite approach; (b) keeps lookups local instead |

What other browsers chose (source choice only; no code consulted):

| Browser | Source | Notes |
|---|---|---|
| Firefox | Google SB, moving to v5 local-list mode (Firefox 147) | [Firefox 147 notes](https://www.firefox.com/en-US/firefox/147.0/releasenotes/); proxying **UNVERIFIED** |
| Brave | Google SB, proxied on desktop | [Brave privacy](https://brave.com/privacy/browser/) |
| Vivaldi | Google SB | Proxying **UNVERIFIED** ([blog](https://vivaldi.com/blog/vivaldi-powerful-privacy-settings/)) |
| Opera | PhishTank for phishing, Yandex for malware | Undated page, may be stale ([Opera](https://security.opera.com/en/?p=1081)) |
| DuckDuckGo | Licensed Netcraft feed, served by DDG, local prefixes | [DDG](https://duckduckgo.com/duckduckgo-help-pages/threat-protection/scam-blocker) |
| Tor Browser | Safe Browsing off | [Tor forum](https://forum.torproject.org/t/torbrowser-googlesafebrowsing/10397) (older evidence) |
| Ladybird | None; possible future web service | [issue 4989](https://github.com/LadybirdBrowser/ladybird/issues/4989) |
| Orion | **UNVERIFIED** (no documentation found) | |
| Safari | GSB, or Tencent in mainland China, Apple-proxied | Above |

---

## 3. Key findings from research

### 3.1 Google: the protocol is usable, the source is not

- **v4 Update API:** "deprecated", with no published shutdown date (https://developers.google.com/safe-browsing/v4).
  - Mozilla cites "the upcoming decommissioning of SafeBrowsing v4 server" (https://groups.google.com/a/mozilla.org/g/firefox-dev/c/0-95oLzcb9c).
  - A one-year notice promise is **UNVERIFIED**.
- **v5 modes:** Real-Time, Local List and No-Storage Real-Time.
  - `hashes.search` sends only 4-byte SHA-256 prefixes, at most 30 per lookup, and decoys are allowed.
  - Using an OHTTP relay (RFC 9458) is optional. The relay is a third party such as Fastly and must authenticate to Google with OAuth scope `3p-relay-safe-browsing` (https://developers.google.com/safe-browsing/reference, https://developers.google.com/safe-browsing/ohttp/reference).
- **The v5 local-list design is the protocol to adopt for Izcal's own list:**
  - Rice-Golomb-coded hash prefixes and hash-prefix diffs.
  - Host-suffix and path-prefix expressions.
  - An optional prefix-match confirmation through a non-colluding relay.

  Izcal can implement this from the published specification (M-1 compliant) and serve its own data in that format through TUF.
- **The Google data cannot be used:**
  - Safe Browsing terms (modified 2025-11-20): "Unless you have a separate agreement with Google, you may not use the Safe Browsing API for commercial purposes." You also may not warn "unless your application has received from Google updated information … within the past thirty minutes" (https://developers.google.com/safe-browsing/terms). A TUF mirror run by Izcal breaks the second clause, and an Izcal that might become commercial breaks the first.
  - Web Risk: "The information returned by the Web Risk must not be redistributed" (https://docs.cloud.google.com/web-risk/docs/overview).
  - A separate agreement with Google is the only way, and it is not a published option.

### 3.2 Literature

- **Speed of the big lists, for comparison** (PhishTime, USENIX Security 2020, https://www.usenix.org/conference/usenixsecurity20/presentation/oest-phishtime):
  - GSB covered 92.9% of reported test phish, with a median of about 56 min to list. SmartScreen covered 93.2% at about 3 h 47 min.
  - Mobile coverage was much lower, and cloaking drops coverage.
- **Free-builder phishing** (https://arxiv.org/html/2212.02563v2, 2023): coverage was GSB 18.4%, eCrimeX 32.9%, OpenPhish 11.7%, PhishTank 4.1%. Every list is weak there.
- **Victim timing** (Sunrise to Sunset, https://www.usenix.org/conference/usenixsecurity20/presentation/oest-sunrise): a campaign lasts about 21 h. Detection comes about 9 h after the first victim, so freshness of minutes to hours matters.
- **Cloaking** (CrawlPhish, PhishDecloaker): cloaking defeats lists. Lists are a floor, not a full defence. This matters for agent tasks, where an agent cannot judge a page visually.
- **Hash-prefix privacy:** prefixes can re-identify visits to small domains, and multi-prefix matches allow tracking (Gerbet, Kumar, Lauradoux, DSN 2016, https://ieeexplore.ieee.org/document/7579754). The detailed figures in that paper are **UNVERIFIED**. A full local list avoids the issue altogether.

### 3.3 abuse.ch access: documentation and observation disagree

- The docs say the URLhaus downloads need an Auth-Key (https://urlhaus.abuse.ch/api/; "Community First", https://abuse.ch/blog/community-first/).
- My unauthenticated downloads of `/downloads/text_online/`, `/downloads/csv_recent/`, ThreatFox `/export/csv/recent/` and MalwareBazaar `/export/txt/sha256/recent/` all returned **HTTP 200** at 22:41Z.
- Either way, the **Terms of Use** govern use, not access control: they allow not-for-profit use only (https://abuse.ch/terms-of-use/).

---

## 4. Measurements

### 4.1 Method

- **Feed URLs and fetching:** listed in `scripts/fetch_once.sh`. Each feed host was fetched with curl, HTTPS only (`--proto '=https'`), saving headers and body to `feeds/<UTC timestamp>/`.
- **Analysis:** `scripts/analyse.py`, run with `python3 -I`, parses each body as untrusted text.
  - **Normalisation:** hostname lowercased, trailing dot and leading `www.` removed. URLs normalised to host plus path plus query, with the scheme dropped.
  - **Registrable domain:** computed with the Public Suffix List (publicsuffix.org, fetched 22:44Z).
  - **Popularity reference:** Tranco top-1M, list ID `8PX8V`.
- **No listed URL or host was requested or resolved.**
- **PhishStats:** the local LAN resolver (192.168.50.128) answers `phishstats.info` with 0.0.0.0. A local blocklist is producing a false positive on a phishing-research site. I used the API host `api.phishstats.info`, which resolves normally.
- **Schedule:**
  - Sample 1: 2026-10-18T22:41:14Z.
  - Samples 2–5: 2026-10-18T23:07:17Z, 2026-10-19T00:07:22Z, 01:07:27Z, 02:07:33Z. All 14 feed fetches returned HTTP 200 in every sample after sample 1.
  - Phishunt is included from sample 2.

### 4.2 Sample 1: sizes, counts, server timestamps (2026-10-18T22:41:14Z)

| Feed | Bytes | Entries (normalised) | Hosts | Server Last-Modified / internal timestamp | Entry age at fetch |
|---|---|---|---|---|---|
| OpenPhish community (openphish.com redirects to GitHub raw) | 14,313 | 300 lines; 296 URLs | 273 | GitHub commits at 00:00 and 12:00 UTC (12 h) | n/a (no timestamps) |
| PhishTank verified_online.csv (no app key; redirected to cdn.phishtank.com) | 11,675,526 | 59,685 URLs (60,872 rows) | 30,485 | Last-Modified 22:21:56Z | median 10,104 h (about 421 days); p10 1,767 h; newest 1.1 h. Submit-to-verify median **11.7 min**, p90 207 min |
| PhishStats API (latest 100; server cap is 100 even with `_size=500`) | 177,117 | 99 URLs | 37 | — | median 4.1 h, max 7.2 h, so about **14 new entries/h** |
| CERT Polska domains.txt | 2,560,830 | 124,483 domains | 124,483 | Last-Modified 22:40:06Z | n/a |
| Phishing Army blocklist | 3,415,483 | 153,648 (header: 156,545) | 153,648 | Generated 22:00:25Z; Expires 4 h | n/a |
| Phishing Army "extended" | identical bytes to standard | — | — | — | — |
| Phishing.Database ACTIVE domains | 11,023,128 | 392,071 | 392,071 | last commit 2026-10-02 15:30Z | n/a |
| Phishing.Database NEW-today | 143 / 91 | 3 links / 3 domains | — | stale (2026-10-02) | — |
| URLhaus text_online | 2,774,162 | 27,723 URLs | 1,984 | Last-Modified 22:37:15Z | — |
| URLhaus csv_recent (30 days) | 7,875,655 | 32,284 URLs (33,683 rows; 16,497 online; 100% `malware_download`) | 7,254 | "Last updated 22:34:20 UTC" | newest 7 min; median 304 h; max 743 h |
| ThreatFox recent | 1,687,137 | 7,486 IOCs: 3,659 ip:port, 2,042 domain, 823 url, 820 sha256 | 2,202 | "Last updated 22:30:48 UTC" | newest 10 min; median 221 h |
| MalwareBazaar recent SHA-256 | 59,906 | 900 hashes | — | "Last updated 22:03:11 UTC" | — |
| Phishunt feed.txt (from sample 2) | 33,621 | 772 URLs | 770 | Last-Modified equals fetch time (generated on request) | n/a (no timestamps) |

- **Fetch errors:** `https://phishstats.info/phish_score.csv` gave a connection failure through the LAN resolver, then **HTTP 404** from the real address. No announcement found. All other feeds returned 200.
- **Integrity:** no feed is signed. All are HTTPS only, from either the operator's own host or GitHub raw.
- **ETags:** present on most feeds. Phishing Army sends only Last-Modified.

### 4.3 Churn and update cadence (samples 1–5)

The table shows added / removed entries per interval. The intervals are 26, 60, 60 and 60 min. The unit is URLs for URL feeds, hosts for domain lists and hashes for MalwareBazaar.

| Feed | 22:41→23:07 | 23:07→00:07 | 00:07→01:07 | 01:07→02:07 | Survived from first to last | Measured cadence (Last-Modified or internal timestamp) |
|---|---|---|---|---|---|---|
| OpenPhish community | 0 / 0 | **162 / 160** | 0 / 0 | 0 / 0 | 136 of 296 (46%) | One change, at the 00:00 UTC GitHub push: the 12 h cadence is confirmed. It is a rolling window of about 300 URLs |
| PhishTank | 0 / 0 | 2 / 22 | 5 / 21 | 0 / 28 | 59,614 of 59,685 | Dump rebuilt hourly (Last-Modified xx:22). Intake 0–5 URLs/h; removals 21–28/h |
| PhishStats API (latest 100) | 10 / 9 | 15 / 15 | 10 / 10 | 7 / 7 | 58 of 99 | 7–23 new/h |
| Phishunt | n/a | 11 / 13 | 7 / 0 | 2 / 0 | 759 of 772 | Generated on request; 2–11 new/h |
| CERT Polska | 36 / 23 | 0 / 0 | 36 / 18 | 0 / 15 | 124,427 of 124,483 | Last-Modified always xx:05:06 or xx:40:06, consistent with the claimed 5-min build. About 72 new domains in 3.4 h (about 21/h) |
| Phishing Army | 0 / 0 | 0 / 0 | **909 / 964** | 0 / 0 | 152,684 of 153,648 | Rebuilt at 22:00:25Z and 01:02:16Z, about 3 h apart (it states a 4 h expiry) |
| Phishing.Database (all files) | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | all | **No change at all**; stalled since 2026-10-02 |
| URLhaus online | 8 / 8 | 40 / 57 | 32 / 27 | 16 / 26 | 27,606 of 27,723 | Export rebuilt every few minutes (internal "Last updated" 1 to 11 min before fetch). 16–40 new/h |
| URLhaus recent (30 days) | 4 / 0 | 20 / 368 | 14 / 0 | 8 / 0 | 31,916 of 32,284 | The 368 removed are entries ageing out of the 30-day window at midnight. 8–20 new/h |
| ThreatFox recent | 6 / 0 | 14 / 0 | 15 / 2 | 19 / 0 | 802 of 804 (by URL) | 14–19 new/h |
| MalwareBazaar SHA-256 | 0 / 0 | 6 / 7 | 10 / 13 | 11 / 30 | 850 of 900 | Rewritten hourly (Last-Modified xx:05). 6–11 new hashes/h |

Readings:

- **Fastest open sources by update cadence:**
  - CERT Polska, rebuilt about every 5 min.
  - URLhaus, every few minutes.
  - OpenPhish community is slowest, at 12 h.
  - Phishing Army about 3 h, PhishTank 1 h.
- **This measures how often each list changes, not detection speed.** I had no ground truth for when each phish went live (the method does not visit URLs).
- **Volumes are small at the hourly scale:** tens of new phishing domains per hour from CERT Polska, about 20 malware URLs per hour from URLhaus. A TUF channel with a short refresh interval (5–15 min, with Rice-coded diffs) can carry this cheaply. Measured diffs are kilobytes per hour.
- The SEC-17 timestamp expiry of 3 days applies to the channel, but phishing freshness needs a much shorter refresh: Sunrise to Sunset reports campaigns lasting about 21 h. The refresh interval and maximum staleness need their own target.

### 4.4 Cross-feed lag (hosts first seen in feed A after sample 1)

For each host that appeared in feed A at sample k > 1, the table shows when feed B had it: earlier, same sample, later, or never within the 3.4 h window.

| A (new hosts) | in B earlier | same sample | B later | never (in window) |
|---|---|---|---|---|
| OpenPhish (141) → PhishTank | 8 | 0 | 0 | 133 |
| OpenPhish (141) → CERT Polska | 0 | 0 | 0 | 141 |
| OpenPhish (141) → Phishing Army | 9 | 0 | **119** | 13 |
| CERT Polska (72) → PhishTank | 2 | 0 | 4 | 66 |
| CERT Polska (72) → Phishing Army | 1 | 16 | 35 | 20 |
| PhishTank (7) → CERT Polska | **7** | 0 | 0 | 0 |
| PhishStats (12) → PhishTank | 11 | 0 | 0 | 1 |
| Phishing Army (909) → CERT Polska | 35 | 16 | 0 | 858 |

Phishunt is left out of the lag table. It entered at sample 2, so every one of its hosts counts as "new", which is an artefact of the method.

Readings:

- **Phishing Army follows its sources** by up to one rebuild, about 3 h. 119 of 141 new OpenPhish hosts appeared in it at the 01:02Z rebuild. It adds no detection of its own, and being one step further from the source only adds delay.
- **CERT Polska leads PhishTank** on Polish phishing: all 7 new PhishTank hosts were already in CERT Polska, while only 4 of CERT Polska's 72 new hosts reached PhishTank in the window.
- **OpenPhish's new entries are almost entirely absent** from PhishTank and CERT Polska. A global, non-Polish phishing source is therefore the main coverage gap of the open Tier 1, which supports Tier 2 (a licensed global feed).
- **Limit of the method:** a 3.4 h window with hourly sampling cannot measure lags longer than the window or shorter than one hour. "Never" here means "not within 3.4 h".

### 4.5 Overlap (normalised hostnames, union over all samples; Phishunt over samples 2–5)

| Pair | Shared hosts | Jaccard | Share of A in B | Share of B in A |
|---|---|---|---|---|
| CERT Polska / Phishing Army | 124,306 | 0.808 | 0.999 | 0.809 |
| PhishTank / Phishing Army | 27,014 | 0.172 | 0.886 | 0.176 |
| OpenPhish / Phishing Army | 267 | 0.002 | 0.978 | 0.002 |
| PhishTank / CERT Polska | 2,445 | 0.016 | 0.080 | 0.020 |
| PhishTank / Phishing.Database | 1,600 | 0.004 | 0.053 | 0.004 |
| OpenPhish / PhishTank | 6 | 0.0002 | 0.022 | 0.0002 |
| OpenPhish / CERT Polska | 5 | 0.00004 | 0.018 | 0.00004 |
| PhishStats / PhishTank | 27 | 0.001 | 0.730 | 0.001 |
| URLhaus / ThreatFox | 17 | 0.002 | 0.002 | 0.008 |
| URLhaus / any phishing feed | ≤11 | ≈0 | — | — |
| Phishunt / Phishing Army | 760 | 0.005 | 0.962 | 0.005 |
| Phishunt / PhishTank | 64 | 0.002 | 0.081 | 0.002 |
| Phishunt / CERT Polska | 17 | 0.0001 | 0.022 | 0.0001 |
| Phishunt / OpenPhish | 16 | 0.014 | 0.020 | 0.039 |

The sample-1 rows above barely change over the union of all five samples. Example: OpenPhish's share in Phishing Army goes from 0.978 to 0.954.

Hosts found in only one feed:

| Feed | Hosts only in this feed | Share |
|---|---|---|
| Phishing.Database | 390,415 of 392,071 | 99.6% |
| URLhaus | 8,466 of 8,500 | 99.6% |
| ThreatFox | 2,175 of 2,202 | 98.8% |
| PhishTank | 3,234 | 10.6% |
| Phishing Army | 4,478 | 2.9% |
| OpenPhish | 6 | 2.2% |
| CERT Polska | 175 | 0.1% |

Across all feeds, the union is 558,388 hosts. 408,951 of them appear in exactly 1 feed, 145,569 in 2, and 3,863 in 3.

Readings:

- **Phishing Army is mostly CERT Polska.** It holds 99.9% of CERT Polska's hosts, and they make up 81% of its list. It also contains 97.8% of OpenPhish and 88.6% of PhishTank. It is an aggregate, not an independent source, which matches its SOURCES.md.
- **The independent phishing sources barely overlap:** PhishTank vs CERT Polska share 8%, OpenPhish vs CERT Polska 1.8%. Combining sources adds real coverage, as Li et al. found for IP feeds (https://www.usenix.org/conference/usenixsecurity19/presentation/li).
- **Phishing and malware URL sources are disjoint.** URLhaus is mostly IP-literal hosts: 7,456 of 8,500 are IP literals. Separate tiers are needed.
- **Phishing.Database's 99.6% "unique" share** reflects stale, accumulated entries, not unique detection. Its NEW-today file has not changed since 2 October.

### 4.6 False-positive proxy (sample 1; Phishunt over samples 2–5)

Method: count listed hosts that **equal** a registrable domain in the Tranco top 100k (listing it blocks a whole popular site), and hosts under a top-10k registrable domain. This proxy flags candidates for review; it does not prove a false positive.

| Feed | Popular apex listed (top 100k) | Examples | Hosts under top-10k domains | IP-literal hosts |
|---|---|---|---|---|
| CERT Polska | 2 | cloudflare.vc (27,969), verificator.cc (48,509): both look like lookalikes, not real false positives | 184 | 0 |
| Phishing Army | 25 | vkontakte.ru (714), genspark.ai, sider.ai, hubs.ly, dub.sh, link.space | 2,863 | 0 |
| Phishing.Database | 74 | bit.ly (118), gravatar.com (156), name.com, **nflxvideo.net** (Netflix CDN), **mercadolibre.com.ar**, telegra.ph, getpocket.com, pastebin.com | 16,611 | 5,466 |
| PhishTank (URLs) | 165 | google.com, linkedin.com, googletagmanager.com, bing.com, tiktok.com, dropbox.com | 3,697 | 25 |
| URLhaus (URLs) | 13 | github.com, dropbox.com, gitlab.com, mediafire.com | 87 | 7,456 |
| OpenPhish | 0 | — | 5 | 0 |
| Phishunt | 2 | a2zapk.dev, roblox.com.bz (lookalikes) | 8 | 0 |

Consequences for the design:

- **URL feeds must be matched at URL level, never host level.** Examples are PhishTank's google.com and URLhaus's github.com entries. This is what the GSB-style host-suffix and path-prefix expressions give. A domain list (CERT Polska) can match by host.
- **Izcal's build step needs an allowlist** of popular and shared-hosting registrable domains. Any URL-feed entry on such a domain keeps its full path. Phishing Army's daily "whitelisted" commits are the practice to copy.
- **Phishing.Database's false-positive history is real:** 10 open false-positive issues filed 8–16 October, including `example.com` (https://github.com/Phishing-Database/Phishing.Database/issues). The listed apex domains above would block Netflix video, Gravatar and bit.ly.

---

## 5. What the privacy analysis must cover

1. **The list fetch.** TUF metadata and target downloads reveal client IP, timing, version and channel to the hosting CDN. State the retention and what the CDN logs. Consider serving through an OHTTP relay or a shared CDN with no per-user identifiers.
2. **The match is local.** The design target is a full local list for Tier 1, so no browsing data leaves the device. Record this as an invariant and test that no network request depends on a URL check.
3. **Optional prefix confirmation, if Tier 2's list is too large to ship in full.** Use v5-style 4-byte prefixes through a non-colluding OHTTP relay, with decoys. Address:
   - Gerbet et al.'s re-identification and multi-prefix tracking.
   - Who runs the relay and the gateway, and that they do not collude.
   - Rate of confirmation queries.
   - Caching.
   - The agent-task path, where an agent may visit far more URLs than a person.

   If the licensed feed's vendor requires lookups at its own servers, that is the SEC-D7 fallback trigger.
4. **Reports by the user** (false positive / "report phishing"). These are the only flow that sends a URL. It must be explicit, per event, and drawn by the prompt renderer (CON-R1).
5. **No telemetry on hits by default.** Contracts must not require it. Quad9 sends partners aggregate volume; record whether any vendor asks for this.
6. **Licence terms that touch privacy.** Attribution text on interstitials (URLAbuse CC BY needs credit; Phishunt bars the phrase "confirmed phishing") and anything a contract adds.
7. **B10.** These are lists, not a classifier, so B10's classifier carve-out is not used. State that.
8. **Agent tasks (B9).** The list is required for them, and the same local-match invariant applies.

---

## 6. Not verified or still pending

- **Measurement limits:** one 3.4 h window on one evening (a Sunday, UTC), with hourly sampling. Daily cycles, weekday intake and lags beyond 3.4 h were not measured. A week of 15-min sampling would settle cadence and lag; the scripts support it with `poll.sh <root> 672 900`.
- **UNVERIFIED:**
  - **Google:** a v4 shutdown date or one-year notice; whether Web Risk explicitly allows consumer-browser use.
  - **Vendors:** Netcraft pricing and terms; the APWG data-sharing agreement; Yandex terms; Spamhaus/abuse.ch commercial price; SURBL/Spamhaus bulk feed price.
  - **Browsers:** Orion's source; whether Vivaldi proxies.
  - **Feeds:** URLAbuse full-feed details (needs an account the owner must create); PhishTank redistribution rights (needs Talos in writing); PhishTank registration closure date; a PhishStats announcement about the CSV.
  - **Literature:** the detailed numbers in Gerbet et al. and in Bell & Komisarczuk.
- **Not assessed:** CERT and NCSC feeds of the Netherlands, UK, US (CISA AIS) and others.

Files:
- Feed samples (214 MB, untrusted data: lists of live malicious URLs; never open them in a browser): kept outside the repository on revire at `~/izcal-measure/sec-d7/sec-d7-feeds.tgz`, sha256 `79ad318b9bba80c5d7fdec24f3203a5bba9c8490a0f706acc030722511d86072`. Several feeds' terms forbid redistribution, so they are not committed.
- Scripts: `scripts/` in this folder (`fetch_once.sh`, `poll.sh`, `analyse.py`; final results in `report.json`, sample-1-only results in `report_interim.json`)
- Reference data: in the same archive (`ref/`).
