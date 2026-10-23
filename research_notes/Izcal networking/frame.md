# Izcal networking: frame

Compiled 19 October 2026 by the synthesis author (phase 1 of the first-principles-spec method). Sources are Izcal's own specifications; IDs are quoted from them. No web research in this file.

## 1. The problem and who has it

| User | Task, in their words | What the network stack must give them |
|---|---|---|
| People browsing | "Pages load fast on any network, my sessions stay mine, and nobody can follow me from site to site" | Fast, correct HTTP/1.1, HTTP/2 and HTTP/3; per-(site, container) partitioning of every piece of network state; no wakeups for hidden pages |
| People on hard networks | "It works behind the hotel portal, the corporate proxy, the VPN, on IPv6-only mobile and on a bad train connection" | Proxies (PAC, system, SOCKS, per-container routes), captive-portal detection, Happy Eyeballs, graceful fallback from QUIC to TCP |
| Web developers | "Why did this request fail, wait, miss the cache, get blocked by CORS, use HTTP/2 instead of 3?" | Protocol-level facts from code Izcal owns, with reasons (DT-D5, `hg-net-observe`) |
| Izcal's engineers and agents | "Reproduce this exactly, fuzz it, prove the parser never panics" | Sans-IO protocol state machines driven by the substrate (B1), recorded inputs, a simulated network with faults |
| Site operators | "Your browser should look like a browser to my CDN and bot defences, and follow the RFCs" | Standard-conformant wire behaviour that is not uniquely fingerprintable |

## 2. What Izcal's specs already decide or constrain

| Source | ID | Constraint on networking |
|---|---|---|
| HGBA-2 2.3.0 | **M-1 part 1** | Izcal builds its own HTTP/1.1, HTTP/2, HTTP/3 and QUIC; neqo is dropped; allowed dependencies: Rust std, generic crates, TLS and cryptography (rustls and its providers), OS interfaces. Owner direction for this spec extends the exclusion to hyper, h2, h3, quinn, quiche, reqwest and any library doing browser networking (DNS resolvers such as hickory-dns included) |
| HGBA-2 2.3.0 | **M-1 parts 2–4** | Every algorithm carries `// spec:` anchors at a pinned revision; other stacks are black-box differential signals only |
| HGBA-2 | **P1–P6** | Security and privacy first, then compatibility, robustness, performance |
| HGBA-2 | **I-1 (b)** | The network service partitions **all** its state by (top-level site, container), extending Fetch's network partition key; enumerated minimum: HTTP cache, connection and socket pools, DNS cache, Alt-Svc, TLS session tickets, HSTS, revocation caches, reporting endpoints, service-worker registrations |
| HGBA-2 | **I-1 (c)** + footprint **FP-D8** | Origin keying of processes always on for desktop classes |
| HGBA-2 | **I-2** | Renderers have no sockets; all networking via the network service |
| HGBA-2 | **I-6** | SAB and high-resolution timers gated on COOP+COEP |
| HGBA-2 | **4.6** | Persistent storage uses crash-consistent stores (SQLite WAL), never custom persistence layers; io_uring excluded from renderer and network-service designs |
| HGBA-2 | **4.7** (neqo superseded) | No kernel bypass; CPU crypto and UDP GSO/GRO inside the service; X25519MLKEM768 hybrid by default; crypto agility; network-service sandbox everywhere as a Stage 2 target (tightened by SEC-D13) |
| HGBA-3 | **B1, DET-1, DET-2** | No engine code touches sockets, clocks, threads or randomness except through the substrate; recorded network bytes replay bit-identically; ≤1%† substrate cost |
| HGBA-3 | **B2** | Typed capability IPC with a simulation transport |
| HGBA-3 | **B9, B10, AG-0** | Agents are a distinct principal; AI off means zero AI network requests |
| Stage 1 | **Section 1** | One network service process; storage service holds cookies at rest (SQLite WAL) |
| Stage 1 | **3.1** | `NetTransport` trait: only the network service holds one |
| Stage 1 | **3.2** | Simulation replays a recorded corpus archive (own format, WARC importer) with network faults |
| Stage 1 | **4.1** | Sync IPC allowlist includes `document.cookie` and sync XHR |
| Stage 1 | **G3, G7, G9** | Only `hg-substrate-os` names `std::net`; renderer crates never depend on `hg-net`; `hg-net` carries spec anchors |
| Stage 1 | **6** | CHIPS-style partitioned third-party cookies by default; GPC sent by default; per-container user-supplied routes (WireGuard, SOCKS, Arti) labelled honestly; content blocking in the network service; no raw TCP/UDP sockets for web content |
| Stage 1 | **8.1 M5** | Network service with HTTP/2 and 3, cookies, CORS, CSP |
| Stage 1 | **8.3** | Network and security track: 2 engineers (pre-2.3.0, assumed neqo, hickory-dns) |
| Stage 1 | **13.2** | Chrome-compatible UA with own token; certificates: platform verifier primary, Mozilla roots fallback, CT enforced in Stage 1, CRLs pushed via update channel then CRLite-style filters |
| Security scope 0.3 | **NET-R1–R7, SEC-28, SEC-31, SEC-D13, SEC-34, IPC-R1–R17, Q-SEC-5** | Context from the endpoint; ORB before delivery; no panic reachable from network input; differential fuzzing; LNA and DNS rebinding; network sandbox required on Linux and macOS by M8; cookies at rest under an OS-bound key |
| Coding standard 0.3.1 | **Tier P, CS-D1 (a), CS-AS-1, CS-AS-4, CS-D8, CS-ER-9, CS-ER-13, CS-T11, Q-CS-2** | Abort on panic; no ambient effects; no tokio; async only on an allowlist; fallible capped allocations; every network failure is the same Fetch network error; own sans-IO HTTP/2 codec (Q-CS-2 decided) |
| JS engine spec 0.3 | **JS-D15** | Timing-dependent decisions are recorded B1 inputs; production never waits for determinism |
| Footprint spec 0.3 | **FP-1, FP-17, FP-18, FP-26, FP-D9, FP-D16, Section 6.18.1** | Network service ≤ 8 MiB† idle; hidden pages 0 wakeups; ≤ 1 wakeup/s† whole browser for a static page; HTTP cache 256 MiB† visible budget; telemetry opt-in only; regenerable-cache class; shared-resource channels closed or bounded |
| DevTools spec 0.3 | **DT-D5, DT-18, `hg-net-observe`, `hg-export`** | Typed network events (DNS, connection decisions, TLS, QUIC as qlog); live driving only with write capability; what-if faults in simulation only; views must be true |

## 3. Success, in measurable terms

1. Conformance: every protocol passes its oracle suites (WPT fetch/cors/cookies/websockets/webtransport, h2spec-class and QUIC interop-class suites, Izcal's RFC-derived state-machine oracles) at thresholds set in the spec.
2. Security: zero panics reachable from network input; every pre-authentication allocation capped; every protocol DoS class from the public record closed by a named limit.
3. Privacy: no cross-partition observable difference on any listed network state axis (a test per axis).
4. Performance: page-load and goodput parity with the best shipping browser on the lab corpus per protocol; CPU per byte on QUIC competitive; idle wakeups within FP-18.
5. Determinism: recorded network input replays bit-identically (DET-1); the protocol stack runs unchanged under simulation.
6. Observability: every connection decision explainable with a reason, matching what the stack did (DT-18).
