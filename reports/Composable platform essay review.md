# Review: "The Browser as a Composable Platform"

*Critique of a pasted design essay in two parts: "composable platform" features, and a 2026 "minimum viable platform" survey. 14 October 2026. Three fact-checkers verified 47 claims against primary sources; the design critique is mine. Notes are in `research_notes/Composable platform essay review/`.*

## Verdict

The essay is a well-researched survey with a weak design thesis. Its factual base is better than earlier pasted material. Of 47 checked claims, **17 hold up as written, 24 are partly correct** (mostly wrong dates or versions, or overreach), and **6 are wrong**. One cited tool, OctoFlow, appears to be invented. Its design frame, "a kernel of capabilities and a user responsible for the composition", is wrong for HGBA's users, and its two most novel proposals are unsafe: BitTorrent as a shared cache, and publishing browser state over MQTT. What it contributes is a set of concrete gaps in HGBA's own plans:

- containers as policy contexts;
- graded per-site JavaScript;
- a PDF viewer;
- bfcache as engine work;
- post-quantum DTLS;
- GPU fingerprint masking;
- a user-invoked offline mode built on the deterministic-replay archive.

---

## 1. Fact-check summary

### Part 1: composable features (26 claims)

| Claim | Verdict | Correction |
|---|---|---|
| Containers native in Firefox 153 (July 2026), with Mozilla quotes | **Partly correct** | Firefox 153 (21 July 2026) added built-in Containers as a preview. The "Mozilla quotes" are a news site's paraphrase, found on no Mozilla page |
| Brave removed WebTorrent in 1.81 (August 2025) | **Partly correct** | Disabled in 1.81 (6 August 2025); code removed in 1.82 (4 September 2025) |
| P2P Media Loader adopted by PeerTube | **Partly correct** | PeerTube uses its own fork for HLS and dropped WebTorrent playback in v6 (2023) |
| WebGPU stable everywhere: Chrome 113+, Firefox 125+, Safari 26+ | **Wrong** | Firefox shipped in 141 (Windows) and 145 (Apple Silicon, macOS 26+). Firefox still has no WebGPU on Linux or Android, and Chrome on Linux only from 144 |
| Chrome 144 `writeBuffer` up to 2×; Chrome 145 subgroup uniformity, sync mapping in workers | **Correct** | Synchronous mapping in workers is behind a flag |
| WebNN usable for NPU access via ONNX Runtime Web | **Partly correct** | The execution provider exists, but WebNN needs a flag and Chrome's trial was delayed twice |
| Opera GX Smart Home (September 2025), MQTT, open source | **Partly correct** | A preview (11 September 2025). The example is backwards: the room lights follow the browser theme |
| Home Assistant with Ollama and REST/WebSocket APIs | **Correct** | — |
| uBlock Origin disabled in Chrome 138, removed August 2026; uBO Lite lacks cosmetic filtering and scriptlets | **Partly correct** | Removal on 31 August 2026 confirmed. uBO Lite's default "Optimal" mode **includes** cosmetic filtering and scriptlets; only generic cosmetic filters are off. Edge's MV2 phase-out began in August 2026 and is not complete |
| Firefox 149 "quietly shipped Brave's adblock-rust engine"; Opera and Waterfox adopted it | **Partly correct** | Firefox 149 contains the engine **disabled**, with no UI and no lists, so it blocks nothing. The Opera (14 May 2026) and Waterfox parts are correct |
| 0.3% of ads use 27% of ad data and 28% of ad CPU | **Correct, dated** | Chromium blog, 14 May 2020 |
| No browser has a per-tab JavaScript switch; JS-toggle extensions are "among the most popular" | **Wrong** | Chrome and Edge have per-site JavaScript blocking in Site settings, and Brave has per-site script blocking. Firefox and Safari only have global switches. NoJSLite has about 1 user |
| A page with no JavaScript cannot mine, exfiltrate form data, fingerprint via canvas or execute prompt injection | **Half right** | Mining and canvas fingerprinting need JavaScript. But CSS alone fingerprints browsers (NDSS 2025: 97.95% of 1,176 browser/OS combinations told apart), CSS selectors can leak attribute values, HTML forms post anywhere, and tracking pixels work. Prompt injection is text, so disabling JavaScript does nothing against it |
| Firefox ETP is global only | **Wrong** | A per-site switch adds the site to an exceptions list |
| Glide (Vim-style Firefox fork) | **Correct** | — |
| Zen live folders as an RSS reader and GitHub dashboard | **Mostly correct** | Zen 1.19b (27 February 2026); folders filled with GitHub items or RSS entries. "Dashboard" overstates it |
| Adaptive-browser built from YAML manifests | **Partly correct** | A small prototype; the manifest is JSON, and YAML holds preferences and organization guardrails |
| Waterfox 6.7.5 (October 2026) native feed reader | **Partly correct** | Shipped 30 September 2026; OPML import/export already existed |
| No raw TCP sockets in browsers; WebTransport is HTTP/3 only; every browser IRC client needs a gateway | **Partly correct, outdated** | Chrome's Direct Sockets has been on since Chrome 131, but only for Isolated Web Apps, allowlist-gated from Chrome 143; Mozilla is opposed. WebTransport also allows HTTP/2. The IRCv3 WebSocket spec lets servers accept browsers directly |
| Brave Tor windows, bridges, Arti over SOCKS | **Correct** | Brave's words: it does "NOT implement most of the privacy protections from Tor Browser" |
| Firefox 149 free VPN (50 GB, four countries); Firefox 151 country choice | **Correct** | The five destinations are the US, UK, France, Germany and Canada |
| Opera VPN auto-activates on public Wi-Fi | **Partly correct** | Opt-in, desktop only, announced 24 September 2026 |
| No user-facing per-identity routing exists | **Wrong** | Firefox containers plus Mozilla VPN have given each container its own exit since 2022, and Edge's proxy policy applies per profile |
| OmniScout, Stagehand, Nanobrowser, OctoFlow | **Mixed** | Nanobrowser dropped its Validator agent in 2025. **OctoFlow could not be found and looks fabricated** |
| Tor guidance; banks blocking Tor | **Confirmed** | The Tor Project "strongly recommend[s] against using Tor in any browser other than Tor Browser" and warns that mixing Tor and non-Tor browsing can link the two |
| HTTP cache partitioning | **Correct, with a 2026 counter-trend** | Chrome 86 (October 2020). In 2026 Chrome proposed a shared cache for a few hundred popular resources (Firefox negative), and a WICG cross-site, hash-keyed cache proposal is under discussion |

