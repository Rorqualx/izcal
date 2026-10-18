# Shell UI design research notes: tabs, address bar, onboarding/defaults, settings, accessibility, recovery, density

Compiled 14 October 2026. Primary sources were preferred: peer-reviewed papers, vendor posts with data, W3C documents, and Bugzilla. Confidence levels:
- **High:** read in the primary source.
- **Med:** primary source exists, but the figure came from an abstract, excerpt, or reputable secondary coverage.
- **Low:** secondary or self-selected data only.

---

## 1. Tab use, tab hoarding, tab groups, vertical tabs, Arc

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **Tab hoarding is driven by fear of loss, not laziness.** Chang et al. found competing pressures to keep tabs open (reminders, avoiding re-finding, emotional or sunk cost, the "black hole" fear that anything out of sight is gone) against pressures to close them (attention, memory, crashes). | 10 information workers interviewed, with their tabs walked through 4 times over 2 weeks. Survey N=103 (MTurk, ages 21–70); a separate preliminary survey had N=64. 59% agreed they would find tabs that should be closed. 55% "feel like I can't let go of my tabs." Only 19% blamed laziness. ~25% said their browser or computer had crashed from too many tabs (CMU press release). ~30% self-described as having a "tab hoarding problem" (press coverage). | Chang, Hahn, Kim, Coupland, Breneisen, Kim, Hwong, Kittur. "When the Tab Comes Due," CHI '21. https://doi.org/10.1145/3411764.3445585 ; CMU news https://www.cmu.edu/news/stories/archives/2021/may/overcoming-tab-overload.html | May 2021 | Med. Abstract and design read directly; the percentages came from ACM full-text excerpts and the CMU release (the ACM PDF was blocked to fetch). |
| 1.2 | **Most browsing happens with few tabs, but there is a long tail of heavy users.** The most common state is one open tab, falling steadily to about 9, with a second bump at about 16 caused by heavy users. Over two-thirds of tabs are used once or less. Tab switching was more frequent than Back for 17 of 21 users. | N=21 Firefox users over 13–21 days. 45% of tabs were selected exactly once, 25% never, and 30% several times. A median 73.3% of tab switches were revisits. Back made up a median 7.1% of navigation actions. The highest per-user median was 17 open tabs (all others ≤6), and the highest max was 42. | Dubroy & Balakrishnan, "A Study of Tabbed Browsing Among Mozilla Firefox Users," CHI 2010. https://www.dgp.toronto.edu/public_user/ravin/papers/chi2010_tabbedbrowsing.pdf | Apr 2010 | High (read PDF), but the sample is old and small |
| 1.3 | **Parallel browsing is the norm.** A log study of millions of users found a power-law distribution of tab switches. | 57.4% of tab sessions involve parallel browsing (at least one tab switch). | Huang & White, "Parallel Browsing Behavior on the Web," ACM Hypertext 2010. https://jeffhuang.com/papers/ParallelBrowsing_HT10.pdf | Jun 2010 | High |
| 1.4 | **Mozilla telemetry on tab counts is bucketed and not published as a distribution.** `browser.engagement.max_concurrent_tab_count` reports values 1–10 exactly and buckets above that. A community reading of a Nightly dashboard suggested a median max of about 4 tabs per session. | Median about 4 (Nightly, unofficial reading). | Firefox source docs https://firefox-source-docs.mozilla.org/browser/BrowserUsageTelemetry.html | — | Low for the median, High for the probe definition |
| 1.5 | **Firefox tab groups adoption** (shipped in Fx 138, spring 2025). | ">18 million people" had used tab groups (cumulative, not a share). Readers noted that groups are easy to create by accident with drag-to-group. | Mozilla blog, "Firefox tab groups just got an upgrade" https://blog.mozilla.org/en/firefox/tab-groups-updates/ | Oct/Nov 2025 | Med (vendor number, method undisclosed) |
| 1.6 | **Firefox vertical tabs** (Fx 136) were built from Mozilla Connect feedback. The post gives no adoption data. Horizontal tabs are framed as unmanageable at ">10 tabs". | Team of 1 PM, 1 UX lead, and 5 engineers. | Mozilla blog https://blog.mozilla.org/en/firefox/vertical-tabs-and-the-firefox-community/ | 2 Apr 2025 | High (and no numbers exist there) |
| 1.7 | **Chrome native vertical tabs** shipped to stable via right-click on the tab strip, then "Show Tabs Vertically". The setting persists once on. No adoption data. | — | Google blog https://blog.google/products-and-platforms/products/chrome/new-chrome-productivity-features/ ; TechCrunch https://techcrunch.com/2026/04/07/chrome-is-finally-getting-vertical-tabs/ | 7 Apr 2026 | High |
| 1.8 | **Edge vertical tabs** (GA around March 2021). Research found that people who like vertical tabs switch back and forth with horizontal tabs, so Edge added an always-visible toggle and a collapsible pane. One developer described 200 open tabs as "mental gymnastics". | No adoption share published. | Microsoft Research blog https://www.microsoft.com/en-us/research/?p=728920 | 4 Mar 2021 | High (qualitative) |
| 1.9 | **Chrome tab groups** were designed around "tab minimalists" vs "tab collectors" and tested internally for months. No public adoption numbers. | — | Google blog https://blog.google/products/chrome/manage-tabs-with-google-chrome/ | 13 May 2020 | High (qualitative only) |
| 1.10 | **Arc's power features had tiny reach.** Arc's novelty imposed a "novelty tax". Scott Forstall compared it to a saxophone ("powerful but hard to learn") and recommended a piano instead. | **5.52%** of DAUs regularly use >1 Space. **4.17%** use Live Folders. **0.4%** use Calendar Preview on Hover. By contrast, Dia's chat-with-tabs feature reaches 40% of DAUs and personalization 37%. D1 retention was described as "strong", with no number given. | The Browser Company, "Letter to Arc members 2025" https://browsercompany.substack.com/p/letter-to-arc-members-2025 | 26 May 2025 | High |
| 1.11 | **What fraction of users change tab layout:** no vendor has published a share for vertical tabs, tab groups, or tab-strip customization (Chrome, Edge, Firefox). Arc's Spaces figure (1.10) is the best public proxy for opt-in organization features: about 5%. | — | — | — | Gap |

