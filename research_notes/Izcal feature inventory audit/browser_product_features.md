# Izcal feature inventory audit: browser product features

Compiled 24 October 2026. Primary sources were preferred (vendor help centres, release notes, official feature pages).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

**Scope.** This file lists the browser's own user-facing product features. It does not list web platform APIs. One row is one coarse feature. Section 22 maps the feature classes in the brief to these rows.

**How to read the columns.**

- **Shipped by** gives Chrome / Firefox / Safari / Edge, then other browsers that ship the feature or a distinctive version of it. **Y** = ships it. **P** = ships part of it, or only on some platforms or in some regions. **N** = does not ship it. **OS** = the operating system provides it, and the browser hands off to it. **?** = not verified in this pass.
- **Tags.** **[TS]** = table stakes: users widely expect it, and leaving it out is a visible defect. **[Diff]** = differentiator: one or a few browsers ship it, and users choose browsers for it. Untagged rows are expected by power users but not by everyone.
- **Conf.** grades the row's main claim against the cited source. The four main-browser columns were filled in from that source where it covers them. Otherwise they come from the researcher's knowledge of shipping browsers up to mid-2026, and were not each re-checked. A row whose main claim was not checked in any source opened during this pass is graded **Low** and cites "not re-checked" (see "Gaps in this inventory").
- **Sources opened this pass.** These were read on 24 October 2026: the Chrome features page, the Chrome Help index (161 article titles), the Chrome performance help article, the Firefox all-features page, the Microsoft Edge features page, the Safari User Guide for Mac (macOS 27) table of contents, and the feature pages for Vivaldi, Brave, Opera, Zen, Orion, Dia and Arc. Firefox AI Controls, Edge Copilot Actions and Journeys, and Safari 26 and 27 were checked through news coverage and release-note snippets only.
- **Status of Arc.** Arc is being discontinued. New account creation ended on 22 October 2026. Arc for macOS and Windows stops working on 22 December 2026. Dia is its successor ([Arc][ARC]). Arc is still listed below as the origin of several design patterns.

---

## 1. Windows, tabs and sessions

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-1.1 | Multiple windows [TS] | Open, move, merge and close several top-level browser windows, and drag tabs between them. | Y / Y / Y / Y / all | [Chrome Help: Manage tabs][CHR-H] | Medium |
| BF-1.2 | Tab strip basics [TS] | Open, close, reorder, duplicate, move to a new window, and close other tabs or tabs to the right. | Y / Y / Y / Y / all | [Chrome Help: Manage tabs][CHR-H] | Medium |
| BF-1.3 | Pinned tabs [TS] | Small, persistent tabs that stay at the start of the strip and survive restarts. | Y / Y / Y / Y / Arc, Zen, Dia | [Firefox: all features][FX-F] | High |
| BF-1.4 | Tab groups [TS] | Named, colour-coded groups of tabs that can be collapsed. | Y / Y / Y / Y / Opera "Tab Islands", Dia | [Firefox: all features][FX-F]; [Safari Guide: Grouping tabs][SAF-G] | High |
| BF-1.5 | Saved and synced tab groups | Close a group, restore it later, and get it on other devices. | Y / ? / Y / ? / Vivaldi | [Safari Guide: Grouping tabs][SAF-G] | Medium |
| BF-1.6 | Two-level tab stacks [Diff] | Tabs nested inside tabs. Vivaldi offers two-level stacks, accordion and compact styles. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-1.7 | Vertical tabs | Tabs shown in a side column with full titles. | ? / Y / N / Y / Brave, Opera, Vivaldi, Arc, Zen, Dia | [Firefox: all features][FX-F]; [Brave features][BRAVE-F]; [Opera features][OPERA-F] | High |
| BF-1.8 | Split view / tab tiling [Diff→TS] | Two or more pages side by side in one window. | Y / ? / N / Y / Vivaldi, Opera, Zen, Arc, Dia | [Chrome Help: split view][CHR-H]; [Edge features][EDGE-F]; [Vivaldi features][VIV-F]; [Zen][ZEN] | High |
| BF-1.9 | Workspaces / Spaces [Diff] | Separate, switchable sets of tabs and groups inside one profile. | N / N / P (tab groups) / Y / Vivaldi, Opera, Zen, Arc | [Vivaldi features][VIV-F]; [Opera features][OPERA-F]; [Zen][ZEN] | High |
| BF-1.10 | Tab search / tab switcher [TS] | Find an open tab by its title or URL. | Y / Y / Y / Y / Opera, Vivaldi | [Edge features][EDGE-F]; [Opera features][OPERA-F] | High |
| BF-1.11 | Tab hover preview | Hover over a tab to see its title, URL, thumbnail and memory use. | Y / Y / ? / Y / Vivaldi | [Chrome Help: Performance][CHR-P] | High |
| BF-1.12 | Mute tab or site [TS] | An audio indicator on the tab, and one click to mute it. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-1.13 | Reopen closed tab or window [TS] | Undo a close, and a "recently closed" list. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-1.14 | Session restore at startup [TS] | Reopen the previous session's windows and tabs on launch. | Y / Y / Y / Y / all | [Chrome Help: homepage and startup page][CHR-H] | Medium |
| BF-1.15 | Named saved sessions [Diff] | Save the open tabs as a named session to reopen later. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-1.16 | Crash recovery [TS] | After a browser crash, offer to restore tabs. A crashed page shows a "sad tab" page with a reload button. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-1.17 | Link peek / glance [Diff] | Open a link in a temporary floating preview without opening a tab. | N / P (AI link previews) / N / ? / Zen "Glance", Arc "Peek" | [Zen][ZEN]; [Firefox AI controls (news)][FX-AI] | Medium |
| BF-1.18 | Auto-archive of idle tabs [Diff] | Close tabs left unused for a set time, and keep them in an archive. | N / N / N / N / Arc, Zen | Not re-checked | Low |
| BF-1.19 | Tab emoji / custom tab names | Rename a pinned tab, or give it an emoji. | N / N / N / N / Opera, Arc, Dia | [Opera features][OPERA-F]; [Arc][ARC] | High |
| BF-1.20 | Journeys / topic history [Diff] | Groups past browsing into topics so the user can resume interrupted work. | N / N / Y (Safari 27 "Resume Browsing") / Y / — | [Edge features][EDGE-F]; [Safari 27 (news)][SAF-27] | Medium |
| BF-1.21 | Full screen and toolbar-in-full-screen [TS] | Hide the browser UI, with an option to keep the toolbar visible. | Y / Y / Y / Y / all | [Chrome features][CHR-F] | High |
| BF-1.22 | UI auto-hide / compact mode [Diff] | Hide the tab bar and toolbars until the user needs them. | N / P (density) / P (compact tabs) / ? / Zen, Vivaldi, Arc | [Zen][ZEN]; [Vivaldi features][VIV-F] | High |
| BF-1.23 | Tabs from other devices | Show the tabs open on the user's other synced devices. | Y / Y / Y / Y / all with sync | [Firefox: all features (sidebar: synced tabs)][FX-F] | High |
| BF-1.24 | Open-tab list in the OS app switcher (mobile) | Show the browser's open tabs in Android's recent-apps list. | Y (Android) / ? / N / ? / — | [Chrome Help index][CHR-H] | Medium |

**Implications:** Pinned tabs, groups, search, restore and crash recovery are the minimum. Split view and vertical tabs are becoming table stakes, now that Chrome ships split view and Firefox and Edge ship vertical tabs. Izcal must decide whether workspaces and stacks are first-class objects in its session model or a view over tab groups. Retrofitting this later is costly, because sync and restore depend on it.

