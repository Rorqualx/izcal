# Izcal enterprise policy: independent design critique (phase 8)

Written 24 October 2026 by a reviewer who did not write the spec. Subject: `reports/Izcal enterprise policy.md`, draft 0.2 (line numbers below refer to that draft). Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8 (design critique against the existing specs, looking for contradictions, missed constraints and M-1 violations).

**Read against:** Stage 1 design 0.14.2 (Section 1 process map, 2.1 layers, 2.2 G-rules and registry, 3.4, 4.1, 8.1, 8.3, 13.2, D-S4); security scope 0.3.x (6.1 ATK-2/5/7 and the out-of-scope line, 6.3a per-process table, 6.6 IPC-R2, IPC-R3, IPC-R15, IPC-R16, IPC-R17, REC-R1–R5, SEC-6, SEC-D14); shell (6.7, 6.9, 6.11, 6.14, 6.16, 6.17, 6.18, SH-D9, SH-D26, SH-D29); networking 0.3.7 (6.1, 6.13, 6.14, NW-D17, NW-D20, NW-36); extensions (6.5, 6.6, 6.12, G171, Q-EX-5); app host 6.7; devices (6.10, 6.18, G210); content blocking (6.2, 6.10, 6.12, CB-D19); phishing protection 6.13 and Q-PH-15; DevTools (automation gateway, DT-D2); storage (SG-D11, Q-SG-6); resource footprint 0.4.x (FP-1, FP-17, Section 8.2 rows, the A-SH-39 owner decision); coding standard (6.2 tier table, CS-ER-6); HGBA-3 B10 and D13; feature inventory audit (FI-3, §5 C1, §7.1, §8). **Code read:** `crates/hg-ipc-codegen/src/{syntax,check}.rs`, `crates/hg-ipc-gen/idl/{process,sample}.hgidl`, `crates/hg-substrate/src/ipc.rs`, `crates/hg-ipc/src/lib.rs` (M1 as merged at `3d2fc4a`).

**Owner direction applied as binding:** build our own; MIT only for shipped dependencies; no third-party source code as evidence (M-1 part 3); open protocols first (FI-D14); no shortcuts for time or complexity; FI-D9 (spec before M5, delivery in Stage 2), FI-D12, FI-D17. The coordinator's accepted M1-exit direction (IPC snapshots from an OS-adapter process, a 32-byte `PolicyDigest` in the shared header, a reserved `policy.hgidl`) is taken as given; findings below concern how the spec realises it, not whether to adopt it.

