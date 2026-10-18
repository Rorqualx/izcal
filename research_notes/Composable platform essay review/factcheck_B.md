# Fact-check: Group B (B1–B12)

Checked on 14 October 2026 against primary sources: vendor release notes, GitHub repos and releases, chromestatus, W3C/WICG specs, Tor Project support pages, and Mozilla, Brave, Opera and Microsoft docs. Secondary press is cited only where no primary source could be found, and is marked as such.

| # | Verdict | One-line correction |
|---|---|---|
| B1 | Correct (minor nuance) | Glide is real. Its modes also include visual and op-pending, and process spawning is a config API rather than a feature of macros |
| B2 | Mostly correct | Zen 1.19b (27 Feb 2026) added live folders filled with GitHub issues/PRs or RSS items. Calling them an "RSS reader / GitHub dashboard" overstates them |
| B3 | Partly correct | The repo is real (jonnonz1/adaptive-browser). The manifest is JSON (`/.well-known/ui-manifest.json`). YAML is used for user/org *preferences* (the guardrails), and the UI is LLM-generated |
| B4 | Partly wrong (date) | Waterfox 6.7.5 shipped **30 Sep 2026**, not October. OPML import/export already existed: 6.7.5 moved it and kept nested folders. RSS Ninja is real |
| B5 | Partly correct / outdated | Chrome's **Direct Sockets API** (TCP/UDP/server sockets) has been on by default since **Chrome 131**, but only in Isolated Web Apps. The WebTransport spec also defines an HTTP/2 fallback. IRCv3 WebSocket lets IRC servers accept browser clients without a separate gateway |
| B6 | Correct (quote slightly off) | Brave's own wording is "Brave does NOT implement most of the privacy protections from Tor Browser". Bridges: snowflake, obfs4, meek-azure (since 1.44). `arti proxy` runs a SOCKS proxy on port 9150 |
| B7 | Correct | Firefox 149 (24 Mar 2026): free VPN, 50 GB/month, US/UK/DE/FR. Firefox 151 (19 May 2026): location choice. The five launch locations were US, UK, FR, DE and CA. A temporary promotion (to 31 Aug 2026) raised this to 28 locations with no cap |
| B8 | Partly correct | It is **opt-in** ("Always use VPN on Public WiFi", off by default), desktop only, and launched 24 Sep 2026 in the US and France first |
| B9 | Partly wrong | Edge and Chrome have per-profile proxy policy (`ProxySettings`, "Per Profile: Yes"). But user-facing per-identity routing **does exist**: Firefox Multi-Account Containers + Mozilla VPN (per-container exit location, since 2022), container proxy settings, and per-site on/off for Firefox's built-in VPN |
| B10 | Mixed: OmniScout, Stagehand OK; Nanobrowser outdated; **OctoFlow unverifiable / likely fabricated** | See details |
| B11 | Guidance confirmed | Tor Project "strongly recommend[s] against using Tor in any browser other than Tor Browser". Banks locking accounts or geo-blocking Tor is documented by the Tor Project |
| B12 | Correct (version detail) | HTTP cache partitioning shipped in **Chrome 86 (Oct 2020)**, to stop cross-site cache probing / XS-Leaks. Note: in 2026 Chrome *proposed* re-introducing shared caching for a curated list of resources, and Cross-Origin Storage, a content-addressed cache, is proposed in WICG |

---

## B1. Glide
**Verdict: correct, with nuance.**
- Glide is a Firefox fork with a TypeScript config. Its modes are normal, insert, visual, hint and ignore (Shift+Esc); operator keys (`d`, `c`) imply an op-pending state. Source: https://glide-browser.app/ (accessed 14 Oct 2026).
- Robert Craigie's launch post (30 Sep 2025) says the config lets you "define macros" and "spawn arbitrary processes". Its example `gC` keymap uses `glide.process.execute` to run `gh` and clone the GitHub repo in the focused tab, then opens it in kitty/neovim. Source: https://blog.craigie.dev/introducing-glide
- API: `glide.process.spawn` / `glide.process.execute`, https://glide-browser.app/api
- Nuance: macros and process spawning are separate config features. The essay's "macros that run external processes" blends them, but you can combine them. Glide is alpha and macOS/Linux only (korben.info review, secondary).

## B2. Zen Browser live folders
**Verdict: mostly correct; the description is embellished.**
- Zen 1.19b (27 Feb 2026, Firefox 148 base): "Added live folders. Automatically fill your folders with opened GitHub issues, pull requests or RSS feeds." Created via sidebar + → "Live Folder". Fixes followed in 1.19.2b (RSS) and 1.19.9b (GitHub PRs). Source: https://www.zen-browser.app/release-notes
- Correction: live folders fill a sidebar folder with tabs/items. They are not a full RSS reader (no article view or read state is documented) and not a dashboard.