**Design implications:** Keep a horizontal strip as the default, with a one-click persistent toggle to vertical (Edge, Chrome). Design for a bimodal user base (minimalists vs. collectors). Fear of losing information is the main barrier to closing tabs, so offer safe ways to close: a reliable "closed tabs / tab history", save-and-close of a group, and search. Expect advanced organization features to reach no more than about 5% of users unless they are the default.

---

## 2. Address bar / omnibox and search choice

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **Chromium omnibox design principles.** Reduce cognitive overhead by merging the URL and search boxes. Keep the dropdown visually minimal ("we believe users ignore the dropdown most of the time"). Keep the default match stable and predictable (synchronous inline autocomplete; the top item does not reshuffle as async results arrive). The model is "accelerate my typing" rather than proactive results, because the proactive designs tested had too much overhead. Single-word inputs default to search because they are mostly searches. | Qualitative, no published share figures. | https://www.chromium.org/user-experience/omnibox/ | undated (long-standing) | High |
| 2.2 | **Direct access (typing a URL, bookmarks, homepage) is a small share of navigation.** Links make up 44% of actions, form submissions (including search) 15%, new window or tab 11%, and Back 14.3%. | Direct access = 9% of navigation actions. N=25, long-term click-stream study. | Obendorf, Weinreich, Herder, Mayer, "Web Page Revisitation Revisited," CHI 2007. https://vsis-www.informatik.uni-hamburg.de/getDoc.php/publications/280/chi2007-newformat.pdf | 2007 | High (read), but dated |
| 2.3 | **Firefox urlbar telemetry** tracks engagement by selection method (enter, click, arrow) and by result type. The legacy `urlbar.engagement` scalar was removed in Fx 137 and moved to a Glean event. No public split between typed URL, search, and suggestion. | — | https://firefox-source-docs.mozilla.org/browser/urlbar/telemetry.html | 2025 | High (definitions only) |
| 2.4 | **Gap:** no recent public vendor data on the share of omnibox navigations that are typed URL vs. search vs. history or bookmark suggestion. | — | — | — | Gap |
| 2.5 | **The DMA browser choice screen effect (NBER, difference-in-differences: EU vs. 43 non-EU countries).** Active choice moved users away from preset defaults. On iOS, Safari's share fell and Chrome gained most. | From month 15, Firefox usage was **+91% on iOS** and **+13% on Android** vs. counterfactual. (Mozilla's blog cites +113% / +12% from an earlier version.) | Akesson, Amlani, Cepeda Suarez, Chissell, Hunt, Luca, Petrie, NBER WP 35112 https://www.nber.org/papers/w35112 | Apr 2026, rev. Jul 2026 | High |
| 2.6 | **Mozilla's DMA results** (interested party). | More than 6 million Firefox selections via choice screens, about one every 10 seconds. Retention is **5×** higher when Firefox is picked from a choice screen. | Mozilla netpolicy blog https://blog.mozilla.org/netpolicy/2026/05/11/six-million-selections-later-how-the-dma-is-giving-people-browser-choice/ | 11 May 2026 | Med (vendor) |
| 2.7 | **Choice screen design experiment** (N=12,000, simulated device setup on Android and Windows, across EU countries). | 98% of choosers expected to stay with their pick, vs. 51% of the control group expecting to switch away from the preset. Position effects: Chrome ranked 4th instead of 1st lost 7.1 pp, and the preset browser lost 6.3 pp (a 38% relative drop). Showing the screen at first browser launch instead of device setup raised preset-browser choice from **11% to 19%**. Preferred timing: 65% at device setup vs. 33% at first launch. Choice screens raised "felt in control" by 12% (to 75%), with no significant added setup time. More information raised satisfaction with the information from 56% to 64%. | Mozilla (Akesson, Luca, Petrie, Amlani), "Can browser choice screens be effective?" https://research.mozilla.org/files/2023/09/Can-browser-choice-screens-be-effective_-Mozilla-experiment-report.pdf | Sep 2023 | High (read PDF) |
| 2.8 | **DMA search engine choice screen (Android, March 2024, 14 providers):** no Commission or Google data on share shifts found. DuckDuckGo claims about 40% higher selection rates vs. the old Android screen. | ~40% (self-reported) | DuckDuckGo submission to the UK CMA https://assets.publishing.service.gov.uk/media/69b970da635612b767a46662/DuckDuckGo.pdf | 2025–26 | Low |

