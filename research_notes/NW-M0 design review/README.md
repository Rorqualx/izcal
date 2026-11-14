# NW-M0 design review: the agenda

Opened 13 November 2026, for the networking track's M0 milestone
(`reports/Izcal networking.md` §9.2). **Opening the review with its
agenda fixed is M0's *start*; completing it is M0's exit.**

NW-M0 differs from the other four M0s in two ways that shape this
agenda:

- **Nothing waits for it.** NW-M0's "Needed by" is Stage 1 M1 and the
  critical path runs NW-M0 → NW-M3 → NW-M5 in parallel with the
  engine; §9.2 records that "NW-M1 lands at 9–12" months against
  JS-M1's 15–21, so the track has about twelve months of float. It
  blocks no other M0 and no other M0 blocks it.
- **Its review is the widest of the five.** The row asks for a "design
  review fixing **every state machine's interface** (HTTP/1–3, QUIC,
  TLS and its handshake core for QUIC and DTLS, DNS, HEv3, ICE, DTLS,
  cache, cookies, matcher) **and the partition key type**". That is
  thirteen interfaces and one type, and §9.1 says why they are fixed
  together: "the recording boundary, recorded unit and decision-point
  log … shape every state machine's interface".

---

## 1. What is already built, and therefore reviewed rather than designed

| Built | Section | What the review checks |
|---|---|---|
| The budget (`hg-net-wire`) | §6.2 | that charging at allocation time is reachable from every peer-sized path, and that the roll-up's three levels are the right three |
| The named limits (NW-15) | §6.2 | that the eleven are all of them, and the values against a crawl (Q-NW-9) |
| The checked kernels | §6.2, G33 | varint, packet number, ACK ranges, flow control — proved total by Kani; the review checks the *set*, not the proofs |
| `hg-netsim`'s links and resolver | §6.21 | that the parameters are the ones a protocol bug needs, and that a run stays a function of its seed |
| `hg-crypto`'s interfaces and pins | §6.5, NW-D23 | **the pins**, one by one — see §3.2 |
| `hg-tls`'s transition table | §6.5 | that it is RFC 8446 Appendix A.1 and nothing more |
| `hg-crypto-check` | §6.21, NW-50 | that the gate can detect what it is meant to — see §3.3 |

Two measured things belong in the record rather than in a later report:

- **Wycheproof, at the pinned set:** HMAC-SHA-256 **174 of 174**,
  HKDF-SHA-256 **86 of 86**, with the other 111 algorithms classified
  rather than counted. 145,424 cases over 359 files.
- **The TLS table:** 9 states against 9 messages is 81 pairs, of which
  **10 are transitions and 71 are alerts**, each checked.

---

## 2. The central item: the partition key type

> the partition key type

It is one type and it is the hardest thing in the track, because
**four different consumers have to agree on it and three of them are
in other specs.**

1. **The network key.** §6.8's full key, which the connection pool,
   the DNS cache, the HTTP cache, cookies, Alt-Svc and the
   broken-HTTP/3 state are all keyed by.
2. **The renderer's projection**, `(T, C, X, n)`. §6.8: "A renderer's
   in-process caches … and its glyph atlas key by the projection",
   because "a renderer cache coarser than this key would re-create by
   timing the leak the network key closes".
3. **The text stack's caches.** The fonts and text spec's TX-D16 keys
   shaped segments, raster entries and variation instances by the same
   projection.
4. **The content GPU process.** §6.8 notes that P18's projection
   "assumes F fixed per process, which holds because content GPU
   processes are keyed like renderers (GR-D2, G68)".

So the review has to fix one type whose projection is used by crates
in three other tracks, and the thing to decide is not its fields but
**who may construct one** and **where the projection is taken**. A
projection taken in two places is two projections.

---

## 3. Items the review must decide, in their own right

### 3.1 Thirteen state-machine interfaces

The row names them: HTTP/1.1, HTTP/2, HTTP/3, QUIC, TLS (and its
handshake core as used by QUIC and DTLS), DNS, Happy Eyeballs v3, ICE,
DTLS, the HTTP cache, cookies, and the content-blocking matcher.

They are fixed together because NW-D1 gives them one shape —
`on_bytes`/`on_datagram(now, data, metadata)`, `on_timer(now)`,
application calls, each returning a bounded list of actions — and the
value of one shape is lost the moment two of them differ. What the
review decides per interface:

