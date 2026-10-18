# Design critique: "The Browser as a Composable Platform" (draft, written before the fact-checks returned)

## Verdict

Two ideas in the essay are worth adopting, and both are smaller than the essay makes them: **containers as a policy context** (identity, plus ad-block, JavaScript and agent policy) and **graded per-site JavaScript**. The essay's frame, "a kernel of capabilities and a user responsible for the composition", is wrong for HGBA's users. The two most novel proposals are unsafe as stated: **BitTorrent as a shared cache**, and **the browser publishing its state over MQTT**. The rest are integrations that sit outside the engine and belong, if anywhere, in HGBA-2.0's Stage 2 component host (W-1): RSS, IRC, VPN, Tor and the agent settings page.

## 1. The frame: "closer to Linux than to a consumer appliance"

- **It contradicts the evidence the project just reviewed.** Arc's own letter said it was "too different, with too many new things to learn, for too little reward", and only 5.52% of daily users regularly used more than one Space. Multi-Account Containers, the essay's lead example, has about 411k users on addons.mozilla.org. "Users keep installing extensions" is true of ad blockers and of almost nothing else on the essay's list. A browser that makes the user "responsible for the composition" builds for the power-user niche Arc found too small to sustain a product.
- **It contradicts P1 and the threat model.** Each composed primitive adds attack surface in a privileged process: a BitTorrent stack, a DHT node, an MQTT client, a raw-socket API, an IRC parser, a Tor client. Brave removed WebTorrent, and the essay itself names the reason: the threat model treated peer-to-peer code as attack surface. The essay then proposes more of the same without answering that objection.
- **"Every primitive exists somewhere" is not evidence that the composition works.** Composition multiplies the interactions that need testing. HGBA-3.1's discipline asks for an oracle per component. The essay names no metric, no maturity label and no failure condition for any proposal.
- **What survives:** a small set of **policy contexts** with good defaults, and a Stage 2 component host for opt-in additions. That is an ordinary-user product with a power-user escape hatch, not a distribution.

## 2. Containers as a policy context: adopt, with the engine consequences stated

This is the best idea in the essay, and HGBA can do it better than Firefox because the engine is being designed now.

