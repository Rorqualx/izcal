# Verification of Missing rows: browser product features (BF) and OS integration and operations (PO)

Checked 24 October 2026 against `reports/` in the `feature-audit` worktree. This is an independent check of the **Missing** rows in `map_browser_product_features.md` (96 BF rows) and `map_os_integration_operations.md` (59 PO rows): 155 rows.

**Method.** For every row I searched all 30 spec files in `reports/` (the three review documents excluded, as the mapper did) using regular expressions with terms the mapper did not use: synonyms, API and OS-interface names, decision IDs, and the names of nearby features. I then read every hit in context. Before the row-by-row search I read the scope and refusal lists that most often decide a row: S1 §6 (including the "Out of scope for Stages 1 and 2" list), S1 §9 and 13.2, shell 6.7–6.14, 6.19–6.20 and 9.1, HGBA-3 §7, §10 and B10, ML 6.2 and 6.5, SEC 6.12 and 6.16, and NW 6.6 and 6.14. I used the mapper's own status rules: Partial means a spec names the feature or designs part of it, and Excluded includes a recorded deferral. Where a spec only names a feature (for example, it hands it to a spec that does not exist), I record Partial (named only), because by the mapper's rule a named feature is not Missing. The "Evidence" column gives my extra search terms for confirmed rows, and the spec section with a short quotation for changed rows.

## Summary

| | Rows | Confirmed missing | Actually covered | Actually partial | Actually excluded / deferred |
|---|---|---|---|---|---|
| BF | 96 | 87 | 0 | 8 | 1 |
| PO | 59 | 53 | 0 | 3 | 3 |
| **Total** | **155** | **140** | **0** | **11** | **4** |

**Changed verdicts (15).** No row turned out to be fully covered. Each change is small: a feature that a spec names, partly designs, or refuses by a decision the mapper did not cite.

| # | Feature | New verdict | Why |
|---|---|---|---|
| BF-4.10 | Third-party sign-in prompts (FedCM UI) | Partial | Named only. SEC 6.15: "CHIPS and FedCM … are a privacy-spec question, out of scope here"; SEC 11.4 same. No sign-in dialog design; the privacy spec does not exist. |
| BF-6.11 | Wallet payment sheet | Partial | Named only. DV 6.14: "Payments stay in Payment Request"; DM 6.5 lists "payment" as activation-gated. No payment sheet or wallet design. |
| BF-10.5 | Read aloud | Partial | PD 6.9: "Find, selection highlights, copy …, read-aloud and the semantic subtree all read this model"; PD-15 "Read aloud" target. Only PDF, via the OS screen reader; no speech engine or read-aloud UI for pages. |
| BF-10.12 | Screenshot tool | Partial | GR 6.23 O4: "Canonical screenshots from the CPU path for DevTools, WebDriver BiDi and … agents". Capture exists for tools; no user capture, region select or annotate tool. |
| BF-11.10 | Microphone volume adjustment | Partial | MD 6.8 (MD-D16): `hg-voice` is "Izcal's own acoustic echo canceller, noise suppressor and gain control". Automatic gain only; no user input-level control. |
| BF-15.9 | AI memory / personalisation | Partial | SEC 6.12 AGT-R2: "Agent memory is per task by default. Anything kept across tasks needs confirmation". Rule only; no memory feature or view/delete UI. |
| BF-15.10 | AI access to personal data sources | Partial | SEC 6.12 AGT-R6 names agent tasks on "a read-write origin in a sensitive class (mail, banking …)". Security rule only; no feature reading mail, calendar or files. |
| BF-17.7 | Handoff / continuity | Excluded | Shell 9.1 "Stage 3 or never": "Sync and multi-device" — "the product says there is no account and no sync". Handoff is a multi-device feature. |
| BF-19.7 | In-browser help | Partial | Shell 6.8 Why? surface answers "a bounded set of questions about the current state"; DV 6.18: "Each error has a short help page". No general help content or help search. |
| PO-1.2 | [SB] Windows enterprise MSI | Excluded | Stage 1 only. SEC-D14 decided "(a) for Stage 1 … per-user installs only"; reversal trigger: "Enterprise deployment (Stage 2) needs per-machine installs". No MSI design for Stage 2. |
| PO-3.7 | Staged (percentage) rollout | Partial | Named only. S1 §6 Updates row: "Signed, staged updates"; S1 §1 updater "staged updates". No percentage, cohort or halt design. |
| PO-7.9 | TLS version and cipher policies | Partial | HGBA-2 4.7: "hybrid by default, policy-controlled fallback" for post-quantum TLS. NW 6.5 fixes suites by design; no min-version, cipher or FIPS policy (absent from NW 6.14 list). |
| PO-7.16 | OS VPN and per-app VPN compatibility | Partial | NW 6.6 (NW-D8): OS split-DNS rules read "so enterprise and VPN names resolve as the OS would"; reversal on "VPN breakage". Per-app and Always On VPN routing unstated. |
| PO-8.14 | Continuity and cross-device handoff | Excluded | Shell 9.1 "Stage 3 or never": "Sync and multi-device"; S1 §9 Stage 3: "Sync (if wanted)". |
| PO-9.22 | OS speech synthesis and recognition | Excluded | Deferred, named only. EX 9.1: "TTS … Stage 3 / Each needs a host surface (… OS voices …)". No owner for OS speech or `speechSynthesis`. |

