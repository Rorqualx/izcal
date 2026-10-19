# Izcal networking: history and postmortems

Compiled 19 October 2026. Primary sources were preferred (RFCs, maintainers' blogs, Chromium and Mozilla docs, papers, release notes, bug trackers and commit messages). Bugzilla records were read through its REST API, Chromium source through googlesource, Ladybird and rustls history through the GitHub API, and the QUIC SIGCOMM paper from its full text.
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, secondary coverage that quotes the primary, or a vendor claim without data; **Low** = secondary source only, or the researcher's recollection not checked against a source this session.

M-1 note: this file records ideas, measurements and reasons. It contains no code and no design template from any stack. Where it says what a stack "does", that is evidence to reason from, not a shape to copy.

## 0. Chronology at a glance

| Year | Event | Section |
|---|---|---|
| 1994–97 | Netscape cookie spec; RFC 2109 (Feb 1997) | 8 |
| 1999 | HTTP/1.1 (RFC 2616) defines pipelining; IE's WinInet era | 1, 4 |
| 2000 | RFC 2965 Cookie2/Set-Cookie2 (never adopted by browsers) | 8 |
| 2008 | Chrome ships its own cross-platform net stack, not WinHTTP/WinINet | 4 |
| Nov 2009 | SPDY announced (whitepaper: up to 64% faster page loads in lab) | 2 |
| 2010–12 | Chrome TLS False Start; restricted to NPN sites in Chrome 20 (Apr 2012) | 6 |
| 2011 | RFC 6265 obsoletes RFC 2965 | 8 |
| 2012–13 | gQUIC designed; first experiment 2013 | 3 |
| 2013 | Safari partitions third-party storage/cache by top-level site | 7 |
| Feb 2015 | Chrome announces HTTP/2, plans to drop SPDY and NPN | 2 |
| May 2015 | HTTP/2 (RFC 7540) and HPACK (RFC 7541) | 2 |
| 2016 | Sans-IO pattern named (h11, hyper-h2; PyCon 2016); rustls repo created | 5 |
| May 2016 | Chrome 51 removes SPDY and NPN | 2 |
| Nov 2016 | Intent to Unship SDCH (Chrome) | 6 |
| Dec 2016 | Mozilla opens "move all TCP/UDP into a dedicated process" (still open 2026) | 4 |
| Mar 2017 | Firefox 54 removes HTTP/1 pipelining | 1 |
| Aug 2017 | SIGCOMM QUIC paper: >30% of Google egress, ~7% of Internet | 3 |
| Oct 2017 | Chrome intent to remove HPKP | 6 |
| Jun 2018 | Apple Network.framework: user-space transport in the app | 4 |
| 2018–19 | Chrome Network Service rollout (Chrome 72 experiment, Feb 2019) | 4 |
| Feb 2020 | Firefox DoH on by default for US users | 7 |
| May 2020 | Chrome 83 Secure DNS (same-provider auto-upgrade) | 7 |
| Jun 2020 | Cure53 audit of rustls/ring/webpki | 5 |
| Oct 2020 | Chrome deploys IETF QUIC h3-29 to 25% of stable; Chrome 86 partitions HTTP cache | 3, 7 |
| Jan 2021 | Firefox 85 network partitioning | 7 |
| May 2021 | QUIC (RFC 9000) | 3 |
| Oct 2021 | AppCache gone from Chrome (reverse origin trial ends) | 6 |
| Jun 2022 | RFC 9113 (HTTP/2 bis, deprecates 7540 priorities), RFC 9114 (HTTP/3), RFC 9218 (Extensible Priorities) | 2 |
| Sep 2022 | Chrome 106 disables HTTP/2 server push | 2 |
| Sep–Oct 2023 | ECH ships in Chrome (117) and Firefox (118) | 7 |
| Feb 2024 | rustls 0.23: aws-lc-rs default, unbuffered (sans-IO) API | 5 |
| Sep 2024 | Ladybird RequestServer replaces home-grown HTTP with libcurl | 4 |
| Dec 2024 | Let's Encrypt announces end of OCSP | 6 |
| Feb 2025 | Firefox 135 removes Do Not Track, points to GPC | 6 |
| Apr 2025 | Google keeps third-party cookies, no standalone prompt | 6 |
| Aug 2025 | Let's Encrypt OCSP responders turned off | 6 |
| Oct 2025 | Google retires most Privacy Sandbox APIs (Topics, Protected Audience, ...) | 6 |
| 2025 | Cloudflare: HTTP/2 50%, HTTP/1.x 29%, HTTP/3 21% of requests | 3 |
| Mar 2026 | ECH published as RFC 9849 | 7 |
| 2026 | RFC 6265bis-22 sits in the RFC Editor queue; Ladybird hardens curl against Fetch rules | 8, 4 |

## 1. HTTP/1.1 pipelining (1999–2017)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | Pipelining was defined in HTTP/1.1 so a client could send several requests on one connection without waiting; responses must come back **in order**. The era constraint: one TCP connection was expensive (slow start, handshake), and multiplexing would have needed framing HTTP/1.1 did not have. | — | [RFC 2616 §8.1.2.2](https://www.rfc-editor.org/rfc/rfc2616#section-8.1.2.2) | 1999-06 | Medium |
| 1.2 | **Chromium's reasons for removing it:** broken servers that ignore pipelined requests or garble replies; broken proxies, including transparent ones the user never configured; front-of-queue blocking where a slow first response delays everything behind it. Status: the option was removed, "known crashing bugs" remained, and "it's recommended nobody uses pipelining." | no numbers given | [Chromium: HTTP Pipelining](https://www.chromium.org/developers/design-documents/network-stack/http-pipelining/) | undated (c. 2013) | High |
| 1.3 | **Firefox removal** (bug 1340655, Patrick McManus): pipelining "has been subsumed by the more effective mux of spdy, h2, and soon quic"; it "was never able to overcome the limitations of HoL blocking"; the code was "a maintenance burden with minimal value given the default off state." Shipped in Firefox 54. | opened 2017-02-17, fixed 2017-03-02 | [Bugzilla 1340655](https://bugzilla.mozilla.org/show_bug.cgi?id=1340655) | 2017-03-02 | High |
| 1.4 | The SPDY whitepaper names pipelining's limits as motivation: still a single in-order stream, a slow or lost first response delays the rest, and "major browsers" left it off because it was hard to deploy. | — | [SPDY whitepaper](https://www.chromium.org/spdy/spdy-whitepaper/) | 2009-11 | High |
| 1.5 | Opera (Presto) was the one major desktop browser that shipped pipelining on by default, with heuristics to detect broken servers; it went away with Presto when Opera moved to Chromium (2013). | — | (recollection; no primary checked) | 2000s–2013 | Low |

**Implications:** Pipelining failed for two separate reasons: the protocol (in-order responses mean head-of-line blocking by design) and the deployed base (intermediaries that never implemented it correctly). Izcal should not ship HTTP/1.1 pipelining; HTTP/1.1 should be a correct, strictly serial-per-connection fallback, and parallelism belongs to HTTP/2 and HTTP/3.

## 2. SPDY to HTTP/2, server push and priorities (2009–2022)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **SPDY goals:** cut page load time ~50%, run over TCP and TLS so no network changes, no content changes, many concurrent requests on one session, header compression, server push. Always-on: multiplexing, prioritization, header compression; optional: push and server hint. | Lab: 27–60% faster than HTTP, up to 64%; headers 85–88% smaller; benefit vanished above 2.5% loss | [SPDY whitepaper](https://www.chromium.org/spdy/spdy-whitepaper/) | 2009-11 | High |
| 2.2 | Chrome had supported SPDY since Chrome 6. On HTTP/2's approval Chrome announced removal of SPDY and of **NPN in favour of ALPN** (the standards-track TLS extension); both were removed in **Chrome 51**. | Chrome 6 → 51 | [Chromium blog, "Hello HTTP/2, Goodbye SPDY"](https://blog.chromium.org/2015/02/hello-http2-goodbye-spdy.html) | 2015-02-09 (updated 2016) | High |
| 2.3 | **RFC 7540 priority tree failed.** RFC 9113: the scheme "proved to be complex, and it was not uniformly implemented... the prioritization signaling in RFC 7540 was not successful." RFC 9218: "a complex system where clients signal stream dependencies and weights to describe an unbalanced tree"; "Many RFC 7540 server implementations do not act on HTTP/2 priority signals." Replaced by the `Priority` header / PRIORITY_UPDATE frame with urgency 0–7 and an incremental flag, shared by HTTP/2 and HTTP/3. | — | [RFC 9113 §5.3](https://www.rfc-editor.org/rfc/rfc9113.html#section-5.3); [RFC 9218 §2](https://www.rfc-editor.org/rfc/rfc9218.html) | 2022-06 | High |
| 2.4 | **Server push removed from Chrome 106.** Usage: 1.25% of HTTP/2 sites in HTTP Archive 2021, falling to 0.7% on a rerun. Benefits "were often difficult to realize"; Chrome and Akamai analyses showed no clear net gain and often regressions; many HTTP/3 clients and servers never implemented it. Replacements: 103 Early Hints and preload, where the browser decides. | 1.25% → 0.7% of sites | [developer.chrome.com, "Removing HTTP/2 Server Push"](https://developer.chrome.com/blog/removing-push) | 2022-08-18 | High |
| 2.5 | Root mismatch: push assumed the server knows what the client lacks, but only the client knows its cache. Cache Digests (a draft to send a cache summary) tried to close that gap and was abandoned; it would also have leaked cache state. | — | [draft-ietf-httpbis-cache-digest](https://datatracker.ietf.org/doc/draft-ietf-httpbis-cache-digest/) | 2016–2019 | Low |
| 2.6 | Firefox also disabled HTTP/2 push later (believed Firefox 132, 2024). | — | (recollection) | 2024 | Low |

**Implications:** HTTP/2's lasting value is framing, multiplexing and HPACK; its two ambitious features (push and the dependency tree) failed because the party sending the signal was not the party that had the information, or because servers ignored it. Izcal's HTTP/2 and HTTP/3 codecs should implement RFC 9218 priorities, accept but never request push (send SETTINGS_ENABLE_PUSH=0; for HTTP/3 never send MAX_PUSH_ID), and support 103 Early Hints.

## 3. gQUIC to IETF QUIC and HTTP/3 (2012–2025)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Why userspace:** TCP lives in OS kernels, so changes need OS upgrades; "Building QUIC in user-space... enabled iterative changes to occur at application update timescales." **Why UDP:** it traverses middleboxes, which block anything unfamiliar and rewrite transport headers. **Why encrypt headers:** "preventing modification and limiting ossification"; TCP changes "now expected to take upwards of a decade." | — | [Langley et al., SIGCOMM 2017, §1–2](https://dl.acm.org/doi/10.1145/3098822.3098842) | 2017-08 | High |
| 3.2 | **Results:** Search latency −8.0% desktop, −3.6% mobile; YouTube rebuffers −18.0% desktop, −15.3% mobile; over 30% of Google egress bytes; about 7% of Internet traffic. First experiment in 2013. | as listed | [SIGCOMM 2017 abstract/§1](https://research.google/pubs/the-quic-transport-protocol-design-and-internet-scale-deployment/) | 2017-08 | High |
| 3.3 | **TCP Fast Open's failure, per the paper:** limited to data that fits in the SYN, only works on repeat connections to the same IP (load balancers break that), and "deployment is limited due to middlebox interference and due to slow client OS upgrades." | — | [SIGCOMM 2017 §8](https://dl.acm.org/doi/10.1145/3098822.3098842) | 2017-08 | High |
| 3.4 | **UDP blocking:** QUIC worked for 95.3% of video clients; 4.4% could not use it (mostly enterprise firewalls); 0.3% throttled. AS-level throttling fell from 1% (Jun 2015) to 0.3% (Nov 2016) after Google contacted operators. TCP fallback is therefore mandatory. | 95.3 / 4.4 / 0.3% | [SIGCOMM 2017 §7.2](https://dl.acm.org/doi/10.1145/3098822.3098842) | 2016-11 data | High |
| 3.5 | **Ossification within a year:** a one-bit flags change in Oct 2016 made one firewall brand let initial packets through then black-hole the rest, defeating TCP fallback. Lesson stated: "encryption is the only means available to ensure that bits that ought not be used by a middlebox are in fact not used by one" ("deployment impossibility cycle"). | — | [SIGCOMM 2017 §7.5](https://dl.acm.org/doi/10.1145/3098822.3098842) | 2016-10 | High |
| 3.6 | **Costs:** server CPU was ~3.5× TLS/TCP at first, reduced to ~2× (hand-optimized ChaCha20, mmap'd RX ring, cache-efficient state). XOR FEC was tried and dropped: no significant search gain, worse video latency. Its home-grown crypto handshake was replaced by TLS 1.3 at the IETF. | 3.5× → 2× CPU | [SIGCOMM 2017 §6.8, §7.3, §8](https://dl.acm.org/doi/10.1145/3098822.3098842) | 2017-08 | High |
| 3.7 | Chrome moves to IETF QUIC h3-29: search latency −2%+, YouTube rebuffer time −9%+, throughput +3% desktop / +7% mobile **vs HTTP over TLS 1.3 over TCP**; 25% of stable users at launch; gQUIC Q050 kept in parallel during transition. | as listed | [Chromium blog, "Chrome is deploying HTTP/3 and IETF QUIC"](https://blog.chromium.org/2020/10/chrome-is-deploying-http3-and-ietf-quic.html) | 2020-10-07 | High |
| 3.8 | Standards: QUIC RFC 9000 (May 2021); HTTP/3 RFC 9114 and QPACK RFC 9204 (June 2022). | — | [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html); [RFC 9114](https://www.rfc-editor.org/rfc/rfc9114.html) | 2021–22 | Medium |
| 3.9 | **Deployment share 2025** (Cloudflare, requests): HTTP/2 50%, HTTP/1.x 29%, HTTP/3 21%, "largely unchanged" from 2024. Post-quantum share of TLS rose from 29% to 52% during 2025. | 50 / 29 / 21% | [Cloudflare Radar 2025 Year in Review](https://blog.cloudflare.com/radar-2025-year-in-review) | 2025-12 | High |

**Implications:** QUIC exists because transport evolution needed to move at application speed; Izcal owning its QUIC stack is the intended deployment model, not an anomaly. But every browser must race or fall back to TCP (4–5% of clients cannot use UDP), must budget for higher CPU per byte than kernel TCP, and must keep HTTP/1.1 first-class (29% of requests in 2025). Version negotiation and GREASE are survival features, not extras.

## 4. Browser network stack architectures

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **Chromium net/** was written to "provide greater control than would be available with higher-level system networking libraries (e.g. WinHTTP or WinINET)", "avoid bugs that may exist in system libraries", and enable optimizations. Mostly single-threaded, non-blocking, callback-driven; DNS and cert verification on worker threads. | 6 sockets/host, 32/proxy, 256/process; 8 concurrent resolves; HostCache 1000 | [Chromium network stack design doc](https://www.chromium.org/developers/design-documents/network-stack/) | c. 2010 (marked outdated) | High |
| 4.2 | **Network Service:** a Mojo service for low-level networking, "meant to be oblivious to Chrome's features"; CORS and ORB enforced inside it. Out-of-process is preferred "for isolation and stability" and is the default on most platforms; **in-process on Android** "because of unresolved issues". A crash restarts it, and clients must reconnect their URLLoaderFactories. Servicification goals: maintainability, extensibility, runtime flexibility. | — | [services/network/README.md](https://chromium.googlesource.com/chromium/src/+/HEAD/services/network/README.md); [servicification.md](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/servicification.md) | 2018–2026 | High |
| 4.3 | The out-of-process Network Service rolled out by field trial; in Chrome 72 (Feb 2019) it broke some extensions' request interception, and Google kept it, citing its long-term importance. Exact default-on milestone not verified. | — | [9to5google, Chrome 72 extensions bug](https://9to5google.com/2019/02/19/chrome-72-extensions-bug/) | 2019-02 | Medium |
| 4.4 | **The network process is still mostly unsandboxed.** Chromium's sandbox table (M128) lists Network as unsandboxed on Android, Windows and Linux; sandboxed on Mac and Fuchsia. At HEAD (Oct 2026) `kNetworkServiceSandbox` is still `FEATURE_DISABLED_BY_DEFAULT` outside Mac/Fuchsia, and on Windows also requires Windows 10 20H1 for the network-change API. | — | [process-sandboxes-by-platform.md](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/process-sandboxes-by-platform.md); [sandbox/policy/features.cc](https://chromium.googlesource.com/chromium/src/+/HEAD/sandbox/policy/features.cc) | 2024–2026 | High |
| 4.5 | **Firefox Necko:** under e10s, networking stayed in the parent process with IPC channels to content. The meta bug "Move all TCP/UDP network operations into a dedicated process" (opened 2016-12-07) is still NEW in Aug 2026; the socket process shipped only for WebRTC networking (meta 1509472 FIXED). | 10 years open | [Bugzilla 1322426](https://bugzilla.mozilla.org/show_bug.cgi?id=1322426); [Bugzilla 1509472](https://bugzilla.mozilla.org/show_bug.cgi?id=1509472) | 2016–2026 | High (status); Medium (e10s detail) |
| 4.6 | **Apple Network.framework** (WWDC 2018): URLSession is built on it; it moves the transport stack into the app process ("avoiding the socket layer entirely") with packets in a memory-mapped region, no copy, fewer context switches; 30% less receive-side overhead in one UDP demo. Connection establishment races resolved addresses and interfaces in parallel. | 30% (single demo) | [WWDC18 session 715](https://developer.apple.com/videos/play/wwdc2018/715/) | 2018-06 | High |
| 4.7 | **Servo** net uses hyper (HTTP/1, HTTP/2), hyper-rustls, tokio and rustls with aws-lc-rs: an assembled stack. | — | [servo components/net/Cargo.toml](https://github.com/servo/servo/blob/main/components/net/Cargo.toml) | 2026-10 | High |
| 4.8 | **Ladybird RequestServer** replaced its "home-grown implementations inherited from SerenityOS" with libcurl (2024-09-05); WebSockets followed (2025-02). Through 2026 the commit log shows the cost of a non-browser HTTP library behind Fetch: restricting ports Fetch allows, rejecting forbidden methods, stopping curl sending URL credentials, keeping only final-response headers, not following redirects for AIA fetches, routing preconnect through its own DNS to avoid curl's resolver thread blocking the main thread, and a Linux sandbox added June 2026. | ~10 hardening commits Sep 2026 | [Ladybird commit e483bb70f0](https://github.com/LadybirdBrowser/ladybird/commit/e483bb70f0); [03d1b37354](https://github.com/LadybirdBrowser/ladybird/commit/03d1b37354); [ea1ecdad5a](https://github.com/LadybirdBrowser/ladybird/commit/ea1ecdad5a); [88c475c0cc](https://github.com/LadybirdBrowser/ladybird/commit/88c475c0cc) | 2024–2026 | High |
| 4.9 | Internet Explorer used the OS library WinInet, shared with other Windows apps (cache and cookies were OS-wide). | — | (recollection) | 1995–2015 | Low |

**Implications:** Every major engine converged on "network in its own process", yet Chromium's is still unsandboxed on most platforms and Firefox's general socket process never finished: retrofitting a sandbox onto a stack that grew up in the browser process is the hard part (OS network-change APIs, resolver config, cert stores, proxy autoconfig). Izcal should design the network service sandboxed from day one, with brokered access to resolver/cert/proxy configuration, and with the browser-semantic checks (Fetch port blocking, CORS, credentials) inside it rather than layered over a generic client, which is Ladybird's 2026 lesson.

## 5. Rust and user-space protocol stacks; sans-IO; rustls

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **Sans-IO** rationale: protocol libraries that only consume and produce bytes can be reused "regardless of their I/O decisions", in sync and async code, with the user driving I/O. Examples include h11 (HTTP/1.1), hyper-h2, wsproto, aioquic. Arose with Python's move to async/await (Cory Benfield, PyCon US 2016). | — | [sans-io.readthedocs.io](https://sans-io.readthedocs.io/) | 2016 | High |
| 5.2 | Repository creation dates (GitHub): quinn 2018-04-03, mvfst 2018-04-09 (C++), quiche 2018-09-29, neqo 2019-02-18 (Firefox's QUIC/HTTP/3), msquic 2019-10-26 (C), s2n-quic 2020-06-25; all active in Oct 2026. quinn separates a sans-IO core (quinn-proto) from its async layer. | — | [GitHub API: quinn-rs/quinn](https://github.com/quinn-rs/quinn), [mozilla/neqo](https://github.com/mozilla/neqo), [cloudflare/quiche](https://github.com/cloudflare/quiche), [aws/s2n-quic](https://github.com/aws/s2n-quic), [microsoft/msquic](https://github.com/microsoft/msquic), [facebook/mvfst](https://github.com/facebook/mvfst) | 2018–2026 | High (dates); Medium (quinn-proto split) |
| 5.3 | **rustls** created 2016-05-02. Cure53 audited rustls, ring and webpki in May–June 2020 (CNCF-funded for linkerd): four findings, two Low, two informational. | 4 findings | [Cure53 report](https://cure53.de/pentest-report_rustls.pdf); [jbp.io on the audit](https://jbp.io/2020/06/14/rustls-audit/) | 2020-06 | Medium |
| 5.4 | **rustls 0.23** (2024-02-29) changed the default provider to aws-lc-rs (with FIPS mode; costs: cmake, nasm on Windows), added process-wide `CryptoProvider` selection, and a new **unbuffered, event-driven API** that does not buffer internally and is required for no_std. | — | [rustls v/0.23.0 release notes](https://github.com/rustls/rustls/releases/tag/v%2F0.23.0) | 2024-02-29 | High |

**Implications:** The sans-IO split (pure state machine plus thin I/O driver) is now the standard way to write protocol code that is testable and reusable, and rustls itself moved further that way in 0.23. Izcal's HTTP/1.1, HTTP/2, QUIC and HTTP/3 codecs should be sans-IO state machines driven by the network service's own event loop, which also makes deterministic replay and fuzzing natural. rustls's pluggable `CryptoProvider` lets Izcal choose the provider explicitly rather than inheriting a default.

## 6. Failed or abandoned network features

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **TLS False Start** (~30% faster handshakes) was restricted to NPN-advertising sites in Chrome 20 after 18 months on a blacklist: a small number of servers, nearly all SSL terminators, failed, some nondeterministically; vendors did not tell customers about fixes, one refused to fix. | ~30% | [Langley, "False Start's Failure"](https://www.imperialviolet.org/2012/04/11/falsestart.html) | 2012-04-11 | High |
| 6.2 | **TCP Fast Open** in browsers: middlebox interference and slow OS upgrades (3.3). Edge disabled it over firewalls dropping SYNs with data; Firefox shipped it off by default and removed it (reported Firefox 87). The TFO cookie is system-wide state usable for tracking. | — | [SIGCOMM 2017 §8](https://dl.acm.org/doi/10.1145/3098822.3098842); [Wikipedia: TCP Fast Open](https://en.wikipedia.org/wiki/TCP_Fast_Open) | 2016–2021 | Medium (paper), Low (Firefox version) |
| 6.3 | **SDCH** (shared dictionary compression): Intent to Unship by Ryan Sleevi. Reasons: no other browser adopted it, IPR concerns blocked standardization, security/privacy skepticism at IETF 97 whose mitigations would erase the gains, and //net concerns on robustness, spec compliance, staffing and integration with Service Workers, Fetch and the same-origin policy. Removed around Chrome 59. (Shared-dictionary compression later returned in Chrome as "Compression Dictionary Transport", with CORS and partitioning built in; not checked this session.) | — | [blink-dev: Intent to Unship SDCH](https://groups.google.com/a/chromium.org/g/blink-dev/c/nQl0ORHy7sw/m/HNpR96sqAgAJ) | 2016-11-22 | Medium |
| 6.4 | **HPKP** (RFC 7469) removed: in 2016 only 375 of the Alexa top million used it (76 report-only); spurious pin errors cause error fatigue; operators struggle to choose pins that work across trust stores, risking self-inflicted outages; **hostile pinning** by an attacker with a misissued cert. Replacement offered: Expect-CT. Dynamic pins targeted for Chrome 67. | 375 / 1M sites | [blink-dev: Intent to Deprecate and Remove Public Key Pinning](https://groups.google.com/a/chromium.org/g/blink-dev/c/he9tr7p3rZ8/m/eNMwKPmUBAAJ) | 2017-10-27 | High |
| 6.5 | **Expect-CT** itself became pointless once Chrome required CT for all publicly trusted certificates, and was removed from Chrome (believed Chrome 107). | — | [MDN: Expect-CT](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Expect-CT) | 2022 | Low |
| 6.6 | **OCSP and Must-Staple:** Let's Encrypt ended OCSP. Reasons: OCSP "represents a considerable risk to privacy" (the responder learns which site an IP visits, and CAs "could be legally compelled" to log it); cost and complexity; CRLs (supported since 2022) make it redundant; Must-Staple "has failed to get wide browser support" and server stapling implementations "create serious risks of downtime." Dates: Must-Staple requests fail 2025-01-30; OCSP URLs dropped 2025-05-07; responders off 2025-08-06. | — | [Let's Encrypt, "Ending OCSP Support in 2025"](https://letsencrypt.org/2024/12/05/ending-ocsp/) | 2024-12-05 | High |
| 6.7 | **NPN → ALPN:** NPN (Google, for SPDY) negotiated the protocol inside the encrypted handshake; ALPN moved it to ClientHello as a standards-track extension; Chrome dropped NPN with SPDY in Chrome 51. | — | [Chromium blog 2015-02](https://blog.chromium.org/2015/02/hello-http2-goodbye-spdy.html) | 2016-05 | High |
| 6.8 | **AppCache:** deprecated in secure contexts (Chrome 79), off by default in Chrome 85, reverse origin trial to 2021-10-05 (~Chrome 95), then removed; Service Workers replace it. Firefox deprecated it in Firefox 44. | — | [web.dev, "Preparing for AppCache removal"](https://web.dev/articles/appcache-removal) | 2020–2021 | Medium |
| 6.9 | **Do Not Track → GPC:** Firefox 135 removed the DNT checkbox: "Many sites do not respect this indication... and, in some cases, it can reduce privacy" (fingerprinting bit). GPC is offered instead because it is "enforced with legislation in some regions" (e.g. CCPA). | — | [Mozilla support, via coverage](https://mezha.ua/en/2024/12/12/mozilla-removes-do-not-track-feature-from-firefox-because-it-doesn-t-work/) | 2024-12 / 2025-02 | Medium |
| 6.10 | **Privacy Sandbox:** 2025-04-22 Google keeps its "current approach to offering users third-party cookie choice" with no standalone prompt, citing divergent feedback and a changed regulatory landscape. 2025-10-17 it retires Attribution Reporting, IP Protection, Protected Audience, Topics, Related Website Sets (incl. requestStorageAccessFor), Shared Storage/Private Aggregation, SelectURL and others for "low levels of adoption"; keeps CHIPS, FedCM, Private State Tokens; attribution moves to the W3C PATCG. | ~10 APIs retired | [Privacy Sandbox, "Next steps"](https://privacysandbox.google.com/blog/privacy-sandbox-next-steps); [Privacy Sandbox, "Update on plans"](https://privacysandbox.google.com/blog/update-on-plans-for-privacy-sandbox-technologies) | 2025-04-22; 2025-10-17 | High |
| 6.11 | Cookie2/Set-Cookie2 (RFC 2965) never gained browser support and was obsoleted by RFC 6265; P3P (IE's machine-readable policy headers) was widely gamed with meaningless compact policies and dropped from Edge; Signed Exchanges/Web Packaging remains Chromium-only (Mozilla and WebKit did not adopt). | — | [RFC 6265 (Obsoletes 2965)](https://www.rfc-editor.org/rfc/rfc6265.html) | 2011; 2015; 2019– | Medium (Cookie2), Low (P3P, SXG) |

**Implications:** The pattern behind most removals: features whose correctness depended on every middlebox or server operator behaving (pipelining, False Start, TFO, Must-Staple, HPKP) failed, and features that exposed per-user state across sites (SDCH, TFO cookies, OCSP queries, DNT bit) were pulled for privacy. Izcal should prefer mechanisms that are end-to-end and encrypted, fail safe when the other side misbehaves, and are keyed by the (top-level site, container) partition from the start. Revocation should come from CRL-derived pushed sets, not live OCSP.

## 7. Partitioning, encrypted DNS and ECH

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **Safari** partitioned the cache by top-level eTLD+1 (generally dated 2013); Chrome's 2020 article confirms Safari's key but gives no date. | — | [developer.chrome.com, HTTP cache partitioning](https://developer.chrome.com/blog/http-cache-partitioning) | 2013 | Low (date) |
| 7.2 | **Chrome 86** partitioned the HTTP cache with a Network Isolation Key of (top-level site, current-frame site). Measured cost: cache miss rate +~3.6%, bytes from network +~4%, FCP ~0.3%. | 3.6% / 4% / 0.3% | [developer.chrome.com, HTTP cache partitioning](https://developer.chrome.com/blog/http-cache-partitioning) | 2020-10 | High |
| 7.3 | **Firefox 85** partitioned by top-level site: HTTP, image, favicon, HSTS, OCSP, style sheet, font, DNS, HTTP auth, Alt-Svc and TLS certificate caches, plus pooled, prefetch, preconnect and speculative connections and TLS session IDs. Page load cost 0.09–0.75% at ≤p80, max 1.32% at p85. | ≤1.32% | [Mozilla Security blog, "Firefox 85 Cracks Down on Supercookies"](https://blog.mozilla.org/security/2021/01/26/supercookie-protections/) | 2021-01-26 | High |
| 7.4 | **Firefox DoH** default for US users from 2020-02-25 (Cloudflare default, NextDNS option), under Mozilla's Trusted Recursive Resolver policy. | — | [Mozilla blog](https://blog.mozilla.org/en/products/firefox/firefox-continues-push-to-bring-dns-over-https-by-default-for-us-users/) | 2020-02-25 | High |
| 7.5 | **Chrome 83 Secure DNS:** auto-upgrade to DoH only if the user's current resolver supports it (same provider), with manual choice; disabled in managed (enterprise-policy) environments. | — | [Chromium blog, Secure DNS](https://blog.chromium.org/2020/05/a-safer-and-more-private-browsing-DoH.html) | 2020-05-19 | Medium |
| 7.6 | **ECH:** grew from ESNI (encrypting only SNI) into encrypting the whole ClientHello; Firefox rolled it out worldwide on 2023-10-03 (Firefox 118); Chrome shipped in 117 after 1% (115) and 10% (from 2023-08-31) stable experiments; use depends on server opt-in via DNS HTTPS records. Published as **RFC 9849** (Proposed Standard) in March 2026. | — | [Mozilla blog, ECH](https://blog.mozilla.org/en/products/firefox/encrypted-hello/); [blink-dev Intent to Ship ECH](https://groups.google.com/a/chromium.org/g/blink-dev/c/CmlXjQeNWDI/m/hx-_4lNBAQAJ); [RFC 9849](https://www.rfc-editor.org/rfc/rfc9849.html) | 2023-10; 2026-03 | High (RFC, Firefox date); Medium (Chrome milestone) |

**Implications:** Partitioning costs a few percent of cache hits and well under 2% of load time in two independent measurements; it is cheap enough to be the only mode. Firefox's list is the right scope: every cache and every pooled connection or session ticket, not just the HTTP cache. Izcal's partition key should be the (top-level site, container) tuple applied uniformly to DNS, Alt-Svc, HSTS, TLS resumption and connection pools. ECH depends on HTTPS-record resolution, so DoH/DoT and ECH should be designed together.

## 8. Fetch and cookie standards

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | XMLHttpRequest began as Microsoft's XMLHTTP ActiveX (1999); CORS was specified at W3C (Recommendation January 2014); the WHATWG Fetch standard (from 2014) unified request semantics (CORS, redirects, credentials, cache mode, service-worker interception) that XHR, `<img>`, navigation and `fetch()` had defined inconsistently. | — | [Fetch Standard](https://fetch.spec.whatwg.org/) | 1999–2014 | Low |
| 8.2 | Cookies: Netscape informal spec (1994), RFC 2109 (1997), RFC 2965 with Cookie2 (2000, unused), RFC 6265 (April 2011) documented what browsers actually did. **6265bis** (SameSite, prefixes, cookie-name rules) is at draft -22 (2025-12-01), WG state "Submitted to IESG for Publication", IESG state **RFC Ed Queue** as of 2026-10-19, so not yet an RFC. | draft -22 | [IETF Datatracker: draft-ietf-httpbis-rfc6265bis](https://datatracker.ietf.org/doc/draft-ietf-httpbis-rfc6265bis/) | 2026-10-19 | High |

**Implications:** Cookie and fetch standards both succeeded only once they described deployed behaviour (RFC 6265, Fetch) rather than an ideal design (RFC 2965). Izcal should anchor its cookie store to 6265bis-22 now, expect the RFC number soon, and implement Fetch's algorithms as the single definition of request semantics inside the network service.

## Lessons for Izcal

1. Do not depend on intermediaries behaving: pipelining, False Start, TFO and Must-Staple all died on middleboxes or server operators. Encrypt and authenticate what you can, and always keep a fallback (TCP for QUIC).
2. Signals must flow from whoever has the information: push failed because the server did not know the client's cache; the RFC 7540 tree failed because servers ignored it. Use client-decided mechanisms (Early Hints, RFC 9218).
3. Ship HTTP/1.1, HTTP/2 and HTTP/3 as equal citizens; in 2025 HTTP/1.x still carried 29% of Cloudflare requests and 4–5% of clients could not use UDP.
4. Budget CPU for user-space QUIC (Google started at 3.5× TCP and reached 2×); design for batched I/O and cheap per-packet state from the start.
5. Sandbox the network process from day one: Chromium's is still unsandboxed on Windows, Linux and Android in 2026, and Firefox's general socket process has been open for ten years.
6. Put browser semantics (Fetch rules, ports, credentials, CORS) inside the network service and drive sans-IO codecs from it; Ladybird's 2026 curl hardening shows the cost of a generic client.
7. Partition everything by (top-level site, container), including DNS, Alt-Svc, HSTS, TLS session state and connection pools; measured cost is a few percent of cache hits.
8. Prefer privacy-preserving, push-delivered revocation over live OCSP; plan ECH and encrypted DNS as one feature.
9. Treat any feature that adds cross-site state or a fingerprintable bit (SDCH, TFO cookies, DNT) as a privacy design problem before a performance one.

## Claims not verified this session

- Opera Presto's default-on pipelining (1.5), IE/WinInet details (4.9), Firefox push and TFO removal versions (2.6, 6.2), Expect-CT removal version (6.5), P3P and Signed Exchanges status (6.11), Safari 2013 date (7.1), XHR/CORS/Fetch dates (8.1).
- Exact Chrome milestone when the out-of-process Network Service became default on desktop (4.3).
- The exact Chrome milestone for SDCH removal (6.3).