- the action list's type, and its bound;
- which events the layer above sees, and which stay inside;
- where the recording boundary is: §9.1 puts "the recording boundary,
  recorded unit and decision-point log" at NW-M0 precisely because
  they "shape every state machine's interface";
- what a protocol error is, as a value, and which of them close a
  stream rather than a connection.

**`hg-tls`'s table is the first of the thirteen and is built**, so it
is the worked example the other twelve are reviewed against.

### 3.2 `hg-crypto`'s pins, one by one

> `hg-crypto`'s interfaces with every pin resolved and licence-checked

`crates/hg-crypto/pins.toml` resolves thirteen primitives and its
tests check that every one has a row, that every row names a licence
HGBA-2 2.3.2 allows, and that no pinned crate is a dependency before
its code is here. What the review decides is the content of three of
those rows, because they are the ones where "Izcal writes it" and "pin
a reviewed implementation" pull in different directions:

| Row | The question |
|---|---|
| `aes-gcm` | pinned today. A table-driven AES is a cache-timing leak and a constant-time one needs per-ISA intrinsics. NW-D23 has the owner writing P-256, P-384 and RSA verification; it does not say AES. |
| `ml-kem-768` | pinned today, for X25519MLKEM768 at NW-M1. A lattice KEM is new enough that a published implementation is worth more than a first attempt — but it is also the one with the least review history. |
| `sha-384` | Izcal's, status `pinned`. Same construction as SHA-256 over 64-bit words, so it is a day's work; the review decides whether it waits for the cipher suite that needs it. |

### 3.3 What NW-50's gate is, exactly

§9.1: "Secret-dependent code in `hg-crypto` merges only after
`hg-crypto-check` passes on every target ISA (G36, NW-50)." Two halves
exist and only one is built:

- **The statistical timing test** is built, with its control. Measured
  on this machine: the planted short-circuit comes out at ratio
  1,539.6 and `ct::eq` at 0.0, against a threshold of 0.25. What the
  review decides is whether a ratio is the right statistic and what
  the threshold should be on the slowest target.
- **The binary taint check** is not. It reads a compiled object and
  follows secret-derived values into branches and addresses, so it
  needs a disassembler per ISA. The review decides whether it is
  written per ISA or over one IR, and that decision sets whether
  "every target ISA" means four tools or one.

**The gate exists before the code it gates, which is the point.** A
timing test written after the first primitive is a test written
against that primitive.

### 3.4 Items with their own measurement

| # | Item | Needs |
|---|---|---|
| 3.4a | Izcal's ClientHello design, and what it looks like to a JA4 fingerprinter | Q-NW-1 |
| 3.4b | Whether the corpus still needs TLS 1.2, and any suite outside the AEAD set | Q-NW-13 — a suite is added "only on that evidence and with an owner decision" |
| 3.4c | Windows socket passing into an AppContainer without `internetClient` | Q-NW-2 — on rig253 |
| 3.4d | GRO and in-place decryption receive cost on Linux | a Linux rig |
| 3.4e | The limits' values, against a crawl | Q-NW-9; until then every value in `limits.rs` is a placeholder† and says so |

### 3.5 The ACVP runner

NW-M0's row names "Wycheproof **and ACVP** runners". Wycheproof is
built and green. ACVP is not, and the reason is worth recording rather
than fixing by stubbing: its published vector sets cover AES, SHA,
ECDSA, RSA and the SP 800-56 key agreements, and `hg-crypto`
implements none of them yet, so a runner would have nothing to run and
nothing to be wrong about. It lands with the first primitive it covers.

---

## 4. Cross-track items, each with its counterpart named

### 4.1 The projection — **with style, fonts and text, and graphics**

§2's items 2 to 4. The style spec's renderer caches, the fonts and
text spec's TX-D16 caches and the graphics spec's content GPU
processes all key by `(T, C, X, n)`. The review names a counterpart
per track and records that the projection is **taken in one place**.

### 4.2 `hg-url` — **from the style spec**

§6.2: "`hg-url` (the browser's one URL parser, owned by the HTML and
CSS parsing and style spec since the reconciliation)". It is built and
passes WPT's url suite 904/904. G31 makes it the only URL parser, and
this track's job is to use it rather than to own it.

### 4.3 `hg-compress` — **to the fonts and text spec**