**Overall.** The model is right and better than any incumbent's: one catalogue generating every artefact, admin-only sources checked at read time, a pure total merge with fixed rules, non-tattooing, per-policy failure, and exact disclosure. The weak points are at the **seams with code and with other specs**. The reserved `policy.hgidl`, the one artefact M1's exit adopts, does not pass the merged generator and cannot carry the catalogue's own types. The remote document, the spec's answer to FI-D14, keeps its anti-rollback and grace state only in memory, so it un-manages at every relaunch. The always-on adapter is costed against an FP-1 budget that has 1 MiB† left, with a fallback process that does not exist. Several claims about other specs are wrong (content blocking's "site-pattern syntax", "the profile lock, as SG requires", "SH 6.7's provenance field", the network system service as a fetcher of non-Izcal endpoints, the component host as a policy consumer). No M-1 violation was found: the spec and its notes cite rendered documentation only, and correctly route the DLP protocol (code-only) to black-box work. Five findings are High.

---

## Summary

| Severity | Count | Findings |
|---|---|---|
| High | 5 | C1 (`policy.hgidl` fails the merged generator and cannot carry the catalogue), C2 (remote document keeps rollback and grace state only in memory), C3 (always-on adapter breaks FP-1; the fallback process does not exist), C4 (remote, reporting and site-list traffic contradict NW-D17 and skip IPC-R16), C5 (slices go to a consumer G171 forbids and miss the processes that enforce) |
| Medium | 13 | C6–C18 |
| Low | 6 | C19–C24 |
| **Total** | **24** | |

---

## High

### C1. High — The reserved `policy.hgidl` does not pass M1's generator, and its message shape cannot carry the catalogue's types

**Location:** 6.6 (lines 340–387), 6.21 item 3 (line 600), EP-M0 (line 719).

**Problem.** The shape is the one concrete M1-exit artefact, and it is described as "reserved" so that M1 can adopt it. Checked against `hg-ipc-codegen` as merged, it fails in at least six independent ways, and two of them are structural, not cosmetic:

1. **`remote_document(bytes: bytes<65536>)` exceeds the message cap.** `bytes<N>` encodes as a 4-byte length plus N bytes, and the checker adds 2 bytes of ordinal: 65,542 bytes against `MAX_MESSAGE_LEN` = `MAX_MESSAGE_BYTES` = 65,536. The generator rejects it ("send bulk data as shared memory").
2. **`bytes` is a reserved parameter name** (the generated reader binds it).
3. **No item, variant, field or parameter carries a `///` doc**, and the checker requires one on every name (the generated tier-P code denies `missing_docs`).
4. **`/* … */` is not a comment** in `.hgidl` (only `//` and `///`), and **a union with no case is rejected** ("needs at least one case"). The text should say the union is emitted by `hg-policy-gen`, not shown as an empty block.
5. **`vec<PolicyEntry, 256>` caps every policy value at about 250 bytes.** The checker sizes a message by its **maximum** encoding: 256 × (4 + 4 + max(`PolicyValue`)) must fit 64 KiB, so the largest union case must encode to ≲ 246 bytes. The catalogue's own types (a URL, a URL-pattern list, `ExtensionSettings`, `ManagedBookmarks`, `TrustAnchors` as DER, `SiteSwitchingRules`, `KioskMode`) are far larger, and a single `UrlBlocklist`, `ExtensionSettings` or site list in real deployments exceeds 64 KiB on its own. Chunking by entry cannot split one value.
6. **`ManagedBookmarks` is a recursive tree** (`children`, line 231), and the checker rejects recursive types ("recursive types have no size bound"). Every structured type needs a flattened, bounded encoding (records with parent indices and a depth cap).

Also: `reason: u8` on `reread` is an open integer where CS-ER-6 and IPC-R3 want a closed enum; `PolicyEntry.index` duplicates the union tag and creates a mismatch case the decoder must reject; the remote document's detached `.sig` (line 437) has no field, so the adapter cannot verify what it receives; and the site list (6.16, line 537) has no message at all. The spec's escape, "when the shared-memory handle kind lands" (line 387), names a handle kind M1 does not have: `HandleKind` holds only `ChannelSender`, `BoundChannelSender` and `ChannelReceiver`.

**Evidence.** `crates/hg-substrate/src/ipc.rs:25` "`pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;`"; `crates/hg-ipc-codegen/src/check.rs:525–531` "message `{}` can encode to {size} bytes, over the {} byte cap; send bulk data as shared memory"; `check.rs:79–82` `RESERVED_FIELDS` includes `"bytes"`; `check.rs:115–119` "needs a `///` doc comment"; `check.rs:322` "recursive types have no size bound"; `check.rs:347–348` "needs at least one case"; `syntax.rs:24` "`//` starts a comment"; `syntax.rs:106–108` lexer error "expected `//` or `///`"; `ipc.rs:32–40` the three handle kinds. Spec line 371: "Up to N entries per message, so each message stays under 64 KiB (S1 4.1)"; line 383: `remote_document(bytes: bytes<65536>)`.

**Fix.** Before M1 exit, run the reserved shape through `hg-ipc-codegen --check` as a checked-in golden file (CS-ST-17's sample pattern), with a stand-in `PolicyValue` that has the catalogue's largest case. Redesign the transfer: (a) values above a stated bound (say 4 KiB†) cross as **typed, bounded chunks of one value** (`value_part(source, seq, index, offset, part: bytes<60000>)`) reassembled and decoded by the generated decoder on `end`, or (b) reserve a **read-only shared-memory handle kind** now (S1 4.1 already promises it) and send each snapshot as one region with a generated decoder. Flatten recursive types. Give `remote_document` a separate signature field and a size cap stated per IPC-R16 (C4), add a `site_list` message, replace `reason: u8` with an enum, and drop `index` or make it the only tag.

### C2. High — The remote document's rollback refusal and stale grace live only in memory, so every relaunch un-manages the browser until a fetch succeeds

**Location:** 6.5 (line 310), 6.10 (lines 436–439), EP-16 (line 708), 6.22 threat model (line 612).

**Problem.** 6.10 promises three properties: `version` "must increase" (rollback refused), an expired document "is kept as stale for 7 days", and a blocked network cannot "silently un-manage the browser for good". 6.5 forbids persisting any snapshot ("The stale snapshot is never persisted"), and nothing else persists the document or the highest version seen. Consequences:

- **At every launch the remote source contributes nothing** until the network system service completes a fetch (6 h† schedule, or at start). Every remote-only restriction (URL blocks, extension blocks, DevTools block, force-installs) is absent for that window, and force-installed extensions and locked rows flap at each start.
- **ATK-5, which the threat model lists for this source**, needs only to block the fetch at start to keep the browser unmanaged for the whole run, and the 7-day grace never starts because there is nothing to be stale.
- **Rollback is refused only within one run.** After a relaunch, ATK-5 can serve any older document whose signature and `expires` are still valid (up to 30 days† old), re-enabling whatever a newer document revoked. EP-16 ("0 … rolled-back … documents accepted") cannot pass across restarts.

The 6.5 argument against persistence ("a persisted cache would sit in a user-writable profile (ATK-7) and would need its own integrity key") does not apply to the remote document: it is **signed**, so the signature is the integrity key. What ATK-7 can do to a persisted copy is delete it or reset the version counter, which is a smaller, stated residual (and ATK-7 can already inject into the broker, C11).

**Evidence.** Line 310: "The stale snapshot is never persisted: a persisted cache would sit in a user-writable profile (ATK-7) and would need its own integrity key." Line 437: "`version` is a u64 that must increase." Line 439: "An expired document is kept as stale for 7 days† and then dropped, which freezes the last state without letting a blocked network silently un-manage the browser for good." Line 612: "ATK-5 serving a remote document or site list (refused by signature, version and expiry)".

**Fix.** Persist the **signed document bytes, its signature and the highest accepted `(version, issued)`** per enrolment URL and key set, in a per-OS-user location (not per profile, FI-D17), re-verified by the adapter at launch before any other source is merged. Apply the cached document at launch as `stale` until the first fetch. State the residual: ATK-7 can delete the cache or reset the counter (restoring "unmanaged by remote until fetch", never a forged policy). Where the OS offers an admin-only location the adapter can write through a broker-held handle, use it for the counter. Add EP-16 cases across a relaunch: blocked fetch at start, older valid document served after restart.

### C3. High — An always-on policy adapter does not fit FP-1, contradicts FP's OS-adapter row, and its fallback names a process that does not exist

**Location:** 6.6 (lines 315, 325), EP-8 (line 700), 6.25 alternatives (line 676), A3 (line 775), A20 (line 792).

**Problem.** The adapter is "one per machine, launched by the broker at startup and kept alive", on every install, managed or not, and in Stage 1 release builds that by EP-D19 "read no OS source" (line 668). It is budgeted at ≤ 2 MiB†.

- **FP-1 has no room.** FP's line budgets sum to 63 MiB† of 64 (1 MiB† left; 3.5 MiB† at idle with no tabs). A new always-on 2 MiB† line breaks FP-1 on Linux and macOS, and on Windows the owner accepted ~74 MiB† for the UI host only (A-SH-39), not for further processes.
- **FP already has an OS-adapter row, and it is on demand.** FP 8.2: "OS-adapter process (per container, on demand) … ≤ 2 MiB† idle, exits after 10 s†". The spec makes the same process kind always-on without amending that row.
- **The fallback does not exist.** "Share the network-configuration adapter (machine-wide, no credentials)" assumes a machine-wide always-on network OS adapter. NW 6.1 has OS adapters **per container** for resolver fallback, SSPI/GSSAPI and signing, and says OS proxy and resolver configuration is "read by the broker as today"; FP counts helpers "only where configured". A2 proposes to move that reading into an adapter, but nothing creates a machine-wide one.
- **Nearly every install pays for nothing.** Unmanaged machines have no source; Stage 1 release builds read none by design.

**Evidence.** FP 8.2 row (resource footprint line 892): "OS-adapter process (per container, on demand) … ≤ 2 MiB† idle, exits after 10 s†". FP line 965: "Line budgets: 20 + 14 + 12 + 2 + 8 + 4 + 2 + 1 = **63** (1 left)". FP line 207 (A-SH-39): the ~74 MiB† Windows figure is accepted for the shell's UI host. NW line 261: "OS proxy settings, resolver configuration and network-change events are … read by the broker as today." Spec line 315: "launched by the broker at startup and kept alive, because it holds the OS change registrations."

**Fix.** Make the adapter **on demand**: the broker (through `hg-substrate-os`, parsing nothing) does the cheap existence and change-notification work (registry key exists, `RegNotifyChangeKeyValue`/`RegisterGPNotification`, inotify on `/etc/izcal/policies`, activation on macOS), and launches the adapter only when a source exists or changes; the adapter sends its snapshots and exits after 10 s† like FP's other helpers. Unmanaged machines and Stage 1 release builds then run zero adapter processes. If the OS notification must be held by the reader, state the measured always-on cost per OS at EP-M0 and take it to the owner as an FP-1 decision, as A-SH-39 did, rather than as a fallback. Amend FP 8.2's OS-adapter row in A20 accordingly.

### C4. High — Fetching the remote document, site list and reporting through the network system service contradicts NW-D17, and the routed bytes skip IPC-R16's registration

**Location:** 6.0 diagram (lines 160–165), 6.10 Fetch (line 438), 6.16 (line 537), 6.17 Transport (line 546), 6.22 (line 611), 9.5 (no NW-D17, NW-36 or IPC-R16 row).

**Problem.** Three new flows run through the network system service: the remote document from an organisation URL, the Enterprise Mode site list from an organisation URL, and enterprise reporting uploads to an organisation endpoint. NW defines that service as Izcal's own traffic only, to a **published endpoint list**, and checks it with NW-36. None of the three is on the list, and the reporting upload carries device- and user-specific inventory, which NW-D17 routes through OHTTP or keeps out of the system service. Organisation endpoints are typically intranet hosts reachable only through the organisation's route, proxy and authentication; the system service by design uses none of a container's routes or credentials, and the spec adds "sends no cookies, credentials", which breaks the common case of a site list on an authenticated intranet server or a file share.

Separately, the document and site-list bytes cross the broker as opaque bytes that the adapter later parses as JSON and XML. The spec calls this "the IPC-R16 pattern", but IPC-R16 is a **named list**: each entry needs an owner, a size cap, a fuzz target in the receiver and the security owner's approval through SEC-6; the TUF bytes are entry 3 by that process. IPC-R3 also bans "self-describing nested payloads". Neither is amended.

**Evidence.** NW line 233: "**System service** (one): Izcal's own traffic (Section 6.13), never web content." NW line 525: "Only the system service talks to Izcal's servers … Its endpoints are a published list … Requests that carry user-specific data … go through an Oblivious HTTP relay". NW line 798 (NW-36): "the browser's only network traffic is to the published endpoint list". Security scope line 882: "IPC-R16. Payloads outside `.hgidl` are a named exception list. Each entry names the payload, its receiver, an owner, a size cap, a fuzz target … A new entry needs the security owner's approval through SEC-6." Line 869 (IPC-R3): "no self-describing nested payloads, no strings the privileged side later parses as URLs, paths, JSON or expressions." Spec line 438: "The bytes reach the adapter through the broker as opaque data (IPC-R16 pattern)."

**Fix.** Decide which service carries **organisation traffic**. The consistent choice with NW-D3 and NW-D17 is a dedicated **organisation (management) container service and its partition service**, with the organisation's proxy, trust anchors and NW-D20 allowlist, so intranet routes and Negotiate work and Izcal-bound traffic stays separate; or else amend NW-D17 explicitly with an "organisation endpoints" class, NW-36's quiet-browser definition (unmanaged machines still 0 extra requests) and the OHTTP exemption for reporting with its reason (the organisation is the controller). Support `file:` and UNC site lists through a broker-held read grant. Register **IPC-R16 entries** for the remote document with its signature and the site list, each with a size cap, receiver (`hg-policy-format` in the adapter), owner and fuzz target, and add A-rows to 9.5 for NW-D17, NW-36, IPC-R16 and SEC-6.

### C5. High — The slices go to a consumer G171 forbids, and miss most of the processes that actually enforce policy

**Location:** 6.6 slices (lines 330–338), 6.11 (line 465), G238 (line 661).

**Problem.** G238 says a policy reaches a consumer only through its slice, so the slice list is the enforcement map. It is wrong in one place and incomplete in several:

- **`ExtensionPolicy` to the component host contradicts G171.** EX makes the broker the only holder of grant truth; the component host is one process **per extension origin**, presents tokens, and "hold[s] no grant truth". Install modes, blocked capabilities and runtime host lists are broker decisions in `hg-ext-capability`; only each extension's own `storage.managed` value belongs in its host.
- **Partition services are missing.** `TlsMinimumVersion`, `QuicAllowed`, `EncryptedClientHelloAllowed`, `HttpsFirstExceptions`, LNA defaults and the CB matcher's `UrlBlocklist` run in the partition service (S1 §1), not in the system or container service.
- **The crash handler and updater are missing.** `CrashReportsEnabled` (and the per-crash prompt under D-S4) is enforced by the native crash handler; `UpdatesEnabled`, `UpdateChannel`, `TargetVersionPrefix` by the updater process and the update client. Neither has a slice, and the crash handler also needs the digest for the shared header (6.21 item 1).
- **Storage, ML service and agent host.** `EphemeralProfile` and `ProfileDirectory` bind the storage service; `AiEnabled` and `AccessibilityModelEnabled` reach the ML service and agent host only through the settings row (fine), which the spec should say.
- **Confidentiality claim.** "A consumer never sees another consumer's slice" is stated as a privacy property, but `NetworkPolicy` as described goes whole to every network service, so a compromised partition service (ATK-2) reads the proxy, `AuthServerAllowlist` and internal hostnames.

**Evidence.** Extensions line 414 (G171): "Every capability check is made by the broker against `hg-ext-capability` state; renderers and the component host present capability tokens and hold no grant truth". Extensions line 202: "**One component host process per extension origin per profile**". S1 line 26 (partition service): "TLS (`hg-tls`, `hg-pki`) … CORS/CSP/HSTS/LNA enforcement … and the content-blocking matcher". S1 line 51: "**Crash handler** … crash reports and the D-S4 opt-out". Spec line 335: "`ExtensionPolicy` to the component host"; line 338: "A consumer never sees another consumer's slice, so a compromised renderer learns its JIT ceiling and nothing else".

**Fix.** Rewrite the slice table as **policy → enforcing process → slice**, generated from the catalogue's `owner` field (one more catalogue field: `enforced_in`). Route `ExtensionSettings` to the broker's extension capability service and send each component host only its `storage.managed` value. Add `PartitionNetworkPolicy` (only the fields a partition enforces, keyed per container), `CrashPolicy` (to the crash handler, with the digest), `UpdatePolicy` (to the update client and updater) and `StoragePolicy`. Add a target: for each process kind, the set of catalogue fields it receives equals the generated enforcement map (CI check), and no slice carries an `internal-hostnames` field to a process that does not enforce it.

---

## Medium

### C6. Medium — Crate layers are misnamed, the policy backend has no link restriction, and the adapter's binary and process kind are unnamed or contradictory

**Location:** 6.0 (line 186), 6.24 crate table (lines 643–649), G231, G239, 6.6 (line 387), A4.

**Problem.** `hg-policy-catalogue` is placed at "L1 data", but L1 is the substrate layer (`hg-substrate`, `hg-substrate-os`, `hg-substrate-sim`, `hg-ipc`); a data crate whose generated types `hg-ipc-gen` (L2) and `hg-policy` (L5) consume belongs at L2 (contracts) or L2s. `hg-policy-format` is placed at "L3", the **document** layer (`hg-dom`, `hg-bindings`, …); format parsers belong at L2p beside `hg-xml-syntax`, which already exists and which the site-list parser should reuse (one parser per format) instead of shipping a second XML parser. The policy backend is a module of `hg-substrate-os`, which **every** binary links; without a rule like devices' G210 ("Only `hg-device-arb` may request a device effect from `hg-substrate-os`"), the broker or a renderer could call it, which is the very ambient authority 6.21 argument 2 cites against a trait. Finally, 6.6 calls the adapter "a configuration of S1's OS-adapter process kind", but then adds a new `ProcessKind::PolicyAdapter`; and no binary is named (S1 has `hg-os-adapter-main` and the L5 crate `hg-os-adapter`), so G231 ("linked only by the policy-adapter binary") has nothing to bind to.

**Evidence.** S1 line 116: "L1  substrate   hg-substrate (traits)  hg-substrate-os (includes hg-sandbox)  hg-substrate-sim  hg-ipc (transport)"; line 96: "L3  document    hg-script-api  hg-dom …"; line 109: "L2p parsers     hg-html-syntax  hg-xml-syntax …"; line 74: "`hg-os-adapter-main`". Devices line 684 (G210). Spec line 645: "`hg-policy-catalogue` | L1 data, tier P"; line 647: "`hg-policy-format` | L3, tier P".

**Fix.** Move `hg-policy-catalogue` to L2 contracts (generated, like `hg-ipc-gen`) and `hg-policy-format` to L2p, reusing `hg-xml-syntax`. Add a G rule: only the policy adapter's L5 crate (name it, e.g. `hg-policy-adapter`, or the policy module of `hg-os-adapter`) may call `hg-substrate-os::policy`, enforced by the dependency allowlist as G210 is, ideally behind a cargo feature only `hg-os-adapter-main` enables. Decide kind versus configuration: either one `ProcessKind::OsAdapter` with a typed configuration in its launch message, or a separate kind, and say which.

### C7. Medium — "One adapter per machine" conflicts with per-user brokers and user-scope sources, and macOS loses the machine/user distinction that force-install depends on

**Location:** 6.6 (line 315), 6.3 macOS row (line 256), `PolicySource` enum (lines 348–352), 6.11 force-install (line 454), 6.20 VDI (lines 571–576).

**Problem.** Brokers are per OS user (FI-D17: profiles share one broker, per user), and the adapter is "launched by the broker". On a multi-user or multi-session machine (EP-D18's own VDI case) there are therefore N adapters, one per user broker, each reading HKLM plus **its own** HKCU and the user's managed-preference channel. "One per machine" is either wrong or would require a machine-wide process serving several users' brokers across a user boundary, which the sandbox cannot grant. On macOS, `PolicySource` has only `MacForced` and `MacManagedDefault`: device-channel and user-channel profiles collapse into one source, so 6.4's "machine mandatory" versus "user mandatory" and 6.11's "force-install only from machine scope" are undecidable on macOS. `CFPreferencesAppValueIsForced` does not say which channel set the value; distinguishing them is part of Q-EP-3, but the enum and precedence are fixed before the answer.

**Evidence.** Spec line 315: "one per machine, launched by the broker at startup"; line 256: "`CFPreferencesAppValueIsForced` = true for the domain `dev.izcal.browser` (device or user channel)"; line 349: "`MacForced, MacManagedDefault`"; line 454: "Force-install is honoured only from machine-scope mandatory sources". Audit line 643 (FI-D17): "several profiles share one broker".

**Fix.** Say "one per OS-user broker" and size EP-8 per user. Split the macOS source into `MacDeviceForced`, `MacUserForced` (and their defaults), with EP-M0 measuring how the adapter tells them apart (the per-user versus device managed-preferences layout, Q-EP-3); until measured, treat macOS forced values as **user** scope for force-install (fail safe) and say so.

### C8. Medium — What the digest covers is undefined, its canonical form is unspecified, and it is not stable across builds

**Location:** 6.4 total order (lines 281–289), merge signature (line 301), 6.21 item 1 (lines 592–596), EP-3 (line 695).

**Problem.** 6.4's total order puts **"the person's own setting"** at level 4, but the merge signature `merge(catalogue, [SourceSnapshot])` takes no settings, and the digest is over "the effective entries … values only". If `EffectivePolicy` includes the person's setting, the digest changes when a user flips a recommended row and differs per profile under one broker (FI-D17), so one header digest per recording is ambiguous; if it excludes them, level 4 is not part of the merge and 6.4 should say so. "Values only (no sources or statuses)" also drops the **level**: the same value as mandatory (locked) and as recommended (changeable) gives the same digest, though behaviour differs, so a replay cannot tell whether the person could change the row. "Canonical" is asserted but not defined for lists (union order across sources; registry `1`, `2`… versus JSON array order; duplicates), URLs and hostnames (case, IDNA, trailing dots), or record-key order in structured values, so EP-3's "identical digests" on three OSes depends on an unstated normalisation. Finally, VER-1's encoding is **same-build only** (audit §8 point 3; the spec itself relies on that at line 387), so two builds with the same catalogue version may encode the same value differently after a generator change; `about:policies` on build X and a recording from build Y then disagree.

**Evidence.** Line 285: "4. **the person's own setting**"; line 301: "`merge(catalogue, [SourceSnapshot]) -> (EffectivePolicy, [PolicyStatus])`"; line 594: "values only (no sources or statuses), so equal effective policy gives an equal digest on every OS". Audit line 677: "`.hgidl` is same-build only … The wire format has no self-describing schema (VER-1)".

**Fix.** Define `EffectivePolicy` as the policy layers only (levels 1–3 and 5–7), per OS user, and state that settings resolution (level 4) happens downstream in the settings service. Digest a dedicated, versioned **canonical policy encoding** (`izcal-policy-v1`) owned by `hg-policy-catalogue`, not the IPC wire format: entries sorted by stable policy name (not index), each with its level, lists sorted and de-duplicated where the merge rule is a set, URLs and hosts in `hg-url`'s canonical form, no floats (add a G rule that the catalogue has no float type). Publish test vectors, and define `NO_POLICY` as the digest of the empty set independent of catalogue version, so an unmanaged machine's header always reads `NO_POLICY`.

### C9. Medium — The recording claims overstate what exists, and saved bundles would carry the organisation's whole policy

**Location:** 3 (line 90), 6.21 item 1 of the "why" list (line 585), 6.21 item 2 (line 599), A13.

**Problem.** "The broker's own recordings (SH 6.17's UI recording, DST traces) record the adapter's snapshots as inputs" is not true of production: S1 3.4's production recording is per tab in a fresh renderer, and SH 6.17's UI recording records "every typed event the broker sends to the shell", not the broker's inputs from the adapter. The broker is unsandboxed and has no production recording, so "a crash on a managed machine replays under the same policy" holds for renderer and shell recordings (which receive their slices), not for the broker, where the digest only lets support match a policy. Separately, 6.21 item 2 puts **the effective snapshots** into every saved bundle that crosses a policy change, "because a replay that shows `about:policies` needs them". A per-tab renderer bundle needs only the `RendererPolicy` slices it already recorded; adding the whole effective policy exports internal hostnames, proxy, trust anchors and extension inventory into bundles users attach to bug reports, contradicting both the slicing principle (line 338) and the default masking of exports (line 623).

**Evidence.** S1 line 269 (3.4): "user-initiated per tab … recording starts at process launch, in a dedicated, fresh renderer". Shell line 565: "It records: the initial shell-state snapshot; every typed event the broker sends to the shell". Spec line 585: "The broker's own recordings (SH 6.17's UI recording, DST traces) record the adapter's snapshots as inputs"; line 599: "contains the effective snapshots (not the raw sources) … because a replay that shows `about:policies` needs them".

**Fix.** Say plainly that broker-side policy replay exists in DST only, and that production recordings replay policy through the slices their process received. Limit a bundle to the slices that process received (already in its stream); only a shell UI recording carries `ShellPolicy`, with `internal-hostnames` fields masked under SH 6.17's origin-only redaction default. Reword the §3 claim to "a recorded tab or shell replays under the policy it saw; a broker crash is matched by digest".

### C10. Medium — `UrlBlocklist` as a content-blocking trust domain is user-cancellable under CB's rules, and the "shared site-pattern syntax" does not exist

**Location:** 6.1 Types (line 208), 6.2 URL filtering and content blocking rows (lines 234–235), A18 (line 790).

**Problem.** CB 6.2 defines **Izcal filter syntax** (ABP-family rules with options and trust domains), not a URL-pattern grammar; there is no "site-pattern syntax" to share, so "one pattern means one thing" needs either a new pattern grammar or a mapping into filter rules. More importantly, CB's cross-domain rule lets **the user's own rules cancel blocks in every domain**, and CB 6.12's "Protection off here" turns off "every layer this spec adds" for a site. An enterprise `UrlBlocklist` placed as a CB trust domain is therefore cancellable by the person with one click unless CB is amended; A18 says only "distinct from user lists". The CB matcher also filters **requests** in the partition service; admins expect `UrlBlocklist` to block **navigations** (a top-level decision in `hg-navigation`, with an explanation page), and subresource blocking is a different policy. `ContentBlockingSiteExceptions` (proposed) could, as written, cancel Izcal's Security entries, which CB lets only a same-ID exception do.

**Evidence.** CB line 259: "**the user's own rules**, whose exceptions may cancel blocks in every domain, because the user is the principal"; line 260: "the request is blocked if any domain's own outcome is a block that no user rule cancels"; CB 6.12 item 4: "**Protection off here**: every layer this spec adds, for this site". Spec line 208: "one Izcal pattern grammar, shared with content blocking's site-pattern syntax".

**Fix.** Define one **URL-pattern grammar** in this spec (or `hg-url`), with a published mapping to CB filter syntax where lists are compiled. Enforce `UrlBlocklist`/`UrlAllowlist` as a **navigation decision** in `hg-navigation` (recorded, explained on the error page with the policy name), and optionally as a request filter in an **enterprise CB domain that user rules and per-site controls cannot cancel**; amend CB 6.2 rule 3 and CB-D19 to say so. Make `ContentBlockingSiteExceptions` unable to cancel Security entries (the same rule CB already applies).

### C11. Medium — The admin-only check is presented as closing ATK-7's route, but same-user malware can inject into the broker or adapter and replace per-user binaries

**Location:** 6.3 "Why check at read time" (line 270), EP-D4 (lines 272–277), 6.11 (line 463), 6.22 Residual (line 614).

**Problem.** The security scope's stance on ATK-7 is "raises cost and leaves evidence; it does not prevent theft": same-user malware "reads or injects into same-user process memory where the OS allows" and "kills and relaunches Izcal with any flags". The broker and the adapter run as the user. Malware can therefore change effective policy by injecting into either, or (under SEC-D14's per-user installs) by replacing the browser's binaries in the user-writable install directory, without writing any policy source. The spec's claims "removes the in-scope attacker's route at its source and costs nothing" and "Malware cannot add an organisation key without administrator rights" are true of the **source** route only. The EP-D4 change notice is drawn by the same broker the malware controls.

**Evidence.** Security scope line 253 (ATK-7): "Runs as the user; kills and relaunches Izcal with any flags; … reads or injects into same-user process memory where the OS allows"; line 258: "Against same-user code execution (ATK-7), Izcal raises cost and leaves evidence; it does not prevent theft." Spec line 270: "The admin-only check therefore removes the in-scope attacker's route at its source and costs nothing."

**Fix.** Restate EP-D3's benefit in SEC's terms: it closes the **durable, injection-free** route (a written key that survives reboots and looks legitimate), which is what the 2014–2022 abuse used, and leaves ATK-7 needing code injection per run, which SEC's other mitigations make costlier and more visible. Add the residual to 6.22 and to A12, note that per-machine installs (EP-D17) remove the binary-replacement route, and keep the "changed by your organisation" ledger entry in the storage service (not only the broker's memory) so a later session shows it.

### C12. Medium — The remote document can set every machine-only policy, so its signing key becomes the most powerful credential in the deployment, and its precedence above OS user policy is not argued

**Location:** 6.4 level 2 (line 283), 6.10 (lines 436–440), 6.11 (line 454), 6.12 trust anchors (line 474).

**Problem.** Because the remote document sits at "machine level", it may force-install extensions and apps, add `ExtensionOrganisationKeys` (code in every page without Izcal review), add `TrustAnchors` (silent interception of all TLS), set `KioskMode`, and, unless forbidden, set `RemotePolicyDocument` itself (re-pointing its own URL and keys). The key that signs it typically lives on a web-publishing workflow, not in the domain's administrative tier, and the document is network-delivered with a 37-day freeze window. Placing it **above OS user mandatory** is defensible (the trust root is machine policy), but the spec gives no reason, no option for machine policy to choose otherwise, and no conflict rule when OS user mandatory and remote mandatory disagree on an allow-list. The fetch also "follows the OS and enterprise proxy", which may itself come from the remote document (a bootstrap loop when the proxy setting is wrong).

**Evidence.** Line 283: "2. **remote document mandatory** (6.10; machine level, because its URL and keys come from machine policy)"; line 454: "(OS machine policy, or the remote document, whose keys come from machine policy)"; line 438: "The fetch follows the OS and enterprise proxy".

**Fix.** Add a catalogue field `remote: allowed | machine-opt-in | never`. Mark `RemotePolicyDocument`, `TrustAnchors`, `ExtensionOrganisationKeys` and `AuthDelegationAllowlist` `never` or `machine-opt-in` (allowed only if OS machine policy names the class in `RemotePolicyDocument.allowed_classes`). State the reason for level 2's rank, and fetch the document with the OS proxy only (never a proxy the document itself sets).

### C13. Medium — Kiosk mode treats the OS accessibility shortcut as an escape and can remove accessibility settings, with no rule that assistive technology keeps working

**Location:** 6.15 (lines 508–531), EP-14 (line 706), EP-D12.

**Problem.** The kiosk profile removes "the settings and diagnostics pages", lets the record prune keyboard shortcuts ("`keyboard`: the shortcuts that remain"), and lists "the OS accessibility shortcut" in the escape suite. Public kiosks are exactly where accessibility law and procurement apply, and SH 6.16 commits the shell to WCAG 2.2 AA (WCAG2ICT) with every action reachable by keyboard and shortcuts that never collide with screen-reader keys. Nothing says screen readers, the UI host's UIA/IA2 adapters, page zoom, forced colours, text size or the accessibility-model grant keep working in kiosk mode, or that pruning cannot remove F6 or focus-navigation keys. Treating the OS accessibility shortcut as an escape invites admins to disable it. The idle reset destroys the ephemeral container; whether a person's accessibility settings survive the session (they should within it, and the next user should get defaults) is unstated. Kiosk also needs **ephemeral containers**, which SH-D26 does not provide (four fixed containers in Stage 1, editing in Stage 2), and the text alternates between "profile" and "container".

**Evidence.** Spec line 515: "`keyboard`: the shortcuts that remain"; line 524: "the settings and diagnostics pages"; line 531: "… crash and restart, the OS accessibility shortcut, and new-window paths". Shell line 539: "The shell conforms to WCAG 2.2 AA as applied to software by WCAG2ICT (2024)"; shell line 543: "Every action has a keyboard path … No shortcut requires a chord that a screen reader reserves". Shell SH-D26 (line 804): containers "read-only in Stage 1 and editable in Stage 2".

**Fix.** Add a kiosk rule: accessibility paths are **not capabilities the kiosk removes**. AT adapters, zoom, forced colours, text size and a kiosk-safe accessibility panel (no file or protocol hand-offs) stay available; `keyboard` pruning cannot remove the SH 6.16 navigation set. Treat the OS accessibility shortcut as an OS hand-off whose **escape routes** (help links, file dialogs inside OS accessibility settings) are the OS kiosk configuration's to close, and document that. Add an EP-14 companion target: the kiosk suite passes the SH 6.16 AT walkthrough with NVDA, VoiceOver and Orca. Add the ephemeral-container need to the SH-D26 amendments, and use one term.

### C14. Medium — Managed bookmarks rely on a provenance value and a profile lock that the cited specs do not define

**Location:** 6.13 (line 482), 6.20 VDI (line 574), EP-D18 (line 756), A19 (line 791).

**Problem.** "Policy provenance (SH 6.7's provenance field)": SH 6.7's provenance is a **closed 13-value vocabulary** on visit and bookmark rows (`typed`, `link`, … `unknown`), with a schema test rejecting other values (SH-D9, SH-19); it has no `policy` value and no folder-level provenance. The schema is the **shell's** (Q-SG-6 closed 21 October 2026), so A19's target, "Storage, bookmark store", is the wrong owner. "Concurrent use of one profile directory is refused by the profile lock, as SG requires": the storage spec defines no profile lock; with FI-D17's one broker per user and EP-D18's per-session disks, who locks what, and how a second broker on another session host detects a held profile on a shared disk, is undesigned. EP-D18 then lists "a profile lock" as a decided option.

**Evidence.** Shell line 412: "The provenance value is one closed vocabulary, owned by this schema and quoted identically by SH-D9, SH-19 and the export schema: `typed`, `link`, `redirect`, `form`, `reload`, `back-forward`, `restore`, `extension`, `agent`, `bookmark`, `import`, `external`, `unknown`." Storage line 376: "Q-SG-6 … **Closed 21 October 2026**: the shell owns the history and bookmark schema". A search of `reports/Izcal storage.md` for "profile lock", "lock file" and "single instance" finds nothing.

**Fix.** Propose an SH 6.7/SH-D9 amendment: a 14th provenance value `policy` and a folder attribute `managed_by_policy`, both owned by the shell, with SH-19's test updated; retarget A19 to the shell. Design the profile lock here or hand it explicitly to FI-4/FI-10 (single-instance and profiles), covering the shared-disk VDI case, and drop "as SG requires".

### C15. Medium — One label family erases the devices spec's distinction between a locked value and a revocable policy grant

**Location:** 6.8 Lock display (lines 403–408), A9 (line 781), EP-C2.

**Problem.** The unified label "Set by your organisation" means **locked: cannot be changed** (SH 6.9). DV 6.18 shows "Set by your administrator" on a policy pre-grant that "can still be revoked by the person". Replacing the devices label with SH's mandatory label either makes pre-grants unrevocable (a change to DV's consent model, not a wording fix) or makes the label mean two things.

