# Izcal networking: protocols, standards and conformance

Compiled 19 October 2026. Primary sources were preferred: RFC text from rfc-editor.org, IETF Datatracker pages, WHATWG and W3C specification sources, crate documentation on docs.rs, and test suites' published descriptions. *(2.3.3: no third-party project's source code is evidence; row 3.7, which 0.3 read from a test tool's source file, now cites the tool's documentation.)*
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, a Datatracker summary, or a vendor claim without data; **Low** = secondary source or the researcher's prior knowledge, not re-checked on 19 October 2026.

**Method for size numbers.** The RFC plain-text files were downloaded on 19 October 2026 and measured by this researcher. "Lines" and "words" are `wc` counts of the `.txt` file. A printed RFC page is about 58 lines, so pages ≈ lines / 58. **MUST** counts every upper-case `MUST` or `MUST NOT` token, and **SHOULD** counts `SHOULD` or `SHOULD NOT`. They include repeats in examples and IANA text, so they are an order-of-magnitude signal, not a requirement inventory. RFC 1034/1035 predate RFC 2119 and have no key words. WPT counts are test *files* in a blob-less clone of `web-platform-tests/wpt` at commit `4ad507b` (19 October 2026), with `resources/` and `support/` excluded. One file can hold hundreds of subtests, so subtest totals are much larger.

**Pinning.** For `// spec:` anchors: RFCs are immutable, so the RFC number plus section is a pin, and errata must be tracked separately. WHATWG Fetch was at commit `59afe34` (19 October 2026) and WHATWG WebSockets at `7f25d7d` (19 October 2026). Drafts must be pinned by draft number (for example `draft-ietf-webtrans-http3-16`).

## 0. Size table (measured)

| RFC | Title | Date | Lines | Words | MUST | SHOULD |
|---|---|---|---|---|---|---|
| 9110 | HTTP Semantics | 2022-06 | 10,785 | 68,887 | 206 | 125 |
| 9111 | HTTP Caching | 2022-06 | 1,956 | 11,838 | 66 | 20 |
| 9112 | HTTP/1.1 | 2022-06 | 2,461 | 15,015 | 87 | 31 |
| 9651 | Structured Field Values | 2024-09 | 1,996 | 10,220 | 26 | 5 |
| 8297 | 103 Early Hints | 2017-12 | 395 | 1,914 | 4 | 3 |
| 9931 | Optimistic Protocol Transitions in HTTP/1.1 | 2026-03 | 532 | 2,940 | 7 | 2 |
| 9842 | Compression Dictionary Transport | 2025-09 | 878 | 4,395 | 26 | 2 |
| 9113 | HTTP/2 | 2022-06 | 4,188 | 26,185 | 225 | 31 |
| 7541 | HPACK | 2015-05 | 3,083 | 13,013 | 22 | 2 |
| 8441 | Bootstrapping WebSockets with HTTP/2 | 2018-09 | 451 | 2,300 | 10 | 2 |
| 9218 | Extensible Priorities | 2022-06 | 1,149 | 7,361 | 26 | 17 |
| 8999 | QUIC invariants | 2021-05 | 441 | 2,513 | 7 | 2 |
| 9000 | QUIC transport | 2021-05 | 8,485 | 57,682 | 304 | 123 |
| 9001 | TLS for QUIC | 2021-05 | 2,756 | 17,166 | 85 | 21 |
| 9002 | QUIC loss detection and CC | 2021-05 | 2,070 | 11,686 | 35 | 27 |
| 9221 | QUIC DATAGRAM | 2022-03 | 435 | 2,528 | 13 | 6 |
| 9287 | Greasing the QUIC bit | 2022-08 | 260 | 1,479 | 9 | 3 |
| 9368 | Compatible version negotiation | 2023-05 | 813 | 5,641 | 35 | 8 |
| 9369 | QUIC v2 | 2023-05 | 650 | 3,094 | 14 | 5 |
| 9312 | QUIC manageability (Informational) | 2022-09 | 1,601 | 10,794 | 0 | 0 |
| 9114 | HTTP/3 | 2022-06 | 3,234 | 21,470 | 154 | 63 |
| 9204 | QPACK | 2022-06 | 2,133 | 12,169 | 41 | 5 |
| 9297 | HTTP Datagrams and Capsules | 2022-08 | 717 | 4,356 | 32 | 13 |
| 9220 | Bootstrapping WebSockets with HTTP/3 | 2022-06 | 168 | 821 | 2 | 3 |
| 9298 | Proxying UDP in HTTP (CONNECT-UDP) | 2022-08 | 819 | 4,992 | 37 | 14 |
| 8446 | TLS 1.3 (obsoleted) | 2018-08 | 8,963 | 40,349 | 330 | 76 |
| 9846 | TLS 1.3 (8446bis) | 2026-07 | 7,397 | 42,199 | 345 | 78 |
| 8470 | Using Early Data in HTTP (425) | 2018-09 | 675 | 3,882 | 22 | 6 |
| 7301 | ALPN | 2014-07 | 507 | 2,160 | 5 | 5 |
| 8879 | TLS certificate compression | 2020-12 | 359 | 1,816 | 14 | 3 |
| 9849 | TLS Encrypted Client Hello | 2026-03 | 2,361 | 14,560 | 75 | 40 |
| 9848 | Bootstrapping ECH with DNS | 2026-03 | 427 | 2,318 | 7 | 5 |
| 1034 / 1035 | DNS concepts / implementation | 1987-11 | 3,077 / 3,077 | 17,163 / 15,901 | — | — |
| 6891 | EDNS(0) | 2013-04 | 899 | 4,929 | 31 | 15 |
| 7858 | DNS over TLS | 2016-05 | 1,067 | 5,715 | 21 | 15 |
| 8484 | DNS over HTTPS | 2018-10 | 1,179 | 6,778 | 18 | 7 |
| 9250 | DNS over QUIC | 2022-05 | 1,450 | 8,930 | 35 | 38 |
| 9460 | SVCB and HTTPS RRs | 2023-11 | 2,485 | 14,559 | 57 | 54 |
| 8305 | Happy Eyeballs v2 | 2017-12 | 843 | 4,834 | 7 | 24 |
| 6265 | Cookies (current RFC) | 2011-04 | 2,075 | 10,474 | 28 | 30 |
| draft-ietf-httpbis-rfc6265bis-22 | Cookies (bis) | 2025-12-01 | 3,472 | — | 47 | — |
| 6455 | WebSocket protocol | 2011-12 | 3,979 | 23,044 | 140 | 30 |
| 7692 | permessage-deflate | 2015-12 | 1,571 | 8,338 | 43 | 3 |
| draft-ietf-webtrans-http3-16 | WebTransport over HTTP/3 | 2026-07-06 | 2,240 | — | 76 | — |
| draft-ietf-webtrans-http2-15 | WebTransport over HTTP/2 | 2026-07-06 | 2,352 | — | 72 | — |
| 1928 | SOCKS v5 | 1996-03 | 507 | 2,550 | 15 | 1 |
| Fetch (WHATWG `fetch.bs`) | Living Standard | 2026-10-19 | 10,595 | — | 136 `<div algorithm>` blocks, ~1,362 list steps, 431 `<dfn>` | — |

