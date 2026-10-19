# Izcal networking: independent design critique (phase 8)

Written 19 October 2026 by a reviewer who did not write the spec. Subject: `reports/Izcal networking.md`, draft 0.1 (line numbers below refer to that draft). Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8 (design critique against the existing specs, looking for contradictions, missed constraints and M-1 violations).

**Read against:** HGBA-2 2.3.0 (M-1, P1–P6, I-1 (a)–(c), I-2, I-6, Sections 4.6, 4.7); HGBA-3 (B1, B2, B9, B10, DET-1, DET-2, AG-0); Stage 1 design (Section 1 process map, 2, 3.1–3.4, 4.1–4.2, 6, 8.1–8.3, 13.2); JavaScript engine spec 0.3 (JS-D15, Section 6.9, 9.3 staffing format); resource-footprint spec 0.3 (FP-1, FP-17, FP-18, FP-26, FP-D8, FP-D9, FP-D16, FP-D18, Sections 6.8, 6.11, 6.12, 6.18.1, 8.2); DevTools spec 0.3 (DT-D2, DT-D5, DT-18, DT-21, `hg-net-observe`, Q-DT-5); security scope 0.3 on the `security-scope` branch (6.3 assets, 6.5 network-service and WebRTC threat models, 6.6 IPC-R1–R17, 6.7 sandbox table, 6.10a, 6.11 NET-R1–R7, 6.13 REC-R, 6.14a SH-R6–R9, SEC-28, SEC-31, SEC-34, SEC-D13, Q-SEC-5, 9.2 costs); coding standard 0.3.1 on the `cs-q56` branch (tiers, CS-D1, CS-D8, CS-AS-1, CS-AS-4, CS-AS-7, CS-ER-9, CS-ER-13, CS-T11, Q-CS-1, Q-CS-2); fonts and text spec (WOFF2 decoders, Q-TX-11); and this spec's seven research notes. Where a claim about a dependency was checked locally, the crate source in the Cargo registry was read (rustls-webpki 0.103.15, webpki 0.22.4).

**Owner direction applied as binding:** engine networking is Izcal's own (no neqo, quiche, quinn, hyper, h2, h3, reqwest, Chromium net or any library doing browser networking; allowed: Rust std, general-purpose crates, rustls and crypto libraries, OS APIs); no shortcuts for time or complexity; staging only for correctness, security or measurement; JS-D15, FP-D16, FP-D8, FP-D9, the Section 8.2 footprint budgets and "hidden pages get no wakeups" honoured; P1 and I-1 first-class.

**Overall.** The protocol core is strong: sans-IO state machines on substrate sequences, budgets charged at allocation, Kani-proved arithmetic kernels, one parser per format, strict HTTP/1.1 framing, and a partition table that goes further than any shipping engine. The weak points are at the **process and trust boundaries** the spec introduces, which is where its three most novel decisions sit: the socket broker (NW-D4), the recording boundary (NW-D2) and the per-container services (NW-D3). In several places the spec claims a property ("policy enforced outside the parser", "replays every layer above decryption", "the container boundary becomes a process boundary", "closes the connection-pool XS-Leak") more strongly than the design as written supports, or in a form that contradicts an accepted rule of another spec (IPC-R2, IPC-R15, FP-D16, FP-D8, DT-D2). WebRTC transport, which the security scope places in this process, is missing entirely. Nine findings are High.

---

## Summary

| Severity | Count | Findings |
|---|---|---|
| High | 9 | C1 (recording has no start state or recorded unit), C2 (recording misses QUIC inputs; connection IDs from a non-crypto stream), C3 (broker socket policy uses identity from the message), C4 (brokered OS calls parse network bytes in the broker and make it a credential oracle), C5 (HTTP cache store breaks FP-D16's compromise rules), C6 (`document.cookie` region breaks origin keying and IPC-R1/R7), C7 (WebRTC transport missing), C8 (NW-D3 stops at the container without analysis; deferred to NW-M4), C9 (Clear-Site-Data crosses containers) |
| Medium | 13 | C10–C22 |
| Low | 7 | C23–C29 |
| **Total** | **29** | |

---

## High

### C1. High — The production recording (NW-D2) has no start state and no recorded unit, so NW-37 cannot be met as written and the replay breaks DT-D2's P1 split

**Location:** 6.13 (line 366), NW-D2 (line 604), NW-37 (line 536), 9.5 Stage 1 3.2 amendment.

**Problem.** NW-D2 records "per connection" the decrypted inputs, timer firings and decision points, and claims that "on replay, `hg-net` runs from the decrypted layer upward, so HTTP/2 and HTTP/3 framing, QPACK, QUIC streams and recovery, the cache, cookies and Fetch all re-execute". An input log replays only from a known start state. The network service is long-lived and per container (NW-D3); when a user starts recording, its HPACK and QPACK dynamic tables, open HTTP/2 and QUIC connections (with flow-control windows, congestion state and stream IDs), DNS cache, Alt-Svc and broken-HTTP/3 state, HSTS, cookie jar and HTTP-cache index already exist. Replaying "the cache, cookies and Fetch" from the decrypted bytes alone is impossible without that state. The spec never says what is recorded at start, or what a "production recording" covers:

- **Start state.** DT-D2 solved the same problem for renderers by starting every recording at process launch ("a fresh renderer plus its recorded inputs is the one start state Izcal can reproduce"). NW-D2 has no equivalent: no fresh service, no snapshot, no rule that recorded connections are opened fresh.
- **Recorded unit.** DT-D2 records a tab's browsing context group in dedicated processes and says "other tabs of the site stay in their own processes and are never recorded". Network connections are pooled per partition, so an HTTP/2 or QUIC connection used by the recorded tab also carries other tabs' streams of the same partition, and replaying the connection needs all of them. The recording therefore holds traffic the user did not choose to record.
- **Replay placement.** The security scope's replay host is "per (site, container), sandboxed like a renderer"; DevTools 0.2 split replay per (site, container) precisely "so no P1 exception is needed". A network-service replay is cross-site (one container's whole partition set, or at least every frame site under one top-level site). Where it runs, at which tier, and under which P1 exception is unspecified.
- **NW-37 measures the wrong thing.** It requires "bit-identical DET-1 hashes". DET-1's hashes are DOM serialization, fragment tree, display list and semantic tree (HGBA-3 DET-1); renderers replay from their own IPC inputs (DT-D2 "Contents"), so those hashes do not depend on network-service replay at all. Only the outgoing-plaintext hash tests the network replay, and it says nothing about the cache, cookie or pool state the spec claims re-executes.

**Evidence.** DT-D2 (DevTools spec, lines 214–222): "A replayable recording starts at process launch"; "The log is per process" (DevTools 4.2). Security scope 6.3: replay host "per (site, container)". HGBA-3 DET-1 (line 398 of HGBA-3). Stage 1 3.4: recording "user-initiated per tab".