**Design implications:** An in-product search engine choice at onboarding should randomize or rotate order, carry short descriptions, and fit within a single step. Ordering effects are large, and choices made actively stick.

---

## 3. Onboarding and defaults

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Few people change settings.** A sample of Word settings files found that "less than 5%" had changed anything. Autosave shipped off by accident, so about 95% ran without it, and users assumed the default was deliberate. | <5% (sample of "several hundred") | Jared Spool, UIE, "Do users change their settings?" https://archive.uie.com/brainsparks/2011/09/14/do-users-change-their-settings/ | 14 Sep 2011 | Med (informal study, often over-cited) |
| 3.2 | **Counterpoint for power users:** 92% of WordPerfect users customized something (mean 9.1 changes). Customization concentrates in experienced users. | 92% | Page, Johnsgard, Albert, Allen, "User Customization of a Word Processor," CHI '96 https://chi1996.acm.org/proceedings/papers/Page/srp_txt.html | 1996 | High |
| 3.3 | **Defaults dominate privacy settings.** Before Firefox turned Enhanced Tracking Protection on by default, only about 20% had it on. The default flip took coverage to effectively everyone (new users from June 2019, all users with Fx 69). | >20% opt-in before, ~100% after | Mozilla blog https://blog.mozilla.org/en/?p=62557 | 3 Sep 2019 | High |
| 3.4 | **Defaults dominate browser choice.** About 90% of iPhone users use Safari and ~25% of Samsung users use Samsung Internet. Only about 10% of users actively install a different browser. In the experiment, 49% of the control group expected to stay with the preset. | See 2.7 | Mozilla choice screen report (above) | 2023 | High |
| 3.5 | **First-run import: most users decline when asked. Auto-import helps but raises privacy concerns, and slow import hurts usage.** Firefox's prior research found most users skipped import because they wanted to try the browser first, wanted a fresh start, or were unclear about how long it would take. An auto-import prototype (with undo) was tested. | Only 22/67 discovered the auto-import message unprompted. 57/67 kept the imported data, including 13 of 17 concerned participants. 14–67% voiced privacy concerns depending on design. In a 1% in-product experiment, longer import time was associated with fewer subsequent sessions (p=0.00127). Mozilla shelved auto-import in favor of a better import prompt. | Davidson, Guha, Kaye, Dolanjski, Verdi, Petrie, Heubusch, CHI '18 LBW, "Will Automatically Importing User Data Help Overcome the Blank Slate Problem?" https://doi.org/10.1145/3170427.3188542 (PDF: https://research.mozilla.org/files/2018/05/Will-Automatically-Importing-User-Data-Help-Overcome-the-Blank-Slate-Problem.pdf) | Apr 2018 | High |
| 3.6 | **A default-browser prompt at first run showed no retention effect.** Mozilla moved the modal prompt off first run (to the 2nd startup in Fx 55). Preliminary cohort data showed "no change (positive or negative) to retention". Average daily installs were about 5% higher for the prompt cohort, which is not a retention measure. | ~5% ADI difference, retention flat | Bugzilla 1143116 (dup of 1367073) https://bugzilla.mozilla.org/show_bug.cgi?id=1143116 | ~2015–2017 | Med |
| 3.7 | **The about:welcome onboarding is a multi-screen, experiment-driven (Nimbus) flow.** No public completion-vs-length data. | — | https://firefox-source-docs.mozilla.org/browser/components/asrouter/docs/about-welcome.html | — | Gap for numbers |

