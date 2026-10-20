# Izcal accessibility: fact-check

Checked 19 October 2026 by an independent fact-checker (phase 8 of the first-principles-spec method), who did not write the spec or the notes.
Subject: `reports/Izcal accessibility.md` (draft 0.1) and the eight notes in `research_notes/Izcal accessibility/`.

**Method.** Every number, date, quote and causal claim in Sections 1–3, 5, 6, 7, 9.3 and 11.2 was traced to the cited note row, and the load-bearing ones to the primary source. Primary sources were fetched directly; no web search was used. They were: Teh's "Cache the World" post; the Chrome accessibility-performance blog and both Chrome UIA blog posts; the NVDA 2026.2 User Guide and nvaccess/nvda#19276; the NVD 2.0 API query in the spec; the Bugzilla REST API (counts and titles for Core › Disability Access APIs); WebAIM Screen Reader Surveys #11 and #5 and the WebAIM Million; the W3C Design Principles; the AOM explainer; the ariaNotify explainer; Core-AAM (ED); CSS Contain 2; Media Queries 5; the Microsoft Learn ProviderOptions page; the Mozilla CacheTheWorld wiki; Chromium's prose docs `overview.md`, `how_a11y_works_2.md` and `android.md`; the ARIA-AT CG minutes of 1 July 2026; interop-accessibility#202; wpt rfcs#204; the State of HTML 2025 accessibility page; the Firefox Accessibility Inspector docs; Soueidan's and TetraLogical's live-region posts; the AccessKit README (prose only); MDN browser-compat-data (`api.Element.ariaNotify`); and crates.io's API for every crate in Section 7. The macOS 26.2 SDK's public headers `AXWebConstants.h` and `NSAccessibilityConstants.h` in the local Xcode were counted: these are Apple OS interface definitions, the same material the PA note used, and not engine code. Arithmetic was recomputed. Rows marked "note only" were checked against a note row graded High, without fetching the primary source again. No engine or AccessKit source code was opened.

**Counts.** 140 claims checked: **115 correct**, **17 partly correct**, **3 wrong**, **5 unverifiable**. Of the 17 partly correct:
- 5 are quotation problems: altered quotes, or a paraphrase shown as a quote.
- 6 present a Low- or Medium-confidence note row as fact without a flag.
- 1 is an unflagged conflict between two notes.
- 5 are imprecise wording or attribution. Must-fix: 9. Recommended: 19.

## Claims