NW-M1 owes Brotli to TX-M1 for WOFF2 (§9.2's NW-M1 row). Not NW-M0's,
but recorded here because the obligation is easy to discover late.

### 4.4 PAC in a utility process — **with the JavaScript engine**

§6.13: PAC is JavaScript and IPC-R10 forbids an interpreter in a
network service, so it runs in a proxy-resolver utility process "with
Izcal's jitless engine and no DOM". That makes a *networking* feature
depend on the engine's jitless mode, which is JS-M1's. The review
records the direction.

### 4.5 The OS-adapter process — **with the OS integration spec**

§6.1 moves "OS calls that consume network data (resolver fallback,
SSPI and GSSAPI, client-key signing)" out of the broker into a
sandboxed OS-adapter process. M1's exit recorded that the pattern is
decided and `policy.hgidl` exists only as a codegen golden, with no
`OsAdapter` process kind. The review decides when the kind is added,
which is a recording-schema version.

---

## 5. What blocks this review's completion

> the root program's charter and its lead being recruited

**There is no root-program lead.** That is **D-S8**, which Stage 1
§13.1 records as "Deferred by the owner (14 October 2026)".

As with the other M0s, this is not a reason to leave the review
unopened — opening it with the agenda fixed is what "start" means —
but it is a reason it **cannot be completed**. What is unusual here is
how *little* it blocks:

1. **One clause of the row needs the lead**, and §§1–4 do not. Every
   other item above is engineering.
2. **The charter is writeable now.** §6.5.2 already states what the
   program does — review every root, publish each decision with its
   evidence, check CT for misissuance — and NW-53 is its credibility
   gate. What needs a person is the reviewing, not the statement.
3. **NW-D24 already covers the gap.** Its trigger was decided on 19
   October 2026: if the program has not passed NW-53 at the first
   public release, that release ships on the OS trust stores, read as
   an OS interface, "and the site panel says so". So the absence of a
   root program delays Izcal's own store and does not block shipping,
   which is a different shape of block from LY-M0's.

---

## 6. Provenance

NW-M0's row does not name a provenance review. §7 is the track's
standing record, and revision 0.2's fact-check is in the revision
note: 94 claims and 82 external links checked.

The review's provenance job is the narrow one §7 implies, over what
§1 built: that `hg-net-wire`'s kernels are from RFC 9000's own prose
and worked examples; that `hg-crypto`'s SHA-256, HMAC and HKDF are
from FIPS 180-4, RFC 2104 and RFC 5869 and not from any
implementation; that `hg-tls`'s table is Appendix A.1's diagram; and
that `hg-netsim`'s parameters are Izcal's own names.

---

## 7. The interlock, in one place

| This track owes | To | Clause |
|---|---|---|
| The partition key type, and its `(T, C, X, n)` projection | **ST**, **TX**, **GR** | §6.8; P18 |
| `hg-net-wire`'s header, Structured Fields and MIME parsers | **ST** (renderers) | §6.2, G31; NW-M1 |
| `hg-crypto`'s primitives | the update client, the trust pipeline, **WebCrypto** (WC-D1) | §6.21: in L2s so one implementation is linked |
| `hg-compress`'s Brotli | **TX** | NW-M1, for WOFF2 at TX-M1 |
| The socket broker's policy and LNA tickets | the broker | §6.1 |

| This track is owed | By | Clause |
|---|---|---|
| `hg-url` | **ST** | §6.2, G31; built, 904/904 on WPT |
| `hg-encoding` | **ST** | NW-M1's row |
| A jitless engine with no DOM, for PAC | **JS** | §6.13, IPC-R10; JS-M1 |
| The OS-adapter process kind | **OS integration** | §6.1; a recording-schema version |
| A **root-program lead** | **D-S8** | §5 |

---

## 8. The measurements this review takes as input

| Question | Due | What it feeds |
|---|---|---|
| Q-NW-1 | NW-M0 | the ClientHello's shape and its JA4 fingerprint — §3.4a |
| Q-NW-2 | NW-M0 | Windows socket passing into an AppContainer — §3.4c |
| Q-NW-9 | before freezing | the limits' values, against a crawl — §3.4e |
| Q-NW-12 | NW-M0 onward | what Chrome Stable actually runs sandboxed, per OS, black-box |
| Q-NW-13 | NW-M0 | whether the corpus needs TLS 1.2 — §3.4b |

Three of the five need a machine rather than a person: Q-NW-2 is
rig253's, the GRO and in-place decryption measurement is a Linux
rig's, and Q-NW-12 is a black-box observation of shipping browsers.
**They are the part of this milestone that work can finish and funding
cannot hurry**, and the review should order them first.
