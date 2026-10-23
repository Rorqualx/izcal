# Izcal WebCrypto: Izcal interfaces

Compiled 2026-10-20. Authored by the synthesis author (the strand's researcher was lost to API rate limits; every source here is an Izcal document read directly, so confidence is High throughout). Quotes are verbatim from the cited section.

Confidence: **High** = primary source read and the claim checked in its text.

## 1. HGBA-2 (browser architecture, frozen 2.3.3)

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 1.1 | **M-1 part 1** allows cryptography primitives from MIT-licensed crates as generic infrastructure; everything shipped must be MIT (alone or as one choice of a dual licence); anything else Izcal builds itself. "Allowed dependencies are generic infrastructure only: … cryptography primitives from MIT-licensed crates (the TLS protocol itself is Izcal's own)" | M-1 (2.3.2) | HGBA-2 §2 | High |
| 1.2 | **M-1 part 3** evidence rule covers every third-party project's source code: "no spec or research note links to its code files, headers, commits, diffs or code reviews, and no constant, data layout or algorithm is read from them. A prose document hosted in a repository … is prose when no code file is opened. A value only its source states is measured black-box or set from first principles (marked †)." Standards prose (W3C, RFC, NIST) is allowed evidence | M-1 part 3 (2.3.3) | HGBA-2 §2 | High |
| 1.3 | **M-1 part 2**: Web-visible behavior implemented from the normative specification at a pinned revision, every hand-written algorithm carrying a spec anchor (ANC-1) | M-1 part 2, ANC-1 | HGBA-2 §2, §6 | High |
| 1.4 | **P1** outranks P4: "Treat web content as hostile. Renderers hold zero ambient authority. Never co-locate cross-site data in one address space." A WebCrypto design that puts page key material in a shared service must survive this ranking test | P1, P4 | HGBA-2 §2 | High |
| 1.5 | **I-1**: one locked renderer per (site, container); renderers are sandboxed with zero ambient authority. The broker is the trusted process: "UI, navigation & session history, policy, permission store, capability grants" | I-1, §3 diagram | HGBA-2 §3, §4.1 | High |
| 1.6 | 4.7 sets the crypto posture: CPU crypto instructions live inside the network service; hybrid X25519MLKEM768 is the default; "crypto agility" required. Superseded crate choices (rustls, neqo) are rewritten from the networking spec (A-H2S-19, A-H2S-20) | HGBA-2 §4.7 | HGBA-2 §4.7; ledger A-H2S-19/20 | High |
| 1.7 | 4.8: Rust default; "Every `unsafe` block and FFI boundary is inventoried and owned by a named reviewer" | §4.8 | HGBA-2 §4.8 | High |
| 1.8 | Non-goal: "It does not put post-quantum or any other cryptography on quantum hardware" — PQ is software ML-KEM on classical CPUs | §8 | HGBA-2 §8 | High |

**Implications:** the WebCrypto spec must implement the API surface itself (Web-visible behavior, M-1 part 2), may use MIT crypto crates for primitives only (M-1 part 1), may cite NIST/RFC prose freely, and must argue placement against P1/I-1, not convenience.

## 2. HGBA-3 (future browser, building blocks)

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 2.1 | **B1**: "No engine code calls the OS clock, OS threads, OS randomness or sockets directly. All of them go through injectable interfaces with two implementations: production (real OS) and simulation (seeded …)". A recorded page plus a seed must replay bit-identically (DET-1 ≤ 1%† substrate cost) | B1, DET-1 | HGBA-3 §3 | High |
| 2.2 | Stage 1 3.1 fixes the randomness interface: `Entropy::stream(purpose)` returns "a named, seeded stream so adding a consumer does not perturb existing ones"; `Entropy::crypto()` — "simulation: seeded; production: OS CSPRNG only". G3: only `hg-substrate-os` may name `rand::thread_rng` or `getrandom` | Stage 1 §3.1, G3 | Stage 1 design §3.1 | High |
| 2.3 | The coding standard's open question Q-CS-1 (answered at M1 per ledger A-S1-31) forbids a seeded `Entropy::crypto()` in any shipping binary — seeded crypto randomness exists only under `hg-substrate-sim` | Q-CS-1; A-S1-31 | networking critique; ledger | High |
| 2.4 | **B2**: all IPC is typed, sequence-affine, generated from IDL; holding an endpoint is the authority to call it | B2 | HGBA-3 §3 | High |
| 2.5 | **B9/B10**: the agent principal exists only with AI on; with AI off, nothing is computed for agents (AG-0/G4: "off means not loaded") | B9, B10, G4 | HGBA-3 §3 | High |

**Implications:** WebCrypto randomness is a named consumer of `Entropy::crypto()`; `crypto.getRandomValues` outputs are recorded as **drawn values** (never a seed), replay replays them; no production API may set a CSPRNG seed; any agent-facing crypto surface is compiled behind feature `ai`.

## 3. Networking spec 0.3.3 (`hg-crypto`'s owner today)

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 3.1 | **G36**: "Only `hg-crypto` implements a cryptographic primitive or depends on a cryptography crate; `cargo deny` and an `ast-grep` check reject others. Secret-dependent code in `hg-crypto` is marked and passes `hg-crypto-check` on every target ISA before merging" | G36 | NW §6.21 | High |
| 3.2 | **NW-D23** (owner decision, 19 Oct 2026): `hg-crypto` = hashes, HMAC, HKDF, AES-GCM, ChaCha20-Poly1305, BLAKE2s, ML-KEM from MIT-dual RustCrypto crates; X25519, Ed25519, P-256, P-384, RSA verification, HPKE written by Izcal on `fiat-crypto`; Izcal's constant-time helpers; secret types. Reversal trigger: per primitive, "a pinned crate fails review … : Izcal writes that primitive" | NW-D23 | NW §9.4 | High |
| 3.3 | `hg-crypto` sits in **L2s** "so that update and trust-data verification, and WebCrypto if its owner chooses (Q-NW-14), link one implementation" | NW §6.21 crate table | NW §6.21 | High |
| 3.4 | **The constant-time surface is small, which is why Izcal can own it** (NW §6.5). Secret-dependent today: ephemeral key generation and ECDH (X25519, P-256), ML-KEM decapsulation, the AEADs, HMAC, HKDF, and ECDSA P-256 signing (RTCCertificate). "Everything else is verification on public data … written for correctness and speed, not constant time" | NW §6.5 | NW §6.5 | High |
| 3.5 | **NW-50** (constant time): 0 secret-dependent branches/indices/variable-time instructions in the named set, binary taint check on x86-64 and AArch64 builds, statistical timing tests (Welch's \|t\| < 4.5 after 10⁷† measurements per primitive) on each reference CPU. **NW-49**: 100% Wycheproof, ACVP where present, RFC vectors; differential fuzzing against two black-box implementations. **NW-51**: AEAD throughput ≥ 80%† of the fastest black-box library. **NW-52**: external audit of `hg-crypto`, `hg-tls`, `hg-dtls` before first public release; re-audit after any change to key schedule, record layer or a primitive | NW-49–NW-52 | NW §8 | High |
| 3.6 | **Secret types**: "Traffic secrets, key-schedule inputs and ephemeral private keys live in `hg-crypto` types that zeroize on drop and have no `Debug`, `Clone` or serialization; the recording boundary never sees them" | NW §6.5 | NW §6.5 | High |
| 3.7 | **Q-NW-14** (open): "Who owns WebCrypto (`crypto.subtle`), and does it link `hg-crypto` in renderers? WebCrypto needs signing, key generation and algorithms the network stack does not (RSA-OAEP, PBKDF2, AES-CBC and AES-CTR, Ed25519 and ECDSA signing), each of which would join `hg-crypto`'s constant-time surface and the audit." Ledger **G-18** = the same gap; "Staffing (counted nowhere)" | Q-NW-14; G-18 | NW §11.1; ledger | High |
| 3.8 | The ledger already routes one extension of `hg-crypto`: A-NW-5 (media) adds **AES-CTR and `cbcs`** to `hg-crypto` for media CDM-style decryption | A-NW-5 | ledger | High |
| 3.9 | **Entropy::crypto()** draws for wire identifiers come from G35's lint; `Entropy::crypto()` values recorded "as the values drawn, never a seed" (NW §6.16). Recording boundary is above decryption; "No key, traffic secret or key-derivation input is ever recorded" | NW §6.16 | NW §6.16 | High |
| 3.10 | NW staffing counts the cryptography track at **2–3†** and the security-response rotation at 2–3†; its 9.5 lists the amendments its decisions need (Stage 1, security scope, coding standard, CONTRIBUTING) | NW §9.3, §9.5 | NW §9.3 | High |

**Implications:** G36 gives exactly one place crypto may live; the open question is not *where the code is* but *who grows it and who pays for the audit growth*. Q-NW-14's own text already lists what WebCrypto adds to `hg-crypto`'s secret-dependent surface: RSA (OAEP decrypt, PSS/PKCS1v1.5 signing), PBKDF2, AES-CBC/CTR/KW, Ed25519 signing, ECDSA signing (P-256/P-384), plus the AES-KW wrap path.

## 4. JavaScript engine spec 0.4.1

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 4.1 | **JS-D15**: every timing-dependent decision is a recorded B1 input. The nondeterminism table pins `Math.random` as "A per-realm generator seeded from a named B1 entropy stream; each realm gets a distinct sequence, as ECMA-262 requires" | JS-D15; §6.9 table | JS §6.9 | High |
| 4.2 | `Math.random` is deliberately **not** the CSPRNG: it is a per-realm seeded stream (B1), separate from `Entropy::crypto()`. So WebCrypto's `crypto.getRandomValues` cannot share the `Math.random` stream without breaking JS-50's cross-ISA identity work | §6.9 table | JS §6.9 | High |
| 4.3 | **Structured clone** gets "internal-slot access through a typed serializer interface over engine objects, never raw layouts" (JS §6.14); the DOM spec builds `hg-clone-wire` (one versioned record format, G95) on it — `CryptoKey` is a `[Serializable]` platform object and must ride that format | JS §6.14; DM §6.11 | JS §6.14 | High |
| 4.4 | JS G6: only `hg-dom`, `hg-bindings`, `hg-script`, `hg-inspect-renderer` and named embedders depend on `hg-js-host`/`hg-heap`. A crypto crate cannot reach into the heap; `CryptoKey`'s secret bytes live where G36 and the DOM's storage rules allow | JS G6 | JS §6.15 | High |
| 4.5 | JS G13 (no engine code reads an OS clock) and CS-AS-2 (hash seeds from B1 entropy) are the pattern every ambient-value consumer follows | §6.9 | JS §6.9 | High |

**Implications:** `crypto.getRandomValues` needs a **second named stream class** (`Entropy::crypto()` draws, recorded as drawn values), never the `Math.random` generator; `CryptoKey` serializes over `hg-clone-wire`'s format with new record kinds, and the serializer must never expose key bytes to script except through `exportKey`.

## 5. DOM, bindings and event loop spec 0.2.1

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 5.1 | **The ownership row**: "WebCrypto (`SubtleCrypto`) \| Bindings, once the owner of the algorithms is named \| **Unowned: ledger G-18**". This spec says it is "ready to build the bindings" (§9.5 last row) | §6.18 table | DM §6.18 | High |
| 5.2 | `hg-webapi` is the script-facing web-API crate; WebCrypto's bindings belong there once the algorithm owner is named (§6.18 "Script-facing web APIs" table) | §6.18 | DM §6.18 | High |
| 5.3 | **DM-D8** worker placement: workers run "in the key's renderer, preferring one that holds the worker's storage key" — so a `SubtleCrypto` operation in a dedicated worker runs in that worker's renderer process, same process as its window | DM-D8 | DM §6.19 | High |
| 5.4 | **DM-D12**: `hg-clone-wire` is the one format for everything serialized; its decoder is the only parser (G95), Kani-harnessed and fuzzed. `CryptoKey` cloning (same agent and cross-worker `postMessage`) extends it | DM-D12 | DM §6.11 | High |
| 5.5 | **DM-D18** observability classes: always (counters), DevTools-attached (per-session journals), recording. A crypto operation journal belongs in the DevTools-attached class | DM-D18 | DM §6.15 | High |
| 5.6 | The DOM spec's error taxonomy (typed refusals, cross-origin refusals journal, "not explained, never a guess") is the UX precedent a crypto error journal follows | §6.15, §6.16 | DM §6.15 | High |
| 5.7 | G90–G99 are the DOM spec's rule range; its 12–14† web-API track covers the script-facing side (staffing now 72–92†) | §9.3, §9.5 | DM §9 | High |

**Implications:** this spec (WebCrypto) should own the algorithm implementations' web requirements and the `SubtleCrypto` semantics; `hg-webapi`'s bindings and the async-operation plumbing stay with the DOM spec; `CryptoKey` record kinds join `hg-clone-wire` with a DOM-spec amendment.

## 6. Resource footprint spec 0.3

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 6.1 | Budgets: empty site process ≤ 4 MiB† (FP-2); each further agent heap ≤ 768 KiB† (FP-34); further same-agent frame ≤ 128 KiB† / realm ≤ 64 KiB† (FP-4); idle empty site process has **two threads** (FP-32); pool size ≤ min(cores−1, 4)† | §8.2, §6.x | FP §8.2 | High |
| 6.2 | A new **process kind** would need its own row in the baseline budget (FP-1) and a thread count in FP-32's table; process kinds are enumerated (broker, shell renderer, GPU service, network line, storage, crash handler, spawner, utility jobs, audio and capture services) | §8.2 | FP §8.2 | High |
| 6.3 | FP-D14: in-process caches keyed by (T, C, X, n) projection of NW-D7 — anything partition-scoped follows the same key shape | FP-D14 | FP §6.4 | High |

**Implications:** renderer-resident WebCrypto costs no new budget line if it is a library inside the existing renderer (some KiB of code, lazy-initialized); a crypto *service* process would need a new FP-1 row, a thread count, and IPC budget — the burden of proof sits on whoever proposes the service.

## 7. DevTools spec 0.3 and the IIP boundary

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 7.1 | **IIP** is Izcal's own inspection protocol: `.hgidl` declarations with `[inspect, tier = stable|beta|experimental]`, generated schema + clients; sessions hold handles; every event carries an execution point and `caused_by`; provenance (origin, frame, principal, `untrusted` flag) | DT-D1 | DT §6.1 | High |
| 7.2 | Crates: `hg-inspect-schema` (L2, generated types), `hg-inspect-renderer` (L4 agent), `hg-inspect` (L5 broker), `hg-export`/`hg-redact` (L5; writers accept only `Redacted<T>`, G11) | §6.1 crate table | DT §6.1 | High |
| 7.3 | **Never recorded** (DT-D2): keystrokes into credential fields, password-manager fills, "WebAuthn, FedCM and payment-handler results … placeholder of the same length and type"; "TLS secrets stay below the recorded boundary" — the same rule must cover WebCrypto key material | DT-D2 | DT §6.1 | High |
| 7.4 | DT-D5: network views come from `hg-net-observe`; production recordings cannot re-run below their boundary "because its TLS handshakes used the OS CSPRNG" | DT-D5 | DT §6.1 | High |
| 7.5 | Write capability per origin gates interventions; every view labels what it changed (DT-21). A crypto panel's replay/override features would follow the same capability check | DT-21 | DT §6.1 | High |

**Implications:** a crypto observability domain is an IIP domain served from the renderer agent (`hg-inspect-renderer` reads it), with key material structurally excluded (a `Redacted<T>`-style type for anything key-derived), fitting the existing process and consent model without new exceptions.

## 8. Reconciliation, ledger, security scope

| # | Finding | IDs and quotes | Source | Conf. |
|---|---|---|---|---|
| 8.1 | **L2s** is the shared standards sub-layer of L2, defined by the style spec's G24 (P1, reconciliation 1); L2s crates today: `hg-math`, `hg-ucd`, `hg-unicode`, `hg-net-wire`, `hg-compress`, `hg-cookie-parse`, `hg-psl`, `hg-crypto` | P1; A-S1-12 | RN1; ledger | High |
| 8.2 | **G-rule registry** (P7): G19–G24 ST, G25–G29 TX, G30–G36 NW, G90–G99 DOM, G100–G109 AX, G110–G139 batch 2 (SVG/animation/editing/PDF). **G160–G169 is unassigned** and reserved for this spec | P7; per-spec 9.5s | RN1; ledger | High |
| 8.3 | **G-18**: "WebCrypto owner, and whether it links `hg-crypto` (Q-NW-14)"; blocks "Staffing (counted nowhere)". **G-19** storage is being spec'd in parallel; nothing may assume its internals | ledger | ledger | High |
| 8.4 | Security scope 0.3 (in review): **NET-R5** lists `hg-tls` and `hg-crypto` (with pins) as the network tier-P set; **SEC-25** advisory scope covers `hg-crypto`'s pins and Izcal's own crypto; **REC-R2** keys recordings at rest; the renderer threat model treats renderer compromise as assumed | A-SEC-17, A-SEC-19, REC-R2 | ledger | High |
| 8.5 | The ledger's A-CON-1 gives `hg-crypto` the single CONTRIBUTING carve-out from the "no hand-rolled cryptography" review rule, under the audit and NW-49/NW-50 | A-CON-1 | ledger | High |
| 8.6 | M-1 part 1 default: "A dependency whose class is unclear is decided by the owner and recorded in the Stage 1 design; the default is to build it" | M-1 | HGBA-2 §2 | High |

**Implications:** the answer to G-18 names an owner (this spec) without moving the code: `hg-crypto` stays NW's crate under G36, this spec owns the WebCrypto surface and files its algorithm needs as amendments to NW 6.5/NW-D23's surface and to NW-50's constant-time list. The security scope's renderer threat model (compromise assumed) is the threat model the placement decision must answer.

## 9. Consolidated constraints for the WebCrypto spec

1. **One crypto crate** (G36). Whatever WebCrypto needs joins `hg-crypto`; no new crate may implement or import a primitive.
2. **The audit and constant-time surface grow with the web API**, so the owner decision must name who staffs and pays for the growth (NW-50/NW-52 amendments).
3. **Two entropy streams**: `Math.random` stays the seeded per-realm B1 stream (JS §6.9); `crypto.getRandomValues` draws from `Entropy::crypto()`, production = OS CSPRNG only, simulation = seeded (Stage 1 3.1, Q-CS-1). Outputs recorded as drawn values, never a seed (NW §6.16).
4. **Secret types and the recording boundary**: `CryptoKey` private material lives in `hg-crypto` secret types (zeroize, no `Debug`/`Clone`/serialization); nothing key-derived crosses into recordings; DevTools sees metadata only (`Redacted<T>` discipline, DT G11).
5. **Placement must beat P1/I-1's renderer threat model**: keys in a compromised renderer's address space are the baseline; any stronger boundary (service, OS keystore) is an upgrade with a cost and an IPC budget.
6. **Bindings** (`hg-webapi`), **structured clone** (`hg-clone-wire` record kinds), **workers** (DM-D8's renderer) and **observability classes** (DM-D18) are the DOM spec's; this spec supplies the semantics and the records.
7. **Rule numbers G160–G169; target prefix WC-; milestones WC-M\***; amendments in Section 9.5, never applied to other files.
8. Staffing must be counted (G-18's complaint) and reconciled with NW's cryptography track (2–3†) and the DOM spec's web-API track.

## 10. Decision points this spec must answer

| # | Decision point | Constraints from |
|---|---|---|
| 1 | Ownership of WebCrypto and of `hg-crypto`'s growth (G-18, Q-NW-14) | NW §6.5, §11.1; DM §6.18 |
| 2 | Which primitives WebCrypto adds to `hg-crypto`, and which become secret-dependent | NW-D23, NW-50; W3C registry |
| 3 | Entropy source per platform and the `getRandomValues` plumbing | Stage 1 3.1; B1; JS §6.9 |
| 4 | Key model: extraction, structured clone, worker boundaries, storage posture | W3C; DM-D12; G-19 parallel spec |
| 5 | Placement: renderer library vs process boundary vs OS keystore | P1, I-1; security scope threat model; FP §8.2 |
| 6 | Constant-time targets and verification per primitive | NW-50; M-1 part 3 |
| 7 | Permission/policy surface (none? secure context? fingerprinting?) | W3C; P2 (compat) |
| 8 | Determinism and recording of random outputs | B1, DET-1; NW §6.16; DT-D2 |
| 9 | Post-quantum direction: the ML-KEM spike and its trigger | FIPS 203; HGBA-2 §4.7 |
| 10 | Observability: the IIP crypto domain and the DevTools boundary | DT-D1, DT-D2, DT G11 |
| 11 | Staffing, timeline, milestones WC-M0… | Stage 1 §8; NW §9.3 format |
| 12 | Closure of G-18 and Q-NW-14 | ledger; NW §11.1 |