**Evidence.** Shell line 477: "Policy-set rows are visible, marked 'Set by your organisation', and cannot be changed". Devices line 710: "A grant created by policy shows 'Set by your administrator' and can still be revoked by the person." Spec line 781 (A9): "'Set by your administrator' → SH's label".

**Fix.** Add a fourth label for revocable policy grants ("Allowed by your organisation", revocable), or keep "Default from your organisation" semantics for them, and state in A9 that DV's revocability is unchanged.

### C16. Medium — The staffing is called a carve-out from the shell track but is sized as an addition, and S1 8.3 is not amended; EP-M0 adds measurement work to M1's exit

**Location:** 9.2 (lines 728–731), 9.1 EP-M0 (line 719), 9.5.

**Problem.** S1 8.3's shell row is 6–8† engineers for **thirteen** sub-tracks (168–243 engineer-months†) and already names "policy". Stage 1 at 1.5–2† engineers from EP-M0 (M1) to EP-M3 (M7) is roughly 70–100 engineer-months†, 30–60% of the whole shell programme; it cannot be a carve-out, and if additive, S1 8.3's total needs an amendment row, which 9.5 lacks. Stage 2's 4–6† plus a writer is likewise unledgered. EP-M0 makes "the measurement list in 11.2 has results" an **M1-exit** criterion (HKCU ACLs, GP notification behaviour, sandbox reads on three OSes), but the accepted M1-exit direction is the header field, `NO_POLICY` and the reserved shape; tying M1's exit to Stage 2 delivery measurements adds a gate nobody accepted.