**Design implications:** Treat every default as the product, since 80–95% will never change it. Put the few consequential choices (search engine, import, maybe tab layout) in onboarding, make import fast, local, and explained ("never leaves your device"), and make every first-run choice undoable. Don't count on a "set as default" nag to buy retention.

---

## 4. Settings information architecture

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **Firefox settings redesign (2026).** The long-standing "General" page was retired and its options split into Appearance, Accessibility, Languages, and Tabs & browsing. Passwords and autofill were reworked, and search was improved. The rationale: crowded pages, related options scattered across sections, and users unable to predict where an option lives. Methods: interviews, usability testing, card sorting, usage-data analysis, and Mozilla Connect/Reddit feedback. | No metrics published | Mozilla blog https://blog.mozilla.org/en/firefox/firefox-settings/ ; Phoronix (default in Nightly 153) https://www.phoronix.com/news/Firefox-Nightly-New-Settings | 16 Jun 2026 | High |
| 4.2 | **The Firefox 2017 Preferences reorg** (Quantum era) cut the number of panes and added search in preferences. Section moves were tracked in Bug 1365133 (fixed in Fx 56). A card sort was run. | Panes reduced (planned to 5) | gHacks https://www.ghacks.net/2017/03/07/mozilla-updating-firefox-preferences/ ; https://bugzilla.mozilla.org/show_bug.cgi?id=1365133 | 2017 | Med |
| 4.3 | **Chrome desktop settings (2025):** "You and Google" sits at the top, "Basics" is trimmed to 4 items (Search engine, Address bar, Privacy & security, Safety check), a new "Passwords and Autofill" group was added, and the rest sits under Advanced. Privacy & security was promoted out of "Advanced" in 2019. Earlier, Chrome added in-settings search. | — | 9to5Google (secondary); Android Central (2019) https://androidcentral.com/google-wants-make-privacy-and-security-settings-more-prominent-chrome | 2019–2025 | Med/Low |
| 4.4 | **Gap:** no published data from any vendor on how often search-in-settings is used. | — | — | — | Gap |

**Design implications:** Base the categories on card sorting. Avoid a catch-all "General" page. Promote the 4–6 most consequential settings, and make search in settings first-class, including synonyms.

---