## B3. Adaptive-browser
**Verdict: partly correct.**
- Repo: https://github.com/jonnonz1/adaptive-browser (created 26 Mar 2026, last push 5 Apr 2026, about 344 stars). Stack: Tauri v2 (Rust backend that proxies all API calls) + React (OpenUI components).
- Corrections: services publish a JSON **UI manifest** (`/.well-known/ui-manifest.json`; only a bundled GitHub manifest ships). **YAML is used for preferences**: "User + organization preferences… Organization preferences layer on top as guardrails." The UI is built by an LLM prompt from the manifest/spec plus preferences. The claim "constructs its UI from a YAML manifest" is therefore inaccurate. It is a small personal prototype, not an established project.

## B4. Waterfox 6.7.5 / RSS Ninja
**Verdict: partly wrong (date); features otherwise correct.**
- Waterfox 6.7.5 "Read the Feed" was published **30 Sep 2026**, not October 2026. Source: https://github.com/BrowserWorks/Waterfox/releases/tag/6.7.5 and https://www.waterfox.com/releases/6.7.5. The current release is 6.7.6 (13 Oct 2026).
- Confirmed: a feed manager (`about:feeds`) that "brings posts from your Live Bookmarks together in one place", a built-in reader based on Reader View, feed previews, and Settings → Feeds.
- Correction: OPML import/export was **not new**. 6.7.5 "moved" it to Settings → Feeds and made it "preserve nested bookmark folders".
- RSS Ninja: confirmed as "a fast, private RSS and Atom feed reader built directly into your Firefox sidebar" (Digital Fracture, v1.0.8, 6 Apr 2026). It needs an account and is a paid product after a 14-day trial, and AMO shows about 1 user. Source: https://addons.mozilla.org/en-US/firefox/addon/rss-ninja/