**Notes for the mapper.**

- **Two rows reverse with the multi-device decision.** BF-17.7 and PO-8.14 (Handoff and continuity) fall under shell 9.1's "Stage 3 or never: Sync and multi-device". BF-1.5 already uses that row to exclude syncing tab groups, so these two rows should follow the same rule.
- **PO-1.2 is a Stage 1 refusal.** SEC-D14 decides per-user installs only for Stage 1, and its reversal trigger names Stage 2 enterprise per-machine installs. The real gap is that nobody owns the Stage 2 decision. It is not true that no spec mentions it.
- **Five "named only" rows** (BF-4.10, BF-6.11, PO-3.7, PO-9.22, and partly BF-10.12) are thin. They change status only because the mapper's own rule says a named feature is not Missing. They still have no design, so they still need work.
- **Enterprise policy delivery (PO-6.1 to 6.6) is confirmed missing.** The policy *consumers* are well specified: shell 6.9's per-row "can be set by policy" and "Set by your organisation", `about:policies`, NW 6.14's policy list, DV 6.10's policy keys, the `hg-policy` crate (S1 2.1) and the "signed policy" in DV and AH. What is missing is the mechanism by which a policy reaches the broker (ADMX, MDM, configuration profiles, a policy file). A future spec can rely on the consumers that already exist.
- **Mobile rows** (BF-1.24, 2.11, 10.16, 13.6, 17.8, 17.10, 17.12, 21.x) are confirmed missing. S1 schedules the Android engine for Stage 2 and refuses iOS for Stages 1–2 (S1 13.2), but no spec covers a mobile shell. The platform has a stage; these features have none.

## All rows

