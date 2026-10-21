# Izcal editing: fact-check

Checked 20 October 2026 by an independent fact-checker (phase 8 of the first-principles-spec method), who did not write the spec or the notes.
Subject: `reports/Izcal editing.md` (draft 0.1) and the seven notes in `research_notes/Izcal editing/`.

**Method.** Every number, date, quote, status and causal claim in Sections 1–3, 5, 6, 7, 9.3 and 11.2 was traced to the note row it cites, and the load-bearing ones to the primary source. No web search was used; known primary pages were fetched directly. They were:
- **Specifications:** Input Events Level 2 TR (WD 2026-10-15); Clipboard API TR (WD 2026-06-24); Selection API TR (WD 2026-06-11); EditContext TR (WD 2026-09-10); the execCommand draft; HTML `interaction.html` and `dnd.html`; UI Events ED; CSS Pseudo-Elements 4 ED; CSS UI 4 ED; `text-input-unstable-v3.xml` at wayland-protocols tags 1.47 and 1.48 (protocol definitions); wayland.app's compositor table.
- **Vendor and OS documentation:** Microsoft Learn's InputScope page; Apple TN2150; MDN's Clipboard API page; web.dev's Async Clipboard article; the Chrome mutation-events post; the VS Code 1.101 release notes; Google Workspace Updates (Docs canvas); Mozilla Hacks (RLBox, Firefox 95); the Firefox IME handling guide (prose); the Chromium "System Dictionary Pop-Up Architecture" design page.
- **Security reports:** Project Zero's CTF post; Microsoft Threat Intelligence's ClickFix post; ESET's H1 2025 report page; Proofpoint's ClickFix brief; both Citizen Lab reports; BleepingComputer's spell-jacking article.
- **Issue trackers (prose and metadata only):** w3c/editing #480, #150 and #551; mozilla/standards-positions #199 (labels and timeline); WebKit/standards-positions #243; w3c/webdriver #1683; microsoft/playwright #5777; ProseMirror discuss topic 5246; and the Bugzilla REST API. Through Bugzilla the checker re-counted the security bugs in Core › DOM: Editor, DOM: Selection, DOM: Copy & Paste and Drag & Drop, and Spelling checker, and read bugs 202251 and 298127.
- **Maintainers' prose:** Haverbeke's ProseMirror post; QuirksBlog; the CKEditor 5 architecture post; the Lexical FAQ; the `@codemirror/view` CHANGELOG (release notes); the SymSpell and Harper READMEs; Norvig's "How to Write a Spelling Corrector"; Fatin's "Typing with pleasure".
- **Data:** LanguageTool's language table; MDN browser-compat-data 8.1.6 (timestamp 2026-10-15); the AMO language-tools API (dictionary count); crates.io (zbus).
- **Izcal documents:** the internal cross-references to the DM, TX, GR, AX and ledger documents were read in those files.

Rows marked "note only" were checked against a note row graded High without fetching the primary source again. WPT percentages were checked against the note, not recomputed. No source code of any project was opened.

**Counts.** 201 claims checked: **159 correct**, **32 partly correct**, **4 wrong**, **6 unverifiable**. Of the 32 partly correct:
- 6 are quotation problems: an altered quote, a quote taken out of its context, or a quote attributed to the wrong party.
- 8 present a Low- or Medium-confidence note row, or a researcher's inference, as fact.
- 18 are imprecise wording, scope or citation.

There are **8 must-fix items** and **21 recommended items**, plus **8 fixes to the notes**.

## Must-fix

1. **Intro paragraph: misquoted ledger.** The spec says the ledger records C-22 as "no owner, stage or budget". That phrase is not in `research_notes/Phase 3 amendments/ledger.md` or in any other report or note. The ledger's C-22 row says "**Unresolved, minor.** Recommendation: keep TX's placement and record the stage and budget as the editing spec's (G-16)", and TX's placement is "spellcheck in a utility process". Quote the row, or paraphrase it without quotation marks. Separately, the spec's own design puts spelling in the renderer (ED-D7), which departs from the ledger's recommendation. Say so, so the design critic and the owner see it (claim 1).
2. **ClickFix: "no browser does anything about it" (intro), "browsers show nothing" (§5, ClickFix row), "defences are all on the OS side" (§2.1, ClickFix row), "Microsoft's ClickFix guidance offers only OS-side mitigations and user awareness" (§6.10).** These claims are contradicted by the very Microsoft post the spec cites. That post says "Microsoft Defender SmartScreen displays a warning to Microsoft Edge users when they visit a ClickFix landing page", and it recommends "enterprise-managed browsers". Note SR 2.6 (no browser-side mitigation) is graded Low. What the sources support is narrower: no browser is documented to warn about **what a page writes to the clipboard**. Rescope all four sentences to that claim (claims 14, 69, 93, 166).
3. **§6.8 "Where it runs": misattributed reason.** "The reason other browsers moved their spellchecker into a sandbox was memory-unsafe C++ parsing complex dictionary files, which had memory bugs exactly in its affix and suggestion paths". Mozilla's post gives only general reasons: it treats the five modules as untrusted code so that "even a zero-day vulnerability in any of them should pose no threat", and attackers chain two vulnerabilities. SR 6.3 states this ("The post gives no Hunspell-specific reason"). Only one browser (Firefox) is documented, and its RLBox sandbox is **in-process**, not a separate process. Keep the Hunspell bug record as **evidence** of risk (SR 6.1–6.2), but do not present it as Mozilla's stated reason (claim 147).
4. **§6.8 "Engine": Norvig misattributed.** The spec says "Norvig's own figures show that the error model, not candidate generation, limits accuracy". The figures are right (75% on 270 words, 68% on 400), but Norvig does not draw that conclusion. He names four areas for improvement:
   - the language model, where "the more serious" source of error is unknown words (5% and 11%);
   - the error model, which "has been trivial";
   - the candidate set;
   - context, which he calls "a fourth (and best) way to improve".

   Rephrase as "Norvig's analysis of his 68–75% result names a trivial error model, unknown words and lack of context" (claim 146).