**Evidence.** S1 line 44: "Shell | UI, profile, policy … thirteen non-overlapping sub-tracks, 168–243 engineer-months† … 6–8†". Spec line 728: "This spec carves out an **enterprise policy sub-track**"; line 719: exit criterion "M1 recordings carry `NO_POLICY`; the measurement list in 11.2 has results".

**Fix.** State the sub-track as **additive** with an S1 8.3 amendment row (Stage 1 and Stage 2 figures separate), or show which shell sub-tracks shrink. Split EP-M0: the M1-exit part is the header field, `NO_POLICY` and the checked reserved shape (C1); the black-box measurements move to EP-M1 at M5, before the adapter is built.

### C17. Medium — `ExtensionManagedStorage` validation needs per-profile extension data in an adapter that holds none

**Location:** 6.11 (line 465), 6.6 (line 321), 6.22 Assets (line 610).

**Problem.** The adapter validates each extension's managed value "against the extension's declared `storage.managed_schema` (shipped to the adapter as data with the package)". The adapter "holds no profile data", is per user (C7) while extensions are installed per profile, and someone must extract the schema from the package; if the broker reads it from the manifest, the broker parses JSON (IPC-R15). JSON Schema evaluation is also a sizeable evaluator to add to the adapter's parser surface, not listed in `hg-policy-format`.