**Proposed fix.** Make the recorded unit and start state a decision (extend NW-D2): (a) while a tab records, its partitions run on **fresh connections only** (no reuse of pre-recording connections; recording-only pools keyed by a recording nonce, which the key already has room for as `n`), with a **start snapshot** of the partition's cookie jar, HSTS, Alt-Svc and cache-index view taken at recording start and stored in the bundle; or (b) a **dedicated, freshly launched network service** for the recorded group, seeded from such a snapshot. State which one, its cost, and that other tabs' traffic never enters the bundle. Say where network replay runs (a replay host per (top-level site, container) at the network service's tier, with the P1 analysis written down), and replace NW-37's DET-1 clause with network checkpoints: a digest of the decision log, per-connection stream state and cache/cookie state at each flush, compared at 1,000 of 1,000.

### C2. High — The decryption boundary does not capture everything QUIC recovery and the outgoing hash depend on, and connection IDs come from a non-crypto stream

**Location:** 6.2 (line 217, "`EntropyPurpose::QuicConnectionId`"), 6.13 (line 366), NW-37, NW-39, 9.5 (Stage 1 3.1: "add `EntropyPurpose` entries for QUIC connection IDs and tokens").

**Problem.** The spec records "QUIC packet payloads after packet and header protection are removed" and "values drawn from the crypto stream that appear in decrypted plaintext". That is not the full input of the QUIC state machine:

1. **Unprotected header fields.** Loss detection and ACK generation need the received **packet number**, packet type, key phase, spin bit, DCID and the datagram the packet arrived in (coalesced packets, datagram size). These are in the header, not the payload.
2. **Packets the crypto layer rejected.** Whether a packet decrypted is a crypto-layer outcome that changes state: an undecryptable datagram is checked against **stateless-reset tokens** on its raw tail (RFC 9000 §10.3.1), key-update and key-phase handling depend on which keys succeeded, and dropped packets still count toward some limits. None of this is "plaintext".
3. **IP-layer inputs.** ECN codepoints and GRO segment sizes arrive in control messages; ECN counts feed ACK_ECN and congestion response (the spec reports ECN "from the start", 6.4).
4. **Outgoing values from the crypto stream.** The outgoing-plaintext hash covers frames the client generates from crypto entropy that never appear in received plaintext: client-issued connection IDs and their stateless-reset tokens in NEW_CONNECTION_ID, PATH_CHALLENGE data, the initial DCID. Replay cannot reproduce them without recording them, and the stateless-reset tokens are secrets in RFC 9000's sense ("difficult to guess"), which sits awkwardly with NW-39's "0 secrets".
5. **Entropy class.** 6.2 draws connection IDs from `EntropyPurpose::QuicConnectionId`, a **named seeded stream** (Stage 1 3.1: "every consumer gets a named, seeded stream"). RFC 9000 §5.1 requires that connection IDs not let an observer correlate them, and §9.5 requires unlinkable IDs across paths after migration. If a stream's outputs are predictable from a seed (or from earlier outputs, for a non-cryptographic generator), an on-path observer can link a migrated connection, which defeats a privacy property of QUIC; if the seed is recorded, so can anyone holding a recording. Address-validation tokens and stateless-reset tokens must also be unpredictable.

**Evidence.** RFC 9000 §§5.1, 9.5, 10.3, 13.4; RFC 9002 §§3, 6 (loss detection keyed by packet number). Stage 1 3.1 (`Entropy::stream` versus `Entropy::crypto`). Q-CS-1 forbids a seeded `crypto()` in a network binary, which is right, but it does not stop the spec drawing security-relevant values from a non-crypto stream. The brief's question "does NW-D2 truly replay QUIC recovery" is answered: not as specified.

**Proposed fix.** Define the recorded QUIC input as a typed record per **datagram**: arrival time, ECN and segment-size metadata, and per contained packet the unprotected header fields plus either the payload or a "rejected" outcome with its reason (no keys, AEAD failure, stateless-reset match). Draw connection IDs, stateless-reset tokens, PATH_CHALLENGE data and address tokens from `Entropy::crypto()` (OS CSPRNG in production) and record the **drawn values**, not a seed, as decision inputs (JS-D15's model); classify stateless-reset tokens as recording-sensitive and encrypt bundles under SEC-34 (REC-R2 already does). Remove the `EntropyPurpose` amendment for these values from 9.5. Add a planted test to NW-37: replay of a recorded connection with loss, reordering, ECN-CE marks, a key update, a NAT rebinding and a stateless reset, compared at the outgoing-plaintext and recovery-state level.

### C3. High — The socket broker's policy rests on identity and purpose the network service supplies, which IPC-R2 forbids and which defeats NW-D4's stated purpose

**Location:** 6.1 (lines 203–209), NW-D4 (line 606), NW-16 (line 500), 6.12 PAC (line 354), 6.16 "Blast radius".

**Problem.** NW-D4's justification is that "policy enforcement sits in a process that a hostile server cannot reach" and that a compromised network service "cannot reach the local network or loopback ... without a grant". But the broker checks LNA "against the container's LNA grants and the request's initiator, **which the network service passes** as the endpoint-bound frame identity". The broker's endpoint to the network service is bound to the container's service, not to a frame; the frame identity in a `mint` call is message data from the very process the design treats as compromised. The same holds for every other input the broker cannot verify:

- **LNA.** Once any site in the container holds a grant (a router admin page, a printer, a local dev server), a compromised service can name that frame and reach the granted address for any site. The real guarantee is "no local address that no frame in this container was granted", which is weaker than NET-R7 ("a Local Network Access grant for that (site, container)").
- **`purpose`.** `dns` and `proxy` sockets must reach local addresses (home resolvers at 192.168.x.y, corporate proxies). Unless the broker restricts `dns` to the resolver addresses and ports in the OS configuration it read itself, and `proxy` to configured proxies, the purpose field is an LNA bypass.
- **PAC.** PAC returns proxy host:port from a script (user-, enterprise- or WPAD-supplied, and WPAD on a hostile network is attacker-supplied). The PAC result travels PAC process → network service → broker, so the broker cannot tell a PAC-chosen proxy from an invented one.
- **WebRTC.** ICE needs **unconnected** UDP sockets that send to many peers, including local addresses for LAN calls. A bound, unconnected UDP socket can `sendto` any address, so a broker check "against the address" at mint time enforces nothing afterwards. "One mechanism serves both" (line 209) is not true for WebRTC.
- **Connection migration.** QUIC client migration and a server's `preferred_address` need new sockets to new addresses; the text covers redirects and DNS changes but not these.

**Evidence.** IPC-R2: "Identity comes from a bound endpoint, never from the message"; (c) "no authorization API accepts" message-carried origins. IPC-R17: "No process trusts another". NET-R7 and SEC-31 (grant per (site, container)). Security scope 6.10a: WebRTC UDP sockets "per (site, container)". GO 7.10: LNA for WebRTC is "only proposed".

**Proposed fix.** Give the broker its own ground truth: (a) the broker issues, per frame and worker, an **LNA ticket** (an unforgeable token bound to (frame, granted address set)) when it creates the `FetchEndpoint`; the renderer's request carries it as data, the network service forwards it, and the broker accepts a local-address mint only with a ticket it issued whose grant covers the address; (b) `dns` and `proxy` purposes accept only addresses from the configuration the broker itself read, and PAC results go **PAC process → broker** (validated against the configured proxy policy) as well as to the network service; (c) WebRTC gets a separate design (C7): either connected UDP sockets per candidate pair (with `SO_REUSEPORT` or per-OS equivalents measured), or LNA for WebRTC defined as "no host or local-address candidates without a grant", enforced by the broker refusing local-interface bindings; (d) state migration and `preferred_address` as new mints. Rewrite 6.16's blast-radius line to the guarantee actually provided, and make NW-16's probe include a compromised-service harness (IPC-R17 adversarial peer) that tries each forgery above.

### C4. High — "Everything else ambient is brokered" moves network-byte parsing into the broker and turns it into a credential oracle

**Location:** 6.1 (line 211), 6.6 OS fallback (line 272), NW-G5 (line 141 of 6.18), 6.16 "Blast radius".

**Problem.** The broker is unsandboxed and holds every capability (security scope 6.3). The spec hands it four jobs that each process data a hostile server controls:

1. **OS resolver fallback.** "Names the stub cannot answer ... fall back to the OS resolver API through the broker." On Linux without systemd-resolved, glibc's `getaddrinfo` parses DNS responses **in the calling process**; that parser had a remotely exploitable stack overflow (CVE-2015-7547). The broker would parse network bytes, which IPC-R15 forbids.
2. **Kerberos and NTLM.** "Kerberos and NTLM tokens (signed by the broker through SSPI or GSSAPI)". The server's `WWW-Authenticate: Negotiate`/`NTLM` challenge is network input; SSPI and GSSAPI parse it in the broker's address space. Worse, nothing restricts which servers may trigger ambient authentication: any web server can make the broker produce NTLM responses (crackable offline) or Kerberos tickets for any SPN the compromised network service names. Browsers restrict ambient authentication to an intranet or allowlist for this reason.
3. **Client-certificate signing.** "The key never enters the network service" is true, but the broker signs whatever the network service sends. A compromised service gets a signing oracle for the user's client key (smart card included), usable for any server in any container.
4. **Trust anchors.** "User- and enterprise-added trust anchors" read by the broker must be passed as opaque DER, never parsed there.

NW-G5 ("parses no network bytes") is a rule about `hg-socket-broker`'s own code; it does not cover the OS libraries the broker calls on network data.

**Evidence.** IPC-R15 ("No parsing of web formats in the broker, even though it is Rust: parse in a site-locked renderer or a utility process"); IPC-R10; security scope 6.3 broker row ("The design goal is that no web-format parsing ... run here"); [CVE-2015-7547](https://nvd.nist.gov/vuln/detail/CVE-2015-7547) (glibc `getaddrinfo`; to be confirmed by the fact-checker).

**Proposed fix.** Move each OS call that consumes network data into a **sandboxed OS-adapter utility process** per container (seccomp/SBPL/AppContainer allowing only the OS service it needs), not the broker: resolver fallback, SSPI/GSSAPI, and client-key signing. Bind authentication to policy the broker owns: ambient Negotiate/NTLM only for hosts on an enterprise or user allowlist (default: none on the public internet), per container; client-certificate signing only for (container, host) pairs the user selected through CON-R1, with a per-handshake grant the broker issues and the adapter checks. Pass trust anchors as opaque DER to `hg-pki`. Add these processes to the security scope's 6.3 and 6.7 tables and a harness that drives them from a compromised network service.

### C5. High — The HTTP cache store (NW-D11) contradicts FP-D16's rules against persistence of a compromise, and NW-31 contradicts FP-D16 directly

**Location:** 6.10 "Store" (line 336), NW-D11 (line 613), NW-31 (line 525), 9.5 footprint amendment (FP-D16 format-version binding).

**Problem.** FP-D16 (owner-accepted) admits a custom store into the regenerable-cache class only if it is "authenticated, bound to the engine build, discarded and never repaired on failure, **discarded when the writing process later crashed, failed a check or was killed for a bad message**". Those clauses exist because the writer may be compromised (footprint 6.9, "persistence of a compromise ... encryption does not help, because the writer is the attacker"). NW-D11 breaks three of them:

- **The MAC key is held by the writer.** Records are authenticated "under a per-container key"; the network service writes the cache, so it holds the key. A compromised network service can write well-formed, authenticated entries for any origin in the container. Because cached responses are delivered to renderers as that origin's content, this is **persistent cross-origin script injection** in that container that survives restarts, the opposite of what the class is for.
- **Crash handling.** NW-31 requires "≥ 99%† of committed entries readable" after `kill -9`, while FP-D16 requires discarding a member when its writing process crashed. A crash of the network service is exactly the case FP-D16 targets (a crash from a bug reached by hostile input).
- **Format-version binding.** The proposed amendment binds the cache to its format version instead of the build, so entries written by a vulnerable build **survive the security update that fixes it**. That is the main reason for build binding.

**Evidence.** FP-D16 (footprint 9.3, line 951); footprint 6.9 "Persistence of a compromise" (line 510); HGBA-2 4.6 ("never custom persistence layers"), which FP-D16 resolves only on those terms.

**Proposed fix.** Keep the crash-only segment design (it is sound for availability) and add the compromise rules: (a) entries are **not authoritative**: a cached response is delivered only if a validator recorded at write time still matches, or (stronger) the cache key includes a per-boot epoch that the **broker** issues and rotates whenever a network service crashes, fails a check, or is updated with a security fix to `hg-net`, `hg-http-cache`, `hg-http*` or `hg-compress`; old epochs are discarded, not repaired; (b) bind to "format version **and** security epoch" rather than format version alone, so ordinary updates keep the cache and security updates drop it; (c) rewrite NW-31 as "after `kill -9` **of an uncompromised service by the test harness**, ≥ 99%† readable; after a crash, a check failure or a bad-message kill, 0 entries from the old epoch served". Record the residual (a compromised service can poison its container's cache until its next crash or update) in the security scope's network-service threat model.

### C6. High — The `document.cookie` region (NW-D10) leaks sibling-subdomain cookies into origin-keyed renderers and does not fit IPC-R1 or IPC-R7

**Location:** 6.9 (line 330), NW-D10 (line 612), 9.5 Stage 1 4.1 amendment.

**Problem.**

1. **Origin keying.** The region is "per (site, partition) ... non-HttpOnly, matching the **site**". FP-D8 (owner-accepted) turns origin keying on for every desktop class, so `a.example.com` and `b.example.com` run in different processes precisely to stop subdomains reading each other's data (I-1 (c), Kilger et al.). Host-only cookies of `b.example.com` would sit in the region mapped into `a.example.com`'s process. The claim "gives a compromised renderer nothing its own site's requests would not" is false under origin keying: the renderer's own origin's requests would never carry `b`'s host-only cookies.
2. **IPC-R1.** IPC-R1 defines read-only shared memory as an immutable object (a memfd sealed with `F_SEAL_WRITE`, a section duplicated without write access, a read-only Mach entry). An updatable region whose producer keeps writing is a different handle kind; on Linux `F_SEAL_WRITE` would forbid the producer's writes too (`F_SEAL_FUTURE_WRITE` is the seal that fits).
3. **IPC-R7.** "The getter reads the region" while the network service writes it. The renderer is the receiver that parses the cookie strings, so copy-then-validate applies; a version-counter read without copying is a double fetch.
4. **Ordering with responses.** `await fetch('/login'); document.cookie` must show cookies the response set. The spec does not order the region update before the response's delivery to script.
5. **Writes and the one-parser rule.** For read-your-writes, the renderer must parse the assigned string (name, value, attributes, prefix rules) to put it in the overlay. That is either `hg-cookies` linked into the renderer (allowed if declared, but the spec should say so given G7) or a second parser (NW-G2 forbids it). Rejections and evictions the network service applies later (prefix violations, per-domain limits) make the overlay briefly show a cookie that never existed.
6. The Cookie Store API and its change events are not covered.

**Evidence.** FP-D8 (footprint line 943); HGBA-2 I-1 (c); IPC-R1 and IPC-R7 (security scope 6.6); NW-G2; 6265bis host-only flag.

**Proposed fix.** Key the region by **(origin's host, partition)** and fill it with cookies that domain-match the host (host-only cookies of the host, domain cookies of its parent domains), so its content equals what the origin's own requests could carry. Add an IPC-R1 handle kind "producer-writable, consumer-read-only shared memory" per OS (Linux `F_SEAL_FUTURE_WRITE` after the producer maps it; Windows section duplicated without `FILE_MAP_WRITE`; Mach entry with read-only protection) and make the getter copy a snapshot under a seqlock before parsing. Specify that the network service publishes the region version before delivering a response that set cookies, and that the response carries the minimum version the renderer must see. State that `hg-cookies`' parser is linked into renderers (one parser, two tiers). Add WPT and corpus cases for read-after-fetch, rejected writes and Cookie Store events to NW-D10's reversal trigger.

### C7. High — WebRTC transport is missing from the network service's spec

**Location:** whole spec; 6.1 (line 209, "WebRTC already uses broker-minted UDP sockets"), 6.8 table, 6.13, 9.3, 9.2.

**Problem.** The security scope places **ICE, DTLS, SRTP and SCTP in the network service**, with UDP sockets per (site, container), built by Izcal from the RFCs (6.10a), and SEC-28 and NET-R4 fuzz "WebRTC transport" in this process. Stage 1 M7 requires "PQC for WebRTC DTLS". This spec, which is the network service's design, mentions WebRTC only as a `mint` purpose and as "relay-only through the route". It gives no:
- **crate or owner** (ICE agent, STUN/TURN, DTLS 1.2/1.3, SRTP, SCTP over DTLS);
- **TLS position**: rustls implements TLS, not DTLS (to be confirmed by the fact-checker), and a general WebRTC crate (webrtc-rs) is a browser-networking library under the owner's direction, so Izcal builds its own DTLS, a second TLS-class protocol implementation with X25519MLKEM768 (HGBA-2 4.7);
- **partition row**: the research note proposes ICE candidates and device IDs keyed (T, C, R) (PP 8, line 191), which the spec's table drops;
- **socket model** (C3: unconnected UDP defeats per-address checks);
- **recording boundary** (DTLS-SRTP is decrypted where, and is media plaintext recorded?), **budgets**, **wakeups** (ICE consent freshness, RFC 7675, sends traffic every few seconds; footprint 6.8 makes "WebRTC with live tracks" a declared reason, so it needs an explicit rule), **footprint**, **staffing** or **milestone**.

**Evidence.** Security scope 6.10a and the WebRTC threat model (6.5, lines 567–574); SEC-28; NET-R4; Stage 1 8.1 M7; HGBA-2 4.7 (hybrid DTLS); PP 8.2 and the PP partition proposal.

**Proposed fix.** Add a Section 6.x "WebRTC transport" with crates (`hg-ice`, `hg-dtls`, `hg-srtp`, `hg-sctp`, tier P, sans-IO like the rest), the DTLS decision recorded under NW-D6 (own DTLS on rustls's crypto provider and key schedule pieces where they are public API, or own DTLS outright), the socket model of C3, the partition row (T, C, R), the recording boundary (DTLS application data after decryption; media plaintext never recorded by default), consent-freshness timers under a declared reason, targets (ICE and DTLS interop against at least two independent stacks, SEC-30 fuzzers, a no-host-candidate-without-grant test, route leak test in NW-21), a staffing row (realistically 4–6†) and a milestone tied to Stage 1 M7.

### C8. High — NW-D3 stops at the container although the spec's own argument extends to the top-level site; option (c) is rejected without analysis, and the split is deferred to Stage 1 exit

**Location:** 6.1 (line 201), 4.4 first row, NW-D3 (line 605), NW-M4 (line 568), Section 5 lessons ("Moving code into a process late is the expensive direction").

**Problem.**
1. **The argument proves more than the decision.** NW-D3 rests on "no state is shared across containers anyway, so running one network service per container loses no sharing". By the spec's own table (6.8), almost every row also contains **T**: cache, connections, DNS, Alt-Svc, tickets, HSTS, client certificates, CORS preflight, partitioned cookies, reporting. The exceptions are first-party unpartitioned cookies (used only when that site is the top level, or under a Storage Access grant), DoH connections and proxy/PAC (C, R), and static data. So one service per **(top-level site, container)** would also lose almost no sharing, and would bring the network service much closer to P1's "never co-locate cross-site data in one address space", which every shipping browser's network service violates. NW-D3 lists (c) "one per top-level site" and gives no reason against it.
2. **The blast-radius claim is overstated for most users.** Stage 1 ships containers as "1 key design, 2 UI" (Stage 1 Section 6), so in Stage 1 nearly every user has one container plus private windows; a compromise of that service still yields "every site's sessions and traffic" (security scope 6.3), as for every other browser.
3. **Schedule staging.** NW-M4 delivers "per-container services and WireGuard routes" at Stage 1 exit, after three milestones with one service. The spec's own lesson from Firefox's socket process is that moving code into a process late is the expensive direction; no correctness, security or measurement reason for the order is given in 9.1.

**Evidence.** Section 6.8 table; HGBA-2 P1 and I-1 (a); security scope 6.3 network-service row; Stage 1 Section 6 containers row and 8.1; owner direction on staging.

**Proposed fix.** Present NW-D3 with a real analysis of (a), (c) and a hybrid (per (T, C) services for the cross-site-heavy state, with first-party cookies served by a per-container cookie service through Storage-Access-aware endpoints): process count on the corpus (Chrome's p99 of 41.9 sites gives the order of magnitude), per-service fixed cost against FP-1 and the ≤ 4 MiB† line, navigation latency for a new top-level site (service start on the critical path, or a spare service), DoH and proxy duplication, and the P1 gain. If (a) stays, say why P1 does not require (c). Either way, build the **multi-service placement from NW-M0** (the service is born per container, even if only one container exists), and list the order in 9.1 with its reason.

### C9. High — NW-D18's Clear-Site-Data rule reaches across containers, which contradicts itself and creates a cross-container linking channel

**Location:** 6.8 (line 322), NW-D18 (line 620).

**Problem.** The rule says a first party "clears its own state in every partition **and container** where it is the top-level site" and, in the same sentence, "it never clears another container's state". The two halves contradict each other. Read as the first half (and NW-D18 option (a) says "in every partition where it is top-level", without the container limit), a site in container A could erase its state in container B; the site can then observe in B that its state vanished just after an action in A, which links the user's two identities. I-1 (a) makes the container a separate policy context with its own storage partitions; a cross-container side effect controlled by a site is exactly the channel containers exist to close. With NW-D3 it would also need a cross-service clearing path through the broker that nothing else needs.

**Evidence.** HGBA-2 I-1 (a); Stage 1 Section 6 containers row ("Each of these is an axis on which an attacker ... could otherwise correlate two containers", Stage 1 Section 1); NW-D18 text.

**Proposed fix.** Restrict NW-D18 (a) to **the current container**: the site's own state in every partition of that container where it is the top-level site, plus its third-party state in the current partition. Add a privacy test (in NW-19's suite): Clear-Site-Data in container A leaves every byte of the site's state in container B unchanged and no timing difference on B's next load.

---

## Medium

### C10. Medium — Per-partition budgets do not close the connection-pool channel; they move the cap and let one page multiply partitions

**Location:** 6.8 "Budgets instead of a global cap" (line 320), NW-20 (line 509), 9.5 footprint 6.18.1 amendment ("connection pools and socket budgets (closed, NW-20)").

**Problem.** A service-wide limit of 256† remains; "when the service is under pressure the limit applies per partition in round-robin order". Round-robin is fairer than FIFO, but a partition's wait still depends on how many other partitions are active, so the channel is bounded, not closed. Worse, the key contains F and the per-navigation nonce n, so **one page can create many partitions**: a top-level page embedding k cross-site iframes, or k credentialless iframes (a fresh n each), gets k shares. 256 / 32 = 8 full shares. Below Izcal there are further shared caps the design does not mention: per-process descriptor limits (in the broker until handles are closed, and in the service), ephemeral ports, and the broker's mint queue. The research note itself proposed "a per-partition budget plus a **randomized** global capacity" and called the combination "untested" (PP 1.24, open risk 1); the spec dropped the randomization and calls the leak closed. NW-20 tests the one published attack, which cannot show that the class is closed.

**Evidence.** PP 1.12, 1.24 and the partition proposal (line 174: "per-partition budget plus a randomized global capacity; no deterministic shared hard cap"); xsleaks connection-pool page.

**Proposed fix.** Budget per **top-level site** first (all partitions under one T share one quota), then per partition inside it, so one page cannot fan out; cap the number of live nonce partitions per top-level document; keep the global capacity randomized per session (the note's proposal) and state the residual. Record the pool as **bounded** in footprint 6.18.1, not closed. Replace NW-20's pass criterion with a statistical one: an attacker page with up to N† partitions and a victim page in another top-level site, the attacker's classifier accuracy ≤ 50% + ε† at a stated power, plus the published attacks.

### C11. Medium — NW-19 is not falsifiable as written, and the partition table misses rows and shared resources

**Location:** NW-19 (line 508), 6.8 table (lines 296–315), 6.4 "receive buffer per service" (line 243), 6.14 (line 374).

**Problem.** NW-19 passes when a probe "observes no timing or state difference beyond the noise of an **unpartitioned control**". An unpartitioned control *is* the leak, so its "noise" is not a meaningful baseline; there is no sample size, test or power, so the target cannot fail in a defined way. The table also omits rows the notes listed or the design creates: WebRTC ICE candidates (T, C, R) (PP line 191); Resource Timing and connection-timing coarsening (PP line 192, Fetch's "clamp and coarsen connection timing info"); favicons (fetched for the shell, which is cross-site); downloads; credentials entered in HTTP-auth prompts; the Web Push channel (C21). Cookies are keyed (T, C) without the cross-site-ancestor bit, while CHIPS added that bit to its partition key; the spec should say whether A-in-B-in-A shares A's partitioned jar. Finally, the network service's shared resources (one receive buffer per service, connection-group sequences, the decryption queue) are shared timing channels that belong in footprint 6.18.1 as bounded, which Q-NW-10 only half raises.

**Evidence.** NW-19 text; PP partition proposal (lines 174–192); 6.4 and 6.14 "Receive buffers are per service".

**Proposed fix.** Define NW-19 per axis as a two-sample test: for each row, attacker-observable measurements (timing, presence, sizes) in an "other partition visited X" condition against "other partition did not visit X", n† trials each, pass when a classifier's accuracy is within ε† of 50% with stated power; the unpartitioned build is the **positive control** that must fail. Add the missing rows, decide the CHIPS ancestor bit, and list the service's shared queues and buffers as bounded channels with their mechanism (per-partition scheduling of decryption and delivery).

### C12. Medium — NW-D5's trust-data rules: "fails closed" contradicts "reduced mode" and P3, distrust semantics are mixed, and the trust-data pipeline is undesigned

**Location:** 6.5 (lines 259–268), NW-17 (line 501), NW-D5 (line 607), Risks table, Q-NW-3.

**Problem.**
1. **Staleness.** "Staleness fails closed on CT and revocation data older than 10 weeks† with a visible reduced mode." Fail-closed means no HTTPS site loads; a reduced mode means it does. Chrome's CT policy stops **enforcing** CT when its log list is about 10 weeks old, because the alternative locks users out (to be confirmed by the fact-checker against Chrome's CT policy page). A user who blocks updates, an air-gapped enterprise, a broken OHTTP relay (NW-D17), battery-saver skipping checks (6.14) or the end of the project would brick every HTTPS load. P3 (graceful degradation) applies; P1 does not require bricking, because a stale revocation set is still better than none.
2. **Distrust semantics.** Mozilla's "distrust after" applies to a certificate's notBefore date; the overlay uses "not after SCT date" constraints (Chrome's Entrust model). Both are allowed in one store without a rule for which applies.
3. **"Replicate decisions announced by at least one of Mozilla, Chrome or Apple"** is a root-program policy choice (Izcal would distrust what only Apple distrusts), not "no root program". Q-SEC-5 asked for a policy **process**; this answers it with a rule but no owner, timeline or appeal path.
4. **OS-level distrust** (a user or enterprise removing or distrusting a root in the OS store) is ignored, while OS-added anchors are honoured: an asymmetry enterprises will hit.
5. **Supply.** A CRLite-style filter must be **generated** from all CT logs and CA CRLs every few hours, or consumed from Mozilla. The spec says only "updated through the trust-data channel" and staffs `hg-trust-build` as a tool; the operational pipeline, its monitoring and its licence (Q-NW-3 covers CCADB only) are missing.

**Evidence.** 6.5 text; NET-R6 and Q-SEC-5 (security scope 6.11, 11.1); HGBA-2 P3; SR R20 (the note's "fail closed ... e.g. 10 weeks" is a proposal, not evidence).

**Proposed fix.** Specify staleness as: CT enforcement and revocation **degrade** (warn, then stop enforcing CT for public roots, keep the last revocation set) with the site panel and `izcal://` status showing it, and a hard fail only for a signed "kill" record from the trust channel; make NW-17 test both states. Define one distrust model (SCT-dated where SCTs exist, notBefore otherwise) and map Mozilla's data into it. Turn the overlay rule into a policy process with an owner (security owner, SEC-D2) and publication. Honour OS-level distrust of a root. Add a trust-data pipeline section (generator or upstream source, cadence, signing under UPD-R1, monitoring, licence), staffed.

### C13. Medium — The M-1 classification of rustls-webpki is asserted, not decided, and its lineage is relevant evidence

**Location:** Section 7 table (rustls-webpki row: "classification confirmed in NW-D6"), NW-D6 (line 608), 6.5.

**Problem.** M-1 part 1 excludes "libraries extracted from browsers" and reserves unclear classes for the owner. Certificate path validation is the core of a browser's certificate verifier (Chrome built its own for that reason, as 2.1 notes). rustls-webpki is maintained by the rustls project, which supports "TLS (named as allowed)", but its predecessor states that "webpki is strongly influenced by mozilla::pkix" (Firefox's verifier), and the current source cites it: "This limit is taken from mozilla::pkix" and several "like mozilla::pkix" comments (`rustls-webpki-0.103.15/src/verify_cert.rs` lines 336, 355–357; `der.rs` line 441; `webpki-0.22.4/README.md`). Influence is not extraction, and the classification may well be "allowed"; but "confirmed in NW-D6" is circular, because NW-D6 is a pending owner decision whose options do not mention this evidence.

**Evidence.** HGBA-2 M-1 part 1; local crate sources listed above.

**Proposed fix.** Put rustls-webpki to the owner as its own NW-D6 item with the lineage evidence and two options: (a) allowed as TLS infrastructure (path building and signature checks only, every policy in `hg-pki`), (b) Izcal's own path builder in `hg-pki`, with rustls-webpki as a differential oracle. State the cost of (b) (a few thousand lines plus a PKI test suite such as the one NW-17 already needs) and drop "confirmed".

### C14. Medium — NW-23 is probably unachievable with unmodified rustls, and the UA/handshake mismatch is itself a bot signal

**Location:** 6.17 (line 398), NW-23 (line 512), NW-D12 (line 614), Risks (bot management, High).

**Problem.** NW-23 requires Izcal's JA4 and HTTP/2 fingerprint to match a mainstream browser family "or differ only in fields that browsers randomize", with no Izcal-unique field. JA4 encodes the cipher-suite list and extension set. rustls implements only AEAD suites and no CBC suites, while Chrome, Firefox and Safari ClientHellos still offer some legacy CBC suites, and rustls lacks some extensions those browsers send (for example ALPS in Chrome, `record_size_limit` and delegated credentials in Firefox) (to be confirmed by the fact-checker against current captures). Unless Izcal adds suites and extensions upstream, its JA4 will match rustls clients, not a browser. Adding cipher suites a browser does not need in order to match a fingerprint is the impersonation NW-D12 rejects. Separately, Stage 1 13.2 sends a Chrome-compatible UA; a Chrome UA on a non-Chrome handshake is a standard bot-management heuristic, so the combination may be worse than either honest choice. Ladybird's case does not isolate the network fingerprint (spec 11.2), so the spec has no evidence either way.

**Evidence.** NW-23 text; Stage 1 13.2 UA row; PP 4.1–4.4; spec 11.2.

**Proposed fix.** Make the measurement the decision: at NW-M1 (HTTP/1.1 and TLS are enough), run the corpus with Izcal's real handshake under (i) the Chrome-compatible UA and (ii) an honest UA, and record challenge and block rates per vendor. Rewrite NW-23 as "Izcal's fingerprint is stable, documented and shared by no single high-risk client population; challenge rate on the corpus ≤ X†", and carry the result into NW-D12 and UA-1.

### C15. Medium — Footprint arithmetic: the content-blocking matcher alone fills FP-1's network line, and NW-35 contradicts NW-D3's reversal trigger

**Location:** 6.14 Memory (line 374), NW-35 (line 529), NW-D3 reversal trigger, 6.1 (shared read-only data).

**Problem.** FP-1 gives the network service ≤ 8 MiB† idle; footprint 6.11 separately charges the content-blocking matcher "≤ 8 MiB† in the network service for the default lists". The spec puts the first container service **and** the system service inside that same 8 MiB, and adds root store, CT log list, revocation filter (CRLite-class filters are megabytes), HSTS preload and PSL, all "mapped once". Whether a shared clean file mapping is charged once or per process depends on footprint 6.2's charged-memory definition, which the spec does not apply. NW-35's "If it fails" says "the per-container split is not the lever", while NW-D3's reversal trigger makes the split exactly the lever on low-RAM devices. NW-33 ("0 wakeups per minute after 60 s†") also conflicts with 6.14's own 10-minute† per-partition expiry timer unless that timer is lazy.

**Evidence.** Footprint 8.2 baseline row and 6.11; FP-1; spec 6.14, NW-33, NW-35, NW-D3.

**Proposed fix.** Give an itemized idle budget per service kind (system, first container, further container) using footprint 6.2's charged metric, with the matcher and trust data as named lines and their sharing stated; propose the FP-1 amendment with those numbers. Align NW-35's failure action with NW-D3. Make partition expiry lazy (evaluated on next use or on the wake grid only while other work is due).

### C16. Medium — Closing connections at freeze and the "no PING" rule change web-visible behaviour without a P2 analysis

**Location:** 6.14 "No keepalives for hidden pages" (line 375), 6.3 (HTTP/2 "No idle PING keepalive"), NW-34.

**Problem.** "A connection used only by frozen pages is closed at freeze" includes WebSocket and WebTransport sessions once their 60 s† "live exchange" reason lapses. Closing a WebSocket is visible to the page (a `close` event on resume, lost server pushes) and breaks chat, mail and collaboration apps that sit idle waiting for the server; footprint 6.8 defines the declared reasons but not this side effect. For pooled HTTP connections, never sending a PING means a request after a network change or long idle can stall on a dead connection until TCP times out; a liveness probe sent **with** a request is not a keepalive.

**Evidence.** Footprint 6.8 declared-reason table; HGBA-2 P2; FP-17's compatibility suite (≤ 1%† breakage).

**Proposed fix.** Keep WebSocket and WebTransport sessions of frozen pages open without wakeups (the service holds them; incoming traffic is buffered or wakes the page under a declared reason), and close them only under memory pressure, through the same path as discard. Allow a PING before reusing a connection idle longer than T† or after a network change, triggered by a request, never by a timer. Add both to FP-17's background compatibility suite.

### C17. Medium — Several reversal triggers name measurements that no target performs

**Location:** NW-D2, NW-D4, NW-D7 reversal triggers (lines 604, 606, 609).

**Problem.** NW-D2's trigger is "NW-37 shows the per-connection log costs more than 2%† of network-service CPU", but NW-37 measures replay mismatches, not CPU. NW-D4's trigger is "connection-setup cost > 1 ms† p95", but no target measures mint latency (the spec's "tens of microseconds" is unmeasured, and Do53 port randomization needs a fresh broker-minted UDP socket **per query**, which multiplies broker calls). NW-D7's "> 2%† p75 page-load time over (b)" has no target ID or corpus definition. A trigger that no measurement feeds cannot fire.

**Evidence.** Section 8 targets; owner direction (staging and reversal only on measurement).

**Proposed fix.** Add targets: recording overhead (network-service CPU and bytes per page load, recording against not); mint latency p50/p95/p99 per OS, mints per page load (including DNS), and broker CPU, with the broker side on its own sequence so UI work cannot delay it; NW-D7's cost on the corpus as a named target at NW-M2.

### C18. Medium — Staffing and duration are asserted rather than derived, omit whole components, and double-count with the security scope

**Location:** 9.3 (lines 572–596), 9.2 critical path (line 570).

**Problem.**
1. **Not derived.** The JS spec's format gives per-track scope and a stated basis; this spec says "first-principles estimates from each track's deliverables" but shows no derivation, although it has the inputs (43 RFCs, 2,336 MUSTs, Fetch's 1,362 steps, the oracle suites to build).
2. **Omissions.** WebRTC transport including DTLS (C7); the content-blocking engine (C20); the trust-data generation pipeline (C12); the push client (C21); the OS-adapter process (C4); and own deflate, Brotli and zstd decoders (NW-D6), which sit inside the 6–8-engineer Fetch track with all of Fetch, CORS, ORB, COEP, LNA, mixed content and service-worker routing, and which the fonts spec also depends on (Q-TX-11).
3. **Overlap.** The security scope's 9.2 already costs "network protocol fuzzing, resource caps, LNA suite" (5 engineer-months) and an on-call rotation; this spec adds a verification track and a 2–3-person security response track. Which document owns which is not stated.
4. **Arithmetic.** Year one is 17 engineer-years, but a ramp from 15–18 at start to about 30 at month 12 averages about 23; the JS spec's 30 for a 20–25 → 40 ramp is consistent, this one is not.
5. **Critical path.** "NW-M0 → NW-M1 → Stage 1 M2 (WPT)" assumes WPT waits for networking, but WPT's testharness needs the JS engine and DOM, and JS-M1 is at 15–21 months against NW-M1's 9–12. Networking is not on that path.

**Evidence.** JS spec 9.3; security scope 9.2; fonts spec Q-TX-11 and line 233; Stage 1 8.1–8.2.

**Proposed fix.** Derive each track from its deliverables (oracle cases to write, MUSTs per engineer-month from a stated basis, measured velocity at the first review), add rows for the omitted components, reconcile the fuzzing, response and on-call rows with security scope 9.2 in one table, fix year one, and replace the critical-path claim with the actual dependency (NW-M1 is needed when the JS engine and DOM can run testharness, and earlier only for `hg-net` unit and oracle work).

### C19. Medium — The system service, routes and NW-36 contradict each other

**Location:** 6.12 Routes (line 356), NW-D17 (line 360), NW-21 (line 510), NW-36 (line 530).

**Problem.** 6.12 says that under a non-direct route "every request the container makes, including ... trust data ... uses the route", and NW-21 lists "trust data" among packets that must not leave by another path; but NW-D17 says only the **system** service fetches trust data, updates and filter lists, so a container never makes those requests. The route and proxy configuration of the system service itself (OS proxy, enterprise proxy, captive portal) is unspecified. NW-36 allows "only update and trust-data checks" with default settings, but content blocking is on by default (Stage 1 Section 6), so filter-list updates happen; and B10 allows security classifiers. Either NW-36 fails by design or the list is wrong.

**Evidence.** Spec 6.12, NW-D17, NW-21, NW-36; Stage 1 Section 6 content-blocking row; HGBA-3 B10, AG-0.

**Proposed fix.** State that the system service follows the OS proxy configuration and never a container route; remove "trust data" from NW-21's container list (keep AIA fetches and favicons); enumerate NW-36's allowed endpoints (updates, trust data, filter lists, security classifiers if enabled), each on the published schedule.

### C20. Medium — The content-blocking engine has no owner or design

**Location:** 6.7 (line 282, "content-blocking"), 6.1 (automata shared read-only), NW-22, 9.3.

**Problem.** M-1 2.3.0 drops the `adblock` crate; footprint 6.11 specifies the matcher's footprint, compilation in a utility process and the automaton file, and the coding standard lists `hg-content-block` as tier P; Stage 1 M7 requires content blocking. This spec runs the matcher in `hg-net` but specifies neither its semantics (which list syntax is supported, `$third-party`, `$domain`, redirect and removeparam rules, exceptions, per-container list sets), its interaction with partitioning (is a rule's "third-party" the frame site or the top-level site?), nor anyone who builds it. Footprint 6.11 says "list governance ... belong to the shell and policy work, not this spec", so the matcher itself falls between three documents.

**Evidence.** HGBA-2 M-1 part 1 (adblock excluded); footprint 6.11; coding standard tier P list; Stage 1 8.1 M7.

**Proposed fix.** Either own it here (a Section 6.x for request-blocking semantics, anchored to a written rule-syntax page built from public list documentation as black-box observations, with a staffing row) or name the spec that will, and list it in 9.5.

### C21. Medium — Missing surfaces: downloads, HTTP authentication prompts, client-certificate UX, Web Push, enterprise policy, CSP placement

**Location:** whole spec; 6.1 (service exits "after its last tab and download close"), 6.7 (CSP), 6.8 (auth cache row).

**Problem.** Each of these needs a network-service design that the spec leaves out or mentions in passing:
- **Downloads.** Who writes the bytes (a broker-granted writable file per download, IPC-R1 and SH-R9), when Mark-of-the-Web and quarantine are applied (SH-R6), resumption with Range requests, the download's partition and credentials, and behaviour when the owning container's service restarts.
- **HTTP authentication.** Basic/Digest prompts (who draws them: CON-R2's prompt renderer?), blocking prompts for cross-origin subresources (a phishing vector), proxy 407 prompts per container, where entered credentials live.
- **Client certificates.** Selection UI through CON-R1, auto-selection policy, and the TLS handshake waiting on a prompt (a recorded decision under NW-D2).
- **Web Push** (Stage 1 13.2: Stage 2, Izcal-hosted autopush). The client channel is a long-lived connection with heartbeats in the system service, which NW-33, NW-34 and NW-36 currently forbid.
- **Enterprise policy.** Policies for DoH, QUIC off, ECH, proxy lock, auth allowlists (C4), LNA defaults and trust anchors are referred to but no policy surface is defined.
- **CSP placement.** The network service "re-checks the CSP ... directives it is given **by the broker**", but CSP arrives in response headers the broker must not parse (IPC-R15) and in `<meta>` elements only the renderer sees; the source of truth is unspecified, and the re-check's security value against a compromised renderer of the same site is not argued.

Later-stage items to note now so interfaces allow them: **extensions** (Stage 2, Wasmtime) need request interception through the same scoped, audited IIP path as agents (6.16); **Android** (S-8) needs a socket-passing and datapath design (isolated processes, per-network binding, Doze, Private DNS as OS configuration); **IPv6-only** networks need NAT64 prefix discovery (RFC 7050) for IPv4 literals, not only DNS64-style synthesis.

**Evidence.** Security scope 6.6a CON-R1/R2, 6.14a SH-R6–R9; IPC-R1, IPC-R15; Stage 1 13.2 (service workers and push), Section 6 (extensions Stage 2), 9 (Android Stage 2).

**Proposed fix.** Add short sections for downloads, HTTP authentication and client certificates (with the CON-R paths and C4's adapter), a push-client section with an explicit exception in NW-33/34/36 tied to a user grant, a policy surface list, and a CSP rule: the network service parses CSP from the navigation response it delivered (the one header parser), `<meta>` policies are renderer-only, and the network-side re-check is stated as defence in depth with its limit.

### C22. Medium — Several targets cannot fail, or fail with no consequence

**Location:** NW-9, NW-20, NW-30, NW-37 (see C1), NW-19 (see C11).

**Problem.** NW-9 checks that substrate timers fire at their deadline under a virtual clock, which is true by construction; its useful half is "0 other timers" (a lint). NW-20 passes on the one published attack (C10). NW-30's failure action is "track performance work". NW-28 and NW-29 are measured "in the simulator" against Izcal's own simulated peers, which share Izcal's interpretation of the RFCs.

**Proposed fix.** Fold NW-9 into a lint target; add lab runs against real stacks to NW-28 and NW-29; give NW-30 a decision consequence at the Stage 1 exit review (an owner decision on which performance work precedes which feature). C10 and C11 cover NW-19 and NW-20.

---

## Low

### C23. Low — Multipath and WebTransport use inconsistent gates

**Location:** 6.4 (line 241), 9.1, NW-D13.

**Problem.** WebTransport is built against a pinned draft and enabled after interop; multipath QUIC, already in the RFC Editor queue, waits for its RFC. Both are correctness gates, but they differ without a stated reason.

**Proposed fix.** Apply one rule: build against the pinned document when its wire format is in last call or later, and enable after interop with two independent implementations.

### C24. Low — WireGuard primitives may need a crypto crate besides aws-lc-rs

**Location:** Section 7 (aws-lc-rs row: "`hg-route-wg` primitives").

**Problem.** WireGuard uses BLAKE2s, which AWS-LC may not expose through aws-lc-rs (to be checked). A second crypto crate is in the allowed class but should be named and pinned.

**Proposed fix.** List the primitives `hg-route-wg` needs and the crate that provides each.

### C25. Low — DNS keyed by (T, C, F, X, R, n) multiplies queries without a measured cost

**Location:** 6.6 (line 274), NW-D7.

**Problem.** Keying the DNS cache by frame site and nonce multiplies queries (and, with Do53, broker mints per query, C17). NW-D7's reversal trigger covers only connection state.

**Proposed fix.** Include DNS query count and resolution latency in NW-D7's NW-M2 measurement.

### C26. Low — PAC sees every HTTP URL of the container

**Location:** 6.12 (line 354), partition row "(C, R)".

**Problem.** A PAC script keeps global state across calls, so a WPAD-supplied script on a hostile network sees every `http:` URL (and every HTTPS host) in the container. The spec strips HTTPS paths but does not restrict WPAD.

**Proposed fix.** WPAD only when the OS is set to auto-detect **and** the network is not marked public, and show the PAC source in the site panel.

### C27. Low — "Matching what Chrome made default" is used as a reason

**Location:** 6.7 HTTPS-First (line 286).

**Problem.** Under M-1 part 3 another engine's behaviour is evidence, not a template; the reason for HTTPS-First's scope should be first-principles (P1, P2 measured on the corpus).

**Proposed fix.** State the reason and cite Chrome as evidence that the scope is deployable.

### C28. Low — `hg-netsim` tier T versus replay

**Location:** 6.18 table (`hg-netsim` "T, linked by the simulation binary only, Q-CS-1").

**Problem.** Consistent with Q-CS-1 for simulation, but NW-D2's replay needs a "recorded-transport mode in `hg-net-io`" in `hg-replay-main` (tier R binary linking tier P crates). The table should say where that mode lives and that it is not `hg-netsim`.

**Proposed fix.** Name the crate (for example `hg-net-replay`, tier P) and add it to the graph lint's binary list.

### C29. Low — The amendment list in 9.5 is incomplete

**Location:** 9.5.

**Problem.** Several affected documents and sections are missing (see the reconciliation list below), for example the security scope's 6.3 assets row and 6.3a index (blast radius per container), IPC-R1's handle kinds, and footprint 6.8 (closing at freeze).

**Proposed fix.** Replace 9.5 with the reconciliation list below, updated after the fixes.

---

## Reconciliation: amendments other specs would need

Items marked **new** are not in the spec's 9.5; the others refine a 9.5 row.

| Document | Section or ID | Amendment | From |
|---|---|---|---|
| Security scope 0.3 | 6.3 assets table, 6.3a index, 6.5 network-service threat model | **New.** Network service row: "per (container) [or per (top-level site, container), C8]"; compromise yields that container's sessions; entry points without neqo and hickory-dns; residuals: cache poisoning until the next epoch (C5), use of the container's LNA grants (C3) | C3, C5, C8 |
| Security scope 0.3 | 6.6 IPC-R1 | **New.** Handle kinds: connected TCP socket, connecting TCP socket, connected UDP socket, bound UDP socket (WebRTC), producer-writable consumer-read-only shared memory (per-OS definitions) | C3, C6 |
| Security scope 0.3 | 6.6 IPC-R2 | **New.** Broker-issued LNA tickets as the one sanctioned way for a frame's grant to reach a broker check through a service | C3 |
| Security scope 0.3 | 6.3, 6.7 | **New.** OS-adapter utility process (resolver fallback, SSPI/GSSAPI, client-key signing) with its sandbox row; the broker does none of these | C4 |
| Security scope 0.3 | 6.7 network-service row | No socket creation; sockets as typed handles; no certificate-store access (NW-D5) | spec 9.5, C3 |
| Security scope 0.3 | 6.10a WebRTC | **New.** Point to the networking spec's WebRTC section; socket model per C3; DTLS ownership | C7 |
| Security scope 0.3 | 6.11 NET-R5, NET-R6, NET-R7; Q-SEC-5 | Dependency list rustls, aws-lc-rs, rustls-webpki (subject to C13); NET-R6 per NW-D5 with the degrade-not-brick staleness rule and a named policy process; NET-R7 enforced at the broker with tickets | spec 9.5, C3, C12, C13 |
| Security scope 0.3 | 9.2 costs | **New.** Reconcile "network protocol fuzzing, resource caps, LNA suite" and the on-call rotation with the networking spec's verification and response tracks | C18 |
| Coding standard 0.3.1 | Tier table, `tools/tiers.toml` | Add the 6.18 crates, WebRTC crates, the replay crate (C28) and `hg-cookies` as linked by renderers (tier P declared) | C6, C7, C28 |
| Coding standard 0.3.1 | CS-AS-1 list | **New.** `EntropyPurpose` values for protocol identifiers must come from `Entropy::crypto()`; a lint flags seeded streams feeding wire identifiers | C2 |
| Stage 1 design | Section 1 process map | Network service per container (or per (T, C)); proxy-resolver utility, route process, OS-adapter process | C4, C8 |
| Stage 1 design | 3.1 | `NetTransport` mints through the broker; **no** `EntropyPurpose` for QUIC connection IDs or tokens (replace the 9.5 row) | C2 |
| Stage 1 design | 3.2, 3.4 | Network recordings: start snapshot, fresh connections or a dedicated service, typed datagram records | C1, C2 |
| Stage 1 design | 4.1 | Remove `document.cookie` from the sync allowlist only once NW-D10's ordering and origin-keyed region are specified | C6 |
| Stage 1 design | 8.2, 8.3 | Networking not on the WPT critical path; staffing per the corrected 9.3, with WebRTC and content blocking rows | C18, C20 |
| Stage 1 design | 13.2 | Certificates per NW-D5 (with C12's changes); UA row cross-referenced to the NW-M1 fingerprint measurement | C12, C14 |
| HGBA-2 | I-1 (b) | Add F, R and n to the key; name the static global exceptions; note that limits are bounded, not closed, channels | C10 |
| HGBA-2 | 4.7 | Rewrite from this spec, including WebRTC DTLS ownership | C7 |
| Footprint spec | FP-1, 8.2 | Itemized network-service lines (matcher, trust data, per service kind) | C15 |
| Footprint spec | FP-D16 | HTTP cache admitted with a **security epoch** binding and the crash-discard rule kept; not format-version binding alone | C5 |
| Footprint spec | 6.8 | **New.** WebSocket and WebTransport sessions of frozen pages stay open without wakeups; liveness PING only on request | C16 |
| Footprint spec | 6.18.1 | Connection pools and socket budgets **bounded** (not closed); network-service shared queues and receive buffer bounded; HTTP-cache eviction bounded | C10, C11 |
| DevTools spec | DT-D2, DT-D5, Q-DT-5 | Network recording unit and start state consistent with DT-D2; "qlog from `hg-quic`"; live link model per granted partition (proposal) | C1, spec 9.5 |
| JS engine spec | 9.5 | The proxy-resolver process as a jitless embedder | spec 9.5 |
| Fonts and text spec | Q-TX-11 | Answered by NW-D6: `hg-compress` is Izcal's own, owned by the networking spec's staffing | C18 |

---

## Summary of the High findings

1. **C1**: NW-D2 has no start state or recorded unit, so a recording cannot replay the cache, cookies or pooled connections it claims to replay. It would also capture other tabs' traffic, which breaks DT-D2's per-process, per-(site, container) replay rule. NW-37 checks renderer hashes that do not depend on network replay.
2. **C2**: the decryption boundary leaves out QUIC header fields, rejected packets and stateless resets, ECN, and crypto values the client sends, so QUIC recovery does not replay as claimed. Connection IDs drawn from a seeded stream break RFC 9000's unlinkability.
3. **C3**: the socket broker checks LNA against a frame identity and purpose that the network service supplies. That contradicts IPC-R2: any LNA grant in a container becomes usable by every site in it. WebRTC's unconnected UDP gets past per-address checks.
4. **C4**: the brokered OS calls (getaddrinfo fallback, SSPI/GSSAPI, client-key signing) parse network bytes inside the unsandboxed broker (IPC-R15). They also make it an ambient NTLM/Kerberos and signing oracle.
5. **C5**: the HTTP cache's MAC key is held by the process that writes the cache. Crash survival (NW-31) and format-version binding contradict FP-D16 and allow persistent cross-origin cache poisoning.
6. **C6**: the `document.cookie` region is keyed per site, so sibling-subdomain host-only cookies reach origin-keyed renderers, against FP-D8. It also does not fit IPC-R1's sealed read-only kind or IPC-R7.
7. **C7**: WebRTC transport (ICE, DTLS, SRTP, SCTP), which the security scope places in the network service, has no design, owner, DTLS plan, partition row or milestone.
8. **C8**: NW-D3's own "no sharing lost" argument also supports one service per (top-level site, container). Option (c) is rejected without analysis, and the per-container split is deferred to NW-M4 for no stated first-principles reason.
9. **C9**: NW-D18 lets Clear-Site-Data reach other containers. The rule contradicts itself and creates a cross-container linking channel.