- **Isolation key.** A container must be part of the storage key *and* the process key. Today HGBA-2.0 I-1 isolates by site. Containers make the key (site, container): the same site in two containers gets two processes and two storage partitions, and no cache, connection pool, HSTS state or service worker is shared. Otherwise a container is a cookie jar that cross-container cache-timing attacks can see through. That costs memory (S-3's 15% budget is measured per site; it needs a per-container variant).
- **Policy carried by the container:** cookie and storage partition (always), content-blocking list set, JavaScript mode (Section 3 below), agent policy (B9 origin sets scoped by container), and network route (Section 5). Each is a capability the broker grants to the container's renderers, which fits B2 directly.
- **Defaults matter more than flexibility.** Ship 3–4 named containers with sensible policies, and let users edit them. A blank policy editor is the Arc failure mode.
- **Interaction with anti-fingerprinting (D-S1).** The seeded per-site, per-session noise should also be per container, so two containers on one site cannot be linked by their fingerprint. That is a free consequence of seeding by (site, container, session).

## 3. Graded JavaScript: adopt as per-site and per-container policy, with honest claims

- **The security claims are overstated.** With JavaScript off, a page can still submit forms, load tracking pixels, fingerprint through CSS and HTTP headers, and exfiltrate data through CSS selectors. Prompt injection is text, so disabling JavaScript does nothing against it. JavaScript off removes the largest class of *engine* exploits (the JIT and the bindings) and most active tracking. That is a real benefit, and it is the Hardened profile's argument taken further.
- **"No browser ships it" needs the fact-check.** Chrome has had a per-site JavaScript block in Site settings for years. The real gap is the middle mode.
- **The middle mode ("first-party JavaScript only, no eval") is technically tractable in HGBA.** Script loading goes through the network service and the bindings, so the engine can refuse third-party script fetches and disable `eval` and `Function` per realm. V8 supports refusing code generation from strings per context. The breakage cost is unknown: many sites load their own scripts from CDN origins that count as third-party. It needs a corpus measurement before it becomes a default, under the P2 scope note (class (b), user-invoked).
- **Interaction with B10 and agents:** a container whose JavaScript mode is "off" can still host an agent that reads the semantic plane, because the semantic tree does not need page script.

## 4. BitTorrent: reject the shared cache, defer the protocol handler

- **The shared cache is a privacy regression the platform already fixed.** Browsers partitioned the HTTP cache by top-level site (Chrome in 2020) because a shared cache lets one site learn what the user visited on another. A global, content-addressed swarm cache makes the same leak *network-visible*. Peers learn which assets the user fetches, and the DHT announces the user's IP for each content hash. It contradicts P1 and the partitioning HGBA-2.0 already requires.
- **GPU piece verification is the opposite of what HGBA-2.0's HAL rules allow.** Hashing is I/O-bound and fast on CPUs with SHA extensions. Per-piece GPU dispatch pays the 3–80 µs launch cost plus readback for microseconds of work. That is exactly the "per-task accelerator scheduling" error HGBA-2.0's changelog corrects. BitTorrent v2 also uses SHA-256 Merkle trees, not per-piece SHA-1.
- **A magnet-link handler** is a legitimate Stage 2 component (W-1) with an explicit warning that the user's IP is exposed to the swarm. It belongs outside the renderer and outside the default build.

## 5. Per-container network routing (VPN, Tor, DNS): adopt the routing, reject "Tor container" anonymity claims

- **Per-container routing fits the architecture.** All traffic already goes through the network service (HGBA-2.0 Section 4.7). A route (proxy, VPN, DNS resolver, TLS minimum) is a property of the network-service context bound to a container, so the change is contained. This is a genuine differentiator, because consumer browsers apply VPNs and proxies globally.
- **A Tor container is not Tor Browser.** Tor Browser's anonymity comes from every user looking identical and from strict isolation. A Tor container inside a browser whose other containers have different fingerprints, extensions and timing is distinguishable. Brave's own documentation says its Tor windows do not give Tor Browser's protections. HGBA can offer "route this container through Tor" with an honest label (hides the IP, not the browser), and should not claim the anonymity set.
- **The essay's examples contradict each other.** One section says the banking container routes through Tor with no fingerprinting protection; another says it uses no VPN at all. Many banks block Tor exit nodes, so the first example would break the use case it proposes.
- **A built-in VPN service** is a business and operations commitment (servers, abuse handling, jurisdiction), not an engine feature. HGBA should support user-supplied routes (WireGuard config, SOCKS, Arti) per container and leave operating a VPN to a later product decision.

## 6. MQTT and smart-home integration: reject as browser state publishing, allow as an opt-in component

- **Publishing "meeting = active", "focus = deep-work" and "media = playing" to a home broker is a surveillance channel.** It turns behavioural inference about the user into network messages on a LAN that other devices and household members can read. Under P1 it must be off by default. Under the P2 scope note it is outside content's reach entirely: no web page may trigger or observe it.
- **The LAN is a trust boundary HGBA already treats carefully** (Local Network Access in HGBA-2.0 Section 1). An MQTT client in a privileged process is new attack surface reachable from the LAN.
- **If built:** a Stage 2 component with an explicit topic allowlist, a local broker only, and state the user chooses to publish. Never inferred states.

## 7. WebGPU and local inference: mostly already decided

- WebGPU is in HGBA-2.0 already (D3, two capability tiers), and B10 and the ML service already cover local inference, **off by default**.
- **"WebNN for NPU access" is not shipping in any stable browser** (HGBA-2.0 Section 9.1); treat it as experimental.
- **"WebGPU as the display substrate for non-web content, making the browser a window manager"** contradicts HGBA-2.0's isolation model: local apps compositing into the browser's GPU service is a new trusted path for untrusted code. It is rejected.
- **A LAN model host (Home Assistant with Ollama)** is a reasonable *agent backend* option under B9 and B10, if it is opt-in and labelled. It moves page content off the device to another machine. That is not "no cloud round trip" in privacy terms if the home server is shared. AG-0 must count it as an AI endpoint.

## 8. Raw sockets and IRC

- A raw TCP socket API for web content would bypass the network service's SOP, CORS, CSP and Local Network Access checks, which HGBA-2.0 Section 4.7 forbids. The platform's answer is Chrome's Direct Sockets, limited to Isolated Web Apps (subject to the fact-check). HGBA should follow that standard if it settles, not invent its own.
- **IRC and RSS as built-in readers** are shell components with no engine dependency. Feeds as a rendering mode is a reasonable Stage 2 shell feature, because it is local-first and opt-in. IRC has a small audience. Both belong in the W-1 component host, not in the default build.

## 9. Wasm components as browser modules

This is HGBA-2.0 W-1, already planned for Stage 2: extensions and privileged plug-ins as WASI 0.3 components with declared capabilities. The essay adds nothing new architecturally, but it gives W-1 a better product description. Its capability examples (network to listed hosts, files in listed directories, GPU) map onto B2's endpoints. The one caution: **GPU access for a module** is a capability with a large attack surface and should go through the GPU service like WebGPU, never as raw device access.

## 10. User-authored shell

D-S2 already makes the shell HTML and CSS. Treating the shell as a page users can restyle is a cheap consequence. It needs one guard: the shell runs with privileges, so user-authored shell code must run in an unprivileged sandboxed context and talk to privileged shell services through the same capability endpoints. Otherwise a pasted "theme" becomes a privilege escalation, which is the userChrome.js lesson.

## 11. Recommended changes to the HGBA documents

| Change | Where | Stage |
|---|---|---|
| Containers as policy contexts keyed into both storage and process isolation: (site, container) | HGBA-2.0 I-1 note; Stage 1 design Sections 1, 6 | Stage 1 key design; Stage 2 UI |
| Container policy set: storage partition, block lists, JavaScript mode, agent policy, network route | Stage 1 design Section 6; HGBA-3.1 B9 (agent policy per container) | 2 |
| JavaScript modes: off, first-party only with no eval, full; per site and per container; breakage measured on corpus | Stage 1 design Section 6; new target | 2 |
| Per-container routes from user-supplied configs (WireGuard, SOCKS, Arti); Tor route labelled "hides IP, not browser" | Stage 1 design Section 6 | 2 |
| Fingerprint noise seeded by (site, container, session) | D-S1 row | 1 |
| Shell theming in an unprivileged context | Stage 1 design D-S2 | 1 |
| W-1 product description: feeds, magnet handler, MQTT bridge, IRC as optional components | Stage 1 design Section 9 | 2–3 |
| Explicit rejections: shared swarm cache, browser-state publishing by default, raw sockets for web content, GPU piece verification, browser as window manager | Stage 1 design out-of-scope line | — |

---

# Part 2: the second pasted section ("minimum viable platform" for 2026–27)

This section is a different kind of document: a survey of what shipped in 2025–26, ending in a "non-negotiable" table. As a survey it is useful. As requirements it mixes four categories that HGBA must treat differently: (a) web-platform conformance, which HGBA gets from WPT and Interop, not from feature picks; (b) engine architecture HGBA already specifies; (c) shell table stakes; and (d) one real differentiator claim, offline-first.

## 12. What it adds that the HGBA documents are missing

- **A PDF viewer.** The Stage 1 design has no PDF viewer at all, and every browser ships one. It is a large untrusted-input parser, so it belongs in a sandboxed utility process (HGBA-2.0 Rule of 2), written in Rust or run through the RLBox-style library sandbox. Local PDF editing (annotate, sign, redact) is a Stage 2 shell feature on top. **Add to Stage 1.**
- **bfcache as an engine requirement, not only a site property.** The essay is right that bfcache is the navigation feature users feel most. For sites, eligibility is mechanical. For an engine it is not: frozen documents must keep their process, their V8 context and their data-plane state, survive the B1 substrate's simulated time, and interact correctly with site isolation and the memory policy. HGBA-2.0 lists bfcache only as an existing mechanism. **Add a Stage 1 target:** bfcache hit rate on corpus back/forward navigations, with WPT `html/browsers/browsing-the-web/back-forward-cache/` passing.
- **Speculation Rules and prerendering.** A prerendered page is a hidden renderer whose script runs before the user navigates, so it touches the process model, the B7 pixel-optional idea (prerender is "render without a viewer") and the network service's priorities. **Stage 2**, after bfcache.
- **PQC for WebRTC DTLS.** HGBA-2.0 Section 4.7 specifies hybrid post-quantum key exchange for TLS only. WebRTC's DTLS needs the same crypto agility. **Amend 4.7.**
- **Origin-keyed processes.** HGBA-2.0 I-1 isolates by site. Origin-keyed processes are stricter and cost more memory. Combined with containers (Section 2 of this critique), the process key could become (origin, container), which multiplies processes. **Decision needed:** site-keyed by default with origin-keyed above a RAM threshold, as Chrome does, measured against S-3.
- **GPU fingerprint masking.** WebGL and WebGPU adapter information is a high-entropy surface. It belongs in D-S1's normalized discrete surfaces. **Amend the D-S1 row.**
- **Trusted Types, WebTransport, View Transitions, JSPI and the Interop 2026 list** are platform features. HGBA does not choose them. WPT tracks them, and C-1 and S-4d gate them. One note: JSPI and the Wasm features come with V8, but the bindings must support them, so `wasm/jsapi` WPT coverage matters (S-4d already names it).

## 13. Where the section is wrong or overreaches

- **"Non-negotiable" UI chrome contradicts the earlier essay and the evidence.** Vertical tabs, split view and tab groups are table stakes for power users, not what makes a browser worth switching to. The Stage 1 design's out-of-scope line rules them out *as differentiation*, not as features. Resolution: they are Stage 2 shell table stakes, built cheaply on the HTML shell (D-S2), with no claim that they differentiate.
- **"Origin isolation is non-negotiable because it is default on 4 GB+ devices"** confuses what Chrome does with what security requires. HGBA-2.0 already accepts site isolation's 9–13% memory cost. Origin keying is a further trade, to be measured, not adopted because a competitor did.
- **"PQC is a regulatory direction"** is a reason about timing, not architecture. HGBA's crypto agility already covers it.
- **The IEEE paper's finding** (side channels cross process boundaries; same-site resources share a process) argues *for* origin keying and for HGBA-2.0's I-6 Spectre rules. It does not argue that isolation is futile. The essay states it without that conclusion.
- **"WebAssembly 3.0 is the next milestone"** is out of date if the fact-check confirms the standard was finalized in 2025. Either way, HGBA inherits Wasm features from V8, so this is not a design choice.

## 14. Offline-first: a real idea, framed wrongly

- **"Cache as the primary source, network as refresh" is unsafe as an engine default.** HTTP caching semantics, freshness rules, security updates to scripts, and authentication all assume the network is authoritative. Serving stale pages by default would break sites (P2) and could serve revoked or vulnerable script.
- **"No mainstream browser serves it" needs the fact-check.** Safari's Reading List saves pages offline, Chrome has offline pages, and the gap may be quality rather than absence.
- **The defensible version is user-invoked, and HGBA has an unusual asset for it.** The B1 substrate already defines a recorded archive of a page load (Section 3.2 of the Stage 1 design) for deterministic replay. The same machinery can save a page **as the user saw it**: either a deterministic replay archive, or a frozen DOM-and-resources snapshot like MHTML or Web Bundles. That is a reading list that works on a plane, built on infrastructure HGBA needs anyway. Background sync of subscribed feeds and pages is a privacy question (it makes requests the user did not make at that moment), so it is opt-in per source.
- **Recommendation:** a Stage 2 "save for offline" feature built on the B1 archive format, plus an optional reader rendering. Offline is a mode the user enters, not the engine's default.

## 15. Additions to the recommended-changes table

| Change | Where | Stage |
|---|---|---|
| PDF viewer in a sandboxed utility process; PDF editing later | Stage 1 design Sections 1, 6 | 1 (viewer), 2 (editing) |
| bfcache as an engine target (hit rate on corpus, WPT bfcache directory) | Stage 1 design Section 6 and milestones; HGBA-2.0 Stage 1 table | 1 |
| Speculation Rules / prerender | Stage 1 design Section 9 | 2 |
| PQC hybrid key exchange for WebRTC DTLS | HGBA-2.0 Section 4.7 | 1 |
| Origin-keyed processes above a RAM threshold, measured against S-3; interaction with container keying | HGBA-2.0 I-1; decision log | 1 decision |
| GPU adapter information in D-S1's normalized surfaces | Stage 1 design D-S1 | 1 |
| Vertical tabs, split view, tab groups as Stage 2 table stakes (not differentiators) | Stage 1 design Section 6 out-of-scope line | 2 |
| "Save for offline" on the B1 archive format, plus reader rendering; background sync opt-in per source | Stage 1 design Section 9 | 2 |