## B5. Raw sockets, WebSocket, WebTransport, IRC
**Verdict: partly correct; misses Direct Sockets.**
- **Direct Sockets API**: `TCPSocket`, `UDPSocket` (with multicast), and `TCPServerSocket`. chromestatus shows "Enabled by default" from **Chrome 131 (desktop)**, limited to **Isolated Web Apps** (IWAs). The IWA manifest must grant the `direct-sockets` permissions policy and the app must be cross-origin isolated.
  - chromestatus: https://chromestatus.com/feature/6398297361088512 (last updated 14 Jan 2025)
  - Spec: https://wicg.github.io/direct-sockets (WICG draft, not a standard)
  - Dev docs: https://developer.chrome.com/docs/iwa/direct-sockets (updated 17 Dec 2025)
  - Intent to Ship (Aug 2024): https://groups.google.com/a/chromium.org/g/blink-dev/c/5R0P_aYBWQI/m/rFMq-qYAEQAJ
  - Mozilla standards position: **negative** (https://github.com/mozilla/standards-positions/issues/431). Safari: no signal.
  - IWAs launched first on ChromeOS, installed by enterprise policy (`IsolatedWebAppInstallForceList`). From Chrome 143 they are allowlist-gated. Other OSes are "following later" (Intent to Ship IWAs; https://developer.chrome.com/docs/iwa/allowlist).
  - Proposed extensions also exist: Direct Sockets in shared/service workers, and multicast.
  - Bottom line: the open web has no raw sockets, but Chromium does for packaged, policy-installed IWAs (and legacy Chrome Apps since 125).
- **WebSocket**: it is an HTTP-upgrade protocol (RFC 6455; RFC 8441 adds HTTP/2 extended CONNECT). Correct.
- **WebTransport**: the W3C CR Snapshot (30 Jul 2026) defines sessions over HTTP/3 **and HTTP/2** (HTTP/2 is reliable-only, `reliability = "reliable-only"`). "HTTP/3-only" is wrong for the spec, though in practice deployments are HTTP/3. Source: https://www.w3.org/TR/webtransport/
- **IRC**: browser clients do need a WebSocket endpoint. However, the IRCv3 WebSocket spec (https://ircv3.net/specs/extensions/websocket) lets IRC servers accept WebSocket connections directly, without a separate gateway process. An IWA with Direct Sockets could also speak raw IRC. "Every browser-based IRC client is a gateway" is too absolute.

## B6. Brave Tor windows, bridges, Arti
**Verdict: correct, but the quote is paraphrased.**
- Brave Help Center (published 16 Oct 2025): "Private Windows with Tor Connectivity in Brave are just regular private windows that use Tor as a proxy. **Brave does NOT implement most of the privacy protections from Tor Browser.**" It also says that Shields site settings, bookmarks and global settings are shared with normal windows, that WebRTC, geolocation and plugins are disabled, and that "If your personal safety depends on remaining anonymous, we highly recommend using Tor Browser." Source: https://support.brave.app/hc/en-us/articles/360018121491-What-is-a-Private-Window-with-Tor-Connectivity
  - Correction: Brave says "privacy protections", not "anti-fingerprinting protections". In Tor windows Brave uses farbling (randomisation) rather than Tor Browser's uniformity approach (https://github.com/brave/brave-browser/wiki/Fingerprinting-Protections).
- Bridges: support was added in Brave 1.44 (brave.com/tor-bridges, 12 Jan 2023). The built-in types are **snowflake, obfs4 and meek-azure** (brave-core `components/tor/tor_utils.cc`). Brave 1.47 added one-click install of the Snowflake *proxy* extension, which lets you volunteer as a proxy; that is different from using Snowflake as a bridge. .onion resolution and auto-redirect are confirmed.
- Arti: "it can be run as a SOCKS proxy on port 9150 by running … `arti proxy`". "Arti's proxy and onion service support are ready for use, but not all features of C Tor are available." Sources: https://arti.torproject.org/guides/starting-arti and https://arti.torproject.org/ (page updated 5 Aug 2026). **Confirmed.**

## B7. Firefox built-in VPN
**Verdict: correct.**
- Firefox 149.0 (24 Mar 2026): "You will get 50 GB of protection every month, with the option to turn it on or off for specific websites." "Progressively rolling out in the US, UK, Germany and France." It needs a Mozilla account and covers browser traffic only. Source: https://www.firefox.com/en-US/firefox/149.0/releasenotes/
- Firefox 151.0 (19 May 2026): the VPN "now lets you choose your browsing location", with a "Recommended" option, as a progressive rollout. Source: https://www.firefox.com/en-US/firefox/151.0/releasenotes/
- The five locations are US, UK, France, Germany and **Canada** (https://support.mozilla.org/kb/built-in-vpn; korben.info). A promotion until 31 Aug 2026 gave unlimited data and 28 locations. Since 1 Sep 2026 free accounts are back to 50 GB and a reduced location list (OMG! Ubuntu, Jun 2026, secondary).

## B8. Opera free VPN on public Wi-Fi
**Verdict: partly correct. It is not automatic by default.**
- Opera blog, 24 Sep 2026: "Opera free VPN can now automatically protect you on public Wi-Fi networks." Users must "Toggle on 'Always use VPN on Public WiFi'" in Settings → VPN. It is desktop only (the changelog points to Opera 136), starting in **France and the US**, "more regions to follow". Source: https://blogs.opera.com/news/2026/09/opera-free-vpn-can-now-automatically-protect-you-on-public-wi-fi-networks/
- Correct framing: an *opt-in* setting that activates the VPN when Opera detects public Wi-Fi. It is limited to some regions and is new as of September 2026.

## B9. Per-profile routing; "user-facing version does not exist"
**Verdict: the first half is correct, the second half is wrong.**
- Edge `ProxySettings` policy: "Per Profile: Yes" (Windows/macOS ≥ 77, Android ≥ 30). Source: https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/proxysettings (ms.date 20 May 2026). Chrome's `ProxySettings` can likewise be applied as a user/profile cloud policy. These are real per-profile policies; "attempt" undersells them.
- User-facing alternatives exist:
  - Firefox Multi-Account Containers + Mozilla VPN give each container its own VPN exit location (Mozilla blog, 2022: https://blog.mozilla.org/products/mozilla-vpn/multi-account-containers-add-on-on-mozilla-vpn/; SUMO: https://support.mozilla.org/en-US/kb/use-multi-account-containers-mozilla-vpn).
  - The containers add-on also has "Advanced proxy settings" for any proxy per container.
  - Firefox's built-in VPN (149+) can be toggled per website.
  - Brave's Tor window is a per-window route.
- What the essay could fairly say: there is no *mainstream built-in, free, per-profile/per-tab VPN or proxy selector* in Chrome, Edge or Safari.

## B10. Agent tools
- **OmniScout: correct.** PyPI `omniscout` ("local-first multi-browser automation, semantic search, and research for AI agents"), v0.4.17 (3 Oct 2026), first release 29 May 2026. The README documents `~/.config/omniscout/config.toml` and `profile` commands, and it drives the user's existing Chrome, Brave or Edge. Source: https://pypi.org/project/omniscout/. The linked GitHub repo (sriramramnath/omniscout) returns 404, so it is private or removed. This is a very young project.
- **Stagehand: correct.** Browserbase's MIT framework. `localBrowser.launch({ userDataDir })` launches a local Chrome; Browserbase cloud is optional. Source: https://github.com/browserbase/stagehand (about 25.7k stars).
- **Nanobrowser: outdated.** It is a Chrome extension that uses your own API keys or local models (Ollama/LM Studio).
  - The **Validator agent was removed in v0.1.9 (25 Aug 2025)**. v0.1.9–v0.2.0 (v0.2.0 released 2 Oct 2026) list only "Navigator, Planner".
  - The `main` branch (nb-v0.3, merged 14 Oct 2026) is a rewrite with a single side-panel agent and per-chat model selection. Its old per-agent `agent-models` settings are now treated as "legacy".
  - Source: https://github.com/nanobrowser/nanobrowser (README at tags v0.1.8 vs v0.1.9; releases).
- **OctoFlow: unverifiable / likely fabricated.** No browser-agent project named OctoFlow with Ollama backends turned up on GitHub, PyPI or the web. The GitHub repos named "octoflow" are a GPU programming language, a Discord bot, a 2022 text-classification repo and a Gemini/Groq business tool. Remove it or replace it with a verifiable tool, for example browser-use (supports Ollama via ChatOllama) or BrowserOS (Ollama provider).

## B11. Tor Project guidance; banks and Tor exits
- **Using Tor in another browser**: "We strongly recommend against using Tor in any browser other than Tor Browser." Other browsers can leak the real IP (DNS/WebRTC), OS, fonts and plugins for fingerprinting, and cookies and cache. "Tor Browser is carefully developed and modified to prevent these kinds of leaks and to make Tor users look alike, which greatly improves anonymity. Other browsers lack these defenses, so using them with Tor can make you easier to identify and track." Source: https://support.torproject.org/tbb/tbb-9/ (accessed 14 Oct 2026).
- **Mixing Tor and non-Tor browsing**: the same page says that running both at once "won't affect Tor's performance or privacy properties", **but** "your Tor activity could be linked to your non-Tor (real) IP from the other browser, simply by moving your mouse from one browser into the other", and you may "accidentally use that non-private browser".
  - Within a single browser such as Brave, settings, bookmarks and Shields exceptions are shared between Tor and normal windows (Brave Help Center, above). The fingerprint is a Chromium/Brave one, not the uniform Tor Browser one, so a Tor-window user is in a much smaller anonymity set.
- **Banks**: Tor Project: "Some websites, such as banks or email providers, might interpret this as a sign that your account has been compromised, and lock you out." Recommended remedies are 2FA and the site's recovery process (https://support.torproject.org/tor-browser/encountering-issues/website-lockout/).
  - Also: "The Tor network is public, and websites can always detect when connections are coming from Tor… In the case of banks, and other sensitive websites, it is also common to see geography-based blocking" (https://support.torproject.org/tor-browser/general/website-blocking-tor-exits/).
  - Verdict: lockouts and geo-blocking by banks are officially documented as common. There is no authoritative statistic on what share of banks outright block exit IPs.

## B12. HTTP cache partitioning
**Verdict: correct; the precise version is Chrome 86.**
- "Starting in Chrome 86" (rolled out Oct 2020), the cache key is the URL plus a Network Isolation Key of (top-level site, current-frame site).
  - Rationale: a shared cache "can reveal that the browser has accessed the same resource in the past". This enables (1) detecting whether a user visited a site, (2) cross-site search attacks (probing for a cached "no results" image), and (3) cross-site tracking via cache-stored identifiers.
  - Measured cost: cache miss rate about +3.6%, FCP about +0.3%, bytes from network about +4%.
  - Source: https://developer.chrome.com/blog/http-cache-partitioning (6 Oct 2020)
  - chromestatus "Partition the HTTP Cache": https://chromestatus.com/feature/5730772021411840 (Firefox and Safari positive; Fetch issue https://github.com/whatwg/fetch/issues/904).
  - Safari partitioned earlier (top-level eTLD+1, around 2013). Firefox shipped Network Partitioning in Firefox 85 (Jan 2021).
  - Later refinements: navigation initiator added to the key (Chrome 136); the frame site may be replaced with an "is-cross-site" bit (dev trial).
- **History of content-addressed shared caches**: SRI hash-based cross-origin cache sharing was proposed and closed in WebAppSec because of cache-probing / privacy concerns (https://github.com/w3c/webappsec-subresource-integrity/issues/22, opened Dec 2015, closed). So the cross-origin cache was *removed* in 2013–2021, and hash-based sharing was *never shipped*.
- **2026 counter-trend the essay should note**:
  - Chrome "Cache sharing for extremely-pervasive resources" (Proposed, desktop milestone 146; Firefox **negative**) would use a single-keyed cache for a few hundred hand-curated popular third-party resources (https://chromestatus.com/feature/5202380930678784).
  - WICG **Cross-Origin Storage** is "a content-addressable browser cache that shares files across origins by hash, with k-anonymity-based availability gating" (Proposed; Firefox "under consideration"; updated 14 Sep 2026; https://chromestatus.com/feature/5163371507875840, https://github.com/WICG/cross-origin-storage).