### Part 2: the 2026 survey (21 claims)

| Claim | Verdict | Correction |
|---|---|---|
| bfcache: Yahoo! JAPAN News +9% mobile revenue, +13% page views | **Partly correct** | 9% is right. The +13% comes from an AdSense email; the web.dev case study reports +2.26% page views |
| Chrome 144 prerender-until-script origin trial; WordPress Speculative Loading plugin | **Correct** | The plugin went into WordPress core in 6.8 (April 2025) |
| Chrome 151 Soft Navigations API; CrUX soft-navigation data | **Correct / unverifiable** | API shipped 28 July 2026. CrUX data could not be confirmed |
| Firefox roadmap: QUIC/IPv6, resource monitor, power saving | **Correct** | — |
| Origin isolation on by default only at ≥4 GB RAM | **Correct** | Chrome 142 |
| 2026 IEEE paper: exfiltration despite site isolation | **Correct** | Kilger et al., IEEE S&P 2026, exploiting subdomains that share a process |
| Chrome 142 PQC for WebRTC DTLS; opt-out removed by 152 | **Correct** | — |
| Edge 151 baseline: six new policies incl. renderer AppContainer and network sandbox | **Partly correct** | Six new *recommendations* for existing settings |
| Firefox 151: about 14% fewer uniquely identifiable users; Blocked Tracker Widget | **Partly correct** | 14% is right (about 49% on macOS); the widget shipped in 152 |
| Brave 1.93 GPU masking; Email Aliases; Shred; adblock memory −75% | **Partly correct** | Spread over 1.89–1.94; the 75% applies to the ad-block engine's memory |
| Safari anti-fingerprinting on by default in all modes | **Partly correct** | Safari 26 enables a basic layer everywhere; Advanced Fingerprinting Protection still defaults to Private Browsing only |
| Firefox roadmap list | **Partly correct** | Mixes plans with shipped items (widget in 152; iOS blocking about September 2026) |
| Chrome vertical tabs and split view in 2026; PDF translation in Reading Mode | **Partly correct** | Vertical tabs April 2026; split view late 2025 (142–143); PDF translation Canary-only behind a flag |
| Baseline March 2026 additions | **Correct** | — |
| Interop 2026 focus areas | **Partly correct** | Omits Anchor Positioning and CSS Zoom; the Fetch item is "Fetch Uploads and Ranges" |
| View Transitions Baseline in January 2026; Chrome 147 element-scoped; Simon Willison quote | **Wrong / unsourced** | Baseline on 14 October 2025 (Firefox 144). The Chrome 147 part is correct. No source found for the quote |
| Trusted Types Baseline | **Correct** | 24 February 2026, Firefox 148 |
| WebGPU on Linux Intel and compat mode on Android; `TRANSIENT_ATTACHMENT` in 149–150 | **Partly correct** | `TRANSIENT_ATTACHMENT` arrived in 146 |
| Wasm 3.0 "next milestone"; WASI 1.0 late 2026 or early 2027; String Builtins "in version 26.2" | **Wrong** | Wasm 3.0 was finalized 17 September 2025. WASI 0.3 with async shipped 11 June 2026, and no official date exists for 1.0. "26.2" is a Safari version; Chrome 130 and Firefox 134 shipped String Builtins first |
| Offpunk 3.0; Kiwix | **Correct** | — |
| No mainstream browser serves offline-first browsing | **Overstated** | Safari's Reading List saves offline, Chrome on Android downloads pages, desktop Chrome saves MHTML; Firefox lists Android offline mode on its roadmap. Pocket closed on 8 July 2025 |