All rows are **High** (computed from the primary files listed). **Total RFC surface in scope (the 43 RFC files above, excluding the obsoleted RFC 8446 and the drafts): 86,391 lines, roughly 1,490 printed pages, and 2,336 MUST tokens.** Of this, 9110 + 9000 + 9846 + 9113 + 9114 + 9001 make up about half.

## 1. HTTP semantics, caching and HTTP/1.1

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **RFC 9110 (HTTP Semantics)**, 9111 (Caching) and 9112 (HTTP/1.1) are Internet Standards (STD 97, 98, 99). They obsolete 7230–7235. 9110 holds everything version-independent: methods, status codes, header fields, content negotiation, range requests, conditional requests, authentication framework, and message abstraction ("control data, headers, content, trailers"). Clients must implement: redirect handling (with the 301/302 POST→GET legacy rule), conditional requests (`If-None-Match`, `If-Modified-Since`) for cache revalidation, `Range`/`206` (media), `Vary`, the `Authorization`/`WWW-Authenticate` challenge grammar, and field-value validation (no CR, LF or NUL). | 206 MUSTs in 9110 | [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110) | 2022-06 | High |
| 1.2 | **RFC 9111 (Caching)** obligates a private cache: freshness from `max-age`/`Expires`/heuristics, the `Age` calculation (§4.2.3), `Vary` secondary keys, validation, invalidation on unsafe methods (§4.4), `no-store`/`no-cache`/`private`, and storing only what the rules allow. Browser additions on top: `stale-while-revalidate` (RFC 5861), `immutable` (RFC 8246), cache partitioning (Fetch), **Cache Groups (RFC 9875, 2025-10)** and **No-Vary-Search** (draft-ietf-httpbis-no-vary-search-10, in the RFC Editor queue). The WPT suite `fetch/http-cache` is the browser-side oracle (17 test files by row 7.6's method, which excludes resources; this row's earlier count of 25 included non-test files). | 66 MUSTs | [RFC 9111](https://www.rfc-editor.org/rfc/rfc9111); [httpbis documents](https://datatracker.ietf.org/wg/httpbis/documents/) | 2022-06; 2026-10-19 | High (9111); Medium (9875, NVS status) |
| 1.3 | **Request smuggling surface (RFC 9112 §6.3, message body length).** Eight ordered rules. A response to HEAD, 1xx, 204, 304 has no body; a 2xx to CONNECT switches to tunnel and "a client MUST ignore any Content-Length or Transfer-Encoding"; `Transfer-Encoding` overrides `Content-Length`, and a message with both "ought to be handled as an error"; if chunked is not final in a response, read until close; an invalid `Content-Length` in a response received by a user agent MUST be treated as an unrecoverable error (close connection, discard). Also: `obs-fold` must be rejected or replaced by SP; bare CR must be rejected or replaced; whitespace between field name and colon must be rejected (§5.1). A client is the *last* hop, so smuggling risk is lower than for a proxy, but response-splitting and pooled-connection desync still matter (a desynced keep-alive connection serves one origin's bytes as another response). | 8 rules; 87 MUSTs | [RFC 9112 §6.3](https://www.rfc-editor.org/rfc/rfc9112#section-6.3) | 2022-06 | High |
| 1.4 | **RFC 9931 (March 2026)** adds requirements to 9112 and 9298 for data sent *optimistically* after an `Upgrade` or CONNECT request but before the 101/2xx. This is directly relevant to WebSocket-over-HTTP/1.1 and CONNECT proxies: Izcal must not pipeline bytes behind an upgrade request. | 7 MUSTs | [RFC 9931](https://www.rfc-editor.org/rfc/rfc9931) | 2026-03 | High (abstract); Medium (details) |
| 1.5 | **Structured Fields, RFC 9651** (obsoletes 8941, adds Date and Display String types). It defines parse and serialize algorithms step by step, so it is anchorable at step level. Used by `Priority`, `Sec-Fetch-*`, `Accept-CH`, `Cache-Status`, `Reporting-Endpoints`, `Use-As-Dictionary`, `Permissions-Policy` and others. Conformance oracle: **httpwg/structured-field-tests** (JSON fixtures, parse and serialize; Low on exact count, about 1,500 cases from memory). "Retrofit Structured Fields" expired in 2023. | 26 MUSTs | [RFC 9651](https://www.rfc-editor.org/rfc/rfc9651) | 2024-09 | High (RFC); Low (test count) |
| 1.6 | **103 Early Hints, RFC 8297** (Experimental). The client must accept multiple 1xx before the final response and must not treat Early Hints headers as part of the final response. Browser behavior (preload/preconnect only, navigation only, CSP/`Link` processing) is specified in **HTML** ("process early hint headers"), not in the RFC. | 4 MUSTs | [RFC 8297](https://www.rfc-editor.org/rfc/rfc8297) | 2017-12 | High |
| 1.7 | **Other HTTP RFCs a browser now touches:** **RFC 9842 Compression Dictionary Transport** (2025-09; `Use-As-Dictionary`, `Available-Dictionary`, `Dictionary-ID`, `dcb`/`dcz` content codings; WPT `fetch/compression-dictionary`, 88 files incl. resources); **RFC 10008 QUERY method** (2026-06); **RFC 10036 Incremental Forwarding** (2026-08). In flight: Resumable Uploads (-14, 2026-10-18), connect-tcp (-16, IESG telechat 2026-10-22), Unencoded Digest (RFC Ed queue), Secondary Certificates (WGLC), and **layered cookies** (§7.6). Alt-Svc is still RFC 7838; its revision (rfc7838bis) is parked. | — | [httpbis documents](https://datatracker.ietf.org/wg/httpbis/documents/); [RFC 9842](https://www.rfc-editor.org/rfc/rfc9842) | 2026-10-19 | High (9842); Medium (others, Datatracker listing) |
| 1.8 | **Conformance for HTTP/1.1 clients is thin.** There is no client-side RFC 9112 suite. WPT `fetch/h1-parsing` (115 files, mostly resources driven by `wptserve` raw responses) checks status-line and header parsing at the Fetch layer. Research suites (HTTP Garden; "T-Reqs"; smuggling fuzzers) target servers and proxies. **Cache tests:** `httpwg/cache-tests` (cache-tests.fyi) covers RFC 9111 behaviour across browsers and CDNs. | 115 files | WPT tree `4ad507b` | 2026-10-19 | High (WPT); Medium (cache-tests.fyi, from prior knowledge) |

**Implications:** 9110 is the largest single text and is mostly declarative, not algorithmic. Anchors will be at section granularity (`#section-15.4.4`), unlike Fetch's step-level anchors. Izcal should write the 9112 parser as a strict state machine with every "MAY accept leniently" clause resolved to *reject* by default, and build its own response-side smuggling corpus.

## 2. HTTP/2

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **RFC 9113** obsoletes 7540 and 8740. Changes that matter for a client: the RFC 7540 priority tree is **deprecated** (`SETTINGS_NO_RFC7540_PRIORITIES` from 9218); stricter field validation (no uppercase names, no connection-specific fields, no leading/trailing whitespace in values, `:authority` rules); h2c `Upgrade` removed. Surface: **10 frame types** (DATA, HEADERS, PRIORITY, RST_STREAM, SETTINGS, PUSH_PROMISE, PING, GOAWAY, WINDOW_UPDATE, CONTINUATION), **6 settings** in 9113 plus `SETTINGS_ENABLE_CONNECT_PROTOCOL` (8441) and `SETTINGS_NO_RFC7540_PRIORITIES` (9218), **14 error codes**, a **7-state stream state machine** (idle, reserved local/remote, open, half-closed local/remote, closed), two-level flow control, and the HEADERS+CONTINUATION atomicity rule. | 225 MUSTs | [RFC 9113](https://www.rfc-editor.org/rfc/rfc9113) | 2022-06 | High (RFC, MUST count); Medium (frame/setting/error tallies from prior knowledge of the text) |
| 2.2 | **Security-driven client limits not in the RFC's MUSTs:** CONTINUATION flood (2024), Rapid Reset (CVE-2023-44487), HPACK bomb, SETTINGS/PING floods, zero-window stalls. Each needs a configured cap. | — | prior knowledge | 2023–2024 | Medium |
| 2.3 | **HPACK, RFC 7541**: a 61-entry static table, a FIFO dynamic table bounded by `SETTINGS_HEADER_TABLE_SIZE`, a canonical Huffman code (257 symbols incl. EOS), integer encoding with N-bit prefix, and 4 representation types (indexed; literal with incremental indexing; without indexing; never indexed). Decoder must enforce table-size updates only at block start. No errata changed the wire format. Oracle: **http2jp/hpack-test-case** (JSON stories from several encoders). | 22 MUSTs | [RFC 7541](https://www.rfc-editor.org/rfc/rfc7541) | 2015-05 | High (RFC); Medium (test-case repo) |
| 2.4 | **Extended CONNECT, RFC 8441**: `SETTINGS_ENABLE_CONNECT_PROTOCOL = 1` from the server lets the client send `:method CONNECT` with a `:protocol` pseudo-header. Needed for WebSockets over h2 and for WebTransport over HTTP/2. | 10 MUSTs | [RFC 8441](https://www.rfc-editor.org/rfc/rfc8441) | 2018-09 | High |
| 2.5 | **Extensible Priorities, RFC 9218**: `Priority` header (SF dictionary: urgency `u=0..7`, default 3; `i` incremental) and the `PRIORITY_UPDATE` frame for h2 (type 0x10) and h3 (0xF0700/0xF0701). It is a *signal*; servers may ignore it. Browsers must map resource type and fetch priority (`fetchpriority`, Fetch's request priority) onto it. | 26 MUSTs | [RFC 9218](https://www.rfc-editor.org/rfc/rfc9218) | 2022-06 | High (RFC); Medium (frame codes) |
| 2.6 | **h2spec tests servers only** and targets RFC 7540/7541, not 9113. Its README describes running against a server's `GET /`/`POST /`; no client mode is documented. It is therefore not an oracle for Izcal's client codec except by building an Izcal test server from the same codec (which would test the shared frame layer). | ~146 cases (Low, from memory) | [summerwind/h2spec](https://github.com/summerwind/h2spec) | 2026-10-19 (read) | High (server-only, 7540); Low (count) |

**Implications:** the h2 client has no ready-made conformance oracle. Izcal must build an adversarial h2 *server* harness (malformed frames, flow-control abuse, CONTINUATION floods, GOAWAY races) derived from 9113's MUST list, and run h2spec only against an Izcal-built server to cover the shared codec.

## 3. QUIC

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **RFC 9000** (transport): **20 frame types** (24 rows in Table 3 because STREAM, MAX_STREAMS, STREAMS_BLOCKED and CONNECTION_CLOSE occupy ranges), **17 transport parameters** (0x00–0x10), long-header packet types (Initial, 0-RTT, Handshake, Retry) plus Version Negotiation and 1-RTT short header, varint encoding, stream state machines (sending: Ready/Send/Data Sent/Data Recvd/Reset Sent/Reset Recvd; receiving: Recv/Size Known/Data Recvd/Data Read/Reset Recvd/Reset Read), connection states including closing/draining, connection ID management, path validation and migration, stateless reset, anti-amplification (3x), and address validation tokens. | 304 MUSTs; 57,682 words | [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000) | 2021-05 | High |
| 3.2 | **RFC 9001** (TLS for QUIC): CRYPTO frames carry TLS handshake bytes in 3 encryption levels; packet protection (AEAD) and header protection; Initial secrets from a version-specific salt; key update via the Key Phase bit; 0-RTT rules; `quic_transport_parameters` TLS extension; no TLS record layer, no `KeyUpdate` messages, no middlebox compatibility mode. | 85 MUSTs | [RFC 9001](https://www.rfc-editor.org/rfc/rfc9001) | 2021-05 | High |
| 3.3 | **rustls exposes exactly the RFC 9001 interface.** `rustls::quic` (rustls 0.23.46) has `ClientConnection`/`ServerConnection`/`Connection`, `Keys`, `DirectionalKeys`, `PacketKeySet`, `Secrets` (for key update), `Suite`, `KeyChange` (handshake → 1-RTT), traits `PacketKey` and `HeaderProtectionKey`, and `Version` {`V1Draft`, `V1`, `V2`} (non-exhaustive). It handles the TLS state machine and key derivation; Izcal writes everything else (packets, frames, recovery, streams). This is the same split Quinn and others use. | 9 structs, 3 enums, 3 traits | [rustls::quic](https://docs.rs/rustls/latest/rustls/quic/index.html); [Version](https://docs.rs/rustls/latest/rustls/quic/enum.Version.html) | 2026-10-19 | High (types); Medium (method names such as `read_hs`/`write_hs`/`quic_transport_parameters`/`zero_rtt_keys`, not shown on the pages read) |
| 3.4 | **RFC 9002** (recovery): packet-number spaces, RTT estimation (`smoothed_rtt`, `rttvar`, `min_rtt`), packet/time threshold loss detection (kPacketThreshold 3, time threshold 9/8), Probe Timeout, and a NewReno-style reference congestion controller with persistent-congestion detection. Pseudo-code in Appendix A/B is close to anchorable. Browsers ship CUBIC or BBR instead; 9002 permits any controller that complies with RFC 8085 guidance. | 35 MUSTs | [RFC 9002](https://www.rfc-editor.org/rfc/rfc9002) | 2021-05 | High |
| 3.5 | **Extensions published:** RFC 9221 DATAGRAM (frame types 0x30/0x31, `max_datagram_frame_size`; needed for WebTransport/MASQUE), RFC 9287 grease the QUIC bit (`grease_quic_bit`), RFC 9368 compatible version negotiation (`version_information` parameter) and RFC 9369 **QUIC v2** (different salts, labels, packet type codes). RFC 8999 is the version-invariant header. RFC 9312 is Informational (no client obligations), and 9308 is the applicability statement. | 13 / 9 / 35 / 14 MUSTs | [QUIC WG documents](https://datatracker.ietf.org/wg/quic/documents/) | 2026-10-19 | High |
| 3.6 | **Drafts, status on 19 October 2026:** **Multipath QUIC** (draft-ietf-quic-multipath-21, 2026-03-17) is in the RFC Editor queue (blocked, final approval pending). **Reliable Stream Reset** (-11, 2026-09-06) is in the RFC Editor queue. **ACK Frequency** (-14, 2026-02-05) has WG consensus but the draft has *expired* waiting for a write-up. **qlog** main schema -14, QUIC events -13, H3 events -13 (all 2026-07-06) are still WG documents. Also active: Address Discovery -01, Extended Key Update -03, QMux -02, Receive Timestamps -03. QUIC-LB expired (2025-08-27). | — | [QUIC WG documents](https://datatracker.ietf.org/wg/quic/documents/); [multipath](https://datatracker.ietf.org/doc/draft-ietf-quic-multipath/) | 2026-10-19 | High |
| 3.7 | **QUIC Interop Runner** test cases as its QUIC documentation describes them: handshake, transfer, chacha20, keyupdate (client only), retry, resumption, zerortt, http3, multiconnect, v2, rebind-port, rebind-addr, connectionmigration; versionnegotiation is defined but disabled. Some cases inspect the packet capture for protocol requirements; downloaded files must match. The runner uses ns-3 network simulation in Docker and tests **both client and server roles** against other stacks. The case list the runner currently runs (21† at the 0.3 count) and its measurements are recorded from the runner's own output when Izcal first runs it (NW-M3, NW-4). *(2.3.3: the 0.3 row read the list from the runner's test-definition source file; withdrawn.)* | 13 documented run cases; current list to be observed | [quic-interop-runner QUIC test descriptions](https://github.com/quic-interop/quic-interop-runner/blob/master/quic.md); [Seemann and Iyengar, EPIQ 2020](https://research.protocol.ai/publications/automating-quic-interoperability-testing/) | 2026-10-20 | High (documented cases) |

**Implications:** QUIC is the best-served protocol for oracles (interop runner in both roles, qlog for traces). Izcal's QUIC crate should emit qlog from day one, pinned to the -14/-13 drafts, so the runner and qvis work. Multipath and ACK frequency should be designed for (path abstraction, ACK policy hook) but not built in the first edition.

## 4. HTTP/3, QPACK, datagrams, WebTransport, MASQUE

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **RFC 9114 HTTP/3**: **7 frame types** (DATA, HEADERS, CANCEL_PUSH, SETTINGS, PUSH_PROMISE, GOAWAY, MAX_PUSH_ID; 0x02/0x06/0x08/0x09 are reserved h2 codes), **17 H3_ error codes**, control stream + QPACK encoder/decoder unidirectional streams, push (browsers have dropped server push; a client must still parse and can refuse by never sending MAX_PUSH_ID), and reserved/grease frame and stream types. Discovery is by **Alt-Svc** (RFC 7838) or HTTPS RR `alpn=h3`. | 154 MUSTs | [RFC 9114](https://www.rfc-editor.org/rfc/rfc9114) | 2022-06 | High |
| 4.2 | **RFC 9204 QPACK**: a 99-entry static table, a dynamic table driven by **encoder instructions** (Set Dynamic Table Capacity, Insert with Name Reference, Insert with Literal Name, Duplicate) and **decoder instructions** (Section Acknowledgment, Stream Cancellation, Insert Count Increment), and **5 field-line representations** plus the Required Insert Count / Base prefix. The blocked-streams mechanism (`SETTINGS_QPACK_BLOCKED_STREAMS`) is the main source of complexity and deadlock bugs. A client may use a zero-capacity dynamic table on its encoder side and still must implement the full decoder. Oracle: **qpackers/qifs** (interop encoded files). | 41 MUSTs | [RFC 9204 §4](https://www.rfc-editor.org/rfc/rfc9204#section-4) | 2022-06 | High (RFC); Medium (qifs) |
| 4.3 | **RFC 9297 HTTP Datagrams and the Capsule Protocol**: Quarter Stream ID context on QUIC DATAGRAM frames; `SETTINGS_H3_DATAGRAM`; `Capsule-Protocol` header; capsule TLV on the request stream (used by WebTransport over h2 and by MASQUE). **RFC 9220** brings extended CONNECT to h3. | 32 / 2 MUSTs | [RFC 9297](https://www.rfc-editor.org/rfc/rfc9297); [RFC 9220](https://www.rfc-editor.org/rfc/rfc9220) | 2022-06/08 | High |
| 4.4 | **WebTransport is still not an RFC.** draft-ietf-webtrans-http3-16, draft-ietf-webtrans-http2-15 and draft-ietf-webtrans-overview-13 (all 2026-07-06) are in **WG Last Call**. An individual "WebTransport over WebSocket" draft (-05, 2026-05-10) exists. The W3C **WebTransport API is a Candidate Recommendation Snapshot dated 30 July 2026**, citing overview-11. It has HTTP/2 fallback (`reliability` = `reliable-only`), `allowPooling`, `congestionControl` hints, `serverCertificateHashes`, datagram queue/age options, and `sendGroup`/`sendOrder`. | 76 / 72 MUSTs | [webtrans documents](https://datatracker.ietf.org/wg/webtrans/documents/); [W3C WebTransport](https://www.w3.org/TR/webtransport/) | 2026-07-06; 2026-07-30 | High |
| 4.5 | **MASQUE**: RFC 9298 CONNECT-UDP (and RFC 9484 CONNECT-IP, from prior knowledge) let a browser use an HTTP/3 proxy for UDP, which is the only way to carry QUIC through a proxy. Chrome's IP Protection and Apple's Private Relay use it. For Izcal it is relevant only if HTTP/3 must work through an explicit proxy; classic HTTP CONNECT proxies force a fallback to TCP. connect-tcp (template-driven CONNECT for TCP) is at IESG evaluation. | 37 MUSTs | [RFC 9298](https://www.rfc-editor.org/rfc/rfc9298) | 2022-08 | High (9298); Low (9484, deployments) |
| 4.6 | **Oracles:** the interop runner's `http3` case only checks GET downloads. WPT `webtransport/` has 41 test files (39 `.any.js`) and needs an h3 WebTransport server (WPT's `aioquic`-based `webtransport-h3-server`). No independent QPACK/H3 client conformance suite exists beyond qifs and interop. | 41 files | WPT `4ad507b` | 2026-10-19 | High |

**Implications:** HTTP/3 and WebTransport share the stream/datagram layer with QUIC, so the H3 crate needs a capsule and datagram abstraction from the start. Because WebTransport is still a draft, Izcal must pin the draft number in anchors and expect wire changes (the `WT_` setting codepoints have changed between drafts before).

## 5. TLS 1.3 and extensions

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **RFC 9846 (July 2026) is the new TLS 1.3** (8446bis). It obsoletes RFC 8446, 5246 (TLS 1.2), 5077, 6961, 7627 and 8422, updates 5705 and 6066, and "specifies new requirements for TLS 1.2 implementations". Anchors should cite 9846, not 8446. rustls implements the protocol; Izcal's obligations are on configuration and the HTTP integration. | 345 MUSTs | [RFC 9846](https://www.rfc-editor.org/rfc/rfc9846) | 2026-07 | High |
| 5.2 | **0-RTT and replay, RFC 8470**: a client that sends early data must resend the request after the handshake on a **425 Too Early**; it should only send safe/idempotent requests in early data; intermediaries add `Early-Data: 1`. For QUIC, 0-RTT has the same rules (9001 §4.6). | 22 MUSTs | [RFC 8470](https://www.rfc-editor.org/rfc/rfc8470) | 2018-09 | High |
| 5.3 | **ALPN (RFC 7301)** selects `h2`/`http/1.1` over TLS and `h3` in QUIC. **Certificate compression (RFC 8879)**: brotli/zlib/zstd compressed Certificate message; widely deployed by Chrome. Support in rustls exists (from prior knowledge, 0.23 adds `CertificateCompressionAlgorithm` with zlib/brotli features) — not re-checked. | 5 / 14 MUSTs | [RFC 7301](https://www.rfc-editor.org/rfc/rfc7301); [RFC 8879](https://www.rfc-editor.org/rfc/rfc8879) | 2014; 2020 | High (RFCs); Low (rustls cert compression) |
| 5.4 | **ECH is RFC 9849 (March 2026)** and its DNS bootstrap is **RFC 9848 (March 2026)**. 9849 §6 lists the client duties: offer ECH with HPKE, padding (§6.1.3), GREASE ECH when no config (§6.2.1), handle `retry_configs` and authenticate the public name on rejection (§6.1.6–6.1.7). RFC 9848 §5.1: "ECH-capable, SVCB-optional clients **MUST** switch to SVCB-reliant connection establishment if SVCB resolution succeeded … and all alternative endpoints have an `ech` SvcParam", i.e. no silent fallback to cleartext SNI. | 75 / 7 MUSTs | [RFC 9849](https://www.rfc-editor.org/rfc/rfc9849); [RFC 9848 §5.1](https://www.rfc-editor.org/rfc/rfc9848#section-5.1); [Datatracker](https://datatracker.ietf.org/doc/draft-ietf-tls-esni/) | 2026-03 | High |
| 5.5 | **rustls client ECH**: `rustls::client::EchMode` has `Enable(EchConfig)` and `Grease(EchGreaseConfig)` (rustls 0.23.46), with a `fips()` check on the HPKE suite. Server-side ECH and provider coverage (HPKE availability in the ring provider) were not confirmed on the page read. | — | [EchMode](https://docs.rs/rustls/latest/rustls/client/enum.EchMode.html) | 2026-10-19 | High (client API); Low (server, providers) |
| 5.6 | **Post-quantum key exchange in rustls**: the aws-lc-rs provider's `DEFAULT_KX_GROUPS` doc says standalone MLKEM768 is not in the default list and "by default MLKEM768 is only offered in hybrid with X25519", i.e. **X25519MLKEM768 is on by default with aws-lc-rs**. The ring provider has no post-quantum algorithms (rustls README). Post-quantum **signatures** (ML-DSA in WebPKI certificates) are not deployed by public CAs as of this date; no rustls default was found. | — | [DEFAULT_KX_GROUPS](https://docs.rs/rustls/latest/rustls/crypto/aws_lc_rs/static.DEFAULT_KX_GROUPS.html); [rustls README](https://github.com/rustls/rustls) | 2026-10-19 | High (KX); Low (ML-DSA status) |
| 5.7 | **rustls 0.24 is in development**: the README says from 0.24 the crypto provider must be supplied explicitly. docs.rs "latest" is 0.23.46. Izcal should pin 0.23.x and track the 0.24 API change. | — | [rustls README](https://github.com/rustls/rustls) | 2026-10-19 | High |

**Implications:** TLS itself is delegated, but ECH forces coupling between DNS (HTTPS RR `ech`), Happy Eyeballs, connection pooling and TLS config. That coupling must be one component, not three. Oracles: rustls's own tests and BoGo; for ECH, test against Cloudflare's `crypto.cloudflare.com/cdn-cgi/trace` (`sni=encrypted`) and `defo.ie`, which are interop endpoints, not conformance suites.

## 6. DNS

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **Stub resolver base**: RFC 1034/1035 (message format, name compression, QTYPE/QCLASS, TC bit → retry over TCP), **EDNS(0) RFC 6891** (OPT pseudo-RR, UDP payload size, extended RCODE; 31 MUSTs), plus 0x20/ID randomization (RFC 5452, prior knowledge). Encrypted transports: **DoT RFC 7858** (port 853), **DoH RFC 8484** (`application/dns-message`, GET with `?dns=` base64url or POST; ID SHOULD be 0 for cache-friendliness), **DoQ RFC 9250**. EDNS padding (RFC 7830/8467) is needed to make DoH/DoT private. | 31 / 21 / 18 / 35 MUSTs | RFCs as listed | 1987–2022 | High |
| 6.2 | **SVCB/HTTPS, RFC 9460**: 7 initial SvcParamKeys (`mandatory`, `alpn`, `no-default-alpn`, `port`, `ipv4hint`, `ech`, `ipv6hint`); AliasMode (priority 0) and ServiceMode; client algorithm in §3 (follow aliases, chain limit, SVCB-optional vs SVCB-reliant). Browser-specific duties: query name rules for HTTPS RR incl. `_port._https` (§9.1); ALPN set construction (§9.2.1); **§9.5 HSTS-like upgrade**: before an `http` request the client "SHOULD perform a lookup" and, on a compatible HTTPS RR, "SHOULD behave as if it has received an HTTP 307" to the `https` URL; **§7.3 hints**: if A/AAAA are available, the client SHOULD ignore `ipv4hint`/`ipv6hint`, otherwise it may use them while querying. | 57 MUSTs | [RFC 9460](https://www.rfc-editor.org/rfc/rfc9460) | 2023-11 | High |
| 6.3 | **Happy Eyeballs v2 (RFC 8305)** is current. **v3 is draft-ietf-happy-happyeyeballs-v3-04 (2026-07-02)** in the new HAPPY WG, intended to update 8305. It adds HTTPS RR first, grouping by ALPN/ECH/priority, QUIC preference, waiting for SVCB before the TLS handshake when ECH is wanted, and timers: Resolution Delay 50 ms, Connection Attempt Delay 250 ms (min 100 ms, never below 10 ms; max 2 s), Last Resort Local Synthesis Delay 2 s, and NAT64/PREF64 handling. | 7 MUSTs in 8305 | [HEv3 draft](https://datatracker.ietf.org/doc/draft-ietf-happy-happyeyeballs-v3/) | 2026-07-02 | High |
| 6.4 | **DNSSEC** is not required of a stub; browsers do not validate (Chrome removed its DNSSEC-stapled-certificate experiment years ago). A stub must pass through the DO bit if asked and can ignore AD. DANE has no browser deployment. | — | prior knowledge | — | Medium |
| 6.5 | **OS resolver APIs**: `getaddrinfo` returns addresses only, no TTL, no HTTPS RR, and is blocking. **Windows**: `DnsQueryEx` with `DNS_TYPE_HTTPS`/`DNS_TYPE_SVCB` returns `DNS_SVCB_DATA` when `DNS_QUERY_PARSE_ALL_RECORDS` is set (Windows 11-era SDK; exact minimum build not confirmed). **Apple**: `dnssd` `DNSServiceQueryRecord` can query type 65; Network.framework / `nw_resolver` uses HTTPS RR internally for URLSession (prior knowledge). **Linux**: `res_nquery` or a direct stub; `systemd-resolved` has a varlink/D-Bus `ResolveRecord`. **Android**: `DnsResolver.rawQuery` (API 29). Whether to use OS resolvers or Izcal's own stub decides whether enterprise split-DNS and VPN DNS work; Chrome uses its own stub only when it can read the OS config. | — | [Microsoft DNS constants](https://learn.microsoft.com/en-us/windows/win32/dns/dns-constants) | 2026-10-19 | Medium (Windows); Low (Apple, Linux, Android) |
| 6.6 | **Oracles:** no stub-resolver conformance suite exists. Available tools: `dnsperf`/Knot's `deckard` (resolver scenario tests, for recursive resolvers), RFC 9460's Appendix D test vectors (wire format), and public DoH/DoT/DoQ endpoints for interop. | — | prior knowledge | — | Medium |

**Implications:** DNS is where browsers and RFCs diverge most (OS integration, split DNS, enterprise policy). The HTTPS RR is the hinge for ECH, h3 discovery and HTTP→HTTPS upgrade. Izcal needs its own stub (to get HTTPS RR on every OS) plus an OS-fallback path, and must build its own scenario oracle.

## 7. WHATWG Fetch and the web security layer

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **Fetch** (`fetch.bs` at `59afe34`, 2026-10-19): 10,595 source lines, **136 algorithm blocks**, about 1,362 list steps, 431 definitions. Core algorithms (section ids are anchor targets): `#main-fetch`, `#concept-override-fetch` (new hook: "potentially override response for a request", for scheme-fetch and http-fetch), `#concept-scheme-fetch`, `#concept-http-fetch`, `#concept-http-redirect-fetch`, `#concept-http-network-or-cache-fetch`, `#concept-http-network-fetch`, `#cors-preflight-fetch`, CORS-preflight cache, CORS check, TAO check, deferred fetching (`fetchLater`), **network partition keys** and **HTTP cache partitions**, port blocking, connections and connection pool, resolving domains, `Origin`, CORS protocol, `Content-Type`/`X-Content-Type-Options`/`Cross-Origin-Resource-Policy`/`Sec-Purpose` headers, and the Headers/Request/Response/Body API. Step-level anchors are feasible. | 136 algorithms | [fetch.bs](https://github.com/whatwg/fetch/blob/main/fetch.bs) | 2026-10-19 | High |
| 7.2 | **Spread across other specs**: `Sec-Fetch-*` (W3C Fetch Metadata), **Referrer Policy** (W3C), **Mixed Content** (W3C, Level 2 adds autoupgrade of passive content), **CSP** (W3C), **COOP/COEP incl. `credentialless`** (in WHATWG HTML), CORP (in Fetch), **Clear-Site-Data** (W3C), **Reporting API** (W3C), **Storage Partitioning** (Privacy CG; folded into HTML/Fetch/Storage), Service Workers (W3C). Exact maturity of each W3C document was not rechecked. | — | prior knowledge | — | Medium |
| 7.3 | **Local Network Access** (WICG Draft CG Report, 2026-08-07) replaces Private Network Access preflights with a **permission** (`local-network`, `loopback-network`; Chromium alias `local-network-access`). Address spaces: public > local > loopback. The check runs when a connection is obtained, on the resolved IP, and again for each new connection (DNS-rebinding defence); also on cache hits using the stored IP. Adds `fetch(..., {targetAddressSpace})`. Only Chromium is referenced as shipping. WPT `fetch/local-network-access` (31 files); the old `private-network-access` directory is gone. | — | [LNA spec](https://wicg.github.io/local-network-access/) | 2026-08-07 | High |
| 7.4 | **Cookies**: RFC 6265 is still the RFC. **draft-ietf-httpbis-rfc6265bis-22** (2025-12-01) is approved and in the RFC Editor queue, *blocked*. It has SameSite (48 mentions), `__Host-` and `__Secure-` prefixes, and no `Partitioned` attribute. A successor **draft-ietf-httpbis-layered-cookies-02** (2026-05-21; van Kesteren and Hofmann) "obsoletes RFC 6265 and 6265bis" and moves the user-agent algorithms out of the IETF document into WHATWG HTML/Fetch ("layering"); it mentions `__Http-` prefixes. **CHIPS/`Partitioned`** was specified in draft-cutler-httpbis-partitioned-cookies and is shipped by Chrome; where it lives in the layered model was not confirmed. WPT `cookies/` has 89 test files. | 47 MUSTs (bis) | [6265bis Datatracker](https://datatracker.ietf.org/doc/draft-ietf-httpbis-rfc6265bis/); [layered-cookies](https://httpwg.org/http-extensions/draft-ietf-httpbis-layered-cookies.html) | 2025-12-01; 2026-05-21 | High (status, counts); Low (CHIPS location) |
| 7.5 | **WebSockets**: wire protocol RFC 6455 (140 MUSTs; masking, framing, close handshake, UTF-8 validation), **permessage-deflate RFC 7692** (43 MUSTs), over h2 (8441) and h3 (9220). The API is the WHATWG WebSockets standard (814 source lines, at `7f25d7d`), which delegates the handshake to Fetch ("obtain a WebSocket connection"). **WebSocketStream** is not in the WHATWG text (0 mentions); it is a WICG-style proposal shipped in Chromium (Low). **Autobahn testsuite** tests *clients* in `fuzzingserver` mode: 521 cases (framing, pings, reserved bits, opcodes, fragmentation, UTF-8, close, and 12.*/13.* for permessage-deflate). It runs on PyPy 2 in a frozen Docker image (tag 25.10.1). WPT `websockets/` has 224 test files. | 521 Autobahn cases | [autobahn-testsuite](https://github.com/crossbario/autobahn-testsuite); [WHATWG websockets](https://github.com/whatwg/websockets) | 2026-10-19 | High |
| 7.6 | **WPT file counts** (commit `4ad507b`, 2026-10-19, test files excluding resources): `fetch/` 527 (of which `api` 206, `metadata` 87, `compression-dictionary` 32, `corb` 27, `orb` 16, `fetch-later` 47, `http-cache` 17, `local-network-access` 11, `cross-origin-resource-policy` 9), `cors/` 27, `cookies/` 89, `websockets/` 224, `webtransport/` 41, `mixed-content/` 390, `referrer-policy/` 1,398 (generated), `content-security-policy/` 922, `html/cross-origin-opener-policy/` 116, `html/cross-origin-embedder-policy/` 79, `clear-site-data/` 15, `reporting/` 25, `xhr/` 334, `service-workers/` 316. Subtest totals (wpt.fyi) were not fetched. | as listed | WPT tree | 2026-10-19 | High (file counts) |
| 7.7 | **ORB** (Opaque Response Blocking) has replaced CORB in the Fetch standard; WPT keeps both directories. Izcal should implement ORB, not CORB. | — | WPT tree; prior knowledge | 2026-10-19 | Medium |

**Implications:** Fetch is the anchor spine: every HTTP/DNS/TLS crate is invoked from an algorithm step in `fetch.bs`. The network crate's API should mirror Fetch's seams (fetch params, connection pool keyed by network partition key, "obtain a connection", "HTTP-network fetch"), so each step has one Rust function to anchor.

## 8. Proxies

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **PAC has no standard.** The de facto definition is Netscape's 1996 "Navigator Proxy Auto-Config File Format" plus Microsoft's IPv6 extensions (`FindProxyForURLEx`, `isInNetEx`, `myIpAddressEx`). A PAC file is JavaScript with about a dozen helper functions (`isPlainHostName`, `dnsDomainIs`, `localHostOrDomainIs`, `isResolvable`, `isInNet`, `dnsResolve`, `myIpAddress`, `dnsDomainLevels`, `shExpMatch`, `weekdayRange`, `dateRange`, `timeRange`, `alert`). Chrome strips the path/query of https URLs before calling `FindProxyForURL` (privacy fix, 2016). PAC therefore needs Izcal's JS engine in a sandbox with no DOM. | ~13 helpers | prior knowledge | 1996–2016 | Medium |
| 8.2 | **WPAD** (DHCP option 252 and DNS `wpad.<domain>` search) was an Internet-Draft that never became an RFC. It has a long security history: `wpad` name-collision attacks, the "badWPAD" research (2016) and Windows/IE MITM via auto-discovery. Firefox and Chrome follow OS settings. Recommendation: DHCP/DNS WPAD only when the OS says "auto-detect". | — | prior knowledge | 1999–2016 | Medium |
| 8.3 | **HTTP CONNECT** (RFC 9110 §9.3.6) for HTTPS via proxy; RFC 9931 adds rules for bytes sent before the 2xx. **SOCKS5 RFC 1928** (15 MUSTs; plus RFC 1929 username/password); browsers use SOCKS5 with remote DNS (`socks5h` semantics). **HTTPS proxies** (TLS to the proxy) and **MASQUE** (CONNECT-UDP) for QUIC. | 15 MUSTs | [RFC 1928](https://www.rfc-editor.org/rfc/rfc1928) | 1996-03 | High (RFC); Medium (browser behaviour) |
| 8.4 | **System proxy APIs**: Windows `WinHttpGetIEProxyConfigForCurrentUser` and `WinHttpGetProxyForUrl` (which runs PAC itself); macOS `CFNetworkCopySystemProxySettings` and `CFNetworkExecuteProxyAutoConfigurationURL`; Linux has no single API (GNOME `gsettings org.gnome.system.proxy`, KDE `kioslaverc`, environment variables `http_proxy`/`https_proxy`/`no_proxy` with no standard semantics). | — | prior knowledge | — | Low |
| 8.5 | **Oracles**: none. No PAC conformance suite exists; Chromium's and Firefox's PAC unit tests are the de facto corpus (data, readable). | — | — | — | Medium |

**Implications:** proxy support is all de facto behaviour. Izcal must write its own PAC spec page (from the Netscape text and observed Chrome/Firefox behaviour) before it can anchor anything, and build its own test corpus.

## 9. Known ambiguities and errata to track

| # | Finding | Source | Conf. |
|---|---|---|---|
| 9.1 | RFC 9110/9111/9112/9113/9114 and 9000 have verified errata on rfc-editor.org; Izcal's anchor tool should pull the errata list per RFC and flag anchors in affected sections. (Errata counts not fetched.) | [RFC errata search](https://www.rfc-editor.org/errata_search.php) | Medium |
| 9.2 | Fetch vs. RFC 9111: Fetch defines its own cache-mode semantics (`default`, `no-store`, `reload`, `no-cache`, `force-cache`, `only-if-cached`) layered on 9111, and says little about heuristics; browsers diverge (cache-tests.fyi). | fetch.bs; prior knowledge | Medium |
| 9.3 | Cookies: three live texts (RFC 6265, 6265bis-22 blocked in the RFC queue, layered-cookies-02 rewriting the layering). Browsers implement 6265bis plus CHIPS. Anchor to 6265bis-22 for parsing and to HTML/Fetch for UA policy until layered-cookies lands. | Datatracker | High |
| 9.4 | WebTransport: three drafts in WGLC, the W3C CR cites overview-11 while the IETF is at -13. Wire codepoints may still change. | Datatracker; W3C TR | High |
| 9.5 | HTTPS RR §9.5 upgrade is a SHOULD and interacts with HSTS, mixed-content autoupgrade and HTTPS-First mode; no spec defines their combined order. | RFC 9460 | High |

## 10. Implications for Izcal

### 10.1 Normative surface checklist

- [ ] **HTTP semantics** RFC 9110 (methods, status, fields, conneg, ranges, conditionals, auth framework, redirects)
- [ ] **HTTP caching** RFC 9111 + RFC 5861 (stale-while-revalidate) + RFC 8246 (immutable) + RFC 9875 (Cache Groups) + No-Vary-Search draft-10
- [ ] **HTTP/1.1** RFC 9112 (strict parser; §6.3 body length rules) + RFC 9931 (optimistic upgrades)
- [ ] **Structured Fields** RFC 9651
- [ ] **103 Early Hints** RFC 8297 + HTML processing
- [ ] **Compression**: gzip/deflate/br/zstd content codings + **RFC 9842** dictionaries (`dcb`, `dcz`)
- [ ] **HTTP/2** RFC 9113 + **HPACK** RFC 7541 + **extended CONNECT** RFC 8441 + **Extensible Priorities** RFC 9218
- [ ] **QUIC** RFC 8999, 9000, 9001 (via rustls `quic`), 9002 (+ own CUBIC/BBR), 9221, 9287, 9368, 9369; qlog drafts (-14/-13); design hooks for multipath (draft-21) and ACK frequency (draft-14)
- [ ] **HTTP/3** RFC 9114 + **QPACK** RFC 9204 + **datagrams/capsules** RFC 9297 + RFC 9220 + Alt-Svc RFC 7838
- [ ] **WebTransport** draft-ietf-webtrans-http3-16 / -http2-15 / -overview-13 + W3C CR 2026-07-30
- [ ] **MASQUE** RFC 9298 (only for proxied h3)
- [ ] **TLS** RFC 9846 via rustls; ALPN RFC 7301; 0-RTT + 425 RFC 8470; cert compression RFC 8879; **ECH** RFC 9849 + RFC 9848; X25519MLKEM768
- [ ] **DNS** RFC 1034/1035, 6891, 7830/8467 padding, 7858, 8484, 9250, **9460**, Happy Eyeballs RFC 8305 → v3 draft-04
- [ ] **Fetch** (all 136 algorithms) incl. CORS, CORP, ORB, network partition keys, cache partitions, port blocking, `fetchLater`
- [ ] **Fetch Metadata**, **Referrer Policy**, **Mixed Content**, **CSP** (network-relevant parts), **COOP/COEP** (HTML), **Clear-Site-Data**, **Reporting API**, **Storage Partitioning**, **Local Network Access**
- [ ] **Cookies** 6265bis-22 + CHIPS + layered-cookies (watch)
- [ ] **WebSockets** RFC 6455 + RFC 7692 + WHATWG WebSockets (+ WebSocketStream, watch)
- [ ] **Proxies**: PAC (own spec page), WPAD, HTTP CONNECT, HTTPS proxies, SOCKS5 RFC 1928/1929, OS proxy APIs

### 10.2 Conformance oracles available

| Protocol | Oracle | Tests client? |
|---|---|---|
| QUIC + H3 basics | QUIC Interop Runner (21 tests + 2 measurements) | Yes, both roles |
| QPACK | qpackers/qifs | Yes (decoder) |
| HPACK | http2jp/hpack-test-case | Yes (decoder) |
| HTTP/2 | h2spec (RFC 7540) | **No**, server only |
| Structured Fields | httpwg/structured-field-tests | Yes |
| HTTP caching | httpwg/cache-tests | Yes (browser mode) |
| Fetch, CORS, cookies, mixed content, referrer, COOP/COEP, LNA, compression dictionaries, WebSockets API, WebTransport API | WPT (file counts in 7.6) | Yes |
| WebSocket wire + deflate | Autobahn fuzzingserver (521 cases) | Yes |
| TLS | rustls tests, BoGo (rustls's job) | Delegated |
| ECH | public interop endpoints only | Partial |

### 10.3 Gaps where Izcal must build oracles

1. **HTTP/1.1 client parsing and smuggling resistance**: no client suite; build a hostile-server corpus from 9112 §5–7 and RFC 9931.
2. **HTTP/2 client**: build an adversarial h2 server harness from 9113's 225 MUSTs (h2spec only covers servers, and only 7540).
3. **HTTP/3 client beyond GET**: error handling, GOAWAY, QPACK blocking, push refusal, priorities.
4. **Extensible Priorities**: no suite; needs trace-based checks (qlog).
5. **DNS stub, HTTPS RR handling, and Happy Eyeballs (v2/v3)**: no suite; build a scripted DNS + network simulator (timers, ECH grouping, NAT64).
6. **ECH client behaviour** (GREASE, retry configs, no-fallback rule): no suite.
7. **Proxies**: PAC semantics, WPAD, SOCKS5, CONNECT: no suite at all.
8. **Congestion control**: only the runner's goodput/crosstraffic measurements; no correctness oracle.
9. **WebTransport wire protocol**: drafts in WGLC; only WPT via one aioquic server.

**Unverified in this note:** exact h2spec and structured-field-test counts; rustls server-side ECH and RFC 8879 support; Apple/Linux/Android DNS API details; CHIPS's home in the layered-cookies model; W3C maturity of Fetch Metadata, Referrer Policy, Mixed Content, Clear-Site-Data and Reporting; WebSocketStream shipping status; RFC errata counts; WPT subtest totals.
