# Izcal feature inventory audit: mapping of browser product features

Compiled 24 October 2026 against the reports on main at 217e02d.

**Input.** `browser_product_features.md` (270 rows, BF-1.1 to BF-21.6). Every row was checked with `grep -n -i` across `reports/*.md`, using several synonyms per feature, and the matching section was then read. The three review documents (`Browser user pain points 2026.md`, `Composable platform essay review.md`, `Feature design essays review.md`) are reviews, not specs, so they are never cited as an owner.

**Status rules used.**

- **Covered** means a spec states the behaviour, not only the name. A Covered row can still have a small sub-feature gap, which the evidence cell marks "gap:". Those gaps are also listed in Summary §5.
- **Partial** means the feature is named, or part of it is designed, but a substantive part has no design. The evidence cell says what is missing.
- **Excluded** means a recorded decision, non-goal or out-of-scope list refuses the feature. **Deferred (Stage N)** means a spec names the feature for a later stage but gives no design. A feature that is designed but scheduled for Stage 2 is **Covered**, with Stage 2 in the "Lands in" column.
- **Missing** means no spec mentions the feature. Every Missing row lists at least three search terms in the form `"term1" / "term2" / "term3"`, so the search can be repeated.

**Abbreviations.** Shell = `Izcal shell.md`. S1 = `HGBA Stage 1 architecture design.md`. UI = `HGBA shell UI design.md`. HGBA-2 = `HGBA 2 browser architecture spec.md`. HGBA-3 = `Future software browser engine.md`. The component specs use their ID prefixes: AX (accessibility), AH (app host), AN (animation), CB (content blocking), DT (DevTools), DV (devices), ED (editing), EX (extensions), FP (resource footprint), MD (images and media), ML (ML service), NW (networking), PD (PDF), PH (phishing protection), SEC (security scope), SG (storage). Shell milestones follow shell 9.2: SH-M1 = M2, SH-M2 = M3, SH-M3 = M4 (interface) and M7 (real service), SH-M4 = M5, SH-M5 = M6, SH-M6 = M7, SH-M7 = M8.

