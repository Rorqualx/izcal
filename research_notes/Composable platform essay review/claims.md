# Factual claims in "The Browser as a Composable Platform" (pasted by the user, 14 October 2026)

Numbered for the fact-checkers. Quotes are the essay's wording.

## Group A: containers, torrent, WebGPU, smart home, ad blocking, JavaScript
A1. "in July 2026 [containers] became native in Firefox 153." Mozilla quotes: Containers are "a lightweight solution for separating identities within a single browser window," Profiles deliver "complete data separation".
A2. Brave shipped WebTorrent natively and "removed it in Brave 1.81 (August 2025)".
A3. P2P Media Loader is an open-source WebRTC P2P video library "adopted by PeerTube and niche platforms". WebTorrent supports Chrome, Firefox, Opera and Safari with no plugins.
A4. "WebGPU is stable in all major browsers as of 2026 — Chrome 113+, Edge 113+, Firefox 125+, Safari 26+."
A5. "Chrome 144 optimized writeBuffer() and writeTexture() for up to 2× gains; Chrome 145 added subgroup uniformity and experimental synchronous buffer mapping in workers."
A6. ONNX Runtime Web supports WebGPU and WebNN execution providers; "WebNN for hardware-accelerated NPU access" is usable in browsers.
A7. "Opera GX shipped a Smart Home feature in September 2025: an open-source MQTT-based integration" with Home Assistant, Homebridge, Node-RED; examples dim lights when a movie plays, change browser color to match room lighting; "The open-source Opera Smart Home extension is a reference implementation".
A8. Home Assistant has a local LLM integration (via Ollama or similar) and REST and WebSocket APIs.
A9. uBlock Origin "was disabled from Chrome 138 and removed from the Web Store in August 2026". uBlock Origin Lite "lacks cosmetic filtering and scriptlet injection by default". "Edge followed with its own MV2 phase-out."
A10. "Firefox quietly shipped Brave's adblock-rust engine in Firefox 149 (March 2026) without announcing it in the release notes." "Opera One rebuilt its native ad blocker with uBlock list compatibility in May 2026." "Waterfox adopted the same engine."
A11. "0.3% of ads use 27% of ad data and 28% of ad CPU" (Chrome heavy ad intervention).
A12. "Every major browser has a JavaScript toggle in developer tools. None has a per-tab JavaScript switch as a first-class user control." Extensions NoJSLite (Firefox), JS Toggle (Chrome), toggle-javascript are "among the most popular in every store". Also check Chrome's per-site JavaScript setting in Site settings / page info, Firefox `javascript.enabled`, Safari, Brave Shields per-site script blocking.
A13. "A page with no JavaScript cannot run a crypto-miner, cannot exfiltrate form data, cannot fingerprint via canvas, and cannot execute a prompt-injection payload." (Check: CSS-based exfiltration and fingerprinting, form submission, tracking pixels, prompt injection being text.)
A14. Firefox Enhanced Tracking Protection "is a global setting, not a per-tab switch" (check per-site ETP toggle).

