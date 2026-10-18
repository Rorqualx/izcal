# Web usability and annoyance pain points: claim fact-check and browser-level mitigations (as of October 14, 2026)

## Claim 1: "62% experience recurring digital burnout", "one in five manage 11+ tabs", "nearly half say the browser helps and distracts equally"

### Takeaway
**Verdict: mostly correct, but the source is a vendor survey.** All three figures come from the **Shift "2026 State of Browsing Report"**. Shift is a desktop browser vendor (a modular, multi-account browser), so it has a commercial interest in the findings. The report surveyed 1,000 U.S. adults in September 2025, reportedly fielded by MX8 Labs, and was released in November 2025. The word "recurring" in the claim overstates the finding: the report says 62% experience burnout "occasionally or regularly".

### Cited Findings
- 62% of consumers report digital burnout, occasionally or regularly. 31% rarely or never intentionally unplug. 81% are "ready to switch" (one version says "considering switching") browsers for better personalization. 43% are distracted at work multiple times a day. 20% name switching between web apps and pages as a top productivity killer. — [Agile Brand Guide research summary, "2026 State of Browsing Report" (Shift)](https://research.agilebrandguide.com/research/the-2026-state-of-browsing-report); [Agile Brand Guide press release](https://agilebrandguide.com/consumers-declare-the-browser-broken-62-report-digital-burnout-81-ready-to-switch-2/)
- Sample: a survey of 1,000 U.S. adults conducted in September 2025, with data weighted to be nationally representative. — [Adweek Wire (Shift press release)](https://www.adweek.com/adweek-wire/the-legacy-browser-is-at-a-breaking-point-according-to-the-state-of-browsing-report/); [Adweek Wire, Shift digital burnout release](https://www.adweek.com/adweek-wire/new-report-from-shift-reveals-rising-levels-of-digital-burnout-across-generations/)
- "1 in 5 users have 11+ open tabs." The quote is from Shift's CEO. — [Adweek Wire](https://www.adweek.com/adweek-wire/the-legacy-browser-is-at-a-breaking-point-according-to-the-state-of-browsing-report/); [Techcouver, Nov 19, 2025](https://techcouver.com/2025/11/19/distractions-shift-reimagines-browser-tool-productivity/)
- "47% of users say their browser distracts them just as often as it helps them focus." StudyFinds names MX8 Labs as the survey panel. — [StudyFinds](https://studyfinds.com/digital-burnout-is-now-the-default-state-of-being-online/); [StudyFinds (second piece)](https://studyfinds.com/americans-are-hemorrhaging-time-online-distractions/)
- Coverage lists these drivers of browser fatigue: endless notifications (24%), social media overload (23%), news "rabbit holes" (18%), and switching apps and tabs (13%). — [Benefits and Pensions Monitor](https://www.benefitsandpensionsmonitor.com/news/industry-news/when-every-click-counts-digital-burnout-quietly-drains-performance/392906); [CXM World](https://cxm.world/digital-experience/digital-burnout-is-becoming-a-daily-experience-for-always-on-customers/)

### Inferences
- The sponsor is **Shift**, not Opera, Mozilla, Vivaldi or Microsoft. Shift sells a "fix" for the problem it measured, so the figures should be cited as vendor-commissioned.
- With n=1,000, the margin of error is roughly ±3 percentage points.

### Gaps
- I did not find the full questionnaire or topline PDF. The exact question wording, including how "burnout" was defined, is unverified.
- I found no evidence that heavy tab users report more burnout. The tab and burnout figures are reported separately.

## Claim 2: Scrolljacking reduces satisfaction and harms users with vestibular disorders

### Takeaway
**Verdict: partly correct.** NN/g's "Scrolljacking 101" (Sara Paul, August 6, 2023) documents disorientation, loss of control, lower efficiency and lower task success. It does not measure "satisfaction" as a metric, and it does not mention vestibular disorders. The vestibular harm comes from a separate source: W3C WCAG motion guidance plus epidemiology on how common vestibular dysfunction is. No source I found tests scrolljacking on vestibular users directly.

### Cited Findings
- From the NN/g usability study across industries: "The majority of our study participants were at least mildly disoriented by scrolljacking." Disorientation was "severely exacerbated when scrolljacks were excessively lengthy." Some users thought the page was broken, refreshed it or left. Scrolljacking was worse on mobile. NN/g found threats to "user control and freedom, discoverability, attention, efficiency, and task success." It recommends avoiding scrolljacking on mobile and never changing scroll direction. NN/g did not report a participant count. — [NN/g, Scrolljacking 101 (2023-08-06)](https://www.nngroup.com/articles/scrolljacking-101/)
- NN/g separates scroll fading (an animation that does not change scroll rate) from scrolljacking (overriding the OS scroll rate or direction). — [NN/g, Scroll Fading 101 (Dec 2023)](https://www.nngroup.com/articles/scroll-fading-101/)
- 35.4% of U.S. adults aged 40 and over (about 69 million people) had vestibular dysfunction in NHANES data from 2001–2004. Prevalence rose from 18.5% in people in their 40s to 85% at age 80 and over. — [Agrawal et al., Arch Intern Med 2009, PubMed 19468085](https://pubmed.ncbi.nlm.nih.gov/19468085)

### Inferences
- Accurate wording: "NN/g usability testing found scrolljacking disorients most users and harms efficiency and task success." The vestibular risk is a separate WCAG and accessibility argument, not an NN/g finding.

### Gaps
- I found no controlled study measuring scrolljacking's effect on people with vestibular disorders, and none quantifying satisfaction (SUS or similar).
- I did not fetch WCAG 2.3.3 "Animation from Interactions" or `prefers-reduced-motion` adoption data in this session.

## Claim 3: "Up to 90% of people find autoplay video with sound irritating"

### Takeaway
**Verdict: wrong as stated, or unverifiable.** I found no source for 90%. Published surveys from 2018–2019 put it at roughly **49–71%**. The most-cited figure is **66.7%** (eMarketer/Bizrate Insights, 2019).

### Cited Findings
- 66.7% of U.S. digital buyers rated "video ads that play automatically on websites with sound" as annoying, the highest of any ad type. The panel skewed older, female and affluent and was not representative. — [eMarketer, "Which Types of Ads Annoy Consumers Most?"](https://www.emarketer.com/content/why-consumers-avoid-ads/)
- 71.2% of 1,008 U.S. users found auto-play videos annoying (December 2018). This question was about autoplay in general, not specifically with sound. — [DEJAN Marketing via Search Engine Journal](https://www.searchenginejournal.com/study-finds-which-types-of-digital-ads-are-most-annoying-for-consumers/323285/)
- Statista Consumer Insights (5,000+ U.S. respondents): 49% find autoplay video ads with sound annoying, the most-disliked type. — [Statista chart 31659](https://www.statista.com/chart/31659/the-most-annoying-types-of-online-advertising/)

### Inferences
- A defensible replacement is "about two-thirds (49–71% across surveys)". The "90%" figure looks like an exaggeration.

### Gaps
- I found no 2024–2026 survey on this.

## Claim 4: "Sticky headers, cookie bars and chat bubbles can consume 30–40% of a mobile viewport"

### Takeaway
**Verdict: unverifiable.** I found no empirical study measuring how much of the viewport these elements cover together. Only vendor blogs give unsourced estimates: "30% or more" for a sticky header when zoomed in, and "a third" for two strips. The claim is plausible but should be labeled an estimate, or measured directly.

### Cited Findings
- "On smaller screens, sticky headers can consume 30% or more of the viewport when users zoom in." No data is cited. — [Slider Revolution blog](https://www.sliderrevolution.com/design/sticky-header/)
- Two stacked strips can take a third of a small viewport. This is a blog estimate. — [21st.dev](https://21st.dev/blog/react-header-banner-components)
- WCAG 2.2 SC 2.4.11 (Focus Not Obscured, Minimum) targets sticky overlays that hide the focused element. — [WCAG 2.4.11 explainer](https://www.disabilityworld.org/toolkit/standards/wcag/2-4-11-focus-not-obscured-minimum/)

### Gaps
- No primary measurement exists. One could be produced with a crawl, for example of HTTP Archive top sites at 390×844, measuring the area of fixed and sticky elements.

## Claim 5: Forced login walls and "Open in App" interstitials drive abandonment

### Takeaway
**Verdict: correct for forced account creation, and partly correct for app interstitials.** Baymard's 2025 survey finds that **19%** of U.S. online shoppers (one page version says 18%) abandoned an order because the site wanted them to create an account. Google's guidance discourages promotional interstitials, explicitly including app-install prompts, but it frames the effect as possible "poor search performance". It gives no abandonment figure.

### Cited Findings
- 2025 Baymard checkout benchmark: survey of 1,026 U.S. adults who had shopped online in the past 3 months. "Site wanted me to create an account" was given by 19% (18% in one page version). Other top reasons were extra costs (39%), slow delivery (21%) and trust (19%). — [Baymard, Current State of Checkout UX](https://baymard.com/blog/current-state-of-checkout-ux)
- One third-party figure of 26% (n=4,560) conflicts with this and is not on Baymard's page. It is likely an older Baymard wave and should not be cited as 2025. — [LaunchMyStore](https://launchmystore.io/blog/checkout-optimization-frictionless-ecommerce)
- Google Search Central: "Don't obscure the entire page with interstitials." "This applies to all kinds of promotional dialogs, including app install prompts." Intrusive dialogs "may lead to poor search performance." Mandatory interstitials such as age gates are exempt. The page was last updated 2025-12-10. — [Google Search Central, Avoid intrusive interstitials and dialogs](https://developers.google.com/search/docs/appearance/avoid-intrusive-interstitials)

### Gaps
- I found no recent public dataset quantifying abandonment caused by "Open in App" interstitials.

## Claim 6: "A CAPTCHA takes an average of 32 seconds" and "endless CAPTCHA loops on YouTube"

### Takeaway
**Verdict on 32 s: partly correct, but misattributed.** The figure comes from **Cloudflare's own data (blog, May 13, 2021)**, not from a Stanford study. The Stanford work (Bursztein et al., IEEE S&P 2010) measured **9.8 s for distorted text and 28.4 s for audio**. The USENIX Security 2023 study (Searles et al., UC Irvine; 1,400 participants, 14,000 CAPTCHAs) found averages from **3.6 to 42.7 s** depending on type. Click-based reCAPTCHA had the lowest median, **3.7 s**. Average solve time also rose **26.7%** when CAPTCHAs were embedded in a realistic task.

**Verdict on YouTube loops: unverifiable.** Anecdotal reports exist of YouTube's "Sign in to confirm you're not a bot" wall, mostly affecting VPN users, third-party front-ends and embeds. I found no official acknowledgment of an "endless loop".

### Cited Findings
- "Based on our data, it takes a user on average 32 seconds to complete a CAPTCHA challenge." The same post estimates "500 human years wasted every single day". Published 2021-05-13. — [Cloudflare blog, Humanity wastes about 500 years per day on CAPTCHAs](https://blog.cloudflare.com/introducing-cryptographic-attestation-of-personhood/)
- Searles et al. 2023: click-based reCAPTCHA "had the lowest median solving time at 3.7 seconds." Distorted text took 12.5 s on average. Overall averages ranged from 3.6 to 42.7 s per CAPTCHA. Bursztein et al. 2010 measured 9.8 s (text) and 28.4 s (audio). Contextualized solving was 26.7% slower than direct. Bots outperformed humans on both time and accuracy. Human image-classification accuracy was 81% on reCAPTCHA and 70.6–81.4% on hCaptcha. Solving time increased 0.09 s per year of participant age. — [USENIX Security 2023, Searles et al.](https://www.usenix.org/conference/usenixsecurity23/presentation/searles); [arXiv PDF](https://arxiv.org/pdf/2307.12108)
- YouTube bot wall: an Oracle help page says "Google policies now require this confirmation" for embedded playback. Users report the only workaround is rotating IPs. — [Oracle Help Center FAQ](https://docs.oracle.com/en/cloud/saas/readiness/common/readiness-questions-answers/why-do-i-get-the-error-sign-in-to-confirm-you-re-not-a-bot-when-i-play-readiness-demos-or-help-videos-from-oracle-help-center.html); [Privacy Guides forum](https://discuss.privacyguides.net/t/sign-in-to-confirm-you-re-not-a-bot-youtube-requiring-sign-in-work-around/23860)

### Gaps
- I found no official YouTube or Google statement on CAPTCHA or bot-check loops.

## Claim 7: Chrome rate limits of 1,000 messages/minute for abusive sites, and "cut over 7 billion unwanted Android notifications per day"

### Takeaway
**Verdict: correct.** Google's Chrome Security blog (August 11, 2026) says abusive domains are throttled to **1,000 messages per minute** via Firebase Cloud Messaging. Domains over the limit get HTTP 429 responses, and repeat offenders face stricter limits. The same post reports that notifications on Android were reduced "by over 7 billion a day in Q1 alone" (Q1 2026). The 7 billion is a reduction in notifications sent, not a count of blocked malicious notifications, though some press described it that way.

### Cited Findings
- By Hannah Buonomo (Chrome Security), Jonathan Li (Safe Browsing) and Nidhi Davawala (FCM): Chrome automatically revokes notification permission from sites the user hasn't engaged with recently and from sites with repeated suspicious-notification warnings. Users can regrant in Safety Hub. Behavioral detection looks at service-worker activity. Disruptive domains are "limited to 1,000 messages per minute" and get HTTP 429 when over. The effort was "reducing notifications on Android by over 7 billion a day in Q1 alone." — [Google blog, The multi-layered defenses that harden Chrome against abusive notifications (2026-08-11)](https://blog.google/security/the-multi-layered-defenses-that-harden-chrome-against-abusive-notifications/)
- Secondary coverage — [TechRepublic](https://www.techrepublic.com/article/news-chrome-7-billion-unwanted-android-notifications/); [Help Net Security](https://www.helpnetsecurity.com/?p=381452)

### Gaps
- Google gives no baseline, so the 7 billion cannot be expressed as a percentage reduction.

## Mitigation 1: Autoplay policies

### Takeaway
Since 2017–2019, every major browser blocks audible autoplay by default. Only Chrome has published a measured effect, from 2018: it blocks "about half" of unwanted autoplays.

### Cited Findings
- Chrome 66 (2018) uses the Media Engagement Index (MEI) to allow audible autoplay only on sites where the user often plays media, and it learns preferences within roughly 6–20 visits. Sites with no history fall back to a pre-seeded list. A Google PM said in May 2018 that the policy blocks about half of unwanted autoplays. — [Google blog, Improving Autoplay in Chrome](https://blog.google/products/chrome/improving-autoplay-chrome/); [GamesBeat](https://gamesbeat.com/google-says-chrome-blocks-about-half-of-unwanted-autoplays/); [Chromium autoplay pre-seeding](https://www.chromium.org/audio-video/autoplay/autoplay-pre-seeding-in-chrome)
- Safari 11 (2017) blocks auto-playing media with sound. — [WebKit blog, Auto-Play Policy Changes for macOS](https://webkit.org/blog/7734/auto-play-policy-changes-for-macos/) (not re-fetched this session)
- Firefox 66 (March 2019) blocks audible autoplay by default. — [Mozilla Hacks](https://hacks.mozilla.org/2019/02/firefox-66-to-block-automatically-playing-audible-video-and-audio/) (not re-fetched this session)

### Gaps
- I found no post-2018 measurement from any vendor, and no data from Firefox or Safari on blocked autoplay volume.

## Mitigation 2: Quieting notification permission prompts

### Takeaway
This is the best-evidenced mitigation. Requiring a user gesture (Firefox 72) and Chrome's quiet UI both cut prompt noise substantially without killing legitimate grants.

### Cited Findings
- Firefox 63 baseline (one month): 1.45 billion prompts were shown and only 23.66 million accepted. About 99% went unaccepted and 48% were actively denied. After requiring user interaction, first-time allow rates were 52% (Nightly), 24% for existing users and 56% for new users (Release). The gesture requirement shipped in Firefox 72 (January 2020). Prompts without a gesture now show only a URL-bar icon. — [Mozilla Future Releases blog (2019-11-04)](https://blog.mozilla.org/futurereleases/2019/11/04/restricting-notification-permission-prompts-in-firefox/); [Mozilla Hacks, Firefox 72 changes](https://hacks.mozilla.org/2019/11/upcoming-notification-permission-changes-in-firefox-72/)
- Chrome quiet UI (Chrome 80, February 2020): an A/B test on more than 40 million users showed "up to 30% fewer unnecessary actions on the prompts, with less than 5% impact on grant rates." Baseline notification grant rates were 10% on desktop and 21% on Android (the earlier paper version says 12% and 23%). Notifications made up 74% of all permission prompts. — [Bilogrevic et al., USENIX Security 2021](https://www.usenix.org/conference/usenixsecurity21/presentation/bilogrevic); [Google Research](https://research.google/pubs/pub49767/)
- An ML-based, context-aware activation of quiet prompts was studied with 13.1k participants. — [NDSS 2024 paper](https://www.ndss-symposium.org/wp-content/uploads/2024-108-paper.pdf)
- CrUX publishes per-origin notification Accept, Block, Ignore and Dismiss rates (since the 202001 dataset). — [Chrome Developers blog](https://developer.chrome.com/blog/notification-permission-data-in-crux/)
- Automatic revocation, Safety Hub, and the 2026 FCM rate limits are covered under Claim 7. — [Google blog 2026-08-11](https://blog.google/security/the-multi-layered-defenses-that-harden-chrome-against-abusive-notifications/)

## Mitigation 3: Reader modes, interstitial and sticky blockers, scroll overrides, reduced motion

### Takeaway
I found no measured outcome data in this session. These mitigations exist (Safari Reader, Firefox Reader View, Chrome Reading Mode, the `prefers-reduced-motion` OS setting), but no vendor or academic outcome metrics turned up.

### Cited Findings
- The only directly relevant evidence is NN/g's finding that sticky navigation helped users escape confusing scrolljacks. That suggests a "restore native scrolling" override would address the core harm. — [NN/g Scrolljacking 101](https://www.nngroup.com/articles/scrolljacking-101/)
- Google's legal-interstitial exemption means cookie consent dialogs are not penalized in search, so blocking them falls to browsers and extensions. — [Search Engine Journal](https://www.searchenginejournal.com/google-intrusive-interstitials/)

### Gaps
- I found no measured data on reader-mode usage or outcomes, on sticky or cookie-banner blockers (for example Brave's or DuckDuckGo's cookie-consent auto-handling), or on how often browsers enforce reduced motion.

## Mitigation 4: Tab management (groups, vertical tabs, workspaces, discarding, auto-archive)

### Takeaway
The only measured outcomes are resource savings, all self-reported by vendors. I found none for productivity or burnout.

### Cited Findings
- Chrome Memory Saver (December 2022) can reduce memory use by "up to 40%" according to Google's launch claim. An independent test by a competing vendor (32 tabs, 20 idle minutes) found about 21% savings in Balanced mode and about 38% in Maximum mode. — [AlternativeTo (launch coverage)](https://alternativeto.net/news/2022/12/google-chrome-s-new-memory-saver-and-energy-saver-modes-seek-to-optimize-hardware-resource-use); [SuperchargeBrowser test (vendor, treat cautiously)](https://www.superchargebrowser.com/library/tab-suspender-vs-chrome-memory-saver.md)
- Edge Sleeping Tabs (Microsoft data, 2022): about 39.1 MB saved per sleeping tab, 6 billion tabs slept in 28 days, 273.7 PB of RAM saved, and total memory down 16% on average. Tabs sleep after 2 hours by default. — [TechSpot](https://www.techspot.com/news/94872-microsoft-edge-browser-has-saved-users-273-petabytes.html); [Windows Central](https://windowscentral.com/software-apps/microsoft-edge-put-6-billion-tabs-to-sleep-over-the-last-month)

### Gaps
- I found no published measured outcomes (focus, task time, tab counts) for tab groups, vertical tabs, Vivaldi workspaces, Firefox tab groups or vertical tabs, or Arc's auto-archive. Only feature announcements exist. The Shift survey (Claim 1) is the only tab-overload datapoint.

## Mitigation 5: CAPTCHA alternatives (Privacy Pass / Private Access Tokens / Turnstile)

### Takeaway
Cloudflare reports that its non-interactive challenges cut visitor time from **about 32 s to about 1 s**, and that Private Access Tokens can eliminate "nearly 100%" of CAPTCHAs for supported Apple devices. Both are vendor-reported and not independently measured.

### Cited Findings
- Private Access Tokens (announced June 8, 2022) extend Privacy Pass. Apple attests the device and Cloudflare issues the token. They require iOS/iPadOS 16 or macOS 13 and later. Cloudflare says they "will eliminate nearly 100% of CAPTCHAs served to these users." Its Managed Challenge had already cut its own CAPTCHA use by 91%. — [Cloudflare blog, Eliminating CAPTCHAs on iPhones and Macs](https://blog.cloudflare.com/eliminating-captchas-on-iphones-and-macs-using-new-standard/)
- Turnstile launch (September 28, 2022): Cloudflare reduced the time visitors spend in a challenge from an average of 32 seconds to about 1 second for non-interactive challenges. Turnstile also uses Private Access Tokens as a signal. — [Cloudflare press release](https://www.cloudflare.com/press-releases/2022/privacy-preserving-captcha-alternative)
- Cryptographic Attestation of Personhood (WebAuthn security key, 2021): "Completing this flow takes five seconds," per Cloudflare. — [Cloudflare blog 2021](https://blog.cloudflare.com/introducing-cryptographic-attestation-of-personhood/)

### Inferences
- Benchmark against the 3.7 s median for click-based reCAPTCHA (Searles 2023), not only against Cloudflare's 32 s baseline. Turnstile's real gain over modern checkbox CAPTCHAs is a few seconds per solve, plus no image tasks.

### Gaps
- I found no independent measurement of Turnstile or PAT completion time, or of false-challenge rates on non-Apple and privacy-hardened browsers.