**Evidence.** Line 465 (quoted above); line 321: "It holds no profile data, no network access and no page input." EX line 198: "`storage.managed_schema` reads enterprise policy".

**Fix.** Validate the value **at install time in the extension package utility** (EX's `hg-ext-package-main`, strictest tier, which already parses the package) against the schema there, and have the adapter check only the catalogue's generic structure; or send the adapter a pre-compiled, typed schema produced by the package utility. Name the evaluator and its fuzz target either way.

### C18. Medium — The amendment table misses rows the design depends on

**Location:** 9.5 (lines 769–794).

**Problem.** Beyond the rows other findings name, the design changes accepted text in places 9.5 does not list: SEC 6.3a's **OS adapter** row ("Resolver fallback, SSPI/GSSAPI and client-key signing only") must gain the policy configuration and its threat model file; **EX G171** and EX 6.6's "configuration reader" wording (C5, C17); **FP 8.2's OS-adapter row** (C3); **NW-D17 and NW-36** (C4); **IPC-R16** entries (C4); **SH 6.7, SH-D9, SH-19** (C14); **SH-D26** ephemeral containers (C13); **CB 6.2 rule 3 and CB-D19** (C10); **S1 8.3** (C16); **coding standard 6.2's tier table** with the named crates; and the **crash handler** (S1 §1, D-S4) as a policy consumer.

**Evidence.** Security scope line 363: "OS adapter (one per OS source) | … Resolver fallback, SSPI/GSSAPI and client-key signing only". The 9.5 table has no row for security scope 6.3a, FP 8.2's OS-adapter row, NW-D17, IPC-R16, SH 6.7, SH-D26, CB 6.2 or S1 8.3.

**Fix.** Add the rows, each with the exact sentence to change.

---

## Low

### C19. Low — "Break for protections" is broader than the catalogue

**Location:** 4.4 last row (line 132), 6.1 (line 210), 6.2.

**Problem.** 4.4 says policy cannot switch on "AI, Izcal-bound telemetry and protections", but the catalogue lets policy weaken several protections without `restrict-only`: `HttpsFirstMode`, `TrackingProtectionLevel`, `GlobalPrivacyControl`, `EncryptedClientHelloAllowed`, `QuicAllowed`, `DnsOverHttpsMode`, LNA defaults, `ContentBlockingSiteExceptions`. Most are legitimate enterprise needs (NW 6.14 lists several), so the rule is right and the sentence is wrong.

**Evidence.** Line 132: "**Break** for AI, Izcal-bound telemetry and protections | B10, D-S4 and PH 6.13 allow only the protective direction".

**Fix.** Say "for AI, Izcal-bound telemetry and phishing protection", and list the protections policy may weaken, each shown on `about:policies` and in the site panel as "weakened by your organisation". State that D-S1 and partitioning have no catalogue entry (CB 6.12 already says they are never switched off).

### C20. Low — The catalogue fails its own EP-1 in small ways

**Location:** 6.1 Types (line 208), 6.2, 6.14 (line 490), 6.6 (line 381).

**Problem.** `DeveloperToolsBlockedUrls` is used in 6.14 but is not in the catalogue. `ProfileDirectory` is a "path", and the Types list has no path type (and IPC-R3 wants paths as dedicated validated types). `KioskMode`, `SiteSwitchingBrowser` and `EnterpriseReporting` are "record" without a field schema.

**Evidence.** Line 490: "pages matching `DeveloperToolsBlockedUrls`"; line 249: "`ProfileDirectory` … path".

**Fix.** Add the entry, a bounded path type with OS-specific validation (no `..`, absolute, no device paths), and the record schemas.

### C21. Low — The Linux and Windows admin-only checks need to be made on the opened object

**Location:** 6.3 (lines 255–257), G239.

**Problem.** "Each file and every parent directory must be owned by root" checked by path leaves a check-then-open race and symlinks (a root-owned directory containing a symlink to a user-writable file, or a parent swapped between check and read). Windows registry symbolic links (`REG_LINK`) raise the same question.

**Evidence.** Line 257: "Each file and every parent directory must be owned by root and not group- or world-writable".

**Fix.** Open with `O_NOFOLLOW` relative to an already-checked directory descriptor, check ownership and mode with `fstat` on the opened descriptor, and refuse symlinks; on Windows open keys with `REG_OPTION_OPEN_LINK` refused and check the security descriptor of the opened handle. Add the cases to EP-6.

### C22. Low — DLP's preferred route inverts FI-D14's order

**Location:** 6.18 (line 554), EP-D15.

**Problem.** FI-D14: open protocols first, Izcal's own implementation, a proprietary protocol "only where no open route works". 6.18 prefers a black-box reimplementation of Google's SDK-defined protocol, with an Izcal-designed open protocol as the fallback. Interoperability with existing agents may justify the order, but the spec does not make the "no open route works" argument, and a black-box reimplementation of a vendor protocol also needs a licence and legal check of the SDK's terms before anyone observes it.

**Evidence.** Line 554: "Izcal therefore either writes a **black-box specification** by observing a real agent at the pipe boundary (M-1 part 2), or adopts a vendor's prose specification if one is published. A protocol Izcal designs itself is the fallback, published openly."

**Fix.** State the FI-D14 argument (no agent vendor implements any open protocol, so an open one does not "work" for buyers), add the legal review to Q-EP-6, and keep the Izcal-published protocol on the table as the route offered to agent vendors in parallel.

### C23. Low — Several decisions have no target

**Location:** 8 (lines 689–711).

**Problem.** No target checks the EP-D4 high-impact notice (shown once, lists every high-impact change, ledger entry written), site switching (EP-D13: rule matching, the v.2 list parse, the fixed argument vector), the organisation trust tier (EP-D9: an org-signed package never enters the directory tier), the slice enforcement map (C5), adapter crash and restart (snapshots re-sent, stale marking), OS SSO (EP-D16: only allowlisted hosts receive device credentials), or the DLP hooks' zero cost while no-op.

**Evidence.** Section 8 lists EP-1 to EP-19; none names EP-D4's notice, EP-D9, EP-D13, EP-D16 or the hooks.

**Fix.** Add a target per decision, each with a release-blocker or fix action.

### C24. Low — The shared header's owning crate is unnamed and does not exist yet

**Location:** 6.21 item 1 (line 596), A5.

**Problem.** "The header crate stores 32 opaque bytes and depends on no crypto crate", but no crate is named and M1 as merged has no recording or crash-report header type (a search of `crates/` for a header struct, build ID or data-versions field finds none). The audit assigns the header jointly with FI-11.

**Evidence.** Line 596 (quoted). `crates/` contains `hg-dst`, `hg-ipc`, `hg-ipc-codegen`, `hg-ipc-gen`, `hg-substrate`, `hg-substrate-os`, `hg-substrate-sim` and `hg-types`, none with a recording header.

**Fix.** Name the crate (for example a `RecordingHeader` in `hg-types` or `hg-substrate`, both L1/L2 and crypto-free) and its owner (FI-11), and state the field as `policy: [u8; 32]`, which `.hgidl` would carry as a struct of fixed fields or `bytes<32>` with an exact-length check, because the IDL has no fixed-array type.