5. **§6.8 ED-D8: "Firefox offers 175 dictionaries in 114 locales".** This was not reproduced. On 20 October 2026, the AMO language-tools API (`type=dictionary`) returns **207 dictionaries for 170 target locales (130 language subtags)**. SL 3.4 is a Medium own count from the rendered page. Replace the figure with "about 200 dictionaries in about 170 locales (AMO API, 2026-10-20)". The argument still holds, and is stronger (claim 150).
6. **§6.8 "Grammar": LanguageTool range.** The spec says "from 892 rules for Russian to 8,251 for Catalan". Both numbers are correct, but LanguageTool's table lists **31 languages, from 32 XML rules (Swedish) to 8,251 (Catalan)**. 14 of the 31 have fewer than 500 rules (for example Danish 78, Greek 55, Italian 141). The 892–8,251 range covers only the nine languages the note copied, so it overstates the scale. Say "8,251 for Catalan, 6,074 for English and 892 for Russian; most of the other 31 languages have under 500" (claim 152; SL 3.6 should list the full table).
7. **Intro paragraph: "says the browser's event stream 'is hardly reliable'".** The quote comes from one editor, Lexical, about `input` and `beforeinput`. The FAQ reads "these are hardly reliable", and the spec attributes it to "every serious editor". Write "and Lexical's FAQ calls `input` and `beforeinput` 'hardly reliable'" (claim 9).
8. **§6.10 "Async reads": the 1 s delay attributed to the specification.** The spec says "enabled after 1 s, which is the rule the Clipboard API now specifies and Firefox and Safari ship". The Clipboard API WD (§9.1.1) requires transient activation **and** "user interaction with a 'Paste' element created by the user agent or operating system". It says nothing about a delay. The 1 s enable delay is Firefox's and Safari's behaviour, as MDN describes it (SR 2.8). Separate the two: the spec requires the Paste element, and Firefox and Safari add the 1 s delay (claim 160).

No broken links and no source-code links were found in the spec (see the link check below).

## Recommended