## 5. Accessibility of browser chrome

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **WCAG2ICT (W3C Group Note) maps WCAG 2.0/2.1/2.2 to non-web software, so it applies to a browser's native chrome.** It is informative, not normative. Most success criteria apply with word substitution ("content" for "web content"). For 1.4.4 Resize Text it allows "platform text sizing capabilities". Criteria tied to "sets of web pages" (2.4.1, 2.4.5, 3.2.3, 3.2.4, 3.2.6) apply only to "sets of software programs", which are rare. Section 508 and EN 301 549 exclude 2.4.1, 2.4.5, 3.2.3, and 3.2.4 for software. Browsers are defined as "platform software" and "user agents" that must expose information through platform accessibility services (1.3.1, 4.1.2). | — | https://www.w3.org/TR/wcag2ict-22/ ; announcement https://www.w3.org/WAI/news/2024-10-08/wcag2ict | 8 Oct 2024 (rev. 15 Nov 2024) | High |
| 5.2 | **UAAG 2.0 (W3C Note)** is the browser-specific guideline: full keyboard access to UI, focus visibility, honoring platform settings, and so on. | — | https://www.w3.org/TR/UAAG20/ | Dec 2015 | High (existence and scope) |
| 5.3 | **Chromium keyboard-access principles.** The goal is full keyboard operability without a separate "accessibility mode". Most toolbar controls are kept out of the Tab order. F6/Shift+F6 cycle panes (toolbar, bookmarks, content, downloads). Alt+Shift+T focuses the toolbar and Alt+Shift+B the bookmarks bar, arrow keys move within a toolbar, and Esc returns to content. Alt or F10 focuses the menu. | — | https://www.chromium.org/user-experience/keyboard-access/ | long-standing | High |
| 5.4 | **Safari does not include links in the Tab order by default.** Users need Option-Tab or Settings > Advanced > "Press Tab to highlight each item", plus macOS Keyboard Navigation for full control access. This is a well-known keyboard-user pitfall. | — | University/vendor help pages, e.g. https://uwaterloo.atlassian.net/wiki/spaces/WCMSKB/pages/44619595897/Tabbing+in+Safari | — | Med |
| 5.5 | **Screen reader users' browsers (WebAIM survey #10, N=1,539).** "Lack of keyboard accessibility" ranked 5th of 12 problem items. | Chrome 52.3%, Edge 19.3%, Firefox 16.0%, Safari 8.0%. JAWS 40.5%, NVDA 37.7%, VoiceOver 9.7%. | https://webaim.org/projects/screenreadersurvey10/ | Dec 2023–Jan 2024 | High |
| 5.6 | **Multi-process architecture can hurt AT performance. Firefox's fix ("Cache the World") caches the accessibility tree, for both content and browser UI, in the parent process, modeled on Chrome.** Quantum had made screen reader performance worse. | Very large tables up to 20× faster, other very large pages ~10× faster, everyday actions 2–3×. Shipped on Android Fx 102, Win/Linux Fx 112, macOS Fx 113. | Mozilla Accessibility blog https://blog.mozilla.org/accessibility/firefox-113-accessibility-performance/ | 9 May 2023 | High |
| 5.7 | **The Firefox UI is moving from XUL to HTML/custom elements.** Windows, popups, and panels still need XUL. Practical lessons: custom elements need dashed names (moz-*), XUL layout attributes (flex, orient) must move to CSS, and 300 XBL bindings had to be removed. A web-tech UI gets web a11y semantics (ARIA, HTML) for free, but the Storybook "moz-" component library is how consistency, including focus and HCM, is enforced. | 300 → 115 bindings at one milestone | Firefox source docs https://firefox-source-docs.mozilla.org/browser/components/storybook/docs/README.xul-and-html.stories.html ; Brian Grinstead's newsletters https://briangrinstead.com/blog/xbl-replacement-newsletter-12/ | 2018–2020 | Med (no a11y-specific retrospective found) |
| 5.8 | **Forced colors / high contrast in chrome.** Firefox reports `forced-colors: active` under Windows HCM or Firefox's own HCM. Windows lets users override 7 colors and Firefox HCM 4. macOS "Increase Contrast" alone does not trigger forced colors (`prefers-contrast` vs `forced-colors` are distinct and not interchangeable). Firefox's own UI is styled with system colors under HCM. | 7 vs 4 overridable colors | https://firefox-source-docs.mozilla.org/accessible/HCMMediaQueries.html ; https://support.mozilla.org/kb/firefox-contrast-control | 2023–2025 | High |
| 5.9 | **Firefox Nova (2026)** lists contrast, focus states, keyboard behavior, target sizes, and honoring system settings as design criteria, and frames customization (dark mode, density) as accessibility. | — | https://blog.mozilla.org/en/firefox/new-firefox-design/ | 21 May 2026, updated 29 Sep 2026 | High |
| 5.10 | **Arc / Vivaldi accessibility.** Evidence is mostly anecdotal. Vivaldi community posts call its Workspaces and Mail the least accessible among Chromium browsers for NVDA users, and note that macOS Full Keyboard Access defaults to "Focus Input" rather than all controls. For Arc, no vendor accessibility conformance statement was found. | — | Vivaldi forum https://forum.vivaldi.net/post/471276 , https://forum.vivaldi.net/post/679422 | 2020–2024 | Low |