| # | Feature | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| BF-1.1 | Multiple windows | Covered | Shell 6.3 (`shell.window`, `shell.tabs`), 6.8, 6.17 | Stage 1, SH-M1 | "List windows, create and close a window, get and set bounds"; tabs "create, close, activate, move". Gap: no spec covers dragging a tab between windows or merging windows. |
| BF-1.2 | Tab strip basics | Covered | Shell 6.8 "The tab strip", 6.3 `shell.tabs` | Stage 1, SH-M1 | Tablist with "a context menu reachable by the Menu key"; "A tab move is a broker operation". Gap: duplicate tab, "close other tabs", "close tabs to the right" and "move to new window" are not specified. |
| BF-1.3 | Pinned tabs | Covered | Shell 6.8, SH-D27 | Stage 1, SH-M1 | "Pinned tabs are Stage 1"; the session store keeps "pinned state". |
| BF-1.4 | Tab groups | Excluded | Shell 9.1, SH-D27; S1 §6 "Tab-strip conveniences"; Q-SH-9 | Deferred (Stage 2) | "vertical tabs, split view and tab groups are Stage 2". The group model is open: Q-SH-9 asks "What is the tab-group model for Stage 2?" |
| BF-1.5 | Saved and synced tab groups | Excluded | Shell 9.1 (groups Stage 2; "Sync and multi-device" Stage 3 or never), Q-SH-5 | Deferred (Stage 2 / Stage 3) | Recently closed "restores ... for a group, its structure" (shell 6.8). Saving a group is not designed, and syncing one is excluded with sync. |
| BF-1.6 | Two-level tab stacks | Excluded | UI §8 Non-goals, UP5; S1 §6 out-of-scope list | — | "The shell does not compete on tab-strip layout (UP5)". |
| BF-1.7 | Vertical tabs | Excluded | SH-D27; S1 §6 "Tab-strip conveniences"; UI R18 | Deferred (Stage 2) | "one-click and persistent, never forced". No design: the Appearance group's "tab strip position (Stage 2)" is a placeholder only. |
| BF-1.8 | Split view | Excluded | SH-D27; S1 §6; UI R18 | Deferred (Stage 2) | Named as Stage 2 "table stakes". There is no design. |
| BF-1.9 | Workspaces / Spaces | Excluded | S1 §6 "Out of scope for Stages 1 and 2: UI chrome as differentiation (vertical tabs, spaces …)"; Shell §3, §5 (Arc 5.52%) | — | The refusal is implicit: it targets "chrome as differentiation", not Spaces by name. Containers (SH-D26) carry context separation instead. |
| BF-1.10 | Tab search | Covered | Shell 6.8 "Recently closed and tab search", 6.3 | Stage 1, SH-M3 (M4) | "searches open tabs by title, URL and container ... never sends a query anywhere". |
| BF-1.11 | Tab hover preview | Missing | — | — | Searched "thumbnail" / "tooltip" / "hover card" / "tab preview". Hits are PDF thumbnails and SVG title tooltips only. |
| BF-1.12 | Mute tab or site | Covered | Shell 6.3 (`shell.tabs` "pin, mute"), 6.8 | Stage 1, SH-M1 | "Each tab exposes its title, container name and colour, mute state". Gap: no audible indicator is specified, and muting a whole site is not covered (the only site-level mute is S1 §6's Stage 2 per-site mute for notifications). |
| BF-1.13 | Reopen closed tab or window | Covered | Shell 6.8 | Stage 1, SH-M3 | "Every tab close, window close and group close is recorded ... searchable and restores the whole set ... survives a crash". |
| BF-1.14 | Session restore at startup | Covered | Shell 6.8 "Session store and restore", SH-21; 6.9 Updates and recovery | Stage 1, SH-M3 (M7) | "written by the broker, atomically, every 5 s†"; "A failed restore is never silent". |
| BF-1.15 | Named saved sessions | Missing | — | — | Searched "named session" / "save session" / "session manager" / "saved tabs" / "tab sets". |
| BF-1.16 | Crash recovery | Covered | Shell 6.8 (tab crash), 6.8 restore, 6.14 recovery window, SH-21, SH-30 | Stage 1, SH-M3, SH-M4 | "A tab that crashes shows a neutral 'Page stopped' state and a Reload action"; restore "≥ 99%† after a crash". |
| BF-1.17 | Link peek / glance | Missing | — | — | Searched "link preview" / "peek" / "glance" (only "at a glance" headings) / "preview card". |
| BF-1.18 | Auto-archive of idle tabs | Missing | — | — | Searched "idle tab" / "auto-archive" / "auto-close" / "unused tabs". The nearest design is shell 6.8 cold-tab discard, which keeps the tab, and SH-D27, which rules out forced grouping. |
| BF-1.19 | Custom tab names / emoji | Missing | — | — | Searched "rename" (finds container renames only) / "emoji" / "tab title edit". |
| BF-1.20 | Journeys / topic history | Missing | — | — | Searched "journey" / "topic" / "resume browsing" / "interrupted". |
| BF-1.21 | Full screen | Covered | Shell 6.8 "Fullscreen and zoom" | Stage 1, SH-M1 | "keeps an exit control reachable by keyboard and by pointer at the top edge"; Esc and F11 always exit. Gap: no option to keep the toolbar visible in full screen. |
| BF-1.22 | UI auto-hide / compact mode | Partial | Shell 6.2, 6.9 Appearance | Stage 1 (density) | Compact is specified: "two densities, normal and compact". Auto-hiding the tab strip or toolbar is not (searched "auto-hide" / "hide the toolbar"). |
| BF-1.23 | Tabs from other devices | Excluded | Shell 9.1 "Sync and multi-device" (Stage 3 or never), SH-D20, Q-SH-5 | Stage 3 or never | "There is no Izcal account and no sync in this version." |
| BF-1.24 | Open-tab list in OS app switcher (mobile) | Missing | — | — | Searched "app switcher" / "recent apps" / "Android" (the hits are engine and accessibility only). S1 §9 schedules the Android engine for Stage 2, but no mobile shell spec exists. |
| BF-2.1 | Unified address and search bar | Covered | Shell 6.8 "The address bar" | Stage 1, SH-M1 | "Typing a search sends it to the person's chosen engine only after Enter". |
| BF-2.2 | Suggestions | Covered | Shell 6.8; Q-UI-1 | Stage 1, SH-M1 (study at SH-M4) | "Autocomplete candidates come from open tabs, history, bookmarks, typed URLs and the chosen search engine, in that order, with a provenance label". |
| BF-2.3 | Default search engine choice | Partial | Shell 6.7, 6.8 ("chosen search engine"); EX 6.9 ("Izcal's search settings are the user's") | — | The engine choice is assumed but never designed. No settings group covers search (shell 6.9), and adding a custom engine and the default engine list are unspecified. Only three hits for "search engine". |
| BF-2.4 | Search engine choice screen (EU DMA) | Missing | — | — | Searched "choice screen" / "DMA" / "Digital Markets" / "search engine". The one DMA hit is the DRM regulatory track (S1 13.1a). |
| BF-2.5 | Site search keywords / @ scopes | Missing | — | — | Searched "keyword" / "@bookmarks" / "@tabs" / "site search". |
| BF-2.6 | Calculator and unit conversion | Missing | — | — | Searched "calculat" / "unit conver" / "equation". |
| BF-2.7 | Quick actions / command palette | Excluded | SH-D27; shell 6.20; Q-SH-8 | Deferred (Stage 2 trial) | "a palette is a Stage 2 trial"; removed if used by "Under 5%† of weekly users after three months". |
| BF-2.8 | Voice search | Missing | — | — | Searched "voice" (hits are AT products only) / "speech recognition" / "dictation" / "voice search". ML-ASR produces captions, not queries. |
| BF-2.9 | Visual search (Lens) | Missing | — | — | Searched "visual search" / "image search" / "lens" / "reverse image". |
| BF-2.10 | Security indicator and URL display | Covered | Shell 6.8, 6.6, SH-D14; UI 4.1–4.2 | Stage 1, SH-M1/SH-M2 | "Plain HTTP shows a neutral 'Not secure' chip ... There is no EV treatment, no padlock"; site-controls button opens the panel. |
| BF-2.11 | Address bar position (mobile) | Missing | — | — | Searched "address bar position" / "bottom of the screen" / "bottom toolbar". No mobile shell spec exists. |
| BF-2.12 | AI search mode in address bar | Excluded | Shell 6.8; SH-D16; UI R4 | — | "sponsored or AI-generated suggestions do not exist". Nothing is designed for the AI-on case either. |
| BF-2.13 | Sponsored / contextual suggestions | Excluded | Shell 6.8; 6.7 (new-tab page) | — | "sponsored or AI-generated suggestions do not exist"; the new-tab page has "no sponsored tiles". |
| BF-2.14 | Home button / homepage | Partial | EX 6.9, EX-D15 (homepage override prompt, Stage 2) | — | Only the extension override prompt is named. The shell specifies no homepage setting, home button or startup-page choice; the startup behaviour it does name is "session restore" in Updates and recovery. |
| BF-2.15 | Mouse gestures | Missing | — | — | Searched "mouse gesture" / "rocker" / "gesture" (hits are user-activation gestures only). |
| BF-2.16 | Bundled own search engine | Missing | — | — | Searched "search engine" / "search provider" / "own search". D-S8 lists "search-engine revenue share" only as a funding option. |
| BF-2.17 | Desktop search bar outside the browser | Missing | — | — | Searched "desktop search" / "taskbar search" / "search box on the". |
| BF-3.1 | Bookmarks and bookmark bar | Covered | Shell 6.7 "Surfaces", SH-D8, SH-D9 | Stage 1, SH-M3 (schema), SH-M5 (bar) | Star control; "The bookmarks bar is hidden by default ... folders are menus". |
| BF-3.2 | Bookmark manager with tags | Partial | Shell 6.7, 6.12 | Stage 1, SH-M5 | The manager is specified: "search first, folders second ... provenance, health". Import and export are specified. **Tags are not**: the record has folder and "saved reason" only (searched "tag" near "bookmark"). |
| BF-3.3 | History view and search | Covered | Shell 6.7, SH-19, SH-20 | Stage 1, SH-M3/SH-M5 | "defaults to search and recent, grouped by day, with filters for site, container and date"; delete a row, range or site. |
| BF-3.4 | History statistics | Missing | — | — | Searched "timeline" / "statistic" / "browsing activity". The shell's `aggregate` record (6.7) would hold the data, but no surface is specified. |
| BF-3.5 | Reading list / read later | Partial | S1 §6 "Save for offline" (Stage 2); UI R18 | Stage 2 (offline save only) | Saving offline is designed as a "frozen DOM-and-resources snapshot". No reading-list queue, surface or read state exists (searched "reading list" / "read later" / "Pocket"). |
| BF-3.6 | Collections / pinboards | Missing | — | — | Searched "collection" / "pinboard" / "clipping" / "notes". |
| BF-3.7 | New tab page | Covered | Shell 6.7 "Surfaces" | Stage 1, SH-M5 | "local: no sponsored tiles, no remote favicons, no network request on open; it shows a search field ... top sites ... recently closed tabs". Gap: no customisation (background, layout, shortcuts). |
| BF-3.8 | Speed dial tiles | Partial | Shell 6.7 | Stage 1 | "the person's top sites from history (opt-out)". Pinned or user-arranged tiles and tile groups are not specified. |
| BF-3.9 | News feed on new tab page | Excluded | Shell 6.7 (new-tab page "no network request on open"), SH-D29, SH-28 | — | A feed would need the network request the new-tab page forbids, and "The shell sends no network request of its own". |
| BF-3.10 | Dashboard widgets on start page | Excluded | Shell 6.7, SH-D29 | — | The same no-network rule applies to the new-tab page. Local-only widgets are not considered anywhere (searched "widget" / "dashboard"). |
| BF-3.11 | Start page shows library content | Partial | Shell 6.7 | Stage 1 | Top sites from history and recently closed tabs are specified. Bookmarks and a reading list on the start page are not. |
| BF-3.12 | Page change notifications | Partial | Shell 6.7 "Content fingerprint and health", SH-D9 | Stage 1 | Detection on revisit is specified: "changed since you last saw it". Watching a page and alerting is not: "there is no background crawl by default". |
| BF-3.13 | Wayback Machine fallback on 404 | Missing | — | — | Searched "wayback" / "archive.org" / "404" (hits are citations only). |
| BF-3.14 | Recent tabs / continue-where-you-left-off cards | Partial | Shell 6.7 (new-tab page recently closed), 6.8 | Stage 1 | Recently closed tabs appear on the new-tab page. Cards that resume tasks or searches are not specified. |
| BF-4.1 | Multiple profiles | Partial | Shell 4.1 ("profiles later"), 6.2 ("One shell renderer per profile"), SH-D26; S1 13.2 "Profiles" (format) | — | A profile format with migrations exists, and containers are the isolation model; SH-D26 rejects "profiles as the only isolation". No spec covers creating, switching or picking profiles (searched "profile picker" / "multiple profiles" / "profile switch"). |
| BF-4.2 | Guest mode | Missing | — | — | Searched "guest" (hits are VM and app-host guests) / "guest mode" / "shared device". The closest design is NW 6.1: "Private windows are ephemeral containers". |
| BF-4.3 | Browser sign-in / account | Excluded | SH-D20; shell 6.12, 6.14; Q-SH-5 | Stage 3 or never | "There is no account, no sign-in, no sync". |
| BF-4.4 | Sync | Excluded | Shell 9.1 (Stage 3 or never), SH-D20, Q-SH-5; S1 §9 Stage 3 "Sync (if wanted)"; EX-D6 | Stage 3 or never | "Sync is not silently absent; it is named as out of scope in the product". |
| BF-4.5 | End-to-end encrypted sync | Excluded | As BF-4.4 | Stage 3 or never | Excluded together with sync. No encryption model is recorded for a future sync. |
| BF-4.6 | Send tab to device | Excluded | Shell 9.1 "Sync and multi-device" | Stage 3 or never | Falls under the "multi-device" exclusion (searched "send tab" / "handoff" / "continuity"; no other hit). |
| BF-4.7 | Send files and notes between devices | Excluded | Shell 9.1 "Sync and multi-device" | Stage 3 or never | Covered by the same exclusion. Nothing else mentions it. |
| BF-4.8 | Work/personal auto profile switching, work SSO | Partial | SH-D26 (named containers personal/work/banking/shopping); NW-D20 (ambient authentication allowlist) | Stage 1 (containers, read-only) | A work container exists, and Negotiate/NTLM run under an allowlist. Automatic assignment of a site to a container and a managed work profile are not specified (searched "SSO" / "work profile" / "always open in"). |
| BF-4.9 | Sync troubleshooting and data reset | Excluded | As BF-4.4 | Stage 3 or never | Follows from the sync exclusion. |
| BF-4.10 | Third-party sign-in prompts (FedCM UI) | Missing | — | — | Searched "FedCM" (DT 6.1 names it only as a redaction class) / "federated" / "Sign in with". CB-D19's "Allow sign-in with a provider, for this tab" is a blocking exception, not a FedCM dialog. |
| BF-4.11 | Export my data | Covered | Shell 6.12 "Export", SH-D20 | Stage 1, SH-M3 (M7) | "an open, documented, versioned JSON Lines schema (`izcal-export` v1)" covering pages, visits, bookmarks, settings, containers, ledger, and credentials as CSV. |
| BF-5.1 | Downloads UI | Covered | Shell 6.10, 6.3 `shell.downloads`, SH-D18, SH-23 | Stage 1, SH-M4 (M5) | "toolbar-anchored bubble"; actions "pause, resume, cancel, clear; open, show in folder, copy path". |
| BF-5.2 | Download location and ask-where-to-save | Covered | Shell 6.9 Downloads group; NW 6.14 | Stage 1, SH-M4 | "Download directory, ask where to save, auto-open (off by default)". |
| BF-5.3 | Dangerous download warnings | Covered | Shell 6.10, SH-D18, SH-R7; PH-D7 | Stage 1 | "two-level warning prompt"; "Suspicious" and "Dangerous" carry "the reason ... and the confidence". |
| BF-5.4 | Save page | Covered | S1 §6 "Save for offline"; PH 6.4.10 (saved pages) | Stage 2 | "Default format is a frozen DOM-and-resources snapshot ... the B1 replay archive is kept as an option". Gap: "HTML only" and a standard web-archive format are not offered. |
| BF-5.5 | Print and print preview | Excluded | S1 §6 "Printing", 13.2; shell 6.19 ("print UI (Stage 2)"); GR 6.27 | Deferred (Stage 2) | Paged layout and preview raster are Stage 1 ("print raster and print preview raster are Stage 1 (GR-M3)"). The print UI is Stage 2 and has no design. |
| BF-5.6 | Save as PDF | Covered | PD 6.19 writer, PD-D19–PD-D21, PD-M4; S1 13.2 | Stage 2 (writer core Stage 1) | "print-to-PDF stays Stage 2"; the writer is designed (PD-M4). |
| BF-5.7 | Built-in PDF viewer | Covered | PD-D2, PD 6.15 viewer; S1 §6 "PDF viewer" | Stage 1 (M7) | "continuous and paged scrolling, spreads, zoom ..., rotation, thumbnails, the outline, page labels, find ..., print, save". |
| BF-5.8 | PDF form fill, annotate and sign | Covered | PD-D12, PD-D17, PD 9.5 | Stage 1 (forms, annotation at PD-M3); Stage 2 (signing, redaction) | "AcroForm editing in the PDF process, incremental save ... authoring at PD-M3". **Inconsistency:** S1 §6 still says "Stage 2 adds local annotate, sign and redact", but PD 9.5 moves annotation authoring to Stage 1. |
| BF-5.9 | PDF image alt text (AI) | Missing | — | — | Searched "alt text" / "image description" / "PDF" + "alt". PD-D11 uses OCR through the ML service, and ML-IMG describes images in pages, but no spec authors alt text for images added to a PDF. |
| BF-5.10 | PDF summarise and ask | Excluded | ML-D2 (summarise is a Stage 2 task under `ai.enabled`); HGBA-3 B10 | Deferred (Stage 2) | "translate, summarise, prompt and writing assistance are Stage 2 tasks". No PDF surface is designed. |
| BF-5.11 | Easy file picker | Missing | — | — | Searched "file picker" / "upload dialog" / "recent downloads". |
| BF-6.1 | Password manager | Covered | Shell 6.12 "The password manager, bounded", 6.3 `shell.password`, 6.9; S1 §6 "Passkeys and credentials" (A-S1-51) | Stage 1 | "a basic manager as a UI and broker service over the OS credential store"; "revealing a password requires a fresh OS authentication". |
| BF-6.2 | Password generator | Missing | — | — | Searched "password generator" / "generate password" / "strong password". The one hit is AH §1, which uses a password generator as an example of a local app. |
| BF-6.3 | Breached and weak password alerts | Excluded | Shell 6.12 | Excluded in Stage 1 (no later owner) | "There is no sync, no cloud and no breach-check service in Stage 1." Weak and reused password checks, which need no service, are not mentioned (searched "breach" / "reused" / "weak password"). |
| BF-6.4 | Passkey manager | Covered | S1 §6 "Passkeys and credentials"; shell 6.12, 6.4 | Stage 1 | "WebAuthn through platform authenticators; passwords and passkeys live in the OS keychain". Gap: the OS manages passkeys; the browser has no passkey list of its own. |
| BF-6.5 | Password and passkey import/export | Covered | Shell 6.12, SH-D19, SH-R2 | Stage 1, SH-M3 (M7) | "passwords only from a CSV the person exports"; "export is a CSV behind re-authentication". Gap: passkey export is not addressed. |
| BF-6.6 | Password manager PIN / device unlock | Covered | Shell 6.12; 6.4 Credential class | Stage 1 | "revealing a password requires a fresh OS authentication". Gap: filling a password is not gated behind device unlock. |
| BF-6.7 | Third-party password manager integration | Missing | — | — | Searched "third-party password" / "1Password" / "Bitwarden" / "credential provider". Extension managers are Stage 2, but no hand-off from the built-in manager is specified. |
| BF-6.8 | Address autofill | Excluded | S1 13.2 "Everyday features" ("autofill beyond passwords" Stage 2), 13.2 batch-4 note; ED 6.6 and 9.5 (autofill preview), UI 10 (A-UI-8) | Deferred (Stage 2) | Only the autofill preview window and chooser are described (UI 10, A-UI-8). The address store and its fill rules are not. |
| BF-6.9 | Payment card autofill | Excluded | Shell 6.9 ("Payment autofill is Stage 2 and has no row until it ships"), D-S3/13.2 | Deferred (Stage 2) | A "payment autofill" credential prompt class exists (6.4). There is no card store design. |
| BF-6.10 | Virtual cards, BNPL, card benefits | Missing | — | — | Searched "virtual card" / "buy now" / "BNPL" / "card benefit". |
| BF-6.11 | Wallet payment sheet | Missing | — | — | Searched "Apple Pay" / "Google Pay" / "wallet" / "Payment Request". The only Payment Request hit is DV 6.14: "Payments stay in Payment Request". |
| BF-6.12 | Shopping: price tracking, coupons | Missing | — | — | Searched "price track" / "coupon" / "cashback" / "shopping" (hits are the shopping container only). |
| BF-6.13 | Store reviews | Missing | — | — | Searched "review summary" / "store review" (hits are extension and app store review). |
| BF-6.14 | Crypto wallet | Missing | — | — | Searched "crypto wallet" / "cryptocurrency" / "web3" / "ethereum". |
| BF-6.15 | Creator rewards | Missing | — | — | Searched "reward" / "BAT" / "token to" / "private ads". |
| BF-7.1 | Private browsing window | Covered | NW 6.1 ("Private windows are ephemeral containers with their own services and no disk state"); SG 6.9; shell 6.7, 6.10; CB 6.12 | Stage 1 | "Private windows write nothing"; storage "memory-only". Gap: no spec covers opening a private window or its visual indicator. |
| BF-7.2 | Lock private tabs with biometrics | Missing | — | — | Searched "lock private" / "biometric" / "Touch ID" / "Face ID". |
| BF-7.3 | Private window over Tor | Covered | NW 6.13 "Routes", NW-D6; UI R8; S1 §6 "Containers" | Stage 2 (container routes) | "Tor routes are SOCKS to a Tor client the user runs ... Izcal does not embed a Tor client"; label "hides your IP address, not your browser". |
| BF-7.4 | Cross-site tracker blocking | Covered | CB (whole spec), CB-D19; S1 §6 "Built-in content blocking" | Stage 1 | Six layers and default lists. Levels are refused: "there is no Strict level in Stage 1" (CB 6.12). |
| BF-7.5 | Cookie partitioning | Covered | NW 6.9, NW-D7, NW-D16; S1 §6 "Third-party cookies" | Stage 1 | "Partitioned by (T, C, X) by default ... matches Firefox's Total Cookie Protection". |
| BF-7.6 | Third-party cookie blocking setting | Partial | NW 6.9 | Stage 1 | Partitioning plus the Storage Access API "with a user prompt" are specified. A global "block third-party cookies" setting with per-site exceptions is not. |
| BF-7.7 | Fingerprinting protection | Covered | D-S1, OD-6; S1 §6 "Anti-fingerprinting"; shell 6.6 Fingerprinting row | Stage 1 baseline, Stage 2 full | "Noise is an output transform ... Discrete surfaces are normalised by construction". |
| BF-7.8 | Tracking-parameter removal | Covered | CB 6.5, CB-D8 | Stage 1 | "Link-decoration stripping. Izcal's parameter list names query parameters that carry cross-site identifiers". |
| BF-7.9 | Privacy report / dashboard | Partial | CB 6.12–6.13, CB-D20; shell 6.6 Blocking row; NW 6.18 "who received what" | Stage 1 | Per-site counts ("Blocked here: N parties from M organisations") and a DevTools Protections view are specified. There is no browser-wide report or dashboard (searched "privacy report" / "protections dashboard" / "blocked trackers"). |
| BF-7.10 | Per-site shields panel | Covered | CB 6.12, CB-D19; shell 6.6 Blocking row | Stage 1, SH-M2 | Graduated controls: "Allow one party here ... Hiding off here ... Protection off here". |
| BF-7.11 | Built-in ad blocking | Covered | CB (Ads list); S1 §6 "Built-in content blocking" (CB-D1, CB-D13) | Stage 1 | "the Ads list admits no entry on resource cost alone and ships with the first lists". |
| BF-7.12 | Cookie banner handling | Covered | CB 6.15, CB-D22; S1 §6 "Cookie-banner handling" | Stage 2 | "Rule-based auto-reject in Izcal's own rule format ... hide-first, never auto-accept". |
| BF-7.13 | Clear browsing data | Partial | Shell 6.4 (ledger retention), 6.7 (history clearing), 6.9 ("clear-on-close choices") | Stage 1 | History clearing by range, site and container is specified, and a "Clear browsing data" flow is referenced. No spec designs the dialog itself: which categories (cookies, cache, passwords), which time ranges, and clear-on-exit behaviour. |
| BF-7.14 | Site data manager | Covered | Shell 6.6 Storage row, SG-D5/SG-D6; 6.9 Permissions and site settings | Stage 1, SH-M2 | "Exact usage, quota, persist state ... 'Clear data for this site' ... clearing states what remains". |
| BF-7.15 | GPC / Do Not Track | Covered | S1 §6 "Global Privacy Control"; shell 6.9 Privacy row | Stage 1 | "`Sec-GPC: 1` sent by default"; DNT is "explained as a request, not a promise". |
| BF-7.16 | Telemetry controls | Covered | D-S4, SH-D29; shell 6.3 `shell.telemetry`, 6.14 | Stage 1 | "Telemetry is opt-in, aggregated, identifier-free, schema-published"; crash reports are "opt-out per crash". |
| BF-7.17 | IP masking in private browsing | Excluded | S1 §6 out-of-scope list ("a browser-operated VPN service"); routes are "user-supplied" | — | IP hiding is possible only through routes the user supplies (Stage 2). |
| BF-7.18 | Distraction control / element hiding | Partial | CB 6.10, CB-D14 ("write their own rules"); CB 6.7 | Stage 1 | Users can write cosmetic rules. A point-and-hide element picker is not specified (searched "element picker" / "zapper" / "hide element" / "distraction"). |
| BF-8.1 | Safe browsing interstitials | Covered | PH spec, PH-D6; shell 6.4 Security interstitial class; UI 4.5 | Stage 1 | "the `security-interstitial` class drawn by the prompt renderer"; "proceed is per navigation, never persisted". |
| BF-8.2 | Enhanced protection modes | Partial | HGBA-2 S-4a "Hardened profile"; JS-D13 (Jitless for the banking container and per site); SEC-D5 | Stage 1 (engine) | A jitless tier per container and per site exists. No user-facing "enhanced protection" mode or setting is designed, and the phishing check makes "no per-user query" (a prefix-lookup mode "is not Stage 1", NW 6.13, PH-D1, PH-D12). |
| BF-8.3 | Scareware / tech-support-scam blocker | Partial | CB 6.8, CB-D23 Security list ("scam redirectors") | Stage 1 | The security list blocks endpoints that redirect to scams. Detecting a full-screen scam page and leaving it is not specified (searched "scareware" / "tech support" / "scam"). |
| BF-8.4 | HTTPS-only / HTTPS-First | Covered | NW 6.7 "HTTPS-First", NW 6.5.3; shell 6.9 | Stage 1 | "Public sites load over HTTPS first, with an interstitial before plain HTTP". |
| BF-8.5 | Secure DNS settings | Covered | NW 6.6, NW-D8; NW 6.14 (policy) | Stage 1 | "same-provider upgrade by default ... a user-chosen DoH resolver ... as an option". Gap: no row in shell 6.9 holds this setting. |
| BF-8.6 | Certificate viewer | Partial | Shell 6.6 Identity row ("certificate details expander"); NW 6.18 | Stage 1 | Only the expander is named. Nothing says what it shows (chain, validity dates, fingerprints). |
| BF-8.7 | Certificate error interstitials | Covered | Shell 6.4 Security interstitial; NW 6.5.3, 6.18; UI 4.5, UI-9 | Stage 1 | "Back to safety; proceed behind Details"; for HSTS-preloaded domains "certificate errors cannot be clicked through". |
| BF-8.8 | Safety check | Missing | — | — | Searched "safety check" / "security checkup" / "check updates passwords". |
| BF-8.9 | Unwanted software cleanup | Missing | — | — | Searched "unwanted software" / "hijack" (hits are about extensions) / "cleanup tool". |
| BF-8.10 | "Managed by your organisation" indicator | Covered | Shell 6.9 ("Set by your organisation"), 6.14 `about:policies`; NW 6.14 `izcal://policy` | Stage 1, SH-M4 | "Policy-set rows are visible ... the policy's source and the administrator's contact are shown". Inconsistency: the shell names the page `about:policies` and NW names it `izcal://policy`. |
| BF-8.11 | Lookalike-domain warning | Partial | Shell 6.4 (interstitial class includes "lookalike domain"), SH-11; UI Q-UI-5 / Q-PH-16 (open) | Stage 1 (class) | "who owns the lookalike-domain warning class? ... left open". The detection algorithm is not specified. |
| BF-8.12 | Auto-update | Covered | Shell 6.11, SH-24; S1 13.2 "Updates" (TUF), 7.8 | Stage 1 | "a non-modal update row ... 'Restarting will restore your 14 tabs and 2 windows'". |
| BF-9.1 | Page info panel | Covered | Shell 6.6, SH-D14 | Stage 1, SH-M2 | "The site panel is the one place every per-site control lives". |
| BF-9.2 | Permission prompts | Covered | Shell 6.4, 6.5, SH-D2, SH-D12; S1 13.2 "Permissions" | Stage 1, SH-M2 | Classes, anatomy, timing, ledger; "'Allow this time', 'Allow on this site', 'Block'". |
| BF-9.3 | Quieter notification prompts | Covered | Shell 6.4 "Quiet UI and prompt volume", quiet notification class; SH-D17 | Stage 2 | "a site must have been interacted with before a notification prompt can appear; notifications use a quiet chip first". |
| BF-9.4 | Site settings manager | Covered | Shell 6.9 "Permissions and site settings", SH-D15 | Stage 1, SH-M4 | Each registry row has "its scopes (global, container, site)"; "Ask by default; one-time first". |
| BF-9.5 | Auto-revoke unused permissions | Covered | Shell 6.4; S1 13.2 "Permissions" | Stage 1 | "retires grants a site has not used for 30 days† (notifications) or 90 days† (other capabilities), with a visible 'expired' state". |
| BF-9.6 | Device access choosers | Covered | DV spec (DV-D1–DV-D15); shell 6.4 device classes, 6.6 Devices row | Stage 1 (behind the Experimental device APIs setting) | "Broker-owned chooser (listbox semantics) ... No default selection". |
| BF-9.7 | Popup blocker | Partial | DOM 6.5 (activation record gates popups); CB 6.2 (`$popup`) | Stage 1 (engine) | Popups are gated on user activation and blocked by list rules. The shell specifies no blocked-popup indicator and no per-site allow. |
| BF-9.8 | Autoplay controls | Covered | MD 6.12, MD-D13; S1 §6 "Autoplay"; UI 10 (A-UI-3) | Stage 1 | "audible playback only with transient user activation ... or when the user has set 'allow audio and video' for the site in the site panel". Gap: no "block all media" option. |
| BF-9.9 | In-use indicators | Partial | Shell 6.6 device indicator (SH-35); SEC 6.5 capture threat model (screen capture "shows an OS-level capture indicator") | Stage 1 (devices) | The device indicator is fully specified. Camera, microphone, location and tab-sharing indicators with a stop control are not (searched "in-use indicator" / "recording indicator" / "camera indicator"). |
| BF-9.10 | Screen-share picker | Partial | MD 6.10 Capture ("the OS pickers"); GR 6.27 (tab capture in the compositor) | Stage 1 | The OS picker is used and tab capture is composed by the compositor. A browser-side tab picker and a "sharing this tab" bar are not specified. |
| BF-9.11 | Per-site JavaScript, images, content toggles | Partial | S1 §6 "JavaScript modes" (Stage 2), UI R9; shell 6.6 JavaScript row | Stage 2 (JavaScript) | Three JavaScript modes are designed ("off; same-site only; full"). Per-site toggles for images and other content are missing (searched "block images" / "images off" / "load images"). |
| BF-9.12 | Protocol handler / external app prompts | Covered | Shell 6.4 External protocol class; SEC 6.14a; PH 6.4.6 | Stage 1 | "gesture-gated; the prompt names the scheme, the handler application and the requesting origin". |
| BF-10.1 | Find in page | Covered | Shell 6.8 "The find bar", ED-D17, SH-32; S1 §6 "Find in page" | Stage 1, SH-M5 | "case, diacritics, whole word and regular expression ... hidden content revealed, not skipped". |
| BF-10.2 | Page zoom | Covered | Shell 6.8 "Fullscreen and zoom"; AX-D19; GR 6.28 (pinch) | Stage 1, SH-M1 | "Page zoom is per site and global, 25–500%, with a reset"; pinch scale is handled by the visual viewport (AX-D19). |
| BF-10.3 | Text-only zoom and minimum font size | Partial | AX-D19 (text scale), ST-D17 (minimum font size); AX 6.11 | Stage 1 (mechanism) | Text scale is specified as an engine factor, taken by default from the OS. The shell has no control for text-only zoom or minimum font size; 6.9 lists only "page zoom default, UI text zoom". |
| BF-10.4 | Reader mode | Excluded | UI R18, UI §6 ("save for offline and reader view", Stage 2); S1 §6 ("Reader rendering on top"); PD (reflow view, Stage 2) | Deferred (Stage 2) | Named only. There is no extraction or view design. |
| BF-10.5 | Read aloud | Missing | — | — | Searched "read aloud" / "text-to-speech" / "TTS" / "speech synthesis". EX stages the extension `tts` API to Stage 3 ("OS voices"), but no browser read-aloud is specified. |
| BF-10.6 | Page translation | Excluded | ML-D2, ML-M5; HGBA-3 B10 | Deferred (Stage 2, behind `ai.enabled`) | "translate ... Stage 2 tasks under `ai.enabled`". Only the task API is named; there is no page-translation surface. Translation is placed behind the AI switch. |
| BF-10.7 | Spellcheck | Covered | ED-D7, ED-D8, ED-D22; S1 13.2 "Text and input"; shell 6.9 Language and writing | Stage 1 (interim OS tier, M6), Stage 2 (settings group) | "Izcal's own engine and lexicons, offline"; spelling menu with "Add to dictionary" (UI 10). |
| BF-10.8 | Cloud / enhanced spellcheck | Excluded | ED §4.4 ("Spellchecking is the OS's or a cloud's: Break"), ED-D7; ED-D9 | — | Offline only. A learned grammar model is "Stage 2 under the AI grant". |
| BF-10.9 | Share page | Missing | — | — | Searched "share sheet" / "Web Share" / "navigator.share" / "share page". |
| BF-10.10 | Link to highlighted text | Partial | ED 6.12 "Text directives" | Stage 1 (navigation) | Following `#:~:text=` is specified. Copying a link to selected text is not. |
| BF-10.11 | QR code for page | Missing | — | — | Searched "QR code" / "generate a QR" / "QR". PH 6.4.10 checks a "QR-decoded URL", but nothing generates or scans codes. |
| BF-10.12 | Screenshot tool | Missing | — | — | Searched "screenshot" / "screen capture tool" / "capture full page". |
| BF-10.13 | Eyedropper | Missing | — | — | Searched "eyedropper" / "EyeDropper" / "colour picker" (the hit is a form-control picker). |
| BF-10.14 | Notes in sidebar | Missing | — | — | Searched "notes" / "note-taking" / "sidebar". |
| BF-10.15 | Highlights / page key facts | Missing | — | — | Searched "key facts" / "highlights" / "summary in the address bar". |
| BF-10.16 | Request desktop site (mobile) | Missing | — | — | Searched "desktop site" / "desktop mode" / "user agent switch". The UA is fixed per S1 13.2 and NW-D12. |
| BF-10.17 | Dark mode for web content | Missing | — | — | Searched "force dark" / "dark mode" / "auto dark". PD has dark mode for PDFs only ("role-based"). |
| BF-10.18 | View page source | Partial | PH 6.2 and HGBA-3 B9 (AGT-R3) (refer to `view-source:` URLs) | — | Two specs assume `view-source:` exists, but no spec owns the viewer. |
| BF-11.1 | Picture-in-picture | Covered | MD 6.12; UI 10 (A-UI-3); S1 13.2 | Stage 2 | "Video PiP is a shell-owned OS window"; Document PiP has its own restrictions. |
| BF-11.2 | Automatic PiP on tab switch | Missing | — | — | Searched "picture-in-picture" / "PiP" / "auto PiP" / "tab switch". The PiP design does not cover automatic entry. |
| BF-11.3 | Global media controls | Partial | MD 6.12 Media Session | Stage 1 | "Media Session metadata and actions reach the OS media controls through the broker". There is no browser hub that controls media across tabs. |
| BF-11.4 | Casting | Missing | — | — | Searched "cast" / "AirPlay" / "Chromecast" / "Presentation API" / "Remote Playback". |
| BF-11.5 | Live captions | Covered | ML-D2 (ML-ASR), MD-D25; HGBA-3 B10; Q-ML-1 | Stage 1 (ML-M3) or Stage 2 | "Generated captions for media"; behind the `accessibility.model` grant. Gap: translated captions are not covered. |
| BF-11.6 | Media playlist / offline play | Missing | — | — | Searched "playlist" (hits are HLS) / "queue videos" / "offline play". |
| BF-11.7 | Built-in music player | Missing | — | — | Searched "Spotify" / "music player" / "Deezer". |
| BF-11.8 | Volume booster / video enhancement | Missing | — | — | Searched "volume boost" / "amplify" / "Lucid". |
| BF-11.9 | DRM playback | Covered | D-S7, S1 13.1a; MD-D32; shell 6.4 Protected-media switch | Stage 1 (M7) | Protected media mode through WebView2 or WKWebView, plus native PlayReady. Linux ships Clear Key only. |
| BF-11.10 | Microphone volume adjustment | Missing | — | — | Searched "microphone volume" / "input level" / "gain". |
| BF-12.1 | Tab sleeping / memory saver | Covered | Shell 6.8 "Cold-tab discard"; FP-D15; S1 §6 "Tab memory and bfcache" | Stage 1 | "A sleeping tab shows a moon glyph with the text 'Sleeping'". Gap: no user-selectable levels. |
| BF-12.2 | Always-keep-active site list | Covered | FP-D15; UI 10 (A-UI-1); S1 §6 "Per-site resource meter" | Stage 2 | "keep this site running in the background" (a declared reason). |
| BF-12.3 | Energy / battery saver | Partial | AN 6.11, AN-D11 (30 Hz under the OS low-power signal); FP-D15 "limit this site on battery" (Stage 2, gated by FP-30) | Stage 1 (follows the OS), Stage 2 (per site) | There is no browser-wide energy-saver mode or setting. |
| BF-12.4 | Performance issue alerts | Excluded | Shell 6.8 "Cold-tab discard"; UP2 | — | "The person is never asked to close tabs to save memory; the browser does the work". |
| BF-12.5 | Browser task manager | Partial | Shell 6.14 `about:diagnostics` ("process list with PIDs and budgets"); shell 6.6 Resources row (Stage 2); FP 6.14 | Stage 1 (diagnostics), Stage 2 (meter) | A per-tab and per-extension view with "end process" is not specified. The site panel says "no kill button without an explanation". |
| BF-12.6 | Preloading settings | Partial | S1 §9 ("Speculation Rules and prerender", Stage 2) | Stage 2 (engine) | Preloading is scheduled, but no user setting is specified. |
| BF-12.7 | Startup boost / background running | Missing | — | — | Searched "startup boost" / "background running" / "run in the background" / "keep running after". |
| BF-12.8 | Hardware acceleration toggle | Covered | Shell 6.14 recovery window, SH-D22 | Stage 1, SH-M4 | "Start without hardware acceleration ... the setting is not changed unless the person ticks 'remember'". Gap: no row for it in Settings. |
| BF-13.1 | Extension support | Covered | EX spec (EX-D1–EX-D16); D-S3; S1 §6 "Extensions" | Stage 2 | "Stage 1 ships no extensions". MV3 runs in a component host (EX-D3). |
| BF-13.2 | Extension store | Covered | EX-D10 | Stage 2 | "Izcal directory with published review decisions and a decision log; signing after review". Gap: ratings and reviews are not mentioned. |
| BF-13.3 | Extension management UI | Covered | Shell 6.15, SH-D28; EX-D14, EX-D15 | Stage 2 | "lists installed extensions, versions, enabled state, granted capabilities ... enable, disable and revoke". |
| BF-13.4 | Cross-ecosystem extension support | Covered | EX-D1 (three compatibility rings), EX-D4 | Stage 2 | WebExtensions MV3 namespaces are implemented in Ring A, each under a published conformance contract. Gap: packages must be "Izcal-signed", so a user cannot install directly from another vendor's store. |
| BF-13.5 | Legacy MV2 support | Excluded | EX-D2 | — | "MV3 only; MV2 refused with a published migration mapping". |
| BF-13.6 | Extensions on mobile | Missing | — | — | Searched "Android" in the extensions spec (one hit, not about mobile extensions) / "mobile extensions" / "iOS". |
| BF-13.7 | AI-generated extensions | Missing | — | — | Searched "generate an extension" / "AI-built" / "Boosts". |
| BF-13.8 | Themes | Covered | Shell 6.2 "Theming is CSS", SH-D6; EX-D15 | Stage 1 | "A user theme is a set of custom-property values". Gap: no theme gallery; extension themes are refused ("theme overrides refused"). |
| BF-13.9 | AI theme generator | Missing | — | — | Searched "theme" + "AI" / "generate a theme" / "theme generator". |
| BF-13.10 | Light/dark mode for the browser UI | Covered | Shell 6.2; UI 4.10 | Stage 1 | "Built-in light and dark themes follow the OS by default". |
| BF-13.11 | Toolbar customisation | Missing | — | — | Searched "customize toolbar" / "rearrange" / "toolbar button order". |
| BF-13.12 | Deep UI customisation / mods | Partial | Shell 6.2 (user CSS limited to custom properties and ordinary properties), D-S2/UI R2 (W-1 components), SH-D6 | Stage 1 (CSS) | User CSS is allowed but restricted. A behaviour or layout mod system is not specified, and W-1 is constrained to "project-authored components". |
| BF-13.13 | Keyboard shortcuts | Covered | Shell 6.16 "Keyboard", SH-4; UI 4.9 | Stage 1 | "Shortcuts are discoverable in Settings > Accessibility > Keyboard and in a shortcut sheet on `?`". |
| BF-13.14 | Remappable shortcuts | Partial | EX-D15 (extension "command editor", Stage 2); shell 6.9 ("keyboard options") | Stage 2 (extensions only) | Only extension commands can be edited. Remapping browser shortcuts is not specified (searched "remap" / "rebind" / "custom shortcut"). |
| BF-13.15 | Command chains / macros | Missing | — | — | Searched "macro" (hits are Rust macros) / "command chain" / "sequence of commands". |
| BF-13.16 | Peripheral lighting integration | Missing | — | — | Searched "Razer" / "Chroma" / "RGB lighting". |
| BF-13.17 | Experimental feature flags | Covered | Shell 6.9 "No hidden settings", SH-D15; DV "Experimental device APIs" | Stage 1, SH-M4 | "Diagnostic and experimental settings live in a separate 'Diagnostics' page behind an explicit warning". There is deliberately no `about:config`. |
| BF-13.18 | Language settings | Excluded | Shell 6.9 "Language and writing" (Stage 2), ED-D8; shell 6.16 (English-only Stage 1, T12) | Deferred (Stage 2) | "hidden until the ED-D8 Stage 1 amendment is applied". |
| BF-13.19 | Built-in game | Missing | — | — | Searched "offline game" / "dinosaur" / "built-in game". |
| BF-13.20 | Touch Bar customisation | Missing | — | — | Searched "Touch Bar" / "hardware control" / "Mac Touch". |
| BF-14.1 | Screen reader support | Covered | AX spec (AX-D3, AX-D4, AX-D9, AX-D12); shell 6.16, SH-D24 | Stage 1 | Tested with "NVDA, JAWS, VoiceOver and Orca in the AT lab each release". |
| BF-14.2 | High contrast / forced colours | Covered | AX 6.11 (R0, AccessibilityPreferences); shell 6.16 "Colour and contrast" | Stage 1 | "Forced colours and increased contrast are tested as separate modes". Android gets "a few fixed palettes". |
| BF-14.3 | Focus highlight and caret browsing | Covered | AX 6.12, AX-D20; shell 6.16 "Focus" | Stage 1 | "F7 toggles engine-provided caret browsing"; "2 px outline at 3:1 contrast". |
| BF-14.4 | Keyboard-only browsing | Covered | Shell 6.16, SH-4; UI-2 | Stage 1 | "100% of shipped actions reachable and operable by keyboard; 0 traps". |
| BF-14.5 | AI image descriptions | Covered | ML-IMG, ML-D2, ML-D4; MD-D25; HGBA-3 B10 (S_A) | Stage 1 (ML-M1) | Behind the off-by-default `accessibility.model` grant; "never to the page". |
| BF-14.6 | Live caption | Covered | As BF-11.5 | Stage 1 (ML-M3) or Stage 2 | Same rows as BF-11.5. |
| BF-14.7 | Magnifier support / full-page zoom | Covered | AX 6.8 (bounds, Win32 system caret), shell 6.16 "Zoom, reflow and text", SH-27 | Stage 1 | "the Windows adapter mirrors the caret into a Win32 system caret ... so magnifiers follow it"; page zoom up to 400% with reflow. |
| BF-14.8 | Reduced motion and default fonts | Partial | AN-D10, shell 6.16 "Motion", SH-D25 | Stage 1 | Reduced motion is fully specified (R/S/X). Settings for default font family and size are missing (searched "default font" / "standard font" / "font settings"). |
| BF-14.9 | Talk to type / dictation | Covered | AX 6.9 (dictation produces `insertReplacementText`); ED-D5 (IME hosting) | Stage 1 (OS hand-off) | OS dictation enters fields through the input path. Gap: the browser has no speech recognition of its own. |
| BF-15.1 | AI assistant panel | Excluded | UI §6 Stage 3 ("Agent surfaces (only with the AI switch on)"); HGBA-3 §7, B10 | Deferred (Stage 3) | HGBA-3 §7 positions Izcal as an "agent runtime ... rather than as a consumer AI browser". No panel is designed. |
| BF-15.2 | Choice of AI provider | Missing | — | — | Searched "AI provider" / "model provider" / "chatbot" / "cloud model". |
| BF-15.3 | Page summarisation | Excluded | ML-D2, ML-M5; HGBA-3 B10 | Deferred (Stage 2) | "summarise ... Stage 2 tasks under `ai.enabled`". There is no product surface. |
| BF-15.4 | Ask about multiple tabs | Missing | — | — | Searched "multiple tabs" / "compare tabs" / "ask about". |
| BF-15.5 | AI tab organiser | Missing | — | — | Searched "tab organiser" / "group by topic" / "AI" + "tab group". |
| BF-15.6 | Agentic browsing | Partial | HGBA-3 B9, B10, §7; SEC AGT-R1–AGT-R8; shell 6.4 "Agent and automation" class; S1 §9 Stage 2 "agent principal" | Stage 2 (principal), Stage 3 (surfaces) | The security model is specified: origin sets, no self-approval, refusal of phishing-listed URLs. The user-facing task UI, task start, review and confirmation flow are not. |
| BF-15.7 | AI history search | Missing | — | — | Searched "natural-language" + "history" / "AI history" / "semantic search". Shell 6.7 ranking is lexical. |
| BF-15.8 | Writing help | Excluded | ML-D2 (writing assistance, Stage 2), ED-D9 | Deferred (Stage 2) | "ED-D9's learned grammar model is Stage 2 under the AI grant". |
| BF-15.9 | AI memory / personalisation | Missing | — | — | Searched "memory" + "assistant" / "personalisation" / "remembers". |
| BF-15.10 | AI access to personal data sources | Missing | — | — | Searched "mail" / "calendar" / "personal data" + "AI". |
| BF-15.11 | Image generation and editing | Missing | — | — | Searched "image generation" / "generate images" / "edit images". |
| BF-15.12 | Voice / live AI conversation | Missing | — | — | Searched "voice" + "assistant" / "Go Live" / "share screen with". |
| BF-15.13 | Virtual try-on | Missing | — | — | Searched "try-on" / "try on" / "clothes". |
| BF-15.14 | On-device models and their management | Covered | ML 6.3 "model manager", ML-D11, ML-D4, ML-D10 | Stage 1 (ML-M3) | "The model manager ... lists installed models with size and purpose, and offers removal". |
| BF-15.15 | AI master off switch | Covered | HGBA-3 B10, D13; SH-D16, SH-8; S1 §6 "AI switch" | Stage 1 | "off by default, per-profile switch, policy lock, no AI UI surfaces when off, survives updates". |
| BF-15.16 | AI content creation from browsing | Missing | — | — | Searched "report" + "tabs" / "slide deck" / "synthesis". |
| BF-15.17 | Agent / automation server for external AI tools | Covered | DT spec (IIP; BiDi, CDP, DAP and MCP adapters), D-S11, DT-D10 | Stage 1 (BiDi/CDP), MCP with AI on | "WebDriver BiDi, a CDP compatibility profile, DAP, and MCP when AI is on". |
| BF-15.18 | Link previews with AI summary | Missing | — | — | Searched "link preview" / "hover" + "summary" / "peek". |
| BF-16.1 | Sidebar / side panel | Partial | EX-D15 ("sidebar Stage 3"), EX 6.9 ("pending the shell's panel design") | Stage 3 (extension slot) | A future "shell panel design" is referenced but does not exist. Shell 6.6's site panel is per-site only, and there is no sidebar for bookmarks, history or tools. |
| BF-16.2 | Web panels / sidebar apps | Missing | — | — | Searched "web panel" / "sidebar app" / "pin website" + "panel". |
| BF-16.3 | Integrated messengers | Missing | — | — | Searched "messenger" / "WhatsApp" / "Telegram" / "Slack". |
| BF-16.4 | Mail client | Missing | — | — | Searched "mail client" / "e-mail client" / "email". |
| BF-16.5 | Calendar | Missing | — | — | Searched "calendar" (hits are calendar months and Arc's Calendar Preview) / "events". |
| BF-16.6 | Feed reader | Excluded | S1 §9 Stage 2 ("optional W-1 components (feeds, …)"); S1 §6 "Save for offline" (opt-in feed refresh) | Deferred (Stage 2) | Named only, as an optional W-1 component. |
| BF-16.7 | Built-in VPN / secure proxy | Excluded | S1 §6 out-of-scope list ("a browser-operated VPN service"); S1 §6 "Containers" (routes "user-supplied") | — | WireGuard routes come from a config the user supplies (NW 6.13, Stage 2). |
| BF-16.8 | Proxy settings | Covered | NW 6.13 "Proxies", NW 6.14 (policy) | Stage 1 (OS settings), Stage 2 (per-container editing) | "Per-container configuration: direct, the OS settings (brokered), PAC, HTTP and HTTPS proxies ..., SOCKS5". |
| BF-16.9 | Video calling | Missing | — | — | Searched "video call" / "Brave Talk" / "conferencing". |
| BF-16.10 | Productivity service shortcuts | Missing | — | — | Searched "Drive" / "Microsoft 365" / "service shortcut". |
| BF-16.11 | Live work destinations, meeting prep | Missing | — | — | Searched "meeting" / "work items" / "destinations". |
| BF-16.12 | Remote desktop access | Missing | — | — | Searched "remote desktop" / "remote access" / "Chrome Remote". |
| BF-16.13 | Location and directions cards | Missing | — | — | Searched "directions" / "map card" / "location info". |
| BF-16.14 | Phone-number click-to-call | Partial | Shell 6.4 External protocol class ("`mailto:`, `tel:`") | Stage 1 | `tel:` is handed to the OS handler behind a prompt. Dialling through a linked phone service is not specified. |
| BF-17.1 | Install site as app | Partial | PH 6.4.9 (assumes "An installed web app launches in a window with no address bar"); AH 6.8 and 6.12 (app windows for `app://` packages, Stage 3) | — | **Gap:** the phishing spec checks installed web apps, but no spec owns installing a site: manifest handling, install UI and the standalone window. The app host covers signed WASI packages only. |
| BF-17.2 | Web app management | Partial | Shell 6.3 `shell.apps` (for `app://` packages); AH 6.12 | Stage 3 (packages) | Covers app packages, not installed sites. "Open links in app" for sites is not specified. |
| BF-17.3 | Site shortcuts on desktop | Missing | — | — | Searched "desktop shortcut" / "shortcut for website" / "add to dock". The one hit is AH 6.12 ("Install offers a desktop shortcut") and covers app packages only. |
| BF-17.4 | Default browser setting and prompt | Covered | Shell 6.11, SH-D21; S1 §6 "Default-browser handling"; UI R19 | Stage 1 | "The ask uses the OS-sanctioned API exactly once ... A 'no' is final". |
| BF-17.5 | OS notifications for web notifications | Excluded | SH-D17; shell 6.13; Q-SH-10 (open) | Deferred (Stage 2) | "Stage 1 denies the permission honestly". The Stage 2 delivery path is still open: Q-SH-10 asks "OS service, an in-product centre, or both?" |
| BF-17.6 | OS share sheet integration | Missing | — | — | Searched "share sheet" / "Web Share" / "navigator.share" / "share target". |
| BF-17.7 | Handoff / continuity | Missing | — | — | Searched "handoff" (hits are accessibility and IPC) / "continuity" / "continue on". Possibly excluded under "multi-device" (shell 9.1), but not by name. |
| BF-17.8 | Mobile widgets / home-screen shortcuts | Missing | — | — | Searched "home screen" / "widget" / "quick action". |
| BF-17.9 | In-car browser | Missing | — | — | Searched "car" / "automotive" / "in-car". |
| BF-17.10 | Password autofill in other apps (mobile) | Missing | — | — | Searched "other apps" / "autofill service" / "credential provider". |
| BF-17.11 | App installs from the browser (iOS EU) | Excluded | S1 13.2 "iOS" | — | "Out of scope for Stages 1–2." |
| BF-17.12 | Calendar event add (mobile) | Missing | — | — | Searched "calendar event" / "add event" / "ics". |
| BF-18.1 | Developer tools | Covered | DT spec, D-S11, DT-D10 Option B; S1 13.2 "DevTools" | Stage 1 | "Izcal builds its own DevTools frontend, `hg-devtools`". |
| BF-18.2 | DevTools entry points | Partial | Shell 6.2 ("open the shell's inspector from the about:diagnostics page"), 6.8 (Why? "open DevTools with the same query") | Stage 1 | F12, Ctrl/Cmd+Opt+I, "Inspect" in the context menu and a menu item are not specified (searched "F12" / "Inspect" / "open DevTools"). |
| BF-18.3 | Responsive design / device mode | Missing | — | — | Searched "device mode" / "responsive design" / "emulate screen" / "touch emulation". DT has network shaping only. |
| BF-18.4 | Report a broken site / feedback | Covered | CB 6.11, CB-D18; shell 6.6 Blocking row ("report a broken site") | Stage 1 | "consented and local by default". Gap: general product feedback is not covered. |
| BF-18.5 | Internal status pages | Covered | Shell 6.14 "About pages", 6.9 | Stage 1, SH-M4 | `about:version`, `about:build`, `about:licences`, `about:diagnostics`, `about:policies`, `about:shell`. |
| BF-18.6 | Remote debugging and automation | Covered | DT spec (IIP, BiDi, CDP, DAP, MCP); S1 M2 ("WebDriver server") | Stage 1 | Same adapters as BF-15.17; "session-scope checks". |
| BF-18.7 | Beta / preview channels | Covered | S1 7.8 "Release engineering"; shell 6.9 ("Update channel") | Stage 1 | "Nightly, Beta and Release channels". |
| BF-19.1 | Import from other browsers | Covered | Shell 6.12, SH-D19, SH-R2; S1 13.2 "Everyday features" | Stage 1, SH-M3 (M7) | "import from Chrome and Firefox (bookmarks, history and passwords)"; preview, undo within 24 h†. Gap: no Safari importer, and settings and extensions are not imported. |
| BF-19.2 | First-run onboarding | Covered | Shell 6.14, SH-D23, SH-6; UI 4.6 | Stage 1 | "four skippable screens". |
| BF-19.3 | What's new page | Partial | Shell 6.11 ("no ... new-version page re-asks") | — | A new-version page is mentioned in passing but never specified. |
| BF-19.4 | Reset / refresh settings | Covered | Shell 6.14 recovery ("Reset profile settings"), 6.9 ("Reset to default", profile reset) | Stage 1 | "Restores registry defaults; does not touch bookmarks, history, passwords". |
| BF-19.5 | Crash reporting with consent | Covered | D-S4; S1 §6 "Crash reports"; shell 6.14, SH-D29 | Stage 1 | "a prompt at each crash with 'send' pre-selected ... Reports are viewable before sending". |
| BF-19.6 | Troubleshooting / safe mode | Covered | Shell 6.14 recovery window, SH-D22; 6.8 error pages and Why? | Stage 1, SH-M4 | "Start with themes and extensions off"; the window is titled "Troubleshoot". |
| BF-19.7 | In-browser help | Missing | — | — | Searched "help page" / "user guide" / "in-product help" / "F1". |
| BF-19.8 | Lost-device lock and erase | Excluded | SH-D20, Q-SH-5 (no account) | Stage 3 or never | Needs an account, which is excluded. |
| BF-20.1 | Enterprise policies | Partial | Shell 6.9 (policy rows, "A policy can set only registry settings"), 6.14 `about:policies`; NW 6.14 "Enterprise policy surface"; HGBA-3 B10 (AI lock) | Stage 1 | Policy effects are specified in many places. No spec owns how policies are delivered (GPO, MDM profiles, JSON), their schema or their precedence. The policy page has two names (BF-8.10). |
| BF-20.2 | Kiosk mode | Missing | — | — | Searched "kiosk" (hits are incidental mentions in DV and WebCrypto) / "single-site" / "locked-down". |
| BF-20.3 | Parental controls | Missing | — | — | Searched "parental" / "family" / "supervised" / "screen time". |
| BF-20.4 | Kids mode | Missing | — | — | Searched "kids" / "child-safe" / "children". |
| BF-20.5 | Data saver / lite mode | Missing | — | — | Searched "data saver" / "lite mode" / "data saving". |
| BF-20.6 | Shared device / multi-user | Partial | SG §1 (shared-device risk), SG 6.9 and SG-D7 (per-container keys in the OS key store); shell 4.1 ("profiles later") | — | Separation between OS users follows from OS-bound keys. A per-person profile UI on one OS account is missing (see BF-4.1). |
| BF-21.1 | Tab grid / switcher (mobile) | Missing | — | — | Searched "tab grid" / "tab switcher" / "swipe". No mobile shell spec exists; S1 §9 schedules Android for Stage 2. |
| BF-21.2 | Swipe gestures (mobile) | Missing | — | — | Searched "swipe" / "edge swipe" / "gesture navigation". |
| BF-21.3 | Pull to refresh | Missing | — | — | Searched "pull to refresh" / "pull-to-refresh" / "overscroll reload". |
| BF-21.4 | Offline page saving | Covered | S1 §6 "Save for offline" | Stage 2 | "User-invoked. Default format is a frozen DOM-and-resources snapshot". Gap: no mobile surface. |
| BF-21.5 | Mobile privacy hub | Missing | — | — | Searched "privacy hub" / "mobile dashboard" / "blocked trackers". Desktop has per-site counts only (BF-7.9). |
| BF-21.6 | Camera search / QR scan | Missing | — | — | Searched "camera search" / "QR scan" / "scan QR". PH checks a QR-decoded URL but nothing scans. |

---

## Summary

### 1. Counts per status per group

"Excluded" has three kinds, shown in separate columns. **Refused** means a decision or non-goal turns the feature down. **Stage 3 or never** means the feature depends on the account and sync exclusion (shell 9.1, SH-D20, Q-SH-5). **Deferred** means the feature is named for a later stage but has no design.

| Group | Rows | Covered | Partial | Excluded: refused | Excluded: Stage 3 or never | Excluded: deferred | Missing |
|---|---|---|---|---|---|---|---|
| 1 Windows, tabs and sessions | 24 | 9 | 1 | 2 | 1 | 4 | 7 |
| 2 Address bar, search and navigation | 17 | 3 | 2 | 2 | 0 | 1 | 9 |
| 3 Bookmarks, history, reading list, start page | 14 | 3 | 6 | 2 | 0 | 0 | 3 |
| 4 Profiles, accounts, sync, cross-device | 11 | 1 | 2 | 0 | 6 | 0 | 2 |
| 5 Downloads, saving, printing, PDF | 11 | 7 | 0 | 0 | 0 | 2 | 2 |
| 6 Passwords, passkeys, autofill, payments | 15 | 4 | 0 | 1 | 0 | 2 | 8 |
| 7 Privacy controls | 18 | 12 | 4 | 1 | 0 | 0 | 1 |
| 8 Security and safety UI | 12 | 6 | 4 | 0 | 0 | 0 | 2 |
| 9 Site permissions and page info | 12 | 8 | 4 | 0 | 0 | 0 | 0 |
| 10 Content, reading and page tools | 18 | 3 | 3 | 1 | 0 | 2 | 9 |
| 11 Media | 10 | 3 | 1 | 0 | 0 | 0 | 6 |
| 12 Performance and resource controls | 8 | 3 | 3 | 1 | 0 | 0 | 1 |
| 13 Extensions, themes, customisation | 20 | 8 | 2 | 1 | 0 | 1 | 8 |
| 14 Accessibility | 9 | 8 | 1 | 0 | 0 | 0 | 0 |
| 15 AI features and off switches | 18 | 3 | 1 | 0 | 0 | 3 | 11 |
| 16 Built-in apps and services | 14 | 1 | 2 | 1 | 0 | 1 | 9 |
| 17 Web apps and OS integration | 12 | 1 | 2 | 1 | 0 | 1 | 7 |
| 18 Developer and power-user entry points | 7 | 5 | 1 | 0 | 0 | 0 | 1 |
| 19 Setup, onboarding, maintenance | 8 | 5 | 1 | 0 | 1 | 0 | 1 |
| 20 Enterprise, family, special modes | 6 | 0 | 2 | 0 | 0 | 0 | 4 |
| 21 Mobile-specific | 6 | 1 | 0 | 0 | 0 | 0 | 5 |
| **Total** | **270** | **94** | **42** | **13** | **8** | **17** | **96** |

So 94 rows (35%) are Covered, 42 (16%) Partial, 38 (14%) Excluded and 96 (36%) Missing. The owner was right to suspect gaps. The gaps are not spread evenly, though:

- **Well covered:** accessibility (8/9), permissions (8/12), privacy (12/18), security UI, downloads and PDF, and developer entry points. These are the areas the existing specs chose to compete on.
- **Missing:** the gaps cluster in seven places. These are AI-on product features, built-in suite services, mobile, OS and web-app integration, page utilities, the address bar beyond the basics, and payments and shopping.

### 2. All Missing rows (96)

- **Group 1:** BF-1.11 tab hover preview; BF-1.15 named saved sessions; BF-1.17 link peek; BF-1.18 auto-archive of idle tabs; BF-1.19 custom tab names or emoji; BF-1.20 Journeys / topic history; BF-1.24 open tabs in the OS app switcher (mobile).
- **Group 2:** BF-2.4 search-engine choice screen (EU DMA); BF-2.5 site search keywords and @ scopes; BF-2.6 calculator and unit conversion; BF-2.8 voice search; BF-2.9 visual search; BF-2.11 address bar position (mobile); BF-2.15 mouse gestures; BF-2.16 a bundled search engine of Izcal's own; BF-2.17 a desktop search bar outside the browser.
- **Group 3:** BF-3.4 history statistics; BF-3.6 collections and pinboards; BF-3.13 Wayback Machine fallback on a 404.
- **Group 4:** BF-4.2 guest mode; BF-4.10 FedCM sign-in UI.
- **Group 5:** BF-5.9 AI alt text for images in PDFs; BF-5.11 easy file picker.
- **Group 6:** BF-6.2 password generator; BF-6.7 third-party password manager integration; BF-6.10 virtual cards, BNPL and card benefits; BF-6.11 wallet payment sheet; BF-6.12 price tracking and coupons; BF-6.13 store reviews; BF-6.14 crypto wallet; BF-6.15 creator rewards.
- **Group 7:** BF-7.2 locking private tabs with biometrics.
- **Group 8:** BF-8.8 safety check; BF-8.9 unwanted-software cleanup.
- **Group 10:** BF-10.5 read aloud; BF-10.9 share page; BF-10.11 QR code for page; BF-10.12 screenshot tool; BF-10.13 eyedropper; BF-10.14 notes; BF-10.15 page key facts; BF-10.16 request desktop site (mobile); BF-10.17 dark mode for web content.
- **Group 11:** BF-11.2 automatic PiP on tab switch; BF-11.4 casting; BF-11.6 media playlist; BF-11.7 built-in music player; BF-11.8 volume booster; BF-11.10 microphone volume.
- **Group 12:** BF-12.7 startup boost / background running.
- **Group 13:** BF-13.6 extensions on mobile; BF-13.7 AI-generated extensions; BF-13.9 AI theme generator; BF-13.11 toolbar customisation; BF-13.15 command chains and macros; BF-13.16 peripheral lighting; BF-13.19 built-in game; BF-13.20 Touch Bar customisation.
- **Group 15:** BF-15.2 choice of AI provider; BF-15.4 asking about several tabs; BF-15.5 AI tab organiser; BF-15.7 AI history search; BF-15.9 AI memory; BF-15.10 AI access to personal data; BF-15.11 image generation; BF-15.12 voice and live AI conversation; BF-15.13 virtual try-on; BF-15.16 AI content creation from browsing; BF-15.18 AI link previews.
- **Group 16:** BF-16.2 web panels; BF-16.3 messengers; BF-16.4 mail client; BF-16.5 calendar; BF-16.9 video calling; BF-16.10 productivity-service shortcuts; BF-16.11 live work destinations; BF-16.12 remote desktop; BF-16.13 location and directions cards.
- **Group 17:** BF-17.3 site shortcuts on the desktop; BF-17.6 OS share sheet; BF-17.7 handoff / continuity; BF-17.8 mobile widgets; BF-17.9 in-car browser; BF-17.10 password autofill in other apps (mobile); BF-17.12 adding calendar events (mobile).
- **Group 18:** BF-18.3 responsive design / device mode in DevTools.
- **Group 19:** BF-19.7 in-browser help.
- **Group 20:** BF-20.2 kiosk mode; BF-20.3 parental controls; BF-20.4 kids mode; BF-20.5 data saver.
- **Group 21:** BF-21.1 tab grid; BF-21.2 swipe gestures; BF-21.3 pull to refresh; BF-21.5 mobile privacy hub; BF-21.6 camera search and QR scan.

### 3. All Partial rows (42), with what is missing

| # | Feature | What is missing |
|---|---|---|
| BF-1.22 | UI auto-hide / compact | Auto-hiding the toolbar and tab strip. Compact density is specified. |
| BF-2.3 | Default search engine choice | A Search settings row, the default engine list, adding a custom engine. Search is referenced in shell 6.7 and 6.8 and EX 6.9 but never designed. |
| BF-2.14 | Home button / homepage | A homepage or startup-page setting and a home button. Only EX's override prompt exists. |
| BF-3.2 | Bookmark manager with tags | Tags. The bookmark record has folders and a saved reason only. |
| BF-3.5 | Reading list | The list itself, its surface and read/unread state. Only Stage 2 "save for offline" exists. |
| BF-3.8 | Speed dial | Pinned, user-arranged tiles and tile groups. The new-tab page shows top sites from history only. |
| BF-3.11 | Start page shows library | Bookmarks and reading list on the start page. |
| BF-3.12 | Page change notifications | Watching a page and sending an alert. Change is detected only on revisit. |
| BF-3.14 | Continue-where-you-left-off cards | Resuming a task or search. Only recently closed tabs are specified. |
| BF-4.1 | Multiple profiles | Creating, switching and picking profiles ("profiles later" in shell 4.1). |
| BF-4.8 | Work/personal separation, SSO | Automatic container assignment per site and a managed work profile with SSO. |
| BF-7.6 | Third-party cookie setting | A global block switch with per-site exceptions. Partitioning plus the Storage Access API is the only mode. |
| BF-7.9 | Privacy report | A browser-wide report or dashboard. Per-site counts exist. |
| BF-7.13 | Clear browsing data | The dialog design: categories, time ranges, clear-on-exit. |
| BF-7.18 | Element hiding | A point-and-hide element picker. Users can already write cosmetic rules. |
| BF-8.2 | Enhanced protection mode | A user-facing mode or setting. The jitless tier per container and per site exists. |
| BF-8.3 | Scareware blocker | Detecting a full-screen scam page and getting the user out of it. |
| BF-8.6 | Certificate viewer | What the "certificate details expander" shows. |
| BF-8.11 | Lookalike-domain warning | The owner (Q-UI-5 / Q-PH-16 is open) and the detection rules. |
| BF-9.7 | Popup blocker | A blocked-popup indicator and a per-site allow. Popups are gated on activation in the engine. |
| BF-9.9 | In-use indicators | Indicators with a stop control for camera, microphone, location and tab sharing. The device indicator is done. |
| BF-9.10 | Screen-share picker | A browser tab picker and a "sharing this tab" bar. The OS picker is used. |
| BF-9.11 | Per-site content toggles | Per-site toggles for images and other content. JavaScript modes are designed for Stage 2. |
| BF-10.3 | Text-only zoom, minimum font size | Shell controls. The engine factors exist (AX-D19, ST-D17). |
| BF-10.10 | Link to highlighted text | Creating and copying a text-fragment link. Following one is specified. |
| BF-10.18 | View page source | An owner and design for `view-source:`. PH and AGT-R3 assume it exists. |
| BF-11.3 | Global media controls | An in-browser hub that controls media across tabs. Media Session reaches the OS controls. |
| BF-12.3 | Energy saver | A browser-wide mode. Izcal follows the OS low-power signal and plans a per-site battery limit in Stage 2. |
| BF-12.5 | Task manager | A per-tab and per-extension view with "end process". `about:diagnostics` lists processes. |
| BF-12.6 | Preloading settings | Any user setting. Speculation Rules are a Stage 2 engine item. |
| BF-13.12 | Deep UI mods | A mod system beyond restricted user CSS. |
| BF-13.14 | Remappable shortcuts | Editing browser shortcuts. Only the extension command editor (Stage 2) exists. |
| BF-14.8 | Default fonts | Default font family and size settings. Reduced motion is done. |
| BF-15.6 | Agentic browsing | The user-facing agent task UI. The security model in HGBA-3 B9 and AGT-R is specified. |
| BF-16.1 | Sidebar | The "shell panel design" that EX-D15 says it is waiting for. |
| BF-16.14 | Click-to-call | Dialling through a linked phone. `tel:` goes to the OS through a prompt. |
| BF-17.1 | Install site as app | Any owner. PH 6.4.9 assumes installed web apps and standalone windows; the app host covers only `app://` packages. |
| BF-17.2 | Web app management | Management of installed sites. `shell.apps` covers packages only. |
| BF-18.2 | DevTools entry points | F12, the Inspect context-menu item, the menu item and the shortcut. |
| BF-19.3 | What's new page | A design. The page is mentioned only in shell 6.11. |
| BF-20.1 | Enterprise policies | Policy delivery (GPO, MDM, JSON), the schema and precedence. "Enterprise policy" appears in 12 specs. |
| BF-20.6 | Shared device / multi-user | A per-person profile UI on one OS account. |

### 4. Deferred rows with no design (17)

These need a design before their stage starts: BF-1.4 tab groups (Q-SH-9 open); BF-1.5 saved groups; BF-1.7 vertical tabs; BF-1.8 split view; BF-2.7 command palette (a measured trial); BF-5.5 print UI; BF-5.10 and BF-15.3 summarise; BF-6.8 address autofill; BF-6.9 payment autofill; BF-10.4 reader mode; BF-10.6 translation (behind `ai.enabled`); BF-13.18 language settings; BF-15.1 AI assistant panel (Stage 3); BF-15.8 writing help; BF-16.6 feed reader (W-1); BF-17.5 OS notification delivery (Q-SH-10 open).

### 5. Covered rows with minor sub-feature gaps

- **Windows and tabs:** BF-1.1 has no tab drag between windows and no window merge. BF-1.2 lacks duplicate, close others, close to the right and move to a new window. BF-1.12 has no audible indicator and no site mute. BF-1.21 has no option to keep the toolbar in full screen.
- **Start page and saving:** BF-3.7 has no new-tab page customisation. BF-5.4 offers no HTML-only or web-archive format.
- **Passwords:** BF-6.4 has no browser-side passkey list. BF-6.5 does not cover passkey export. BF-6.6 does not gate filling behind device unlock.
- **Privacy and security:** BF-7.1 has no UI for opening or marking a private window. BF-8.5 has no settings row. BF-9.8 has no "block all media" option.
- **Media and performance:** BF-11.5 has no translated captions. BF-12.1 has no levels. BF-12.8 has no settings row.
- **Extensions and themes:** BF-13.2 has no ratings or reviews. BF-13.4 cannot install from another vendor's store. BF-13.8 has no theme gallery and refuses extension themes.
- **Accessibility:** BF-14.9 has no speech recognition of its own.
- **Feedback and import:** BF-18.4 has no general feedback. BF-19.1 has no Safari importer.

### 6. Inconsistencies between specs found during the audit

These are not feature gaps, but each was found while checking rows. No file in `reports/` was edited.

1. **PDF annotation stage.** S1 §6's PDF row says "Stage 2 adds local annotate, sign and redact". PD 9.5 moves annotation authoring to Stage 1 (PD-M3). See BF-5.8.
2. **Shell milestone mapping.** Shell 9.2 maps SH-M2 to "prompt layer, ledger, site panel" (M3) and SH-M4 to "downloads; settings ...; recovery window" (M5). S1 §8.1's spec-group table says "SH-M4 site panel, settings and address bar at M5; SH-M5 downloads, import/export, onboarding and recovery at M6". This audit uses shell 9.2's mapping.
3. **Policy page name.** The shell uses `about:policies` (6.9, 6.14). NW 6.14 says "Each policy is visible on `izcal://policy`".
4. **Installed web apps have no owner.** PH 6.4.9 checks "an installed web app ... in a window with no address bar" and the app host contrasts app windows with "an installed site's standalone window". No spec defines installing a site.
5. **`view-source:` has no owner.** PH 6.2 and AGT-R3 assume the scheme exists.
6. **Search settings are not designed.** EX 6.9 refuses the search-provider override because "Izcal's search settings are the user's", but no spec designs search settings.
7. **Sidebar slot depends on a missing design.** EX-D15 stages the sidebar slot until "the shell's panel design lands". No such design is scheduled in shell 9.1 or 9.2.

### 7. Proposed grouping of Missing and Partial rows into candidate specs and amendments

Each line gives the candidate, its type, the rows it absorbs, and one line of reasoning. Rows marked † are candidates to **record as explicit refusals** rather than to build. Recording them costs a line in a non-goals section and keeps the audit from reopening them.

| # | Candidate | Type | Rows | Reasoning |
|---|---|---|---|---|
| A | **Installed web apps and OS integration** | New spec | BF-17.1, 17.2, 17.3, 17.6, 17.7, 12.7, 10.9 (share), 17.5 (settle Q-SH-10) | Two accepted specs (PH 6.4.9, AH) assume installed sites exist, yet nothing owns the manifest, install flow, standalone window, share sheet or OS hand-off. This is the largest structural gap. |
| B | **Passwords, passkeys and autofill** | New spec (shell 6.20's reversal trigger already names "a dedicated password-manager spec") | BF-6.2, 6.3 (local weak/reused checks), 6.5/6.6 gaps, 6.7, 6.8, 6.9, 6.10†, 6.11, 4.10 (FedCM UI), 17.10 | Stage 2 address and payment autofill have prompt classes but no store or fill design. FedCM and wallet sheets are credential surfaces with the same consent rules. |
| C | **AI features with the switch on** | New spec, under ML and HGBA-3 §7 | BF-15.2, 15.4, 15.5, 15.6 (UI), 15.7, 15.9†, 15.10†, 15.11†, 15.12†, 15.13†, 15.16†, 15.18, 13.7†, 13.9†, 2.12, 5.9, and the deferred 15.1, 15.3, 15.8, 5.10, 10.6 | ML-D2 schedules Stage 2 tasks with no product surface, and HGBA-3 §7 bets on "agent runtime, not consumer AI browser". One document should list what Izcal builds and what it refuses, decide provider choice, and settle whether translation and reading tools should stay behind `ai.enabled`. |
| D | **Reading tools: reader, read aloud, translation** | New spec | BF-10.4 (deferred), 10.5, 10.6, 10.15, 10.17, 3.5 (offline queue) | The inventory and S1 §6 ("Reader rendering on top") point to one content-extraction pipeline. ML-OCR already names "Reading Mode", yet there is no owner. Translation is currently AI-gated, which may not match user expectations. |
| E | **Library: reading list, tags, collections, notes, start page, sidebar** | Amendment to shell 6.7, plus the "shell panel design" EX-D15 waits for | BF-3.2, 3.4, 3.5, 3.6, 3.8, 3.11, 3.12, 3.13†, 3.14, 10.14, 16.1, 16.2 | The one-store schema (SH-D8) is the natural home. New record types (reading-list entry, tag, collection, note) and a sidebar host should be specified before the schema freezes at SH-M3. |
| F | **Tabs, windows and sessions extras** | Amendment to shell 6.8, plus the Stage 2 designs | BF-1.2 gaps, 1.11, 1.15, 1.17, 1.18, 1.19, 1.20†, 1.22, 1.21 gap, and the deferred 1.4/1.5/1.7/1.8 (Q-SH-9) | The session store and tab record are shell-owned and versioned. Named sessions and the group model should be fixed before the Stage 1 session format ships, since the inventory warns that retrofitting is costly. |
| G | **Address bar and search settings** | Amendment to shell 6.8 and 6.9 | BF-2.3, 2.4, 2.5, 2.6, 2.14, 2.8†, 2.9†, 2.15†, 2.16†, 2.17† | The engine choice is assumed by three specs but never designed. An EU choice screen is a legal requirement. Keywords and calculator results are ranked providers in the existing autocomplete model. |
| H | **Profiles, guest and shared devices** | New shell section, or a short spec with SG | BF-4.1, 4.2, 4.8, 20.6, 7.2 | "Profiles later" (shell 4.1) is referenced by the ML, EX and B10 per-profile switches. The relationship between profiles and containers, a guest mode and a picker need one decision. |
| I | **Privacy, site data and permission surfaces** | Amendment to shell 6.4, 6.6 and 6.9, with CB and MD | BF-7.6, 7.9, 7.13, 7.18, 9.7, 9.9, 9.10, 9.11, 21.5 | Each is a surface over facts the broker already holds (decision log, ledger, capture grants). The missing pieces are the clear-data dialog, in-use indicators, popup and image controls, and a report page. |
| J | **Security surfaces** | Amendment to shell, PH and NW | BF-8.2, 8.3, 8.6, 8.8, 8.11 (settle Q-UI-5) | A safety-check page and a certificate viewer read existing broker facts. The lookalike-class ownership question is still open and blocks SH-11's scope. |
| K | **Page utilities** | Amendment to shell (menus and context menus) | BF-10.3, 10.10, 10.11, 10.12, 10.13, 10.18, 18.2 | Small shell features over engine capabilities that already exist (compositor capture, text directives, the `view-source:` scheme). Most need only a menu entry, a broker endpoint and a prompt-class check. |
| L | **Media surfaces and casting** | Amendment to MD 6.12 and shell; casting as its own small spec | BF-11.2, 11.3, 11.4, 11.10, 11.6†, 11.7†, 11.8† | A media hub and auto-PiP are shell surfaces over Media Session and PiP. Casting needs per-ecosystem protocol work and a decision on licensing. |
| M | **Performance and resource controls** | Amendment to FP and shell 6.9 | BF-12.3, 12.5, 12.6 | FP-D15 already holds the data. What is missing is user-facing controls: an energy mode, a task-manager view and a preload setting. |
| N | **Customisation: toolbar, shortcuts, fonts, mods** | Amendment to shell 6.2, 6.9 and 6.16 | BF-13.11, 13.12, 13.14, 13.15, 14.8, 13.16†, 13.19†, 13.20† | The settings registry (SH-D15) can carry editable shortcuts, toolbar layout and default fonts. The deep-mod question is constrained by D-S2's W-1 rule. |
| O | **Enterprise and managed deployment** | New spec | BF-20.1, 20.2, 4.8 (SSO), plus the BF-8.10 name conflict | "Enterprise policy" appears in 12 specs (among them shell, NW, HGBA-3 B10, DV, EX, ML, PH, SG and AH) with no owner for delivery, schema or precedence. Kiosk mode depends on it. |
| P | **Mobile shell (Android)** | New spec, Stage 2 | BF-1.24, 2.11, 10.16, 13.6, 17.8, 17.10, 17.12, 21.1, 21.2, 21.3, 21.5, 21.6 | S1 §9 schedules the Android engine for Stage 2, and AX, NW and DV already have Android rows, but no shell spec covers a phone. |
| Q | **Help, feedback and update notes** | Amendment to shell 6.14 | BF-19.3, 19.7, plus the BF-18.4 general-feedback gap | Local help and a what's-new page fit the sealed shell image and the no-network rule. |
| R | **DevTools device mode** | Amendment to the DevTools spec | BF-18.3 | DT-D10 Option B names the CDP Page and Overlay domains but no viewport or touch emulation. This is table stakes for web developers. |
| S | **Product-identity non-goals** | Amendment to shell §4.3 or UI §8 (record refusals) | BF-16.3†, 16.4†, 16.5†, 16.9†, 16.10†, 16.11†, 16.12†, 16.13†, 6.12†, 6.13†, 6.14†, 6.15†, 20.3, 20.4, 20.5 | These conflict with the lean-core, privacy and no-network posture, or belong to the OS (family controls). Writing them down as non-goals, or as AH Stage 3 app candidates for mail and calendar, closes them honestly. |

The sync and multi-device rows (BF-1.23, 4.3–4.7, 4.9, 19.8) are already Excluded by shell 9.1 and Q-SH-5. The inventory argues that a future sync must be end-to-end encrypted from the start. If the owner ever reopens Q-SH-5, that becomes candidate T, a sync and accounts spec. Until then, no work is proposed.