| # | Claim (spec location) | Verdict | Evidence | Fix |
|---|---|---|---|---|
| 1 | Chrome's first multi-process design was "a dead-end" (intro; §5) | Correct | Chromium `how_a11y_works_2.md` section heading "A dead-end: proxying accessibility requests"; PE 1.1 | — |
| 2 | Firefox 57's partial cache and synchronous IPC made screen readers "take a massive step backwards" (intro) | Partly correct | Teh: "The performance with assistive technologies took a massive step backwards when this was first released in Firefox 57." The quote is altered (verb form, subject) | Quote exactly: "performance with assistive technologies 'took a massive step backwards'" |
| 3 | Cache the World ran 2021–23 (intro; §2.1) | Correct | HP 2.18: proof of concept December 2021, macOS in Firefox 113 on 9 May 2023 | — |
| 4 | NVDA load of a 12,000-row table fell from 128 s to 6 s (intro; §1; §2.1) | Correct | Teh's table: "a table with over 12000 rows 128 sec 6 sec" | — |
| 5 | WHATWG HTML spec load fell from 175 s to 15 s (intro; §2.1) | Correct | Teh's table: "175 sec 15 sec" | — |
| 6 | CtW "removing about 20,000 lines" / "about 20,000 lines removed" (intro; §2.1; §5) | Partly correct | Teh (December 2022) gives a projection: "once this is fully released, we will be able to remove around 20,000 lines of code". No source confirms the removal | "was expected to remove about 20,000 lines" in all three places |
| 7 | NVDA maintainers said in November 2025 that they "cannot move the default to UIA" (intro; §2.1) | Correct | #19276: seanbudd, 27 Nov 2025: "We cannot move the default to UIA, as overall UIA has a worse implementation than the current IA2 implementation from Chromium in Browse mode" | — |
| 8 | Windows screen readers build their browse-mode buffers inside the browser process through IA2 (intro; §1) | Correct | NVDA 2026.2 guide: UIA is used only "When NVDA is unable to inject into the browser process in order to use IA2"; PA 2.3 (nvdaHelper readme) | — |
| 9 | 16 Chrome accessibility CVEs in 2026, most needing a compromised renderer (intro; §3) | Correct | NVD query fetched today: 31 results, 16 published in 2026; 12 of those 16 say "compromised the renderer process" | — |
| 10 | Several 2026 CVEs are stated as sandbox escapes or site-isolation bypasses (§3) | Correct | NVD: CVE-2026-13806 and CVE-2026-106403 ("bypass si[te isolation]"); CVE-2026-79175 "execute arbitrary code outside the sandbox"; SR 4.3 | — |
| 11 | Mozilla bug titles name self-referential trees, ID-0 cycles and double parents (intro; §5) | Correct | Bugzilla REST titles: 2016664 "Self-Referential Accessible Tree", 2027971 "ID=0 self-reparenting cycle", 2016661 "Split Show Double-Parent UAF", 2027237 "cross-PBrowser actor forgery" | — |
| 12 | AOM's AT events died because they reveal AT use (intro; §2.1; §5) | Correct | AOM explainer: events on virtual nodes "would be an immediate indication that the user must be using assistive technology, which is private information"; AT actions trigger "a synthesised DOM event" "To preserve the privacy of assistive technology users" | — |
| 13 | W3C Design Principles §2.11 forbids revealing AT use (intro) | Correct | §2.11 "Don't reveal that assistive technologies are being used" | — |
| 14 | Ledger G-10 (B8 and 5.8 rest on a superseded dependency) and G-17 (accessibility unowned) (§1) | Correct | ledger.md rows G-10 and G-17 | — |
| 15 | DT-D6 reads "the AccessKit update stream"; shell 4.9 "exposes its tree through the same AccessKit path as content" (§1) | Correct | DevTools spec, DT-D6 paragraph; shell UI design 4.9 | — |
| 16 | JS-D11 is owner-accepted and classifies AccessKit as engine work (§1) | Correct | JS engine spec, revision 0.3 line | — |
| 17 | Windows is 90.7% of primary screen-reader use; JAWS 55.0%, NVDA 32.9%, VoiceOver 6.5%, Orca 1.4% (§1) | Correct | WebAIM SR Survey #11 tables | — |
| 18 | A synchronous-IPC browser was 21× slower than a cached one on the 12,000-row table (§1) | Correct | 128 ÷ 6 = 21.3 | — |
| 19 | WCAG 2.5.3 exists because names that differ from visible text break speech commands (§1) | Correct | UX 5.1, SR 7.1 (Understanding 2.5.3), note only | — |
| 20 | A custom-drawn caret is invisible to Windows magnifiers unless mirrored as a system caret (§1; 6.8) | Correct | PA 3.4 (Microsoft Learn TextPattern page), note only | — |
| 21 | 37% of Android and 34% of iOS users change the system text size (§1) | Partly correct | UX 1.21 is **Medium**: Appt data cited second-hand in a CSSWG explainer. The spec states it without a flag | Add to 11.2 as a secondary figure |
| 22 | Under 10% of sites honour the Android OS text size (§1; §3) | Correct | UX 7.3: chromestatus, "<10% of sites honor that slider" (Google's own claim) | — |
| 23 | On macOS no major browser responds to OS text scaling (§1; §3) | Correct | UX 7.4: meta-text-scale explainer quote | — |
| 24 | Computed role and name are 87–97% interoperable in WPT (§1; §3) | Correct | SS 8.5–8.7: accname 95–97%, wai-aria 93–97%, html-aam 87–95% (note only) | — |
| 25 | The platform layer is "barely measured" (§1) | Partly correct | The phrase is the SS note author's own implication text ("barely measured yet"), not a source quote | Drop the quotation marks |
| 26 | The AAMs leave text, caret and hypertext unspecified (§1; §2.2; 6.8) | Correct | SS 5.12 (full-text counts in Core-AAM and HTML-AAM) | — |
| 27 | Servo's AccessKit adapter panics on out-of-order updates and has no iframe support (§1; §5; 6.5) | Correct | HP 4.7 (Servo book): ordering mistakes make the adapter panic; "We currently don't support accessibility trees in IFrames" | — |
| 28 | Chrome turns accessibility on for 5–10% of **desktop** users, **mostly** because password managers and antivirus ask (§1; §2.2; §5) | Partly correct | Chrome blog: "Approximately 5-10% of users have accessibility code enabled, **often** due to tools like password managers and antivirus software". It does not say desktop, and it says "often", not "mostly" | "5–10% of users … often because …" |
| 29 | Chrome calls the cost "steep" (§1) | Correct | Chromium `android.md`: "The Chrome accessibility code comes at a steep performance cost" (Android doc) | Optional: say it is the Android doc |
| 30 | Chrome's Auto Disable experiment shows the cost is "quite high" (6.3) | Correct | Blog: "a recent experiment called Auto Disable Accessibility … shows it's quite high" | — |
| 31 | FP-1 broker budget 20 MiB† (§1; 9.5) | Correct | Footprint spec | — |
| 32 | MSAA dates from 1997 as a Windows 95 add-in (§2.1) | Correct | HP 1.1 (Microsoft Learn), HP 1.5 (Campbell) | — |
| 33 | MSAA "tends to run slowly" out of process, so screen readers inject code (§2.1; §5) | Correct | HP 1.4, Microsoft Learn quote | — |
| 34 | ATK/AT-SPI over CORBA (2001); moved to D-Bus with GNOME 3.0 in April 2011 because ORBit was deprecated and memory-hungry (§2.1) | Correct | HP 1.14–1.16 (dates Medium, from Wikipedia; GNOME wiki High) | — |
| 35 | AT-SPI's pull design "predates sandboxing" (§2.1) | Partly correct | Campbell: it "was designed when per-app sandboxing was mostly research". The quoted words are a paraphrase | Unquote, or quote Campbell |
| 36 | NSAccessibility key-based API from 10.2 (2002) (§2.1) | Partly correct | HP 1.20 is **Low** (recollection). The 10.10 method-based protocol (2014) is sourced (HP 1.21, PA 4.1) | Flag 10.2 in 11.2, or drop it |
| 37 | macOS 26 SDK `AXWebConstants.h`: 59 web attributes, 32 parameterised attributes (§2.1; 6.10) | Correct | Local MacOSX26.2 SDK header: 59 attribute defines before the attributed-string-key block (plus 5 keys); 32 `ParameterizedAttribute` defines | — |
| 38 | 8 web notifications and 56 web subroles (6.10) | Correct | Same header: 8 `Notification`, 56 `Subrole`, 1 `Role` | — |
| 39 | 42 public search keys (§2.1; 6.8; 6.10) | Correct | macOS 26.2 `NSAccessibilityConstants.h`: 42 `NSAccessibilitySearchKey const` declarations | — |
| 40 | UIA was managed-only (2005), then rewritten as COM for Windows 7 (2009) (§2.1) | Correct | HP 1.6, Microsoft Learn quote | — |
| 41 | UIA has one live-region event and one `LabeledBy` target, and landmarks become "custom" (§2.1; §2.2) | Correct | HP 1.26 (Teh, 19 March 2025) | — |
| 42 | IA2 came from IBM's ODF accessibility project and kept vendors' MSAA investment (§2.1) | Correct | HP 1.10 (Medium, Wikipedia) | — |
| 43 | IA2 was an "open alternative to UIA" (§2.1) | Partly correct | HP 1.11 is **Low** (search snippets) | Flag it or drop it |
| 44 | "Little change since IA2 1.3 (2011)" (§2.1) | Partly correct | HP 1.12: the 1.3 proposal page was last edited in 2011. PE's chronology and PA 2.13 date the 1.3 interfaces 2013 | "(IA2 1.3: drafted 2010–11, published 2013)" |
| 45 | IA2 is still NVDA's default for web content in 2026 (§2.1) | Correct | NVDA 2026.2 guide: the default is "Only when necessary"; UIA in Chromium "may not provide the same level of access as IA2" | — |
| 46 | Chrome first proxied queries, which "blocked the UI and content processes until each query finished" (§2.1) | Partly correct | Teh: "the entire UI and the web content process were blocked until each a[ccessibility query …]". The quoted text is the HP note's paraphrase | Quote Teh's words, or unquote |
| 47 | Chrome then moved to a push model: renderers serialize, the browser caches and answers (§2.1) | Correct | Chromium `overview.md` (HP 2.4); Teh | — |
| 48 | Android: pull over Binder; virtual hierarchies at API 16 (§2.1) | Correct | HP 1.24 (Medium, recollection; already flagged in 11.2) | — |
| 49 | Android prefetch is capped at 50 nodes per request (§2.1; 6.10) | Correct | PA 7.3: `MAX_NUMBER_OF_PREFETCHED_NODES` = 50 (Android reference) | — |
| 50 | Edge Legacy at launch: "no 3rd party assistive technologies were able to provide any meaningful access" (§2.1) | Correct | PA 2.9, NV Access post (link resolves, 200) | — |
| 51 | Edge Legacy two years later: browse mode "significantly slower", no live regions (§2.1; §5) | Correct | PA 2.9 | — |
| 52 | Firefox 57: minimal parent cache, other queries relayed synchronously, COM handler with partial cache (§2.1; §5) | Correct | Teh: COM client-side handler; "The entire cache for all nodes was invalidated whenever anything changed" | — |
| 53 | Firefox 57's reason: shipping was urgent, and a full cache risked wrong information (§2.1) | Correct | HP 2.15, Teh quote | — |
| 54 | COM "was never designed to handle the massive number of objects" (§2.1) | Correct | Teh, verbatim | — |
| 55 | AOM: reflection and ElementInternals shipped; AT events and virtual nodes abandoned; the computed tree became test-only (§2.1) | Correct | AOM explainer: "highly unlikely to land in a format similar to the API proposed … may be approached initially as a testing-only API" | — |
| 56 | WebKit isolated tree made unconditional on macOS in January 2026 (§2.1; §3) | Correct | HP 2.25 (bug 305266: "is, always will be, true for macOS") | — |
| 57 | WebKit isolated tree: "Why: Main-thread cost (public reason: 'performance')" (§2.1) | Partly correct | HP 2.26 is **Low**: main-thread cost as the motive is the researcher's inference | "Performance (stated); main-thread hangs inferred" |
| 58 | CtW's motive included "19,000 lines of Windows-only code" (§2.1) | Correct | CtW wiki: "the IPC layer on Windows (~8000 lines)" plus "especially on Windows (where we depend on a whole separate ~11000 line module)" | Optional: "about 19,000 lines, mostly Windows-specific" |
| 59 | CtW 2–3× faster on everyday tasks (§2.1) | Correct | Mozilla blog (HP 2.18); Teh's Gmail and Slack figures (1.9–2.7×) | — |
| 60 | Synchronous IPC was deadlock-prone (§2.1; §5) | Correct | CtW wiki: "there is a high risk of deadlocks" | — |
| 61 | AccessKit (2021): push schema, adapters for five platforms (§2.1) | Correct | HP 4.1, 4.3; PA 9.4 | — |
| 62 | AccessKit's README says the adapters "don't yet support rich text or hypertext" (§2.1; §3; AX-D1) | Correct | README: "They don't yet support rich text or hypertext." | — |
| 63 | Newton is a prototype only (June 2024), with no 2025–26 status found (§2.1) | Correct | PA 6.6, 6.8 | — |
| 64 | Native UIA on by default in Chrome 138 (§2.1; §3) | Correct | Chrome blog, 14 August 2025: "From Chrome 138, Chromium-based browsers on Windows enable native UI Automation (UIA) support by default" | — |
| 65 | "MSAA and IA2 clients still connect directly" (§2.1) | Partly correct | Blog: "assistive technologies that use Microsoft Active Accessibility (MSAA) or IAccessible2 (IA2) connect directly to Chromium … and that's not changing". The quote is a paraphrase | Quote the blog's words |
| 66 | `ariaNotify` is in Chrome 141, Firefox 150 and Safari 27 (§2.1) | Correct | MDN BCD `api.Element.ariaNotify`: chrome 141, firefox 150, safari 27 | — |
| 67 | `ariaNotify` is designed to avoid timing differences that reveal a screen reader (§2.1; 6.7) | Correct | Explainer: "avoid optimizing-out the synchronous aspects … timing differences … (a screen reader is connected)" | — |
| 68 | WPT `aamtest` merged 27 March 2026; macOS not yet in CI (§2.1; §3) | Correct | wpt rfcs#204 "Merged … Mar 27, 2026"; SS 9.5 | — |
| 69 | Sandboxing (2008) and site isolation (2018) moved content out of the injectable process (§2.2) | Correct | HP 2.1–2.2, 6.2 (Teh) | — |
| 70 | Across processes, answering costs 12–21× (§2.2; 4.4; 6.0) | Correct | 175 ÷ 15 = 11.7; 128 ÷ 6 = 21.3 | — |
| 71 | Nine obstacles to async OS APIs; Teh is "skeptical about the return on investment" (§2.2) | Correct | HP 3.2, PE 1.16, note only | — |
| 72 | Async OS APIs: "Hope since 2010" (§2.2) | Unverifiable | No note row gives 2010 | Drop the year or cite a source |
| 73 | Linux AT detection paused in 2020 for false positives (§2.2; §5) | Correct | HP 2.7, PE 5.8 (orca-list, 26 May 2020) | — |
| 74 | AT-SPI 3 plans to remove screen coordinates (§2.2) | Correct | PA 5.11 (AT-SPI 3 roadmap) | — |
| 75 | MathML-AAM cells are "TBD" for MSAA, IA2 and UIA (§2.2) | Correct | SS 1.7 | — |
| 76 | Twenty points are open in the specifications (§2.2; 6.2) | Correct | SS 11 has 20 rows | — |
| 77 | CSS Containment 2 forbids main-thread accessibility work for skipped content (§2.2) | Correct | Contain 2 §4.6 (`#cv-a11y` is numbered 4.6): if the UA skips layout and paint for off-screen `auto` content, "it must similarly avoid doing that work" for the accessibility tree | Optional: "forbids extra work" (the spec says nothing about threads) |
| 78 | "Every multi-process engine has converged on renderer push plus a parent-process cache" (§3) | Partly correct | True of Chrome and Firefox. WebKit's isolated tree is threading inside one process, not a cross-process cache (HP 2.25); HP §2's own implication says "Two of three engines" | "Chrome and Firefox have converged …" |
| 79 | Firefox answers every query from its cache since Firefox 113 (§3) | Correct | 112 on Windows and Linux, 113 on macOS completed the rollout (HP 2.18) | — |
| 80 | Accessibility is an Interop 2026 investigation at 66.1% (§3) | Correct | interop-accessibility#202, October 2026 update: "= 66.1%" | — |
| 81 | ARIA-AT has "21 recommended test plans" (§3) | Partly correct | ARIA-AT CG minutes, 1 July 2026: "3 at draft, 3 at candidate review, and 21 at recommended", under "Current interop reporting status". SS 9.7 (ARIA-AT GraphQL API, 19 October 2026) counts 45 plans, **3 Recommended**, 23 Candidate. The notes conflict and the spec does not say so | Reword to "21 recommended reports (July 2026)" or similar, and record the conflict in 11.2 |
| 82 | ARIA-AT's AT Driver has one editor (§3) | Correct | GO 6.2; SS 1.11 (editor Mike Pennisi); minutes: "We need some help in this space" | — |
| 83 | 70.9% of screen-reader+browser pairs are JAWS or NVDA with Chrome or Edge (§3) | Correct | Survey #11: 31.2 + 17.3 + 17.0 + 5.4 = 70.9 | — |
| 84 | Users' main complaints "unchanged in 14 years" (§3) | Correct | UX 1.14 (Survey #10 wording), note only | — |
| 85 | Mozilla's accessibility component has 36 public security bugs filed in 2026 (§3) | Correct | Bugzilla REST, filed since 2026-01-01: sec-high 27, sec-moderate 6, sec-low 3 | — |
| 86 | … "against 0–7 a year before" (§3) | Wrong | Same query, by year filed (deduplicated): 2018 6, 2019 2, 2020 2, 2021 2, **2022 8, 2023 9**, 2024 1, 2025 3. The note SR 4.6 has the same error | "against 1–9 a year in 2018–2025"; correct SR 4.6 too |
| 87 | The parent-process tree code is C++ in both engines (§3) | Correct | SR 1.4, 1.11 | — |
| 88 | No tool traces AccName steps; Chrome lists ranked candidate sources only (§3) | Correct | GO 3.4 (CDP `AXValueSource`), GO 4.1 (absence, Medium) | — |
| 89 | Firefox's inspector shows "Gecko's accessibility tree, rather than information from the platform accessibility layer" (§3) | Correct | Firefox docs, verbatim | — |
| 90 | CDP has two accessibility events (§3) | Correct | GO 3.2 (`browser_protocol.json`) | — |
| 91 | "Once an announcement is made, it disappears forever" (§3) | Correct | Soueidan, verbatim | — |
| 92 | Firefox warns its accessibility engine perturbs performance metrics (§3) | Correct | GO 2.6 | — |
| 93 | Focus history: a live `document.activeElement` expression in Chrome, a static tab-order snapshot in Firefox (§3) | Correct | GO 2.5, 2.11 | — |
| 94 | State of HTML 2025: more respondents "test with" Chrome DevTools (1,306) than NVDA (742) (§3) | Correct | stateofhtml.com 2025: "Chrome DevTools … 1,306 … NVDA 742" | — |
| 95 | Full snapshots cost 8k–50k tokens on ordinary pages; the HTML spec about 2.9 million (§3; 6.14) | Correct | PE 7.4: 7.9k–49k and 11.66 M chars ÷ 4. These are the researcher's own measurements, estimated by characters ÷ 4 | Optional: say "estimated" |
| 96 | Checkers find 13–57% of issues depending on how one counts (§3) | Correct | GO 5.1–5.4 (axe README 57%; GDS 13–40%) | — |
| 97 | Live regions fail on timing and registration (§5; 6.7) | Correct | TetraLogical: "Screen readers need to register for these events"; pre-populated containers | — |
| 98 | Chrome's bounds-only fast path gave up to 825% on scroll tests (§4.4; §5) | Correct | Blog: "up to 825% in scrolling tests" | — |
| 99 | ~7% of Android users run Chrome's accessibility code (§5; 11.2) | Correct | `android.md`: "roughly 7% of all Android users" | — |
| 100 | §5 row "Chrome and Firefox, 2026": "31 Chrome accessibility CVEs, 14 needing a compromised renderer" | Wrong | NVD: 31 and 14 are the totals for 2020–2026. In 2026 the counts are 16 and 12. The row is labelled 2026, and it contradicts the intro and §3 | "16 Chrome accessibility CVEs in 2026 (31 since 2020), 12 of the 2026 ones needing a compromised renderer" |
| 101 | Firefox 2026 re-entrancy: script ran during an AT action and freed what accessibility code held (§5; 6.9) | Correct | Bugzilla 2029743: "… after TakeFocus runs blur handler" | — |
| 102 | Firefox 2026 derived-table bugs: cached tables went stale after `aria-owns` moves (§5) | Correct | Bugzilla 2027288: "CachedTableCellAccessible::Selected when table row is relocated via aria-owns" | — |
| 103 | CVE-2017-5464: the DOM and accessibility trees fell out of sync (§5; 6.7) | Correct | SR 4.10 (NVD text), note only | — |
| 104 | WebKit fallbacks to a busy main thread made VoiceOver unresponsive (§5) | Correct | PE 1.19 (WebKit bug 243456: "irresponsive") | — |
| 105 | Newton: push, frame-synchronized, surface-relative coordinates, hit regions client-side (§5) | Correct | PA 6.1–6.4 | — |
| 106 | Flutter web keeps accessibility off until a hidden button is pressed (§5) | Correct | HP 4.9, GO 10.3 | — |
| 107 | accessiBe: FTC order in 2025 for claiming compliance (§5) | Correct | GO 8.1: FTC final order, 22 April 2025, $1M | — |
| 108 | WebAIM Million: 95.9% of home pages fail; pages with ARIA average 59.1 errors against 42 (§5) | Correct | WebAIM Million 2026, verbatim | — |
| 109 | Geometry in 1/64 CSS px fixed point (LY-D3's `LayoutUnit`) (6.1) | Correct | Layout spec: "32-bit fixed point at 1/64 CSS px" (LY-D3) | — |
| 110 | `aria-hidden="true"` on the root element is not honoured (ARIA 1.3) (6.1) | Correct | SS 2.13 | — |
| 111 | `drawFocusIfNeeded()` sets the fallback element's bounds (6.1) | Partly correct | GO 10.8 is **Low** (recollection, HTML not re-read) | Check the HTML canvas section, or flag it |
| 112 | HTML-AAM has 146 element entries and Core-AAM 97 role entries (6.2) | Correct | SS 4.1, 5.1 (the researcher's parse, High), note only | — |
| 113 | The generated tables may ship under the W3C Software and Document License notice (6.2; §7) | Correct | SS 10.1 | — |
| 114 | AccName sets no depth or size bound (6.2) | Correct | SR 8.1 | — |
| 115 | Firefox's cache missed Gmail name changes (6.2) | Correct | CtW wiki: "name change events for selection changes in Gmail message lists" | — |
| 116 | AX-O8: the ARIA fallback formula is the half-range, not the midpoint (6.2) | Correct | SS 2.12 (text High; the intended midpoint is the researcher's reading) | — |
| 117 | AX-O16: 67 + 19 TBD cells; AX-O20: WPT 2–6 of 10 (6.2) | Correct | SS 1.7, 5.1, 8.8 | — |
| 118 | Chrome's full mode costs up to 3.6× the main-thread cost of building a large DOM, in a black-box test (6.3; 4.4 "tripled") | Unverifiable | PE 8.3 is the researcher's own run (not reproduced). The stated conditions match this machine (Apple M6, 32 GB, Chrome 155.0.8059.80 installed), and 1,204 ÷ 330 = 3.65 | Already flagged in 11.2 |
| 119 | Chrome's tree lags the renderer "by a fraction of a second" (6.3) | Correct | `overview.md`: "the cache may lag what's in the renderer process by a fraction of a second" | — |
| 120 | Auto-disable is Android-only in Chrome (6.4) | Correct | `android.md`: "This is not done on any of the other platforms" | — |
| 121 | ≤ 2²² nodes is about 4.2 million; the HTML spec has 0.89 million (6.5) | Correct | 2²² = 4,194,304; PE 8.1: 887,197 accessibility nodes (own measurement) | — |
| 122 | Chrome: every frame its own tree with an unguessable id; one path for in-process and remote frames (4.4; 6.6) | Correct | PE 2.1–2.2 (`how_a11y_works_3.md`), note only | — |
| 123 | Android: scroll events at most every 100 ms; more than 5 content-changed events collapse to one (6.7) | Correct | `android.md`: "scroll events can only be sent at most once every 100ms"; PE 3.10 | — |
| 124 | Core-AAM §4.1.1 gives the `ariaNotify` mapping (UIA notification, AT-SPI announcement, AX) (6.7) | Correct | Core-AAM ED: "4.1.1 ariaNotify"; UIA row calls `UiaRaiseNotificationEvent` | — |
| 125 | The explainer's spoofing mitigation: suppress content announcements while focus is outside web content (6.7) | Correct | Explainer: "Mitigations should be applied to suppress notifications when focus moves outside of the web content" | — |
| 126 | Embedded objects count one U+FFFC each in IA2 and AT-SPI hypertext (6.8) | Unverifiable | PE 6.2 is **Low**, PA 2.13 Medium (via Qt), PA 5.8 Medium. The IA2 general-info and `IAccessibleHypertext` pages fetched today do not state it | Verify in the IA2 `IAccessibleText` reference, or flag it |
| 127 | IA2 and macOS count UTF-16 units; AT-SPI and Android count characters (6.8) | Unverifiable | No note row states the IA2 or macOS unit; PA 5.8 says only that AT-SPI uses "character offsets" | Cite a source or add it to Q-AX-5 / AX-8 |
| 128 | Core-AAM §3.7: the default action SHOULD "simulate a click" (6.9) | Correct | Core-AAM ED, `#mapping_actions` = "3.7 Actions": "the user agent SHOULD simulate a click on the DOM element" | — |
| 129 | Chrome reports AT actions as accepted asynchronously (6.9) | Correct | PE 4.10 (`how_a11y_works_3.md`), note only | — |
| 130 | `ProviderOptions_UseComThreading`: MTA providers are called on another MTA thread, off the UI thread (6.10) | Correct | Microsoft Learn: `= 0x20`; "MTA-based provider implementations will be called back on some other thread in the MTA" | — |
| 131 | `IsControlElement` and `IsContentElement` are "set per Core-AAM ([HP 2.23])" (6.10) | Partly correct | HP 2.23 cites Chromium's `uiautomation.md` (prose), not Core-AAM | Cite HTML-AAM/Core-AAM rows (SS 4.9) or reword |
| 132 | AppKit services AX requests on the main thread (6.10) | Partly correct | PA 4.5 is **Medium** (recollection; "No AppKit header or page read this session states it"). The spec states it as fact | Add main-thread confirmation to Q-AX-5 (PA open 5 asks for it) |
| 133 | Every crate licence in §7 (`windows` family MIT OR Apache-2.0; `objc2`, `objc2-foundation`, `block2` MIT; `objc2-app-kit`, `-application-services`, `-core-foundation`, `dispatch2` Zlib OR Apache-2.0 OR MIT; `zbus`, `zvariant` MIT; `jni`, `jni-sys`, `ndk` MIT OR Apache-2.0) | Correct | crates.io API today: windows 0.62.2, windows-core 0.100.0, windows-sys 0.61.2; objc2 0.6.5, objc2-foundation 0.3.2, block2 0.6.2; objc2-app-kit, -application-services, -core-foundation 0.3.2; dispatch2 0.3.1; zbus 6.0.0, zvariant 5.15.1; jni 0.22.4, jni-sys 0.4.1, ndk 0.9.0. All licences match | — |
| 134 | `accesskit` 0.25 is MIT OR Apache-2.0 with Chromium-derived parts; `atspi` Apache-2.0 OR MIT; `core-foundation` maintained in the Servo organisation (§7) | Correct | crates.io: accesskit 0.25.1, atspi 0.30.0, core-foundation repository `github.com/servo/core-foundation-rs`; README: "based largely on Chromium's cross-platform accessibility abstraction" | — |
| 135 | ATK is LGPL (§7) | Unverifiable | No note row states ATK's licence (well known as LGPL, but not checked here) | Cite it, or drop the word |
| 136 | CtW took about 18 months from proof of concept to all platforms (9.3) | Correct | December 2021 to 9 May 2023 is about 17 months (HP 2.18) | — |
| 137 | Chrome's native UIA took releases 126 to 138 (9.3) | Correct | Chrome blogs: "gradual rollout … starting in Chrome version 126"; on by default from 138 | — |
| 138 | Totals: 28–36† engineers (desktop), 31–39† with Android (9.3) | Correct | Track rows sum to 28–36; adding Android's 3 gives 31–39 | — |
| 139 | "about 15 in year one, 30 in year two, then 28–36 a year to Stage 1 exit [48–60 months]: about 120–175 engineer-years†" (9.3) | Wrong | 15 + 30 + (28 × 2 years) = 101 at 48 months; 15 + 30 + (36 × 3 years) = 153 at 60 months. The stated range is about 100–155 | Correct to about 100–155, or change the ramp |
| 140 | Stage 1 exits at 48–60 months; Stage 1 8.3 had "Accessibility 1–2, plus contractor" sized for AccessKit integration (9.3) | Correct | reconciliation_2.md result paragraph; Stage 1 design 8.3 row "Semantic plane, AccessKit integration … 1–2, plus contractor" | — |

Section 11.2's own flags were also checked. Each matches its note and the primary source where one was fetched: the CtW figures are Mozilla's; Chrome gives "5–10%" and "7%" without a method; the Chrome 155 runs are three runs on one machine; the CVE and Bugzilla counts are lower bounds (re-run today with the same results); WebAIM 2014 gives 54.0% + 24.4% = 78.4%; Edge Legacy rests on NV Access's 2017 post; Survey #11 gives JAWS 55.0% against #10's 40.5%. These rows are not counted separately above.

## Must-fix

1. **§5, row "Chrome and Firefox, 2026"** (claim 100): replace "31 Chrome accessibility CVEs, 14 needing a compromised renderer" with "16 Chrome accessibility CVEs in 2026 (31 since 2020), 12 of the 2026 ones needing a compromised renderer". As written, the row contradicts the intro and Section 3.
2. **§3, "against 0–7 a year before"** (claim 86): Bugzilla gives 1–9 a year for 2018–2025 (2022: 8, 2023: 9). Correct the spec, and correct SR 4.6, which carries the same error.
3. **§9.3, engineer-years** (claim 139): the stated ramp gives about 101–153 engineer-years to AX-M3 at 48–60 months, not 120–175. Fix the range or the ramp.
4. **§6.5, validator check 8**: "total mirror bytes per renderer within a budget set from AX-31". AX-31 is the recording-cost target; the memory target is AX-16.
5. **Intro, Firefox 57 quote** (claim 2): Teh wrote "The performance with assistive technologies took a massive step backwards". Quote it exactly.
6. **§2.1, Chrome 2008 row quote** (claim 46): "blocked the UI and content processes until each query finished" is not Teh's wording ("the entire UI and the web content process were blocked until each a[ccessibility query …]"). Quote him exactly, or remove the quotation marks.
7. **"About 20,000 lines removed"** (intro, §2.1, §5; claim 6): Teh's figure is a projection ("will be able to remove around 20,000 lines"). Say "expected to remove".
8. **§3, "Every multi-process engine has converged"** (claim 78): WebKit's isolated tree is in-process threading, not a parent-process cache (HP 2.25). Say "Chrome and Firefox".
9. **§3, ARIA-AT "21 recommended test plans"** (claim 81): GO 6.1 (July 2026 minutes, "interop reporting status") and SS 9.7 (API, October 2026: 3 Recommended of 45 plans) conflict. Reword to what the minutes count, and record the conflict in 11.2.

## Recommended

1. §1 and §2.2: Chrome's "5–10%" is "of users", "often" because of password managers and antivirus, not "mostly", and not specifically "desktop" (claim 28).
2. §1: flag the 37% (Android) and 34% (iOS) text-size figures as second-hand Appt data (UX 1.21 Medium) in 11.2 (claim 21).
3. §1: remove the quotation marks from "barely measured"; it is the note author's phrase (claim 25).
4. §2.1: flag or drop NSAccessibility "10.2" (HP 1.20 Low) (claim 36).
5. §2.1: flag or drop "open alternative to UIA" (HP 1.11 Low) (claim 43).
6. §2.1: IA2 1.3 was published in 2013 (drafted 2010–11) (claim 44).
7. §2.1: WebKit's "Why" column: "performance" is the stated reason; main-thread hangs are inferred (HP 2.26 Low) (claim 57).
8. §2.1: quote the Chrome UIA blog's words ("assistive technologies that use MSAA or IA2 connect directly to Chromium … that's not changing") (claim 65).
9. §2.1: AT-SPI "predates sandboxing" is a paraphrase of Campbell; unquote it (claim 35).
10. §2.2: drop "since 2010" or cite it (claim 72).
11. §4.4 "its maintainers will not move (November 2025)": the maintainer's statement is conditional on UIA's quality ("as overall UIA has a worse implementation … in Browse mode"). Prefer "will not move until Chromium's UIA matches IA2 in browse mode".
12. §6.1: verify `drawFocusIfNeeded()` against the HTML canvas section (GO 10.8 Low) (claim 111).
13. §6.8: verify the U+FFFC rule and the per-platform offset units (UTF-16 for IA2 and macOS), or make them AX-M0 observations (claims 126–127).
14. §6.10: AppKit's main-thread servicing is Medium (recollection); add it to Q-AX-5 (claim 132).
15. §6.10: re-cite `IsControlElement`/`IsContentElement` to HTML-AAM or Core-AAM text rather than HP 2.23, which is Chromium's prose (claim 131).
16. §6.19: "Firefox's prose (… cache domains)" rests on PE 5.12, which is **Low** (recollection). Flag it, or cite a Firefox prose page.
17. §6.13: Chrome's black-box figure is about 2.2 **KB** (2,198 bytes) per node, which is 2.15 KiB. The rounding is harmless, but match the unit to the note.
18. §7: give a source for ATK's LGPL licence, or drop it (claim 135).
19. §9.1 and §9.3: the Android adapter ships at AX-M4, but its staffing row says "from AX-M3". Say whether this is a ramp-up before delivery.

## Link check

- **Spec, external links (13 URLs, 2 with fragments).** All resolve with HTTP 200: the Teh post, NVDA user guide, nvaccess/nvda#19276, Chrome accessibility-performance blog, NVD API query, AOM explainer, W3C Design Principles, ariaNotify explainer, Microsoft Learn ProviderOptions, Core-AAM ED, CSS Contain 2 and Media Queries 5. Both fragments exist: `css-contain-2/#cv-a11y` (section "4.6") and `core-aam/#mapping_actions` (section "3.7 Actions"). One weakness: the link `https://w3c.github.io/core-aam/` for "Core-AAM §4.1.1" has no fragment. `#arianotifymixin-map-arianotify` exists and could be used.
- **Spec, link reference definitions (22).** All resolve to existing files: HP, SS, PA, PE, SR, UX, GO, FR, H2, H3, S1, JS, FP, ST, TX, LY, GR, MD, DT, UI, LG and RN2.
- **Spec, internal `[XX n.m]` citations.** A script parsed every citation to HP, SS, PA, PE, SR, UX, GO and FR, including ranges, section-only citations such as `[GO 11]` and `[UX 6, 7, 8]`, and "open n" citations. Every one names a row, section or open question that exists. `[PA M-1 note]` points to PA's M-1 paragraph, which exists. `[ST 6.10]` points to a section of the style spec. Two rows that exist but support the claim only partly are listed above (claims 21 and 131).
- **Notes, sample.** Of 390 unique URLs, a sample of 48 (every eighth, API endpoints excluded) plus 12 fetched while checking claims were requested.
  - All returned 200 except 3:
    - `hacks.mozilla.org/2024/05/…local-alt-text…` (GO 9.3) returned 403.
    - `blog.mozilla.org/accessibility/firefox-113-accessibility-performance/` (HP 2.13, 2.18; PE 1.12) returned 403.
    - The FTC press release of 3 January 2025 (GO 8.2) returned 404.
  - The two Mozilla 403s look like blocks on non-browser clients. The FTC URL should be checked in a browser and replaced if it is dead.
  - The sample also confirmed the NV Access "Edge 2 years in" post, both Chrome UIA posts, the WebAIM surveys and Million, the CtW wiki, the ARIA-AT minutes, interop-accessibility#202, rfcs#204, State of HTML 2025, and the Firefox inspector docs.

## M-1 engine-source check

- **Spec.** No link points to engine or AccessKit source code. The 13 external URLs are specifications, vendor blogs and documentation, an NVD API query, a screen-reader user guide and a screen-reader issue. No value in Sections 1–3, 5, 6, 7, 9.3 or 11.2 traces to engine source. Every number traces to prose, standards, NVD, Bugzilla metadata, crates.io, Apple's public SDK headers, or the PE note's black-box measurements of stock Chrome.
- **Notes.** A scan of all 390 note URLs for source hosts and code paths found no code file:
  - **Source hosts and code paths scanned for:** `googlesource`, `searchfox`, `source.chromium.org`, `hg.mozilla.org`, `trac.webkit`, code extensions, `/blob/` and `/tree/` paths, commits and code-review hosts.
  - **What the hits were:** the `chromium.googlesource.com` links are all `docs/**.md` prose (allowed under OD-7, as the spec says). The `/blob/` links are READMEs, explainers, LICENSE files, CSSWG explainers, the Interop README, NVDA's `nvdaHelper/readme.md`, and the CDP `browser_protocol.json`. That file is a published protocol interface definition, used under the same reading as DT-D11. `trac.webkit.org/wiki/…` is a wiki page.
- **Borderline items, none blocking:**
  1. HP 2.24 and PE 1.18 summarise the "review discussion" on WebKit bug 193348. Bugzilla review comments on an attached patch are close to a code review. Both notes say the patches were not opened, and no value or structure in the spec derives from those comments; the spec cites only the dates and the in-process reading. Rephrase the two rows as "bug discussion", or drop the review detail.
  2. The page for nvaccess/nvda#19276 embeds a snippet of NVDA's `UIAHandler` source. NVDA is a screen reader, not a browser engine, and the spec uses only the maintainer's prose comment.
  3. Mozilla bug titles quoted in SR 4.7–4.9 contain Gecko class and method names (for example `DocAccessibleParent::Unbind`). Titles are metadata; only the bug shapes are used (6.5), not any code.
- **Confirmation.** No research note or spec section cites another engine's or AccessKit's source code. Section 7's evidence rule holds.