## 2. Address bar, search and navigation

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-2.1 | Unified address and search bar [TS] | One field accepts either a URL or a search query. | Y / Y / Y / Y / all | [Chrome Help: Search the web][CHR-H] | Medium |
| BF-2.2 | Suggestions [TS] | As the user types, suggest matches from history, bookmarks, open tabs and the search engine. | Y / Y / Y / Y / all | [Chrome features][CHR-F] | High |
| BF-2.3 | Default search engine choice [TS] | Choose and change the default engine, and add a custom one. | Y / Y / Y / Y / all | [Chrome Help: default search engine][CHR-H] | High |
| BF-2.4 | Search engine choice screen (EU DMA) [TS in EU] | Prompt the user to pick a default engine, as required by law in some markets. | Y / ? / Y / Y / — | [Chrome Help: choice screens][CHR-H] | High |
| BF-2.5 | Site search keywords / shortcuts | Type a keyword plus a query to search one site, or "@" scopes such as @bookmarks, @tabs and @history. | Y / Y / N / Y / Vivaldi | [Chrome features][CHR-F]; [Chrome Help: site search shortcuts][CHR-H] | High |
| BF-2.6 | Calculator and unit conversion in the address bar | Show the answer to an equation or a unit conversion inline. | Y / Y / Y / ? / Opera (on-page unit converter) | [Chrome features][CHR-F]; [Opera features][OPERA-F] | High |
| BF-2.7 | Quick actions / command palette [Diff] | Run browser commands (settings, clear data, open a tab) from the address bar or a palette. | P / Y / N / ? / Vivaldi "Quick Commands", Arc command bar | [Vivaldi features][VIV-F] | High |
| BF-2.8 | Voice search | Speak a query instead of typing it. | Y / N / P (OS dictation) / Y / — | [Chrome features][CHR-F] | High |
| BF-2.9 | Visual search (Lens) | Search with an image or a region of the screen, without opening a new tab. | Y / N / P (OS Visual Look Up) / Y / — | [Chrome features][CHR-F]; [Edge features (Search bar images)][EDGE-F] | High |
| BF-2.10 | Security indicator and URL display [TS] | Show the origin clearly, label a connection "Not secure", and open page info from the icon. | Y / Y / Y / Y / all | [Chrome Help: secure connection][CHR-H] | Medium |
| BF-2.11 | Address bar position (mobile) | Move the address bar to the top or the bottom of the screen. | Y / Y / Y / ? / — | [Chrome features][CHR-F]; [Chrome Help: address bar position][CHR-H] | High |
| BF-2.12 | AI search mode in the address bar | Send a query to an AI answer mode, or to a chat, from the address bar. | Y ("AI Mode", @gemini) / P / ? / Y (Copilot) / Dia, Opera | [Chrome Help: AI Mode][CHR-H]; [Chrome features][CHR-F] | High |
| BF-2.13 | Sponsored and contextual suggestions, with an off switch | Suggestions from partners or trending queries, which the user can turn off. | Y / Y (Firefox Suggest) / ? / Y / — | Not re-checked | Low |
| BF-2.14 | Home button / homepage [TS] | A configurable homepage and a home button on the toolbar. | Y / Y / Y / Y / all | [Chrome Help: homepage][CHR-H]; [Safari Guide: homepage][SAF-G] | High |
| BF-2.15 | Mouse gestures [Diff] | Run actions such as back or close tab by drawing with the mouse. | N / N / N / Y / Vivaldi, Opera | [Vivaldi features][VIV-F] | High |
| BF-2.16 | Search engine bundled with the browser [Diff] | The vendor's own private or independent search engine is the default. | — / — / — / — / Brave Search, Orion with Kagi | [Brave features][BRAVE-F]; [Orion][ORION] | High |
| BF-2.17 | Desktop search bar outside the browser | A search box on the OS desktop that opens results in the browser. | N / N / N / Y / — | [Edge features][EDGE-F] | High |

**Implications:** Izcal needs a ranked suggestion engine that queries several providers (history, bookmarks, tabs, search, commands and AI). It also needs a policy on sponsored suggestions and a defined choice-screen path for the EU. The command-palette design will set how discoverable every other feature is.

## 3. Bookmarks, history, reading list and start page

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-3.1 | Bookmarks and bookmark bar [TS] | Save, edit and organise pages in folders, with a toolbar for favourites. | Y / Y / Y / Y / all | [Chrome Help: bookmarks][CHR-H]; [Safari Guide][SAF-G] | High |
| BF-3.2 | Bookmark manager with tags | A full-window manager with search, folders, tags and import/export. | Y / Y (tags) / Y / Y / Opera, Vivaldi | [Firefox: all features][FX-F] | High |
| BF-3.3 | History view and search [TS] | A list of visited pages by date, with search and delete. | Y / Y / Y / Y / all | [Chrome Help: browsing history][CHR-H] | High |
| BF-3.4 | History statistics [Diff] | A visual timeline and statistics of browsing activity. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-3.5 | Reading list / read later | Save a page to read later, offline on mobile. | Y / N (Pocket was retired in 2025) / Y / P (Collections) / Brave (sidebar) | [Chrome Help: Read pages later & offline][CHR-H]; [Safari Guide: Reading List][SAF-G] | Medium |
| BF-3.6 | Collections / pinboards [Diff] | Clip pages, images and notes into named boards to share. | N / N / N / Y (Collections) / Opera "Pinboards" | [Opera features][OPERA-F] | Medium |
| BF-3.7 | New tab page / start page [TS] | A customisable page with shortcuts, a background, a search box and widgets. | Y / Y / Y / Y / all | [Chrome Help: New Tab page][CHR-H]; [Safari Guide: start page][SAF-G] | High |
| BF-3.8 | Speed dial tiles | A grid of pinned site tiles, in groups. | P (shortcuts) / P (top sites) / Y (Favorites) / P / Vivaldi, Opera | [Vivaldi features][VIV-F] | High |
| BF-3.9 | News feed on the new tab page | Curated or personalised news on the start page. | P / Y / N / Y / Brave News, Opera Personal News | [Brave features][BRAVE-F]; [Opera features][OPERA-F] | High |
| BF-3.10 | Dashboard widgets on the start page [Diff] | Mail, calendar, notes, tasks, feeds and web widgets on one start page. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-3.11 | Start page shows bookmarks, reading list and history | Library content appears directly on the start page. | N / N / Y (Safari 27) / N / — | [Safari 27 (news)][SAF-27] | Low |
| BF-3.12 | Page change notifications [Diff] | Watch a page and get an alert when it changes in a way the user describes. | N / N / Y (Safari 27 "Notify Me") / N / — | [Safari Guide: notified when a website changes][SAF-G]; [Safari 27 (news)][SAF-27] | High |
| BF-3.13 | Wayback Machine fallback on a 404 [Diff] | Offer an archived copy when a page is missing. | N / N / N / N / Brave | [Brave features][BRAVE-F] | High |
| BF-3.14 | Recent tabs and "continue where you left off" cards | The new tab page offers to resume recent tasks and searches. | Y / ? / Y (Safari 27) / Y / Dia | [Safari 27 (news)][SAF-27] | Low |

**Implications:** Izcal needs one library store that holds bookmarks, history, reading list and collections, and that the address bar, sync and the start page all read. Firefox's retirement of Pocket shows that a read-later service tied to a vendor's cloud is fragile. A local-first design avoids that risk.

## 4. Profiles, accounts, sync and cross-device

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-4.1 | Multiple profiles [TS] | Separate browsing data, extensions and settings per person or role. | Y / Y / Y / Y / Vivaldi, Dia | [Chrome Help: multiple profiles][CHR-H]; [Safari Guide: profiles][SAF-G]; [Vivaldi features][VIV-F] | High |
| BF-4.2 | Guest mode | A throwaway session that cannot see the owner's data. | Y / N / N / Y / — | [Chrome Help: Browse as a guest][CHR-H] | High |
| BF-4.3 | Browser sign-in / account [TS] | Sign in to the vendor's account to turn on sync and services. | Y / Y / Y (Apple Account) / Y / all | [Chrome Help: Sign in to Chrome][CHR-H] | High |
| BF-4.4 | Sync [TS] | Bookmarks, passwords, history, tabs, settings and extensions shared across devices. | Y / Y / Y (iCloud) / Y / Brave, Vivaldi, Opera, Dia (all end-to-end encrypted except where noted) | [Firefox: all features][FX-F]; [Brave features][BRAVE-F]; [Dia][DIA] | High |
| BF-4.5 | End-to-end encrypted sync [Diff] | Sync data that the vendor cannot read. | P (passphrase) / Y / Y / ? / Brave, Vivaldi, Dia | [Brave features][BRAVE-F]; [Dia][DIA] | Medium |
| BF-4.6 | Send tab to device | Push the current tab to another of the user's devices. | Y / Y / OS (Handoff) / ? / Opera "Flow", Edge "Drop" | [Chrome Help: Share tabs][CHR-H] | Medium |
| BF-4.7 | Send files and notes between devices [Diff] | A personal chat-like channel to move files, links and notes. | N / N / OS (AirDrop) / Y (Drop) / Opera "Flow" | [Opera features][OPERA-F] | High |
| BF-4.8 | Profile-based work/personal separation | Automatic profile switching per site or account, and a work profile with SSO. | N / N / N / Y / Dia for Work | [Dia][DIA] | Medium |
| BF-4.9 | Sync troubleshooting and data reset | Reset sync data on the server, and see sync status. | Y / Y / OS / Y / — | [Chrome Help: Fix issues with sync][CHR-H] | High |
| BF-4.10 | Third-party sign-in prompts (FedCM UI) | The browser shows its own dialog for "Sign in with X". | Y / ? / ? / Y / — | [Chrome Help: third-party sign-in][CHR-H] | Medium |
| BF-4.11 | Export my data | Export the browser's personal data. | Y / P / P / ? / Arc (export tabs) | [Chrome Help: Export your data][CHR-H]; [Arc][ARC] | High |

**Implications:** A sync design must choose end-to-end encryption from the start. Brave, Vivaldi and Dia now advertise it, and Arc's shutdown shows users need their data exported. Profiles and workspaces must be distinct, documented layers.