| # | Feature | Mapper said | Verdict | Evidence |
|---|---|---|---|---|
| BF-1.11 | Tab hover preview | Missing | Confirmed missing | Also tried "hover … tab", "tab thumbnail", "preview of a tab": 0 hits. Shell 6.8 tab strip lists title, container, mute, close only. |
| BF-1.15 | Named saved sessions | Missing | Confirmed missing | Tried "saved session/window", "session snapshot", "session export/file": none. Shell 6.8 session store is restore-only, unnamed. |
| BF-1.17 | Link peek / glance | Missing | Confirmed missing | Tried "quick look", "preview link", "link card", "in a popup/overlay": 0 hits. |
| BF-1.18 | Auto-archive of idle tabs | Missing | Confirmed missing | Tried "tabs … archive/expire/auto-close", "inactive tabs": none. Shell 6.8 cold-tab discard keeps the tab record (not archive). |
| BF-1.19 | Custom tab names / emoji | Missing | Confirmed missing | Tried "rename a tab", "custom title", "tab label override": only shell 6.2 "Custom title bars" (window frame, unrelated). |
| BF-1.20 | Journeys / topic history | Missing | Confirmed missing | Tried "history cluster", "grouped visits", "by topic": 0 hits. Shell 6.7 History page groups "by day" only. |
| BF-1.24 | Open-tab list in OS app switcher (mobile) | Missing | Confirmed missing | Tried "mobile shell", "Android shell/UI", "recents": 0 hits. S1 §1: "Stage 1 ships desktop only; Android is S-8 in Stage 2" (platform, not feature). |
| BF-2.4 | Search engine choice screen (EU DMA) | Missing | Confirmed missing | Tried "choice screen", "choose … search", "default search", "chosen engine". Shell 6.7/6.8 say "the chosen search engine" but never how it is chosen; no choice screen. |
| BF-2.5 | Site search keywords / @ scopes | Missing | Confirmed missing | Tried "search shortcut", "keyword search", "custom search", "OpenSearch", "%s": none. Shell 6.8 autocomplete sources only. |
| BF-2.6 | Calculator and unit conversion | Missing | Confirmed missing | Tried "arithmetic", "instant answer", "inline answer", "conversion": hits are JS/IDL internals only. |
| BF-2.8 | Voice search | Missing | Confirmed missing | Tried "speech-to-text", "SpeechRecognition", "speech input", "ASR": ML-ASR is captions only (ML 6.5). |
| BF-2.9 | Visual search (Lens) | Missing | Confirmed missing | Tried "search by image", "image query", "Lens", "similar images": 0 hits. |
| BF-2.11 | Address bar position (mobile) | Missing | Confirmed missing | Tried "bottom bar", "toolbar position", "one-handed", "thumb": hits are PDF thumbnails/scroll thumbs. Shell 6.9 has only "tab strip position (Stage 2)". |
| BF-2.15 | Mouse gestures | Missing | Confirmed missing | Tried "rocker", "pointer gesture", "stroke gesture": only ED 6.11 drag gesture rule. |
| BF-2.16 | Bundled own search engine | Missing | Confirmed missing | Tried "Izcal search", "own search engine", "search provider/service": only EX §4.4 (extension search-provider overrides refused). |
| BF-2.17 | Desktop search bar outside the browser | Missing | Confirmed missing | Tried "search box", "search widget", "taskbar", "deskband", "Spotlight": TX 6.1 taskbar/jump-list string rule only. |
| BF-3.4 | History statistics | Missing | Confirmed missing | Tried "history statistics/report/dashboard", "time spent", "screen time": none. Shell 6.7 `aggregate` record has data but no surface. |
| BF-3.6 | Collections / pinboards | Missing | Confirmed missing | Tried "collection" (DOM collections only), "scrapbook", "moodboard", "reading list" (EX lists `readingList` as an API name only). |
| BF-3.13 | Wayback Machine fallback on 404 | Missing | Confirmed missing | Tried "Internet Archive", "cached/archived copy": citations only. Shell 6.7 marks `gone` (404/410) health but offers no archive fallback. |
| BF-4.2 | Guest mode | Missing | Confirmed missing | Tried "guest profile/window", "temporary profile", "shared computer": DV 6.10 grant default "for … a shared machine" only. Private windows ≠ guest. |
| BF-4.10 | Third-party sign-in prompts (FedCM UI) | Missing | Actually partial | Named only. SEC 6.15: "CHIPS and FedCM … are a privacy-spec question, out of scope here"; SEC 11.4 same. No sign-in dialog design; the privacy spec does not exist. |
| BF-5.9 | PDF image alt text (AI) | Missing | Confirmed missing | Tried "alt text", "image description" + PDF: ML-IMG describes page images (ML 6.5); PDF editing (Stage 2) adds annotate/sign/redact, never alt-text authoring. |
| BF-5.11 | Easy file picker | Missing | Confirmed missing | Tried "file chooser", "input type=file", "file dialog", "portal": none designs upload suggestions. |
| BF-6.2 | Password generator | Missing | Confirmed missing | Tried "generate password", "password strength/suggest", "passwordrules": 0 hits. Shell 6.12 password manager is save/fill only. |
| BF-6.7 | Third-party password manager integration | Missing | Confirmed missing | Tried "1Password", "Bitwarden", "KeePass", "autofill provider", "external password": 0 hits. Shell 6.20 options are built-in or OS manager only. |
| BF-6.10 | Virtual cards, BNPL, card benefits | Missing | Confirmed missing | Tried "virtual card", "card number", "payment method", "instalment": none. Shell 6.9: "Payment autofill is Stage 2" — sub-feature not named. |
| BF-6.11 | Wallet payment sheet | Missing | Actually partial | Named only. DV 6.14: "Payments stay in Payment Request"; DM 6.5 lists "payment" as activation-gated. No payment sheet or wallet design. |
| BF-6.12 | Shopping: price tracking, coupons | Missing | Confirmed missing | Tried "price drop/track/alert", "coupon", "cashback", "deal": hits unrelated (S1 D-S8 "search deals"). |
| BF-6.13 | Store reviews | Missing | Confirmed missing | Tried "review summary", "product review", "store rating": only extension/app-directory review. |
| BF-6.14 | Crypto wallet | Missing | Confirmed missing | Tried "web3", "ethereum", "EIP-1193", "blockchain", "NFT": 0 hits. |
| BF-6.15 | Creator rewards | Missing | Confirmed missing | Tried "BAT", "creator", "tipping", "micropayment", "private ads": hits are JS "creator inheritance" only. |
| BF-7.2 | Lock private tabs with biometrics | Missing | Confirmed missing | Tried "biometric", "Windows Hello", "lock private", "re-authenticate". Shell 6.12 OS auth gates password reveal only, not private tabs. |
| BF-8.8 | Safety check | Missing | Confirmed missing | Tried "security/privacy check", "checkup", "health check": none. `about:diagnostics` (shell 6.14) is not a checkup. |
| BF-8.9 | Unwanted software cleanup | Missing | Confirmed missing | Tried "unwanted software", "potentially unwanted", "settings hijack", "cleaner": none. Recovery window "reset settings" is crash recovery. |
| BF-10.5 | Read aloud | Missing | Actually partial | PD 6.9: "Find, selection highlights, copy …, read-aloud and the semantic subtree all read this model"; PD-15 "Read aloud" target. Only PDF, via the OS screen reader; no speech engine or read-aloud UI for pages. |
| BF-10.9 | Share page | Missing | Confirmed missing | Tried "share button/menu", "copy link", "navigator.share": only ED 6.12 "copy link to highlight". No share-page feature. |
| BF-10.11 | QR code for page | Missing | Confirmed missing | Tried "QR", "barcode", "BarcodeDetector": PH 6.4.10 checks "QR-decoded URL"; nothing generates a code. |
| BF-10.12 | Screenshot tool | Missing | Actually partial | GR 6.23 O4: "Canonical screenshots from the CPU path for DevTools, WebDriver BiDi and … agents". Capture exists for tools; no user capture, region select or annotate tool. |
| BF-10.13 | Eyedropper | Missing | Confirmed missing | Tried "EyeDropper", "colour picker", "input type=color": LY 6.3 colour pickers for form controls only. |
| BF-10.14 | Notes in sidebar | Missing | Confirmed missing | Tried "notes panel/sidebar", "side panel", "annotat": DT/PD hits only. EX 9.1 sidebar waits for "the shell panel design". |
| BF-10.15 | Highlights / page key facts | Missing | Confirmed missing | Tried "key facts", "page summary", "tl;dr": none. ML 6.5 lists "summarise" as a Stage 2 task, no address-bar surface (B10 bars AI chips when off). |
| BF-10.16 | Request desktop site (mobile) | Missing | Confirmed missing | Tried "desktop site/mode", "mobile UA", "UA override": 0 relevant. UA is fixed (S1 13.2, NW-D12); no mobile shell. |
| BF-10.17 | Dark mode for web content | Missing | Confirmed missing | Tried "auto-dark", "force dark", "invert", "color-scheme": UI 4.9/4.10 dark theme is for the shell; AX forced colours are OS-driven (R0); PD dark mode is PDF-only. |
| BF-11.2 | Automatic PiP on tab switch | Missing | Confirmed missing | Tried "autopictureinpicture", "PiP window", "Document PiP": MD 6.12 PiP needs activation; no automatic entry on tab switch. |
| BF-11.4 | Casting | Missing | Confirmed missing | Tried "AirPlay", "Chromecast", "DIAL", "PresentationRequest", "RemotePlayback", "second screen", "Miracast": 0 relevant. |
| BF-11.6 | Media playlist / offline play | Missing | Confirmed missing | Tried "watch later", "offline playback/video", "download the video": hits are HLS playlists. S1 §6 "Save for offline" is pages, not media. |
| BF-11.7 | Built-in music player | Missing | Confirmed missing | Tried "Spotify", "music", "audio/media player": MD 6.16 OS players are energy references only. |
| BF-11.8 | Volume booster / video enhancement | Missing | Confirmed missing | Tried "loudness", "normali[sz]e", "equalizer", "upscal", "super-resolution": 0 relevant. |
| BF-11.10 | Microphone volume adjustment | Missing | Actually partial | MD 6.8 (MD-D16): `hg-voice` is "Izcal's own acoustic echo canceller, noise suppressor and gain control". Automatic gain only; no user input-level control. |
| BF-12.7 | Startup boost / background running | Missing | Confirmed missing | Tried "background mode/process", "launch at login", "prelaunch", "login item": AH 6.4 `app-autostart` (apps only); S1 spare renderer prelaunches inside a running browser. |
| BF-13.6 | Extensions on mobile | Missing | Confirmed missing | Tried "Android … extension", "mobile extension": S1 §1 Android is Stage 2 engine work; EX has no mobile scope. |
| BF-13.7 | AI-generated extensions | Missing | Confirmed missing | Tried "generate an extension", "userscript", "Boost": EX 6.8 user scripts are developer-mode, user-written; no AI generation. |
| BF-13.9 | AI theme generator | Missing | Confirmed missing | Tried "theme generat", "custom theme": EX 6.9 refuses extension themes; shell theming is "CSS-only user theming (D-S2)"; no AI. |
| BF-13.11 | Toolbar customisation | Missing | Confirmed missing | Tried "toolbar layout/order/customi", "pin … toolbar", "overflow menu": only UI 4.9 toolbar keyboard model. |
| BF-13.15 | Command chains / macros | Missing | Confirmed missing | Tried "macro" (Rust macros), "record a sequence", "custom shortcut", "rebind": 0 relevant. |
| BF-13.16 | Peripheral lighting integration | Missing | Confirmed missing | Tried "Razer", "Chroma", "keyboard lighting", "peripheral": hits are RGB colour spaces. |
| BF-13.19 | Built-in game | Missing | Confirmed missing | Tried "offline game", "dinosaur", "game" (non-gamepad): metaphors only. |
| BF-13.20 | Touch Bar customisation | Missing | Confirmed missing | Tried "Touch Bar", "TouchBar", "function row": 0 hits. |
| BF-15.2 | Choice of AI provider | Missing | Confirmed missing | Tried "cloud model/inference", "third-party model", "ChatGPT", "Gemini", "OpenAI": history only. ML-D5 service is network-less and local; no provider choice. |
| BF-15.4 | Ask about multiple tabs | Missing | Confirmed missing | Tried "across tabs", "several tabs", "tabs as context": 0 hits. |
| BF-15.5 | AI tab organiser | Missing | Confirmed missing | Tried "suggest/auto group", "organise tabs": 0 relevant. Groups are Stage 2, "never forced" (SH-D27). |
| BF-15.7 | AI history search | Missing | Confirmed missing | Tried "semantic search/index", "embedding", "vector search", "natural-language query": none. Shell 6.7 ranking is lexical. |
| BF-15.9 | AI memory / personalisation | Missing | Actually partial | SEC 6.12 AGT-R2: "Agent memory is per task by default. Anything kept across tasks needs confirmation". Rule only; no memory feature or view/delete UI. |
| BF-15.10 | AI access to personal data sources | Missing | Actually partial | SEC 6.12 AGT-R6 names agent tasks on "a read-write origin in a sensitive class (mail, banking …)". Security rule only; no feature reading mail, calendar or files. |
| BF-15.11 | Image generation and editing | Missing | Confirmed missing | Tried "image generation/edit", "text-to-image", "diffusion": one review citation only. |
| BF-15.12 | Voice / live AI conversation | Missing | Confirmed missing | Tried "voice assistant/mode", "talk to", "share the screen with": 0 relevant. |
| BF-15.13 | Virtual try-on | Missing | Confirmed missing | Tried "try-on", "garment", "apparel": 0 relevant. |
| BF-15.16 | AI content creation from browsing | Missing | Confirmed missing | Tried "generate a report/deck", "slide", "synthesis across": 0 relevant. |
| BF-15.18 | Link previews with AI summary | Missing | Confirmed missing | Tried "preview link", "link card", "summary" + hover: 0 hits. Summarise is a Stage 2 ML task (ML 6.5) with no link-preview surface. |
| BF-16.2 | Web panels / sidebar apps | Missing | Confirmed missing | Tried "web panel", "sidebar … site", "pinned site": only the per-site panel (UI 4.3), which is not a web panel. |
| BF-16.3 | Integrated messengers | Missing | Confirmed missing | Tried "WhatsApp", "Telegram", "Slack", "Discord", "chat app": history/slack-as-margin only. |
| BF-16.4 | Mail client | Missing | Confirmed missing | Tried "mail client", "webmail", "IMAP", "JMAP": CB 6.14 mentions webmail as a long-lived tab only. |
| BF-16.5 | Calendar | Missing | Confirmed missing | Tried "calendar app/integration", "CalDAV", "iCalendar", ".ics": 0 hits. |
| BF-16.9 | Video calling | Missing | Confirmed missing | Tried "video call", "conferencing", "meeting": MD supports WebRTC calls for pages; no built-in calling product. |
| BF-16.10 | Productivity service shortcuts | Missing | Confirmed missing | Tried "Google Drive", "OneDrive", "Microsoft 365", "productivity": citations only. |
| BF-16.11 | Live work destinations, meeting prep | Missing | Confirmed missing | Tried "meeting", "work items", "destinations": 0 relevant. |
| BF-16.12 | Remote desktop access | Missing | Confirmed missing | Tried "remote desktop", "RDP" (Firefox protocol), "VNC", "remote access": 0 relevant. |
| BF-16.13 | Location and directions cards | Missing | Confirmed missing | Tried "directions", "map card", "geo:", "address detection": 0 relevant. |
| BF-17.3 | Site shortcuts on desktop | Missing | Confirmed missing | Tried "shortcut", ".url/.desktop file", "webloc", "add to dock": AH 6.12 desktop shortcut covers `app://` packages only. |
| BF-17.6 | OS share sheet integration | Missing | Confirmed missing | Tried "share sheet", "DataTransferManager", "NSSharing", "share picker", "Web Share": 0 hits (portal hits are screen capture). |
| BF-17.7 | Handoff / continuity | Missing | Actually excluded | Shell 9.1 "Stage 3 or never": "Sync and multi-device" — "the product says there is no account and no sync". Handoff is a multi-device feature. |
| BF-17.8 | Mobile widgets / home-screen shortcuts | Missing | Confirmed missing | Tried "widget", "home screen", "quick action", "jump list": TX 6.1 jump-list string rule (desktop); no mobile widgets. |
| BF-17.9 | In-car browser | Missing | Confirmed missing | Tried "automotive", "in-car", "vehicle", "CarPlay": 0 relevant. |
| BF-17.10 | Password autofill in other apps (mobile) | Missing | Confirmed missing | Tried "autofill service/framework", "credential provider", "ASCredentialProvider": none (passwords in OS keychain, S1 13.2, but no mobile fill). |
| BF-17.12 | Calendar event add (mobile) | Missing | Confirmed missing | Tried "add to calendar", "text/calendar", "vcalendar": 0 hits. |
| BF-18.3 | Responsive design / device mode | Missing | Confirmed missing | Tried "emulation", "viewport override", "device toolbar", "responsive": the DevTools spec contains neither "emulat" nor "viewport". |
| BF-19.7 | In-browser help | Missing | Actually partial | Shell 6.8 Why? surface answers "a bounded set of questions about the current state"; DV 6.18: "Each error has a short help page". No general help content or help search. |
| BF-20.2 | Kiosk mode | Missing | Confirmed missing | Tried "kiosk", "single-app", "assigned access", "locked-down": incidental only (DV 6.10, WC 6.9, AH §5). |
| BF-20.3 | Parental controls | Missing | Confirmed missing | Tried "Family Safety", "Screen Time", "Family Link", "supervised", "SafeSearch": 0 relevant. |
| BF-20.4 | Kids mode | Missing | Confirmed missing | Tried "kids", "children" (DOM children), "child-safe": 0 relevant. |
| BF-20.5 | Data saver / lite mode | Missing | Confirmed missing | Tried "Save-Data", "metered", "low data", "slow network", "reduce data": PH 6.7 "metered" is hosting cost. |
| BF-21.1 | Tab grid / switcher (mobile) | Missing | Confirmed missing | Tried "tab grid/switcher/overview", "tab tray": 0 hits. No mobile shell. |
| BF-21.2 | Swipe gestures (mobile) | Missing | Confirmed missing | Tried "swipe", "edge/back gesture", "two-finger": 0 hits. |
| BF-21.3 | Pull to refresh | Missing | Confirmed missing | Tried "pull-to-refresh", "overscroll": only GR §7 lists the Overscroll Behavior spec. |
| BF-21.5 | Mobile privacy hub | Missing | Confirmed missing | Tried "privacy dashboard/report", "trackers blocked", "weekly report": EX §4.1 mentions privacy dashboards as extension tasks. |
| BF-21.6 | Camera search / QR scan | Missing | Confirmed missing | Tried "camera search/scan", "scan QR", "BarcodeDetector": 0 hits. |
| PO-1.2 | [SB] Windows enterprise MSI | Missing | Actually excluded | Stage 1 only. SEC-D14 decided "(a) for Stage 1 … per-user installs only"; reversal trigger: "Enterprise deployment (Stage 2) needs per-machine installs". No MSI design for Stage 2. |
| PO-1.3 | MSIX / Microsoft Store package | Missing | Confirmed missing | Tried "AppX", "winget", "Microsoft Store": AH 6.1 rejects MSIX only as an app-packaging alternative. |
| PO-1.5 | Mac App Store distribution | Missing | Confirmed missing | Tried "App Store", "App Sandbox": ML 6.2 App Sandbox note (per-process profile) only; no distribution decision. |
| PO-1.11 | Initial preferences / partner distributions | Missing | Confirmed missing | Tried "initial settings", "bundled defaults", "preinstall", "OEM": UI 4.6 cites a "preinstalled browser" study only. |
| PO-1.12 | Install-time options | Missing | Confirmed missing | Tried "install options/flow/wizard", "install location", "dock icon": SEC 6.3/6.7 install directory privileges only. Default-browser ask is at onboarding (shell 6.11). |
| PO-1.13 | [SB] Uninstall, repair and leftover cleanup | Missing | Confirmed missing | Searched "uninstall", "leftover", "delete profile" outside AH: only `shell.apps` app uninstall and EX/SG extension data. No browser uninstaller. |
| PO-1.16 | VDI, roaming profiles and multi-session | Missing | Confirmed missing | Tried "network profile", "terminal services", "multi-session": 0 relevant. |
| PO-1.17 | Kiosk mode | Missing | Confirmed missing | Tried "app mode", "single-site", "assigned access": same incidental hits as BF-20.2. |
| PO-2.2 | [SB] SmartScreen app reputation | Missing | Confirmed missing | Tried "Authenticode", "EV signing", "reputation … signer/installer": SEC 6.16 code signing exists; no SmartScreen reputation plan. |
| PO-3.3 | Delta (differential) updates | Missing | Confirmed missing | Tried "bsdiff", "Courgette", "binary patch", "incremental update": PH 6.7 diffs are for the phishing bundle only. |
| PO-3.5 | Release cadence | Missing | Confirmed missing | Tried "release cycle/train/schedule", "major release", "version scheme": "release cycle" used only as a time unit (SEC 8, AX 6.18). |
| PO-3.6 | Extended support channel | Missing | Confirmed missing | Tried "long-term support", "LTS", "extended release": 0 relevant. S1 7.8 channels are Nightly, Beta, Release. |
| PO-3.7 | Staged (percentage) rollout | Missing | Actually partial | Named only. S1 §6 Updates row: "Signed, staged updates"; S1 §1 updater "staged updates". No percentage, cohort or halt design. |
| PO-3.8 | Version pinning and rollback | Missing | Confirmed missing | Tried "downgrade", "target version", "pinned release": S1 7.8 "rollback" drills are the TUF attack; AH pins app versions only. |
| PO-5.4 | OS crash and hang reporting integration | Missing | Confirmed missing | Tried "Error Reporting", "spindump", "watchdog", "MetricKit", "hang detect": S1 crash-handler process (D-S4) only; no OS reporter integration. |
| PO-5.9 | Field trials and server-side experiments | Missing | Confirmed missing | Tried "feature flag", "Nimbus", "variations", "remote config", "cohort": none. UPD-R5 (SEC 6.16) is a closed security-switch list, not experiments. |
| PO-6.1 | [SB] Windows Group Policy (ADMX/ADML) | Missing | Confirmed missing | Tried "HKLM", "registry policy", "policy delivery/source": none. Consumers exist (shell 6.9 "Set by your organisation", `about:policies`, NW 6.14, `hg-policy` crate) but no delivery path. |
| PO-6.2 | [SB] MDM on Windows | Missing | Confirmed missing | Tried "managed device", "device management", "OMA": 0 relevant. Same `hg-policy` gap as PO-6.1. |
| PO-6.3 | [SB] macOS configuration profiles | Missing | Confirmed missing | Tried ".mobileconfig", "managed preference", "defaults domain": 0 relevant. |
| PO-6.4 | Apple declarative device management for Safari | Missing | Confirmed missing | Tried "declarative", "DDM": 0 relevant (Safari-only item). |
| PO-6.5 | [SB] Linux policy files | Missing | Confirmed missing | Tried "policies.json", "/etc/", "JSON policy": TX 6.4 /etc/fonts and one SEC attacker row only. |
| PO-6.6 | Cross-platform policy file | Missing | Confirmed missing | Tried "policy file", "policy schema", "signed policy": DV 6.10 and AH 6.7 cite a "signed policy" with no format or location. |
| PO-6.7 | Cloud browser management | Missing | Confirmed missing | Tried "enrolment", "managed browser", "Chrome Enterprise": precedent citations only (PH §3, shell 6.11). |
| PO-6.10 | Managed bookmarks | Missing | Confirmed missing | Tried "bookmarks … policy/admin", "pushed bookmarks": 0 hits. |
| PO-6.14 | Enterprise reporting | Missing | Confirmed missing | Tried "reporting connector", "fleet", "inventory report": "inventory" hits are ML/surface inventories. |
| PO-6.15 | DLP / content-analysis connectors | Missing | Confirmed missing | Tried "DLP", "content analysis", "enterprise connector", "paste block": 0 relevant. |
| PO-6.17 | Legacy browser support / IE mode | Missing | Confirmed missing | Tried "enterprise mode", "browser switch", "site list": S1 13.1a cites IE mode only as precedent for protected media. |
| PO-6.19 | Enterprise admin documentation set | Missing | Confirmed missing | Tried "admin guide", "policy reference/list", "administrator": per-spec policy keys exist (DV 6.10); no admin documentation set. |
| PO-7.3 | OS single sign-on to cloud identity | Missing | Confirmed missing | Tried "SSO", "Entra", "Primary Refresh", "Kerberos": NW-D20 covers Negotiate/NTLM/Kerberos allowlists (PO-7.1/7.2), not OS cloud-identity SSO. |
| PO-7.9 | TLS version and cipher policies | Missing | Actually partial | HGBA-2 4.7: "hybrid by default, policy-controlled fallback" for post-quantum TLS. NW 6.5 fixes suites by design; no min-version, cipher or FIPS policy (absent from NW 6.14 list). |
| PO-7.12 | Intranet and single-word navigation | Missing | Confirmed missing | Tried "dotless", "bare host", "search or URL": NW 6.7 excludes single-label hosts from HTTPS-first; NW-D20 rejects intranet heuristics for auth. Address-bar resolution unspecified. |
| PO-7.16 | OS VPN and per-app VPN compatibility | Missing | Actually partial | NW 6.6 (NW-D8): OS split-DNS rules read "so enterprise and VPN names resolve as the OS would"; reversal on "VPN breakage". Per-app and Always On VPN routing unstated. |
| PO-8.5 | Enterprise default-browser assignment | Missing | Confirmed missing | Tried "DefaultAssociations", "file association", "default browser … policy": shell 6.11 user ask only; SEC 6.3 lists default-browser APIs as attack surface. |
| PO-8.11 | Background mode and start at login | Missing | Confirmed missing | Tried "after the last window", "system tray", "menu bar", "launch at login": AH 6.4 `app-autostart` for apps only. |
| PO-8.12 | OS scripting and automation hooks | Missing | Confirmed missing | Tried "COM automation", "D-Bus interface", "command-line argument", "--new-window": SEC AGT-R8 restricts automation attach; no scripting hooks. |
| PO-8.13 | Set as desktop background | Missing | Confirmed missing | Tried "wallpaper", "set as background": EX 6.1 Ring C refuses the `wallpaper` extension API only; no browser action. |
| PO-8.14 | Continuity and cross-device handoff | Missing | Actually excluded | Shell 9.1 "Stage 3 or never": "Sync and multi-device"; S1 §9 Stage 3: "Sync (if wanted)". |
| PO-9.9 | OS share sheet | Missing | Confirmed missing | Tried "share picker", "xdg-desktop-portal", "NSSharing": portal hits are ScreenCast/Camera (MD 6.1). |
| PO-9.22 | OS speech synthesis and recognition | Missing | Actually excluded | Deferred, named only. EX 9.1: "TTS … Stage 3 / Each needs a host surface (… OS voices …)". No owner for OS speech or `speechSynthesis`. |
| PO-9.27 | Parental controls integration | Missing | Confirmed missing | Tried "Family Safety", "Screen Time", "Family Link", "content restriction": 0 hits. |
| PO-12.1 | Web app manifest processing | Missing | Confirmed missing | Tried "installable", "appmanifest", "progressive web": AH history rows and AH's own manifest only. |
| PO-12.3 | PWA uninstall through the OS | Missing | Confirmed missing | Uninstall hits are AH/`shell.apps` (`app://`) only; PH 6.4.9 assumes installed web apps but no lifecycle. |
| PO-12.4 | OS-level app integration | Missing | Confirmed missing | Tried "setAppBadge", "registerProtocolHandler", "protocol_handlers": EX 6.1 stages extension handler keys to Stage 3 only. |
| PO-12.5 | Enterprise force-installed web apps | Missing | Confirmed missing | Tried "force-install", "allowlists apps": EX 6.5 (extensions) and AH 6.7 (`.izapp`) only. |
| PO-12.6 | Store packaging of PWAs | Missing | Confirmed missing | MSIX/Store hits are AH comparisons only. |
| PO-12.7 | Site-specific shortcuts / taskbar tabs | Missing | Confirmed missing | Tried "site-specific browser", "own window", "taskbar tab": 0 relevant. |
| PO-13.8 | Testing builds (pinned, non-updating) | Missing | Confirmed missing | Tried "non-updating", "frozen build", "test channel", "headless build": 0 hits. |
| PO-13.10 | Single instance and remote control of a running browser | Missing | Confirmed missing | Tried "single-instance", "second instance", "already running", "remote control": SEC-20/DT 6.5 only refuse automation launches on a held profile. |
| PO-13.11 | Developer tools enablement policy | Missing | Confirmed missing | Tried "DevTools … policy/disable/admin": 0 relevant. DT 6.5 has no admin switch. |
| PO-14.5 | Regional builds and defaults | Missing | Confirmed missing | Tried "regional build/default", "Yandex", "Baidu", "per-country": "region" hits are memory regions. |
| PO-15.5 | DMA browser choice screens | Missing | Confirmed missing | Tried "Digital Markets", "gatekeeper", "browser choice": S1 13.1a DRM complaint and 13.2 iOS row only. |
| PO-15.6 | DMA data portability import | Missing | Confirmed missing | Tried "Data Portability", "takeout", "Article 6(9)": shell 9.1 export "is the portability answer"; import from portability APIs absent. |
| PO-15.7 | Age assurance and age signals | Missing | Confirmed missing | Tried "age verification/signal", "Declared Age", "Online Safety Act", "minors": ML data-ledger "minor" flags only. |
| PO-15.9 | [SB] EU Cyber Resilience Act | Missing | Confirmed missing | Tried "ENISA", "support period", "conformity", "CE mark", "end of life": 0 relevant (pieces: SBOM S1 7.8, disclosure S1 13.2). |
| PO-15.10 | European Accessibility Act exposure | Missing | Confirmed missing | Tried "EN 301 549", "Section 508", "VPAT", "accessibility act": 0 relevant. |
| PO-16.3 | CVE assignment (CNA) | Missing | Confirmed missing | Tried "GHSA", "CVE Numbering", "assign … CVE": S1 13.2 promises "security advisories" with no CVE/CNA plan. |
| PO-16.10 | Security notes for enterprise | Missing | Confirmed missing | Tried "release notes" (citation sense), "known issues", "admin notes": B10 says ML inventory changes go in release notes; no enterprise security notes. |
| PO-17.6 | Network geolocation provider | Missing | Confirmed missing | Tried "location provider/service", "GeoClue", "CoreLocation", "IP geolocation": geolocation is a permission only (S1 13.2, shell 6.4). |
| PO-17.9 | Support site and community | Missing | Confirmed missing | Tried "help centre", "knowledge base", "community forum", "user support": DV 6.18 device-error help pages only. |
