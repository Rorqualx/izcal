# AI Bloat in Browsers and Browser-Choice / OS Lock-in (fact-check as of 14 Oct 2026)

## Claim 1: "Chrome has been quietly downloading a 4GB Gemini Nano model onto user devices without clear opt-in consent"

### Takeaway
**Verdict: mostly correct, with caveats.** Chrome does download Gemini Nano (about 4GB, in the `OptGuideOnDeviceModel` folder) in the background on eligible hardware, with no prompt. Researcher Alexander Hanff documented this on 4 May 2026, and mainstream press repeated it. Two caveats: (1) Google's developer docs give no fixed size ("may vary"), and the download only happens on devices that pass a hardware gate with at least 22GB free; (2) Google says that since **February 2026** a Settings > System > "On-device AI" toggle turns the model off and removes it. Users do have an opt-out. There is no opt-in. No formal regulator complaint or ruling was found.

### Cited Findings
- Primary source: Alexander Hanff ("That Privacy Guy"), 4 May 2026. On a fresh Apple Silicon macOS profile with no user input, Chrome downloaded `OptGuideOnDeviceModel/2025.8.8.1141/weights.bin` (about 4GB) in 14 min 28 s, with no prompt or notification. Deleted files were downloaded again. He reports the eligibility gate as about 16GB RAM/unified memory plus a capable GPU, with flags `OnDeviceModelBackgroundDownload` and `ShowOnDeviceAiSettings` active. He argues breaches of ePrivacy Art. 5(3), GDPR Arts. 5(1) and 25, PECR and CCPA, but **does not say any complaint was filed** — [That Privacy Guy](https://www.thatprivacyguy.com/blog/chrome-silent-nano-install/)
- Secondary coverage (5–8 May 2026) says the same behaviour was confirmed on Windows 11, Apple Silicon and Ubuntu — [gHacks, 6 May 2026](https://www.ghacks.net/2026/05/06/google-chrome-is-silently-downloading-a-4gb-gemini-nano-ai-model-to-user-devices-without-consent/); [Quartz](https://qz.com/google-chrome-gemini-nano-silent-download-050826); [TechRadar](https://www.techradar.com/pro/the-climate-costs-are-insane-why-chrome-users-are-outraged-over-a-forced-4gb-gemini-ai-update-that-may-affect-billions-worldwide-without-their-consent-and-it-even-redownloads-automatically-when-deleted); [TweakTown](https://www.tweaktown.com/news/111514/google-chrome-silently-downloads-a-4gb-ai-model-on-your-pc-without-asking/index.html)
- Google's statement, as reported: "In February, we began rolling out the ability for users to easily turn off and remove the model directly in Chrome settings… Once disabled, the model will no longer download or update." Google says the model powers on-device security tasks such as scam detection — [Yahoo Tech/Quartz syndication](https://tech.yahoo.com/ai/gemini/articles/google-chrome-silently-downloads-4gb-143602025.html)
- The Register, 9 May 2026: Google changed the wording of Chrome's "On-device AI" settings text and said "This doesn't reflect a change to how we handle on-device AI for Chrome… The data that is passed to the model is processed solely on device." Google says the model powers scam detection and developer APIs — [The Register](https://www.theregister.com/a/5237580)
- Android Authority reports the Settings > System > On-device AI toggle was **missing for many users** despite Google's claim — [Android Authority](https://androidauthority.com/how-to-disable-chrome-gemini-nano-delete-weights-bin-3665109)
- Official developer documentation sets these requirements for the built-in AI APIs (Prompt, Summarizer, Writer, Rewriter, etc.): **at least 22GB free** on the volume holding the Chrome profile; GPU with **strictly more than 4GB VRAM**, or CPU with 16GB+ RAM and 4+ cores; an unmetered connection for the download. If free space falls **below 10GB after the download, the model is removed** and is downloaded again once the requirements are met. It also says "Gemini Nano's exact size may vary as the browser updates the model"; check `chrome://on-device-internals`. Supported on Windows 10/11, macOS 13+, Linux, and ChromeOS on Chromebook Plus. Not supported on Android or iOS — [Chrome for Developers: Get started with built-in AI](https://developer.chrome.com/docs/ai/get-started)
- The same docs say that for web APIs, the download is triggered the first time a page calls the API (`availability()` = "downloadable" → `create()` after user activation) — [Chrome for Developers](https://developer.chrome.com/docs/ai/get-started). Hanff's test shows that browser features also trigger a background download without any page calling these APIs.
- Enterprise control: the `GenAILocalFoundationalModelSettings` policy (DWORD at HKLM\SOFTWARE\Policies\Google\Chrome). 0 = allow; 1 = do not download, and remove any existing model. It supports dynamic refresh. One report says Chrome 147+ downloads by default when the policy is unset (not confirmed by Google) — [Neowin](https://www.neowin.net/news/official-windows-11-registry-mod-blocks-automatic-download-of-4gb-ai-model-on-google-chrome/); [Overclock3D](https://overclock3d.net/reviews/software/how-to-block-googles-chrome-ai-model-with-an-easy-windows-11-registry-mod/); a DISA STIG finding also references the policy — [STIG Viewer V-275782](https://www.stigviewer.com/stigs/google_chrome_current_windows/2025-05-15/finding/V-275782)
- Microsoft Edge has a policy with the same name, supported from Edge 132 (Windows/macOS) and Edge 147 (Android). This suggests Edge also downloads a local model (Phi-4-mini) — [Microsoft Learn: Edge policy](https://learn.microsoft.com/en-my/deployEdge/microsoft-edge-policies/genailocalfoundationalmodelsettings)

### Inferences
- "Quietly / without opt-in consent" holds: there is no prompt, and the download is on by default for eligible hardware. "Without any way to refuse" would be wrong: there is a settings toggle (rolling out since Feb 2026, unevenly), a flag, and an enterprise policy.
- "4GB" matches the on-disk weights Hanff measured. Google's docs give no number, and the size changes between model versions. Better wording: "about 4GB, variable."
- It applies only to desktop machines with at least 22GB free and a capable GPU/CPU. Low-end PCs and phones do not get it, so "billions of devices" claims are overstated.

### Gaps
- I could not fetch Google's official Chrome Enterprise policy page entry for `GenAILocalFoundationalModelSettings` (chromeenterprise.google/policies). The policy semantics above come from secondary sources plus the Edge equivalent.
- No Chrome release-notes entry gives the exact milestone for the "On-device AI" toggle reaching Stable.
- I found no formal DPA, CNIL or noyb complaint or regulator action about the download as of Oct 2026.

## Claim 2: "Users filed formal complaints about forced 'Do more with AI' and 'Create images' shortcuts on the New Tab Page"

### Takeaway
**Verdict: partly correct, and about Chrome, not Edge.** The shortcuts are real Chrome New Tab Page AI action chips under the search box ("Do more with AI [Mode]"/AI Mode, "Create images"). They were first tested in Canary in mid-2026 and reached stable users by Sept 2026. The complaints I found are **informal**: a Reddit r/chrome post, how-to sites and blog comments. I found **no regulator complaint, formal filing, or Chromium bug** about them, so "formal complaints" is unverified or wrong.

### Cited Findings
- Windows Report saw "Do more with AI Mode" ("Create, plan, summarize") chips intermittently in Chrome Canary; Google gave no commitment to ship them to Stable — [Windows Report](https://windowsreport.com/chrome-tests-new-ai-mode-suggestion-chips-on-the-new-tab-page/); [Windows Report: AI action chips](https://windowsreport.com/chromes-next-new-tab-page-could-bring-ai-action-chips-not-just-a-new-search-box/); [Windows Report: Chrome pushing AI Mode prompts on NTP](https://windowsreport.com/chrome-is-pushing-google-ai-mode-with-new-prompts-on-the-new-tab-page/)
- Piunikaweb, 28 Sep 2026: after updating to the latest Chrome, users saw "AI Mode"/"Ask AI" and "Create Image" (Nano Banana) buttons on the NTP. The complaint source is a Reddit r/chrome post (screenshot only). Workaround: chrome://flags → "NTP compose box" → Disabled. No regulator, bug-tracker or formal complaint is mentioned — [Piunikaweb](https://piunikaweb.com/2026/09/28/how-to-remove-chrome-ai-mode-create-image-buttons/)
- AskVG how-to (about Sept 2026) on removing the "AI action chips" from Chrome's NTP; commenters report mixed success with flags (NTP Realbox Next / Composebox) — [AskVG](https://www.askvg.com/disable-or-remove-ai-action-chips-from-new-tab-page-in-google-chrome/); [DroidWin](https://droidwin.com/how-to-remove-do-more-with-ai-mode-from-chrome-home-page/); [Vishal Gupta on X](https://x.com/VishalGuptaMVP/status/2103766200711028803)
- Edge has a similar but separate issue: in March 2026 Edge's default NTP opened Copilot (chat history plus compose box) — [Windows Latest, 15 Mar 2026](https://www.windowslatest.com/2026/03/15/microsoft-edge-feels-more-like-copilot-than-a-browser-now-but-some-ai-features-can-be-turned-off/)

### Inferences
- Better wording: "Chrome users complained on Reddit and tech sites about non-removable 'AI Mode' and 'Create images' chips on the New Tab Page (Sept 2026); the only off-switch is an experimental flag."

### Gaps
- I did not retrieve the original Reddit thread URL. I found no Chromium issue tracker entry, and did not check issues.chromium.org directly.
- No official Google announcement or support doc for the chips was found.

## Claim 3: Other AI-in-browser opt-in/opt-out controversies, 2025–2026

### Takeaway
Browsers fall into four groups. **Default-on with buried or partial opt-out:** Chrome, Edge. **Default-on features plus a global kill switch:** Firefox 148+. **Explicit opt-in:** Brave Leo, DuckDuckGo (optional, can be turned off). **No LLM features:** Vivaldi.

### Cited Findings
- **Firefox:** User backlash in Dec 2025 after new CEO Anthony Enzor-DeMeo's AI plans led Mozilla to promise a "kill switch." Critics said opt-out is still not opt-in — [Futurism](https://futurism.com/artificial-intelligence/outcry-firefox-promises-kill-switch-ai-features); [Cybernews](https://cybernews.com/ai-news/mozilla-firefox-ai-kill-switch/); [Vice](https://www.vice.com/en/article/firefox-will-let-users-shut-off-ai-with-a-kill-switch/)
- **Firefox 148 (24 Feb 2026)** added "AI controls" in desktop settings. A master toggle, "Block AI enhancements", blocks current and future generative AI features and their pop-ups or reminders. Individual controls cover translations, PDF alt text, AI tab grouping, link previews and the sidebar chatbot. Choices persist across updates — [Mozilla blog: AI controls are coming to Firefox](https://blog.mozilla.org/en/firefox/ai-controls/); [Malwarebytes, Feb 2026](https://www.malwarebytes.com/blog/news/2026/02/firefox-is-giving-users-the-ai-off-switch)
- Vice claims Firefox AI features are off by default on fresh install. This is **unconfirmed** by Mozilla documentation, and features such as local translation are on by default — [Vice](https://www.vice.com/en/article/firefox-will-let-users-shut-off-ai-with-a-kill-switch/)
- **Edge:** Copilot Mode launched in Jul 2025 as an opt-in experiment — [The Register, 28 Jul 2025](https://www.theregister.com/2025/07/28/microsoft_edge_copilot_mode/). Edge 144 (16 Jan 2026) allowed hiding the Copilot toolbar icon after backlash — [WinBuzzer](https://winbuzzer.com/2026/01/19/new-microsoft-edge-144-allows-copilot-removal-after-sustained-user-backlash-xcxwbn/). On 13 May 2026 Microsoft retired Copilot Mode as a separate opt-in mode and **folded its features into default Edge** (multi-tab reasoning, Journeys, voice/vision, tab-to-podcast) — [gHacks, 15 May 2026](https://www.ghacks.net/2026/05/15/microsoft-edge-retires-copilot-mode-and-integrates-ai-features-across-desktop-and-mobile/); [Windows Central](https://www.windowscentral.com/software-apps/microsoft-edge-ditching-copilot-mode-means-more-ai-in-your-browser-not-less)
- **Brave Leo:** Brave's help center says Leo is "not active or accessible and does not work until you confirm" the opt-in on first use. Consent can be withdrawn via Settings → Leo / Clear Leo data. Forum users still report Leo UI reappearing and the `BraveAIChatEnabled` policy not working (anecdotal) — [Brave Help: How do I disable Brave Leo?](https://support.brave.app/hc/articles/27331617852045-How-do-I-disable-Brave-Leo); [Brave Community](https://community.brave.com/t/leo-being-forced-again-main-menu/564824)
- **DuckDuckGo:** "All our AI features are optional and can be turned off or tuned down." Duck.ai can be hidden, and Search Assist can be set to "Never" — [DuckDuckGo Help](https://duckduckgo.com/duckduckgo-help-pages/ai-features/opting-out-of-ai)
- **Vivaldi:** "We will not use an LLM to add a chatbot, a summarization solution or a suggestion engine to fill up forms for you until more rigorous ways to do those things are available." Vivaldi still allows users to visit any online LLM. The stance was reiterated in 2025 coverage — [Vivaldi blog](https://vivaldi.com/blog/technology/vivaldi-wont-allow-a-machine-to-lie-to-you/); [It's FOSS](https://itsfoss.com/news/vivaldi-stance-on-ai/); [OSnews](https://www.osnews.com/story/140107/vivaldi-takes-firm-stance-against-ai-will-not-include-it-in-its-browser/)

### Inferences
- The industry trend in 2026 is toward AI on by default (Chrome NTP chips plus local model, Edge Copilot folded in). The counter-trend is opt-in or kill switches as a selling point (Brave, DDG, Firefox 148, Vivaldi).

### Gaps
- **Opera** (Aria, Opera Neon agentic browser): no sources retrieved on defaults or opt-out controversies.
- I found no 2026 confirmation that Vivaldi's stance is unchanged.

## Claim 4: Windows 11 browser-choice survey (72% / 29% / 44% vs 15%)

### Takeaway
**Verdict: correct as quoted, but it is a vendor-coalition advocacy survey.** The figures come from a **Savanta** survey of **2,000 U.S. adult Windows desktop or laptop users**, commissioned by the **Browser Choice Alliance** (Google Chrome, Opera, Vivaldi, Waterfox, Wavebox) and published **12 Aug 2026**. No fieldwork dates, margin of error, weighting or question wording were published. It covers U.S. users only, so it does not measure the EEA, where Microsoft's DMA changes apply.

### Cited Findings
- BCA release, 12 Aug 2026. 72% of those who tried to change their default browser in the past 1–2 years reported challenges. 29% said Windows updates contributed to their browser preference being changed or reset (the "Reset Trap"). 44% of those aged 65+ found changing browser complex, vs 15% of 18–44s. 61% of those 65+ were only slightly or not at all confident. 86% say users should have primary control; 80% favour one-click default; 81% say Edge is not their preferred browser — [Browser Choice Alliance](https://browserchoicealliance.org/survey-finds-browser-choice-on-windows-undermines-user-expectations/)
- Press repeated the figures without independent validation — [Windows Central](https://www.windowscentral.com/software-apps/windows-undermines-user-expectations-survey-shows-windows-11-keeps-complicating-basic-browser-choices); [TechRadar](https://www.techradar.com/computing/browsers/1-in-3-windows-users-have-fallen-into-a-reset-trap-and-cant-escape-edge-i-think-its-high-time-microsoft-stopped-messing-around-with-default-browser-choices); [Windows Report](https://windowsreport.com/most-windows-11-users-are-struggling-to-change-their-default-browser-new-survey-claims/)
- BCA launched in Nov 2024 with Google (Chrome), Opera, Vivaldi, Waterfox and Wavebox, plus unnamed others. It pushed the EC to designate Edge under the DMA — [Vivaldi blog](https://vivaldi.com/blog/browser-choice-alliance-launch/); [gHacks, 27 Nov 2024](https://ghacks.net/?p=206507); [Wavebox](https://hub.wavebox.io/introducing-the-browser-choice-alliance/)

### Inferences
- Google, which sponsors the alliance, is itself criticised for default-on AI. Treat these numbers as advocacy data: directionally consistent with Mozilla's independent-researcher study (below), but not neutral.
- "29% claim updates reset their preference" is self-reported attribution. It is not measured reset behaviour.

### Gaps
- Fieldwork dates, margin of error and questionnaire were not published. No Microsoft response was found.

## Claim 5: Microsoft EEA DMA changes, Mozilla "Over the Edge", and the effects of DMA choice screens

### Takeaway
Microsoft made Edge and Windows changes in the EEA only, announced Nov 2023 and expanded around May/June 2025: Edge no longer nags unless opened directly (Edge 137.0.3296.52), and the default browser now covers more protocols and file types. Mozilla's Jan 2024 "Over the Edge" report (by Harry Brignull and Cennydd Bowles) documented the dark patterns. On mobile choice screens, Mozilla and an NBER paper report large relative gains for Firefox on iOS (+113% DAU vs counterfactual). Independent share data shows **small absolute share changes**, and the Commission has published no figures.

### Cited Findings
- Microsoft EU Policy Blog, 22 Nov 2023: EEA changes including no Edge recommendations during Windows setup and no pop-ups when users change their default browser — [Microsoft EU Policy Blog](https://blogs.microsoft.com/eupolicy/2023/11/22/windows-updates-comply-dma-edge/)
- 2025 EEA expansion: Edge prompts to become default only if opened directly (from Edge 137.0.3296.52, 29 May 2025). The default browser now also covers .htm, .html, .svg, .pdf, http, https, ftp and read, and can be pinned to the taskbar automatically. Edge and the Store can be uninstalled; Bing prompts are reduced. EEA only (EU plus Iceland, Liechtenstein, Norway) — [Windows Central](https://www.windowscentral.com/software-apps/windows-11/windows-11-dma-compliance-features-edge-default-announcement-2025); [Eleven Forum](https://www.elevenforum.com/t/microsoft-digital-markets-act-dma-updates-to-windows-10-and-windows-11-in-eu.36705/); [Pureinfotech](https://pureinfotech.com/microsoft-eases-edge-prompts-windows-users-europe-dma/); [Born City, 4 Jun 2025](https://borncity.com/win/2025/06/04/eu-dma-is-working-microsoft-no-longer-imposes-edge-store-search-etc/)
- The one-click "Set default" button returned worldwide in Mar 2022 (KB5011563), predating the DMA — [WindowsForum](https://windowsforum.com/threads/windows-11-default-browser-one-click-switch-and-eu-dma-changes.400542/)
- Mozilla "Over the Edge: The Use of Design Tactics to Undermine Browser Choice" (published Jan/Feb 2024), authored by Harry Brignull and Cennydd Bowles. Examples: Edge ads injected on the Chrome download page, Bing ads for Edge, and Edge-centred defaults. Mozilla called Microsoft's EEA-only pledges insufficient — [Mozilla Foundation](https://www.mozillafoundation.org/en/research/library/over-the-edge-the-use-of-design-tactics-to-undermine-browser-choice/); [The Register, 2 Feb 2024](https://www.theregister.com/2024/02/02/mozilla_slams_microsoft_dark_patterns/)
- Mozilla, 11 May 2026: more than 6 million Firefox selections via DMA choice screens, with 5x higher retention. An NBER working paper (w35112), comparing the EU with 43 non-EU countries over 15 months before and after, found Firefox DAU **+113% on iOS** and **+12% on Android** vs counterfactual. Mozilla says about 310 million EU desktops and laptops still lack equivalent choice — [Mozilla Netpolicy blog](https://blog.mozilla.org/netpolicy/2026/05/11/six-million-selections-later-how-the-dma-is-giving-people-browser-choice/); [NBER w35112](https://www.nber.org/system/files/working_papers/w35112/w35112.pdf)
- Early 2024: Brave, Opera and Vivaldi reported uplift after the iOS choice screen. DuckDuckGo and Firefox said it was too early to tell. Vivaldi appears on the iOS screen in only 8 countries — [TechCrunch, 10 Apr 2024](https://techcrunch.com/2024/04/10/eu-dma-browser-choice-screen-early-impact/)
- Counterpoint: Truth on the Market (13 Oct 2026, think-tank aligned with ICLE) ran a difference-in-differences analysis on public share data for 11 countries and estimated the choice-screen effect at about **0.1 percentage point**. It reports that the Commission's April 2026 DMA review claimed a "surge" for alternative browsers but gave no figures. Firefox's Europe-wide share is about 4.94% (July 2026) — [Truth on the Market](https://truthonthemarket.com/2026/10/13/we-measured-europes-browser-revolution-bring-a-magnifying-glass/)
- Mozilla's earlier DMA post — [Mozilla blog](https://blog.mozilla.org/en/firefox/eu-digital-markets-act/)
- The EU declined to designate Edge as a DMA core platform service (Oct 2024) — [The Register, 7 Oct 2024](https://www.theregister.com/2024/10/07/microsoft_edge_eu_gatekeeper/)

### Inferences
- Choice screens produce large **relative** gains for small browsers (especially on iOS) but small **absolute** share shifts. Both "DMA works" and "DMA barely moved share" can be true at once, depending on the metric.
- Windows desktops have no choice screen, which is why Mozilla and BCA keep pushing on Windows.

### Gaps
- No Commission-published browser-share figures were found. The Apple-side numbers (e.g., Safari share change in the EU) come only from vendor or academic sources here; StatCounter EU iOS data was not retrieved.
- NBER w35112 authors and funding were not verified. Mozilla calls them "independent."

## Summary: what browsers do differently on AI (as of Oct 2026)

| Browser | Default AI posture | Off switch | Evidence |
|---|---|---|---|
| Chrome | Default-on: local Gemini Nano (about 4GB) auto-downloaded on eligible desktops; AI Mode / Create image NTP chips | Settings > System > On-device AI (rolling out since Feb 2026, not universal); `GenAILocalFoundationalModelSettings`=1; NTP chips flag-only | [Hanff](https://www.thatprivacyguy.com/blog/chrome-silent-nano-install/), [Register](https://www.theregister.com/a/5237580), [Piunikaweb](https://piunikaweb.com/2026/09/28/how-to-remove-chrome-ai-mode-create-image-buttons/) |
| Edge | Default-on: Copilot Mode folded into standard Edge (May 2026); Copilot NTP | Partial: hide icon (Edge 144), individual toggles, same-name GenAI local model policy | [gHacks](https://www.ghacks.net/2026/05/15/microsoft-edge-retires-copilot-mode-and-integrates-ai-features-across-desktop-and-mobile/), [Windows Latest](https://www.windowslatest.com/2026/03/15/microsoft-edge-feels-more-like-copilot-than-a-browser-now-but-some-ai-features-can-be-turned-off/) |
| Firefox | Some features on by default (e.g., translation); chatbot sidebar and others offered | Global "Block AI enhancements" kill switch plus per-feature toggles (Fx 148, 24 Feb 2026) | [Mozilla](https://blog.mozilla.org/en/firefox/ai-controls/) |
| Brave | Leo opt-in: inactive until consent | Settings → Leo; group policy | [Brave Help](https://support.brave.app/hc/articles/27331617852045-How-do-I-disable-Brave-Leo) |
| DuckDuckGo | Optional AI (Duck.ai, Search Assist) | Can be hidden or set to "Never" | [DDG Help](https://duckduckgo.com/duckduckgo-help-pages/ai-features/opting-out-of-ai) |
| Vivaldi | No built-in LLM features | N/A | [Vivaldi blog](https://vivaldi.com/blog/technology/vivaldi-wont-allow-a-machine-to-lie-to-you/) |
| Opera | Not verified in this research | — | Gap |