1. **Intro paragraph.** "Two decades of attempts to fix `contenteditable`" should be "a decade": #480 says the draft was "written a decade ago", and the earliest attempt in the notes is Gregor's in 2011 (claim 5). "Android composition is where every editor's bug list concentrates" is stronger than GO 2.6's label counts support: CKEditor 5 has `domain:typing/ime` 202 and `browser:safari` 137 against `browser:android` 85. Say "where a large share of editors' IME bugs sit" (claim 11).
2. **"Thousands of devices a day" (intro; §2.1; §6.17).** Microsoft says campaigns are "targeting thousands of enterprise and end-user devices globally every day". That counts devices targeted, not devices compromised. Say "targets" (claim 13).
3. **§1 bullet on DM.** The quotation drops DM's "of 6.5" ("beyond the event plumbing of 6.5") without an ellipsis (claim 2).
4. **§2.1 Gregor row.** "Engines were 'not remotely consistent'" quotes a note about **unsupported commands** ("as of June 2011 UAs aren't remotely consistent on what they do with unsupported commands"). Quote it in that context, or paraphrase (claim 51).
5. **§2.1 Async Clipboard row.** web.dev's sentence reads "clipboard access was synchronous, and could only read and write to the DOM". The spec's quote changes the subject and drops the comma (claim 62).
6. **Low and Medium rows stated as fact without a flag.** Add each of the following to 11.2, or soften it:
   - IE 5.5's 2000 date (HP 1.1 Medium);
   - Midas as "the web's first permission model" (the researcher's interpretation);
   - "Firefox 2 … with Hunspell and MySpell dictionaries" (HP 9.5; the engine detail is recollection, and Firefox 2 is generally reported to have shipped MySpell, with Hunspell adopted later);
   - Google Docs' 2010 motives (HP 4.1 Low);
   - Gregor's spec "stalled after one editor" and "never a vendor conformance target" (HP 1.5 Low/Medium);
   - finite-state spellers handling Finnish, Sámi and Greenlandic morphology (SL 2.5 is **Low**, not just its performance figures);
   - the Hunspell, Nuspell and LanguageTool licences (SL 3.3 and 3.7 Medium).

   This covers claims 41, 44, 45, 50, 52, 86, 145 and 192.
7. **§4.4: "Mutation events are gone from every engine".** Only Chrome's removal in version 127 is sourced (SR 1.5). Say "Chrome removed them in 127, and the DOM deprecated them in 2011", or add sources for Gecko and WebKit (claim 83).
8. **§6.2: "Izcal follows Input Events, whose model the three engines converged on".** The specs note says the behaviour is "to be confirmed by measurement" (Q-ED-1), and HP 2.4 says Firefox follows Level 1. Write "which the current Level 2 text states and which Chromium and Gecko are reported to follow, pending Q-ED-1" (claim 97).
9. **§6.4, word movement.** "End of word on macOS, start of next word on Windows and Linux" has no note row. The usual Linux (GTK) convention is commonly described as end-of-word, so the Linux value is doubtful. Leave the per-OS values to the ED-M0 observation and do not state them in the text (claim 107).
10. **§6.5 "The constraint": "Every documented multi-process browser answers the synchronous queries from a cache the content process pushes".** Chromium's documented design (IM 4.1) blocks the UI thread with a 1.5 s timed wait, and §5 of the spec says so itself. Write "Mozilla answers from a pushed cache; Chromium's documented Mac design used a bounded timed wait" (claim 109).
11. **§6.5 step 3.** "Never immediately after reporting a layout change" comes from Mozilla bug 1187351, which is cited only in IM's implications, not in IM 4.5. Add a row to IM or cite the bug (claim 111). Also in §6.5, the "input sources restricted to Roman" detail comes from IM 5.4 (`allowedInputSourceLocales`) and IM's implications, not from IM 10.6 or TN2150. Cite IM 5.4 (claim 123).
12. **§3: "HTML has no word on undo".** The specs note counts three mentions (SS 4.10). Say "HTML says nothing about the undo history" (claim 78).
13. **§5 CTF row.** "No access control for twenty years" should be "nearly twenty years", which is Project Zero's wording (claim 89).
14. **§6.7.**
    - CSS UI 4 cites "WCAG Guideline 2.2 and Guideline 2.3", not success criterion 2.2.2 (claim 137).
    - The list of properties that apply to highlights omits `stroke-width` (SS 7.1; claim 138).
15. **§6.11.** "Policy bugs, not memory bugs" should be "mostly policy bugs". One of the 34 titles is a UAF, and SR 4.7 lists an overflow and a 2026 heap overflow in drag-image conversion (claim 171).
16. **§6.13.** `resizes-visual` is CSS Viewport's default (SS 10.4), not HTML's (claim 179).
17. **§6.16: "A W3C issue opened 2026-10-05 asks for exactly such a channel".** The body of w3c/editing #551 (now 16 comments) proposes that "the JS editors could agree on a common API for outside tools to hook into their editor". That is an editor-level API, not a browser-mediated edit channel. Say "asks for a sanctioned way for outside tools to edit; Izcal proposes its channel there" (claim 181).
18. **§6.10 Paste.** For "what Chrome and Firefox return in paste events", also cite HP 5.8 (Chrome); SS 6.6 covers only Firefox (claim 158).
19. **§6.17 copy-paste XSS row.** "Two engines and five editors" rests on SR 3.3, which is Medium (a search summary of an unreachable write-up). Flag it (claim 185).
20. **§7 and §9.3.**
    - `zbus` (MIT) is correct per crates.io (6.0.0, MIT) but has no note row. Add it to IM 12 (claim 190).
    - "LanguageTool's decade of rules" has no source in SL. Drop the duration or source it (claim 198).
21. **M-1 borderline items** (not blocking; see the M-1 section below): IM 4.1's 1.5 s value, and GO 5.2's `/tree/` link for Lexical DevTools.

**Fixes to the notes** (the spec is not affected unless stated):
- **HP 2.7 and HP 0:** Input Events Level 2's latest TR is **WD 2026-10-15**, not "2025-08". SS 2.1 and the spec are right.
- **SS 0 and SS 3.6:** Mozilla's "positive" label is dated 2024-12-09, and the concern labels 2024-11-15 and 2025-08-27 (from the GitHub timeline), not 2019. The spec already flags this.
- **SR 2.7:** the Clipboard API document of 2026-06-24 is a **WD**, not an Editor's Draft.
- **SL 4.3:** the quotation "too coarse" does not appear in the Mozilla Hacks post. The post speaks of "serious limitations" of process isolation.
- **SL 3.4:** replace the count (must-fix 5). **SL 3.6:** add the full LanguageTool table (must-fix 6).
- **SR 6.5:** the Passbolt incident link returns 404 (it redirects to `/incidents/`, which returns 404). The row is already Medium.
- **IM 1.3:** "every out-of-process browser … from a cache" should carry Chromium's timed wait in its main clause.

## Claims

"Note only" = checked against a High note row, without re-fetching the primary source.

| # | Claim (spec location) | Verdict | Evidence | Fix |
|---|---|---|---|---|
| 1 | Ledger records C-22 as "no owner, stage or budget" (intro) | **Wrong** | Phrase absent from the ledger and all notes; C-22 row: "Unresolved, minor … keep TX's placement" | Must-fix 1 |
| 2 | DM leaves "caret movement … visual selection" to this spec (intro) | Partly correct | DM text has "beyond the event plumbing of 6.5"; dropped without ellipsis | Rec. 3 |
| 3 | TX 6.8a leaves "the editing commands themselves" here (intro) | Correct | TX text | — |
| 4 | HTML delegates to an execCommand draft "not expected that it will advance beyond draft status" (intro; §4.4) | Correct | execCommand draft status text; SS 4.2 | — |
| 5 | "Two decades of attempts to fix contenteditable" (intro) | Partly correct | #480: "written a decade ago"; earliest attempt 2011 | Rec. 1 |
| 6 | Draft published and discontinued at once in 2025 (intro) | Correct | #480 CfC 2025-02-12, title "FPWD, and then immediately republish as a Discontinued Draft" | — |
| 7 | ProseMirror, Lexical, CKEditor 5, Docs keep own models (intro; §5) | Correct | Haverbeke post; HP 4.3–4.9 | — |
| 8 | CKEditor 5 about 42 person-years (intro; 2.1; 9.3) | Correct | CKEditor blog: "Estimated number of person-years … (to Sept 2018): 42" | — |
| 9 | Editors say the event stream "is hardly reliable" (intro) | Partly correct | Lexical FAQ only: "these are hardly reliable" (about `input`/`beforeinput`) | Must-fix 7 |
| 10 | Cancelable composition dropped because the IME owns its buffer (intro) | Correct | IE L2 WD 2026-10-15 table; HP 2.2–2.6 | — |
| 11 | Android composition is where every editor's bug list concentrates (intro) | Partly correct | GO 2.6 labels: IME 202, safari 137, android 85 (CKEditor 5) | Rec. 1 |
| 12 | 40 of Firefox's 74 Editor+Selection security bugs are UAFs (intro) | Correct | Bugzilla REST re-count: Editor 52 (26 UAF), Selection 22 (14 UAF) | — |
| 13 | ClickFix on "thousands" of devices a day (intro) | Partly correct | Microsoft: "campaigns targeting thousands of … devices globally every day" | Rec. 2 |
| 14 | "No browser does anything about it" (intro) | **Wrong** | Same Microsoft post: SmartScreen warns Edge users on ClickFix landing pages | Must-fix 2 |
| 15 | Cloud spellcheck sent revealed passwords (intro; §2.1) | Correct | BleepingComputer: CNN, Facebook "username and password when using 'show password'" | — |
| 16 | 8 of 9 cloud keyboards leaked keystrokes (intro; 6.17) | Correct | Citizen Lab: "eight of the nine apps" | — |
| 17 | CTF "no access control whatsoever" for nearly twenty years (intro) | Correct | Project Zero post, both phrases | — |
| 18 | 35–47 engineers plus 10–16 language data (intro; 9.3) | Correct | Track sum 35–47 recomputed | — |
| 19 | Candidate windows misplaced or missing in ≥ 8 Windows IMEs (§1) | Correct | IM 4.5 (Bugzilla titles), note only | — |
| 20 | Browser-engine editor 49 ms vs native 0.9 ms (§1) | Correct | Fatin, Windows averages: Atom 49.4, GVim 0.9 | — |
| 21 | No published keystroke-to-paint measurement for browsers (§1) | Unverifiable | Absence claim; IM 9.3 ran without search budget | — |
| 22 | Lexical: "don't even trigger any events at all" (§1) | Correct | Lexical FAQ | — |
| 23 | About 550 issue titles across editor trackers (§1) | Correct | GO 2.5 totals sum to 546; Medium (title search) | — |
| 24 | 32 Android workarounds in `@codemirror/view` release notes (§1; §5) | Correct | CHANGELOG: 32 lines name Android, Gboard or Samsung | — |
| 25 | Maintainer cannot install the Samsung keyboard (§1; §5) | Correct | ProseMirror discuss 5246, 2023-03-02 | — |
| 26 | HTML says nothing about undo after script sets `value` (§1; 6.6) | Correct | SS 4.10, note only | — |
| 27 | `execCommand('insertText')` is the only way to keep undo (§1; 6.2) | Correct | GO 6.2 (MDN) | — |
| 28 | Chromium persistent read permission; Firefox, Safari ask per read (§1) | Correct | MDN Clipboard API | — |
| 29 | ClickFix > 500% in H1 2025; "second most common attack vector after phishing" (§1) | Correct | ESET page: "over 500% compared to H2 2024" | — |
| 30 | Copy-paste XSS bypassed browser and editor sanitizers (§1) | Correct | SR 3.1–3.4 (MFSA 2020-01, Chrome 83 release) | — |
| 31 | Lexical ships `@lexical/dragon` (§1) | Correct | GO 2.7, note only | — |
| 32 | No specification maps a browser's own spelling marks to AT APIs (§1; 6.14) | Correct | GO 8.4 (Core-AAM absence Medium) | — |
| 33 | Firefox diacritics bug open 16.7 years (§1; §5) | Correct | Bugzilla 202251: 2003-04-16 to 2019-12-10 | — |
| 34 | Regex find requested since 2005 (§1; 6.12) | Correct | Bugzilla 298127, 2005-06-18, NEW | — |
| 35 | Shadow-tree find bugs filed October 2026 (§1) | Correct | GO 7.4, note only | — |
| 36 | WebDriver, BiDi, Playwright, Puppeteer cannot drive an IME; requests since 2019–2022 (§1) | Correct | #1683 open 2022-09-09; #5777 open 2021-03-10; GO 4.1–4.6 | — |
| 37 | No DevTools shows composition, target ranges, undo stack, clipboard flavours (§1) | Correct | GO 5.1–5.4 (absence Medium) | — |
| 38 | AI agents double or drop the first letter (§1) | Correct | GO 10.1 (titles) | — |
| 39 | GR's 16† cap on compositor-mutable nodes (§1; 6.7) | Correct | GR 6.3: "16† per document" | — |
| 40 | 1.19% of page loads call `execCommand` (§4.3; 6.2) | Correct | DM 6.12 text | — |
| 41 | IE 5.0 DnD 1999; IE 5.5 contentEditable 2000 (§2.1) | Partly correct | IE 5.5 date is recollection (HP 1.1 Medium) | Rec. 6 |
| 42 | Copied "in a haphazard and imprecise fashion" (§2.1; §5) | Correct | execCommand draft Introduction | — |
| 43 | Midas: clipboard off unless the user set a pref (§2.1) | Correct | HP 1.2 (also "or using signed JS") | — |
| 44 | Midas was "the web's first permission model" (§2.1) | Unverifiable | Researcher's interpretation; IE history Low | Rec. 6 |
| 45 | Firefox 2 spellcheck "with Hunspell and MySpell dictionaries" (§2.1) | Unverifiable | HP 9.5 engine detail is recollection | Rec. 6 |
| 46 | Flash Player 10 click rule; ZeroClipboard (§2.1) | Partly correct | HP 5.2 Medium (snippet), cause Low | Flag |
| 47 | Gesture-gated `execCommand('copy')`: Chrome 42/43, Firefox 41 (§2.1) | Correct | HP 5.3, note only | — |
| 48 | PPK: "No less than seven events"; `dropEffect` "made no difference" (§2.1) | Correct | QuirksBlog | — |
| 49 | Docs canvas: "consistency in how content appears across different platforms" (§2.1) | Correct | Workspace Updates post | — |
| 50 | Docs 2010 motives: pagination, collaboration (§2.1) | Partly correct | HP 4.1 Low (post body unread) | Rec. 6 |
| 51 | Gregor: engines "not remotely consistent" (§2.1) | Partly correct | Draft note is about unsupported commands only | Rec. 4 |
| 52 | Gregor's spec "stalled after one editor" (§2.1) | Partly correct | HP 1.5 Low/Medium | Rec. 6 |
| 53 | UndoManager never shipped (§2.1; §5) | Correct | HP 8.1 (fate Medium) | — |
| 54 | Seven editor projects called browser undo "useless or harmful" (§2.1; §5; 6.9) | Correct | #150: "either **useless** or **harmful**", seven projects listed | — |
| 55 | The WG chair said the drafts "eventually led to the EditContext API" (§2.1) | Correct | #480, Johannes Wilm (Web Editing WG Chair) | — |
| 56 | Haverbeke: "very little control over what the user and the browser are doing to our document" (§2.1) | Correct | ProseMirror post, 2015-07-07 | — |
| 57 | Draft.js archived 2023 for Lexical (§2.1) | Correct | HP 4.6, note only | — |
| 58 | Input Events split in 2017 over Chrome Android (§2.1) | Correct | HP 2.2 (as reported in #78) | — |
| 59 | Level 1 discontinued 2023 (§2.1) | Correct | SS 2.1 (DISC 2023-09-28) | — |
| 60 | Level 2 is WD 2026-10-15 (§2.1; 6.2) | Correct | TR header: "W3C Working Draft 15 October 2026" (HP 2.7's "2025-08" is stale) | Note fix |
| 61 | Level 2 keeps one non-cancelable `insertCompositionText`; three commit types gone (§2.1; 6.2) | Correct | TR table: only `insertCompositionText` has "No"; none of the three types present | — |
| 62 | `execCommand` "was synchronous and could only read and write to the DOM" (§2.1) | Partly correct | web.dev: "clipboard access was synchronous, and could only read and write to the DOM" | Rec. 5 |
| 63 | Safari (2020), Firefox (2024) per-read Paste menu; spec now requires it (§2.1; §5) | Correct | BCD readText Safari 13.1, Firefox 125; Clipboard WD 9.1.1 | — |
| 64 | EditContext: hidden editables "negatively impact accessibility and increase complexity" (§2.1) | Correct | HP 3.1 (explainer), note only | — |
| 65 | EditContext Chrome 121; VS Code default 2025-06; Mozilla positive with accessibility concern; WebKit silent; one engine (§2.1) | Correct | BCD 8.1.6; VS Code 1.101; #199 labels; #243 open, 0 comments | — |
| 66 | CTF began with Office XP's text services (§2.1) | Correct | Project Zero: earliest MSCTF "from the 2001 release of Office XP" | — |
| 67 | CTF fixed in 2019 (§2.1) | Correct | Project Zero 2019-08-13; CVE number flagged in 11.2 | — |
| 68 | Spell-jacking: Google promised to exclude passwords; `spellcheck="false"` advised (§2.1) | Correct | BleepingComputer: "exclude passwords proactively"; LastPass added `spellcheck=false` | — |
| 69 | ClickFix: "defences are all on the OS side" (§2.1) | Partly correct | Microsoft: SmartScreen in Edge; enterprise-managed browsers | Must-fix 2 |
| 70 | `plaintext-only` took about 15 years to reach a third engine (§2.2) | Correct | BCD: Safari 5, Firefox 136 | — |
| 71 | Firefox sandboxed Hunspell in 2021; bugs moved to the boundary (§2.2) | Correct | Hacks post; SR 6.4 titles | — |
| 72 | Selection API: caret undefined when not collapsed; "We need to more precisely define …" (§3) | Correct | Selection API WD 2026-06-11 | — |
| 73 | Level 2 still names `contentEditable=typing` (§3) | Correct | TR §6.2 default action | — |
| 74 | UI Events contradicts Input Events on composition order and cancelability (§3; 6.2) | Correct | UI Events 3.6.6 vs IE L2 §7 ("Are not cancellable") | — |
| 75 | Clipboard API says "Add definition of sanitized copy" and never "strip scripts" (§3) | Correct | Clipboard WD, four occurrences of the note | — |
| 76 | HTML DnD filtering: "does not specify how" (§3; 6.11) | Correct | HTML §6.11.8 | — |
| 77 | Caret stacking undefined; "Not sure if this is the correct way …" (§3; 6.7) | Correct | CSS UI 4; CSS Pseudo 4 | — |
| 78 | "HTML has no word on undo" (§3) | Partly correct | SS 4.10: three mentions, none on the history | Rec. 12 |
| 79 | Selection, Input Events, clipboard, execCommand never scored Interop areas; highlights in 2026 (§3) | Correct | SS 11.1–11.3, note only | — |
| 80 | WPT 2026-10-19: 90.5, 94.3, 81.8, 95.4; third engine 52.4, 60.1 (§3) | Correct | Match SS 1.9, 2.12, 5.7, 6.12 (not recomputed; flagged in 11.2) | — |
| 81 | Chrome's Recorder stores only a field's final value (§3) | Correct | GO 4.9 (Medium) | — |
| 82 | Chrome's custom-formats article needs a macOS app to show writes (§3) | Correct | GO 5.4, note only | — |
| 83 | "Mutation events are gone from every engine" (§4.4) | Unverifiable | Only Chrome 127 sourced (SR 1.5) | Rec. 7 |
| 84 | Selection API maps left and right through the bidi level; "additional context (such as caret affinity)" (§4.4; 6.4) | Correct | Selection API WD `modify()` | — |
| 85 | Still not specified 26 years later (§5) | Correct | 2000 to 2026; SS 4.2 | — |
| 86 | Gregor's spec "never a vendor conformance target" (§5) | Partly correct | HP 1.5 Low | Rec. 6 |
| 87 | `cE=minimal`: ten years of subsets (§5) | Correct | #480 "a decade ago" | — |
| 88 | Mozilla ContentCache; Chromium's 1.5 s timed wait (§5) | Correct | IME handling guide; Chromium design page (value inside a code listing; see M-1 section) | — |
| 89 | CTF: no access control "for twenty years" (§5) | Partly correct | Project Zero: "nearly twenty years" | Rec. 13 |
| 90 | Third-party TIP crashes land in the host process (§5; 6.5) | Correct | IM 4.7 (titles) | — |
| 91 | Bentkowski's copy-paste XSS (§5) | Correct | SR 3.1–3.2 (MFSA 2020-01; Chrome 83, $3,000) | — |
| 92 | Hunspell memory bugs in affix and suggestion paths (§5; 6.8) | Correct | Bugzilla titles: `SfxEntry::checkword`, `SuggestMgr::leftcommonsubstring`, `AffixMgr::setcminmax` | — |
| 93 | ClickFix: "browsers show nothing" (§5) | Partly correct | SmartScreen flags landing pages; nothing documented for clipboard writes | Must-fix 2 |
| 94 | EditContext text "not exposed to assistive technologies" (§5; 6.14) | Correct | EditContext WD 2026-09-10 | — |
| 95 | Target ranges cover "only the code points that the user agent would normally replace …" (6.2) | Correct | IE L2 TR | — |
| 96 | Composition sequence and cancelability (6.2) | Correct | IE L2 TR §7 | — |
| 97 | The three engines converged on Input Events' model (6.2) | Partly correct | SS implications: "to be confirmed by measurement"; HP 2.4 | Rec. 8 |
| 98 | Draft defines 40 commands, about a dozen TODOs, undo and redo undefined (6.2) | Correct | Draft: "I haven't defined the 'undo' or 'redo' commands yet"; SS 5.2 | — |
| 99 | Best engine passes 96.0% of `editing/run/` (6.2) | Correct | SS 5.7 (Firefox) | — |
| 100 | "Keep the markup as concise as possible" (6.2) | Correct | execCommand draft | — |
| 101 | Command to `inputType` map (6.2) | Correct | SS 5.3, note only | — |
| 102 | Draft fires cancelable `beforeinput`; Chrome does not; Safari with empty `inputType` (6.2) | Correct | SS 5.3, 4.11 (engine behaviour Medium, as reported) | — |
| 103 | `plaintext-only` in all three engines in 2025 (6.2) | Correct | BCD: Chrome 51, Safari 5, Firefox 136 | — |
| 104 | Empty-host caret "is expected to act like an inline replaced element …" (6.2) | Correct | SS 4.3, note only | — |
| 105 | Firefox titles: `selectstart` removing a table cell, rules replaced "while running scripts", clipboard events (6.3) | Correct | Bugzilla (Selection bug 2026-04-06; SR 1.4) | — |
| 106 | Chrome removed mutation events in 127 for the same reason (6.3) | Correct | Chrome post: listeners "can change the entire DOM underneath a running DOM operation" | — |
| 107 | Word moves stop at end of word on macOS, start of next word on Windows and Linux (6.4) | Unverifiable | No note row | Rec. 9 |
| 108 | Android tolerates a slow editor for "a couple seconds" (6.5) | Correct | IM 7.2 | — |
| 109 | Every documented multi-process browser answers from a pushed cache (6.5) | Partly correct | IM 4.1: Chromium blocked with a 1.5 s timed wait | Rec. 10 |
| 110 | Mozilla's whole-text cache "may be too big" (6.5) | Correct | IME handling guide: "This may be too big" | — |
| 111 | Never return `TS_E_NOLAYOUT` right after reporting a layout change (6.5) | Partly correct | Rule is from bug 1187351 (IM implications), not IM 4.5 | Rec. 11 |
| 112 | `textInputClientDidUpdateSelection()` from macOS 15.4 (6.5) | Correct | IM 5.4, note only | — |
| 113 | text-input-v3 seven-step `done` order (6.5) | Correct | Protocol XML; IM 6.3 | — |
| 114 | Hold notifications until the renderer acknowledges (Mozilla rule) (6.5) | Correct | IME guide: "If all sent WidgetCompositionEvents and WidgetSelectionEvents are already handled …" | — |
| 115 | UTF-8 bytes on Wayland; Android warns on surrogate pairs (6.5) | Correct | Protocol XML; IM 1.2 | — |
| 116 | `keyCode` 229, `"Process"`, `"Dead"` (6.5) | Correct | UI Events: "return 229" | — |
| 117 | `NSTextInputClient` has 21 members (6.5) | Correct | IM 5.1 list counts 21 | — |
| 118 | v2 in Mutter 51 and KWin 6.7; v1 elsewhere; none in Weston (6.5) | Correct | wayland.app table | — |
| 119 | Surrounding text limited to 4,000 bytes (6.5) | Correct | XML at 1.47 and 1.48: "4000 bytes" | — |
| 120 | text-input-v3 version 2 in wayland-protocols 1.48 (1.47: v1), with `set_available_actions` (6.5; 6.13) | Correct | XML tags: `version="1"` vs `version="2"`; 1.48 adds `set_available_actions`, `preedit_hint`, `language`, `show_input_panel` | — |
| 121 | IBus, Fcitx, GTK and Qt IM modules are LGPL (6.5) | Partly correct | IM 12.6 Medium (recollection) | Flag |
| 122 | Microsoft: "All passwords fields should have text services disabled"; `IS_PASSWORD` "is not supported" (6.5) | Correct | InputScope page | — |
| 123 | Secure event input on focus, off on blur or deactivation, "as TN2150 requires" (6.5) | Correct | TN2150: enable "when the keyboard focus moves to a private data entry field"; disable on change or on "losing keyboard focus". Roman restriction is IM 5.4, not 10.6 | Rec. 11 (citation) |
| 124 | `IME_FLAG_NO_PERSONALIZED_LEARNING` names browsers' private modes (6.5) | Correct | IM 7.5 | — |
| 125 | Wayland purpose `password`, hints `sensitive_data`, `hidden_text` (6.5) | Correct | IM 6.4 | — |
| 126 | Windows `IS_PRIVATE` (6.5) | Correct | InputScope page: value 61, "private data", Windows 10 | — |
| 127 | `NSWritingToolsBehavior.none`; `setWritingToolsEnabled(false)` API 36 (6.5) | Correct | IM 5.7, 7.5, note only | — |
| 128 | macOS autocorrect works only when the engine owns the text (6.5) | Correct | HP 3.9 (w3c/editing #506), note only | — |
| 129 | TSF loads TIP DLLs into the app's process; some "download and upload user input on the fly" (6.5) | Correct | IM 2.9, note only | — |
| 130 | TX-D23's reversal trigger quoted (6.5) | Correct | TX: "An OS offers out-of-process IME hosting for a window owned by another process" | — |
| 131 | AX-D9 (b) UI host holds no grants and owns the HWNDs (6.5) | Correct | AX text | — |
| 132 | EditContext update methods; events fire at the EditContext (6.5) | Correct | SS 3.1 (EditContext WD) | — |
| 133 | HTML text-control quotes ("either a selection or …", "set the selection range", "last changed by a user edit") (6.6) | Correct | SS 4.8–4.9, note only | — |
| 134 | `getTargetRanges()` empty for text controls (6.6) | Correct | IE L2 TR ("Otherwise, the target ranges MUST be empty"); SS 2.6 | — |
| 135 | Chromium threat model: a renderer-tracked gesture "can be forged" (6.6) | Correct | SR 8.3 (prose doc) | — |
| 136 | 11 of 11 password-manager extensions (6.6; 6.17) | Correct | SR 8.2, note only | — |
| 137 | CSS UI 4 blink setting (WCAG 2.2.2) (6.7) | Partly correct | CSS UI 4 cites "Guideline 2.2 and Guideline 2.3" | Rec. 14 |
| 138 | Highlight property list (6.7) | Partly correct | Omits `stroke-width` (SS 7.1) | Rec. 14 |
| 139 | Overlay order; `::search-text` directly below `::selection` allowed (6.7) | Correct | CSS Pseudo 4: "directly over or below the ::selection overlay depending on the UA" | — |
| 140 | Overlay covers the em box; may extend to the line box (6.7) | Correct | CSS Pseudo 4 §3.4 | — |
| 141 | Custom Highlight API is an Interop 2026 focus area (6.7) | Correct | SS 7.6, note only | — |
| 142 | `caret-shape`, `caret-animation` Chrome only (6.7) | Correct | SS 7.9, note only | — |
| 143 | HTML's eight-step spellcheck algorithm (6.8) | Correct | SS 4.4, note only | — |
| 144 | Hunspell model: affixes, twofold suffixes, compounding, REP, MAP, KEY, PHONE (6.8) | Correct | SL 2.1–2.2 (`hunspell(5)`) | — |
| 145 | Weighted finite-state spellers handle Finnish, Sámi, Greenlandic (6.8) | Partly correct | SL 2.5 Low (paper not read) | Rec. 6 |
| 146 | Norvig's figures show the error model, not candidate generation, limits accuracy (6.8) | Partly correct (misattributed) | Norvig: unknown words "the more serious"; context "best" | Must-fix 4 |
| 147 | Other browsers sandboxed spellcheckers because of memory-unsafe dictionary parsing (6.8) | **Wrong** (misattributed cause) | Hacks post gives only general reasons; SR 6.3 | Must-fix 3 |
| 148 | CSS Pseudo 4 requires mitigation; quotes on "programatic modification" and the address book (6.8) | Correct | CSS Pseudo 4 §3.7 | — |
| 149 | Spell-jacking and "show password" (6.6; 6.8) | Correct | BleepingComputer | — |
| 150 | Firefox offers 175 dictionaries in 114 locales (6.8) | **Wrong** | AMO API: 207 dictionaries, 170 locales | Must-fix 5 |
| 151 | Windows and Android accept third-party spelling providers (6.8) | Correct | SL 4.5–4.6, note only | — |
| 152 | LanguageTool rules range "from 892 for Russian to 8,251 for Catalan" (6.8) | Partly correct | LanguageTool table: 31 languages, 32 to 8,251 | Must-fix 6 |
| 153 | `autocorrect` states; `password`, `email`, `url` always Off (6.8) | Correct | SL 1.5–1.6; SS 4.6 | — |
| 154 | `writingsuggestions` Chrome 124, Safari 18 (6.8) | Correct | BCD | — |
| 155 | `historyUndo` fires only with native entries (w3c/editing #509) (6.9) | Correct | SS 4.10–4.11 (as reported) | — |
| 156 | Compromised renderers attacked clipboard IPC in both other engine families (6.10; 6.17) | Correct | SR 2.10 (CVE-2020-16025, CVE-2022-46872), note only | — |
| 157 | "Should create alternate text/html and text/plain" (6.10) | Correct | Clipboard WD §8 | — |
| 158 | Chrome and Firefox return raw HTML in paste events (6.10) | Correct | HP 5.8 (Chrome), SS 6.6 (Firefox) | Rec. 18 |
| 159 | Pasted data "must not download referenced online resources" (6.10) | Correct | SR 3.6, note only | — |
| 160 | Paste button enabled after 1 s is "the rule the Clipboard API now specifies" (6.10) | Partly correct (misattributed) | WD 9.1.1 has no delay; MDN: Firefox and Safari | Must-fix 8 |
| 161 | No button for same-origin clipboard content (Firefox and Safari) (6.10) | Correct | MDN: "suppressed if reading same-origin clipboard content" | — |
| 162 | Chrome and Safari sanitize async reads (6.10) | Correct | HP 5.5, 5.8; SS 6.6 | — |
| 163 | `unsanitized` Chrome only, negative WebKit position (6.10) | Correct | HP 5.8, note only | — |
| 164 | `clipboardchange` Chrome only (6.10) | Correct | BCD: Chrome 144 | — |
| 165 | `allowWithoutGesture` write permission (6.10) | Correct | Clipboard WD §9 | — |
| 166 | Microsoft offers only OS-side mitigations and awareness (6.10) | Partly correct | SmartScreen in Edge; enterprise-managed browsers | Must-fix 2 |
| 167 | Web custom formats capped at 100 per session (6.10) | Correct | HP 5.7, note only | — |
| 168 | Images "may be transcoded to produce a safe version" (6.10) | Correct | SS 6.5, note only | — |
| 169 | CVE-2023-6208, primary selection from the Selection API (6.10) | Correct | SR 2.9, note only | — |
| 170 | Drag store modes; data unavailable "until the drop event"; "not … in response to script actions" (6.11) | Correct | HTML §6.11.8 | — |
| 171 | Firefox drag record is "policy bugs, not memory bugs" (6.11) | Partly correct | 1 UAF of 34; SR 4.7 overflows | Rec. 15 |
| 172 | "Cross-origin iframe receives file stream before drop" filed 2026-09 (6.11) | Correct | Bugzilla, 2026-09-19 | — |
| 173 | Drag-out planting, CVE-2022-22756 (6.11) | Correct | SR 4.5, note only | — |
| 174 | HTML find: "synchronously (and so is not observable)"; tracking vector; "not acting on every character" (6.12) | Correct | HTML §6.9 (tracking-vector marker present) | — |
| 175 | Text directives in three engines; constant-shape rule (6.12) | Correct | SS 9.3; SR 7.2 | — |
| 176 | `inputmode` 8 keywords; `enterkeyhint` 7 (6.13) | Correct | SS 4.7, note only | — |
| 177 | InputScope has 74 values (6.13) | Correct | Page: 0–68 plus −1 to −5 | — |
| 178 | VirtualKeyboard API and `interactive-widget` Chromium only (6.13) | Correct | BCD VirtualKeyboard Chrome 94 only; SS 10.4 | — |
| 179 | "HTML's default (`resizes-visual`)" (6.13) | Partly correct | CSS Viewport's default (SS 10.4) | Rec. 16 |
| 180 | WebDriver "IME support in actions" open since 2022; Playwright since 2021 (6.16) | Correct | GitHub API: both open | — |
| 181 | A W3C issue of 2026-10-05 asks for "exactly such a channel" (6.16) | Partly correct | #551 body: editors agreeing a common API | Rec. 17 |
| 182 | All 7 Chrome "in Editing" 2026 CVEs are leaks or authorization bugs (6.17) | Correct | SR 1.6 (one is an origin-policy bypass), note only | — |
| 183 | 56 iOS apps read the pasteboard (6.17) | Correct | SR 2.11 (Mysk) | — |
| 184 | Pastejacking 800 ms after keydown (6.17) | Correct | SR 2.1, note only | — |
| 185 | Bentkowski: two engines and five editors (6.17) | Partly correct | SR 3.3 Medium | Rec. 19 |
| 186 | 34 Firefox drag security bugs, 1 a UAF (6.17) | Correct | Bugzilla re-count: 34, 1 UAF | — |
| 187 | TSF CVEs into 2026 (6.17) | Correct | SR 5.3, note only | — |
| 188 | Loophole recovered keystroke timing across origins (6.17) | Correct | SR 9.2 | — |
| 189 | `windows` 0.62, `objc2-app-kit` 0.3.2 and their licences (§7) | Correct | IM 12.1–12.2 (crates.io) | — |
| 190 | `zbus` is MIT (§7) | Correct | crates.io: 6.0.0, MIT (no note row) | Rec. 20 |
| 191 | `xkbcommon-dl` MIT; `jni` MIT OR Apache-2.0 (§7) | Correct | IM 12.4–12.5 | — |
| 192 | Hunspell MPL/GPL/LGPL; Nuspell LGPL-3.0; LanguageTool LGPL (§7) | Partly correct | SL 3.3, 3.7 Medium | Rec. 6 |
| 193 | Harper Apache-2.0 only, English only (§7) | Correct | GitHub: Apache-2.0; "Harper currently only supports English" | — |
| 194 | SymSpell MIT (§7) | Correct | README: "changed from LGPL 3.0 to … MIT" | — |
| 195 | SCOWL "MIT-like" (§7) | Correct | SL 3.1 (README) | — |
| 196 | TX's language-data track is 4–6 (9.3) | Correct | TX revision 0.3.2 | — |
| 197 | DM staffing 83–106 (9.3) | Correct | DM 9.3 | — |
| 198 | "LanguageTool's decade of rules" (9.3) | Unverifiable | No source in SL | Rec. 20 |
| 199 | Spell-jacking 30-site figures from news; BleepingComputer tested five sites (11.2) | Correct | BleepingComputer lists five sites | — |
| 200 | Project Zero's post does not name CVE-2019-1162 (11.2) | Correct | No "CVE-2019" string in the post | — |
| 201 | Mozilla's "positive" label dated 2024-12-09 (11.2) | Correct | GitHub timeline: 2024-12-09 | — |

The six unverifiable claims are 21, 44, 45, 83, 107 and 198. The four wrong claims are 1, 14, 147 and 150.

## Link check

- **Spec, external links (9 URLs, 0 with fragments).** All resolve:
  - ESET H1 2025, QuirksBlog, w3c/editing #480, Haverbeke's post, Project Zero, Microsoft's ClickFix post and Microsoft Learn InputScope returned 200 on the first request.
  - The execCommand draft (GitHub Pages) returned a transient 503, then 200.
  - Input Events L2 TR returned 403 to a browser user-agent (a Cloudflare challenge), then 200 with curl's default client.

  No external link carries a `#fragment`, so there are no fragments to check.
- **Spec, link reference definitions (24).** All resolve to existing files: FR, HP, SS, IM, SL, SR, GO, H2, H3, S1, JS, FP, ST, TX, LY, GR, MD, DM, AX, DT, UI, PD, CB and LG.
- **Spec, internal `[XX n.m]` citations.** A script parsed all 326 citation parts to HP, SS, IM, SL, SR, GO and FR, including ranges and section-only citations such as `[GO 4–5]`. Every row and section exists. The `Q` and `method notes` citations were checked by hand: `[SS Q1, Q2, Q6–Q8]`, `[IM Q1–Q4, Q6, Q7]`, `[SR Q3, Q5, Q6]`, `[GO Q1–Q4]`, `[SL Q-SP-4]` and both method-notes references exist and say what the spec says. The rows that exist but support the spec only in part are listed above (claims 9, 109, 111, 123, 146, 147, 160).
- **Notes, sample.** The notes contain 246 unique URLs. A sample of 39 (every sixth, NVD and Bugzilla REST queries excluded) was requested, plus about 45 more fetched while checking claims.
  - **Resolved:** 33 of the 39 returned 200. The Apple `characterIndex(for:)` page's first failure was an extraction artefact, and it returns 200.
  - **Blocked or rate-limited, but live in a browser:**
    - `developer.android.com` CursorAnchorInfo (302 to a Google sign-in redirect for non-browser clients);
    - Lexical `testing.md` (GitHub 429);
    - `w3.org/TR/input-events-1/` (429);
    - `w3c.github.io/webdriver/` (time-out, then 200).
  - **Dead:** `help.passbolt.com/incidents/20220920_spell-jacking` (SR 6.5) returns 404.
  - **Fragments:** the note fragments checked all exist: HTML `#editing-2`, `#spelling-and-grammar-checking`, `#writing-suggestions`, `#autocapitalization`, `#autocorrection`; CSS Pseudo 4 `#highlight-pseudos`; UI Events `#keys-dead`.

## M-1 source check

- **Spec.** No link points to any project's source code. The nine external URLs are specifications, security reports, vendor documentation, an issue discussion, a blog post and a postmortem. The cross-spec facts trace to Izcal documents. No value in Sections 1–3, 5, 6, 7, 9.3 or 11.2 is derived from source code. The values the spec could only have from engines are set with † or deferred to ED-M0 measurements (6.21, §7 "Where only another engine's source would state a value").
- **Notes.** All 246 note URLs were scanned for source hosts and code paths (`googlesource`, `searchfox`, `source.chromium.org`, `hg.mozilla.org`, `trac.webkit`, `/blob/`, `/tree/`, `/raw/`, commit and code-review hosts, code extensions). The hits are:
  - `chromium.googlesource.com/…/docs/security/side-channel-threat-model.md`: a prose document.
  - `/blob/` links: the `@codemirror/view` CHANGELOG (release notes), READMEs, docs Markdown (Lexical, Slate), W3C explainers, the SCOWL README, and a BCD JSON file (CC0 data).
  - The wayland-protocols XML at tags 1.47 and 1.48: the published protocol definition, used as an interface specification like the CDP definition (DT-D11).
  - `raw.githubusercontent.com/…/interop-2026-stable-v2.csv`: data.
  - w3c/input-events PR #122: a specification repository's PR, cited for its discussion only.

  No code file of any browser, editor, spellchecker, IME or OS component is cited.
- **Borderline items, none blocking:**
  1. **IM 4.1 and the spec's §5 ("Chromium's 1.5 s timed wait").** On the Chromium design page, the 1.5 s value appears inside an embedded code listing (`condition_.TimedWait(1.5 seconds)`). The page is a prose design document, not a source file, but the number itself is read from code text. The spec uses the value only as history, not as an Izcal parameter (Izcal's own budgets are † values). Describe it as "a bounded timed wait of about 1.5 s described in the design document", or drop the number.
  2. **GO 5.2** links Lexical DevTools as `github.com/facebook/lexical/tree/main/packages/lexical-devtools`, a source directory whose page renders the README. Link the README file or the Lexical docs page instead.
  3. **The Firefox "IME handling guide"** is on `firefox-source-docs.mozilla.org`, which is generated prose documentation. It is allowed as prose, the same reading as the Chromium `docs/**.md` pages.
  4. **The note M-1 statements** say that commit titles from the w3c/input-events and wayland-protocols repositories were listed before the 19 October rule and then dropped as evidence. No row cites a commit, and the version dating rests on published TR versions and the XML at release tags. This is consistent with M-1.
- **Confirmation.** No research note or spec section cites any project's source code, and no value in the spec is derived from it.