**Design implications:**
- Expose every chrome control through native a11y APIs.
- Implement F6 pane cycling, toolbar focus shortcuts, and arrow-key roving within toolbars.
- Never ship novel surfaces (Spaces, side panels, split view, command bar) without keyboard and screen-reader paths.
- Test with JAWS and NVDA on Chrome-class engines first, since that is where the users are.
- Honor forced-colors/HCM with system colors, and treat `prefers-contrast` separately.
- Use WCAG2ICT plus UAAG 2.0 as the conformance checklist for native chrome, including 2.4.11 Focus Not Obscured and 2.5.8 Target Size.

---

## 6. Recovery / safe mode

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **Firefox Troubleshoot Mode** (renamed from "Safe Mode" around Fx 87–88, early 2021). It disables extensions, themes, and some customizations temporarily and does not touch user data or prefs. It launches from Help > Troubleshoot Mode, by holding Shift at launch, or with `-safe-mode`. The rename reflects that it "does not only disable add-ons". Thunderbird followed in bugs 1696499 and 1697246, citing Firefox's wording: "Use this special mode … to diagnose issues. Your extensions and customizations will be temporarily disabled." | — | https://support.mozilla.org/kb/diagnose-firefox-issues-using-troubleshoot-mode ; https://bugzilla.mozilla.org/show_bug.cgi?id=1696499 | Mar 2021 | High |
| 6.2 | **Old wording was inaccurate.** It claimed "custom settings" were disabled when prefs stayed in effect, so the copy was changed. Lesson: recovery-mode copy must say exactly what is and isn't disabled. | — | Bugzilla 699233 https://bugzilla.mozilla.org/show_bug.cgi?id=699233 | 2011–2012 | High |
| 6.3 | **Automatic offer after startup crashes.** `toolkit.startup.max_resumed_crashes` (default 3, within ~6 h) offers Troubleshoot Mode after repeated startup crashes. A single crash auto-restores the session, and repeated crashes prompt. Users complain when the prompt recurs (Bug 1711737). | 3 crashes | SUMO threads, Bugzilla 1012715 / 1711737 | various | Med |
| 6.4 | **Refresh Firefox** creates a new profile and carries over bookmarks, history, passwords, cookies, and autofill. It drops extensions and prefs, keeping an "Old Firefox Data" folder. Profile-manager compatibility arrived in Fx 150. | — | https://support.mozilla.org/en-US/kb/refresh-firefox-reset-add-ons-and-settings/ | 2026 | High |
| 6.5 | **Chrome has no safe mode.** Its equivalents are Settings > Reset settings (added Oct 2013) and an automatic "Reset altered Chrome settings?" prompt (Jan 2014), because "settings hijacking remains our number one user complaint". The reset disables extensions and themes. | — | eWeek / SecurityWeek coverage of Chrome blog https://www.securityweek.com/google-prompts-windows-users-if-chrome-settings-hijacked/ | Jan 2014 | Med |
| 6.6 | **Gap:** no published usage rates for Troubleshoot Mode or Refresh. | — | — | — | Gap |

**Design implications:** Build in a named, discoverable recovery mode that is temporary and non-destructive (extensions, themes, and customizations off; data untouched). Write precise copy. Auto-offer it after N startup crashes, with a cap on re-prompts. Add a separate one-click "reset to defaults, keep data" that preserves a backup.

---