---

## 2. Design critique

### 2.1 The frame is wrong for HGBA's users

"Closer to Linux than to a consumer appliance", with "a user who is responsible for the composition", builds for the niche Arc found too small to sustain a product. Arc's own letter said it was "too different, with too many new things to learn", and only 5.52% of daily users regularly used more than one Space. "Users keep installing extensions that provide these features" holds for ad blockers and almost nothing else on the list: Multi-Account Containers has about 411k users, and NoJSLite about 1. Each composed primitive also adds attack surface in a privileged process. Brave removed WebTorrent for exactly that reason, which the essay concedes and then sets aside. Composition also multiplies interactions to test, while the essay names no metric, maturity label or failure condition for any proposal.

**What survives:** a few **policy contexts** with good defaults, plus HGBA-2.0's Stage 2 component host (W-1) as the power-user escape hatch.

### 2.2 Adopt: containers as policy contexts

This is the essay's best idea, and HGBA can do it better than an add-on because the engine is still being designed.

- **Isolation key.** A container must be part of both the storage key and the process key, so the same site in two containers gets two processes and partitions, and shares no cache, connection pool, HSTS state or service worker. Otherwise a container is a cookie jar that cross-container timing attacks can see through. This multiplies processes, so S-3's memory budget needs a per-container measurement.
- **Policy per container:** storage partition, block lists, JavaScript mode, agent origin sets (B9) and a network route.
- **Ship a few named containers with sensible defaults.** A blank policy editor repeats Arc's mistake.
- **Seed fingerprint noise by (site, container, session)** (D-S1), so two containers on one site cannot be linked.
- **Correction to my draft:** per-container routing is not new to consumers. Firefox containers with Mozilla VPN have done it since 2022. HGBA's distinction is narrower: routes the user supplies (WireGuard, SOCKS, Arti) per container, without depending on a vendor VPN service.

### 2.3 Adopt: graded JavaScript, with honest claims

The three modes (off, first-party only with no `eval`, full), set per site and inherited from the container, are tractable in HGBA. Script fetches go through the network service, and V8 can refuse string-to-code generation per context. The **middle mode is new**: Chrome, Edge and Brave already offer per-site blocking, so "no browser ships it" is wrong. But no browser offers a first-party-only mode. Its breakage cost is unknown, because many sites serve their own scripts from third-party CDN origins. It needs a corpus measurement and is user-invoked under the P2 scope note (class b).