## 5. Downloads, files, saving, printing and PDF

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-5.1 | Downloads UI [TS] | A download bubble or panel with progress, pause, resume, retry, open and show-in-folder. | Y / Y / Y / Y / all | [Chrome Help: Download a file][CHR-H]; [Safari Guide: Downloading][SAF-G] | High |
| BF-5.2 | Download location and ask-where-to-save [TS] | Choose the default folder, or ask each time. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-5.3 | Dangerous download warnings [TS] | Warn about or block malicious or uncommon files, with a "keep anyway" path. | Y / Y / Y / Y / all | [Edge features: SmartScreen][EDGE-F] | High |
| BF-5.4 | Save page [TS] | Save as HTML only, complete page, or a web archive. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-5.5 | Print and print preview [TS] | Print with preview, page range, layout, headers, background graphics and scale. | Y / Y / Y / Y / all | [Chrome Help: Print][CHR-H] | High |
| BF-5.6 | Save as PDF [TS] | Print to a PDF file. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-5.7 | Built-in PDF viewer [TS] | Open PDFs in a tab, with zoom, find, thumbnails, outline, rotate and print. | Y / Y / Y / Y / all | [Chrome Help: Manage PDFs][CHR-H] | High |
| BF-5.8 | PDF form fill, annotate and sign | Fill forms, draw, highlight, add text, add a signature, then save. | P / Y / P / Y / — | [Firefox: all features (PDF editor)][FX-F]; [Edge features (PDF Reader)][EDGE-F] | High |
| BF-5.9 | PDF image alt text (AI) | Generate alt text for images added to a PDF, on device. | N / Y / N / N / — | [Firefox AI controls (news)][FX-AI] | Low |
| BF-5.10 | PDF summarise and ask | Summarise a PDF, or answer questions about it, with AI. | Y (Gemini) / P / ? / Y / Brave Leo | [Edge features (PDF Reader)][EDGE-F] | High |
| BF-5.11 | Easy file picker [Diff] | Suggest recent downloads and screenshots in the upload dialog. | N / N / N / N / Opera "Easy Files" | [Opera features][OPERA-F] | High |

**Implications:** A PDF viewer that can edit is now expected, after Firefox and Edge shipped one. Izcal builds its own components, so a PDF renderer and editor is a large standalone component. The specs should name it explicitly.

## 6. Passwords, passkeys, autofill and payments

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-6.1 | Password manager [TS] | Save, fill, view, edit and delete passwords, with a manager page. | Y / Y / Y (Passwords app) / Y / all | [Chrome Help: Google Password Manager][CHR-H]; [Firefox: all features][FX-F] | High |
| BF-6.2 | Password generator [TS] | Suggest a strong password at sign-up. | Y / Y / Y / Y / all | [Edge features][EDGE-F]; [Firefox: all features][FX-F] | High |
| BF-6.3 | Breached and weak password alerts [TS] | Flag passwords that are compromised, reused or weak, with a change-password flow. | Y / Y / Y / Y / — | [Edge features (Password health)][EDGE-F]; [Chrome Help: compromised passwords][CHR-H] | High |
| BF-6.4 | Passkey manager [TS] | Create, store, sync and delete passkeys. | Y / P / Y / Y / — | [Chrome Help: Manage passkeys][CHR-H]; [Edge features][EDGE-F] | High |
| BF-6.5 | Password and passkey import/export | Import from CSV or another manager, and export. | Y / Y / Y / Y / — | [Chrome Help: Import or export passwords and passkeys][CHR-H] | High |
| BF-6.6 | Password manager PIN / device unlock | Require a PIN or biometric before showing or filling a password. | Y / Y (primary password) / Y / Y / — | [Chrome Help: Password Manager PIN][CHR-H] | High |
| BF-6.7 | Third-party password manager integration | Let an OS-level or extension password manager fill instead. | Y / Y / Y / Y / — | [Chrome Help: saved passwords in other apps][CHR-H] | Medium |
| BF-6.8 | Address autofill [TS] | Save and fill names, addresses, phone numbers and email addresses. | Y / Y / Y / Y / all | [Chrome Help: Fill out forms automatically][CHR-H] | High |
| BF-6.9 | Payment card autofill [TS] | Save cards and fill them at checkout, with CVC and biometric checks. | Y / P (regions) / Y / Y / — | [Chrome Help: payment info][CHR-H]; [Chrome features][CHR-F] | High |
| BF-6.10 | Virtual cards, BNPL and card benefits | Fill a virtual card number, offer pay-later options, and show card benefits. | Y / N / N / ? / — | [Chrome Help: virtual cards; buy now, pay later; card benefits][CHR-H] | High |
| BF-6.11 | Wallet payment sheet | Pay with a platform wallet. | Y (Google Pay) / N / Y (Apple Pay) / ? / — | [Safari Guide: Apple Pay][SAF-G]; [Chrome features][CHR-F] | High |
| BF-6.12 | Shopping: price tracking and insights | Track prices, show price history, and offer coupons. | Y / N / N / Y / Opera "Cashback" | [Chrome Help: price tracking][CHR-H]; [Opera features][OPERA-F] | High |
| BF-6.13 | Store reviews | Show review summaries for an online store. | Y / N / N / ? / — | [Chrome Help: Store reviews][CHR-H] | High |
| BF-6.14 | Crypto wallet [Diff] | A self-custody crypto wallet built into the browser. | N / N / N / N / Brave, Opera | [Brave features][BRAVE-F] | High |
| BF-6.15 | Creator rewards [Diff] | Opt-in private ads that pay a token to the user or to creators. | N / N / N / N / Brave Rewards | [Brave features][BRAVE-F] | High |

**Implications:** A credential store with passwords, passkeys, PIN gating, breach checks and import is table stakes. Breach checking needs a privacy-preserving lookup service, which is a dependency to plan for. Payment autofill without a wallet partner is P at best.