## 7. Dark mode, density, typography

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **Firefox removed Compact density in Proton (Fx 89, 2021) without measured usage data.** The bug says compact is "hard to discover, and we assume gets low engagement". The rationale used a screen-height hardware report, with the minimum height target set at 768 px. Critics noted that the report counts physical pixels and ignores DPI scaling. Outcome: compact was kept only for existing users and hidden behind an about:config pref (Bug 1703254). Telemetry for compact usage (Bug 1698100) was added only afterward. | 31.7% at 768 px height, 61.6% above. Proton tabs and toolbar were 92 px. Proton measured 26% taller than Quantum and 15% taller than Chromium (Ubuntu). | Bugzilla 1693028 https://bugzilla.mozilla.org/show_bug.cgi?id=1693028 | Feb–Apr 2021 | High |
| 7.2 | **Compact mode is back as a first-class option in Firefox Nova** ("People told us that they missed it, and we listened"). It appears as Window Density under Appearance, with an Automatic default. The design reached all users in Fx 157 (late Sep 2026). | No usage data published | Mozilla blog https://blog.mozilla.org/en/firefox/new-firefox-design/ ; PCWorld https://www.pcworld.com/article/3247664/firefoxs-biggest-redesign-in-years-just-landed-with-nova.html | May / Sep 2026 | High (return), Med (version) |
| 7.3 | **Dark mode prevalence: no representative vendor or peer-reviewed data.** Available figures: NCSU Libraries analytics found 38% of site visitors had OS dark mode on (one institution's audience). An Android Authority reader poll (>3,000) found ~70% "always on" (self-selected). Several academic sources explicitly note the lack of comprehensive studies. | 38% (analytics) vs ~70% (polls) | https://www.lib.ncsu.edu/node/68686 ; https://androidauthority.com/dark-mode-survey-results-3682352 | 2020s | Low–Med |

**Design implications:** Ship density as a visible Appearance setting (Compact / Normal / Touch, defaulting to Automatic). Instrument it *before* any removal decision. Follow the OS light/dark setting by default. Measure the UI height budget in logical (DPI-scaled) pixels.

---

## 8. Usability methods used by browser vendors

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **Mozilla Firefox UX research is mixed-methods.** Generative research covers concepts and mental models. Evaluative research uses usability testing, unmoderated remote tests on usertesting.com, copy tests, and prototype tests, plus in-product Nimbus/Shield experiments on 1% of the population (e.g., 3.5). A "Usability Mentorship" program trains designers to run their own usability studies because demand exceeds research capacity. | — | https://blog.mozilla.org/ux/2019/12/designer-you-can-run-a-usability-study-usability-mentorship-at-mozilla/ ; CHI '18 LBW (3.5) | Dec 2019 | High |
| 8.2 | **Firefox Mobile benchmarking** used task analysis, surveys, and unmoderated video sessions. New users found bookmarks less intuitive than long-time users did, which led to a redesign. | — | UserTesting podcast https://www.usertesting.com/resources/podcast/mozilla-ux-research-familiarity-innovation-firefox-mobile | 2024 | Med |
| 8.3 | **Vendor process patterns:** Microsoft Edge uses user research plus a feedback toggle (1.8) and rewrote tracking-prevention copy after confusion. Mozilla runs community programs (Mozilla Connect) for vertical tabs, tab groups, and Nova. Google ran months of internal dogfood for tab groups. | — | sources in 1.6, 1.8, 1.9 | 2020–2026 | High |
| 8.4 | **Gap:** no published SUS/UX-Lite benchmark comparing browsers' shell UIs was found. Use SUS ≥68 (industry average) and task-success benchmarks as an internal bar. | — | — | — | Gap |

---

## Cross-cutting takeaways for the shell design

1. **Defaults are the design.** 80–95% of users keep them (3.1, 3.3, 3.4), and opt-in power features reach about 0.4–5% (1.10).
2. **Fear of loss drives tab clutter** (1.1). Safe closing, search, and recovery matter more than more organization chrome.
3. **Make layout switching one click and persistent** (Edge, Chrome). Do not force vertical tabs or Spaces as the default.
4. **Active choice moments are powerful and order-sensitive** (2.5, 2.7). Randomize order, keep them short, and place them at setup.
5. **Instrument before removing** (Proton compact, 7.1). Removing a feature on assumption cost Mozilla credibility, and the feature was later restored.
6. **Accessibility must be architectural:** native a11y tree caching (5.6), pane and toolbar keyboard model (5.3), and forced colors (5.8). Use WCAG2ICT plus UAAG as the checklist (5.1, 5.2).
7. **Ship a recovery mode with precise copy** (6.1, 6.2), auto-offered after repeated startup crashes.

## Open gaps (no public primary data found)
- Share of navigations from the omnibox by type (typed URL, search, or suggestion).
- Share of users on vertical tabs or tab groups in Chrome or Edge.
- Usage of search-in-settings.
- Completion rate of onboarding by length.
- Usage of Troubleshoot Mode, Refresh, or Reset.
- Representative dark mode share.
- Browser SUS benchmarks.