The security claim must be stated honestly. JavaScript off removes the largest class of engine exploits (the JIT and the bindings) and most active tracking. It does **not** stop CSS fingerprinting, CSS exfiltration, form posts or tracking pixels, and it does nothing against prompt injection.

### 2.4 Reject or restrict

- **BitTorrent swarm as a shared cache: reject.** A global, content-addressed cache would reopen the cross-site leak that cache partitioning (Chrome 86, 2020) closed, this time visible on the network: peers learn what you fetch, and the DHT announces your IP for every content hash. The 2026 proposals (Chrome's shared cache of a few hundred popular resources, WICG's hash-keyed cross-site cache) are deliberately narrow, curated and local, and Firefox has rejected even those. A swarm is the opposite. A **magnet-link handler** is acceptable as an opt-in W-1 component that warns the user their IP is exposed to the swarm.
- **GPU piece verification: reject.** Hashing is I/O-bound and fast on CPUs with SHA extensions. Dispatching each piece to the GPU pays microseconds of launch overhead plus readback, which is the per-task accelerator error HGBA-2.0 corrects. BitTorrent v2 uses SHA-256 Merkle trees, not per-piece SHA-1.
- **Browser state published over MQTT: off by default, user-chosen topics only, never inferred states.** Publishing "meeting = active" or "focus = deep-work" to a home broker is a surveillance channel readable on the LAN. Opera's own feature is a preview whose example runs the other way (the lights follow the browser theme). If built at all, it is a W-1 component with a topic allowlist and a local broker, and web content can neither trigger nor observe it.
- **A "Tor container" claiming Tor Browser's anonymity: reject the claim, keep the route.** The Tor Project recommends against Tor in any other browser and warns that mixing Tor and non-Tor browsing can link them. HGBA may offer "route this container through Tor" labelled *hides your IP, not your browser*. The essay's banking examples also contradict each other (Tor with no fingerprinting protection in one place, no VPN at all in another), and banks commonly lock accounts accessed over Tor.
- **Raw sockets for web content: reject.** They would bypass the network service's SOP, CORS, CSP and Local Network Access checks. The platform's answer is Direct Sockets, restricted to Isolated Web Apps and opposed by Mozilla. HGBA follows the standard if one settles.
- **WebGPU as a window manager for local apps: reject.** It adds a trusted path from untrusted local code into the GPU service.
- **A browser-operated VPN service: defer.** Servers, abuse handling and jurisdiction make it a business commitment, not an engine feature.
- **Local and LAN inference: already covered.** B10 and the ML service handle it, off by default. A Home Assistant/Ollama backend is a valid B9 agent option, but it sends page content to another machine, so AG-0 must count it as an AI endpoint.

### 2.5 Gaps the survey exposes in HGBA's own plans