## Group B: customization, protocols, Tor, VPN, agents
B1. Glide is a Firefox fork with Vim-style modes (Normal, Insert, Hint, Ignore), macros that run external processes, shortcuts that clone a GitHub repo.
B2. Zen Browser "added 'live folders' that turn the sidebar into an RSS reader and a GitHub dashboard."
B3. "Adaptive-browser is a Tauri v2 + React project that constructs its UI from a YAML manifest and API specs, with organization-level guardrails".
B4. "Waterfox 6.7.5 (October 2026) added a native feed manager and built-in reader, with Live Bookmarks integration, OPML import/export...". RSS Ninja runs in the Firefox sidebar.
B5. Browsers have no raw TCP socket API; "WebSocket is HTTP-upgrade only, and WebTransport is HTTP/3-only." (Check Chrome's Direct Sockets API for Isolated Web Apps.) Every browser-based IRC client is a gateway.
B6. Brave Private Windows with Tor: bridges (obfs4, Snowflake), .onion resolution; Brave "explicitly does not implement most of the Tor Browser's anti-fingerprinting protections". Arti can be launched independently and connected over SOCKS.
B7. "Firefox 149 (March 2026) shipped a free built-in VPN with a 50 GB monthly data cap, available in the US, France, Germany, and the UK. Firefox 151 added country selection with five destinations."
B8. "Opera's free VPN automatically activates when the browser detects public or unsecured Wi-Fi."
B9. Enterprise browsers (Edge for Business, Chrome Enterprise) "attempt" per-profile routing with policy; "the user-facing version does not exist".
B10. OmniScout (CLI, config.toml, browser profile management), Stagehand (launches local Chrome), Nanobrowser (Chrome extension, Navigator/Planner/Validator models, configurable API keys), OctoFlow (local Ollama backends).
B11. Tor Project guidance on using Tor in a non-Tor-Browser browser, or mixing Tor and non-Tor browsing in one browser (anonymity-set and fingerprinting implications); do banks commonly block Tor exit nodes?
B12. Chrome's HTTP cache partitioning (2020) and the reason (cross-site cache probing / timing leaks); any history of shared cross-origin content-addressed caches being removed for privacy.

## Group C: the second pasted section (performance, security, privacy, UI, web platform, graphics, offline)
C1. bfcache: AdSense pushing publishers to adopt it; Yahoo! JAPAN News reported +9% mobile revenue and +13% page views per session. Blockers: unload handlers, Cache-Control: no-store, open connections; "the fix is mechanical".
C2. Speculation Rules: "Chrome 144 added prerender until script, an origin trial". WordPress Performance Team "Speculative Loading" plugin (and whether it is now in WordPress core).
C3. Soft navigation metrics in CrUX as experimental data; "Chrome 151 shipped the Soft Navigations web platform API".
C4. Firefox performance roadmap: QUIC and IPv6 prioritization, a resource monitor, a mobile power-saving mode for resource-intensive tabs.
C5. Chrome Origin Isolation (origin-keyed processes) on by default only on devices with ≥4 GB RAM; `OriginKeyedProcessesEnabled` policy.
C6. "A 2026 IEEE paper on process isolation found that attackers can still exfiltrate sensitive data across all major browsers despite site isolation".
C7. "Chrome 142 enables PQC with DTLS in WebRTC"; enterprise opt-out policy planned for removal by Chrome 152.
C8. "Microsoft Edge 151's security baseline adds six new policies, including renderer AppContainer and network service sandbox enforcement."
C9. "Firefox 151 tightened fingerprinting defenses in the default Standard ETP mode", "reduces the number of uniquely identifiable users by about 14%"; "Blocked Tracker Widget".
C10. "Brave 1.93 masks detailed GPU information exposed through WebGL and WebGPU by default on desktop and Android"; Brave Email Aliases; Shred button on Android; overhauled Rust adblock engine "cut memory consumption by 75%".
C11. "Safari's anti-fingerprinting is on by default across all modes".
C12. Firefox roadmap: free built-in VPN for mobile, native containers, ad/tracker blocking on iOS, blocked-tracker dashboard, breach alerts.
C13. Chrome vertical tabs and split view in 2026 via tab context menu "Show Tabs Vertically". Firefox tab groups on Android, iOS later in 2026. Chrome Canary translating PDFs through Reading Mode.
C14. Baseline March 2026 additions: readable byte streams, WebTransport, Reporting API, Iterator.concat(), math font family, text-indent each-line and hanging.
C15. Interop 2026 focus areas list: attr(), contrast-color(), container style queries, custom highlights, dialogs and popovers, Fetch streaming request bodies, IndexedDB getAllRecords(), JSPI for Wasm, media pseudo-classes, Navigation API, scoped custom element registry, scroll-driven animations, scroll snap, shape(), View Transitions, Web compat, WebRTC, WebTransport.
C16. View Transitions "reached Baseline Newly Available in January 2026"; "Chrome 147 adds element-scoped view transitions" via Element.startViewTransition(); Simon Willison "most excited about" cross-document View Transitions in 2026.
C17. Trusted Types API is now Baseline (which browsers/versions, when).
C18. WebGPU: Chromium Linux Intel Gen12+ (January 2026); Compatibility Mode on older Android GLES 3.1 (February 2026); Chrome 144 subgroup_id WGSL extension; Chrome 149–150 TRANSIENT_ATTACHMENT texture usage.
C19. "WebAssembly 3.0 is the next milestone. It includes Garbage Collection and Memory64". "WASI 1.0 release is expected in late 2026 or early 2027". WASI P3 async in the Component Model. "JavaScript String Builtins arrived in Wasm in version 26.2". JSPI.
C20. Offpunk 3.0 (terminal offline-first browser; Web, Gemini, Gopher, RSS; "unmerdify"). Kiwix packaging (PWA, Electron, NWJS, UWP).
C21. "no mainstream browser serves [offline-first browsing]": check Safari Reading List offline saving, Chrome offline pages / reading list on Android, Firefox/Pocket status (Pocket shutdown 2025), Edge, and Web Bundles/MHTML save.