## 7. Privacy controls

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-7.1 | Private browsing window [TS] | A session that keeps no history, cookies or form data after it closes. | Y (Incognito) / Y / Y / Y (InPrivate) / all | [Chrome Help: Incognito][CHR-H]; [Edge features][EDGE-F]; [Safari Guide][SAF-G] | High |
| BF-7.2 | Lock private tabs with biometrics | Require Face ID, Touch ID or a PIN to reopen private tabs. | P (mobile) / ? / Y / ? / — | Not re-checked | Low |
| BF-7.3 | Private window over Tor [Diff] | A private window that routes traffic through Tor. | N / N / N / N / Brave | [Brave features][BRAVE-F] | High |
| BF-7.4 | Cross-site tracker blocking [TS] | Block known trackers by default, with levels such as Standard and Strict. | P / Y / Y / Y / Brave, Vivaldi, Orion, Dia | [Firefox: all features][FX-F]; [Safari Guide: cross-site tracking][SAF-G] | High |
| BF-7.5 | Cookie partitioning / Total Cookie Protection | Keep each site's third-party cookies in a separate jar. | P / Y / Y / ? / Brave | [Firefox features][FX-F] | High |
| BF-7.6 | Third-party cookie blocking setting [TS] | Allow or block third-party cookies globally, with exceptions per site. | Y / Y / Y / Y / all | [Chrome Help: cookies][CHR-H]; [Safari Guide: Managing cookies][SAF-G] | High |
| BF-7.7 | Fingerprinting protection | Block or randomise the signals that fingerprinting scripts read. | N / Y / Y / P / Brave (randomisation) | [Firefox: all features][FX-F]; [Brave features][BRAVE-F]; [Safari 26 (news)][SAF-26] | Medium |
| BF-7.8 | Tracking-parameter removal from URLs | Strip known click-ID parameters from links. | ? / Y (Strict) / Y (all browsing since Safari 26) / ? / Brave | [Safari 26 (news)][SAF-26] | Low |
| BF-7.9 | Privacy report / protections dashboard | Show counts of blocked trackers and who tried to track the user. | N / Y / Y / Y / Brave (Privacy Hub) | [Firefox: all features][FX-F]; [Safari Guide: who tried to track you][SAF-G]; [Brave features][BRAVE-F] | High |
| BF-7.10 | Per-site shields panel [Diff] | Toggle blocking for one site from the toolbar. | N / Y (shield icon) / P / P / Brave Shields | [Brave features][BRAVE-F] | High |
| BF-7.11 | Built-in ad blocking [Diff] | Block ads with filter lists, without an extension. | P (abusive-ad filter) / N / N / N / Brave, Vivaldi, Opera, Orion, Dia | [Chrome Help: Chrome's ad blocker][CHR-H]; [Vivaldi features][VIV-F]; [Opera features][OPERA-F]; [Orion][ORION] | High |
| BF-7.12 | Cookie banner handling [Diff] | Auto-reject or hide cookie consent banners. | N / P / N / N / Brave, Vivaldi, Opera | [Brave features][BRAVE-F]; [Vivaldi features][VIV-F] | High |
| BF-7.13 | Clear browsing data [TS] | Delete history, cookies, cache and passwords for a time range. Includes clear-on-exit. | Y / Y / Y / Y / all | [Chrome Help: Delete browsing data][CHR-H]; [Safari Guide][SAF-G] | High |
| BF-7.14 | Site data manager [TS] | View and delete storage per site. Includes "forget this site". | Y / Y / Y / Y / all | [Chrome Help: on-device site data][CHR-H] | High |
| BF-7.15 | Global Privacy Control / Do Not Track setting | Send an opt-out signal to sites. | N / Y / ? / ? / Brave | Not re-checked | Low |
| BF-7.16 | Zero telemetry / telemetry controls | Turn off usage statistics and crash reports, or ship with no telemetry at all. | Y (toggle) / Y (toggle) / OS / Y (toggle) / Orion (none) | [Chrome Help: usage statistics & crash reports][CHR-H]; [Orion][ORION] | High |
| BF-7.17 | IP masking in private browsing | Hide the user's IP from trackers through a relay. | ? / P / Y (iCloud Private Relay) / P (Secure Network) / — | [Edge features][EDGE-F] | Medium |
| BF-7.18 | Distraction control / element hiding | Hide distracting page elements, such as sign-in pop-ups, permanently. | N / N / Y (Safari 18+) / N / Orion, Brave (element picker) | Not re-checked | Low |

**Implications:** Tracker blocking and cookie partitioning on by default are now table stakes outside Chrome. Built-in ad blocking and cookie-banner handling set Brave, Vivaldi, Opera and Orion apart. Izcal must choose its default privacy posture and its filter-list licensing. The MIT-only rule matters here, because most filter lists carry other licences.

## 8. Security and safety UI

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-8.1 | Safe browsing interstitials [TS] | A full-page warning before a known phishing or malware site. | Y / Y / Y / Y (SmartScreen) / all | [Chrome Help: warnings about unsafe sites][CHR-H]; [Edge features][EDGE-F] | High |
| BF-8.2 | Enhanced protection modes | Opt-in stronger checks, such as real-time lookups or hardened JIT off on unfamiliar sites. | Y (Enhanced Safe Browsing) / N / OS (Lockdown Mode) / Y (Enhanced security mode) / — | [Edge features][EDGE-F] | High |
| BF-8.3 | Scareware / tech-support-scam blocker | Detect full-screen scam pages and exit them. | ? / N / N / Y / — | [Edge features][EDGE-F] | High |
| BF-8.4 | HTTPS-only / always use secure connections [TS] | Upgrade to HTTPS, and warn before loading HTTP. | Y / Y / Y / Y / all | [Chrome Help: Manage Chrome safety and security][CHR-H] | Medium |
| BF-8.5 | Secure DNS / DNS-over-HTTPS settings | Choose a DoH provider, or use the OS resolver. | Y / Y / OS / Y / Brave, Vivaldi | Not re-checked | Low |
| BF-8.6 | Certificate viewer [TS] | Inspect the certificate chain, validity dates and fingerprints from page info. | Y / Y / Y / Y / all | [Chrome Help: secure connection][CHR-H] | Medium |
| BF-8.7 | Certificate error interstitials [TS] | Warning pages for invalid, expired or mismatched certificates, with a guarded override. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-8.8 | Safety check | One page that checks updates, passwords, extensions and permissions. | Y / N / N / Y / — | [Chrome Help: Manage Chrome safety and security][CHR-H] | Medium |
| BF-8.9 | Unwanted software cleanup | Detect and remove software that hijacks settings or injects ads. | Y / N / N / ? / — | [Chrome Help: Remove unwanted software][CHR-H] | High |
| BF-8.10 | "Managed by your organisation" indicator [TS] | Show the user that enterprise policy controls the browser. | Y / Y / OS / Y / — | [Chrome Help: Check if your Chrome browser is managed][CHR-H] | High |
| BF-8.11 | Lookalike-domain warning | Warn when a domain imitates a well-known site. | Y / ? / ? / ? / — | Not re-checked | Low |
| BF-8.12 | Auto-update [TS] | Silent background updates, a relaunch prompt and an "About" page. | Y / Y / OS / Y / all | Not re-checked | Low |

**Implications:** Safe browsing needs a reputation data service. Izcal must build one, or choose a feed that the MIT-only rule allows. The licence terms of the major feeds are a gap to research. HTTPS-only and certificate UI are table stakes and cheap to ship.

## 9. Site permissions and page info

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-9.1 | Page info panel [TS] | Click the address-bar icon to see connection security, cookies, permissions and site settings. | Y / Y / Y / Y / all | [Chrome Help: secure connection][CHR-H] | Medium |
| BF-9.2 | Permission prompts [TS] | Ask for camera, microphone, location, notifications and similar, with allow, block and "this time only". | Y / Y / Y / Y / all | [Chrome Help: camera and microphone; location][CHR-H] | High |
| BF-9.3 | Quieter notification prompts | Hide prompts from abusive or unpopular sites, or require a user gesture first. | Y / Y / ? / Y / — | [Chrome Help: notifications][CHR-H] | Medium |
| BF-9.4 | Site settings manager [TS] | Global and per-site defaults for every permission, with exceptions. | Y / Y / Y / Y / all | [Chrome Help: site settings permissions][CHR-H] | High |
| BF-9.5 | Auto-revoke unused permissions | Remove permissions from sites the user has not visited for a while. | Y / ? / ? / ? / — | Not re-checked | Low |
| BF-9.6 | Device access choosers | Pickers for USB, serial, HID, Bluetooth and MIDI devices. | Y / P (MIDI, serial) / N / Y / — | [Chrome Help: Bluetooth; USB, Serial, HID; MIDI][CHR-H] | High |
| BF-9.7 | Popup blocker [TS] | Block unrequested pop-ups, with an address-bar indicator and an allow option. | Y / Y / Y / Y / all | [Safari Guide: Blocking pop-ups][SAF-G] | High |
| BF-9.8 | Autoplay controls | Block audio or all media autoplay, globally or per site. | P / Y / Y / Y / — | Not re-checked | Low |
| BF-9.9 | In-use indicators | Show when the camera, microphone, location or screen sharing is active, with a way to stop it. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-9.10 | Screen-share picker | Choose a tab, window or screen to share, with a "sharing this tab" bar. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-9.11 | Per-site JavaScript, images and content toggles | Allow or block scripts, images and other content per site. | Y / P / Y / Y / Vivaldi | Not re-checked | Low |
| BF-9.12 | Protocol handler and external app prompts | Ask "Open in app X?" for custom URL schemes. | Y / Y / Y / Y / all | Not re-checked | Low |

**Implications:** One permission model and one prompt framework should serve every capability. Prompt design (one-time grants, quiet UI, auto-revoke) is where browsers now compete. Izcal should spec it once and centrally.

## 10. Content, reading and page tools

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-10.1 | Find in page [TS] | Search text with highlight-all, match case and whole word. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-10.2 | Page zoom [TS] | Zoom per site, with a default zoom level and pinch zoom. | Y / Y / Y / Y / Vivaldi (status-bar slider) | [Chrome Help: zoom][CHR-H]; [Vivaldi features][VIV-F] | High |
| BF-10.3 | Text-only zoom and minimum font size | Enlarge text without scaling the layout. | P / Y / Y / P / — | Not re-checked | Low |
| BF-10.4 | Reader mode [TS] | Show a simplified view of an article with font, size and theme options. | Y (side panel) / Y / Y / Y / Brave "Speedreader", Vivaldi | [Chrome Help: reading mode][CHR-H]; [Edge features][EDGE-F]; [Brave features][BRAVE-F] | High |
| BF-10.5 | Read aloud / listen to page | Text-to-speech for a page, with speed, voice and highlighting. | Y / Y (Narrate) / Y / Y / — | [Chrome Help: Listen to this page][CHR-H] | High |
| BF-10.6 | Page translation [TS] | Translate a whole page, or a selection, with an "always translate" option. | Y / Y (on device) / Y / Y / Brave, Vivaldi, Opera, Orion (Kagi) | [Chrome features][CHR-F]; [Firefox: all features][FX-F]; [Safari Guide][SAF-G] | High |
| BF-10.7 | Spellcheck [TS] | Underline misspellings in fields, with suggestions and language choice. | Y / Y / OS / Y / all | [Chrome Help: spell check][CHR-H] | High |
| BF-10.8 | Cloud / enhanced spellcheck and grammar | Server-side spelling and grammar suggestions. | Y / N / OS / ? / — | Not re-checked | Low |
| BF-10.9 | Share page [TS] | The OS share sheet, copy link, email link. | Y / Y / Y / Y / all | [Chrome Help: Share tabs][CHR-H] | High |
| BF-10.10 | Link to highlighted text | Copy a link that scrolls to and highlights the selected text. | Y / ? / ? / Y / — | [Chrome Help: link to quotes & text][CHR-H] | High |
| BF-10.11 | QR code for page | Generate a QR code for the current URL. On mobile, scan a QR code to open a site. | Y / P / N / Y / — | Not re-checked | Low |
| BF-10.12 | Screenshot tool | Capture the visible area, a selected region or the full page, then annotate and copy. | P / Y / N / Y (Web capture) / Vivaldi "Capture", Opera "Snapshot" | [Vivaldi features][VIV-F]; [Opera features][OPERA-F] | High |
| BF-10.13 | Eyedropper | Pick a colour from the page and copy its hex code. | N / Y / N / N / — | [Firefox: all features][FX-F] | High |
| BF-10.14 | Notes in sidebar [Diff] | Take notes while browsing, attached to pages, and synced. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-10.15 | Highlights / page key facts | Show key facts from a page (directions, a summary) in the address bar. | N / N / Y / N / — | Not re-checked | Low |
| BF-10.16 | Request desktop site (mobile) [TS on mobile] | Switch a mobile user agent to the desktop one, per site or globally. | Y / Y / Y / Y / all | [Chrome Help: desktop mode on mobile][CHR-H] | High |
| BF-10.17 | Dark mode for web content | Force dark rendering on sites that have no dark theme. | P (flag) / N / N / P / Opera, Vivaldi | [Chrome Help: Dark mode][CHR-H] | Medium |
| BF-10.18 | View page source [TS] | Show the page's source code in a tab. | Y / Y / Y (Develop menu) / Y / all | Not re-checked | Low |

**Implications:** Reader mode, translation and read aloud form one "content extraction" pipeline and should share it. Firefox shows that on-device translation is a privacy differentiator. Izcal's no-third-party-models rule means it must train or build its own translation models, which is a large data-licensing question.

## 11. Media

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-11.1 | Picture-in-picture [TS] | Pop a video into an always-on-top floating window. | Y / Y / Y / Y / Vivaldi, Opera "Video Popout" | [Firefox: all features][FX-F]; [Chrome features][CHR-F]; [Vivaldi features][VIV-F] | High |
| BF-11.2 | Automatic PiP on tab switch | Move a playing video into PiP when the user leaves its tab. | Y / ? / Y / ? / Arc, Zen | Not re-checked | Low |
| BF-11.3 | Global media controls | A toolbar hub for play/pause across tabs, plus OS media-key integration. | Y / P (OS) / OS / Y / — | [Chrome Help: Control your music, videos][CHR-H] | High |
| BF-11.4 | Casting [TS on Chrome] | Cast a tab, the screen or media to a TV or speaker. | Y (Cast) / N / Y (AirPlay) / Y / — | [Chrome features][CHR-F]; [Chrome Help: Cast][CHR-H] | High |
| BF-11.5 | Live captions for media | Generate captions on device, and optionally translate them. | Y / N / OS / P / — | [Chrome features][CHR-F] | High |
| BF-11.6 | Media playlist / offline play [Diff] | Queue videos and audio from sites for later playback. | N / N / N / N / Brave "Playlist" | [Brave features][BRAVE-F] | High |
| BF-11.7 | Built-in music player [Diff] | Spotify, Deezer or Apple Music in the sidebar, controlled from the browser. | N / N / N / N / Opera | [Opera features][OPERA-F] | High |
| BF-11.8 | Volume booster and video enhancement [Diff] | Raise audio volume above 100%, and sharpen video ("Lucid Mode"). | N / N / N / N / Opera | [Opera features][OPERA-F] | High |
| BF-11.9 | DRM content playback [TS] | Play protected streaming video, with a toggle for the content decryption module. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-11.10 | Microphone volume adjustment | Adjust the microphone input level from the browser. | Y / N / N / ? / — | [Chrome Help: microphone volume][CHR-H] | High |

**Implications:** Users expect DRM playback. Izcal builds everything itself, so DRM needs a licensed CDM, which is a business decision, not an engineering one. Casting needs protocol work for each ecosystem.

## 12. Performance and resource controls

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-12.1 | Tab sleeping / memory saver [TS] | Discard inactive tabs to free memory, with levels and a visual marker. | Y / P (unload on low memory) / N (silent) / Y (Sleeping tabs) / Vivaldi "Hibernation" | [Chrome Help: Performance][CHR-P]; [Vivaldi features][VIV-F] | High |
| BF-12.2 | Always-keep-active site list | Exclude chosen sites from tab discarding. | Y / ? / N / Y / — | [Chrome Help: Performance][CHR-P] | High |
| BF-12.3 | Energy / battery saver | Throttle background work and frame rates on battery. | Y / N / N / Y (Efficiency mode) / Opera "Battery Saver" | [Chrome Help: Performance][CHR-P]; [Opera features][OPERA-F] | High |
| BF-12.4 | Performance issue alerts | Notify the user about heavy tabs and suggest deactivating them. | Y / N / N / Y / — | [Chrome Help: Performance][CHR-P] | High |
| BF-12.5 | Browser task manager | Show CPU, memory and network use per tab, extension and process, with end process. | Y / Y (about:processes) / N / Y / Vivaldi | Not re-checked | Low |
| BF-12.6 | Preloading settings | Choose standard or extended preloading of likely next pages. | Y / ? / ? / Y / — | [Chrome Help: Performance][CHR-P] | High |
| BF-12.7 | Startup boost / background running | Keep a light process running so launch is faster, with an off switch. | ? / N / N / Y / — | Not re-checked | Low |
| BF-12.8 | Hardware acceleration toggle | Turn GPU acceleration off to work around driver problems. | Y / Y / N / Y / — | Not re-checked | Low |

**Implications:** Memory saver, energy saver and a task manager are expected on desktop. They require the process model and scheduler to support discard, freeze and throttle states with visible UI. This should sit in the substrate specs, not be added later.

## 13. Extensions, themes and customisation

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-13.1 | Extension support [TS] | Install add-ons that change pages and the UI. | Y / Y / Y / Y / Brave, Vivaldi, Opera, Orion, Zen, Dia | [Chrome Help: extensions][CHR-H]; [Firefox: all features][FX-F]; [Safari Guide: extensions][SAF-G] | High |
| BF-13.2 | Extension store [TS] | An in-browser catalogue, with install, update, ratings and reviews. | Y / Y (AMO) / Y (App Store) / Y / Opera add-ons | [Chrome features][CHR-F] | High |
| BF-13.3 | Extension management UI [TS] | Enable, disable, remove, pin to toolbar, set permissions and site access, and set shortcuts. | Y / Y / Y / Y / all | [Chrome Help: Install and manage extensions][CHR-H] | High |
| BF-13.4 | Cross-ecosystem extension support [Diff] | Run extensions written for other browsers' stores. | N / N / N / N / Orion (Chrome + Firefox + Safari), Vivaldi/Brave/Opera (Chrome store) | [Orion][ORION]; [Vivaldi features][VIV-F] | High |
| BF-13.5 | Legacy (MV2) extension support [Diff] | Keep running Manifest V2 ad blockers. | N / Y / N / N / Brave (some) | [Brave features][BRAVE-F] | Medium |
| BF-13.6 | Extensions on mobile | Install extensions on phones. | N / Y (Android) / Y (iOS) / ? / Orion (iOS) | Not re-checked | Low |
| BF-13.7 | AI-generated extensions [Diff] | Describe an extension in plain language and have one built. | N / N / Y (Safari 27, reported) / N / Arc "Boosts" (historic) | [Safari 27 (news)][SAF-27] | Low |
| BF-13.8 | Themes [TS] | Apply downloadable or colour themes to the browser chrome. | Y / Y / N / Y / Vivaldi, Opera (with sounds) | [Firefox: all features][FX-F]; [Vivaldi features][VIV-F] | High |
| BF-13.9 | AI theme generator | Generate a theme from a text description. | Y / N / N / Y / — | [Chrome Help: themes with AI][CHR-H]; [Edge features][EDGE-F] | High |
| BF-13.10 | Light/dark mode for the browser UI [TS] | Follow the OS appearance, or force light or dark. | Y / Y / Y / Y / all | [Chrome Help: Dark mode][CHR-H] | High |
| BF-13.11 | Toolbar customisation | Add, remove and rearrange toolbar buttons. | Y / Y / Y / Y / Vivaldi (any toolbar) | [Chrome Help: Customize toolbar][CHR-H]; [Safari Guide: Customizing the window][SAF-G] | High |
| BF-13.12 | Deep UI customisation / mods [Diff] | Change layout, CSS and behaviour of the browser itself. | N / P (userChrome, unsupported) / N / N / Vivaldi, Zen "Mods", Orion | [Zen][ZEN]; [Vivaldi features][VIV-F]; [Orion][ORION] | High |
| BF-13.13 | Keyboard shortcuts [TS] | A documented set of shortcuts and gestures. | Y / Y / Y / Y / all | [Safari Guide: Keyboard shortcuts and gestures][SAF-G] | High |
| BF-13.14 | Remappable keyboard shortcuts [Diff] | Edit any shortcut. | N / N / OS / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-13.15 | Command chains / macros [Diff] | One keystroke or gesture runs a sequence of browser commands. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-13.16 | Peripheral lighting integration [Diff] | Sync RGB lighting with the browser's state. | N / N / N / N / Vivaldi (Razer Chroma), Opera GX | [Vivaldi features][VIV-F] | High |
| BF-13.17 | Experimental feature flags / labs | Let users opt into unfinished features. | Y (chrome://flags) / Y (Firefox Labs) / Y (Feature Flags) / Y / — | [Chrome Help: Chrome flags; Test experimental features][CHR-H] | High |
| BF-13.18 | Language settings [TS] | Choose the UI language, preferred content languages and spellcheck languages. | Y / Y / OS / Y / all | [Chrome Help: Translate pages and change languages][CHR-H] | High |
| BF-13.19 | Built-in game [Diff] | An offline game, such as Chrome's dinosaur or Vivaldia. | Y / N / N / Y (Surf) / Vivaldi | [Vivaldi features][VIV-F] | Medium |
| BF-13.20 | Touch Bar / hardware control customisation | Edit Mac Touch Bar buttons. | Y / Y / Y / ? / — | [Chrome Help: Mac Touch Bar][CHR-H] | High |

**Implications:** Extension compatibility is the largest table-stakes decision. Users expect WebExtensions-compatible add-ons and a store. Izcal must choose an API surface (Chrome-compatible, Firefox-compatible, or Orion's "all of them") and decide whether it runs its own store or a review process.

## 14. Accessibility

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-14.1 | Screen reader support [TS] | An accessibility tree exposed to the OS screen readers. | Y / Y / Y / Y / all | [Chrome features (Accessibility)][CHR-F] | High |
| BF-14.2 | High contrast / forced colours [TS] | Honour the OS high-contrast mode, and offer a browser-level contrast setting. | Y / Y / OS / Y / — | [Chrome features][CHR-F] | High |
| BF-14.3 | Focus highlight and caret browsing | Highlight the focused element, and navigate with a text cursor. | Y / Y / N / Y / Vivaldi (spatial navigation) | [Chrome features][CHR-F]; [Vivaldi features][VIV-F] | High |
| BF-14.4 | Keyboard-only browsing [TS] | Every browser control is reachable from the keyboard. | Y / Y / Y / Y / Vivaldi | [Chrome features][CHR-F] | High |
| BF-14.5 | AI image descriptions | Generate alt text for unlabelled images for screen-reader users. | Y / P (PDF only) / OS / ? / — | [Chrome features][CHR-F] | High |
| BF-14.6 | Live caption (see BF-11.5) | On-device captions for any audio. | Y / N / OS / P / — | [Chrome features][CHR-F] | High |
| BF-14.7 | Magnifier support / full-page zoom | Work with OS magnifiers, and scale the whole page. | Y / Y / Y / Y / all | [Chrome features][CHR-F] | High |
| BF-14.8 | Reduced motion and default fonts | Font family and size settings, and honouring the OS reduce-motion setting. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-14.9 | Talk to type / voice dictation | Dictate into text fields. | Y / N / OS / ? / — | [Chrome Help: talk to type][CHR-H] | High |

**Implications:** Accessibility is table stakes and is often regulated. AI image descriptions and live caption are now the differentiating layer. They need on-device models that Izcal must build or train itself.

## 15. AI features and their off switches

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-15.1 | AI assistant panel | A chat assistant beside the page that can read the current page. | Y (Gemini in Chrome) / Y (user's choice of chatbot) / N / Y (Copilot) / Brave Leo, Opera AI, Dia, Comet, ChatGPT Atlas | [Chrome Help: Use Gemini in Chrome][CHR-H]; [Firefox: all features][FX-F]; [Edge features][EDGE-F]; [Opera features][OPERA-F] | High |
| BF-15.2 | Choice of AI provider [Diff] | Pick which AI model or chatbot the browser uses. | N / Y / N / N / Opera (Opera AI, ChatGPT, Gemini), Brave Leo (models) | [Firefox features][FX-F]; [Opera features][OPERA-F] | High |
| BF-15.3 | Page summarisation | Summarise the current page or a PDF. | Y / Y / Y (Apple Intelligence) / Y / Brave, Opera, Dia | [Edge features][EDGE-F]; [Firefox: all features][FX-F] | High |
| BF-15.4 | Ask about multiple tabs / compare tabs | Answer questions using several open tabs at once. | Y / ? / ? / Y / Dia | [Chrome Help: ask about your tabs][CHR-H]; [Edge features][EDGE-F]; [Dia][DIA] | High |
| BF-15.5 | AI tab organiser | Suggest or create tab groups by topic. | Y / Y / Y (Safari 27) / ? / Arc, Zen | [Firefox AI controls (news)][FX-AI]; [Safari 27 (news)][SAF-27] | Medium |
| BF-15.6 | Agentic browsing [Diff] | The browser carries out multi-step tasks on websites for the user. | Y ("auto browse") / N / N / Y (Copilot Actions, preview) / Opera, Dia, Comet, ChatGPT Atlas | [Chrome Help: auto browse][CHR-H]; [Edge Copilot Actions (news)][EDGE-CP] | Medium |
| BF-15.7 | AI history search | Search history with natural-language queries. | Y / ? / ? / Y / Dia | [Chrome Help: history with AI][CHR-H] | High |
| BF-15.8 | Writing help | Draft or rewrite text in page fields. | Y ("Help me write") / N / OS (Writing Tools) / Y / Opera, Dia | [Chrome Help: Help me write][CHR-H] | High |
| BF-15.9 | AI memory / personalisation | The assistant remembers the user's interests across sessions, with view and delete controls. | ? / N / N / Y (Copilot Memory) / Dia, ChatGPT Atlas | [Edge features][EDGE-F]; [Dia][DIA] | High |
| BF-15.10 | AI access to personal data sources | The assistant reads mail, calendar and files to answer questions. | P / N / N / Y / Dia (morning brief, meetings) | [Edge features][EDGE-F]; [Dia][DIA] | High |
| BF-15.11 | Image generation and editing | Generate images, or edit images from the web, with AI. | Y (edit) / N / N / Y / Opera | [Chrome Help: edit images][CHR-H]; [Edge features][EDGE-F]; [Opera features][OPERA-F] | High |
| BF-15.12 | Voice / live AI conversation and screen sharing | Talk to the assistant in real time and share the screen with it. | Y ("Go Live", share screen) / N / N / Y / — | [Chrome Help: Go Live with Gemini][CHR-H] | High |
| BF-15.13 | Virtual try-on | Try clothes on a photo of the user while shopping. | Y / N / N / N / — | [Chrome Help: Try on clothes virtually][CHR-H] | High |
| BF-15.14 | On-device models and their management | Download, use and delete local models for AI features. | Y ("Manage on-device Generative AI models") / Y (on-device AI section) / OS / ? / — | [Chrome Help: on-device models][CHR-H]; [Firefox AI controls (news)][FX-AI] | High |
| BF-15.15 | AI master off switch [Diff→TS] | One control that blocks all current and future AI features and their prompts. | P (per-feature) / Y ("Block AI enhancements", Firefox 148, 24 February 2026) / OS / P / Vivaldi (no AI shipped) | [Firefox AI controls (news)][FX-AI] | Low |
| BF-15.16 | AI-generated content creation from browsing [Diff] | Turn gathered tabs into a report or slide deck. | N / N / N / N / Dia ("Decks", synthesis reports) | [Dia][DIA] | High |
| BF-15.17 | Agent / automation server for external AI tools | Let outside AI agents inspect and drive the browser (DOM, network, console, screenshots). | P (DevTools MCP) / ? / Y (Safari 27, reported) / ? / — | [Safari 27 MCP (news)][SAF-27-MCP] | Low |
| BF-15.18 | Link previews with AI summary | Hover over a link to see a preview and key points. | N / Y / N / ? / — | [Firefox AI controls (news)][FX-AI] | Low |

**Implications:** This is the fastest-moving group. Izcal must decide whether AI is in its product at all. If it is, it must decide on-device versus cloud, provider choice, permission scoping for agentic actions, and a single kill switch. Firefox's February 2026 "Block AI enhancements" control shows that an off switch is now a competitive feature. Agentic browsing also needs its own security model: prompt-injection isolation, and confirmation before purchases or sends.

## 16. Built-in apps and services

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-16.1 | Sidebar / side panel [TS] | A docked panel for bookmarks, history, reading list, AI and tools. | Y / Y / Y / Y / Brave, Vivaldi, Opera | [Chrome Help: side panel][CHR-H]; [Firefox: all features][FX-F]; [Brave features][BRAVE-F] | High |
| BF-16.2 | Web panels / sidebar apps [Diff] | Pin any website as a narrow panel in the sidebar. | N / N / N / P / Vivaldi, Opera, Brave | [Vivaldi features][VIV-F]; [Brave features][BRAVE-F] | High |
| BF-16.3 | Integrated messengers [Diff] | WhatsApp, Telegram, Messenger, Discord, Slack and similar in the sidebar. | N / N / N / N / Opera | [Opera features][OPERA-F] | High |
| BF-16.4 | Mail client [Diff] | A built-in email client that handles several accounts. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-16.5 | Calendar [Diff] | A built-in calendar. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-16.6 | Feed reader (RSS/Atom) [Diff] | Subscribe to feeds, with a chronological reader. | N / N / N / N / Vivaldi | [Vivaldi features][VIV-F] | High |
| BF-16.7 | Built-in VPN / secure proxy | Route browser traffic through the vendor's VPN or proxy. | N / Y / P (iCloud Private Relay) / Y (Secure Network) / Opera (free), Brave (paid, system-wide), Vivaldi (Proton VPN) | [Firefox: all features][FX-F]; [Edge features][EDGE-F]; [Opera features][OPERA-F]; [Vivaldi features][VIV-F] | High |
| BF-16.8 | Proxy settings | Configure HTTP, SOCKS or PAC proxies. | OS / Y / OS / OS / — | Not re-checked | Low |
| BF-16.9 | Video calling [Diff] | Private video calls built into the browser. | N / N / N / N / Brave Talk | [Brave features][BRAVE-F] | High |
| BF-16.10 | Google / productivity service shortcuts | Quick access to Drive files and services from the address bar or a sidebar. | Y / N / N / Y (Microsoft 365) / Opera | [Chrome features][CHR-F]; [Opera features][OPERA-F] | High |
| BF-16.11 | Live work destinations and meeting prep [Diff] | Work items appear in the tab bar, and meeting context opens before a call. | N / N / N / N / Dia | [Dia][DIA] | High |
| BF-16.12 | Remote desktop access | Access another computer from the browser. | Y / N / N / N / — | [Chrome Help: Remote Desktop][CHR-H] | High |
| BF-16.13 | Location and directions cards | Show location information and directions for places mentioned on a page. | Y / N / ? / ? / — | [Chrome Help: location info & directions; Chrome Actions][CHR-H] | High |
| BF-16.14 | Phone-number click-to-call | Dial from a page through a linked phone service. | Y / N / OS / ? / — | [Chrome Help: one-click dialing][CHR-H] | Medium |

**Implications:** These rows are optional differentiators. Vivaldi shows that a mail, calendar and feed suite is buildable inside a browser. Izcal should decide its identity: lean core (Chrome, Safari), or a suite (Vivaldi, Opera). A VPN implies running a network service, which is an operational commitment.

## 17. Web apps and operating-system integration

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-17.1 | Install site as app [TS on Chromium] | Install a site as an app with its own window, icon and launcher entry. | Y / P (Windows "taskbar tabs") / Y (Add to Dock) / Y / — | [Chrome features][CHR-F]; [Chrome Help: web apps][CHR-H] | High |
| BF-17.2 | Web app management | List, uninstall and change settings of installed apps. Includes "open links in app". | Y / P / Y / Y / — | [Chrome Help: Use web apps][CHR-H] | Medium |
| BF-17.3 | Site shortcuts on desktop | Create a desktop shortcut for a website. | Y / ? / Y / Y / — | [Chrome Help: shortcuts for websites][CHR-H] | High |
| BF-17.4 | Default browser setting and prompt [TS] | Set the browser as the OS default, with a prompt in the app. | Y / Y / Y / Y / all | [Chrome Help: default browser][CHR-H]; [Safari Guide: default browser][SAF-G] | High |
| BF-17.5 | OS notifications for web notifications [TS] | Deliver site notifications through the OS notification centre. | Y / Y / Y / Y / all | [Chrome Help: notifications][CHR-H] | High |
| BF-17.6 | OS share sheet integration [TS] | Share to and from other apps. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-17.7 | Handoff / continuity | Continue browsing on another device of the same OS family. | P / N / Y / ? / — | Not re-checked | Low |
| BF-17.8 | Mobile widgets and home-screen shortcuts | Search widgets, quick actions and an "add to home screen" option. | Y / Y / Y / Y / — | [Chrome Help: Add Chrome to your iPhone's dock][CHR-H] | Medium |
| BF-17.9 | In-car browser | Use the browser on a car's built-in display. | Y (Google built-in) / N / N / N / — | [Chrome Help: Chrome in your car][CHR-H] | High |
| BF-17.10 | Password autofill in other apps (mobile) | The browser's password store fills native apps. | Y / ? / Y / Y / — | [Chrome Help: saved passwords in other apps][CHR-H] | High |
| BF-17.11 | Default app installs from the browser (iOS EU) | Install third-party apps from the browser, in regions where law allows it. | Y / ? / N / ? / — | [Chrome Help: third-party apps for iPhone][CHR-H] | High |
| BF-17.12 | Calendar event add (mobile) | Add events detected on a page to the OS calendar. | Y (iOS) / ? / OS / ? / — | [Chrome Help: Add event functionality][CHR-H] | High |

**Implications:** Installing a site as an app is expected by Chromium users and is a key piece for Izcal's "app host" spec. OS integration differs per platform, and each path needs its own spec line.

## 18. Developer and power-user entry points

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-18.1 | Developer tools [TS] | Inspector, console, network, debugger, performance, application storage and similar. | Y / Y / Y (enabled from the Develop menu) / Y / all | Not re-checked | Low |
| BF-18.2 | DevTools entry points [TS] | F12, Ctrl/Cmd+Opt+I, "Inspect" in the context menu, and a menu item. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-18.3 | Responsive design / device mode | Emulate screen sizes and touch input. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-18.4 | Report a broken site / send feedback | Send a compatibility report or feedback from the browser. | Y / Y / Y (Safari 26) / Y / — | [Chrome Help: Report an issue][CHR-H]; [Safari 26 (news)][SAF-26] | Medium |
| BF-18.5 | Internal status pages | about: or chrome:// pages for version, GPU, network and similar. | Y / Y / P / Y / — | Not re-checked | Low |
| BF-18.6 | Remote debugging and automation | Drive the browser from external tools with a debugging protocol, WebDriver, or an MCP server for AI agents. | Y / Y / Y / Y / — | [Safari 27 MCP (news)][SAF-27-MCP] | Low |
| BF-18.7 | Beta / preview channels | Optional pre-release builds. | Y / Y / Y (Technology Preview) / Y / Zen Twilight | [Chrome Help: Join Chrome Beta][CHR-H]; [Zen][ZEN] | High |

**Implications:** DevTools and their entry points are table stakes for a credible browser. Izcal already has a first-principles DevTools skill. The MCP and automation surface is now a product feature, because Safari 27 ships one.

## 19. Setup, onboarding and maintenance

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-19.1 | Import from other browsers [TS] | Import bookmarks, history, passwords, settings and extensions. | Y / Y / Y / Y / Brave, Dia (from Arc) | [Chrome Help: Import bookmarks & settings][CHR-H]; [Brave features][BRAVE-F]; [Arc][ARC] | High |
| BF-19.2 | First-run onboarding [TS] | A welcome flow for sign-in, import, default browser and privacy choices. | Y / Y / Y / Y / all | [Chrome Help: Get started][CHR-H] | Medium |
| BF-19.3 | What's new page | A page that shows the features in a new version after an update. | Y / Y / ? / Y / Vivaldi | Not re-checked | Low |
| BF-19.4 | Reset / refresh settings [TS] | Restore default settings, and optionally keep data. | Y / Y (Refresh) / N / Y / — | [Chrome Help: Reset Chrome settings][CHR-H] | High |
| BF-19.5 | Crash reporting with consent [TS] | Send crash dumps automatically, with an opt-out. | Y / Y / OS / Y / — | [Chrome Help: reporting errors & crashes][CHR-H] | High |
| BF-19.6 | Troubleshooting help | Built-in fixes for loading errors and slowness, and a safe mode with extensions disabled. | Y / Y (Troubleshoot Mode) / N / Y / — | [Chrome Help: Fix problems][CHR-H] | Medium |
| BF-19.7 | In-browser help / AI help | Help answered inside the browser. | Y ("Ask Google Help") / ? / ? / ? / — | [Chrome Help: Ask Google Help][CHR-H] | Medium |
| BF-19.8 | Lost-device lock and erase | Lock or wipe a lost device that has the browser signed in. | Y / N / OS / ? / — | [Chrome Help: Lock or erase lost phone][CHR-H] | Medium |

**Implications:** Import is how users switch browsers, and the Arc-to-Dia migration shows how important it is. Izcal needs importers for each source browser's on-disk formats, written only from black-box observation.

## 20. Enterprise, family and special modes

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-20.1 | Enterprise policies [TS for business] | Admin-set policies with a viewer page that shows them. | Y / Y / Y (MDM) / Y / Dia for Work | [Chrome Help: managed browser][CHR-H]; [Dia][DIA] | Medium |
| BF-20.2 | Kiosk mode | A locked-down full-screen single-site mode. | Y / Y / N / Y / — | Not re-checked | Low |
| BF-20.3 | Parental controls / family safety | Supervised accounts, site filtering and time limits. | P (Family Link) / N / OS (Screen Time) / P / — | Not re-checked | Low |
| BF-20.4 | Kids mode | A child-safe browsing mode. | N / N / N / Retired (2025, not re-checked) / — | Not re-checked | Low |
| BF-20.5 | Data-saver / lite mode (mobile) | Compress or reduce data on slow networks. | Retired / N / N / N / Opera Mini, Brave Speedreader (reduces data) | [Brave features][BRAVE-F] | Medium |
| BF-20.6 | Shared device / multi-user | Per-user browser data on shared computers. | Y (profiles) / Y / OS / Y / Vivaldi | [Vivaldi features][VIV-F] | High |

**Implications:** Enterprise policy and kiosk mode open managed-fleet markets. Parental controls on desktop are now mostly left to the OS. Izcal should state what it defers to the OS.

## 21. Mobile-specific product features

| # | Feature | What it is | Shipped by (Chrome / Firefox / Safari / Edge / others) | Source | Conf. |
|---|---|---|---|---|---|
| BF-21.1 | Tab grid / switcher [TS] | A thumbnail grid of open tabs, with groups and swipe to close. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-21.2 | Swipe gestures between tabs and back [TS] | Swipe on the toolbar to switch tabs, and swipe from the edge to go back. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-21.3 | Pull to refresh [TS] | Pull down on a page to reload it. | Y / Y / Y / Y / all | Not re-checked | Low |
| BF-21.4 | Offline page saving | Download a page to read later without a network. | Y / ? / Y (Reading List) / ? / Brave Playlist | [Chrome Help: Read pages later & offline][CHR-H] | High |
| BF-21.5 | Mobile privacy hub | A mobile dashboard of blocked trackers. | N / ? / Y / ? / Brave | [Brave features][BRAVE-F] | High |
| BF-21.6 | Camera search / QR scan | Search with the camera, and scan QR codes from the browser. | Y / Y / OS / Y / — | [Chrome features (Lens translation)][CHR-F] | Medium |

**Implications:** Izcal is desktop-first. These rows record what a later mobile build must ship, so that the core architecture does not rule them out.

## 22. Coverage map for the feature classes in the brief

| Brief item | Rows |
|---|---|
| Windows, tabs, tab groups, vertical tabs, split view, workspaces | BF-1.1–1.10 |
| Profiles, guest mode, private browsing | BF-4.1–4.2, BF-7.1–7.3 |
| Address bar, search engines, suggestions | BF-2.1–2.17 |
| Bookmarks, reading list, history, new tab page, start pages | BF-3.1–3.14 |
| Downloads, save page, print, PDF | BF-5.1–5.11 |
| Session restore, crash recovery | BF-1.13–1.16 |
| Sidebar, web panels, notes, mail/calendar/feeds | BF-16.1–16.6, BF-10.14 |
| Passwords, passkeys, autofill, payments | BF-6.1–6.13 |
| Sync, send tab | BF-4.4–4.7 |
| Translation, reader mode, read aloud, find, zoom, spellcheck | BF-10.1–10.8 |
| Page info, permissions, certificates, cookies and site data | BF-9.*, BF-8.6–8.7, BF-7.6, BF-7.13–7.14 |
| Tracking, fingerprinting, cookie banners, HTTPS-only, DoH, safe browsing, breach alerts | BF-7.4–7.12, BF-8.1–8.5, BF-6.3 |
| Popups, notifications, autoplay | BF-9.3, BF-9.7–9.8 |
| Memory saver, energy saver, task manager | BF-12.1–12.5 |
| Screenshots, share, QR | BF-10.9–10.12 |
| Casting, PiP, media controls | BF-11.1–11.5 |
| Extensions, themes, customisation, shortcuts | BF-13.* |
| Accessibility | BF-14.* |
| Developer tools entry points | BF-18.* |
| AI features and off switches | BF-15.* |
| VPN / proxy | BF-16.7–16.8 |
| Import, onboarding, default-browser prompt | BF-19.1–19.2, BF-17.4 |
| Parental controls, kiosk | BF-20.2–20.4 |

## Gaps in this inventory

- **Search budget.** This pass opened 13 primary feature pages and indexes, plus three news searches. It did not open individual help articles for most rows. Rows marked "Not re-checked" (48 of 270) rely on the researcher's knowledge of shipping browsers up to mid-2026. A follow-up should check them against vendor help pages.
- **Shipped-by columns are not individually verified.** Where a cited source covers only one vendor, the other columns come from prior knowledge. The **?** cells are honest unknowns. These matter most: Chrome vertical tabs (BF-1.7), Firefox split view (BF-1.8), Edge send-tab (BF-4.6), and Firefox's passkey storage (BF-6.4).
- **The Firefox help centre did not load.** The support.mozilla.org index returned an error. The Firefox rows rest on the firefox.com feature pages and news coverage. Firefox AI Controls ("Block AI enhancements", Firefox 148, 24 February 2026) was confirmed only through news (BleepingComputer, Linuxiac), not a Mozilla page.
- **Safari 27 (autumn 2026).** Notify Me appears in Apple's own Safari Guide table of contents. AI tab grouping, the start-page changes, AI-built extensions and the MCP server were confirmed only by MacRumors, iThinkDiff and iPhoneSoft.fr reports. The iOS Safari guide could not be read (it was truncated).
- **Edge.** Copilot Actions and Journeys come from news (Winaero) plus the Edge features page. The status of Edge Collections, Kids Mode (believed retired in 2025) and Microsoft Editor was not verified.
- **Chrome AI rows.** The Chrome Help index confirms that the articles exist ("auto browse", "Go Live", "Manage on-device Generative AI models"). Availability is limited by region and account, and each article's text was not read.
- **Not covered by vendor pages read:** Opera GX gaming features (CPU/RAM limiters, GX Corner), Opera Neon (agentic), Perplexity Comet and ChatGPT Atlas details, Samsung Internet, DuckDuckGo browser (Duck Player, Fire button), Firefox Focus, Mullvad Browser and Tor Browser. These are named only where they set a pattern.
- **Licensing questions raised but not researched:** filter-list licences (BF-7.11), safe-browsing data feeds (BF-8.1), the DRM CDM (BF-11.9) and translation and caption model training data (BF-10.6, BF-14.5). The MIT-only rule makes each of these a build-or-license decision. None was researched here.
- **No engine source was opened.** Per M-1 part 3, every claim comes from prose pages: vendor feature pages, help indexes and news. No code, commits or code reviews were used.

## Sources

[CHR-F]: https://www.google.com/chrome/browser-features/ "Google Chrome: browser features"
[CHR-H]: https://support.google.com/chrome/?hl=en "Google Chrome Help (index, 161 article titles)"
[CHR-P]: https://support.google.com/chrome/answer/12929150?hl=en "Personalize Chrome performance"
[FX-F]: https://www.firefox.com/en-US/features/all/ "Firefox: all features"
[FX-AI]: https://www.bleepingcomputer.com/news/software/mozilla-will-let-you-turn-off-all-firefox-ai-features "BleepingComputer: Mozilla will let you turn off all Firefox AI features (secondary); also Linuxiac, System-Wide Controls to Block AI Features Come to Firefox 148"
[SAF-G]: https://support.apple.com/guide/safari/welcome/mac "Safari User Guide for Mac (macOS 27)"
[SAF-26]: https://developer.apple.com/documentation/safari-release-notes/safari-26-release-notes "Safari 26.0 Release Notes (snippet only); iGeeksBlog overview (secondary)"
[SAF-27]: https://www.macrumors.com/guide/ios-27-safari/ "MacRumors: iOS 27, all the new Safari features (secondary)"
[SAF-27-MCP]: https://iphonesoft.fr/2026/09/18/safari-27-navigateur-apple-se-ia-serveur-mcp "iPhoneSoft: Safari 27 se met à l'IA avec un serveur MCP (secondary)"
[EDGE-F]: https://explore.microsoft.com/en-us/edge/features "Microsoft Edge features"
[EDGE-CP]: https://winaero.com/copilot-actions-and-copilot-journeys-are-now-available-for-microsoft-edge "Winaero: Copilot Actions and Journeys in Edge (secondary)"
[VIV-F]: https://vivaldi.com/features/ "Vivaldi features"
[BRAVE-F]: https://brave.com/features/ "Brave features"
[OPERA-F]: https://www.opera.com/features "Opera features"
[ZEN]: https://zen-browser.app/ "Zen Browser"
[ORION]: https://orionbrowser.com/ "Orion Browser by Kagi"
[DIA]: https://www.diabrowser.com/ "Dia browser"
[ARC]: https://arc.net/ "Arc (discontinuation notice and Dia migration)"

- Chrome browser features: https://www.google.com/chrome/browser-features/
- Google Chrome Help index: https://support.google.com/chrome/?hl=en
- Personalize Chrome performance: https://support.google.com/chrome/answer/12929150?hl=en
- Firefox all features: https://www.firefox.com/en-US/features/all/
- Firefox AI Controls (news): https://www.bleepingcomputer.com/news/software/mozilla-will-let-you-turn-off-all-firefox-ai-features and https://linuxiac.com/system-wide-controls-to-block-ai-features-come-to-firefox-148/
- Safari User Guide for Mac: https://support.apple.com/guide/safari/welcome/mac
- Safari 26 release notes: https://developer.apple.com/documentation/safari-release-notes/safari-26-release-notes
- Safari 27 (news): https://www.macrumors.com/guide/ios-27-safari/ ; https://iphonesoft.fr/2026/09/18/safari-27-navigateur-apple-se-ia-serveur-mcp
- Microsoft Edge features: https://explore.microsoft.com/en-us/edge/features
- Edge Copilot Actions and Journeys (news): https://winaero.com/copilot-actions-and-copilot-journeys-are-now-available-for-microsoft-edge
- Vivaldi features: https://vivaldi.com/features/
- Brave features: https://brave.com/features/
- Opera features: https://www.opera.com/features
- Zen Browser: https://zen-browser.app/
- Orion Browser: https://orionbrowser.com/
- Dia browser: https://www.diabrowser.com/
- Arc: https://arc.net/