| Gap | Why it matters | Change |
|---|---|---|
| **No PDF viewer** | Every browser has one, and it is a large untrusted-input parser | Stage 1: a viewer in a sandboxed utility process (Rust, or the RLBox-style sandbox). Stage 2: local annotate, sign and redact |
| **bfcache treated as an existing mechanism** | It is the navigation users feel most (9% mobile revenue in Yahoo! JAPAN's case). For the engine it means keeping frozen pages' process, V8 context and data-plane state alive, correctly under site isolation, memory pressure and simulated time | New Stage 1 target: bfcache hit rate on corpus back/forward navigations, plus the WPT bfcache directory |
| **Post-quantum crypto only for TLS** | Chrome 142 ships PQC for WebRTC's DTLS, and its opt-out is going away | Amend HGBA-2.0 Section 4.7 to cover DTLS |
| **Origin-keyed processes undecided** | Chrome 142 keys by origin at ≥4 GB RAM. Kilger et al. (IEEE S&P 2026) exfiltrated data through subdomains sharing a process | Decision: site-keyed by default, origin-keyed above a RAM threshold, measured against S-3, with container keying multiplying on top. This argues for origin keying where affordable, not that isolation is futile |
| **GPU adapter information not in D-S1** | A high-entropy fingerprint surface that Brave masks by default | Add to D-S1's normalized surfaces |
| **Speculation Rules / prerender** | Prerender runs a page's script before the user arrives, which touches the process model and the B7 pixel-optional idea | Stage 2, after bfcache |

The platform features in the survey (Trusted Types, WebTransport, View Transitions, JSPI, the Interop 2026 list) are not HGBA's to choose. WPT tracks them, and C-1 and S-4d gate them. JSPI and the Wasm 3.0 features come with V8, but the bindings must support them, so S-4d's `wasm/jsapi` coverage matters.

### 2.6 The "non-negotiable" table

- **UI chrome (vertical tabs, split view, tab groups):** these are **table stakes, not differentiators**. Build them in Stage 2 on the HTML shell, with no claim that they set HGBA apart. The Stage 1 design's out-of-scope line already excludes them *as differentiation*, and should say so explicitly.
- **"Origin isolation is non-negotiable because Chrome does it at 4 GB"** confuses a competitor's choice with a requirement. Treat it as a measured decision (Section 2.5).
- **"PQC is a regulatory direction"** is about timing. HGBA's crypto agility already covers it; DTLS is the missing piece.

### 2.7 Offline-first: a real need, framed wrongly

"Cache as the primary source, network as refresh" is unsafe as an engine default. HTTP freshness, authentication and script security updates all assume the network is authoritative, so stale-by-default pages break sites (P2) and can serve revoked or vulnerable code. "No mainstream browser serves it" is also overstated: Safari's Reading List works offline, and Chrome saves pages on Android and as MHTML on desktop. The gap is quality, not absence.

HGBA has an unusual asset here. B1's recorded archive of a page load, built for deterministic replay, can also save a page **exactly as the user saw it**: as a replayable archive, or as a frozen DOM-and-resources snapshot. That gives a reading list that works on a plane, built on infrastructure the project needs anyway. Background sync of feeds or pages makes requests the user did not make at that moment, so it is opt-in per source. **Stage 2, user-invoked; never the engine default.**

---

## 3. Recommended changes

| Change | Where | Stage |
|---|---|---|
| Containers keyed into storage **and** process isolation; per-container memory measurement under S-3 | HGBA-2.0 I-1, S-3; Stage 1 design Sections 1, 6 | 1 (key design), 2 (UI) |
| Container policy set: partition, block lists, JavaScript mode, agent origin sets, user-supplied route | Stage 1 design Section 6; HGBA-3.1 B9 | 2 |
| JavaScript modes (off / first-party, no eval / full) per site and container; corpus breakage measured | Stage 1 design Section 6; new target | 2 |
| Fingerprint noise seeded by (site, container, session); GPU adapter info normalized | D-S1 row | 1 |
| PDF viewer in a sandboxed utility process | Stage 1 design Sections 1, 6, M7 | 1 |
| bfcache engine target | Stage 1 design Section 6; HGBA-2.0 Stage 1 table | 1 |
| PQC hybrid key exchange for WebRTC DTLS | HGBA-2.0 Section 4.7 | 1 |
| Origin-keyed processes above a RAM threshold (decision-log entry) | HGBA-2.0 I-1 | 1 decision |
| Speculation Rules / prerender | Stage 1 design Section 9 | 2 |
| "Save for offline" on the B1 archive format, plus a reader view | Stage 1 design Section 9 | 2 |
| Optional W-1 components: feeds, magnet handler (IP warning), MQTT bridge (allowlisted, user-chosen topics), IRC | Stage 1 design Section 9 | 2–3 |
| Vertical tabs, split view and tab groups as Stage 2 table stakes | Stage 1 design Section 6 | 2 |
| Shell theming runs unprivileged and talks to shell services through capability endpoints | Stage 1 design D-S2 | 1 |
| Explicit rejections: swarm shared cache, GPU piece verification, browser-state publishing by default, raw sockets for web content, browser as window manager, Tor-container anonymity claims | Stage 1 design out-of-scope line | — |
